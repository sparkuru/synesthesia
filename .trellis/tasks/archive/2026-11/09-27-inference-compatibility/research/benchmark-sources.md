# Controlled benchmark source preparation

This is source research for the approved D-02c English/Japanese ten-minute benchmarks, not a downloaded fixture or measured result. No audio or transcript has been downloaded or committed by this investigation.

## English candidate

[OpenSLR's LibriSpeech release](https://www.openslr.org/12/) provides 16 kHz read English speech with aligned transcripts and identifies CC BY 4.0 licensing. Its `test-clean` archive is approximately 346 MB. Select one speaker and a deterministic ordered list of utterances, with enough varied material for a 600-second fixture; do not combine speakers under a single-speaker label.

The original release provides archive MD5 values. After acquisition, additionally record archive and chosen file SHA256, speaker/chapter/utterance IDs, exact lengths, resampling/concatenation operations, inserted silence, final PCM hash, and attribution. An archive checksum alone does not identify the derived benchmark.

## Japanese candidate

[The JSUT publisher](https://sites.google.com/site/shinnosuketakamichi/publication/jsut) describes a native Japanese female speaker recorded at 48 kHz in an anechoic room, with transcription and varied reading material. Version 1.1 is approximately 2.7 GB; `basic5000` offers varied sentences, while `travel1000` is domain-specific.

The publisher permits academic/noncommercial research and personal use of its audio; redistribution is restricted and commercial use has separate terms. Text licenses vary by subcorpus. Use a task-owned external research fixture with its original license and attribution, never add corpus audio to application releases or assume all JSUT assets share a CC audio license.

## Fixture acceptance still required

- Acquire only the minimum suitable source assets through an identified source and verify license/identity before retaining them.
- Freeze a single-speaker selection per language with short/long utterances and explicit silence, exactly 600 seconds, without repeated padding speech or silently clipped reference sentences.
- Keep references and controlled fixtures separate from application session data. The application's D-09 prohibition on recording audio/subtitle content remains in force.
- Record speech onset/end annotations, final hashes, preparation version, and reference normalization rules before latency or text scoring. Objective ASR references do not supply approved Chinese translation-quality criteria.
- Report edited/concatenated read-speech scope honestly. Multi-speaker/background-music material is supplementary; D-10 numerical release thresholds remain subject to review after the real C2 baseline.

No source has yet been selected as the final benchmark, no boundary annotations exist yet, and no inference or quality result is claimed here.
