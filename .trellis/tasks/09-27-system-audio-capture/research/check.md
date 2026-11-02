# Independent C3 review

Date: 2026-09-27. Scope: all of `crates/synesthesia-audio`, including the native
WASAPI implementation and controlled diagnostic renderer. Reviewed against C3
PRD/design/implement, its check-context references, current backend/frontend
indexes and source-backed quality/core/logging guidelines. Package discovery
reports one repository with backend, frontend and trellis-plus spec layers.
The C1 foundation was not reopened; shared interfaces were checked as consumers.

## Findings fixed

1. `src/native.rs`: `CaptureHandle.device.is_default` described the selection mode,
   so a locked endpoint that was actually default reported false. It now compares
   the selected opaque ID with the queried current eRender/eMultimedia default.
   The diagnostic prints only numeric `selected_default`, never the ID. Locked
   selection remains strict and is not replaced by this metadata query.
2. `src/native.rs`, `src/capture.rs`: the capture worker retained the normalizer in
   a plain JoinHandle. Unwinding could detach it before the capture worker's join
   completed. `OwnedNormalizer` now stops and joins it on both explicit finish and
   Drop. Its terminal raw-frame count is cleared only after joining. An ownership
   regression deliberately unwinds the owner and verifies normalizer quiescence,
   no late publication, and explicit WorkerFailed.
3. A normalizer panic previously supplied no immediate failure signal; an idle
   capture could appear healthy until more packets or a later explicit stop.
   `WorkerCompletion` records WorkerFailed on unwind and marks termination after
   the worker body/resources finish. Both capture and normalization use it. A
   focused panic test requires failure without subsequent audio and idempotent
   joining. These are deterministic worker-ownership tests, not fake COM evidence.
4. `src/native.rs`: notification-unregister and capture-client Stop failures were
   discarded. Cleanup still continues, but the first safe kind/numeric HRESULT is
   now stored through Shared::fail. Success cannot hide these cleanup failures.
   The callback reference remains strongly held through unregister; callbacks
   access only an Arc and atomics and do not unregister, wait, activate devices or
   retain caller-owned pointers. The diagnostic renderer retains its separate
   finite ownership path and never changes system default/volume.

Files changed by this reviewer: `src/capture.rs`, `src/native.rs`,
`examples/audio-diagnostic.rs`, and this report. No manifest/lock, root/shared
source, other task, lifecycle, spec or protected runtime/platform edit was made.

## Complete-scope checks

- Dedicated STA initialization is paired with CoUninitialize only after successful
  initialization. Capture/client/device/enumerator references, task allocations,
  packet GetBuffer/ReleaseBuffer and notification teardown stay on the owning
  thread. The normalizer receives owned bounded bytes, not COM interfaces or raw
  pointers. No unsafe Send/Sync implementation was introduced.
- Active render endpoints only: follow-default rechecks actual selection/format;
  locked selection uses the supplied ID and validates render flow/active state.
  Invalidation/format/default change fails the old input explicitly; the caller
  joins/clears before creating a new generation. There is no microphone fallback.
- Empty packets do not create samples or inspect undefined data. SILENT packets
  create exactly their actual frame count of zeros. Noninitial discontinuity ends
  the input; timestamp errors are counted without inferring missing samples.
- WAVEFORMAT bounds/GUID, container/valid bits, alignment/byte rate and channel
  mask are validated at the shared pure decoder. PCM16, packed24, 24-in-32/full32
  and finite float32 follow documented semantics. Ambiguous multichannel/direct
  out and LFE-only masks fail; assigned non-LFE channels are equally averaged.
- Stateful bandlimited conversion preserves filter/fractional state across
  packets. Initial output delay is excluded; idle tail padding owns no new audio
  time and emitted count is floor(total actual frames *16000/native rate). Resume
  preserves global fractional counts; stop/switch discards pending tails.
  Tests cover 8/16/44.1/48/96/192 kHz counts, partition invariance, attenuation,
  clipping, SILENT/absence, descriptor failures and fresh-generation identity.
- Raw pool/count/time, single decoded packet, FIFO/filter, output, core queue and
  ASR in-flight bounds are accounted explicitly. The conservative pending-audio
  bound is 2.848 s (200+200+32+16+200+2000+200 ms), not a release-latency promise.
  The terminal atomic error is independent of PCM queue saturation; full queues
  do not drop middle audio and continue. Stop joins both workers and clears output.
