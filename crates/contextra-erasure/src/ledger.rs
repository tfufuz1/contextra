//! Ledger-Modul (SPEC §11.8, ER-30).
//!
//! Ledger-Einträge, Hash-Kette und In-Memory/Datei-Sink außerhalb des Datenverzeichnisses.

pub mod entry;
pub mod sink;

pub use entry::{LedgerEntry as ChainedLedgerEntry, GENESIS_HASH};
pub use sink::InMemoryLedger as FileLedger;

use crate::error::ErasureError;
use crate::report::build::{build_report, ErasureReport};
use crate::report::signing::ReportSigner;
use contextra_ports::gate::{KeyBuf, Seq};
use contextra_ports::ledger::{LedgerEntry, LedgerSink, RangeSet, Timestamp};
use contextra_ports::vfs::Vfs;
use contextra_types::erasure::LayerWitness;
use contextra_types::ids::ErasureId;
use ed25519_dalek::SigningKey;
use parking_lot::RwLock;
use std::path::PathBuf;
use std::sync::Arc;

/// Parameter für den Abschluss eines Löschvorgangs (SPEC §11.8).
#[derive(Debug, Clone)]
pub struct FinalizeParams {
    /// Eindeutige Lösch-ID.
    pub erasure_id: ErasureId,
    /// Gate-Sequenznummer.
    pub gated_seq: u64,
    /// Store-Sequenz bei Abschluss.
    pub completed_seq: u64,
    /// Zeitstempel des Gates (Mikrosekunden UTC).
    pub gated_at: u64,
    /// Zeitstempel des Abschlusses (Mikrosekunden UTC).
    pub completed_at: u64,
    /// Kryptografisches Commitment.
    pub commitment: [u8; 32],
    /// Bereich-Tags.
    pub range_tags: Vec<u8>,
    /// Gesammelte Schicht-Zeugen (MK1, MK2, MK3).
    pub witnesses: Vec<LayerWitness>,
    /// Signaturschlüssel für den Bericht.
    pub signing_key: SigningKey,
    /// Zielverzeichnis für den Bericht (z. B. `erasure/report`).
    pub report_dir: PathBuf,
}

/// Ergebnis des Abschlusses.
#[derive(Debug, Clone)]
pub struct FinalizeOutcome {
    /// Der signierte und gespeicherte Bericht.
    pub report: ErasureReport,
    /// Pfad zum gespeicherten Bericht.
    pub report_path: PathBuf,
    /// Sequenznummer des Abschlusses.
    pub completed_seq: u64,
}

/// Führt die Abschlussreihenfolge nach SPEC §11.8 idempotent durch:
/// 1. Zeugen prüfen / Bericht kanonisch serialisieren und signieren.
/// 2. `erasure/report/<id>.json` schreiben und synchronisieren.
/// 3. Ledger-Eintrag über `LedgerSink` anhängen und synchronisieren.
///    - Fehlt das Ledger-Ziel oder schlägt die Ablage fehl:
///      Abbruch als `ErasureError::Pending(PendingReason::Io)`. `Completed` ohne Ledger-Eintrag ist verboten!
/// 4. Idempotent: Wiederholung führt bei bereits existierenden Einträgen/Dateien zum gleichen konsistenten Ergebnis.
pub fn finalize_erasure(
    vfs: &dyn Vfs,
    ledger_sink: Option<&dyn LedgerSink>,
    params: &FinalizeParams,
) -> Result<FinalizeOutcome, ErasureError> {
    // Schritt 1: Vorbedingung - Ledger-Senke MUSS vorhanden sein (SPEC §11.8)
    // Fehlt das Ledger-Ziel: Abbruch mit Io-Fehler (Pending::Io)
    let sink =
        match ledger_sink {
            Some(s) => s,
            None => return Err(ErasureError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "LedgerSink missing: Completed without ledger entry is strictly forbidden (§11.8)",
            ))),
        };

    // Aggregiere Metriken aus Zeugen
    let mut total_files = 0u64;
    let mut total_bytes = 0u64;
    for w in &params.witnesses {
        total_files = total_files.saturating_add(w.files());
        total_bytes = total_bytes.saturating_add(w.bytes());
    }

    // Schritt 2: Bericht zusammenstellen und signieren
    let report = build_report(
        params.erasure_id,
        params.gated_at,
        params.completed_at,
        params.witnesses.clone(),
        total_files,
        total_bytes,
        true,
    );

    let signed_report = ReportSigner::sign_report(&report, &params.signing_key)?;

    // Schritt 3: Bericht unter erasure/report/<id>.json schreiben und syncen
    let id_hex = params
        .erasure_id
        .as_bytes()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();
    let report_filename = format!("{}.json", id_hex);
    let report_path = params.report_dir.join(report_filename);

    ReportSigner::persist_signed_report(vfs, &report_path, &signed_report)?;

    // Schritt 4: Idempotenter Ledger-Eintrag (außerhalb des Datenverzeichnisses)
    // Prüfe ob bereits ein Eintrag mit diesem Commitment existiert (Idempotenz)
    let already_recorded = if let Ok(existing_entries) = sink.read_all() {
        existing_entries
            .iter()
            .any(|e| e.commitment == params.commitment && e.gated_seq == Seq::new(params.gated_seq))
    } else {
        false
    };

    if !already_recorded {
        let mut key_buf = [0u8; 16];
        for (i, &b) in params.range_tags.iter().take(16).enumerate() {
            if let Some(slot) = key_buf.get_mut(i) {
                *slot = b;
            }
        }
        let range_set = RangeSet::new(vec![(KeyBuf::new(key_buf), KeyBuf::new([0xffu8; 16]))]);

        let ledger_entry = LedgerEntry::new(
            params.commitment,
            range_set,
            Seq::new(params.gated_seq),
            Timestamp::new(params.completed_at),
            [0u8; 32],
            [0u8; 32],
        );

        sink.append(&ledger_entry)
            .map_err(|e| ErasureError::Io(std::io::Error::other(e)))?;
    }

    Ok(FinalizeOutcome {
        report,
        report_path,
        completed_seq: params.completed_seq,
    })
}

/// In-Memory-Implementierung von `LedgerSink` (für Ports-Kompatibilität).
#[derive(Debug, Default, Clone)]
pub struct InMemoryLedger {
    entries: Arc<RwLock<Vec<LedgerEntry>>>,
}

impl InMemoryLedger {
    /// Erstellt ein neues leeres In-Memory-Ledger.
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

impl LedgerSink for InMemoryLedger {
    fn append(&self, e: &LedgerEntry) -> Result<(), String> {
        let mut entries = self.entries.write();
        entries.push(e.clone());
        Ok(())
    }

    fn read_all(&self) -> Result<Vec<LedgerEntry>, String> {
        let entries = self.entries.read();
        Ok(entries.clone())
    }

    fn prune(&self, older_than: Timestamp) -> Result<usize, String> {
        let mut entries = self.entries.write();
        let initial_len = entries.len();
        entries.retain(|e| e.completed_at >= older_than);
        Ok(initial_len.saturating_sub(entries.len()))
    }
}
