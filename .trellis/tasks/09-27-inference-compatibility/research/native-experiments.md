# Main-session native inference experiments

Windows11 build26100, existing RX9070XT and private Rust1.90/MSVC/portable SDK;
no Docker/WSL/global installation. User's two model files remain read-only.
No engine audio/text/prompt/raw-log persistence: helpers use framed bounded pipes,
stderr is OS-null, examples print only numerical/safe categorical evidence.
Controlled public fixture files are development inputs, not session recordings.

## Native build progression

Pinned source archive SHA256: CrispASR
`15d21060ffd95bfe3333f2e60775b6114d2659294a5b65138a1956af019fc298`,
its ggml submodule `8e06f263fffea53b169c7839b3b4456451e0b0a3d24d2732a63daa0d5a42810e`,
llama.cpp `a6861d549427f814dc591c439e08206f67ffaba0248344d421589abf18199e67`.
All were checked on destination before extraction. CMake mutates only pinned
task work copies through guarded exact patches; neither user models nor source
repositories outside task-owned directories are changed.

- helper v1 SHA `b27c85c06d8e1405d5f801db94348c3d0a661af24755c03ce358b086dc154da4`:
  configure failed upstream CMAKE_SOURCE_DIR assumed its own project root.
  Worker replaced these paths with an explicit pinned CrispASR root.
- helper v2 SHA `ff6e259d3f5c84e69e9eaff3bd46829b73c70bb935a9872c6bba91846372a4c7`:
  configure passed; long task path caused shader ExternalProject MSVC C1041
  beyond250-character object path. Creating task-only short build root
  `C:\Users\wkyuu\AppData\Local\Temp\sb-e3f0e640` fixed this without registry
  changes. ASR fully built; translation reached final link then failed common.lib.
  Worker corrected the pinned upstream target name to llama-common.
- helper v3 SHA `40b274c1874b6ebb0d0f4d7127e162f7866162461dfa62bfd8d64ac307a3ded2`:
  both native Vulkan helpers compiled/linked successfully. Includes fail-closed
  Vulkan identity, disabled ASR weight CPU spill/audio-stage dumping and actual
  Hy matrix-buffer residency check. Later scratch-context recreation and generic
  CPU ISA flags are still required in final source; v3 is preliminary evidence.

MSVC19.50.35718, CMake4.1.1, Ninja1.12.1, portable Vulkan1.4.309.
All builds used Release/static backends, CUDA/Metal/RPC/OpenMP off, parallel8.
Compiler shader capability tests passed KHR cooperative matrix/NV cooperative
matrix2/integer-dot; unsupported newer shader features were disabled by upstream.
Build warnings are not reported as clean Rust lint or model execution evidence.

## Preliminary v3 model/lifecycle smoke

Rust adapter snapshot adapters-v4 SHA
`6577912e24b8444e809e90077f84e8c7bb142981f7bdf7034d81019ea3486f21`
passed native check/clippy(all-targets,-Dwarnings), tests and examples build:
6inference+16audio tests. No model ran during those source checks.

`native_smoke CONFIG RUNTIME_ROOT ASR_HELPER TRANSLATION_HELPER EN_PCM JA_PCM`
then ran native helpers from the short build root with explicit Vulkan identity
vendor4098/device30032/nameAMD Radeon RX9070XT (actual config includes spaces
as the selected device description), and the exact two approved model paths.
The adapter checks published model SHA256 before load and validates actual ready
backend identity. No otherGPU/CPU fallback is accepted.

Each smoke PCM is240000samples/15seconds: initial1second zero, one intact
3..6second source clip, then zero tail; no reference sentence clipping/repetition.
English clip6930-75918-0000 SHA
`4dd8f4689ab486670409e69625725c3e277d14e46d6ab23a0b58a0bda3bf710e`;
Japanese BASIC5000_0001 SHA
`d78ab2eef4a0d894d7a83d0fde80e467bec693c4c999d9acdc530a5edf682fb3`.
These short slices do not stand for the frozen600second benchmarks.

Actual result, enclosing SSH exit0:

| Observation | Result |
| --- | --- |
| detect/hash/load both requested models | 61050ms |
| cancel/idle probes | 1504ms; ASR graphs0, translation graphs1 |
| retained ownership | same PIDs, model_loads1, idle graph counts stable |
| English paced15s | 3drafts,1final,1translation;15006ms total |
| Japanese paced15s | 3drafts,1final,1translation;15011ms total |
| shutdown | owned_processes0 |

ASR graph count0 does not prove cancellation during encoder/prefill/decode;
stronger actual-work phase evidence is required. Graph count1 for translation
is separate. No text quality/Chinese quality assessment or latency percentile
is inferred from counts. Scratch-context clearing, final genericCPU/source
revalidation, negativeGPU/manualCPU tests, phase cancellation,600second
benchmarks/precise latency annotations and D-10 review remain outstanding.
