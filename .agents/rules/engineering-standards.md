---
description: Mandatory engineering guidelines, error handling, testing, and packaging pipeline for all agent sessions
globs: ["**/*"]
---

# Agent Mandatory Operating Rules

## Rule 1: No Swallowed Errors & Strict Error Handling
- Never ignore errors with `.ok()`, `let _ = ...` on critical functions, or silent fallback defaults.
- All errors must be explicitly typed using `thiserror` in `src/error.rs` (`ViewerError`).
- Any decoding or parsing failures must return informative error messages.
- Third-party library panics (e.g. `rawloader` or `libheif`) must be caught with `std::panic::catch_unwind` and converted to `ViewerError`.

## Rule 2: Mandatory Automated Packaging Before Completion
- After completing any code change or feature, you **MUST** run:
  ```bash
  ./scripts/verify_and_package.sh
  ```
- This runs unit tests, compiles the release binary, updates the local user installation, and regenerates all distribution packages in `dist/` (`.deb`, `.rpm`, `.pkg.tar.zst`, `.tar.gz`, `.zip`).
- Delivering untested or unpackaged changes to the user is strictly prohibited.

## Rule 3: Testing Standard
- Every new format, transformation, shortcut, or UI state must be covered by automated tests.
- All tests must pass before concluding the turn.

## Rule 4: Token Efficiency
- During development iterations, use `./scripts/quick_check.sh` (`cargo check` + unit tests) to save time and tokens.
- Run the full `./scripts/verify_and_package.sh` once work is complete.
- Keep comments and documentation strictly in English.

## Rule 5: No Guessing & Mandatory Code Analysis (Strictly Enforced)
- Guessing, speculating, or assuming how a function, format, schema, or system component works is strictly forbidden.
- Whenever working on a task or debugging a problem, agents must always perform targeted code analysis (`grep_search`, `view_file`) on the active source code first.
- Every assertion, design decision, and code edit must be grounded in verified source code facts.
