# ACP Replay Harness

`tokenmill-acp` is the first adapter slice for Tokenmill.
It translates an ACP-shaped request into `tokenmill-core`, applies the configured saver policy, and returns a structured observation.

Run it with:

```powershell
cargo run -p tokenmill-cli -- replay
```

The replay demonstrates two cases using the same local fixture:

- Compatible mode completes an unverified route and labels the route as unverified.
- Strict mode rejects that route with `RouteUnverified`.

The replay harness does not persist raw context, contact a provider, or claim native VS Code interception.

## Live ACP process check

The adapter also contains a small newline-delimited JSON-RPC process client.
It keeps process and transport details inside `tokenmill-acp` and does not change the core saver or observation contract.

Run an initialization-only check with the native Copilot executable:

```powershell
cargo run -p tokenmill-cli -- acp-check C:\path\to\copilot.exe C:\path\to\workspace
```

After completing the agent's advertised login flow, create a session with:

```powershell
cargo run -p tokenmill-cli -- acp-session-check C:\path\to\copilot.exe C:\path\to\workspace
```

Send one text prompt through the same native ACP process with:

```powershell
cargo run -p tokenmill-cli -- acp-prompt C:\path\to\copilot.exe C:\path\to\workspace "Explain the repository entry point"
```

Send an explicit Tokenmill context package through the saver and then ACP with:

```powershell
cargo run -p tokenmill-cli -- acp-context-prompt C:\path\to\copilot.exe C:\path\to\workspace C:\path\to\context.json 1200 --saver on --routing on --mode strict --report C:\path\to\observation.jsonl
```

Run a paired live evaluation with the same context sent through saver-off and saver-on sessions:

```powershell
cargo run -p tokenmill-cli -- acp-paired-context-prompt C:\path\to\copilot.exe C:\path\to\workspace C:\path\to\context.json 1200 --mode strict --report C:\path\to\paired-observation.jsonl
```

The context document is a JSON object with an `items` array.
Each item contains `id`, `kind`, `content`, and `protected` fields.
Supported kinds are `instruction`, `prompt`, `conversation`, `repository`, `command-output`, and `tool-output`.

```json
{
	"items": [
		{"id": "instructions", "kind": "instruction", "content": "Preserve correctness.", "protected": true},
		{"id": "tool-output", "kind": "tool-output", "content": "Verbose command output.", "protected": false}
	]
}
```

The live client collects `session/update` messages, prints agent message text, and reports the final stop reason, usage-update count, and latest agent-reported context usage when available.
The reported `used` and `size` values describe what the ACP agent exposed for its context window; they are not exact provider billing tokens.
It answers `session/request_permission` with cancellation by default so a non-interactive command cannot silently authorize an agent action.
Library callers can provide an explicit permission handler with `prompt_with_permission_handler`.

The text prompt path sends the supplied text as an ACP text block.
The context prompt path serializes only the caller-supplied JSON package after local pruning.
Neither path intercepts or rewrites hidden Copilot repository context or proxies client-side tools.
When `--report` is provided, the command writes one redacted JSONL observation containing counts, route and saver status, measurement confidence, latency, outcome, usage-update count, and latest reported context usage.
The report does not contain context items, prompts, source, or raw ACP updates.
Use `--saver off` to preserve the supplied context while retaining an observation when possible.
Use `--routing off` to bypass the Tokenmill adapter policy and mark the run as bypassed.
Use `--mode compatible` to allow explicitly labelled compatible behavior instead of strict validation.
The paired live evaluation compares estimated context reduction and records each variant's redacted route, outcome, stop reason, update count, and usage summary.
It reports task success as unmeasured because a live ACP response does not prove that the developer's task succeeded.
The check reports authentication methods but intentionally does not automate terminal login.