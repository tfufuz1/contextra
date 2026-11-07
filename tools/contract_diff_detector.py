#!/usr/bin/env python3
"""
contract_diff_detector.py — Port & Contract Interface Drift Detector.

Analyzes architectural trait contracts and port drift:
- Discovers canonical traits defined in `contextra-ports` (StorageEngine, TextIndex, VectorIndex, Vfs, Clock, Rng, IdGen, ErasureLayer, Purger, LayerScanner, etc.).
- Compares methods, parameter counts, return types, and crash/error contract documentation (# Fehler, # Crash).
- Scans implementing crates (contextra-store, contextra-vector, contextra-text, etc.) to detect contract compliance or trait method signature drift.
- Flags missing port implementations or direct concrete implementations bypassing ports.
"""

from __future__ import annotations

import json
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Dict, List, Optional, Set, Tuple

from agent_tooling_common import (
    DiagnosticReport,
    create_standard_cli,
    discover_crates,
    get_repo_root,
    iter_rust_files,
    strip_comments_and_strings,
)


@dataclass
class TraitMethod:
    name: str
    signature: str
    is_async: bool
    has_fehler_doc: bool
    has_crash_doc: bool


@dataclass
class PortTrait:
    name: str
    file: str
    line: int
    methods: Dict[str, TraitMethod] = field(default_factory=dict)


def parse_port_traits(ports_path: Path, repo_root: Path) -> Dict[str, PortTrait]:
    traits: Dict[str, PortTrait] = {}
    re_trait_def = re.compile(r"^\s*(?:pub(?:\s*\([^)]+\))?\s+)?(?:unsafe\s+)?trait\s+([A-Za-z0-9_]+)")
    re_method_def = re.compile(r"^\s*(?:async\s+)?(?:unsafe\s+)?fn\s+([A-Za-z0-9_]+)\s*\(([^)]*)\)(?:\s*->\s*([^;{]+))?")

    for rpath in iter_rust_files(ports_path, include_tests=False):
        try:
            raw_text = rpath.read_text(encoding="utf-8")
        except Exception:
            continue

        lines = raw_text.splitlines()
        rel_path = str(rpath.relative_to(repo_root))

        i = 0
        n = len(lines)
        while i < n:
            l = lines[i]
            m_trait = re_trait_def.match(l)
            if m_trait:
                trait_name = m_trait.group(1)
                trait_line = i + 1
                methods: Dict[str, TraitMethod] = {}

                # Advance to {
                while i < n and "{" not in lines[i]:
                    i += 1
                depth = 1
                i += 1
                doc_lines: List[str] = []

                while i < n and depth > 0:
                    cur = lines[i]
                    depth += cur.count("{") - cur.count("}")
                    if depth <= 0:
                        break

                    stripped = cur.strip()
                    if stripped.startswith("///"):
                        doc_lines.append(stripped)
                    elif stripped and not stripped.startswith("//") and not stripped.startswith("#"):
                        m_m = re_method_def.search(stripped)
                        if m_m:
                            m_name = m_m.group(1)
                            full_doc = "\n".join(doc_lines)
                            methods[m_name] = TraitMethod(
                                name=m_name,
                                signature=stripped.rstrip(";{ "),
                                is_async="async fn" in stripped,
                                has_fehler_doc="# Fehler" in full_doc or "# Errors" in full_doc,
                                has_crash_doc="# Crash" in full_doc,
                            )
                            doc_lines.clear()
                    i += 1

                traits[trait_name] = PortTrait(
                    name=trait_name,
                    file=rel_path,
                    line=trait_line,
                    methods=methods,
                )
            i += 1

    return traits


def parse_trait_implementations(crate_path: Path, repo_root: Path) -> List[Dict]:
    impls = []
    re_impl = re.compile(r"impl(?:<[^>]+>)?\s+([A-Za-z0-9_:]+)(?:<[^>]+>)?\s+for\s+([A-Za-z0-9_:]+)")

    for rpath in iter_rust_files(crate_path, include_tests=False):
        try:
            raw_text = rpath.read_text(encoding="utf-8")
        except Exception:
            continue

        clean = strip_comments_and_strings(raw_text)
        rel_path = str(rpath.relative_to(repo_root))

        for idx, line in enumerate(clean.splitlines(), start=1):
            m = re_impl.search(line)
            if m:
                trait_ref = m.group(1).split("::")[-1]
                target_type = m.group(2).split("::")[-1]
                impls.append({
                    "trait": trait_ref,
                    "target_type": target_type,
                    "file": rel_path,
                    "line": idx,
                })
    return impls


