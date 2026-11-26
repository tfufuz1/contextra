#!/usr/bin/env python3
"""
unsafe_boundary_auditor.py - Audit aller Unsafe-Blöcke und Funktionen in Rust-Crates.

Prüfungen:
- Vorkommen von `unsafe fn`, `unsafe trait`, `unsafe impl`, `unsafe {`
- Vorhandensein des vorgeschriebenen `// SAFETY:`-Nachweiskommentars gemäß M9
- Abgleich mit Unsafe-Inseln in capabilities.toml (nur contextra-simd, contextra-sys, contextra-wire dürfen unsafe enthalten)
- Erkennung von `#![forbid(unsafe_code)]` vs `#![allow(unsafe_code)]`

Erzeugt Markdown- und JSON-Berichte in docs/diagnostics/.
"""

import os
import sys
import re
import json
import argparse
from pathlib import Path

ALLOWED_UNSAFE_CRATES = {"contextra-simd", "contextra-sys", "contextra-wire"}

UNSAFE_PATTERNS = {
    "unsafe_block": re.compile(r'\bunsafe\s*\{'),
    "unsafe_fn": re.compile(r'\bunsafe\s+fn\b'),
    "unsafe_trait": re.compile(r'\bunsafe\s+trait\b'),
    "unsafe_impl": re.compile(r'\bunsafe\s+impl\b'),
}

FORBID_UNSAFE_PATTERN = re.compile(r'#!\[(?:deny|forbid)\(unsafe_code\)\]')

def audit_file(filepath: Path):
    rel_path = str(filepath)
    try:
        with open(filepath, "r", encoding="utf-8", errors="ignore") as f:
            lines = f.readlines()
    except Exception as e:
        return {"file": rel_path, "error": str(e), "unsafe_items": []}

    unsafe_items = []
    has_forbid_attr = False

    for idx, raw_line in enumerate(lines):
        line = raw_line.strip()
        if FORBID_UNSAFE_PATTERN.search(line):
            has_forbid_attr = True

        for kind, pat in UNSAFE_PATTERNS.items():
            if pat.search(line):
                # Check for preceding SAFETY comment
                has_safety_comment = False
                # Check previous 5 lines for // SAFETY:
                start_back = max(0, idx - 5)
                for back_idx in range(start_back, idx):
                    if "// SAFETY:" in lines[back_idx] or "/// # Safety" in lines[back_idx] or "/// # Invariant" in lines[back_idx]:
                        has_safety_comment = True
                        break

                unsafe_items.append({
                    "line": idx + 1,
                    "kind": kind,
                    "code": line[:120],
                    "has_safety_comment": has_safety_comment,
                })

    return {
        "file": rel_path,
        "has_forbid_attr": has_forbid_attr,
        "unsafe_items": unsafe_items,
    }

def audit_crate(crate_dir: Path):
    crate_name = crate_dir.name
    src_dir = crate_dir / "src"
    if not src_dir.exists():
        return None

    is_allowed_unsafe = crate_name in ALLOWED_UNSAFE_CRATES

    lib_rs = src_dir / "lib.rs"
    main_rs = src_dir / "main.rs"
    root_forbid_unsafe = False

    root_file = lib_rs if lib_rs.exists() else (main_rs if main_rs.exists() else None)
    if root_file:
        try:
            content = root_file.read_text(encoding="utf-8", errors="ignore")
            if FORBID_UNSAFE_PATTERN.search(content):
                root_forbid_unsafe = True
        except Exception:
            pass

    crate_summary = {
        "crate": crate_name,
        "is_allowed_island": is_allowed_unsafe,
        "root_forbid_unsafe": root_forbid_unsafe,
        "total_unsafe_occurrences": 0,
        "missing_safety_comments": 0,
        "files_with_unsafe": [],
    }

    for rs_file in crate_dir.rglob("*.rs"):
        if "target" in rs_file.parts:
            continue
        res = audit_file(rs_file)
        if res["unsafe_items"]:
            crate_summary["total_unsafe_occurrences"] += len(res["unsafe_items"])
            missing = [item for item in res["unsafe_items"] if not item["has_safety_comment"]]
            crate_summary["missing_safety_comments"] += len(missing)
            crate_summary["files_with_unsafe"].append({
                "file": res["file"],
                "items": res["unsafe_items"]
            })

    return crate_summary

def main():
    parser = argparse.ArgumentParser(description="Unsafe Boundary & Island Auditor")
    parser.add_argument("--workspace", default=".", help="Workspace root")
    parser.add_argument("--crate", default=None, help="Specific crate to check")
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

    all_audits = []
    forbidden_unsafe_count = 0
    missing_safety_count = 0

    for c in target_crates:
        res = audit_crate(c)
        if res:
            all_audits.append(res)
            if not res["is_allowed_island"] and res["total_unsafe_occurrences"] > 0:
                forbidden_unsafe_count += res["total_unsafe_occurrences"]
            missing_safety_count += res["missing_safety_comments"]

    md_path = output_dir / "UNSAFE_BOUNDARY_AUDIT.md"
    json_path = output_dir / "unsafe_boundary_audit.json"

    with open(json_path, "w", encoding="utf-8") as jf:
        json.dump(all_audits, jf, indent=2)

    with open(md_path, "w", encoding="utf-8") as mf:
        mf.write("# Unsafe Boundary & Island Compliance Audit\n\n")
        mf.write(f"> Generiert durch `tools/unsafe_boundary_auditor.py`.\n")
        mf.write(f"> Erlaubte Unsafe-Inseln: `{sorted(list(ALLOWED_UNSAFE_CRATES))}`\n\n")

        mf.write("## Crate Übersicht\n\n")
        mf.write("| Crate | Status | Forbid Attribute | Unsafe Total | Fehlende `// SAFETY:` |\n")
        mf.write("|---|---|---|---|---|\n")
        for a in all_audits:
            status = "Insel (Erlaubt)" if a["is_allowed_island"] else ("REIN" if a["total_unsafe_occurrences"] == 0 else "**VERBOTEN**")
            forbid = "Ja" if a["root_forbid_unsafe"] else "Nein"
            mf.write(f"| `{a['crate']}` | {status} | {forbid} | {a['total_unsafe_occurrences']} | {a['missing_safety_comments']} |\n")

        if forbidden_unsafe_count > 0 or missing_safety_count > 0:
            mf.write("\n## Auffälligkeiten & Details\n\n")
            for a in all_audits:
                if (not a["is_allowed_island"] and a["total_unsafe_occurrences"] > 0) or a["missing_safety_comments"] > 0:
                    mf.write(f"### `{a['crate']}`\n\n")
                    for fw in a["files_with_unsafe"]:
                        for it in fw["items"]:
                            safety_tag = "OK" if it["has_safety_comment"] else "**KEIN SAFETY KOMMENTAR**"
                            mf.write(f"- `{fw['file']}:{it['line']}` [{it['kind']}] ({safety_tag}): `{it['code']}`\n")

    if args.json:
        print(json.dumps(all_audits, indent=2))
    else:
        print(f"Unsafe-Boundary Audit abgeschlossen:")
        print(f" - Geprüfte Crates: {len(all_audits)}")
        print(f" - Unsafe außerhalb von Inseln: {forbidden_unsafe_count}")
        print(f" - Fehlende SAFETY-Dokumentation: {missing_safety_count}")
        print(f" - Markdown: {md_path}")
        print(f" - JSON: {json_path}")

if __name__ == "__main__":
    main()
