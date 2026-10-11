#!/usr/bin/env bash
# ==============================================================================
# Vizo Fast Developer Check (Token & Time Saver)
# Performs type checking, lint checking, and tests without full packaging.
# ==============================================================================
set -eo pipefail

export PATH="$HOME/.cargo/bin:$PATH"

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_DIR"

echo "🔍 [1/2] Checking compilation & types..."
cargo check --all-targets

echo "🧪 [2/2] Running test suite..."
cargo test -- --nocapture

echo "✅ Quick checks passed successfully!"
