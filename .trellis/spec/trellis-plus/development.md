# Development Readiness

> Current product requirements are [PRD v2](../../../prd.md). Use
> [Engineering Status](../../../docs/status.md) for implemented capabilities and
> [Architecture](../../../docs/architecture.md) for migration boundaries.
> Historical task approvals and experiments do not authorize new product scope.

- ownership: project-shared
- source: project-authored
- tracking: shared policy; agent settings remain local

## Current Development Environment

`Cargo.toml`, `Cargo.lock` and `rust-toolchain.toml` establish Rust 2024 /
1.90.0. Plain Cargo run/build selects the formal `apps/synesthesia` entry;
quality gates include every workspace member. The
[Rust CI workflow](../../../.github/workflows/rust.yml) defines these gates on
Linux; its presence does not establish a successful remote run. Prior local
execution is recorded in the [1102 validation report](../../tasks/11-02-rust-repository/research/validation.md).
Use Engineering Status for application and adapter readiness rather than
inferring it from build configuration or historical UI framework choices.

Wrapper status: `ready-for-linux-development`. Original `hako` uses
rust:1.90-bookworm, repo-only mounts, host UID/GID, ignored `.devhome`, ephemeral
labelled command containers and no ports. Cached checks run with
`HAKO_NETWORK=none`. No `dev.sh` exists because no dev service exists. Use
session-scoped narrow `./hako` approvals; no permission files were changed.
Native Windows 10/11 x64 development/runtime never requires Docker or WSL.
The historical deployment Compose draft is not a runnable development service.

## Historical C1 Environment Evidence

The [C1 native validation](../../tasks/09-27-foundation-and-contracts/research/native-validation.md)
records its private Rust/MSVC harness on Windows 11. That result applies to the
then-tested foundation, not current capture, GPU, UI or Windows 10 acceptance.
The old C5 framework selection is historical research; a current native-window
implementation must establish its actual dependencies and validation.

## Trellis Plus: Docker Dev-Command Bootstrap

Before a task first needs dependency installation, lint, type-check, test,
build, or a development server, inspect the current manifests and documented
commands. Reuse an existing wrapper if one has appeared. Otherwise apply the
`dev-it-in-docker` skill after the toolchain and command contract are known.
This is a bootstrap/readiness checkpoint, not an additional Trellis phase.

Derive the image, versions, service commands, and ports from current evidence.
Keep application deployment separate from development tooling. Review the
source/license of generated content before proposing it as shared material;
author original wrapper code or leave it local if provenance is unresolved.

For a new wrapper, use repo-scoped mounts, host UID/GID, ephemeral command
containers, and an ignored `.devhome` for caches. One-shot commands publish no
ports. A confirmed dev service may use configurable host/container ports with
loopback binding by default. Add `dev.sh` only for a real runnable dev service;
teardown must select exact project and service-scope labels and preserve
unrelated containers and persistent data.

Verify the executable wrapper with the cheapest real tool version command;
verify service teardown and any short readiness probe when a service exists.
Record missing Docker, image/network, permission, or platform dependencies
without reporting success. Docker checks cannot prove native desktop capture,
overlay, or GPU support.

## Local Execution Settings

For Codex, the maintained base wrapper skill prohibits creating
`.codex/rules/default.rules`; use supported session-scoped prefix approval for
`./hako`. No local rule is installed. Reconcile any future supported adapter
with the active surface and maintained base skill before writing it.
Do not modify managed Trellis `.codex/config.toml` or hooks for this enhancement.

Never add broad `docker`, shell, or package-manager allowances, relax sandbox
or network policy, or treat a hook result as authorization. Local settings do
not override managed permissions. If a supported narrow adapter is needed
later, preserve existing rules, keep it untracked, and verify it before use.
