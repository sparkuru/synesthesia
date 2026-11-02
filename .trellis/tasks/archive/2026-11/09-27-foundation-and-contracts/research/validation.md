# C1 implementation validation

Date: 2026-09-27. Scope: foundation/configuration/domain/codec/diagnostics only.
The main session activated this task after final parent approval. No commit,
archive, release, real audio capture, model execution, GUI or WS service was
performed by this implementer. Parent/root PRD, user readme, other tasks, deploy
scaffold/archive and protected Trellis/platform files were preserved.

## Implemented artifacts

- Root Cargo workspace, Cargo.lock and rust-toolchain.toml: Rust 1.90.0,
  edition 2024; core, protocol and three command executables. Each entry only
  depends on core. Core has serde/serde_json/serde_path_to_error/url dependencies;
  protocol imports core types. No audio, GUI or engine placeholders were created.
- Core strict role configuration, manually selected en/ja -> zh-Hans, safe token
  representation, structured safe parse locations, absolute runtime roots and
  model-path resolution before adapters consume configuration.
- Core AudioChunk/AudioOrder preserve identity, sequence and sample offsets
  through local/remote queues and engine interfaces. SubtitleReducer handles
  draft replacement, final terminal state/idempotency, source revision matching,
  eviction watermark and stale session/generation rejection. PublishedEvent
  carries each immutable payload rather than reconstructing it from a snapshot.
- Core deterministic session effects with load/readiness separation, stopped
  model ownership, generation/session boundaries, explicit queue-overload/gap
  failure and cancellation, one ASR task and four queued+in-flight translations.
- Protocol v1 strict typed JSON, fixed 52-byte PCM header, bounded length/format/
  sample/order/identity validation, state guard and golden JSON/PCM fixtures.
- DiagnosticLog safe field whitelist, cooperating-writer OS lock, five 4 MiB
  segments (20 MiB total including owned engine diagnostics), record bound,
  restart/partial-write recovery and safe IO failure.
- Truthful help/version/config-only entry points; docs/contracts.md fixes current
  schemas, wire layout, budgets, adapter obligations and unfinished boundaries.
- Original Linux-only hako: ephemeral Rust image, no ports, UID/GID, repository
  caches; .gitignore adds .devhome and target. Windows uses native tools instead.

## Actual Linux environment and commands

Existing image: rust:1.90-bookworm,
`sha256:cc39c88add68c6e4ed377beccd6a659e59b286954d2c7e5c8e5b1dde2a82a8e8`.
Image initially contained cargo/rustc/std only. Host has no cargo/rustc;
Docker daemon 26.1.5 was already present (main-session evidence).

Pinned tools in ignored .devhome:

| Tool | Actual version |
| --- | --- |
| rustc | 1.90.0 (1159e78c4 2025-09-14) |
| cargo | 1.90.0 (840b83a10 2025-07-30) |
| rustfmt | 1.8.0-stable (1159e78c47 2025-09-14) |
| clippy | 0.1.90 (1159e78c47 2025-09-14) |

`./hako rustup toolchain install 1.90.0 --profile minimal --component rustfmt
--component clippy` successfully downloaded/installed all components, then exited
1 because self-update expected rustup installed in relocated CARGO_HOME. Direct
rustc/cargo/fmt/clippy probes subsequently exited 0 and all application gates used
these components. Documented future bootstrap adds `--no-self-update`, rather
than treating the original entire command as a success or reinstalling globally.

Final validation:

| Exact command | Result |
| --- | --- |
| ./hako cargo fmt --all -- --check | exit 0 |
| ./hako cargo check --workspace --all-targets --locked | exit 0, all five crates/targets |
| ./hako cargo clippy --workspace --all-targets --locked -- -D warnings | exit 0, no warnings |
| ./hako cargo test --workspace --locked | exit 0, 33 behavior tests, 0 failed |
| ./hako cargo build --workspace --locked | exit 0, all three executables |
| ./hako target/debug/synesthesia-full --help / --version | both exit 0, truthful config-only help/version |
| ./hako target/debug/synesthesia-server --help / --version | both exit 0, truthful config-only help/version |
| ./hako target/debug/synesthesia-client --help / --version | both exit 0, truthful config-only help/version |
| ./hako cargo metadata --no-deps --format-version 1 --locked | exit 0; each entry -> core; protocol -> core; no GUI/inference dependency |
| bash -n hako | exit 0 |
| shellcheck hako | exit 0 |
| shfmt -d -i 4 hako | exit 0 |
| git diff --check | exit 0 (tracked paths); untracked content also checked separately |

