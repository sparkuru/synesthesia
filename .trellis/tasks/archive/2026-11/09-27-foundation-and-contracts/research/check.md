# Independent C1 review

Date: 2026-09-27. Active task: `09-27-foundation-and-contracts`.
Scope: complete C1 workspace, core/protocol, three command entries, contracts,
Linux wrapper and root ignore rules. This report does not accept C2-C7 or the MVP.
The reviewer loaded the full saved hook context, task artifacts and all nine
check-context references, `docs/contracts.md`, Trellis before-dev/check skills,
backend/frontend indexes, shared thinking guides and shell review instructions.
`get_context.py --mode packages` reports a single repo with backend, frontend and
trellis-plus layers. The first two indexes are unfilled templates with no actual
Quality Check section; they are not treated as implemented conventions.

## Findings fixed

1. `crates/synesthesia-core/src/session.rs`: ASR completion acknowledged only an
   input identity. After frame N completed and frame N+1 began, a duplicate
   completion of N could release N+1 and allow another concurrent ASR task.
   In-flight ownership now stores `(InputId, seq)` and
   `asr_completed(input, seq)` requires that exact pair. Wrong, duplicate and old
   generation completions preserve current ownership. Added a focused regression
   in `tests/contracts.rs` and synchronized `docs/contracts.md`.
2. `crates/synesthesia-core/src/config.rs`: Serde internally tagged unit variants
   `FollowDefault` and `Cpu` accepted extra keys despite `deny_unknown_fields`.
   A focused nested-config test failed before correction (exit 101), then passed.
   Empty struct variants `FollowDefault {}` and `Cpu {}` now enforce rejection.
   The schema/wire JSON is unchanged; Rust constructors/patterns use braces.
   Tests also reject duplicate nested discriminants/tokens and null extra fields.
3. The same config boundary accepted `ws://@localhost/`: `url::Url` normalizes
   empty userinfo away before the existing credential checks. A focused address
   test failed before correction (exit 101). The original authority is now checked
   for userinfo; the complete address-rejection test passes after the fix.
4. `crates/synesthesia-core/src/diagnostics.rs`: an existing nonempty
   `writer.lock` could retain arbitrary bytes outside the five-segment aggregate
   budget. The acquired lock is now truncated to zero after successful exclusive
   ownership, never before. A sparse lock larger than 20 MiB is normalized;
   concurrent writers remain rejected. Added an owned-directory regression.

Changed source: core `config.rs`, `session.rs`, `diagnostics.rs`, core
`tests/contracts.rs`, and `docs/contracts.md`. No lockfile, manifest, protocol
fixture, app entry, wrapper, lifecycle, protected platform or other task change
was made by this review. User PRD/README and concurrent task work were preserved.

## Contract review

- Role schema is typed and versioned; explicit null, duplicate/unknown fields,
  unsupported languages, inference fields in clients and improper role fields
  are rejected. Token Debug and parser errors are redacted/safely projected.
  RuntimeRoot resolves configuration/models from an absolute executable/explicit
  root; model availability remains explicitly unverified by the CLIs.
- The shared reducer owns source revisions, terminal final state, translation
  source-version matching, session/generation isolation and bounded history with
  a high watermark. Publication contains immutable accepted event payloads,
  rather than requiring a later snapshot. Final translation is triggered once.
- AudioChunk and AudioOrder are shared across local/remote boundaries. Queue
  weights and counts are bounded; ASR has one exact in-flight frame, translation
  counts queued plus active jobs, and publication is bounded. Overload/gap failure
  clears content and returns explicit cleanup effects rather than skipping audio.
- Start/load/stop/switch/release acknowledgements require actual adapter evidence
  from future schedulers. C1 fake adapters are confined to tests. Stop effects
  preserve loaded model ownership; shutdown requests release. C1 cannot prove
  real process cancellation, capture teardown, inference or GPU residency.
- WS JSON/PCM bounds, version/identity/format and ordering are centralized.
  Golden fixtures and malformed/oversized/replay coverage pass. ConnectionGuard
  permits requests only after authenticated construction/hello and forbids audio
  while starting/switching/stopping/closed. Construction is not real authentication.
