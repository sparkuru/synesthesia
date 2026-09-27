# Project Validation Profile

- ownership: project-shared
- source: project-authored
- evidence reviewed: 2026-09-27

## Current Surface

The repository is at requirements/deployment-scaffold stage. Evidence:

- `prd.md` requires native audio capture and desktop floating subtitles,
  including transparency, always-on-top, click-through recovery, multilingual
  text, and multi-monitor/DPI behavior. Rust and egui/eframe remain candidates.
- `deploy/README.md`, `deploy/scripts/CONTRACTS.md`, and
  `deploy/VALIDATION.md` distinguish draft interfaces and structural validation
  from unavailable applications, installer implementations, images, and models.
- No application manifest, unit/integration tests, CI workflow, web app,
  Playwright dependency/configuration, or equivalent browser runner exists.
- Web/OBS display is proposed later scope in `prd.md`, not an implemented or
  currently approved browser surface.

Classification: planned native desktop UI, with no runnable UI yet.
Do not infer React/TypeScript conventions from generated frontend spec titles.

## Trellis Plus: UUPM Integration

UUPM initialization: approved by the user on 2026-09-27 and completed with
the installed `uipro` 2.10.2 CLI using `uipro init --ai codex`. The first
sandboxed attempt failed to create the protected platform directory; the
authorized elevated retry succeeded using bundled templates. No existing
Trellis platform files were changed.

Local entry point: `.codex/skills/ui-ux-pro-max/SKILL.md`. Actual script and
data locations are `.codex/skills/ui-ux-pro-max/scripts/` and `data/`.
`python3 .codex/skills/ui-ux-pro-max/scripts/search.py --help` passed; required
search modules, domain CSVs, and stack data are present. The initializer also
added its bundled brand, design, design-system, banner-design, slides, and
ui-styling skills. All generated platform files remain personal/local and
are excluded by the clone's existing ignore behavior; never stage them.
The installed CLI declares MIT in its package metadata; no generated source
or data is copied into this shared policy.

For a future approved UI task:

1. Plan: load the project-local skill and relevant frontend guidelines.
   Generate task-specific `--design-system` research with the actual product
   domain and approved stack. Save permitted research under the task's normal
   `research/` directory and selected original decisions in `design.md`.
   Confirm states, readability, interaction, accessibility, and DPI/resize
   expectations before implementation.
2. Context and implementation: reference this profile, relevant frontend specs,
   approved design, and permitted research in both implement and check
   manifests through the existing task context mechanism. Preserve approved
   decisions; record technical changes in the design and both contexts.
3. Check: inspect typography, contrast, keyboard/focus, accessible names,
   loading/empty/error/disabled/success states, motion, resizing, and relevant
   native-window behavior. For web tasks, apply the browser rules below.
   Request human evidence only for the residual criteria automation cannot
   settle.
4. Update Spec: promote stable original project decisions into this directory.
   Keep task-specific research with the task only when its source/license
   permits retention. Do not create another editable global design authority.

Run the search script from the real project path above; the generated entry's
short `skills/` examples are not root-relative paths in this repository.
Its stack list has no Rust/egui entry. Do not substitute React Native or
HTML/Tailwind for the proposed native stack or adopt its hard-coded stack
assumptions. Use applicable general design research without a false stack
flag and validate native behavior against this project's PRD. Do not use
`--persist` to introduce a competing `design-system/MASTER.md`.

Recheck entry point, modules, and data after updates. If initialization becomes
absent/incomplete, ask before reinstalling or replacing files. A complete
installation does not require asking again. This enhancement generated no
product design artifacts and selected no visual style or application stack.

## Checks Before Submit-Ready

For documentation/configuration changes, inspect local links, concrete file
references, consistency with the PRD and deployment contracts, and the exact
write/staging boundary. Run `git diff --check`; newly untracked files also need
an explicit whitespace/content check because ordinary git diff omits them.
Do not stage files solely to make them visible to a validator.

