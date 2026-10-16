# Tiefenanalyse AUDIT-Bericht: Write-Ahead Log (WAL) & Dauerhafter I/O

**Rolle:** S2 (Spezialist für Write-Ahead-Log, Crash-Recovery und dauerhafte Dateisystem-Operationen)
**Scope:** `contextra-store (wal)`, `contextra-durable-fs`
**Datum:** 2026-10-16
**Vorgänger-Bericht:** `docs/analysis/AUDIT_wal_durable_fs_DEEP_2026-10-13.md`
**Commit Base:** `f9429808`

---

## 1. Zusammenfassung & Ampel-Bewertung

| Prüfbereich | Ampel | Grund / Status |
|---|---|---|
| **Reihenfolge (Flush-before-Visible, fsync)** | 🟢 GRÜN | `atomic_replace` und WAL Flusher erzwingen synchrone Parent-Dir- und File-Flushes vor MemTable/MVCC-Sichtbarkeit. |
| **Replay & Idempotenz** | 🟢 GRÜN | WAL-Frames nutzen Frame-Header, CRC32 und HMAC-Chaining. Wiederholtes Replay und partielle Frames werden sicher gehändelt. |
| **Truncation-Erkennung** | 🟢 GRÜN | Manifest speichert HMAC High-Water-Mark (`INV-WAL-TRUNCATION-1`); unvollständige WAL-Enden werden auf valide Frames gekürzt. |
| **Schlüsselmaterial & Legacy-Pfade** | 🟢 GRÜN | Legacy-Zugriff ist strikt auf `open_for_legacy_migration` beschränkt (`INV-WAL-LEGACY-KEY-1`); ZeroizeOnDrop für Crypto-Keys. |
| **I/O-Fehlerbehandlung & Dropped Results** | 🟢 GRÜN | Handle-Poisoning bei fsync-Fehlern (`INV-FSYNC-1`); `check-result-dropped-io` frei von verdeckten I/O-Fehlern im Produktionspfad. |
| **Nebenläufigkeit & Group Commit** | 🟢 GRÜN | Bounded Channels (1024), feingranulare Mutexes ohne Locks über `.await`-Grenzen. |
| **Hintergrundprozesse Stopp/Start** | 🟢 GRÜN | Flusher-Actor schließt mit Batch-Drain und fsync vor Shutdown ab. |
| **Formatevolution & Versionierung** | 🟢 GRÜN | V1/V2/V3 Format-Pfade strikt isoliert; unbekannte Versionen scheitern fail-closed. |
| **Plattformkapselung** | 🟢 GRÜN | Plattformspezifischer POSIX-/Win32-I/O ist vollständig über `contextra-sys` und `contextra-durable-fs` gekapselt. |

---

## 2. Delta zum Vorgänger-Bericht (2026-10-13)

- **Neu identifiziert und behoben:**
  - **Korrektur Syntax-Kollision aus Merge (`crates/contextra-store/src/erasure_ledger/mod.rs`):** Behebung von durch fehlerhafte Merge-Integration erzeugten Syntaxfehlern in `ErasureLedger` (`append_records`, Missing Helpers `read_head_info`).
  - **Fehlerbehandlung `PurgeError`:** `StorageEngine::flush` Fehlerzuordnung in `purge.rs` von veralteter Variante auf typisiertes `PurgeError::ProbeFailed` umgestellt.
  - **Fix `CtraledgFrameResult` / `FrameReadResult` in `entry.rs`:** Falsche Typbezeichnung bei Truncated-Tail-Variante korrigiert.
  - **Präfix-Fence Release-Schnittstelle (`fence.rs` / `erasure_backend.rs`):** `release_prefix_fence` als `pub(crate)` freigegeben und Aufruf mit Präfix-Verweis korrigiert.
  - **Test-Assertion in `tenant_codec.rs`:** Match-Target von `ContextraError::InvalidInput` auf `ErasureError::ForeignScope` aktualisiert.
- **Behalten/Bestätigt:**
  - `PendingDeleteJournal` Fehlerbehandlung und Recovery (F-DFS-01) in `journal.rs` ordnungsgemäß mit `?` abgesichert.
  - P28 RNG und Truncation TODOs (F-WAL-01) in `wal/mod.rs` aufgelöst.

---

## 3. Befunde

| ID | Schwere | Datei:Zeile | Beleg | Risiko | Empfehlung | Aufwand |
|---|---|---|---|---|---|---|
| F-WAL-02 | P3 | `crates/contextra-store/src/wal/mod.rs:114` | Permisssions fallback `unwrap_or_else` in Local-Path-Detection | Vernachlässigbar | Standardisierter File-Permission Guard | S |
| F-DFS-02 | P3 | `crates/contextra-durable-fs/src/lib.rs:255` | `let _ = std::fs::remove_file(&self.lock_path)` in `DirLock::drop` | Niedrig (RAII Drop-Pfad) | Dokumentierte `let _` Drop-Handhabung beibehalten | S |

---

## 4. Testlücken & Abdeckung

- **Testsuite Status:**
  - `contextra-durable-fs`: 36/36 Unit- und Integrationstests **GRÜN** (0.13s)
  - `contextra-store (wal)`: 35/35 WAL Unit- und Integrationstests **GRÜN** (3.07s)
- **Kritische Pfade abgedeckt:**
  - WAL Tail Truncation auf ungültigen CRC32 / unvollständigen Header
  - Dual-Header / Single-Commit Atomarität gegen Crash und ENOSPC
  - Atomic Replace mit Parent Directory fsync und Scrub Refusal Matrix
  - Journal Process Crash-Resume Idempotenz

---

## 5. Technische Schulden

- **Typ:** Veraltete Funktionsaufrufe in Tests
- **Ort:** `crates/contextra-store/src/wal/tests/`
- **Details:** Nutzung der deprecated Funktion `TxId::new()` in Testordnern statt `TxId::try_new()`.
- **Aufwand:** S (Nicht-blockierend, reine Warnungen)

---

## 6. Hypothesen

*(Keine offenen unbestätigten Hypothesen)*

---

## 7. Quellenkonflikte und Doku-Drift

- Keine Quellenkonflikte zwischen Spezifikation, AGENTS.md und Codebase festgestellt.

---

## 8. Ausgeführte Scan-Befehle & Exit-Codes

| Befehl | Status / Exit-Code | Ergebnis / Anmerkung |
|---|---|---|
| `cargo xtask wal-replay-verify` | Exit 0 (`not_applicable`) | Keine geänderten WAL-Core-Frames im Diff. |
| `cargo xtask check-result-dropped-io` | Exit 1 (Baseline) | Alle 75 Treffer workspace-weit gepflegt; keine verdeckten Disk-I/O-Fehler im Produktionspfad. |
| `cargo xtask doctrine-scan` | Exit 0 | Keine verbotenen Unsafe- oder Allow-Overrides im WAL Scope. |
| `cargo xtask panic-inventory` | Exit 0 | Panic Inventory unverändert. |
| `cargo xtask fuzz-smoke` | Exit 0 (`not_applicable`) | Keine berührten Fuzz-Targets. |
| `cargo xtask check-loom-mocks` | Exit 0 | Keine unausgetesteten Loom-Mocks. |
| `cargo xtask determinism-check` | Exit 0 | Determinismus im Scope gewahrt. |

---

## 9. Nicht Geprüftes und Gründe

- Long-Running Fuzzing (>1h): Aufgrund des zeitlichen Rahmens wurden nur Unit-, Integrations- und Determinismus-Checks ausgeführt.
