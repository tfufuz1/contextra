#!/usr/bin/env bash
# =============================================================================
# Contextra — Benchmark Dependencies Installation Script
# Repository: https://github.com/tfufuz1/contextra
# Target: Jules VM / Developer Workstation
# =============================================================================

set -euo pipefail

LOG="${TMPDIR:-/tmp}/contextra-bench-deps.log"
VENV="$HOME/.venv-contextra"

SUDO=""
if [ "$(id -u)" != 0 ]; then
  if sudo -n true 2>/dev/null; then
    SUDO="sudo -n"
  else
    SUDO="__none__"
  fi
fi

# ── 1. Check mode ─────────────────────────────────────────────────────────────
if [ "${1:-}" = "--check" ]; then
  echo "============================================================"
  echo "  Contextra Benchmark Dependencies Check (--check)"
  echo "  $(date -u '+%Y-%m-%d %H:%M:%S UTC')"
  echo "============================================================"

  echo "Checking tool availability..."
  for tool in curl unzip ca-certificates; do
    if command -v "$tool" >/dev/null 2>&1 || dpkg -l "$tool" >/dev/null 2>&1; then
      echo "  ✅ System package / command '$tool' is available."
    else
      echo "  ⚠️ System package / command '$tool' is MISSING."
    fi
  done

  if [ -x "$VENV/bin/python" ]; then
    if "$VENV/bin/python" -c "import huggingface_hub" >/dev/null 2>&1; then
      echo "  ✅ Python module 'huggingface_hub' is available in $VENV."
    else
      echo "  ⚠️ Python module 'huggingface_hub' is NOT installed in $VENV."
    fi

    if "$VENV/bin/python" -c "import h5py" >/dev/null 2>&1; then
      echo "  ✅ Python module 'h5py' is available in $VENV."
    else
      echo "  ℹ️ Python module 'h5py' is NOT installed in $VENV."
    fi
  else
    echo "  ⚠️ Virtualenv $VENV does not exist."
  fi

  echo ""
  echo "Checking network reachability..."
  if curl -sI --max-time 5 https://huggingface.co >/dev/null 2>&1; then
    echo "  ✅ Network reachability to https://huggingface.co OK."
  else
    echo "  ⚠️ Cannot reach https://huggingface.co (offline or blocked)."
  fi

  echo ""
  echo "Available disk space:"
  df -h . | tail -n 1 | awk '{print "  Free: " $4 " (Total: " $2 ", Used: " $3 ", Mount: " $6 ")"}'

  echo "============================================================"
  exit 0
fi

echo "============================================================"
echo "  Contextra Benchmark Dependencies Setup"
echo "  $(date -u '+%Y-%m-%d %H:%M:%S UTC')"
echo "============================================================"

# ── 2. Install System Packages ────────────────────────────────────────────────
echo "[1/3] Checking system packages (curl, unzip, ca-certificates)..."
if [ "$SUDO" != "__none__" ]; then
  $SUDO apt-get update -q >>"$LOG" 2>&1 || true
  $SUDO apt-get install -y -q --no-install-recommends curl unzip ca-certificates >>"$LOG" 2>&1 || true
  echo "  ✅ System packages updated."
else
  echo "  ⚠️ No passwordless sudo available, skipping apt package installation."
fi

# ── 3. Check for HDF5 requirement ───────────────────────────────────────────
echo "[2/3] Checking if HDF5 dependency (hdf5 / h5py) is required..."
HDF5_NEEDED=0
if grep -E -q 'hdf5|h5py' benchmarks/contextra-bench/src xtask/src/bench_download.rs 2>/dev/null; then
  HDF5_NEEDED=1
fi

if [ "$HDF5_NEEDED" -eq 1 ]; then
  echo "  ℹ️ HDF5 dependency detected in source files. Installing libhdf5-dev and h5py..."
  if [ "$SUDO" != "__none__" ]; then
    $SUDO apt-get install -y -q --no-install-recommends libhdf5-dev >>"$LOG" 2>&1 || true
  fi
  if [ -x "$VENV/bin/pip" ]; then
    "$VENV/bin/pip" install -q h5py >>"$LOG" 2>&1 || echo "  ⚠️ Warning: failed to install h5py via pip"
  fi
  echo "  ✅ HDF5 dependencies installed."
else
  echo "  ℹ️ No HDF5 read path (hdf5/h5py) required in source code; skipping libhdf5-dev / h5py."
fi

# ── 4. Install Python Dependencies in Venv ───────────────────────────────────
echo "[3/3] Installing Python benchmark dependencies (huggingface_hub)..."
if [ ! -x "$VENV/bin/python" ]; then
  python3 -m venv "$VENV" >>"$LOG" 2>&1 || true
fi

if [ -x "$VENV/bin/pip" ]; then
  "$VENV/bin/pip" install -q huggingface_hub >>"$LOG" 2>&1 || echo "  ⚠️ Warning: failed to install huggingface_hub"
  echo "  ✅ Python dependencies installed in $VENV."
else
  echo "  ⚠️ Warning: pip not found in $VENV."
fi

echo ""
echo "============================================================"
echo "  ✅ Benchmark Dependencies Setup Complete"
echo "============================================================"
