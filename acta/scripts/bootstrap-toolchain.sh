#!/usr/bin/env bash
# Stellt Rust 1.89.0 ueber conda-forge bereit (static.rust-lang.org ist in der Cloud-VM blockiert).
# Idempotent. Umgebungsvariablen: ACTA_TOOLCHAIN_DIR (Standard $HOME/toolchains/rust-1.89.0),
# ACTA_SKIP_RUSTUP_LINK=1 (nur entpacken), ACTA_WORKDIR (Standard /tmp/acta-bootstrap).
set -euo pipefail
if [ "$(uname -s)-$(uname -m)" != "Linux-x86_64" ]; then
  echo "[skip] conda-forge-Bootstrap ist nur fuer Linux x86_64 gedacht. Lokal: rustup toolchain install 1.89.0 und cargo install just cargo-deny cargo-nextest cargo-mutants cargo-llvm-cov cargo-audit cargo-cyclonedx --locked"
  exit 0
fi
VER=1.89.0
DEST="${ACTA_TOOLCHAIN_DIR:-$HOME/toolchains/rust-$VER}"
WORK="${ACTA_WORKDIR:-/tmp/acta-bootstrap}"
RUSTUP_HOME="${RUSTUP_HOME:-$HOME/.rustup}"
BASE=https://conda.anaconda.org/conda-forge
RUST_PKG="$BASE/linux-64/rust-$VER-h53717f1_0.conda"
STD_PKG="$BASE/noarch/rust-std-x86_64-unknown-linux-gnu-$VER-h2c6d0dc_0.conda"

if [ -x "$DEST/bin/rustc" ] && ls "$DEST"/lib/rustlib/x86_64-unknown-linux-gnu/lib/libstd-*.rlib >/dev/null 2>&1; then
  echo "[ok] Toolchain vorhanden: $("$DEST/bin/rustc" --version)"
else
  mkdir -p "$WORK" "$DEST"
  for u in "$RUST_PKG" "$STD_PKG"; do
    f="$WORK/$(basename "$u")"
    [ -s "$f" ] || curl -fsS --retry 3 -o "$f" "$u"
  done
  python3 -c 'import zstandard' 2>/dev/null || pip install -q --break-system-packages zstandard
  python3 -I - "$DEST" "$WORK/$(basename "$RUST_PKG")" "$WORK/$(basename "$STD_PKG")" <<'PY'
import sys, zipfile, tarfile, io, zstandard
dest, *pkgs = sys.argv[1:]
for pkg in pkgs:
    z = zipfile.ZipFile(pkg)
    for n in z.namelist():
        if n.startswith("pkg-") and n.endswith(".tar.zst"):
            r = zstandard.ZstdDecompressor().stream_reader(io.BytesIO(z.read(n)))
            with tarfile.open(fileobj=r, mode="r|") as t:
                t.extractall(dest, filter="data")
PY
  chmod +x "$DEST"/bin/*
  ls "$DEST"/lib/rustlib/x86_64-unknown-linux-gnu/lib/libstd-*.rlib >/dev/null  # std muss vorhanden sein (sonst E0463)
  echo "[ok] entpackt: $("$DEST/bin/rustc" --version)"
fi

if [ "${ACTA_SKIP_RUSTUP_LINK:-0}" != 1 ]; then
  # `rustup toolchain link 1.89.0` wird abgelehnt; Symlink mit vollem Triple funktioniert.
  mkdir -p "$RUSTUP_HOME/toolchains"
  ln -sfn "$DEST" "$RUSTUP_HOME/toolchains/$VER-x86_64-unknown-linux-gnu"
  echo "[ok] rustup: $(rustup toolchain list | grep "^$VER" || true)"
fi
