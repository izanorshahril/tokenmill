# Tokenmill Agent Guide

## Project shape

- Rust workspace using edition 2024 and the stable toolchain.
- `tokenmill-core` owns protocol-neutral context, pruning, policy, and evaluation behavior.
- `tokenmill-acp` owns ACP transport, process lifecycle, replay, streaming updates, and permissions.
- `tokenmill-cli` owns explicit user-facing workflows and redacted observation reports.
- Read `CONTEXT.md` before changing domain terms or integration boundaries.

## Working rules

- Keep the core independent from ACP, Copilot, desktop, web, tray, and TUI concerns.
- Treat native Copilot interception and model-picker routing as unsupported unless verified by an explicit test.
- Reuse Copilot's own authentication context; never import or persist credentials from another tool.
- Do not persist raw prompts, source, tool output, or full request and response bodies.
- Preserve strict versus compatible policy behavior and label unmeasured or unverified runs honestly.
- Prefer focused tests, then run `cargo fmt --all -- --check` and `cargo test --workspace`.

## Codemapping

Graphify is optional development tooling, not a Tokenmill runtime or Cargo dependency.
Use it only when a task needs repository-wide topology or cross-file relationship discovery:

```powershell
uvx --from graphifyy graphify extract . --code-only --no-viz
uvx --from graphifyy graphify query "<focused architecture question>" --graph graphify-out/graph.json --budget 800
```

Code-only extraction is local and does not require an LLM API key.
Do not run Graphify for routine file-local changes, and do not enable document or media extraction without an explicit privacy decision.
Prefer focused Graphify queries over reading the entire generated report.
Keep query budgets bounded, and use source-file reads only to verify the small set of nodes returned.

## Validation and changes

- Keep changes narrow and preserve existing public APIs unless the task requires otherwise.
- Add or update tests for behavior changes.
- Do not commit generated build output, Graphify output, credentials, or local reports.