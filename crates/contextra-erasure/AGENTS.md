# AGENTS.md — contextra-erasure
> Ring 3 | Modularer Löschkern: Saga, Vault, GateLog, Ledger, Restore-Guard, Witness & Reports | ~7500 LOC

## 1. Zweck & Architekturrolle

Zentraler, modularer Baustein des Contextra-Baukastensystems für alle Aspekte der mehrstufigen Löschung, kryptografischen Nachweisführung und DSGVO-Regelwerke.
Kapselt sämtliche Nachweis-, Saga- und Audit-Logiken, die vollständig aus den Datenhalter-Kernen (`store`, `vector`, `text`, `graph`, `crypto`, `engine`, `db`) entfernt wurden.
Kommuniziert mit Speicher- und Index-Engines ausschließlich über neutrale, minimale Port-Schnittstellen (`Purger`, `LayerScanner`, `StorageBackend`, `ErasureTarget`) aus `contextra-ports`.

**Datenpfad**: Löschantrag → Read-Gate (RCU / ErasedSet) → Vault Key-Destruction (DEK Shredding) → Purger-Window (Store, Vector, Text, Graph) → Witness/Layer-Scan → Signierter Deletion-Report (`MFDR`).

## 2. Modul-Karte

| Datei / Verzeichnis | Verantwortung |
|---|---|
| `lib.rs` | Crate-Root, `#![forbid(unsafe_code)]`, Modul-Deklarationen |
| `saga.rs` & `saga/` | `ErasureCore` & `DeletionSaga`: Zustandssaga (`Requested -> Gated -> KeyDestroyed -> Purging -> Measured -> Completed`), Seam-Traits (`SagaJournal`, `SagaGate`, etc.) |
| `vault.rs` & `vault/` | `FileVault`: 16 Shards, 80-Byte Slots, physisches Zeroing/Scrubbing von Dokumentenschlüsseln (DEKs), Sync & Poison-Handling |
| `gate.rs` & `gate/` | `ReadGate` & `GateLog`: RCU-basierte Lesesperren (`ErasedSet`), atomare lineare Gate-Registrierung |
| `ledger.rs` & `ledger/` | `ErasureLedger`: Revisionssicheres Append-Only Logbuch aller Löschereignisse |
| `journal.rs` & `journal/` | Crash-resistentes WAL-Journal für unvollendete Sagas und automatische Wiederaufnahme |
| `purge.rs` & `purge/` | `PurgeWindow`: Sequenzielle Orchestrierung der Bereinigung über Datenhalter-Purger (a–g) |
| `measure.rs` & `measure/` | Restdaten-Messung (`LayerScanner`), Canary-Prüfung, Erzeugung von `LayerWitness` |
| `report.rs` & `report/` | `DeletionReport`: Ed25519-signierte Zertifikate, V5-Kanonisierung, DSGVO-Art.-17-Nachweise |
| `restore.rs` & `restore/` | `RestoreGuard`: Schutz gegen unbemerktes Wiederherstellen gelöschter Daten aus Backups |
| `scope.rs` & `scope/` | Kanonische Serialisierung von `PurgeScope` und TLV-Codec |
| `backend.rs` | Generische Seams und Schnittstellen zu Speicher- und Index-Backends |
| `error.rs` | Geschlossenes `ErasureError`-Enum |

## 3. Kritische Invarianten & Schutzmechanismen

- **Keine Rückkehr zu monolithischer Kopplung**: `contextra-erasure` darf niemals direkte Abhängigkeiten auf konkrete Speicher-Crates (`contextra-store`, `contextra-vector`, `contextra-text`, etc.) besitzen. Sämtliche Kommunikation läuft über `contextra-ports`.
- **Vault Poisoning (D-55)**: Jeder I/O- oder fsync-Fehler beim Überschreiben/Nullen von Vault-Slots vergiftet den Handle unwiderruflich (Fail-Closed).
- **D-90 Modul-Struktur**: Keine `mod.rs`-Dateien! Root `<module>.rs` neben `<module>/`-Verzeichnis.
- **Forbid Unsafe**: `#![forbid(unsafe_code)]` strikt einhalten.

## 4. Public API Quick-Reference

```rust
pub struct ErasureCore<S: StorageBackend> { ... }
impl<S: StorageBackend> ErasureCore<S> {
    pub async fn execute_erasure(&self, req: ErasureRequest) -> Result<DeletionReport, ErasureError>;
    pub async fn verify_report(&self, report: &DeletionReport) -> Result<VerificationOutcome, ErasureError>;
}
```

## 5. DAG & Ring-Grenzen
- **Ring**: Ring 3
- **Erlaubte Abhängigkeiten**: `contextra-types`, `contextra-ports`, `contextra-crypto`
