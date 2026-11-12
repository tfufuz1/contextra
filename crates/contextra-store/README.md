# contextra-store

Log-Structured Merge-Tree (LSM) Speichertriebwerk für persistente Schlüssel-Wert-Speicherung.

## 1. Zweck
`contextra-store` stellt das persistente Speichertriebwerk für strukturierte Daten und Zustände bereit. Es kombiniert ein Write-Ahead-Log (WAL) mit HMAC-Integritätsketten, eine nebenläufige MemTable, unveränderliche SSTables (Format V4 Standard) mit zweistufigen Bloom-Filtern sowie hierarchische Hintergrund-Kompaktierung.

## 2. Reifegrad & Qualität
- **Status:** `beta`
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]` strikt erzwungen; null Panics in Produktionspfaden.
- **Fail-Closed:** Dateisystemfehler bei `fsync` vergiften das betroffene Handle sofort (INV-FSYNC-1).

## 3. Interne Abhängigkeiten
`contextra-store` ist modular aufgebaut und hängt intern im reinen Kern ausschließlich ab von:
- `contextra-ids` (Identitätstypen `DocId`, `TxId`, `TenantId`)
- `contextra-env` (Laufzeitumgebung und Clock-Abstraktionen)
- `contextra-sys` (Dateisystem- und Sync-Primitive)
- `contextra-crypto` (Verschlüsselung, KeyManager, WAL-HMAC-Kette)
- `contextra-mvcc` (Transaktionsvalidierung und SSI-Prüfung)

Optionale Integrationen mit `contextra-types` und `contextra-ports` liegen hinter dem Feature `workspace`.

## 4. Features
- `default = []`: Minimaler Kern.
- `workspace`: Aktiviert Brücken zu `contextra-types` und `contextra-ports`.
- `fault-injection`: Ermöglicht kontrollierte Fehlerinjektion an Dateisystem- und Fsync-Grenzen für Tests.
- `test-hooks`: Exponiert interne Diagnose- und Testmethoden.
- `docid-128`: Aktiviert 128-Bit-Dokument-IDs über `contextra-ids`.

## 5. Minimalbeispiel (Doctest / Usage)
```rust,no_run
use contextra_store::lsm::{LsmConfig, LsmStorage};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempfile::tempdir()?;
    let config = LsmConfig {
        path: temp_dir.path().to_path_buf(),
        ..Default::default()
    };

    let storage = LsmStorage::new(config).await?;

    let seq = storage.last_applied_seq();
    assert_eq!(seq, 0);

    storage.close().await?;
    Ok(())
}
```

## 6. Commit-Semantik je DurabilityMode
Wann gilt eine geschriebene Mutation (`put` / `delete`) als dauerhaft?
- **`DurabilityMode::Full`:**
  Ein `put` gilt erst dann als dauerhaft gesichert, wenn der zugehörige `commit(tx)` erfolgreich zurückgekehrt ist. Der Flusher führt vor der Bestätigung ein synchrones `sync_all()` (`fsync`/`fdatasync`) auf dem WAL durch. Nach erfolgreichem Commit übersteht der Datensatz Stromausfälle und plötzliche Prozessabstürze.
- **`DurabilityMode::WalNoHmac`:**
  Schreibt in das WAL ohne kryptografische HMAC-Kette. Gilt als dauerhaft nach Rückkehr von `commit(tx)` bei erfolgreichem Fsync.
- **`DurabilityMode::MemoryOnly`:**
  Hält Mutationen ausschließlich im Arbeitsspeicher (MemTable). Gilt als flüchtig; Daten gehen bei Prozessabbruch oder Absturz vollständig verloren.

## 7. Dateiformat-Versionen und Kompatibilität
Die drei Persistenzkomponenten von `contextra-store` führen explizite Versionsfelder zur Erkennung und Abwärtskompatibilität:

1. **Write-Ahead-Log (WAL):**
   - **Header:** Magic-Header `WAL_V3_HEADER` (`WLV3`), `WAL_V2_HEADER` (`WLV2`) oder unversioniertes V1.
   - **Typisierung:** `WalVersion` (`V1`, `V2`, `V3`). Aktueller Standard: `V3`.
   - **Kompatibilität:** Replay liest Versionen V1..V3. Ältere Versionen werden bei erfolgreichem Replay automatisch auf V3 migriert.

2. **SSTable (Sorted String Table):**
   - **Magic:** `SSTABLE_MAGIC_MFSX` (`0x5853_464D` = `MFSX` im 54-Byte-Trailer am Dateiende) bzw. `SSTABLE_MAGIC_LEGACY` (`MFST`).
   - **Versionsfeld:** 2-Byte Little-Endian `format_version` (Trailer-Bytes 48..50).
   - **Unterstützte Versionen:** `v0` bis `v4` (Standard: `4`).

3. **Manifest (VersionSet):**
   - **Header:** 4-Byte Magic `MANIFEST_HEADER_MAGIC` (`MFMN`) gefolgt von 1 Byte Versionsnummer.
   - **Aktuelle Version:** `CURRENT_MANIFEST_VERSION = 1`.
   - **Kompatibilität:** Erkennt Legacy-Manifeste ohne Header (`v0`) sowie `v1`.
