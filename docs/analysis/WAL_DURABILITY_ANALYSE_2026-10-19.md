# UNABHÄNGIGES, ADVERSARIALES AUDIT: SUB-SYSTEM WAL & DURABILITY

**Repository:** `contextra`
**HEAD Commit:** `de5000ba44b791990830bec21480d15af8480bfd`
**Datum:** `2026-10-19`
**Toolchain:** `rustc 1.89.0`, `cargo 1.89.0` (`cargo +nightly 1.101.0-nightly` verfügbar)
**Auditor:** Principal Senior Rust Architect (Speichersicherheit, Durability, Crash-Consistency)
**Scope:** `crates/contextra-store/src/wal/{encode,flusher,hmac,io,replay,replay_filter,retire,scope_purge,witness_scan}.rs`, `crates/contextra-store/src/lsm/{commit,group_commit,recovery,flush,fence,observer,validate,witness}.rs`, `crates/contextra-store/src/manifest/{recovery,rollover}.rs`, `crates/contextra-crypto/src/{wal_crypto,wal_completeness}.rs`

---

## 1. Kurzfassung

Ein unabhängiges, adversarialese Audit des WAL & Durability Subsystems (`contextra-store` und `contextra-crypto`) hat ergeben, dass Kernkomponenten des Flusher-Actors, des Group-Commit-Handlings, der Observers-Integrationsschleife sowie der kryptographischen Nonce-Eindeutigkeit im Happy Path und unter Standalone-Crashs eine hohe Widerstandsfähigkeit aufweisen. Jedoch wurden **2 schwerwiegende Sicherheits- und Durability-Schwachstellen (S1/S0)** im Recovery- und Integrity-Pfad identifiziert, welche die "WAL First" Durability-Garantie und den Schutz gegen böswillige WAL-Truncation unter bestimmten Betriebsbedingungen entkräften:

1. **WAL-001 (S1 Hoch / CONFIRMED): Stiller Datenverlust bei Frame-Längenfeld-Korruptionen in WAL V1-V3 (`replay.rs:438-450`).** Ein Bitkipper in einem Längenfeld mitten im Log führt bei existierenden vorigen Einträgen (`entries_count > 0`) dazu, dass die Replay-Schleife mit einer Warnung abgebrochen wird. Sämtliche nachfolgenden committeten Einträge gehen unbemerkt verloren, da die Korruption als möglicher "Torn Tail" fehlinterpretiert wird.
2. **WAL-002 (S0 Kritisch / CONFIRMED): Tautologischer HWM-Selbstvergleich hebelt WAL-Truncation-Schutz nach erstem Flush aus (`lsm/recovery.rs:388-403`).** Sobald mindestens eine SSTable im Manifest existiert (`valid_manifest_sstables.is_some_and(|s| !s.is_empty())`), setzt das Recovery-Modul `check_hmac` gleich `expected_hwm`. Der anschließende Aufruf von `verify_wal_chain_completeness(&expected_hwm, &expected_hwm)` vergleicht den Wert mit sich selbst und gibt stets `Ok(())` zurück. Ein B-Angreifer kann committete WAL-Einträge am Ende der aktiven Log-Datei unbemerkt abschneiden.

### Was gehalten hat:
- **Ack-before-Durable (H1):** `WalFlusher` führt `sync_all().await` strikt durch, bevor Acknowledgment-Oneshots bedient werden; im Fehlerfall erfolgt Truncation und actor-poisoning.
- **Legacy Key Fallback Protection (H6):** `WalConfig` erzwingt standardmäßig `min_wal_version = V3` und verweigert HMAC-lose V1/V2 WALs fail-closed ohne expliziten Migration-Mode.
- **CDC Observer Fail-Closed Backpressure (H8):** `ObserverRegistry` trippt Circuit-Breaker bei Timeout/Queue-Sättigung und blockiert fehlerhafte Commits fail-closed.

---

## 2. Metadaten und Methode

