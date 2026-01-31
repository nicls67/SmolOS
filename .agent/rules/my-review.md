---
trigger: always_on
---

# Rule: my_review

## Trigger
This rule applies when the user asks to review ("review", "my_review")

## Instructions
You are a strict reviewer focused on bugs and code quality. Your mission is to produce an actionable report.  
Follow the steps below in order, without skipping any.
Before any code update (except imports cleanup and documentation update), propose a modification plan to the user. You need his agrrement before proceeding. 

## 1) Git state and review scope
1. Run `git status -sb` and summarize what is modified.
2. The review is done by default on the uncomitted changes. if the user asks to review the branch, work on all the changes made in the current branch.

## 2) Human-style code review
Read the diff and focus primarily on:
  - Check for potential bugs or performance issues.
  - Ensure code is properly documented and documentation is up-to-date.
  - Check error handling and error messages.
  - Ensure code is well-structured and follows best practices.
  - Verify that the code is easy to understand and maintain.
  - Check all imports are used and remove unused imports.
  
## 3) Documentation update and code cleanup. You are allowed to update the modified files for that.
- First update documentation (or create it if it is missing) for each modified item (method, function, structure, etc...). 
- Correct any comment or documentation that is inconsistent with the code
- Clean imports : remove unused imports

## 4) Automated checks (read-only)
- If a `Cargo.toml` is present:
  - `cargo fmt --check`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo check`
- Otherwise, adapt to the detected ecosystem (package.json, pyproject, etc.) using the standard lint/test commands in check mode.

## 5) Automated checks corrections
- If any issue has been found on a modified file by an automated check, you are allowed to correct it. 

## 6) Required output format
Produce a report with **exactly** the following sections:

### Summary
- 3–6 bullet points describing what changed and the overall risk level.

### Blocking issues (must-fix)
- Numbered list.
- For each item: file/line if possible, explanation, and concrete suggestion.

### Important issues (should-fix)
- Same structure as above.

### Improvements (nice-to-have)
- Same structure as above.

### Check results
- Summarize the results of the automatic checks
- Give explanation/concrete suggestion

### Final recommendation
- “OK to merge” / “OK with fixes” / “Not OK”
- Short justification (2–3 sentences).

## 7) Trigger examples
- `review`
- “Run review before I push”
- “Review this diff and run fmt/clippy/tests”
- "@my_review"

## 8) When the review is finished, remove any temporary file you created (for example check reports ot git diffs)
