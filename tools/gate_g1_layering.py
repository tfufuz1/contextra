#!/usr/bin/env python3
"""
tools/gate_g1_layering.py — Gate G1 (Isolation & Layering Enforcement per SPEC v4.0 §8.1).

Prüfungen:
- Je Crate Soll-Liste interner Deps aus capabilities.toml (may_depend_on, transitional)
- Ist aus cargo metadata (normal, build, dev mit aufgelösten Features)
- Klasse A: Soll may_depend_on ist leer, contextra-testkit ist strikt verboten.
- transitional: Erlaubte Übergangskanten melden Warnungen mit Verfallsdatum "Welle 6".
- Jede nicht erlaubte Kante führt zu Exit-Code 1 (Fail-Closed).
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import tomllib
from pathlib import Path
from typing import Dict, List, Set, Tuple


def run_gate_g1(caps_path: Path, root: Path) -> Tuple[List[str], List[str]]:
    if not caps_path.exists():
        return [f"capabilities.toml nicht gefunden unter {caps_path}"], []

    with open(caps_path, "rb") as f:
        caps_data = tomllib.load(f)

    crates_spec: Dict[str, dict] = caps_data.get("crates", {})

    cmd = ["cargo", "metadata", "--format-version", "1"]
    res = subprocess.run(cmd, cwd=root, capture_output=True, text=True)
    if res.returncode != 0:
        return [f"cargo metadata fehlgeschlagen: {res.stderr}"], []

    meta = json.loads(res.stdout)
    packages = {p["name"]: p for p in meta["packages"]}
    workspace_members = set(meta["workspace_members"])

    violations: List[str] = []
    warnings: List[str] = []

    for pkg in meta["packages"]:
        if pkg["id"] not in workspace_members:
            continue
        pkg_name = pkg["name"]
        spec = crates_spec.get(pkg_name)
        if not spec:
            violations.append(f"Crate `{pkg_name}` ist im Workspace, fehlt aber in `capabilities.toml`.")
            continue

        klasse = spec.get("klasse", "")
        allowed = set(spec.get("may_depend_on", []))
        transitional = set(spec.get("transitional", []))

        for dep in pkg["dependencies"]:
            dep_name = dep["name"]
            # Nur interne Workspace-Abhängigkeiten prüfen
            if dep_name in packages and packages[dep_name]["id"] in workspace_members:
                kind = dep.get("kind") or "normal"

                # Klasse A: Darf laut SPEC §8.1 / §5.1 keine internen Abhängigkeiten haben; testkit strikt verboten
                if klasse == "A":
                    if dep_name == "contextra-testkit":
                        violations.append(
                            f"[G1-VIOLATION] Klasse-A-Crate `{pkg_name}` hat verbotene Dev-Abhängigkeit auf `contextra-testkit` (E-12)."
                        )
                    elif dep_name in transitional:
                        warnings.append(
                            f"[G1-WARN-TRANSITIONAL] Klasse-A `{pkg_name}` -> `{dep_name}` ({kind}) ist transitional (Ablauf: Welle 6)."
                        )
                    else:
                        violations.append(
                            f"[G1-VIOLATION] Klasse-A-Crate `{pkg_name}` hat unzulässige interne Abhängigkeit auf `{dep_name}` ({kind})."
                        )
                elif klasse == "B":
                    if dep_name in transitional:
                        warnings.append(
                            f"[G1-WARN-TRANSITIONAL] Klasse-B `{pkg_name}` -> `{dep_name}` ({kind}) ist transitional (Ablauf: Welle 6)."
                        )
                    elif dep_name not in allowed:
                        violations.append(
                            f"[G1-VIOLATION] Klasse-B-Crate `{pkg_name}` hängt von `{dep_name}` ({kind}) ab (nicht in may_depend_on)."
                        )
                elif klasse in ("C", "T"):
                    if dep_name in transitional:
                        warnings.append(
                            f"[G1-WARN-TRANSITIONAL] Klasse-{klasse} `{pkg_name}` -> `{dep_name}` ({kind}) ist transitional (Ablauf: Welle 6)."
                        )
                    elif dep_name not in allowed:
                        violations.append(
                            f"[G1-VIOLATION] Klasse-{klasse}-Crate `{pkg_name}` hängt von `{dep_name}` ({kind}) ab (nicht in may_depend_on)."
                        )
                else:
                    violations.append(
                        f"[G1-VIOLATION] Crate `{pkg_name}` hat ungültige Klasse '{klasse}' in capabilities.toml."
                    )

    return violations, warnings


def main() -> int:
    parser = argparse.ArgumentParser(description="Gate G1 Layering & Isolation Validator")
    parser.add_argument("--capabilities", default="capabilities.toml", help="Pfad zu capabilities.toml")
    parser.add_argument("--root", default=".", help="Workspace-Root")
    args = parser.parse_args()

    root = Path(args.root).resolve()
    caps = Path(args.capabilities).resolve()

    violations, warnings = run_gate_g1(caps, root)

    print("=== GATE G1: Layering- & Isolations-Report (SPEC v4.0 §8.1) ===")
    if warnings:
        print(f"\n⚠️  {len(warnings)} Übergangskanten (transitional, Ablauf Welle 6):")
        for w in warnings:
            print(f"  - {w}")

    if violations:
        print(f"\n❌ {len(violations)} Verstöße gegen Gate G1 identifiziert:")
        for v in violations:
            print(f"  - {v}")
        return 1

    print("\n✅ Gate G1 bestanden: Alle internen Abhängigkeiten entsprechen capabilities.toml und SPEC v4.0.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