No application lint, format, type-check, unit, integration, build, or end-to-end
command is currently established. Once manifests and CI exist, update this
profile with exact supported commands and run checks relevant to each diff.
Tests should demonstrate promised behavior rather than mirror implementation.

When deployment artifacts change, use the applicable JSON parsing, Compose
configuration-only, checksum, archive, and Windows checks identified in
`deploy/VALIDATION.md`. Reconcile exact commands with the current files/tooling
before running them; no executable installer or image command is established
here. Existing validation claims are historical evidence, not checks rerun by
this enhancement. Parsing a Compose draft or checking scaffold hashes does
not validate a release or start a service.

Future state/protocol changes need deterministic checks for draft replacement,
final-result stability, stale translation rejection, session isolation,
bounded buffering, reconnection, and cleanup as appropriate to the PRD.
Additional sustained-load and GPU/device tests depend on confirmed hardware,
models, sample data, and acceptance thresholds; do not invent those values.

## Native Desktop Residual Review

For relevant implemented changes, request only the real platform checks the
agent cannot reproduce: system-output capture and permissions, audio-device
switching, transparency/stacking, click-through recovery, fullscreen limits,
multilingual fonts, monitor/DPI transitions, and resource release. Name the
target OS and, for Linux, X11/Wayland separately. Record versions, device,
backend/model, steps, observed behavior, and scoped diagnostic evidence.

Distinguish hardware enumeration, backend availability, model loading, and
actual inference. Use controlled samples; avoid production credentials and
unnecessary raw audio/subtitle retention. Docker success is not native-window
or system-audio validation. Subjective readability and translation-quality
decisions require explicit criteria or targeted human feedback.

## Trellis Plus: Playwright Automated Frontend Validation

Current applicability: `playwright-not-effective` for the planned native
overlay; no browser surface or application command exists to exercise it.
No browser test was attempted and none is reported as passed. Missing future
browser dependencies must instead be classified as `playwright-unavailable`
when automation is suitable but blocked.

When a task introduces a browser-accessible acceptance criterion, read this
profile first and inspect manifests, startup/readiness, fixtures, existing
browser runners, and CI. Classify it as `playwright-required`,
`playwright-existing-equivalent`, `playwright-not-effective`, or
`playwright-unavailable`, with evidence. Preserve an established equivalent
runner. If required and absent, add the smallest justified Playwright Test
setup and focused persistent coverage using the confirmed package manager.

Assert relevant routes, actions, changed states, keyboard/focus/accessibility,
and supported desktop/mobile viewports using semantic locators and controlled
fixtures or mocks. Run focused checks and any required wider suite. Record the
exact command, coverage, browser project, fixtures, result, and failure-artifact
paths in task evidence. Preserve traces, screenshots, and console/network logs
on failure. Review snapshot changes intentionally; diagnostic screenshots are
not pass/fail baselines without a controlled environment and approved baseline.

Passing relevant automation removes the need for a generic browser smoke-test
request. Feed only remaining subjective, private-environment, native-device,
or unavailable-check risks into the human review gate.

### Trellis Plus: Playwright Validation Profile

- applicability: deferred; no browser-accessible UI in the current repository
- execution mode: unselected; choose project-local, docker-wrapper, or CI from a runnable future task
- setup/install: none established; no project browser dependency or browser binary setup
- app readiness: no startup command, readiness URL, or base URL established
- focused test command: none established
- full/CI browser command: none established; no CI browser job
- test location and config: none
- browser projects and supported viewports: none configured
- fixtures and test-data boundary: none configured; future work uses controlled data rather than personal sessions
- accessibility policy: no scan configured; define meaningful keyboard/name/focus checks with the UI task
- visual baseline policy: no approved baseline; screenshots are diagnostic only
- failure artifacts: no configured paths; record actual report/trace/screenshot/console/network paths when adopting a runner

Update this single profile when browser infrastructure becomes real. Fill exact
commands and paths from repository evidence rather than example defaults.
Task evidence records its own results without duplicating this shared profile.
