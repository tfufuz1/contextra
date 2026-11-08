# contextra-store

LSM-Tree Storage Engine, WAL Durability und Compaction.

Normativitätsklasse: **Klasse B (Storage Bundle)** (Ring 1, `beta`; SPEC v4.1 §2, §5).

## Rolle & Zweck
`contextra-store` ist das persistente Key-Value-Speichertriebwerk der Contextra-Baukasten-Architektur. Es stellt eine autarke Log-Structured Merge-Tree (LSM) Engine bereit, bestehend aus:
* **WAL (Write-Ahead-Log):** Append-Only Protokollierung mit HMAC-Integritätskette und Open-Heal bei plötzlichen Abbrüchen (Torn Writes).
* **MemTable:** Nebenläufige, geschardete In-Memory-BTreeMap mit sequenzbasierter MVCC-Versionierung.
* **SSTables (Sorted String Tables):** Unveränderliche Block-Dateien (Format V4 Standard) mit zweistufigen Bloom-Filtern (Block- und Dateiebene) und Block-Cache.
* **Compaction:** Asynchrone, mehrstufige Kompaktierung (Leveled und Adaptive) zur Begrenzung von Lese- und Schreibverstärkung.
* **Manifest:** Atomare, transaktionale Verwaltung des SSTable-Lebenszyklus (`MFMN`).

## Baukasten-Einordnung (Klasse B: Storage Bundle)
Im Baukastensystem von Contextra (SPEC §2, §5) ist `contextra-store` nicht als isolierter Einzelbaustein (Klasse A) konzipiert, sondern als Kernkomponente des kohärenten **Storage-Bundles** (Klasse B):
- **Zugehörige Crate-Familie:** `contextra-types`, `contextra-ports`, `contextra-mvcc`, `contextra-crypto`, `contextra-sys`, `contextra-store`.
- **Ring:** Ring 1.
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]` ohne Ausnahme.
- **Export-Anleitung für das Bundle:**
  Für die Verwendung in Drittprojekten wird das gesamte Storage-Bundle gemeinsam exportiert:
  ```bash
  # Export aller Bundle-Komponenten
  python3 tools/export_crate.py contextra-types --out /ziel/vendor/
  python3 tools/export_crate.py contextra-ports --out /ziel/vendor/
  python3 tools/export_crate.py contextra-sys --out /ziel/vendor/
  python3 tools/export_crate.py contextra-crypto --out /ziel/vendor/
  python3 tools/export_crate.py contextra-mvcc --out /ziel/vendor/
  python3 tools/export_crate.py contextra-store --out /ziel/vendor/
  ```

## Modul-Karte
| Modul | Verantwortung |
|---|---|
| `wal` | Append-only WAL, HMAC-Kette (`encode`, `flusher`, `hmac`, `io`, `replay`), Open-Heal |
| `memtable` | Geschardete In-Memory BTreeMap mit Sequenznummern und Tombstones |
| `sstable` | Sortierte Segmente, Block-Cache, Bloom-Filter, Streaming-Reader & Builder |
| `lsm` | Speicherorchester (`LsmStorage`), Commit-, Flush-, Scan- und Recovery-Prozesse |
| `manifest` | Transaktionales Manifest für SSTable-Generationen und Versionierung |
| `compaction` | Leveled / Adaptive Compaction Engine und Merge-Operatoren |
| `scanner` | Übergangsweise Port-Implementierungen (`LayerScanner`, `LsmPurger`) — Ziel: `contextra-erasure-adapters` (Welle 4) |
| `fsync` | Zentrale Flush- und Sync-Abstraktion (INV-FSYNC-1) |
| `tenant_codec` | Mandanten-Präfixierung (`TenantId`) und Schlüssel-Isolation |
| `system_pressure` | Systemdruck-Monitoring für adaptive Compaction |
| `engine` | In-Memory-Double (`InMemoryStorageEngine`) für tests ohne I/O |

## Physische Wartung & Introspektion
- `storage.reclaim().await`: Flusht die MemTable und schneidet verwaiste oder unvollständige WAL-Schwänze ab (`wal.truncate_uncommitted_tail()`).
- `storage.compact().await`: Triggert deterministische physische Zusammenführung und Bereinigung von SSTables.
- Feature `inspect`: Öffnet unveränderliche Schnappschuss-APIs (`sstables_snapshot()`, `memtables_snapshot()`) für externe Audit- und Scan-Adapter.

## Nicht-Ziele & Grenzen
- **Keine Rechts- oder Löschsagas:** Juristische Begriffe (DSGVO/GDPR), Löschbeweise und Sagas gehören nach `contextra-erasure`. Der Speicher stellt neutrale Bereinigungs- und Scan-Primitive bereit.
- **Keine Vektor-/Volltext-Logik:** Reines Key-Value-Triebwerk; Vektoren liegen in `contextra-vector`, Volltextindizes in `contextra-text`.
- **Keine automatische Bit-Rot-Korrektur:** Bit-Rot wird über CRC32C und HMAC deterministisch erkannt, jedoch nicht automatisch repariert.
- **Lokaler HMAC-Schlüssel:** Das Ablegen von `.wal_integrity_key` im Dateisystem schützt vor Bit-Rot, jedoch nicht vor Manipulation durch Angreifer mit Schreibrechten (SECURITY NOTE in `wal/hmac.rs`).

## Abhängigkeiten & Features
- **Abhängigkeiten (Produktion):** `contextra-types`, `contextra-ports`, `contextra-sys`, `contextra-mvcc`, `contextra-crypto`.
- **Features:**
  - `default`: Minimaler Build.
  - `inspect`: Öffnet Schnappschuss-APIs für Scan-Adapter.
  - `block-cache-v2`: Aktiviert den optimierten Block-Cache via `quick_cache`.
  - `memory-only-storage`: Ermöglicht flüchtigen In-Memory-Betrieb ohne Festplatten-I/O.
  - `docid-128`: Unterstützt 128-Bit-Dokument-IDs.
  - `legacy-wal-key`: Abwärtskompatibilitätsunterstützung für ältere WAL-Integritätsschlüssel.
  - `fault-injection`, `test-hooks`: Test- und Chaos-Engineering-Schalter.

## Garantien
- **ACID / Durability:** Atomare Commits über Write-Ahead-Logging mit synchronen Gruppen-Flushes.
- **Speichersicherheit:** Striktes `#![forbid(unsafe_code)]` im gesamten Produktionscode.
- **Neutralität:** Datenhaltung und physische Bereinigung ohne Rechts- oder Berichtsbegrifflichkeiten (SPEC §6.1, E-04). Die Port-Namen in `scanner/` (`Purger`, `LayerScanner`, `WitnessLayer`) verbleiben bis zur Bereitstellung von `contextra-erasure-adapters` hier und bilden die einzige dokumentierte Ausnahme.

```rust,no_run
use contextra_ports::StorageEngine;
use contextra_store::lsm::{LsmConfig, LsmStorage};
use contextra_types::TxId;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_dir = tempfile::tempdir()?;
    let config = LsmConfig {
        path: tmp_dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config).await?;

    let tx = TxId::new(1);
    storage.put(tx, b"benchmark_key", b"sample_value").await?;
    storage.commit(tx).await?;

    let val = storage.get(b"benchmark_key").await?;
    assert_eq!(val.as_deref(), Some(&b"sample_value"[..]));

    storage.close().await?;
    Ok(())
}
```
