# Controlled native adapter and repeatable experiments

Implementation date: 2026-09-27. Source files are under
`crates/synesthesia-inference`; the main session owns native execution and its
results. Linux Rust checks are not model/device evidence.

## Fixed source input

| Input | Commit | Download archive SHA256 |
| --- | --- | --- |
| CrispASR | d08ec2dd83411a8389745c97c4f2e3955e084280 | 15d21060ffd95bfe3333f2e60775b6114d2659294a5b65138a1956af019fc298 |
| CrispStrobe ggml submodule | 2f5a80d258c46e6ac8eee95f1328c0f58376d7ee | 8e06f263fffea53b169c7839b3b4456451e0b0a3d24d2732a63daa0d5a42810e |
| llama.cpp | 7fe450e19305b828c199d602c23a8337aaa1f03b | a6861d549427f814dc591c439e08206f67ffaba0248344d421589abf18199e67 |

Archives came from GitHub codeload `OWNER/REPO/tar.gz/COMMIT`, not latest.
Extract ggml archive contents into the CrispASR tree's `ggml/`; llama's archive
already includes its ggml. No source/model/build output is vendored in the repo.
The Qwen prompt/embedding pipeline derives from the pinned MIT implementation;
the full required notice is in native/NOTICE. C7 must include actual linked
upstream/dependency notices with binaries; this is not a completed release license
inventory. Model identity/template provenance remains in model-metadata.md.

## Native build

Use a task-owned short build root on Windows. The first long Temp build path
produced a 263-character MSVC shader-generator object path and C1041 before
model execution; do not change global long-path registry policy to mask it.
The main session uses an isolated short `sb-<id>` directory for builds, installed
VS2026 vcvars64, its bundled CMake/Ninja, and private portable Vulkan SDK inputs.
See portable-vulkan-sdk.md for original SDK identity and extraction evidence.

Configure each engine independently:

```text
cmake -S <repo-copy>/crates/synesthesia-inference/native -B <short>/asr -G Ninja -DSYNESTHESIA_ENGINE=asr -DSYNESTHESIA_SOURCE=<verified-CrispASR-tree> -DGGML_VULKAN=ON -DCMAKE_BUILD_TYPE=Release
cmake --build <short>/asr --target synesthesia-asr-helper --parallel 8
cmake -S <repo-copy>/crates/synesthesia-inference/native -B <short>/translation -G Ninja -DSYNESTHESIA_ENGINE=translation -DSYNESTHESIA_SOURCE=<verified-llama-tree> -DGGML_VULKAN=ON -DCMAKE_BUILD_TYPE=Release
cmake --build <short>/translation --target synesthesia-translation-helper --parallel 8
```

Set compiler/SDK discovery only in that child environment. For generic CPU
helpers configure separate build directories with GGML_VULKAN=OFF; the Vulkan
flavour links the system loader and is not a CPU distribution for hosts without
that loader. All optional x64 ISA flags are forced OFF in both builds.

CMake applies guarded source patches to these verified temporary trees and
refuses unknown source text. It fixes upstream top-level path assumptions;
replaces Qwen/audio automatic device selection and CPU weight spill; adds actual
Vulkan numeric identity and Hy block-matrix buffer validation; routes ASR graph
calls through cancellation guards; clears KV and replaces session schedulers;
disables activation/dump persistence. Do not apply these patches to an unrelated
checkout. Build helpers separately so differing ggml versions cannot collide.

## Product interface and identity

`NativeInference::new` requires absolute resolved model paths from RuntimeRoot,
explicit device config, native helper absolute paths and source language.
`detect` probes each real selected backend; `load` verifies both model hashes and
waits for both real load acknowledgements. `start_input` requires a fresh shared
InputId and empty prior input. `accept_audio` checks AudioOrder and preserves
sample coordinates. `next_event` returns immutable shared Source/Translation
events. `finish_utterance` requires the current drained emitted cursor; no PCM is
created. `cancel` waits for actual worker stop/clear, retains models; `shutdown`
waits for owned helper exit, with owned-process termination on failures/Drop.

Normal translation uses the embedded verified Hy Jinja template with a single
user message requesting Simplified Chinese. No previous prompt/response history
is reused. Text/identities remain validated by the shared domain reducer.

Stop is cooperative, not a claim of hardware-preemptive Vulkan execution.
Already submitted graphs may finish; cancelled ASR cannot submit another graph
or decode token. Only after the worker has joined are context buffers cleared and
old ASR/audio schedulers or llama_context destroyed/recreated. The owning model,
model buffers and process survive. Failed cancellation destroys the owned helper
and reports failure, never reloads silently or acknowledges normal residency.

## Explicit bounds

- Shared ingress: 100 frames/32000 samples, including one held whole frame.
- ASR current window: 160000 samples; one submitted immutable snapshot has the
  same cap. Submission has one transient PCM serialization buffer of 320001 bytes.
- Helper input: one 320001-byte frame (ASR language + PCM), one worker; ASR float
  conversion has 160000 samples. Encoder output additionally validates at most
  1024 frames x 4096 floats; native graph metadata is fixed at 16384 nodes.
