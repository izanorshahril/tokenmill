# V1 GitHub Copilot verification slice

## Command

The live check used the explicitly identified GitHub Copilot CLI/ACP executable:

```powershell
cargo run -p tokenmill-cli -- acp-paired-context-prompt C:\Users\izanorshahril\AppData\Local\github-copilot-sdk\cli\1.0.71\copilot.exe C:\Work\Project_Personal\tokenmill <temporary-synthetic-context.json> 1200 --mode strict --task-success unknown --report <temporary-report.jsonl> --history <temporary-history.jsonl>
```

The temporary context contained only synthetic protected instructions, a synthetic prompt, and unprotected synthetic repository/tool/conversation items.
The context package, report, and history were deleted after verification.
The committed redacted result is available at `TM-WF-0018-live-history.jsonl`.

## Redacted result

- Route: verified GitHub Copilot ACP; negotiated provider name `Copilot`.
- Mode: strict.
- Both saver-off and saver-on variants completed with stop reason `end_turn`.
- Both variants reported `estimated` measurement and `126` estimated tokens before and after transformation.
- Estimated savings: `0` tokens and `0.0%`; the deterministic pruner had no eligible item to remove in this package.
- Saver-on: `deterministic-pruner`.
- Task success: unknown with source `unknown`; acceptance remained unknown rather than being inferred.
- ACP usage: one update per variant, with `16276/272000` and `16277/272000` respectively.
- The TUI rendered two redacted completed records as `Runs 2`, `Unknown 2`, `Tokens saved 0`, `VERIFIED`, `STRICT`, `ESTIMATED`, and `not billing tokens`.

## Boundaries

ACP context-window usage is supplemental agent-reported evidence, not exact provider billing measurement.
The local token count remains a heuristic estimate.
The run does not intercept hidden VS Code context, rewrite native Copilot routing, or infer task success from an ACP response.
The local replay command continues to demonstrate the distinction between compatible unverified routing and strict rejection.
No raw prompt, source, tool output, ACP update body, credentials, or temporary artifact remains persisted by this check.
