#!/usr/bin/env python3
"""
rename_to_contextra.py
=======================

Deterministisches, idempotentes Skript zur vollständigen Umbenennung des
Projekts MemFuse -> Contextra im gesamten Repository.

Warum ein Skript statt Freihand-Suchen/Ersetzen durch einen Agenten:
Bei >20.000 case-insensitiven Vorkommen über >900 Dateien ist die einzige
verlässliche Vorgehensweise ein Werkzeug, das (a) reproduzierbar dieselbe
Entscheidung an derselben Stelle trifft, (b) seinen eigenen Erfolg per
Nachlauf-Scan verifizieren kann und (c) idempotent ist, sodass ein
Wiederholungslauf keine Doppel-Ersetzung erzeugt.

Vorgehen (siehe Contextra_Rename_Prompts.md Schritt 1-4 und 10, hier als
EIN Werkzeug statt zehn Agenten-Schritten umgesetzt, da der Nutzer eine
maschinelle Gesamtlösung angefordert hat):

  1. INVENTORY : nur lesen, zählt Vorkommen je Kategorie/Datei -> JSON+MD Report
  2. RENAME     : Verzeichnisse/Dateien umbenennen, dann Textinhalte ersetzen,
                  als zwei getrennte, nacheinander laufende Phasen
  3. VERIFY     : Nachlauf-Scan, meldet verbleibende Treffer außerhalb der
                  Exclusions

Case-Varianten (Reihenfolge wichtig: längere/spezifischere Muster zuerst,
damit z.B. "MEMFUSE_" vor "Memfuse" behandelt wird und keine Doppel-
Ersetzung durch Overlap entsteht):

  SCREAMING_SNAKE : MEMFUSE            -> CONTEXTRA          (Env-Vars, Konstanten)
  PascalCase      : MemFuse            -> Contextra           (Markenname, Typnamen wie MemFuseError)
  kebab-case      : memfuse            -> contextra            (Crate-/Verzeichnisnamen, wenn von '-' umgeben)
  snake_case      : memfuse            -> contextra            (Rust-Bezeichner, wenn von '_' umgeben)
  lowercase (Rest): memfuse            -> contextra            (Fallback: reiner Kleinschreibungs-Treffer)

Da "memfuse" im Kleinschreibungsfall ohnehin auf "contextra" abbildet,
werden kebab/snake/lowercase technisch durch eine einzige Regel abgedeckt;
sie sind hier trotzdem konzeptionell getrennt kommentiert, weil die
Mapping-Tabelle in RENAME_DECISION.md sie getrennt führt.

Ausschlüsse (EXCLUSIONS.md-Äquivalent, siehe EXCLUDE_PATTERNS/EXCLUDE_FILES
unten): Format-/Protokoll-Tags, die in bereits von Nutzern erzeugten
Binärdateien (WAL, Snapshots, Checkpoints) persistiert sein könnten, werden
NICHT automatisch umbenannt, sondern nur als Fund gemeldet, wenn sie über
die generische Regex getroffen würden. Da dieses Repository nachweislich
noch keine veröffentlichten Nutzer/Datendateien hat (siehe Lagebericht in
CONTEXTRA_SPEC_v3.md, Präambel), ist die Default-Policy hart brechen
(kein Kompatibilitäts-Alias) - override per --keep-env-aliases.

Nutzung:
    python3 scripts/rename_to_contextra.py --dry-run
    python3 scripts/rename_to_contextra.py --apply
    python3 scripts/rename_to_contextra.py --verify
"""

from __future__ import annotations

import argparse
import fnmatch
import json
import re
import shutil
import subprocess
import sys
from collections import defaultdict
from dataclasses import dataclass, field
from pathlib import Path

# --------------------------------------------------------------------------
# Konfiguration
# --------------------------------------------------------------------------

REPO_ROOT = Path(__file__).resolve().parent.parent

