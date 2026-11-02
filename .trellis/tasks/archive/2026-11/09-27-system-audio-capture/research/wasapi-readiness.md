# Research: Native WASAPI loopback readiness

- Query: Identify a Rust-native, bounded system-output capture path for all approved Windows 10/11 x64 versions, including Windows 10 before 1703, and its conversion/device/lifecycle contracts.
- Scope: mixed; preparation only for C3. No capture implementation, remote command, installation, runtime experiment, dependency change, or task-state change was performed.
- Date: 2026-09-27

## Findings

### Authority and files found

| File | Relevant contract |
| --- | --- |
| `.trellis/workflow.md` | Task phases and project workflow; this research does not start C3 implementation. |
| `.trellis/spec/backend/index.md` | Backend guidelines currently contain bootstrap/template material; do not invent established audio conventions. |
| `.trellis/spec/guides/cross-layer-thinking-guide.md` | Shared contracts must be traced across producers and consumers. |
| `.trellis/tasks/09-27-system-audio-capture/prd.md` | Planning task; depends on checked C1, owns audio only, requires Windows 10/11 real-system evidence. |
| `.trellis/tasks/09-27-system-audio-capture/design.md` | WASAPI, normalization, selection/switching and activity diagnostics; parent design remains authority. |
| `.trellis/tasks/09-27-synesthesia-mvp/design.md` | Single render loopback; candidate 16 kHz mono PCM16; bounded callback transfer; default-follow/device-lock; generation boundary on switching; explicit failure rather than dropped audio. |
| `Cargo.toml`, `rust-toolchain.toml` | Workspace edition 2024 and Rust 1.90.0 baseline. |
| `crates/synesthesia-core/src/config.rs:66` | `CaptureConfig::FollowDefault` or `Device { device_id }`; no configurable default role. |
| `crates/synesthesia-core/src/lib.rs:10` | 16,000 Hz, maximum 3,200 samples/frame and 32,000 queued audio samples. |
| `crates/synesthesia-core/src/session.rs:192` | Validates nonempty normalized frames; overload enforcement belongs to shared session logic. |
| `crates/synesthesia-core/src/session.rs:283` | Ordered stop/cancel/clear/start effects and generation increment; adapter must complete old stop/cancel before new capture. |

Core source was inspected while C1 was still under implementation/check. These line references describe the research-time snapshot, not an assertion that C1 has passed its gate. Re-read the checked interface before C3 starts. No implement/check JSONL was read.

### Candidate dependency pins and OS boundary

Use direct `windows = "=0.62.2"` bindings behind a Windows target dependency and `rubato = "=0.16.2"` for software normalization. These are researched candidates, not added dependencies or a completed selection. Direct binding calls make old-Windows event restrictions, packet flags, COM ownership and endpoint notifications observable.

