//! Durables Auftragsjournal für den Erasure-Kernel (§5).
//!
//! Alle Zugriffe erfolgen ausschließlich über [`StoreIo`] und [`VfsFile`] in `spawn_blocking`.
//! Handgeschriebenes Little-Endian-Binärlayout ohne `serde_json` oder `bincode`.

use crate::io_seam::StoreIo;
use contextra_ports::erasure::{DocScope, KeyPattern, KeyScope, ResolvedScope, Verdict};
use contextra_ports::scanner::WitnessLayer;
use contextra_ports::vfs::{VfsFile, VfsOpenOptions};
use contextra_types::{
    CheckpointTaintPolicy, DocId, ErasureError, ErasureException, ErasureId, ErasurePolicy,
    ErasureReason, LayerErrorKind, ReinsertPolicy, TargetKind, TenantId,
};
use parking_lot::Mutex;
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

const JOURNAL_MAGIC: &[u8; 8] = b"CTXERJ01";
const JOURNAL_VERSION: u16 = 1;
const JOURNAL_HEADER_LEN: usize = 16;
const FRAME_HEADER_LEN: usize = 8; // len(4) + crc(4)

/// Dateiname des aktiven Journals im Erasure-Verzeichnis.
pub(crate) const JOURNAL_FILE_NAME: &str = "journal.log";
/// Temporärer Dateiname während der Verdichtung (Rollover).
const JOURNAL_NEW_NAME: &str = "journal.new";

/// Layer-Tag-Kodierung (§5.2, normativ).
pub(crate) fn encode_layer(layer: &WitnessLayer, out: &mut Vec<u8>) -> Result<(), ErasureError> {
    match layer {
        WitnessLayer::LsmMemtable => out.push(1),
        WitnessLayer::SsTableAllLevels => out.push(2),
        WitnessLayer::HnswIndex => out.push(3),
        WitnessLayer::WalAllSegments => out.push(4),
        WitnessLayer::CsrGraph => out.push(5),
        WitnessLayer::KvCacheSegments => out.push(6),
        WitnessLayer::EmbeddingCache => out.push(7),
        WitnessLayer::Custom(s) => {
            let b = s.as_bytes();
            if b.len() > 64 {
                return Err(ErasureError::InvalidTarget("custom layer name exceeds 64 bytes"));
            }
            out.push(0xFF);
            out.push(b.len() as u8);
            out.extend_from_slice(b);
        }
    }
    Ok(())
}

/// Dekodiert einen Layer-Tag aus `CursorReader`.
pub(crate) fn decode_layer(r: &mut ByteReader<'_>) -> Result<WitnessLayer, ErasureError> {
    let tag = r.read_u8()?;
    match tag {
        1 => Ok(WitnessLayer::LsmMemtable),
        2 => Ok(WitnessLayer::SsTableAllLevels),
        3 => Ok(WitnessLayer::HnswIndex),
        4 => Ok(WitnessLayer::WalAllSegments),
        5 => Ok(WitnessLayer::CsrGraph),
        6 => Ok(WitnessLayer::KvCacheSegments),
        7 => Ok(WitnessLayer::EmbeddingCache),
        0xFF => {
            let len = r.read_u8()? as usize;
            if len > 64 {
                return Err(ErasureError::Io("custom layer name exceeds 64 bytes".into()));
            }
            let bytes = r.read_bytes(len)?;
            let s = std::str::from_utf8(bytes)
                .map_err(|_| ErasureError::Io("invalid utf-8 in custom layer".into()))?;
            Ok(WitnessLayer::Custom(s.to_string()))
        }
        _ => Err(ErasureError::Io("unknown witness layer tag".into())),
    }
}

pub(crate) fn layer_sort_key(layer: &WitnessLayer) -> (u8, &str) {
    match layer {
        WitnessLayer::LsmMemtable => (1, ""),
        WitnessLayer::SsTableAllLevels => (2, ""),
        WitnessLayer::HnswIndex => (3, ""),
        WitnessLayer::WalAllSegments => (4, ""),
        WitnessLayer::CsrGraph => (5, ""),
        WitnessLayer::KvCacheSegments => (6, ""),
        WitnessLayer::EmbeddingCache => (7, ""),
        WitnessLayer::Custom(s) => (0xFF, s.as_str()),
    }
}

pub(crate) fn encode_reason(r: &ErasureReason) -> u8 {
    match r {
        ErasureReason::UserRequest => 1,
        ErasureReason::LegalRequest => 2,
        ErasureReason::RetentionExpiry => 3,
        ErasureReason::OperatorAction => 4,
        ErasureReason::TenantOffboarding => 5,
        _ => 1,
    }
}

pub(crate) fn decode_reason(tag: u8) -> ErasureReason {
    match tag {
        1 => ErasureReason::UserRequest,
        2 => ErasureReason::LegalRequest,
        3 => ErasureReason::RetentionExpiry,
        4 => ErasureReason::OperatorAction,
        5 => ErasureReason::TenantOffboarding,
        _ => ErasureReason::UserRequest,
    }
}

fn encode_checkpoint_policy(p: CheckpointTaintPolicy) -> u8 {
    match p {
        CheckpointTaintPolicy::Drop => 0,
        CheckpointTaintPolicy::Rewrite => 1,
        CheckpointTaintPolicy::LegalHold => 2,
    }
}

fn decode_checkpoint_policy(tag: u8) -> Result<CheckpointTaintPolicy, ErasureError> {
    match tag {
        0 => Ok(CheckpointTaintPolicy::Drop),
        1 => Ok(CheckpointTaintPolicy::Rewrite),
        2 => Ok(CheckpointTaintPolicy::LegalHold),
        _ => Err(ErasureError::Io("invalid checkpoint policy tag".into())),
    }
}

fn encode_reinsert_policy(p: ReinsertPolicy) -> u8 {
    match p {
        ReinsertPolicy::Deny => 0,
        ReinsertPolicy::AllowAfterSealed => 1,
    }
}

fn decode_reinsert_policy(tag: u8) -> Result<ReinsertPolicy, ErasureError> {
    match tag {
        0 => Ok(ReinsertPolicy::Deny),
        1 => Ok(ReinsertPolicy::AllowAfterSealed),
        _ => Err(ErasureError::Io("invalid reinsert policy tag".into())),
    }
}

fn encode_target_kind(k: TargetKind) -> u8 {
    match k {
        TargetKind::Document => 1,
        TargetKind::Collection => 2,
        TargetKind::Tenant => 3,
        TargetKind::Subject => 4,
        TargetKind::DocumentReplace => 5,
        TargetKind::Superseded => 6,
        _ => 4,
    }
}

fn decode_target_kind(tag: u8) -> Result<TargetKind, ErasureError> {
    match tag {
        1 => Ok(TargetKind::Document),
        2 => Ok(TargetKind::Collection),
        3 => Ok(TargetKind::Tenant),
        4 => Ok(TargetKind::Subject),
        5 => Ok(TargetKind::DocumentReplace),
        6 => Ok(TargetKind::Superseded),
        _ => Err(ErasureError::Io("invalid target kind tag".into())),
    }
}

pub(crate) fn encode_error_kind(k: LayerErrorKind) -> u8 {
    match k {
        LayerErrorKind::Retryable => 1,
        LayerErrorKind::Permanent => 2,
        LayerErrorKind::Poisoned => 3,
        LayerErrorKind::ScopeViolation => 4,
        LayerErrorKind::Cancelled => 5,
    }
}

pub(crate) fn decode_error_kind(tag: u8) -> Result<LayerErrorKind, ErasureError> {
    match tag {
        1 => Ok(LayerErrorKind::Retryable),
        2 => Ok(LayerErrorKind::Permanent),
        3 => Ok(LayerErrorKind::Poisoned),
        4 => Ok(LayerErrorKind::ScopeViolation),
        5 => Ok(LayerErrorKind::Cancelled),
        _ => Err(ErasureError::Io("invalid layer error kind tag".into())),
    }
}

