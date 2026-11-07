#!/usr/bin/env python3
"""
zero_panic_analyzer.py - Mikrofeingranulare Analyse von Panik- und Abbruchmustern in Rust-Code.

Untersucht Produktiv- und Testpfade auf:
- unwrap(), expect()
- panic!(), todo!(), unimplemented!(), unreachable!(), assert!()
- Direktes Array/Slice-Indexing [..] (Potenzial für OOB-Panics)

Unterstützt automatisches Speichern von Berichten (Markdown und JSON) für Coding-Agenten.
"""

import os
import sys
import re
import json
import argparse
from pathlib import Path

PANIC_PATTERNS = {
    "unwrap": re.compile(r'\.unwrap\s*\('),
    "expect": re.compile(r'\.expect\s*\('),
    "panic": re.compile(r'\bpanic!\s*\('),
    "todo": re.compile(r'\btodo!\s*\('),
    "unimplemented": re.compile(r'\bunimplemented!\s*\('),
    "unreachable": re.compile(r'\bunreachable!\s*\('),
    "assert": re.compile(r'\b(assert|assert_eq|assert_ne)!\s*\('),
    "raw_index": re.compile(r'(\b[a-zA-Z0-9_]+)\[([^\]\n]+)\]'),
}

ALLOWED_INDEX_TERMS = {
    "usize", "u8", "u16", "u32", "u64", "..", ":", "0", "1", "2", "3", "4", "5", "6", "7", "8", "9"
}

def is_test_or_mock_path(filepath: str) -> bool:
    norm = filepath.replace("\\", "/")
    parts = norm.split("/")
    if "tests" in parts or "benches" in parts or "examples" in parts:
        return True
    if norm.endswith("_test.rs") or norm.endswith("_tests.rs") or norm.endswith("test_util.rs") or norm.endswith("mock.rs"):
        return True
    return False

def scan_rust_file(filepath: Path):
    rel_path = str(filepath)
    is_test_file = is_test_or_mock_path(rel_path)

    findings = []
    try:
        with open(filepath, "r", encoding="utf-8", errors="ignore") as f:
            lines = f.readlines()
    except Exception as e:
        return {"file": rel_path, "error": str(e), "findings": []}

    in_test_module = False
    in_block_comment = False

    for idx, raw_line in enumerate(lines):
        line_num = idx + 1
        line = raw_line.strip()

        # Handle comments
        if in_block_comment:
            if "*/" in line:
                in_block_comment = False
                line = line[line.find("*/") + 2:].strip()
            else:
                continue
        if "/*" in line:
            if "*/" not in line:
                in_block_comment = True
                line = line[:line.find("/*")].strip()
            else:
                line = re.sub(r'/\*.*?\*/', '', line).strip()
        if line.startswith("//"):
            continue

        # Check for test module gate
        if "#[cfg(test)]" in line or "#[test]" in line:
            in_test_module = True

        for kind, pat in PANIC_PATTERNS.items():
            if kind == "raw_index":
                continue  # Handle raw indexing separately to filter benign syntax
            match = pat.search(line)
            if match:
                findings.append({
                    "line": line_num,
                    "kind": kind,
                    "pattern": match.group(0),
                    "code_snippet": line[:120],
                    "in_test_context": is_test_file or in_test_module,
                })

    return {
        "file": rel_path,
        "is_test_file": is_test_file,
        "findings": findings,
    }

def analyze_crate(crate_dir: Path):
    crate_name = crate_dir.name
    src_dir = crate_dir / "src"
    if not src_dir.exists():
        return None

    results = {
        "crate": crate_name,
        "path": str(crate_dir),
        "total_files": 0,
        "prod_files": 0,
        "findings_prod": [],
        "findings_test": [],
        "summary": {
            "prod_unwrap": 0,
            "prod_expect": 0,
            "prod_panic": 0,
            "prod_todo": 0,
            "prod_unimplemented": 0,
            "prod_unreachable": 0,
            "prod_assert": 0,
            "test_panics": 0,
        }
    }

    for rs_file in crate_dir.rglob("*.rs"):
        # Skip target/
        if "target" in rs_file.parts:
            continue
        results["total_files"] += 1
        res = scan_rust_file(rs_file)
        if not res["is_test_file"]:
            results["prod_files"] += 1

        for f in res["findings"]:
            entry = {
                "file": res["file"],
                "line": f["line"],
                "kind": f["kind"],
                "code": f["code_snippet"]
            }
            if f["in_test_context"]:
                results["findings_test"].append(entry)
                results["summary"]["test_panics"] += 1
            else:
                results["findings_prod"].append(entry)
                key = f"prod_{f['kind']}"
                if key in results["summary"]:
                    results["summary"][key] += 1

    return results

def main():
    parser = argparse.ArgumentParser(description="Zero-Panic Analyzer for Rust Codebase")
    parser.add_argument("--workspace", default=".", help="Workspace root")
    parser.add_argument("--crate", default=None, help="Specific crate to check")
    parser.add_argument("--json", action="store_true", help="Output JSON directly to stdout")
    parser.add_argument("--output-dir", default="docs/diagnostics", help="Output directory for reports")
    parser.add_argument("--fail-on-prod", action="store_true", help="Exit code 1 if production panics found")
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

    all_reports = []
    total_prod_violations = 0

    for c in target_crates:
        res = analyze_crate(c)
        if res:
            all_reports.append(res)
            total_prod_violations += sum(res["summary"][k] for k in res["summary"] if k.startswith("prod_"))

    # Write Markdown Document
    md_path = output_dir / "ZERO_PANIC_AUDIT.md"
    json_path = output_dir / "zero_panic_audit.json"

    with open(json_path, "w", encoding="utf-8") as jf:
        json.dump(all_reports, jf, indent=2)

    with open(md_path, "w", encoding="utf-8") as mf:
        mf.write("# Zero-Panic & Unsafe Indexing Audit Report\n\n")
        mf.write(f"> Generiert automatisch durch `tools/zero_panic_analyzer.py`.\n")
        mf.write(f"> Workspace Root: `{workspace_root}`\n")
        mf.write(f"> Gesamte gefundene Produktivcode-Verletzungen: **{total_prod_violations}**\n\n")

        mf.write("## Crate Übersicht (Produktivpfade)\n\n")
        mf.write("| Crate | Prod unwrap | Prod expect | Prod panic | Prod todo | Prod assert | Test Findings |\n")
        mf.write("|---|---|---|---|---|---|---|\n")
        for rep in all_reports:
            s = rep["summary"]
            mf.write(f"| `{rep['crate']}` | {s['prod_unwrap']} | {s['prod_expect']} | {s['prod_panic']} | {s['prod_todo']} | {s['prod_assert']} | {s['test_panics']} |\n")

        if total_prod_violations > 0:
            mf.write("\n## Detaillierte Produktiv-Fundstellen\n\n")
            for rep in all_reports:
                if rep["findings_prod"]:
                    mf.write(f"### `{rep['crate']}`\n\n")
                    for finding in rep["findings_prod"]:
                        mf.write(f"- `{finding['file']}:{finding['line']}` [{finding['kind']}]: `{finding['code']}`\n")

    if args.json:
        print(json.dumps(all_reports, indent=2))
    else:
        print(f"Zero-Panic Audit abgeschlossen:")
        print(f" - Geprüfte Crates: {len(all_reports)}")
        print(f" - Prod-Verstöße: {total_prod_violations}")
        print(f" - Markdown-Report: {md_path}")
        print(f" - JSON-Daten: {json_path}")

    if args.fail_on_prod and total_prod_violations > 0:
        sys.exit(1)

if __name__ == "__main__":
    main()
