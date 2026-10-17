"""Verification pipeline for Contextra DeletionProof v4 and legacy proofs."""

import json
import struct
from typing import Tuple, Dict, Any, Optional, List, Set

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

# Mandatory excluded scope IDs per spec (Rule R5)
# 1: ConsolidatedAndDistilled
# 2: LlmParameterMemory
# 3: StorageMediaRemanence
# 4: UnregisteredBackups
# 5: OsPageCacheSwapCoreDumps
# 6: ClientAndModelContext
# 7: AggregateDerivedStatistics
# 8: LogsAndTelemetryOutsideProcess
# 9: ExportedData
STANDARD_MANDATORY_EXCLUSIONS: Set[int] = {1, 2, 3, 4, 5, 6, 7, 8, 9}
S4_MANDATORY_EXCLUSIONS: Set[int] = {1, 2, 4, 5, 6, 7, 8, 9}
ALLOWED_CLAIM_TEXT_IDS: Set[int] = {1, 2, 3, 4, 5, 6, 7, 8}


def parse_sub_claims(claims_bytes: bytes) -> List[Dict[str, Any]]:
    """Parses claim objects from the 0x800C claims payload stream."""
    claims = []
    offset = 0
    while offset < len(claims_bytes):
        if len(claims_bytes) - offset < 4:
            raise TlvFormatError("Truncated claims payload length header")
        claim_len = struct.unpack(">I", claims_bytes[offset : offset + 4])[0]
        offset += 4
        if len(claims_bytes) - offset < claim_len:
            raise TlvFormatError("Truncated claim payload body")
        claim_tlv = claims_bytes[offset : offset + claim_len]
        offset += claim_len

        claim_fields = parse_tlv_stream(claim_tlv, allow_sub_tlv=True)
        claim_obj = {}
        for cf in claim_fields:
            if cf.tag == 0x0001:
                if len(cf.value) == 2:
                    claim_obj["layer_id"] = struct.unpack(">H", cf.value)[0]
            elif cf.tag == 0x0002:
                if len(cf.value) == 1:
                    claim_obj["assurance"] = cf.value[0]
            elif cf.tag == 0x0003:
                if len(cf.value) == 2:
                    claim_obj["claim_text_id"] = struct.unpack(">H", cf.value)[0]
            elif cf.tag == 0x0004:
                claim_obj["witness_digest"] = cf.value
            elif cf.tag == 0x0005:
                if len(cf.value) == 8:
                    claim_obj["scanned_at_tx"] = struct.unpack(">Q", cf.value)[0]
            elif cf.tag == 0x0006:
                claim_obj["evidence_tlv"] = cf.value
        claims.append(claim_obj)
    return claims


