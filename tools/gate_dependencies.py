#!/usr/bin/env python3
"""
tools/gate_dependencies.py — Gate G1 (Isolation & Dependency Enforcement per SPEC §2 / capabilities.toml Schema v3).

Normative Kriterien:
- Ebene E0 (Autarke Bausteine): Kern-Closure leer.
- Ebene E1 (Domänenbausteine): Kern-Closure beschränkt auf E0 (max. 5 Mitglieder).
- Ebene E2 (Integration): Verwendet E0, E1, E2, nie E3.
- Ebene E3 (Anwendungen/Testwerkzeuge): Darf alle Ebenen verwenden.
- Jedes Crate darf als interne Abhängigkeiten nur deklarierte `allowed_internal` bzw. `feature_closure_allowed` einbinden.
"""

from __future__ import annotations

import argparse
import json
import sys
import tomllib
from pathlib import Path
from typing import Any, Dict, List, Set, Tuple


def run_gate_dependencies(caps_path: Path, root: Path) -> Tuple[List[str], List[str], Dict[str, Any]]:
    if not caps_path.exists():
        return [f"capabilities.toml nicht gefunden unter {caps_path}"], [], {}

    with open(caps_path, "rb") as f:
        caps_data = tomllib.load(f).get("crates", {})

    manifest_file = root / "Cargo.toml"
    with open(manifest_file, "rb") as f:
        ws_data = tomllib.load(f)["workspace"]

    ws_members = ws_data.get("members", [])
    ws_deps = ws_data.get("dependencies", {})

    violations: List[str] = []
    warnings: List[str] = []

    report_details: Dict[str, Any] = {
        "crates_checked": 0,
        "level_counts": {},
        "closure_checks": {},
        "violations": [],
        "warnings": [],
    }

    for m in ws_members:
        manifest_path = root / m / "Cargo.toml"
        if not manifest_path.is_file():
            continue
        with open(manifest_path, "rb") as f:
            pkg = tomllib.load(f)

        pkg_name = pkg["package"]["name"]
        spec = caps_data.get(pkg_name)
        if not spec:
            violations.append(f"Crate `{pkg_name}` ist im Workspace, fehlt aber in `capabilities.toml`.")
            continue

        report_details["crates_checked"] += 1
        level = spec.get("level")
        if level is not None:
            report_details["level_counts"][f"E{level}"] = report_details["level_counts"].get(f"E{level}", 0) + 1

        allowed_internal = set(spec.get("allowed_internal", []))
        feature_closures = spec.get("feature_closure_allowed", {})
        all_feature_allowed = set()
        for f_name, f_deps in feature_closures.items():
            all_feature_allowed.update(f_deps)

        # Check dependencies
        deps = pkg.get("dependencies", {})
        core_internal_deps = set()

        for dep_name, dep_spec in deps.items():
            if not (dep_name.startswith("contextra-") or dep_name == "contextra"):
                continue

            is_optional = False
            if isinstance(dep_spec, dict):
                is_optional = dep_spec.get("optional", False)
                if dep_spec.get("workspace") is True:
                    ws_entry = ws_deps.get(dep_name, {})
                    if isinstance(ws_entry, dict) and ws_entry.get("optional"):
                        is_optional = True

            dep_spec_cap = caps_data.get(dep_name, {})
            dep_level = dep_spec_cap.get("level")

            if not is_optional:
                core_internal_deps.add(dep_name)
                # Check R1
                if level is not None and dep_level is not None and dep_level > level:
                    violations.append(f"[G1-VIOLATION] Ebenen-Verstoß (INV-8): `{pkg_name}` (E{level}) hängt ab von `{dep_name}` (E{dep_level}).")
                # Check allowed_internal
                if dep_name not in allowed_internal:
                    violations.append(f"[G1-VIOLATION] Crate `{pkg_name}` (E{level}) hat unzulässige Kern-Abhängigkeit auf `{dep_name}` (nicht in allowed_internal).")
            else:
                if dep_name not in allowed_internal and dep_name not in all_feature_allowed:
                    violations.append(f"[G1-VIOLATION] Crate `{pkg_name}` (E{level}) hat optionale Abhängigkeit auf `{dep_name}`, die in keinem feature_closure_allowed deklariert ist.")

        # Closure constraints (SPEC §2.1)
        if level == 0:
            # E0: Kern-Closure muss leer sein (außer ids/env untereinander erlaubt)
            e0_exceptions = {"contextra-ids", "contextra-env"}
            illegal_e0 = core_internal_deps - e0_exceptions
            if illegal_e0:
                violations.append(f"[E0-CLOSURE] E0-Crate `{pkg_name}` hat nicht-leere Kern-Closure: {sorted(illegal_e0)}.")
        elif level == 1:
            # E1: Kern-Closure <= 5 Mitglieder
            if len(core_internal_deps) > 5:
                violations.append(f"[E1-CLOSURE] E1-Crate `{pkg_name}` überschreitet Kern-Closure-Limit von 5 (hat {len(core_internal_deps)}: {sorted(core_internal_deps)}).")

        report_details["closure_checks"][pkg_name] = {
            "level": level,
            "core_deps": sorted(core_internal_deps),
            "core_deps_count": len(core_internal_deps),
        }

    report_details["violations"] = violations
    report_details["warnings"] = warnings
    return violations, warnings, report_details


def main() -> int:
    parser = argparse.ArgumentParser(description="Gate G1 Dependencies & Layering Validator (SPEC §2 / capabilities.toml v3)")
    parser.add_argument("--capabilities", default="capabilities.toml", help="Pfad zu capabilities.toml")
    parser.add_argument("--root", default=".", help="Workspace-Root")
    parser.add_argument("--json", action="store_true", help="JSON-Format ausgeben")
    parser.add_argument("--strict", action="store_true", help="Failt auch bei Warnungen")
    args = parser.parse_args()

    root = Path(args.root).resolve()
    caps = Path(args.capabilities).resolve()
    if not caps.is_file() and (root / args.capabilities).is_file():
        caps = root / args.capabilities

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
        return 1 if violations or (args.strict and warnings) else 0

    print("=== GATE G1: Crate-Ebenen & Closure-Report (SPEC §2 / capabilities.toml v3) ===")
    if warnings:
        print(f"\n⚠️  {len(warnings)} Warnungen:")
        for w in warnings:
            print(f"  - {w}")

    if violations:
        print(f"\n❌ {len(violations)} Verstöße gegen Gate G1 identifiziert:")
        for v in violations:
            print(f"  - {v}")
        return 1

    print(f"\n✅ Gate G1 bestanden: Alle {details['crates_checked']} Crates genügen strikt ihren Closure- und Ebenen-Regeln.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
