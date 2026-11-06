#!/usr/bin/env python3
"""
decouple_inspector.py — Präzisions-Diagnosewerkzeug für Coding-Agenten & Architekten.

Zweck:
Untersucht Rust-Crates auf dem Weg zu einem rein modularen Baukastensystem.
Zeigt direkte Verdrahtungen, interne/externe Schnittstellen, ungenutzten Code,
verstreute Löschketten-/DSGVO-Relikte außerhalb von `contextra-erasure`
sowie fehlende Port-/Trait-Abstraktionen auf.

Verwendung:
    python3 tools/decouple_inspector.py --help
    python3 tools/decouple_inspector.py --crate contextra-store
    python3 tools/decouple_inspector.py --all --json
    python3 tools/decouple_inspector.py --erasure-leak-check
"""

from __future__ import annotations

import argparse
import json
import os
import re
import sys
from collections import defaultdict
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Dict, List, Optional, Set, Tuple

# Regex-Muster für Rust-Codeanalyse
RE_USE = re.compile(r'^\s*(?:pub\s+)?use\s+([^;]+);', re.MULTILINE)
RE_PUB_ITEM = re.compile(
    r'^\s*pub(?:\([^)]+\))?\s+(trait|struct|enum|fn|type|const|static)\s+([A-Za-z0-9_]+)',
    re.MULTILINE,
)
RE_FN_DEF = re.compile(
    r'^\s*(?:pub(?:\([^)]+\))?\s+)?(?:async\s+)?(?:unsafe\s+)?fn\s+([A-Za-z0-9_]+)',
    re.MULTILINE,
)
RE_IDENT = re.compile(r'\b[A-Za-z_][A-Za-z0-9_]*\b')

# Suchbegriffe für DSGVO, Löschkette, Deletion-Proofs, AVV etc.
ERASURE_TERMS = [
    "dsgvo",
    "gdpr",
    "löschung",
    "loeschung",
    "löschkette",
    "loeschkette",
    "erasure_journal",
    "deletion_proof",
    "deletionproof",
    "erasurereceipt",
    "erasure_receipt",
    "avv",
    "purger",
    "witness_scan",
    "sovereign_erasure",
]

# Crates, die legal DSGVO / Erasure enthalten dürfen:
ERASURE_EXEMPT_CRATES = {
    "contextra-erasure",
}


@dataclass
class CrateInventory:
    name: str
    path: str
    cargo_path: str
    is_core: bool = False
    dependencies_prod: Set[str] = field(default_factory=set)
    dependencies_dev: Set[str] = field(default_factory=set)
    pub_items: Dict[str, List[str]] = field(default_factory=lambda: defaultdict(list))
    all_fn_defs: Set[str] = field(default_factory=set)
    imported_symbols: Dict[str, Set[str]] = field(default_factory=lambda: defaultdict(set))
    used_identifiers: Set[str] = field(default_factory=set)
    erasure_leaks: List[Dict[str, Any]] = field(default_factory=list)
    direct_coupling_points: List[Dict[str, Any]] = field(default_factory=list)
    loc: int = 0
    file_count: int = 0


def strip_comments_and_strings(source: str) -> str:
    """Entfernt Kommentare und Strings, um Token-Zählungen nicht zu verfälschen."""
    out = []
    i = 0
    n = len(source)
    in_block = False
    in_str = False
    in_char = False

    while i < n:
        if in_block:
            if source[i:i+2] == '*/':
                in_block = False
                i += 2
                out.append('  ')
            else:
                out.append(' ' if source[i] != '\n' else '\n')
                i += 1
        elif in_str:
            if source[i] == '\\':
                out.append('  ')
                i += 2
            elif source[i] == '"':
                in_str = False
                out.append(' ')
                i += 1
            else:
                out.append(' ' if source[i] != '\n' else '\n')
                i += 1
        elif in_char:
            if source[i] == '\\':
                out.append('  ')
                i += 2
            elif source[i] == '\'':
                in_char = False
                out.append(' ')
                i += 1
            else:
                out.append(' ' if source[i] != '\n' else '\n')
                i += 1
        else:
            if source[i:i+2] == '/*':
                in_block = True
                i += 2
                out.append('  ')
            elif source[i:i+2] == '//':
                end = source.find('\n', i)
                if end == -1:
                    out.append(' ' * (n - i))
                    break
                else:
                    out.append(' ' * (end - i))
                    i = end
            elif source[i] == '"':
                in_str = True
                out.append(' ')
                i += 1
            elif source[i] == '\'':
                if i + 2 < n and source[i+2] == '\'':
                    in_char = True
                    out.append(' ')
                    i += 1
                elif i + 3 < n and source[i+1] == '\\' and source[i+3] == '\'':
                    in_char = True
                    out.append(' ')
                    i += 1
                else:
                    out.append(source[i])
                    i += 1
            else:
                out.append(source[i])
                i += 1

    return "".join(out)


