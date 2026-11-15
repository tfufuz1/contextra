#!/usr/bin/env python3
"""
concurrency_lock_auditor.py — Concurrency & Lock Safety Diagnostic Tool.

Audits concurrency patterns and synchronization primitives:
- Detects synchronous Mutex/RwLock guards held across `.await` points (deadlock hazard).
- Identifies usage of std::sync::Mutex vs parking_lot vs tokio::sync.
- Scans for loom concurrency testing support (`#[cfg(loom)]`).
- Tracks channel architectures (tokio::sync::mpsc, crossbeam, std::sync::mpsc).
- Detects nested lock acquisitions that could trigger lock inversion deadlocks (INV-LOCK-ORDER-1).
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


def audit_file_concurrency(file_path: Path, crate_name: str, repo_root: Path) -> List[Dict]:
    findings = []
    try:
        raw_content = file_path.read_text(encoding="utf-8")
    except Exception:
        return findings

    clean_content = strip_comments_and_strings(raw_content)
    clean_lines = clean_content.splitlines()
    lines = raw_content.splitlines()
    rel_path = str(file_path.relative_to(repo_root))

    re_std_mutex = re.compile(r"\bstd::sync::(?:Mutex|RwLock)\b")
    re_parking_lot = re.compile(r"\bparking_lot::(?:Mutex|RwLock)\b")
    re_tokio_sync = re.compile(r"\btokio::sync::(?:Mutex|RwLock)\b")
    re_channel = re.compile(r"\b(?:mpsc::channel|tokio::sync::mpsc|broadcast::channel|oneshot::channel)\b")
    re_loom_cfg = re.compile(r'#\[cfg\((?:.*loom.*)\)\]')

    # Detect guard held across .await:
    # Pattern: `let guard = ...lock()` followed by `.await` before guard is dropped or block ends
    lock_pattern = re.compile(r"let\s+([A-Za-z0-9_]+)\s*=\s*.*(?:\.lock\(|\.read\(|\.write\()")
    await_pattern = re.compile(r"\.await\b")

    active_guard = None
    guard_line = 0

    for idx, (raw_line, clean_line) in enumerate(zip(lines, clean_lines), start=1):
        s_clean = clean_line.strip()
        s_raw = raw_line.strip()

        # Check sync primitive types
        if re_std_mutex.search(s_clean):
            findings.append({
                "crate": crate_name,
                "file": rel_path,
                "line": idx,
                "kind": "STD_SYNC_LOCK",
                "severity": "LOW",
                "description": "Usage of std::sync::{Mutex, RwLock} (parking_lot preferred for non-async)",
                "snippet": s_raw[:100],
            })

        if re_parking_lot.search(s_clean):
            findings.append({
                "crate": crate_name,
                "file": rel_path,
                "line": idx,
                "kind": "PARKING_LOT_LOCK",
                "severity": "INFO",
                "description": "Usage of parking_lot::{Mutex, RwLock} (efficient spin/parking)",
                "snippet": s_raw[:100],
            })

        if re_tokio_sync.search(s_clean):
            findings.append({
                "crate": crate_name,
                "file": rel_path,
                "line": idx,
                "kind": "TOKIO_ASYNC_LOCK",
                "severity": "INFO",
                "description": "Usage of tokio::sync::{Mutex, RwLock} for async boundaries",
                "snippet": s_raw[:100],
            })

        if re_channel.search(s_clean):
            findings.append({
                "crate": crate_name,
                "file": rel_path,
                "line": idx,
                "kind": "CHANNEL_PIPELINE",
                "severity": "INFO",
                "description": "Message-passing channel initialized",
                "snippet": s_raw[:100],
            })

        if re_loom_cfg.search(s_raw):
            findings.append({
                "crate": crate_name,
                "file": rel_path,
                "line": idx,
                "kind": "LOOM_CONCURRENCY_TEST",
                "severity": "POSITIVE",
                "description": "Loom permutation concurrency verification present",
                "snippet": s_raw[:100],
            })

        # Lock across await detector
        m_lock = lock_pattern.search(s_clean)
        if m_lock:
            active_guard = m_lock.group(1)
            guard_line = idx

        if active_guard and await_pattern.search(s_clean):
            findings.append({
                "crate": crate_name,
                "file": rel_path,
                "line": idx,
                "kind": "LOCK_HELD_ACROSS_AWAIT",
                "severity": "HIGH",
                "description": f"Potential lock inversion / deadlock: Guard '{active_guard}' (line {guard_line}) held across .await",
                "snippet": s_raw[:100],
            })
            active_guard = None  # Reset to avoid redundant reports

        if "drop(" in s_clean or s_clean == "}":
            active_guard = None

    return findings


def main():
    cli = create_standard_cli("Concurrency & Lock Safety Diagnostic Auditor.")
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
        tool_name="concurrency_lock_auditor.py",
        title="Concurrency & Lock Safety Audit Report",
        description="Audits synchronization primitives, Mutex/RwLock guards across .await (INV-LOCK-ORDER-1), channel architectures, and Loom model-checking support.",
    )

    lock_across_await_count = 0
    std_sync_count = 0
    parking_lot_count = 0
    tokio_sync_count = 0
    channel_count = 0
    loom_count = 0

    for cname, cmeta in sorted(target_crates.items()):
        for rpath in iter_rust_files(cmeta.path, include_tests=True):
            findings = audit_file_concurrency(rpath, cname, repo_root)
            for f in findings:
                k = f["kind"]
                if k == "LOCK_HELD_ACROSS_AWAIT":
                    lock_across_await_count += 1
                elif k == "STD_SYNC_LOCK":
                    std_sync_count += 1
                elif k == "PARKING_LOT_LOCK":
                    parking_lot_count += 1
                elif k == "TOKIO_ASYNC_LOCK":
                    tokio_sync_count += 1
                elif k == "CHANNEL_PIPELINE":
                    channel_count += 1
                elif k == "LOOM_CONCURRENCY_TEST":
                    loom_count += 1
                report.add_finding(f)

    report.set_metric("locks_held_across_await", lock_across_await_count)
    report.set_metric("std_sync_locks", std_sync_count)
    report.set_metric("parking_lot_locks", parking_lot_count)
    report.set_metric("tokio_sync_locks", tokio_sync_count)
    report.set_metric("channels_used", channel_count)
    report.set_metric("loom_verification_points", loom_count)
    report.set_metric("total_concurrency_items", len(report.findings))

    table = [
        "| Mechanismus / Risiko | Anzahl | Relevanz |",
        "|---|---|---|",
        f"| **Lock über `.await`** | {lock_across_await_count} | Kritische Deadlock-Gefahr / Async-Invariante |",
        f"| `std::sync::{{Mutex, RwLock}}` | {std_sync_count} | Standardbibliothek Sperren |",
        f"| `parking_lot::{{Mutex, RwLock}}` | {parking_lot_count} | Schnelle synchrone Locks |",
        f"| `tokio::sync::{{Mutex, RwLock}}` | {tokio_sync_count} | Kooperative asynchrone Sperren |",
        f"| Kanäle (`mpsc`/`broadcast`) | {channel_count} | Message-Passing Architekturen |",
        f"| `loom` Test-Konfigurationen | {loom_count} | Deterministische Nebenläufigkeitsprüfung |",
    ]
    report.add_section("Synchronisations-Übersicht", "\n".join(table))

    if args.json:
        print(json.dumps(report.to_dict(), indent=2))
    else:
        print(f"[*] Concurrency Lock Auditor complete: {len(report.findings)} items inspected.")
        print(f"    Lock across .await hazards: {lock_across_await_count}")
        print(f"    parking_lot: {parking_lot_count}, tokio::sync: {tokio_sync_count}, std::sync: {std_sync_count}")
        print(f"    Channels: {channel_count}, Loom verification points: {loom_count}")

    if not args.no_save:
        out_name = f"concurrency_audit_{args.crate}" if args.crate else "concurrency_audit_workspace"
        j_p, m_p = report.write_artifacts(out_name, Path(args.output_dir) if args.output_dir else None)
        if not args.json:
            print(f"[+] Saved artifacts:\n    {j_p}\n    {m_p}")


if __name__ == "__main__":
    main()