pub(crate) fn encode_exceptions(excs: &[ErasureException], out: &mut Vec<u8>) -> Result<(), ErasureError> {
    let n = u16::try_from(excs.len())
        .map_err(|_| ErasureError::InvalidTarget("too many exceptions"))?;
    out.extend_from_slice(&n.to_le_bytes());
    for exc in excs {
        match exc {
            ErasureException::OutOfScopeExternalCopies => out.push(1),
            ErasureException::LegalHold { checkpoint } => {
                let b = checkpoint.as_bytes();
                let len = u8::try_from(b.len())
                    .map_err(|_| ErasureError::InvalidTarget("checkpoint name > 255 bytes"))?;
                out.push(2);
                out.push(len);
                out.extend_from_slice(b);
            }
            ErasureException::NotMeasured { layer } => {
                let b = layer.as_bytes();
                let len = u8::try_from(b.len())
                    .map_err(|_| ErasureError::InvalidTarget("layer name > 255 bytes"))?;
                out.push(3);
                out.push(len);
                out.extend_from_slice(b);
            }
            ErasureException::FeatureDisabled { layer } => {
                let b = layer.as_bytes();
                let len = u8::try_from(b.len())
                    .map_err(|_| ErasureError::InvalidTarget("layer name > 255 bytes"))?;
                out.push(4);
                out.push(len);
                out.extend_from_slice(b);
            }
            ErasureException::Unprovenanced { objects } => {
                out.push(5);
                out.extend_from_slice(&objects.to_le_bytes());
            }
            ErasureException::OverlappingSnapshotPinned { min_seq } => {
                out.push(6);
                out.extend_from_slice(&min_seq.to_le_bytes());
            }
            _ => out.push(1),
        }
    }
    Ok(())
}

pub(crate) fn decode_exceptions(r: &mut ByteReader<'_>) -> Result<Vec<ErasureException>, ErasureError> {
    let n = r.read_u16()? as usize;
    let mut out = Vec::with_capacity(n.min(256));
    for _ in 0..n {
        let tag = r.read_u8()?;
        match tag {
            1 => out.push(ErasureException::OutOfScopeExternalCopies),
            2 => {
                let len = r.read_u8()? as usize;
                let bytes = r.read_bytes(len)?;
                let s = std::str::from_utf8(bytes)
                    .map_err(|_| ErasureError::Io("invalid utf-8 in LegalHold".into()))?;
                out.push(ErasureException::LegalHold {
                    checkpoint: s.to_string(),
                });
            }
            3 => {
                let len = r.read_u8()? as usize;
                let bytes = r.read_bytes(len)?;
                let s = std::str::from_utf8(bytes)
                    .map_err(|_| ErasureError::Io("invalid utf-8 in NotMeasured".into()))?;
                out.push(ErasureException::NotMeasured {
                    layer: s.to_string(),
                });
            }
            4 => {
                let len = r.read_u8()? as usize;
                let bytes = r.read_bytes(len)?;
                let s = std::str::from_utf8(bytes)
                    .map_err(|_| ErasureError::Io("invalid utf-8 in FeatureDisabled".into()))?;
                out.push(ErasureException::FeatureDisabled {
                    layer: s.to_string(),
                });
            }
            5 => {
                let objects = r.read_u64()?;
                out.push(ErasureException::Unprovenanced { objects });
            }
            6 => {
                let min_seq = r.read_u64()?;
                out.push(ErasureException::OverlappingSnapshotPinned { min_seq });
            }
            _ => return Err(ErasureError::Io("unknown exception tag".into())),
        }
    }
    Ok(out)
}

pub(crate) fn encode_resolved_scope(scope: &ResolvedScope) -> Result<Vec<u8>, ErasureError> {
    let mut out = Vec::with_capacity(128);
    out.extend_from_slice(&scope.id.0);
    out.push(encode_target_kind(scope.target_kind));
    out.extend_from_slice(&scope.plan_hash);

    let rules = scope.keys.rules();
    let n_rules = u32::try_from(rules.len())
        .map_err(|_| ErasureError::InvalidTarget("too many scope rules"))?;
    out.extend_from_slice(&n_rules.to_le_bytes());
    for (pat, verdict) in rules {
        let v_tag = match verdict {
            Verdict::Out => 0u8,
            Verdict::In => 1u8,
        };
        let k_tag = match pat {
            KeyPattern::Prefix(_) => 1u8,
            KeyPattern::Exact(_) => 2u8,
        };
        out.push(v_tag);
        out.push(k_tag);
        let b = pat.bytes();
        let len = u32::try_from(b.len())
            .map_err(|_| ErasureError::InvalidTarget("pattern too large"))?;
        out.extend_from_slice(&len.to_le_bytes());
        out.extend_from_slice(b);
    }

    let n_docs = u32::try_from(scope.docs.len())
        .map_err(|_| ErasureError::InvalidTarget("too many doc scopes"))?;
    out.extend_from_slice(&n_docs.to_le_bytes());
    for ds in &scope.docs {
        match ds {
            DocScope::Set {
                tenant,
                collection,
                docs,
            } => {
                out.push(1);
                out.extend_from_slice(&tenant.as_u64().to_le_bytes());
                let cb = collection.as_bytes();
                let clen = u32::try_from(cb.len())
                    .map_err(|_| ErasureError::InvalidTarget("collection name too long"))?;
                out.extend_from_slice(&clen.to_le_bytes());
                out.extend_from_slice(cb);
                let nd = u32::try_from(docs.len())
                    .map_err(|_| ErasureError::InvalidTarget("too many doc ids"))?;
                out.extend_from_slice(&nd.to_le_bytes());
                for doc_id in docs {
                    out.extend_from_slice(&doc_id.0.to_le_bytes());
                }
            }
            DocScope::Collection { tenant, collection } => {
                out.push(2);
                out.extend_from_slice(&tenant.as_u64().to_le_bytes());
                let cb = collection.as_bytes();
                let clen = u32::try_from(cb.len())
                    .map_err(|_| ErasureError::InvalidTarget("collection name too long"))?;
                out.extend_from_slice(&clen.to_le_bytes());
                out.extend_from_slice(cb);
            }
            DocScope::Tenant { tenant } => {
                out.push(3);
                out.extend_from_slice(&tenant.as_u64().to_le_bytes());
            }
        }
    }

    out.extend_from_slice(&scope.unprovenanced.to_le_bytes());
    Ok(out)
}

pub(crate) fn decode_resolved_scope(blob: &[u8]) -> Result<ResolvedScope, ErasureError> {
    let mut r = ByteReader::new(blob);
    let id = ErasureId(r.read_arr32_or_16()?);
    let target_kind = decode_target_kind(r.read_u8()?)?;
    let plan_hash = r.read_arr32()?;

    let n_rules = r.read_u32()? as usize;
    if n_rules > KeyScope::MAX_RULES {
        return Err(ErasureError::Io("rule count exceeds KeyScope::MAX_RULES".into()));
    }
    let mut rules = Vec::with_capacity(n_rules);
    for _ in 0..n_rules {
        let v_tag = r.read_u8()?;
        let k_tag = r.read_u8()?;
        let len = r.read_u32()? as usize;
        let bytes = r.read_bytes(len)?.to_vec();
        let verdict = match v_tag {
            0 => Verdict::Out,
            1 => Verdict::In,
            _ => return Err(ErasureError::Io("invalid verdict tag".into())),
        };
        let pat = match k_tag {
            1 => KeyPattern::Prefix(bytes),
            2 => KeyPattern::Exact(bytes),
            _ => return Err(ErasureError::Io("invalid pattern kind tag".into())),
        };
        rules.push((pat, verdict));
    }
    let keys = KeyScope::new(rules)?;

    let n_docs = r.read_u32()? as usize;
    let mut docs = Vec::with_capacity(n_docs.min(1024));
    for _ in 0..n_docs {
        let tag = r.read_u8()?;
        match tag {
            1 => {
                let tenant_raw = r.read_u64()?;
                let tenant = TenantId::try_new(tenant_raw)
                    .map_err(|_| ErasureError::Io("invalid tenant id".into()))?;
                let clen = r.read_u32()? as usize;
                let cbytes = r.read_bytes(clen)?;
                let collection = std::str::from_utf8(cbytes)
                    .map_err(|_| ErasureError::Io("invalid collection utf-8".into()))?
                    .to_string();
                let nd = r.read_u32()? as usize;
                let mut set = BTreeSet::new();
                for _ in 0..nd {
                    #[cfg(feature = "docid-128")]
                    let doc = DocId(r.read_u128()?);
                    #[cfg(not(feature = "docid-128"))]
                    let doc = DocId(r.read_u64()?);
                    set.insert(doc);
                }
                docs.push(DocScope::Set {
                    tenant,
                    collection,
                    docs: set,
                });
            }
            2 => {
                let tenant_raw = r.read_u64()?;
                let tenant = TenantId::try_new(tenant_raw)
                    .map_err(|_| ErasureError::Io("invalid tenant id".into()))?;
                let clen = r.read_u32()? as usize;
                let cbytes = r.read_bytes(clen)?;
                let collection = std::str::from_utf8(cbytes)
                    .map_err(|_| ErasureError::Io("invalid collection utf-8".into()))?
                    .to_string();
                docs.push(DocScope::Collection { tenant, collection });
            }
            3 => {
                let tenant_raw = r.read_u64()?;
                let tenant = TenantId::try_new(tenant_raw)
                    .map_err(|_| ErasureError::Io("invalid tenant id".into()))?;
                docs.push(DocScope::Tenant { tenant });
            }
            _ => return Err(ErasureError::Io("invalid DocScope tag".into())),
        }
    }

    let unprovenanced = r.read_u64()?;
    if !r.is_empty() {
        return Err(ErasureError::Io("trailing bytes in plan_blob".into()));
    }

    Ok(ResolvedScope {
        id,
        target_kind,
        keys,
        docs,
        plan_hash,
        unprovenanced,
    })
}

