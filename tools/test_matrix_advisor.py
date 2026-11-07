#!/usr/bin/env python3
"""
test_matrix_advisor.py - Analyse der Testabdeckung nach Pyramidenstufen (AGENTS.md §8).

Stufen der Testpyramide:
1. Unit Tests (feingranular, in src/ oder tests/)
2. Property-Based Tests (proptest, quickcheck)
3. Concurrency / Chaos Tests (loom, FaultVfs)
4. Mutation & Fuzz Targets (cargo-mutants, fuzz/)
5. Performance Benchmarks (benches/, criterion)

Gibt Coding-Agenten präzise Empfehlungen, welche Teststufe pro Crate gestärkt werden muss.
"""

import os
import sys
import re
import json
import argparse
from pathlib import Path

PROPTEST_PATTERN = re.compile(r'\bproptest!\b|use\s+proptest\b')
LOOM_PATTERN = re.compile(r'\bloom::\b|#\[cfg\(loom\)\]')
FAULT_VFS_PATTERN = re.compile(r'\bFaultVfs\b')
BENCH_PATTERN = re.compile(r'\b(criterion|criterion_group|criterion_main)\b|#\[bench\]')

def analyze_crate_tests(crate_dir: Path):
    crate_name = crate_dir.name

    cargo_toml = crate_dir / "Cargo.toml"
    has_proptest_dep = False
    has_loom_dep = False
    has_criterion_dep = False

    if cargo_toml.exists():
        try:
            c_text = cargo_toml.read_text(encoding="utf-8", errors="ignore")
            has_proptest_dep = "proptest" in c_text
            has_loom_dep = "loom" in c_text
            has_criterion_dep = "criterion" in c_text
        except Exception:
            pass

    unit_test_count = 0
    proptest_count = 0
    loom_test_count = 0
    fault_vfs_count = 0
    bench_count = 0
    has_benches_dir = (crate_dir / "benches").exists()
    has_fuzz_dir = (crate_dir / "fuzz").exists()

    for rs_file in crate_dir.rglob("*.rs"):
        if "target" in rs_file.parts:
            continue
        try:
            content = rs_file.read_text(encoding="utf-8", errors="ignore")
        except Exception:
            continue

        unit_test_count += len(re.findall(r'#\[test\]|#\[tokio::test\]|#\[ctx_test\b', content))
        proptest_count += len(PROPTEST_PATTERN.findall(content))
        loom_test_count += len(LOOM_PATTERN.findall(content))
        fault_vfs_count += len(FAULT_VFS_PATTERN.findall(content))
        bench_count += len(BENCH_PATTERN.findall(content))

    recommendations = []
    if unit_test_count == 0:
        recommendations.append("Dringend Basis-Unit-Tests anlegen")
    if not has_proptest_dep and proptest_count == 0:
        recommendations.append("Property-Tests mit proptest ergänzen")
    if ("store" in crate_name or "mvcc" in crate_name or "durable" in crate_name) and loom_test_count == 0:
        recommendations.append("Kritischer Concurrency-Crate: Loom-Tests erforderlich")
    if ("store" in crate_name or "vector" in crate_name) and fault_vfs_count == 0:
        recommendations.append("Chaos-Tests mit FaultVfs zur Crash-Validierung anlegen")
    if not has_benches_dir and bench_count == 0:
        recommendations.append("Criterion-Benchmarks für Hot Paths anlegen")

    return {
        "crate": crate_name,
        "unit_tests": unit_test_count,
        "proptests": proptest_count,
        "loom_checks": loom_test_count,
        "fault_vfs_tests": fault_vfs_count,
        "benches": bench_count,
        "has_benches_dir": has_benches_dir,
        "has_fuzz_dir": has_fuzz_dir,
        "recommendations": recommendations
    }

def main():
    parser = argparse.ArgumentParser(description="Test Matrix & Pyramid Advisor")
    parser.add_argument("--workspace", default=".", help="Workspace root")
    parser.add_argument("--json", action="store_true", help="Output JSON directly to stdout")
    parser.add_argument("--output-dir", default="docs/diagnostics", help="Output directory for reports")
    args = parser.parse_args()

    workspace_root = Path(args.workspace).resolve()
    crates_dir = workspace_root / "crates"
    output_dir = workspace_root / args.output_dir
    output_dir.mkdir(parents=True, exist_ok=True)

    crates = sorted([p for p in crates_dir.iterdir() if p.is_dir() and (p / "Cargo.toml").exists()])
    results = [analyze_crate_tests(c) for c in crates]

    md_path = output_dir / "TEST_MATRIX_ADVISOR.md"
    json_path = output_dir / "test_matrix_advisor.json"

    with open(json_path, "w", encoding="utf-8") as jf:
        json.dump(results, jf, indent=2)

    with open(md_path, "w", encoding="utf-8") as mf:
        mf.write("# Test-Matrix & Pyramiden-Audit (AGENTS.md §8)\n\n")
        mf.write(f"> Generiert durch `tools/test_matrix_advisor.py`.\n")
        mf.write(f"> Untersucht Test-Stufen: Unit -> Proptest -> Concurrency (Loom) -> Chaos (FaultVfs) -> Benchmarks.\n\n")

        mf.write("## Crate Testpyramiden-Übersicht\n\n")
        mf.write("| Crate | Unit-Tests | Proptests | Loom Checks | FaultVfs | Benchmarks | Empfehlungen |\n")
        mf.write("|---|---|---|---|---|---|---|\n")
        for r in results:
            rec_str = "; ".join(r["recommendations"]) if r["recommendations"] else "Vollständig aufgestellt"
            mf.write(f"| `{r['crate']}` | {r['unit_tests']} | {r['proptests']} | {r['loom_checks']} | {r['fault_vfs_tests']} | {r['benches']} | {rec_str} |\n")

        mf.write("\n## Handlungsempfehlungen für Coding-Agenten\n\n")
        for r in results:
            if r["recommendations"]:
                mf.write(f"### `{r['crate']}`\n\n")
                for rec in r["recommendations"]:
                    mf.write(f"- [ ] {rec}\n")

    if args.json:
        print(json.dumps(results, indent=2))
    else:
        print(f"Test Matrix Advisor abgeschlossen:")
        print(f" - Geprüfte Crates: {len(results)}")
        print(f" - Markdown: {md_path}")
        print(f" - JSON: {json_path}")

if __name__ == "__main__":
    main()
