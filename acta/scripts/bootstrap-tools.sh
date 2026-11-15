#!/usr/bin/env bash
# Installiert Entwicklungswerkzeuge als Binaries aus conda-forge (crates.io-Builds dauern auf 2 Kernen lange).
# Idempotent. Ziel: $ACTA_TOOLS_DIR/bin (Standard $HOME/toolchains/tools/bin), verlinkt nach $CARGO_HOME/bin.
set -euo pipefail
if [ "$(uname -s)-$(uname -m)" != "Linux-x86_64" ]; then
  echo "[skip] conda-forge-Bootstrap ist nur fuer Linux x86_64 gedacht. Lokal: rustup toolchain install 1.89.0 und cargo install just cargo-deny cargo-nextest cargo-mutants cargo-llvm-cov cargo-audit cargo-cyclonedx --locked"
  exit 0
fi
TOOLS="${ACTA_TOOLS_DIR:-$HOME/toolchains/tools}"
WORK="${ACTA_WORKDIR:-/tmp/acta-bootstrap}"
LINKDIR="${ACTA_LINK_DIR:-${CARGO_HOME:-$HOME/.cargo}/bin}"
BASE=https://conda.anaconda.org/conda-forge
# name -> Paketdatei (feste, geprueft vorhandene Versionen; Stand 2026-11-15)
PKGS=(
  "just:just-1.9.0-h2f28480_0"
  "cargo-deny:cargo-deny-0.20.2-hb17b654_0"
  "cargo-audit:cargo-audit-0.22.1-hb17b654_0"
  "cargo-cyclonedx:cargo-cyclonedx-0.5.9-hb17b654_1"
  "cargo-mutants:cargo-mutants-27.1.0-hf19af3b_0"
  "cargo-nextest:cargo-nextest-0.9.99-hb23c6cf_0"
  "cargo-llvm-cov:cargo-llvm-cov-0.9.1-h8569b68_0"
)
mkdir -p "$TOOLS/bin" "$WORK" "$LINKDIR"
python3 -c 'import zstandard' 2>/dev/null || pip install -q --break-system-packages zstandard
for e in "${PKGS[@]}"; do
  bin="${e%%:*}"; pkg="${e#*:}"
  if [ -x "$TOOLS/bin/$bin" ]; then echo "[ok]   $bin vorhanden"; else
    f="$WORK/$pkg.conda"
    if [ ! -s "$f" ]; then
      curl -fsS --retry 3 -o "$f" "$BASE/linux-64/$pkg.conda" || curl -fsS --retry 3 -o "$f" "$BASE/noarch/$pkg.conda"
    fi
    python3 -I - "$TOOLS" "$f" <<'PY'
import sys, zipfile, tarfile, io, zstandard
dest, pkg = sys.argv[1:]
z = zipfile.ZipFile(pkg)
for n in z.namelist():
    if n.startswith("pkg-") and n.endswith(".tar.zst"):
        r = zstandard.ZstdDecompressor().stream_reader(io.BytesIO(z.read(n)))
        with tarfile.open(fileobj=r, mode="r|") as t:
            for m in t:
                if m.isfile() and m.name.startswith("bin/"):
                    m.name = m.name; t.extract(m, dest, filter="data")
PY
    chmod +x "$TOOLS/bin/$bin"; echo "[inst] $bin"
  fi
  ln -sfn "$TOOLS/bin/$bin" "$LINKDIR/$bin"
done
echo "--- Versionen ---"
for e in "${PKGS[@]}"; do
  b="${e%%:*}"; printf '%-16s' "$b"
  case "$b" in
    cargo-*) cargo "${b#cargo-}" --version 2>&1 | head -1 ;;
    *) "$TOOLS/bin/$b" --version 2>&1 | head -1 ;;
  esac
done
