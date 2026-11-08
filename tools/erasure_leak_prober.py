#!/usr/bin/env python3
"""
erasure_leak_prober.py — Micro-granular Deletion & GDPR Leak Prober.

Enforces the strict separation between mechanical data purgers and the erasure domain (M7 / PLAN Phase 3):
- Data-holding storage kernels (store, vector, text, graph, kvcache, checkpoint) must NOT contain:
  - GDPR/DSGVO references or legal terminology.
  - Saga orchestration, DeletionProof creation, or Attestation signing.
  - ErasureJournal, Ledger, or Audit-reporting formats.
- All high-level erasure orchestration belongs exclusively in `contextra-erasure`.
- Scans both source code and comments for unauthorized domain bleeding.
"""

from __future__ import annotations

import json
import re
import sys
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

# Crates allowed to contain erasure domain logic
ALLOWED_ERASURE_CRATES = {"contextra-erasure", "contextra-ports", "contextra-types", "contextra-avv-generator"}

LEAK_PATTERNS = [
    (r"\bDeletionProof\b", "LEAK_DELETION_PROOF", "Usage of DeletionProof in data kernel"),
    (r"\bSagaRecord\b|\bErasureSaga\b", "LEAK_SAGA", "Saga orchestration in data kernel"),
    (r"\bErasureLedger\b", "LEAK_LEDGER", "Erasure ledger reference in data kernel"),
    (r"\bDSGVO\b|\bGDPR\b", "LEAK_GDPR_TERMS", "GDPR/DSGVO legal reference in data kernel"),
    (r"\bkv_shredding\b", "LEAK_KV_SHREDDING", "Legacy kv_shredding reference (DEBT-01)"),
    (r"\brevocation_log\b", "LEAK_REVOCATION_LOG", "Legacy revocation_log reference (DEBT-01)"),
    (r"\bProofKeyProvider\b", "LEAK_PROOF_KEY", "Proof key provider reference in data kernel"),
]


def audit_erasure_leaks(file_path: Path, crate_name: str, repo_root: Path) -> List[Dict]:
    findings = []
    try:
        content = file_path.read_text(encoding="utf-8")
    except Exception:
        return findings

    lines = content.splitlines()
    rel_path = str(file_path.relative_to(repo_root))

    for idx, line in enumerate(lines, start=1):
        for pattern, kind, desc in LEAK_PATTERNS:
            if re.search(pattern, line):
                findings.append({
                    "crate": crate_name,
                    "file": rel_path,
                    "line": idx,
                    "kind": kind,
                    "severity": "CRITICAL" if crate_name not in ALLOWED_ERASURE_CRATES else "INFO",
                    "description": f"{desc} (Rule M7)",
                    "snippet": line.strip()[:100],
                })
    return findings


def main():
    cli = create_standard_cli("Deletion & GDPR Domain Leak Prober.")
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
        tool_name="erasure_leak_prober.py",
        title="Deletion & GDPR Domain Leak Diagnostic Report",
        description="Audits data-holding storage kernels against rule M7 (no GDPR, Saga, DeletionProof, or report logic outside contextra-erasure).",
    )

    leaks_by_crate: Dict[str, int] = {}
    kernel_violations = 0
    allowed_mentions = 0

    for cname, cmeta in sorted(target_crates.items()):
        c_leaks = 0
        for rpath in iter_rust_files(cmeta.path, include_tests=False):
            findings = audit_erasure_leaks(rpath, cname, repo_root)
            for f in findings:
                if cname not in ALLOWED_ERASURE_CRATES:
                    kernel_violations += 1
                else:
                    allowed_mentions += 1

                c_leaks += 1
                report.add_finding(f)

        if c_leaks > 0:
            leaks_by_crate[cname] = c_leaks

    report.set_metric("kernel_violations", kernel_violations)
    report.set_metric("allowed_mentions", allowed_mentions)
    report.set_metric("crates_with_leaks", len(leaks_by_crate))

    table = [
        "| Crate | Schicht/Rolle | Status | Löschdomänen-Bezüge |",
        "|---|---|---|---|",
    ]
    for cname, cnt in leaks_by_crate.items():
        is_allowed = cname in ALLOWED_ERASURE_CRATES
        status = "Zulässig (Löschdomäne)" if is_allowed else "**VERLETZUNG (M7)**"
        table.append(f"| `{cname}` | {crates[cname].role or 'Kernel'} | {status} | {cnt} |")
    report.add_section("Löschketten-Entkopplungs-Status", "\n".join(table))

    if args.json:
        print(json.dumps(report.to_dict(), indent=2))
    else:
        print(f"[*] Erasure Leak Prober complete: {kernel_violations} kernel domain violations found.")
        print(f"    Crates with leaks: {len(leaks_by_crate)} (Allowed mentions in erasure crates: {allowed_mentions})")

    if not args.no_save:
        out_name = f"erasure_leaks_{args.crate}" if args.crate else "erasure_leaks_workspace"
        j_p, m_p = report.write_artifacts(out_name, Path(args.output_dir) if args.output_dir else None)
        if not args.json:
            print(f"[+] Saved artifacts:\n    {j_p}\n    {m_p}")


if __name__ == "__main__":
    main()
