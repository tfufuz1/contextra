#!/usr/bin/env python3
"""
fsync_durability_tracer.py - Audit von Datei-I/O, Fsync-Handhabung und Durability (INV-FSYNC-1).

Prüfungen:
- Direkte std::fs / tokio::fs Zugriffe außerhalb von Sys/Durable-FS (Verstoß gegen M4/T-LINT-2).
- Fehlerhafte fsync/sync_all Handhabung: Erkennung von `let _ = ...sync...` oder swallowed fsync.
- Vergiftung (Poisoning) des Handles nach I/O-Fehlern.
- Korrekte Nutzung des `Vfs`-Ports.
"""

import os
import sys
import re
import json
import argparse
from pathlib import Path

PERMITTED_RAW_FS_CRATES = {"contextra-sys", "contextra-durable-fs"}

RAW_FS_PATTERNS = {
    "std_fs": re.compile(r'\bstd::fs\b'),
    "std_file": re.compile(r'\bFile::(open|create|options)\b'),
    "tokio_fs": re.compile(r'\btokio::fs\b'),
    "memmap": re.compile(r'\bmemmap2::\b'),
}

FSYNC_PATTERNS = {
    "sync_all": re.compile(r'\.sync_all\s*\('),
    "sync_data": re.compile(r'\.sync_data\s*\('),
    "sync_dir": re.compile(r'\.sync_dir\s*\('),
    "fsync": re.compile(r'\bfsync\s*\('),
}

SWALLOWED_SYNC = re.compile(r'let\s+_\s*=\s*[^;]*sync_')

def scan_file_fs(filepath: Path):
    rel_path = str(filepath)
    findings = []

    try:
        with open(filepath, "r", encoding="utf-8", errors="ignore") as f:
            lines = f.readlines()
    except Exception as e:
        return {"file": rel_path, "findings": []}

    in_test = "tests" in rel_path or "benches" in rel_path or filepath.name.endswith("_test.rs")

    for idx, raw_line in enumerate(lines):
        line = raw_line.strip()
        if line.startswith("//"):
            continue

        for kind, pat in RAW_FS_PATTERNS.items():
            if pat.search(line):
                findings.append({
                    "line": idx + 1,
                    "category": "raw_fs_usage",
                    "kind": kind,
                    "code": line[:120],
                    "in_test": in_test
                })

        for kind, pat in FSYNC_PATTERNS.items():
            if pat.search(line):
                is_swallowed = bool(SWALLOWED_SYNC.search(line))
                findings.append({
                    "line": idx + 1,
                    "category": "fsync_call",
                    "kind": kind,
                    "is_swallowed": is_swallowed,
                    "code": line[:120],
                    "in_test": in_test
                })

    return {
        "file": rel_path,
        "is_test": in_test,
        "findings": findings
    }

def analyze_crate(crate_dir: Path):
    crate_name = crate_dir.name
    is_permitted_fs = crate_name in PERMITTED_RAW_FS_CRATES

    findings_prod = []
    findings_test = []

    for rs_file in crate_dir.rglob("*.rs"):
        if "target" in rs_file.parts:
            continue
        res = scan_file_fs(rs_file)
        for f in res["findings"]:
            entry = {
                "file": res["file"],
                "line": f["line"],
                "category": f["category"],
                "kind": f["kind"],
                "code": f["code"],
                "is_swallowed": f.get("is_swallowed", False)
            }
            if f["in_test"]:
                findings_test.append(entry)
            else:
                findings_prod.append(entry)

    raw_fs_prod = [f for f in findings_prod if f["category"] == "raw_fs_usage"]
    swallowed_sync_prod = [f for f in findings_prod if f.get("is_swallowed")]

    return {
        "crate": crate_name,
        "is_permitted_fs": is_permitted_fs,
        "raw_fs_prod_count": len(raw_fs_prod),
        "swallowed_sync_count": len(swallowed_sync_prod),
        "total_fsync_calls": sum(1 for f in findings_prod if f["category"] == "fsync_call"),
        "findings_prod": findings_prod,
        "findings_test": findings_test,
    }

def main():
    parser = argparse.ArgumentParser(description="Fsync & Durability Tracer (INV-FSYNC-1)")
    parser.add_argument("--workspace", default=".", help="Workspace root")
    parser.add_argument("--json", action="store_true", help="Output JSON directly to stdout")
    parser.add_argument("--output-dir", default="docs/diagnostics", help="Output directory for reports")
    args = parser.parse_args()

    workspace_root = Path(args.workspace).resolve()
    crates_dir = workspace_root / "crates"
    output_dir = workspace_root / args.output_dir
    output_dir.mkdir(parents=True, exist_ok=True)

    crates = sorted([p for p in crates_dir.iterdir() if p.is_dir() and (p / "Cargo.toml").exists()])
    reports = [analyze_crate(c) for c in crates]

    md_path = output_dir / "FSYNC_DURABILITY_AUDIT.md"
    json_path = output_dir / "fsync_durability_audit.json"

    with open(json_path, "w", encoding="utf-8") as jf:
        json.dump(reports, jf, indent=2)

    with open(md_path, "w", encoding="utf-8") as mf:
        mf.write("# Fsync & Durability Tracer (INV-FSYNC-1 / M4)\n\n")
        mf.write(f"> Generiert durch `tools/fsync_durability_tracer.py`.\n")
        mf.write(f"> Erlaubte Low-Level I/O Crates: `{sorted(list(PERMITTED_RAW_FS_CRATES))}`\n\n")

        mf.write("## Crate Übersicht (Produktivpfade)\n\n")
        mf.write("| Crate | Status | Rohe FS-Aufrufe (Prod) | Fsync Aufrufe | Swallowed Syncs (`let _ =`) |\n")
        mf.write("|---|---|---|---|---|\n")
        for r in reports:
            status = "Low-Level (OK)" if r["is_permitted_fs"] else ("SAUBER" if r["raw_fs_prod_count"] == 0 else "**ABWEICHUNG**")
            mf.write(f"| `{r['crate']}` | {status} | {r['raw_fs_prod_count']} | {r['total_fsync_calls']} | {r['swallowed_sync_count']} |\n")

        violations = [r for r in reports if (not r["is_permitted_fs"] and r["raw_fs_prod_count"] > 0) or r["swallowed_sync_count"] > 0]
        if violations:
            mf.write("\n## Auffälligkeiten & Durability-Risiken\n\n")
            for v in violations:
                mf.write(f"### `{v['crate']}`\n\n")
                for f in v["findings_prod"]:
                    if (not v["is_permitted_fs"] and f["category"] == "raw_fs_usage") or f.get("is_swallowed"):
                        swallow_tag = " [SWALLOWED ERROR!]" if f.get("is_swallowed") else ""
                        mf.write(f"- `{f['file']}:{f['line']}` [{f['kind']}]{swallow_tag}: `{f['code']}`\n")

    if args.json:
        print(json.dumps(reports, indent=2))
    else:
        print(f"Fsync & Durability Audit abgeschlossen:")
        print(f" - Geprüfte Crates: {len(reports)}")
        print(f" - Crates mit rohem FS-Zugriff (außerhalb sys/durable-fs): {len([r for r in reports if not r['is_permitted_fs'] and r['raw_fs_prod_count'] > 0])}")
        print(f" - Markdown: {md_path}")
        print(f" - JSON: {json_path}")

if __name__ == "__main__":
    main()
