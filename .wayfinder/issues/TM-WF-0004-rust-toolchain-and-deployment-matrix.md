---
id: TM-WF-0004
title: Choose the Rust toolchain and deployment baseline
kind: ticket
labels:
  - wayfinder:research
mode: AFK
status: closed
parent: TM-WF-0001
blocked_by: []
assignee: copilot
created: 2026-09-18
---

## Question

Given the corporate Windows workspace cannot install MSVC for licensing reasons, what Rust GNU, Zig C, MSVC, and cross-platform build/deployment choices are viable for Tokenmill's core and planned desktop, web, CLI/TUI, and tray surfaces, including whether artifacts can be developed in both GNU and MSVC environments later?

Use official Rust, Cargo, target-toolchain, Zig, and relevant GUI framework documentation. Report constraints, licensing implications, native dependencies, and a recommended V1 baseline without assuming administrator access.

## Research asset

[Research: Choose the Rust toolchain and deployment baseline](../research/TM-WF-0004-rust-toolchain-and-deployment-matrix.md)

## Resolution

Use stable GNU Rust for the core and CLI V1 profile, with a pinned toolchain file. Treat Windows MSVC as a separate desktop/tray profile because common GUI packaging paths require Microsoft C++ Build Tools and Windows SDK support. Zig may be an optional build-time tool, but it is not the V1 baseline.

Do not claim one Windows binary or GNU desktop parity until both profiles are tested. The current scaffold follows the GNU core/CLI path.

Evidence: [research asset](../research/TM-WF-0004-rust-toolchain-and-deployment-matrix.md)