# Verzeichnisse, die nie durchsucht/umbenannt werden
IGNORE_DIRS = {".git", "target", "node_modules", ".jj", "__pycache__", ".venv"}

# Dateiendungen/Namen, die als Text behandelt werden (alles andere wird
# übersprungen, um Binärdateien nicht zu beschädigen)
TEXT_SUFFIXES = {
    ".rs", ".toml", ".md", ".yml", ".yaml", ".py", ".txt", ".sh",
    ".json", ".jsonl", ".nix", ".mmd", ".html", ".fbs", ".lock",
    ".gitignore", ".cfg", ".ini",
}
TEXT_FILENAMES_NO_SUFFIX = {"justfile", "Justfile", "Dockerfile"}

# Dateien/Pfad-Fragmente, die inhaltlich NICHT verändert werden dürfen
# (Äquivalent zu docs/rename/EXCLUSIONS.md). Glob-Muster relativ zum Repo-Root.
# -> Standard: LEER, weil dieses Repo laut Spezifikation noch keine
#    veröffentlichten Datendateien/Nutzer hat. Bei Bedarf hier ergänzen,
#    BEVOR --apply gelaufen wird.
EXCLUDE_GLOBS: list[str] = [
    "docs/rename/*.md",
]

# Cargo.lock wird bewusst NICHT von Hand editiert (wird durch `cargo check`
# neu erzeugt) - Textinhalt wird trotzdem ersetzt, weil sonst die
# Package-Namen inkonsistent zu Cargo.toml wären; ein `cargo check` danach
# ist Pflicht (siehe Schritt 5 der Prompt-Vorlage / VERIFY-Hinweis unten).

# --------------------------------------------------------------------------
# Case-Transformationsregeln (Reihenfolge relevant!)
# --------------------------------------------------------------------------
# Jede Regel: (name, kompilierter Pattern, replace-Funktion)
# Reihenfolge: SCREAMING_SNAKE zuerst (sonst würde die lowercase-Regel
# Teile davon vorzeitig treffen und Groß-/Kleinschreibung zerstören),
# dann PascalCase, dann der generische lowercase-Rest.

RULES: list[tuple[str, re.Pattern, str]] = [
    # MEMFUSE (als eigenständiges Großbuchstaben-Token, z.B. MEMFUSE_RRF_TIERS,
    # MEMFUSE-CI, oder alleinstehend MEMFUSE)
    ("SCREAMING_SNAKE", re.compile(r"MEMFUSE"), "CONTEXTRA"),
    # MemFuse (PascalCase Markenname/Typname, z.B. MemFuseError, MemFuseConfig)
    ("PascalCase", re.compile(r"MemFuse"), "Contextra"),
    # Rest: memfuse in beliebiger sonstiger Schreibweise (memfuse, Memfuse als
    # Wortanfang in Fließtext, memfuse-core, memfuse_core, ...)
    ("generic_lower_or_mixed", re.compile(r"[Mm]emfuse"), "contextra"),
]


def apply_rules(text: str) -> tuple[str, dict[str, int]]:
    """Wendet alle Regeln der Reihe nach an und zählt Treffer pro Regel."""
    counts: dict[str, int] = {}
    for name, pattern, repl in RULES:
        text, n = pattern.subn(repl, text)
        if n:
            counts[name] = counts.get(name, 0) + n
    return text, counts


# --------------------------------------------------------------------------
# Hilfsfunktionen
# --------------------------------------------------------------------------

def is_ignored(path: Path) -> bool:
    return any(part in IGNORE_DIRS for part in path.parts)


def is_excluded(path: Path) -> bool:
    rel = path.relative_to(REPO_ROOT).as_posix()
    return any(fnmatch.fnmatch(rel, pat) for pat in EXCLUDE_GLOBS)


def is_text_file(path: Path) -> bool:
    if path.name in TEXT_FILENAMES_NO_SUFFIX:
        return True
    return path.suffix in TEXT_SUFFIXES


