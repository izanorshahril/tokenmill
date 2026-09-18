---
id: TM-WF-0013
title: Harden the redacted core and presentation contract
kind: task
labels:
  - wayfinder:task
status: closed
parent: TM-WF-0012
blocked_by: []
assignee:
created: 2026-09-18
---

## Goal

Define the small stable data contract consumed by reports and the TUI without coupling `tokenmill-core` to terminal, ACP, or desktop concerns.

## Scope

- Preserve explicit estimated, exact, and unmeasured states.
- Expose redacted run summaries for route, saver, measurement, outcome, task evidence, and reduction.
- Keep unknown and unverified values distinct from failure.
- Add focused edge-case tests for empty input, protected over-budget input, zero baselines, and malformed history.

## Acceptance

- The TUI and history summary consume the same typed or validated redacted view contract.
- No raw context or ACP update body crosses the presentation boundary.
- Existing workspace tests and formatting checks pass.
- The contract is documented in the architecture notes.

## Resolution

Implemented the validated redacted view contract in `crates/tokenmill-cli/src/view.rs`.
History parsing now produces one `RedactedRunView` per paired record and one shared `EvaluationHistorySummary` for both `eval-history` and the TUI.
The contract preserves unknown task evidence, unverified routes, measurement confidence, outcome, saver, reduction, and optional agent-reported usage without carrying raw content.
Focused and full CLI tests pass.