- ASR KV: 2048 tokens; prompt <=1792, output <=256 tokens/8192 UTF-8 bytes.
- Hy KV: 2048 tokens, prompt <=1536, batch <=256, output <=512 tokens/8192 bytes.
- Translation jobs: four queued plus active; timeout clear acknowledgement is
  separate from next prompt admission. Original publication continues.
- Publication: 64 shared typed events. Stdout reader: eight replies with <=8200
  bytes each. Header is 20 bytes; oversized frames reject before allocation.
- Real load wait: 180 seconds. Calculation/cancellation wait: 60 seconds; these
  are fail-safe technical deadlines, not approved latency gates.
- No separate content/diagnostic files. stderr is discarded by the OS. Safe
  numeric metrics are exposed for the app's single DiagnosticLog sink.

Private IPC header: LE u32 magic `0x31494e53` (SNI1), u32 kind, u64 strictly
increasing job, u32 payload length. Request kind1 is work, kind2 cancel, kind3
shutdown, kind4 observes numeric work state. Reply10 is loaded/probed
backend/vendor/device (three u32), reply11 is
u64 cumulative graph calls then UTF-8 text, reply12 acknowledges joined clear
with u64 graph count, reply13 is a safe failed category without foreign text.
Reply14 has 24 bytes: stage u32, inside-graph flag u32, cumulative graphs u64,
active job u64. ASR stages are idle0/mel1/encoder2/prefill3/decode4. Observation
does not emit content or change inference. The status caller preserves bounded
earlier work completions for normal publication; helper and Rust versions must
match this private contract.
This is private owned-process IPC, not the product WS protocol.

## Runnable experiments

Build Rust with the standard native pinned Cargo harness, then:

```text
cargo run -p synesthesia-inference --example native_smoke --locked --offline -- CONFIG RUNTIME_ROOT ASR_HELPER TRANSLATION_HELPER EN_PCM JA_PCM
cargo run -p synesthesia-inference --example native_benchmark --locked --offline -- CONFIG RUNTIME_ROOT ASR_HELPER TRANSLATION_HELPER PCM MANIFEST
```

CONFIG is schema1/full, token-free, with absolute external models and explicit
CPU or the actual unique Vulkan triple. All helper/runtime/fixture arguments
are separate argv fields. Native smoke processes up to15 seconds of each fixed
language fixture, tests cancellation/idle graph stability/same PIDs/one model
load, resumes new sessions and shuts down. Each of three cancellation trials
first admits six seconds of actual English PCM and waits for encoder, prefill
or decode respectively, with inside-graph=true and a nonzero active job. A trial
that never observes its requested graph stage fails; buffered audio or a queued
job is insufficient. After cancellation, fresh status must report idle/no graph/
no job, graph counters must remain stable in two idle residency probes, and the
original PIDs and model-load count must remain unchanged. It reports only numeric
evidence. Already launched GPU graphs may finish before cancellation acknowledgement;
this checks cooperative cancellation and does not claim GPU hardware preemption.

Model-free helper probes can also reject an unmatched vendor/device/name triple
before readiness, verify the actual requested Vulkan triple, or initialize the
explicit CPU backend. A CPU probe does not establish CPU model performance or
compatibility on every generic Windows10/11 x64 processor. These small probes do
not reload the models resident in the separately owned experiment processes.

Benchmark requires the frozen 600-second PCM and manifest, checks file length/
SHA256, paces 3000 actual200ms frames, drains the real pipeline, and scores all
reference content in bounded memory. English normalization lowercases and keeps
alphanumeric word tokens; Japanese scores alphanumeric Unicode characters.
Levenshtein edit count includes missing content against the entire reference.
It reports observed draft/final/translation counts, failures/pending deadline,
maximum queue size sampled during drains and segment latency percentiles. The
queue statistic is an observed sample maximum, not an internal atomic high-water
counter. Segment sample boundaries derive
from the current energy policy, not manually verified phonetic clip boundaries;
these percentiles cannot be accepted as precise sentence-start/end D-10 gates.
Generated source/translation/transcript/prompt are never printed or persisted.
Human Chinese translation review and precise speech annotations remain required.

## Validation so far

Linux focused fmt/check/clippy(-D warnings)/test/build examples passed on Rust
1.90.0 through hako, offline locked. Eight behavior tests passed: oversized/
malformed IPC, leading silence, no-packet finish without padding, monotone end
with real silence, bounded long utterance, whole variable-frame handoff, and
reference scoring including missing content. Native build/model execution is
reported separately in [native-experiments.md](native-experiments.md) by the main
session. Its preliminary v3 15-second English/Japanese run used real RX9070XT
models, but ASR cancellation had zero graph calls and therefore did not establish
ASR cancellation during computation. The revised stage observations, stronger
scratch-context destruction and baseline CPU flags require a new paired native/
Rust build; preliminary output cannot substitute for that verification.
Windows10, exact quality/performance gates, full/private
benchmark translation review, 60-minute product chains and release closure remain
open until real evidence and D-10 user review.