/// Alle 16 Journal-Record-Typen (§5.2).
#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) enum JournalRecord {
    Requested {
        id: ErasureId,
        target_hash: [u8; 32],
        reason: ErasureReason,
        policy: ErasurePolicy,
        ts: u64,
    },
    Planned {
        id: ErasureId,
        plan_hash: [u8; 32],
        expected_layers: Vec<WitnessLayer>,
        plan_blob: Vec<u8>,
        unprovenanced: u64,
    },
    GateIntent {
        id: ErasureId,
        ts: u64,
    },
    Gated {
        id: ErasureId,
        gated_seq: u64,
        epoch: u64,
        ts: u64,
    },
    KeysDestroying {
        id: ErasureId,
        vault_generation: u64,
    },
    PurgeStart {
        id: ErasureId,
        ts: u64,
    },
    LayerProgress {
        id: ErasureId,
        layer: WitnessLayer,
        cursor: Vec<u8>,
        units_done: u64,
        bytes_freed: u64,
        attempt: u32,
    },
    StoreProgress {
        id: ErasureId,
        phase: u8,
        cursor: Vec<u8>,
        units_done: u64,
        bytes_freed: u64,
        attempt: u32,
    },
    CollapseKeys {
        id: ErasureId,
        entries: Vec<(Vec<u8>, u64)>,
    },
    TempFile {
        id: ErasureId,
        path: String,
    },
    Verified {
        id: ErasureId,
        witness_root: [u8; 32],
        attempt: u32,
    },
    Sealed {
        id: ErasureId,
        ledger_index: i64,
        guarantee: u8,
        exceptions: Vec<ErasureException>,
    },
    Completed {
        id: ErasureId,
        ts: u64,
    },
    Failed {
        id: ErasureId,
        layer: Option<WitnessLayer>,
        kind: LayerErrorKind,
        attempt: u32,
    },
    Quarantined {
        id: ErasureId,
        layer: WitnessLayer,
    },
    Summary {
        id_hmac: [u8; 32],
        plan_hmac: [u8; 32],
        guarantee: u8,
        completed_at: u64,
    },
}

impl JournalRecord {
    pub(crate) fn encode_frame(&self, max_plan_bytes: u32) -> Result<Vec<u8>, ErasureError> {
        let mut body = Vec::with_capacity(96);
        match self {
            Self::Requested {
                id,
                target_hash,
                reason,
                policy,
                ts,
            } => {
                body.push(1);
                body.extend_from_slice(&id.0);
                body.extend_from_slice(target_hash);
                body.push(encode_reason(reason));
                body.push(encode_checkpoint_policy(policy.checkpoint));
                body.push(encode_reinsert_policy(policy.reinsert));
                body.extend_from_slice(&policy.sla_secs.to_le_bytes());
                body.push(u8::from(policy.allow_g2_unavailable));
                body.extend_from_slice(&ts.to_le_bytes());
            }
            Self::Planned {
                id,
                plan_hash,
                expected_layers,
                plan_blob,
                unprovenanced,
            } => {
                body.push(2);
                body.extend_from_slice(&id.0);
                body.extend_from_slice(plan_hash);
                let n_layers = u8::try_from(expected_layers.len())
                    .map_err(|_| ErasureError::InvalidTarget("expected_layers > 255"))?;
                body.push(n_layers);
                for layer in expected_layers {
                    encode_layer(layer, &mut body)?;
                }
                let blob_len = u32::try_from(plan_blob.len())
                    .map_err(|_| ErasureError::InvalidTarget("plan_blob exceeds u32"))?;
                body.extend_from_slice(&blob_len.to_le_bytes());
                body.extend_from_slice(plan_blob);
                body.extend_from_slice(&unprovenanced.to_le_bytes());
            }
            Self::GateIntent { id, ts } => {
                body.push(3);
                body.extend_from_slice(&id.0);
                body.extend_from_slice(&ts.to_le_bytes());
            }
            Self::Gated {
                id,
                gated_seq,
                epoch,
                ts,
            } => {
                body.push(4);
                body.extend_from_slice(&id.0);
                body.extend_from_slice(&gated_seq.to_le_bytes());
                body.extend_from_slice(&epoch.to_le_bytes());
                body.extend_from_slice(&ts.to_le_bytes());
            }
            Self::KeysDestroying {
                id,
                vault_generation,
            } => {
                body.push(5);
                body.extend_from_slice(&id.0);
                body.extend_from_slice(&vault_generation.to_le_bytes());
            }
            Self::PurgeStart { id, ts } => {
                body.push(6);
                body.extend_from_slice(&id.0);
                body.extend_from_slice(&ts.to_le_bytes());
            }
            Self::LayerProgress {
                id,
                layer,
                cursor,
                units_done,
                bytes_freed,
                attempt,
            } => {
                if cursor.len() > 256 {
                    return Err(ErasureError::InvalidCursor);
                }
                body.push(7);
                body.extend_from_slice(&id.0);
                encode_layer(layer, &mut body)?;
                body.extend_from_slice(&(cursor.len() as u16).to_le_bytes());
                body.extend_from_slice(cursor);
                body.extend_from_slice(&units_done.to_le_bytes());
                body.extend_from_slice(&bytes_freed.to_le_bytes());
                body.extend_from_slice(&attempt.to_le_bytes());
            }
            Self::StoreProgress {
                id,
                phase,
                cursor,
                units_done,
                bytes_freed,
                attempt,
            } => {
                if cursor.len() > 256 {
                    return Err(ErasureError::InvalidCursor);
                }
                body.push(8);
                body.extend_from_slice(&id.0);
                body.push(*phase);
                body.extend_from_slice(&(cursor.len() as u16).to_le_bytes());
                body.extend_from_slice(cursor);
                body.extend_from_slice(&units_done.to_le_bytes());
                body.extend_from_slice(&bytes_freed.to_le_bytes());
                body.extend_from_slice(&attempt.to_le_bytes());
            }
            Self::CollapseKeys { id, entries } => {
                if entries.len() > 1024 {
                    return Err(ErasureError::InvalidTarget("collapse batch > 1024"));
                }
                body.push(9);
                body.extend_from_slice(&id.0);
                body.extend_from_slice(&(entries.len() as u16).to_le_bytes());
                for (k, keep_from) in entries {
                    let klen = u16::try_from(k.len())
                        .map_err(|_| ErasureError::InvalidTarget("collapse key > 64 KiB"))?;
                    body.extend_from_slice(&klen.to_le_bytes());
                    body.extend_from_slice(k);
                    body.extend_from_slice(&keep_from.to_le_bytes());
                }
            }
            Self::TempFile { id, path } => {
                let pb = path.as_bytes();
                let plen = u16::try_from(pb.len())
                    .map_err(|_| ErasureError::InvalidTarget("temp path > 64 KiB"))?;
                body.push(10);
                body.extend_from_slice(&id.0);
                body.extend_from_slice(&plen.to_le_bytes());
                body.extend_from_slice(pb);
            }
            Self::Verified {
                id,
                witness_root,
                attempt,
            } => {
                body.push(11);
                body.extend_from_slice(&id.0);
                body.extend_from_slice(witness_root);
                body.extend_from_slice(&attempt.to_le_bytes());
            }
            Self::Sealed {
                id,
                ledger_index,
                guarantee,
                exceptions,
            } => {
                body.push(12);
                body.extend_from_slice(&id.0);
                body.extend_from_slice(&ledger_index.to_le_bytes());
                body.push(*guarantee);
                encode_exceptions(exceptions, &mut body)?;
            }
            Self::Completed { id, ts } => {
                body.push(13);
                body.extend_from_slice(&id.0);
                body.extend_from_slice(&ts.to_le_bytes());
            }
            Self::Failed {
                id,
                layer,
                kind,
                attempt,
            } => {
                body.push(14);
                body.extend_from_slice(&id.0);
                match layer {
                    Some(l) => {
                        body.push(1);
                        encode_layer(l, &mut body)?;
                    }
                    None => body.push(0),
                }
                body.push(encode_error_kind(*kind));
                body.extend_from_slice(&attempt.to_le_bytes());
            }
            Self::Quarantined { id, layer } => {
                body.push(15);
                body.extend_from_slice(&id.0);
                encode_layer(layer, &mut body)?;
            }
            Self::Summary {
                id_hmac,
                plan_hmac,
                guarantee,
                completed_at,
            } => {
                body.push(16);
                body.extend_from_slice(id_hmac);
                body.extend_from_slice(plan_hmac);
                body.push(*guarantee);
                body.extend_from_slice(&completed_at.to_le_bytes());
            }
        }

        let body_len = u32::try_from(body.len())
            .map_err(|_| ErasureError::InvalidTarget("journal frame exceeds u32"))?;
        if body_len > max_plan_bytes {
            return Err(ErasureError::InvalidTarget("journal frame exceeds max_plan_bytes"));
        }
        let crc = crc32fast::hash(&body);
        let mut frame = Vec::with_capacity(FRAME_HEADER_LEN + body.len());
        frame.extend_from_slice(&body_len.to_le_bytes());
        frame.extend_from_slice(&crc.to_le_bytes());
        frame.extend_from_slice(&body);
        Ok(frame)
    }

