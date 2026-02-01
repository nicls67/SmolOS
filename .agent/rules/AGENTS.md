---
trigger: always_on
---

# AGENTS

## Purpose
This file orients coding agents working in this repo. Keep changes focused, avoid unrelated formatting, and follow existing patterns.

## Repo layout
1. `crates/` contains the Rust workspace members :
  * `crates/display` contains the library for screen display.
  * `crates/hal_interface` contains the interface with the HAL written in C in `drivers/Interface`.
  * `crates/kernel` contains the main OS functions.
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
- Update or create documentation when needed. Methods and functions documentation needs to include : functionality description, parameters description, return description, error handling, panicking (when concerned).
- If you need to touch multiple crates, explain why in the final response.
- Always ask for user review after generating a action plan. Never update code by yourself.

## Naming rules
1. Respect Rust, C and Python naming conventions.
2. Local variables starts with "l_"
3. Global variables starts with "G_"
4. Constants starts with "K_"
5. Functions and methods parameters starts with "p_"

## Git rules
This rule applies each time a git branch needs to be created or renamed

### Instructions
1. Never ask for a branch name, define it by yourself
2. Analyse the task : 
  - In case of a new feature : /feat/task-name
  - In case of bug fix : /fix/task-name
3. Always lower case
4. When the pull request is linked to a Github issue, add 'Closes #ID' to the pull request message. 

### End of task
After each successful merge :
1. Always delete the associated branch
2. Confirm that the associated GitHub issue is closed