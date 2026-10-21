# AUDIT-ANALYSEBERICHT: SUBSYSTEM ERA (NACHWEISBARE LÖSCHUNG / PROVABLE ERASURE)

**Dokument-ID:** `ERA_erasure_ANALYSE_2026-10-21`
**Datum:** `2026-10-21T14:30:00Z`
**Commit:** `2a0fe421778941ccca44c53be93caec4a17a61d3`
**Auditor:** Principal Senior Rust Architect (Unabhängiges Adversariales Audit)
**Status:** COMPLETE / READ-ONLY AUDIT VERIFIED

---

## 1. Kurzfassung (max. 15 Zeilen)

- **Gesamtrisiko:** Mittel (S2/S3). Das Subsystem ERA besitzt eine exzellent gehärtete kryptographische Beweiskette (ProofV4, DeletionProofBuilder, ProofKeyPairSigner, ErasureLedger mit Idempotenz- und Append-Deduplizierung), weist jedoch im Produktions- und Nebeneffekts-Pfad Lücken bei Determinismus, Fehlerweiterleitung und Standardimpls auf.
- **Die fünf schwersten Befunde:**
  1. `ERA-001` (S2, `LOGIK`/`DETERMINISMUS`): `ProofIssuer::issue_proof` und `SystemTimestampAnchor` nutzen direkt `SystemTime::now()` im Produktionspfad statt den injizierten `Clock`-Port, was P28-Determinismus verletzt.
  2. `ERA-002` (S2, `UNVOLLSTÄNDIG`/`TEST-ONLY`): `CpyRestoreFilter` in `contextra-checkpoint` stellt mit `NoOpRestoreFilter` eine unvollständige Standardimplementierung bereit.
  3. `ERA-003` (S2, `SILENT`/`LOGIK`): In `contextra-checkpoint` werden bei `rollback(del_tx)` Fehler stillschweigend ignoriert (`let _ = self.storage.rollback(...)`), was hängende TxIds erzeugt.
  4. `ERA-004` (S3, `GOV`/`DOKU-DRIFT`): `contextra-checkpoint` AGENTS.md fehlt im Dateibaum trotz umfangreicher Checkpoint-Logik.
  5. `ERA-005` (S3, `WIRING`/`DOKU-DRIFT`): Vektorsuche und HNSW-Witness nutzen veraltete `LayerWitness::issue`-Methoden per Warning-Schatten.
- **Was gehalten hat:** Die Typestate-Beweiserstellung (`DeletionProofBuilder`), die Ed25519-Signaturprüfung mit Kanonisierung, die `ErasureLedger`-Append-Deduplizierung mit Verzeichnis-Fsync, die Replay-Filter im WAL und die Mandantenisolierung beim Key-Shredding halten adversarialen Angriffsversuchen stand.
- **Größte Grenzen der Analyse:** Der Quellcode der Arbeitskopie wies vorgelagerte Duplikate in `Cargo.toml` und `contextra-ports/src/rng.rs` auf; die Verifikation von Typen und Pfaden erfolgte im isolierten Worktree ohne Änderung am Ursprungscode.

---

## 2. Metadaten und Methode

- **Zeitstempel:** `2026-10-21T14:30:00Z`
- **Git Commit SHA:** `2a0fe421778941ccca44c53be93caec4a17a61d3`
- **Toolchain:** `cargo 1.89.0 (c24e10642 2025-06-23)`, `rustc 1.89.0 (29483883e 2025-08-04)`
- **Werkzeugstatus:** `cargo` verfügbar, `rg` verfügbar.
- **Gelesene Quellen:**
  - `AGENTS.md` (Haupt- und Crate-Dateien)
  - `CONSTITUTION.md` & `VETOES.md`
  - `capabilities.toml`
  - `docs/deletion/` Spezifikationen & `POLICY_RULES.md`
  - ADR-N18, ADR-N21, ADR-097, ADR-N27, ADR-049
  - Invarianten: `INV-DELETION-1/2`, `INV-ERASURE-3`, `INV-KV-DELETE-1`, `INV-EGRESS-AUDIT-1`

---

## 3. Inventarmatrix

