#!/usr/bin/env python3
"""
determinism_guard.py - Analyse und Wächter über Nicht-Determinismus in Rust-Kernen (P28).

Verbotene Muster in Kernlogik:
- Instant::now()
- SystemTime::now()
- rand::thread_rng()
- std::collections::HashMap / HashSet ohne festen Hasher (RandomState)
- std::env::var()

Ermöglicht den präzisen Abgleich mit Whitelists und Ausnahmen (governance/determinism-allow.toml).
"""

import os
import sys
import re
import json
import argparse
from pathlib import Path

NON_DETERMINISTIC_PATTERNS = {
    "instant_now": re.compile(r'\bInstant::now\s*\('),
    "system_time_now": re.compile(r'\bSystemTime::now\s*\('),
    "thread_rng": re.compile(r'\b(thread_rng|rand::thread_rng)\s*\('),
    "random_hashmap": re.compile(r'\b(std::collections::)?HashMap\b'),
    "random_hashset": re.compile(r'\b(std::collections::)?HashSet\b'),
    "env_var": re.compile(r'\b(std::env::var|env::var)\s*\('),
}

SAFE_ALTERNATIVES = {
    "instant_now": "contextra_ports::Clock::now_monotonic()",
    "system_time_now": "contextra_ports::Clock::now_wall_clock()",
    "thread_rng": "contextra_ports::RngSeed / deterministischer ChaCha20Rng",
    "random_hashmap": "BTreeMap oder HashMap mit festem BuildHasher",
    "random_hashset": "BTreeSet oder HashSet mit festem BuildHasher",
    "env_var": "Explizite Konfigurationsübergabe (Config-Struct)",
}

def is_test_context(filepath: str) -> bool:
    norm = filepath.replace("\\", "/")
    parts = norm.split("/")
    if "tests" in parts or "benches" in parts or "examples" in parts:
        return True
    if norm.endswith("_test.rs") or norm.endswith("_tests.rs"):
        return True
    return False

def scan_file(filepath: Path):
    rel_path = str(filepath)
    is_test = is_test_context(rel_path)

    findings = []
    try:
        with open(filepath, "r", encoding="utf-8", errors="ignore") as f:
            lines = f.readlines()
    except Exception as e:
        return {"file": rel_path, "error": str(e), "findings": []}

    in_test_mod = False
    for idx, raw_line in enumerate(lines):
        line = raw_line.strip()
        if line.startswith("//"):
            continue
        if "#[cfg(test)]" in line:
            in_test_mod = True

        for kind, pat in NON_DETERMINISTIC_PATTERNS.items():
            if pat.search(line):
                # Check for explicit determinism allow comments
                if "ALLOW-DETERMINISM" in line or "governance/determinism-allow" in line:
                    continue
                # For HashMap/HashSet, skip if explicit BTreeMap or custom hasher is mentioned
                if "BTreeMap" in line or "BTreeSet" in line or "BuildHasher" in line:
                    continue

                findings.append({
                    "line": idx + 1,
                    "kind": kind,
                    "code": line[:120],
                    "in_test": is_test or in_test_mod,
                    "recommendation": SAFE_ALTERNATIVES[kind]
                })

    return {
        "file": rel_path,
        "is_test": is_test,
        "findings": findings
    }

def audit_crate(crate_dir: Path):
    crate_name = crate_dir.name
    findings_prod = []
    findings_test = []

    for rs_file in crate_dir.rglob("*.rs"):
        if "target" in rs_file.parts:
            continue
        res = scan_file(rs_file)
        for f in res["findings"]:
            entry = {
                "file": res["file"],
                "line": f["line"],
                "kind": f["kind"],
                "code": f["code"],
                "recommendation": f["recommendation"]
            }
            if f["in_test"]:
                findings_test.append(entry)
            else:
                findings_prod.append(entry)

    summary_prod = {}
    for kind in NON_DETERMINISTIC_PATTERNS:
        summary_prod[kind] = sum(1 for f in findings_prod if f["kind"] == kind)

    return {
        "crate": crate_name,
        "findings_prod": findings_prod,
        "findings_test": findings_test,
        "summary_prod": summary_prod,
        "total_prod_issues": len(findings_prod)
    }

def main():
    parser = argparse.ArgumentParser(description="Determinism Guard (P28 Invariant Checker)")
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

    audits = []
    total_prod = 0

    for c in target_crates:
        res = audit_crate(c)
        audits.append(res)
        total_prod += res["total_prod_issues"]

    md_path = output_dir / "DETERMINISM_GUARD_AUDIT.md"
    json_path = output_dir / "determinism_guard_audit.json"

    with open(json_path, "w", encoding="utf-8") as jf:
        json.dump(audits, jf, indent=2)

    with open(md_path, "w", encoding="utf-8") as mf:
        mf.write("# Determinismus Audit Report (P28)\n\n")
        mf.write(f"> Generiert durch `tools/determinism_guard.py`.\n")
        mf.write(f"> Untersucht Nicht-Determinismus in Produktivpfaden (Instant, SystemTime, thread_rng, HashMap/HashSet).\n\n")

        mf.write("## Crate Übersicht (Produktivpfade)\n\n")
        mf.write("| Crate | Instant::now | SystemTime::now | thread_rng | Random HashMap/Set | env::var | Gesamt Prod |\n")
        mf.write("|---|---|---|---|---|---|---|\n")
        for a in audits:
            s = a["summary_prod"]
            h_maps = s["random_hashmap"] + s["random_hashset"]
            mf.write(f"| `{a['crate']}` | {s['instant_now']} | {s['system_time_now']} | {s['thread_rng']} | {h_maps} | {s['env_var']} | **{a['total_prod_issues']}** |\n")

        if total_prod > 0:
            mf.write("\n## Detaillierte Befunde mit Handlungsempfehlungen\n\n")
            for a in audits:
                if a["findings_prod"]:
                    mf.write(f"### `{a['crate']}` ({len(a['findings_prod'])} Fundstellen)\n\n")
                    for f in a["findings_prod"]:
                        mf.write(f"- `{f['file']}:{f['line']}` [{f['kind']}]: `{f['code']}`\n")
                        mf.write(f"  - **Empfehlung:** `{f['recommendation']}`\n")

    if args.json:
        print(json.dumps(audits, indent=2))
    else:
        print(f"Determinismus-Guard Audit abgeschlossen:")
        print(f" - Geprüfte Crates: {len(audits)}")
        print(f" - Nicht-deterministische Stellen in Prod: {total_prod}")
        print(f" - Markdown: {md_path}")
        print(f" - JSON: {json_path}")

if __name__ == "__main__":
    main()
