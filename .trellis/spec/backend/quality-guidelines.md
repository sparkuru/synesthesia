# Rust Quality Gates

Pinned rust-toolchain.toml: 1.90.0, rustfmt and clippy. Linux development uses
original `./hako` and ignored .devhome; this is not a Windows runtime dependency.
With dependencies cached, set HAKO_NETWORK=none:

```text
./hako cargo fmt --all -- --check
./hako cargo check --workspace --all-targets --locked
./hako cargo clippy --workspace --all-targets --locked -- -D warnings
./hako cargo test --workspace --locked
./hako cargo build --workspace --locked
```

Windows uses the same gates with native Cargo/MSVC, never Docker/WSL.
C1's [native harness](../../tasks/archive/2026-11/09-27-foundation-and-contracts/research/native-foundation-check.ps1)
and [evidence](../../tasks/archive/2026-11/09-27-foundation-and-contracts/research/native-validation.md)
record the exercised Windows 11 baseline. Windows 10 needs separate evidence.
`.github/workflows/rust.yml` runs the full workspace gates on Linux. Use
`cargo test --workspace --all-targets --locked` to include example tests.
There is no runnable application service or native-window CI yet.

Test boundary behavior: strict nested schema/original URL authority,
draft/final/revision isolation, exact completion identity, immutable publication,
audio ordering, budgets including in-flight work, cancellation/ownership,
diagnostics and malformed/golden wire data. Fake readiness cannot replace native
adapters/devices. Integration/sustained gates come from the approved task.
D-10 numerical quality/latency/resource gates require actual baseline review.

For hako edits run bash -n, ShellCheck and shfmt -d -i 4. Check git diff --check
plus explicit UTF-8/newline/trailing-whitespace scans for untracked files.
Preserve user changes and task ownership.
