# contextra-store

LSM-Tree Storage Engine, WAL Durability und Compaction.

Normativitätsklasse: **Kern `[K]`** (Ring 1, `stable`; §6.0, `capabilities.toml`).

## Rolle & Zweck
`contextra-store` ist der primäre Key-Value-Speicherbaustein von Contextra (Ring 1). Er implementiert eine performante Log-Structured Merge-Tree (LSM) Engine mit Write-Ahead-Log (WAL), HMAC-Integritätskette, MemTable, SSTables und asynchroner Compaction.

Im Baukastensystem fungiert `contextra-store` als neutraler, robuster Persistenzbaustein.

## Baukasten-Einordnung & Status
- **Ring:** Ring 1 (Speicherkerne)
- **Status:** 🟢 Modularer Baustein (Autark baubar und testbar)
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`

## Abhängigkeiten & Features
- **Abhängigkeiten (Produktion):** `contextra-types`, `contextra-ports`, `contextra-sys`, `contextra-mvcc`, `contextra-crypto`
- **Features:**
  - `block-cache-v2`: Aktiviert den optimierten Block-Cache via `quick_cache`.
  - `memory-only-storage`: Ermöglicht flüchtigen In-Memory-Betrieb ohne Festplatten-I/O.
  - `docid-128`: Unterstützt 128-Bit-Dokument-IDs.
  - `legacy-wal-key`: Abwärtskompatibilitätsunterstützung für ältere WAL-Integritätsschlüssel.
  - `fault-injection`, `test-hooks`: Test- und Chaos-Engineering-Schalter.

## Garantien
- **ACID / Durability:** Atomare Commits über Write-Ahead-Logging mit synchronen Gruppen-Flushes.
- **Speichersicherheit:** Striktes `#![forbid(unsafe_code)]` im gesamten Produktionscode.
- **Neutralität:** Datenhaltung und physische Bereinigung (`Purger`, `LayerScanner`) ohne Rechts- oder Berichtsbegrifflichkeiten (vollständig entkoppelt).

## Einbettungsbeispiel
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
    let storage = LsmStorage::new(config).await?;

    let tx = TxId::new(1);
    storage.put(tx, b"hello", b"world").await?;
    storage.commit(tx).await?;

    let val = storage.get(b"hello").await?;
    assert_eq!(val.as_deref(), Some(&b"world"[..]));
    storage.close().await?;
    Ok(())
}
```

Ein vollständiges, eigenständiges Beispiel befindet sich unter `examples/store_standalone.rs`.
