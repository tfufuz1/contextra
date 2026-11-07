#!/usr/bin/env python3
"""
unsafe_boundary_tracer.py — Unsafe Boundary & Proof Tracer.

Detailed diagnostic of unsafe boundaries across the Rust workspace:
- Scans all `unsafe` blocks, functions, and traits.
- Checks whether an explicit `// SAFETY:` explanation accompanies each `unsafe` usage.
- Traces raw pointer operations (`*const`, `*mut`, `as_ptr`, `from_raw_parts`).
- Traces foreign function interface (FFI) bindings (`extern "C"`, libc calls).
- Enforces Unsafe Island governance (only `contextra-simd`, `contextra-sys`, `contextra-wire`
  may contain unsafe; all other crates must enforce `#![forbid(unsafe_code)]`).
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

ALLOWED_UNSAFE_CRATES = {"contextra-simd", "contextra-sys", "contextra-wire"}


def audit_file_unsafe(file_path: Path, crate_name: str, repo_root: Path) -> Tuple[List[Dict], bool]:
    findings = []
    try:
        raw_content = file_path.read_text(encoding="utf-8")
    except Exception:
        return findings, False

    lines = raw_content.splitlines()
    clean_content = strip_comments_and_strings(raw_content)
    clean_lines = clean_content.splitlines()
    rel_path = str(file_path.relative_to(repo_root))

    has_forbid_unsafe = False
    for line in lines[:20]:
        if "forbid(unsafe_code)" in line:
            has_forbid_unsafe = True
            break

    re_unsafe = re.compile(r"\bunsafe(?:\s+(?:fn|trait|impl))?\b")
    re_ptr_cast = re.compile(r"as\s+\*(?:const|mut)\s+")
    re_from_raw = re.compile(r"\b(?:from_raw_parts|from_raw_parts_mut)\b")
    re_ffi = re.compile(r'extern\s+"(?:C|system)"')

    for idx, (raw_line, clean_line) in enumerate(zip(lines, clean_lines), start=1):
        # 1. Unsafe blocks/functions
        if re_unsafe.search(clean_line):
            # Check preceding 5 lines for // SAFETY:
            has_safety_proof = False
            start_check = max(0, idx - 6)
            for prev_idx in range(start_check, idx - 1):
                if prev_idx < len(lines) and "// SAFETY:" in lines[prev_idx]:
                    has_safety_proof = True
                    break

            is_island = crate_name in ALLOWED_UNSAFE_CRATES
            severity = "LOW" if (is_island and has_safety_proof) else ("MEDIUM" if is_island else "CRITICAL")

            desc = "Unsafe block/item without // SAFETY: proof" if not has_safety_proof else "Unsafe block/item with valid // SAFETY: proof"
            if not is_island:
                desc = f"VIOLATION: Unsafe in non-island crate '{crate_name}' (M9 rule)"

            findings.append({
                "crate": crate_name,
                "file": rel_path,
                "line": idx,
                "kind": "UNSAFE_BLOCK",
                "severity": severity,
                "has_safety_proof": has_safety_proof,
                "is_allowed_island": is_island,
                "description": desc,
                "snippet": raw_line.strip()[:100],
            })

        # 2. Raw pointer casts
        if re_ptr_cast.search(clean_line):
            findings.append({
                "crate": crate_name,
                "file": rel_path,
                "line": idx,
                "kind": "RAW_POINTER_CAST",
                "severity": "LOW" if crate_name in ALLOWED_UNSAFE_CRATES else "HIGH",
                "has_safety_proof": False,
                "is_allowed_island": crate_name in ALLOWED_UNSAFE_CRATES,
                "description": "Raw pointer cast (`as *const / as *mut`)",
                "snippet": raw_line.strip()[:100],
            })

        # 3. from_raw_parts
        if re_from_raw.search(clean_line):
            findings.append({
                "crate": crate_name,
                "file": rel_path,
                "line": idx,
                "kind": "SLICE_FROM_RAW_PARTS",
                "severity": "MEDIUM" if crate_name in ALLOWED_UNSAFE_CRATES else "CRITICAL",
                "has_safety_proof": False,
                "is_allowed_island": crate_name in ALLOWED_UNSAFE_CRATES,
                "description": "Slice creation from raw parts (`from_raw_parts`)",
                "snippet": raw_line.strip()[:100],
            })

        # 4. FFI extern declarations
        if re_ffi.search(clean_line):
            findings.append({
                "crate": crate_name,
                "file": rel_path,
                "line": idx,
                "kind": "FFI_EXTERN",
                "severity": "LOW" if crate_name in ALLOWED_UNSAFE_CRATES else "CRITICAL",
                "has_safety_proof": False,
                "is_allowed_island": crate_name in ALLOWED_UNSAFE_CRATES,
                "description": "FFI external declaration (`extern \"C\"`)",
                "snippet": raw_line.strip()[:100],
            })

    return findings, has_forbid_unsafe


def main():
    cli = create_standard_cli("Unsafe Boundary & Proof Tracer across crates.")
    cli.add_argument("--missing-proofs-only", action="store_true", help="Only report unsafe instances lacking // SAFETY: proofs.")
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
        tool_name="unsafe_boundary_tracer.py",
        title="Unsafe Boundary & Safety Proof Audit Report",
        description="Comprehensive audit of unsafe code blocks, // SAFETY: proofs, pointer casts, FFI externs, and Unsafe Island adherence (M9 contract).",
    )

    total_unsafe_blocks = 0
    missing_safety_proofs = 0
    non_island_violations = 0
    ptr_ops_count = 0

    crate_audit = []

    for cname, cmeta in sorted(target_crates.items()):
        is_island = cname in ALLOWED_UNSAFE_CRATES
        crate_unsafe = 0
        crate_missing_proof = 0
        has_crate_forbid = False

        lib_rs = cmeta.src_path / "lib.rs"
        main_rs = cmeta.src_path / "main.rs"
        root_rs = lib_rs if lib_rs.is_file() else (main_rs if main_rs.is_file() else None)
        if root_rs:
            try:
                txt = root_rs.read_text(encoding="utf-8")
                if "forbid(unsafe_code)" in txt:
                    has_crate_forbid = True
            except Exception:
                pass

        for rpath in iter_rust_files(cmeta.path, include_tests=False):
            findings, _ = audit_file_unsafe(rpath, cname, repo_root)
            for f in findings:
                if f["kind"] == "UNSAFE_BLOCK":
                    total_unsafe_blocks += 1
                    crate_unsafe += 1
                    if not f["has_safety_proof"]:
                        missing_safety_proofs += 1
                        crate_missing_proof += 1
                    if not is_island:
                        non_island_violations += 1
                elif f["kind"] in ("RAW_POINTER_CAST", "SLICE_FROM_RAW_PARTS"):
                    ptr_ops_count += 1

                if args.missing_proofs_only and f["kind"] == "UNSAFE_BLOCK" and f["has_safety_proof"]:
                    continue

                report.add_finding(f)

        crate_audit.append((cname, is_island, has_crate_forbid, crate_unsafe, crate_missing_proof))

    report.set_metric("total_unsafe_blocks", total_unsafe_blocks)
    report.set_metric("missing_safety_proofs", missing_safety_proofs)
    report.set_metric("non_island_violations", non_island_violations)
    report.set_metric("raw_pointer_operations", ptr_ops_count)

    audit_table = [
        "| Crate | Unsafe-Insel? | `#![forbid(unsafe_code)]` | Unsafe Blöcke | Fehlende SAFETY-Beweise |",
        "|---|---|---|---|---|",
    ]
    for cname, is_isl, has_forbid, c_unsafe, c_miss in crate_audit:
        if c_unsafe > 0 or not has_forbid or is_isl:
            isl_str = "JA (Erlaubt)" if is_isl else "NEIN"
            forbid_str = "Aktiv" if has_forbid else "**FEHLT**"
            audit_table.append(f"| `{cname}` | {isl_str} | {forbid_str} | {c_unsafe} | {c_miss} |")
    report.add_section("Crate-Sicherheits-Status", "\n".join(audit_table))

    if args.json:
        print(json.dumps(report.to_dict(), indent=2))
    else:
        print(f"[*] Unsafe Boundary Tracer complete: {total_unsafe_blocks} unsafe blocks found.")
        print(f"    Missing // SAFETY: proofs: {missing_safety_proofs}")
        print(f"    Non-island violations: {non_island_violations}")
        print(f"    Raw pointer operations: {ptr_ops_count}")

    if not args.no_save:
        out_name = f"unsafe_tracer_{args.crate}" if args.crate else "unsafe_tracer_workspace"
        j_p, m_p = report.write_artifacts(out_name, Path(args.output_dir) if args.output_dir else None)
        if not args.json:
            print(f"[+] Saved artifacts:\n    {j_p}\n    {m_p}")


if __name__ == "__main__":
    main()
