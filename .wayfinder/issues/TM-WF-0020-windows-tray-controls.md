---
id: TM-WF-0020
title: Build the Windows tray menu and shared request policy
kind: ticket
labels:
  - wayfinder:task
status: open
parent: TM-WF-0019
blocked_by: []
assignee: codex
created: 2026-09-20
---

## Goal

Let users see and change `acp-context-prompt` policy from the Windows notification area without remembering CLI flags.
The initial implementation leaves raw prompts, paired evaluations, and external GitHub Copilot requests outside this control scope, and labels that scope in the menu.

## Interaction contract

Show a text status row and distinguish routing OFF, READY, RUNNING, and ERROR without relying on color alone.
READY means locally ready for an explicit request, not verified GitHub Copilot connectivity.
Provide checked Routing and Saver menu items, a Strict/Compatible choice, Open history, and Quit.
Every menu must make scope discoverable: these controls affect Tokenmill-launched requests only.
Expose the last verification or error separately from the requested policy, without raw request data.
Routing OFF skips submission; saver OFF preserves context while routing remains enabled.
Disable the saver and mode menu items while routing is OFF, preserving their selections for re-enabling.
Keep status text clear that saved saver preference is inactive while routing is OFF.
Use first-run defaults of routing OFF, saver ON, and strict mode, with no automatic startup registration.

## Shared policy and lifecycle

Store only versioned local policy settings; never store credentials, prompts, source, or ACP output in settings.
Use one documented settings location shared by tray and explicit context requests; explicit CLI arguments override saved settings.
When settings are absent, preserve existing CLI defaults unless the user starts the tray and opts into its settings.
Validate settings at read boundaries and fail visibly for malformed or unsupported settings rather than silently enabling routing.
Persist updates atomically and serialize writers; failed saves retain the prior effective state and expose the error.
Snapshot policy at request start; changes affect subsequent requests and must not claim to cancel an in-flight request.
Show RUNNING with a separate next-request policy when a toggle changes during a request.
Quit removes the tray icon and stops the control surface; saved policy remains effective for later explicit CLI invocations.
Label Quit accordingly and offer Routing OFF separately so quitting cannot be mistaken for disabling routing.
Reject duplicate tray instances or focus the existing instance.

## Acceptance

Verify left/right click discovery, keyboard navigation, checked states, disabled states, high DPI, and readable light/dark system menus on Windows.
Verify toggles persist across restart and the next explicit request honors them, including CLI override precedence.
Verify routing OFF starts no ACP process, saver OFF does not prune, and strict mode still rejects unverified routes.
Verify rapid toggles, concurrent settings writes, malformed settings, save failures, duplicate instances, and exit cleanup.
Use local fixtures or fake ACP agents for automated checks; live verification must identify GitHub Copilot explicitly and report its actual evidence.
Keep tray/platform dependencies outside `tokenmill-core`, optional for headless builds, and compatible with the installed Windows GNU toolchain.
Record an actual menu capture and interaction results before closing this ticket; a rendered mockup is not implementation evidence.

## Implementation evidence (2026-09-21)

Implemented native WinForms notification icon, controls window, and shared menu, compiled using the installed .NET Framework compiler without a new Cargo dependency.
Rust owns validated settings, serialized atomic saves, and per-request policy snapshots with an OS-held request lock.
Added process-level tests for initial defaults, persistence, CLI override, malformed settings, writer contention, save failure preservation, and running-state lock release.
Existing ACP tests cover saver bypass and strict-mode rejection; parser tests confirm saved saver/mode values and explicit overrides.
Launched the real native tray through computer-use and verified repeat launch reopens the same hidden controls window.
The tray reports last context-request outcome and route verification separately from requested settings; incomplete runs remain unverified and routing ON never implies connectivity.
Computer-use verified initial OFF state with disabled Saver/Mode, Routing ON, Saver OFF, Compatible selection, persisted values, history fixture rendering, Hide to tray, reopening, and normal Quit.
Keyboard menu mnemonics and mouse opening of the shared menu work.
The history fixture displays 6 runs, 1 accepted, 1 rejected, 4 unknown, 70 estimated saved tokens, and 11.7% average reduction.
`Start-Tokenmill.cmd` and the console-free `tokenmill-tray.exe` provide simple local launch.
Native screenshots from the computer-use session are retained locally at `target/tray-evidence/controls.png` and `target/tray-evidence/menu.png` and excluded from Git.
The launcher was executed successfully after normal Quit; controls were left available with routing OFF, saver ON, and strict mode for the user's UX trial.
Ticket stays open for direct notification-area left/right click testing and broader DPI/theme coverage; current-scale native menu and window have been visually inspected.
