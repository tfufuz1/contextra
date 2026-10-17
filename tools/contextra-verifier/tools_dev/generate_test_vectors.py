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

# Deterministic key seeds for reproducible test vector generation
TEST_PRIVKEY_BYTES = b"\x01" * 32
WRONG_PRIVKEY_BYTES = b"\x02" * 32


def build_canonical_v4_fields(
    pubkey_bytes: bytes,
    db_instance_id: bytes = b"db-instance-test",
    stage: int = 2,
    supersedes: bytes = b"",
    deleted_keys_hash: bytes = b"\x02" * 32,
    observed_empty: int = 0,
    deleted_after_tx: int = 100,
    claims_list: list = None,
    excluded_scope_ids: list = None,
    environment_tlv: bytes = b"",
    issued_wall_ms: int = 1700000000000,
    issued_mono_ctr: int = 10,
) -> list:
    key_id = blake3.blake3(pubkey_bytes).digest()

    # Scope: Document { doc_id: 42, tenant_id: 1 }
    scope_bincode = encode_u32_le(0) + encode_u64_le(42) + encode_u64_le(1)

    if claims_list is None:
        claims_list = [
            {
                "layer_id": 1,
                "assurance": 1,
                "claim_text_id": 2,
                "witness_digest": b"\x04" * 32,
                "scanned_at_tx": 100,
                "evidence_tlv": b"\x10\x20",
            }
        ]

    claims_payload = bytearray()
    for c in claims_list:
        c_fields = [
            TlvField(0x0001, struct.pack(">H", c["layer_id"])),
            TlvField(0x0002, bytes([c["assurance"]])),
            TlvField(0x0003, struct.pack(">H", c["claim_text_id"])),
            TlvField(0x0004, c["witness_digest"]),
            TlvField(0x0005, struct.pack(">Q", c["scanned_at_tx"])),
            TlvField(0x0006, c["evidence_tlv"]),
        ]
        c_bytes = serialize_tlv_fields(c_fields)
        claims_payload.extend(struct.pack(">I", len(c_bytes)))
        claims_payload.extend(c_bytes)

    if excluded_scope_ids is None:
        excluded_scope_ids = [1, 2, 3, 4, 5, 6, 7, 8, 9]

    excl_payload = b"".join(struct.pack(">H", eid) for eid in excluded_scope_ids)

    fields = [
        TlvField(0x8001, struct.pack(">H", 4)),
        TlvField(0x8002, b"\x01" * 32),
        TlvField(0x8003, db_instance_id),
        TlvField(0x8004, bytes([stage])),
        TlvField(0x8005, supersedes),
        TlvField(0x8006, scope_bincode),
        TlvField(0x8007, b"doc:42"),
        TlvField(0x8008, deleted_keys_hash),
        TlvField(0x8009, b"\x03" * 32),
        TlvField(0x800A, bytes([observed_empty])),
        TlvField(0x800B, struct.pack(">Q", deleted_after_tx)),
        TlvField(0x800C, bytes(claims_payload)),
        TlvField(0x800D, excl_payload),
        TlvField(0x800E, b""),
        TlvField(0x800F, environment_tlv),
        TlvField(0x8010, b"\x05" * 32),
        TlvField(0x8011, struct.pack(">Q", 1) + b"\x00" * 32),
        TlvField(0x8012, struct.pack(">Q", issued_wall_ms)),
        TlvField(0x8013, struct.pack(">Q", issued_mono_ctr)),
        TlvField(0x8014, key_id),
    ]
    return fields


def sign_proof_fields(fields: list, privkey: Ed25519PrivateKey) -> bytes:
    canonical_tlv = serialize_tlv_fields(fields)
    signed_bytes = PROOF_V4_DOMAIN_TAG + canonical_tlv
    signature = privkey.sign(signed_bytes)
    return canonical_tlv + signature