    pub(crate) fn decode_body(body: &[u8]) -> Result<Self, ErasureError> {
        let mut r = ByteReader::new(body);
        let rec_type = r.read_u8()?;
        let rec = match rec_type {
            1 => {
                let id = ErasureId(r.read_arr32_or_16()?);
                let target_hash = r.read_arr32()?;
                let reason = decode_reason(r.read_u8()?);
                let checkpoint = decode_checkpoint_policy(r.read_u8()?)?;
                let reinsert = decode_reinsert_policy(r.read_u8()?)?;
                let sla_secs = r.read_u64()?;
                let allow_g2_unavailable = r.read_u8()? != 0;
                let ts = r.read_u64()?;
                Self::Requested {
                    id,
                    target_hash,
                    reason,
                    policy: ErasurePolicy {
                        checkpoint,
                        reinsert,
                        sla_secs,
                        allow_g2_unavailable,
                    },
                    ts,
                }
            }
            2 => {
                let id = ErasureId(r.read_arr32_or_16()?);
                let plan_hash = r.read_arr32()?;
                let n_layers = r.read_u8()? as usize;
                let mut expected_layers = Vec::with_capacity(n_layers);
                for _ in 0..n_layers {
                    expected_layers.push(decode_layer(&mut r)?);
                }
                let blob_len = r.read_u32()? as usize;
                let plan_blob = r.read_bytes(blob_len)?.to_vec();
                let unprovenanced = r.read_u64()?;
                Self::Planned {
                    id,
                    plan_hash,
                    expected_layers,
                    plan_blob,
                    unprovenanced,
                }
            }
            3 => Self::GateIntent {
                id: ErasureId(r.read_arr32_or_16()?),
                ts: r.read_u64()?,
            },
            4 => Self::Gated {
                id: ErasureId(r.read_arr32_or_16()?),
                gated_seq: r.read_u64()?,
                epoch: r.read_u64()?,
                ts: r.read_u64()?,
            },
            5 => Self::KeysDestroying {
                id: ErasureId(r.read_arr32_or_16()?),
                vault_generation: r.read_u64()?,
            },
            6 => Self::PurgeStart {
                id: ErasureId(r.read_arr32_or_16()?),
                ts: r.read_u64()?,
            },
            7 => {
                let id = ErasureId(r.read_arr32_or_16()?);
                let layer = decode_layer(&mut r)?;
                let clen = r.read_u16()? as usize;
                if clen > 256 {
                    return Err(ErasureError::Io("cursor len > 256".into()));
                }
                let cursor = r.read_bytes(clen)?.to_vec();
                let units_done = r.read_u64()?;
                let bytes_freed = r.read_u64()?;
                let attempt = r.read_u32()?;
                Self::LayerProgress {
                    id,
                    layer,
                    cursor,
                    units_done,
                    bytes_freed,
                    attempt,
                }
            }
            8 => {
                let id = ErasureId(r.read_arr32_or_16()?);
                let phase = r.read_u8()?;
                let clen = r.read_u16()? as usize;
                if clen > 256 {
                    return Err(ErasureError::Io("cursor len > 256".into()));
                }
                let cursor = r.read_bytes(clen)?.to_vec();
                let units_done = r.read_u64()?;
                let bytes_freed = r.read_u64()?;
                let attempt = r.read_u32()?;
                Self::StoreProgress {
                    id,
                    phase,
                    cursor,
                    units_done,
                    bytes_freed,
                    attempt,
                }
            }
            9 => {
                let id = ErasureId(r.read_arr32_or_16()?);
                let n = r.read_u16()? as usize;
                if n > 1024 {
                    return Err(ErasureError::Io("collapse batch > 1024".into()));
                }
                let mut entries = Vec::with_capacity(n);
                for _ in 0..n {
                    let klen = r.read_u16()? as usize;
                    let key = r.read_bytes(klen)?.to_vec();
                    let keep_from = r.read_u64()?;
                    entries.push((key, keep_from));
                }
                Self::CollapseKeys { id, entries }
            }
            10 => {
                let id = ErasureId(r.read_arr32_or_16()?);
                let plen = r.read_u16()? as usize;
                let pbytes = r.read_bytes(plen)?;
                let path = std::str::from_utf8(pbytes)
                    .map_err(|_| ErasureError::Io("invalid utf-8 in TempFile".into()))?
                    .to_string();
                Self::TempFile { id, path }
            }
            11 => Self::Verified {
                id: ErasureId(r.read_arr32_or_16()?),
                witness_root: r.read_arr32()?,
                attempt: r.read_u32()?,
            },
            12 => Self::Sealed {
                id: ErasureId(r.read_arr32_or_16()?),
                ledger_index: r.read_i64()?,
                guarantee: r.read_u8()?,
                exceptions: decode_exceptions(&mut r)?,
            },
            13 => Self::Completed {
                id: ErasureId(r.read_arr32_or_16()?),
                ts: r.read_u64()?,
            },
            14 => {
                let id = ErasureId(r.read_arr32_or_16()?);
                let layer_present = r.read_u8()?;
                let layer = if layer_present != 0 {
                    Some(decode_layer(&mut r)?)
                } else {
                    None
                };
                let kind = decode_error_kind(r.read_u8()?)?;
                let attempt = r.read_u32()?;
                Self::Failed {
                    id,
                    layer,
                    kind,
                    attempt,
                }
            }
            15 => Self::Quarantined {
                id: ErasureId(r.read_arr32_or_16()?),
                layer: decode_layer(&mut r)?,
            },
            16 => Self::Summary {
                id_hmac: r.read_arr32()?,
                plan_hmac: r.read_arr32()?,
                guarantee: r.read_u8()?,
                completed_at: r.read_u64()?,
            },
            _ => return Err(ErasureError::Io("unknown journal record type".into())),
        };
        if !r.is_empty() {
            return Err(ErasureError::Io("trailing bytes in journal frame".into()));
        }
        Ok(rec)
    }
}

