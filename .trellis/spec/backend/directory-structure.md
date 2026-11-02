# Workspace Structure

`Cargo.toml` uses Rust 2024, resolver 3, toolchain/MSRV 1.90 and members
`crates/*`, `apps/*`. Keep Cargo.lock. New crates inherit package metadata;
coordinate shared root dependencies across concurrent tasks.

| Source | Responsibility |
| --- | --- |
| core/src/config.rs | Strict role schemas and runtime paths |
| core/src/pipeline.rs | Desktop schema 2; independent capture/ASR/translation |
| core/src/domain.rs | Audio identities, subtitle revisions and reducer |
| core/src/session.rs | Deterministic scheduling, effects and bounded queues |
| core/src/diagnostics.rs | One owned bounded persistence sink |
| core/src/cli.rs | Structural validation commands |
| protocol/src/lib.rs | Versioned WS messages, PCM codec and ordering guard |
| apps/synesthesia-{full,server,client}/src/main.rs | Thin role entries |
| apps/synesthesia/src/main.rs | Formal desktop entry, structural checks only |
| docs/contracts.md | Shared schema, wire offsets and lifecycle reference |

Core/protocol above are under `crates/synesthesia-*`. Protocol depends on core;
core remains transport-free. Entries currently expose configuration validation,
not product runtime. Behavior tests live in crate `tests/`; native experimental
evidence lives in task `research/`.

Plain cargo run/build selects apps/synesthesia through default-members. Quality
gates always use --workspace; no members are excluded. Desktop currently owns
borrowed presentation projections, not native windows. Cloud/microphone and
independent provider runtime remain pending; schema variants do not prove them.

No database, ORM or migration layer exists. Audio, subtitles, prompts and tokens
are transient. Only whitelisted diagnostics persist; models are external paths.
Do not add a content cache/database incidentally.

Core/protocol/apps inherit `unsafe_code = "forbid"`. Native FFI belongs in a
narrow adapter package with explicit safety/resource ownership; do not weaken
platform-free crates. Client must not gain inference/model dependencies; server
must not gain window/tray dependencies.
