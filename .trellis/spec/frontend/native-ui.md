# Native UI Boundary

> Current requirements are [PRD v2](../../../prd.md); implemented capability is
> recorded in [Engineering Status](../../../docs/status.md). The pure surface is
> `synesthesia-desktop::presentation::project(&Subtitle, DisplayMode) -> Caption`.
> It borrows core state and exposes source/translation plus Draft/Pending/
> Complete/Failed status; it never schedules work. It does not express disabled
> translation yet; the future runtime must supply that state explicitly.
> No native window/font/tray implementation exists. The September C5 decisions
> below are retained research/context, not new product approval.

## Existing State Authority

Use `crates/synesthesia-core/src/domain.rs::SubtitleReducer` history and the
typed SourceEvent/TranslationEvent/PublishedEvent flow described in
[Core Boundaries](../backend/core-boundaries.md). The current reducer treats a
source final as terminal; translation must match its identity/revision. Before
supporting a provider's final revisions, define shared revision/translation
semantics and verify them with behavior tests. Presentation must not bypass the
reducer, create a second reducer, resurrect evicted items or translate drafts.

No native window, tray, font loader, component library or native event-loop
pattern is implemented. Formal `apps/synesthesia` and the retained role entries
provide structural configuration checks. Do not present planned UI requirements
as tested capabilities or import web scaffolding.

## Current Presentation Requirements

For an approved UI implementation, apply PRD v2 FR-03 through FR-06 and the
task's declared platform/acceptance scope:

- Render readable text over a transparent, topmost window. Provide movement,
  resize, font size, wrapping, clear and a reliable settings/display recovery
  path; keep current captions and errors visible.
- Support source-only, bilingual and translation-only display without starting
  recognition or translation work when the mode changes. Translation disabled
  requires source-only, as enforced by [schema 2](../backend/pipeline.md).
  The future runtime must explicitly supply disabled translation state; the
  existing projection does not distinguish it from a pending final.
- Show source before translation; translation-only shows waiting/failure state
  without falling back to source. Keep utterance history and visual line count
  separately named and controlled; three visual lines are not three groups.
- Separate normal Stop's bounded processing of accepted audio/final captions
  from fault/timeout cancellation. The legacy scheduler's cancel-and-clear Stop
  still needs migration; the UI must not claim this contract is already met.
- Tray and click-through remain future scope. If implemented, preserve reliable
  recovery and visible errors; do not enable an unrecoverable display state.

Use controlled inputs for native transparency/topmost, controls, modes,
keyboard/focus, multilingual fonts, resize and applicable monitor/DPI checks on
the declared platform. Also verify stale-event/clear behavior and resource
release. Pure projection tests do not establish native window behavior.
Subjective readability and unavailable platform checks remain scoped residuals.
UUPM research informs design without selecting a framework, substituting a false
Rust stack flag or creating a competing global design authority.

## Historical C5 Context

The following records the September plan and its requested evidence. It is not
the current UI task's acceptance checklist; use the requirements above instead.

September PRD D-05b/D-08 and the old desktop-overlay task specified Windows
10/11 x64, bilingual default with source above Chinese, three total groups
including current, wrapping and source/translation modes without reinference.
That plan hid source drafts in translation-only mode and showed waiting/failure
until final translation. It allowed input during adjustment and proposed tray
watch/adjust/recovery/settings/stop/quit controls; closing settings would hide
them while service continued. Hotkeys/autostart/updater were excluded.

The old plan required reliable tray recovery before click-through and editable
access when tray initialization failed. Its Stop cleared session content and
capture/work while retaining loaded models; quit released resources. Those
controls were shared by full and client in the historical topology.

### Historical C5 Evidence Scope

The old C5 plan called for implementation examples and native evidence covering
transparency/topmost/stacking, click-through recovery,
settings close/stop/quit, keyboard/focus, multilingual fonts, resize and monitor
DPI. Its stop-clears-content contract, bilingual default, fixed history count
and tray requirements belong to that plan. They do not authorize or implement
those choices in the current pipeline.
