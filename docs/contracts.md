# Foundation contracts (v1)

> Retained compatibility reference. Current product: [PRD v2](../prd.md).
> Desktop schema 2: [pipeline configuration](../.trellis/spec/backend/pipeline.md).
> The v1 role, mandatory translation and stop/cancel semantics below belong to
> legacy code; they are not implicitly adopted by the new pipeline. Consult
> [current status](status.md) rather than historical progress statements.

C1 implements platform-free configuration, state, budgets and codecs. The three
executables provide help, version and configuration validation only. No capture,
GUI, inference, WebSocket listener or model loading is integrated. Passing these
tests is not MVP acceptance or evidence of Windows/GPU compatibility.

## Workspace and build

Rust is pinned to 1.90.0, edition 2024, with rustfmt and clippy. Cargo.lock pins
resolved dependencies. All workspace crates forbid unsafe code. `core` owns
configuration and domain state; `protocol` depends on it. Each product entry
currently depends only on core. There is no inference/model dependency in client,
and no desktop dependency in server. Later adapters must preserve this direction.

Native Windows 10/11 x64 uses `x86_64-pc-windows-msvc` and the native Rust/MSVC
build environment. Run these commands without Docker or WSL:

```text
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --workspace --locked
```

On this Linux development host only, prefix commands with `./hako`. The wrapper
uses the existing `rust:1.90-bookworm` image, host UID/GID, an ephemeral container,
repo-scoped `.devhome` caches and no ports. First bootstrap the pinned cached
components with:

```text
./hako rustup toolchain install 1.90.0 --profile minimal --component rustfmt --component clippy --no-self-update
```

`HAKO_NETWORK=none` works for checks once dependencies are cached. There is no
`dev.sh` because C1 exposes no development service. This wrapper is neither a
Windows prerequisite nor the optional Linux CPU server deployment image.

## Command entries and runtime root

The executables are `synesthesia-full`, `synesthesia-server`, and
`synesthesia-client` (with `.exe` on Windows). They accept `--help`, `-h`,
`--version`, and `--check-config FILE [--runtime-root ABSOLUTE_DIRECTORY]`.
No arguments show help. Successful syntax/config validation returns 0;
invalid arguments or configuration return 2. Validation explicitly reports that
models and runtime readiness remain unverified; unsupported run commands fail.

The default runtime root is the executable's parent directory, matching the
planned portable layout. An explicit runtime root must be absolute. Relative
config and model paths resolve from that root, never the caller's CWD.
`RuntimeRoot::load` validates and returns compute model paths already resolved to
absolute paths. `Config::parse` is structural validation only; adapters must use
`RuntimeRoot::load` rather than interpreting raw relative paths themselves.
Absolute external model paths are supported, including Windows drive/UNC paths
on Windows. Relative parent traversal and Windows drive-relative paths are
rejected. Model existence, hashes, formats and loading are C2 checks. No model
location is inferred or downloaded; no remote machine path is a default.

## Configuration schema 1

Use the single `Config::parse`/`RuntimeRoot::load` boundary. It rejects documents
larger than 65,536 bytes, malformed/trailing JSON, duplicate/unknown fields,
wrong roles and unsupported versions. Nested objects are strict too. Required
common fields are `schema_version: 1`, `role`, `source: en|ja`, and
`target: zh-Hans`. There is no source-language auto detection or implicit default.
Role-specific fields are:

| Role | Required fields | Optional fields | Forbidden examples |
| --- | --- | --- | --- |
| full | inference, capture | overlay | listen, token, connection |
| server | inference, listen, token | none | capture, overlay, connection |
| client | connection, capture | overlay | inference, models, listen |

`capture` is `{"mode":"follow_default"}` or `{"mode":"device","device_id":"..."}`.
Device identifiers are nonempty, at most 256 bytes and contain no controls.
They designate render-output capture; microphone fallback is not permitted.

`inference` requires `asr_model`, `translation_model`, and `device`.
`device` is `{"backend":"cpu"}` (manual choice) or a Vulkan object with nonzero
`vendor_id`, `device_id`, and a nonempty `device_name` of at most 256 bytes.
The adapter must match a unique actual device and report incompatibility or
ambiguity; these fields are not an enumeration index. GPU initialization or
inference failure never silently switches to CPU or another GPU. Loading and
actual device use still require C2 evidence.

