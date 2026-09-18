# Research: Establish reference behavior from 9router and Headroom

Ticket: [Establish reference behavior from 9router and Headroom](../archive/issues/TM-WF-0003-reference-architecture-evidence.md)

## Findings

9Router is an OpenAI-compatible gateway that combines provider/account fallback, OAuth refresh, format translation, local usage/log storage, dashboard controls, and optional token savers.

9Router exposes a token-saver bypass header and can optionally log full request content, which is sensitive.

Headroom is primarily a context-compression layer and transparent proxy with library, proxy, MCP, SDK, and agent-wrapper modes.

Headroom routes content by type, including JSON, code, prose, logs, searches, diffs, and configuration.

Headroom preserves provider model selection unless explicit model routing is enabled, and its VS Code Copilot integration changes API proxy endpoints rather than the selected model or extension model list.

Headroom supports bypass, lossless/cache-aware modes, CCR retrieval, local metrics, logs, health/debug endpoints, dashboards, and Prometheus output.

Both references justify separating routing from saving and making every measurement distinguish exact usage, estimates, counterfactual savings, and missing data.

Tokenmill should use stricter V1 privacy defaults than either reference: no external telemetry by default and no full-content logging by default.

## Sources

- https://github.com/decolua/9router/blob/master/README.md
- https://github.com/decolua/9router/blob/master/docs/ARCHITECTURE.md
- https://github.com/decolua/9router/blob/master/.env.example
- https://github.com/decolua/9router/tree/master/src/sse
- https://github.com/headroomlabs-ai/headroom/blob/main/README.md
- https://github.com/headroomlabs-ai/headroom/blob/main/headroom/proxy/server.py
- https://github.com/headroomlabs-ai/headroom/blob/main/headroom/compress.py
- https://docs.headroomlabs.ai/docs/proxy
- https://docs.headroomlabs.ai/docs/configuration
- https://docs.headroomlabs.ai/docs/vscode-copilot
- https://docs.headroomlabs.ai/docs/limitations
