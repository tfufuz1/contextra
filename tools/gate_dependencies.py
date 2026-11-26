#!/usr/bin/env python3
"""
tools/gate_dependencies.py — Gate G1 (Isolation & Layering Enforcement per SPEC v4.1 §8 / Gate G1).

Normative Kriterien (SPEC §2 B1, B2, §5, §8):
- Klasse A (Autarke Bausteine): 0 interne Abhängigkeiten zu anderen `contextra-*`-Crates in [dependencies] und [build-dependencies].
- Klasse A Dev-Deps: 0 Dev-Abhängigkeiten zu `contextra-*`. `contextra-testkit` ist strikt verboten.
- Klasse B (Storage-Bundle): Nur Kanten innerhalb Klasse B.
- Klasse C (Kompositions- & Integrationsschicht): Kanten gemäß capabilities.toml `may_depend_on`.
- Klasse T (Test & Benchmark): Nur als dev-dependency in Klasse B/C.
- transitional: Erlaubte Übergangskanten melden Warnungen mit Verfallsdatum "Welle 6".
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import tomllib
from pathlib import Path
from typing import Any, Dict, List, Set, Tuple


def run_gate_dependencies(caps_path: Path, root: Path) -> Tuple[List[str], List[str], Dict[str, Any]]:
    if not caps_path.exists():
        return [f"capabilities.toml nicht gefunden unter {caps_path}"], [], {}

    with open(caps_path, "rb") as f:
        caps_data = tomllib.load(f)

    crates_spec: Dict[str, dict] = caps_data.get("crates", {})

    cmd = ["cargo", "metadata", "--format-version", "1"]
    res = subprocess.run(cmd, cwd=root, capture_output=True, text=True)
    if res.returncode != 0:
        return [f"cargo metadata fehlgeschlagen: {res.stderr}"], [], {}

    meta = json.loads(res.stdout)
    packages = {p["name"]: p for p in meta["packages"]}
    workspace_members = set(meta["workspace_members"])

    violations: List[str] = []
    warnings: List[str] = []

    klasse_b_crates: Set[str] = {
        name for name, spec in crates_spec.items() if spec.get("klasse") == "B"
    }

    report_details: Dict[str, Any] = {
        "crates_checked": 0,
        "class_counts": {},
        "violations": [],
        "warnings": [],
    }

    for pkg in meta["packages"]:
        if pkg["id"] not in workspace_members:
            continue
        pkg_name = pkg["name"]
        spec = crates_spec.get(pkg_name)
        if not spec:
            violations.append(f"Crate `{pkg_name}` ist im Workspace, fehlt aber in `capabilities.toml`.")
            continue

        report_details["crates_checked"] += 1
        klasse = spec.get("klasse", "")
        report_details["class_counts"][klasse] = report_details["class_counts"].get(klasse, 0) + 1
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
                    elif dep_name not in allowed and dep_name not in klasse_b_crates:
                        violations.append(
                            f"[G1-VIOLATION] Klasse-B-Crate `{pkg_name}` hängt von `{dep_name}` ({kind}) ab (nicht in may_depend_on und nicht in Klasse B)."
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

    report_details["violations"] = violations
    report_details["warnings"] = warnings
    return violations, warnings, report_details


def main() -> int:
    parser = argparse.ArgumentParser(description="Gate G1 Crate-Klassen & Dependencies Validator (SPEC v4.1 §8 / G1)")
    parser.add_argument("--capabilities", default="capabilities.toml", help="Pfad zu capabilities.toml")
    parser.add_argument("--root", default=".", help="Workspace-Root")
    parser.add_argument("--json", action="store_true", help="JSON-Format für CI/Agent-Pipelines ausgeben")
    parser.add_argument("--strict", action="store_true", help="Failt auch bei Warnungen/Transitional-Kanten")
    args = parser.parse_args()

    root = Path(args.root).resolve()
    caps = Path(args.capabilities).resolve()

    violations, warnings, details = run_gate_dependencies(caps, root)

    if args.json:
        payload = {
            "gate": "G1",
            "name": "gate_dependencies",
            "status": "PASS" if not violations and (not warnings or not args.strict) else "FAIL",
            "violations_count": len(violations),
            "warnings_count": len(warnings),
            "violations": violations,
            "warnings": warnings,
            "details": details,
        }
        print(json.dumps(payload, indent=2))
        if violations or (args.strict and warnings):
            return 1
        return 0

    print("=== GATE G1: Crate-Klassen & Layering-Report (SPEC v4.1 §8) ===")
    if warnings:
        print(f"\n⚠️  {len(warnings)} Übergangskanten (transitional, Ablauf Welle 6):")
        for w in warnings:
            print(f"  - {w}")

    if violations:
        print(f"\n❌ {len(violations)} Verstöße gegen Gate G1 identifiziert:")
        for v in violations:
            print(f"  - {v}")
        return 1

    print("\n✅ Gate G1 bestanden: Alle internen Abhängigkeiten entsprechen capabilities.toml und SPEC v4.1.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
