"""Verification pipeline for Contextra DeletionProof v4 and legacy proofs."""

import json
import struct
from typing import Tuple, Dict, Any, Optional

from .tlv import (
    parse_tlv_stream,
    check_unknown_critical_tags,
    TlvFormatError,
    TlvField,
    MAX_INPUT_SIZE,
)
from .canon import (
    construct_v4_signed_bytes,
    serialize_tlv_fields,
    construct_v3_payload,
    serialize_deletion_scope_bincode,
    serialize_covered_layers_bincode,
    serialize_excluded_scopes_bincode,
)
from .crypto import (
    blake3_hash,
    verify_ed25519,
    verify_hmac_sha256,
    CryptoError,
)


def verify_legacy_json(proof_data: Dict[str, Any], key_bytes: bytes) -> Tuple[int, str, str]:
    """Verifies legacy proof versions 1, 2, and 3."""
    sig_ver = proof_data.get("signature_version", 1)
    if sig_ver not in (1, 2, 3):
        return 2, "FORMAT_ERROR", f"Unsupported legacy signature_version: {sig_ver}"

    try:
        raw_sig = proof_data.get("signature", [])
        if isinstance(raw_sig, list):
            sig_bytes = bytes(raw_sig)
        elif isinstance(raw_sig, str):
            sig_bytes = bytes.fromhex(raw_sig)
        else:
            sig_bytes = bytes(raw_sig)
    except Exception as e:
        return 2, "FORMAT_ERROR", f"Invalid signature format in JSON: {e}"

    if sig_ver == 3:
        if len(key_bytes) != 32:
            return 3, "USAGE_ERROR", "Ed25519 public key must be 32 bytes for version 3 proof"
        try:
            payload = construct_v3_payload(proof_data)
            valid = verify_ed25519(key_bytes, sig_bytes, payload)
            if valid:
                return 0, "VALID", "LEGACY-FORMAT: v3 Ed25519 proof verified"
            return 1, "INVALID", "Ed25519 signature verification failed for v3 proof"
        except CryptoError as ce:
            return 3, "USAGE_ERROR", str(ce)
        except Exception as e:
            return 1, "INVALID", f"Legacy v3 verification failed: {e}"

    elif sig_ver in (1, 2):
        if not key_bytes:
            return 3, "USAGE_ERROR", "Missing HMAC key for version 1/2 proof"
        try:
            scope_bytes = serialize_deletion_scope_bincode(proof_data["scope"])
            deleted_keys_hash = bytes(proof_data["deleted_keys_hash"])
            tx_bytes = struct.pack("<Q", proof_data.get("deleted_after_tx", 0))

            if sig_ver == 1:
                payload = scope_bytes + deleted_keys_hash + tx_bytes
            else:
                covered_bytes = serialize_covered_layers_bincode(proof_data.get("covered_layers", []))
                excluded_bytes = serialize_excluded_scopes_bincode(proof_data.get("excluded_scopes", []))
                receipt = proof_data.get("wal_chain_receipt")
                receipt_part = bytes(receipt) if receipt is not None else b""
                payload = scope_bytes + deleted_keys_hash + tx_bytes + covered_bytes + excluded_bytes + receipt_part

            valid = verify_hmac_sha256(key_bytes, sig_bytes, payload)
            if valid:
                return 0, "VALID", f"LEGACY-FORMAT: v{sig_ver} HMAC proof verified"
            return 1, "INVALID", f"HMAC-SHA256 signature verification failed for v{sig_ver} proof"
        except Exception as e:
            return 1, "INVALID", f"Legacy v{sig_ver} verification failed: {e}"

    return 2, "FORMAT_ERROR", f"Unsupported signature_version: {sig_ver}"


