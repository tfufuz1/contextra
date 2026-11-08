#!/usr/bin/env python3
"""
loc.py - Counts lines of Rust production code per crate under crates/ and benchmarks/.

Rule definition:
- Target: Rust files (.rs) under crates/ and benchmarks/
- Excluded paths: containing /tests/, /benches/, /target/, or files named *_test.rs or *_tests.rs
- Line definition: Code line if it is non-empty and not purely a comment line (single-line // or block comment /* ... */).
- No external dependencies, Python 3 standard library only.
"""

from __future__ import annotations

import argparse
import json
import os
import sys
from pathlib import Path

CORE_CRATES = [
    'contextra-types', 'contextra-ports', 'contextra-mvcc', 'contextra-crypto',
    'contextra-vector', 'contextra-text', 'contextra-store', 'contextra-durable-fs',
    'contextra-simd', 'contextra-sys', 'contextra-rank', 'contextra-erasure',
    'contextra-engine', 'contextra'
]

def count_loc(filepath: str) -> int:
    try:
        with open(filepath, 'r', encoding='utf-8', errors='ignore') as f:
            lines = f.readlines()
    except Exception:
        return 0

    loc = 0
    in_block_depth = 0
    for line in lines:
        line_str = line.strip()
        if not line_str:
            continue

        has_code = False
        i = 0
        n = len(line_str)
        while i < n:
            if in_block_depth > 0:
                if i + 1 < n and line_str[i:i+2] == '*/':
                    in_block_depth -= 1
                    i += 2
                elif i + 1 < n and line_str[i:i+2] == '/*':
                    in_block_depth += 1
                    i += 2
                else:
                    i += 1
            else:
                if i + 1 < n and line_str[i:i+2] == '/*':
                    in_block_depth += 1
                    i += 2
                elif i + 1 < n and line_str[i:i+2] == '//':
                    break
                elif not line_str[i].isspace():
                    has_code = True
                    i += 1
                else:
                    i += 1
        if has_code:
            loc += 1
    return loc

def analyze_directory(base_dir: str, target_crate: str | None = None) -> dict:
    results = {}
    if not os.path.exists(base_dir):
        return results

    for entry in sorted(os.listdir(base_dir)):
        entry_path = os.path.join(base_dir, entry)
        if not os.path.isdir(entry_path):
            continue

        # If Cargo.toml exists, this directory is a crate
        cargo_path = os.path.join(entry_path, "Cargo.toml")
        if not os.path.isfile(cargo_path):
            continue

        crate_name = entry
        if target_crate and crate_name != target_crate:
            continue

        crate_files = []
        total_crate_loc = 0

        for root, dirs, files in os.walk(entry_path):
            for excluded in ["target", "tests", "benches"]:
                if excluded in dirs:
                    dirs.remove(excluded)

            for f in sorted(files):
                if f.endswith('.rs') and not (f.endswith('_test.rs') or f.endswith('_tests.rs')):
                    full_path = os.path.normpath(os.path.join(root, f))
                    rel_path = full_path.replace('\\', '/')

                    path_parts = rel_path.split('/')
                    if 'tests' in path_parts or 'benches' in path_parts:
                        continue

                    lines = count_loc(full_path)
                    crate_files.append({
                        "path": rel_path,
                        "loc": lines
                    })
                    total_crate_loc += lines

        crate_files.sort(key=lambda x: x["path"])
        results[crate_name] = {
            "loc": total_crate_loc,
            "files_count": len(crate_files),
            "files": crate_files,
        }

    return results

def main() -> int:
    parser = argparse.ArgumentParser(description="Zählt produktive Rust-Zeilen (LOC) je Crate.")
    parser.add_argument("--crate", "-c", help="Nur ein spezifisches Crate analysieren")
    parser.add_argument("--json", "-j", action="store_true", help="JSON-Format für Maschinenlesbarkeit")
    args = parser.parse_args()

    crate_data = {}
    for base in ["crates", "benchmarks"]:
        crate_data.update(analyze_directory(base, args.crate))

    total_loc = sum(v["loc"] for v in crate_data.values())
    core_loc = sum(v["loc"] for k, v in crate_data.items() if k in CORE_CRATES)

    if args.json:
        output_obj = {
            "summary": {
                "core_loc": core_loc,
                "total_crates": len(crate_data),
                "total_loc": total_loc,
            },
            "core_crates": CORE_CRATES,
            "crates": crate_data,
        }
        print(json.dumps(output_obj, indent=2, sort_keys=True))
    else:
        print("Code-Zeilen (LOC) je Crate (crates/ und benchmarks/):")
        print("=" * 60)
        for crate in sorted(crate_data.keys()):
            info = crate_data[crate]
            is_core = "[CORE]" if crate in CORE_CRATES else "      "
            print(f"{is_core} {crate:25}: {info['loc']:6} Zeilen ({info['files_count']:2} Dateien)")
        print("=" * 60)
        print(f"Gesamtsumme (alle {len(crate_data)} Crates): {total_loc} Zeilen")
        print(f"Kernprodukt-Summe ({len(CORE_CRATES)} Kern-Crates): {core_loc} Zeilen")

    return 0

if __name__ == "__main__":
    sys.exit(main())
