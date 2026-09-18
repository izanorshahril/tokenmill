---
id: TM-WF-0015
title: Model task-success evidence without false certainty
kind: task
labels:
  - wayfinder:task
status: closed
parent: TM-WF-0012
blocked_by:
  - TM-WF-0013
assignee:
created: 2026-09-18
---

## Goal

Make paired evaluation results useful while keeping task success explicit when ACP cannot infer whether the developer's task was completed correctly.

## Scope

- Preserve `pass`, `fail`, and `unknown` as distinct evidence states.
- Add a redacted evidence source or reason where it can be collected safely.
- Prevent unknown runs from being presented as accepted.
- Keep manual evidence usable before any automatic evaluator exists.

## Acceptance

- Reports and the TUI show task-success state and evidence confidence separately from token reduction.
- Unknown or missing evidence cannot produce an accepted evaluation.
- Tests cover pass, fail, unknown, and malformed evidence.

## Resolution

Paired reports now persist `task_success_source` as `manual` for explicit pass/fail input and `unknown` when no task result is supplied.
The redacted view contract validates the source against the nullable task result and preserves unknown evidence as distinct from rejection or failure.
Unknown runs remain unaccepted.
CLI tests cover serialized provenance and history parsing.