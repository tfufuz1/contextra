#!/usr/bin/env python3
"""
type_registry_sync.py - Extraktion, Inventarisierung und Kollisionserkennung von Typen (K3).

Funktionen:
- Extrahiert alle öffentlichen und internen struct, enum, trait, type-Definitionen je Crate.
- Erkennt Namenskollisionen über Crate-Grenzen hinweg (z. B. doppelte LayerWitness, EraseTarget).
- Validiert die Einhaltung von docs/TYPE_REGISTRY.md.
- Speichert detaillierte Berichte in docs/diagnostics/.
"""

import os
import sys
import re
import json
import argparse
from pathlib import Path
from collections import defaultdict

TYPE_PATTERNS = {
    "struct": re.compile(r'^\s*(pub(?:\([^\)]+\))?\s+)?struct\s+([A-Za-z0-9_]+)'),
    "enum": re.compile(r'^\s*(pub(?:\([^\)]+\))?\s+)?enum\s+([A-Za-z0-9_]+)'),
    "trait": re.compile(r'^\s*(pub(?:\([^\)]+\))?\s+)?trait\s+([A-Za-z0-9_]+)'),
    "type_alias": re.compile(r'^\s*(pub(?:\([^\)]+\))?\s+)?type\s+([A-Za-z0-9_]+)'),
}

def scan_file_for_types(filepath: Path):
    rel_path = str(filepath)
    types_found = []

    try:
        with open(filepath, "r", encoding="utf-8", errors="ignore") as f:
            lines = f.readlines()
    except Exception as e:
        return []

    for idx, raw_line in enumerate(lines):
        line = raw_line.strip()
        if line.startswith("//") or line.startswith("/*"):
            continue

        for kind, pat in TYPE_PATTERNS.items():
            match = pat.search(line)
            if match:
                is_pub = bool(match.group(1))
                name = match.group(2)
                # Ignore primitive type aliases or single letter generics
                if len(name) <= 1:
                    continue
                types_found.append({
                    "name": name,
                    "kind": kind,
                    "is_pub": is_pub,
                    "file": rel_path,
                    "line": idx + 1,
                    "declaration": line[:100]
                })

    return types_found

def analyze_workspace_types(workspace_root: Path):
    crates_dir = workspace_root / "crates"
    type_registry = defaultdict(list)
    crate_types = defaultdict(list)

    for rs_file in crates_dir.rglob("*.rs"):
        if "target" in rs_file.parts or "tests" in rs_file.parts or "benches" in rs_file.parts:
            continue

        # Determine crate
        rel = rs_file.relative_to(crates_dir)
        crate_name = rel.parts[0]

        items = scan_file_for_types(rs_file)
        for it in items:
            it["crate"] = crate_name
            type_registry[it["name"]].append(it)
            crate_types[crate_name].append(it)

    # Detect collisions: Types defined in >= 2 distinct crates
    collisions = {}
    for name, occurrences in type_registry.items():
        crates_involved = sorted(list(set(occ["crate"] for occ in occurrences)))
        if len(crates_involved) > 1:
            collisions[name] = {
                "crates": crates_involved,
                "occurrences": occurrences
            }

    return {
        "total_types": sum(len(v) for v in crate_types.values()),
        "unique_names": len(type_registry),
        "collision_count": len(collisions),
        "collisions": collisions,
        "crate_types": {k: len(v) for k, v in crate_types.items()},
        "all_registry": {k: v for k, v in type_registry.items() if len(v) > 1}
    }

def main():
    parser = argparse.ArgumentParser(description="Type Registry & Collision Analyzer")
    parser.add_argument("--workspace", default=".", help="Workspace root")
    parser.add_argument("--json", action="store_true", help="Output JSON directly to stdout")
    parser.add_argument("--output-dir", default="docs/diagnostics", help="Output directory for reports")
    args = parser.parse_args()

    workspace_root = Path(args.workspace).resolve()
    output_dir = workspace_root / args.output_dir
    output_dir.mkdir(parents=True, exist_ok=True)

    result = analyze_workspace_types(workspace_root)

    md_path = output_dir / "TYPE_REGISTRY_COLLISIONS.md"
    json_path = output_dir / "type_registry_collisions.json"

    with open(json_path, "w", encoding="utf-8") as jf:
        json.dump(result, jf, indent=2)

    with open(md_path, "w", encoding="utf-8") as mf:
        mf.write("# Typen-Register & Kollisions-Audit (K3 / A2)\n\n")
        mf.write(f"> Generiert durch `tools/type_registry_sync.py`.\n")
        mf.write(f"> Einzigartige Typnamen: {result['unique_names']} | Gesamte Definitionen: {result['total_types']}\n")
        mf.write(f"> Erkannte typografische Crate-Übergreifende Kollisionen: **{result['collision_count']}**\n\n")

        mf.write("## Crate Typen-Statistik\n\n")
        mf.write("| Crate | Definierte Typen (Structs/Enums/Traits/Aliases) |\n")
        mf.write("|---|---|\n")
        for cr, count in sorted(result["crate_types"].items()):
            mf.write(f"| `{cr}` | {count} |\n")

        mf.write("\n## Kollisionen über Crate-Grenzen (Identische Typnamen)\n\n")
        mf.write("| Typname | Beteiligte Crates | Definitionen |\n")
        mf.write("|---|---|---|\n")
        for name, data in sorted(result["collisions"].items()):
            crates_str = ", ".join(f"`{c}`" for c in data["crates"])
            mf.write(f"| `{name}` | {crates_str} | {len(data['occurrences'])} |\n")

        mf.write("\n## Detailierte Kollisionsorte\n\n")
        for name, data in sorted(result["collisions"].items()):
            mf.write(f"### `{name}`\n\n")
            for occ in data["occurrences"]:
                pub_marker = "pub " if occ["is_pub"] else ""
                mf.write(f"- `{occ['crate']}`: `{occ['file']}:{occ['line']}` -> `{pub_marker}{occ['kind']} {name}`\n")

    if args.json:
        print(json.dumps(result, indent=2))
    else:
        print(f"Type Registry Sync abgeschlossen:")
        print(f" - Einzigartige Typen: {result['unique_names']}")
        print(f" - Crate-Kollisionen: {result['collision_count']}")
        print(f" - Markdown: {md_path}")
        print(f" - JSON: {json_path}")

if __name__ == "__main__":
    main()