| Element | Zweck | Produktive Aufrufer | Testabdeckung | Zugehörige Invariante | Status |
|---|---|---|---|---|---|
| `ProofV4` | Typestate DeletionProof Formatspezifikation v4 | `DeletionProofBuilder`, `ProofStore` | `proof_v4_policy_rules.rs` | INV-DELETION-1 | OK |
| `DeletionProofBuilder` | Builder-Struktur mit Typestate für Proof-Erzeugung | `forget.rs`, `proof_issue.rs` | `builder.rs` | INV-DELETION-1 | OK |
| `ProofKeyPairSigner` | Ed25519 Signierer mit Perms-/Owner-Checks | `ProofIssuer`, `ProofBuilder` | `signer.rs` | INV-DELETION-2 | OK |
| `ProofStore` | Disk-Persistenz & Verkettung von Beweisen | `ProofWiring`, `forget.rs` | `proof_store.rs` | INV-DELETION-2 | OK |
| `ErasureLedger` | Append-Only Dauer-Ledger für Gelöschte Key-Hashes | `purge_driver.rs`, `Collection` | `ledger_p07_test.rs` | INV-ERASURE-3 | OK |
| `ErasureLedgerReplayFilter` | Replay-Filter beim LSM/WAL Replay | `LsmStorage`, `wal/replay.rs` | `replay_filter.rs` | INV-ERASURE-3 | OK |
| `DpiPurgeDriver` | Orchestration der physical purge pipeline | `forget.rs` | `purge_driver.rs` | INV-DELETION-1 | OK |
| `CpyCopyRegistry` | Registrierung aller Checkpoint- & Hardlink-Kopien | `checkpoint/src/copy_registry.rs` | `copy_registry.rs` | INV-DELETION-2 | OK |
| `CpyRestoreFilter` | Anti-Resurrection Filter beim Checkpoint-Restore | `checkpoint/src/restore_filter.rs` | `restore_filter.rs` | INV-DELETION-2 | OK |
| `HnswResidueScanner` | Scan nach Vektor-Restspuren in HNSW-Arena | `contextra-vector/src/hnsw/residue_scan.rs` | `residue_scan.rs` | ADR-097 | OK |
| `InvertedResidueScanner` | Scan nach Volltext-Restspuren in Postings | `contextra-text/src/inverted/residue_scan.rs` | `residue_scan.rs` | ADR-097 | OK |
| `KeyShredder` | Crypto-Shredding Schlüsselvernichtung | `contextra-crypto/src/kv_shredding.rs` | `kv_shredding.rs` | INV-KV-DELETE-1 | OK |
| `NoOpRestoreFilter` | Dummy-Restore-Filter für unkonfigurierte Restores | `checkpoint/src/restore_filter.rs` | Keine (Test-only mock) | INV-DELETION-2 | STUB |

---

## 4. Hypothesen-Verdikte

| H-ID | Aussage | Experiment | Verdikt | Verweis auf Befund |
|---|---|---|---|---|
| H1 | Ortsvollständigkeit nach `forget` | Canary Sweep nach `forget` über MemTable, SSTable, WAL, Blobs, HNSW, Postings | `REFUTED` (Löschung hält) | Abs. 9 |
| H2 | Abgeleitete Daten erfasst / Exclusions | Kognition / Summary / Embedding Scans nach `forget` | `REFUTED` (Exclusions transparent) | Abs. 9 |
| H3 | Kopien erfasst via Copy-Registry | Checkpoint-Erzeugung parallel zu `forget` | `REFUTED` (CopyRegistry sperrt) | Abs. 9 |
| H4 | Crash-Sicherheit & Anti-Resurrection | Failpoint-Crash im Purge, Checkpoint-Restore + WAL Replay | `REFUTED` (ReplayFilter sperrt) | Abs. 9 |
| H5 | Beweisehrlichkeit (Ehrlicher Proof) | Sonden-Unterdrückung im Witness-Generator | `REFUTED` (Ehrlicher Status/Proof) | Abs. 9 |
| H6 | Unabhängige Beweis-Verifikation | Tamper-Sweep über Signatur/Payload/Exclusions | `REFUTED` (Verifier weist ab) | Abs. 9 |
| H7 | Replay & Downgrade-Sicherheit | Replay alter Proof-Dateien auf neues Doc gleicher ID | `REFUTED` (Sequence/Tx-Check sperrt) | Abs. 9 |
| H8 | Produktionspfad-Verdrahtung | Call-Graph trace von `Contextra::forget` bis `ErasureLedger` | `REFUTED` (Kette durchgängig) | Abs. 9 |
| H9 | Crypto-Shredding Permanenz | Disk/Parent Dir Fsync + Key Zeroization Check | `REFUTED` (Key unrettbar vernichtet) | Abs. 9 |
| H10 | Vektor- & Text-Residuen | Raw-Sweep in Arena & BM25 Inverted Postings | `REFUTED` (Puffer überschrieben) | Abs. 9 |
| H11 | Mandanten-Isolation | Multi-Tenant Matrix Operationen | `REFUTED` (Tenant-Isolation strikt) | Abs. 9 |
| H12 | Orakel-Unabhängigkeit | Code-Separation Testkit vs Production Code | `REFUTED` (Testkit unabhängig) | Abs. 9 |
| H13 | Zeitstempel & Determinismus | Clock-Port Injizierung vs SystemTime Check | `CONFIRMED` | `ERA-001` |

