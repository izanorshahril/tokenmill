---
id: TM-WF-0006
title: Set the V1 Copilot workflow and compatibility contract
kind: ticket
labels:
  - wayfinder:grilling
mode: HITL
status: closed
parent: TM-WF-0001
blocked_by:
  - TM-WF-0002
  - TM-WF-0004
assignee: copilot
created: 2026-09-18
---

## Question

After the integration and deployment facts are known, which Copilot workflows and surfaces are in V1, what does a user explicitly toggle, and what observable guarantees separate strict mode from compatible mode when native model-picker routing is unavailable?

The answer must define supported versus unsupported paths, the meaning of "on" and "off," what is measured in each path, and what the user sees when Tokenmill cannot verify or intercept a request.

## Resolution

V1 supports both the VS Code GitHub Copilot workflow and Copilot CLI through ACP, but ACP/CLI is the canonical measurable reference path and VS Code is initially a smoke-test path.

Tokenmill exposes separate saver and routing/integration toggles. Saver-off bypasses context transformation while preserving observation where possible; routing-off bypasses Tokenmill entirely.

Strict mode fails closed when Tokenmill cannot verify the requested integration or measurement boundary. Compatible mode continues through an explicitly labelled supported fallback. The UI reports observed, estimated, and unmeasured values separately and never treats missing usage as zero.

V1 does not guarantee that Tokenmill appears in the GitHub Copilot native model picker. It preserves the host-selected model and reports provider/model identity when observable.
