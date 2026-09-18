---
id: TM-WF-0017
title: Build the visual evidence dashboard workflow
kind: task
labels:
  - wayfinder:task
status: closed
parent: TM-WF-0012
blocked_by:
  - TM-WF-0013
  - TM-WF-0015
assignee:
created: 2026-09-18
---

## Goal

Let a user quickly verify one Tokenmill run visually from redacted local evidence.

## Scope

- Extend the dependency-free TUI with latest-run details, route identity, saver state, measurement confidence, task-success evidence, and acceptance reason.
- Make unverified, unmeasured, unknown, rejected, and failed states visually distinct.
- Keep `--once` deterministic for screenshots or terminal capture and keep refresh/help/quit controls working.
- Add a small fixture history that demonstrates accepted, rejected, unknown, and unverified cases without raw content.

## Acceptance

- One documented command renders a scannable dashboard containing enough evidence to explain why a run was or was not accepted.
- The dashboard never displays raw prompts, source, tool output, or ACP update bodies.
- CLI tests cover the fixture states and the non-interactive render.
- The UI reports deferred capabilities honestly instead of showing inactive or speculative controls.

## Resolution

The dependency-free TUI now renders a latest-run evidence section with route identity, mode, saver, outcome, measurement confidence, task evidence, acceptance, estimated counts, supplemental ACP usage, and failure reason.
Unverified, unmeasured, unknown, rejected, and failed states are rendered explicitly.
The redacted fixture at `docs/fixtures/visual-evidence-history.jsonl` demonstrates accepted, rejected, unknown, unverified, unmeasured, and failed cases.
Use `cargo run -p tokenmill-cli -- tui docs/fixtures/visual-evidence-history.jsonl --once` for a deterministic visual check.