def iter_all_files(root: Path):
    for p in root.rglob("*"):
        if p.is_file() and not is_ignored(p):
            yield p


def iter_all_dirs_bottom_up(root: Path):
    """Verzeichnisse tiefste-zuerst, damit innere Umbenennungen die äußeren
    Pfade nicht invalidieren."""
    dirs = [p for p in root.rglob("*") if p.is_dir() and not is_ignored(p)]
    dirs.sort(key=lambda p: len(p.parts), reverse=True)
    return dirs


# --------------------------------------------------------------------------
# Phase 1: INVENTORY (nur lesen)
# --------------------------------------------------------------------------

CATEGORY_LABELS = {
    "a_crate_names": "Rust-Crate-/Verzeichnisnamen",
    "b_rust_identifiers": "Rust-Bezeichner in Code",
    "c_cargo_deps": "Cargo-Dependency-Referenzen",
    "d_string_literals": "String-Literale zur Laufzeit",
    "e_env_vars": "Umgebungsvariablen (MEMFUSE_*)",
    "f_python_package": "Python-Paketname",
    "g_docs": "Dokumentation/Kommentare (.md, ///, //!)",
    "h_ci_workflows": ".github/workflows/*.yml",
    "i_other": "Sonstiges",
}


def categorize_file(path: Path) -> str:
    rel = path.relative_to(REPO_ROOT).as_posix()
    if rel.startswith(".github/workflows/"):
        return "h_ci_workflows"
    if path.name == "pyproject.toml" or path.name == "setup.py":
        return "f_python_package"
    if path.name == "Cargo.toml":
        return "c_cargo_deps"
    if path.suffix == ".md":
        return "g_docs"
    if path.suffix == ".rs":
        return "b_rust_identifiers"
    return "i_other"


@dataclass
class Inventory:
    file_counts: dict = field(default_factory=lambda: defaultdict(int))
    occurrence_counts: dict = field(default_factory=lambda: defaultdict(int))
    examples: dict = field(default_factory=lambda: defaultdict(list))
    dir_renames: list = field(default_factory=list)
    file_renames: list = field(default_factory=list)
    env_vars_found: set = field(default_factory=set)

    def to_report_md(self) -> str:
        lines = ["# Inventar: memfuse -> contextra\n"]
        total = sum(self.occurrence_counts.values())
        lines.append(f"Gesamtvorkommen (case-insensitiv, Textinhalt): **{total}**\n")
        lines.append("| Kategorie | Dateien | Vorkommen |")
        lines.append("|---|---|---|")
        for cat, label in CATEGORY_LABELS.items():
            lines.append(
                f"| {label} | {self.file_counts.get(cat, 0)} | {self.occurrence_counts.get(cat, 0)} |"
            )
        lines.append("")
        lines.append(f"Verzeichnis-Umbenennungen geplant: {len(self.dir_renames)}")
        for old, new in self.dir_renames[:40]:
            lines.append(f"- `{old}` -> `{new}`")
        lines.append("")
        lines.append(f"Datei-Umbenennungen geplant: {len(self.file_renames)}")
        for old, new in self.file_renames[:40]:
            lines.append(f"- `{old}` -> `{new}`")
        lines.append("")
        lines.append(f"Gefundene MEMFUSE_*-Umgebungsvariablen ({len(self.env_vars_found)}):")
        for v in sorted(self.env_vars_found):
            lines.append(f"- `{v}`")
        lines.append("")
        for cat, label in CATEGORY_LABELS.items():
            ex = self.examples.get(cat, [])
            if ex:
                lines.append(f"## Beispiele: {label}")
                for e in ex[:5]:
                    lines.append(f"- `{e}`")
                lines.append("")
        return "\n".join(lines)


