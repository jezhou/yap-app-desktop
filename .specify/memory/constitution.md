<!--
  Sync Impact Report
  ==================
  Version change: N/A → 1.0.0 (initial ratification)
  Modified principles: N/A (initial)
  Added sections:
    - Core Principles (5 principles)
    - Performance & Resource Constraints
    - Development Workflow
    - Governance
  Removed sections: N/A
  Templates requiring updates:
    - .specify/templates/plan-template.md ✅ no changes needed (generic)
    - .specify/templates/spec-template.md ✅ no changes needed (generic)
    - .specify/templates/tasks-template.md ✅ no changes needed (generic)
    - .specify/templates/agent-file-template.md ✅ no changes needed (generic)
  Follow-up TODOs: None
-->

# Yap Constitution

## Core Principles

### I. Privacy-First

All user data MUST remain on the user's local machine by default.
No telemetry, analytics, or network calls are permitted unless the
user explicitly opts in via a clear, informed consent mechanism.

- Application MUST function fully offline with zero network
  dependency for core features.
- Stored data MUST use platform-appropriate secure storage
  (e.g., Keychain, Credential Manager, libsecret).
- No user content, metadata, or usage patterns may be transmitted
  to external services without explicit user action.
- If network features are added in the future, they MUST be
  opt-in and clearly disclosed.

### II. Lightweight by Design

The application MUST maintain a minimal resource footprint
appropriate for a desktop utility.

- Binary/bundle size MUST be kept as small as reasonably possible;
  unnecessary dependencies MUST be avoided.
- Idle memory usage MUST remain low; the app MUST NOT consume
  excessive CPU when not actively in use.
- Startup time MUST feel instantaneous to the user (target
  under 2 seconds on supported platforms).
- Prefer native platform APIs and lean libraries over heavy
  frameworks when practical.

### III. Cross-Platform Compatibility

The application MUST run on macOS, Windows, and Linux from a
single codebase.

- All features MUST work consistently across supported platforms
  unless a platform genuinely lacks the required capability.
- Platform-specific code MUST be isolated behind clear
  abstraction boundaries.
- CI MUST build and test on all three target platforms.
- File paths, line endings, and OS conventions MUST be handled
  correctly without platform-specific assumptions in shared code.

### IV. Test-Driven Quality (NON-NEGOTIABLE)

Every feature MUST be accompanied by automated tests. Test
coverage is a first-class deliverable, not an afterthought.

- Unit tests MUST cover core logic and edge cases.
- Integration tests MUST verify cross-component interactions.
- Tests MUST run in CI on all supported platforms before merge.
- Regressions MUST have a corresponding test added alongside
  the fix.
- Test failures MUST block merges; green CI is a hard gate.

### V. Simplicity

Start simple. Resist premature abstraction and speculative
features.

- YAGNI: do not build features or infrastructure for hypothetical
  future requirements.
- Prefer clear, readable code over clever code.
- Three similar lines are better than a premature abstraction.
- Complexity MUST be justified in writing when introduced.

## Performance & Resource Constraints

Desktop users expect responsive, unobtrusive applications.
The following constraints apply to all features:

- **Memory**: Idle memory usage SHOULD remain under 150 MB.
- **CPU**: Background CPU usage MUST be negligible (< 1%) when
  the app is not performing active work.
- **Disk**: Local data storage MUST be bounded and manageable;
  no unbounded growth without user awareness.
- **Startup**: Cold start MUST complete in under 2 seconds on
  hardware from the last 5 years.
- **Dependencies**: Every new dependency MUST be justified.
  Prefer platform-native capabilities over third-party libraries.

## Development Workflow

All contributions MUST follow this workflow to maintain quality
and cross-platform reliability:

- **Branching**: Feature branches from `main`; pull requests
  required for all changes.
- **Testing gate**: CI MUST pass on macOS, Windows, and Linux
  before a PR can be merged.
- **Code review**: All changes require at least one review.
- **Commit hygiene**: Use conventional commits. Each commit
  SHOULD be atomic and independently buildable.
- **No secrets in source**: Environment-specific configuration
  and credentials MUST NOT be committed. Use `.env` files or
  platform-native secret storage.

## Governance

This constitution is the authoritative source for project
principles. It supersedes ad-hoc decisions and informal
conventions.

- **Amendments**: Any change to this constitution MUST be
  proposed via pull request, reviewed, and approved before merge.
  The amendment MUST include a migration plan for any affected
  code or workflows.
- **Versioning**: The constitution follows semantic versioning.
  MAJOR for principle removals or incompatible redefinitions,
  MINOR for new principles or material expansions, PATCH for
  clarifications and wording fixes.
- **Compliance**: All pull requests and code reviews MUST verify
  adherence to these principles. Violations MUST be flagged and
  resolved before merge.
- **Complexity justification**: Any deviation from the Simplicity
  principle MUST be documented with rationale and a rejected
  simpler alternative.

**Version**: 1.0.0 | **Ratified**: 2026-03-01 | **Last Amended**: 2026-03-01
