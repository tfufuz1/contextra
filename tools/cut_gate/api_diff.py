#!/usr/bin/env python3
"""
API Snapshot diff tool for cut-gate CI workflow.
Checks for API deviations between committed api/*.txt snapshots
and freshly generated API snapshots if api/generate_api_snapshot.py exists.
"""

import difflib
import os
import subprocess
import sys
import tempfile


def main():
    generator_script = os.path.join("api", "generate_api_snapshot.py")
    if not os.path.exists(generator_script):
        print(f"INFO: Skript '{generator_script}' existiert nicht. API-Diff wird übersprungen.")
        sys.exit(0)

    with tempfile.TemporaryDirectory() as temp_dir:
        cmd = [sys.executable, generator_script, "--out", temp_dir]
        print(f"Erzeuge frischen API-Snapshot mit: {' '.join(cmd)}")
        res = subprocess.run(cmd, capture_output=True, text=True)
        if res.returncode != 0:
            print(f"FEHLER bei der Erzeugung des API-Snapshots:\n{res.stderr}", file=sys.stderr)
            sys.exit(1)

        diffs_found = False
        files_to_check = ["contextra-ports.txt", "contextra-types.txt"]

        for filename in files_to_check:
            expected_path = os.path.join("api", filename)
            generated_path = os.path.join(temp_dir, filename)

            expected_lines = []
            if os.path.exists(expected_path):
                with open(expected_path, "r", encoding="utf-8", errors="replace") as f:
                    expected_lines = f.readlines()

            generated_lines = []
            if os.path.exists(generated_path):
                with open(generated_path, "r", encoding="utf-8", errors="replace") as f:
                    generated_lines = f.readlines()

            diff = list(
                difflib.unified_diff(
                    expected_lines,
                    generated_lines,
                    fromfile=f"api/{filename} (Soll)",
                    tofile=f"generated/{filename} (Ist)",
                )
            )

            if diff:
                diffs_found = True
                print(f"FEHLER: Abweichungen in {filename} gefunden:", file=sys.stderr)
                sys.stderr.writelines(diff)

        if diffs_found:
            sys.exit(1)

        print("ERFOLG: Keine API-Abweichungen in contextra-ports.txt und contextra-types.txt.")
        sys.exit(0)


if __name__ == "__main__":
    main()
