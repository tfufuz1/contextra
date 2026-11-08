# contextra-store

LSM-Tree Storage Engine, WAL Durability und Compaction.

Normativitätsklasse: **Klasse B (Storage Bundle)** (Ring 1, `beta`; SPEC v4.0 §2, §5).

## Rolle & Zweck
`contextra-store` ist das persistente Key-Value-Speichertriebwerk der Contextra-Baukasten-Architektur. Es stellt eine autarke Log-Structured Merge-Tree (LSM) Engine bereit, bestehend aus:
* **WAL (Write-Ahead-Log):** Append-Only Protokollierung mit HMAC-Integritätskette und Open-Heal bei plötzlichen Abbrüchen (Torn Writes).
* **MemTable:** Nebenläufige, geschardete In-Memory-BTreeMap mit sequenzbasierter MVCC-Versionierung.
* **SSTables (Sorted String Tables):** Unveränderliche Block-Dateien mit Bloom-Filtern und Block-Cache.
* **Compaction:** Asynchrone, mehrstufige Kompaktierung zur Minimierung von Lese- und Schreibverstärkung.
* **Manifest:** Atomare, transaktionale Verwaltung des SSTable-Lebenszyklus.

## Baukasten-Einordnung (Klasse B)
Im Baukastensystem von Contextra (SPEC §2, §5) ist `contextra-store` eine autarke Embedded-Storage-Engine innerhalb des Storage-Bundles:
- **Ring:** Ring 1
- **Klasse:** B (Storage Bundle)
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`
- **Exportierbarkeit:** Mit `python3 tools/export_crate.py contextra-store --out <ziel>` in andere Projekte übertragbar.

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

## Neutrale physische Wartung & Introspektion
- `storage.reclaim().await`: Flusht die MemTable und kürzt abgetrennte/unvollständige WAL-Schwänze (`wal.truncate_uncommitted_tail()`).
- `storage.compact().await`: Triggert deterministische physische Kompaktierung der SSTables.
- Feature `inspect`: Ermöglicht Introspektion über `sstables_snapshot()` und `memtables_snapshot()` für externe Audit- und Scan-Adapter.

## Nicht-Ziele & Entkopplung
- **Keine Lösch-Sagas / Deletion-Proofs:** Rechtsbegriffe (DSGVO/GDPR), Beweise und Löschsagas gehören ausschließlich nach `contextra-erasure`.
- **Keine Vektor-/Volltext-Logik:** Pure KV-Ebene; Indizes liegen in `contextra-vector` und `contextra-text`.

## Inventar nicht-verdrahteter Symbole & I/O-Status
- **Nicht-verdrahtete Symbole (E-07):**
  - `CostBasedAdaptivePlanner::validate_tombstone_safety`
  - `WalConfig::with_legacy_fallback`
  - `AdaptiveCompactionPlanner` Hilfskonstrukte in `compaction/adaptive.rs`
  - Sämtliche Algorithmen bleiben erhalten und inventarisiert.
- **I/O-Status:**
  - Derzeitige Datei-I/O verwendet `std::fs` (innerhalb blockierender Abschnitte) und `tokio::fs`. Die vollständige Umstellung auf den `Vfs`-Port (`contextra-ports`) ist für WP 4.4 geplant.

## Abhängigkeiten & Features
- **Abhängigkeiten (Produktion):** `contextra-types`, `contextra-ports`, `contextra-sys`, `contextra-mvcc`, `contextra-crypto`
- **Features:**
  - `default`: Minimaler Build.
  - `inspect`: Öffnet Schnappschuss-APIs für Scan-Adapter.
  - `block-cache-v2`: Aktiviert den optimierten Block-Cache via `quick_cache`.
  - `memory-only-storage`: Ermöglicht flüchtigen In-Memory-Betrieb ohne Festplatten-I/O.
  - `docid-128`: Unterstützt 128-Bit-Dokument-IDs.
  - `legacy-wal-key`: Abwärtskompatibilitätsunterstützung für ältere WAL-Integritätsschlüssel.
  - `fault-injection`, `test-hooks`: Test- und Chaos-Engineering-Schalter.

## Minimalbeispiel
```rust
use contextra_ports::StorageEngine;
use contextra_store::lsm::{LsmConfig, LsmStorage};
use contextra_types::TxId;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = LsmConfig {
        path: std::path::PathBuf::from("/tmp/storage_demo"),
        ..Default::default()
    };
    let storage = LsmStorage::open(config).await?;

    let tx = TxId::new(1);
    storage.put(tx, b"hello", b"world").await?;
    storage.commit(tx).await?;

    let val = storage.get(b"hello").await?;
    assert_eq!(val.as_deref(), Some(&b"world"[..]));
    storage.close().await?;
    Ok(())
}
```
