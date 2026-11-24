#!/usr/bin/env bash
# Installiert Rust 1.89.0 ohne static.rust-lang.org über conda-forge (Umweg, wenn rustup nicht erreichbar ist).
# Lädt die Pakete rust und rust-std (noarch) und entpackt sie nach $PREFIX (Standard: $HOME/rust-1.89.0).
set -euo pipefail
PREFIX="${PREFIX:-$HOME/rust-1.89.0}"
CF=https://conda.anaconda.org/conda-forge
TMP=$(mktemp -d)
curl -fsSL -o "$TMP/rust.conda" "$CF/linux-64/rust-1.89.0-h53717f1_0.conda"
curl -fsSL -o "$TMP/rust-std.conda" "$CF/noarch/rust-std-x86_64-unknown-linux-gnu-1.89.0-h2c6d0dc_0.conda"
mkdir -p "$PREFIX"
python3 - "$TMP" "$PREFIX" <<PY
import sys, zipfile, tarfile, io, zstandard, glob
tmp, prefix = sys.argv[1], sys.argv[2]
for conda in ("rust.conda", "rust-std.conda"):
    z = zipfile.ZipFile(f"{tmp}/{conda}")
    pkg = [n for n in z.namelist() if n.startswith("pkg-")][0]
    with zstandard.ZstdDecompressor().stream_reader(io.BytesIO(z.read(pkg))) as r:
        with tarfile.open(fileobj=r, mode="r|") as t:
            t.extractall(prefix)
PY
rm -rf "$TMP"
echo "Installiert: $PREFIX/bin/rustc"
echo "Umgebung: export PATH=$PREFIX/bin:\$PATH LD_LIBRARY_PATH=$PREFIX/lib"
"$PREFIX/bin/rustc" --version
