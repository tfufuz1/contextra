#!/usr/bin/env python3
"""M2 Isolation Proof Runner.

Copies the target crate and its allowed dependencies (per capabilities.toml)
into a clean temporary workspace directory and runs:
  - cargo build -p <crate> --no-default-features --locked
  - cargo build -p <crate> --locked
  - cargo test -p <crate> --locked
"""

import argparse
import os
import shutil
import subprocess
import sys
import tomllib


def run_isolation_proof(crate_name: str) -> bool:
    repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    capabilities_path = os.path.join(repo_root, "capabilities.toml")

    with open(capabilities_path, "rb") as f:
        capabilities = tomllib.load(f)

    crate_info = capabilities.get("crates", {}).get(crate_name)
    if not crate_info:
        print(f"Error: Crate '{crate_name}' not found in capabilities.toml", file=sys.stderr)
        return False

    allowed_deps = crate_info.get("may_depend_on", [])
    crate_path = crate_info.get("path", f"crates/{crate_name}")

    tmp_ws = f"/tmp/contextra_isolation_{crate_name}"
    if os.path.exists(tmp_ws):
        shutil.rmtree(tmp_ws)
    os.makedirs(os.path.join(tmp_ws, "crates"), exist_ok=True)

    # Copy target crate
    dest_target = os.path.join(tmp_ws, crate_path)
    os.makedirs(os.path.dirname(dest_target), exist_ok=True)
    shutil.copytree(os.path.join(repo_root, crate_path), dest_target)

    # Copy allowed dependencies
    members = [crate_path]
    for dep in allowed_deps:
        dep_info = capabilities.get("crates", {}).get(dep)
        dep_path = dep_info.get("path", f"crates/{dep}") if dep_info else f"crates/{dep}"
        dest_dep = os.path.join(tmp_ws, dep_path)
        os.makedirs(os.path.dirname(dest_dep), exist_ok=True)
        shutil.copytree(os.path.join(repo_root, dep_path), dest_dep)
        members.append(dep_path)

    # Adapt root Cargo.toml
    with open(os.path.join(repo_root, "Cargo.toml")) as f:
        orig_cargo = f.read()

    lines = orig_cargo.splitlines()
    new_lines = []
    in_members = False
    for line in lines:
        if line.strip() == "members = [":
            in_members = True
            new_lines.append("members = [")
            for m in members:
                new_lines.append(f'    "{m}",')
            continue
        if in_members:
            if line.strip() == "]":
                in_members = False
                new_lines.append("]")
            continue
        if line.strip() == "default-members = [":
            in_members = True
            new_lines.append("default-members = [")
            new_lines.append(f'    "{crate_path}",')
            continue
        new_lines.append(line)

    with open(os.path.join(tmp_ws, "Cargo.toml"), "w") as f:
        f.write("\n".join(new_lines))

    # Generate offline lockfile
    subprocess.run(["cargo", "generate-lockfile", "--offline"], cwd=tmp_ws, check=True)

    print(f"[{crate_name}] Running: cargo build -p {crate_name} --no-default-features --locked")
    res1 = subprocess.run(
        ["cargo", "build", "-p", crate_name, "--no-default-features", "--locked"],
        cwd=tmp_ws,
        capture_output=True,
        text=True,
    )
    if res1.returncode != 0:
        print(f"FAILED build --no-default-features:\n{res1.stderr}", file=sys.stderr)
        return False

    print(f"[{crate_name}] Running: cargo build -p {crate_name} --locked")
    res2 = subprocess.run(
        ["cargo", "build", "-p", crate_name, "--locked"],
        cwd=tmp_ws,
        capture_output=True,
        text=True,
    )
    if res2.returncode != 0:
        print(f"FAILED build:\n{res2.stderr}", file=sys.stderr)
        return False

    print(f"[{crate_name}] Running: cargo test -p {crate_name} --locked")
    res3 = subprocess.run(
        ["cargo", "test", "-p", crate_name, "--locked"],
        cwd=tmp_ws,
        capture_output=True,
        text=True,
    )
    if res3.returncode != 0:
        print(f"FAILED test:\n{res3.stderr}", file=sys.stderr)
        return False

    print(f"[{crate_name}] M2 Isolation Proof PASSED.")
    return True


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Run M2 isolation proof for a crate.")
    parser.add_argument("crate", help="Name of the crate (e.g. contextra-rank)")
    args = parser.parse_args()

    success = run_isolation_proof(args.crate)
    sys.exit(0 if success else 1)
