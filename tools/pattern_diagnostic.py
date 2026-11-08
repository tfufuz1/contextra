#!/usr/bin/env python3
"""
pattern_diagnostic.py — Anti-Pattern and Invariant Violation Scanner.

Scans the codebase for critical architecture invariants and anti-patterns:
- Zero-Panic policy violations: unwrap(), expect(), panic!(), todo!(), unimplemented!().
- Non-determinism violations: Instant::now, SystemTime::now, thread_rng, RandomState.
- Direct memory-mapping forbidden patterns: mmap, memmap2.
- Direct I/O violations: bypass of Vfs, std::fs in core persistency.
- Forbidden marketing and deletion terms per SPEC §6.4 and T-LINT-1/3:
  'garantiert', 'guaranteed', '100 %', 'DSGVO-konform', 'GDPR-compliant',
  'rechtskonform', 'unwiederbringlich', 'irrecoverable', 'unwiderruflich', etc.
"""

from __future__ import annotations

import json
import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, List, Optional, Tuple

from agent_tooling_common import (
    DiagnosticReport,
    create_standard_cli,
    discover_crates,
    get_repo_root,
    iter_rust_files,
    strip_comments_and_strings,
)

FORBIDDEN_WORDS = [
    (r"\bgarantiert\b", "Forbidden wording (§6.4): 'garantiert'"),
    (r"\bguaranteed\b", "Forbidden wording (§6.4): 'guaranteed'"),
    (r"\b100\s*%", "Forbidden wording (§6.4): '100 %'"),
    (r"\bDSGVO-konform\b", "Forbidden wording (§6.4): 'DSGVO-konform'"),
    (r"\bGDPR-compliant\b", "Forbidden wording (§6.4): 'GDPR-compliant'"),
    (r"\brechtskonform\b", "Forbidden wording (§6.4): 'rechtskonform'"),
    (r"\bunwiederbringlich\b", "Forbidden wording (§6.4): 'unwiederbringlich'"),
    (r"\birrecoverable\b", "Forbidden wording (§6.4): 'irrecoverable'"),
    (r"\bunwiderruflich\b", "Forbidden wording (§6.4): 'unwiderruflich'"),
    (r"\binformationstheoretisch\b", "Forbidden wording (§6.4): 'informationstheoretisch'"),
    (r"\bzertifiziert\b", "Forbidden wording (§6.4): 'zertifiziert'"),
    (r"\bcertified\b", "Forbidden wording (§6.4): 'certified'"),
    (r"\bvon\s+Aufsichtsbeh[öo]rde\s+gepr[üu]ft\b", "Forbidden wording (§6.4): 'von Aufsichtsbehörde geprüft'"),
]

CODE_PATTERNS = [
    (r"\.unwrap\(\)", "ZERO_PANIC", "Call to .unwrap() in code"),
    (r"\.expect\(", "ZERO_PANIC", "Call to .expect() in code"),
    (r"\bpanic!\(", "ZERO_PANIC", "Invocation of panic!() macro"),
    (r"\btodo!\(", "ZERO_PANIC", "Invocation of todo!() macro"),
    (r"\bunimplemented!\(", "ZERO_PANIC", "Invocation of unimplemented!() macro"),
    (r"\bInstant::now\(", "DETERMINISM", "Instant::now() violates P28 (use Clock port)"),
    (r"\bSystemTime::now\(", "DETERMINISM", "SystemTime::now() violates P28 (use Clock port)"),
    (r"\bthread_rng\(", "DETERMINISM", "thread_rng() violates P28 (use Rng port)"),
    (r"\bRandomState\b", "DETERMINISM", "RandomState violates P28 (use AHasher/BTreeMap)"),
    (r"\bmmap\b|\bMmap\b|\bmemmap2\b", "MEMORY_MAPPING", "mmap usage violates D-94"),
    (r"\bstd::fs::", "DIRECT_IO", "std::fs usage bypasses Vfs port"),
]


def check_file(file_path: Path, crate_name: str, repo_root: Path, is_test_file: bool) -> List[Dict]:
    findings = []
    try:
        content = file_path.read_text(encoding="utf-8")
    except Exception:
        return findings

    lines = content.splitlines()
    clean_content = strip_comments_and_strings(content)
    clean_lines = clean_content.splitlines()

    rel_path = str(file_path.relative_to(repo_root))

    # 1. Check forbidden wording in raw text (including doc comments and error strings)
    # Ignore AVV generator legal agreements where the law is cited
    if "avv-generator" not in crate_name:
        for idx, line in enumerate(lines, start=1):
            for pattern, desc in FORBIDDEN_WORDS:
                if re.search(pattern, line, re.IGNORECASE):
                    findings.append({
                        "crate": crate_name,
                        "file": rel_path,
                        "line": idx,
                        "category": "FORBIDDEN_WORDING",
                        "severity": "HIGH",
                        "description": desc,
                        "snippet": line.strip()[:100],
                        "in_test": is_test_file,
                    })

    # 2. Check code anti-patterns in stripped code
    for idx, line in enumerate(clean_lines, start=1):
        if not line.strip():
            continue
        for pattern, cat, desc in CODE_PATTERNS:
            if re.search(pattern, line):
                # In tests, unwrap/expect are often tolerated, flag as LOW if in_test
                sev = "LOW" if (is_test_file or "tests" in rel_path) else ("HIGH" if cat == "ZERO_PANIC" else "MEDIUM")
                findings.append({
                    "crate": crate_name,
                    "file": rel_path,
                    "line": idx,
                    "category": cat,
                    "severity": sev,
                    "description": desc,
                    "snippet": line.strip()[:100],
                    "in_test": is_test_file,
                })

    return findings


