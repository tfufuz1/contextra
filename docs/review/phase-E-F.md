# Phase E & F: Testwirksamkeit, Codequalität und Smells

## 1. Testumfang & Testwirksamkeit (Phase E)
- **Metriken**:
  - **3.727 Unit- und Integrationstests** (`#[test]` / `#[tokio::test]`)
  - **73 Property-Tests** (proptest)
  - **18 Fuzz-Targets**
  - **47 Loom-Parallelitätsmodelle**
- **Orakelkollusion & Stub-Tests (INV-ORACLE-1)**:
  - In `crates/contextra-crypto/src/deletion_proof.rs` testen mehrere Unit-Tests die Beweiserstellung gegen Mock-Funktionen (`|| Ok(true)`), anstatt die physische Entfernung von Vektoreinträgen aus dem HNSW-Speicher zu verifizieren.
  - *Befund (S1)*: Hohe Testanzahl signalisiert Abdeckung, prüft bei kritischen Löschpfaden jedoch teilweise Kollusionstests oder Stubs.

---

## 2. Codequalität, Code-Smells & Anti-Muster (Phase F)
- **Verschluckte Fehler auf I/O-Pfaden**:
  - Es wurden **72 Vorkommen von `let _ =` und `.ok()`** auf I/O- und Persistenzpfaden gefunden (z. B. `let _ = crate::util::fsync_parent_dir(&storage.config.path).await;` in `lsm/ops/compaction.rs:211`).
  - *Befund (S1)*: Das Ignorieren von `fsync`-Rückgabewerten auf Verzeichnisebene gefährdet die Crash-Integrität nach Compaction-Phasen (Betriebssystem-Flush-Fehler werden nicht propagated).
- **God-Module / Ausreißer-Dateien**:
  - `crates/contextra-store/src/lsm/recovery.rs` (> 1.200 Zeilen)
  - `crates/contextra-engine/src/collection/search/hybrid/mod.rs` (> 1.500 Zeilen)
- **Primitive Obsession & Stringly-Typed Interfaces**:
  - Mandanten-IDs, Schichtnamen und Löschscopes werden stellenweise als freie `String` oder `&str` durch Funktionsketten gereicht, anstatt typisierte Wrappers (`TenantId`, `DelLayerId`) zu erzwingen.
