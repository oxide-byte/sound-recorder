<!--
Sync Impact Report
- Version change: N/A (template) → 1.0.0
- Modified principles:
  - Template Principle 1 → I. Rust-First, Stable Tooling
  - Template Principle 2 → II. CLI-First Interaction
  - Template Principle 3 → III. Testable Delivery Gates (NON-NEGOTIABLE)
  - Template Principle 4 → IV. Incremental Architecture & Integration Safety
  - Template Principle 5 → V. Observability, Simplicity, and Version Discipline
- Added sections:
  - Repository Guardrails
  - Development Workflow & Quality Gates
- Removed sections:
  - None
- Templates requiring updates:
  - ✅ updated: .specify/templates/plan-template.md
  - ✅ updated: .specify/templates/spec-template.md
  - ✅ updated: .specify/templates/tasks-template.md
  - ⚠ pending: .specify/templates/commands/*.md (directory not present in this repository)
-->

# Sound Recorder Constitution

## Core Principles

### I. Rust-First, Stable Tooling
All production code MUST be implemented in Rust and remain compatible with the
repository's declared toolchain and package metadata. New dependencies MUST be
justified in the feature spec and limited to what is required for user value.
Rationale: stable toolchains and disciplined dependency growth keep builds
predictable and maintenance costs low.

### II. CLI-First Interaction
Primary interaction with this project MUST be available through a deterministic
command-line workflow using arguments/stdin for input, stdout for expected
results, and stderr for errors. Human-readable output is required; structured
formats SHOULD be offered when automation needs are identified in a spec.
Rationale: CLI-first design enables repeatable local runs, scripts, and CI use.

### III. Testable Delivery Gates (NON-NEGOTIABLE)
Every behavior-changing change MUST include a verification path before merge:
either automated tests (preferred) or an explicitly documented manual
reproduction for trivial fixes. Regressions MUST be reproduced before fixes when
feasible, and all touched modules MUST compile before completion.
Rationale: explicit verification gates prevent accidental regressions.

### IV. Incremental Architecture & Integration Safety
Work MUST be scoped into independently testable increments (user stories/tasks)
and preserve clear module boundaries. Changes to shared contracts, CLI surfaces,
or cross-module behavior MUST include integration-level validation.
Rationale: incremental architecture reduces delivery risk and keeps refactors
safe.

### V. Observability, Simplicity, and Version Discipline
Implementations MUST prefer the simplest design that satisfies current accepted
requirements (no speculative abstractions). User-impacting operations and
failures MUST be diagnosable through clear messages/logging. Breaking behavior
changes MUST be explicitly called out in specs/plans and versioned using
semantic versioning.
Rationale: clear diagnostics and disciplined scope support long-term reliability.

## Repository Guardrails

- Preserve the existing repository layout unless a plan explicitly approves
  structural changes.
- Keep public interfaces minimal; remove dead code and unused dependencies during
  related changes.
- Document any new environment assumptions in `README.md` or feature quickstart
  artifacts before completion.

## Development Workflow & Quality Gates

1. Capture feature intent in spec artifacts before implementation.
2. Confirm Constitution Check gates in the implementation plan before design and
   again after design updates.
3. Execute implementation tasks by user story to preserve independently
   verifiable increments.
4. Before completion, record verification evidence (tests or manual checks) and
   unresolved risks.

## Governance
<!-- Example: Constitution supersedes all other practices; Amendments require documentation, approval, migration plan -->

This constitution supersedes conflicting local workflow habits for this
repository.

Amendments MUST be proposed in writing, include impacted principle/section
changes, and document migration impact on templates and active specs.

Versioning policy:
- MAJOR: removal or incompatible redefinition of a principle/governance rule.
- MINOR: new principle/section or materially expanded mandatory guidance.
- PATCH: clarifications, wording improvements, and non-semantic edits.

Compliance review is REQUIRED in every implementation plan's Constitution Check
and during final review prior to merge.

**Version**: 1.0.0 | **Ratified**: 2026-05-11 | **Last Amended**: 2026-05-11
