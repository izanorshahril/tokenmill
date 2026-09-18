# Research: Choose the Rust toolchain and deployment baseline

Ticket: [Choose the Rust toolchain and deployment baseline](../archive/issues/TM-WF-0004-rust-toolchain-and-deployment-matrix.md)

## Findings

Stable Rust with a pinned `rust-toolchain.toml` is suitable for V1.

`x86_64-pc-windows-gnu` is a Rust Tier 1 target and best matches the stated corporate GNU constraint for core and CLI development.

`x86_64-pc-windows-msvc` is also Tier 1 and is the practical profile for Tauri, WebView2, native Windows integration, and later desktop/tray distribution, but it requires Microsoft C++ Build Tools and the Windows SDK.

GNU and MSVC should be treated as separate supported build profiles until the GUI and native dependency matrix is tested.

Zig can be an optional pinned build-time cross-linker, but it adds toolchain and ABI testing burden and does not remove GUI runtime requirements.

egui/eframe plus `tray-icon` is a GNU-friendly prototype candidate with permissive licensing; Linux tray support may need GTK/AppIndicator or KSNI dependencies.

Tauri has the strongest desktop/web/tray packaging story but its Windows development prerequisites conflict with the current no-MSVC constraint.

Per-user installation is preferable where possible; WebView2 runtime availability and offline packaging remain deployment decisions.

## Recommendation

Use GNU Rust for the core and CLI V1 profile.

Keep a separate MSVC desktop/tray profile as a later or optional build target.

Do not claim one Windows binary or GNU desktop parity until both profiles are tested.

## Sources

- https://doc.rust-lang.org/rustc/platform-support.html
- https://rust-lang.github.io/rustup/concepts/toolchains.html
- https://doc.rust-lang.org/cargo/reference/config.html
- https://ziglang.org/documentation/master/#Targets
- https://tauri.app/start/prerequisites/
- https://tauri.app/reference/config/
- https://github.com/tauri-apps/tray-icon#platforms-supported
- https://github.com/emilk/egui#credits
- https://github.com/tauri-apps/tauri#licenses
