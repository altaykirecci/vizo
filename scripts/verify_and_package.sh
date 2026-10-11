#!/usr/bin/env bash
# ==============================================================================
# Vizo Verification and Automated Packaging Pipeline
#
# Rules & Guarantees:
# 1. No swallowed errors: Fails immediately if any command returns non-zero.
# 2. Strict testing: Runs all unit and integration tests before building.
# 3. Clean release build: Compiles optimized release binary.
# 4. Local deployment: Updates ~/.local/bin/vizo and desktop entries.
# 5. Multi-dist packaging: Builds .deb, .rpm, .pkg.tar.zst, .tar.gz, and .zip.
# ==============================================================================
set -eo pipefail

export PATH="$HOME/.cargo/bin:$PATH"

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_DIR"

echo "=========================================================="
echo "🚀 Vizo Pipeline: Test, Build, Install & Package"
echo "=========================================================="

# 1. Run automated test suite
echo ""
echo "🧪 [1/4] Running automated test suite (cargo test)..."
cargo test --release

# 2. Compile release binary
echo ""
echo "⚙️  [2/4] Compiling release binary (cargo build --release)..."
cargo build --release

# 3. Install locally
echo ""
echo "📦 [3/4] Updating local system installation (./install.sh)..."
./install.sh

# 4. Package for all distributions
echo ""
echo "📦 [4/4] Generating all distribution packages (./scripts/build_all_packages.sh)..."
./scripts/build_all_packages.sh

echo ""
echo "=========================================================="
echo "✅ Pipeline successfully completed! All packages are ready."
echo "=========================================================="
