# Native Desktop Guidelines

Current implementation has a formal Rust entry, platform-free subtitle state,
and synesthesia-desktop::presentation, a borrowed projection of core Subtitle.
There is no native window/tray or browser app. Root PRD v2 and current roadmap
supersede the old C5 plan; historical research remains evidence. React hooks,
CSS and browser conventions do not apply.

## Guidelines Index

| Guide | Scope |
| --- | --- |
| [Native UI Boundary](native-ui.md) | Existing state authority and approved presentation requirements |
| [Core Contracts](../backend/core-boundaries.md) | Shared identity, publication and failure behavior |
| [Validation Profile](../trellis-plus/validation.md) | UUPM and platform-specific evidence |

## Pre-Development Checklist

Load root PRD, architecture/status, relevant core contracts and active task
artifacts. For native UI design, load local UUPM as required by the validation
profile. Pure presentation changes do not select a window framework. Never
create a second reducer or treat visual lines as stored utterance groups.

## Quality Check

For code changes, run Rust gates for affected crates and check shared
state/identity/revision ownership. For documentation-only changes, check links,
source references and scope consistency. Once UI exists, verify the approved
PRD v2 scope: transparency/topmost, recovery controls, wrapping, visual lines
separate from history, translation-compatible modes and visible errors.
Do not import C5's fixed three groups, bilingual default or tray/click-through
scope into current acceptance. Test focus, fonts and monitor/DPI behavior for
the implemented platform surface. Native automation precedes targeted residual
human review. Playwright currently has no effective surface.
