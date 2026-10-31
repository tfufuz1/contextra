"""Kommandozeile: `python -m ctxrs <befehl>` bzw. `python ctxrs.py <befehl>`."""
from __future__ import annotations

import argparse
import sys

from . import __version__
from .lib import ToolError, Workspace, find_root


def _apply_flags(p: argparse.ArgumentParser) -> None:
    p.add_argument("--apply", action="store_true", help="Änderungen schreiben (Standard: Dry-Run mit Diff)")
    p.add_argument("--no-diff", action="store_true", help="Im Dry-Run keinen Diff ausgeben (nur Zusammenfassung)")


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(prog="ctxrs", description="Contextra-Umstrukturierungswerkzeuge")
    p.add_argument("--repo", help="Repo-Wurzel (Standard: ab cwd aufwärts suchen)")
    p.add_argument("--version", action="version", version=f"ctxrs {__version__}")
    sub = p.add_subparsers(dest="cmd", required=True)

    s = sub.add_parser("doctor", help="Workspace-Diagnose ohne Cargo (D01–D12)")
    s.add_argument("--json", action="store_true")
    s.add_argument("--full", action="store_true", help="Fundlisten ungekürzt")
    s.add_argument("--no-src", action="store_true", help="Quelltext-Prüfungen (D10–D12) überspringen")
    s.add_argument("--show", default="KHMLI", help="anzuzeigende Schweregrade (Standard: KHMLI)")

    s = sub.add_parser("prune", help="Crates aus Workspace, Manifesten, Features und capabilities.toml entfernen")
    s.add_argument("crates", nargs="*", help="Paketnamen (oder --missing)")
    s.add_argument("--missing", action="store_true", help="alle Mitglieder/Pfad-Deps ohne Verzeichnis entfernen")
    s.add_argument("--features", choices=["auto", "drop", "keep-empty"], default="auto",
                   help="Umgang mit Features, die nur an entfernten Crates hingen (auto: leer behalten, wenn cfg(feature) im Quelltext)")
    s.add_argument("--delete-dirs", action="store_true", help="vorhandene Crate-Verzeichnisse mitlöschen (git rm -r)")
    s.add_argument("--full", action="store_true")
    _apply_flags(s)

    s = sub.add_parser("rename", help="Crate umbenennen (Verzeichnis, Paket, Deps, Rust-Identifier, Doku/CI)")
    s.add_argument("old"); s.add_argument("new")
    s.add_argument("--exclude", action="append", default=[], help="Pfadpräfix von der Textersetzung ausnehmen")
    _apply_flags(s)

    s = sub.add_parser("move", help="Crate-Verzeichnis verschieben (z. B. nach labs/) inkl. Pfad-Reparatur")
    s.add_argument("crate"); s.add_argument("dest", help="Zielverzeichnis, z. B. labs")
    s.add_argument("--exclude-default", action="store_true", help="aus default-members entfernen")
    _apply_flags(s)

    s = sub.add_parser("scaffold", help="Neues Crate nach SOLL-Vorlage anlegen")
    s.add_argument("name")
    s.add_argument("--ring", type=int); s.add_argument("--desc", default="")
    s.add_argument("--deps", default="", help="Komma-Liste interner Abhängigkeiten (Workspace-Schlüssel)")
    s.add_argument("--dev-deps", default="")
    s.add_argument("--bin", action="store_true", help="Binary statt Library")
    s.add_argument("--no-default-member", action="store_true")
    _apply_flags(s)

    s = sub.add_parser("ci-purge", help="xtask-Verweise in .cargo/config.toml, justfile, Workflows neutralisieren")
    s.add_argument("--delete-empty", action="store_true", help="Workflows ohne verbleibende Jobs löschen")
    s.add_argument("--emit-workflow", action="store_true", help=".github/workflows/ctxrs-gates.yml als Ersatz-Gate anlegen")
    s.add_argument("--tool-path", default="tools/ctxrs", help="Ablageort dieses Pakets im Repo (für --emit-workflow)")
    s.add_argument("--full", action="store_true")
    _apply_flags(s)

    s = sub.add_parser("archgate", help="SOLL-Architekturregeln R1–R7 + Wording-Lint mit Ratchet")
    s.add_argument("--baseline", default="governance/arch_baseline.json")
    s.add_argument("--init-baseline", action="store_true", help="Ist-Zustand als Baseline schreiben (einmalig)")
    s.add_argument("--shrink", action="store_true", help="Baseline auf den aktuellen (kleineren) Stand absenken")
    s.add_argument("--rules", default="", help="Komma-Liste, z. B. R1,R3 (Standard: alle)")
    s.add_argument("--json", action="store_true")
    s.add_argument("--full", action="store_true")
    s.add_argument("--blockers", action="store_true", help="Pro zu streichendem Crate die blockierenden Kanten zeigen")

    s = sub.add_parser("findings", help="Befund-Tracker: Prüfbericht-Befunde als Proben gegen den Quelltext")
    s.add_argument("--probes", help="eigene Probendatei (TOML)")
    s.add_argument("--json", action="store_true")
    s.add_argument("--only", default="", help="Komma-Liste Befund-IDs oder WP (z. B. S-01,N3)")
    s.add_argument("--open-only", action="store_true")
    s.add_argument("--snapshot", help="Ergebnis als JSON speichern; vergleichbar mit --compare")
    s.add_argument("--compare", help="Vergleich mit früherem Snapshot (zeigt Fortschritt/Regression)")

    s = sub.add_parser("status", help="Kurz-Dashboard: doctor + archgate + findings")
    s.add_argument("--baseline", default="governance/arch_baseline.json")
    return p


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    try:
        root = find_root(args.repo)
        ws = Workspace(root)
        if args.cmd == "doctor":
            from . import doctor
            return doctor.run(ws, args)
        if args.cmd == "prune":
            from . import prune
            return prune.run(ws, args)
        if args.cmd in ("rename", "move"):
            from . import relocate
            return relocate.run(ws, args)
        if args.cmd == "scaffold":
            from . import scaffold
            return scaffold.run(ws, args)
        if args.cmd == "ci-purge":
            from . import cipurge
            return cipurge.run(ws, args)
        if args.cmd == "archgate":
            from . import archgate
            return archgate.run(ws, args)
        if args.cmd == "findings":
            from . import findings
            return findings.run(ws, args)
        if args.cmd == "status":
            from . import status
            return status.run(ws, args)
    except ToolError as e:
        print(f"Fehler: {e}", file=sys.stderr)
        return 1
    except BrokenPipeError:
        return 0
    return 1


if __name__ == "__main__":
    sys.exit(main())

