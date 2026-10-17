"""Payload reconstruction and canonical serialization helpers."""

import struct
from typing import Dict, Any, List
from .tlv import TlvField

PROOF_V4_DOMAIN_TAG = b"contextra/deletion-proof/v4\0"


def construct_v4_signed_bytes(canonical_tlv_bytes: bytes) -> bytes:
    """Computes exact signed bytes input: PROOF_V4_DOMAIN_TAG || canonical_tlv_bytes."""
    return PROOF_V4_DOMAIN_TAG + canonical_tlv_bytes


def serialize_tlv_fields(fields: List[TlvField]) -> bytes:
    """Encodes TLV fields in sorted tag order for canonical determinism."""
    sorted_fields = sorted(fields, key=lambda f: f.tag)
    out = bytearray()
    for field in sorted_fields:
        out.extend(field.encode())
    return bytes(out)


def encode_u32_le(val: int) -> bytes:
    return struct.pack("<I", val)


def encode_u64_le(val: int) -> bytes:
    return struct.pack("<Q", val)


def serialize_deletion_scope_bincode(scope: Dict[str, Any]) -> bytes:
    """Serializes DeletionScope enum into bincode 1.x format."""
    if "Document" in scope:
        doc = scope["Document"]
        doc_id = doc.get("doc_id", 0)
        if isinstance(doc_id, dict):
            doc_id = doc_id.get("DocId", 0)
        tenant_id = doc.get("tenant_id", 0)
        if isinstance(tenant_id, dict):
            tenant_id = tenant_id.get("TenantId", 0)
        return encode_u32_le(0) + encode_u64_le(doc_id) + encode_u64_le(tenant_id)
    elif "Collection" in scope:
        col = scope["Collection"]
        collection_id = col.get("collection_id", 0)
        if isinstance(collection_id, dict):
            collection_id = collection_id.get("CollectionId", 0)
        tenant_id = col.get("tenant_id", 0)
        if isinstance(tenant_id, dict):
            tenant_id = tenant_id.get("TenantId", 0)
        return encode_u32_le(1) + encode_u64_le(collection_id) + encode_u64_le(tenant_id)
    elif "Tenant" in scope:
        ten = scope["Tenant"]
        tenant_id = ten.get("tenant_id", 0)
        if isinstance(tenant_id, dict):
            tenant_id = tenant_id.get("TenantId", 0)
        return encode_u32_le(2) + encode_u64_le(tenant_id)
    else:
        raise ValueError(f"Unknown DeletionScope variant: {scope}")


def serialize_deletion_layer_bincode(layer: Any) -> bytes:
    """Serializes DeletionLayer enum into bincode 1.x format."""
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
        raise ValueError(f"Unknown DeletionLayer string variant: {layer}")
    elif isinstance(layer, dict):
        if "WalAllSegments" in layer:
            seq_after = layer["WalAllSegments"].get("seq_after", 0)
            return encode_u32_le(3) + encode_u64_le(seq_after)
        raise ValueError(f"Unknown DeletionLayer dict variant: {layer}")
    else:
        raise ValueError(f"Invalid DeletionLayer type: {type(layer)}")


def serialize_covered_layers_bincode(covered_layers: list) -> bytes:
    res = encode_u64_le(len(covered_layers))
    for layer in covered_layers:
        res += serialize_deletion_layer_bincode(layer)
    return res


def serialize_excluded_scope_bincode(scope: str) -> bytes:
    mapping = {
        "ConsolidatedAndDistilled": 0,
        "LlmParameterMemory": 1,
    }
    if scope in mapping:
        return encode_u32_le(mapping[scope])
    raise ValueError(f"Unknown ExcludedScope variant: {scope}")


def serialize_excluded_scopes_bincode(excluded_scopes: list) -> bytes:
    res = encode_u64_le(len(excluded_scopes))
    for sc in excluded_scopes:
        res += serialize_excluded_scope_bincode(sc)
    return res


def construct_v3_payload(proof: Dict[str, Any]) -> bytes:
    """Constructs v3 payload according to deletion_proof.rs construct_v3_payload()."""
    scope_bytes = serialize_deletion_scope_bincode(proof["scope"])
    deleted_keys_hash = bytes(proof["deleted_keys_hash"])
    if len(deleted_keys_hash) != 32:
        raise ValueError(f"deleted_keys_hash must be 32 bytes, got {len(deleted_keys_hash)}")

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
