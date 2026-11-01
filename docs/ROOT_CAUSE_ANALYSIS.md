# Tiefen-Ursachenanalyse: Architektur, Systemdesign und Logikfehler (Contextra v1)

## 1. Einleitung

Diese Ursachenanalyse (Root Cause Analysis) bewertet die architektonischen Widersprüche, Systemdesignfehler und Implementierungsmängel der Contextra-Codebasis im Abgleich mit der normativen Spezifikation `docs/SPEC.md` (v2.0.0). Sie liefert dem Implementierer die genauen Hintergründe und Ursachenketten für die Behebung.

---

## 2. Compiler- und Verdrahtungsfehler: Halb ausgeführter Rückbau (SPEC §3.1, B-01 bis B-05)

### 2.1 Problembeschreibung
`cargo check --workspace` und `cargo check --workspace --all-targets` brechen mit dutzenden Fehlern ab, primär in `crates/contextra-engine`:
- `no field 'graph_index' on type Collection`
- `no method named 'get_community' found`
- `no method named 'relate' found`

### 2.2 Tiefenursache (Root Cause)
In einer früheren Phase wurde der Beschluss gefasst, Graph-Funktionalität, Relationen, Community-Detection und Checkpoints aus dem Kern zu entfernen (Entscheidungen D-16, D-17, D-24 in SPEC §4).
Dabei wurden die entsprechenden Crates (`contextra-graph`, `contextra-checkpoint` etc.) aus dem Workspace entfernt und das Feld `graph_index` aus `Collection` gelöscht.
**Der Rückbau wurde jedoch in `crates/contextra-engine` nur unvollständig vollzogen:**
1. In `crates/contextra-engine/src/collection/search/hybrid/query.rs` und `mod.rs` wird nach wie vor auf `self.graph_index` und `self.get_community(...)` zugegriffen.
2. In Transaktionsmodulen (`src/transaction/db_transaction_commit.rs`, `db_transaction_rollback.rs`, `compensating_actions.rs`, `db_transaction.rs`) wird weiterhin versucht, Graph-Indizes transaktional zu committen oder zu rollbacken.
3. In `crates/contextra-engine/src/import.rs` wird die Methode `self.relate(...)` aufgerufen, die aus `Collection` entfernt wurde.
4. In `crates/contextra-engine/src/background_workers/hyperedge_worker.rs` versucht ein Hintergrund-Worker weiterhin, Hyperedges in einen nicht mehr existierenden Graphen zu schreiben.

### 2.3 Korrekturmaßnahme für den Implementierer
- Vollständige Eliminierung aller Graph- und Community-Referenzen in `contextra-engine`.
- Bereinigung von Hybrid-Search auf die Kernsuchmodi: BM25 (`contextra-text`), HNSW (`contextra-vector`) und RRF-Fusion (`contextra-rank`) gemäß SPEC §8.12 und §8.13.
- Entfernen des `hyperedge_worker.rs` und der `relate`-Aufrufe.

---

## 3. Sicherheits- und Korrektheitsfehler im Speicherkern (SPEC §3.2, B-10 bis B-16)

### 3.1 Snapshot-Sentinel-Bug & SSI-Fehler (SPEC B-11, D-C02)
- **Fundort:** `crates/contextra-store/src/lsm/ops/compaction.rs:225-231` & `crates/contextra-mvcc/src/ssi.rs:646-652`.
- **Root Cause:**
  In `compaction.rs` wird `prune_bound` berechnet als:
  ```rust
  let prune_bound = storage.tx_buffer.min_read_snapshot().unwrap_or(u64::MAX).min(storage.snapshot_registry.min_active_seqno());
  ```
  Wenn kein Snapshot aktiv ist, liefert `min_active_seqno()` den Wert `u64::MAX` oder `0`. Wird `u64::MAX` an `seq_val.prune_through(bound_seq)` übergeben, führt `ssi.rs` Folgendes aus:
  ```rust
  let bound_seq = bound_seq & !crate::types::TOMBSTONE_BIT;
  self.pruned_through.fetch_max(bound_seq, Ordering::SeqCst);
  ```
  Das Maskieren des Tombstone-Bits verwandelt `u64::MAX` in `i64::MAX`. `fetch_max` speichert diesen permanent in `pruned_through`.
  **Folge:** Jeder nachfolgende SSI-Commit scheitert sofort mit `SnapshotTooOld`, da jede echte Transaktionssequenz kleiner als `i64::MAX` ist.
- **Soll-Zustand (SPEC §4 D-11, §8.7):**
  Keine Verwendung von `u64::MAX` als Sentinel. Snapshot-Sequenzen müssen als `Option<Seq>` modelliert werden. Wenn kein Snapshot aktiv ist, darf `prune_through` nicht mit einem künstlichen Maximalwert aufgerufen werden. Ein gesetztes Tombstone-Bit in `bound_seq` muss als Fehler zurückgewiesen werden, statt es stillschweigend zu maskieren.

### 3.2 Lock-Bug beim Löschen (SPEC B-13)
- **Fundort:** `crates/contextra-engine/src/collection/crud/delete.rs:16-17`.
- **Root Cause:**
  ```rust
  let _guard = self.kv_locks.lock_for(id).await;
  drop(_guard);
  let mut db_tx = self.begin_transaction()?;
  ```
  Der Keyed Lock wird sofort per `drop(_guard)` freigegeben, noch *bevor* die Transaktion gestartet oder der Delete-Vorgang ausgeführt wird!
  **Folge:** TOCTOU-Race-Condition (Time-of-Check to Time-of-Use). Parallele Schreibvorgänge oder Updates auf denselben Key interleaven mit dem Delete und führen zu inkonsistentem Index- und Speichereinträgen.
