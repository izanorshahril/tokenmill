---
id: TM-WF-0012
title: Tokenmill V1 implementation and visual evidence map
kind: map
labels:
  - wayfinder:map
status: closed
parent:
blocked_by: []
assignee:
created: 2026-09-18
---

## Destination

Turn the completed V1 specification into a usable local workflow that makes saver behavior, measurement confidence, task outcome, and privacy boundaries easy to verify visually.

[The archived MVP map](TM-WF-0001-tokenmill-mvp-spec-map.md) remains the source of settled product and domain decisions.
This map tracks implementation gaps only.

## Starting evidence

- `tokenmill-core` has one deterministic pruner, heuristic token estimates, policy validation, observations, and paired evaluation primitives.
- `tokenmill-acp` can run explicit GitHub Copilot CLI/ACP prompts but does not intercept hidden Copilot context.
- The CLI TUI renders aggregate redacted history and latest-run evidence, and the explicit GitHub Copilot verification workflow is complete.
- Task success remains explicit external evidence, and exact provider billing measurement remains unverified.

## Frontier

No open child issues remain for the V1 implementation destination.
Completed tickets and their resolutions are archived in this directory.

## Resolution

The V1 implementation now has a protocol-neutral core, redacted reports and history, explicit GitHub Copilot ACP route verification, a dependency-free TUI evidence dashboard, and a live paired verification slice.
The remaining unsupported boundaries are exact provider billing measurement, hidden VS Code context capture, native interception, and automatic task-success inference.

## Working rule

Keep core contracts protocol-neutral and keep raw prompts, source, tool output, and full ACP bodies out of persisted evidence.
Do not claim hidden interception, exact billing measurement, or automatic task success without an explicit supported test.

## Out of scope

- Desktop, web, and tray shells before the CLI/TUI workflow is useful.
- A generic saver plugin framework before a second strategy has passed a paired fixture evaluation.
- Hosted telemetry or raw-content retention.