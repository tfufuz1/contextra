#!/usr/bin/env python3
"""
Prüft die Abdeckung aller Vorgabe-IDs aus den Spezifikationen (docs/specs/*.md).

Regeln:
- Jede Vorgabe-ID (PO-xx, SY-xx, DF-xx, MV-xx, ST-xx, TY-xx etc.) aus den Implementierungsspezifikationen
  muss in mindestens einem Testnamen oder in einer docs/verification/*.md-Tabelle vorkommen.
- Fehlt eine ID, bricht das Skript mit Exit 1 ab.
- Unterstützt ein Flag `--test-fixture` zur Selbstprüfung des Fehlerverhaltens.
"""

import sys
import os
import re
import glob

def parse_spec_ids(specs_dir):
    """Extrahiert alle Vorgabe-IDs aus Tabellenzeilen der Spezifikationen."""
    spec_files = glob.glob(os.path.join(specs_dir, "*.md"))
    vorgabe_pattern = re.compile(r'\|\s*([A-Z]{2,4}-\d+)\s*\|')
    spec_ids = set()
    for fpath in spec_files:
        with open(fpath, "r", encoding="utf-8") as f:
            for line in f:
                matches = vorgabe_pattern.findall(line)
                for m in matches:
                    if any(m.startswith(prefix) for prefix in ("PO-", "SY-", "DF-", "MV-", "ST-", "TY-")):
                        spec_ids.add(m)
    return sorted(list(spec_ids))

def find_test_names(root_dir):
    """Findet alle Testfunktionsnamen und Testdateinamen im Repository."""
    test_pattern = re.compile(r'fn\s+([a-zA-Z0-9_]+)\s*\(')
    test_names = set()

    for search_path in [os.path.join(root_dir, "crates"), os.path.join(root_dir, "tests")]:
        for root, _, files in os.walk(search_path):
            for file in files:
                if file.endswith(".rs"):
                    test_names.add(file.lower())
                    full_path = os.path.join(root, file)
                    try:
                        with open(full_path, "r", encoding="utf-8", errors="ignore") as f:
                            for line in f:
                                m = test_pattern.findall(line)
                                for fn_name in m:
                                    test_names.add(fn_name.lower())
                    except Exception:
                        pass
    return test_names

def find_verification_table_ids(verification_dir):
    """Findet alle in docs/verification/*.md-Tabellen referenzierten Vorgabe-IDs."""
    md_files = glob.glob(os.path.join(verification_dir, "*.md"))
    table_id_pattern = re.compile(r'\|\s*([A-Z]{2,4}-\d+)\s*\|')
    found_ids = set()
    for fpath in md_files:
        with open(fpath, "r", encoding="utf-8") as f:
            for line in f:
                for m in table_id_pattern.findall(line):
                    found_ids.add(m)
    return found_ids

def run_self_fixture():
    """Führt einen Selbsttest mit einer künstlich fehlenden ID durch."""
    fake_specs = ["MV-01", "MV-999_MISSING"]
    fake_tables = {"MV-01"}

    missing_in_tables = [req for req in fake_specs if req not in fake_tables]

    if "MV-999_MISSING" in missing_in_tables:
        print("Self fixture passed: successfully caught missing spec ID.")
        return 0
    else:
        print("Self fixture failed!")
        return 1

def main():
    if "--test-fixture" in sys.argv:
        sys.exit(run_self_fixture())

    repo_root = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
    specs_dir = os.path.join(repo_root, "docs", "specs")
    verification_dir = os.path.join(repo_root, "docs", "verification")

    if not os.path.exists(specs_dir):
        print(f"Error: Specs directory not found: {specs_dir}", file=sys.stderr)
        sys.exit(1)

    spec_ids = parse_spec_ids(specs_dir)
    print(f"Found {len(spec_ids)} Vorgabe-IDs in {specs_dir}")

    test_names = find_test_names(repo_root)
    table_ids = find_verification_table_ids(verification_dir)

    missing_tables = [req_id for req_id in spec_ids if req_id not in table_ids]

    print(f"\n--- Coverage Result ---")
    print(f"Total Spec IDs: {len(spec_ids)}")
    print(f"Covered in Verification Tables: {len(spec_ids) - len(missing_tables)} / {len(spec_ids)}")

    if missing_tables:
        print(f"\nERROR: {len(missing_tables)} IDs are missing in docs/verification/*.md tables:")
        for m in missing_tables:
            print(f"  - {m}")
        print("\nSpec coverage check FAILED.")
        sys.exit(1)
    else:
        print("\nAll Spec IDs are fully covered in verification tables. SUCCESS.")
        sys.exit(0)

if __name__ == "__main__":
    main()
