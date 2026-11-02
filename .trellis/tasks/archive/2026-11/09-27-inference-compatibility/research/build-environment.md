# Native Windows build environment

Read-only preparation on 2026-09-27 for the approved Windows 10/11 x64 implementation. These observations do not prove model loading, Vulkan inference, Windows 10 compatibility, or native application compilation.

## Observed tools

- Target: the user-provided `wkyuu@192.168.9.2`; Windows 11 baseline is recorded in the parent task's hardware report.
- Ordinary SSH PowerShell application lookup found Git at `d:\bin\git\bin\git.exe`. It did not find `cargo`, `rustc`, `rustup`, `cmake`, `ninja`, `cl`, `crispasr`, `llama-server`, or `llama-cli` in PATH.
- `vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64` located Visual Studio Community 2026, installation version `18.0.11217.181`, at `C:\Program Files\Microsoft Visual Studio\18\Community`.
- The installed MSVC directory is `14.50.35717`; its `bin\Hostx64\x64\cl.exe` exists. `VC\Auxiliary\Build\vcvars64.bat` exists. These are file-presence observations, not a completed compiler smoke build.
- Bundled CMake exists at `Common7\IDE\CommonExtensions\Microsoft\CMake\CMake\bin\cmake.exe`; invoking it returned `cmake version 4.1.1-msvc1`.
- Bundled Ninja exists at `Common7\IDE\CommonExtensions\Microsoft\CMake\Ninja\ninja.exe`; invoking it returned `1.12.1`.
- `C:\Users\wkyuu\.cargo\bin\cargo.exe` and `rustc.exe` do not exist. This does not exclude Rust at another location.
- No immediate SDK directories were found under `C:\VulkanSDK`, and process `VULKAN_SDK` is unset. This does not exclude headers/tools elsewhere. Vulkan loader/device detection is separately documented in the parent hardware baseline.
- The account temporary directory is `C:\Users\wkyuu\AppData\Local\Temp\`; its C: drive reported `178657439744` free bytes during the probe.

## Experiment boundaries

Native C++ experiments should first use the installed developer environment and bundled CMake/Ninja. Native Rust compilation still needs an explicit toolchain preparation step. Prefer isolated task-owned temporary tools over global PATH/profile changes; do not infer a need to reinstall Visual Studio from the ordinary PATH lookup.

No tool installation, upload, model execution, remote file edit, global environment change, or GPU inference was performed by these probes. Existing model files remain untouched. Actual SDK/header needs and engine lifecycle compatibility belong to the focused [engine readiness investigation](native-engine-readiness.md).

## Subsequent compiler smoke experiment

After the read-only probes, the main implementation session created the task-owned directory `C:\Users\wkyuu\AppData\Local\Temp\synesthesia-native-20260927-20e02729a4304dc5a3f40decf6f4e0af`. It wrote only `compiler-probe.c` containing `int main(void) { return 0; }`, then called the discovered `vcvars64.bat` in a child `cmd.exe`, compiled/linked with `cl /nologo ... /Fe:compiler-probe.exe`, and ran the result. The SSH command exited 0 and the executable returned 0. The compiler created its source/object/executable only inside that new directory.

This confirms a working native C compiler/linker environment on the Windows 11 baseline, independently of ordinary PATH. It does not compile Rust, either engine, Vulkan shaders, or product code. No global environment, Visual Studio installation, models, or pre-existing user files changed. The directory is reserved for this session's isolated native build tools/work copies; eventual cleanup must target only its known created files and preserve all other temporary directories.

Linux development uses the approved temporary Docker wrapper independently of this machine. Windows build, installation, execution, and acceptance retain the approved native requirement.
