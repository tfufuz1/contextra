#!/usr/bin/env python3
"""
tools/gate_dependencies.py — Gate G1 (Isolation & Dependency Enforcement per SPEC §8.1 / capabilities.toml Schema v3).

Normative Kriterien (SPEC §1.1, §3.1, §8.1):
- Ebene E0 (Autarke Bausteine): Kern-Closure leer.
- Ebene E1a/E1b: Kern-Closure beschränkt auf E0 / E1a gemäß Tabelle §3.1 und allowed_internal.
- Ebene E2 (Integration): Verwendet E0, E1, E2, nie E3.
- Ebene E3 (Anwendungen/Testwerkzeuge).
- INV-8: Keine Aufwärtsabhängigkeiten (E3 -> E2 -> E1b -> E1a -> E0, nie umgekehrt).
- Jedes Crate darf als interne Abhängigkeiten nur deklarierte `allowed_internal` bzw. `feature_closure_allowed` einbinden.
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

    cmd = ["cargo", "metadata", "--format-version", "1", "--no-deps"]
    res = subprocess.run(cmd, cwd=root, capture_output=True, text=True)
    if res.returncode != 0:
        return [f"cargo metadata fehlgeschlagen: {res.stderr}"], [], {}

    meta = json.loads(res.stdout)
    packages = {p["name"]: p for p in meta["packages"]}
    workspace_members = set(meta["workspace_members"])

    violations: List[str] = []
    warnings: List[str] = []

    report_details: Dict[str, Any] = {
        "crates_checked": 0,
        "level_counts": {},
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
        level = spec.get("level")
        if level is not None:
            report_details["level_counts"][f"E{level}"] = report_details["level_counts"].get(f"E{level}", 0) + 1

        allowed_internal = set(spec.get("allowed_internal", []))
        dev_allowed = spec.get("dev_allowed", {})
        dev_allowed_crates = set()
        for k, v in dev_allowed.items():
            if isinstance(v, list):
                dev_allowed_crates.update(v)

        # Features closures
        feature_closures = spec.get("feature_closure_allowed", {})
        all_feature_allowed = set()
        for f_name, f_deps in feature_closures.items():
            all_feature_allowed.update(f_deps)

        for dep in pkg.get("dependencies", []):
            dep_name = dep["name"]
            # Nur interne Workspace-Abhängigkeiten prüfen
            if dep_name in packages and packages[dep_name]["id"] in workspace_members:
                kind = dep.get("kind") or "normal"
                is_optional = dep.get("optional", False)
                dep_spec = crates_spec.get(dep_name, {})
                dep_level = dep_spec.get("level")

                # Prüfe INV-8: Keine Aufwärtsabhängigkeit für normale Abhängigkeiten
                if kind == "normal" and level is not None and dep_level is not None:
                    if dep_level > level:
                        violations.append(
                            f"[G1-VIOLATION] Ebenen-Verstoß (INV-8): `{pkg_name}` (E{level}) hängt ab von `{dep_name}` (E{dep_level}) ({kind})."
                        )

                # Prüfe allowed_internal
                if kind == "normal":
                    if not is_optional:
                        if dep_name not in allowed_internal:
                            violations.append(
                                f"[G1-VIOLATION] Crate `{pkg_name}` (E{level}) hat unzulässige Kern-Abhängigkeit auf `{dep_name}` (nicht in allowed_internal)."
                            )
                    else:
                        # Optionale Abhängigkeit muss in allowed_internal oder in feature_closure_allowed sein
                        if dep_name not in allowed_internal and dep_name not in all_feature_allowed:
                            violations.append(
                                f"[G1-VIOLATION] Crate `{pkg_name}` (E{level}) hat optionale Abhängigkeit auf `{dep_name}`, die in keinem feature_closure_allowed deklariert ist."
                            )
                elif kind == "dev":
                    # Wenn dev_allowed spezifiziert ist
                    if dev_allowed_crates and dep_name not in dev_allowed_crates and dep_name not in allowed_internal:
                        violations.append(
                            f"[G1-VIOLATION] Crate `{pkg_name}` hat unzulässige dev-Abhängigkeit auf `{dep_name}` (nicht in dev_allowed)."
                        )

    report_details["violations"] = violations
    report_details["warnings"] = warnings
    return violations, warnings, report_details


def main() -> int:
    parser = argparse.ArgumentParser(description="Gate G1 Dependencies & Layering Validator (SPEC §8.1 / G1)")
    parser.add_argument("--capabilities", default="capabilities.toml", help="Pfad zu capabilities.toml")
    parser.add_argument("--root", default=".", help="Workspace-Root")
    parser.add_argument("--json", action="store_true", help="JSON-Format für CI/Agent-Pipelines ausgeben")
    parser.add_argument("--strict", action="store_true", help="Failt auch bei Warnungen")
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

    print("=== GATE G1: Crate-Ebenen & Layering-Report (SPEC §8.1 / capabilities.toml v3) ===")
    if warnings:
        print(f"\n⚠️  {len(warnings)} Warnungen:")
        for w in warnings:
            print(f"  - {w}")

    if violations:
        print(f"\n❌ {len(violations)} Verstöße gegen Gate G1 identifiziert:")
        for v in violations:
            print(f"  - {v}")
        return 1

    print("\n✅ Gate G1 bestanden: Alle internen Abhängigkeiten entsprechen capabilities.toml und SPEC.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
