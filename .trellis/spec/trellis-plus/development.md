# Development Readiness

- ownership: project-shared
- source: project-authored
- tracking: shared policy; agent settings remain local

## Repository Evidence

`prd.md` treats Rust and egui/eframe as candidates requiring confirmation.
There is no Cargo/package manifest, lockfile, application source, CI pipeline,
`hako`, `dev.sh`, or `.devhome`. Trellis's Python scripts are workflow tooling,
not an application toolchain declaration. `deploy/docker/compose.example.yaml`
is a deployment draft requiring an unimplemented image; its port 8765 is not
evidence for a development server or readiness endpoint.

Wrapper status: `deferred-toolchain-unconfirmed`. No Docker wrapper, runtime,
container, cache, or agent permission rule is created by this enhancement.

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

For Codex, the prospective narrow wrapper-rule target is
`.codex/rules/default.rules`; no rule is installed while `hako` is absent.
Reconcile any later local rule with the active surface and maintained base
skill before writing it: the currently available base skill explicitly
prohibits creating that rules file and instead uses supported session-scoped
prefix approval for `./hako`. Use that conservative path until reconciled.
Do not modify managed Trellis `.codex/config.toml` or hooks for this enhancement.

Never add broad `docker`, shell, or package-manager allowances, relax sandbox
or network policy, or treat a hook result as authorization. Local settings do
not override managed permissions. If a supported narrow adapter is needed
later, preserve existing rules, keep it untracked, and verify it before use.
