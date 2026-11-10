#!/usr/bin/env python3
"""
tools/feature_audit.py — Prüfung ungenutzter Features im Workspace.

Normativ: docs/SPEC.md §7 (S-12), §8.3 / B14
Meldet Features ohne `cfg(feature = "x")` im Quelltext UND ohne Weiterleitung
in einem anderen Manifest.
Option: --fail-on-unused (Exit 1 bei Funden)
Liest Cargo.toml aller Workspace-Mitglieder.
"""

from __future__ import annotations

import argparse
import os
import re
import sys
import tomllib
from pathlib import Path
from typing import Dict, List, Set


def get_all_workspace_manifests(root: Path) -> Dict[str, Path]:
    root_toml = root / "Cargo.toml"
    with open(root_toml, "rb") as f:
        data = tomllib.load(f)
    members = data.get("workspace", {}).get("members", [])
    manifests = {}
    for m in members:
        p = root / m / "Cargo.toml"
        if p.exists():
            with open(p, "rb") as mf:
                mdata = tomllib.load(mf)
            pkg_name = mdata.get("package", {}).get("name")
            if pkg_name:
                manifests[pkg_name] = p
    return manifests


def scan_source_for_features(crate_dir: Path) -> Set[str]:
    used_features = set()
    pattern = re.compile(r'feature\s*=\s*"([^"]+)"')
    for root, dirs, files in os.walk(crate_dir):
        # Exclude target/ or build artifacts if any
        if "target" in Path(root).parts:
            continue
        for f in files:
            if f.endswith(".rs"):
                fpath = Path(root) / f
                try:
                    with open(fpath, "r", encoding="utf-8", errors="ignore") as fp:
                        content = fp.read()
                        matches = pattern.findall(content)
                        used_features.update(matches)
                except Exception:
                    pass
    return used_features


def collect_feature_forwardings(manifests: Dict[str, Path]) -> Dict[str, Set[str]]:
    # crate_name -> set of features forwarded to it by ANY crate
    forwardings: Dict[str, Set[str]] = {c: set() for c in manifests}
    # Pattern: "crate_name/feature_name" or dependency feature activations
    dep_feat_pattern = re.compile(r'^([a-zA-Z0-9_\-]+)/([a-zA-Z0-9_\-]+)$')

    for cname, mpath in manifests.items():
        with open(mpath, "rb") as f:
            data = tomllib.load(f)
        features = data.get("features", {})
        for feat_name, feat_list in features.items():
            for item in feat_list:
                m = dep_feat_pattern.match(item)
                if m:
                    target_crate, target_feat = m.groups()
                    if target_crate in forwardings:
                        forwardings[target_crate].add(target_feat)
                    elif f"contextra-{target_crate}" in forwardings:
                        forwardings[f"contextra-{target_crate}"].add(target_feat)

        # Also check dependencies: dep = { version = "...", features = [...] }
        for sec in ["dependencies", "dev-dependencies", "build-dependencies"]:
            deps = data.get(sec, {})
            for dname, dval in deps.items():
                if isinstance(dval, dict) and "features" in dval:
                    actual_name = dval.get("package", dname)
                    if actual_name in forwardings:
                        forwardings[actual_name].update(dval["features"])
    return forwardings


def main() -> int:
    parser = argparse.ArgumentParser(description="Feature Audit Tool")
    parser.add_argument("--fail-on-unused", action="store_true", help="Fail if unused features exist")
    parser.add_argument("--root", default=".", help="Workspace root")
    args = parser.parse_args()

    root = Path(args.root).resolve()
    manifests = get_all_workspace_manifests(root)
    forwardings = collect_feature_forwardings(manifests)

    unused_by_crate: Dict[str, List[str]] = {}

    for cname, mpath in sorted(manifests.items()):
        with open(mpath, "rb") as f:
            data = tomllib.load(f)
        declared_features = set(data.get("features", {}).keys())
        if not declared_features:
            continue

        src_features = scan_source_for_features(mpath.parent)
        fwd_features = forwardings.get(cname, set())

        # An unused feature has no cfg(feature = "x") in its crate AND is not forwarded from another manifest
        unused = declared_features - src_features - fwd_features
        # default and docid-128 are convention features, but let's see if unused
        if unused:
            unused_by_crate[cname] = sorted(unused)

    total_unused = sum(len(v) for v in unused_by_crate.values())

    if unused_by_crate:
        print(f"Befund Feature-Audit: {total_unused} ungenutzte Features in {len(unused_by_crate)} Crates gefunden:")
        for cname, feats in unused_by_crate.items():
            print(f"  [{cname}]: {', '.join(feats)}")
    else:
        print("Feature-Audit: GRÜN (keine ungenutzten Features gefunden)")

    if args.fail_on_unused and total_unused > 0:
        return 1

    return 0


if __name__ == "__main__":
    sys.exit(main())