No ports, persistent service, dev.sh, Docker deployment image, sandbox policy or
agent permission files were created/changed. Docker access used narrow hako
escalations; no model/GPU behavior was inferred from this Linux result.

## Behavior coverage and resolved failures

Five diagnostic tests cover rotation/newest retention, restart bounds, actual
20 MiB boundary, partial records, writer exclusion, foreign file refusal,
oversized records, IO failure and field whitelist. Twenty-one core tests cover
strict role/schema/language/null/duplicate/unknown config; private error/Debug;
URL/header injection rejection; relative/external model resolution and config
file bounds; truthful CLI; draft replacement, final idempotency, revision
stability, eviction/stale rejection; overflow; fake model reuse/cancellation/
release; audio timing/gaps and overload; in-flight translation/event budgets;
immutable publication payloads. Seven protocol tests cover every control/result
roundtrip, golden fixtures, malformed length/version/enum/format/identity,
sequence gaps/replay/foreign generation, state ordering/stop/fresh restart,
maximum escaped text, redacted Debug and bounded random-byte decoder smoke tests.
These are controlled test strings/samples, not retained user session content.

The first workspace check failed before a protocol src directory existed;
creating its real source resolved it. An initial malformed-frame test flipped a
nonzero session byte into another valid nonzero ID and incorrectly expected a
codec error; it now zeros the entire ID for syntax validation and separately
tests foreign IDs in AudioOrder. Two Clippy collapsible-if findings were fixed.
A useful-location test exposed that Serde internally tagged configuration enum
buffering loses the source path; the strict RawConfig boundary now retains source
field and original line/column, and rejects explicit null forbidden-role fields.
All failures were followed by passing relevant tests and the final full gate.

## Stable decisions and source basis

Current initial bounds: 16 kHz mono PCM16, 3200 samples/frame; 32,000 queued
samples AND 100 frames; one in-flight ASR frame; four queued+in-flight translation
jobs; 64 actual publication events; default three visible groups, maximum 20;
8192 text bytes; 65,536 JSON/config bytes; 52-byte audio header. All overflow is
checked. These are engineering bounds, not invented accuracy/latency promises.
They must be reconciled with C2/C4 measurements and all consumers if changed.

No existing backend conventions were invented from template specs. Original
source-backed contracts live in docs/contracts.md. Primary API references used:
[Serde strict container attributes](https://serde.rs/container-attrs.html),
[serde_json typed parsing](https://docs.rs/serde_json/1.0.151/serde_json/),
[URL parser API](https://docs.rs/url/2.5.8/url/struct.Url.html), and
[Rust File locking API](https://doc.rust-lang.org/std/fs/struct.File.html#method.try_lock).
Actual pinned compilation is the evidence of API availability on Linux Rust
1.90.0; current API documentation alone does not prove native Windows behavior.

## Remaining gates and review

Independent trellis-check is still pending and owned by the main session.
Native Windows 11 tool preparation is separately recorded by the main session in
native-toolchain.md; this implementer's Linux report does not claim native
application checks. The main session is validating a stable source/lock/fixture
snapshot natively. Windows 10 remains an independent acceptance requirement.

C1's AC-04/05/10/11/14/15 contribution is domain/configuration/lifecycle/diagnostic
behavior; the corresponding full product ACs are not passed. C2-C7 still owe real
ASR/translation/model/actual-GPU compatibility, fixed controlled benchmarks,
WASAPI output capture, WS authentication/network/retry, native tray/overlay,
full/client-server integration, 60-minute load tests, real process/resource
release and packaging/license material. Existing deploy examples are draft and
must be synced by C7 with schema 1, then checksums/archive regenerated.

Human gate: human-required before final native/product acceptance because
Windows 10 and device/GUI/GPU/network behavior are not exercised here. This does
not request the user to manually test unimplemented hardware features or block
independent C1 code review. The smallest current residual is native Windows
build/test/config-only entry behavior, which the main session is automating;
material security/authentication and real-device residuals belong to later tasks.
Do not mark C1/MVP complete or release-ready solely from the Linux gates.
