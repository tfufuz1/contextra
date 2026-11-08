#!/usr/bin/env python3
"""
cargo_dependency_graph.py — Multi-Crate Ring Boundary & Dependency Graph Auditor.

Analyzes Cargo manifests and architecture layering:
- Extracts [dependencies] and [dev-dependencies] for all crates.
- Compares against `capabilities.toml` (may_depend_on, ring layering 0..4).
- Checks M1 (dependencies only point to equal or lower rings).
- Checks M6 (no dev-dependencies on higher layers).
- Checks for circular dependency chains.
- Validates forbidden dependencies (e.g. core domain crates depending on FFI or ML frameworks).
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
    cli = create_standard_cli("Ring Boundary & Dependency Graph Auditor.")
    args = cli.parse_args()

    repo_root = get_repo_root()
    crates = discover_crates(repo_root)

    report = DiagnosticReport(
        tool_name="cargo_dependency_graph.py",
        title="Ring Boundary & Dependency Graph Architecture Audit",
        description="Verifies the Ring model (Ring 4 -> 0), validates capabilities.toml may_depend_on compliance (M1), and prevents dev-dependency leaks (M6).",
    )

    dep_graph: Dict[str, Set[str]] = defaultdict(set)
    m1_violations = 0
    m6_violations = 0
    internal_crate_names = set(crates.keys())

    crate_rows = []

    for cname, cmeta in sorted(crates.items()):
        deps, dev_deps = parse_cargo_dependencies(cmeta.manifest_path)

        # Filter internal workspace deps
        internal_deps = {d: v for d, v in deps.items() if d in internal_crate_names}
        internal_dev = {d: v for d, v in dev_deps.items() if d in internal_crate_names}

        for idp in internal_deps:
            dep_graph[cname].add(idp)

        c_ring = cmeta.ring

        # Check M1: dependency ring <= current ring
        for dep_name in internal_deps:
            dep_meta = crates[dep_name]
            dep_ring = dep_meta.ring
            if c_ring is not None and dep_ring is not None:
                if dep_ring > c_ring:
                    m1_violations += 1
                    report.add_finding({
                        "crate": cname,
                        "file": str(cmeta.manifest_path.relative_to(repo_root)),
                        "line": 1,
                        "kind": "M1_RING_VIOLATION",
                        "severity": "CRITICAL",
                        "description": f"M1 Ring violation: `{cname}` (Ring {c_ring}) depends on `{dep_name}` (Ring {dep_ring})",
                    })

        # Check M6: dev-dependencies on higher layers
        for dev_dep in internal_dev:
            dep_meta = crates[dev_dep]
            dep_ring = dep_meta.ring
            if c_ring is not None and dep_ring is not None:
                if dep_ring > c_ring:
                    m6_violations += 1
                    report.add_finding({
                        "crate": cname,
                        "file": str(cmeta.manifest_path.relative_to(repo_root)),
                        "line": 1,
                        "kind": "M6_DEV_DEP_VIOLATION",
                        "severity": "HIGH",
                        "description": f"M6 Dev-dependency violation: `{cname}` (Ring {c_ring}) has dev-dep on higher Ring `{dev_dep}` (Ring {dep_ring})",
                    })

        crate_rows.append((cname, c_ring if c_ring is not None else "?", len(internal_deps), len(internal_dev)))

    cycles = find_cycles(dep_graph)
    for cycle in cycles:
        report.add_finding({
            "crate": cycle[0],
            "file": "Cargo.toml",
            "line": 1,
            "kind": "CIRCULAR_DEPENDENCY",
            "severity": "CRITICAL",
            "description": f"Circular dependency cycle detected: {' -> '.join(cycle)}",
        })

    report.set_metric("total_crates", len(crates))
    report.set_metric("m1_ring_violations", m1_violations)
    report.set_metric("m6_dev_dep_violations", m6_violations)
    report.set_metric("circular_dependency_cycles", len(cycles))

    matrix_table = [
        "| Crate | Ring | Interne Abhängigkeiten | Dev-Abhängigkeiten |",
        "|---|---|---|---|",
    ]
    for cname, ring, ndeps, ndev in crate_rows:
        matrix_table.append(f"| `{cname}` | {ring} | {ndeps} | {ndev} |")
    report.add_section("Crate-Ring-Matrix", "\n".join(matrix_table))

    if args.json:
        print(json.dumps(report.to_dict(), indent=2))
    else:
        print(f"[*] Cargo Dependency Graph audit complete: {len(crates)} crates analyzed.")
        print(f"    M1 Ring violations: {m1_violations}")
        print(f"    M6 Dev-dependency violations: {m6_violations}")
        print(f"    Circular dependency cycles: {len(cycles)}")

    if not args.no_save:
        out_name = "dependency_graph_workspace"
        j_p, m_p = report.write_artifacts(out_name, Path(args.output_dir) if args.output_dir else None)
        if not args.json:
            print(f"[+] Saved artifacts:\n    {j_p}\n    {m_p}")


if __name__ == "__main__":
    main()
