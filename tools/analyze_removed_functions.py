#!/usr/bin/env python3
"""
analyze_removed_functions.py — Identifies removed/abandoned functions from history and builds cherry-pick catalog.

Compares historical function inventories against the current HEAD workspace, categorizes removed functions,
tracks their historical origin/commit, and writes:
- docs/entfernte_funktionen_historie.csv
- docs/entfernte_funktionen_cherry_pick_katalog.md
"""

from __future__ import annotations

import csv
import os
import re
import subprocess
from collections import defaultdict
from pathlib import Path
from typing import Dict, List, Set, Tuple


def load_csv(path: Path) -> List[Dict[str, str]]:
    with open(path, "r", encoding="utf-8") as f:
        return list(csv.DictReader(f))


def check_presence_in_head(entry: Dict[str, str], head_files_cache: Dict[str, str]) -> Tuple[bool, str]:
    path = entry["datei"]
    name = entry["qualifizierter_name"]
    fn_name = name.split("::")[-1]

    if not os.path.isfile(path):
        return False, "Datei gelöscht oder umbenannt"

    if path not in head_files_cache:
        try:
            head_files_cache[path] = Path(path).read_text(encoding="utf-8", errors="replace")
        except Exception:
            return False, "Datei unlesbar"

    content = head_files_cache[path]
    pattern = rf"\bfn\s+{re.escape(fn_name)}\b"
    if re.search(pattern, content):
        return True, "Vorhanden"
    return False, "Funktion entfernt / umstrukturiert"


def categorize_removal(crate: str, path: str, name: str) -> str:
    path_lower = path.lower()
    name_lower = name.lower()
    if "graph" in crate or "hyperedge" in path_lower or "csr" in path_lower or "ppr" in path_lower:
        return "Graph & Hyperedges (Wissensgraph, PPR, Lovasz, Cascade)"
    if "kvcache" in crate or "quantize_kivi" in path_lower or "snapkv" in path_lower or "radix" in path_lower:
        return "KV-Cache & Quantisierung (KIVI 2-Bit, SnapKV, Radix Trie)"
    if "crypto" in crate or "deletion_proof" in path_lower or "revocation" in path_lower:
        return "Kryptographie & DeletionProof (Löschnachweise, RevocationLog)"
    if "engine" in crate:
        if "decay" in path_lower or "decay" in name_lower:
            return "Engine: Adaptive Decay Controller"
        if "relate" in path_lower or "hyperedge" in path_lower:
            return "Engine: Relate & Hyperedges"
        if "extraction" in path_lower:
            return "Engine: Auto-Extraction (OpenIE, RuleBased)"
        if "merge" in path_lower or "wasm" in path_lower:
            return "Engine: WASM / Merge Operator"
        return "Engine: Transaktionen & Background Maintenance"
    if "rank" in crate or "dibud" in path_lower or "conformal" in path_lower:
        return "Ranking & Kalibrierung (DiBud, Conformal Prediction)"
    if "router" in crate or "fc_ts" in path_lower:
        return "Routing (Flow-Corrected Thompson Sampling, Bandits)"
    if "agent" in crate or "clm" in path_lower or "scratchpad" in path_lower or "dlq" in path_lower:
        return "Agent & Workflow (Scratchpad, DLQ, GoalCondition, Replay)"
    if "mcp" in crate:
        return "MCP-Protokoll & Tools (Parking Tools, Egress Gateway)"
    if "sandbox" in crate:
        return "WASM Sandbox & Isolation"
    if "privacy" in crate:
        return "Privacy & Egress Schutz"
    return "Allgemeine Hilfsfunktionen & Benchmarks"


def get_cherry_pick_commit(crate: str, path: str) -> str:
    if "graph" in crate:
        return "05a6a9af (archiv/v0) / 7fddc8504"
    if "kvcache" in crate:
        return "05a6a9af (archiv/v0) / 8412f0862"
    if "crypto" in crate:
        return "a43391fa5 / 8412f0862"
    if "engine" in crate:
        return "7fddc8504 / 8412f0862"
    if "rank" in crate:
        return "c27326391 / 5d349c188"
    if "adapt" in crate:
        return "05a6a9af (archiv/v0)"
    if "agent" in crate:
        return "05a6a9af (archiv/v0) / 8412f0862"
    return "7fddc8504 (Stand 16. Oktober 2026)"


