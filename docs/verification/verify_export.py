#!/usr/bin/env python3
"""
docs/verification/verify_export.py

Eigenständiges Python-3-Skript zur unabhängigen Prüfung von Contextra-Löschbeweisen (DeletionProof)
OHNE Contextra-Code.

QUELLEN & SPEZIFIKATION:
- crates/contextra-crypto/src/deletion_proof.rs
- crates/contextra-crypto/src/ed25519_proof.rs
- docs/EXPORT_FORMAT.md

FORMAT:
JSON DeletionProof (Version 3 Ed25519) oder Version 1/2 (HMAC-SHA256).
Die Signatur wird über das kanonische Bincode-Payload rekonstruiert und per Ed25519 bzw. HMAC verifiziert.

AUSGABE:
"VALID" (Exit Code 0)
"INVALID: <grund>" (Exit Code 1)
"""

import sys
import json
import struct
import hmac
import hashlib
from typing import Dict, Any, Tuple

try:
    from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PublicKey
    from cryptography.exceptions import InvalidSignature
    HAS_CRYPTOGRAPHY = True
except ImportError:
    HAS_CRYPTOGRAPHY = False


def encode_u32_le(val: int) -> bytes:
    return struct.pack("<I", val)


def encode_u64_le(val: int) -> bytes:
    return struct.pack("<Q", val)


def serialize_deletion_scope_bincode(scope: Dict[str, Any]) -> bytes:
    """
    Serialisiert DeletionScope in bincode 1.x Format.
    Enum DeletionScope {
        Document { doc_id: u64, tenant_id: u64 }, // Variant 0
        Collection { collection_id: u64, tenant_id: u64 }, // Variant 1
        Tenant { tenant_id: u64 }, // Variant 2
    }
    """
    if "Document" in scope:
        doc = scope["Document"]
        doc_id = doc.get("doc_id", 0)
        tenant_id = doc.get("tenant_id", 0)
        return encode_u32_le(0) + encode_u64_le(doc_id) + encode_u64_le(tenant_id)
    elif "Collection" in scope:
        col = scope["Collection"]
        collection_id = col.get("collection_id", 0)
        tenant_id = col.get("tenant_id", 0)
        return encode_u32_le(1) + encode_u64_le(collection_id) + encode_u64_le(tenant_id)
    elif "Tenant" in scope:
        ten = scope["Tenant"]
        tenant_id = ten.get("tenant_id", 0)
        return encode_u32_le(2) + encode_u64_le(tenant_id)
    else:
        raise ValueError(f"Unbekannte DeletionScope Variante: {scope}")


def serialize_deletion_layer_bincode(layer: Any) -> bytes:
    """
    Serialisiert ein DeletionLayer Enum in bincode 1.x Format.
    LsmMemtable = 0
    SsTableAllLevels = 1
    HnswIndex = 2
    WalAllSegments { seq_after: u64 } = 3
    CsrGraph = 4
    KvCacheSegments = 5
    EmbeddingCache = 6
    """
    if isinstance(layer, str):
        mapping = {
            "LsmMemtable": 0,
            "SsTableAllLevels": 1,
            "HnswIndex": 2,
            "CsrGraph": 4,
            "KvCacheSegments": 5,
            "EmbeddingCache": 6,
        }
        if layer in mapping:
            return encode_u32_le(mapping[layer])
        raise ValueError(f"Unbekannte DeletionLayer String-Variante: {layer}")
    elif isinstance(layer, dict):
        if "WalAllSegments" in layer:
            seq_after = layer["WalAllSegments"].get("seq_after", 0)
            return encode_u32_le(3) + encode_u64_le(seq_after)
        raise ValueError(f"Unbekannte DeletionLayer Dict-Variante: {layer}")
    else:
        raise ValueError(f"Ungültiger DeletionLayer Typ: {type(layer)}")


def serialize_covered_layers_bincode(covered_layers: list) -> bytes:
    res = encode_u64_le(len(covered_layers))
    for layer in covered_layers:
        res += serialize_deletion_layer_bincode(layer)
    return res


def serialize_excluded_scope_bincode(scope: str) -> bytes:
    """
    ConsolidatedAndDistilled = 0
    LlmParameterMemory = 1
    """
    mapping = {
        "ConsolidatedAndDistilled": 0,
        "LlmParameterMemory": 1,
    }
    if scope in mapping:
        return encode_u32_le(mapping[scope])
    raise ValueError(f"Unbekannte ExcludedScope Variante: {scope}")


def serialize_excluded_scopes_bincode(excluded_scopes: list) -> bytes:
    res = encode_u64_le(len(excluded_scopes))
    for sc in excluded_scopes:
        res += serialize_excluded_scope_bincode(sc)
    return res