pub(crate) struct ByteReader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> ByteReader<'a> {
    pub(crate) fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.pos >= self.buf.len()
    }

    pub(crate) fn read_bytes(&mut self, len: usize) -> Result<&'a [u8], ErasureError> {
        let end = self
            .pos
            .checked_add(len)
            .ok_or_else(|| ErasureError::Io("offset overflow".into()))?;
        let slice = self
            .buf
            .get(self.pos..end)
            .ok_or_else(|| ErasureError::Io("unexpected EOF in binary layout".into()))?;
        self.pos = end;
        Ok(slice)
    }

    pub(crate) fn read_u8(&mut self) -> Result<u8, ErasureError> {
        let b = self.read_bytes(1)?;
        b.first()
            .copied()
            .ok_or_else(|| ErasureError::Io("unexpected EOF".into()))
    }

    pub(crate) fn read_u16(&mut self) -> Result<u16, ErasureError> {
        let b = self.read_bytes(2)?;
        let arr: [u8; 2] = b
            .try_into()
            .map_err(|_| ErasureError::Io("unexpected EOF".into()))?;
        Ok(u16::from_le_bytes(arr))
    }

    pub(crate) fn read_u32(&mut self) -> Result<u32, ErasureError> {
        let b = self.read_bytes(4)?;
        let arr: [u8; 4] = b
            .try_into()
            .map_err(|_| ErasureError::Io("unexpected EOF".into()))?;
        Ok(u32::from_le_bytes(arr))
    }

    pub(crate) fn read_u64(&mut self) -> Result<u64, ErasureError> {
        let b = self.read_bytes(8)?;
        let arr: [u8; 8] = b
            .try_into()
            .map_err(|_| ErasureError::Io("unexpected EOF".into()))?;
        Ok(u64::from_le_bytes(arr))
    }

    pub(crate) fn read_i64(&mut self) -> Result<i64, ErasureError> {
        let b = self.read_bytes(8)?;
        let arr: [u8; 8] = b
            .try_into()
            .map_err(|_| ErasureError::Io("unexpected EOF".into()))?;
        Ok(i64::from_le_bytes(arr))
    }

    #[cfg(feature = "docid-128")]
    pub(crate) fn read_u128(&mut self) -> Result<u128, ErasureError> {
        let b = self.read_bytes(16)?;
        let arr: [u8; 16] = b
            .try_into()
            .map_err(|_| ErasureError::Io("unexpected EOF".into()))?;
        Ok(u128::from_le_bytes(arr))
    }

    pub(crate) fn read_arr32_or_16(&mut self) -> Result<[u8; 16], ErasureError> {
        let b = self.read_bytes(16)?;
        b.try_into()
            .map_err(|_| ErasureError::Io("unexpected EOF".into()))
    }

    pub(crate) fn read_arr32(&mut self) -> Result<[u8; 32], ErasureError> {
        let b = self.read_bytes(32)?;
        b.try_into()
            .map_err(|_| ErasureError::Io("unexpected EOF".into()))
    }
}

/// Ergebnis beim Öffnen des Journals (§5.4).
#[derive(Debug)]
pub(crate) struct JournalOpenResult {
    pub journal: Journal,
    pub records: Vec<JournalRecord>,
    pub torn_tail_truncated: bool,
    pub corrupt: bool,
}

/// Durables Append-Only-Journal mit Pre-Allokation und Vergiftungsschutz (§5.3).
pub(crate) struct Journal {
    io: StoreIo,
    dir: PathBuf,
    path: PathBuf,
    file: Option<Arc<Mutex<Box<dyn VfsFile>>>>,
    offset: u64,
    reserved_len: u64,
    reserve_bytes: u64,
    max_plan_bytes: u32,
    poisoned: Arc<AtomicBool>,
    completed_since_compact: u64,
}

impl std::fmt::Debug for Journal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Journal")
            .field("dir", &self.dir)
            .field("path", &self.path)
            .field("offset", &self.offset)
            .field("reserved_len", &self.reserved_len)
            .field("poisoned", &self.poisoned.load(Ordering::Relaxed))
            .finish_non_exhaustive()
    }
}

