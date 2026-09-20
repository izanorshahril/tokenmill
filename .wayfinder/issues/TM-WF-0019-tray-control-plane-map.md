---
id: TM-WF-0019
title: Deliver tray controls for explicit Tokenmill requests
kind: map
labels:
  - wayfinder:map
status: open
parent:
blocked_by: []
assignee:
created: 2026-09-20
---

## Destination

Provide a Windows tray menu with visible routing and saver state and easy toggles for requests explicitly launched through Tokenmill.
Keep the Rust core independent of the Windows surface and preserve the headless CLI.
This is a new destination after the archived V1 tray deferral, authorized by the user's UX review follow-up.

## Current evidence

The existing TUI is a history viewer with refresh, help, and quit, not an interactive policy control plane.
The control views promised by TM-WF-0009 remain incomplete despite its historical closed status.
The CLI routing-off path now skips context reads, ACP launch, submission, and report writes and tells users to continue in GitHub Copilot directly.
Help and invalid-command feedback now remain visible until the next TUI command.
Process-level regression tests cover those fixes in `crates/tokenmill-cli/tests/controls.rs`.

## Frontier

[Build the Windows tray menu and shared request policy](TM-WF-0020-windows-tray-controls.md) owns the remaining implementation and acceptance checks.

## Boundaries

Native VS Code interception, global GitHub Copilot switching, native model-picker integration, and exact billing measurement remain unsupported.
The tray must not imply that changing its policy affects requests outside Tokenmill.
Native tray menu and shared context-request settings are implemented; visual and interaction verification remain open in the child ticket.
