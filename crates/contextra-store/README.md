# contextra-store

Log-Structured Merge-Tree (LSM) Speichertriebwerk für persistente Schlüssel-Wert-Speicherung.

## 1. Zweck
`contextra-store` stellt das persistente Speichertriebwerk für strukturierte Daten und Zustände bereit. Es kombiniert ein Write-Ahead-Log (WAL) mit HMAC-Integritätsketten, eine nebenläufige MemTable, unveränderliche SSTables (Format V4 Standard) mit zweistufigen Bloom-Filtern sowie hierarchische Hintergrund-Kompaktierung.

## 2. Reifegrad
- **Status:** `beta`
- **Normativität:** Klasse B (Storage Bundle)
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]` strikt erzwungen; null Panics in Produktionspfaden.

## 3. Interne Abhängigkeiten
`contextra-store` gehört zum Storage-Bundle und hängt intern ausschließlich ab von:
- `contextra-types` (Basistypen: `TxId`, `DocId`, `ContextraError`)
- `contextra-ports` (Port-Schnittstellen: `StorageEngine`, `Clock`, `Vfs`)
- `contextra-sys` (Dateisystem- und Sync-Primitive)
- `contextra-crypto` (Verschlüsselung, KeyManager, WAL-HMAC-Kette)
- `contextra-mvcc` (Transaktionsvalidierung und SSI-Prüfung)

## 4. Features
- `default = ["wal-integrity"]`: Standard-Build mit vollständiger Integritätsprüfung.
- `wal-integrity`: Aktiviert kryptografische HMAC-Integritätsketten für WAL-Frames.
- `fault-injection`: Ermöglicht kontrollierte Fehlerinjektion an Dateisystem- und Fsync-Grenzen für Tests.
- `test-hooks`: Exponiert interne Diagnose- und Testmethoden.

## 5. Minimalbeispiel (Doctest / Usage)
```rust,no_run
use contextra_ports::StorageEngine;
use contextra_store::lsm::{LsmConfig, LsmStorage};
use contextra_types::TxId;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempfile::tempdir()?;
    let config = LsmConfig {
        path: temp_dir.path().to_path_buf(),
        ..Default::default()
    };

    let storage = LsmStorage::new(config).await?;

    let tx = TxId::new(1);
    storage.put(tx, b"user:100", b"Alice").await?;
    storage.commit(tx).await?;

    let val = storage.get(b"user:100").await?;
    assert_eq!(val.as_deref(), Some(&b"Alice"[..]));

    storage.close().await?;
    Ok(())
}
```

## 6. Entnahme-Rezept
Zur Entnahme in Drittprojekte wird das kohärente Storage-Bundle exportiert:
```bash
python3 tools/export_crate.py contextra-types --out /ziel/vendor/
python3 tools/export_crate.py contextra-ports --out /ziel/vendor/
python3 tools/export_crate.py contextra-sys --out /ziel/vendor/
python3 tools/export_crate.py contextra-crypto --out /ziel/vendor/
python3 tools/export_crate.py contextra-mvcc --out /ziel/vendor/
python3 tools/export_crate.py contextra-store --out /ziel/vendor/
```

## 7. Bekannte Grenzen
- **Laufzeit-Bindung:** `contextra-store` setzt eine aktive Tokio-Asynchron-Laufzeit voraus (`tokio::runtime::Handle`). Ein Betrieb in rein synchronen Umgebungen ohne Tokio ist nicht vorgesehen.

## 8. Commit-Semantik je DurabilityMode
Wann gilt eine geschriebene Mutation (`put` / `delete`) als dauerhaft?
- **`DurabilityMode::Full`:**
  Ein `put` gilt erst dann als dauerhaft gesichert, wenn der zugehörige `commit(tx)` erfolgreich zurückgekehrt ist. Der Flusher führt vor der Bestätigung ein synchrones `sync_all()` (`fsync`/`fdatasync`) auf dem WAL durch. Nach erfolgreichem Commit übersteht der Datensatz Stromausfälle und plötzliche Prozessabstürze (verifiziert in `tests/fsync_syscall_verification.rs`, `tests/crash_recovery.rs`).
- **`DurabilityMode::WalNoHmac`:**
  Schreibt in das WAL ohne kryptografische HMAC-Kette. Gilt als dauerhaft nach Rückkehr von `commit(tx)` bei erfolgreichem Fsync.
- **`DurabilityMode::MemoryOnly`:**
  Hält Mutationen ausschließlich im Arbeitsspeicher (MemTable). Gilt als flüchtig; Daten gehen bei Prozessabbruch oder Absturz vollständig verloren (verifiziert in `tests/memory_only_crash_semantics.rs`).

## 9. Dateiformat-Versionen und Kompatibilität
Die drei Persistenzkomponenten von `contextra-store` führen explizite Versionsfelder zur Erkennung und Abwärtskompatibilität:

1. **Write-Ahead-Log (WAL):**
   - **Header:** Magic-Header `WAL_V3_HEADER` (`WLV3`), `WAL_V2_HEADER` (`WLV2`) oder unversioniertes V1.
   - **Typisierung:** `WalVersion` (`V1`, `V2`, `V3`). Aktueller Standard: `V3`.
   - **Kompatibilität:** Replay liest Versionen V1..V3. Ältere Versionen werden bei erfolgreichem Replay automatisch auf V3 migriert.
   - **Ablehnung:** Versionen unterhalb von `min_wal_version` werden fail-closed mit `ContextraError::InvalidInput` abgewiesen.

2. **SSTable (Sorted String Table):**
   - **Magic:** `SSTABLE_MAGIC_MFSX` (`0x5853_464D` = `MFSX` im 54-Byte-Trailer am Dateiende) bzw. `SSTABLE_MAGIC_LEGACY` (`MFST`).
   - **Versionsfeld:** 2-Byte Little-Endian `format_version` (Trailer-Bytes 48..50).
   - **Unterstützte Versionen:** `v0` bis `v4` (Standard: `4`).
   - **Ablehnung:** Versionen > 4 werden ohne Datenverlust oder Panik sauber abgewiesen (`Unsupported SSTable format version: {version}`).

3. **Manifest (VersionSet):**
   - **Header:** 4-Byte Magic `MANIFEST_HEADER_MAGIC` (`MFMN`) gefolgt von 1 Byte Versionsnummer (Bytes 0..5).
   - **Aktuelle Version:** `CURRENT_MANIFEST_VERSION = 1`.
   - **Kompatibilität:** Erkennt Legacy-Manifeste ohne Header (`v0`) sowie `v1`.
   - **Ablehnung:** Unbekannte Versionen > 1 werden fail-closed mit `ContextraError::Storage("Unsupported MANIFEST format version: ...")` abgewiesen.
