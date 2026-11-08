#!/usr/bin/env python3
"""
tools/export_crate.py - Export standalone building block crate per SPEC v4.0 §7.
Löst Workspace-Vererbung auf und testet das Crate in einem isolierten Verzeichnis.
"""

import os
import sys
import shutil
import subprocess
import argparse
import tempfile

WORKSPACE_DEPS = {
    "thiserror": "2.0.20",
    "bytemuck": "1.25.2",
    "tracing": "0.1.44",
    "serde": "1.0.229",
    "serde_json": "1.0.151",
    "ahash": "0.8.12",
    "proptest": "1.4",
    "criterion": "0.5.1",
    "tokio": "1.53.1",
}

def export_crate(crate_name, out_dir=None):
    crate_path = os.path.join("crates", crate_name)
    if not os.path.isdir(crate_path):
        print(f"Error: Crate directory {crate_path} does not exist", file=sys.stderr)
        return False

    is_temp = False
    if not out_dir:
        out_dir = tempfile.mkdtemp(prefix=f"export_{crate_name}_")
        is_temp = True

    print(f"Exporting {crate_name} to {out_dir}...")

    # Copy files
    for item in os.listdir(crate_path):
        s = os.path.join(crate_path, item)
        d = os.path.join(out_dir, item)
        if os.path.isdir(s):
            shutil.copytree(s, d, dirs_exist_ok=True)
        else:
            shutil.copy2(s, d)

    # Read and resolve Cargo.toml
    cargo_toml_path = os.path.join(out_dir, "Cargo.toml")
    with open(cargo_toml_path) as f:
        lines = f.readlines()

    resolved_lines = []
    in_lints = False
    for line in lines:
        if line.strip() == "[lints]":
            in_lints = True
            continue
        if in_lints:
            if line.strip().startswith("["):
                in_lints = False
            else:
                continue

        line = line.replace("rust-version.workspace = true", 'rust-version = "1.89.0"')
        line = line.replace("version.workspace = true", 'version = "0.1.0"')
        line = line.replace("edition.workspace = true", 'edition = "2021"')
        line = line.replace("license.workspace = true", 'license = "MIT OR Apache-2.0"')

        for dep, ver in WORKSPACE_DEPS.items():
            line = line.replace(f'{dep}.workspace = true', f'{dep} = "{ver}"')
            line = line.replace(f'{dep} = {{ workspace = true }}', f'{dep} = "{ver}"')
            line = line.replace(f'{dep} = {{ workspace = true, optional = true }}', f'{dep} = {{ version = "{ver}", optional = true }}')

        # Drop workspace internal deps if present in standalone export
        if "contextra-types = { workspace = true" in line:
            continue
        if "contextra-ports = { workspace = true" in line:
            continue
        if "types-compat = [" in line:
            continue
        line = line.replace('"types-compat"', '').replace(', "types-compat"', '').replace('"types-compat", ', '')

        resolved_lines.append(line)

    with open(cargo_toml_path, "w") as f:
        f.writelines(resolved_lines)

    print("Resolved Cargo.toml in exported directory.")

    # Run cargo check in export dir
    print(f"Running cargo check in {out_dir}...")
    res = subprocess.run(["cargo", "check", "--no-default-features"], cwd=out_dir)
    if res.returncode != 0:
        print("Check failed in export dir", file=sys.stderr)
        return False

    print(f"Successfully exported and verified {crate_name} at {out_dir}")
    if is_temp:
        shutil.rmtree(out_dir)
    return True

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("crate", help="Name of crate to export")
    parser.add_argument("--out", help="Output directory")
    args = parser.parse_args()
    success = export_crate(args.crate, args.out)
    sys.exit(0 if success else 1)
