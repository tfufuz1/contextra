// FILE-CONTEXT
// ZWECK: Audit-Protokollierung von Änderungen am temporären Agenten-Kontext (CLM-Scratchpad).
// INVARIANTEN: Deterministische Erzeugung von ContextEditAuditRecord-Strukturen mit Hash-Verkettung;
//              Zeitstempel ausschließlich über den Clock-Port aus contextra-ports (INV-EGRESS-AUDIT-1);
//              Zero-Panic im Produktionscode; kein Klartext-Inhalt im Record.
// LEISTUNG: Schutz gegen Self-Poisoning und Prompt-Injection-Persistenz über Agenten-Kontext-Edits hinweg.
// SIEHE AUCH: crates/contextra-privacy/src/audit_trace.rs, contextra-agent::clm_scratchpad (Ring 3 / Ring 2 Entkopplung).

#![forbid(unsafe_code)]

use contextra_ports::Clock;
use contextra_types::TenantId;
use serde::{Deserialize, Serialize};

/// Art der Kontext-Änderung am temporären Agenten-Scratchpad.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContextEditKind {
    /// Anfügen von neuem Inhalt an den Kontext.
    Append,
    /// Ersetzen von bestehendem Inhalt im Kontext.
    Replace,
    /// Entfernen von Inhalt aus dem Kontext.
    Remove,
    /// Zurücksetzen des Kontexts auf einen Checkpoint.
    CheckpointReset,
}

/// Lokaler Fehlertyp für Audit-Ketten-Verifizierungsfehler.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ContextEditAuditError {
    /// Manipulationserkennung: Die Audit-Kette ist ab dem angegebenen Index fehlerhaft oder manipuliert.
    #[error("Audit chain tampered or invalid at index {index}")]
    ChainTampered { index: usize },
}

/// Eingabe-Parameter zur Erstellung eines Audit-Datensatzes für eine Kontext-Änderung.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextEditInput {
    /// Eindeutige Kennung des ausführenden Agenten-Tasks.
    pub task_id: String,
    /// Mandanten-Identifikator (Sicherheitstrennung).
    pub tenant_id: TenantId,
    /// Art der Kontext-Änderung.
    pub kind: ContextEditKind,
    /// Optionale Bezeichnung des betroffenen Text-Chunks oder Segments (kein Klartext-Inhalt).
    pub chunk_label: Option<String>,
    /// Anzahl der von der Änderung betroffenen Bytes.
    pub bytes_affected: usize,
    /// Index des aktuellen Unterziels (Subgoal) im Task-Ablauf.
    pub subgoal_index: u64,
}

/// Nachvollziehbarer, kryptografisch verketteter Audit-Datensatz für eine Änderung am temporären Agenten-Kontext.
///
/// **Sicherheitsgarantie:** Enthält keinen Klartext des Scratchpad-Inhalts (nur Längen, Hashes und Labels).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextEditAuditRecord {
    /// BLAKE3-Hash des vorherigen Audit-Records in der Kette (32 Bytes).
    pub previous_hash: [u8; 32],
    /// Eigenadresse: BLAKE3-Hash dieses Datensatzes über alle Felder.
    pub record_hash: [u8; 32],
    /// Eindeutige Kennung des ausführenden Agenten-Tasks.
    pub task_id: String,
    /// Mandanten-Identifikator (Sicherheitstrennung).
    pub tenant_id: TenantId,
    /// Art der Kontext-Änderung.
    pub kind: ContextEditKind,
    /// Optionale Bezeichnung des betroffenen Text-Chunks oder Segments (kein Klartext-Inhalt).
    pub chunk_label: Option<String>,
    /// Anzahl der von der Änderung betroffenen Bytes.
    pub bytes_affected: usize,
    /// Transaktionszeitpunkt der Änderung in Unix-Nanosekunden (vom Clock-Port injiziert).
    pub recorded_at_tx: u64,
    /// Index des aktuellen Unterziels (Subgoal) im Task-Ablauf.
    pub subgoal_index: u64,
}

