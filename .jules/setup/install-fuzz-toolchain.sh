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
    echo "  ℹ️ Installing Rust nightly toolchain with rust-src, llvm-tools-preview, miri..."
    rustup toolchain install nightly --profile minimal -c rust-src -c llvm-tools-preview -c miri
else
    echo "  ✅ Rust nightly toolchain is installed. Ensuring components..."
    rustup component add --toolchain nightly rust-src llvm-tools-preview miri 2>/dev/null || true
fi

# ── 2. Install cargo-fuzz (NEVER cargo install fuzz) ─────────────────────────
echo ""
echo "[2/3] Installing cargo-fuzz sub-command..."
echo "  ⚠️ WARNING: Always install 'cargo-fuzz', NOT 'fuzz'."
echo "  ⚠️ 'fuzz' on crates.io is an unmaintained obsolete crate (v0.1.1)."

if cargo fuzz --version >/dev/null 2>&1 || cargo +nightly fuzz --version >/dev/null 2>&1; then
    echo "  ✅ cargo-fuzz is already installed."
else
    cargo +nightly install --locked cargo-fuzz
fi

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
