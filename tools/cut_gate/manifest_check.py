#!/usr/bin/env python3
"""
Manifest and baseline test checker for cut-gate CI workflow.
Validates:
(a) All entries in docs/restore/*/tests.manifest point to existing .rs.parked files
    in docs/restore/<ID>/tests/ containing the referenced test function name.
(b) |current tests| + |manifest entries| >= |baseline tests|.
"""

import argparse
import glob
import os
import sys


def parse_test_list(filepath):
    tests = set()
    if not filepath or not os.path.exists(filepath):
        return tests
    with open(filepath, "r", encoding="utf-8", errors="replace") as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            if line.startswith("running ") or line.startswith("test result:") or " tests, " in line:
                continue
            if line.endswith(": test") or line.endswith(": benchmark"):
                test_name = line.rsplit(":", 1)[0].strip()
                tests.add(test_name)
            elif " " not in line and line:
                tests.add(line)
    return tests


def check_manifest_entries(restore_base_dir="docs/restore"):
    manifest_files = glob.glob(os.path.join(restore_base_dir, "*", "tests.manifest"))
    manifest_entries = set()
    errors = []

    for manifest_path in sorted(manifest_files):
        restore_dir = os.path.dirname(manifest_path)
        restore_id = os.path.basename(restore_dir)
        tests_dir = os.path.join(restore_dir, "tests")

        with open(manifest_path, "r", encoding="utf-8", errors="replace") as f:
            for line_no, line in enumerate(f, 1):
                entry = line.strip()
                if not entry or entry.startswith("#"):
                    continue

                manifest_entries.add(entry)

                parts = entry.split("::")
                orig_path = parts[0].strip()
                fn_name = parts[-1].strip()

                base_filename = os.path.basename(orig_path)
                name_no_ext = os.path.splitext(base_filename)[0]

                candidates = [
                    os.path.join(tests_dir, f"{base_filename}.parked"),
                    os.path.join(tests_dir, f"{name_no_ext}.rs.parked"),
                    os.path.join(tests_dir, base_filename),
                ]

                found_file = None
                for cand in candidates:
                    if os.path.isfile(cand):
                        found_file = cand
                        break

                if not found_file:
                    errors.append(
                        f"FEHLER: Manifest-Eintrag '{entry}' ({manifest_path}:{line_no}): "
                        f"Keine geparkte Testdatei in '{tests_dir}' gefunden."
                    )
                    continue

                try:
                    with open(found_file, "r", encoding="utf-8", errors="replace") as pf:
                        content = pf.read()
                        if fn_name not in content:
                            errors.append(
                                f"FEHLER: Manifest-Eintrag '{entry}' ({manifest_path}:{line_no}): "
                                f"Funktionsname '{fn_name}' wurde nicht in '{found_file}' gefunden."
                            )
                except Exception as e:
                    errors.append(
                        f"FEHLER beim Lesen von '{found_file}': {e}"
                    )

    return manifest_entries, errors


def main():
    parser = argparse.ArgumentParser(description="Check restore manifest entries and baseline test count.")
    parser.add_argument("--current", help="Path to current test list file (output of cargo test --list)", default=None)
    args = parser.parse_args()

    # Check (a): Validate restore manifest entries
    manifest_entries, manifest_errors = check_manifest_entries()
    if manifest_errors:
        print("=== Manifest Validation Errors ===", file=sys.stderr)
        for err in manifest_errors:
            print(err, file=sys.stderr)
        sys.exit(1)

    print(f"INFO: {len(manifest_entries)} Manifest-Einträge verifiziert.")

    # Check (b): Compare current + manifest against baseline
    baseline_files = sorted(glob.glob("docs/baseline/tests-list-*.txt"))
    if not baseline_files:
        print("WARNUNG: Keine Baseline-Datei docs/baseline/tests-list-*.txt gefunden. Skip Baseline-Vergleich.")
        sys.exit(0)

    baseline_tests = set()
    for bf in baseline_files:
        baseline_tests.update(parse_test_list(bf))

    current_tests = set()
    if args.current:
        current_tests = parse_test_list(args.current)

    total_current = len(current_tests)
    total_manifest = len(manifest_entries)
    total_baseline = len(baseline_tests)
    combined = total_current + total_manifest

    print(f"Baseline Tests: {total_baseline}")
    print(f"Aktuelle Tests: {total_current}")
    print(f"Manifest Tests: {total_manifest}")
    print(f"Summe (Aktuell + Manifest): {combined}")

    if combined < total_baseline:
        print(
            f"FEHLER: Testanzahl unter Baseline! "
            f"Aktuell ({total_current}) + Manifest ({total_manifest}) = {combined} < Baseline ({total_baseline})",
            file=sys.stderr,
        )
        sys.exit(1)

    print("ERFOLG: Manifest- und Testanzahl-Prüfung erfolgreich bestanden.")
    sys.exit(0)


if __name__ == "__main__":
    main()
