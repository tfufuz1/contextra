// FILE-CONTEXT
// ZWECK: Audit-Protokollierung von Änderungen am temporären Agenten-Kontext (CLM-Scratchpad).
// INVARIANTEN: Deterministische Erzeugung von ContextEditAuditRecord-Strukturen; keine eigene Zeiterzeugung (recorded_at_tx wird injiziert); Zero-Panic im Produktionscode.
// LEISTUNG: Schutz gegen Self-Poisoning und Prompt-Injection-Persistenz über Agenten-Kontext-Edits hinweg.
// SIEHE AUCH: crates/contextra-privacy/src/audit_trace.rs, contextra-agent::clm_scratchpad (Ring 3 / Ring 2 Entkopplung).

#![forbid(unsafe_code)]

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

/// Nachvollziehbarer Audit-Datensatz für eine Änderung am temporären Agenten-Kontext.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextEditAuditRecord {
    /// Eindeutige Kennung des ausführenden Agenten-Tasks.
    pub task_id: String,
    /// Mandanten-Identifikator (Sicherheitstrennung).
    pub tenant_id: TenantId,
    /// Art der Kontext-Änderung.
    pub kind: ContextEditKind,
    /// Optionale Bezeichnung des betroffenen Text-Chunks oder Segments.
    pub chunk_label: Option<String>,
    /// Anzahl der von der Änderung betroffenen Bytes.
    pub bytes_affected: usize,
    /// Transaktionszeitpunkt der Änderung (injizierte Logikzeit, keine Wall-Clock).
    pub recorded_at_tx: u64,
    /// Index des aktuellen Unterziels (Subgoal) im Task-Ablauf.
    pub subgoal_index: u64,
}

/// Konstruiert deterministisch einen neuen [`ContextEditAuditRecord`].
///
/// Dies ist eine reine, seiteneffektfreie Konstruktionsfunktion ohne I/O oder Zeiterzeugung.
pub fn build_context_edit_audit_record(
    task_id: String,
    tenant_id: TenantId,
    kind: ContextEditKind,
    chunk_label: Option<String>,
    bytes_affected: usize,
    recorded_at_tx: u64,
    subgoal_index: u64,
) -> ContextEditAuditRecord {
    ContextEditAuditRecord {
        task_id,
        tenant_id,
        kind,
        chunk_label,
        bytes_affected,
        recorded_at_tx,
        subgoal_index,
    }
}

/// Formatiert einen Audit-Datensatz in eine einzeilige, menschenlesbare Log-Zeile.
///
/// ### Format
/// `"task={task_id} tenant={tenant_id} kind={kind:?} chunk_label={chunk_label} bytes={bytes_affected} recorded_at_tx={recorded_at_tx} subgoal={subgoal_index}"`
///
/// Falls `chunk_label` `None` ist, wird `"none"` ausgegeben.
pub fn render_audit_line(record: &ContextEditAuditRecord) -> String {
    let label_str = record.chunk_label.as_deref().unwrap_or("none");
    format!(
        "task={} tenant={} kind={:?} chunk_label={} bytes={} recorded_at_tx={} subgoal={}",
        record.task_id,
        record.tenant_id,
        record.kind,
        label_str,
        record.bytes_affected,
        record.recorded_at_tx,
        record.subgoal_index
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_context_edit_kind_variants_and_construction() {
        let tenant = TenantId(10);

        // 1. Append
        let r_append = build_context_edit_audit_record(
            "task-42".to_string(),
            tenant,
            ContextEditKind::Append,
            Some("chunk-01".to_string()),
            256,
            1001,
            1,
        );
        assert_eq!(r_append.kind, ContextEditKind::Append);
        assert_eq!(r_append.task_id, "task-42");
        assert_eq!(r_append.tenant_id, tenant);
        assert_eq!(r_append.chunk_label.as_deref(), Some("chunk-01"));
        assert_eq!(r_append.bytes_affected, 256);
        assert_eq!(r_append.recorded_at_tx, 1001);
        assert_eq!(r_append.subgoal_index, 1);
        assert_eq!(
            render_audit_line(&r_append),
            "task=task-42 tenant=TenantId(10) kind=Append chunk_label=chunk-01 bytes=256 recorded_at_tx=1001 subgoal=1"
        );

        // 2. Replace
        let r_replace = build_context_edit_audit_record(
            "task-42".to_string(),
            tenant,
            ContextEditKind::Replace,
            Some("chunk-02".to_string()),
            128,
            1002,
            2,
        );
        assert_eq!(r_replace.kind, ContextEditKind::Replace);
        assert_eq!(
            render_audit_line(&r_replace),
            "task=task-42 tenant=TenantId(10) kind=Replace chunk_label=chunk-02 bytes=128 recorded_at_tx=1002 subgoal=2"
        );

        // 3. Remove
        let r_remove = build_context_edit_audit_record(
            "task-42".to_string(),
            tenant,
            ContextEditKind::Remove,
            Some("chunk-03".to_string()),
            64,
            1003,
            3,
        );
        assert_eq!(r_remove.kind, ContextEditKind::Remove);
        assert_eq!(
            render_audit_line(&r_remove),
            "task=task-42 tenant=TenantId(10) kind=Remove chunk_label=chunk-03 bytes=64 recorded_at_tx=1003 subgoal=3"
        );

        // 4. CheckpointReset
        let r_reset = build_context_edit_audit_record(
            "task-42".to_string(),
            tenant,
            ContextEditKind::CheckpointReset,
            None,
            0,
            1004,
            4,
        );
        assert_eq!(r_reset.kind, ContextEditKind::CheckpointReset);
        assert_eq!(r_reset.chunk_label, None);
        assert_eq!(
            render_audit_line(&r_reset),
            "task=task-42 tenant=TenantId(10) kind=CheckpointReset chunk_label=none bytes=0 recorded_at_tx=1004 subgoal=4"
        );
    }
}