def parse_cargo_toml(cargo_path: Path) -> Tuple[Set[str], Set[str]]:
    """Parst direkte interne Workspace-Abhängigkeiten aus Cargo.toml."""
    prod_deps = set()
    dev_deps = set()
    if not cargo_path.exists():
        return prod_deps, dev_deps

    current_section = ""
    with open(cargo_path, 'r', encoding='utf-8', errors='ignore') as f:
        for line in f:
            line_str = line.strip()
            if not line_str or line_str.startswith("#"):
                continue
            if line_str.startswith("[") and line_str.endswith("]"):
                current_section = line_str[1:-1].strip()
                continue

            if "=" in line_str:
                dep_name = line_str.split("=")[0].strip()
                if dep_name.startswith("contextra") or "crates/contextra" in line_str:
                    if "dev-dependencies" in current_section:
                        dev_deps.add(dep_name)
                    elif "dependencies" in current_section:
                        prod_deps.add(dep_name)

    return prod_deps, dev_deps


def inspect_crate(crate_dir: Path, workspace_root: Path) -> CrateInventory:
    """Führt eine Tiefeninspektion eines einzelnen Crates durch."""
    crate_name = crate_dir.name
    cargo_path = crate_dir / "Cargo.toml"
    prod_deps, dev_deps = parse_cargo_toml(cargo_path)

    inv = CrateInventory(
        name=crate_name,
        path=str(crate_dir.relative_to(workspace_root)),
        cargo_path=str(cargo_path.relative_to(workspace_root)) if cargo_path.exists() else "",
        dependencies_prod=prod_deps,
        dependencies_dev=dev_deps,
    )

    src_dir = crate_dir / "src"
    if not src_dir.exists():
        return inv

    for root, dirs, files in os.walk(src_dir):
        for f in files:
            if not f.endswith(".rs"):
                continue

            f_path = Path(root) / f
            rel_file = str(f_path.relative_to(workspace_root))
            try:
                raw_code = f_path.read_text(encoding="utf-8", errors="ignore")
            except Exception:
                continue

            inv.file_count += 1
            inv.loc += len([line for line in raw_code.splitlines() if line.strip() and not line.strip().startswith("//")])

            # 1. DSGVO / Löschketten-Lecks ermitteln
            if crate_name not in ERASURE_EXEMPT_CRATES:
                lower_code = raw_code.lower()
                for line_no, line in enumerate(lower_code.splitlines(), start=1):
                    for term in ERASURE_TERMS:
                        if term in line:
                            inv.erasure_leaks.append({
                                "file": rel_file,
                                "line": line_no,
                                "term": term,
                                "snippet": raw_code.splitlines()[line_no - 1].strip()[:100],
                            })
                            break

            clean_code = strip_comments_and_strings(raw_code)

            # 2. Öffentliche Items katalogisieren
            for kind, item_name in RE_PUB_ITEM.findall(clean_code):
                inv.pub_items[kind].append(f"{rel_file}::{item_name}")

            # 3. Alle Funktionsdefinitionen sammeln
            for fn_name in RE_FN_DEF.findall(clean_code):
                inv.all_fn_defs.add(fn_name)

            # 4. Importierte Symbole und direkte Kopplungen erkennen
            for match in RE_USE.finditer(clean_code):
                use_stmt = match.group(1).strip()
                for dep in prod_deps:
                    crate_ident = dep.replace("-", "_")
                    if crate_ident in use_stmt or dep in use_stmt:
                        inv.direct_coupling_points.append({
                            "file": rel_file,
                            "target_crate": dep,
                            "statement": use_stmt,
                        })
                        for token in RE_IDENT.findall(use_stmt):
                            if token != crate_ident and token not in ("pub", "use", "self", "crate", "super"):
                                inv.imported_symbols[dep].add(token)

            # 5. Genutzte Identifikatoren sammeln
            tokens = set(RE_IDENT.findall(clean_code))
            inv.used_identifiers.update(tokens)

    return inv


def evaluate_crate_decoupling(inv: CrateInventory) -> Dict[str, Any]:
    """Bewertet die Entkopplungsgüte und generiert konkrete Handlungsschritte für Agenten."""
    findings = []
    recommendations = []

    high_coupling_deps = []
    for dep, syms in inv.imported_symbols.items():
        if len(syms) > 8:
            high_coupling_deps.append((dep, len(syms)))
            findings.append(f"Starke direkte Kopplung an '{dep}' ({len(syms)} importierte Symbole).")
            recommendations.append(
                f"Abhängigkeit von '{dep}' über Traits in 'contextra-ports' entkoppeln "
                f"oder als generische Typ-Parameter injizieren."
            )

    if inv.erasure_leaks:
        findings.append(
            f"{len(inv.erasure_leaks)} Referenzen auf DSGVO/Löschkette/Nachweise gefunden. "
            f"Diese gehören ausschließlich in 'contextra-erasure'."
        )
        recommendations.append(
            f"DSGVO-/Löschlogik aus {inv.name} extrahieren und in 'contextra-erasure' modularisieren."
        )

    score = 100
    score -= min(40, len(inv.dependencies_prod) * 6)
    score -= min(30, len(inv.erasure_leaks) * 5)
    score -= min(20, len(high_coupling_deps) * 5)
    score = max(0, score)

    status = "Exzellent (Baustein)" if score >= 80 else ("Akzeptabel" if score >= 60 else "Stark verdrahtet (Refactoring nötig)")

    return {
        "crate": inv.name,
        "score": score,
        "status": status,
        "loc": inv.loc,
        "files": inv.file_count,
        "dependencies_count": len(inv.dependencies_prod),
        "dependencies": sorted(list(inv.dependencies_prod)),
        "findings": findings,
        "recommendations": recommendations,
        "erasure_leaks_count": len(inv.erasure_leaks),
        "erasure_leaks": inv.erasure_leaks[:15],
        "direct_coupling_points_count": len(inv.direct_coupling_points),
        "coupling_summary": {dep: len(syms) for dep, syms in inv.imported_symbols.items()},
    }


