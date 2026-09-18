# Tokenmill

Tokenmill is a local-first Rust project for measuring and reducing context sent through GitHub Copilot workflows.

This repository currently contains a provisional scaffold, not a Copilot interceptor or production router.
The implementation follows the open wayfinding map in `.wayfinder/issues/` and does not claim that GitHub Copilot native model-picker routing is available.

## Workspace

```text
crates/
  tokenmill-core/    Protocol-neutral context model, deterministic saver, and evaluation primitives
  tokenmill-acp/     ACP boundary model, replay harness, and stdio process client
  tokenmill-cli/     Offline CLI and replay commands
.wayfinder/          Local planning map, tickets, and research assets
CONTEXT.md           Domain glossary
```

## Run

The workspace uses a small pinned dependency set and should run with the installed GNU Rust toolchain:

```powershell
cargo test --workspace
cargo run -p tokenmill-cli -- demo
cargo run -p tokenmill-cli -- replay
cargo run -p tokenmill-cli -- acp-check <github-copilot-acp-executable> <workspace>
cargo run -p tokenmill-cli -- acp-prompt <github-copilot-acp-executable> <workspace> "<prompt>"
cargo run -p tokenmill-cli -- acp-context-prompt <github-copilot-acp-executable> <workspace> <context.json> <max-tokens> [--saver on|off] [--routing on|off] [--mode strict|compatible] [--report <observation.jsonl>]
cargo run -p tokenmill-cli -- acp-paired-context-prompt <github-copilot-acp-executable> <workspace> <context.json> <max-tokens> [--mode strict|compatible] [--task-success pass|fail|unknown] [--report <paired-observation.jsonl>] [--history <evaluation-history.jsonl>]
cargo run -p tokenmill-cli -- eval-history <evaluation-history.jsonl>
cargo fmt --all -- --check
```

The demo reports estimated local token reduction and task success separately.
The estimate is not provider billing data.
No network, source upload, telemetry export, or Copilot interception is performed.

The replay command exercises strict and compatible ACP policy locally.
The `acp-check` command launches the GitHub Copilot ACP executable, performs `initialize`, and reports the negotiated agent and authentication methods without creating a session.
Use `acp-session-check` after authenticating the agent to also call `session/new`.
Use `acp-prompt` to send one text prompt through a live session and collect streamed agent output.
Use `acp-context-prompt` to prune an explicit local context JSON package before sending it through a live session.
Use `acp-paired-context-prompt` to send the same explicit context through saver-off and saver-on sessions and compare estimated context reduction.
Use `--saver off`, `--routing off`, or `--mode compatible` for explicit control of the run policy.
Pass `--report` to write one redacted JSONL observation without raw context.
Live ACP output and reports include the latest agent-reported context usage when available (`used` and `size`), but these values are not exact provider billing data.
The paired live evaluation accepts explicit post-run evidence with `--task-success pass|fail|unknown`; it defaults to unknown because ACP cannot infer whether the developer's task succeeded from a prompt response alone.
Pass `--history` to append the redacted paired result to a local JSONL history file without storing raw context or ACP updates.
Use `eval-history <path>` to summarize accepted, rejected, and unknown runs together with estimated savings and average reduction.
History parsing is strict: malformed, unrelated, or unsupported-schema lines fail instead of being silently counted.
Permission requests are cancelled by default in the non-interactive CLI.
On Windows, pass the executable used by GitHub Copilot CLI's ACP mode, not the Microsoft Copilot executable, shell, PowerShell, or a batch wrapper installed on `PATH`.
Do not infer product identity from a filename such as `copilot.exe`; verify the negotiated GitHub Copilot ACP agent with `acp-check`.
The live client is an ACP prompt client, not a Copilot context interceptor.

## Provisional boundary

`tokenmill-core` owns context packages, deterministic saver behavior, measurement labels, and paired evaluation.
`tokenmill-acp` translates ACP-shaped requests into that boundary and reports structured observations.
Future Copilot, router, desktop, web, tray, and TUI integrations must adapt into this boundary rather than change the domain model.

The V1 workflow and [core/adapter boundary](.wayfinder/issues/TM-WF-0007-core-adapter-boundary.md) are now decided in [the wayfinding map](.wayfinder/issues/TM-WF-0001-tokenmill-mvp-spec-map.md).
The V1 control plane is a foreground CLI/TUI; the ACP adapter and replay harness are the next implementation slice.
Desktop, web, and tray surfaces remain deferred, and GitHub Copilot interception remains explicitly unclaimed.

V1 observation storage is local and user-controlled.
Only redacted structured observations and aggregate counters are persisted.
Raw prompts, source, tool output, and full request/response bodies are not persisted until a future machine-learning feature creates a reviewed need for them.
