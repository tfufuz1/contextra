---
name: acta-toolchain
description: "Stellt Rust 1.89.0 für das Acta-Repo bereit, wenn static.rust-lang.org nicht erreichbar ist, und prüft die Toolchain. Verwenden, wenn cargo oder rustc die Toolchain 1.89.0 nicht finden, wenn rustup einen Download-Fehler meldet oder wenn eine neue VM eingerichtet wird."
---

# Rust 1.89.0 bereitstellen (Acta)

Das Repo pinnt `channel = "1.89.0"` in `rust-toolchain.toml`. In der VM ist `static.rust-lang.org` blockiert. Die Toolchain kommt deshalb aus conda-forge.

## 0. Automatisch
Normalerweise erledigt der SessionStart-Hook alles (`scripts/session-start.sh`). Manuell: `bash scripts/bootstrap-toolchain.sh` (Rust) und `bash scripts/bootstrap-tools.sh` (just, cargo-deny, -audit, -nextest, -mutants, -llvm-cov, -cyclonedx). Beide Skripte sind idempotent und getestet; die Schritte unten erklaeren, was sie tun.

## 1. Vorprüfung

```
rustup toolchain list
rustc --version
```

Wenn `1.89.0-x86_64-unknown-linux-gnu` gelistet ist und `rustc --version` 1.89.0 zeigt, ist nichts zu tun. Weiter mit Schritt 4.

## 2. Pakete laden

```
mkdir -p /root/scratch /root/toolchains
cd /root/scratch
curl -sS -o rust189.conda https://conda.anaconda.org/conda-forge/linux-64/rust-1.89.0-h53717f1_0.conda
curl -sS -o rust_std189.conda https://conda.anaconda.org/conda-forge/noarch/rust-std-x86_64-unknown-linux-gnu-1.89.0-h2c6d0dc_0.conda
pip install -q zstandard
```

Die Dateien sind ZIP-Archive. Enthalten sind `pkg-*.tar.zst` und `info-*.tar.zst`.

## 3. Entpacken und zusammenführen

```python
import zipfile, tarfile, zstandard, io
dest = "/root/toolchains/rust-1.89.0"
for pkg in ["rust189.conda", "rust_std189.conda"]:
    z = zipfile.ZipFile(pkg)
    for n in z.namelist():
        if n.startswith("pkg-") and n.endswith(".tar.zst"):
            r = zstandard.ZstdDecompressor().stream_reader(io.BytesIO(z.read(n)))
            with tarfile.open(fileobj=r, mode="r|") as t:
                t.extractall(dest)
```

Danach die Binaries ausführbar machen und die Bibliotheken prüfen:

```
chmod +x /root/toolchains/rust-1.89.0/bin/*
/root/toolchains/rust-1.89.0/bin/rustc -vV
ldd /root/toolchains/rust-1.89.0/bin/rustc | grep "not found" || echo "libs ok"
ls /root/toolchains/rust-1.89.0/lib/rustlib/x86_64-unknown-linux-gnu/lib | grep -c rlib
```

Die rlib-Zahl muss größer als 0 sein. Fehlen die rlibs, war das `rust-std`-Paket nicht korrekt entpackt. Dann tritt später E0463 auf.

## 4. In rustup einbinden

rustup lehnt den Namen `1.89.0` für Custom-Toolchains ab. Der Symlink mit dem Standard-Triple funktioniert:

```
ln -sfn /root/toolchains/rust-1.89.0 /root/.rustup/toolchains/1.89.0-x86_64-unknown-linux-gnu
cd /root/acta && rustc --version && cargo --version
```

Erwartet: `rustc 1.89.0 (29483883e 2025-08-04)`.

## 5. Prüfung

```
cd /root/acta
cargo fetch --locked
cargo check --workspace --all-targets --locked
```

Beide Befehle müssen mit Exit 0 enden.

## Bekannte Stolpersteine

- Ohne `rust-std` schlägt jeder Build mit `E0463: can't find crate for std` fehl. Das betrifft auch die Build-Skripte von `libc` und `proc-macro2`.
- `rustup toolchain link 1.89.0 …` wird abgelehnt. Nur der Symlink funktioniert.
- `conda`, `micromamba` und `api.anaconda.org` sind nicht verfügbar. Pakete werden direkt per `curl` geladen.
- Netzfehler von rustup erzeugen lange Backtraces. Zuerst `rustup toolchain list` prüfen.

## Werkzeuge
`bash scripts/bootstrap-tools.sh` installiert feste Versionen aus conda-forge. `cargo-geiger` per `cargo +stable install cargo-geiger --locked --root /root/toolchains/geiger` (0.14 braucht rustc >= 1.97). Nightly, Miri und cargo-fuzz sind in dieser VM nicht beschaffbar (siehe `docs/UMGEBUNG.md`).
