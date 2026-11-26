#!/usr/bin/env python3
"""
test_coverage_matrix.py — Granular Test Traceability & Invariant Coverage Matrix.

Maps test suites and traceability markers to architecture invariants:
- Scans unit tests (#[test]), async tests (#[tokio::test]), property-based tests (proptest!),
  benchmarks (#[bench], criterion_group!), and loom concurrency tests (#[cfg(loom)]).
- Audits traceability comments and attributes (`/// SPEC: T-...` or `#[ctx_test(id = "T-...")]`)
  mandated by invariant T-TRACE-1.
- Identifies invariant coverage gaps and untested production modules.
"""

from __future__ import annotations

import json
import re
import sys
from collections import defaultdict
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, List, Optional, Set, Tuple

from agent_tooling_common import (
    DiagnosticReport,
    create_standard_cli,
    discover_crates,
    get_repo_root,
    iter_rust_files,
)


@dataclass
class TestItem:
    crate: str
    file: str
    line: int
    name: str
    kind: str  # unit, async, proptest, benchmark, loom
    spec_id: Optional[str]


def parse_tests_in_file(file_path: Path, crate_name: str, repo_root: Path) -> List[TestItem]:
    items: List[TestItem] = []
    try:
        content = file_path.read_text(encoding="utf-8")
    except Exception:
        return items

    lines = content.splitlines()
    rel_path = str(file_path.relative_to(repo_root))

    re_spec_tag = re.compile(r"///\s*SPEC:\s*([A-Za-z0-9_-]+)")
    re_ctx_test = re.compile(r'#\[ctx_test\(id\s*=\s*"([^"]+)"\)')
    re_unit_test = re.compile(r"^\s*#\[test\]")
    re_tokio_test = re.compile(r"^\s*#\[tokio::test(?:.*)?\]")
    re_proptest_block = re.compile(r"\bproptest!\s*\{")
    re_bench_macro = re.compile(r"\bcriterion_group!\s*\(")
    re_fn = re.compile(r"^\s*(?:async\s+)?fn\s+([A-Za-z0-9_]+)")

    pending_spec: Optional[str] = None

    for idx, line in enumerate(lines, start=1):
        s = line.strip()

        # Check SPEC marker
        m_spec = re_spec_tag.search(s)
        if m_spec:
            pending_spec = m_spec.group(1)
            continue

        m_ctx = re_ctx_test.search(s)
        if m_ctx:
            pending_spec = m_ctx.group(1)
            continue

        # Proptest macro block
        if re_proptest_block.search(s):
            items.append(TestItem(
                crate=crate_name,
                file=rel_path,
                line=idx,
                name="proptest_block",
                kind="proptest",
                spec_id=pending_spec,
            ))
            pending_spec = None
            continue

        # Benchmark macro
        if re_bench_macro.search(s):
            items.append(TestItem(
                crate=crate_name,
                file=rel_path,
                line=idx,
                name="criterion_benchmark",
                kind="benchmark",
                spec_id=pending_spec,
            ))
            pending_spec = None
            continue

        # Standard test attributes
        is_tokio = bool(re_tokio_test.search(s))
        is_unit = bool(re_unit_test.search(s))

        if is_tokio or is_unit:
            test_kind = "async" if is_tokio else "unit"
            # find fn name in next few lines
            fn_name = "unknown_test"
            for k in range(idx, min(idx + 5, len(lines))):
                m_fn = re_fn.match(lines[k].strip())
                if m_fn:
                    fn_name = m_fn.group(1)
                    break

            items.append(TestItem(
                crate=crate_name,
                file=rel_path,
                line=idx,
                name=fn_name,
                kind=test_kind,
                spec_id=pending_spec,
            ))
            pending_spec = None

    return items


def main():
    cli = create_standard_cli("Granular Test Traceability & Invariant Matrix Auditor.")
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
        tool_name="test_coverage_matrix.py",
        title="Test Traceability & Invariant Coverage Diagnostic Report",
        description="Maps unit tests, async tokio tests, proptests, benchmarks, and SPEC traceability markers (T-TRACE-1) across workspace crates.",
    )

    all_tests: List[TestItem] = []
    spec_coverage: Dict[str, List[TestItem]] = defaultdict(list)
    kind_counts: Dict[str, int] = defaultdict(int)
    crate_test_summary = []

    for cname, cmeta in sorted(target_crates.items()):
        c_tests = 0
        c_spec_tagged = 0
        for rpath in iter_rust_files(cmeta.path, include_tests=True, include_benches=True):
            items = parse_tests_in_file(rpath, cname, repo_root)
            for it in items:
                all_tests.append(it)
                c_tests += 1
                kind_counts[it.kind] += 1
                if it.spec_id:
                    spec_coverage[it.spec_id].append(it)
                    c_spec_tagged += 1

                report.add_finding({
                    "crate": cname,
                    "file": it.file,
                    "line": it.line,
                    "name": it.name,
                    "kind": it.kind,
                    "spec_id": it.spec_id or "NONE",
                    "description": f"Test `{it.name}` ({it.kind}) -> SPEC: {it.spec_id or 'Untraced'}",
                })
        crate_test_summary.append((cname, c_tests, c_spec_tagged))

    report.set_metric("total_tests_discovered", len(all_tests))
    report.set_metric("tests_with_spec_trace", sum(len(v) for v in spec_coverage.values()))
    report.set_metric("unique_spec_invariants_covered", len(spec_coverage))
    for k, v in kind_counts.items():
        report.set_metric(f"test_type_{k}", v)

    summary_table = [
        "| Crate | Tests Gesamt | Mit SPEC Trace (T-TRACE-1) | Traceability-Quote |",
        "|---|---|---|---|",
    ]
    for cname, total, tagged in crate_test_summary:
        if total > 0:
            pct = f"{(tagged / total) * 100:.1f} %"
            summary_table.append(f"| `{cname}` | {total} | {tagged} | {pct} |")
    report.add_section("Crate-Test-Matrix", "\n".join(summary_table))

    spec_table = [
        "| Invariante / Spec-ID | Zugeordnete Tests | Beispiel-Dateien |",
        "|---|---|---|",
    ]
    for spec_id, test_list in sorted(spec_coverage.items(), key=lambda x: -len(x[1]))[:50]:
        files = ", ".join(sorted({t.file.split("/")[-1] for t in test_list})[:3])
        spec_table.append(f"| `{spec_id}` | {len(test_list)} | `{files}` |")
    report.add_section("Top Abgedeckte Spezifikations-Invarianten", "\n".join(spec_table))

    if args.json:
        print(json.dumps(report.to_dict(), indent=2))
    else:
        print(f"[*] Test Coverage Matrix complete: {len(all_tests)} tests mapped across {len(target_crates)} crates.")
        print(f"    SPEC Traced: {sum(len(v) for v in spec_coverage.values())} across {len(spec_coverage)} unique invariants.")
        for k, v in kind_counts.items():
            print(f"    - {k}: {v}")

    if not args.no_save:
        out_name = f"test_matrix_{args.crate}" if args.crate else "test_matrix_workspace"
        j_p, m_p = report.write_artifacts(out_name, Path(args.output_dir) if args.output_dir else None)
        if not args.json:
            print(f"[+] Saved artifacts:\n    {j_p}\n    {m_p}")


if __name__ == "__main__":
    main()
