# -*- coding: utf-8 -*-
"""Prepare fixed private research fixtures, never application session recordings."""

import argparse
import concurrent.futures
import hashlib
import json
import struct
import subprocess
import sys
import urllib.request
import wave
import zlib
from pathlib import Path


class CLIStyle:
    """CLI tool unified style config."""

    COLORS = {"TITLE": 7, "SUB_TITLE": 2, "CONTENT": 3, "ERROR": 2}

    @staticmethod
    def color(text: str = "", color: int = 3) -> str:
        """Color status and help text."""
        return f"\033[1;{30 + color}m{text}\033[0m"


class ColoredArgumentParser(argparse.ArgumentParser):
    """Preserve semantic help coloring for the research CLI."""

    def format_help(self) -> str:
        """Format readable colored help."""
        return CLIStyle.color(super().format_help(), CLIStyle.COLORS["TITLE"])


def fetch_entry(entry: dict, destination: Path) -> Path:
    """Fetch one ZIP entry, checking range, structure, size and CRC."""
    output = destination / Path(entry["name"]).name
    if output.exists():
        data = output.read_bytes()
        if len(data) == entry["size"] and zlib.crc32(data) == entry["crc"]:
            return output
        raise ValueError("Existing fixture does not match its ZIP index")
    start = entry["offset"]
    end = start + entry["compressed"] + 30 + 1024 - 1
    request = urllib.request.Request(
        "https://ss-takashi.sakura.ne.jp/corpus/jsut_ver1.1.zip",
        headers={"Range": f"bytes={start}-{end}"},
    )
    with urllib.request.urlopen(request, timeout=30) as response:
        if response.status != 206 or response.headers["Content-Range"] != f"bytes {start}-{end}/2688749844":
            raise ValueError("Publisher did not return the exact requested range")
        payload = response.read(end - start + 2)
    if len(payload) != end - start + 1 or payload[:4] != b"PK\x03\x04":
        raise ValueError("Invalid ZIP local entry")
    fields = struct.unpack_from("<4s5H3I2H", payload)
    name_length, extra_length = fields[-2:]
    name = payload[30:30 + name_length].decode("utf-8")
    if name != entry["name"] or fields[3] != entry["method"]:
        raise ValueError("ZIP index/local header mismatch")
    offset = 30 + name_length + extra_length
    compressed = payload[offset:offset + entry["compressed"]]
    if entry["method"] != 8:
        raise ValueError("Unsupported ZIP compression method")
    data = zlib.decompress(compressed, -15)
    if len(data) != entry["size"] or zlib.crc32(data) != entry["crc"]:
        raise ValueError("ZIP size/CRC mismatch")
    output.write_bytes(data)
    return output


def decode_pcm(path: Path) -> bytes:
    """Decode/resample a controlled source with fixed FFmpeg settings."""
    process = subprocess.run(
        ["ffmpeg", "-v", "error", "-i", str(path), "-map", "0:a:0", "-ac", "1",
         "-af", "aresample=16000:resampler=soxr:precision=28", "-ar", "16000",
         "-f", "s16le", "-"], capture_output=True, check=True,
    )
    if not process.stdout or len(process.stdout) % 2:
        raise ValueError("Invalid decoded fixture")
    return process.stdout