def check_environment_tlv(env_bytes: bytes) -> None:
    """Validates environment sub-TLV stream tri-state values."""
    if not env_bytes:
        return
    env_fields = parse_tlv_stream(env_bytes, allow_sub_tlv=True)
    for ef in env_fields:
        # EnvTri fields are tags 0x0003 .. 0x0008
        if 0x0003 <= ef.tag <= 0x0008:
            if len(ef.value) != 1 or ef.value[0] not in (0, 1, 2):
                val_str = str(ef.value[0]) if len(ef.value) > 0 else "empty"
                raise ValueError(f"Invalid environment tri-state value {val_str} for tag 0x{ef.tag:04X}")


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
    if not signature_bytes:
        if len(proof_bytes) >= 64:
            tlv_part = proof_bytes[:-64]
            signature_bytes = proof_bytes[-64:]
            try:
                fields = parse_tlv_stream(tlv_part)
                proof_bytes = tlv_part
            except TlvFormatError:
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

    # Validate unknown critical tags (Rule R13)
    try:
        check_unknown_critical_tags(fields)
    except TlvFormatError as e:
        return 2, "FORMAT_ERROR", str(e)

    # Extract version tag (0x8001) (Rule R2)
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

    # Verify signer_key_id == BLAKE3(pubkey) (Rule R10)
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

    # Verify Ed25519 signature (Rule R1)
    try:
        valid = verify_ed25519(pubkey_or_hmac_bytes, signature_bytes, signed_bytes)
        if not valid:
            return 1, "INVALID", "Ed25519 signature verification failed or proof payload tampered"
    except CryptoError as ce:
        return 3, "USAGE_ERROR", str(ce)
    except Exception as e:
        return 1, "INVALID", f"Verification error: {e}"

    # --- POLICY VERIFICATION (Rules R3-R16) ---

    # 1. proof_id (0x8002)
    proof_id_field = next((f for f in fields if f.tag == 0x8002), None)
    proof_id = proof_id_field.value if proof_id_field else b""

    # 2. db_instance_id (0x8003)
    db_inst_field = next((f for f in fields if f.tag == 0x8003), None)
    if not db_inst_field:
        return 2, "POLICY_VIOLATION", "Missing mandatory db_instance_id tag (0x8003)"
    if len(db_inst_field.value) == 0:
        return 2, "POLICY_VIOLATION", "db_instance_id must not be empty"

    # 3. stage (0x8004) & supersedes (0x8005) (Rules R11, R15)
    stage_field = next((f for f in fields if f.tag == 0x8004), None)
    if not stage_field or len(stage_field.value) != 1:
        return 2, "POLICY_VIOLATION", "Missing or invalid proof stage tag (0x8004)"
    stage = stage_field.value[0]
    if stage not in (1, 2):
        return 2, "POLICY_VIOLATION", f"Invalid proof stage {stage} (expected 1 or 2)"

    supersedes_field = next((f for f in fields if f.tag == 0x8005), None)
    if supersedes_field and len(supersedes_field.value) > 0:
        if len(supersedes_field.value) != 32:
            return 2, "POLICY_VIOLATION", "Invalid supersedes length (expected 32 bytes)"
        if supersedes_field.value == bytes(32) or supersedes_field.value == proof_id:
            return 2, "POLICY_VIOLATION", "Invalid supersedes self-reference or zeroes"

    # 4. deleted_keys_hash (0x8008) vs observed_empty (0x800A) (Rule R12)
    del_keys_field = next((f for f in fields if f.tag == 0x8008), None)
    obs_empty_field = next((f for f in fields if f.tag == 0x800A), None)
    if del_keys_field and obs_empty_field and len(del_keys_field.value) == 32 and len(obs_empty_field.value) == 1:
        is_zero_keys = (del_keys_field.value == bytes(32))
        is_observed_empty = (obs_empty_field.value[0] != 0)
        if is_zero_keys and not is_observed_empty:
            return 2, "POLICY_VIOLATION", "Zero deleted_keys_hash requires observed_empty == true"

    # 5. deleted_after_tx (0x800B)
    tx_field = next((f for f in fields if f.tag == 0x800B), None)
    deleted_after_tx = struct.unpack(">Q", tx_field.value)[0] if (tx_field and len(tx_field.value) == 8) else 0

    # 6. claims (0x800C) (Rules R3, R8, R9, R14, Closed Enumeration INV-D7)
    claims_field = next((f for f in fields if f.tag == 0x800C), None)
    if not claims_field:
        return 2, "POLICY_VIOLATION", "Missing required claims tag (0x800C)"
    try:
        sub_claims = parse_sub_claims(claims_field.value)
    except TlvFormatError as e:
        return 2, "FORMAT_ERROR", f"Malformed claims payload: {e}"

    if len(sub_claims) == 0:
        return 2, "POLICY_VIOLATION", "Missing required claims"

    has_s4 = False
    for claim in sub_claims:
        assurance = claim.get("assurance")
        if assurance is None or assurance > 4:
            return 2, "POLICY_VIOLATION", f"Invalid assurance level {assurance}"
        if assurance == 4:
            has_s4 = True

        claim_text_id = claim.get("claim_text_id")
        if claim_text_id is None or claim_text_id not in ALLOWED_CLAIM_TEXT_IDS:
            return 2, "POLICY_VIOLATION", f"Unallowed claim_text_id: {claim_text_id}"

        scanned_at_tx = claim.get("scanned_at_tx", 0)
        if scanned_at_tx < deleted_after_tx:
            return 2, "POLICY_VIOLATION", f"Claim scanned_at_tx ({scanned_at_tx}) earlier than deleted_after_tx ({deleted_after_tx})"

    # 7. excluded_scopes (0x800D) (Rule R5)
    excl_field = next((f for f in fields if f.tag == 0x800D), None)
    excluded_scope_ids = set()
    if excl_field and len(excl_field.value) >= 2:
        for i in range(0, len(excl_field.value), 2):
            if i + 2 <= len(excl_field.value):
                excluded_scope_ids.add(struct.unpack(">H", excl_field.value[i : i + 2])[0])

    mandatory_ids = S4_MANDATORY_EXCLUSIONS if has_s4 else STANDARD_MANDATORY_EXCLUSIONS
    missing_mandatory = sorted(mandatory_ids - excluded_scope_ids)
    if missing_mandatory:
        return 2, "POLICY_VIOLATION", f"Missing mandatory excluded scopes: {missing_mandatory}"

    # 8. environment_tlv (0x800F) (Tri-State Validation)
    env_field = next((f for f in fields if f.tag == 0x800F), None)
    if env_field and env_field.value:
        try:
            check_environment_tlv(env_field.value)
        except ValueError as ve:
            return 2, "POLICY_VIOLATION", str(ve)
        except TlvFormatError as te:
            return 2, "FORMAT_ERROR", str(te)

    # 9. issued_wall_ms (0x8012) & issued_mono_ctr (0x8013) (Rule R6)
    wall_field = next((f for f in fields if f.tag == 0x8012), None)
    mono_field = next((f for f in fields if f.tag == 0x8013), None)
    issued_wall_ms = struct.unpack(">Q", wall_field.value)[0] if (wall_field and len(wall_field.value) == 8) else 0
    issued_mono_ctr = struct.unpack(">Q", mono_field.value)[0] if (mono_field and len(mono_field.value) == 8) else 0

    if issued_wall_ms == 0 or issued_mono_ctr == 0:
        return 2, "POLICY_VIOLATION", "Invalid timestamp or zero monotonic counter"

    return 0, "VALID", "Version 4 DeletionProof signature verified successfully"