impl Journal {
    /// Öffnet oder erstellt `<dir>/journal.log` und liest alle gültigen Frames (§5.4).
    pub async fn open(
        io: StoreIo,
        dir: PathBuf,
        reserve_bytes: u64,
        max_plan_bytes: u32,
    ) -> Result<JournalOpenResult, ErasureError> {
        io.create_dir_all(&dir)
            .await
            .map_err(|e| ErasureError::Io(e.to_string()))?;

        let path = dir.join(JOURNAL_FILE_NAME);
        let exists = io
            .exists(&path)
            .await
            .map_err(|e| ErasureError::Io(e.to_string()))?;

        if !exists {
            let io_c = io.clone();
            let dir_c = dir.clone();
            let path_c = path.clone();
            let target_len = reserve_bytes.max(JOURNAL_HEADER_LEN as u64);
            let file_arc = tokio::task::spawn_blocking(move || -> Result<Arc<Mutex<Box<dyn VfsFile>>>, ErasureError> {
                let opts = VfsOpenOptions {
                    read: true,
                    write: true,
                    create: true,
                    ..Default::default()
                };
                let f = io_c
                    .open(&path_c, &opts)
                    .map_err(|e| ErasureError::Io(e.to_string()))?;
                let mut hdr = [0u8; JOURNAL_HEADER_LEN];
                hdr[..8].copy_from_slice(JOURNAL_MAGIC);
                hdr[8..10].copy_from_slice(&JOURNAL_VERSION.to_le_bytes());
                f.write_at(&hdr, 0)
                    .map_err(|e| ErasureError::Io(e.to_string()))?;
                f.set_len(target_len)
                    .map_err(|e| ErasureError::Io(e.to_string()))?;
                f.sync_all()
                    .map_err(|e| ErasureError::Io(e.to_string()))?;
                io_c.vfs()
                    .sync_dir(&dir_c)
                    .map_err(|e| ErasureError::Io(e.to_string()))?;
                if let Some(parent) = dir_c.parent() {
                    if !parent.as_os_str().is_empty() {
                        io_c.vfs()
                            .sync_dir(parent)
                            .map_err(|e| ErasureError::Io(e.to_string()))?;
                    }
                }
                Ok(Arc::new(Mutex::new(f)))
            })
            .await
            .map_err(|e| ErasureError::Io(e.to_string()))??;

            return Ok(JournalOpenResult {
                journal: Self {
                    io,
                    dir,
                    path,
                    file: Some(file_arc),
                    offset: JOURNAL_HEADER_LEN as u64,
                    reserved_len: target_len,
                    reserve_bytes,
                    max_plan_bytes,
                    poisoned: Arc::new(AtomicBool::new(false)),
                    completed_since_compact: 0,
                },
                records: Vec::new(),
                torn_tail_truncated: false,
                corrupt: false,
            });
        }

        // Bestehende Datei lesen und prüfen.
        let bytes = io
            .read(&path)
            .await
            .map_err(|e| ErasureError::Io(e.to_string()))?;

        if bytes.is_empty() {
            // Leere Datei (0 Bytes) -> wie neue Datei initialisieren.
            let io_c = io.clone();
            let dir_c = dir.clone();
            let path_c = path.clone();
            let target_len = reserve_bytes.max(JOURNAL_HEADER_LEN as u64);
            let file_arc = tokio::task::spawn_blocking(move || -> Result<Arc<Mutex<Box<dyn VfsFile>>>, ErasureError> {
                let opts = VfsOpenOptions {
                    read: true,
                    write: true,
                    create: true,
                    ..Default::default()
                };
                let f = io_c
                    .open(&path_c, &opts)
                    .map_err(|e| ErasureError::Io(e.to_string()))?;
                let mut hdr = [0u8; JOURNAL_HEADER_LEN];
                hdr[..8].copy_from_slice(JOURNAL_MAGIC);
                hdr[8..10].copy_from_slice(&JOURNAL_VERSION.to_le_bytes());
                f.write_at(&hdr, 0)
                    .map_err(|e| ErasureError::Io(e.to_string()))?;
                f.set_len(target_len)
                    .map_err(|e| ErasureError::Io(e.to_string()))?;
                f.sync_all()
                    .map_err(|e| ErasureError::Io(e.to_string()))?;
                io_c.vfs()
                    .sync_dir(&dir_c)
                    .map_err(|e| ErasureError::Io(e.to_string()))?;
                Ok(Arc::new(Mutex::new(f)))
            })
            .await
            .map_err(|e| ErasureError::Io(e.to_string()))??;

            return Ok(JournalOpenResult {
                journal: Self {
                    io,
                    dir,
                    path,
                    file: Some(file_arc),
                    offset: JOURNAL_HEADER_LEN as u64,
                    reserved_len: target_len,
                    reserve_bytes,
                    max_plan_bytes,
                    poisoned: Arc::new(AtomicBool::new(false)),
                    completed_since_compact: 0,
                },
                records: Vec::new(),
                torn_tail_truncated: false,
                corrupt: false,
            });
        }

        if bytes.len() < JOURNAL_HEADER_LEN {
            return Ok(JournalOpenResult {
                journal: Self::corrupt_stub(io, dir, path, reserve_bytes, max_plan_bytes),
                records: Vec::new(),
                torn_tail_truncated: false,
                corrupt: true,
            });
        }

        let magic_ok = bytes.get(..8) == Some(JOURNAL_MAGIC.as_slice());
        let ver_ok = bytes
            .get(8..10)
            .and_then(|s| <[u8; 2]>::try_from(s).ok())
            .map(u16::from_le_bytes)
            == Some(JOURNAL_VERSION);
        if !magic_ok || !ver_ok {
            return Ok(JournalOpenResult {
                journal: Self::corrupt_stub(io, dir, path, reserve_bytes, max_plan_bytes),
                records: Vec::new(),
                torn_tail_truncated: false,
                corrupt: true,
            });
        }

        let mut pos = JOURNAL_HEADER_LEN;
        let mut records = Vec::new();
        let mut torn_tail_truncated = false;
        let mut completed_count = 0u64;

        while pos < bytes.len() {
            // Prüfen, ob wir in den vorallokierten Nullbereich am Ende eingetreten sind:
            // Ein gültiges Frame hat len >= 1. `len == 0` bedeutet Ende des geschriebenen Bereichs,
            // sofern der Rest der Datei genullt ist.
            let Some(len_slice) = bytes.get(pos..pos + 4) else {
                // Unvollständiger Frame-Header am Dateiende -> Torn Tail.
                torn_tail_truncated = true;
                break;
            };
            let len_arr: [u8; 4] = match len_slice.try_into() {
                Ok(a) => a,
                Err(_) => {
                    torn_tail_truncated = true;
                    break;
                }
            };
            let body_len = u32::from_le_bytes(len_arr);
            if body_len == 0 {
                // Prüfen, ob der gesamte Rest der Datei 0x00 ist (Pre-Allokation).
                let rest = bytes.get(pos..).unwrap_or(&[]);
                if rest.iter().all(|b| *b == 0) {
                    break;
                }
                // Nicht-Null-Bytes nach len==0 -> Korruption in der Mitte!
                return Ok(JournalOpenResult {
                    journal: Self::corrupt_stub(io, dir, path, reserve_bytes, max_plan_bytes),
                    records: Vec::new(),
                    torn_tail_truncated: false,
                    corrupt: true,
                });
            }

            if body_len > max_plan_bytes {
                // Prüfen, ob es ein Torn Tail am Ende ist oder mitten in der Datei.
                let expected_end = pos.saturating_add(FRAME_HEADER_LEN).saturating_add(body_len as usize);
                if expected_end >= bytes.len() {
                    torn_tail_truncated = true;
                    break;
                }
                return Ok(JournalOpenResult {
                    journal: Self::corrupt_stub(io, dir, path, reserve_bytes, max_plan_bytes),
                    records: Vec::new(),
                    torn_tail_truncated: false,
                    corrupt: true,
                });
            }

            let Some(crc_slice) = bytes.get(pos + 4..pos + 8) else {
                torn_tail_truncated = true;
                break;
            };
            let crc_arr: [u8; 4] = match crc_slice.try_into() {
                Ok(a) => a,
                Err(_) => {
                    torn_tail_truncated = true;
                    break;
                }
            };
            let expected_crc = u32::from_le_bytes(crc_arr);
            let frame_end = pos + FRAME_HEADER_LEN + (body_len as usize);

            let Some(body) = bytes.get(pos + FRAME_HEADER_LEN..frame_end) else {
                // Frame ragt über EOF hinaus -> Torn Tail.
                torn_tail_truncated = true;
                break;
            };

            let actual_crc = crc32fast::hash(body);
            if actual_crc != expected_crc {
                // Falsche CRC: Ist dies der letzte Frame (oder folgt nur vorallokiertes Nullpadding)?
                let tail = bytes.get(frame_end..).unwrap_or(&[]);
                if tail.is_empty() || tail.iter().all(|b| *b == 0) {
                    torn_tail_truncated = true;
                    break;
                }
                // Nach diesem fehlerhaften Frame folgen noch weitere Nicht-Null-Bytes -> Korruption!
                return Ok(JournalOpenResult {
                    journal: Self::corrupt_stub(io, dir, path, reserve_bytes, max_plan_bytes),
                    records: Vec::new(),
                    torn_tail_truncated: false,
                    corrupt: true,
                });
            }

            match JournalRecord::decode_body(body) {
                Ok(rec) => {
                    if matches!(rec, JournalRecord::Completed { .. } | JournalRecord::Summary { .. }) {
                        completed_count += 1;
                    }
                    records.push(rec);
                    pos = frame_end;
                }
                Err(_) => {
                    return Ok(JournalOpenResult {
                        journal: Self::corrupt_stub(io, dir, path, reserve_bytes, max_plan_bytes),
                        records: Vec::new(),
                        torn_tail_truncated: false,
                        corrupt: true,
                    });
                }
            }
        }

        let valid_end = pos as u64;
        let io_c = io.clone();
        let path_c = path.clone();
        let current_len = bytes.len() as u64;
        let (file_arc, reserved_len) = tokio::task::spawn_blocking(move || -> Result<(Arc<Mutex<Box<dyn VfsFile>>>, u64), ErasureError> {
            let opts = VfsOpenOptions {
                read: true,
                write: true,
                ..Default::default()
            };
            let f = io_c
                .open(&path_c, &opts)
                .map_err(|e| ErasureError::Io(e.to_string()))?;
            let final_len = if torn_tail_truncated {
                f.set_len(valid_end)
                    .map_err(|e| ErasureError::Io(e.to_string()))?;
                f.sync_all()
                    .map_err(|e| ErasureError::Io(e.to_string()))?;
                valid_end
            } else {
                current_len.max(valid_end)
            };
            Ok((Arc::new(Mutex::new(f)), final_len))
        })
        .await
        .map_err(|e| ErasureError::Io(e.to_string()))??;

        Ok(JournalOpenResult {
            journal: Self {
                io,
                dir,
                path,
                file: Some(file_arc),
                offset: valid_end,
                reserved_len,
                reserve_bytes,
                max_plan_bytes,
                poisoned: Arc::new(AtomicBool::new(false)),
                completed_since_compact: completed_count,
            },
            records,
            torn_tail_truncated,
            corrupt: false,
        })
    }

    fn corrupt_stub(
        io: StoreIo,
        dir: PathBuf,
        path: PathBuf,
        reserve_bytes: u64,
        max_plan_bytes: u32,
    ) -> Self {
        Self {
            io,
            dir,
            path,
            file: None,
            offset: 0,
            reserved_len: 0,
            reserve_bytes,
            max_plan_bytes,
            poisoned: Arc::new(AtomicBool::new(true)),
            completed_since_compact: 0,
        }
    }

    pub fn is_poisoned(&self) -> bool {
        self.poisoned.load(Ordering::Acquire)
    }

    pub fn offset(&self) -> u64 {
        self.offset
    }

    pub fn completed_since_compact(&self) -> u64 {
        self.completed_since_compact
    }

