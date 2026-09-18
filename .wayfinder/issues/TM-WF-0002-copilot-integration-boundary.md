---
id: TM-WF-0002
title: Verify GitHub Copilot integration and model-picker boundary
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

Which GitHub Copilot surfaces and supported protocols can Tokenmill observe, transform, route, and measure in the target workplace environment, including VS Code, Copilot CLI, Copilot desktop, and Agent Client Protocol (ACP), and can any of them expose a Tokenmill-routed model/provider in the native model picker without unsupported interception?

Use first-party GitHub, GitHub Copilot, VS Code, and ACP documentation or source. Distinguish documented extension points from assumptions, and record strict-mode failure behavior plus any compatible workflow that can be honestly measured.

## Research asset

[Research: Verify GitHub Copilot integration and model-picker boundary](../research/TM-WF-0002-copilot-integration-boundary.md)

## Resolution

Tokenmill must not assume that it can inject a provider into GitHub Copilot's private native model catalog. VS Code exposes a generic language-model provider boundary, and Copilot CLI exposes custom providers, model selection, hooks, and MCP; ACP transports sessions but does not define provider discovery.

Strict mode requires a verified supported path, known transformed context, route identity, lifecycle evidence, and provider-supplied usage where available. Compatible mode may use Copilot CLI or ACP-compatible paths, but missing usage is reported as unmeasured rather than zero.

Evidence: [research asset](../research/TM-WF-0002-copilot-integration-boundary.md)
