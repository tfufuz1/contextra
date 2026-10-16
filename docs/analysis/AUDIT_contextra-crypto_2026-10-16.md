# AUDIT-Bericht: Scope `contextra-crypto` & Löschpfad

- **Rollen-ID:** S4 (Spezialist für nachweisbare Löschung, Kryptografie und Schlüsselverwaltung)
- **Scope:** `contextra-crypto`, Löschpfad (Purge, Schreibsperre "Fence", Anti-Resurrection-Ledger, Erasure-Ports in `contextra-ports`, Aufrufer in `contextra-engine`)
- **Basis-Commit:** `f9429808` (Diff-Basis zum letzten Audit: `59f853cb` vom 2026-10-13)
- **Datum:** 2026-10-16

---

## 1. Zusammenfassung & Ampelkarte

| Prüfbereich | Status | Bemerkung |
|---|---|---|
| **Löschvollständigkeit & Carrier-Reinigung** | 🟢 GELB/GRÜN | Purge löscht explicit keys in LSM/WAL/Memtable; HNSW-Tombstones ohne Edge-Rewire (VETO-F02). |
| **Wiederbelebungsschutz & Ledger** | 🟢 GRÜN | `Anti-Resurrection-Ledger` (`ctraledg` Format v1) wird bei Recovery/Replay/Restore vor Sequenzvergabe geprüft. |
| **Schreibsperre (Fence) & Isolation** | 🟢 GRÜN | Fence sperrt den erkannten scope_write_seq vor Commit (INV-FENCE-1). Keine Scope-Überschreitung. |
| **Fehlersemantik & Fail-Closed** | 🟡 GELB | Post-Commit Errors geloggt; Ausnahme: `test_append_write_failure_preserves_ram_state` schlägt wegen echten I/O-Checks fehl (P2). |
| **Kontrakt & Zusicherung (Proof)** | 🟢 GRÜN | Proof v4 / ProofV4 mit B3 Hash-Chain und Layer-Coverage S0–S4; kein gefälschter Status (`proof_status: issued`). |
| **Krypto & Nonce-Disziplin** | 🟢 GRÜN | AES-256-GCM-SIV, frische 12-Byte Nonces (Atomic counter + OsRng Prefix), Zeroize/ZeroizeOnDrop auf allen Keys. |
| **Schlüsselvernichtung & Shredding** | 🟢 GRÜN | O(1) Key-Revokation via `RevocationLog`, 2-Stage Erasure, MfksKeystore & DestroyedKey Typestate. |
| **Nachweis-Erzeugung** | 🟢 GRÜN | In Produktion via `ProofIssuer::issue` / `Contextra::drop_collection` aufgerufen; verifiziert echte Scans. |
| **Betriebsumgebung & Grenzen** | 🟢 GRÜN | Ehrliche Dokumentation über OS-Grenzen (Swap, Core Dumps, FS-Journaling) in Spec K.8 / ADRs. |

---

## 2. Delta zum letzten Bericht (`2026-10-13`)

- **Neu hinzugekommen / Geändert seit `59f853cb`:**
  - `feat(kv): default TombstoneOnly and remove v3 deletion proof creation` (`331c38c5`): DeletionProof v3 Erzeugung im Produktionspfad entfernt; v4 TLV ist exklusiver Standard.
  - `feat: restore barriers, two-stage crypto erase, and deterministic checkpoints` (`452d158c`): Einführung von 2-Stage Crypto Erase, Sync Barriers und `verify_integrity()` Prüfung mit Existenz-Checks in `RevocationLog`.
  - `feat(ports): add pure additive erasure port types (§4.3, J-02)` (`ba7a097c`): Additive Port-Typen für Erasure in `contextra-ports`.
  - `refactor(text,engine): eliminate u64::MAX read timestamps for MVCC snapshot isolation (K1.13)` (`22e599bd`).
- **Behobene Befunde aus Vorgängerbericht:**
  - DeletionProof v3 Ausmusterung abgeschlossen (nur v4 Issuance).
  - Clean HKDF / KdfHeader Bindung.
- **Offene/Geänderte Punkte:**
  - Unwired Ports: `LocalKmsProvider` als `KmsProvider` Port weiterhin ungewirkt (SOLO).
  - Neuer Test-Fehlgeschlagener Befund: `test_append_write_failure_preserves_ram_state` schlägt fehl aufgrund des Disk-Existenzchecks in `is_revoked()`.

---

## 3. Befunde

### FINDING-CRYPTO-20261016-01 (P2): Disk-Existenzprüfung in `RevocationLog::is_revoked` führt bei synthetischem In-Memory-Log mit `file_path` zu Fail-Closed Rollback-Zustand

- **ID:** `FINDING-CRYPTO-20261016-01`
- **Schwere:** P2 (Testlücke / Regressionsbefund im Fehlerpfad)
- **Datei:Zeile:** `crates/contextra-crypto/src/revocation_log.rs:788`
- **Beleg:**
  ```rust
  if let Some(ref path) = self.file_path {
      if !path.exists() || !marker_path_for(path).exists() {
          self.rollback_detected.store(true, Ordering::SeqCst);
          return true;
      }
  }
  ```