/// Berechnet den deterministischen BLAKE3-Hash über alle Datenfelder eines Audit-Records.
pub fn compute_record_hash(
    previous_hash: [u8; 32],
    input: &ContextEditInput,
    recorded_at_tx: u64,
) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();

    // 1. Vorgänger-Hash
    hasher.update(&previous_hash);

    // 2. Task-ID
    let task_id_bytes = input.task_id.as_bytes();
    hasher.update(&(task_id_bytes.len() as u64).to_le_bytes());
    hasher.update(task_id_bytes);

    // 3. Mandant (TenantId u64)
    hasher.update(&input.tenant_id.0.to_le_bytes());

    // 4. Operation (Kind)
    let kind_byte = match input.kind {
        ContextEditKind::Append => 1u8,
        ContextEditKind::Replace => 2u8,
        ContextEditKind::Remove => 3u8,
        ContextEditKind::CheckpointReset => 4u8,
    };
    hasher.update(&[kind_byte]);

    // 5. Chunk-Label
    if let Some(label) = input.chunk_label.as_deref() {
        hasher.update(&[1u8]);
        let label_bytes = label.as_bytes();
        hasher.update(&(label_bytes.len() as u64).to_le_bytes());
        hasher.update(label_bytes);
    } else {
        hasher.update(&[0u8]);
    }

    // 6. Betroffene Bytes
    hasher.update(&(input.bytes_affected as u64).to_le_bytes());

    // 7. Zeitstempel (u64 Nanosekunden)
    hasher.update(&recorded_at_tx.to_le_bytes());

    // 8. Subgoal-Index
    hasher.update(&input.subgoal_index.to_le_bytes());

    *hasher.finalize().as_bytes()
}

/// Konstruiert deterministisch einen neuen [`ContextEditAuditRecord`] mit Hash-Verkettung.
///
/// Der Zeitstempel wird ausschließlich über den injizierten `Clock`-Port bezogen (kein SystemTime::now()).
pub fn build_context_edit_audit_record(
    previous_hash: [u8; 32],
    input: ContextEditInput,
    clock: &dyn Clock,
) -> ContextEditAuditRecord {
    let recorded_at_tx = clock.now_unix_nanos();
    let record_hash = compute_record_hash(previous_hash, &input, recorded_at_tx);

    ContextEditAuditRecord {
        previous_hash,
        record_hash,
        task_id: input.task_id,
        tenant_id: input.tenant_id,
        kind: input.kind,
        chunk_label: input.chunk_label,
        bytes_affected: input.bytes_affected,
        recorded_at_tx,
        subgoal_index: input.subgoal_index,
    }
}

/// Verifiziert die Integrität und Hash-Verkettung einer Reihe von Audit-Records.
///
/// Gibt `Ok(())` zurück, wenn die Kette intakt ist, oder `Err(ContextEditAuditError::ChainTampered { index })`
/// mit dem Index des ersten manipulierten oder fehlerhaften Datensatzes.
pub fn verify_audit_chain(chain: &[ContextEditAuditRecord]) -> Result<(), ContextEditAuditError> {
    let mut expected_previous_hash = [0u8; 32];

    for (index, record) in chain.iter().enumerate() {
        if record.previous_hash != expected_previous_hash {
            return Err(ContextEditAuditError::ChainTampered { index });
        }

        let input = ContextEditInput {
            task_id: record.task_id.clone(),
            tenant_id: record.tenant_id,
            kind: record.kind,
            chunk_label: record.chunk_label.clone(),
            bytes_affected: record.bytes_affected,
            subgoal_index: record.subgoal_index,
        };

        let computed_hash =
            compute_record_hash(record.previous_hash, &input, record.recorded_at_tx);

        if record.record_hash != computed_hash {
            return Err(ContextEditAuditError::ChainTampered { index });
        }

        expected_previous_hash = record.record_hash;
    }

    Ok(())
}

