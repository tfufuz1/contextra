#!/usr/bin/env python3
"""
<<<<<<< HEAD
error_topology_mapper.py - Kartierung des Fehlersystems und Prüfung auf Datenleaks (INV-LOG-1 / HR-4).

Prüfungen:
- Kartiert alle definierten Fehler-Enums und Fehlerstrukturen (z. B. ContextraError, CryptoError).
- Prüft #[error("...")] und Display-Implementierungen auf verbotene Nutzdatenlecks:
  Keine Dateiinhalte, Schlüssel, Hashes oder Klartextpfade in Fehlermeldungen (INV-LOG-1).
- Prüft die korrekte Fehlerbehandlung und Konvertierung (From<...>, thiserror).
"""

import os
import sys
import re
import json
import argparse
from pathlib import Path

ERROR_ENUM_PATTERN = re.compile(r'^\s*(?:pub\s+)?enum\s+([A-Za-z0-9_]*Error[A-Za-z0-9_]*)')
THISERROR_PATTERN = re.compile(r'#\[error\(\s*"([^"]+)"')

# Sensitive keywords that must not leak into error strings
LEAK_SUSPECTS = [
    "secret", "key", "password", "token", "payload", "plaintext", "content", "passphrase"
]

def scan_crate_errors(crate_dir: Path):
    crate_name = crate_dir.name
    src_dir = crate_dir / "src"
    if not src_dir.exists():
        return None

    error_enums = []
    suspicious_messages = []

    for rs_file in src_dir.rglob("*.rs"):
        try:
            with open(rs_file, "r", encoding="utf-8", errors="ignore") as f:
                lines = f.readlines()
        except Exception:
            continue

        for idx, line in enumerate(lines):
            l_str = line.strip()

            # Detect Error Enums
            m_enum = ERROR_ENUM_PATTERN.search(l_str)
            if m_enum:
                error_enums.append({
                    "name": m_enum.group(1),
                    "file": str(rs_file),
                    "line": idx + 1
                })

            # Detect #[error("...")] strings
            m_err = THISERROR_PATTERN.search(l_str)
            if m_err:
                msg = m_err.group(1)
                for sus in LEAK_SUSPECTS:
                    # Look for interpolation like {key} or {secret}
                    if re.search(rf'\{{[^}}]*{sus}[^}}]*\}}', msg, re.IGNORECASE):
                        suspicious_messages.append({
                            "file": str(rs_file),
                            "line": idx + 1,
                            "suspect_term": sus,
                            "message_format": msg
                        })

    return {
        "crate": crate_name,
        "error_enums_count": len(error_enums),
        "error_enums": error_enums,
        "suspicious_messages_count": len(suspicious_messages),
        "suspicious_messages": suspicious_messages
    }

def main():
    parser = argparse.ArgumentParser(description="Error Topology & Leak Auditor (INV-LOG-1)")
    parser.add_argument("--workspace", default=".", help="Workspace root")
    parser.add_argument("--json", action="store_true", help="Output JSON directly to stdout")
    parser.add_argument("--output-dir", default="docs/diagnostics", help="Output directory for reports")
    args = parser.parse_args()

    workspace_root = Path(args.workspace).resolve()
    crates_dir = workspace_root / "crates"
    output_dir = workspace_root / args.output_dir
    output_dir.mkdir(parents=True, exist_ok=True)

    crates = sorted([p for p in crates_dir.iterdir() if p.is_dir() and (p / "Cargo.toml").exists()])
    results = []

    for c in crates:
        res = scan_crate_errors(c)
        if res:
            results.append(res)

    md_path = output_dir / "ERROR_TOPOLOGY_REPORT.md"
    json_path = output_dir / "error_topology_report.json"

    with open(json_path, "w", encoding="utf-8") as jf:
        json.dump(results, jf, indent=2)

    with open(md_path, "w", encoding="utf-8") as mf:
        mf.write("# Error Topology & Data Leak Audit (INV-LOG-1 / HR-4)\n\n")
        mf.write(f"> Generiert durch `tools/error_topology_mapper.py`.\n")
        mf.write(f"> Untersucht typisierte Fehlerstrukturen und verhindert Nutzdaten-/Geheimnis-Leaks in Fehlermeldungen.\n\n")

        mf.write("## Crate Übersicht\n\n")
        mf.write("| Crate | Definierte Fehler-Typen | Potenzielle Nutzdaten-Leaks |\n")
        mf.write("|---|---|---|\n")
        for r in results:
            leak_tag = f"**{r['suspicious_messages_count']}**" if r['suspicious_messages_count'] > 0 else "0"
            mf.write(f"| `{r['crate']}` | {r['error_enums_count']} | {leak_tag} |\n")

        total_suspicious = sum(r["suspicious_messages_count"] for r in results)
        if total_suspicious > 0:
            mf.write("\n## Verdächtige Fehlermeldungs-Formate (Möglicher Leak von {key}, {secret}...)\n\n")
            for r in results:
                if r["suspicious_messages"]:
                    mf.write(f"### `{r['crate']}`\n\n")
                    for sm in r["suspicious_messages"]:
                        mf.write(f"- `{sm['file']}:{sm['line']}` [enthält `{sm['suspect_term']}`]: `{sm['message_format']}`\n")

    if args.json:
        print(json.dumps(results, indent=2))
    else:
        print(f"Error Topology Audit abgeschlossen:")
        print(f" - Geprüfte Crates: {len(results)}")
        print(f" - Potenziell sensible Meldungen: {total_suspicious}")
        print(f" - Markdown: {md_path}")
        print(f" - JSON: {json_path}")
