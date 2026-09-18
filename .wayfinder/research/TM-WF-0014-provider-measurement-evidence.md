# Provider Measurement Evidence

Date: 2026-09-18

## Run

The live check used the explicitly identified GitHub Copilot CLI/ACP executable:

`C:\Users\izanorshahril\AppData\Local\github-copilot-sdk\cli\1.0.71\copilot.exe`

The executable was launched through Tokenmill's `acp-check` and `acp-context-prompt` commands from the repository workspace.

## Observed evidence

| Field | Observed value | Interpretation |
| --- | --- | --- |
| ACP protocol version | `1` | The process completed ACP initialization. |
| ACP agent name | `Copilot` | Generic name; GitHub identity was verified by the recognized GitHub SDK path marker. |
| ACP agent version | `1.0.86` | Version reported by the running process. |
| Authentication methods | `copilot-login` | The process exposed its own GitHub Copilot login method. |
| Route status | `verified` | Verified for this explicit GitHub Copilot CLI/ACP process only. |
| Usage updates | `1` | One ACP usage update was exposed. |
| Latest reported usage | `16189 / 272000` | Agent-reported context-window usage, not billing tokens. |
| Tokenmill local estimate | `18 -> 18` | Heuristic estimate; no reduction occurred for this tiny fixture. |
| Tokenmill measurement | `estimated` | The local estimate is intentionally not provider-exact. |
| Task success | `null` | ACP output did not prove developer task success. |

The redacted live report contained only route, estimate, outcome, measurement, and usage counters. The temporary context and report were deleted after the run.

## Measurement boundary

Tokenmill may display ACP `used` and `size` as agent-reported context-window usage when present.
Those fields must remain separate from `before_estimated_tokens`, `after_estimated_tokens`, and reduction calculations.

The run did not measure:

- Provider billing input, output, or cache tokens.
- The exact serialized request sent after all provider-side assembly.
- Hidden repository context assembled by VS Code GitHub Copilot.
- Native VS Code model-picker routing or interception.
- Automatic task success.

The negative results are intentional: Tokenmill must not label these values exact or imply that an explicit CLI/ACP prompt observes hidden VS Code traffic.

## Implementation evidence

- ACP usage extraction is implemented by [`AcpUsageSummary`](../../crates/tokenmill-acp/src/lib.rs).
- Local token estimation is documented as non-provider-specific in [`tokenmill-core`](../../crates/tokenmill-core/src/lib.rs).
- Redacted report fields are written by [`tokenmill-cli`](../../crates/tokenmill-cli/src/main.rs).
- The supported workflow and privacy boundary are documented in [`docs/acp-replay.md`](../../docs/acp-replay.md).

## Decision

Keep the current contract:

- Use `MeasurementStatus::Estimated` for Tokenmill's local context estimates.
- Keep agent-reported ACP context-window usage as supplemental evidence.
- Treat exact billing measurement and hidden VS Code context capture as unmeasured until a supported, repeatable test exists.
- Preserve the explicit GitHub Copilot identity check and never infer identity from a filename such as `copilot.exe`.
