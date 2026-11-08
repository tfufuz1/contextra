#!/usr/bin/env python3
"""
audit_modular_building_blocks.py — Workspace-weiter Entkopplungs- und Baukasten-Audit.

Generiert eine detaillierte architektonische Bestandsaufnahme für das gesamte Contextra-Projekt,
prüft alle Crates auf Modularität, direkte Verdrahtungen, Schnittstellenabstraktion,
Dead-Code-Kandidaten und verstreute Löschketten/DSGVO-Artefakte und schreibt einen
aussagekräftigen Bericht nach docs/generated/modular_decoupling_audit.md.
"""

from __future__ import annotations

import argparse
import datetime
import json
import os
import subprocess
import sys
from pathlib import Path

# decouple_inspector als Modul laden
tools_dir = Path(__file__).resolve().parent
sys.path.insert(0, str(tools_dir))

try:
    from decouple_inspector import evaluate_crate_decoupling, inspect_crate
except ImportError:
    print("Fehler: decouple_inspector konnte nicht importiert werden.", file=sys.stderr)
    sys.exit(1)


def generate_markdown_report(evaluations: list[dict], out_path: Path) -> None:
    now = datetime.datetime.now().strftime("%Y-%m-%d %H:%M:%S")

    total_crates = len(evaluations)
    total_loc = sum(e["loc"] for e in evaluations)
    autarkic = [e for e in evaluations if e["dependencies_count"] == 0]
    high_priority = [e for e in evaluations if e["dependencies_count"] > 0 or e["erasure_leaks_count"] > 0]
    total_erasure_leaks = sum(e["erasure_leaks_count"] for e in evaluations)
    total_dead_code = sum(e.get("dead_code_count", 0) for e in evaluations)
    total_missing_ports = sum(e.get("missing_ports_count", 0) for e in evaluations)

    md = []
    md.append("# Contextra Modularisierungs- & Entkopplungs-Audit")
    md.append("")
    md.append(f"> **Datum:** {now} | **Analysierte Crates:** {total_crates} | **Gesamtumfang:** {total_loc:,} LOC")
    md.append("")
    md.append("## 1. Strategische Zielsetzung & Architekturvision")
    md.append(
        "Dieses Dokument dient als zentrale Orientierung für Architekten und Coding-Agenten zur "
        "Transformation des Contextra-Ökosystems von einem fest verdrahteten Monolithen "
        "zu einem **vollständig modularen Baukastensystem (Building Blocks)**."
    )
    md.append("")
    md.append("### Kernprinzipien des Baukastensystems:")
    md.append("1. **Vollständige Entkopplung:** Jeder Crate muss isoliert verständlich, testbar und in Fremdprojekten nutzbar sein.")
    md.append("2. **Schnittstellen via Ports/Traits:** Direkte Abhängigkeiten zwischen konkreten Crates werden aufgelöst; Abstraktionen liegen in `contextra-ports` oder generischen Parametern.")
    md.append("3. **Isolation der Löschkette:** Alle DSGVO-, Gesetzes- und multi-layer Deletion-Proof-Logiken werden vollständig in `contextra-erasure` isoliert und aus Kernkomponenten (Store, Vector, Text, Crypto, DB) entfernt.")
    md.append("4. **Zero-Dead-Code & Saubere Fassaden:** Öffentliche Schnittstellen werden minimal gehalten und lecken keine internen Fremdtypen.")
    md.append("")

    md.append("## 2. Gesamtübersicht des Baukastensystems")
    md.append("")
    md.append(f"- **Vollkommen autarke Bausteine:** {len(autarkic)} Crates")
    md.append(f"- **Entkopplungs- / Bereinigungsbedarf:** {len(high_priority)} Crates")
    md.append(f"- **Verstreute Löschketten-/DSGVO-Relikte außerhalb `contextra-erasure`:** {total_erasure_leaks} Fundstellen")
    md.append(f"- **Identifizierte Dead-Code-Kandidaten (ungenutzte interne Definitionen):** {total_dead_code} Stellen")
    md.append(f"- **Fehlende Port-Abstraktionen (direkte Kernel-Importe):** {total_missing_ports} Stellen")
    md.append("")

    md.append("| Crate | Status | LOC | Dateien | Interne Deps | Schnittstellen (T/S/E) | Pub/Gesamt Fns | Dead Code | DSGVO/Lösch-Relikte |")
    md.append("|---|---|---:|---:|---:|---:|---:|---:|---:|")
    for e in sorted(evaluations, key=lambda x: (x["score"], -x["loc"])):
        md.append(
            f"| `{e['crate']}` | **{e['score']}/100** | {e['status']} | {e['loc']:,} | {e['files']} | "
            f"{e['dependencies_count']} | {e.get('traits_count',0)}/{e.get('structs_count',0)}/{e.get('enums_count',0)} | "
            f"{e.get('pub_functions_count',0)}/{e.get('functions_count',0)} | {e.get('dead_code_count',0)} | {e['erasure_leaks_count']} |"
        )
    md.append("")

    md.append("## 3. Priorisierte Entkopplungs-Roadmap für Coding-Agenten")
    md.append("")
    for e in high_priority:
        md.append(f"### 📦 Crate: `{e['crate']}` (Score: {e['score']}/100)")
        md.append(f"- **Umfang:** {e['loc']:,} LOC ({e['files']} Quelldateien)")
        md.append(f"- **Abhängigkeiten ({e['dependencies_count']}):** `{', '.join(e['dependencies']) or 'keine'}`")
        md.append(f"- **Funktionsinventar:** {e.get('pub_functions_count', 0)} pub-Funktionen von {e.get('functions_count', 0)} gesamt")
        md.append(f"- **Schnittstellen:** {e.get('traits_count', 0)} Traits, {e.get('structs_count', 0)} Structs, {e.get('enums_count', 0)} Enums")
        if e["coupling_summary"]:
            md.append("- **Schnittstellen-Kopplung (importierte Fremdsymbole):**")
            for dep, c in e["coupling_summary"].items():
                md.append(f"  - `{dep}`: {c} Symbole")
        if e.get("missing_ports_count", 0) > 0:
            md.append(f"- **⚠️ Direkte Kernel-Verdrahtung:** {e['missing_ports_count']} Importe müssen über `contextra-ports` entkoppelt werden.")
        if e.get("dead_code_count", 0) > 0:
            md.append(f"- **⚠️ Dead Code:** {e['dead_code_count']} ungenutzte interne Definitionen identifiziert.")
        if e["erasure_leaks_count"] > 0:
            md.append(f"- **⚠️ DSGVO-/Löschketten-Lecks:** {e['erasure_leaks_count']} Fundstellen müssen in `contextra-erasure` überführt werden.")
        if e["findings"]:
            md.append("- **Befunde:**")
            for f in e["findings"]:
                md.append(f"  - {f}")
        if e["recommendations"]:
            md.append("- **Handlungsschritte für Coding-Agenten:**")
            for r in e["recommendations"]:
                md.append(f"  - [ ] {r}")
        if e.get("dead_code_candidates"):
            md.append("- **Dead-Code-Kandidaten (Auszug):**")
            for dc in e["dead_code_candidates"][:5]:
                md.append(f"  - `[{dc['kind']}] {dc['name']}` ({dc['file']}:{dc['line']})")
        md.append("")

    md.append("## 4. Löschketten- und DSGVO-Fundstelleninventar")
    md.append("")
    md.append("Folgende Stellen müssen im Rahmen der Entkopplung aus den jeweiligen Crates bereinigt werden:")
    md.append("")
    for e in evaluations:
        if e["erasure_leaks"]:
            md.append(f"#### `{e['crate']}` ({e['erasure_leaks_count']} Stellen)")
            for leak in e["erasure_leaks"]:
                md.append(f"- `{leak['file']}:{leak['line']}` — Suchbegriff `{leak['term']}`: `{leak['snippet']}`")
            md.append("")

    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text("\n".join(md), encoding="utf-8")
    print(f"✅ Audit-Bericht erfolgreich geschrieben: {out_path}")


def main() -> int:
    parser = argparse.ArgumentParser(description="Audit-Runner für Baukastensystem und Modularisierung.")
    parser.add_argument("--root", default=".", help="Workspace-Root")
    parser.add_argument("--out", default="docs/generated/modular_decoupling_audit.md", help="Ausgabepfad für Markdown-Report")
    args = parser.parse_args()

    root = Path(args.root).resolve()
    crates_dir = root / "crates"

    crate_dirs = sorted([d for d in crates_dir.iterdir() if d.is_dir() and (d / "Cargo.toml").exists()])
    evaluations = []

    print(f"Analysiere {len(crate_dirs)} Crates unter {crates_dir}...")
    for c_dir in crate_dirs:
        inv = inspect_crate(c_dir, root)
        ev = evaluate_crate_decoupling(inv)
        evaluations.append(ev)

    generate_markdown_report(evaluations, Path(args.out).resolve())
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except BrokenPipeError:
        sys.stderr.close()
        sys.exit(0)