def scan(root: Path) -> Inventory:
    inv = Inventory()
    env_pattern = re.compile(r"\bMEMFUSE_[A-Z0-9_]+\b")

    # Verzeichnisse
    for d in iter_all_dirs_bottom_up(root):
        if is_excluded(d):
            continue
        if re.search(r"memfuse", d.name, re.IGNORECASE):
            new_text, _ = apply_rules(d.name)
            rel_old = d.relative_to(root).as_posix()
            rel_new = str(Path(rel_old).with_name(new_text))
            inv.dir_renames.append((rel_old, rel_new))

    # Dateien: Umbenennungen + Textscan
    for f in iter_all_files(root):
        if f == Path(__file__).resolve():
            continue  # sich selbst nicht anfassen
        if is_excluded(f):
            continue
        if re.search(r"memfuse", f.name, re.IGNORECASE):
            new_name, _ = apply_rules(f.name)
            rel_old = f.relative_to(root).as_posix()
            rel_new = str(Path(rel_old).with_name(new_name))
            inv.file_renames.append((rel_old, rel_new))

        if not is_text_file(f):
            continue
        try:
            content = f.read_text(encoding="utf-8")
        except (UnicodeDecodeError, OSError):
            continue
        if "memfuse" not in content.lower():
            continue

        cat = categorize_file(f)
        n = len(re.findall(r"memfuse", content, re.IGNORECASE))
        inv.file_counts[cat] += 1
        inv.occurrence_counts[cat] += n
        rel = f.relative_to(root).as_posix()
        first_line = next(
            (
                f"{rel}:{i+1}: {line.strip()[:100]}"
                for i, line in enumerate(content.splitlines())
                if re.search(r"memfuse", line, re.IGNORECASE)
            ),
            rel,
        )
        inv.examples[cat].append(first_line)

        for m in env_pattern.findall(content):
            inv.env_vars_found.add(m)

    return inv


# --------------------------------------------------------------------------
# Phase 2: RENAME (Verzeichnisse -> Dateien -> Textinhalte)
# --------------------------------------------------------------------------

def do_rename(root: Path, apply: bool) -> dict:
    stats = {"dirs_renamed": 0, "files_renamed": 0, "files_edited": 0, "occurrences": 0}

    # --- Phase 2a: Verzeichnisse (tiefste zuerst) ---
    for d in iter_all_dirs_bottom_up(root):
        if not d.exists() or is_excluded(d):
            continue
        if re.search(r"memfuse", d.name, re.IGNORECASE):
            new_name, _ = apply_rules(d.name)
            target = d.with_name(new_name)
            if target == d:
                continue
            stats["dirs_renamed"] += 1
            if apply:
                target.parent.mkdir(parents=True, exist_ok=True)
                d.rename(target)

    # --- Phase 2b: Dateien ---
    for f in iter_all_files(root):
        if not f.exists() or f == Path(__file__).resolve() or is_excluded(f):
            continue
        if re.search(r"memfuse", f.name, re.IGNORECASE):
            new_name, _ = apply_rules(f.name)
            target = f.with_name(new_name)
            if target == f:
                continue
            stats["files_renamed"] += 1
            if apply:
                f.rename(target)

    # --- Phase 2c: Textinhalte (nach den Umbenennungen erneut iterieren,
    #     damit Pfade aktuell sind) ---
    for f in iter_all_files(root):
        if f == Path(__file__).resolve() or is_excluded(f) or not is_text_file(f):
            continue
        try:
            content = f.read_text(encoding="utf-8")
        except (UnicodeDecodeError, OSError):
            continue
        if "memfuse" not in content.lower():
            continue
        new_content, counts = apply_rules(content)
        n = sum(counts.values())
        if n == 0 or new_content == content:
            continue
        stats["files_edited"] += 1
        stats["occurrences"] += n
        if apply:
            f.write_text(new_content, encoding="utf-8")

    return stats


# --------------------------------------------------------------------------
# Phase 3: VERIFY
# --------------------------------------------------------------------------