`overlay` defaults to `{"mode":"bilingual","history":3}`. Mode is
`bilingual|source_only|translation_only`; history is 1..20 utterance groups,
including the current draft/final. Visual wrapping does not increase the count.
Display choice does not change recognition or final-only translation timing.
Original appears above translation in bilingual mode. C5 implements rendering,
fonts, position, transparency, tray recovery and user controls.

`listen` is a SocketAddr string with a nonzero port. `connection` requires a
`ws://` address (up to 2048 bytes) and a private shared `token`. URL userinfo,
query, fragment, port zero and other schemes are rejected. The token is 16..256
printable ASCII bytes with no whitespace/control bytes; it must be supplied
privately, not copied from a committed example. Configuration may retain this
private value; Debug, errors, protocol bodies and diagnostics cannot expose it.
C4 passes it only in the authentication handshake header, authenticates before
accepting audio, and implements the approved controlled-LAN boundary. Ordinary
WS does not encrypt the token, audio or subtitles. TLS/public deployment is
outside this version.

A token-free full configuration for structural validation:

```json
{
  "schema_version": 1,
  "role": "full",
  "source": "en",
  "target": "zh-Hans",
  "capture": { "mode": "follow_default" },
  "inference": {
    "asr_model": "models/asr.gguf",
    "translation_model": "models/translation.gguf",
    "device": { "backend": "cpu" }
  }
}
```

Errors expose only a stable category, a whitelisted field projection and available
parser line/column (zero when unavailable), never a foreign parser message or
user payload. Source/target choices are included in the safe explanatory text.
Legacy `deploy/` examples remain drafts, including obsolete `source=auto` and
`schema_status=draft`; they are not accepted schema-1 input. C7 must reconcile
examples, launch contracts, archives and checksums with the implementation.

## Domain events and reducer

Source and translation types are defined in core, imported by the protocol.
Identities are `(session_id, input_generation, utterance_id)`, all nonzero u64.
Session/utterance/revision allocation is checked and monotone; exhaustion fails
rather than wrapping. Generation starts at 1 and increments on input changes.
Sample offsets are u64 at 16,000 samples/second within the current input generation,
not wall-clock timestamps; a new generation restarts its audio timeline at zero.

`SourceEvent` carries identity, nonzero revision, `phase: draft|final`, text,
start_sample and end_sample. Ranges cannot reverse. Text is nonempty after trim,
at most 8192 UTF-8 bytes, and has no controls except newline/tab. Adapter revisions
are allocated locally, not assumed to exist in upstream output.

The reducer keeps 1..20 groups (default 3) and a single highest-utterance watermark:

- Different session/generation is ignored. Reset clears content and IDs; it
  rejects a reused/older identity. Old translations cannot revive cleared text.
- Higher-revision drafts replace the same group. Lower revisions and exact
  duplicates are ignored. A same-revision final is allowed only for exactly the
  same text/range as its draft; changed text at the same revision is invalid.
  Within a group start_sample is fixed and end_sample cannot move backwards.
- Final is terminal. Repeated finals and late drafts, including higher revisions,
  never replace final or trigger another translation. Translation is scheduled
  exactly once on the first accepted final; drafts schedule none.
- New utterance IDs are monotone; their start_sample cannot precede the most
  recently added group. Evicted IDs and previously unseen old IDs are ignored.
  No unbounded set of completed identities is retained.
- Translation carries the same identity and `source_revision`, with either
  `complete {text}` or `failed {code}`. It applies only to a visible matching
  final and only once. Failure preserves original text. Evicted/stale/mismatched
  results are ignored. ASR publication continues independently.

`Update` is the reducer decision, not a wire payload. `Session::pop_event` returns
`PublishedEvent::Source(SourceEvent)` or `Translation(TranslationEvent)` containing
an immutable copy of the exact accepted event. It never requires reading a newer
snapshot to recover a draft or translation. `Message::from(PublishedEvent)` bridges
local publication to remote serialization; there is no second remote reducer.
Snapshots are for rendering; events are for ordered publication.

## Lifecycle and budgets

Compute state is Uninitialized -> Detecting -> Loading -> Ready; failure is
explicit. `loaded()` is only called after both actual model loads succeed, not
after enumerating a device. `ComputeAdapter` defines detect/load/accept_audio/
cancel/shutdown with typed errors. It does not implement a real engine in C1.
The adapter also accepts final `translate` jobs and returns typed content through
`next_event`; outputs still go through Session's identity/revision checks before
publication. Cancellation success must acknowledge cessation, not merely enqueue
a request.

