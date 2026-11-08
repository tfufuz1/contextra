#!/usr/bin/env python3
"""
Historie Forensik & Cherry-Pick Navigator für Contextra

Ermöglicht das Abfragen, Filtern und Prüfen von historischen Modulzuständen,
Shell-Commits und leeren Commits im GitHub-Verlauf.
"""

import sys
import json
import os
import argparse

def load_data():
    base_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    json_path = os.path.join(base_dir, "docs", "HISTORIE_FORENSIK_AUDIT.json")
    if not os.path.exists(json_path):
        print(f"Fehler: {json_path} nicht gefunden.")
        sys.exit(1)
    with open(json_path, "r", encoding="utf-8") as f:
        return json.load(f)

def main():
    parser = argparse.ArgumentParser(description="Contextra History Forensics & Cherry-Pick Tool")
    parser.add_argument("--modules", action="store_true", help="Zeigt reifste Zustände aller Module an")
    parser.add_argument("--shell-commits", action="store_true", help="Listet analysierte Shell-Commits auf")
    parser.add_argument("--empty-commits", action="store_true", help="Listet identifizierte leere Commits auf")
    parser.add_argument("--cherry-pick", metavar="MODULE", help="Gibt Cherry-Pick Befehl für ein Modul aus")
    parser.add_argument("--json", action="store_true", help="Ausgabe im JSON-Format")
    args = parser.parse_args()

    data = load_data()

    if args.cherry_pick:
        target = args.cherry_pick.lower()
        found = [m for m in data["modules"] if target in m["name"].lower()]
        if not found:
            print(f"Kein Modul gefunden für: {args.cherry_pick}")
            sys.exit(1)
        for m in found:
            print(f"Modul: {m['name']}")
            print(f"  Reifster Commit: {m.get('mature_hash')}")
            print(f"  Cherry-Pick: git cherry-pick -n {m.get('mature_hash')}")
            print(f"  Checkout:    git checkout {m.get('mature_hash')} -- {m['path']}")
        return

    if args.modules:
        if args.json:
            print(json.dumps(data["modules"], indent=2))
        else:
            print(f"{'Modul':<22} | {'Reifster Hash':<10} | {'Schnitt-Hash':<12} | {'Beschreibung'}")
            print("-" * 80)
            for m in data["modules"]:
                print(f"{m['name']:<22} | {m.get('mature_short', 'N/A'):<10} | {m.get('removal_short', 'N/A'):<12} | {m['desc'][:40]}")
        return

    if args.shell_commits:
        if args.json:
            print(json.dumps(data["shell_commits"], indent=2))
        else:
            print(f"Total Shell-Commits: {len(data['shell_commits'])}")
            for s in data["shell_commits"][:20]:
                print(f"{s['short_hash']} | {s['date']} | {s['category']:<15} | {s['precise_message']}")
        return

    if args.empty_commits:
        if args.json:
            print(json.dumps(data["empty_commits"], indent=2))
        else:
            print(f"Total Empty Commits: {len(data['empty_commits'])}")
            for e in data["empty_commits"][:20]:
                print(f"{e['short_hash']} | Parent: {e['parent_hash']} | {e['subject'][:50]}")
        return

    parser.print_help()

if __name__ == "__main__":
    main()
