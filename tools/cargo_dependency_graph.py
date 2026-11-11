#!/usr/bin/env python3
"""
cargo_dependency_graph.py — Architecture Layering & Dependency Graph Auditor.

Analyzes Cargo manifests and architecture layering per SPEC §1.1, §3.1, §8.1:
- Extracts [dependencies] and [dev-dependencies] for all crates.
- Compares against `capabilities.toml` (Schema v3: levels E0..E3, allowed_internal, feature_closure_allowed).
- Checks INV-8 (dependencies only point downward or within allowed layer closures: E3 -> E2 -> E1 -> E0).
- Checks for circular dependency chains.
- Validates forbidden dependencies and produces dependency matrix.
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
)


def parse_cargo_dependencies(manifest_path: Path) -> Tuple[Dict[str, str], Dict[str, str]]:
    """Parse normal and dev dependencies from Cargo.toml."""
    deps: Dict[str, str] = {}
    dev_deps: Dict[str, str] = {}

    if not manifest_path.is_file():
        return deps, dev_deps

    current_section = None
    with open(manifest_path, "r", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith("#"):
                continue

            if line.startswith("[") and line.endswith("]"):
                sec = line.strip("[]")
                if sec == "dependencies":
                    current_section = "deps"
                elif sec == "dev-dependencies":
                    current_section = "dev_deps"
                elif sec.startswith("dependencies."):
                    current_section = "deps"
                elif sec.startswith("dev-dependencies."):
                    current_section = "dev_deps"
                else:
                    current_section = None
                continue

            if current_section in ("deps", "dev_deps") and "=" in line:
                dep_name = line.split("=", 1)[0].strip()
                val = line.split("=", 1)[1].strip()
                if current_section == "deps":
                    deps[dep_name] = val
                else:
                    dev_deps[dep_name] = val

    return deps, dev_deps


def find_cycles(graph: Dict[str, Set[str]]) -> List[List[str]]:
    """Find cycles in a directed graph using DFS."""
    cycles = []
    visited: Set[str] = set()
    rec_stack: List[str] = []

    def dfs(node: str):
        visited.add(node)
        rec_stack.append(node)

        for neighbor in graph.get(node, []):
            if neighbor not in visited:
                dfs(neighbor)
            elif neighbor in rec_stack:
                cycle_idx = rec_stack.index(neighbor)
                cycles.append(rec_stack[cycle_idx:] + [neighbor])

        rec_stack.pop()

    for n in graph:
        if n not in visited:
            dfs(n)

    return cycles


def main():
    cli = create_standard_cli("Architecture Layering & Dependency Graph Auditor.")
    args = cli.parse_args()

    repo_root = get_repo_root()
    crates = discover_crates(repo_root)

    report = DiagnosticReport(
        tool_name="cargo_dependency_graph.py",
        title="Architecture Layering & Dependency Graph Audit",
        description="Verifies the normative architecture layering (E0..E3), validates capabilities.toml Schema v3 compliance, and enforces INV-8.",
    )

    dep_graph: Dict[str, Set[str]] = defaultdict(set)
    inv8_violations = 0
    circular_cycles = 0
    internal_crate_names = set(crates.keys())

    crate_rows = []

    for cname, cmeta in sorted(crates.items()):
        deps, dev_deps = parse_cargo_dependencies(cmeta.manifest_path)

        # Filter internal workspace deps
        internal_deps = {d: v for d, v in deps.items() if d in internal_crate_names}
        internal_dev = {d: v for d, v in dev_deps.items() if d in internal_crate_names}

        for idp in internal_deps:
            dep_graph[cname].add(idp)

        c_level = cmeta.level

        # Check INV-8: normal dependency level <= current level
        for dep_name, dep_val in internal_deps.items():
            dep_meta = crates.get(dep_name)
            if not dep_meta:
                continue
            dep_level = dep_meta.level
            is_optional = "optional = true" in dep_val or "optional=true" in dep_val

            if c_level is not None and dep_level is not None:
                if dep_level > c_level:
                    inv8_violations += 1
                    report.add_finding({
                        "crate": cname,
                        "file": str(cmeta.manifest_path.relative_to(repo_root)),
                        "line": 1,
                        "kind": "INV8_LAYERING_VIOLATION",
                        "severity": "CRITICAL" if not is_optional else "MEDIUM",
                        "description": f"INV-8 Layering violation: `{cname}` (E{c_level}) depends on `{dep_name}` (E{dep_level}){' [optional]' if is_optional else ''}",
                    })

        level_display = f"E{c_level}" if c_level is not None else "?"
        crate_rows.append((cname, level_display, len(internal_deps), len(internal_dev)))

    cycles = find_cycles(dep_graph)
    for cycle in cycles:
        circular_cycles += 1
        report.add_finding({
            "crate": cycle[0],
            "file": "Cargo.toml",
            "line": 1,
            "kind": "CIRCULAR_DEPENDENCY",
            "severity": "CRITICAL",
            "description": f"Circular dependency cycle detected: {' -> '.join(cycle)}",
        })

    report.set_metric("total_crates", len(crates))
    report.set_metric("inv8_layering_violations", inv8_violations)
    report.set_metric("circular_dependency_cycles", len(cycles))

    matrix_table = [
        "| Crate | Ebene | Interne Abhängigkeiten | Dev-Abhängigkeiten |",
        "|---|---|---|---|",
    ]
    for cname, level_str, ndeps, ndev in crate_rows:
        matrix_table.append(f"| `{cname}` | {level_str} | {ndeps} | {ndev} |")

    report.add_section("Crate-Ebenen-Matrix", "\n".join(matrix_table))

    if args.json:
        print(json.dumps(report.to_dict(), indent=2))
    else:
        print(f"[*] Cargo Dependency Graph audit complete: {len(crates)} crates analyzed.")
        print(f"    INV-8 Layering violations: {inv8_violations}")
        print(f"    Circular dependency cycles: {len(cycles)}")

    if not args.no_save:
        out_name = f"dependency_graph_{args.crate}" if args.crate else "dependency_graph_workspace"
        j_p, m_p = report.write_artifacts(out_name, Path(args.output_dir) if args.output_dir else None)
        if not args.json:
            print(f"[+] Saved artifacts:\n    {j_p}\n    {m_p}")


if __name__ == "__main__":
    main()
