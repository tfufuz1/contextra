#!/usr/bin/env python3
"""
async_boundary_scanner.py - Diagnosewerkzeug für Asynchronitäts- und Sperr-Grenzen (M8 / INV-LOCK-ORDER-1).

Prüfungen:
- Vorkommen von `async fn`, `tokio::`, `.await` in Ring 0 und Persistenzschichten.
  (M8: Ring 0 und Speicherkerne halten ihre Logik synchron. Async nur an der Port-Grenze).
- Erkennung von potenziellen Lock-Guards über `.await`-Aufrufen (clippy::await_holding_lock).
- Identifikation von Blockierern in synchronen Threads.
"""

import os
import sys
import re
import json
import argparse
from pathlib import Path

# Crates that MUST remain synchronous in their core logic per M8
SYNC_MANDATE_CRATES = {
    "contextra-types",
    "contextra-ports",
    "contextra-mvcc",
    "contextra-crypto",
    "contextra-rank",
    "contextra-adapt",
    "contextra-durable-fs",
    "contextra-sys",
    "contextra-simd",
    "contextra-wire",
}

ASYNC_PATTERNS = {
    "async_fn": re.compile(r'\basync\s+fn\b'),
    "await_expr": re.compile(r'\.await\b'),
    "tokio_import": re.compile(r'\btokio::'),
    "block_on": re.compile(r'\bblock_on\s*\('),
}

LOCK_ACQUISITIONS = re.compile(r'\b(lock|read|write)\s*\(\)\s*(\.unwrap\(\))?')

def scan_file_async(filepath: Path):
    rel_path = str(filepath)
    findings = []

    try:
        with open(filepath, "r", encoding="utf-8", errors="ignore") as f:
            lines = f.readlines()
    except Exception as e:
        return {"file": rel_path, "findings": []}

    in_test = "tests" in rel_path or "benches" in rel_path or filepath.name.endswith("_test.rs")
    held_locks = []

    for idx, raw_line in enumerate(lines):
        line = raw_line.strip()
        if line.startswith("//"):
            continue

        for kind, pat in ASYNC_PATTERNS.items():
            if pat.search(line):
                findings.append({
                    "line": idx + 1,
                    "kind": kind,
                    "code": line[:120],
                    "in_test": in_test
                })

        # Track possible lock across await heuristic
        if "let " in line and LOCK_ACQUISITIONS.search(line):
            held_locks.append((idx + 1, line))
        if ".await" in line and held_locks:
            # Report potential lock held across await
            recent_lock = held_locks[-1]
            if idx + 1 - recent_lock[0] < 15: # heuristic scope proximity
                findings.append({
                    "line": idx + 1,
                    "kind": "lock_held_across_await_warning",
                    "code": f"Lock acquired at line {recent_lock[0]} near await at line {idx+1}",
                    "in_test": in_test
                })

    return {
        "file": rel_path,
        "is_test": in_test,
        "findings": findings
    }

def analyze_crate(crate_dir: Path):
    crate_name = crate_dir.name
    is_sync_mandated = crate_name in SYNC_MANDATE_CRATES

    all_findings_prod = []
    all_findings_test = []

    for rs_file in crate_dir.rglob("*.rs"):
        if "target" in rs_file.parts:
            continue
        res = scan_file_async(rs_file)
        for f in res["findings"]:
            entry = {
                "file": res["file"],
                "line": f["line"],
                "kind": f["kind"],
                "code": f["code"]
            }
            if f["in_test"]:
                all_findings_test.append(entry)
            else:
                all_findings_prod.append(entry)

    summary_prod = {
        "async_fn": sum(1 for f in all_findings_prod if f["kind"] == "async_fn"),
        "await_expr": sum(1 for f in all_findings_prod if f["kind"] == "await_expr"),
        "tokio_import": sum(1 for f in all_findings_prod if f["kind"] == "tokio_import"),
        "block_on": sum(1 for f in all_findings_prod if f["kind"] == "block_on"),
        "lock_across_await": sum(1 for f in all_findings_prod if f["kind"] == "lock_held_across_await_warning"),
    }

    return {
        "crate": crate_name,
        "is_sync_mandated": is_sync_mandated,
        "summary_prod": summary_prod,
        "findings_prod": all_findings_prod,
        "findings_test": all_findings_test,
        "violations": sum(summary_prod.values()) if is_sync_mandated else summary_prod["lock_across_await"]
    }

def main():
    parser = argparse.ArgumentParser(description="Async Boundary & Lock-Across-Await Scanner")
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

    md_path = output_dir / "ASYNC_BOUNDARY_REPORT.md"
    json_path = output_dir / "async_boundary_report.json"

    with open(json_path, "w", encoding="utf-8") as jf:
        json.dump(reports, jf, indent=2)

    with open(md_path, "w", encoding="utf-8") as mf:
        mf.write("# Async Boundary & Concurrency Scanner (M8 / INV-LOCK-ORDER-1)\n\n")
        mf.write(f"> Generiert durch `tools/async_boundary_scanner.py`.\n")
        mf.write(f"> Regel M8: Ring 0 und Speicherkerne halten ihre Logik synchron. Async nur an der Port-Grenze.\n\n")

        mf.write("## Crate Übersicht (Produktivpfade)\n\n")
        mf.write("| Crate | Sync Mandat (M8) | async fn | .await | tokio:: | block_on | Lock across Await |\n")
        mf.write("|---|---|---|---|---|---|---|\n")
        for r in reports:
            mandate = "**MANDAT**" if r["is_sync_mandated"] else "Nein"
            s = r["summary_prod"]
            mf.write(f"| `{r['crate']}` | {mandate} | {s['async_fn']} | {s['await_expr']} | {s['tokio_import']} | {s['block_on']} | {s['lock_across_await']} |\n")

        violations = [r for r in reports if (r["is_sync_mandated"] and sum(r["summary_prod"].values()) > 0) or r["summary_prod"]["lock_across_await"] > 0]
        if violations:
            mf.write("\n## Auffälligkeiten & M8-Abweichungen\n\n")
            for v in violations:
                mf.write(f"### `{v['crate']}`\n\n")
                for f in v["findings_prod"]:
                    mf.write(f"- `{f['file']}:{f['line']}` [{f['kind']}]: `{f['code']}`\n")

    if args.json:
        print(json.dumps(reports, indent=2))
    else:
        print(f"Async-Boundary Scan abgeschlossen:")
        print(f" - Geprüfte Crates: {len(reports)}")
        print(f" - Crates mit Befunden: {len(violations)}")
        print(f" - Markdown: {md_path}")
        print(f" - JSON: {json_path}")

if __name__ == "__main__":
    main()
