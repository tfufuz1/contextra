#!/usr/bin/env python3
"""
loc.py - Counts lines of Rust production code per crate under crates/.

Rule definition:
- Target: Rust files (.rs) under crates/
- Excluded paths: containing /tests/, /benches/, /target/, or files named *_test.rs or *_tests.rs
- Line definition: Code line if it is non-empty and not purely a comment line (single-line // or block comment /* ... */).
- No external dependencies, Python 3 standard library only.
"""

import sys
import os
import json

CORE_CRATES = [
    "contextra-types",
    "contextra-ports",
    "contextra-mvcc",
    "contextra-crypto",
    "contextra-vector",
    "contextra-text",
    "contextra-store",
    "contextra-checkpoint",
    "contextra-engine",
    "contextra-db",
    "contextra",
    "contextra-mcp",
]

def count_loc(filepath):
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

def analyze_crates(crates_dir="crates"):
    results = {}
    if not os.path.exists(crates_dir):
        return results

    for crate in sorted(os.listdir(crates_dir)):
        crate_path = os.path.join(crates_dir, crate)
        if not os.path.isdir(crate_path):
            continue

        crate_files = []
        total_crate_loc = 0

        for root, dirs, files in os.walk(crate_path):
            # Prune excluded directories from traversal
            for excluded in ["target", "tests", "benches"]:
                if excluded in dirs:
                    dirs.remove(excluded)

            for f in sorted(files):
                if f.endswith('.rs') and not (f.endswith('_test.rs') or f.endswith('_tests.rs')):
                    full_path = os.path.normpath(os.path.join(root, f))
                    # Normalize path separators to forward slashes for cross-platform consistency
                    rel_path = full_path.replace('\\', '/')

                    # Secondary check for excluded path components
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
        results[crate] = {
            "loc": total_crate_loc,
            "files_count": len(crate_files),
            "files": crate_files,
        }

    return results

def main():
    as_json = False
    if "--json" in sys.argv or "-j" in sys.argv:
        as_json = True

    crate_data = analyze_crates("crates")

    total_loc = sum(v["loc"] for v in crate_data.values())
    core_loc = sum(v["loc"] for k, v in crate_data.items() if k in CORE_CRATES)

    if as_json:
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
        print("Code-Zeilen (LOC) je Crate unter crates/:")
        print("=" * 60)
        for crate in sorted(crate_data.keys()):
            info = crate_data[crate]
            is_core = "[CORE]" if crate in CORE_CRATES else "      "
            print(f"{is_core} {crate:25}: {info['loc']:6} Zeilen ({info['files_count']:2} Dateien)")
        print("=" * 60)
        print(f"Gesamtsumme (alle {len(crate_data)} Crates): {total_loc} Zeilen")
        print(f"Kernprodukt-Summe ({len(CORE_CRATES)} Kern-Crates): {core_loc} Zeilen")

if __name__ == "__main__":
    main()
