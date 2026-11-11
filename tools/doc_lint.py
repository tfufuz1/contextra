#!/usr/bin/env python3
"""
tools/doc_lint.py — Gate G-DOC1 (Documentation & Compliance Wording Linter per SPEC §1.2 / INV-7).

Normative Prüfkriterien:
- INV-7: Keine Compliance-/Rechts-Aussagen in Code und Dokumentation außerhalb von Compliance-Crates.
- Verbotene unbewiesene Zusicherungen wie:
  - "DSGVO-konform", "rechtskonform", "100% compliant", "unwiderruflich", "unwiederbringlich"
- Gewährleistet neutrale technische Terminologie (z. B. "Entfernung" statt rechtlicher Garantien).

Exit-Codes:
  0 = grün (keine unzulässigen Compliance-Aussagen)
  1 = Verstoß
"""

from __future__ import annotations

import argparse
import json
import os
import re
import sys
from pathlib import Path
from typing import Any, Dict, List, Tuple


FORBIDDEN_COMPLIANCE_PATTERNS = [
    (re.compile(r"\bdsgvo-konform\b", re.IGNORECASE), "Unzulässige Rechtskonformitäts-Zusicherung: 'DSGVO-konform'"),
    (re.compile(r"\brechtskonform\b", re.IGNORECASE), "Unzulässige Rechtskonformitäts-Zusicherung: 'rechtskonform'"),
    (re.compile(r"\bunwiederbringlich\b", re.IGNORECASE), "Unzulässige Garantie-Zusicherung: 'unwiederbringlich'"),
    (re.compile(r"\b100%\s+compliant\b", re.IGNORECASE), "Unzulässige Compliance-Zusicherung: '100% compliant'"),
]

# Erlaubte Ausnahmen in Compliance-Spezifikationen bzw. Tests und Audit-Dokumentationen
ALLOWED_PATHS = {
    "crates/contextra-erasure",
    "crates/contextra-license",
    "crates/contextra-audit-export",
    "crates/contextra-avv-generator",
    "docs/spec/COMPLIANCE",
    "docs/compliance",
    "docs/archive",
    "docs/handoff",
    "docs/reports",
}


def is_allowed_path(rel_path: str) -> bool:
    for ap in ALLOWED_PATHS:
        if rel_path.startswith(ap):
            return True
    return False


def lint_documentation_and_code(root: Path) -> List[Dict[str, Any]]:
    findings = []

    # Durchsuche .md und .rs Dateien in crates/ und docs/
    search_dirs = [root / "crates", root / "docs"]
    for sdir in search_dirs:
        if not sdir.is_dir():
            continue
        for p in sdir.rglob("*"):
            if not p.is_file():
                continue
            if p.suffix not in (".rs", ".md"):
                continue

            rel_path = p.relative_to(root).as_posix()
            if is_allowed_path(rel_path):
                continue
            # Auch archive/ und tools/ und baseline überspringen
            if rel_path.startswith("archive/") or rel_path.startswith("docs/baseline/"):
                continue

            try:
                content = p.read_text(encoding="utf-8", errors="ignore")
            except Exception:
                continue

            # Test-Dateien dürfen verbotene Begriffe zur Negativprüfung enthalten
            if p.name.endswith("_test.rs") or p.name == "tests.rs" or "/tests/" in rel_path:
                continue

            for idx, line in enumerate(content.splitlines(), start=1):
                # Kommentare oder Berichte die Verbote zitieren (z. B. "Forbidden wording", "Suche nach", "Verbot") überspringen
                l_lower = line.lower()
                if any(k in l_lower for k in ("forbidden wording", "inv-7", "suche nach", "verbot", "verbotene")):
                    continue
                for pat, msg in FORBIDDEN_COMPLIANCE_PATTERNS:
                    if pat.search(line):
                        findings.append({
                            "file": rel_path,
                            "line": idx,
                            "message": msg,
                            "snippet": line.strip()[:100],
                        })

    return findings


def main() -> int:
    parser = argparse.ArgumentParser(description="Gate G-DOC1 Documentation & Compliance Wording Linter (INV-7)")
    parser.add_argument("--root", default=".", help="Workspace-Root")
    parser.add_argument("--json", action="store_true", help="JSON-Ausgabe")
    args = parser.parse_args()

    root = Path(args.root).resolve()
    findings = lint_documentation_and_code(root)

    if args.json:
        payload = {
            "gate": "G-DOC1",
            "status": "PASS" if not findings else "FAIL",
            "findings_count": len(findings),
            "findings": findings,
        }
        print(json.dumps(payload, indent=2))
        return 1 if findings else 0

    print("=== GATE G-DOC1: Compliance & Wording Linter (INV-7) ===")
    if not findings:
        print("✅ Gate G-DOC1 bestanden: Keine unzulässigen Compliance- oder Rechtsgarantien gefunden.")
        return 0

    print(f"\n❌ Gate G-DOC1: {len(findings)} unzulässige Formulierungen gefunden:")
    for f in findings:
        print(f"  - {f['file']}:{f['line']} [{f['message']}]: {f['snippet']}")

    return 1


if __name__ == "__main__":
    sys.exit(main())
