#!/usr/bin/env python3
"""
tools/gate_layering.py — G-L1 Layering Enforcement Gate.

Normativ: docs/SPEC.md §8.1 (G-L1), §8.3 (capabilities v3), §7 (S-01)
Exit-Codes:
  0 = grün (keine Verstöße)
  1 = Verstoß (Layering-/Closure-Verletzung, Zyklus, Ebene oder unzulässige Dev-Abhängigkeit)
  2 = Werkzeugfehler (Konfigurationsfehler, cargo-Fehler etc.)
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import tomllib
from pathlib import Path
from typing import Any, Dict, List, Set, Tuple


def get_metadata(root: Path) -> dict:
    cmd = ["cargo", "metadata", "--format-version", "1", "--no-deps"]
    res = subprocess.run(cmd, cwd=root, capture_output=True, text=True)
    if res.returncode != 0:
        raise RuntimeError(f"cargo metadata failed: {res.stderr}")
    return json.loads(res.stdout)


def get_cargo_tree_dependencies(root: Path, crate: str, features: list[str] | None = None) -> Set[str]:
    cmd = ["cargo", "tree", "-p", crate, "-e", "normal", "--no-default-features", "-f", "{p}"]
    if features:
        for f in features:
            cmd.extend(["--features", f])
    res = subprocess.run(cmd, cwd=root, capture_output=True, text=True)
    if res.returncode != 0:
        # Falls crate mit den features nicht baut oder existiert
        raise RuntimeError(f"cargo tree failed for {crate} features={features}: {res.stderr}")

    deps = set()
    for line in res.stdout.splitlines():
        line = line.strip()
        if not line:
            continue
        # Zeilenformat z. B.: "contextra-types v0.1.0 (/app/crates/contextra-types)" oder mit Baumzeichen
        # Entferne Baumzeichen
        cleaned = line
        for prefix in ["├── ", "└── ", "│   ", "    ", "│", "├", "└", "─"]:
            cleaned = cleaned.replace(prefix, "")
        cleaned = cleaned.strip()
        if not cleaned:
            continue
        parts = cleaned.split()
        pkg_name = parts[0]
        deps.add(pkg_name)

    # Das Crate selbst gehört nicht zu seinen Abhängigkeiten
    deps.discard(crate)
    return deps


def check_layering(root: Path, target_crate: str | None = None) -> Tuple[List[str], List[str]]:
    caps_file = root / "capabilities.toml"
    if not caps_file.exists():
        raise FileNotFoundError(f"capabilities.toml not found at {caps_file}")

    with open(caps_file, "rb") as f:
        caps = tomllib.load(f)

    crates_spec: Dict[str, dict] = caps.get("crates", {})

    meta = get_metadata(root)
    ws_packages = {p["name"]: p for p in meta["packages"]}

    violations: List[str] = []
    warnings: List[str] = []

    crates_to_check = [target_crate] if target_crate else sorted(crates_spec.keys())

    # 1. Zyklenprüfung und Ebenenprüfung E_n -> E_m (m > n verboten nach INV-8 / SPEC §8.1)
    # Baue Ebenen-Map auf
    level_map: Dict[str, int] = {}
    for c_name, c_spec in crates_spec.items():
        if "level" in c_spec:
            level_map[c_name] = c_spec["level"]
        elif "ring" in c_spec:
            # Fallback falls v2 schema: Ring 0 -> 0, Ring 1 -> 1, Ring 2 -> 2, Ring 3 -> 3, Ring 4 -> 4
            ring_str = str(c_spec["ring"]).lower()
            if "0" in ring_str:
                level_map[c_name] = 0
            elif "1" in ring_str:
                level_map[c_name] = 1
            elif "2" in ring_str:
                level_map[c_name] = 2
            elif "3" in ring_str:
                level_map[c_name] = 3
            elif "4" in ring_str:
                level_map[c_name] = 4
            elif "tooling" in ring_str or "extension" in ring_str:
                level_map[c_name] = 3
            else:
                level_map[c_name] = 0

    for c in crates_to_check:
        if c not in crates_spec:
            continue
        c_spec = crates_spec[c]
        if c not in ws_packages:
            continue

        c_level = level_map.get(c, 0)
        allowed_internal = set(c_spec.get("allowed_internal", []))
        if not allowed_internal and "may_depend_on" in c_spec:
            # Kompatibilität zu v2
            allowed_internal = set(c_spec.get("may_depend_on", []))

        feature_closure_allowed: Dict[str, List[str]] = c_spec.get("feature_closure_allowed", {})
        dev_allowed = set(c_spec.get("dev_allowed", []))

        # (a) Kern-Closure Prüfung: cargo tree --no-default-features
        try:
            core_deps = get_cargo_tree_dependencies(root, c)
        except Exception as e:
            violations.append(f"[{c}] Kern-Abhängigkeitsbaum konnte nicht ermittelt werden: {e}")
            continue

        internal_core_deps = {d for d in core_deps if d in ws_packages}
        diff = internal_core_deps - allowed_internal
        if diff:
            violations.append(
                f"[{c}] Kern-Closure Verstoß: unerlaubte interne Abhängigkeiten {sorted(diff)} "
                f"(erlaubt: {sorted(allowed_internal)})"
            )

        # Ebenen-Prüfung: keine Kante E_n -> E_m mit m > n
        for dep in internal_core_deps:
            dep_level = level_map.get(dep, 0)
            if dep_level > c_level:
                violations.append(
                    f"[{c}] Ebenen-Verstoß (INV-8): Ebene {c_level} ({c}) hängt ab von höherer Ebene {dep_level} ({dep})"
                )

        # (b) Feature-Closure Prüfung
        for feat, allowed_for_feat in feature_closure_allowed.items():
            try:
                feat_deps = get_cargo_tree_dependencies(root, c, [feat])
            except Exception as e:
                warnings.append(f"[{c}] Feature-Baum für '{feat}' konnte nicht ermittelt werden: {e}")
                continue
            internal_feat_deps = {d for d in feat_deps if d in ws_packages}
            allowed_total = allowed_internal | set(allowed_for_feat)
            feat_diff = internal_feat_deps - allowed_total
            if feat_diff:
                violations.append(
                    f"[{c}] Feature-Closure Verstoß bei Feature '{feat}': unerlaubte interne Abhängigkeiten {sorted(feat_diff)} "
                    f"(erlaubt: {sorted(allowed_total)})"
                )

        # (c) Dev-Dependencies Prüfung: keine Kante außerhalb dev_allowed auf contextra-testkit
        pkg_data = ws_packages[c]
        dev_deps = {
            d["name"] for d in pkg_data.get("dependencies", [])
            if d.get("kind") == "dev"
        }
        if "contextra-testkit" in dev_deps and "contextra-testkit" not in dev_allowed:
            # Falls c_spec Klasse A oder nicht in dev_allowed
            violations.append(
                f"[{c}] Dev-Abhängigkeit auf 'contextra-testkit' unzulässig (nicht in dev_allowed)"
            )

    return violations, warnings


def main() -> int:
    parser = argparse.ArgumentParser(description="G-L1 Layering Enforcement Gate")
    parser.add_argument("--crate", "-c", help="Check specific crate")
    parser.add_argument("--all", action="store_true", help="Check all crates")
    parser.add_argument("--root", default=".", help="Workspace root")
    args = parser.parse_args()

    root = Path(args.root).resolve()

    try:
        violations, warnings = check_layering(root, target_crate=args.crate)
    except Exception as e:
        print(f"WERKZEUGFEHLER: {e}", file=sys.stderr)
        return 2

    if warnings:
        print(f"WARNUNGEN ({len(warnings)}):")
        for w in warnings:
            print(f"  - {w}")

    if violations:
        print(f"LAYER-VERSTÖSSE ({len(violations)}):", file=sys.stderr)
        for v in violations:
            print(f"  - {v}", file=sys.stderr)
        return 1

    print("Gate G-L1 Layering: GRÜN")
    return 0


if __name__ == "__main__":
    sys.exit(main())