    /// Schreibt alle Records als einen zusammenhängenden Puffer + `sync_data` (Write-Ahead, I-2).
    ///
    /// Bei jedem I/O- oder `sync_data`-Fehler wird `poisoned = true` gesetzt.
    pub async fn append_batch(&mut self, records: &[JournalRecord]) -> Result<(), ErasureError> {
        if self.is_poisoned() {
            return Err(ErasureError::JournalPoisoned);
        }
        let Some(ref file_arc) = self.file else {
            self.poisoned.store(true, Ordering::Release);
            return Err(ErasureError::JournalPoisoned);
        };

        let mut buf = Vec::with_capacity(records.len() * 96);
        let mut completed_delta = 0u64;
        for rec in records {
            if matches!(rec, JournalRecord::Completed { .. }) {
                completed_delta += 1;
            }
            let frame = rec.encode_frame(self.max_plan_bytes)?;
            buf.extend_from_slice(&frame);
        }

        let write_offset = self.offset;
        let new_offset = write_offset
            .checked_add(buf.len() as u64)
            .ok_or(ErasureError::JournalPoisoned)?;
        let f_arc = Arc::clone(file_arc);
        let poisoned = Arc::clone(&self.poisoned);
        let need_extend = new_offset > self.reserved_len;
        let new_reserved = if need_extend {
            new_offset.saturating_add(self.reserve_bytes)
        } else {
            self.reserved_len
        };

        let res = tokio::task::spawn_blocking(move || -> Result<(), ErasureError> {
            let f = f_arc.lock();
            if let Err(_e) = f.write_at(&buf, write_offset) {
                poisoned.store(true, Ordering::Release);
                return Err(ErasureError::JournalPoisoned);
            }
            if need_extend {
                if let Err(_e) = f.set_len(new_reserved) {
                    // Best-effort: wenn set_len fehlschlägt, versuchen wir dennoch sync_data,
                    // sofern write_at bereits erfolgreich war. Bei Fehler vergiften.
                    poisoned.store(true, Ordering::Release);
                    return Err(ErasureError::JournalPoisoned);
                }
            }
            if let Err(_e) = f.sync_data() {
                poisoned.store(true, Ordering::Release);
                return Err(ErasureError::JournalPoisoned);
            }
            Ok(())
        })
        .await
        .map_err(|_| {
            self.poisoned.store(true, Ordering::Release);
            ErasureError::JournalPoisoned
        })?;

        res?;
        self.offset = new_offset;
        self.reserved_len = new_reserved;
        self.completed_since_compact = self.completed_since_compact.saturating_add(completed_delta);
        Ok(())
    }

    /// Prüft, ob vorab genug Platz (oder Reserve) für einen Eintrag vorhanden ist (§4.2).
    pub fn has_space_for(&self, estimated_bytes: u64) -> bool {
        if self.offset.saturating_add(estimated_bytes) <= self.reserved_len {
            return true;
        }
        match contextra_sys::available_space(&self.dir) {
            Ok(avail) => avail >= estimated_bytes,
            Err(_) => true,
        }
    }