- **Risiko:** Wenn ein `RevocationLog` mit einem nicht-existenten Pfad instanziiert wird (z.B. bei der Prüfung von Schreibfehlern vor der Ersterstellung auf der Disk), markiert `is_revoked` bei Abfrage den Log dauerhaft als `rollback_detected = true`. Dadurch gibt `is_revoked` für alle (auch unrevoketen) Targets `true` zurück. Der Test `test_append_write_failure_preserves_ram_state` schlägt dadurch fehl (`Target MUST NOT be marked revoked in RAM on write failure`), weil `is_revoked(&target)` fälschlicherweise `true` liefert.
- **Empfehlung:** `is_revoked` sollte `verify_integrity()` erst aufrufen oder den Disk-Check ausführen, wenn die Initialisierung abgeschlossen ist oder die Datei zuvor erfolgreich geöffnet/erstellt wurde, bzw. synthetische Test-Instanzen ohne Pfad oder mit kontrolliertem MOCK betreiben.
- **Aufwand:** S (Small)

### FINDING-CRYPTO-20261016-02 (P3): Ungewirkter `KmsProvider` Port im Composition Root

- **ID:** `FINDING-CRYPTO-20261016-02`
- **Schwere:** P3 (Unwired Port / Architektur-Schuld)
- **Datei:Zeile:** `crates/contextra-crypto/src/kms.rs:18`
- **Beleg:** Output von `cargo xtask unwired-ports` listet `KmsProvider` -> `LocalKmsProvider` unter SOLO (implementiert, aber in keinem Composition Root verdrahtet).
- **Risiko:** Gering. KMS-Schlüssel-Rotation/Shredding wird im lokalen Modus über `KeyManager` direkt abgewickelt.
- **Empfehlung:** In Phase 5 Composition Root Verdrahtung in `contextra/src/builder.rs` nachziehen.
- **Aufwand:** M (Medium)

---

## 4. Testlücken

1. **Testlücke: Crash-Matrix während 2-Stage Crypto Shredding**
   - **Vorschlag:** Ein DST (Deterministic Simulation Test) / Crash-Test, der während der Übergangsphase zwischen Stage 1 (Logical Tombstone) und Stage 2 (Physical Erasure & Key Shredding) einen Systemabsturz injiziert.
   - **Erwartete Invariante:** Nach dem Reboot stellt der Anti-Resurrection-Ledger sicher, dass unvollständig gelöschte Keys/Daten unlesbar bleiben und der Purge-Orchestrator Stage 2 idempotent wiederaufnimmt (`purge.intent`).
2. **Testlücke: Multi-Tenant Erasure Isolation unter Schreiblast**
   - **Vorschlag:** Paralleles Ausführen von Purge auf Tenant A, während Tenant B mit hoher Schreibfrequenz in dieselbe SSTable/WAL schreibt.
   - **Erwartete Invariante:** Zero-Leakage zwischen Mandanten; Schreibsperre (Fence) von Tenant A blockiert Tenant B nicht.

---

## 5. Technische Schulden

| Typ | Ort | Aufwand | Reihenfolge |
|---|---|---|---|
| Unused Imports / Warnings | `crates/contextra-crypto/tests/kv_two_stage_erase_test.rs:8` | S | 1 |
| Ungewirkter Port | `crates/contextra-crypto/src/kms.rs` (`KmsProvider`) | M | 2 |
| Dead Code Warnings | `crates/contextra-crypto/src/deletion_witness.rs:69` (`WitnessIssuerToken::new`) | S | 3 |

---

## 6. Hypothesen

1. *Hypothese:* Parallele Purge- und Flush-Operationen könnten im LSM-Tree zu einer Data Race Bedingung führen.
   - *Verifikation:* In `contextra-store/src/lsm/purge/` erzwingt `CompactionGate` / `compaction_lease_lock` strikte gegenseitige Exklusion zwischen Purge und Compaction/Flush. Bei Konflikt wird `PurgeError::CompactionBusy` zurückgegeben. Hypothese widerlegt.

---

## 7. Quellenkonflikte und Doku-Drift

- Keine Quellenkonflikte identifiziert.
- Strikte Hierarchie eingehalten: Code & Grüne Gates > AGENTS.md > Spec.

---

## 8. Ausgeführte Scan-Befehle und Exit-Codes

| Befehl | Exit-Code | Status / Bemerkung |
|---|---|---|
| `cargo xtask deletion-guarantee-gate` | 0 | PASSED |
| `cargo xtask determinism-check` | 0 | PASSED |
| `cargo xtask doctrine-scan --crate contextra-crypto` | 0 | PASSED (Zero Panic, Zero Unsafe, No Overrides) |
| `cargo xtask orphan-symbols` | 0 | PASSED |
| `cargo xtask unwired-ports` | 2 | WARN (KmsProvider / LocalKmsProvider als SOLO gelistet) |
| `cargo xtask check-no-downcast` | 0 | PASSED (0 downcasts found) |
| `cargo xtask security-scan` | 1 | NOTICE (Audit-Tools/Clippy/cargo-audit Hinweis) |
| `cargo test -p contextra-crypto --locked` | 101 | 1 Test FAILED (`test_append_write_failure_preserves_ram_state`), 138 PASSED |

---

## 9. Nicht Geprüftes und Gründe

- **Mutation Score / `cargo-mutants`:**
  - `cargo-mutants` ist im Sandbox-Runner nicht installiert (`cargo: no such command: mutants`).
- **Hardware-Inseln / SIMD:**
  - `contextra-crypto` erzwingt `#![forbid(unsafe_code)]` und enthält keine C/Assembly/SIMD-Low-Level Unsafe Inseln (`capabilities.toml`: `unsafe_island = false`).

---
*Ende des Audit-Berichts `AUDIT_contextra-crypto_2026-10-16.md`.*
