#!/usr/bin/env python3
"""
docs/verification/verify_export.py

Dünner Kompatibilitäts-Wrapper für den neuen externen Verifier (tools/contextra-verifier).
Bewahrt bestehende Aufrufsyntax (`python3 verify_export.py <export_json_path> [<pubkey_or_hmackey>]`)
für Version 1 bis 3 und verweist für Version 4 auf tools/contextra-verifier.
"""

import sys
import os

# Füge tools/contextra-verifier zum Python-Pfad hinzu
script_dir = os.path.dirname(os.path.abspath(__file__))
verifier_dir = os.path.abspath(os.path.join(script_dir, "..", "..", "tools", "contextra-verifier"))
if verifier_dir not in sys.path:
    sys.path.insert(0, verifier_dir)

from contextra_verifier.cli import main as verifier_cli_main


def main():
    if len(sys.argv) < 2:
        print("Verwendung: python3 verify_export.py <export_json_path> [<pubkey_or_hmackey_hex_or_file>]")
        sys.exit(3)

    proof_path = sys.argv[1]
    key_arg = sys.argv[2] if len(sys.argv) > 2 else None

    cmd_args = [proof_path]
    if key_arg:
        cmd_args.extend(["--pubkey", key_arg])

    sys.exit(verifier_cli_main(cmd_args))


if __name__ == "__main__":
    main()
