# Lessons Learned for Tokenmill

This document records implementation lessons from the 9Router and Headroom integration work that should shape Tokenmill's future design and validation.

## Runtime ownership is part of the feature

An interactive shell, a packaged server, a tray process, and a child process can resolve different executables, interpreters, `PATH` values, and environment variables.

Before installing or diagnosing an integration, identify the manifest owner, executable, package manager, interpreter, process command line, working directory, and inherited environment.

For ACP, always identify the GitHub Copilot CLI/ACP executable and negotiated identity explicitly; a filename such as `copilot.exe` is not sufficient.

Verify the runtime that serves the user-visible behavior, not only the runtime that works in the current shell.

## Build a red-capable feedback loop first

Reproduce the exact user symptom with the narrowest unattended check available before forming a fix hypothesis.

For a CLI or ACP path, prefer a focused command or protocol test.

For a dashboard or tray path, prefer an authenticated live endpoint or a helper-level regression test that asserts the missing or incorrect user-visible state.

Keep the check fast and repeat it after each meaningful change.

## Treat packaging and deployment as separate boundaries

Source tests do not prove that a packaged or already-running process loads the new code.

When a change crosses a build or packaging boundary, verify the source artifact, compiled bundle, installed bundle, active process command line, and restart state.

Compare hashes or another stable identity when copying generated artifacts, then re-run the original live check against the restarted process.

Do not declare success from a completed build or an active process alone.

## Make cross-process behavior observable

A tray icon or helper process can be alive while its menu, IPC channel, startup ordering, or command handling is broken.

Test the startup handshake and the first user-visible state, including fallback helpers used under constrained Windows environments.

For ACP and adapter boundaries, test identity negotiation, permission behavior, streaming lifecycle, and strict versus compatible policy outcomes rather than only process creation.

Keep diagnostics structured and redacted.

Temporary diagnostics should use a unique tag, capture the boundary that distinguishes the hypotheses, and be removed after the original check passes.

## Preserve honest operational status

Separate child exit status from PowerShell or launcher wrapper noise.

Report stale logs, unavailable health checks, unverified connectivity, and incomplete runs as distinct states instead of converting them into success-shaped fallbacks.

Do not persist raw prompts, source, tool output, credentials, or full request and response bodies while adding observability.

## Treat local routers as security and lifecycle boundaries

The 9Router incident showed why a local model router needs an explicit owner and a narrow contract rather than being an implicit dependency of a desktop client.

- Codex was configured to select a `9router` provider with base URL `http://127.0.0.1:20128/v1`. This coupling lived in Codex configuration, not in the Codex AppX state.
- The installed 9router package did not provide a detach or disable-autostart command. Its npm `postinstall` hook only warmed runtime dependencies, and its tray mode was opt-in.
- At investigation time there was no 9router process, listener on port `20128`, Windows Run entry, scheduled task, startup shortcut, or service. A stale notification-area registry record remained, but it was not an autostart mechanism.
- Historical Codex logs showed requests being sent to the local endpoint even when no router was available. A client can therefore appear configured while the actual provider is unavailable.
- The provider configuration contained an authorization header. Secrets must never be copied into lessons, logs, diagnostics, or generated reports; router configuration should use an explicit secret store or inherited credential reference.

The design implication is to make Tokenmill's router lifecycle explicit: own the process, expose health and shutdown operations, provide a clear detach/reset path, fail closed when the endpoint is unavailable, isolate credentials from general configuration, and test restart, upgrade, uninstall, and client reconfiguration behavior. This incident is additional evidence for building a small Tokenmill-owned router boundary instead of relying on undocumented behavior in a third-party local gateway.

## Validate destructive configuration changes

The same OMP cleanup session that inspected the environment later ran a malformed PowerShell regex while filtering User `PATH`:

```powershell
$_ -notmatch '(?i)Programs\Herdr'
```

The invalid `\H` escape produced an error for every entry. The pipeline consequently yielded no entries, but the script continued and persisted the empty result. This removed discoverability for npm, Bun, pnpm, Codex, Claude, Gemini, and other tools even though many binaries and packages remained installed. A separate earlier command explicitly removed the Bun-global Pi package; that was a package removal, not the cause of the broad PATH failure.

For any environment mutation, snapshot the original value, use literal path comparisons where possible, validate the candidate is non-empty and preserves required roots, write only after validation, and re-read the persisted value afterward. Treat malformed filters and partial command output as failure, never as a successful cleanup.

## Treat client model state as a separate persistence boundary

A provider can return one unique model catalog while the client still displays older identifiers retained in global storage.

Reloading an extension, reinstalling its package, or deleting a provider profile does not necessarily remove cached models, recent selections, pinned models, or model-configuration records.

When model identifiers or provider namespaces change, define a stable identifier scheme before shipping and provide an explicit migration or reset path for stale client state.

Test the transition from the old catalog to the new catalog with the real client state store, not only a server `/models` response or provider unit test.

Verify the package selected by the running client, because a marketplace build can silently replace a locally patched artifact while preserving the same extension version.

Keep cache reset scoped and reversible, preserve user settings and credentials, and report whether the displayed catalog came from the live provider or a retained client cache.

## Tokenmill application checklist

Before merging an integration change:

1. Prove the exact runtime and product identity.
2. Run one red-capable focused check that fails on the original symptom.
3. Test source and packaged or helper paths separately when both exist.
4. Re-run the live or protocol-level check after restart.
5. Test client cache and identifier migration when an integration exposes models or other persisted external state.
6. Remove tagged diagnostics and preserve only redacted evidence.