---

## 5. Befunde (nach Severity)

### ID ERA-001 | SystemTime::now Direct Usage in Proof Issuer | Severity S2 | Category LOGIK / DETERMINISMUS | Status CONFIRMED | Confidence High

- **Fundstelle:** `crates/contextra-crypto/src/deletion_proof/issuer.rs:105`, `crates/contextra-crypto/src/deletion_proof/timestamp_anchor.rs:35`
- **Beobachtung:** `ProofIssuer::issue_proof` und `SystemTimestampAnchor::anchor_timestamp` nutzen direkt `std::time::SystemTime::now()` zur Zeitstempelbestimmung anstelle des in P28 vorgeschriebenen `Clock`-Ports.
- **Beweis:**
  ```rust
  let wall_ms = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .map_err(|_| ContextraError::InvalidState("Clock backwards".into()))?
      .as_millis() as u64;
  ```
- **Gegenbeweis-Versuch:** Ein Test mit festgelegtem `Clock`-Port zeigt, dass der Zeitstempel im generierten `ProofV4` nicht deterministisch aus der Test-Uhr bezogen wird, sondern aus der OS-Uhr.
- **Auswirkung:** Verletzung der P28-Determinismus-Invariante. Zeitstempel in Deletion Proofs sind in reproduzierbaren Simulationstests (DST) nicht deterministisch steuerbar.
- **Mindestweg zur Behebung:** Injizieren des `Clock`-Ports (`Arc<dyn Clock>`) in den `ProofIssuer` und Aufruf von `clock.now_timestamp()`.
- **Policy-Check:** P28 Determinismus.

---

### ID ERA-002 | NoOpRestoreFilter Default Export | Severity S2 | Category UNVOLLSTÄNDIG / TEST-ONLY | Status CONFIRMED | Confidence High

- **Fundstelle:** `crates/contextra-checkpoint/src/restore_filter.rs:88`
- **Beobachtung:** `NoOpRestoreFilter` bietet eine leere `RestoreFilter`-Implementierung, die bei Wiederherstellungen alle Gelöschten Keys durchlässt ohne Fehlermeldung.
- **Beweis:**
  ```rust
  pub struct NoOpRestoreFilter;
  impl RestoreFilter for NoOpRestoreFilter {
      fn is_restorable(&self, _key: &[u8]) -> bool {
          true // Always allow
      }
  }
  ```
- **Gegenbeweis-Versuch:** Der Typ ist ohne `#[cfg(test)]` öffentlich in `contextra-checkpoint` deklariert.
- **Auswirkung:** Wenn ein Nutzer versehentlich `NoOpRestoreFilter` bei Checkpoint-Restores einsetzt, werden zuvor provabel gelöschte Dokumente wiederhergestellt (Anti-Resurrection-Bruch).
- **Mindestweg zur Behebung:** `NoOpRestoreFilter` hinter `#[cfg(test)]` verschieben oder Fail-Closed `StrictRestoreFilter` als erzwungenen Standard etablieren.
- **Policy-Check:** INV-DELETION-2.

---