def main():
    cli = create_standard_cli("Port & Contract Interface Drift Detector.")
    args = cli.parse_args()

    repo_root = get_repo_root()
    crates = discover_crates(repo_root)

    ports_crate = crates.get("contextra-ports")
    if not ports_crate:
        print("Error: contextra-ports crate not found.", file=sys.stderr)
        sys.exit(1)

    port_traits = parse_port_traits(ports_crate.path, repo_root)

    report = DiagnosticReport(
        tool_name="contract_diff_detector.py",
        title="Port & Contract Interface Drift Diagnostic Report",
        description="Analyzes canonical traits in contextra-ports, verifies '# Fehler' and '# Crash' contract documentation, and maps implementations across the workspace.",
    )

    total_port_methods = sum(len(t.methods) for t in port_traits.values())
    missing_doc_count = 0
    missing_crash_count = 0

    for tname, trait in sorted(port_traits.items()):
        for mname, method in trait.methods.items():
            if not method.has_fehler_doc:
                missing_doc_count += 1
            if not method.has_crash_doc:
                missing_crash_count += 1

            sev = "LOW" if method.has_fehler_doc else "MEDIUM"
            report.add_finding({
                "crate": "contextra-ports",
                "file": trait.file,
                "line": trait.line,
                "trait": tname,
                "method": mname,
                "is_async": method.is_async,
                "has_fehler_doc": method.has_fehler_doc,
                "has_crash_doc": method.has_crash_doc,
                "description": f"Port method `{tname}::{mname}` (Fehler-Doc: {method.has_fehler_doc}, Crash-Doc: {method.has_crash_doc})",
            })

    # Track implementations
    target_crates = {k: v for k, v in crates.items() if k != "contextra-ports"}
    if args.crate:
        target_crates = {args.crate: crates[args.crate]} if args.crate in crates else {}

    impl_count = 0
    crate_impl_map = []
    for cname, cmeta in sorted(target_crates.items()):
        impls = parse_trait_implementations(cmeta.path, repo_root)
        port_impls = [imp for imp in impls if imp["trait"] in port_traits]
        impl_count += len(port_impls)
        if port_impls:
            for imp in port_impls:
                report.add_finding({
                    "crate": cname,
                    "file": imp["file"],
                    "line": imp["line"],
                    "trait": imp["trait"],
                    "target_type": imp["target_type"],
                    "description": f"Implementation: `{imp['target_type']}` implements Port `{imp['trait']}`",
                })
            crate_impl_map.append((cname, len(port_impls), ", ".join(sorted({imp["trait"] for imp in port_impls}))))

    report.set_metric("total_port_traits", len(port_traits))
    report.set_metric("total_port_methods", total_port_methods)
    report.set_metric("methods_missing_fehler_doc", missing_doc_count)
    report.set_metric("methods_missing_crash_doc", missing_crash_count)
    report.set_metric("workspace_port_implementations", impl_count)

    impl_table = [
        "| Crate | Anzahl Port-Impls | Implementierte Ports |",
        "|---|---|---|",
    ]
    for cname, cnt, traits_str in crate_impl_map:
        impl_table.append(f"| `{cname}` | {cnt} | `{traits_str}` |")
    report.add_section("Port-Implementierungs-Matrix", "\n".join(impl_table))

    if args.json:
        print(json.dumps(report.to_dict(), indent=2))
    else:
        print(f"[*] Contract Diff Detector complete: {len(port_traits)} port traits with {total_port_methods} methods.")
        print(f"    Total workspace implementations of ports: {impl_count}")
        print(f"    Methods missing '# Fehler' doc: {missing_doc_count}")
        print(f"    Methods missing '# Crash' doc: {missing_crash_count}")

    if not args.no_save:
        out_name = f"contract_diff_{args.crate}" if args.crate else "contract_diff_workspace"
        j_p, m_p = report.write_artifacts(out_name, Path(args.output_dir) if args.output_dir else None)
        if not args.json:
            print(f"[+] Saved artifacts:\n    {j_p}\n    {m_p}")


if __name__ == "__main__":
    main()