- Controlled playback uses the exact selected endpoint and synthesizes 697/941 Hz
  only in memory. Pause is renderer Stop+Reset; resume follows old-capture join.
  Frequency projections, sample/queue/format/counter outputs are numeric-only.
  Ambient output is not disabled to force no-packet success; unavailable no-packet
  conditions remain explicitly unavailable. No audio recording/file/cache/log or
  arbitrary endpoint/foreign error output is added.

## Primary-source verification

Microsoft's [GetBuffer documentation](https://learn.microsoft.com/en-us/windows/win32/api/audioclient/nf-audioclient-iaudiocaptureclient-getbuffer)
permits omitting ReleaseBuffer for zero-size packets; nonzero packets require
paired release on the same thread. Thus the empty-packet early exit is intentional.
Its [notification registration documentation](https://learn.microsoft.com/en-us/windows/win32/api/mmdeviceapi/nf-mmdeviceapi-immdeviceenumerator-registerendpointnotificationcallback)
and [unregistration documentation](https://learn.microsoft.com/en-us/windows/win32/api/mmdeviceapi/nf-mmdeviceapi-immdeviceenumerator-unregisterendpointnotificationcallback)
require caller-owned callback lifetime through unregister; it does not acquire
and release that reference for the client. These were rechecked during review.
Bindings remain pinned to windows/windows-core0.62.2; DSP to rubato0.16.2.

## Verification

All final focused Linux commands below exited 0 with cached Rust1.90.0, Cargo.lock,
`HAKO_NETWORK=none` and Cargo offline mode where applicable:

```text
HAKO_NETWORK=none ./hako cargo fmt -p synesthesia-audio -- --check
HAKO_NETWORK=none ./hako cargo check -p synesthesia-audio --all-targets --locked --offline
HAKO_NETWORK=none ./hako cargo clippy -p synesthesia-audio --all-targets --locked --offline -- -D warnings
HAKO_NETWORK=none ./hako cargo test -p synesthesia-audio --locked --offline
HAKO_NETWORK=none ./hako cargo build -p synesthesia-audio --examples --locked --offline
```

**18 tests pass**, zero fail: six capture/ownership tests plus twelve normalization
tests. Earlier 16-test snapshots predate the two ownership regressions. Formatting
was limited to this package. `git diff --check` and an explicit UTF-8/final-newline/
trailing-whitespace scan of the audio files/report pass. No remote command, native
execution, model run, service, global installation, commit or archive was performed
by this reviewer. Linux does not compile the Windows-only FFI paths.

Final source identity for the main session's refreshed native snapshot:

| File | SHA256 |
| --- | --- |
| src/capture.rs | 1dbeeb3261bc9748b233af4c7b057395d29161341fcf54b6624becdf643ddba3 |
| src/native.rs | 814fe18a531c82eb3e608fcefcaead684f97c37c375bfef393930cde348d5334 |
| examples/audio-diagnostic.rs | 509991cc862e9358e5f72b56ab577f9fa461046a833bcc9308aacf13ce1300f2 |

Paths in the table are relative to `crates/synesthesia-audio`.

## Residuals and acceptance boundary

No known unresolved code finding remains in reviewed C3 scope. Main-session
[native evidence](native-validation.md) documents earlier Windows11 compilation,
actual 44.1 kHz stereo float32 capture, controlled playback, restart and stop joins.
It is historical and does not validate this revised snapshot. The main session
must run native check/clippy/tests/examples and controlled/locked/overload probes
again; `--locked-default` can now report the real numeric default metadata.

Human-required residuals remain scoped: Windows10 including pre1703, physical
default-switch/unplug/disable/re-enable/format scenarios, actual multichannel
devices and natural no-packet/idle behavior when ambient output prevents the
condition. Focused automation above is complete; these device/platform limits do
not block other approved development. C6 owns end-to-end cancellation/idle
finalization; D-10 sustained/quality/latency/resource acceptance remains separate.
Do not infer success from enumeration, fake ownership tests or Linux checks.

Main-session spec follow-up should preserve native worker RAII joining, explicit
cleanup HRESULT failures, selection-versus-metadata distinction, packet-absence
semantics and bounded sample-time conversion with actual source examples.
This reviewer does not declare task/MVP completion, release readiness or archive.