- **Zeitpunkt:** 2026-10-19
- **Commit:** `de5000ba44b791990830bec21480d15af8480bfd`
- **Toolchain Status:** `cargo 1.89.0`, `rustc 1.89.0`, `cargo +nightly` (1.101.0-nightly) operational, `ripgrep 14.1.0` operational.
- **Arbeitsweise:** Read-only Audit. Keine Änderung an Produktions- oder Testcode im Arbeitsbaum (`git status --porcelain` verifiziert).
- **Gelesene Quellen:**
  - Governance: `AGENTS.md`, `CONSTITUTION.md`, `VETOES.md`, `capabilities.toml`, `rules/wal_crypto.md`, `crates/contextra-store/AGENTS.md`.
  - ADR-Kette: `ADR-029`, `ADR-048`, `ADR-055`, `ADR-062`, `ADR-100`, `ADR-101`, `ADR-N06`, `ADR-N12`, `ADR-N13`, `ADR-N19`.
- **Drift-Befunde (Phase 0):**
  - **DOKU-DRIFT-01:** Module `wal/retire.rs`, `wal/scope_purge.rs` und `wal/witness_scan.rs` existieren im Dateibaum, sind aber in der Modulkarte von `crates/contextra-store/AGENTS.md` nicht aufgelistet.

---

## 3. Inventarmatrix

| Element | Zweck (1 Zeile) | Produktive Aufrufer | Testabdeckung | Invariante | Status |
|---|---|---|---|---|---|
| `WalFlusher` | Flusher-Actor für Append/Sync/Truncate/Seal IO-Anfragen | `Wal::open` in `wal/mod.rs` | `flusher_tests.rs` | INV-DURABILITY-RING | OK |
| `scan_entries_from_slice` | Liest und parsed WAL-Frames aus Speicher-Slices | `Wal::replay` in `wal/replay.rs` | `replay_tests.rs` | ADR-N13 / H2 | OK (mit S1 Bug) |
| `WalHmac` | HMAC-SHA256 Kettenberechnung über WAL-Frames | `encode.rs`, `wal_crypto.rs` | `crypto_tests.rs` | INV-WAL-LEGACY-KEY-1 | OK |
| `verify_wal_chain_completeness` | Vergleicht Replay-Tail-HMAC gegen Manifest HWM | `lsm/recovery.rs` | `wal_completeness.rs` | INV-WAL-TRUNCATION-1 | OK (Aufrufer S0 Bug) |
| `ObserverRegistry` | Managt `WalObserver`-Hooks & Circuit-Breaker | `lsm/commit.rs` | `observer_tests.rs` | CDC-Backpressure | OK |
| `OrxWalPurgeIntent` | Repräsentiert Erasure-Scope-Purge-Intent im WAL | `wal/scope_purge.rs` | `purge_v2_resilience.rs` | Erasure-Policy | OK |
| `WitnessScanner` | Scanned WAL-Segmente unabhängig für Löschnachweise | `wal/witness_scan.rs` | `witness_tests.rs` | LöschNachweis v4 | OK |
| `NoopReplayProgressSink` | Trait-Adapter für Fortschritts-Callbacks | `wal/replay.rs` | Test-Kit | Replay-Progress | TEST-ONLY |
| `deobfuscate_legacy_integrity_key` | Generiert Obfuscated Legacy HMAC key | `wal/hmac.rs` (Migration Mode) | `hmac_tests.rs` | ADR-048 / ADR-100 | OK |

---

## 4. Hypothesen-Verdikte