- Windows package source identity: [`windows` 0.62.2 package VCS record](https://docs.rs/crate/windows/0.62.2/source/.cargo_vcs_info.json), commit `32c3144490c016fe496a0aed769bce60987a2e9d`. Its [Cargo manifest](https://github.com/microsoft/windows-rs/blob/32c3144490c016fe496a0aed769bce60987a2e9d/crates/libs/windows/Cargo.toml#L4) sets MSRV 1.82.
- Verified [`windows` package API](https://crates.io/api/v1/crates/windows/0.62.2) checksum: `527fadee13e0c05939a6a05d5bd6eec6cd2e3dbd648b9f8e447c6518133d8580`.
- Rubato package source: commit `c20dfb7fa80fa43d0781312be67efe81f2607219`, [manifest](https://github.com/HEnquist/rubato/blob/c20dfb7fa80fa43d0781312be67efe81f2607219/Cargo.toml#L3) version 0.16.2, MSRV 1.61, MIT. The Sinc path can disable default FFT features if not used; resolve and lock transitive dependencies during implementation.
- The workspace Rust 1.90 baseline satisfies both declared MSRVs. These declarations do not prove a complete native build or runtime compatibility. Native `x86_64-pc-windows-msvc` compilation, linkage and older-OS execution still require evidence.
- The baseline below uses classic desktop `IMMDeviceEnumerator`, `IAudioClient`, `IAudioCaptureClient`, COM, events and ordinary waitable timers. These WASAPI APIs document Vista-era minimum clients, so their API introduction does not exclude early Windows 10. Avoid requiring `IAudioClient3`, process-specific loopback activation or high-resolution timer flags introduced later.

Binding feature candidate: `Win32_Foundation`, `Win32_Media_Audio`, `Win32_Media_KernelStreaming`, `Win32_Media_Multimedia`, `Win32_System_Com`, `Win32_System_Threading`, `Win32_UI_WindowsAndMessaging`. Add `Win32_UI_Shell_PropertiesSystem` and `Win32_System_Com_StructuredStorage` only if endpoint property display is implemented. The pinned [Audio source](https://github.com/microsoft/windows-rs/blob/32c3144490c016fe496a0aed769bce60987a2e9d/crates/libs/windows/src/Windows/Win32/Media/Audio/mod.rs#L2436) exposes capture methods; its notification implementation trait is at line 5787. `KSDATAFORMAT_SUBTYPE_PCM` and speaker masks are in KernelStreaming; [`KSDATAFORMAT_SUBTYPE_IEEE_FLOAT`](https://github.com/microsoft/windows-rs/blob/32c3144490c016fe496a0aed769bce60987a2e9d/crates/libs/windows/src/Windows/Win32/Media/Multimedia/mod.rs#L3062) is in Multimedia. Feature names are verified in the pinned manifest, but the minimal build has not been run.

### A baseline that covers pre-1703 Windows 10

**Primary-source fact:** [Loopback Recording](https://learn.microsoft.com/en-us/windows/win32/coreaudio/loopback-recording) specifies that event-driven loopback capture does not signal capture events before Windows 10 version 1703; it supports them starting at 1703. Loopback is shared-mode capture from a render endpoint and does not require a driver-provided Stereo Mix device. The dedicated page is more precise than the broader “Windows 10” language in [Initialize](https://learn.microsoft.com/en-us/windows/win32/api/audioclient/nf-audioclient-iaudioclient-initialize).

**Candidate baseline:** use shared loopback with timer polling on every approved OS. This avoids an OS-version branch and the pre-1703 no-event trap. It remains native WASAPI and is not microphone capture. Event mode can be a later tested optimization on 1703+; successful initialization alone cannot establish that events work.

1. Create a dedicated COM worker and initialize its apartment before creating the enumerator. Prefer a dedicated STA with a real message pump as the conservative candidate: Initialize documents a first-use STA warning for Windows 8, and STA COM dispatch requires pumping. This warning is not evidence that MTA fails on Windows 10/11. Keeping all capture COM creation/use/destruction on this worker avoids that unresolved assumption and interface transfer across threads.
2. `CoCreateInstance(MMDeviceEnumerator, CLSCTX_ALL)`; enumerate active `eRender` endpoints for selection. For follow mode, call `GetDefaultAudioEndpoint(eRender, eMultimedia)`. `eMultimedia` is a proposed consistent media role, not a newly approved setting; C1 currently has no role field. Use exactly that same flow/role in notifications. For lock mode, use `GetDevice` with the opaque saved ID and validate render flow/active state.
3. `IMMDevice::Activate<IAudioClient>`; obtain and preserve the exact `GetMixFormat` descriptor; validate that software decoding supports it. Initialize with `AUDCLNT_SHAREMODE_SHARED`, `AUDCLNT_STREAMFLAGS_LOOPBACK`, duration initially 100 ms as an experiment parameter, periodicity 0, and the endpoint mix format. No `EVENTCALLBACK` flag in this baseline. Read actual `GetBufferSize` and `GetDevicePeriod`; requested duration is not proof of actual capacity.
4. Get `IAudioCaptureClient` through `GetService`, then `Start`. Wait interruptibly on stop/control plus an ordinary timer and pump STA messages with `MsgWaitForMultipleObjectsEx`, `PeekMessageW` and dispatch. `CreateWaitableTimerW`/`SetWaitableTimer` avoids newer timer flags. Start with a polling interval safely below half the actual buffer duration, e.g. 5–10 ms when the measured buffer permits it; measure scheduler delays and overrun instead of assuming a latency guarantee.
5. At each wake, repeatedly call `GetNextPacketSize` and `GetBuffer` until empty, copy bounded packet metadata/data, then `ReleaseBuffer` for the complete packet. GetBuffer/ReleaseBuffer must be paired on the same thread. Packets are variable sized; no fixed engine-period packet-size assumption.

The [official capture example](https://learn.microsoft.com/en-us/windows/win32/coreaudio/capturing-a-stream) demonstrates draining packets and timer-based polling. Its sleeps/example duration are teaching code, not this application's latency policy. The selected worker must be stoppable during idle capture and have no GUI-thread COM or DSP work.

For an optional 1703+ event route, initialize shared loopback with `LOOPBACK | EVENTCALLBACK`, duration and periodicity both zero, create an event and call `SetEventHandle` before Start. Wait on event plus stop/control with a finite health watchdog. Recreate the audio client when changing initialization mode; each client is initialized once. The documented workaround of a separate event-driven render stream waking legacy loopback capture adds another stream/resource lifecycle; it is unnecessary for the timer baseline and has not been selected.

### Native endpoint selection and restart boundaries

[GetDefaultAudioEndpoint](https://learn.microsoft.com/en-us/windows/win32/api/mmdeviceapi/nf-mmdeviceapi-immdeviceenumerator-getdefaultaudioendpoint) returns the current endpoint for a flow/role. An opened loopback stream does not itself implement the product's follow-default policy.

- Register a custom `IMMNotificationClient` before selecting/opening, then re-query after opening to catch a racing default change. Its `OnDefaultDeviceChanged` filters the selected render role; `OnDeviceStateChanged`, removal and relevant format/property changes cause a control-thread recheck. Addition can refresh the selectable list. Coalesce notifications into a bounded dirty/requery signal rather than opening COM clients inside callbacks.
- Follow-default: if the actual selected endpoint changes or its stream becomes invalid, stop old capture, cancel the old input generation, clear queued/raw/resampler state and start a fresh client for the re-queried default. Deduplicate repeated notifications by actual endpoint identity/state. Missing default is an explicit unavailable condition; no microphone fallback.
- Device-lock: changes to other/default endpoints do not switch the selected device. Removal/disable/invalidation of the selected endpoint stops that input with a concrete diagnostic; never pick a different endpoint silently. Explicit user retry/reselection can establish a new generation.
- IDs are opaque UTF-16 strings. Copy owned values when necessary; do not parse IDs or use device display names as stable identities. Callback pointers cannot be retained beyond the callback.
- [Notification callback rules](https://learn.microsoft.com/en-us/windows/win32/api/mmdeviceapi/nn-mmdeviceapi-immnotificationclient) require nonblocking callbacks, no registration/unregistration within callbacks, and no final MMDevice-object release there. A callback should only set atomic state/try a bounded control signal; no waits, GUI locks, DSP, file writes or endpoint activation.
- The enumerator [does not AddRef/Release the registered sink](https://learn.microsoft.com/en-us/windows/win32/api/mmdeviceapi/nf-mmdeviceapi-immdeviceenumerator-unregisterendpointnotificationcallback). Retain a strong sink reference through unregister and callback quiescence. Do not drop shared state or hold callback-needed locks while unregistering.

C1's `switch_input` effects require an actual completion barrier: after `StopCapture`, no frame from the old capture worker may be published; after cancellation/queue-clear, no old normalized tail or result may leak into the new generation. New capture gets a new normalizer and counters. Adapter acknowledgement semantics should be checked against final C1 before implementation. Changing endpoints does not unload C2 models.

### Actual format conversion to candidate 16 kHz mono PCM16

[GetMixFormat](https://learn.microsoft.com/en-us/windows/win32/api/audioclient/nf-audioclient-iaudioclient-getmixformat) returns task-allocated WAVEFORMATEX/EXTENSIBLE data accepted by the same endpoint's shared-mode Initialize. It can be float despite integer hardware output. Copy descriptor fields and extension before freeing with `CoTaskMemFree`; do not assume PCM16 or request 16 kHz then merely relabel native bytes. If probing another format with `IsFormatSupported`, inspect the closest-match pointer/status rather than equating every successful HRESULT with an exact match.

The pinned bindings mark [WAVEFORMATEX and WAVEFORMATEXTENSIBLE](https://github.com/microsoft/windows-rs/blob/32c3144490c016fe496a0aed769bce60987a2e9d/crates/libs/windows/src/Windows/Win32/Media/Audio/mod.rs#L9044) `repr(C, packed(1))`. Read by value/unaligned operations inside a small reviewed unsafe boundary; do not form aligned references to packed fields or cast arbitrary packet bytes to an aligned Rust sample slice.

| Native descriptor | Required software decode |
| --- | --- |
| PCM16, tag PCM | Little-endian signed i16, normalize by 32768. |
| PCM24 packed | Read three bytes, sign-extend signed 24-bit value, normalize by 2^23; not an i32 byte cast. |
| PCM32 with 24 valid bits in EXTENSIBLE | Read signed LE container, account for left-aligned valid bits by shifting unused low bits, normalize by 2^23. Full 32-bit PCM can use its own valid-bit scale. |
| IEEE float32, plain tag or EXTENSIBLE float subformat | Decode LE f32; reject non-finite samples explicitly, preserve finite values until the final clipping/quantization step. |
| EXTENSIBLE | Require sufficient extension (`cbSize >= 22`), recognized PCM/float GUID, legal container/valid-bit sizes, validated block alignment and rate/channels; inspect channel mask. |

Reject unsupported/compressed formats or inconsistent bounds with an explicit format error; no misinterpretation as PCM. Implementation must set numeric maximum rate/channels/bytes per packet before allocations. The [format descriptor specification](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/ksmedia/ns-ksmedia-waveformatextensible) distinguishes container bits from left-aligned valid bits and orders channel positions by set mask bits. Mask/channel counts can legally differ or be zero/direct-out; those cases require explicit handling, not a presumed stereo layout.

**Proposed deterministic downmix policy:** mono passes through; ordinary stereo uses `(L+R)/2`. For recognized multichannel speaker masks, order channels by mask bits, exclude LFE and average equally across the assigned non-LFE channels. This simple speech-oriented policy retains a center channel and normalizes total weight; it is a software choice, not a Microsoft-mandated matrix. Handle extra/unassigned channels deliberately: conservatively reject ambiguous multichannel direct-out/mismatched layouts until a documented policy is implemented. For a two-channel nonstandard EXTENSIBLE mask, use the mapped-channel policy instead of assuming L/R. Validate 5.1/7.1 center speech and antiphase stereo behavior; equal averaging can cancel antiphase sources and is not an audio enhancement algorithm.

**Proposed resampling path:** decode/downmix to one f32 channel on a separate normalizer worker; keep a bounded partial-input FIFO; feed `SincFixedIn<f32>::new(16000.0 / native_rate, 1.0, parameters, chunk_size, 1)` with input chunks requested by `input_frames_next`. Use a band-limited sinc filter (source suggests initial sinc length 256 and cutoff 0.95) and preallocated `process_into_buffer` outputs. Record actual returned input/output counts, not independent per-packet rounding. At native 16 kHz, bypass the resampler but keep format/downmix/quantization validation.

Pinned Rubato code: [constructor/parameters](https://github.com/HEnquist/rubato/blob/c20dfb7fa80fa43d0781312be67efe81f2607219/src/asynchro_sinc.rs#L256), [buffer processing contract](https://github.com/HEnquist/rubato/blob/c20dfb7fa80fa43d0781312be67efe81f2607219/src/lib.rs#L127), [output delay/reset](https://github.com/HEnquist/rubato/blob/c20dfb7fa80fa43d0781312be67efe81f2607219/src/asynchro_sinc.rs#L509). Preserve fractional/filter state across packets, including SILENT zero packets. Account for initial `output_delay` in the normalization/timestamp policy and measured expected lengths; otherwise synthetic leading latency becomes audio time. Stop/switch discards old filter/FIFO tail instead of flushing it into new input. Partial processing/end-padding is normalizer work and needs an explicit tail-trimming policy; never claim padded samples were captured audio.

Finally quantize each finite mono output: clamp to [-1,1], round `x*32768`, clamp integer to [-32768,32767], encode PCM16 LE at the wire boundary. Emit nonempty frames <=3,200 samples, with session/generation, sequence, start_sample and sample_count from cumulative **emitted normalized samples**, reset per input generation. Measure 44.1→16 and 48→16 long-run sample counts/filter delay; resetting/resizing the resampler at every WASAPI packet would create drift and boundary artifacts.

### Bounded capture transfer and packet conditions

WASAPI supplies borrowed packet memory. [GetBuffer](https://learn.microsoft.com/en-us/windows/win32/api/audioclient/nf-audioclient-iaudiocaptureclient-getbuffer) and [ReleaseBuffer](https://learn.microsoft.com/en-us/windows/win32/api/audioclient/nf-audioclient-iaudiocaptureclient-releasebuffer) delimit its lifetime; release complete packets promptly. Use checked `frames * nBlockAlign` and an owned preallocated packet pool. Capture worker only copies bytes/flags/positions and tries bounded transfer; normalizer handles decode/downmix/resample/frame formation. No unbounded channel or allocation-per-notification design.

Bound both raw packet count and raw bytes/audio duration, then bound normalization FIFO/output and account for C1's 32,000 queued samples (2 s at 16 kHz). C1's generic bounded queue is not a synchronized real-time channel. Layering independent 2 s queues creates additional latency; choose an explicit total budget instead. Queue/pool exhaustion signals fatal overload through a reserved control path/atomic flag, releases any held WASAPI buffer, and ends the input. Releasing discarded data during an explicit terminal failure is distinct from silently dropping old audio and continuing.

| Condition | Required interpretation/boundary |
| --- | --- |
| `AUDCLNT_BUFFERFLAGS_SILENT` | Actual packet frames represent silence; supply exactly that many zeros without dereferencing meaningless packet data. Feed them through persistent resampling state. |
| Empty `GetNextPacketSize` / `AUDCLNT_S_BUFFER_EMPTY` | No packet available; no captured samples to append. Do not inspect undefined out pointers or manufacture zeros from elapsed wall time. |
| `DATA_DISCONTINUITY` after established capture | Explicit data/timeline break; stop/cancel/clear and establish a new generation or fail if recovery is unavailable. Do not stitch unknown missing samples or hide with zero fill. Initial packet is already a fresh boundary; test initial flags separately from later data loss. |
| `TIMESTAMP_ERROR` | Device/QPC positions may be invalid; mark numeric diagnostic and do not use unreliable timestamps to infer gap length. Actual sample counts remain the data basis; require explicit policy/test for continued capture. |
| `AUDCLNT_E_DEVICE_INVALIDATED` / resources invalidated | Stop using that stream; re-query/recreate according to follow/lock policy. |
| Audio service stopped, unsupported format, no endpoint | Explicit failure and cleanup, with stable error category plus numeric backend code; no microphone substitution. |

Flag meanings are primary-source evidence from [AUDCLNT_BUFFERFLAGS](https://learn.microsoft.com/en-us/windows/win32/api/audioclient/ne-audioclient-_audclnt_bufferflags); restart/failure responses above are proposed mappings to the approved generation/continuity contract. Do not expose unrestricted HRESULT messages/device names or audio content in diagnostics.

**Critical C2/C3 integration gap:** when output stops, loopback may provide no packets rather than SILENT packets. Absence alone cannot distinguish legitimate idle output from a stalled stream. ASR silence-finalization needs an explicit idle/control clock policy that respects sample-based subtitle time and does not quietly count wall-time gaps as captured audio. First experiment must observe this behavior, device/QPC positions and event flags across play/pause/resume before selecting a silence-clock adapter. This research does not invent protocol fields or weaken continuity to resolve it.

### COM, thread and cleanup obligations

[CoInitializeEx](https://learn.microsoft.com/en-us/windows/win32/api/combaseapi/nf-combaseapi-coinitializeex) is per-thread; both S_OK and S_FALSE require a matching CoUninitialize. Failed `RPC_E_CHANGED_MODE` does not grant a successful initialization to uninitialize. The candidate dedicated STA must pump messages; no COM interface or raw packet pointer crosses to the normalizer. Do not add unsafe Send/Sync impls merely to move capture clients into a runtime task.

Stop/control handling must release an outstanding packet on its capture thread, stop the audio client, unregister the notification sink outside callbacks, wait for callback publication to quiesce, destroy capture/client/device/enumerator references on their owning worker, free copied mix-format/GetId task allocations, close timer/event/stop handles after waits finish, then CoUninitialize last. An invalidated stream can make Stop fail; cleanup must continue through all remaining resources rather than return early. If MMCSS registration is added with AvSetMmThreadCharacteristics, pair it with AvRevertMmThreadCharacteristics. Only acknowledge StopCapture once capture and normalization publication have stopped and old generation state has been cleared. Join workers; no detached thread retains an endpoint or writes late frames after shutdown.

### Smallest meaningful native experiment, after checked C1

Required assets/toolchain: checked C1, native Rust 1.90.0 MSVC target, working MSVC linker/Windows SDK, locked candidate Rust dependencies, Windows 10/11 x64 with an active render endpoint and a deterministic local playback fixture. No model, GPU, Docker, microphone or Internet runtime service is needed. Root's environment probe found Visual Studio/MSVC and bundled CMake/Ninja but no standard Rust installation; availability of a complete Rust/SDK build remains a prerequisite, not a reason to install from this sub-agent.

The first smoke program should use the timer baseline and the actual normalization worker, print only safe numeric format/frame/flag/queue diagnostics, and retain samples only in memory for assertions. Play a known finite stereo fixture at the endpoint's native 48 kHz or 44.1 kHz mix format; verify nonzero captured activity, correct decoding/downmix, 16 kHz mono PCM16 frame bounds, expected sample counts within documented filter/timing tolerance and monotonically increasing counters. Then pause to observe no-packet versus SILENT behavior, resume, and stop while idle. Assert all workers finish and restart produces a fresh generation. Endpoint enumeration or merely opening a stream is not a meaningful capture/normalization pass.

Follow with independent focused checks:

1. Synthetic in-memory PCM16/packed PCM24/24-in-32/float32/EXTENSIBLE descriptors, masks, block alignment, invalid GUID/bounds and non-finite float inputs. Known mono/stereo/center-only 5.1 patterns; anti-alias attenuation and output-count tests at 44.1/48/96 kHz; packet partition invariance.
2. Real early Windows 10 (e.g. build 10240 or 14393) timer capture and idle stop; modern Windows 10 and Windows 11. Running only current Windows 11 does not prove pre-1703 support. Optional event route must be tested independently on 1703+ and never required on older Windows.
3. Default render switch, unplug/disable/re-enable, locked device removal, invalidated format change, duplicate/racing notifications and playback ending during switch. Prove old-generation frames cannot leak and lock mode never silently follows default.
4. Slow normalizer/consumer and forced bounded-pool exhaustion; establish explicit overload failure rather than drop-and-continue. Multi-second scheduler stall/discontinuity, service restart and suspend/resume as available; do not write unknown gaps as silence.
5. Repeated start/stop and failure at each initialization stage, checking worker/handle/COM allocation cleanup. Stereo 44.1/48 kHz plus a real multichannel endpoint if claimed supported. Diagnostics contain no captured audio/text or unrestricted endpoint payload.

## Caveats / Not Found

- C3 is still planning; C1 must pass its check before dependent implementation. No test above was executed, no native source was built, and no compatibility claim is earned by these references alone.
- Default-role selection (`eMultimedia` candidate), raw/total queue budgets, resampler delay/tail treatment, multichannel ambiguity policy, timestamp-error continuation and idle/silence signaling need explicit implementation decisions and tests against checked shared contracts. Most are technical details within approved scope; idle signaling may expose a cross-task contract gap requiring coordination.
- Microsoft Learn references are primary docs inspected on the research date, not immutable revisions. Candidate binding/DSP source links use fixed commits. Crate feature availability/source signatures do not establish complete dependency resolution, linkage or OS runtime behavior.
- PCM24 decoding must exist even if the tested Windows mix format is float32; one endpoint does not cover all claimed formats. Legal descriptor layouts outside a documented downmix policy should fail explicitly until supported, rather than be guessed.
- Protected-output/driver/session differences can affect loopback availability. The stream is the selected render endpoint's mixed output; this approach does not promise process-isolated capture or capture from all simultaneous render endpoints.
- No microphone, substitute audio source, substitute model, GPU-policy change, or model lifecycle implementation is proposed here.