def main():
    cli = create_standard_cli("Anti-Pattern & Invariant Violation Diagnostic Scanner.")
    cli.add_argument("--category", choices=["ALL", "ZERO_PANIC", "DETERMINISM", "MEMORY_MAPPING", "DIRECT_IO", "FORBIDDEN_WORDING"], default="ALL", help="Filter by diagnostic category.")
    cli.add_argument("--prod-only", action="store_true", help="Only scan production source code (exclude tests).")
    args = cli.parse_args()

    repo_root = get_repo_root()
    crates = discover_crates(repo_root)

    target_crates = {}
    if args.crate:
        if args.crate in crates:
            target_crates[args.crate] = crates[args.crate]
        else:
            print(f"Error: Crate '{args.crate}' not found.", file=sys.stderr)
            sys.exit(1)
    else:
        target_crates = crates

    report = DiagnosticReport(
        tool_name="pattern_diagnostic.py",
        title="Anti-Pattern & Invariant Violation Diagnostic Report",
        description="Comprehensive audit of Zero-Panic (unwrap/expect), Determinism (Clock/Rng/Hash), Memory Mapping (mmap), Direct I/O, and §6.4 Forbidden Wording invariants.",
    )

    cat_counts = {
        "ZERO_PANIC": 0,
        "DETERMINISM": 0,
        "MEMORY_MAPPING": 0,
        "DIRECT_IO": 0,
        "FORBIDDEN_WORDING": 0,
    }
    prod_violations = 0
    test_violations = 0

    for cname, cmeta in sorted(target_crates.items()):
        # Scan src
        if cmeta.src_path.is_dir():
            for rpath in iter_rust_files(cmeta.src_path, include_tests=False):
                # Is it a unit test module inside src?
                rel = str(rpath.relative_to(cmeta.path))
                is_unit_test = "test" in rel.lower()
                if args.prod_only and is_unit_test:
                    continue
                file_findings = check_file(rpath, cname, repo_root, is_test_file=is_unit_test)
                for f in file_findings:
                    if args.category != "ALL" and f["category"] != args.category:
                        continue
                    cat_counts[f["category"]] = cat_counts.get(f["category"], 0) + 1
                    if f["in_test"]:
                        test_violations += 1
                    else:
                        prod_violations += 1
                    report.add_finding(f)

        # Scan tests if not prod_only
        if not args.prod_only and cmeta.tests_path.is_dir():
            for rpath in iter_rust_files(cmeta.tests_path, include_tests=True):
                file_findings = check_file(rpath, cname, repo_root, is_test_file=True)
                for f in file_findings:
                    if args.category != "ALL" and f["category"] != args.category:
                        continue
                    cat_counts[f["category"]] = cat_counts.get(f["category"], 0) + 1
                    test_violations += 1
                    report.add_finding(f)

    report.set_metric("total_findings", len(report.findings))
    report.set_metric("production_violations", prod_violations)
    report.set_metric("test_violations", test_violations)
    for cat, cnt in cat_counts.items():
        report.set_metric(f"category_{cat.lower()}", cnt)

    cat_summary = [
        "| Kategorie | Funde Gesamt | Beschreibung |",
        "|---|---|---|",
        f"| `ZERO_PANIC` | {cat_counts['ZERO_PANIC']} | `unwrap()`, `expect()`, `panic!()` in Code |",
        f"| `DETERMINISM` | {cat_counts['DETERMINISM']} | Zeit/Zufall/Hasher ohne Ports (P28) |",
        f"| `MEMORY_MAPPING` | {cat_counts['MEMORY_MAPPING']} | Verbotene mmap-Nutzung (D-94) |",
        f"| `DIRECT_IO` | {cat_counts['DIRECT_IO']} | Direkte std::fs Umgehung von Vfs |",
        f"| `FORBIDDEN_WORDING` | {cat_counts['FORBIDDEN_WORDING']} | Garantieversprechen / DSGVO-Labels (§6.4) |",
    ]
    report.add_section("Kategorien-Übersicht", "\n".join(cat_summary))

    if args.json:
        print(json.dumps(report.to_dict(), indent=2))
    else:
        print(f"[*] Pattern Diagnostic scan complete: {len(report.findings)} findings (Prod: {prod_violations}, Test: {test_violations})")
        for cat, cnt in cat_counts.items():
            print(f"    - {cat}: {cnt}")

    if not args.no_save:
        out_name = f"pattern_diagnostic_{args.crate}" if args.crate else "pattern_diagnostic_workspace"
        j_p, m_p = report.write_artifacts(out_name, Path(args.output_dir) if args.output_dir else None)
        if not args.json:
            print(f"[+] Saved artifacts:\n    {j_p}\n    {m_p}")


if __name__ == "__main__":
    main()