def main():
    repo_root = Path.cwd()
    oct_csv_path = repo_root / "docs" / "funktionsinventar_oktober_16.csv"
    if not oct_csv_path.is_file():
        oct_csv_path = Path("/tmp/file_attachments/jules/contextra_funktionsinventar.csv")

    rows = load_csv(oct_csv_path)
    head_cache: Dict[str, str] = {}

    removed_entries = []
    category_counts = defaultdict(int)
    crate_counts = defaultdict(int)

    for r in rows:
        if r["crate"] == "xtask":
            continue
        present, reason = check_presence_in_head(r, head_cache)
        if not present:
            cat = categorize_removal(r["crate"], r["datei"], r["qualifizierter_name"])
            commit_ref = get_cherry_pick_commit(r["crate"], r["datei"])
            entry = dict(r)
            entry["kategorie"] = cat
            entry["status_grund"] = reason
            entry["cherry_pick_referenz"] = commit_ref
            removed_entries.append(entry)
            category_counts[cat] += 1
            crate_counts[r["crate"]] += 1

    print(f"Total historical functions inspected: {len(rows)}")
    print(f"Identified removed / decoupled functions: {len(removed_entries)}")

    out_csv = repo_root / "docs" / "entfernte_funktionen_historie.csv"
    fieldnames = [
        "crate",
        "ring",
        "datei",
        "zeile",
        "ende",
        "loc",
        "qualifizierter_name",
        "sichtbarkeit",
        "async",
        "unsafe",
        "dokumentiert",
        "generiert",
        "kontext",
        "signatur",
        "kategorie",
        "status_grund",
        "cherry_pick_referenz",
    ]
    with open(out_csv, "w", newline="", encoding="utf-8") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        for re_entry in removed_entries:
            writer.writerow(re_entry)
    print(f"Saved: {out_csv}")

    out_md = repo_root / "docs" / "entfernte_funktionen_cherry_pick_katalog.md"
    md_lines = [
        "# Cherry-Pick-Katalog: Historisch entfernte und verworfene Funktionen",
        "",
        "Dieser Katalog dokumentiert alle Funktionen, Methoden und Algorithmen, die im reifsten, funktionsreichen Zustand (Stand 16. Oktober 2026 / Tags `archiv/v0`, `park/EP1..EP6/v1`) implementiert waren, jedoch im Zuge der Baukasten-Transformation, Entkopplung oder Vereinfachung aus dem aktuellen HEAD entfernt oder modifiziert wurden.",
        "",
        f"> **Gesamtanzahl erfasster entfernter Funktionen:** {len(removed_entries)}  ",
        "> **Zweck:** Präzise Referenz für gezielte Cherry-Picks und Wiederinbetriebnahme hochentwickelter Algorithmen.  ",
        "",
        "## Übersicht nach Themenbereichen & Domänen",
        "",
        "| Domäne / Kategorie | Anzahl Funktionen | Primäre Cherry-Pick-Referenz |",
        "|---|---|---|",
    ]

    for cat in sorted(category_counts.keys()):
        sample_commit = get_cherry_pick_commit(cat.lower(), "")
        md_lines.append(f"| **{cat}** | {category_counts[cat]} | `{sample_commit}` |")

    md_lines.extend([
        "",
        "## Übersicht nach Crates",
        "",
        "| Crate | Entfernte Funktionen |",
        "|---|---|",
    ])
    for c in sorted(crate_counts.keys()):
        md_lines.append(f"| `{c}` | {crate_counts[c]} |")

    md_lines.extend([
        "",
        "## Detail-Katalog der entfernten Funktionen (nach Domänen gegliedert)",
        "",
    ])

    by_cat = defaultdict(list)
    for e in removed_entries:
        by_cat[e["kategorie"]].append(e)

    for cat in sorted(by_cat.keys()):
        md_lines.extend([
            f"### {cat}",
            "",
            "| Crate | Historische Datei & Zeile | Funktion / Methode | Kontext | Signatur | Cherry-Pick-Quelle |",
            "|---|---|---|---|---|---|",
        ])
        for item in by_cat[cat]:
            sig_clean = item["signatur"].replace("|", "\\|")
            md_lines.append(
                f"| `{item['crate']}` | `{item['datei']}:{item['zeile']}` | `{item['qualifizierter_name']}` | `{item['kontext'] or '-'}` | `{sig_clean}` | `{item['cherry_pick_referenz']}` |"
            )
        md_lines.append("")

    out_md.write_text("\n".join(md_lines), encoding="utf-8")
    print(f"Saved: {out_md}")


if __name__ == "__main__":
    main()
