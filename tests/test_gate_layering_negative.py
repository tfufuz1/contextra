#!/usr/bin/env python3
"""
tests/test_gate_layering_negative.py — Negativtest für Gate G-L1 (tools/gate_layering.py).

Fügt temporär in einer Kopie des Workspaces eine verbotene Kante ein und erwartet Exit-Code 1.
Normativ: docs/SPEC.md §8.1 Punkt 5, §7 (S-01)
"""

import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path


def main() -> int:
    repo_root = Path(__file__).resolve().parent.parent

    with tempfile.TemporaryDirectory() as tmp_dir_str:
        tmp_dir = Path(tmp_dir_str)
        # Kopiere Repo-Dateien minimal: Cargo.toml, Cargo.lock, capabilities.toml, tools/, crates/contextra-sys, crates/contextra-types
        for item in ["Cargo.toml", "Cargo.lock", "capabilities.toml"]:
            src = repo_root / item
            if src.exists():
                shutil.copy(src, tmp_dir / item)

        shutil.copytree(repo_root / "tools", tmp_dir / "tools")
        (tmp_dir / "crates").mkdir(parents=True, exist_ok=True)

        for c in ["contextra-sys", "contextra-types", "contextra-ids"]:
            src_c = repo_root / "crates" / c
            if src_c.exists():
                shutil.copytree(src_c, tmp_dir / "crates" / c)

        # Passe Root-Cargo.toml in tmp_dir an, sodass nur die kopierten Crates in members sind
        root_toml_path = tmp_dir / "Cargo.toml"
        with open(root_toml_path, "r", encoding="utf-8") as f:
            lines = f.readlines()

        new_lines = []
        in_members = False
        in_default_members = False
        for line in lines:
            if line.strip().startswith("members = ["):
                in_members = True
                new_lines.append(line)
                new_lines.append('    "crates/contextra-sys",\n')
                new_lines.append('    "crates/contextra-types",\n')
                new_lines.append('    "crates/contextra-ids",\n')
                continue
            if in_members:
                if line.strip().startswith("]"):
                    in_members = False
                    new_lines.append(line)
                continue
            if line.strip().startswith("default-members = ["):
                in_default_members = True
                new_lines.append(line)
                new_lines.append('    "crates/contextra-sys",\n')
                continue
            if in_default_members:
                if line.strip().startswith("]"):
                    in_default_members = False
                    new_lines.append(line)
                continue
            new_lines.append(line)

        with open(root_toml_path, "w", encoding="utf-8") as f:
            f.writelines(new_lines)

        # Verbotene Kante einfügen: contextra-sys soll unter [dependencies] unerlaubt von contextra-types abhängen
        sys_cargo_toml = tmp_dir / "crates" / "contextra-sys" / "Cargo.toml"
        with open(sys_cargo_toml, "r", encoding="utf-8") as f:
            sys_lines = f.readlines()

        inserted_lines = []
        for s_line in sys_lines:
            inserted_lines.append(s_line)
            if s_line.strip() == "[dependencies]":
                inserted_lines.append('contextra-types = { path = "../contextra-types" }\n')

        with open(sys_cargo_toml, "w", encoding="utf-8") as f:
            f.writelines(inserted_lines)

        # Führe gate_layering.py aus
        cmd = [sys.executable, str(tmp_dir / "tools" / "gate_layering.py"), "--crate", "contextra-sys", "--root", str(tmp_dir)]
        res = subprocess.run(cmd, capture_output=True, text=True)

        print("--- Output from negative test run ---")
        print("Exit code:", res.returncode)
        print("STDOUT:\n", res.stdout)
        print("STDERR:\n", res.stderr)
        print("-------------------------------------")

        if res.returncode == 1:
            print("OK: gate_layering.py hat den absichtlichen Verstoß korrekt mit Exit 1 abgefangen.")
            return 0
        else:
            print(f"FEHLER: Erwarteter Exit-Code 1, erhalten: {res.returncode}", file=sys.stderr)
            return 1


if __name__ == "__main__":
    sys.exit(main())
