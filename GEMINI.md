# Vizo AI Agent Guidelines

All AI coding assistants interacting with this codebase must adhere to the following rules:

1. **Error Handling & Architecture**:
   - Every operation must be strongly typed with `ViewerError` and `ViewerResult<T>`.
   - Never swallow errors or use silent fallback defaults.
   - Any external parser panic must be handled via `std::panic::catch_unwind`.
   - Keep code modular across dedicated files with clear English documentation.

2. **Automated Verification & Packaging**:
   - Before completing any task, always execute:
     ```bash
     ./scripts/verify_and_package.sh
     ```
   - This script runs `cargo test --release`, builds `cargo build --release`, runs `./install.sh`, and rebuilds all distribution packages in `dist/` (`.deb`, `.rpm`, `.pkg.tar.zst`, `.tar.gz`, `.zip`).
   - If this script fails, the task is incomplete. Fix the errors before answering the user.

3. **Token Optimization**:
   - Use `./scripts/quick_check.sh` during development to quickly test and type-check without generating packages repeatedly.
   - Run `./scripts/verify_and_package.sh` only upon final completion.

4. **No Guessing & Mandatory Code Analysis**:
   - Guessing, speculating, or making assumptions is **strictly prohibited**.
   - Whenever investigating or implementing functionality, agents **MUST** inspect the actual code and data structures (`grep_search`, `view_file`) first.
   - Ground every statement and solution in verified source code evidence.
