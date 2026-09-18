# Research: Explain the Copilot and 9Router authentication workaround

Question: Why can 9Router make requests without asking for GitHub Copilot login again, and how should Tokenmill use that behavior without copying credentials between tools?

## Findings

### 1. The two processes have different authentication domains

GitHub Copilot CLI stores its own authentication state in the operating-system credential store when possible.
If no credential store is available, it can use its configuration directory as a fallback.
The configuration directory is `~/.copilot` by default and can be changed with `COPILOT_HOME`.

Copilot CLI also supports headless authentication through `COPILOT_GITHUB_TOKEN`, `GH_TOKEN`, or `GITHUB_TOKEN`, in that precedence order.
Those variables are alternatives to the CLI login flow, not a bridge to another application's OAuth state.

Launching Copilot CLI through ACP starts the Copilot process as a child process.
The ACP client therefore gets the authentication context that the child process can resolve from its inherited Windows user, environment, `COPILOT_HOME`, and credential store.
An `authMethods` entry advertised by `initialize` describes an available ACP authentication method; it does not by itself prove that a fresh login is required for every process.

### 2. 9Router avoids repeated login by persisting and refreshing its own provider state

9Router is a local OpenAI-compatible gateway at `http://localhost:20128/v1`.
Its documented architecture includes OAuth and API-key provider connections, local persistence, provider-specific execution, token refresh, fallback, and usage tracking.
Its README explicitly describes automatic OAuth token refresh and says that an expired token is refreshed by 9Router before asking the user to reconnect.

That explains the observed behavior: after the 9Router provider was connected once, 9Router reused its own persisted OAuth connection and refreshed it when necessary.
The absence of a new browser prompt does not mean that 9Router imported or reused the Copilot CLI credential.

### 3. The supported bridge is model routing, not credential sharing

Copilot CLI documents bring-your-own-key (BYOK) providers through `COPILOT_PROVIDER_BASE_URL`, `COPILOT_PROVIDER_TYPE`, `COPILOT_PROVIDER_API_KEY`, and `COPILOT_MODEL`.
The OpenAI provider type is intended for OpenAI-compatible endpoints.
Copilot CLI also documents `providers.json` as a provider and model registry, and says it takes precedence over the legacy provider environment variables when it declares providers or models.

Therefore a deliberate 9Router mode can point Copilot CLI at 9Router's OpenAI-compatible endpoint, provide a 9Router API key, and select a model returned by `GET /v1/models`.
In that mode, 9Router owns upstream provider authentication and Copilot CLI authenticates only to the local gateway.
This changes the inference route and model selection; it does not make 9Router's provider OAuth token a Copilot login token.

There is no supported basis for Tokenmill to copy 9Router's database, browser session, OAuth refresh token, or provider credential into Copilot CLI.
Tokenmill must not inspect or migrate those secrets.

## Tokenmill decision

Tokenmill should expose two explicit integration modes:

1. **GitHub Copilot CLI ACP**
   Launch the exact GitHub Copilot CLI/ACP executable with the user's normal Windows identity and inherited environment.
   Preserve the existing `COPILOT_HOME` when set and do not override `COPILOT_GITHUB_TOKEN`, `GH_TOKEN`, or `GITHUB_TOKEN`.
   This is the preferred path when the user wants GitHub-hosted Copilot behavior.

2. **9Router BYOK**
   Launch Copilot CLI with a separately configured OpenAI-compatible provider pointing to `http://localhost:20128/v1`.
   Obtain the gateway API key from the user's 9Router configuration and select a model from the gateway model list.
   Treat this as explicit routing to a third-party or alternate provider path, not as GitHub Copilot native authentication.

The ACP adapter should report the selected route, provider, model, and measurement status.
If Tokenmill cannot verify which path handled a request, strict mode should fail closed and compatible mode should label the result as unverified or estimated.
Missing provider usage must remain unmeasured rather than being reported as zero.

## Local verification

A metadata-only PowerShell check in the Tokenmill workspace found the default Copilot state directory and its `config.json` and `settings.json` files.
No `COPILOT_GITHUB_TOKEN`, `GH_TOKEN`, or `GITHUB_TOKEN` value was present in the Tokenmill process environment.
The check did not read file contents or credential values.

## Sources

- https://docs.github.com/en/copilot/reference/copilot-cli-reference/cli-command-reference#copilot-login-options
- https://docs.github.com/en/copilot/reference/copilot-cli-reference/cli-command-reference#environment-variables
- https://docs.github.com/en/copilot/reference/copilot-cli-reference/cli-config-dir-reference
- https://docs.github.com/en/copilot/reference/copilot-cli-reference/cli-config-dir-reference#providersjson
- https://docs.github.com/en/copilot/concepts/agents/about-copilot-cli#model-usage
- https://docs.github.com/en/copilot/reference/copilot-cli-reference/acp-server
- https://github.com/decolua/9router/blob/master/README.md
- https://github.com/decolua/9router/blob/master/docs/ARCHITECTURE.md

## Unknowns

- The exact 9Router provider and model configured by the user are not known.
- Whether the user's enterprise Copilot policy permits BYOK providers must be verified in the target environment.
- Tokenmill has not yet implemented authenticated `session/prompt` forwarding or a 9Router client.
