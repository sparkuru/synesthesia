# Task-local Vulkan compiler preparation

Date: 2026-09-27. Main-session-owned preparation, not model/GPU acceptance.

The fixed llama.cpp Vulkan CMake contract requires Vulkan headers/import library,
glslc and SPIRV-Headers CMake configuration. Existing Visual Studio supplies
MSVC/CMake/Ninja, but no standard Vulkan SDK was found.

Acquired the official [SDK 1.4.309.0 Windows archive](https://sdk.lunarg.com/sdk/download/1.4.309.0/windows/vulkan_sdk.exe).
The [official hash API](https://sdk.lunarg.com/sdk/sha/1.4.309.0/windows/VulkanSDK-1.4.309.0-Installer.exe.json)
identifies SHA256 `48b132169b64fe65cdb0f20970195335a65354e73f1ea5373032c2a8bbad4297`,
which matched the complete 200187952-byte download before extraction.
The installer was never executed.

7z initially recognized the outer PE rather than embedded archive. A bounded
local scan found the valid 7z signature at byte 24663576, checked its start-header
CRC, and calculated its complete extent from next-header offset/size:
175513633 bytes. Extracting that byte range allowed normal 7z inspection and
selected extraction of Include/*, Bin/glslc.exe, Lib/vulkan-1.lib and
Lib/cmake/*. No driver/layer/runtime installation or registry/PATH/profile
change occurred. Build components total approximately 32 MiB, rather than the
full SDK payload; the selected library is an import library for the existing
system Vulkan loader, not a replacement driver.

The selected transfer tar.gz SHA256 is
`ad5481a67909bbb6b05122a99749c5686288deab14a6791a2bb5d6f96b95d456`.
It is unpacked only after destination hash verification into the main session's
private Windows workspace child `vulkan-sdk-1.4.309.0`. Build processes can use
process-local VULKAN_SDK/CMAKE_PREFIX_PATH and Bin paths. Pinning this older,
extractable compiler does not imply current driver/API validation; exact source
build and shader feature checks must establish compatibility. Tools are not
application release assets; release provenance/licensing belongs to C7.

Native glslc --version ran with exit 0 on the Windows baseline: shaderc
v2023.8/v2025.1, spirv-tools v2025.1/v2022.4-695-gf289d047, glslang
11.1.0-1194-g8b822ee8. This is compiler readiness only; no engine shader/model
execution or GPU acceptance is claimed. Engine builds are recorded separately.