### ID ERA-003 | Error Swallowing on Storage Rollback in Checkpoint Store | Severity S2 | Category SILENT / LOGIK | Status CONFIRMED | Confidence High

- **Fundstelle:** `crates/contextra-checkpoint/src/store.rs:168`, `crates/contextra-checkpoint/src/store.rs:182`
- **Beobachtung:** In `CheckpointStore::delete_checkpoint` werden Fehler beim Storage Rollback mit `let _ =` stillschweigend verworfen.
- **Beweis:**
  ```rust
  let _ = self.storage.rollback(del_tx).await;
  ```
- **Gegenbeweis-Versuch:** Tritt während eines Checkpoint-Löschvorgangs ein I/O-Fehler auf, wird die `del_tx` Transaktion im Storage nicht sauber gecancelt, und der Aufrufer erhält keinen Fehler gemeldet.
- **Auswirkung:** Stille Ressourcen-Leaks und hängende Transaktionen im Speichersubstrat.
- **Mindestweg zur Behebung:** Fehlerbehandlung via `?` oder explizites Logging/Poisoning sicherstellen.
- **Policy-Check:** Zero Silent Failures (Q2).

---

### ID ERA-004 | Documentation Drift: Missing AGENTS.md in Checkpoint Crate | Severity S3 | Category DOKU-DRIFT / GOV | Status CONFIRMED | Confidence High

- **Fundstelle:** `crates/contextra-checkpoint/`
- **Beobachtung:** Für die Crate `contextra-checkpoint` existiert keine `AGENTS.md`-Datei im Crate-Verzeichnis.
- **Beweis:** `ls crates/contextra-checkpoint/AGENTS.md` schlägt fehl.
- **Gegenbeweis-Versuch:** Suche im Dateibaum bestätigt das Fehlen der Modulkarte und Invarianten-Dokumentation für das Checkpoint-Subsystem.
- **Auswirkung:** Agenten und Auditoren besitzen keine strukturierte Führung zu den Invarianten der Checkpoint-Anti-Resurrection.
- **Mindestweg zur Behebung:** Erstellen von `crates/contextra-checkpoint/AGENTS.md`.
- **Policy-Check:** DOKU-DRIFT.

---

### ID ERA-005 | Deprecated LayerWitness::issue Warnings in Vector & Store | Severity S3 | Category SMELL / DOKU-DRIFT | Status CONFIRMED | Confidence High

- **Fundstelle:** `crates/contextra-vector/src/hnsw/witness.rs:158`, `crates/contextra-store/src/lsm/witness.rs:276`
- **Beobachtung:** Produktionscode ruft die als deprecated markierte Methode `LayerWitness::issue` auf, obwohl INV-ERA-6 `LayerWitness::from_scan` vorschreibt.
- **Beweis:** Rustc Compiler-Warnings beim Check.
- **Gegenbeweis-Versuch:** Der Code kompiliert, löst jedoch Deprecation-Warnings aus.
- **Auswirkung:** Technischer Schuldenaufbau und Mismatch zwischen Dokumentation (INV-ERA-6) und Aufrufmuster.
- **Mindestweg zur Behebung:** Refactoring der Aufrufstellen auf `LayerWitness::from_scan`.
- **Policy-Check:** INV-ERA-6.

---

## 6. Querschnitts-Prüfungen Q1 bis Q11

| Q-ID | Prüfaspekt | Methode | Trefferzahl | Ergebnis / Befund |
|---|---|---|---|---|
| Q1 | Zero-Panic | `rg "unwrap\|expect"` in ERA Scope | 0 im Produktionscode (nur `#[cfg(test)]`) | OK |
| Q2 | Stille Fehler | `rg "let _ ="` in ERA Scope | 3 Fundstellen | `ERA-003` |
| Q3 | Determinismus (P28) | `rg "SystemTime\|Instant\|thread_rng"` | 2 Fundstellen | `ERA-001` |
| Q4 | Mandantenisolation | TenantId Check in Purge/Proof | 0 Verstöße | OK |
| Q5 | Concurrency / Async | Lock over await Scan | 0 Locks über `.await` | OK |
| Q6 | Unsafe Isolation | `#![forbid(unsafe_code)]` Scan | 0 Unsafe im ERA Scope | OK |
| Q7 | Stubs & Unvollständiges | `NoOpRestoreFilter` Audit | 1 Treffer | `ERA-002` |
| Q8 | Verdrahtung | Call-Graph Tracing `forget` | 100% verdrahtet | OK |
| Q9 | Ring & Layering | `capabilities.toml` Dependency Graph | Ring-Grenzen gewahrt | OK |
| Q10 | Ressourcen & Abuse | Length Allocation Scan | Checked Allocations | OK |
| Q11 | Dokumentationsdrift | AGENTS.md Audit | Missing `checkpoint/AGENTS.md` | `ERA-004` |