Session is Idle -> Starting -> Running -> Stopping -> Idle, or Failed. Model
readiness is separate. Input/capture confirmation must match identity. Stop
prohibits new input, emits stop/cancel/clear effects and clears audio, source,
translation context and pending events. `stopped()` is called only after adapters
execute cancellation/cleanup. Loaded models remain owned by the application.
Restart allocates a new session and reuses those models. Input changes stop and
cancel old work before new capture and clear all generation-bound content.
Language changes use a new session. Shutdown additionally emits Shutdown;
`released()` requires actual adapter evidence that owned engines/tasks/processes
are released. C1 fake tests verify ordering/ownership, not OS process cleanup.

| Boundary | Initial technical budget |
| --- | --- |
| PCM input | signed PCM16 LE, 16 kHz, mono |
| Frame | 1..3200 samples (at most 200 ms); 6452 encoded bytes |
| Audio queue | at most 100 frames AND 32,000 samples (2 seconds) |
| ASR in-flight | one frame/task, at most 3200 additional samples |
| Translation | at most 4 queued + in-flight jobs combined |
| Publication queue | 64 immutable typed events, each text-bounded |
| Visible history | default 3, configurable 1..20 groups |
| JSON message | 65,536 bytes, including escaped text and all fields |
| Diagnostic record | 1024 bytes, including terminating newline |
| Diagnostic disk | 5 x 4 MiB = 20 MiB total |

AudioChunk is a core type carrying input/seq/start_sample/PCM through the bounded
queue and into ComputeAdapter; `protocol::AudioFrame` reexports it. AudioOrder is
also shared from core, so local and remote gap rules cannot diverge. `push_audio`
validates the timeline, and a discontinuity fails the session. `pop_audio` retains
the complete chunk and allows one in-flight task until `asr_completed(input, seq)`
matches that exact frame. Duplicate, stale and future completions cannot release
the current frame's ownership or permit another in-flight task.
`pop_translation` records the active identity/revision; a matching result releases
its budget even if its group was evicted. Duplicates/stale completions do not
release another job. Actual deadlines and sustained throughput are C2/C4/C6 work.

A full queue does not evict middle audio or silently concatenate gaps. It returns
Overloaded, transitions Session to Failed, clears content and records bounded
stop/cancel effects in `take_cleanup()`. The scheduler must execute them and
surface the failure before recovery; it must not treat the rejected operation as
success. Model state stays Ready unless an inference error also invalidates it.
Every separate later capture/network queue must enforce the same aggregate
budget or an explicitly documented bound; stacking unlimited per-layer queues
is forbidden. These are initial technical bounds, not approved release latency
or accuracy promises; measured changes must update all consumers and fixtures.

## WS v1 JSON and PCM codecs

Codecs validate boundaries and return safe ErrorCode values. C4 owns sockets,
authentication, service capacity/readiness and retry decisions. Ping/pong is WS
transport control, not JSON or model readiness.

JSON envelope:

```json
{"protocol_version":1,"request_id":1,"message":{"type":"hello","source":"en","target":"zh-Hans"}}
```

`request_id` is a nonzero monotonically increasing sequence per sender/connection.
It does not echo another sender's sequence. Control operations are single-flight;
acks are matched by the pending state/identity. No token field exists. All
objects reject unknown/duplicate fields and invalid enums/identities/versions.

| Direction | Message | Fields |
| --- | --- | --- |
| client -> server | hello | source, target |
| server -> client | hello_ack | sample_rate=16000, channels=1, max_frame_samples=3200, max_audio_samples=32000, max_text_bytes=8192 |
| client -> server | session_start | source, target |
| server -> client | session_started | input (generation=1, fresh session) |
| client -> server | input_switch | input (same session, exactly next generation) |
| server -> client | input_switched | input (ack after old cancellation/new capture readiness) |
| server -> client | source / translation | event (shared domain type) |
| server -> client | status / error | state / code (safe enums only) |
| client -> server | session_stop | input |
| server -> client | session_stopped | input (ack after cancellation) |

The server-side ConnectionGuard is constructed only after real transport
header authentication. It requires hello before session_start and session_started
before audio. Source/translation and server acks cannot be accepted as client
requests. It checks strictly increasing request IDs, one active stream,
generation switch acknowledgements, matching stop and fresh session IDs.
Switching/stopping/closed states reject audio. C4 separately gates model readiness
and single-stream service capacity; constructing this guard is not authentication.
A client consumer uses the domain reducer to reject stale result identity.