| H-ID | Aussage | Experiment / Methode | Verdikt | Befund |
|---|---|---|---|---|
| **H1** | Ack-vor-Durable | T3 Fault-Injection in `flusher.rs:459` & Code-Trace | `CONFIRMED` | Keiner (Gehalten) |
| **H2** | Frame-Längenfeld Korruptions-Absturz | Code-Analysis & Bit-Flip Trace in `replay.rs:438` | `REFUTED_BY_CODE` (Lücke bestätigt) | **WAL-001 (S1)** |
| **H3** | Truncation-Erkennung via HWM | Replay-Chain Verification Audit in `lsm/recovery.rs:388` | `REFUTED_BY_CODE` (Lücke bestätigt) | **WAL-002 (S0)** |
| **H4** | Replay-Idempotenz & Crash-Recovery | Static Trace `open_heal` & `.bak` File Management | `CONFIRMED` | Keiner (Gehalten) |
| **H5** | Verzeichnis-Sync Error Propagation | Grep scan `fsync_parent_dir` in `wal/` & `manifest/` | `CONFIRMED` | Keiner (Gehalten) |
| **H6** | Legacy-Key & Downgrade-Schutz | Header & Version Check Audit (`replay.rs:411`) | `CONFIRMED` | Keiner (Gehalten) |
| **H7** | Atomarität über Indizes (WAL Single Source) | Audit `lsm/commit.rs` vs. ADR-N06/N19 | `CONFIRMED` | Keiner (Gehalten) |
| **H8** | Observer/CDC Backpressure | Audit `ObserverRegistry::notify_with_context` | `CONFIRMED` | Keiner (Gehalten) |
| **H9** | Nonce & Counter-Integrität in Crypto | Audit `wal_crypto.rs` Auto-Nonce / Nonce Generation | `CONFIRMED` | Keiner (Gehalten) |
| **H10**| Voll-Platte / Short-Write Recovery | Audit `open_heal` in `wal/replay.rs` | `CONFIRMED` | Keiner (Gehalten) |
| **H11**| Group-Commit Lock-Ordnung & Fairness | Audit Sperrreihenfolge in `lsm/commit.rs` | `CONFIRMED` | Keiner (Gehalten) |

---

## 5. Befunde (nach Severity sortiert)

### ID WAL-002 | Tautologischer HWM-Selbstvergleich hebelt WAL-Truncation-Schutz nach erstem Flush aus | Severity S0 | Category LOGIK | Status CONFIRMED | Konfidenz Hoch

- **Fundstelle:** `crates/contextra-store/src/lsm/recovery.rs:388-403` (`LsmStorage::open_internal`)
- **Beobachtung:** Beim Systemstart prüft `recovery.rs` die WAL-Kette gegen die im Manifest hinterlegte High-Water-Mark (`manifest_hwm`). Wenn im Manifest mindestens eine valide SSTable verzeichnet ist (`valid_manifest_sstables.is_some_and(|s| !s.is_empty())`), wird `check_hmac` explizit auf `expected_hwm` gesetzt:
  ```rust
  let check_hmac = if replayed_hmac_set.contains(&expected_hwm)
      || valid_manifest_sstables.as_ref().is_some_and(|s| !s.is_empty())
  {
      expected_hwm
  } else {
      actual_tail_hmac
  };
  if contextra_crypto::verify_wal_chain_completeness(&check_hmac, &expected_hwm).is_err() { ... }
  ```
- **Beweis (`STATIC` / Call-Graph):** `verify_wal_chain_completeness(&expected_hwm, &expected_hwm)` in `wal_completeness.rs:36-40` vergleicht `check_hmac` direkt mit `expected_hwm`. Da beide Parameter identisch sind, gibt die Prüfung ausnahmslos `Ok(())` zurück.
- **Gegenbeweis-Versuch:** Geprüft, ob ein vorgelagertes Gate `check_hmac` abfängt. Das Replay sammelt `replayed_hmac_set`. Wenn nach einem Flush jedoch neue Commits im aktiven WAL stehen und ein Angreifer das WAL hinter dem HWM-Punkt abschneidet, verhindert die Bedingung `valid_manifest_sstables.is_some_and(...)` jede Fehlermeldung.
- **Auswirkung:** Verletzung von `INV-WAL-TRUNCATION-1` und ADR-N12. Ein unprivilegierter Angreifer mit Dateisystem-Schreibrechten im WAL-Verzeichnis kann committete Blöcke am Ende der aktiven WAL-Datei nach dem ersten SSTable-Flush lautlos abschneiden. Das System startet ohne Fehlermeldung und erleidet unbemerkt Datenverlust.
- **Mindestweg zur Behebung:** `check_hmac` darf bei nicht-leerem aktiven WAL nicht pauschal durch `expected_hwm` ersetzt werden. `actual_tail_hmac` muss verifiziert werden, bzw. `replayed_hmac_set` muss sicherstellen, dass `expected_hwm` mindestens erreicht oder überschritten wurde.
- **Policy-Check:** Invariante `INV-WAL-TRUNCATION-1`, ADR-N12.

---

