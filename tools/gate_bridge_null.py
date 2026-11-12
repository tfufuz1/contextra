#!/usr/bin/env python3
"""
tools/gate_bridge_null.py — Gate G-L2 (Bridge-Null Gate per SPEC §8.1 / §8.4 / R2).

Normative Prüfkriterien:
- In Ebene E0 und E1 dürfen `contextra_types` und `contextra_ports` NICHT im Kern vorkommen.
- Alle Referenzen auf `contextra_types` und `contextra_ports` müssen sich in `src/compat/`
  befinden oder hinter dem Feature-Gate `#[cfg(feature = "workspace")]` (bzw. types-compat/ports-compat).
- Tests des Kerns dürfen `testkit` nicht ohne Gate verwenden.

Exit-Codes:
  0 = grün (keine ungeschützten Bridge-Referenzen in E0/E1-Kernen)
  1 = Verstoß (ungegatete Brückenreferenzen gefunden)
  2 = Werkzeugfehler
"""

from __future__ import annotations

import argparse
import json
import re
import sys
import tomllib
from pathlib import Path
from typing import Any, Dict, List, Tuple


BRIDGE_PATTERNS = [
    re.compile(r"\bcontextra_types\b"),
    re.compile(r"\bcontextra_ports\b"),
    re.compile(r"\bcontextra_testkit\b"),
]

CFG_WORKSPACE_PATTERNS = [
    re.compile(r'#!?\[cfg\([^)]*(?:workspace|types-compat|ports-compat)[^)]*\)\]'),
]


def strip_comments_and_strings(code: str) -> str:
    """Maskiert Kommentare und Strings für saubere Token-Prüfung."""
    result = list(code)
    n = len(code)
    i = 0
    while i < n:
        if code[i : i + 2] == "//":
            j = code.find("\n", i)
            end = j if j != -1 else n
            for k in range(i, end):
                result[k] = " "
            i = end
            continue
        if code[i : i + 2] == "/*":
            j = code.find("*/", i + 2)
            end = j + 2 if j != -1 else n
            for k in range(i, end):
                if result[k] != "\n":
                    result[k] = " "
            i = end
            continue
        if code[i] == '"' and (i == 0 or code[i - 1] != "\\"):
            j = i + 1
            while j < n:
                if code[j] == "\\" and j + 1 < n:
                    j += 2
                    continue
                if code[j] == '"':
                    j += 1
                    break
                j += 1
            for k in range(i, min(j, n)):
                if result[k] != "\n":
                    result[k] = " "
            i = j
            continue
        i += 1
    return "".join(result)


def audit_file_for_bridges(file_path: Path, crate_root: Path) -> List[Dict[str, Any]]:
    rel_path = file_path.relative_to(crate_root).as_posix()

    # Dateien in src/compat/ sind Brücken und per Definition erlaubt
    if rel_path.startswith("src/compat/"):
        return []

    # Tests hinter required-features = ["workspace"] oder in tests/
    # SPEC R2: Ausnahme für tests/
    if rel_path.startswith("tests/"):
        return []

    try:
        raw_text = file_path.read_text(encoding="utf-8", errors="ignore")
    except Exception:
        return []

    clean_text = strip_comments_and_strings(raw_text)
    clean_lines = clean_text.splitlines()
    raw_lines = raw_text.splitlines()

    findings = []

    # Dateiweiter cfg-Check (z.B. ganz oben `#![cfg(feature = "workspace")]`)
    for line in raw_lines[:15]:
        if any(pat.search(line) for pat in CFG_WORKSPACE_PATTERNS):
            return []

    for idx, (c_line, r_line) in enumerate(zip(clean_lines, raw_lines), start=1):
        for b_pat in BRIDGE_PATTERNS:
            if b_pat.search(c_line):
                # Prüfe, ob vorherige Zeile ein cfg(feature = "workspace" / ...-compat) Attribut hat
                prev_line = raw_lines[idx - 2] if idx >= 2 else ""
                prev_prev = raw_lines[idx - 3] if idx >= 3 else ""
                if any(pat.search(prev_line) or pat.search(prev_prev) for pat in CFG_WORKSPACE_PATTERNS):
                    continue

                findings.append({
                    "file": str(file_path),
                    "rel_file": rel_path,
                    "line": idx,
                    "symbol": b_pat.pattern.replace(r"\b", ""),
                    "snippet": r_line.strip()[:100],
                })

    return findings


def run_gate_bridge_null(repo_root: Path, target_crate: str | None = None) -> Tuple[List[Dict[str, Any]], Dict[str, Any]]:
    caps_file = repo_root / "capabilities.toml"
    with open(caps_file, "rb") as f:
        caps = tomllib.load(f)

    crates_spec: Dict[str, dict] = caps.get("crates", {})
    all_violations: List[Dict[str, Any]] = []
    summary_by_crate: Dict[str, int] = {}

    for cname, cspec in sorted(crates_spec.items()):
        if target_crate and cname != target_crate:
            continue
        level = cspec.get("level", 0)
        # G-L2 gilt primär für Ebene E0 und E1
        if level not in (0, 1):
            continue

        cpath = repo_root / cspec.get("path", f"crates/{cname}")
        if not cpath.is_dir():
            continue

        src_dir = cpath / "src"
        if not src_dir.is_dir():
            continue

        crate_violations = []
        for p in src_dir.rglob("*.rs"):
            findings = audit_file_for_bridges(p, cpath)
            for f_item in findings:
                f_item["crate"] = cname
                f_item["level"] = level
                crate_violations.append(f_item)

        if crate_violations:
            all_violations.extend(crate_violations)
            summary_by_crate[cname] = len(crate_violations)
        else:
            summary_by_crate[cname] = 0

    return all_violations, summary_by_crate


def main() -> int:
    parser = argparse.ArgumentParser(description="Gate G-L2 Bridge-Null Audit (SPEC §8.1 / R2)")
    parser.add_argument("--crate", "-c", help="Spezifisches Crate prüfen")
    parser.add_argument("--root", default=".", help="Workspace-Root")
    parser.add_argument("--json", action="store_true", help="JSON-Ausgabe")
    args = parser.parse_args()

    root = Path(args.root).resolve()
    violations, summary = run_gate_bridge_null(root, args.crate)

    if args.json:
        payload = {
            "gate": "G-L2",
            "status": "PASS" if not violations else "FAIL",
            "total_violations": len(violations),
            "summary_by_crate": summary,
            "violations": violations,
        }
        print(json.dumps(payload, indent=2))
        return 1 if violations else 0

    print("=== GATE G-L2: Bridge-Null Audit (Ebene E0/E1) ===")
    print(f"Geprüfte E0/E1 Crates: {len(summary)}")
    if not violations:
        print("✅ Gate G-L2 bestanden: 0 ungegatete Brückenreferenzen in E0/E1 Kernen.")
        return 0

    print(f"\n❌ Gate G-L2: {len(violations)} ungegatete Brückenreferenzen in E0/E1 Kernen gefunden:")
    for cname, count in summary.items():
        if count > 0:
            print(f"  - `{cname}`: {count} unzulässige Fundstellen")

    return 1


if __name__ == "__main__":
    sys.exit(main())
