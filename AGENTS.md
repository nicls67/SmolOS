# AGENTS

## Purpose
This file orients coding agents working in this repo. Keep changes focused, avoid unrelated formatting, and follow existing patterns.

## Repo layout
1. `crates/` contains the Rust workspace members :
  * `crates/display` contains the library for screen display.
  * `crates/hal_interface` contains the interface with the HAL written in C in `drivers/Interface`.
  * `crates/kernel` contains the main OS functions.
  * `crates/kernel_apps` contains the applications for the kernel.
  * `crates/smolos` is the entry point for the application.

2. `drivers/` contains platform-specific drivers written in C. Only `drivers/Interface` can be updated, others folders are auto-generated.

3. `config/` contains platform-specific configuration files.

4. `tools/` contains build tools.

## Build & test
- Build the full workspace: `cargo build --release`

## Change guidelines
- Prefer small, targeted edits; avoid sweeping refactors unless asked.
- Keep ASCII in new content unless the file already uses non-ASCII.
- Add comments only when logic is non-obvious.
- Update or create documentation when needed.
- If you need to touch multiple crates, explain why in the final response.

## Naming rules
1. Respect Rust, C and Python naming conventions.
2. Local variables starts with "l_"
3. Global variables starts with "G_"
4. Constants starts with "K_"
5. Functions and methods parameters start with "p_"
