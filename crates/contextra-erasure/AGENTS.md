# AGENTS.md — `contextra-erasure`
> Ring 3 | Modularer Löschkern: Saga, Vault, GateLog, Ledger, Restore-Guard, Witness & Reports | ~7500 LOC

## 1. Rolle & Zweck (Baukasten-Prinzip)
`contextra-erasure` ist der autarke, revisionssichere Löschbaustein des Contextra-Ökosystems (Ring 3).
Er kapselt die vormals im gesamten Speicherkern verstreute mehrstufige Löschkette und stellt die kryptografisch verifizierbare Bereinigung (Saga, Gate-Log, Tresor/Vault, Journal, Witness/Messung und Ed25519-signierte Berichte) als unabhängigen Building Block bereit.
Speicherkerne (`store`, `vector`, `text`, `graph`, `crypto`, `engine`, `db`) bleiben dadurch vollkommen neutral und implementieren ausschließlich generische Bereinigungs-Ports (`Purger`, `LayerScanner`, `StorageBackend`, `ErasureTarget`) aus `contextra-ports`, während `contextra-erasure` alle regulatorischen und auditierbaren Nachweise orchestriert.

**Datenpfad**: Löschantrag → Read-Gate (RCU / ErasedSet) → Vault Key-Destruction (DEK Shredding) → Purger-Window (Store, Vector, Text, Graph) → Witness/Layer-Scan → Signierter Deletion-Report (`MFDR`).

## 2. Ring, Klasse, Reifegrad & `may_use`
- **Ring:** Ring 3 (Lösch-Orchestrierung & Audit-Kryptographie)
- **Klasse:** Kern `[K]`
- **Reifegrad:** `stable`
- **`may_use` (Obergrenze):** `contextra-types`, `contextra-ports`, `contextra-crypto` (Spec §6.9, `capabilities.toml`).

## 3. Modul-Karte

| Datei / Verzeichnis | Verantwortung |
|---|---|
| `lib.rs` | Crate-Root, `#![forbid(unsafe_code)]`, Modul-Deklarationen |
| `saga.rs` & `saga/` | `ErasureCore` & `DeletionSaga`: Zustandssaga (`Requested -> Gated -> KeyDestroyed -> Purging -> Measured -> Completed`), Seam-Traits (`SagaJournal`, `SagaGate`, etc.) |
| `vault.rs` & `vault/` | `FileVault`: 16 Shards, 80-Byte Slots, physisches Zeroing/Scrubbing von Dokumentenschlüsseln (DEKs), Sync & Poison-Handling |
| `gate.rs` & `gate/` | `ReadGate` & `GateLog`: RCU-basierte Lesesperren (`ErasedSet`), atomare lineare Gate-Registrierung |
| `ledger.rs` & `ledger/` | `ErasureLedger`: Revisionssicheres Append-Only Logbuch aller Löschereignisse |
| `journal.rs` & `journal/` | Crash-resistentes WAL-Journal für unvollendete Sagas und automatische Wiederaufnahme |
| `purge.rs` & `purge/` | `PurgeWindow`: Sequenzielle Orchestrierung der Bereinigung über Datenhalter-Purger |
| `measure.rs` & `measure/` | Restdaten-Messung (`LayerScanner`), Canary-Prüfung, Erzeugung von `LayerWitness` |
| `report.rs` & `report/` | `DeletionReport`: Ed25519-signierte Zertifikate, V5-Kanonisierung, Nachweise |
| `restore.rs` & `restore/` | `RestoreGuard`: Schutz gegen unbemerktes Wiederherstellen gelöschter Daten aus Backups |
| `scope.rs` & `scope/` | Kanonische Serialisierung von `PurgeScope` und TLV-Codec |
| `backend.rs` | Generische Seams und Schnittstellen zu Speicher- und Index-Backends |
| `error.rs` | Geschlossenes `ErasureError`-Enum |

## 4. Garantien & Architektur-Invarianten
- **Strikte Speichersicherheit:** `#![forbid(unsafe_code)]` wird über das gesamte Crate hinweg erzwungen.
- **Saga-Zustandsintegrität:** Feste Zustandsübergänge (Requested -> Gated -> KeyDestroyed -> Purging -> Measured -> Completed); crash-resistent über `ErasureJournal`.
- **Durable Gate-Log & Ledger:** Revisionssichere Protokollierung aller Read-Gates und Hash-Chaining mit HMAC-Absicherung (`chain = SHA-256(prev || len || ct || tag)`).
- **Crypto-Shredding & Vault (D-55):** 16 Shards; physisches Zeroing von Schlüsseln; jeder I/O- oder fsync-Fehler vergiftet den Handle unwiderruflich (Fail-Closed).
- **Messung & Witness:** Physische Speicher-Scans über `LayerScanner` und canary-gestützte Kontrollmessungen.
- **Signierte Berichte:** Generierung kanonischer JSON-Audit-Berichte mit Ed25519-Signaturen via `ReportSigner`.
- **Zero-Panic-Doktrin:** Vollständiger Ausschluss von `unwrap()` und `expect()` in produktiven Löschpfaden.

## 5. Verbotene Muster
- Jegliche Verwendung von `unsafe`.
- Direkte Abhängigkeiten zu konkreten Speicher-Engines (`LsmStorage`, `HnswIndex` etc.); Kommunikation erfolgt AUSSCHLIESSLICH über Ports (`Purger`, `LayerScanner`, `StorageBackend`).
- Fortsetzen von Löschungen bei inkonsistentem Journal-Zustand.
- Unsignierte oder unvollständig gemessene Abschluss-Berichte.
- D-90 Verletzungen: Keine `mod.rs`-Dateien verwenden.

## 6. Schnittstellen & Datenfluss (Ports & Abstraktionen)
- **Implementiert:** `ErasureCore`, `ErasureJournal`, `ReadGate`.
- **Konsumiert:** `Purger`, `LayerScanner`, `LedgerSink`, `ReportSigner`, `Clock`, `EntropyRng` aus `contextra-ports`.
- **Bereitgestellt für:** `contextra-db`, `contextra-mcp`, `contextra`.

## 7. Tests, Sabotage & Qualitätssicherung
- **Saga Crash-Recovery Tests:** Simulation von Systemabstürzen in jedem Zwischenzustand der Lösch-Saga mit lückenlosem Wiederanlauf.
- **Witness & Canary Tests:** Sabotagetests mit absichtlich platzierten Restfragmenten zur Prüfung der Scanner-Sensitivität.
- **Signatur-Verifikation:** Striktes Offline-Verification-Testing der erzeugten Audit-Berichte.

## 8. Offene Punkte & Technische Schulden
- DEBT-01: Vollständige Übernahme aller verbliebenen Bereinigungs-Workflows aus Altbeständen.
