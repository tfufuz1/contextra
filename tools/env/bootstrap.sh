#!/usr/bin/env bash
# Installiert Rust 1.89.0 (Projekt-Toolchain) ohne rustup, über conda-forge.
# Idempotent. Ausgabe auf stdout: nur export-Zeilen (eval "$(bash tools/env/bootstrap.sh)").
# Meldungen auf stderr. static.rust-lang.org ist auf der VM nicht erreichbar.
set -euo pipefail
PREFIX="${CTX_TOOLCHAIN:-$HOME/rust189}"
MM_ROOT="${MAMBA_ROOT_PREFIX:-$HOME/micromamba}"
MM="$HOME/.local/bin/micromamba"
log(){ echo "[bootstrap] $*" >&2; }

if [ ! -x "$PREFIX/bin/cargo" ]; then
  if [ ! -x "$MM" ]; then
    log "lade micromamba 2.10.0 von conda-forge"
    mkdir -p "$(dirname "$MM")"
    tmp=$(mktemp -d)
    curl -fsSL https://conda.anaconda.org/conda-forge/linux-64/micromamba-2.10.0-0.tar.bz2 -o "$tmp/mm.tar.bz2"
    tar -xjf "$tmp/mm.tar.bz2" -C "$tmp" bin/micromamba
    cp "$tmp/bin/micromamba" "$MM"; rm -rf "$tmp"
  fi
  log "installiere rust=1.89.0 nach $PREFIX (dauert mehrere Minuten)"
  MAMBA_ROOT_PREFIX="$MM_ROOT" "$MM" create -y -q -p "$PREFIX" -c conda-forge --override-channels "rust=1.89.0" >&2
  rm -rf "$MM_ROOT/pkgs"   # Paket-Cache entfernen, sonst Platzmangel
  log "fertig"
fi
ver=$("$PREFIX/bin/cargo" --version | awk '{print $2}')
[ "$ver" = "1.89.0" ] || { log "FEHLER: cargo $ver statt 1.89.0"; exit 1; }
log "cargo $ver OK"
cat <<EXP
export PATH="$PREFIX/bin:\$PATH"
export CARGO_TERM_PROGRESS_WHEN=never
export CARGO_TERM_COLOR=never
export CARGO_INCREMENTAL=0
export CARGO_BUILD_JOBS=1
EXP
