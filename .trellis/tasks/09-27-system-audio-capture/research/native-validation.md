# Main-session native C3 evidence

Windows11 build26100, private Rust1.90/MSVC; native commands, no Docker/WSL.
Task-only source/cache/outputs stay under the C1 native workspace. No audio file,
content log, global output/volume/profile/registry change is made by the probe.
Device/API/package tests are separate from Windows10 and complete MVP acceptance.

## Snapshot and compilation progression

- v1 source SHA `aed4bf4462848a1afc28319a4318cb4f2c1e50db6d4fb149f5d463a3cb4f0378`;
  first offline check stopped at unfetched concurrent inference sha2 dependency,
  before WASAPI source compilation. Not a source-test pass.
- v2 source SHA `dbaa77986de33af0fe93035ba5f65b2cb883e406c8a970e79049b55e9ccbf7c0`,
  registry SHA `839b5966f120043f0faae4f092c85046a4fbed7a29e31c5bbbe5276ea13075f2`.
  Native check exposed Error::from_win32 and missing Activate feature. Worker
  corrected against pinned windows0.62.2 and enabled System_Variant.
- v3 source SHA `2dba85541fac9ded0b61cad18baf3e1ca5926b5d59a4247b2d821d0fa52efe3c`.
  Native check(all-targets), test and examples build passed locked/offline:
  16 tests (4capture,12normalization). Basic diagnostic below then ran.
- combined adapters v4 source SHA
  `6577912e24b8444e809e90077f84e8c7bb142981f7bdf7034d81019ea3486f21`.
  Native check/clippy(all-targets,-Dwarnings), test and examples build passed:
  16audio+6inference tests. Controlled renderer below ran; later tone-correlation
  assertions were added to distinguish known playback from ambient activity.

## Actual runtime observations

`audio-diagnostic --seconds2` passed:4render endpoints,1default, missing locked
endpoint rejected; actual default mix44100Hz/2channels/float32/channel-mask3.
Two generations restarted at seq/sample0 and stopped with both worker joins and
empty old queues. Stop elapsed14/17ms. Each2second generation captured around
31.6ksamples and nonzero activity. Endpoint already had unrelated playback;
this initial run is real loopback evidence but not a controlled-tone comparison.

`audio-diagnostic --controlled-playback --seconds3 --expect-active` passed on
v4: memory-only697/941Hz renderer used the exact capture endpoint, paused its
own renderer and resumed across generation2. Capture continued; stop15/10ms,
queues cleared. Existing ambient output prevented a no-packet interval;
`no_packet_assertion_available=0` was correctly reported in both generations.
No user playback was stopped to force a result. v4's basic activity assertion
predates correlation checks, so final controlled-tone verification is pending.

## Final reviewed snapshot

Combined adapters v7 source SHA256
`ed52d08e19e6dc5d0264862c36f82838aec5a7a72b18c0869fc4bd51d2fb02d1`.
Includes C3 independent review fixes: queried default identity, joined normalizer
on unwind, prompt worker failure signalling and safe Stop/unregister errors.
Native check/clippy(all-targets,-Dwarnings), tests and examples build all passed
locked/offline:18 audio tests and7 inference library tests. Benchmark example's
additional unit test was verified by focused Linux all-target checks, not this
native cargo test invocation.

Final `audio-diagnostic --controlled-playback --seconds 3 --expect-active`
passed both generations. Numeric697/941Hz projections were89/60 and90/60
milli-amplitude; after owned renderer pause1/1 and2/1. This distinguishes the
controlled memory-only tones from unrelated playback. Stop10/5ms, queues clear,
47740..48018 received samples, actual44100Hz stereo float32. Generation2 resumed
the same endpoint with new input identity and fresh seq/sample0.

`--controlled-playback --seconds 2 --locked-default --expect-active` passed
both generations and reported selected_default=1, validating the reviewed
metadata fix. Tone89/60 and89/59; pause5/3 and2/2; Stop3/5ms, queues clear.

`--controlled-playback --seconds 2 --overload` passed both generations:
explicit overloaded=1,3157 queued samples discarded at the stop barrier,
no later publication, Stop0ms. Enclosing SSH command exited0 for all three probes.
No packet-absence assertion was available because unrelated output continued;
the test correctly reported0 and did not stop user playback.

C3's checked dependency delivery is available to C5/C6. Pending platform/product
evidence: physical default-switch/unplug/re-enable, naturally idle no-packet,
Windows10/pre1703 and integrated lifecycle. These are not reported as passed.
