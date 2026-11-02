# Trellis Plus Project Policy

- ownership: project-shared
- source: project-authored
- tracking: commit this file and its referenced project-owned detail files
- evidence reviewed: 2026-11-02; current product PRD v2; original policy introduced 2026-09-27

## Scope and Loading

This directory contains the shared enhancement policy for this repository.
Read it at the beginning of each Trellis Plus run and before applying these
enhancements to a task. It is guidance data, not a runtime hook. Existing
Trellis phases and task states continue to govern task work.

For an approved active task, register this index and the applicable detail
files in both implement and check context through the existing `task.py
add-context` command. Reuse existing entries. Without an active task, leave
task pointers, manifests, and statuses alone.

| Detail | Use |
| --- | --- |
| [Validation](validation.md) | Repository evidence, UUPM integration, automated checks, residual desktop checks |
| [Development readiness](development.md) | Docker wrapper bootstrap and local execution boundary |

## Ownership and Update Boundary

Keep new shared rules here and task evidence in the normal task directory.
Treat Trellis workflow, scripts, agents, configuration, ignore file, version,
template hashes, backups, managed AGENTS blocks, and generated platform
integrations as protected. Read them without editing or staging them through
Trellis Plus. Preserve existing spec indexes outside this directory.

Platform settings remain personal/local. Never stage `.codex/`, `.agents/`,
`.claude/`, or `.opencode/` files through this enhancement. Stage only an
explicit reviewed list of shared or authorized ordinary project paths;
broad or forced staging is prohibited. Check whitespace and the candidate
and staged path lists before any commit.

No Trellis LICENSE, COPYING, NOTICE, or COPYRIGHT file was found in this
clone during discovery. Notice status: `missing` / `license-notice-needed`.
Verify the exact installed distribution's notice separately before proposing
distribution of protected upstream material; do not invent a root license.

These new files are outside the current template-hash target list. After
`trellis update`, reconcile their paths and task references with repository
evidence. Do not restore customizations into protected files or change
`update.skip`. No backup recovery was needed for this installation.

## Trellis Plus: Submit-Ready Human Review Gate

Before a commit plan or task completion/archive, compare the finished diff
with its acceptance criteria and actual validation results. Run the relevant
automatable checks first, including focused browser coverage when applicable.
Classify the remaining review as follows:

- `human-required`: a material check is unavailable or user-facing behavior is
  unverified; approval needs visual/product judgment, real devices, private
  environments, or hardware; or the change affects permissions, security,
  authentication, billing, migration, deletion, deployment, CI/CD, generated
  assets, or irreversible operations. Request specific residual evidence
  before committing or declaring completion.
- `human-optional`: relevant automation passed and a small, low-risk user
  check would add confidence. State whether the request blocks the next step.
- `human-not-needed`: documentation/mechanical work or meaningful focused
  checks leave no useful human judgment. Record a short reason and proceed
  within existing authorization.

A required request must describe the concrete change, commands and results
already obtained, the smallest remaining scenario to inspect, and useful
feedback such as pass/fail, expected versus actual behavior, platform details,
or failure logs. Do not substitute a generic smoke-test request for runnable
automation or claim an unavailable check passed. Review of this documentation
enhancement is `human-not-needed` after link and boundary verification.

## Trellis Plus: Commit Completion and Attribution

Before each Phase 3.4 work commit, assess substantial AI authorship. Use
`yes` for meaningful implementation, non-obvious design/debugging, or
substantial validation work; use `no` for mechanical edits, small follow-ups,
unrecognized/user-authored work, user-made commits, archive, and journal
commits. Reserve `ask` for an ambiguous established project convention.

For `yes`, preview a compact body explaining the problem, design reason,
resulting behavior, relevant boundaries, checks run, and material gaps. Add
`Co-authored-by: OpenAI Codex <codex@openai.com>` after a blank line. There is
no commit history yet, so this is the default rather than an inferred local
convention. Reconcile it with later history. Do not pad a small commit to
justify attribution. Present exact paths and follow the existing commit
authorization process; installing this policy authorizes no commit.

## Trellis Plus: Mainline Continuity

For project-relevant no-task or post-archive continuation requests, perform a
read-only Project Pulse: read `.trellis/mainline.md` if present, normal active
and archived task evidence, git state, and validation results. Report the
approved initiative and completed evidence, concrete blockers/dirty state,
one uniquely ready candidate if available, and the next permitted action.

Default to `guided`: recommend and wait for the user's choice. Honor `paused`
without creating work. Continue in `serial` only when the user has explicitly
authorized a bounded initiative and ordered child list, with readiness and
stop conditions recorded in `.trellis/mainline.md`. Stop for ambiguity, risk,
scope change, dirty unresolved work, or missing dependencies. Keep normal
planning, checks, review, commit, and archive requirements.

No mainline record is established. Root `prd.md` now defines the current
product, and `docs/roadmap.md` records the delivery order; neither alone
authorizes autonomous implementation of every stage. The 1102 repository
cleanup is explicitly authorized, while runtime work still needs its own
approved scope. Old task approval and task archive alone grant no continuation
authority. Preserve superseded tasks' historical status and evidence.

The main session owns orchestration, acceptance mapping, task lifecycle,
commit, archive, and mainline updates. When delegation is authorized, workers
receive a bounded active task and report changed files, checks, and open
decisions. They do not select new work, archive tasks, or dispatch further
implementation/check workers.