def construct_v3_payload(proof: Dict[str, Any]) -> bytes:
    """
    Konstruiert das v3-Payload gemäß deletion_proof.rs construct_v3_payload().
    """
    scope_bytes = serialize_deletion_scope_bincode(proof["scope"])
    deleted_keys_hash = bytes(proof["deleted_keys_hash"])
    if len(deleted_keys_hash) != 32:
        raise ValueError(f"deleted_keys_hash muss 32 Bytes haben, hat {len(deleted_keys_hash)}")

    tx_bytes = encode_u64_le(proof.get("deleted_after_tx", 0))
    timestamp_bytes = encode_u64_le(proof.get("timestamp", 0))

    covered_layers_bytes = serialize_covered_layers_bincode(proof.get("covered_layers", []))
    excluded_scopes_bytes = serialize_excluded_scopes_bincode(proof.get("excluded_scopes", []))

    wal_receipt = proof.get("wal_chain_receipt")
    receipt_part = bytes(wal_receipt) if wal_receipt is not None else b""

    audit_pos = proof.get("audit_chain_position")
    audit_pos_part = encode_u64_le(audit_pos) if audit_pos is not None else b""

    return (
        scope_bytes
        + deleted_keys_hash
        + tx_bytes
        + timestamp_bytes
        + covered_layers_bytes
        + excluded_scopes_bytes
        + receipt_part
        + audit_pos_part
    )


def verify_proof(proof_data: Dict[str, Any], pubkey_bytes: bytes = None, hmac_key: bytes = None) -> Tuple[bool, str]:
    sig_version = proof_data.get("signature_version", 1)

    if sig_version == 3:
        if not HAS_CRYPTOGRAPHY:
            return False, "Python 'cryptography' Modul nicht installiert (Ed25519 Verifikation nicht möglich)"

        if not pubkey_bytes or len(pubkey_bytes) != 32:
            return False, "Ungültiger oder fehlender Ed25519 Public Key (32 Bytes erforderlich)"

        try:
            payload = construct_v3_payload(proof_data)
            signature = bytes(proof_data.get("signature", []))
            if len(signature) != 64:
                return False, f"Ungültige Signaturlänge: {len(signature)} Bytes (erwartet 64)"

            public_key = Ed25519PublicKey.from_public_bytes(pubkey_bytes)
            public_key.verify(signature, payload)
            return True, "VALID"
        except InvalidSignature:
            return False, "Ed25519 Signatur ungültig oder Proof manipuliert"
        except Exception as e:
            return False, f"Verifikationsfehler: {str(e)}"

    elif sig_version in (1, 2):
        if not hmac_key:
            return False, "Fehlender HMAC Schlüssel für Version 1/2 Proof"

        scope_bytes = serialize_deletion_scope_bincode(proof_data["scope"])
        deleted_keys_hash = bytes(proof_data["deleted_keys_hash"])
        tx_bytes = encode_u64_le(proof_data.get("deleted_after_tx", 0))

        if sig_version == 1:
            data = scope_bytes + deleted_keys_hash + tx_bytes
        else:
            covered_layers_bytes = serialize_covered_layers_bincode(proof_data.get("covered_layers", []))
            excluded_scopes_bytes = serialize_excluded_scopes_bincode(proof_data.get("excluded_scopes", []))
            wal_receipt = proof_data.get("wal_chain_receipt")
            receipt_part = bytes(wal_receipt) if wal_receipt is not None else b""
            data = scope_bytes + deleted_keys_hash + tx_bytes + covered_layers_bytes + excluded_scopes_bytes + receipt_part

        computed_sig = hmac.new(hmac_key, data, hashlib.sha256).digest()
        provided_sig = bytes(proof_data.get("signature", []))

        if hmac.compare_digest(computed_sig, provided_sig):
            return True, "VALID"
        else:
            return False, "HMAC-SHA256 Signatur ungültig"
    else:
        return False, f"Nicht unterstützte signature_version: {sig_version}"


def main():
    if len(sys.argv) < 2:
        print("Verwendung: python3 verify_export.py <export_json_path> [<pubkey_or_hmackey_hex_or_file>]")
        sys.exit(1)

    export_path = sys.argv[1]
    key_arg = sys.argv[2] if len(sys.argv) > 2 else None

    try:
        with open(export_path, "r", encoding="utf-8") as f:
            proof_data = json.load(f)
    except Exception as e:
        print(f"INVALID: Datei konnte nicht gelesen/geparst werden ({e})")
        sys.exit(1)

    key_bytes = None
    if key_arg:
        # Versuche key_arg als Pfad zu einer Datei zu lesen, sonst als Hex-String
        try:
            with open(key_arg, "rb") as f:
                key_bytes = f.read()
        except OSError:
            try:
                key_bytes = bytes.fromhex(key_arg)
            except ValueError:
                key_bytes = key_arg.encode("utf-8")

    sig_version = proof_data.get("signature_version", 1)
    if sig_version == 3:
        valid, msg = verify_proof(proof_data, pubkey_bytes=key_bytes)
    else:
        valid, msg = verify_proof(proof_data, hmac_key=key_bytes)

    if valid:
        print("VALID")
        sys.exit(0)
    else:
        print(f"INVALID: {msg}")
        sys.exit(1)


if __name__ == "__main__":
    main()
