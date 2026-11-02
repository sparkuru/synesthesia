# Boundary Errors

`crates/synesthesia-core/src/lib.rs::ErrorCode` is a finite safe category shared
by domain, adapters and wire errors. `config.rs::ConfigError` adds safe
field/line/column context without copying parser messages or user values.
`diagnostics.rs::storage` maps I/O failures to StorageUnavailable.

Do not wrap foreign error text, audio, prompts, subtitles or secrets. Preserve
useful details as safe enums/numeric fields. Secret redacts Debug; expose it
only at authentication. Queue overflow/gaps explicitly fail and clean up the
session. Consumers execute `Session::take_cleanup` effects and acknowledge
actual adapter completion. Stop acknowledgement follows cancellation/context
cleanup, not merely sending a request.

Shared CLI returns 0 for structural validation and 2 for invalid/unsupported
input; success explicitly leaves model/runtime readiness unverified.
`crates/synesthesia-core/tests/contracts.rs` covers roles, schema, paths and
redaction. GPU/native failure must not silently select CPU or another GPU.
