#!/usr/bin/env python3
"""Abhängigkeits-Budget für Contextra-Crates.

Misst für jedes Workspace-Crate die Menge an Code, die man beim Entnehmen
mitkopieren müsste: das Crate selbst plus alle transitiv erreichbaren
internen Crates über normale, nicht-optionale Abhängigkeiten.

Dev-Abhängigkeiten und optionale Features werden bewusst NICHT mitgezählt,
weil sie beim Kopieren eines Bausteins nicht zwingend nötig sind.

Aufruf (im Repository-Root):
    cargo metadata --no-deps --format-version 1 > meta.json
    python3 -I abhaengigkeits_budget.py meta.json [crate ...]

Ohne Crate-Namen wird eine Tabelle für alle Workspace-Crates ausgegeben.
"""
import json
import os
import sys


def lade_workspace(pfad):
    meta = json.load(open(pfad, encoding="utf-8"))
    return {p["name"]: p for p in meta["packages"] if p["source"] is None}


def abhaengigkeiten(ws):
    graph = {name: set() for name in ws}
    for name, paket in ws.items():
        for dep in paket["dependencies"]:
            if dep.get("kind") is not None:  # dev/build ausschließen
                continue
            if dep.get("optional"):  # Feature-Abhängigkeiten ausschließen
                continue
            if dep["name"] in ws and dep["name"] != name:
                graph[name].add(dep["name"])
    return graph


def transitive(graph, start):
    gesehen = set()
    stapel = [start]
    while stapel:
        x = stapel.pop()
        for y in graph[x]:
            if y not in gesehen:
                gesehen.add(y)
                stapel.append(y)
    return gesehen


def zeilen_code(ws, name):
    """Zählt Zeilen unter src/ (ohne Tests-Verzeichnisse)."""
    wurzel = os.path.join(os.path.dirname(ws[name]["manifest_path"]), "src")
    summe = 0
    for root, _, files in os.walk(wurzel):
        for f in files:
            if f.endswith(".rs"):
                with open(os.path.join(root, f), encoding="utf-8", errors="ignore") as fh:
                    summe += sum(1 for _ in fh)
    return summe


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        sys.exit(2)
    ws = lade_workspace(sys.argv[1])
    graph = abhaengigkeiten(ws)
    ziele = sys.argv[2:] or sorted(ws)
    print(f"{'Crate':28s} {'Kern-LOC':>9s} {'mit Abh.-LOC':>13s} {'Crates':>7s}  direkte Abhängigkeiten")
    for name in ziele:
        if name not in ws:
            print(f"{name}: nicht im Workspace gefunden")
            continue
        hülle = transitive(graph, name) | {name}
        kern = zeilen_code(ws, name)
        gesamt = sum(zeilen_code(ws, n) for n in hülle)
        print(f"{name:28s} {kern:9d} {gesamt:13d} {len(hülle):7d}  {sorted(graph[name])}")


if __name__ == "__main__":
    main()
