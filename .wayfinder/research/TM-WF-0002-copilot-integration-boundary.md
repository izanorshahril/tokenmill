# Research: Verify GitHub Copilot integration and model-picker boundary

Ticket: [Verify GitHub Copilot integration and model-picker boundary](../issues/TM-WF-0002-copilot-integration-boundary.md)

## Findings

A third-party routed provider can appear in VS Code's generic language-model picker through `LanguageModelChatProvider`, but this is a VS Code extension/provider boundary rather than a documented way to inject a provider into GitHub Copilot's private backend catalog.

VS Code documents language-model providers, chat participants, language-model tools, MCP servers, and provider registration.

Copilot CLI documents custom providers, model selection, hooks, MCP, and programmatic mode.

ACP standardizes client-agent sessions, prompts, tools, modes, streamed updates, and extensions, but does not define provider discovery or native Copilot model-picker integration.

A strict mode can require a verified supported path, known transformed context, provider/model identity, request and response lifecycle evidence, and provider-supplied usage where available.

A compatible mode can support Copilot CLI custom-provider routing, ACP clients around Copilot CLI, local token estimates, and lifecycle hooks, but missing usage must be reported as unmeasured rather than zero.

The Copilot desktop app has no public provider/plugin protocol for inserting a routed provider into its native model catalog in the reviewed sources.

## Sources

- https://code.visualstudio.com/api/extension-guides/ai/language-model
- https://code.visualstudio.com/api/extension-guides/ai/ai-extensibility-overview
- https://raw.githubusercontent.com/microsoft/vscode/main/src/vscode-dts/vscode.proposed.chatProvider.d.ts
- https://github.com/microsoft/vscode/blob/main/src/vs/workbench/api/common/extHostLanguageModels.ts
- https://docs.github.com/en/copilot/concepts/agents/about-copilot-cli#model-usage
- https://docs.github.com/en/copilot/how-tos/copilot-cli/customize-copilot/use-hooks
- https://docs.github.com/en/copilot/reference/copilot-cli-reference/acp-server
- https://agentclientprotocol.com/protocol/v1/prompt-turn
- https://agentclientprotocol.com/protocol/v1/initialization
- https://agentclientprotocol.com/protocol/v1/extensibility

## Unknowns

- Whether GitHub permits third-party extensions to register models under the `copilot` vendor.
- Whether VS Code's proposed provider API becomes stable and Marketplace-usable.
- Whether Copilot clients expose the final serialized prompt, hidden instructions, or billing-token accounting.
- Whether ACP implementations consistently emit accurate usage data.
