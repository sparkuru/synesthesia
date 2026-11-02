# Frozen controlled benchmark inputs

Date: 2026-09-27, prepared by the main session. No inference/quality result yet.
Private assets: `/tmp/synesthesia-benchmark-20260927`, outside the application
and repository release payload. Preparation script: [prepare-benchmark.py](prepare-benchmark.py).
The script hash at preparation was
`23b6de31c53be2ef26a551bf635a6bd17577be0d957df9afb06c8bec96ca53fe`.
FFmpeg 7.1.5-0+deb13u1 decodes to mono PCM16 and uses libsoxr precision 28 for
16 kHz resampling. No utterance is repeated or clipped to fill the target.

## Source identity and retention

English: [OpenSLR LibriSpeech](https://www.openslr.org/12/), test-clean,
CC BY 4.0, authors Vassil Panayotov, Daniel Povey, Guoguo Chen and Sanjeev
Khudanpur; speech derived from LibriVox. Complete 346663984-byte archive matches
the publisher MD5 `32fa31d27d2e1cad72775fee3f4849a9`; additional SHA256
`39fde525e59672dc6d1551919b1478f724438a95aa55f874b576be21967e6c23`.
Select all 78 lexicographically ordered utterances from speaker 6930 only,
including chapters 75918/76324/81414. LICENSE/README and original transcripts
are retained with the private corpus inputs.

Japanese: [JSUT 1.1 publisher](https://sites.google.com/site/shinnosuketakamichi/publication/jsut),
native female single-speaker 48 kHz reading in an anechoic room. Used privately
under personal/noncommercial research terms; audio is not redistributed.
basic5000 text licenses in the actual LICENCE.txt include CC-BY-SA 3.0,
CC-BY 2.0 and original CC-BY-SA 4.0 sentences. Retain that license with references.
The publisher ZIP is 2688749844 bytes, not fully downloaded. Exact HTTP 206
ranges retrieved EOCD/central directory (7737 entries, directory offset
2687848979, size900843). The directory-plus-EOCD bytes SHA256 is
`d905a0d06121ea90d82f5643ff2086a0cd545e942190dda03b603e86bcd3260f`.
Every selected entry is checked against index/local-header name/method, exact
compressed range, uncompressed size and CRC before use; local file SHA256 is
recorded per annotation. This is not an assertion of a full ZIP SHA256.

The first range acquisition timed out; repeating reused only already verified
files and filled the same selection. The initial fixture's longest Japanese
clip was9.92 seconds. Before any model measurement, preparation added the three
longest basic5000 clips (4497/0617/4195, 16.37/15.99/15.71 seconds), placed first,
then lexicographic low-ID clips without duplicate IDs. The final 600-second
selection below supersedes that unmeasured draft; no failed inference sample
was replaced.

## Frozen input hashes and scope

| Language | PCM SHA256 | Full manifest SHA256 |
| --- | --- | --- |
| en | 07130f47d8cb4274cb66aaf84e05f20d040e89d46b2ff9d615b2f63443793863 | 47df947407579160f917f42613ad0768affc914d7a523b6814f1ea511c993c6d |
| ja | 830f6c26062ff7890a957b798cfbed03fd8be7d474d7aee021770e243e04f0e7 | 6b1a28e9bb32329a55575d4e41e0fd9d641f093ce5b842867b452bf7941d4d24 |

Both inputs contain exactly9600000 samples/600 seconds, mono16kHz PCM16. English
has78 utterances, clip durations1.815..23.315 seconds, 480.0500625 seconds of
original clips (including natural within-clip silence), plus119.9499375 seconds
of explicitly inserted silence. Japanese has112 utterances, clip durations
2.39..16.37 seconds, 483.568625 seconds of original clips plus116.431375 seconds
inserted silence. There is initial1-second silence, 1second between clips, and
a final zero tail. These are concatenated read-speech benchmarks, not ten
minutes of uninterrupted speech or natural conversation.

Manifests include each immutable ID, original file SHA, corpus reference and
exact clip start/end sample. Corpus clip boundaries are not verified phonetic
speech-onset/end annotations. Precise latency scoring still needs separate
waveform annotation/inspection; reference text does not establish Chinese
translation quality. Short smoke and cancellation experiments must identify
their own slices rather than claim they tested the whole benchmark. D-10
threshold approval and supplementary noisy/dialogue cases remain separate.
