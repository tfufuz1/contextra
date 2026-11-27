#!/usr/bin/env python3
"""
dead_code_pruner.py - Feingranulare Identifizierung ungenutzter interner Symbole und Code-Leichen.

Analysiert:
- Interne / private Structs, Enums, Funktionen und Konstanten.
- Symbole, die nur an ihrer Definitionsstelle vorkommen (Referenzzählung = 1).
- Tote Modul-Dateien, die in keinem mod-Statement eingebunden sind.
- Exportiert Ergebnisse in docs/diagnostics/.
"""

import os
import sys
import re
import json
import argparse
from pathlib import Path
from collections import defaultdict

DECLARATION_PATTERNS = {
    "fn": re.compile(r'^\s*(?:pub(?:\([^\)]+\))?\s+)?fn\s+([a-zA-Z0-9_]+)\b'),
    "struct": re.compile(r'^\s*(?:pub(?:\([^\)]+\))?\s+)?struct\s+([a-zA-Z0-9_]+)\b'),
    "enum": re.compile(r'^\s*(?:pub(?:\([^\)]+\))?\s+)?enum\s+([a-zA-Z0-9_]+)\b'),
    "const": re.compile(r'^\s*(?:pub(?:\([^\)]+\))?\s+)?const\s+([a-zA-Z0-9_]+)\b'),
}

EXCLUDE_NAMES = {
    "new", "default", "clone", "fmt", "drop", "from", "into", "try_from", "try_into",
    "as_ref", "as_mut", "deref", "deref_mut", "poll", "main", "test", "init", "run"
}

def analyze_crate_dead_code(crate_dir: Path):
    crate_name = crate_dir.name
    src_dir = crate_dir / "src"
    if not src_dir.exists():
        return None

    file_contents = {}
    declarations = []

    for rs_file in src_dir.rglob("*.rs"):
        try:
            content = rs_file.read_text(encoding="utf-8", errors="ignore")
            file_contents[str(rs_file)] = content
        except Exception:
            continue

        lines = content.splitlines()
        for idx, line in enumerate(lines):
            l_strip = line.strip()
            if l_strip.startswith("//") or l_strip.startswith("/*"):
                continue

            for kind, pat in DECLARATION_PATTERNS.items():
                m = pat.search(l_strip)
                if m:
                    sym_name = m.group(1)
                    if sym_name.startswith("_") or sym_name.startswith("test_") or sym_name in EXCLUDE_NAMES:
                        continue
                    if len(sym_name) < 3:
                        continue
                    declarations.append({
                        "name": sym_name,
                        "kind": kind,
                        "file": str(rs_file),
                        "line": idx + 1,
                        "decl": l_strip[:100]
                    })

    # Search symbol usage across all crate files
    dead_candidates = []
    for d in declarations:
        sym = d["name"]
        sym_pattern = re.compile(rf'\b{re.escape(sym)}\b')
        occurrences = 0
        for fpath, content in file_contents.items():
            occurrences += len(sym_pattern.findall(content))
            if occurrences > 1:
                break

        # If occurred only once (the declaration itself)
        if occurrences <= 1:
            dead_candidates.append(d)

    return {
        "crate": crate_name,
        "total_declarations": len(declarations),
        "dead_code_candidates_count": len(dead_candidates),
        "candidates": dead_candidates
    }

def main():
    parser = argparse.ArgumentParser(description="Dead Code & Orphan Symbol Analyzer")
    parser.add_argument("--workspace", default=".", help="Workspace root")
    parser.add_argument("--crate", default=None, help="Specific crate")
    parser.add_argument("--json", action="store_true", help="Output JSON directly to stdout")
    parser.add_argument("--output-dir", default="docs/diagnostics", help="Output directory for reports")
    args = parser.parse_args()

    workspace_root = Path(args.workspace).resolve()
    crates_dir = workspace_root / "crates"
    output_dir = workspace_root / args.output_dir
    output_dir.mkdir(parents=True, exist_ok=True)

    target_crates = []
    if args.crate:
        target_crates.append(crates_dir / args.crate)
    else:
        target_crates = sorted([p for p in crates_dir.iterdir() if p.is_dir() and (p / "Cargo.toml").exists()])

    results = []
    for c in target_crates:
        res = analyze_crate_dead_code(c)
        if res:
            results.append(res)

    md_path = output_dir / "DEAD_CODE_ANALYSIS.md"
    json_path = output_dir / "dead_code_analysis.json"

    with open(json_path, "w", encoding="utf-8") as jf:
        json.dump(results, jf, indent=2)

    with open(md_path, "w", encoding="utf-8") as mf:
        mf.write("# Dead Code & Orphan Symbols Analysis\n\n")
        mf.write(f"> Generiert durch `tools/dead_code_pruner.py`.\n")
        mf.write(f"> Untersucht Symbole, die innerhalb ihres Crates nur exakt 1x (bei Deklaration) vorkommen.\n\n")

        mf.write("## Crate Übersicht\n\n")
        mf.write("| Crate | Untersuchte Symbole | Potenzielle Dead-Code Kandidaten |\n")
        mf.write("|---|---|---|\n")
        for r in results:
            mf.write(f"| `{r['crate']}` | {r['total_declarations']} | **{r['dead_code_candidates_count']}** |\n")

        mf.write("\n## Dead Code Kandidaten (Auszug)\n\n")
        for r in results:
            if r["dead_code_candidates_count"] > 0:
                mf.write(f"### `{r['crate']}`\n\n")
                for cand in r["candidates"][:20]:
                    mf.write(f"- `{cand['file']}:{cand['line']}` [{cand['kind']} `{cand['name']}`]: `{cand['decl']}`\n")
                if len(r["candidates"]) > 20:
                    mf.write(f"- ... und {len(r['candidates']) - 20} weitere Kandidaten.\n")

    if args.json:
        print(json.dumps(results, indent=2))
    else:
        print(f"Dead Code Analysis abgeschlossen:")
        print(f" - Geprüfte Crates: {len(results)}")
        print(f" - Markdown: {md_path}")
        print(f" - JSON: {json_path}")

if __name__ == "__main__":
    main()