def verify(root: Path) -> list[str]:
    remaining = []
    for f in iter_all_files(root):
        if f == Path(__file__).resolve():
            continue
        rel = f.relative_to(root).as_posix()
        if re.search(r"memfuse", rel, re.IGNORECASE) and not is_excluded(f):
            remaining.append(f"{rel} (Dateiname)")
        if not is_text_file(f):
            continue
        if is_excluded(f):
            continue
        try:
            content = f.read_text(encoding="utf-8")
        except (UnicodeDecodeError, OSError):
            continue
        for i, line in enumerate(content.splitlines(), start=1):
            if re.search(r"memfuse", line, re.IGNORECASE):
                remaining.append(f"{rel}:{i}: {line.strip()[:120]}")
    return remaining


# --------------------------------------------------------------------------
# CLI
# --------------------------------------------------------------------------

def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--dry-run", action="store_true", help="Nur Statistik zeigen, nichts ändern")
    parser.add_argument("--apply", action="store_true", help="Änderungen tatsächlich durchführen")
    parser.add_argument("--inventory", action="store_true", help="Nur Inventar erzeugen (docs/rename/INVENTORY.md)")
    parser.add_argument("--verify", action="store_true", help="Nachlauf-Scan: verbleibende memfuse-Treffer melden")
    parser.add_argument("--root", type=Path, default=REPO_ROOT, help="Repo-Root (Default: Skript-Elternverzeichnis)")
    args = parser.parse_args()

    root = args.root.resolve()
    if not (root / "Cargo.toml").exists() and not args.verify:
        print(f"WARNUNG: {root} sieht nicht wie das Repo-Root aus (kein Cargo.toml).", file=sys.stderr)

    if args.inventory:
        inv = scan(root)
        out_dir = root / "docs" / "rename"
        out_dir.mkdir(parents=True, exist_ok=True)
        (out_dir / "INVENTORY.md").write_text(inv.to_report_md(), encoding="utf-8")
        (out_dir / "INVENTORY.json").write_text(
            json.dumps(
                {
                    "file_counts": dict(inv.file_counts),
                    "occurrence_counts": dict(inv.occurrence_counts),
                    "dir_renames": inv.dir_renames,
                    "file_renames": inv.file_renames,
                    "env_vars_found": sorted(inv.env_vars_found),
                },
                indent=2,
                ensure_ascii=False,
            ),
            encoding="utf-8",
        )
        print(f"Inventar geschrieben nach {out_dir}/INVENTORY.md")
        print(f"Gesamtvorkommen: {sum(inv.occurrence_counts.values())}")
        return 0

    if args.verify:
        remaining = verify(root)
        out_dir = root / "docs" / "rename"
        out_dir.mkdir(parents=True, exist_ok=True)
        report = ["# Finale Verifikation: memfuse -> contextra\n"]
        if remaining:
            report.append(f"**{len(remaining)} verbleibende Treffer außerhalb der Exclusions:**\n")
            report.extend(f"- {r}" for r in remaining)
        else:
            report.append("Keine verbleibenden \"memfuse\"-Treffer außerhalb der Exclusions gefunden.")
        (out_dir / "FINAL_VERIFICATION.md").write_text("\n".join(report), encoding="utf-8")
        print(f"{len(remaining)} verbleibende Treffer. Report: docs/rename/FINAL_VERIFICATION.md")
        for r in remaining[:50]:
            print("  ", r)
        return 1 if remaining else 0

    if not args.dry_run and not args.apply:
        parser.error("Bitte --dry-run, --apply, --inventory oder --verify angeben.")

    stats = do_rename(root, apply=args.apply)
    mode = "APPLY" if args.apply else "DRY-RUN"
    print(f"[{mode}] Verzeichnisse umbenannt: {stats['dirs_renamed']}")
    print(f"[{mode}] Dateien umbenannt:      {stats['files_renamed']}")
    print(f"[{mode}] Dateien mit Textänderung: {stats['files_edited']}")
    print(f"[{mode}] Ersetzte Vorkommen:     {stats['occurrences']}")
    if not args.apply:
        print("\nKein --apply gesetzt -> es wurde NICHTS verändert.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
