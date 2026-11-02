# Native Windows foundation validation

Date: 2026-09-27. This report belongs to the main session and is separate from the implementer's [Linux validation](validation.md) and independent [review](check.md). Scope is configuration/domain/codec/diagnostic foundation only; no capture, GUI, model loading, inference or WS listener runs here.

## Environment and exact input

- User-provided Windows 11 baseline machine, build 26100, accessed with existing SSH key authentication.
- Existing Visual Studio Community 2026/MSVC 14.50.35717 developer environment; [native tool preparation](native-toolchain.md) provides private Rust/Cargo 1.90.0, rustfmt and Clippy in this session's independent account Temp directory.
- Source snapshot v1 SHA256: `889232a85002222b7cdb7fa964f4b2cf3143958b4c9a57838adb5d102fc7bcf8`.
- Registry cache/index transfer SHA256: `d737a168906104e664227e04434b66286505b86a91a7eee7d9cb000ba2c3d890`. Only public dependencies from the approved Linux build cache were copied; credentials, user configuration, model weights and unrelated repository files were excluded.
- Cargo.lock SHA256: `e07f9545f8b0dfe662c008edfa62fc9d00d16e31fbec388ff51849c1fe436dfc`.
- Uploaded archive hashes were checked again on the destination before extraction. The source and caches occupy newly created `foundation` / `cargo-cache` children of the recorded task-owned Windows workspace; build outputs use `foundation-target`.

## Actual commands and v1 result

[The native harness](native-foundation-check.ps1) runs in a child developer environment with process-local PATH/RUSTC/CARGO_HOME/CARGO_TARGET_DIR. It uses native Windows executables and MSVC, without Docker/WSL/global installation.

| Command | v1 result |
| --- | --- |
| cargo fmt --all -- --check | exit 0 |
| cargo check --workspace --all-targets --locked --offline | exit 0 |
| cargo clippy --workspace --all-targets --locked --offline -- -D warnings | exit 0 |
| cargo test --workspace --locked --offline | exit 0; 33 passed, 0 failed |
| cargo build --workspace --locked --offline | exit 0; three native executable entries |
| Each native executable --help / --version | exit 0; correct role and truthful foundation-only status |
| full --check-config ABSOLUTE_TEST_CONFIG --runtime-root ABSOLUTE_SOURCE_ROOT, from another CWD | exit 0; explicitly reports model availability/runtime readiness unverified |

The test distribution was 5 diagnostic + 21 core + 7 protocol, including real Windows file-lock/rotation behavior and native path resolution. All commands above and the enclosing SSH command completed successfully. This v1 result predates independent review fixes and cannot be used as the final checked-source result.

## Harness failure and correction

The first launch used nested `powershell.exe` after the developer environment. It failed because `Get-FileHash` was unavailable in that inherited PowerShell 5/7 module environment, before any Cargo command ran. It was not a product test failure or a successful native gate.

A targeted read-only probe confirmed SSH's existing shell is PowerShell 7.6.6 at `C:\Program Files\PowerShell\7\pwsh.exe`. The second launch used that same installed executable, `-NoProfile -NonInteractive`, and passed the commands above. No global module path, profile, registry or software installation changed. PowerShell 7 is a session harness choice, not a product runtime requirement or a narrowing of Windows 10/11 support.

## Reviewed-source v2 result

The independent reviewer fixed per-frame ASR completion identity, unknown fields on internally tagged unit configuration variants, normalized empty URL userinfo, and lock-file budget handling. The stable reviewed source was transferred as v2 (SHA256 `43129cdafc8f2355b9f5ff9337c1c36052cf5ca689731a866e8e03c7f80288ea`), verified before extraction into a new `foundation-v2` directory, and tested with the same native harness using `-SourceName foundation-v2`. Cargo.lock and registry cache remained unchanged.

All five Cargo gates in the table above passed again with exit 0. The final suite passed **36 tests**, zero failures: 6 diagnostics, 23 core contracts and 7 protocol tests. All three native help/version checks and the explicit-runtime-root configuration check also passed. The enclosing SSH command exited 0. This v2 result is the checked C1 dependency delivery; C2-C7 runtime acceptance remains separate.

Windows 10 remains untested. C2-C7 must supply actual models/device inference, system-output capture, windows/tray/GUI, networking/authentication, integration, sustained runs, quality thresholds and release evidence separately. Temporary tool/source/cache/build directories contain only this task's development inputs/outputs and must not be confused with application audio/subtitle persistence.