PCM binary header is exactly 52 bytes; all multibyte integers are little-endian:

| Offset | Bytes | Value |
| --- | --- | --- |
| 0 | 4 | ASCII SYNA |
| 4 | 2 | protocol version 1 |
| 6 | 2 | header length 52 |
| 8 | 8 | session_id |
| 16 | 8 | input_generation |
| 24 | 8 | seq (first 0) |
| 32 | 8 | start_sample (first 0) |
| 40 | 4 | sample_count, 1..3200 |
| 44 | 4 | sample_rate 16000 |
| 48 | 1 | channels 1 |
| 49 | 1 | sample format 1 (signed PCM16 LE) |
| 50 | 2 | reserved flags 0 |
| 52 | 2 x sample_count | PCM payload |

Length must exactly match, with no trailing data. Zero/oversized sample counts,
foreign format/header/version, impossible identities, offset/sequence overflow
and oversize frames fail. `AudioOrder` checks expected session/generation,
consecutive seq and contiguous samples, and advances only on success. A gap or
replay returns Discontinuity; the scheduler must end/reset the affected session,
not splice audio. Create a new guard when a fresh input is acknowledged.

Reconnect cancels and discards old audio/results/context, creates a fresh
connection and session, and resumes upload only after its new session_started.
Stopped sessions must not auto-reopen. C4 implements bounded transient-network
retry and immediate stop for authentication/protocol errors; no retry scheduler
is implemented by this codec. Golden fixtures are under
`crates/synesthesia-protocol/tests/fixtures/` and use controlled synthetic inputs.

## Safe diagnostic persistence

`DiagnosticLog` is the sole sink shared by app and owned engines for a runtime
root. Its exclusively owned directory contains diagnostics.0.jsonl (newest)
through diagnostics.4.jsonl (oldest) and a zero-byte writer.lock. An OS exclusive
file lock prevents simultaneous cooperating writers and releases on process
exit; the lock file can remain safely for restart. A second writer or foreign
file/nonregular entry in the directory fails with StorageUnavailable, preserving
foreign content. This contract assumes a private runtime directory and excludes
hostile concurrent filesystem mutation; it does not manage another process's
logs or provide a filesystem security sandbox.

`Diagnostic` accepts only timestamp, component/kind/error/backend enums, numeric
vendor/device IDs, elapsed_ms and queued_samples. There is no arbitrary String,
message formatter, payload, audio, token, prompt, device-name or foreign error
argument. Content-bearing events have redacted Debug implementations too.
Engine stdout/stderr must be parsed and safe fields extracted in memory by C2;
never tee unfiltered output to disk. Engines must disable content/transcript/
input-file persistence and use this sink, not additional files that bypass the
20 MiB application-plus-owned-engine budget. Compatibility failure must be
reported if an engine cannot meet that policy.

Records larger than 1024 bytes are rejected before writing. Before an append
would exceed 4 MiB, oldest is removed and older segments rotate; files are never
allowed beyond 5 x 4 MiB. Startup checks all segments, removes oversized owned
segments and truncates a partial final record in the active file after an
interrupted write. It does not promise lossless or crash-durable diagnostics.
Write/rotation/lock failures return a safe storage category rather than raw
paths/foreign errors or fallback content logging. Models and non-content caches
are independent of this budget. Audio/subtitle/prompt persistence/export is
absent; configurations and controlled benchmark source inputs are separate.

## Verification boundary

Deterministic tests cover payload-preserving publication, draft replacement,
final idempotency, version match, eviction, stale session/generation isolation,
checked ID/weight overflow, fake adapter cancellation/model reuse/release,
queue overload including in-flight translations, strict role/config privacy,
CWD-independent model resolution, golden PCM/JSON and malformed bounds/order,
log rotation/restart/partial writes/exclusive writers and actual 20 MiB budget.
Random-byte codec smoke coverage is bounded and is not exhaustive fuzzing.

C2/C3/C4/C5/C6/C7 still own real engine compatibility, samples, native capture,
network authentication/retry, tray/windows, full integration and release assets.
Windows 10 and Windows 11 native build and device evidence are required
separately; Linux checks do not substitute for them. Neither the 60-minute real
chain tests nor quality/latency/resource release gates have been met in C1.
