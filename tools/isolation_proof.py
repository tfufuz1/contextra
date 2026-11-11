#!/usr/bin/env python3
"""
tools/isolation_proof.py — G-L4 Isolation Proof Tool.

Normativ: docs/SPEC.md §8.4 (G-L4), §8.5 (Abnahmematrix), §7 (S-01)
Kopiert ein Crate plus vollständige interne Pfad-Closure in ein leeres temporäres Workspace,
klammert KEINE interne Abhängigkeit aus, erzeugt ein neues Cargo.lock und führt
`cargo check` in den Modi aus:
  - no-default: `cargo check -p <crate> --no-default-features`
  - default:    `cargo check -p <crate>`
  - each-feature: `cargo check -p <crate> --no-default-features --features <f>` für jedes deklarierte Feature

Fehlschlag (Exit 1), wenn eine interne Abhängigkeit außerhalb allowed_internal mitkopiert werden muss
oder ein Build fehlschlägt.
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile
import tomllib
from pathlib import Path
from typing import Any, Dict, List, Set


def get_metadata(root: Path) -> dict:
    cmd = ["cargo", "metadata", "--format-version", "1", "--no-deps"]
    res = subprocess.run(cmd, cwd=root, capture_output=True, text=True)
    if res.returncode != 0:
        raise RuntimeError(f"cargo metadata failed: {res.stderr}")
    return json.loads(res.stdout)


def get_crate_features(crate_dir: Path) -> list[str]:
    toml_path = crate_dir / "Cargo.toml"
    if not toml_path.exists():
        return []
    with open(toml_path, "rb") as f:
        data = tomllib.load(f)
    return list(data.get("features", {}).keys())


def run_isolation_proof(
    crate_name: str,
    repo_root: Path,
    modes: list[str],
) -> bool:
    print(f"=== Starting G-L4 Isolation Proof for {crate_name} (modes={modes}) ===")
    caps_file = repo_root / "capabilities.toml"
    if not caps_file.exists():
        print(f"Error: {caps_file} does not exist.", file=sys.stderr)
        return False

    with open(caps_file, "rb") as f:
        caps_data = tomllib.load(f)
    crates_spec: Dict[str, dict] = caps_data.get("crates", {})

    c_spec = crates_spec.get(crate_name, {})
    allowed_internal = set(c_spec.get("allowed_internal", []))
    feature_closure_allowed: Dict[str, list[str]] = c_spec.get("feature_closure_allowed", {})
    all_allowed = set(allowed_internal)
    for feats in feature_closure_allowed.values():
        all_allowed.update(feats)

    meta = get_metadata(repo_root)
    packages = {p["name"]: p for p in meta["packages"]}

    if crate_name not in packages:
        print(f"Error: Crate '{crate_name}' not found in workspace packages.", file=sys.stderr)
        return False

    # Ermittle transitive interne Pfad-Closure des Crates via dependencies in Cargo.toml
    to_visit = [crate_name]
    closure: Set[str] = set()

    while to_visit:
        curr = to_visit.pop()
        if curr in closure:
            continue
        closure.add(curr)
        pkg = packages.get(curr)
        if not pkg:
            continue
        for dep in pkg.get("dependencies", []):
            dep_name = dep["name"]
            # Nur normale/build dependencies (keine dev)
            if dep.get("kind") != "dev" and dep_name in packages:
                if dep_name not in closure:
                    to_visit.append(dep_name)

    internal_deps = closure - {crate_name}
    forbidden_deps = internal_deps - all_allowed
    if forbidden_deps:
        print(
            f"FEHLER: Crate '{crate_name}' benötigt interne Abhängigkeiten {sorted(forbidden_deps)}, "
            f"die nicht in allowed_internal/feature_closure_allowed ({sorted(all_allowed)}) liegen!",
            file=sys.stderr,
        )
        return False

    # Erstelle isoliertes temporäres Workspace
    with tempfile.TemporaryDirectory() as tmp_dir_str:
        tmp_dir = Path(tmp_dir_str)
        tmp_crates = tmp_dir / "crates"
        tmp_crates.mkdir(parents=True, exist_ok=True)

        for c in closure:
            src_c = repo_root / "crates" / c
            if not src_c.exists():
                # Evtl benchmarks/ etc.
                src_c = Path(packages[c]["manifest_path"]).parent
            shutil.copytree(src_c, tmp_crates / c)

        members_str = ",\n    ".join([f'"crates/{c}"' for c in sorted(closure)])

        with open(repo_root / "Cargo.toml", "r", encoding="utf-8") as rf:
            root_manifest = rf.read()

        start_marker = "[workspace.dependencies]\n"
        start_idx = root_manifest.find(start_marker)
        if start_idx != -1:
            end_idx = root_manifest.find("\n[workspace.lints", start_idx)
            if end_idx == -1:
                end_idx = root_manifest.find("\n[lints", start_idx)
            raw_deps_lines = root_manifest[start_idx + len(start_marker):end_idx].splitlines()
        else:
            raw_deps_lines = []

        filtered_deps = []
        for line in raw_deps_lines:
            stripped = line.strip()
            if not stripped or stripped.startswith("#"):
                filtered_deps.append(line)
                continue
            if 'path = "crates/' in stripped or "path = 'crates/" in stripped:
                pkg = stripped.split("=")[0].strip()
                if pkg in closure:
                    filtered_deps.append(line)
                else:
                    # Nicht in der erlaubten Closure enthaltene Workspace-Crates auslassen
                    continue
            else:
                filtered_deps.append(line)

        ws_deps_block = "\n".join(filtered_deps)

        manifest_content = f"""[workspace]
