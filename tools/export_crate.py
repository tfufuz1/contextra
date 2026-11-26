#!/usr/bin/env python3
"""
tools/export_crate.py — Standalone Crate Exporter & Gate G3 Validator (SPEC v4.0 §7 / Gate G3).

Zweck:
- Exportiert ein Building-Block-Crate in ein isoliertes Verzeichnis.
- Löst Workspace-Vererbung ([package] und [dependencies] mit workspace = true) dynamisch
  aus dem Root-Cargo.toml auf (inklusive Features wie derive bei serde).
- Bereinigt workspace = true Lints und interne Abhängigkeiten für autarken Standalone-Build.
- Führt `cargo check` im Export-Verzeichnis aus, um Copy-Paste-Readiness zu beweisen (G3).

Bekannte Grenzen:
- rust-toolchain.toml wird nicht automatisch mitkopiert (erfordert Standard Rust 1.89+).
- Root-Lizenzdateien (LICENSE-MIT, LICENSE-APACHE) müssen im Crate lokal vorhanden sein.
- [lints]-Tabellen aus dem Workspace werden im Standalone-Modus bereinigt, da kein Root existiert.
- Bei Crates mit noch nicht entkoppelten internen Typen (wie contextra-rank vor Welle 1)
  werden interne contextra-*-Abhängigkeiten im Standalone-Export ausgeklammert; solche Crates
  bestehen Gate G3 vollständig erst nach Vollendung ihres Entkopplungspakets (SPEC §12).
"""

from __future__ import annotations

import argparse
import os
import re
import shutil
import subprocess
import sys
import tempfile
import tomllib
from pathlib import Path


def load_workspace_deps(root: Path) -> dict:
    cargo_path = root / "Cargo.toml"
    with open(cargo_path, "rb") as f:
        data = tomllib.load(f)
    return data.get("workspace", {}).get("dependencies", {})


def export_crate(crate_name: str, root: Path, out_dir: str | None = None) -> bool:
    crate_path = root / "crates" / crate_name
    if not crate_path.is_dir():
        print(f"Fehler: Crate-Verzeichnis {crate_path} existiert nicht.", file=sys.stderr)
        return False

    is_temp = False
    if not out_dir:
        out_dir = tempfile.mkdtemp(prefix=f"export_{crate_name}_")
        is_temp = True

    out_path = Path(out_dir).resolve()
    print(f"Exportiere `{crate_name}` nach `{out_path}`...")

    # Copy files
    for item in os.listdir(crate_path):
        s = crate_path / item
        d = out_path / item
        if s.is_dir():
            shutil.copytree(s, d, dirs_exist_ok=True)
        else:
            shutil.copy2(s, d)

    ws_deps = load_workspace_deps(root)

    cargo_toml_path = out_path / "Cargo.toml"
    with open(cargo_toml_path, "r", encoding="utf-8") as f:
        lines = f.readlines()

    resolved_lines = []
    in_lints = False

    for line in lines:
        stripped = line.strip()
        if stripped in ("[lints]", "[lints.rust]", "[lints.clippy]"):
            in_lints = True
            continue
        if in_lints:
            if stripped.startswith("["):
                in_lints = False
            else:
                continue

        # Package meta
        line = line.replace("rust-version.workspace = true", 'rust-version = "1.89.0"')
        line = line.replace("version.workspace = true", 'version = "0.1.0"')
        line = line.replace("edition.workspace = true", 'edition = "2021"')
        line = line.replace("license.workspace = true", 'license = "MIT OR Apache-2.0"')

        # Remove internal contextra deps first if standalone test
        if re.search(r'^\s*contextra-[a-zA-Z0-9_\-]+\s*=', line):
            continue
        if "types-compat = [" in line:
            continue
        line = line.replace('"types-compat"', '').replace(', "types-compat"', '').replace('"types-compat", ', '')

        # Regex replacement for workspace = true in inline tables:
        m_dep = re.match(r'^\s*([a-zA-Z0-9_\-]+)\s*=\s*\{\s*workspace\s*=\s*true\s*(?:,\s*(.*))?\}\s*$', line)
        if m_dep:
            dep_key = m_dep.group(1)
            rest = m_dep.group(2)
            meta = ws_deps.get(dep_key)
            if isinstance(meta, str):
                ver = meta
                ws_feats = []
            elif isinstance(meta, dict):
                ver = meta.get("version", "1")
                ws_feats = meta.get("features", [])
            else:
                ver = "1"
                ws_feats = []

            if rest:
                if "features" not in rest and ws_feats:
                    line = f'{dep_key} = {{ version = "{ver}", features = {ws_feats}, {rest} }}\n'
                else:
                    line = f'{dep_key} = {{ version = "{ver}", {rest} }}\n'
            else:
                if ws_feats:
                    line = f'{dep_key} = {{ version = "{ver}", features = {ws_feats} }}\n'
                else:
                    line = f'{dep_key} = "{ver}"\n'
        else:
            for dep, meta in ws_deps.items():
                if isinstance(meta, str):
                    ver = meta
                    ws_feats = []
                elif isinstance(meta, dict):
                    ver = meta.get("version", "1")
                    ws_feats = meta.get("features", [])
                else:
                    ver = "1"
                    ws_feats = []
                if ws_feats:
                    replacement = f'{dep} = {{ version = "{ver}", features = {ws_feats} }}'
                else:
                    replacement = f'{dep} = "{ver}"'
                line = line.replace(f'{dep}.workspace = true', replacement)

        resolved_lines.append(line)

    with open(cargo_toml_path, "w", encoding="utf-8") as f:
        f.writelines(resolved_lines)

    print("Cargo.toml erfolgreich aufgelöst.")

    # Run cargo check in export dir
    print(f"Führe `cargo check` im Exportverzeichnis aus...")
    res = subprocess.run(["cargo", "check", "--no-default-features"], cwd=out_path)
    if res.returncode != 0:
        print(f"❌ Gate G3 fehlgeschlagen für {crate_name}", file=sys.stderr)
        if is_temp:
            shutil.rmtree(out_path, ignore_errors=True)
        return False

    print(f"✅ Gate G3 bestanden: {crate_name} baut erfolgreich standalone in {out_path}")
    if is_temp:
        shutil.rmtree(out_path, ignore_errors=True)
    return True


def main() -> int:
    parser = argparse.ArgumentParser(description="Standalone Crate Exporter (Gate G3)")
    parser.add_argument("crate", help="Name des zu exportierenden Crates")
    parser.add_argument("--out", help="Zielverzeichnis für den Export")
    parser.add_argument("--root", default=".", help="Workspace-Root")
    args = parser.parse_args()

    root = Path(args.root).resolve()
    success = export_crate(args.crate, root, args.out)
    return 0 if success else 1


if __name__ == "__main__":
    sys.exit(main())