def main() -> int:
    parser = argparse.ArgumentParser(
        description="decouple_inspector: Präzisionswerkzeug zur Analyse und Entkopplung von Rust-Workspace-Crates."
    )
    parser.add_argument("--root", default=".", help="Workspace-Root-Verzeichnis")
    parser.add_argument("--crate", "-c", help="Nur ein bestimmtes Crate analysieren (z. B. contextra-store)")
    parser.add_argument("--all", action="store_true", help="Alle Crates im Workspace analysieren")
    parser.add_argument("--json", action="store_true", help="Ausgabe als JSON für Agenten-Pipelines")
    parser.add_argument("--erasure-leak-check", action="store_true", help="Nur nach verstreuten Löschketten-/DSGVO-Relikten suchen")
    args = parser.parse_args()

    root = Path(args.root).resolve()
    crates_dir = root / "crates"

    if not crates_dir.exists():
        print(f"Fehler: Verzeichnis 'crates' unter {root} nicht gefunden.", file=sys.stderr)
        return 1

    crate_dirs = []
    if args.crate:
        target = crates_dir / args.crate
        if not target.exists():
            print(f"Fehler: Crate '{args.crate}' nicht gefunden.", file=sys.stderr)
            return 1
        crate_dirs.append(target)
    else:
        crate_dirs = sorted([d for d in crates_dir.iterdir() if d.is_dir() and (d / "Cargo.toml").exists()])

    inspections = []
    evaluations = []

    for c_dir in crate_dirs:
        inv = inspect_crate(c_dir, root)
        inspections.append(inv)
        evaluations.append(evaluate_crate_decoupling(inv))

    if args.json:
        print(json.dumps(evaluations, indent=2, ensure_ascii=False))
        return 0

    if args.erasure_leak_check:
        print("=" * 80)
        print("LÖSCHKETTEN- & DSGVO-LEAK AUDIT (Alles außerhalb von 'contextra-erasure')")
        print("=" * 80)
        total_leaks = 0
        for ev in evaluations:
            leaks = ev.get("erasure_leaks", [])
            if leaks:
                print(f"\n📦 Crate: {ev['crate']} ({ev['erasure_leaks_count']} Fundstellen)")
                for leak in leaks:
                    print(f"   - {leak['file']}:{leak['line']} [{leak['term']}]: {leak['snippet']}")
                total_leaks += ev["erasure_leaks_count"]
        print("-" * 80)
        print(f"Gesamtanzahl Fundstellen: {total_leaks}")
        return 0

    print("=" * 80)
    print("CONTEXTRA MODULARITÄTS- & ENTKOPPLUNGS-INSPEKTOR")
    print("=" * 80)

    for ev in evaluations:
        print(f"\n📦 Crate: {ev['crate']}")
        print(f"   Status: {ev['status']} (Modularitäts-Score: {ev['score']}/100)")
        print(f"   Umfang: {ev['loc']} LOC in {ev['files']} Dateien")
        print(f"   Interne Abhängigkeiten ({ev['dependencies_count']}): {', '.join(ev['dependencies']) or 'keine (völlig autark)'}")

        if ev["coupling_summary"]:
            print("   Schnittstellen-Kopplung (importierte Symbole):")
            for dep, count in ev["coupling_summary"].items():
                print(f"     -> {dep:25}: {count} Symbole")

        if ev["erasure_leaks_count"] > 0:
            print(f"   ⚠️  DSGVO-/Löschketten-Relikte: {ev['erasure_leaks_count']} Stellen identifiziert!")

        if ev["findings"]:
            print("   Befunde:")
            for f in ev["findings"]:
                print(f"     • {f}")

        if ev["recommendations"]:
            print("   Entkopplungs-Handlungsempfehlungen für Agenten:")
            for r in ev["recommendations"]:
                print(f"     ✓ {r}")

    print("\n" + "=" * 80)
    print("ZUSAMMENFASSUNG:")
    autark_count = sum(1 for e in evaluations if e["dependencies_count"] == 0)
    verdrahtet_count = sum(1 for e in evaluations if e["score"] < 60)
    print(f"Geprüfte Crates: {len(evaluations)}")
    print(f"Vollkommen autarke Bausteine: {autark_count}")
    print(f"Stark verdrahtete Module (Priorität für Entkopplung): {verdrahtet_count}")
    print("=" * 80)

    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except BrokenPipeError:
        sys.stderr.close()
        sys.exit(0)