def write_fixture(language: str, sources: list[Path], references: dict[str, str], root: Path) -> dict:
    """Concatenate intact utterances plus explicit silence to exactly 600 seconds."""
    target_samples = 600 * 16000
    cursor = 16000
    chunks = [b"\0" * (cursor * 2)]
    annotations = []
    for source in sources:
        pcm = decode_pcm(source)
        samples = len(pcm) // 2
        if cursor + samples + 16000 > target_samples:
            break
        key = source.stem
        annotations.append({
            "id": key, "clip_start_sample": cursor, "clip_end_sample": cursor + samples,
            "source_sha256": hashlib.sha256(source.read_bytes()).hexdigest(),
            "reference": references[key],
        })
        chunks.extend([pcm, b"\0" * 32000])
        cursor += samples + 16000
    chunks.append(b"\0" * ((target_samples - cursor) * 2))
    output = b"".join(chunks)
    if len(output) != target_samples * 2 or not annotations:
        raise ValueError("Fixture does not have the exact expected duration")
    pcm_path = root / f"{language}-600s.pcm"
    pcm_path.write_bytes(output)
    with wave.open(str(root / f"{language}-600s.wav"), "wb") as wav:
        wav.setnchannels(1)
        wav.setsampwidth(2)
        wav.setframerate(16000)
        wav.writeframes(output)
    speech_clip_samples = sum(row["clip_end_sample"] - row["clip_start_sample"] for row in annotations)
    manifest = {
        "version": 1, "language": language, "duration_samples": target_samples,
        "sample_rate": 16000, "channels": 1, "format": "pcm_s16le",
        "pcm_sha256": hashlib.sha256(output).hexdigest(),
        "utterance_count": len(annotations), "source_clip_samples": speech_clip_samples,
        "inserted_silence_samples": target_samples - speech_clip_samples,
        "boundary_scope": "Corpus clip boundaries; precise speech onsets/ends require separate annotation",
        "annotations": annotations,
    }
    (root / f"{language}-manifest.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
    return manifest


def prepare(root: Path) -> None:
    """Use previously verified publisher indexes and private source assets."""
    rows = json.loads((root / "jsut-entries.json").read_text())
    destination = root / "jsut-selected"
    destination.mkdir(exist_ok=True)
    audio = sorted((row for row in rows if row["name"].endswith(".wav")), key=lambda row: row["name"])
    longest = sorted(audio, key=lambda row: row["size"], reverse=True)[:3]
    selection = []
    estimated_seconds = 0.0
    for entry in audio:
        selection.append(entry)
        estimated_seconds += (entry["size"] - 44) / 96000
        if estimated_seconds >= 490:
            break
    selection.extend(row for row in longest if row not in selection)
    selection.extend(row for row in rows if not row["name"].endswith(".wav"))
    with concurrent.futures.ThreadPoolExecutor(max_workers=6) as pool:
        futures = [pool.submit(fetch_entry, entry, destination) for entry in selection]
        for future in concurrent.futures.as_completed(futures):
            future.result()
    references_ja = dict(line.split(":", 1) for line in (destination / "transcript_utf8.txt").read_text().splitlines())
    references_en = {}
    source_en = root / "LibriSpeech/test-clean/6930"
    for transcript in source_en.rglob("*.trans.txt"):
        references_en.update(dict(line.split(" ", 1) for line in transcript.read_text().splitlines()))
    manifests = [
        write_fixture("en", sorted(source_en.rglob("*.flac")), references_en, root),
        write_fixture("ja", [destination / Path(row["name"]).name for row in longest]
                      + [path for path in sorted(destination.glob("*.wav"))
                         if path.name not in {Path(row["name"]).name for row in longest}],
                      references_ja, root),
    ]
    summary = [{key: value for key, value in manifest.items() if key != "annotations"} for manifest in manifests]
    print(CLIStyle.color(json.dumps(summary, indent=2), CLIStyle.COLORS["CONTENT"]))


def main() -> int:
    """Run the private fixture preparation CLI."""
    parser = ColoredArgumentParser(
        description="Prepare private single-speaker benchmark fixtures from verified assets.",
        epilog="Example: prepare-benchmark.py --root /tmp/synesthesia-benchmark-20260927\n"
               "Inputs/indexes must be acquired separately. Do not redistribute JSUT audio.",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument("--root", type=Path, required=True, help=CLIStyle.color("Private verified asset directory"))
    parser.add_argument("--log", action="store_true", help=CLIStyle.color("Print error details for controlled research inputs"))
    arguments = parser.parse_args()
    try:
        prepare(arguments.root)
    except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
        detail = str(error) if arguments.log else type(error).__name__
        print(CLIStyle.color(f"Fixture preparation failed: {detail}", CLIStyle.COLORS["ERROR"]), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
