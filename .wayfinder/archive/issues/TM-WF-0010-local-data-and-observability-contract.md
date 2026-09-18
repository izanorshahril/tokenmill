---
id: TM-WF-0010
title: Specify local data retention and observability boundaries
kind: ticket
labels:
  - wayfinder:grilling
mode: HITL
status: closed
parent: TM-WF-0001
blocked_by:
  - TM-WF-0006
  - TM-WF-0007
assignee: copilot
created: 2026-09-18
---

## Question

What raw and derived data may Tokenmill retain locally, for how long, under which redaction rules, and how can users inspect, export, delete, and compare telemetry without exposing source code or prompts by default?

Define the minimum observability needed to explain saver behavior and strict/compatible routing outcomes while preserving the local-first promise.

## Resolution

V1 retains redacted structured observations and aggregate counters only under an explicit user-controlled retention setting. Tokenmill does not impose an automatic 30-day expiry; users can inspect retention state, export selected data, and delete selected or all retained data from the CLI/TUI.

Raw prompts, source code, tool output, and full request/response bodies are not persisted in V1. They may exist in memory for the active run, but persistent raw-content capture is deferred until a concrete machine-learning feature establishes a necessary, reviewed use case and privacy contract.

Explicit export produces versioned redacted JSONL observations plus a manifest describing schema, component versions, and measurement confidence. Every observation exposes run identity/time, adapter and route status, provider/model when observable, integration mode and toggles, saver identity, before/after counts, exact/estimated/counterfactual/unmeasured status, latency, failure reasons, and task outcome. Raw content and content fingerprints are excluded by default.
