---
sessionId: session-260516-110003-1bsx
isActive: true
---

# Requirements

### Overview & Goals
Prepare an implementation-ready execution plan for the active feature (`specs/004-audio-format-compression`) that follows `/speckit.implement` gates and resolves current runtime blockers before code execution.

### In Scope
- Run `.specify/scripts/bash/check-prerequisites.sh --json --require-tasks --include-tasks` and treat failures as hard-stop gates.
- Apply `before_implement` and `after_implement` hook handling from `.specify/extensions.yml` (optional `speckit.git.commit`).
- Evaluate all checklist files under `specs/004-audio-format-compression/checklists/` and enforce consent behavior if any item is incomplete.
- Load implementation context from required docs (`tasks.md`, `plan.md`) and optional docs (`research.md`, `data-model.md`, `contracts/`, `quickstart.md`, `.specify/memory/constitution.md`) when present.
- Verify ignore-file coverage based on detected stack (Rust + git in this repository) and append only missing critical patterns.
- Execute implementation work in `tasks.md` phase order, update task status in-place (`[X]`), and validate final build/test outcomes.

### Out of Scope
- Changing feature requirements in `specs/004-audio-format-compression/spec.md`.
- Bypassing Speckit prerequisite checks with ad-hoc edits.
- Reworking unrelated modules outside task scope.

# Technical Design

### Current Implementation (Observed)
- Prerequisite script behavior is defined in `.specify/scripts/bash/check-prerequisites.sh`:
  - validates feature branch,
  - requires `plan.md`,
  - with `--require-tasks` also requires `tasks.md`,
  - returns `FEATURE_DIR` and `AVAILABLE_DOCS` JSON on success.
- Active feature pointer is `.specify/feature.json` → `specs/004-audio-format-compression`.
- `specs/004-audio-format-compression/tasks.md` exists and is mostly complete; `T031` remains unchecked and explicitly marked as deferred due interactive hardware/TTY constraints.
- Checklist file `specs/004-audio-format-compression/checklists/requirements.md` is fully checked.
- Known preflight blocker from current session: prerequisite script fails on branch `master` (feature-branch naming gate).
- Current post-upgrade compile errors are in known touched modules:
  - `src/audio/record.rs` (`config.sample_rate().0`),
  - `src/audio/monitor.rs` (`stream_config.sample_rate().0`),
  - `src/tui/app.rs` (`run_event_loop` draw error mapping with generic backend error).
- Dependency context in `Cargo.toml`: `cpal = 0.17.3`, `ratatui = 0.30.0`, `crossterm = 0.29.0`.

### Key Decisions
1. **Keep prerequisite + branch checks as hard gates** before any implementation execution.
2. **Treat checklist scan as a formal consent checkpoint** exactly per outlined rules.
3. **Use `tasks.md` as the authoritative execution order** and keep status synchronization via immediate `[X]` updates.
4. **Handle post-upgrade compile regressions as implementation validation blockers** in the same run, with minimal compatibility edits in `src/audio/*` and `src/tui/app.rs`.
5. **Use verification-first hygiene for ignore files** (`.gitignore` append-only if required patterns are missing).

### Proposed Changes
- **Preflight**
  - Run prerequisite script and stop on branch/tasks failures with explicit remediation.
  - Surface optional `before_implement` hook metadata from `.specify/extensions.yml`.
- **Checklist + context loading**
  - Build checklist status table for all files in `FEATURE_DIR/checklists/`.
  - If incomplete items exist, pause for explicit yes/no before continuing.
  - Load required and optional docs to assemble full implementation context.
- **Execution**
  - Parse `tasks.md` phases/dependencies and execute only in allowed order.
  - Respect `[P]` parallel markers only for file-independent tasks.
  - Mark completed tasks in `specs/004-audio-format-compression/tasks.md` immediately.
  - Resolve build breaks surfaced by upgraded crates in `src/audio/record.rs`, `src/audio/monitor.rs`, and `src/tui/app.rs` while preserving behavior.
- **Completion**
  - Validate spec alignment, build/test pass status, and task completion.
  - Surface optional `after_implement` hook metadata.

### Risks
- **Blocked start risk:** branch naming gate still fails on `master`.
  - Mitigation: switch to compliant feature branch before execution.
- **Tracking drift risk:** tasks marked complete may not match actual code state after dependency upgrades.
  - Mitigation: re-run `cargo build`/tests and reconcile `tasks.md` status with real outcomes.
- **Manual validation risk:** interactive step T031 cannot be automated in this environment.
  - Mitigation: keep deferred note explicit and separate from non-interactive completion criteria.

# Testing

### Validation Approach
- Validate in strict order: prerequisites → checklist gate → task execution order → build/test completion.

### Key Scenarios
- Prerequisite script fails on invalid branch and passes once branch context is compliant.
- Checklist table correctly reports totals/completed/incomplete and prompts only when needed.
- Required implementation docs (`tasks.md`, `plan.md`) are loaded before execution.
- Post-upgrade compile issues in `record.rs`, `monitor.rs`, and `app.rs` are resolved without unrelated behavior changes.
- `tasks.md` checkbox state reflects completed work at end of run.

### Completion Checks
- `.specify/scripts/bash/check-prerequisites.sh --json --require-tasks --include-tasks` succeeds.
- All required implementation tasks are complete or explicitly deferred with rationale.
- `cargo build` succeeds and validation tests pass for affected modules.
- Final report includes before/after implement hook visibility and remaining follow-ups (if any).

# Delivery Steps

###   Step 1: Clear preflight gates and hook context
Implementation prerequisites are verified and blocker status is explicit.
- Run `.specify/scripts/bash/check-prerequisites.sh --json --require-tasks --include-tasks`.
- If branch validation fails, switch to a compliant feature branch and re-run checks.
- Report optional `before_implement` hook details from `.specify/extensions.yml`.

###   Step 2: Run checklist gate and load full implementation context
Execution starts only after checklist and document inputs are validated.
- Scan `specs/004-audio-format-compression/checklists/*.md` and publish total/completed/incomplete status table.
- If any checklist is incomplete, pause for explicit proceed/stop consent.
- Load `tasks.md`, `plan.md`, and available optional docs (`research.md`, `data-model.md`, `contracts/`, `quickstart.md`, constitution).
- Verify ignore-file coverage for Rust/git and append missing critical patterns only.

###   Step 3: Execute tasks in dependency order and apply compatibility fixes
Implementation changes are applied safely with task-state tracking.
- Parse phases and dependencies from `specs/004-audio-format-compression/tasks.md` and execute sequentially.
- Apply compile compatibility updates in `src/audio/record.rs`, `src/audio/monitor.rs`, and `src/tui/app.rs` if still failing after dependency upgrades.
- Mark each completed task as `[X]` immediately in `tasks.md`.

###   Step 4: Validate completion and publish handoff
Implementation closes with reproducible evidence and hook follow-up info.
- Run `cargo build` and relevant test subsets to confirm regression-free state.
- Reconcile task completion, deferred/manual items, and spec alignment.
- Report optional `after_implement` hook details and any remaining actionable follow-ups.