- **Soll-Zustand:**
  `_guard` muss über die gesamte Lebensdauer der Delete-Operation (einschließlich Commit bzw. Rollback) gehalten werden (`let _guard = self.kv_locks.lock_for(id).await; ... db_tx.commit().await?; drop(_guard);`).

### 3.3 Optionale Verschlüsselung & Klartext-Defaults (SPEC B-14, D-05, R-SEC-1)
- **Fundort:** `crates/contextra-store/src/lsm/config.rs:89,111` & `crates/contextra/src/builder.rs`.
- **Root Cause:**
  ```rust
  pub encryption_passphrase: Option<String>,
  // Default:
  encryption_passphrase: None,
  ```
  Verschlüsselung At-Rest ist optional konfiguriert. Im Default-Zustand (`None`) schreibt der Store unverschlüsselte SSTables und WALs auf die Platte.
  **Folge:** Verstoß gegen P5 ("Keine Schalter für Sicherheit") und R-SEC-1 (Pflichtverschlüsselung). Ein DSGVO-Löschkonzept kann in einem unverschlüsselten Store ohne physischen Full-Disk-Erase keine kryptografische Unkenntlichmachung gewährleisten.
- **Soll-Zustand:**
  Verschlüsselung ist zwingend und nicht abschaltbar (`SecretBytes` / `Zeroizing<Vec<u8>>` Pflichtparameter im Builder). Kein `Option<String>`, keine Speicherung als ungeschützter Heap-String.

### 3.4 Zu niedrige KDF-Mindestanforderungen (SPEC B-15, D-06, R-SEC-3)
- **Fundort:** `crates/contextra-crypto/src/kdf.rs:36`.
- **Root Cause:**
  `pub const MIN_M_COST_KIB: u32 = 19456; // ~19 MiB`
  Die Argon2id-Speichergrenze ist auf 19 MiB festgesetzt. Dies entspricht nicht dem anerkannten Stand der Technik gegen GPU/ASIC-Angriffe.
- **Soll-Zustand:**
  Anhebung von `MIN_M_COST_KIB` auf mindestens 65.536 KiB (64 MiB), Standard m = 128 MiB, t >= 3, p = 1. Unterschreiten führt zu `ConfigError` (fail-closed).

### 3.5 WAL-Integrität & verdeckte Feature-Gates (SPEC B-10, S-SEC-1)
- **Fundort:** `crates/contextra-store/src/lib.rs:45` & `crates/contextra-store/src/wal/replay.rs`.
- **Root Cause:**
  `#![allow(unexpected_cfgs)]` maskierte, dass `cfg(feature = "wal-integrity")` im Code verwendet wurde, aber im Manifest gar nicht definiert war.
- **Soll-Zustand:**
  WAL-Integrität (HMAC-SHA256-Kette und CRC32) muss **unbedingt** und ohne jedes Feature-Gate immer aktiv sein. `#![allow(unexpected_cfgs)]` muss aus `lib.rs` entfernt werden.

---

## 4. Löschketten-Lücken (Erasure Architecture, SPEC §11, §12)

### 4.1 Problem
Im aktuellen Stand ist der alte Löschpfad (`scanner/`, `deletion_proof/`, `kv_shredding.rs`) zwar gelöscht, die drei Säulen der neuen Löscharchitektur sind jedoch noch nicht vollständig implementiert:
1. **Säule 1 (Physischer Purge & Gate):** Das `ReadGate` (Sperre sofort bei Erase-Anforderung) und der bereichsbasierte Purge-Rewrite im LSM existieren noch nicht modular als `ErasureLayer`.
2. **Säule 2 (Tresor / Schlüsselvernichtung je Dokument):** Der Schlüsseltresor (`vault.dat`) zur Speicherung und Vernichtung des Dokumentschlüssels (DEK-Salt) fehlt als separates Subsystem.
3. **Säule 3 (Unabhängige Messung & Bericht):** Die Messklassen E1 (Rohscan), E2 (Decoder-Scan), E3 (Schlüsselabwesenheit) und der Ed25519-signierte `ErasureReport` sind noch nicht als `contextra-erasure` realisiert.

### 4.2 Lösungskonzeption
Implementierung von `contextra-erasure` in Ring 3, Implementierung von `ErasureLayer` in `store`, `text` und `vector`, sowie Integration der Saga in `engine`.

---

## 5. Zusammenfassung der Handlungsanweisungen für den Implementierer

1. **Phase 1 (Kompilierung & Graph-Rückbau):** Bereinigen von `crates/contextra-engine` von allen verwaisten Graph-, Relate- und Community-Referenzen.
2. **Phase 2 (Speicherkern-Härtung):** Beheben des Snapshot-Sentinel-Bugs (`mvcc`/`store`), des Delete-Locks (`engine`), Festlegung von Argon2id >= 64 MiB (`crypto`) und Pflichtverschlüsselung mit `Zeroizing` (`store`/`engine`).
3. **Phase 3 (Erasure-Subsystem):** Implementierung der Drei-Säulen-Löschkette gemäß SPEC §11 und §12.
4. **Phase 4 (Code-Hygiene):** Beseitigung aller `unwrap()`, `expect()` und unsicheren Index-Zugriffe im Produktivcode zur Erfüllung von P6 und den Clippy-Gates.
