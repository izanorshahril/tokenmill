---
id: TM-WF-0008
title: Define token-saving evaluation and acceptance rules
kind: ticket
labels:
  - wayfinder:grilling
mode: HITL
status: closed
parent: TM-WF-0001
blocked_by:
  - TM-WF-0005
  - TM-WF-0007
assignee: copilot
created: 2026-09-18
---

## Question

What repeatable local task corpus, baseline/saver pairing, metrics, quality checks, thresholds, and reporting rules decide whether a Tokenmill saver is accepted for V1?

The contract must cover context-token reduction and task success together, identify which input/output/latency/telemetry values are directly observable on each integration path, and prevent a saver from being accepted solely because it removes tokens.

## Resolution

V1 uses a versioned local fixture corpus covering repository understanding, debugging, code changes, command-output handling, and test repair. Fixtures use sanitized repositories and repeatable tasks rather than live production prompts.

Every evaluation pairs saver-off and saver-on runs with the same repository, prompt, model, settings, and task. ACP/CLI is the canonical replayable path.

A saver is accepted for V1 only when it achieves at least 15% median input-context reduction, has no critical task failures, does not regress task-success rate, and does not increase latency by more than 20% unless that trade-off is explicitly accepted.

Reports keep exact, estimated, counterfactual, and unmeasured values separate. Automated checks are used where possible, with a human rubric for semantic tasks such as explanation quality and debugging correctness. Token reduction alone cannot accept a saver.