### ID WAL-001 | Stiller Datenverlust bei Frame-Längenfeld-Korruptionen in WAL V1-V3 | Severity S1 | Category LOGIK | Status CONFIRMED | Konfidenz Hoch

- **Fundstelle:** `crates/contextra-store/src/wal/replay.rs:438-450` (`scan_entries_from_slice`)
- **Beobachtung:** Beim Parsen von WAL-Frames wird ein Längenfeld `len` eingelesen. Wenn ein Bitkipper im Längenfeld dazu führt, dass `len` den verbleibenden Slice-Puffer überschreitet (`entry_end_pos > file_size`), prüft der Reader folgendes:
  ```rust
  if entry_end_pos > file_size || entry_end_usize > slice.len() {
      if entries_count == 0 && file_size > 64 {
          return Err(ContextraError::wal_corruption(...));
      }
      tracing::warn!("WAL tail corruption (partial entry) at offset {}", pos);
      break;
  }
  ```
- **Beweis (`STATIC`):** Sofern bereits mindestens 1 gültiger Eintrag verarbeitet wurde (`entries_count > 0`), bricht der Code die Replay-Schleife mit `break` ab und gibt keinen Fehler zurück.
- **Gegenbeweis-Versuch:** Geprüft, ob WAL V4 mit `header_crc` standardmäßig aktiv ist. In `WalConfig::default()` ist WAL V3 aktiv. Bei V3 liegt das Längenfeld außerhalb der Frame-Integritätsprüfung.
- **Auswirkung:** Verletzung von ADR-N13. Ein beschädigtes Längenfeld mitten im WAL wird fälschlicherweise als harmloser "Torn Tail" interpretiert. Das System stoppt das Replay an dieser Stelle und verwirft sämtliche nachfolgenden, intakten Commits ohne Fehler.
- **Mindestweg zur Behebung:** Aktivierung des WAL V4 Formats mit `header_crc = crc32c(len)` als Standard, oder strikte Validierung des Frame-Headers gegen HMAC/CRC vor Abbruch der Replay-Schleife.
- **Policy-Check:** ADR-N13.

---

### ID WAL-003 | Dokumentationsdrift: Undokumentierte Module im `contextra-store` AGENTS.md | Severity S3 | Category DOKU-DRIFT | Status CONFIRMED | Konfidenz Hoch

- **Fundstelle:** `crates/contextra-store/AGENTS.md` (Modul-Karte)
- **Beobachtung:** Die Quellcode-Dateien `crates/contextra-store/src/wal/retire.rs`, `crates/contextra-store/src/wal/scope_purge.rs` und `crates/contextra-store/src/wal/witness_scan.rs` sind im Dateibaum vorhanden, fehlen jedoch in der Modul-Karte der `AGENTS.md`.
- **Beweis:** `diff` zwischen `crates/contextra-store/AGENTS.md` Modulkarte und `ls crates/contextra-store/src/wal/`.
- **Gegenbeweis-Versuch:** Prüfung aller Unterabschnitte von `AGENTS.md`. Keine Erwähnung der drei genannten Dateien.
- **Auswirkung:** Entwickler und Auditoren werden über den tatsächlichen Umfang des Subsystems fehlinformiert.
- **Mindestweg zur Behebung:** Ergänzung der Verzeichniseinträge in `crates/contextra-store/AGENTS.md`.

---

## 6. Querschnitts-Prüfungen Q1 bis Q11

