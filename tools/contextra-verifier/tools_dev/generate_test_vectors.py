#!/usr/bin/env python3
"""Dev tool to generate canonical v4 test vectors for tools/contextra-verifier."""

import os
import sys
import json
import struct

# Add tools/contextra-verifier to sys.path
script_dir = os.path.dirname(os.path.abspath(__file__))
verifier_dir = os.path.abspath(os.path.join(script_dir, ".."))
if verifier_dir not in sys.path:
    sys.path.insert(0, verifier_dir)

from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey
from cryptography.hazmat.primitives.serialization import Encoding, PublicFormat
import blake3

from contextra_verifier.tlv import TlvField
from contextra_verifier.canon import serialize_tlv_fields, PROOF_V4_DOMAIN_TAG, encode_u32_le, encode_u64_le


def build_canonical_v4_fields(pubkey_bytes: bytes) -> list:
    key_id = blake3.blake3(pubkey_bytes).digest()

    # Scope: Document { doc_id: 42, tenant_id: 1 }
    scope_bincode = encode_u32_le(0) + encode_u64_le(42) + encode_u64_le(1)

    # Claim: layer_id 1 (0x0001), assurance 1 (0x0002), claim_text_id 2 (0x0003), witness_digest (0x0004), scanned_at_tx 100 (0x0005), evidence_tlv (0x0006)
    claim_fields = [
        TlvField(0x0001, struct.pack(">H", 1)),
        TlvField(0x0002, b"\x01"),
        TlvField(0x0003, struct.pack(">H", 2)),
        TlvField(0x0004, b"\x04" * 32),
        TlvField(0x0005, struct.pack(">Q", 100)),
        TlvField(0x0006, b"\x10\x20"),
    ]
    claim_bytes = serialize_tlv_fields(claim_fields)
    claims_payload = struct.pack(">I", len(claim_bytes)) + claim_bytes

    fields = [
        TlvField(0x8001, struct.pack(">H", 4)),
        TlvField(0x8002, b"\x01" * 32),
        TlvField(0x8003, b"db-instance-test"),
        TlvField(0x8004, b"\x02"),
        TlvField(0x8005, b""),
        TlvField(0x8006, scope_bincode),
        TlvField(0x8007, b"doc:42"),
        TlvField(0x8008, b"\x02" * 32),
        TlvField(0x8009, b"\x03" * 32),
        TlvField(0x800A, b"\x00"),
        TlvField(0x800B, struct.pack(">Q", 100)),
        TlvField(0x800C, claims_payload),
        TlvField(0x800D, struct.pack(">H", 1)),
        TlvField(0x800E, b""),
        TlvField(0x800F, b""),
        TlvField(0x8010, b"\x05" * 32),
        TlvField(0x8011, struct.pack(">Q", 1) + b"\x00" * 32),
        TlvField(0x8012, struct.pack(">Q", 1700000000000)),
        TlvField(0x8013, struct.pack(">Q", 10)),
        TlvField(0x8014, key_id),
    ]
    return fields