def main():
    testvectors_dir = os.path.join(verifier_dir, "testvectors")
    os.makedirs(testvectors_dir, exist_ok=True)

    privkey = Ed25519PrivateKey.from_private_bytes(TEST_PRIVKEY_BYTES)
    pubkey = privkey.public_key()
    pubkey_bytes = pubkey.public_bytes(Encoding.Raw, PublicFormat.Raw)

    fields = build_canonical_v4_fields(pubkey_bytes)
    valid_proof = sign_proof_fields(fields, privkey)

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

    # 3. Mutated tag version
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

    # 4. Mutated tag tx
    mut_tx = bytearray(valid_proof)
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

    # 5. Wrong public key
    wrong_privkey = Ed25519PrivateKey.from_private_bytes(WRONG_PRIVKEY_BYTES)
    wrong_key = wrong_privkey.public_key().public_bytes(Encoding.Raw, PublicFormat.Raw)
    write_vector(
        "v05_wrong_public_key",
        valid_proof,
        wrong_key,
        "INVALID",
        1,
        "INVALID",
        "Signer key ID mismatch",
    )

    # 6. Truncated file
    write_vector(
        "v06_truncated_file",
        valid_proof[:50],
        pubkey_bytes,
        "INVALID",
        2,
        "FORMAT_ERROR",
        "Proof binary payload too short",
    )

    # 7. Trailing bytes
    write_vector(
        "v07_trailing_bytes",
        valid_proof + b"\x00\x00\x00\x00",
        pubkey_bytes,
        "INVALID",
        2,
        "FORMAT_ERROR",
        "Truncated TLV payload",
    )

    # 8. Unknown critical tag (0x8999)
    bad_crit_fields = fields + [TlvField(0x8999, b"\x01\x02")]
    bad_crit_proof = sign_proof_fields(bad_crit_fields, privkey)
    write_vector(
        "v08_unknown_critical_tag",
        bad_crit_proof,
        pubkey_bytes,
        "INVALID",
        2,
        "FORMAT_ERROR",
        "Unknown critical TLV tag encountered: 0x8999",
    )

    # 9. Unknown non-critical tag (0x0999) -> VALID
    noncrit_fields = fields + [TlvField(0x0999, b"\x01\x02")]
    noncrit_proof = sign_proof_fields(noncrit_fields, privkey)
    write_vector(
        "v09_unknown_noncritical_tag",
        noncrit_proof,
        pubkey_bytes,
        "VALID",
        0,
        "VALID",
        "Version 4 DeletionProof signature verified successfully",
    )

    # 10. Duplicate tag (0x8004 duplicate placed right after 0x8004)
    dup_fields = fields[:4] + [TlvField(0x8004, b"\x02")] + fields[4:]
    dup_tlv = bytearray()
    for f in dup_fields:
        dup_tlv.extend(f.encode())
    sig = privkey.sign(PROOF_V4_DOMAIN_TAG + dup_tlv)
    write_vector(
        "v10_duplicate_tag",
        bytes(dup_tlv) + sig,
        pubkey_bytes,
        "INVALID",
        2,
        "FORMAT_ERROR",
        "Duplicate TLV tag encountered",
    )

    # 11. Wrong tag order
    rev_fields = list(reversed(fields))
    rev_tlv = bytearray()
    for f in rev_fields:
        rev_tlv.extend(f.encode())
    rev_sig = privkey.sign(PROOF_V4_DOMAIN_TAG + rev_tlv)
    write_vector(
        "v11_wrong_tag_order",
        bytes(rev_tlv) + rev_sig,
        pubkey_bytes,
        "INVALID",
        2,
        "FORMAT_ERROR",
        "TLV tags not in strictly ascending order",
    )

    # 12. Unsupported version
    unsupp_v_fields = [f if f.tag != 0x8001 else TlvField(0x8001, struct.pack(">H", 99)) for f in fields]
    unsupp_v_proof = sign_proof_fields(unsupp_v_fields, privkey)
    write_vector(
        "v12_unsupported_version",
        unsupp_v_proof,
        pubkey_bytes,
        "INVALID",
        2,
        "FORMAT_ERROR",
        "Unsupported proof version",
    )

    # 13. Empty file
    write_vector(
        "v13_empty_file",
        b"",
        pubkey_bytes,
        "INVALID",
        2,
        "FORMAT_ERROR",
        "Empty file input",
    )

    # --- NEW POLICY VIOLATION VECTORS (v14..v25) ---

    # 14. Missing / empty db_instance_id
    v14_fields = build_canonical_v4_fields(pubkey_bytes, db_instance_id=b"")
    v14_proof = sign_proof_fields(v14_fields, privkey)
    write_vector(
        "v14_missing_db_instance_id",
        v14_proof,
        pubkey_bytes,
        "INVALID",
        2,
        "POLICY_VIOLATION",
        "db_instance_id must not be empty",
    )

    # 15. Invalid stage (stage = 3)
    v15_fields = build_canonical_v4_fields(pubkey_bytes, stage=3)
    v15_proof = sign_proof_fields(v15_fields, privkey)
    write_vector(
        "v15_invalid_stage",
        v15_proof,
        pubkey_bytes,
        "INVALID",
        2,
        "POLICY_VIOLATION",
        "Invalid proof stage 3 (expected 1 or 2)",
    )

    # 16. Invalid supersedes self-reference
    v16_fields = build_canonical_v4_fields(pubkey_bytes, supersedes=b"\x01" * 32)
    v16_proof = sign_proof_fields(v16_fields, privkey)
    write_vector(
        "v16_invalid_supersedes_self",
        v16_proof,
        pubkey_bytes,
        "INVALID",
        2,
        "POLICY_VIOLATION",
        "Invalid supersedes self-reference or zeroes",
    )

    # 17. Invalid supersedes all zeroes
    v17_fields = build_canonical_v4_fields(pubkey_bytes, supersedes=b"\x00" * 32)
    v17_proof = sign_proof_fields(v17_fields, privkey)
    write_vector(
        "v17_invalid_supersedes_zeroes",
        v17_proof,
        pubkey_bytes,
        "INVALID",
        2,
        "POLICY_VIOLATION",
        "Invalid supersedes self-reference or zeroes",
    )

    # 18. Invalid claim_text_id (claim_text_id = 99)
    c18 = [
        {
            "layer_id": 1,
            "assurance": 1,
            "claim_text_id": 99,
            "witness_digest": b"\x04" * 32,
            "scanned_at_tx": 100,
            "evidence_tlv": b"\x10\x20",
        }
    ]
    v18_fields = build_canonical_v4_fields(pubkey_bytes, claims_list=c18)
    v18_proof = sign_proof_fields(v18_fields, privkey)
    write_vector(
        "v18_invalid_claim_text_id",
        v18_proof,
        pubkey_bytes,
        "INVALID",
        2,
        "POLICY_VIOLATION",
        "Unallowed claim_text_id: 99",
    )

    # 19. Invalid assurance level (assurance = 5)
    c19 = [
        {
            "layer_id": 1,
            "assurance": 5,
            "claim_text_id": 2,
            "witness_digest": b"\x04" * 32,
            "scanned_at_tx": 100,
            "evidence_tlv": b"\x10\x20",
        }
    ]
    v19_fields = build_canonical_v4_fields(pubkey_bytes, claims_list=c19)
    v19_proof = sign_proof_fields(v19_fields, privkey)
    write_vector(
        "v19_invalid_assurance_level",
        v19_proof,
        pubkey_bytes,
        "INVALID",
        2,
        "POLICY_VIOLATION",
        "Invalid assurance level 5",
    )

    # 20. Missing mandatory excluded scope (omits ID 2)
    v20_fields = build_canonical_v4_fields(pubkey_bytes, excluded_scope_ids=[1, 3, 4, 5, 6, 7, 8, 9])
    v20_proof = sign_proof_fields(v20_fields, privkey)
    write_vector(
        "v20_missing_mandatory_exclusions",
        v20_proof,
        pubkey_bytes,
        "INVALID",
        2,
        "POLICY_VIOLATION",
        "Missing mandatory excluded scopes: [2]",
    )

    # 21. Invalid environment tri-state value (\x03)
    env_tlv = struct.pack(">HI", 0x0003, 1) + b"\x03"
    v21_fields = build_canonical_v4_fields(pubkey_bytes, environment_tlv=env_tlv)
    v21_proof = sign_proof_fields(v21_fields, privkey)
    write_vector(
        "v21_invalid_environment_tristate",
        v21_proof,
        pubkey_bytes,
        "INVALID",
        2,
        "POLICY_VIOLATION",
        "Invalid environment tri-state value 3 for tag 0x0003",
    )

    # 22. Claim scanned_at_tx earlier than deleted_after_tx (90 < 100)
    c22 = [
        {
            "layer_id": 1,
            "assurance": 1,
            "claim_text_id": 2,
            "witness_digest": b"\x04" * 32,
            "scanned_at_tx": 90,
            "evidence_tlv": b"\x10\x20",
        }
    ]
    v22_fields = build_canonical_v4_fields(pubkey_bytes, deleted_after_tx=100, claims_list=c22)
    v22_proof = sign_proof_fields(v22_fields, privkey)
    write_vector(
        "v22_scanned_before_deleted_tx",
        v22_proof,
        pubkey_bytes,
        "INVALID",
        2,
        "POLICY_VIOLATION",
        "Claim scanned_at_tx (90) earlier than deleted_after_tx (100)",
    )

    # 23. Zero deleted_keys_hash requires observed_empty == True
    v23_fields = build_canonical_v4_fields(pubkey_bytes, deleted_keys_hash=b"\x00" * 32, observed_empty=0)
    v23_proof = sign_proof_fields(v23_fields, privkey)
    write_vector(
        "v23_zero_deleted_keys_hash_not_empty",
        v23_proof,
        pubkey_bytes,
        "INVALID",
        2,
        "POLICY_VIOLATION",
        "Zero deleted_keys_hash requires observed_empty == true",
    )

    # 24. Zero issued_wall_ms
    v24_fields = build_canonical_v4_fields(pubkey_bytes, issued_wall_ms=0)
    v24_proof = sign_proof_fields(v24_fields, privkey)
    write_vector(
        "v24_zero_issued_wall_ms",
        v24_proof,
        pubkey_bytes,
        "INVALID",
        2,
        "POLICY_VIOLATION",
        "Invalid timestamp or zero monotonic counter",
    )

    # 25. Valid S4 proof (Crypto Erased, claim_text_id = 6, excluded scopes omitting ID 3)
    c25 = [
        {
            "layer_id": 1,
            "assurance": 4,
            "claim_text_id": 6,
            "witness_digest": b"\x04" * 32,
            "scanned_at_tx": 100,
            "evidence_tlv": b"\x10\x20",
        }
    ]
    v25_fields = build_canonical_v4_fields(
        pubkey_bytes, claims_list=c25, excluded_scope_ids=[1, 2, 4, 5, 6, 7, 8, 9]
    )
    v25_proof = sign_proof_fields(v25_fields, privkey)
    write_vector(
        "v25_valid_s4_assurance",
        v25_proof,
        pubkey_bytes,
        "VALID",
        0,
        "VALID",
        "Version 4 DeletionProof signature verified successfully",
    )

    print("Test vectors v01..v25 successfully generated.")


if __name__ == "__main__":
    main()
