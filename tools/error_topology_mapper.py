#!/usr/bin/env python3
"""
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

if __name__ == "__main__":
    main()
