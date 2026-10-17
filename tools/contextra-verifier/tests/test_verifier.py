"""Unit tests for Contextra DeletionProof Verifier."""

import os
import sys
import json
import unittest

# Add tools/contextra-verifier to sys.path
test_dir = os.path.dirname(os.path.abspath(__file__))
verifier_dir = os.path.abspath(os.path.join(test_dir, ".."))
if verifier_dir not in sys.path:
    sys.path.insert(0, verifier_dir)

from contextra_verifier.verify import verify_proof
from contextra_verifier.cli import main as cli_main


class TestVerifierTestVectors(unittest.TestCase):
    """Loads all test vectors from testvectors/ and validates expected exit codes and categories."""

    def test_all_vectors(self):
        vectors_dir = os.path.join(verifier_dir, "testvectors")
        self.assertTrue(os.path.isdir(vectors_dir), f"Directory missing: {vectors_dir}")

        vec_names = sorted(os.listdir(vectors_dir))
        self.assertGreater(len(vec_names), 0, "No test vectors found")

        for vec_name in vec_names:
            vec_path = os.path.join(vectors_dir, vec_name)
            if not os.path.isdir(vec_path):
                continue

            proof_file = os.path.join(vec_path, "proof.bin")
            pubkey_file = os.path.join(vec_path, "pubkey.bin")
            expected_file = os.path.join(vec_path, "expected.json")

            self.assertTrue(os.path.isfile(expected_file), f"Missing expected.json in {vec_name}")

            with open(expected_file, "r", encoding="utf-8") as f:
                meta = json.load(f)

            proof_bytes = b""
            if os.path.isfile(proof_file):
                with open(proof_file, "rb") as f:
                    proof_bytes = f.read()

            pubkey_bytes = b""
            if os.path.isfile(pubkey_file):
                with open(pubkey_file, "rb") as f:
                    pubkey_bytes = f.read()

            exit_code, category, reason = verify_proof(proof_bytes, pubkey_bytes)

            expected_exit = meta["exit_code"]
            expected_res = meta["result"]
            expected_cat = meta.get("category")
            expected_reason = meta.get("reason")
            expected_origin = meta["origin"]

            self.assertIn(expected_origin, ("rust-issued", "python-selfsigned"))

            self.assertEqual(
                exit_code,
                expected_exit,
                f"Vector '{vec_name}' expected exit code {expected_exit}, got {exit_code} ({category}: {reason})",
            )

            if expected_cat:
                self.assertEqual(
                    category,
                    expected_cat,
                    f"Vector '{vec_name}' expected category '{expected_cat}', got '{category}' ({reason})",
                )

            if expected_reason:
                self.assertIn(
                    expected_reason,
                    reason,
                    f"Vector '{vec_name}' expected reason fragment '{expected_reason}' in '{reason}'",
                )

            if expected_res == "VALID":
                self.assertEqual(exit_code, 0, f"Vector '{vec_name}' should be VALID")
            else:
                self.assertNotEqual(exit_code, 0, f"Vector '{vec_name}' should NOT be VALID")

    def test_byte_flip_mutation_every_position(self):
        """3.2 Byte-Flip Test: Flips every byte of valid vector and verifies no mutated state is VALID."""
        valid_dir = os.path.join(verifier_dir, "testvectors", "v01_valid_v4")
        proof_file = os.path.join(valid_dir, "proof.bin")
        pubkey_file = os.path.join(valid_dir, "pubkey.bin")

        with open(proof_file, "rb") as f:
            valid_bytes = f.read()
        with open(pubkey_file, "rb") as f:
            pubkey_bytes = f.read()

        # Sanity check: valid vector must yield 0
        code, cat, reason = verify_proof(valid_bytes, pubkey_bytes)
        self.assertEqual(code, 0, f"Valid vector failed sanity check: {cat}: {reason}")

        total_bytes = len(valid_bytes)
        self.assertGreater(total_bytes, 0, "Valid proof bytes cannot be empty")

        mutated_invalid_count = 0

        # Loop through every single byte position in the valid proof payload
        for pos in range(total_bytes):
            original_byte = valid_bytes[pos]

            # Test bit-flip mutations (flip bit 0x01 and bit 0x80)
            for mask in (0x01, 0x80, 0xFF):
                mutated_byte = original_byte ^ mask
                if mutated_byte == original_byte:
                    continue

                mutated_payload = bytearray(valid_bytes)
                mutated_payload[pos] = mutated_byte

                exit_code, category, reason = verify_proof(bytes(mutated_payload), pubkey_bytes)

                self.assertNotEqual(
                    exit_code,
                    0,
                    f"Byte-flip mutation at position {pos} (byte 0x{original_byte:02X} -> 0x{mutated_byte:02X}) unexpectedly returned VALID!",
                )
                mutated_invalid_count += 1

        self.assertGreater(mutated_invalid_count, total_bytes)

    def test_cli_invocation(self):
        """Tests CLI execution with valid proof file."""
        valid_dir = os.path.join(verifier_dir, "testvectors", "v01_valid_v4")
        proof_file = os.path.join(valid_dir, "proof.bin")
        pubkey_file = os.path.join(valid_dir, "pubkey.bin")

        ret = cli_main([proof_file, "--pubkey", pubkey_file, "--json"])
        self.assertEqual(ret, 0)


if __name__ == "__main__":
    unittest.main()
