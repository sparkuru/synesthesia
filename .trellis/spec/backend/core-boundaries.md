# Executable Core Boundaries

> Retained schema 1 / WS v1 source contracts. Current desktop configuration is
> [schema 2](pipeline.md). The old combined ComputeAdapter, required translation
> and stop-cancels-all behavior need migration before serving the current PRD.

## 1. Scope / Trigger

Config, schedulers, audio adapters and WS consumers share C1 source contracts.
Full schemas/offsets: `docs/contracts.md`. Authorities: core config.rs,
domain.rs, session.rs and protocol src/lib.rs. Tests: core tests/contracts.rs
and protocol tests/wire.rs under their `crates/synesthesia-*` packages.

## 2. Signatures

```rust
Config::parse(bytes: &[u8], expected: Role) -> Result<Config, ConfigError>
RuntimeRoot::load(&self, path: &Path, role: Role) -> Result<Config, ConfigError>
Session::asr_completed(&mut self, input: InputId, seq: u64) -> Result<(), ErrorCode>
Session::pop_event(&mut self) -> Option<PublishedEvent>
AudioOrder::accept(&mut self, frame: &AudioChunk) -> Result<(), ErrorCode>
```

ComputeAdapter owns detect/load/audio/translation/events/cancel/shutdown.
Success acknowledges actual completion. Model ownership is separate from
session context and pending work.

## 3. Contracts

Config v1 is role-specific and strict, source en/ja, target zh-Hans. Use
FollowDefault {} / Cpu {} for no-field internally tagged objects; unit variants
can accept extra keys. Relative paths resolve from absolute RuntimeRoot or
executable directory, never CWD. WS forbids userinfo including normalized-away
empty @; check the original authority too.

InputId carries session/generation; utterances/translations carry matching
identity/revision. Final is terminal/idempotent and translates once.
PublishedEvent contains the accepted immutable payload. AudioChunk carries
input/seq/start_sample and mono 16 kHz PCM16. AudioOrder validates continuity.
Frames: 1..3200 samples; audio queue: 32000 samples/100 frames; ASR: one exact
(input, seq); translation: four queued-plus-active; publication: 64; text:
8192 UTF-8 bytes; history: max 20. WS JSON: 65536 bytes; PCM header: 52 bytes,
max frame: 6452 bytes. No environment override changes these contracts.

## 4. Validation & Error Matrix

| Boundary | Condition | Behavior |
| --- | --- | --- |
| Config | unknown/duplicate/null forbidden field | Safe ConfigError without values |
| Config | wrong role/language/path/token/address | Typed category rejection |
| ASR completion | previous frame/wrong identity | Preserve current owner |
| Audio | gap/overflow/budget exceeded | Explicit failure/cleanup; no skipping |
| Subtitle | stale/duplicate event | No duplicate work/publication or revival |
| Wire | malformed/version/size/order failure | Reject before admission |
| Authentication | transport not authenticated | Do not construct authenticated guard |

## 5. Good / Base / Bad Cases

Good: publish final source immediately; later translation matches its revision.
Base: stop cancels old work/clears context but retains loaded models.
Bad: late completion of frame N releases N+1, normalized URL bypasses policy,
or a flag replaces actual loading evidence.

## 6. Tests Required

Assert duplicate/wrong ASR completions cannot release current work; no-field
configs reject extras; ws://@localhost/ rejects; stale events never return after
reset/eviction; final translates once; queued plus active limits hold; shared
audio ordering rejects gaps; maximum escaped valid text survives wire bounds;
malformed inputs never panic/leak. Native tests establish cancellation, identity
and reuse separately.

## 7. Wrong vs Correct

Wrong: acknowledge only input or reconstruct publication from a later snapshot.
Correct: acknowledge (input, seq), send PublishedEvent's original payload.
Wrong: assume a tagged Cpu unit variant rejects unknown keys.
Correct: Cpu {} and focused unknown/duplicate nested-field regressions.
