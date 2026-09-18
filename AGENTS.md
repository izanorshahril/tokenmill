# Tokenmill Agent Guide

## Project shape

- Rust workspace using edition 2024 and the stable toolchain.
- `tokenmill-core` owns protocol-neutral context, pruning, policy, and evaluation behavior.
- `tokenmill-acp` owns ACP transport, process lifecycle, replay, streaming updates, and permissions.
- `tokenmill-cli` owns explicit user-facing workflows and redacted observation reports.
- Read `CONTEXT.md` before changing domain terms or integration boundaries.

## Working rules

- Keep the core independent from ACP, Copilot, desktop, web, tray, and TUI concerns.
- Tokenmill's product focus is GitHub Copilot; do not use Microsoft Copilot or an ambiguous `copilot.exe` reference when naming or validating the target.
- Live ACP examples must identify the GitHub Copilot CLI/ACP executable explicitly; a filename alone does not establish product identity.
- Treat GitHub Copilot interception and native model-picker routing as unsupported unless verified by an explicit test.
- Reuse Copilot's own authentication context; never import or persist credentials from another tool.
- Do not persist raw prompts, source, tool output, or full request and response bodies.
- Preserve strict versus compatible policy behavior and label unmeasured or unverified runs honestly.
- Prefer focused tests, then run `cargo fmt --all -- --check` and `cargo test --workspace`.

## Codemapping

Graphify is optional development tooling, not a Tokenmill runtime or Cargo dependency.
Use Graphify first for repository-wide topology, architecture, ownership, and cross-file relationship questions instead of broad grep, glob, or whole-file reads:

```powershell
uvx --from graphifyy graphify extract . --code-only --no-viz
uvx --from graphifyy graphify query "<focused architecture question>" --graph graphify-out/graph.json --budget 800
```

Code-only extraction is local and does not require an LLM API key.
Run the extraction once per working session when the map is absent or stale, then use focused queries rather than reading the entire generated report.
Use exact symbol or literal search, targeted source reads, compiler diagnostics, and tests after Graphify has narrowed the relevant files or when the question is inherently file-local.
Do not enable document or media extraction without an explicit privacy decision.
Keep query budgets bounded and verify returned nodes against source before making changes.

## Future codemap integration

The current Graphify workflow is a useful prototype for a future optional Tokenmill codemap adapter.
That adapter should expose a local derived index for context selection and architecture queries, while keeping source files as the source of truth.
It should be pluggable, cacheable, invalidated when inputs change, and disabled unless the user opts in.
It must not become a Cargo runtime dependency, upload code or prompts, or silently add graph data to Copilot requests.
The first useful product seam would be an optional `tokenmill map` developer command that invokes a configured local provider such as Graphify and reports provenance for every selected file or symbol.

## Validation and changes

- Keep changes narrow and preserve existing public APIs unless the task requires otherwise.
- Add or update tests for behavior changes.
- Do not commit generated build output, Graphify output, credentials, or local reports.