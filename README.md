# Tokenmill

Tokenmill is a local-first Rust project for measuring and reducing context sent through GitHub Copilot workflows.

This repository currently contains a provisional scaffold, not a Copilot interceptor or production router.
The completed V1 implementation is recorded in the archived map under `.wayfinder/archive/issues/` and does not claim that GitHub Copilot native model-picker routing is available.

## Workspace

```text
crates/
  tokenmill-core/    Protocol-neutral context model, deterministic saver, and evaluation primitives
  tokenmill-acp/     ACP boundary model, replay harness, and stdio process client
  tokenmill-cli/     CLI, local TUI, shared settings, and Windows tray adapter
.wayfinder/          Archived V1 implementation record, tickets, and research assets
  archive/issues/    Closed specification, implementation, and decision tickets
  issues/            Active tray control-plane map and implementation ticket
  research/          Redacted provider and live verification evidence
  TRACKER.md         Local Wayfinder tracker rules
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
cargo run -p tokenmill-cli -- tui <evaluation-history.jsonl>
cargo run -p tokenmill-cli -- tui docs/fixtures/visual-evidence-history.jsonl --once
cargo run -p tokenmill-cli -- tray
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
For `acp-context-prompt`, routing OFF skips submission without reading context, starting ACP, or writing a report, then exits successfully with an explicit notice.
Continue in GitHub Copilot directly when bypassing Tokenmill; no request is forwarded automatically.
Pass `--report` to write one redacted JSONL observation without raw context.
Live ACP output and reports include the latest agent-reported context usage when available (`used` and `size`), but these values are not exact provider billing data.
The paired live evaluation accepts explicit post-run evidence with `--task-success pass|fail|unknown`; it defaults to unknown because ACP cannot infer whether the developer's task succeeded from a prompt response alone.
Pass `--history` to append the redacted paired result to a local JSONL history file without storing raw context or ACP updates.
Use `eval-history <path>` to summarize accepted, rejected, and unknown runs together with estimated savings and average reduction.
Use `tui <path>` to open the first Tokenmill user interface: a local ANSI terminal dashboard over the same redacted history.
The TUI supports `r` to refresh, `h` for controls, and `q` to quit, each followed by Enter; add `--once` for a non-interactive render.
The redacted visual evidence fixture demonstrates accepted, rejected, unknown, and unverified states without raw content.
History parsing is strict: malformed, unrelated, or unsupported-schema lines fail instead of being silently counted.
Permission requests are cancelled by default in the non-interactive CLI.
On Windows, pass the executable used by GitHub Copilot CLI's ACP mode, not the Microsoft Copilot executable, shell, PowerShell, or a batch wrapper installed on `PATH`.
Do not infer product identity from a filename such as `copilot.exe`.
For GitHub Copilot CLI versions that report the generic ACP name `Copilot`, strict verification also requires a GitHub-specific executable path such as the GitHub Copilot SDK or GitHub CLI installation path.
An explicit negotiated identity of `GitHub Copilot` or `GitHub Copilot CLI` is accepted when the executable path is not clearly identified as Microsoft Copilot.
The live client is an ACP prompt client, not a Copilot context interceptor.

## Provisional boundary

`tokenmill-core` owns context packages, deterministic saver behavior, measurement labels, and paired evaluation.
`tokenmill-acp` translates ACP-shaped requests into that boundary and reports structured observations.
Future Copilot, router, desktop, web, and tray integrations must adapt into this boundary rather than change the domain model.

The settled V1 workflow and [Define the Tokenmill core and adapter boundary](.wayfinder/archive/issues/TM-WF-0007-core-adapter-boundary.md) are recorded in the [Tokenmill MVP product and technical specification map](.wayfinder/archive/issues/TM-WF-0001-tokenmill-mvp-spec-map.md).
Completed V1 implementation decisions and evidence are recorded in the [Tokenmill V1 implementation and visual evidence map](.wayfinder/archive/issues/TM-WF-0012-tokenmill-v1-implementation-map.md).
The V1 control plane is a foreground CLI/TUI, with the ACP adapter and replay harness implemented as the first integration slice.
Desktop, web, and tray surfaces were deferred in the archived V1 scope, and GitHub Copilot interception remains explicitly unclaimed.
The next [tray control-plane map](.wayfinder/issues/TM-WF-0019-tray-control-plane-map.md) defines visible state and easy toggles for explicit Tokenmill requests.
Windows tray and shared saved policy are implemented; core menu interactions have been verified on Windows, with broader DPI/theme acceptance still open.
The current TUI only displays history.

## Tray controls

Double-click `Start-Tokenmill.cmd` from the repository root to open Tokenmill controls.
It uses the existing local build, or builds offline with Cargo when binaries are absent; after source changes, quit the tray and run `cargo build --locked --offline -p tokenmill-cli --bins` to refresh it.
Alternatively, double-click `target/debug/tokenmill-tray.exe` or run `cargo run -p tokenmill-cli -- tray`.
The native WinForms GUI is compiled locally using Windows' installed .NET Framework 4 compiler, with no downloaded GUI dependency.
The controls window opens the same menu as the notification icon: Routing, Saver, Strict/Compatible mode, Open history, and Quit.
Hide to tray closes the window without stopping the app; launching again reopens the existing controls window.
Its scope is **`acp-context-prompt` only**; raw `acp-prompt`, paired evaluations, and GitHub Copilot requests outside Tokenmill are unaffected.
Routing OFF skips submission; saver OFF preserves context; READY means locally ready, not verified connectivity.
RUNNING reflects one tracked request, while displayed settings apply to the next request; toggles do not cancel work in flight.
Quit closes the tray but preserves saved policy; turn Routing OFF separately to disable future managed submissions.

First tray launch saves routing OFF, saver ON, and strict mode in `%LOCALAPPDATA%\tokenmill\settings.json`.
Set `TOKENMILL_HOME` to use another local directory; headless non-Windows settings use `$XDG_CONFIG_HOME/tokenmill` or `$HOME/.config/tokenmill`.
Without a settings file, existing CLI defaults remain unchanged; explicit context-command flags override saved settings for that invocation.
Use `tokenmill settings show`, `tokenmill settings init`, or `tokenmill settings set routing off` for scriptable control.
Invalid settings fail visibly; writes use a file lock and atomic replacement, and duplicate launches focus existing controls.
Settings contain policy only; lock files carry no request content, and temporary GUI source is removed after compilation.
The generated native GUI executable remains beside settings as local runtime output.
`last-request.json` stores only outcome, route-status labels, and policy snapshot; an interrupted or failed request stays explicitly incomplete and unverified.
The tray adds no startup registration; the headless CLI does not require .NET or a GUI runtime.

V1 observation storage is local and user-controlled.
Only redacted structured observations and aggregate counters are persisted.
Raw prompts, source, tool output, and full request/response bodies are not persisted until a future machine-learning feature creates a reviewed need for them.
