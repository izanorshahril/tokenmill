---
id: TM-WF-0007
title: Define the Tokenmill core and adapter boundary
kind: ticket
labels:
  - wayfinder:grilling
mode: HITL
status: closed
parent: TM-WF-0001
blocked_by:
  - TM-WF-0002
  - TM-WF-0003
  - TM-WF-0004
assignee: copilot
created: 2026-09-18
---

## Question

What stable product boundary should the MVP specification promise between the Rust core and adapters for Copilot surfaces, routers/providers, repository analysis, saver methods, evaluation, and user interfaces?

Choose the smallest boundary that supports the Copilot-focused V1, strict and compatible modes, later farseer/peon integration, and future non-Copilot harnesses without coupling the domain model to undocumented Copilot internals.

## Resolution

`tokenmill-core` remains protocol-neutral. It owns context packages, saver transforms, measurement labels, evaluation, explicit run policy, and structured observations. It must not depend on ACP, Copilot, VS Code, router/provider SDKs, or UI frameworks.

Adapters translate external protocols into the core types and own process, network, editor, and provider side effects. The first real adapter is ACP, developed with a mock/replay harness before connecting to Copilot CLI. VS Code, desktop, web, tray, router, farseer, peon, and future harness integrations consume the same adapter contract.

The core's policy input includes saver enabled, routing enabled, strict or compatible mode, and measurement requirements. Observations include route status, saver identity, before/after counts, measurement status, and failure reasons; raw prompts and source remain opt-in and local-only.
