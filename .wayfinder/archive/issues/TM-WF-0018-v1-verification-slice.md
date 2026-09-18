---
id: TM-WF-0018
title: Prove the V1 GitHub Copilot verification slice
kind: task
labels:
  - wayfinder:task
status: closed
parent: TM-WF-0012
blocked_by:
  - TM-WF-0014
  - TM-WF-0015
  - TM-WF-0017
assignee:
created: 2026-09-18
---

## Goal

Connect one explicit GitHub Copilot CLI/ACP run to redacted history and the visual dashboard so the user can verify the evidence end to end.

## Scope

- Use an explicitly identified GitHub Copilot CLI/ACP executable.
- Run saver-off and saver-on for the same caller-supplied context package.
- Persist only the redacted paired result and open it in the TUI.
- Show route identity, measurement confidence, task evidence, and reduction together.
- Document the exact unsupported boundaries encountered during the run.

## Acceptance

- The workflow has a reproducible command sequence and a captured redacted example.
- The dashboard agrees with the JSONL summary and does not overstate what ACP measured.
- Strict and compatible behavior remain distinguishable.
- The end-to-end validation passes without persisting raw content.

## Resolution

Ran a strict paired evaluation with the explicitly identified GitHub Copilot CLI/ACP executable at `C:\Users\izanorshahril\AppData\Local\github-copilot-sdk\cli\1.0.71\copilot.exe` and a temporary synthetic context package.
Both saver-off and saver-on variants completed on the verified route with `estimated` measurement; the package produced `126 -> 126` estimated tokens and zero savings because the deterministic pruner had no eligible item to remove.
The redacted history rendered successfully in the TUI with verified route, strict mode, unknown task evidence, and agent-reported `16276/272000` or `16277/272000` usage explicitly labeled as non-billing context-window data.
Exact billing measurement, hidden VS Code context capture, native interception, and automatic task success remain unsupported and are recorded in [Prove the V1 GitHub Copilot verification slice](../../research/TM-WF-0018-v1-verification-slice.md).