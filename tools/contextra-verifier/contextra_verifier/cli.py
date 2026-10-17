"""Command line interface for Contextra DeletionProof verifier."""

import sys
import os
import json
import argparse
from typing import Tuple

from .verify import verify_proof


def parse_pubkey_arg(pubkey_arg: str) -> Tuple[bytes, str]:
    """Reads public key from file or parses hex string."""
    if not pubkey_arg:
        return b"", "Missing public key argument"

    if os.path.isfile(pubkey_arg):
        try:
            with open(pubkey_arg, "rb") as f:
                content = f.read().strip()
                # If key file contains hex text
                if len(content) in (64, 128) and all(c in b"0123456789abcdefABCDEF" for c in content):
                    return bytes.fromhex(content.decode("ascii")), ""
                return content, ""
        except Exception as e:
            return b"", f"Failed to read public key file '{pubkey_arg}': {e}"

    # Try hex string directly
    clean_hex = pubkey_arg.strip()
    if clean_hex.startswith("0x") or clean_hex.startswith("0X"):
        clean_hex = clean_hex[2:]

    try:
        key_bytes = bytes.fromhex(clean_hex)
        return key_bytes, ""
    except ValueError:
        return b"", f"Invalid hex string for public key: '{pubkey_arg}'"


def main(args_list=None) -> int:
    """Main CLI entry point."""
    parser = argparse.ArgumentParser(
        description="Contextra DeletionProof External Verifier (Version 4, TLV)",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument("proof_file", help="Path to DeletionProof file (.json or .bin)")
    parser.add_argument(
        "--pubkey",
        required=False,
        help="Path to public key file or hex-encoded 32-byte Ed25519 public key / HMAC key",
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="Output structured JSON result instead of plain text",
    )

    try:
        args = parser.parse_args(args_list)
    except SystemExit as se:
        return 3 if se.code != 0 else 0
    except Exception as e:
        print(f"INVALID: USAGE_ERROR: {e}")
        return 3

    proof_path = args.proof_file
    pubkey_arg = args.pubkey

    if not os.path.isfile(proof_path):
        msg = f"Proof file not found: '{proof_path}'"
        if args.json:
            print(json.dumps({"valid": False, "exit_code": 3, "category": "USAGE_ERROR", "message": msg}))
        else:
            print(f"INVALID: USAGE_ERROR: {msg}")
        return 3

    try:
        with open(proof_path, "rb") as f:
            proof_bytes = f.read()
    except Exception as e:
        msg = f"Failed to read proof file '{proof_path}': {e}"
        if args.json:
            print(json.dumps({"valid": False, "exit_code": 3, "category": "USAGE_ERROR", "message": msg}))
        else:
            print(f"INVALID: USAGE_ERROR: {msg}")
        return 3

    key_bytes = b""
    if pubkey_arg:
        key_bytes, err_msg = parse_pubkey_arg(pubkey_arg)
        if err_msg:
            if args.json:
                print(json.dumps({"valid": False, "exit_code": 3, "category": "USAGE_ERROR", "message": err_msg}))
            else:
                print(f"INVALID: USAGE_ERROR: {err_msg}")
            return 3

    try:
        exit_code, category, reason = verify_proof(proof_bytes, key_bytes)
    except Exception as e:
        exit_code, category, reason = 2, "FORMAT_ERROR", f"Unhandled error during verification: {e}"

    if args.json:
        result_obj = {
            "valid": (exit_code == 0),
            "exit_code": exit_code,
            "category": category,
            "message": reason,
        }
        print(json.dumps(result_obj, indent=2))
    else:
        if exit_code == 0:
            print("VALID")
        else:
            print(f"INVALID: {category}: {reason}")

    return exit_code


if __name__ == "__main__":
    sys.exit(main())
