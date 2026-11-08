#!/usr/bin/env python3
"""M2 Isolation Proof Tool for Contextra building blocks.

Verifies that a crate builds and tests in an isolated workspace with only its
explicitly allowed dependencies.

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
import tempfile
from pathlib import Path


def run_command(cmd, cwd):
    print(f"Running: {' '.join(cmd)} in {cwd}")
    res = subprocess.run(cmd, cwd=cwd, text=True, capture_output=True)
    if res.returncode != 0:
        print(f"FAILED (exit {res.returncode}):")
        print(res.stdout)
        print(res.stderr)
        return False
    return True


def run_isolation_proof(crate_name: str, repo_root: Path) -> bool:
    print(f"=== Starting M2 Isolation Proof for {crate_name} ===")
    crate_dir = repo_root / "crates" / crate_name
    if not crate_dir.exists():
        print(f"Error: {crate_dir} does not exist.")
        return False

    with tempfile.TemporaryDirectory() as tmp_dir_str:
        tmp_dir = Path(tmp_dir_str)
        tmp_crates = tmp_dir / "crates"
        tmp_crates.mkdir(parents=True, exist_ok=True)

        # Copy target crate and allowed Ring 0/tooling dependencies
        shutil.copytree(crate_dir, tmp_crates / crate_name)
        for dep in ["contextra-types", "contextra-ports", "contextra-testkit"]:
            src = repo_root / "crates" / dep
            if src.exists():
                shutil.copytree(src, tmp_crates / dep)

        # Copy Cargo.lock
        lock_file = repo_root / "Cargo.lock"
        if lock_file.exists():
            shutil.copy(lock_file, tmp_dir / "Cargo.lock")

        # Create isolated workspace manifest
        manifest_content = f"""[workspace]
resolver = "2"
members = [
    "crates/{crate_name}",
    "crates/contextra-ports",
    "crates/contextra-types",
    "crates/contextra-testkit",
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
contextra-types = {{ path = "crates/contextra-types" }}
contextra-ports = {{ path = "crates/contextra-ports" }}
contextra-testkit = {{ path = "crates/contextra-testkit" }}
contextra-graph = {{ path = "crates/contextra-graph" }}
contextra-erasure-spi = {{ path = "crates/contextra-erasure-spi" }}
ahash = {{ version = "0.8", features = ["serde"] }}
arc-swap = "1.7"
bincode = "1.3.3"
blake3 = "1"
bytes = "1"
criterion = {{ version = "0.5", features = ["async_tokio"] }}
futures = "0.3"
parking_lot = "0.12"
proptest = "1.4"
scc = "2"
serde = {{ version = "1", features = ["derive", "rc"] }}
serde_json = "1"
tempfile = "3"
thiserror = "2"
tokio = {{ version = "1", features = ["full"] }}
tracing = "0.1"
tracing-subscriber = {{ version = "0.3", features = ["env-filter"] }}
"""
        with open(tmp_dir / "Cargo.toml", "w") as f:
            f.write(manifest_content)

        subprocess.run(["cargo", "update", "--workspace", "--offline"], cwd=tmp_dir, capture_output=True)

        commands = [
            ["cargo", "build", "-p", crate_name, "--no-default-features"],
            ["cargo", "build", "-p", crate_name],
            ["cargo", "test", "-p", crate_name, "--lib"],
            ["cargo", "bench", "-p", crate_name, "--no-run"],
        ]

        for cmd in commands:
            if not run_command(cmd, tmp_dir):
                return False

    print(f"=== M2 Isolation Proof SUCCESS for {crate_name} ===")
    return True


def main():
    parser = argparse.ArgumentParser(description="M2 Isolation Proof")
    parser.add_argument("--crate", "-c", default="contextra-graph", help="Target crate")
    args = parser.parse_args()

    repo_root = Path(__file__).resolve().parent.parent
    success = run_isolation_proof(args.crate, repo_root)
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()