- Diagnostics permit only safe enums/numeric fields; no arbitrary content String
  exists in the persistence API. Rotation, restart, oversize, exclusive writer,
  partial-record recovery, write failure and actual 20 MiB behavior are exercised.
  Engines must share the sink and suppress raw content outputs when integrated.
- App manifests depend only on core; protocol depends on core; client has no
  inference/model package and server has no desktop package. Normal commands
  truthfully expose help/version/config validation and do not claim readiness.
- `hako` uses repo-only mounts/cache, host UID/GID, ephemeral labelled containers
  and no ports. Windows does not use it. Existing tracked scaffold exception in
  `.gitignore` is preserved. No protected platform/template change occurred.

## Verification

Final reviewed source was checked using `rust:1.90-bookworm` with the cached
pinned toolchain. Every command below used `HAKO_NETWORK=none` and exited 0:

```text
HAKO_NETWORK=none ./hako cargo fmt --all -- --check
HAKO_NETWORK=none ./hako cargo check --workspace --all-targets --locked
HAKO_NETWORK=none ./hako cargo clippy --workspace --all-targets --locked -- -D warnings
HAKO_NETWORK=none ./hako cargo test --workspace --locked
HAKO_NETWORK=none ./hako cargo build --workspace --locked
```

Tests: **36 passed**, zero failed (6 diagnostics unit tests, 23 core contract
tests, 7 protocol tests; app/unit/doc targets with no tests are not counted).
The earlier implementation snapshot had 33 tests and is historical evidence.
Focused failing-before-fix tests used the same offline wrapper and `--locked`.

Version probes, also offline, all exited 0: rustc 1.90.0
(`1159e78c4747b02ef996e55082b704c09b970588`, host
`x86_64-unknown-linux-gnu`, LLVM 20.1.8), cargo 1.90.0
(`840b83a10`), rustfmt 1.8.0-stable and clippy 0.1.90.
`bash -n hako`, `shellcheck hako` (0.10.0), `shfmt -d -i 4 hako` (3.8.0), and
`git diff --check` all exited 0. Source/contract whitespace was also explicitly
checked for untracked C1 files, since git diff does not include them.

Snapshot identity for the main session's native revalidation:

| File | SHA256 |
| --- | --- |
| Cargo.lock | e07f9545f8b0dfe662c008edfa62fc9d00d16e31fbec388ff51849c1fe436dfc |
| core/src/config.rs | 5b87b0ffcc8ee03da7c6c65f151514fd6b89cc55161c53b06c0fbcc5b88f3619 |
| core/src/session.rs | 69e579785557340a2e685fc70980a75c9e9f89ff79d7335bdfc91f089e483e32 |
| core/src/diagnostics.rs | 0d14fb8b3d10cdc8376e40cb68bd0181aeec225bba4a3bd42004abf992be9c64 |
| core/tests/contracts.rs | 9737dc3e06212ee2927f0c16dc98503b76d5f98cdf60797202d59938e6c78dcc |
| docs/contracts.md | 3f20b74aaad4fa11b10891a49e9c2bfab68de70d56079d6c987bbf14c7650efb |

Core rows are relative to `crates/synesthesia-core/`.

## Residual evidence and spec handoff

No known unresolved C1 code finding remains. Linux checks prove platform-free
contracts, not Windows compatibility, capture, GUI, network authentication or GPU
inference. The main session owns native Windows revalidation of this updated
snapshot; previous native snapshots cannot substitute for it. Windows 10 evidence
is separate from the available Windows 11 host. Native/device/product evidence
and D-10 quality/latency/resource thresholds remain human-required before final
MVP release acceptance; these residuals do not block other approved development.
No generic human smoke check substitutes for the completed C1 automation.

Backend/frontend bootstrap placeholders and Trellis Plus readiness descriptions
need source-backed follow-up by the main session, within its spec ownership.
Stable lessons to preserve: use empty struct variants for strict internally
tagged no-field config objects; retain exact work identity in completion
acknowledgements; keep original URL authority checks when normalization erases
forbidden forms; include lock metadata in the disk-ownership budget. Existing
spec indexes/runtime/platform templates were deliberately left untouched.

No task completion/archive, commit/push, remote installation, model execution,
listener/deployment or MVP/release completion is declared by this reviewer.