def main():
    testvectors_dir = os.path.join(verifier_dir, "testvectors")
    os.makedirs(testvectors_dir, exist_ok=True)

    privkey = Ed25519PrivateKey.generate()
    pubkey = privkey.public_key()
    pubkey_bytes = pubkey.public_bytes(Encoding.Raw, PublicFormat.Raw)

    fields = build_canonical_v4_fields(pubkey_bytes)
    canonical_tlv = serialize_tlv_fields(fields)
    signed_bytes = PROOF_V4_DOMAIN_TAG + canonical_tlv
    signature = privkey.sign(signed_bytes)

    valid_proof = canonical_tlv + signature

    def write_vector(name, proof_b, key_b, result, exit_code, category, reason):
        vec_dir = os.path.join(testvectors_dir, name)
        os.makedirs(vec_dir, exist_ok=True)
        with open(os.path.join(vec_dir, "proof.bin"), "wb") as f:
            f.write(proof_b)
        with open(os.path.join(vec_dir, "pubkey.bin"), "wb") as f:
            f.write(key_b)
        meta = {
            "origin": "python-selfsigned",
            "result": result,
            "exit_code": exit_code,
            "category": category,
            "reason": reason,
        }
        with open(os.path.join(vec_dir, "expected.json"), "w", encoding="utf-8") as f:
            json.dump(meta, f, indent=2)

    # 1. Valid proof
    write_vector(
        "v01_valid_v4",
        valid_proof,
        pubkey_bytes,
        "VALID",
        0,
        "VALID",
        "Version 4 DeletionProof signature verified successfully",
    )

    # 2. Mutated signature
    mutated_sig = bytearray(valid_proof)
    mutated_sig[-1] ^= 0xFF
    write_vector(
        "v02_mutated_signature",
        bytes(mutated_sig),
        pubkey_bytes,
        "INVALID",
        1,
        "INVALID",
        "Ed25519 signature verification failed or proof payload tampered",
    )

    # 3. Mutated field for each signed TLV tag
    # Mutate tag 0x8001 (version) value
    mut_v = bytearray(valid_proof)
    mut_v[6] ^= 0x01
    write_vector(
        "v03_mutated_tag_version",
        bytes(mut_v),
        pubkey_bytes,
        "INVALID",
        2,
        "FORMAT_ERROR",
        "Unsupported proof version",
    )

    # Mutate tag 0x800B (deleted_after_tx) value in canonical payload
    mut_tx = bytearray(valid_proof)
    # find offset of 0x800B tag payload and flip a byte
    idx = valid_proof.find(struct.pack(">H", 0x800B))
    if idx != -1:
        mut_tx[idx + 6] ^= 0xFF
    write_vector(
        "v04_mutated_tag_tx",
        bytes(mut_tx),
        pubkey_bytes,
        "INVALID",
        1,
        "INVALID",
        "Ed25519 signature verification failed or proof payload tampered",
    )

    # 4. Wrong public key
    wrong_key = Ed25519PrivateKey.generate().public_key().public_bytes(Encoding.Raw, PublicFormat.Raw)
    write_vector(
        "v05_wrong_public_key",
        valid_proof,
        wrong_key,
        "INVALID",
        1,
        "INVALID",
        "Signer key ID mismatch",
    )

    # 5. Truncated file
    write_vector(
        "v06_truncated_file",
        valid_proof[:50],
        pubkey_bytes,
        "INVALID",
        2,
        "FORMAT_ERROR",
        "Truncated TLV header or payload",
    )

    # 6. Trailing bytes
    write_vector(
        "v07_trailing_bytes",
        valid_proof + b"\x00\x00\x00\x00",
        pubkey_bytes,
        "INVALID",
        2,
        "FORMAT_ERROR",
        "Trailing bytes after TLV stream",
    )

    # 7. Unknown critical tag (0x8999)
    bad_crit_fields = fields + [TlvField(0x8999, b"\x01\x02")]
    bad_crit_tlv = serialize_tlv_fields(bad_crit_fields)
    bad_crit_sig = privkey.sign(PROOF_V4_DOMAIN_TAG + bad_crit_tlv)
    write_vector(
        "v08_unknown_critical_tag",
        bad_crit_tlv + bad_crit_sig,
        pubkey_bytes,
        "INVALID",
        2,
        "FORMAT_ERROR",
        "Unknown critical TLV tag encountered: 0x8999",
    )

    # 8. Unknown non-critical tag (0x0999) -> MUST succeed / yield "VALID"!
    noncrit_fields = fields + [TlvField(0x0999, b"\x01\x02")]
    noncrit_tlv = serialize_tlv_fields(noncrit_fields)
    noncrit_sig = privkey.sign(PROOF_V4_DOMAIN_TAG + noncrit_tlv)
    write_vector(
        "v09_unknown_noncritical_tag",
        noncrit_tlv + noncrit_sig,
        pubkey_bytes,
        "VALID",
        0,
        "VALID",
        "Version 4 DeletionProof signature verified successfully",
    )

    # 9. Duplicate tag (0x8004 duplicate)
    dup_fields = fields + [TlvField(0x8004, b"\x02")]
    dup_tlv = bytearray()
    for f in dup_fields:
        dup_tlv.extend(f.encode())
    write_vector(
        "v10_duplicate_tag",
        bytes(dup_tlv) + signature,
        pubkey_bytes,
        "INVALID",
        2,
        "FORMAT_ERROR",
        "Duplicate TLV tag encountered",
    )

    # 10. Wrong tag order
    rev_fields = list(reversed(fields))
    rev_tlv = bytearray()
    for f in rev_fields:
        rev_tlv.extend(f.encode())
    write_vector(
        "v11_wrong_tag_order",
        bytes(rev_tlv) + signature,
        pubkey_bytes,
        "INVALID",
        2,
        "FORMAT_ERROR",
        "TLV tags not in strictly ascending order",
    )

    # 11. Unsupported version
    unsupp_v_fields = [f if f.tag != 0x8001 else TlvField(0x8001, struct.pack(">H", 99)) for f in fields]
    unsupp_v_tlv = serialize_tlv_fields(unsupp_v_fields)
    write_vector(
        "v12_unsupported_version",
        unsupp_v_tlv + signature,
        pubkey_bytes,
        "INVALID",
        2,
        "FORMAT_ERROR",
        "Unsupported proof version",
    )

    # 12. Empty file
    write_vector(
        "v13_empty_file",
        b"",
        pubkey_bytes,
        "INVALID",
        2,
        "FORMAT_ERROR",
        "Empty file input",
    )

    print("Test vectors successfully generated.")


if __name__ == "__main__":
    main()
