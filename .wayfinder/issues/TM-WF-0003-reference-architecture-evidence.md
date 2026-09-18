---
id: TM-WF-0003
title: Establish reference behavior from 9router and Headroom
kind: ticket
labels:
  - wayfinder:research
mode: AFK
status: closed
parent: TM-WF-0001
blocked_by: []
assignee: copilot
created: 2026-09-18
---

## Question

What behavior and boundaries should Tokenmill borrow, reject, or adapt from 9router and Headroom for model routing, context reduction, provider compatibility, observability, configuration, and user control?

Inspect the referenced repositories and their primary documentation/source. Separate verified behavior from marketing claims, and identify the smallest reference concepts that matter to a Copilot-focused MVP.

## Research asset

[Research: Establish reference behavior from 9router and Headroom](../research/TM-WF-0003-reference-architecture-evidence.md)

## Resolution

Tokenmill should separate routing, provider fallback, and authentication from context saving and evaluation. Headroom is the closer reference for transparent compression that preserves provider model selection; 9Router is the closer reference for gateway routing, fallback, usage accounting, and user bypass controls.

V1 observability must distinguish exact usage, estimates, counterfactual savings, and missing data. V1 privacy defaults should be stricter than either reference: no external telemetry and no full-content logging by default.

Evidence: [research asset](../research/TM-WF-0003-reference-architecture-evidence.md)
