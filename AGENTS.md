# Vizo Agent Engineering Guidelines & Operating Standards

This document defines the mandatory development, quality assurance, and packaging protocols for all AI agents working in this repository.

---

## 1. Core Engineering Principles

1. **Explicit Error Handling (No Swallowed Errors, No Silent Fallbacks)**:
   - All errors must use strongly-typed variants in `src/error.rs` (`ViewerError` and `ViewerResult<T>`).
   - Never swallow errors silently (e.g., using `.ok()`, ignoring `Result`, or silently returning default values on fatal failures).
   - If a failure occurs, return meaningful diagnostics with path context.
   - Wrap external or third-party crates prone to panicking on malformed inputs (such as raw decoders) using `std::panic::catch_unwind` and map panics cleanly to `ViewerError`.

2. **Modular Architecture & Code Quality**:
   - Maintain strict separation of concerns across modules (`decoder`, `exif_info`, `navigation`, `toolbar`, `transform`, `shortcuts`, `about`).
   - Do not bloat single files. All documentation, comments, and docstrings must be written in **English**.
   - Keep public symbols documented with clear markdown docstrings.

3. **No Guessing & Mandatory Code Analysis (Strictly Enforced)**:
   - Guessing, assuming, or speculating about implementation details, formats, schemas, or bug causes is **strictly forbidden**.
   - Whenever an issue arises or a modification is needed, agents **MUST** perform concrete code analysis (`grep_search`, `view_file`) on the active source code before writing code or making assertions.
   - Trace the exact data flow, read actual definitions, and inspect logs/traces rather than making assumptions.

4. **Mandatory Testing Requirement**:
   - Never deliver work to the user without verifying that all tests pass.
   - Add unit tests for every new feature, format decoder, or behavior modification.
   - Verify test suite passes via `cargo test`.

---

## 2. Mandatory Post-Task Pipeline

Before completing any user request or delivering changes, agents **MUST** execute the automated verification and packaging script:

```bash
./scripts/verify_and_package.sh
```

This automated script performs the following sequential stages:
1. **`cargo test --release`**: Verifies all unit and integration tests. Halts immediately on failure.
2. **`cargo build --release`**: Compiles an optimized release binary.
3. **`./install.sh`**: Updates the local binary (`~/.local/bin/vizo`) and desktop launcher (`~/.local/share/applications/vizo.desktop`).
4. **`./scripts/build_all_packages.sh`**: Rebuilds all native distribution packages in `dist/`:
   - Debian / Ubuntu (`.deb`)
   - Arch Linux (`.pkg.tar.zst`)
   - Fedora / RHEL (`.rpm`)
   - Universal Standalone Linux (`.tar.gz`)
   - Windows Portable (`.zip`)

**Rule:** If `./scripts/verify_and_package.sh` fails at any step, the task is **NOT DONE**. Fix the failure before reporting to the user.

---

## 3. Token & Performance Optimization Guidelines

To preserve LLM context and minimize token usage during agent sessions:

1. **Use `scripts/quick_check.sh` during development**:
   - When iterating on code changes, run `./scripts/quick_check.sh` (`cargo check` + fast test run) instead of repeatedly running full packaging.
   - Only execute `./scripts/verify_and_package.sh` once the implementation is finalized.
2. **Targeted Inspections**:
   - Use `view_file` with precise `StartLine` and `EndLine` ranges rather than reading large files in bulk.
   - Use `grep_search` with specific query filters instead of whole-disk directory listings.
3. **Concise Communication**:
   - Keep model responses concise, factual, and focused on the solution.
