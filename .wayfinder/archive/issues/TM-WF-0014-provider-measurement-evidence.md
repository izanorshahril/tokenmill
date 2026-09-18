---
id: TM-WF-0014
title: Verify provider measurement boundaries
kind: research
labels:
  - wayfinder:research
status: closed
parent: TM-WF-0012
blocked_by: []
assignee:
created: 2026-09-18
---

## Goal

Determine which context-usage values the supported GitHub Copilot CLI/ACP path exposes and which values Tokenmill must continue to label estimated or unmeasured.

## Scope

- Test the current ACP usage updates with an explicitly identified GitHub Copilot CLI/ACP executable.
- Separate agent-reported context-window usage from provider billing tokens.
- Record the negative result if exact billing measurement or hidden VS Code context capture is unsupported.
- Recommend the smallest core and report changes justified by the evidence.

## Acceptance

- A redacted evidence note records the executable identity, observed fields, and measurement confidence.
- No claim of exact billing measurement or hidden interception is made without a repeatable test.
- Any implementation change has focused tests and preserves strict versus compatible behavior.

## Resolution

Ran the supported GitHub Copilot CLI/ACP executable through `acp-check` and `acp-context-prompt`.
The process reported ACP protocol `1`, agent `Copilot`, version `1.0.86`, and one usage update of `16189 / 272000`.
The GitHub-specific executable path verified the route, while Tokenmill's local `18 -> 18` estimate remained `estimated` and task success remained unknown.
Exact billing tokens, hidden VS Code context, native interception, and automatic task success remain unmeasured.
Evidence: [Verify provider measurement boundaries](../../research/TM-WF-0014-provider-measurement-evidence.md).