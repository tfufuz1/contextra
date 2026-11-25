#!/usr/bin/env bash
# =============================================================================
# Contextra — Fuzzing Toolchain Installation Script
# Repository: https://github.com/tfufuz1/contextra
# Target: Jules VM / Developer Workstation
# =============================================================================

set -euo pipefail

echo "============================================================"
echo "  Contextra Fuzzing Toolchain Setup"
echo "  $(date -u '+%Y-%m-%d %H:%M:%S UTC')"
echo "============================================================"

# ── 1. Verify / Install Rust Nightly Toolchain ───────────────────────────────
echo ""
echo "[1/3] Ensuring Rust nightly toolchain is available..."

if ! rustup toolchain list | grep -q "nightly"; then
    echo "  ℹ️ Installing Rust nightly toolchain..."
    rustup toolchain install nightly --profile minimal
else
    echo "  ✅ Rust nightly toolchain is installed."
fi

# ── 2. Install cargo-fuzz (NEVER cargo install fuzz) ─────────────────────────
echo ""
echo "[2/3] Installing cargo-fuzz sub-command..."
echo "  ⚠️ WARNING: Always install 'cargo-fuzz', NOT 'fuzz'."
echo "  ⚠️ 'fuzz' on crates.io is an unmaintained obsolete crate (v0.1.1)."

# Install cargo-fuzz using nightly toolchain to ensure compatibility with modern dependencies
cargo +nightly install cargo-fuzz --locked

# ── 3. Verification ──────────────────────────────────────────────────────────
echo ""
echo "[3/3] Verifying cargo-fuzz installation..."

CARGO_FUZZ_VER=$(cargo fuzz --version 2>/dev/null || cargo +nightly fuzz --version 2>/dev/null || echo "FAILED")

if [ "$CARGO_FUZZ_VER" = "FAILED" ]; then
    echo "  ❌ Failed to verify cargo-fuzz execution."
    exit 1
fi

echo "  ✅ $CARGO_FUZZ_VER successfully installed and verified."

echo ""
echo "============================================================"
echo "  ✅ Fuzzing Toolchain Setup Complete"
echo "============================================================"
