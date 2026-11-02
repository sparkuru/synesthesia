<!-- TRELLIS:START -->
# Trellis Instructions

These instructions are for AI assistants working in this project.

This project is managed by Trellis. The working knowledge you need lives under `.trellis/`:

- `.trellis/workflow.md` — development phases, when to create tasks, skill routing
- `.trellis/spec/` — package- and layer-scoped coding guidelines (read before writing code in a given layer)
- `.trellis/workspace/` — per-developer journals and session traces
- `.trellis/tasks/` — active and archived tasks (PRDs, research, jsonl context)

If a Trellis command is available on your platform (e.g. `/trellis:finish-work`, `/trellis:continue`), prefer it over manual steps. Not every platform exposes every command.

If you're using Codex or another agent-capable tool, additional project-scoped helpers may live in:
- `.agents/skills/` — reusable Trellis skills
- `.codex/agents/` — optional custom subagents

Managed by Trellis. Edits outside this block are preserved; edits inside may be overwritten by a future `trellis update`.

<!-- TRELLIS:END -->

## Current product baseline (2026-11-02)

- Read root `prd.md`, `docs/architecture.md`, `docs/status.md` and
  `docs/roadmap.md`. The product is microphone/system audio -> local/cloud STT
  -> optional translation -> transparent topmost captions, implemented in Rust.
- Formal entry: `apps/synesthesia`; current desktop configuration lives in
  `core::pipeline` (schema 2). Existing full/server/client schema 1 and WS v1
  remain compatibility interfaces, not the new product's required topology.
- September tasks, PRD and `deploy/` preserve historical work. Do not resume
  unfinished old scope or infer new approvals from historical metadata.
- Input, ASR and translation are independent. Cloud ASR needs no local ASR
  model; disabled translation needs no translator. Existing combined local
  adapters still need migration before they satisfy these rules.
- All workspace members participate in checks. Pure logic, structural checks
  and historical device experiments do not establish current runtime support.
- Do not read or modify the user-owned private `archive/` directory. It is
  excluded from version control. Never copy prototype credentials or content.