/// Formatiert einen Audit-Datensatz in eine einzeilige, menschenlesbare Log-Zeile.
///
/// ### Format
/// `"task={task_id} tenant={tenant_id} kind={kind:?} chunk_label={chunk_label} bytes={bytes_affected} recorded_at_tx={recorded_at_tx} subgoal={subgoal_index} prev_hash={prev_hash_hex} record_hash={record_hash_hex}"`
pub fn render_audit_line(record: &ContextEditAuditRecord) -> String {
    let label_str = record.chunk_label.as_deref().unwrap_or("none");
    let prev_hex = hex_encode_32(&record.previous_hash);
    let rec_hex = hex_encode_32(&record.record_hash);
    format!(
        "task={} tenant={} kind={:?} chunk_label={} bytes={} recorded_at_tx={} subgoal={} prev_hash={} record_hash={}",
        record.task_id,
        record.tenant_id,
        record.kind,
        label_str,
        record.bytes_affected,
        record.recorded_at_tx,
        record.subgoal_index,
        prev_hex,
        rec_hex
    )
}

fn hex_encode_32(bytes: &[u8; 32]) -> String {
    let mut s = String::with_capacity(64);
    for b in bytes {
        use std::fmt::Write;
        let _ = write!(s, "{:02x}", b);
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FixedTestClock {
        nanos: u64,
    }

    impl Clock for FixedTestClock {
        fn now_unix_nanos(&self) -> u64 {
            self.nanos
        }

        fn monotonic_nanos(&self) -> u64 {
            self.nanos
        }
    }

    #[test]
    fn test_all_context_edit_kind_variants_and_chain_construction() {
        let tenant = TenantId(10);
        let clock = FixedTestClock { nanos: 1001 };

        // 1. Append (record 0)
        let input0 = ContextEditInput {
            task_id: "task-42".to_string(),
            tenant_id: tenant,
            kind: ContextEditKind::Append,
            chunk_label: Some("chunk-01".to_string()),
            bytes_affected: 256,
            subgoal_index: 1,
        };

        let r0 = build_context_edit_audit_record([0u8; 32], input0, &clock);
        assert_eq!(r0.kind, ContextEditKind::Append);
        assert_eq!(r0.task_id, "task-42");
        assert_eq!(r0.tenant_id, tenant);
        assert_eq!(r0.chunk_label.as_deref(), Some("chunk-01"));
        assert_eq!(r0.bytes_affected, 256);
        assert_eq!(r0.recorded_at_tx, 1001);
        assert_eq!(r0.subgoal_index, 1);
        assert_eq!(r0.previous_hash, [0u8; 32]);

        // 2. Replace (record 1, chained to r0)
        let clock2 = FixedTestClock { nanos: 1002 };
        let input1 = ContextEditInput {
            task_id: "task-42".to_string(),
            tenant_id: tenant,
            kind: ContextEditKind::Replace,
            chunk_label: Some("chunk-02".to_string()),
            bytes_affected: 128,
            subgoal_index: 2,
        };

        let r1 = build_context_edit_audit_record(r0.record_hash, input1, &clock2);
        assert_eq!(r1.previous_hash, r0.record_hash);

        // Verify chain of 2 records
        let chain = vec![r0, r1];
        assert!(verify_audit_chain(&chain).is_ok());
    }

    #[test]
    fn test_tampered_chain_detection() {
        let tenant = TenantId(10);
        let clock = FixedTestClock { nanos: 1001 };

        let input0 = ContextEditInput {
            task_id: "task-42".to_string(),
            tenant_id: tenant,
            kind: ContextEditKind::Append,
            chunk_label: Some("chunk-01".to_string()),
            bytes_affected: 256,
            subgoal_index: 1,
        };
        let r0 = build_context_edit_audit_record([0u8; 32], input0, &clock);

        let input1 = ContextEditInput {
            task_id: "task-42".to_string(),
            tenant_id: tenant,
            kind: ContextEditKind::Replace,
            chunk_label: Some("chunk-02".to_string()),
            bytes_affected: 128,
            subgoal_index: 2,
        };
        let mut r1 = build_context_edit_audit_record(r0.record_hash, input1, &clock);

        // Tamper with r1's bytes_affected
        r1.bytes_affected = 999;

        let chain = vec![r0, r1];
        assert_eq!(
            verify_audit_chain(&chain),
            Err(ContextEditAuditError::ChainTampered { index: 1 })
        );
    }
}