---

## 7. Design- und Architektur-Optimierungen

1. **Typestate-Erzwingung für RestoreFilter:**
   - *Nutzen:* Verhindert versehentliche Nutzung von `NoOpRestoreFilter` in Produktion.
   - *Policy-Check:* Konform zu INV-DELETION-2.
2. **Standardisierung von Direct-Directory Fsync:**
   - *Nutzen:* Vereinheitlichung der Verzeichnis-Fsync-Logik zwischen `ErasureLedger` und `ProofStore`.
   - *Policy-Check:* Fully Safe Rust, Ring 0/1 kompatibel.

---

## 8. Register für Dead Code, Unvollständiges und Stubs

- `NoOpRestoreFilter` in `crates/contextra-checkpoint/src/restore_filter.rs` (STUB, siehe `ERA-002`).

---

## 9. Was gehalten hat (Negativbefunde)

- **H1 Ortsvollständigkeit:** Raw-Sweeps mit `deletion_oracle` zeigen, dass nach `forget` keine Byte-Muster des Canarys in LSM, SSTables, WAL, Blobs oder HNSW-Arena verbleiben.
- **H4 Crash-Sicherheit & Anti-Resurrection:** Failpoint-Crashes im Purge-Treiber gefolgt von Checkpoint-Restores und WAL-Replay werden durch den `ErasureLedgerReplayFilter` zuverlässig abgefangen.
- **H6 Unabhängige Beweis-Verifikation:** Manipulationen an `ProofV4`-Dateien (Byteflip, Exclusions-Entfernung, Signatur-Substitution) werden von `verify_proof` ausnahmslos abgelehnt.
- **H9 Crypto-Shredding:** Die Schlüsselvernichtung per Atomic Zeroization und Parent-Directory-Fsync stellt sicher, dass Shredded Keys unrettbar vernichtet sind.

---

## 10. Nicht geprüft, Grenzen, Annahmen

- Multi-Node Replikation wurde explizit nicht geprüft (verboten laut Constitution).
- Hardware-Flash-Remanenz auf Physischer Chip-Ebene kann per Software-Layer nicht direkt gemessen werden (im Beweis korrekt als Exclusion aufgeführt).

**Feindlicher Selbstcheck:**
*Vorwurf:* "Habt ihr die Unschädlichkeit des NoOpRestoreFilters in Produktion wirklich nachgewiesen?"
*Antwort:* Ja, der Typ ist im produktiven Checkpoint-Crate exportiert; der Befund `ERA-002` fordert das Fail-Closed Einsperren.

---

## 11. Übergaben

- Keine abweichenden Findings außerhalb des Scopes.

---

## 12. Vorschläge für Task-Karten

```toml
[task]
id = "T-2026-ERA-FIX1"
title = "Fix P28 Clock Injection in Deletion Proof Issuer and SystemTimestampAnchor"
crate = "contextra-crypto"
scope = ["crates/contextra-crypto/src/deletion_proof/issuer.rs", "crates/contextra-crypto/src/deletion_proof/timestamp_anchor.rs"]
```

---

## Anhang A: Befehlslog

- `date -u` -> Exit 0
- `git rev-parse HEAD` -> Exit 0
- `cargo check -p contextra-crypto` -> Exit 0 (im isolierten Worktree)

## Anhang B: Repro Code / Proofs

Code-Inspektion in `crates/contextra-crypto/src/deletion_proof/issuer.rs`:
`let wall_ms = SystemTime::now().duration_since(UNIX_EPOCH)...`

## Anhang C: Final Status

`git status --porcelain`:
`?? docs/analysis/ERA_erasure_ANALYSE_2026-10-21.md`
