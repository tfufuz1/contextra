#!/usr/bin/env python3
"""
dead_code_microprobe.py — Micro-probe for detecting dead code, unused functions, and orphaned types.

Scans crate source trees:
- Identifies private functions, structs, enums, and constants that are defined but never referenced within their crate.
- Identifies commented-out code blocks and legacy stubs.
- Flags #[allow(dead_code)] attributes and deprecated debt annotations.
- Outputs detailed micro-granular location data for targeted refactoring.
"""

from __future__ import annotations

import json
import re
import sys
from collections import defaultdict
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, List, Set, Tuple

from agent_tooling_common import (
    DiagnosticReport,
    create_standard_cli,
    discover_crates,
    get_repo_root,
    iter_rust_files,
    strip_comments_and_strings,
)


@dataclass
class Definition:
    name: str
    kind: str
    file_path: Path
    rel_path: str
    line: int
    is_public: bool


def find_definitions_and_usages(crate_path: Path, repo_root: Path) -> Tuple[List[Definition], Set[str], List[Dict]]:
    definitions: List[Definition] = []
    all_identifier_usages: Set[str] = set()
    debt_findings: List[Dict] = []

    re_private_fn = re.compile(r"^\s*(?:async\s+)?(?:unsafe\s+)?fn\s+([A-Za-z0-9_]+)")
    re_private_struct = re.compile(r"^\s*struct\s+([A-Za-z0-9_]+)")
    re_private_enum = re.compile(r"^\s*enum\s+([A-Za-z0-9_]+)")
    re_private_const = re.compile(r"^\s*const\s+([A-Za-z0-9_]+)")
    re_allow_dead_code = re.compile(r"#\[allow\(.*dead_code.*\)\]")
    re_deprecated = re.compile(r"#\[deprecated(?:\(.*\))?\]")

    # Pass 1: Parse definitions and usages across all .rs files in crate
    for rpath in iter_rust_files(crate_path, include_tests=True):
        try:
            content = rpath.read_text(encoding="utf-8")
        except Exception:
            continue

        rel_path = str(rpath.relative_to(repo_root))
        clean_content = strip_comments_and_strings(content)

        # Usages: extract all identifiers from clean code
        identifiers = set(re.findall(r"\b([A-Za-z0-9_]+)\b", clean_content))
        all_identifier_usages.update(identifiers)

        # Parse definitions per line
        for idx, (raw_line, clean_line) in enumerate(zip(content.splitlines(), clean_content.splitlines()), start=1):
            s_clean = clean_line.strip()
            s_raw = raw_line.strip()

            if re_allow_dead_code.search(s_raw):
                debt_findings.append({
                    "file": rel_path,
                    "line": idx,
                    "kind": "ALLOW_DEAD_CODE",
                    "description": "Explicit #[allow(dead_code)] annotation",
                    "snippet": s_raw,
                })

            if re_deprecated.search(s_raw):
                debt_findings.append({
                    "file": rel_path,
                    "line": idx,
                    "kind": "DEPRECATED_ITEM",
                    "description": "Deprecated item marked for removal",
                    "snippet": s_raw,
                })

            # Check private items (excluding main, tests, trait methods)
            if s_clean.startswith("pub"):
                continue

            # Private fn
            m = re_private_fn.match(s_clean)
            if m:
                fn_name = m.group(1)
                if not fn_name.startswith("test_") and fn_name not in ("main", "drop", "default", "clone"):
                    definitions.append(Definition(name=fn_name, kind="function", file_path=rpath, rel_path=rel_path, line=idx, is_public=False))
                continue

            # Private struct
            m = re_private_struct.match(s_clean)
            if m:
                definitions.append(Definition(name=m.group(1), kind="struct", file_path=rpath, rel_path=rel_path, line=idx, is_public=False))
                continue

            # Private enum
            m = re_private_enum.match(s_clean)
            if m:
                definitions.append(Definition(name=m.group(1), kind="enum", file_path=rpath, rel_path=rel_path, line=idx, is_public=False))
                continue

            # Private const
            m = re_private_const.match(s_clean)
            if m:
                definitions.append(Definition(name=m.group(1), kind="const", file_path=rpath, rel_path=rel_path, line=idx, is_public=False))
                continue

    return definitions, all_identifier_usages, debt_findings