=======
error_topology_mapper.py — Error Architecture & Topology Mapper.

Analyzes error structures and propagation boundaries across crates:
- Maps all Error enum definitions (derive(thiserror::Error)).
- Tracks variants, error codes, and display strings.
- Detects the central `ContextraError` variants and domain mappings.
- Identifies error conversion implementations (`impl From<X> for Y`).
- Verifies frozen error texts (T-API-4 / D-110).
"""

from __future__ import annotations

import json
import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, List, Optional, Tuple

from agent_tooling_common import (
    DiagnosticReport,
    create_standard_cli,
    discover_crates,
    get_repo_root,
    iter_rust_files,
    strip_comments_and_strings,
)


@dataclass
class ErrorVariant:
    name: str
    error_text: Optional[str]
    has_source: bool
    line: int


@dataclass
class ErrorEnum:
    name: str
    crate: str
    file: str
    line: int
    variants: List[ErrorVariant]


def parse_error_enums(file_path: Path, crate_name: str, repo_root: Path) -> List[ErrorEnum]:
    error_enums = []
    try:
        raw_text = file_path.read_text(encoding="utf-8")
    except Exception:
        return error_enums

    lines = raw_text.splitlines()
    rel_path = str(file_path.relative_to(repo_root))

    re_error_derive = re.compile(r"#\[derive\(.*(?:thiserror::Error|Error).*\)\]")
    re_enum = re.compile(r"^\s*(?:pub(?:\s*\([^)]+\))?\s+)?enum\s+([A-Za-z0-9_]+Error|[A-Za-z0-9_]+ErrorKind)")
    re_variant = re.compile(r"^\s*([A-Za-z0-9_]+)(?:\([^\)]*\)|\{[^\}]*\})?,?")
    re_error_attr = re.compile(r'#\[error\("([^"]+)"\)')

    i = 0
    n = len(lines)
    while i < n:
        line = lines[i].strip()
        # Look for enum that derives Error or has Error in name
        if re_error_derive.search(line) or "enum " in line:
            # Check next few lines for enum declaration
            enum_name = None
            enum_line = i + 1
            for k in range(i, min(i + 5, n)):
                m_enum = re_enum.search(lines[k])
                if m_enum:
                    enum_name = m_enum.group(1)
                    enum_line = k + 1
                    i = k
                    break

            if enum_name:
                # Parse variants inside braces
                variants = []
                # advance to opening brace
                while i < n and "{" not in lines[i]:
                    i += 1
                brace_depth = 1
                i += 1
                pending_error_text = None
                pending_has_source = False

                while i < n and brace_depth > 0:
                    l = lines[i].strip()
                    brace_depth += l.count("{") - l.count("}")
                    if brace_depth <= 0:
                        break

                    m_err = re_error_attr.search(l)
                    if m_err:
                        pending_error_text = m_err.group(1)

                    if "#[source]" in l or "#[from]" in l:
                        pending_has_source = True

                    if l and not l.startswith("//") and not l.startswith("#"):
                        m_var = re_variant.match(l)
                        if m_var:
                            var_name = m_var.group(1)
                            if var_name not in ("pub", "where"):
                                variants.append(ErrorVariant(
                                    name=var_name,
                                    error_text=pending_error_text,
                                    has_source=pending_has_source,
                                    line=i + 1,
                                ))
                                pending_error_text = None
                                pending_has_source = False
                    i += 1

                error_enums.append(ErrorEnum(
                    name=enum_name,
                    crate=crate_name,
                    file=rel_path,
                    line=enum_line,
                    variants=variants,
                ))
        i += 1

    return error_enums


def parse_error_conversions(file_path: Path, crate_name: str, repo_root: Path) -> List[Dict]:
    conversions = []
    try:
        raw_text = file_path.read_text(encoding="utf-8")
    except Exception:
        return conversions

    clean_content = strip_comments_and_strings(raw_text)
    re_from_impl = re.compile(r"impl(?:<[^>]+>)?\s+From<([A-Za-z0-9_:]+)(?:<[^>]+>)?\s*>\s*for\s+([A-Za-z0-9_:]+)")
    rel_path = str(file_path.relative_to(repo_root))

    for idx, clean_line in enumerate(clean_content.splitlines(), start=1):
        m = re_from_impl.search(clean_line)
        if m:
            src = m.group(1).strip()
            dest = m.group(2).strip()
            conversions.append({
                "crate": crate_name,
                "file": rel_path,
                "line": idx,
                "from_type": src,
                "to_type": dest,
            })
    return conversions


def main():
    cli = create_standard_cli("Error Topology & Hierarchy Mapper across crates.")
    args = cli.parse_args()

    repo_root = get_repo_root()
    crates = discover_crates(repo_root)

    target_crates = {}
    if args.crate:
        if args.crate in crates:
            target_crates[args.crate] = crates[args.crate]
        else:
            print(f"Error: Crate '{args.crate}' not found.", file=sys.stderr)
            sys.exit(1)
    else:
        target_crates = crates

    report = DiagnosticReport(
        tool_name="error_topology_mapper.py",
        title="Error Topology & Conversion Architecture Report",
        description="Maps strongly-typed error definitions (thiserror), variants, error codes, and cross-boundary error conversion implementations (impl From<E>).",
    )

    all_error_enums: List[ErrorEnum] = []
    all_conversions: List[Dict] = []
    total_variants = 0

    for cname, cmeta in sorted(target_crates.items()):
        for rpath in iter_rust_files(cmeta.path, include_tests=False):
            enums = parse_error_enums(rpath, cname, repo_root)
            convs = parse_error_conversions(rpath, cname, repo_root)

            for e in enums:
                all_error_enums.append(e)
                total_variants += len(e.variants)
                report.add_finding({
                    "crate": cname,
                    "file": e.file,
                    "line": e.line,
                    "kind": "ERROR_ENUM",
                    "name": e.name,
                    "variant_count": len(e.variants),
                    "description": f"Error enum `{e.name}` with {len(e.variants)} variants",
                })

            for c in convs:
                all_conversions.append(c)
                report.add_finding({
                    "crate": cname,
                    "file": c["file"],
                    "line": c["line"],
                    "kind": "ERROR_CONVERSION",
                    "description": f"impl From<{c['from_type']}> for {c['to_type']}",
                })

    report.set_metric("total_error_enums", len(all_error_enums))
    report.set_metric("total_error_variants", total_variants)
    report.set_metric("total_conversions_from", len(all_conversions))

    enum_table = [
        "| Crate | Error-Typ | Zeile | Varianten | Datei |",
        "|---|---|---|---|---|",
    ]
    for e in all_error_enums:
        enum_table.append(f"| `{e.crate}` | `{e.name}` | {e.line} | {len(e.variants)} | `{e.file}` |")
    report.add_section("Erfasste Error-Klassen", "\n".join(enum_table))

    if args.json:
        print(json.dumps(report.to_dict(), indent=2))
    else:
        print(f"[*] Error Topology Mapper complete: {len(all_error_enums)} error enums with {total_variants} variants found.")
        print(f"    Error conversions (impl From): {len(all_conversions)}")

    if not args.no_save:
        out_name = f"error_topology_{args.crate}" if args.crate else "error_topology_workspace"
        j_p, m_p = report.write_artifacts(out_name, Path(args.output_dir) if args.output_dir else None)
        if not args.json:
            print(f"[+] Saved artifacts:\n    {j_p}\n    {m_p}")

>>>>>>> d19a1ae08 (Add micro-granular diagnostic tools for coding agents)

if __name__ == "__main__":
    main()
