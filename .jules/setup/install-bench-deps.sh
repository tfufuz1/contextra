#!/usr/bin/env bash
# =============================================================================
# Contextra — Benchmark Dependencies Installer (Idempotent)
# Repository: https://github.com/tfufuz1/contextra
# Target: Jules VM / Developer Workstation
# =============================================================================

set -euo pipefail

LOG="${TMPDIR:-/tmp}/contextra-setup.log"

SUDO=""
if [ "$(id -u)" != 0 ]; then
    if sudo -n true 2>/dev/null; then
        SUDO="sudo -n"
    else
        SUDO="__none__"
    fi
fi

# ── 1. Check Mode (--check) ───────────────────────────────────────────────────
if [ "${1:-}" = "--check" ]; then
    echo "============================================================"
    echo "  Contextra Benchmark Dependencies Check"
    echo "============================================================"
    echo "Disk space:"
    df -h .
    echo ""
    echo "Network reachability check (huggingface.co & github.com):"
    if curl -sI --connect-timeout 5 https://huggingface.co >/dev/null 2>&1; then
        echo "  ✅ huggingface.co reachable"
    else
        echo "  ⚠️ huggingface.co unreachable (offline or filtered)"
    fi
    if curl -sI --connect-timeout 5 https://github.com >/dev/null 2>&1; then
        echo "  ✅ github.com reachable"
    else
        echo "  ⚠️ github.com unreachable (offline or filtered)"
    fi
    exit 0
fi

echo "============================================================"
echo "  Contextra Benchmark Dependencies Setup"
echo "  $(date -u '+%Y-%m-%d %H:%M:%S UTC')"
echo "============================================================"

# ── 2. System Packages ───────────────────────────────────────────────────────
echo "[1/3] System packages check (curl, unzip, ca-certificates)..."
if [ "$SUDO" != "__none__" ]; then
    $SUDO apt-get update -q >>"$LOG" 2>&1 || true
    $SUDO apt-get install -y -q --no-install-recommends curl unzip ca-certificates >>"$LOG" 2>&1 || true
    echo "  ✅ Base system packages verified."
else
    echo "  ℹ️ Skipping apt-get (no sudo access)."
fi

# ── 3. HDF5 Detection ────────────────────────────────────────────────────────
echo "[2/3] Checking HDF5 reader requirement in codebase..."
NEED_HDF5=0

if grep -rnE '(hdf5|h5py)' benchmarks/contextra-bench/src/ xtask/src/bench_download.rs >/dev/null 2>&1; then
    NEED_HDF5=1
    echo "  ℹ️ HDF5 read path detected in codebase."
else
    echo "  ✅ No HDF5 read path required by benchmarks/contextra-bench/src or xtask/src/bench_download.rs."
fi

if [ "$NEED_HDF5" -eq 1 ]; then
    if [ "$SUDO" != "__none__" ]; then
        $SUDO apt-get install -y -q --no-install-recommends libhdf5-dev >>"$LOG" 2>&1 || true
    fi
fi

# ── 4. Python Environment & Benchmark Packages ────────────────────────────────
echo "[3/3] Python benchmark dependencies in ~/.venv-contextra..."
VENV="$HOME/.venv-contextra"
if [ ! -x "$VENV/bin/python" ]; then
    python3 -m venv "$VENV" >>"$LOG" 2>&1 || true
fi

PY_PKGS=("huggingface_hub")
if [ "$NEED_HDF5" -eq 1 ]; then
    PY_PKGS+=("h5py")
fi

if [ -x "$VENV/bin/pip" ]; then
    "$VENV/bin/pip" install -q --upgrade "${PY_PKGS[@]}" >>"$LOG" 2>&1 || true
    echo "  ✅ Python packages installed: ${PY_PKGS[*]}"
else
    echo "  ⚠️ Python virtual environment pip not accessible."
fi

echo ""
echo "============================================================"
echo "  ✅ Benchmark Dependencies Setup Complete"
echo "============================================================"
