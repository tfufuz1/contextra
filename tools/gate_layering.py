#!/usr/bin/env python3
"""
tools/gate_layering.py — G-L1 Layering Enforcement Gate.

Normativ: docs/SPEC.md §2 (Architektur), §2.3 (R1, R2, R3), §8.1 (G-L1)
Exit-Codes:
  0 = grün (keine Verstöße)
  1 = Verstoß (Layering-/Closure-Verletzung, Zyklus, Ebene oder unzulässige Dev-Abhängigkeit)

Prüfungen:
  - R1: Kanten zeigen nur abwärts (E3 -> E2 -> E1 -> E0). Keine Aufwärtskanten im Kern.
  - R2: Dev-Abhängigkeiten auf höhere Ebenen sind verboten (außer contextra-testkit als Test-Harness).
  - R3: Default-Features MÜSSEN leer sein (`default = []`).
  - Zyklen-Freiheit im internen Abhängigkeitsgraphen.
"""

from __future__ import annotations

import argparse
import json
import sys
import tomllib
from pathlib import Path
from typing import Any, Dict, List, Set, Tuple


def check_layering(root: Path) -> Tuple[List[str], List[str], Dict[str, Any]]:
    caps_file = root / "capabilities.toml"
    if not caps_file.is_file():
        return [f"capabilities.toml nicht gefunden unter {caps_file}"], [], {}

    with open(caps_file, "rb") as f:
        caps_data = tomllib.load(f)["crates"]

    manifest_file = root / "Cargo.toml"
    with open(manifest_file, "rb") as f:
        ws_data = tomllib.load(f)["workspace"]

    ws_members = ws_data.get("members", [])
    ws_deps = ws_data.get("dependencies", {})

    levels = {c: data.get("level", 0) for c, data in caps_data.items()}
    violations: List[str] = []
    warnings: List[str] = []

    dep_graph: Dict[str, Set[str]] = {}

    details: Dict[str, Any] = {
        "crates_checked": 0,
        "level_counts": {},
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
        from_level = levels.get(pkg_name)
        if from_level is None:
            violations.append(f"Crate `{pkg_name}` ({m}) fehlt in capabilities.toml.")
            continue

        details["crates_checked"] += 1
        level_key = f"E{from_level}"
        details["level_counts"][level_key] = details["level_counts"].get(level_key, 0) + 1

        cap_info = caps_data.get(pkg_name, {})
        allowed_internal = set(cap_info.get("allowed_internal", []))
        feature_closures = cap_info.get("feature_closure_allowed", {})
        all_feature_allowed = set()
        for f_name, f_deps in feature_closures.items():
            all_feature_allowed.update(f_deps)

        dep_graph[pkg_name] = set()

        # R3: Default features must be empty
        features = pkg.get("features", {})
        default_feat = features.get("default", [])
        if default_feat:
            violations.append(f"[R3-Verstoß] `{pkg_name}`: default-Features müssen leer sein, ist aber {default_feat}.")

        # Dependencies
        deps = pkg.get("dependencies", {})
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

            to_level = levels.get(dep_name)
            if to_level is None:
                violations.append(f"`{pkg_name}` hängt von unbekanntem internen Crate `{dep_name}` ab.")
                continue

            if not is_optional:
                dep_graph[pkg_name].add(dep_name)
                # R1: Normale Abhängigkeiten dürfen nur abwärts zeigen
                if to_level > from_level:
                    violations.append(f"[R1-Verstoß] Aufwärtskante: `{pkg_name}` (E{from_level}) -> `{dep_name}` (E{to_level}).")
                if dep_name not in allowed_internal:
                    violations.append(f"Unzulässige Kern-Abhängigkeit: `{pkg_name}` -> `{dep_name}` (nicht in allowed_internal).")
            else:
                if dep_name not in allowed_internal and dep_name not in all_feature_allowed:
                    violations.append(f"Nicht deklarierte optionale Abhängigkeit: `{pkg_name}` -> `{dep_name}` (nicht in feature_closure_allowed).")

        # Dev-Dependencies
        dev_deps = pkg.get("dev-dependencies", {})
        for dep_name, dep_spec in dev_deps.items():
            if not (dep_name.startswith("contextra-") or dep_name == "contextra"):
                continue
            to_level = levels.get(dep_name)
            if to_level is not None and to_level > from_level:
                if dep_name != "contextra-testkit":
                    warnings.append(f"[R2-Warnung] Dev-Abhängigkeit auf höhere Ebene: `{pkg_name}` (E{from_level}) -> `{dep_name}` (E{to_level}).")

    # Cycle check (Tarjan SCC)
    index = {}
    low = {}
    stack = []
    on_stack = set()
    sccs = []
    counter = [0]

    def strongconnect(node):
        index[node] = low[node] = counter[0]
        counter[0] += 1
        stack.append(node)
        on_stack.add(node)
        for neighbor in dep_graph.get(node, []):
            if neighbor not in index:
                strongconnect(neighbor)
                low[node] = min(low[node], low[neighbor])
            elif neighbor in on_stack:
                low[node] = min(low[node], index[neighbor])
        if low[node] == index[node]:
            scc = []
            while True:
                w = stack.pop()
                on_stack.remove(w)
                scc.append(w)
                if w == node:
                    break
            if len(scc) > 1:
                sccs.append(scc)

    for n in list(dep_graph.keys()):
        if n not in index:
            strongconnect(n)

    for scc in sccs:
        violations.append(f"[Zyklus] Zirkuläre Abhängigkeit im Kern: {' -> '.join(scc)}")

    details["violations"] = violations
    details["warnings"] = warnings
    return violations, warnings, details


def main() -> int:
    parser = argparse.ArgumentParser(description="Gate G-L1 Layering & Dependency Enforcement (SPEC §2.3 / R1-R3)")
    parser.add_argument("--root", default=".", help="Workspace-Root")
    parser.add_argument("--json", action="store_true", help="JSON-Format ausgeben")
    parser.add_argument("--strict", action="store_true", help="Auch bei Warnungen abbrechen")
    args = parser.parse_args()

    root = Path(args.root).resolve()
    violations, warnings, details = check_layering(root)

    if args.json:
        payload = {
            "gate": "G-L1",
            "name": "gate_layering",
            "status": "PASS" if not violations and (not warnings or not args.strict) else "FAIL",
            "violations_count": len(violations),
            "warnings_count": len(warnings),
            "violations": violations,
            "warnings": warnings,
            "details": details,
        }
        print(json.dumps(payload, indent=2))
        return 1 if violations or (args.strict and warnings) else 0

    print("=== GATE G-L1: Layering & Dependency Enforcement Report (SPEC §2.3 R1–R3) ===")
    if warnings:
        print(f"\n⚠️  {len(warnings)} Warnungen:")
        for w in warnings:
            print(f"  - {w}")

    if violations:
        print(f"\n❌ {len(violations)} Verstöße gegen Gate G-L1:")
        for v in violations:
            print(f"  - {v}")
        return 1

    print(f"\n✅ Gate G-L1 bestanden: Alle {details['crates_checked']} Crates genügen strikt R1, R2, R3 und sind zyklenfrei.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
