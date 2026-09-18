---
id: TM-WF-0009
title: Choose the first user surface and control plane
kind: ticket
labels:
  - wayfinder:grilling
mode: HITL
status: closed
parent: TM-WF-0001
blocked_by:
  - TM-WF-0004
  - TM-WF-0006
  - TM-WF-0007
assignee: copilot
created: 2026-09-18
---

## Question

Which combination of CLI/TUI, web, desktop, and tray controls is the first usable Tokenmill control plane, and which views are required for usage monitoring, telemetry, analytics, saver toggles, routing toggles, and simple evaluations?

Keep the decision consistent with the chosen deployment baseline and V1 workflow contract; defer surfaces that do not improve the first measurable Copilot workflow.

## Resolution

V1's first usable control plane is a Rust CLI plus TUI running as a foreground local process. It is the GNU-friendly surface for the ACP/CLI measurement path and remains easy to debug and replay.

The minimum control-plane views are integration status, saver/routing/mode toggles, paired evaluation results, and structured observations. These views cover usage monitoring and telemetry at the decided measurement boundary without prematurely adding analytics or repository graphs.

Desktop, web, and tray surfaces are deferred until the ACP measurement loop works. They remain future adapters and control surfaces rather than V1 guarantees.
