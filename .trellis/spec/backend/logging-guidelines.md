# Bounded Diagnostics

Use `crates/synesthesia-core/src/diagnostics.rs::DiagnosticLog::write(Diagnostic)`.
Diagnostic accepts timestamp/component/kind and optional safe error/backend,
vendor/device IDs, elapsed milliseconds and queued samples. It has no arbitrary
String, payload or formatting callback. Extend the whitelist deliberately.

Application and owned engines share one sink. Parse/discard raw engine
stdout/stderr in bounded memory; never tee it to disk or echo it as app logs.
Do not persist audio, subtitles, prompts, tokens or session content in logs,
crash dumps, temporary files or caches.

The private sink directory owns only diagnostics.0.jsonl through .4.jsonl and
zero-length writer.lock. Five 4 MiB segments enforce 20 MiB aggregate, including
restart/oversize recovery; a record is at most 1024 bytes including newline.
Acquire exclusive ownership before truncating the lock. Unknown/nonregular
files cause refusal and are preserved. The directory is not an adversarial
filesystem sandbox.

Keep diagnostic unit assertions for rotation, restart, concurrent writers,
oversized records/lock files, foreign files and unwritable storage.
Wrong: truncate the lock before acquiring it or append foreign process output.
Correct: acquire ownership before recovery and project typed safe records.
