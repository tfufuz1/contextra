#!/usr/bin/env python3
"""Abhängigkeits-Budget für Contextra-Crates.

Misst für jedes Workspace-Crate die Menge an Code, die man beim Entnehmen
mitkopieren müsste: das Crate selbst plus alle transitiv erreichbaren
internen Crates über normale, nicht-optionale Abhängigkeiten.

Dev-Abhängigkeiten und optionale Features werden bewusst NICHT mitgezählt,
weil sie beim Kopieren eines Bausteins nicht zwingend nötig sind.

Aufruf (im Repository-Root):
    python3 tools/abhaengigkeits_budget.py [meta.json] [crate ...]
    python3 tools/abhaengigkeits_budget.py --crate contextra-vector
    python3 tools/abhaengigkeits_budget.py --json

Wird keine meta.json angegeben, ruft das Skript automatisch `cargo metadata` auf.
"""
from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path
from typing import Any, Dict, Set


def get_metadata(pfad: str | None = None) -> dict:
    if pfad and os.path.exists(pfad):
        with open(pfad, "r", encoding="utf-8") as fh:
            return json.load(fh)
    cmd = ["cargo", "metadata", "--format-version", "1", "--no-deps"]
    res = subprocess.run(cmd, capture_output=True, text=True)
    if res.returncode != 0:
        raise RuntimeError(f"cargo metadata fehlgeschlagen: {res.stderr}")
    return json.loads(res.stdout)


def lade_workspace(meta: dict) -> dict:
    return {p["name"]: p for p in meta["packages"] if p.get("source") is None}


def abhaengigkeiten(ws: dict) -> Dict[str, Set[str]]:
    graph: Dict[str, Set[str]] = {name: set() for name in ws}
    for name, paket in ws.items():
        for dep in paket["dependencies"]:
            if dep.get("kind") is not None:  # dev/build ausschließen
                continue
            if dep.get("optional"):  # Feature-Abhängigkeiten ausschließen
                continue
            if dep["name"] in ws and dep["name"] != name:
                graph[name].add(dep["name"])
    return graph


def transitive(graph: Dict[str, Set[str]], start: str) -> Set[str]:
    gesehen = set()
    stapel = [start]
    while stapel:
        x = stapel.pop()
        for y in graph.get(x, set()):
            if y not in gesehen:
                gesehen.add(y)
                stapel.append(y)
    return gesehen


def zeilen_code(ws: dict, name: str) -> int:
    """Zählt Zeilen unter src/ (ohne Tests-Verzeichnisse)."""
    if name not in ws:
        return 0
    wurzel = os.path.join(os.path.dirname(ws[name]["manifest_path"]), "src")
    if not os.path.isdir(wurzel):
        return 0
    summe = 0
    for root, _, files in os.walk(wurzel):
        for f in files:
            if f.endswith(".rs"):
                with open(os.path.join(root, f), encoding="utf-8", errors="ignore") as fh:
                    summe += sum(1 for _ in fh)
    return summe


def main() -> int:
    parser = argparse.ArgumentParser(description="Abhängigkeits-Budget für Contextra-Crates (SPEC v4.1)")
    parser.add_argument("meta_or_crates", nargs="*", help="Optional meta.json gefolgt von Crate-Namen")
    parser.add_argument("--crate", "-c", action="append", help="Spezifisches Crate analysieren")
    parser.add_argument("--json", action="store_true", help="JSON-Format für Maschinenlesbarkeit ausgeben")
    parser.add_argument("--meta", help="Pfad zu bestehender meta.json")
    args = parser.parse_args()

    meta_file = args.meta
    crates_to_check: list[str] = []

    for item in args.meta_or_crates:
        if item.endswith(".json") and meta_file is None:
            meta_file = item
        else:
            crates_to_check.append(item)

    if args.crate:
        crates_to_check.extend(args.crate)

    try:
        meta = get_metadata(meta_file)
    except Exception as e:
        print(f"Fehler beim Laden von Cargo-Metadaten: {e}", file=sys.stderr)
        return 1

    ws = lade_workspace(meta)
    graph = abhaengigkeiten(ws)
    ziele = crates_to_check or sorted(ws)

    results = []
    for name in ziele:
        if name not in ws:
            continue
        huelle = transitive(graph, name) | {name}
        kern = zeilen_code(ws, name)
        gesamt = sum(zeilen_code(ws, n) for n in huelle)
        results.append({
            "crate": name,
            "kern_loc": kern,
            "transitive_loc": gesamt,
            "crates_count": len(huelle),
            "direct_dependencies": sorted(graph.get(name, set())),
            "transitive_dependencies": sorted(huelle - {name}),
        })

    if args.json:
        print(json.dumps(results, indent=2))
        return 0

    print(f"{'Crate':28s} {'Kern-LOC':>9s} {'mit Abh.-LOC':>13s} {'Crates':>7s}  direkte Abhängigkeiten")
    for r in results:
        print(f"{r['crate']:28s} {r['kern_loc']:9d} {r['transitive_loc']:13d} {r['crates_count']:7d}  {r['direct_dependencies']}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
