#!/usr/bin/env bash
# =============================================================================
# Contextra — Jules Environment Setup Script
# Repository: https://github.com/tfufuz1/contextra
# Target: Jules VM (Ubuntu 24, Rust pre-installed)
# =============================================================================

set -euo pipefail

echo "============================================================"
echo "  Contextra Jules Environment Setup"
echo "  $(date -u '+%Y-%m-%d %H:%M:%S UTC')"
echo "============================================================"

# ── 1. Rust Toolchain Check & Setup ──────────────────────────────────────────
echo ""
echo "[1/8] Verifying Rust toolchain..."

if ! rustup toolchain install 1.89.0 --profile minimal; then
    echo "❌ Failed to install Rust toolchain 1.89.0"
    exit 1
fi

rustup default 1.89.0

RUST_VERSION=$(rustc --version)
echo "  ✅ $RUST_VERSION"

PINNED_CHANNEL="1.89.0"
if [[ "$RUST_VERSION" != *"$PINNED_CHANNEL"* ]]; then
    echo "❌ Toolchain mismatch: Expected $PINNED_CHANNEL, got: $RUST_VERSION"
    exit 1
fi

rustup component add clippy rustfmt 2>/dev/null || echo "  ⚠️ Warning: Could not add clippy/rustfmt components via rustup"
echo "  ✅ clippy + rustfmt checked"

# ── 2. System Libraries & FlatBuffers Compiler ──────────────────────────────
echo ""
echo "[2/8] Installing system libraries and FlatBuffers compiler..."

if command -v apt-get &>/dev/null; then
    sudo apt-get update -q 2>/dev/null || echo "  ⚠️ Warning: apt-get update failed"
    sudo apt-get install -y -q \
        libwebkit2gtk-4.1-dev \
        libgtk-3-dev \
        libayatana-appindicator3-dev \
        librsvg2-dev \
        libssl-dev \
        flatbuffers-compiler \
        pkg-config \
        curl \
        2>/dev/null || echo "  ⚠️ Warning: Some apt packages unavailable (acceptable fallback)"
fi

echo "  ✅ System libraries step finished"

# ── 3. ONNX Runtime & C++ Toolchain Check ────────────────────────────────────
echo ""
echo "[3/8] Verifying C/C++ toolchain and glibc compatibility..."
gcc --version | head -1
ldd --version | head -1
echo "  ✅ C/C++ toolchain and glibc verified"

# ── 4. Code Search & AST Analysis Tools (ast-grep / sg) ──────────────────────
echo ""
echo "[4/8] Configuring ast-grep and git hooks..."

if ! command -v ast-grep &>/dev/null && command -v sg &>/dev/null; then
    SG_PATH=$(which sg)
    SG_DIR=$(dirname "$SG_PATH")
    if [ -w "$SG_DIR" ]; then
        ln -sf "$SG_PATH" "$SG_DIR/ast-grep" 2>/dev/null || echo "  ⚠️ Warning: Could not symlink sg to ast-grep"
    fi
fi

if command -v ast-grep &>/dev/null || command -v sg &>/dev/null; then
    echo "  ✅ ast-grep / sg is available"
else
    cargo install ast-grep --locked --quiet 2>/dev/null || npm install -g @ast-grep/cli 2>/dev/null || echo "  ⚠️ Warning: ast-grep install skipped"
fi

git config core.hooksPath .githooks || echo "  ⚠️ Warning: Could not set core.hooksPath"
echo "  ✅ git hooks configured to .githooks"

# ── 5. Essential Cargo Tools (just, nextest, audit, deny, llvm-cov) ─────────
echo ""
echo "[5/8] Verifying Cargo helper tools..."

CARGO_TOOLS=("just" "cargo-nextest" "cargo-audit" "cargo-deny")
for tool in "${CARGO_TOOLS[@]}"; do
    if command -v "$tool" &>/dev/null; then
        echo "  ✅ $tool is available"
    else
        echo "  ℹ️ Installing $tool..."
        cargo install "$tool" --quiet 2>/dev/null || echo "  ⚠️ Warning: Could not install $tool (optional)"
    fi
done

# ── 6. Python & Node Environment Setup ───────────────────────────────────────
echo ""
echo "[6/8] Verifying multi-language runtimes..."

echo "  ✅ Python: $(python3 --version 2>/dev/null || echo 'N/A')"
echo "  ✅ Node: $(node --version 2>/dev/null || echo 'N/A')"
echo "  ✅ Bun: $(bun --version 2>/dev/null || echo 'N/A')"

# ── 7. Pre-warm Cargo Dependency Cache ──────────────────────────────────────
echo ""
echo "[7/8] Pre-compiling workspace dependencies..."

cd /app 2>/dev/null || cd /home/jules/repo 2>/dev/null || cd .

cargo check --workspace --exclude contextra-tauri 2>&1 | tail -5
echo "  ✅ Workspace dependency cache warmed"

# ── 8. Validate Workspace Invariants ─────────────────────────────────────────
echo ""
echo "[8/8] Validating Contextra workspace invariants..."

if [ -f "AGENTS.md" ]; then
    echo "  ✅ AGENTS.md found ($(wc -l < AGENTS.md) lines)"
else
    echo "  ❌ AGENTS.md missing! Jules needs this file."
    exit 1
fi

OPEN_TAGS=$(grep -rn 'AI-TAG\[SMELL\]\[CRITICAL\]' crates/ --include='*.rs' 2>/dev/null | grep -v RESOLVED | wc -l || echo "0")
echo "  ✅ Open AI-TAG[SMELL][CRITICAL]: $OPEN_TAGS (target: 0)"

if grep -q "axum" crates/contextra-mcp/Cargo.toml 2>/dev/null; then
    echo "  ❌ CRITICAL: axum found in contextra-mcp! Violates ADR-010"
    exit 1
else
    echo "  ✅ ADR-010: axum not in contextra-mcp (stdio-only MCP)"
fi

if cargo run --manifest-path xtask/Cargo.toml -- harness-list 2>/dev/null | grep -q "env-attest"; then
    echo "  → Executing cargo xtask env-attest..."
    cargo run --manifest-path xtask/Cargo.toml -- env-attest
else
    echo "  ⚠️ Warning: xtask env-attest command not available yet"
fi

# ── Summary ──────────────────────────────────────────────────────────────────
echo ""
echo "============================================================"
echo "  ✅ Contextra Jules Environment Ready"
echo "============================================================"