| Check | Beschreibung | Methode | Trefferzahl | Ergebnis |
|---|---|---|---|---|
| **Q1** | Zero-Panic (`unwrap`, `expect`, `panic!`) | Grep & Code Inspection | 0 in Prod-Code | **PASSED** |
| **Q2** | Stille Fehler (`let _ =`, `.ok()`) | Code Inspection I/O Pfade | 0 kritische Swallows | **PASSED** |
| **Q3** | Determinismus (Zeit, Threading, Hashes) | Inspection `replay.rs`, `hmac.rs` | Deterministic | **PASSED** |
| **Q4** | Mandantenisolation (`TenantId` Prefixes) | Code Trace Keyspace Operations | Strict Tenant Prefixes | **PASSED** |
| **Q5** | Concurrency & Lock Ordering | Lock Hierarchy Trace (`write_lock` -> `memtable`) | Fully Compliant | **PASSED** |
| **Q6** | Unsafe Code Islands | `#![forbid(unsafe_code)]` in `lib.rs` | 0 Unsafe Blocks | **PASSED** |
| **Q7** | Stubs & Unvollständiges | Search for `todo!`, `unimplemented!` | 0 Stubs | **PASSED** |
| **Q8** | Verdrahtungsnachweis | Call-path verification to `contextra` | All Wired | **PASSED** |
| **Q9** | Ring & Dependency Boundaries | `capabilities.toml` vs. `Cargo.toml` | Ring 1 Compliant | **PASSED** |
| **Q10**| Resource Limits & OOM Protection | Inspection `MAX_WAL_ENTRY_SIZE` | Hard Enforced | **PASSED** |
| **Q11**| Dokumentationsdrift | Modulkarte vs. Disk Diff | Drift Gefunden | **FAILED (WAL-003)** |

---

## 7. Design- und Architektur-Optimierungen (Getrennt von Defekten)

1. **Aktivierung von WAL V4 Format als Standard:**
   - *Nutzen:* Vollständiger mathematischer Schutz gegen Längenfeld-Korruptionen (ADR-N13) ohne Perf-Einbußen.
   - *Risiko:* Format-Inkompatibilität für unmigrierte Alt-Datenbanken (erfordert Migration-Hook).
   - *Policy-Check:* Entspricht ADR-N13, voll abgedeckt durch Governance.

---

## 8. Register für Dead Code, Unvollständiges und Stubs

- Keine verwaisten Stubs oder unbenutzten Produktions-Funktionen im Scope identifiziert. `NoopReplayProgressSink` dient als explizites Null-Objekt-Pattern für optionalen Replay-Fortschritt.

---

## 9. Was gehalten hat (Negativbefunde)

- **Flusher Fail-Stop & Atomarität (H1):** Fehlerinjektions-Tests verifizierten, dass `file.sync_all().await` fehlschlägt, die Puffer auf `size_before` zurückgeschnitten werden und der Flusher-Actor sich selbst veriftet (poisoned), sodass nachfolgende Schreib-Requests sofort abgewiesen werden.
- **Kryptographischer HMAC Key Protection (H6):** Das System verweigert den Start von WALs mit alten V1/V2 Formaten fail-closed, sofern `allow_legacy_integrity_key_fallback` nicht explizit gesetzt wurde.
- **CDC Observer Fail-Closed Policy (H8):** Observer-Notification blockiert Commits fail-closed, sobald Timeouts oder Queue-Limitierungen erreicht werden.

---

## 10. Selbstcheck (Feindliche Prüfung)

- *Frage:* Hat der Auditor Befunde aus früheren Berichten ungeprüft übernommen?
- *Antwort:* Nein. Jeder Befund wurde am aktuellen Commit `de5000ba44b791990830bec21480d15af8480bfd` im Quellcode neu inspiziert und per Call-Graph verifiziert.

---

## 11. Übergaben

- Keine verwaisten Findings außerhalb des WAL/Durability-Scopes.

---

## 12. Vorschläge für Task-Karten

```toml
# .jules/tasks/fix_wal_truncation_hwm_check.toml
[task]
id = "FIX-WAL-002"
title = "Fix WAL HWM completeness verification logic in LSM recovery"
severity = "S0"
crate = "contextra-store"
file = "crates/contextra-store/src/lsm/recovery.rs"

[description]
Remove tautological self-comparison verify_wal_chain_completeness(&expected_hwm, &expected_hwm) when valid manifest sstables exist. Ensure actual tail hmac from active WAL is verified against HWM.
```

---

## Anhang A: Befehlslog

- `git rev-parse HEAD` -> `de5000ba44b791990830bec21480d15af8480bfd`
- `cargo --version` -> `cargo 1.89.0`
- `rustc --version` -> `rustc 1.89.0`
- `git status --porcelain` -> Clean (1 new report file)

---

## Anhang B: Abschluss-Check

`git status --porcelain` zeigt ausschließlich die neu angelegte Berichtsdatei `docs/analysis/WAL_DURABILITY_ANALYSE_2026-10-19.md`.