    /// Führt eine atomare Journal-Verdichtung (Rollover) durch (§5.5).
    ///
    /// Schreibt `journal.new` mit allen Records der offenen Aufträge und nur `Summary`
    /// für abgeschlossene Aufträge, ruft `sync_all`, benennt atomar in `journal.log` um
    /// und synchronisiert das Verzeichnis.
    pub async fn compact(&mut self, records: &[JournalRecord]) -> Result<(), ErasureError> {
        if self.is_poisoned() {
            return Err(ErasureError::JournalPoisoned);
        }

        let mut buf = Vec::with_capacity(JOURNAL_HEADER_LEN + records.len() * 96);
        buf.extend_from_slice(JOURNAL_MAGIC);
        buf.extend_from_slice(&JOURNAL_VERSION.to_le_bytes());
        buf.extend_from_slice(&0u16.to_le_bytes());
        buf.extend_from_slice(&0u32.to_le_bytes());

        let mut summary_count = 0u64;
        for rec in records {
            if matches!(rec, JournalRecord::Summary { .. }) {
                summary_count += 1;
            }
            let frame = rec.encode_frame(self.max_plan_bytes)?;
            buf.extend_from_slice(&frame);
        }

        let valid_end = buf.len() as u64;
        let target_len = valid_end.max(self.reserve_bytes);
        let new_path = self.dir.join(JOURNAL_NEW_NAME);
        let log_path = self.path.clone();
        let dir_c = self.dir.clone();
        let io_c = self.io.clone();
        let poisoned = Arc::clone(&self.poisoned);

        let new_file_arc = tokio::task::spawn_blocking(move || -> Result<Arc<Mutex<Box<dyn VfsFile>>>, ErasureError> {
            let opts_w = VfsOpenOptions {
                read: true,
                write: true,
                create: true,
                truncate: true,
                ..Default::default()
            };
            let nf = io_c.open(&new_path, &opts_w).map_err(|_| {
                poisoned.store(true, Ordering::Release);
                ErasureError::JournalPoisoned
            })?;
            nf.write_at(&buf, 0).map_err(|_| {
                poisoned.store(true, Ordering::Release);
                ErasureError::JournalPoisoned
            })?;
            nf.set_len(target_len).map_err(|_| {
                poisoned.store(true, Ordering::Release);
                ErasureError::JournalPoisoned
            })?;
            nf.sync_all().map_err(|_| {
                poisoned.store(true, Ordering::Release);
                ErasureError::JournalPoisoned
            })?;
            drop(nf);

            io_c.vfs().rename(&new_path, &log_path).map_err(|_| {
                poisoned.store(true, Ordering::Release);
                ErasureError::JournalPoisoned
            })?;
            io_c.vfs().sync_dir(&dir_c).map_err(|_| {
                poisoned.store(true, Ordering::Release);
                ErasureError::JournalPoisoned
            })?;

            let opts_rw = VfsOpenOptions {
                read: true,
                write: true,
                ..Default::default()
            };
            let reopened = io_c.open(&log_path, &opts_rw).map_err(|_| {
                poisoned.store(true, Ordering::Release);
                ErasureError::JournalPoisoned
            })?;
            Ok(Arc::new(Mutex::new(reopened)))
        })
        .await
        .map_err(|_| {
            self.poisoned.store(true, Ordering::Release);
            ErasureError::JournalPoisoned
        })??;

        self.file = Some(new_file_arc);
        self.offset = valid_end;
        self.reserved_len = target_len;
        self.completed_since_compact = summary_count;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use contextra_ports::erasure::KeyPattern;

    fn sample_id(b: u8) -> ErasureId {
        ErasureId([b; 16])
    }

    fn all_sample_records() -> Vec<JournalRecord> {
        let id = sample_id(0x11);
        let scope = ResolvedScope {
            id,
            target_kind: TargetKind::Tenant,
            keys: KeyScope::new(vec![
                (KeyPattern::Prefix(b"t:7:".to_vec()), Verdict::In),
                (KeyPattern::Exact(b"t:7:keep".to_vec()), Verdict::Out),
            ])
            .unwrap(),
            docs: vec![DocScope::Tenant {
                tenant: TenantId::try_new(7).unwrap(),
            }],
            plan_hash: [0x22; 32],
            unprovenanced: 3,
        };
        let plan_blob = encode_resolved_scope(&scope).unwrap();
        vec![
            JournalRecord::Requested {
                id,
                target_hash: [0x33; 32],
                reason: ErasureReason::UserRequest,
                policy: ErasurePolicy {
                    checkpoint: CheckpointTaintPolicy::Rewrite,
                    reinsert: ReinsertPolicy::AllowAfterSealed,
                    sla_secs: 3600,
                    allow_g2_unavailable: true,
                },
                ts: 1_000_000,
            },
            JournalRecord::Planned {
                id,
                plan_hash: [0x22; 32],
                expected_layers: vec![
                    WitnessLayer::LsmMemtable,
                    WitnessLayer::SsTableAllLevels,
                    WitnessLayer::WalAllSegments,
                    WitnessLayer::Custom("test-layer".to_string()),
                ],
                plan_blob,
                unprovenanced: 3,
            },
            JournalRecord::GateIntent { id, ts: 1_000_001 },
            JournalRecord::Gated {
                id,
                gated_seq: 42,
                epoch: 7,
                ts: 1_000_002,
            },
            JournalRecord::KeysDestroying {
                id,
                vault_generation: 9,
            },
            JournalRecord::PurgeStart { id, ts: 1_000_003 },
            JournalRecord::LayerProgress {
                id,
                layer: WitnessLayer::HnswIndex,
                cursor: vec![1, 2, 3, 4],
                units_done: 100,
                bytes_freed: 4096,
                attempt: 1,
            },
            JournalRecord::StoreProgress {
                id,
                phase: 2,
                cursor: vec![5, 6],
                units_done: 12,
                bytes_freed: 8192,
                attempt: 1,
            },
            JournalRecord::CollapseKeys {
                id,
                entries: vec![(b"i:term1".to_vec(), 99)],
            },
            JournalRecord::TempFile {
                id,
                path: "sst-001.erase.tmp".to_string(),
            },
            JournalRecord::Verified {
                id,
                witness_root: [0x44; 32],
                attempt: 1,
            },
            JournalRecord::Sealed {
                id,
                ledger_index: 5,
                guarantee: 0b0000_0111,
                exceptions: vec![
                    ErasureException::OutOfScopeExternalCopies,
                    ErasureException::LegalHold {
                        checkpoint: "cp-1".to_string(),
                    },
                    ErasureException::Unprovenanced { objects: 3 },
                ],
            },
            JournalRecord::Completed { id, ts: 1_000_010 },
            JournalRecord::Failed {
                id,
                layer: Some(WitnessLayer::WalAllSegments),
                kind: LayerErrorKind::Retryable,
                attempt: 2,
            },
            JournalRecord::Quarantined {
                id,
                layer: WitnessLayer::CsrGraph,
            },
            JournalRecord::Summary {
                id_hmac: [0x55; 32],
                plan_hmac: [0x66; 32],
                guarantee: 0b11,
                completed_at: 1_000_010,
            },
        ]
    }

    /// TST-STORE-JRN-1: Round-Trip aller 16 Frame-Typen.
    #[tokio::test]
    async fn test_jrn_1_roundtrip_all_16_types() {
        let dir = tempfile::tempdir().unwrap();
        let io = StoreIo::default();
        let records = all_sample_records();

        {
            let opened = Journal::open(io.clone(), dir.path().to_path_buf(), 4096, 16 * 1024 * 1024)
                .await
                .unwrap();
            assert!(!opened.corrupt);
            let mut j = opened.journal;
            j.append_batch(&records).await.unwrap();
        }

        let reopened = Journal::open(io, dir.path().to_path_buf(), 4096, 16 * 1024 * 1024)
            .await
            .unwrap();
        assert!(!reopened.corrupt);
        assert!(!reopened.torn_tail_truncated);
        assert_eq!(reopened.records, records);

        if let JournalRecord::Planned { plan_blob, .. } = &reopened.records[1] {
            let decoded = decode_resolved_scope(plan_blob).unwrap();
            assert_eq!(decoded.plan_hash, [0x22; 32]);
            assert!(decoded.keys.contains(b"t:7:doc1"));
            assert!(!decoded.keys.contains(b"t:7:keep"));
        } else {
            panic!("expected Planned record");
        }
    }

    /// TST-STORE-JRN-2: Golden-Test für kanonische Frame-Bytes.
    #[test]
    fn test_jrn_2_golden_layout() {
        let rec = JournalRecord::GateIntent {
            id: ErasureId([0xAA; 16]),
            ts: 0x0102_0304_0506_0708,
        };
        let frame = rec.encode_frame(1024).unwrap();
        // body: [type=3] + [16 x 0xAA] + [ts LE] = 25 bytes
        let body_len = u32::from_le_bytes(frame[0..4].try_into().unwrap());
        assert_eq!(body_len, 25);
        assert_eq!(frame[8], 3);
        assert_eq!(&frame[9..25], &[0xAA; 16]);
        assert_eq!(&frame[25..33], &0x0102_0304_0506_0708u64.to_le_bytes());
        let expected_crc = crc32fast::hash(&frame[8..33]);
        assert_eq!(u32::from_le_bytes(frame[4..8].try_into().unwrap()), expected_crc);
    }

    /// TST-STORE-JRN-3: CP-13/CP-14 (Torn Tail vs. Korruption in der Mitte).
    #[tokio::test]
    async fn test_jrn_3_torn_tail_vs_mid_corruption() {
        let dir = tempfile::tempdir().unwrap();
        let io = StoreIo::default();
        let id = sample_id(0x01);
        let r1 = JournalRecord::GateIntent { id, ts: 10 };
        let r2 = JournalRecord::PurgeStart { id, ts: 20 };

        let end_after_r1;
        {
            let mut j = Journal::open(io.clone(), dir.path().to_path_buf(), 64, 1024 * 1024)
                .await
                .unwrap()
                .journal;
            j.append_batch(&[r1.clone()]).await.unwrap();
            end_after_r1 = j.offset();
            j.append_batch(&[r2.clone()]).await.unwrap();
        }

        // CP-13: Torn tail am Ende (CRC-Bitflip im letzten Frame) -> wird gekürzt, r1 bleibt erhalten.
        let log_path = dir.path().join(JOURNAL_FILE_NAME);
        let mut data = std::fs::read(&log_path).unwrap();
        let last_non_zero = data.iter().rposition(|b| *b != 0).unwrap();
        data[last_non_zero] ^= 0xFF;
        std::fs::write(&log_path, &data).unwrap();

        let res_torn = Journal::open(io.clone(), dir.path().to_path_buf(), 64, 1024 * 1024)
            .await
            .unwrap();
        assert!(!res_torn.corrupt);
        assert!(res_torn.torn_tail_truncated);
        assert_eq!(res_torn.records, vec![r1.clone()]);
        assert_eq!(res_torn.journal.offset(), end_after_r1);

        // CP-14: Zweites Frame anhängen und erstes Frame beschädigen -> JournalCorrupt!
        let mut j = res_torn.journal;
        j.append_batch(&[r2]).await.unwrap();
        drop(j);

        let mut data2 = std::fs::read(&log_path).unwrap();
        // Flip a byte inside the first frame body (offset 16 + 8 + 1 = 25)
        data2[25] ^= 0xFF;
        std::fs::write(&log_path, &data2).unwrap();

        let res_corrupt = Journal::open(io, dir.path().to_path_buf(), 64, 1024 * 1024)
            .await
            .unwrap();
        assert!(res_corrupt.corrupt);
        assert!(res_corrupt.journal.is_poisoned());
    }

    /// TST-STORE-JRN-5: Rollover/Verdichtung entfernt Zielbytes (Plan-Blob) nach Completed.
    #[tokio::test]
    async fn test_jrn_5_compaction_removes_target_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let io = StoreIo::default();
        let canary = b"CANARY_SECRET_USER_KEY_999";
        let id = sample_id(0x77);
        let scope = ResolvedScope {
            id,
            target_kind: TargetKind::Subject,
            keys: KeyScope::new(vec![(KeyPattern::Prefix(canary.to_vec()), Verdict::In)]).unwrap(),
            docs: vec![],
            plan_hash: [0x88; 32],
            unprovenanced: 0,
        };
        let plan_blob = encode_resolved_scope(&scope).unwrap();

        let mut j = Journal::open(io.clone(), dir.path().to_path_buf(), 64, 1024 * 1024)
            .await
            .unwrap()
            .journal;
        j.append_batch(&[
            JournalRecord::Requested {
                id,
                target_hash: [0x11; 32],
                reason: ErasureReason::UserRequest,
                policy: ErasurePolicy::default(),
                ts: 1,
            },
            JournalRecord::Planned {
                id,
                plan_hash: [0x88; 32],
                expected_layers: vec![WitnessLayer::LsmMemtable],
                plan_blob,
                unprovenanced: 0,
            },
            JournalRecord::Completed { id, ts: 2 },
        ])
        .await
        .unwrap();

        let before = std::fs::read(dir.path().join(JOURNAL_FILE_NAME)).unwrap();
        assert!(before.windows(canary.len()).any(|w| w == canary));

        // Verdichten auf Summary
        j.compact(&[JournalRecord::Summary {
            id_hmac: [0xAA; 32],
            plan_hmac: [0xBB; 32],
            guarantee: 0b11,
            completed_at: 2,
        }])
        .await
        .unwrap();

        let after = std::fs::read(dir.path().join(JOURNAL_FILE_NAME)).unwrap();
        assert!(!after.windows(canary.len()).any(|w| w == canary));
    }
}
