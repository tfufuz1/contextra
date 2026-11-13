#!/usr/bin/env python3
"""
delete_16b_files.py — findet und entfernt Dateien mit exakt N Byte (Standard: 16).

Sicherheitsmechanismen:
  * Standard ist Dry-Run. Nur mit --apply wird etwas verändert.
  * Geschützte Pfade (Golden-Corpora, Fixtures, .git, target/) werden ausgelassen,
    außer --include-protected wird gesetzt. Platzhalter mit Inhalt
    "DELETED_CONTENT" werden dagegen immer gelöscht, unabhängig vom Pfad.
  * Jeder Kandidat wird auf Referenzen im Quelltext geprüft (Basisname kommt in
    .rs/.py/.toml/.sh/.json/.md vor). Referenzierte Dateien werden gemeldet.
  * --mode tombstone überschreibt statt zu löschen: Inhalt wird durch
    "DELETED_CONTENT\\n" ersetzt (genau 16 Byte), damit die Datei im Lieferpaket
    als Platzhalter erhalten bleibt.

Beispiele:
  python3 tools/delete_16b_files.py                       # nur auflisten
  python3 tools/delete_16b_files.py --apply               # ungeschützte löschen
  python3 tools/delete_16b_files.py --mode tombstone --apply
  python3 tools/delete_16b_files.py --manifest out.json   # Bericht schreiben
"""

from __future__ import annotations

import argparse
import fnmatch
import json
import sys
from pathlib import Path
from typing import List, Optional

TOMBSTONE = b"DELETED_CONTENT\n"  # 15 Zeichen + LF = 16 Byte

SKIP_DIRS = {".git", "target", "node_modules", ".venv", "__pycache__"}

PROTECTED_GLOBS = [
    "*/golden/*", "*/tests/golden/*", "*/fixtures/*", "*/tests/fixtures/*",
    "*/corpus/*", "*/tests/corpus/*", "*/testdata/*",
]

SOURCE_SUFFIXES = {".rs", ".py", ".toml", ".sh", ".json", ".md", ".yml", ".yaml", ".txt"}


def iter_files(root: Path) -> List[Path]:
    out: List[Path] = []
    stack = [root]
    while stack:
        d = stack.pop()
        try:
            entries = list(d.iterdir())
        except OSError:
            continue
        for p in entries:
            if p.is_symlink():
                continue
            if p.is_dir():
                if p.name in SKIP_DIRS:
                    continue
                stack.append(p)
            elif p.is_file():
                out.append(p)
    return sorted(out)


def is_protected(rel: str) -> bool:
    return any(fnmatch.fnmatch(rel, g) for g in PROTECTED_GLOBS)


def find_references(name: str, root: Path, exclude: Path) -> List[str]:
    """Sucht den Basisnamen in Quell-/Konfigurationsdateien (ohne die Datei selbst)."""
    hits: List[str] = []
    for p in iter_files(root):
        if p == exclude or p.suffix not in SOURCE_SUFFIXES:
            continue
        try:
            if name.encode() in p.read_bytes():
                hits.append(p.relative_to(root).as_posix())
        except OSError:
            continue
    return hits


def main(argv: Optional[List[str]] = None) -> int:
    ap = argparse.ArgumentParser(description="Dateien mit exakter Größe entfernen/ersetzen")
    ap.add_argument("--root", type=Path, default=Path.cwd(), help="Wurzelverzeichnis")
    ap.add_argument("--size", type=int, default=16, help="Zielgröße in Byte (Standard 16)")
    ap.add_argument("--mode", choices=["delete", "tombstone"], default="delete")
    ap.add_argument("--apply", action="store_true", help="Änderungen wirklich ausführen")
    ap.add_argument("--include-protected", action="store_true",
                    help="auch Golden-/Fixture-Pfade berücksichtigen (gefährlich)")
    ap.add_argument("--manifest", type=Path, default=None, help="JSON-Bericht schreiben")
    args = ap.parse_args(argv)

    root = args.root.resolve()
    candidates = []
    for p in iter_files(root):
        try:
            if p.stat().st_size != args.size:
                continue
        except OSError:
            continue
        rel = p.relative_to(root).as_posix()
        candidates.append((p, rel))

    report = []
    for p, rel in candidates:
        is_tomb = p.read_bytes() == TOMBSTONE
        protected = is_protected(rel) and not is_tomb  # Platzhalter sind nie geschützt
        refs = find_references(p.name, root, p) if not is_tomb else []
        if protected and not args.include_protected:
            action = "skip-protected"
        else:
            action = "tombstone" if args.mode == "tombstone" else "delete"
        if args.apply and action in ("delete", "tombstone"):
            try:
                if args.mode == "tombstone":
                    if p.read_bytes() != TOMBSTONE:
                        p.write_bytes(TOMBSTONE)
                    done = True
                else:
                    p.unlink()
                    done = True
            except OSError as exc:
                done, action = False, f"error: {exc}"
        else:
            done = False
        report.append({"path": rel, "size": args.size, "protected": protected,
                       "action": action, "applied": bool(args.apply and done),
                       "referenced_by": refs})

    for r in report:
        flag = " [GESCHÜTZT]" if r["protected"] else ""
        refs = f"  referenziert von: {', '.join(r['referenced_by'])}" if r["referenced_by"] else ""
        print(f"{r['action']:<16} {r['path']}{flag}{refs}")
    mode = "angewendet" if args.apply else "Dry-Run (kein --apply)"
    print(f"\n{len(report)} Kandidat(en) mit {args.size} Byte — {mode}")

    if args.manifest:
        args.manifest.write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding="utf-8")
        print(f"Bericht: {args.manifest}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
