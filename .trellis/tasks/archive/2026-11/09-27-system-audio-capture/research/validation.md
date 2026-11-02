# C3 validation evidence

Date: 2026-09-27. Implementation delivery, not product completion or release.

## Linux package checks

Through existing repository hako, Rust1.90.0, native Linux development only:

- `HAKO_NETWORK=none ./hako cargo test -p synesthesia-audio --locked --offline`:16 passed (4 bounded queue/stop/privacy,12 descriptors/PCM/downmix/nonfinite/rate/filter/count/partition/idle-tail tests), zero failed after correction of one malformed-fixture byte that had accidentally preserved its original value.
- `HAKO_NETWORK=none ./hako cargo check -p synesthesia-audio --all-targets --locked --offline`: passed on final v4 source.
- `HAKO_NETWORK=none ./hako cargo clippy -p synesthesia-audio --all-targets --locked --offline -- -D warnings`: passed on final v4 source. A prior run encountered concurrent C2 lock regeneration pending; retried after resolution without changing criteria.
- `HAKO_NETWORK=none ./hako cargo fmt -p synesthesia-audio -- --check`: passed on final v4 source; formatting only touched owned package.
- `HAKO_NETWORK=none ./hako cargo build -p synesthesia-audio --all-targets --locked --offline`: passed, including numeric diagnostic executable.

No Linux check exercises WASAPI. Unsupported platforms explicitly report PlatformUnavailable.

## Native Windows evidence supplied by main session

Main session's private Rust1.90/MSVC harness on Windows11 build26100:

- First native compile:3 real API errors (Error::from_win32 unavailable; Activate feature missing). Fixed against pinned source using Error::from_thread and Win32_System_Variant; no compatibility claim from initial failure.
- C3 v3 `check/test/build --examples` passed;16 tests passed natively. Basic `audio-diagnostic --seconds2` passed both input generations,4 active outputs/1default, missing locked output rejected, actual default44100 Hz float32 stereo mask3; stop14/17 ms, old output queues cleared. This is actual capture evidence, not only endpoint enumeration.
- Earlier v4 native check/clippy/test/examples passed (16 audio tests; combined run also included6 inference tests, which are not counted as C3 tests). Its controlled3-second capture passed both generations, same endpoint, pause/resume, stop15/10 ms; no-packet assertion unavailable because the endpoint had ambient output.
- The final example adds streaming697/941 Hz numeric projections to prove owned test tones, with paused projections separately. Those exact final-source assertions, locked and overload runs remain root's next native checks. Root will record archive identities, exact commands and observed output; earlier v4 results do not stand in for that final-source validation.

## Privacy and verification boundaries

Probe retains PCM only in bounded transient queues and uses streaming numeric tone projections. No audio/text/model/token file is written, system volume/default output is not changed, and user playback is not stopped. Opaque endpoint identity is for selection only and redacted from Debug/diagnostic output.

Remaining required evidence: checked final source on native Windows; actual controlled tone capture and owned pause/resume; real no-packet idle on a naturally idle endpoint if ambient prevents it; default output switching, unplug/disable/re-enable/lock retention, multi-channel hardware where claimed, repeated failure cleanup, and separate Windows10/pre1703 native runs. D-10 sustained end-to-end/quality gates remain C6/C7.

Human gate classification remains `human-required` for unavailable Windows10/device scenarios, after runnable automation. Main session owns any concrete residual request and task completion/archive; this implementation does not declare those checks passed.
