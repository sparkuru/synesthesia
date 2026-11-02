# Rust Backend and Shared Contracts

Current product: root PRD v2 and docs/architecture.md. Formal entry:
apps/synesthesia with schema 2. Three-role entries, schema 1 and existing native
adapters remain for compatibility; independent ASR/translation and complete
runtime integration are pending.

## Guidelines Index

| Guide | Scope |
| --- | --- |
| [Directory Structure](directory-structure.md) | Workspace ownership and role dependencies |
| [Core Boundaries](core-boundaries.md) | Executable configuration/session/audio/wire contracts |
| [Pipeline Configuration](pipeline.md) | Current desktop schema 2 and independent choices |
| [Error Handling](error-handling.md) | Safe typed boundary failures |
| [Logging](logging-guidelines.md) | Whitelist-only bounded diagnostics |
| [Quality](quality-guidelines.md) | Gates and meaningful regression coverage |

## Pre-Development Checklist

Read relevant guides, `docs/contracts.md`, and the active task's approved
artifacts. Verify source signatures before implementing an adapter. Preserve
client-without-models and server-without-desktop dependencies. Load applicable
native-language/tooling skills before adding native code.

## Quality Check

Run [Quality](quality-guidelines.md) gates. Review complete task scope and data
flow, queue/context budgets, exact completion identity, cancellation
acknowledgements, safe errors and diagnostics. Enumeration, fake adapters and
Linux checks do not prove Windows loading, inference, capture or residency.
