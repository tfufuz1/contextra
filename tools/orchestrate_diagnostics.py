#!/usr/bin/env python3
"""
orchestrate_diagnostics.py - Master-Orchestrierung aller Kontext- und Diagnose-Werkzeuge.

Führt sequentiell oder parallel alle Diagnose-Werkzeuge aus:
1. zero_panic_analyzer.py
2. unsafe_boundary_auditor.py
3. determinism_guard.py
4. type_registry_sync.py
5. async_boundary_scanner.py
6. fsync_durability_tracer.py
7. port_contract_auditor.py
8. dead_code_pruner.py
9. error_topology_mapper.py
10. test_matrix_advisor.py

Aggregiert die Ergebnisse in docs/diagnostics/DIAGNOSTIC_SUMMARY.md.
"""

import os
import sys
import subprocess
import json
import argparse
from pathlib import Path

TOOLS = [
    ("zero_panic_analyzer.py", "Zero-Panic & Unsafe Indexing", "ZERO_PANIC_AUDIT.md", "zero_panic_audit.json"),
    ("unsafe_boundary_auditor.py", "Unsafe Boundary & Island Compliance", "UNSAFE_BOUNDARY_AUDIT.md", "unsafe_boundary_audit.json"),
    ("determinism_guard.py", "Determinismus (P28)", "DETERMINISM_GUARD_AUDIT.md", "determinism_guard_audit.json"),
    ("type_registry_sync.py", "Typ-Register & Kollisionen (K3/A2)", "TYPE_REGISTRY_COLLISIONS.md", "type_registry_collisions.json"),
    ("async_boundary_scanner.py", "Async Boundary & Lock-Across-Await (M8)", "ASYNC_BOUNDARY_REPORT.md", "async_boundary_report.json"),
    ("fsync_durability_tracer.py", "Fsync & Durability Tracer (INV-FSYNC-1)", "FSYNC_DURABILITY_AUDIT.md", "fsync_durability_audit.json"),
    ("port_contract_auditor.py", "Port Contracts & Facade Leaks (M3/M7)", "PORT_CONTRACT_AUDIT.md", "port_contract_audit.json"),
    ("dead_code_pruner.py", "Dead Code & Orphan Symbols", "DEAD_CODE_ANALYSIS.md", "dead_code_analysis.json"),
    ("error_topology_mapper.py", "Error Topology & Data Leaks (INV-LOG-1)", "ERROR_TOPOLOGY_REPORT.md", "error_topology_report.json"),
    ("test_matrix_advisor.py", "Test Matrix & Pyramid Advisor", "TEST_MATRIX_ADVISOR.md", "test_matrix_advisor.json"),
]

def run_tool(script_path: Path, workspace_root: Path, output_dir: Path):
    cmd = [
        sys.executable,
        str(script_path),
        "--workspace", str(workspace_root),
        "--output-dir", str(output_dir)
    ]
    try:
        res = subprocess.run(cmd, capture_output=True, text=True, check=True)
        return True, res.stdout.strip()
    except subprocess.CalledProcessError as e:
        return False, e.stderr.strip()

def main():
    parser = argparse.ArgumentParser(description="Master Diagnostics Orchestrator")
    parser.add_argument("--workspace", default=".", help="Workspace root")
    parser.add_argument("--output-dir", default="docs/diagnostics", help="Output directory")
    args = parser.parse_args()

    workspace_root = Path(args.workspace).resolve()
    tools_dir = workspace_root / "tools"
    output_dir = workspace_root / args.output_dir
    output_dir.mkdir(parents=True, exist_ok=True)

    summary = []
    print("Starte Orchestrierung der Contextra-Diagnosewerkzeuge...")

    for script_name, title, md_file, json_file in TOOLS:
        script_file = tools_dir / script_name
        if not script_file.exists():
            summary.append({"name": script_name, "title": title, "status": "FEHLT", "output": ""})
            continue

        print(f" -> Führe {script_name} aus...")
        ok, out = run_tool(script_file, workspace_root, output_dir)
        summary.append({
            "name": script_name,
            "title": title,
            "status": "ERFOLG" if ok else "FEHLER",
            "md_report": str(output_dir / md_file),
            "json_report": str(output_dir / json_file),
            "output": out
        })

    # Write Master Summary Report
    summary_md = output_dir / "DIAGNOSTIC_SUMMARY.md"
    with open(summary_md, "w", encoding="utf-8") as sf:
        sf.write("# Gesamtdiagnose & Systemzustand der Rust-Codebase\n\n")
        sf.write(f"> Generiert durch `tools/orchestrate_diagnostics.py`.\n")
        sf.write(f"> Ausführungszeitpunkt: automatisiert durch Principal Agentic Engineering.\n\n")

        sf.write("## Übersicht der ausgeführten Diagnose-Module\n\n")
        sf.write("| Modul | Analysebereich | Status | Dokumentation | Maschinendaten |\n")
        sf.write("|---|---|---|---|---|\n")
        for s in summary:
            sf.write(f"| `{s['name']}` | {s['title']} | **{s['status']}** | [`{Path(s.get('md_report', '')).name}`]({Path(s.get('md_report', '')).name}) | [`{Path(s.get('json_report', '')).name}`]({Path(s.get('json_report', '')).name}) |\n")

        sf.write("\n## Schnelleinstieg & Werkzeugverzeichnis für Coding-Agenten\n\n")
        sf.write("Alle Werkzeuge können einzeln aufgerufen oder via `tools/orchestrate_diagnostics.py` als Gesamtsuite ausgeführt werden:\n\n")
        for s in summary:
            sf.write(f"- **`python3 tools/{s['name']}`**: {s['title']}\n")

    print(f"\nGesamtdiagnose erfolgreich abgeschlossen!")
    print(f" -> Master Report: {summary_md}")

if __name__ == "__main__":
    main()