def verify_proof(proof_bytes: bytes, pubkey_or_hmac_bytes: bytes) -> Tuple[int, str, str]:
    """
    Main entry point for verifying a DeletionProof (Version 4 TLV or legacy v1-v3).
    Returns (exit_code, category, reason).
    """
    if not proof_bytes:
        return 2, "FORMAT_ERROR", "Empty file input"
    if len(proof_bytes) > MAX_INPUT_SIZE:
        return 2, "FORMAT_ERROR", f"Input file size exceeds {MAX_INPUT_SIZE} bytes limit"

    # Attempt to parse as JSON first
    is_json = False
    json_data = None
    try:
        text = proof_bytes.decode("utf-8")
        json_data = json.loads(text)
        if isinstance(json_data, dict):
            is_json = True
    except Exception:
        is_json = False

    if is_json and json_data is not None:
        version = json_data.get("version") or json_data.get("signature_version", 1)
        if version in (1, 2, 3):
            return verify_legacy_json(json_data, pubkey_or_hmac_bytes)

        if version == 4:
            # Handle JSON representation of v4
            sig_hex = json_data.get("signature")
            if isinstance(sig_hex, list):
                signature_bytes = bytes(sig_hex)
            elif isinstance(sig_hex, str):
                signature_bytes = bytes.fromhex(sig_hex)
            else:
                signature_bytes = bytes(sig_hex) if sig_hex else b""

            if "canonical_tlv_hex" in json_data:
                proof_bytes = bytes.fromhex(json_data["canonical_tlv_hex"])
            elif "raw_tlv" in json_data:
                proof_bytes = bytes(json_data["raw_tlv"])
            else:
                return 2, "FORMAT_ERROR", "JSON v4 proof missing canonical_tlv_hex or raw_tlv"
        else:
            return 2, "FORMAT_ERROR", f"Unsupported proof version: {version}"
    else:
        # Binary stream: signature is trailing 64 bytes or embedded in TLV container
        signature_bytes = b""

    # At this point, proof_bytes contains the binary TLV payload stream
    if len(proof_bytes) < 64:
        return 2, "FORMAT_ERROR", "Proof binary payload too short"

    # If signature wasn't provided separately in JSON wrapper, check if last 64 bytes is signature
    # or if signature is passed in wrapper.
    # In v4 binary TLV representation, raw_canonical_tlv (all fields 0x8001..0x8014) is signed,
    # and signature is either 64 bytes appended at end or extracted from JSON.
    if not signature_bytes:
        # Extract signature from last 64 bytes if input is concatenated payload || signature
        # or parse TLV stream directly
        if len(proof_bytes) >= 64:
            tlv_part = proof_bytes[:-64]
            signature_bytes = proof_bytes[-64:]
            try:
                fields = parse_tlv_stream(tlv_part)
                proof_bytes = tlv_part
            except TlvFormatError:
                # Try parsing entire proof_bytes as TLV
                try:
                    fields = parse_tlv_stream(proof_bytes)
                except TlvFormatError as e:
                    return 2, "FORMAT_ERROR", str(e)
        else:
            try:
                fields = parse_tlv_stream(proof_bytes)
            except TlvFormatError as e:
                return 2, "FORMAT_ERROR", str(e)
    else:
        try:
            fields = parse_tlv_stream(proof_bytes)
        except TlvFormatError as e:
            return 2, "FORMAT_ERROR", str(e)

    # Validate unknown critical tags
    try:
        check_unknown_critical_tags(fields)
    except TlvFormatError as e:
        return 2, "FORMAT_ERROR", str(e)

    # Extract version tag (0x8001)
    version_field = next((f for f in fields if f.tag == 0x8001), None)
    if not version_field:
        return 2, "FORMAT_ERROR", "Missing mandatory version tag (0x8001)"
    if len(version_field.value) != 2:
        return 2, "FORMAT_ERROR", "Invalid version tag length (expected 2 bytes)"
    v_num = struct.unpack(">H", version_field.value)[0]
    if v_num != 4:
        return 2, "FORMAT_ERROR", f"Unsupported proof version: {v_num} (expected 4)"

    # Extract signer_key_id tag (0x8014)
    key_id_field = next((f for f in fields if f.tag == 0x8014), None)
    if not key_id_field:
        return 2, "FORMAT_ERROR", "Missing mandatory signer_key_id tag (0x8014)"
    if len(key_id_field.value) != 32:
        return 2, "FORMAT_ERROR", f"Invalid signer_key_id length: {len(key_id_field.value)} bytes (expected 32)"

    if not pubkey_or_hmac_bytes or len(pubkey_or_hmac_bytes) != 32:
        return 3, "USAGE_ERROR", "Invalid or missing 32-byte Ed25519 public key"

    # Verify signer_key_id == BLAKE3(pubkey)
    try:
        computed_key_id = blake3_hash(pubkey_or_hmac_bytes)
    except CryptoError as ce:
        return 3, "USAGE_ERROR", str(ce)

    if computed_key_id != key_id_field.value:
        return 1, "INVALID", "Signer key ID mismatch (BLAKE3 hash of pubkey does not match tag 0x8014)"

    if len(signature_bytes) != 64:
        return 2, "FORMAT_ERROR", f"Invalid Ed25519 signature length: {len(signature_bytes)} bytes (expected 64)"

    # Reconstruct signed bytes: PROOF_V4_DOMAIN_TAG || canonical_tlv_bytes
    signed_bytes = construct_v4_signed_bytes(proof_bytes)

    # Verify Ed25519 signature
    try:
        valid = verify_ed25519(pubkey_or_hmac_bytes, signature_bytes, signed_bytes)
        if valid:
            return 0, "VALID", "Version 4 DeletionProof signature verified successfully"
        return 1, "INVALID", "Ed25519 signature verification failed or proof payload tampered"
    except CryptoError as ce:
        return 3, "USAGE_ERROR", str(ce)
    except Exception as e:
        return 1, "INVALID", f"Verification error: {e}"