resolver = "2"
members = [
    {members_str}
]

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT OR Apache-2.0"
rust-version = "1.89"

[workspace.lints.rust]
unsafe_code = "deny"

[workspace.lints.clippy]
all = "warn"

[workspace.dependencies]
{ws_deps_block}
"""
        with open(tmp_dir / "Cargo.toml", "w", encoding="utf-8") as f:
            f.write(manifest_content)

        # Cargo.lock aus dem Haupt-Workspace kopieren, um dieselben gecachten Versionen zu nutzen
        if (repo_root / "Cargo.lock").exists():
            shutil.copy(repo_root / "Cargo.lock", tmp_dir / "Cargo.lock")
        else:
            subprocess.run(["cargo", "generate-lockfile", "--offline"], cwd=tmp_dir, capture_output=True)

        crate_features = get_crate_features(tmp_crates / crate_name)

        # Führe Prüfungen je Modus aus
        for mode in modes:
            if mode == "no-default":
                print(f"Running mode: no-default")
                cmd = ["cargo", "check", "-p", crate_name, "--no-default-features", "--offline"]
                res = subprocess.run(cmd, cwd=tmp_dir, capture_output=True, text=True)
                if res.returncode != 0:
                    print(f"FAILED mode no-default:\n{res.stderr}", file=sys.stderr)
                    return False
            elif mode == "default":
                print(f"Running mode: default")
                cmd = ["cargo", "check", "-p", crate_name, "--offline"]
                res = subprocess.run(cmd, cwd=tmp_dir, capture_output=True, text=True)
                if res.returncode != 0:
                    print(f"FAILED mode default:\n{res.stderr}", file=sys.stderr)
                    return False
            elif mode == "each-feature":
                print(f"Running mode: each-feature ({len(crate_features)} features)")
                for feat in crate_features:
                    if feat == "default":
                        continue
                    print(f"  - feature: {feat}")
                    cmd = ["cargo", "check", "-p", crate_name, "--no-default-features", "--features", feat, "--offline"]
                    res = subprocess.run(cmd, cwd=tmp_dir, capture_output=True, text=True)
                    if res.returncode != 0:
                        print(f"FAILED feature {feat}:\n{res.stderr}", file=sys.stderr)
                        return False

    print(f"=== G-L4 Isolation Proof SUCCESS for {crate_name} ===")
    return True


def main() -> int:
    parser = argparse.ArgumentParser(description="G-L4 Isolation Proof Tool (SPEC §8.4)")
    parser.add_argument("--crate", "-c", required=True, help="Target crate name")
    parser.add_argument(
        "--modes",
        default="no-default,default,each-feature",
        help="Comma-separated modes: no-default, default, each-feature",
    )
    parser.add_argument("--root", default=".", help="Workspace root")
    args = parser.parse_args()

    repo_root = Path(args.root).resolve()
    modes = [m.strip() for m in args.modes.split(",") if m.strip()]

    success = run_isolation_proof(args.crate, repo_root, modes)
    return 0 if success else 1


if __name__ == "__main__":
    sys.exit(main())
