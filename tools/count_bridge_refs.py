#!/usr/bin/env python3
"""
tools/count_bridge_refs.py — Zählung der contextra_(types|ports|testkit)-Referenzen je Crate
außerhalb von compat/ und ohne Kommentarzeilen (//).

Normativ: docs/SPEC.md §7 (S-00), §8.2 (G-L2), JULES_RAHMEN.md (R6)
"""

from __future__ import annotations

import argparse
import os
import re
import sys
from pathlib import Path

PATTERN = re.compile(r'\bcontextra_(types|ports|testkit)\b')


def count_in_file(filepath: Path) -> int:
    count = 0
    with open(filepath, "r", encoding="utf-8", errors="ignore") as f:
        for line in f:
            stripped = line.strip()
            if stripped.startswith("//"):
                continue
            matches = PATTERN.findall(line)
            count += len(matches)
    return count


def count_crate_bridge_refs(crate_dir: Path) -> int:
    src_dir = crate_dir / "src"
    if not src_dir.exists():
        return 0
    total = 0
    for root, dirs, files in os.walk(src_dir):
        rel_root = Path(root).relative_to(src_dir)
        if "compat" in rel_root.parts:
            continue
        for file in files:
            if file.endswith(".rs"):
                total += count_in_file(Path(root) / file)
    return total


def main() -> int:
    parser = argparse.ArgumentParser(description="Count bridge references (contextra_types|ports|testkit)")
    parser.add_argument("--crate", "-c", help="Specific crate to check")
    parser.add_argument("--root", default=".", help="Workspace root directory")
    args = parser.parse_args()

    root = Path(args.root).resolve()
    crates_dir = root / "crates"

    results: dict[str, int] = {}
    if args.crate:
        target = crates_dir / args.crate
        if not target.exists():
            print(f"Error: crate directory not found: {target}", file=sys.stderr)
            return 2
        cnt = count_crate_bridge_refs(target)
        results[args.crate] = cnt
    else:
        for item in sorted(crates_dir.iterdir()):
            if item.is_dir() and (item / "Cargo.toml").exists():
                results[item.name] = count_crate_bridge_refs(item)

    print(f"{'Crate':<32} | {'Bridge Refs (types|ports|testkit)':<35}")
    print("-" * 70)
    for c, cnt in results.items():
        print(f"{c:<32} | {cnt:<35}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
