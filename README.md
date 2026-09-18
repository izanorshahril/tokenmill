# Tokenmill

Tokenmill is a local-first Rust project for measuring and reducing context sent through GitHub Copilot workflows.

This repository currently contains a provisional scaffold, not a Copilot interceptor or production router.
The implementation follows the open wayfinding map in `.wayfinder/issues/` and does not claim that native Copilot model-picker routing is available.

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
cargo run -p tokenmill-cli -- acp-check <native-copilot-executable> <workspace>
cargo run -p tokenmill-cli -- acp-prompt <native-copilot-executable> <workspace> "<prompt>"
cargo run -p tokenmill-cli -- acp-context-prompt <native-copilot-executable> <workspace> <context.json> <max-tokens> [--saver on|off] [--routing on|off] [--mode strict|compatible] [--report <observation.jsonl>]
cargo fmt --all -- --check
```

The demo reports estimated local token reduction and task success separately.
The estimate is not provider billing data.
No network, source upload, telemetry export, or Copilot interception is performed.

The replay command exercises strict and compatible ACP policy locally.
The `acp-check` command launches a native ACP agent, performs `initialize`, and reports the negotiated agent and authentication methods without creating a session.
Use `acp-session-check` after authenticating the agent to also call `session/new`.
Use `acp-prompt` to send one text prompt through a live session and collect streamed agent output.
Use `acp-context-prompt` to prune an explicit local context JSON package before sending it through a live session.
Use `--saver off`, `--routing off`, or `--mode compatible` for explicit control of the run policy.
Pass `--report` to write one redacted JSONL observation without raw context.
Permission requests are cancelled by default in the non-interactive CLI.
On Windows, pass the native `copilot.exe`, not the shell, PowerShell, or batch wrapper installed on `PATH`.
The live client is an ACP prompt client, not a Copilot context interceptor.

## Provisional boundary

`tokenmill-core` owns context packages, deterministic saver behavior, measurement labels, and paired evaluation.
`tokenmill-acp` translates ACP-shaped requests into that boundary and reports structured observations.
Future Copilot, router, desktop, web, tray, and TUI integrations must adapt into this boundary rather than change the domain model.

The V1 workflow and [core/adapter boundary](.wayfinder/issues/TM-WF-0007-core-adapter-boundary.md) are now decided in [the wayfinding map](.wayfinder/issues/TM-WF-0001-tokenmill-mvp-spec-map.md).
The V1 control plane is a foreground CLI/TUI; the ACP adapter and replay harness are the next implementation slice.
Desktop, web, and tray surfaces remain deferred, and native Copilot interception remains explicitly unclaimed.

V1 observation storage is local and user-controlled.
Only redacted structured observations and aggregate counters are persisted.
Raw prompts, source, tool output, and full request/response bodies are not persisted until a future machine-learning feature creates a reviewed need for them.
