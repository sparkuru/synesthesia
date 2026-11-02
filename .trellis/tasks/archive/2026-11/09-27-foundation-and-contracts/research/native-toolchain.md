# Isolated native Windows toolchain preparation

On 2026-09-27 the main implementation session prepared Rust 1.90.0 for native `x86_64-pc-windows-msvc` validation on the approved Windows 11 baseline. This is tool readiness, not product build or Windows 10 acceptance.

Official component URLs/hashes were read from [the fixed Rust distribution manifest](https://static.rust-lang.org/dist/channel-rust-1.90.0.toml), dated 2025-09-18. Each complete archive was checked against its manifest SHA256 before extraction:

| Component | SHA256 of Windows x64 `.tar.xz` archive |
| --- | --- |
| rustc | `a18b3912905f154c34a7572954dda9e9849ac6411a1e02fa390bf742f918fbcb` |
| cargo | `a238d6a87cb0935e4f1bf0b4d093cbac3833d2f8b04413417bb622779669bec5` |
| rust-std | `3b8c22f81c93c7e914369433430792bd6c06403806ae25b0b95f3cfc540a8b48` |
| clippy | `d7d71f603ea9c2f79ba186cd3572ea41593907f933d26bd028d60b7b0943b49e` |
| rustfmt | `5412f36b61f928ef2e27d722418dc88ef09297d338017536f8697ea43077873b` |

The rustc full download and one resume timed out at 180 seconds each. Eight range requests completed six ranges; the other two timed out at 120 seconds with partial data. Two short missing-tail requests then completed. Each completed range's HTTP Content-Range and byte length were checked; concatenating the ordered prefix/ranges produced the exact official full-archive SHA256 above. Partial/unverified bytes were never executed. This is recorded rather than treating the failed downloads as successful checks.

Component contents were extracted into a dedicated local `/tmp/synesthesia-native-rust-1.90.2ieuWZ/prefix`, without running bundled installer scripts. The resulting transfer archive `rust-native-tools.tar.gz` has SHA256 `f272f0fc45311f788e6dbe9f930964be28f882f4f53834dec1f1539542314905`. It was uploaded into this session's new Windows temporary workspace and checked again before extraction:

`C:\Users\wkyuu\AppData\Local\Temp\synesthesia-native-20260927-20e02729a4304dc5a3f40decf6f4e0af\rust`

Direct native version probes all exited 0:

- `rustc 1.90.0 (1159e78c4 2025-09-14)`, full commit `1159e78c4747b02ef996e55082b704c09b970588`, host `x86_64-pc-windows-msvc`, LLVM 20.1.8.
- `cargo 1.90.0 (840b83a10 2025-07-30)`.
- `rustfmt 1.8.0-stable (1159e78c47 2025-09-14)`.
- `clippy 0.1.90 (1159e78c47 2025-09-14)`.

The existing Visual Studio developer environment separately compiled/linked/ran a trivial C program, documented in [C2 build environment](../../09-27-inference-compatibility/research/build-environment.md). Subsequent Cargo commands must use this private tool prefix and process-local developer environment/cache, not install Rustup or change global PATH/profiles/registry. Windows does not need Docker or WSL.

No product source, model, GPU inference, capture or GUI was executed by these version probes. Models and existing user files were untouched. The temporary directory remains owned by this implementation session for native work copies; cleanup must target only known task-created artifacts when no further native work depends on them. Tools are development preparation, not application release payloads.