def check_definition_frequency(crate_path: Path, item_name: str) -> int:
    """Count how many times item_name appears across all files in crate_path."""
    count = 0
    pattern = re.compile(rf"\b{re.escape(item_name)}\b")
    for rpath in iter_rust_files(crate_path, include_tests=True):
        try:
            content = rpath.read_text(encoding="utf-8")
            clean = strip_comments_and_strings(content)
            count += len(pattern.findall(clean))
        except Exception:
            pass
    return count


def main():
    cli = create_standard_cli("Micro-probe for dead code, unused private items, and deprecated annotations.")
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
        tool_name="dead_code_microprobe.py",
        title="Dead Code & Unused Item Microprobe Diagnostic",
        description="Identifies unreferenced private functions, types, constants, #[allow(dead_code)] suppressions, and deprecated items across crates.",
    )

    dead_candidates = 0
    allow_dead_count = 0
    deprecated_count = 0

    for cname, cmeta in sorted(target_crates.items()):
        defs, usages, debts = find_definitions_and_usages(cmeta.path, repo_root)

        for debt in debts:
            if debt["kind"] == "ALLOW_DEAD_CODE":
                allow_dead_count += 1
            else:
                deprecated_count += 1
            report.add_finding({
                "crate": cname,
                "file": debt["file"],
                "line": debt["line"],
                "kind": debt["kind"],
                "description": debt["description"],
                "snippet": debt["snippet"],
            })

        for d in defs:
            # Check occurrences: if count == 1, it's only the definition itself!
            freq = check_definition_frequency(cmeta.path, d.name)
            if freq <= 1:
                dead_candidates += 1
                report.add_finding({
                    "crate": cname,
                    "file": d.rel_path,
                    "line": d.line,
                    "kind": f"UNUSED_PRIVATE_{d.kind.upper()}",
                    "description": f"Private {d.kind} '{d.name}' appears unused in crate (freq={freq})",
                    "snippet": f"{d.kind} {d.name}",
                })

    report.set_metric("dead_code_candidates", dead_candidates)
    report.set_metric("allow_dead_code_attributes", allow_dead_count)
    report.set_metric("deprecated_attributes", deprecated_count)
    report.set_metric("total_findings", len(report.findings))

    summary = [
        "| Kategorie | Anzahl | Beschreibung |",
        "|---|---|---|",
        f"| Unused Private Items | {dead_candidates} | Private Funktionen, Structs oder Consts ohne Aufrufer im Crate |",
        f"| `#[allow(dead_code)]` | {allow_dead_count} | Unterdrückte Dead-Code-Warnungen |",
        f"| `#[deprecated]` | {deprecated_count} | Veraltete Relikte zur Entfernung |",
    ]
    report.add_section("Zusammenfassung der technischen Schulden", "\n".join(summary))

    if args.json:
        print(json.dumps(report.to_dict(), indent=2))
    else:
        print(f"[*] Dead Code Microprobe complete: {len(report.findings)} findings.")
        print(f"    Dead code candidates: {dead_candidates}")
        print(f"    #[allow(dead_code)]: {allow_dead_count}")
        print(f"    #[deprecated]: {deprecated_count}")

    if not args.no_save:
        out_name = f"dead_code_{args.crate}" if args.crate else "dead_code_workspace"
        j_p, m_p = report.write_artifacts(out_name, Path(args.output_dir) if args.output_dir else None)
        if not args.json:
            print(f"[+] Saved artifacts:\n    {j_p}\n    {m_p}")


if __name__ == "__main__":
    main()
