---
id: TM-WF-0001
title: Tokenmill MVP product and technical specification map
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

An implementation-ready MVP product and technical specification for Tokenmill: a local-first Rust-based tool that measurably reduces GitHub Copilot request context while preserving task success, exposes strict and compatible integration behavior, and defines the first supported user workflows and observability boundary.

## Notes

Domain: developer tooling, AI request-context efficiency, GitHub Copilot ecosystem, local privacy, and evaluation.

Consult `research`, `grilling`, and `domain-modeling` for decisions; use `prototype` when a UI or state-model question needs a concrete artifact.

Standing preferences: GNU-compatible Rust toolchain in the corporate workspace; no silent fallback; every fallback and measurement boundary must be observable; V1 targets the GitHub Copilot ecosystem; later harness integrations remain separate.

Execution note: the user explicitly authorized a provisional local Rust scaffold before the map decisions are closed. The scaffold is evidence for the open decisions, not a resolved product specification or production integration.

## Decisions so far

<!-- Closed tickets only. Open child issues are discovered from frontmatter, not listed here. -->

- [Verify GitHub Copilot integration and model-picker boundary](TM-WF-0002-copilot-integration-boundary.md): GitHub Copilot native model-picker injection is unverified; strict and compatible paths must report their measurement boundary.
- [Establish reference behavior from 9router and Headroom](TM-WF-0003-reference-architecture-evidence.md): keep routing separate from saving and use stricter local-first observability defaults.
- [Choose the Rust toolchain and deployment baseline](TM-WF-0004-rust-toolchain-and-deployment-matrix.md): use GNU Rust for the core/CLI profile and treat MSVC desktop support as separate.
- [Build an evidence-backed saver and steering taxonomy](TM-WF-0005-saver-technique-taxonomy.md): prioritize deterministic local savers; keep ML, pruning, steering, and ambiguous techniques experimental or unresolved.
- [Set the V1 Copilot workflow and compatibility contract](TM-WF-0006-v1-copilot-workflow-boundary.md): support VS Code and Copilot CLI through ACP, use ACP/CLI for measurement, separate saver/routing toggles, and fail closed in strict mode.
- [Define the Tokenmill core and adapter boundary](TM-WF-0007-core-adapter-boundary.md): keep the core protocol-neutral, make policy explicit, use ACP as the first adapter, and emit structured privacy-preserving observations.
- [Define token-saving evaluation and acceptance rules](TM-WF-0008-evaluation-and-acceptance-contract.md): use paired local fixtures, require meaningful reduction without task or latency regressions, and keep measurement confidence explicit.
- [Choose the first user surface and control plane](TM-WF-0009-first-user-surface-and-control-plane.md): use a foreground GNU-friendly CLI/TUI with status, toggles, evaluations, and observations; defer desktop, web, and tray.
- [Specify local data retention and observability boundaries](TM-WF-0010-local-data-and-observability-contract.md): use user-controlled retention for redacted observations, persist no raw content in V1, and expose inspect/export/delete plus structured measurement fields.

## Not yet specified

- No further decisions remain within the V1 MVP-specification destination.

## Out of scope

- Full product implementation and production integrations while this map is being charted; the user-authorized provisional scaffold is limited to evidence for the open decisions.
- Hosted telemetry or analytics in V1; the product remains local-first with explicit export only.
- Direct V1 integration with farseer or peon-py; those are later integration targets.
- Non-Copilot harnesses such as Codex until a later effort redraws the destination.
- Post-V1 desktop, web, tray, multi-surface packaging, and cross-harness product shape; these require a new destination and map.
