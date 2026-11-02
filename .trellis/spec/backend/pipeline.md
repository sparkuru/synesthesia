# Desktop Pipeline Configuration (schema 2)

## 1. Scope / Trigger

Current product: microphone/system -> local/cloud STT -> optional translation ->
transparent captions. Root PRD v2 is authoritative. Schema 1 and WS v1 remain
legacy contracts, not implicit aliases for schema 2.

## 2. Signatures

~~~rust
PipelineConfig::parse(bytes: &[u8]) -> Result<PipelineConfig, PipelineError>
PipelineConfig::load(path: &Path) -> Result<PipelineConfig, PipelineError>
PipelineConfig::resolve_paths(&mut self, directory: &Path) -> Result<(), PipelineError>
PipelineConfig::validate(&self) -> Result<(), PipelineError>
~~~

Implementation: crates/synesthesia-core/src/pipeline.rs.
CLI: synesthesia --check-config FILE; exit 0 for structural success, 2 for
invalid input or unsupported runtime. No arguments show truthful help. Output
does not echo values. Use parse/load at boundaries; if constructing or mutating
public fields in Rust, validate before use.

## 3. Contracts

- Required schema_version: 2, source_language: zh|en|ja, capture, asr,
  translation. Unknown/duplicate fields and trailing JSON fail. The shared
  65536-byte MAX_CONFIG_BYTES bounds parsing and file reads.
- capture.kind: microphone|system; selection.mode: default|device. Device mode
  requires a nonblank, control-free device_id <=256 UTF-8 bytes.
- asr.provider: local requires model and existing CPU/Vulkan device object.
  fun_asr requires region cn-beijing|ap-southeast-1, a nonblank model name and
  api_key_env (1..128 ASCII bytes; initial letter/underscore, then letters,
  digits/underscore). Inline API keys and local model fields are not accepted.
- Fun-ASR optional workspace_id/vocabulary_id use the identifier bounds above;
  sentence_silence_ms defaults to 800 and accepts 200..6000. Optional context
  has <=400 characters without controls. These are configuration constraints,
  not evidence of current cloud model capability.
- translation.provider: disabled accepts no model/device/target. Local requires
  model, device and target_language: zh-Hans. New services require explicit
  schema variants and actual implementations.
- overlay defaults to source_only with visible_lines: 3. Lines: 1..6; disabled
  translation requires source_only. Visual lines are not reducer history size.
- Relative model paths follow the canonical configuration file's parent.
  Existing safe path and identifier primitives are reused. Parent traversal is
  rejected; use an absolute path for models outside the config subtree.
- Parsing and file loading never access environment values, cloud services,
  audio devices or model contents. Runtime capability/readiness is separate.
- Pipeline Debug is redacted; content-bearing nested config types omit Debug.
  Errors contain only safe categories and static field names.

## 4. Validation & Error Matrix

| Input | Result |
| --- | --- |
| Unsupported version | UnsupportedVersion at schema_version |
| Malformed/unknown/duplicate/oversized JSON | InvalidConfig at document |
| Missing/unreadable configuration | InvalidConfig at file |
| Invalid model path/device/provider parameters | InvalidConfig at asr or translation |
| Disabled translation with bilingual/translation-only | InvalidConfig at overlay |
| Missing secret environment value | Structure may succeed, no authentication attempted |
| Missing model file | Structure may succeed, no loading attempted |

## 5. Good / Base / Bad Cases

Good: cloud ASR plus disabled translation requires no local model; cloud ASR
can also select local translation. Base: a not-yet-installed local model can
pass structural checks. Bad: silently switching provider/input, accepting an
inline API key, or requiring translation readiness for source-only operation.

## 6. Tests Required

core/tests/pipeline.rs exercises the 2x2x2 input/ASR/translation matrix, all
repository examples, no-secret structure checks, invalid combinations,
redaction, strict JSON, provider/display bounds and model path resolution.
Existing v1 state/config/protocol tests remain active. Real cloud/device/model
acceptance is separate; these tests do not instantiate a runtime adapter.

## 7. Wrong vs Correct

Wrong: feed schema 2 to the old combined ComputeAdapter, construct a fake
translator for disabled mode, or call configuration success ready.

Correct: keep version boundaries explicit, implement separate recognizer and
translator lifecycles, then establish real readiness and cleanup evidence.
