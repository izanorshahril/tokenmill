---
id: TM-WF-0016
title: Evaluate the next explainable saver strategy
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

Evaluate one additional explainable context-saving strategy against the deterministic pruner without introducing a generic plugin framework prematurely.

## Scope

- Choose the next strategy from a concrete fixture need, such as structured compaction or kind-aware summarization.
- Define what information may be removed and how dropped or transformed content is reported.
- Compare the strategy with saver-off and the current deterministic pruner on local paired fixtures.
- Keep all transformations local and redacted in reports.

## Acceptance

- The strategy has a focused implementation and deterministic tests, or a documented evidence-based rejection.
- Paired results report reduction, task outcome, measurement confidence, and failure states.
- No new abstraction is added solely for hypothetical future savers.

## Resolution

Implemented `ConsecutiveOutputCompactor` in `tokenmill-core` for repeated lines and repeated blank lines in unprotected command/tool output.
The saver reports transformed item ids and remains measured as estimated.
A paired local fixture proves positive savings where the deterministic pruner saves zero; the ACP default remains the deterministic pruner until broader evidence justifies promotion.