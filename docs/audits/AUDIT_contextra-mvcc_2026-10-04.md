# Contextra MVCC Algorithm Audit Report (`contextra-mvcc`)

- **Datum:** 2026-10-04
- **Auditor:** Principal Senior Rust Architect (Contextra)
- **Crate:** `contextra-mvcc` (Ring 0 / Layer 2, `#![forbid(unsafe_code)]`)
- **Fokus:** Algorithmen-Audit (MVCC Snapshot Isolation, SeqLog, SnapshotRegistry, TxBuffer, SSI)

---

## 1. Formale Invarianten-Beweistabelle (P1–P8)

| Prüfpunkt | Aussage / Invariante | Verifiziert | Beleg Datei:Zeile | Gegenbeispiel / Befund |
| :--- | :--- | :---: | :--- | :--- |
| **P1 SICHTBARKEITSFORMEL-EXAKTHEIT** | `sichtbar(e, as_of) <=> e.insert_seq <= as_of AND (e.delete_seq == None OR e.delete_seq > as_of)`. Der Grenzfall `delete_seq == Some(as_of)` bewertet strikt zu `false` (Eintrag NICHT sichtbar). | **JA** | `crates/contextra-mvcc/src/seq_log.rs:41` | Keines. Die Zeile `insert_seq <= as_of && delete_seq.is_none_or(|del| del > as_of)` erzwingt strikte Ungleichung `del > as_of`. Verifiziert via `seq_log.rs:298-300` & `visibility_proptest.rs:60-63`. |
| **P2 O(1)-KOMPLEXITÄT DER SICHTBARKEITSPRÜFUNG** | Der heißeste Prüfpfad `SeqLogEntry::is_visible` für ein gegebenes `as_of` ist vollkommen allokationsfrei und schleifenfrei (maximal zwei Integer-Vergleiche). | **JA** | `crates/contextra-mvcc/src/seq_log.rs:36-42` | Keines. Reines Bitwise-Masking und zwei `u64`-Vergleiche. Benchmark bestätigt Ausführungszeit von ~1.37 ns. |
| **P3 SNAPSHOT-REGISTRY-DATENSTRUKTUR** | Underling Data Structure ist `parking_lot::Mutex<BTreeMap<u64, Vec<Instant>>>`. `min_active_seqno()` erfolgt lock-frei via `AtomicU64` in $O(1)$ Zeit (`Acquire/Release`-Ordering). | **JA** | `crates/contextra-mvcc/src/snapshot.rs:33, 87-89, 193-195` | Keines. Dokumentationsaussage stimmt exakt mit der $O(1)$ Atomic-Read-Implementierung überein. Benchmark bestätigt konstante 1.16 ns bei $N \in \{10, 1000, 100000\}$. |
| **P4 KEIN-LOST-UPDATE** | `SeqLogEntry::insert_seq` wird nach der Konstruktion niemals überschrieben; Mutationen in `record_insert` und `record_delete` setzen ausschließlich additiv `delete_seq = Some(seq)`. | **JA** | `crates/contextra-mvcc/src/seq_log.rs:183-189, 198-204` | Keines. `insert_seq` ist nach Erstellung immutabel. |
| **P5 P28-DETERMINISMUS** | Kein Wanduhr-Bezug für logische Sequenznummern/Sichtbarkeiten. `SystemTime::now()` (0 Funde) und `thread_rng()` (0 Funde) existieren nicht im Crate. | **PARTIELL** | `crates/contextra-mvcc/src/tx_buffer.rs:302, 574`, `crates/contextra-mvcc/src/snapshot.rs:63, 97`, `crates/contextra-mvcc/src/seq_log.rs:130, 171` | `Instant::now()` wird in 14 Produktions-Convenience-Methoden als Fallback für Reaping/Pin-TTL genutzt. Sämtliche Kernfunktionen bieten jedoch deterministische `_at`-Varianten mit injiziertem Timestamp. |
| **P6 NEBENLÄUFIGKEITS-TEST** | Loom-Modelltest (`loom_tests`) und Multi-Thread Stress-Test (`normal_tests::test_concurrent_snapshot_registry_stress`) für concurrent `register`/`drop`/`unpin` und `min_active_seqno()` existieren. | **JA** | `crates/contextra-mvcc/tests/loom_snapshot_registry.rs:1-125` | Keines. Concurrent Stress-Test läuft mit 4 Threads x 1000 Iterationen vollkommen grün durch. |
| **P7 TX_BUFFER-SHARDING** | `TxBuffer` ist in unabhängig gelockte Shards (`Vec<parking_lot::RwLock<TxShard<T>>>`) aufgeteilt. Die Deadlock-Invariante "nie mehr als ein Shard-Lock gleichzeitig" wird eingehalten. | **NEIN** | `crates/contextra-mvcc/src/tx_buffer.rs:8-10, 209-215, 351-360, 587-617` | **[MAJOR] Keine typbasierte oder Runtime-Erzwingung (Debug-Assertion) der Regel "nie mehrere Shard-Locks gleichzeitig" vorhanden — nur ein Freitext-Kommentar.** Code führt zwar sequentielle Locks aus, erzwingt dies aber nicht gegen Re-Entrancy. (Sperrenhierarchie `key_shards` → `shards` ist konform). |
| **P8 ORPHAN-REAPER** | Mechanismus für verwaiste Transaktionen (`TxBuffer::reap_orphans()`, `reap_orphans_at()`, `reap_orphans_bounded()`) bereinigt abgelaufene Staged-Ops. `SnapshotGuard` nutzt RAII `Drop` für automatische Deregistrierung. | **JA** | `crates/contextra-mvcc/src/tx_buffer.rs:563-625`, `crates/contextra-mvcc/src/snapshot.rs:194-198`, `crates/contextra-mvcc/src/seq_log.rs:202-212`, `crates/contextra-mvcc/src/ssi.rs:674-688` | Ein verwaister manueller Persistent-Pin (ohne `SnapshotGuard`) hält `min_active_seqno` dauerhaft fest, wird jedoch nach `max_pin_duration` (300s) via `diagnose_pruning_blocker()` als `tracing::warn!` diagnostiziert. |

---

## 2. Datenstruktur-Analyse mit Komplexitätsaussagen

### `SeqLogEntry` & `SequenceLog`
- **Speicher-Overhead:** `SeqLogEntry` benötigt exakt 24 Bytes (8 B `DocId` + 8 B `insert_seq` + 8 B `delete_seq: Option<u64>`).
- **Sichtbarkeitsprüfung (`SeqLogEntry::is_visible`):** $O(1)$ allokations- und schleifenfrei.
- **Kompaktierung (`SequenceLog::compact`):** $O(N)$ In-Memory Sweep über `entries.retain()`, entfernt Einträge mit `delete_seq < min_active_seqno`.

### `SnapshotRegistry`
- **Interne Datenstruktur:** `parking_lot::Mutex<BTreeMap<u64, Vec<Instant>>>` für aktive Snapshots.
- **Registrierung & Release (`register`, `release`, `pin`, `unpin`):** $O(\log N)$ bzgl. der Anzahl aktiver Snapshots $N$ durch BTreeMap-Einfügung/Löschung unter Mutex-Sperre.
- **Minimal Aktive Sequenznummer (`min_active_seqno`):** Lock-freies $O(1)$ `AtomicU64.load(Ordering::Acquire)`. Verhindert Lock-Contention in Compaction-Workern von `contextra-store`.

### `TxBuffer`
- **Sharded Architecture:** `shards: Vec<parking_lot::RwLock<TxShard<T>>>` (default 64 Shards).
- **Operationen (`stage`, `drain`, `discard`):** $O(1)$ Hash-basiert auf `tx_id.inner() % shard_count`.
- **Globaler Sweep (`reap_orphans`, `min_read_snapshot`):** $O(S)$ sequentieller Durchlauf über Shards $0..S-1$, wobei jeder Lock einzeln erworben und sofort wieder freigegeben wird.

### `SsiValidator` & `SequenceLogSsiValidator`
- **Read-Set Tracking:** `ReadSet` nutzt `AHashMap<Vec<u8>, u64>` für Point-Keys und Range-Prefixes.
- **Committed Writes Index:** `CommittedWrites` verwendet `AHashMap<Vec<u8>, u64>` in Kombination mit `BTreeMap<u64, SeqBucket>`.
- **Coarsening & Bounded Capacity (B-16 / INV-MVCC-SSI-1):** Bei Erreichen der Kapazitätsgrenze (`DEFAULT_MAX_TRACKED_COMMIT_KEYS = 1_000_000`) werden ältere Buckets durch LCP (Longest Common Prefix) Extraktion verdichtet. Garantiert **0% False Negatives** bei der Write-Skew-Konflikterkennung.

---

## 3. Property-Test-Ergebnisse

- **`visibility_proptest.rs`**:
  - `prop_mvcc_visibility_matches_reference_10k`: **10.000 Fälle generiert**, 0 Fehlschläge.
  - `prop_mvcc_visibility_explicit_boundary_cases`: Explizite Grenzfall-Prüfungen (`as_of == insert_seq`, `as_of == delete_seq`, `delete_seq == None`), 0 Fehlschläge.
- **Ergänzende Crate-Property-Tests:**
  - `prop_snapshot_registry_min_active`: PASS (0 Fehlschläge)
  - `prop_snapshot_register_unregister_stress`: PASS (0 Fehlschläge)
  - `prop_snapshot_pin_unpin_interleaving`: PASS (0 Fehlschläge)
  - `prop_tx_buffer_isolation`: PASS (0 Fehlschläge)
  - `prop_tx_buffer_stage_drain_stage_lifecycle`: PASS (0 Fehlschläge)
  - `prop_tx_buffer_partial_discard_isolation`: PASS (0 Fehlschläge)
  - `prop_tx_buffer_reap_is_complete`: PASS (0 Fehlschläge)
  - `prop_ssi_coarsening_zero_false_negatives`: PASS (0 Fehlschläge)
  - `prop_ssi_validation_is_100_percent_deterministic`: PASS (0 Fehlschläge)
  - `prop_prune_equivalence_above_watermark`: PASS (0 Fehlschläge)

**Gesamtergebnis der Testsuite:** **75 Tests bestanden, 0 fehlgeschlagen, 0 ignoriert.**

---

## 4. Benchmark-Zahlen

Gemessen auf Criterion v0.5.1 (`mvcc_bench.rs`):

| Benchmark / Funktion | N / Parameter | Zeit (p50) | Konfidenzintervall | Skalierung / Verhalten |
| :--- | :--- | :--- | :--- | :--- |
| `snapshot_registry_min_active` | N = 10 | **1.16 ns** | [1.1619 ns, 1.1640 ns] | Strictly $O(1)$ Lock-free Read |
| `snapshot_registry_min_active` | N = 1,000 | **1.16 ns** | [1.1627 ns, 1.1647 ns] | Strictly $O(1)$ Lock-free Read |
| `snapshot_registry_min_active` | N = 100,000 | **1.16 ns** | [1.1616 ns, 1.1635 ns] | Strictly $O(1)$ Lock-free Read |
| `snapshot_registry_register_drop` | N = 10 | **147.67 ns** | [147.44 ns, 147.90 ns] | Mutex + BTreeMap Insert/Remove |
| `snapshot_registry_register_drop` | N = 1,000 | **183.64 ns** | [178.62 ns, 190.21 ns] | $O(\log N)$ BTreeMap Scaling |
| `snapshot_registry_register_drop` | N = 100,000 | **191.99 ns** | [189.76 ns, 195.02 ns] | $O(\log N)$ Flatline (10,000x N -> 1.3x latency) |
| `seq_log_entry_is_visible_o1` | single entry | **1.37 ns** | [1.3682 ns, 1.3715 ns] | Strictly $O(1)$ Allocation-free |

---

## 5. Major-Befunde & Empfehlungen

1. **[MAJOR] Typ/Runtime-Erzwingung für Multi-Shard Lock Prohibition (`TxBuffer`):**
   - *Problem:* In `tx_buffer.rs` existiert die Invariante „Callers must never acquire more than one shard lock simultaneously". Diese Regel ist lediglich als Freitext-Kommentar dokumentiert (`tx_buffer.rs:8-10, 209-215`). Es gibt weder eine `debug_assert!` noch eine typbasierte Guard, die verhindert, dass ein Entwickler versehentlich zwei Shard-Locks gleichzeitig hält.
   - *Empfehlung:* Einführung einer Thread-Local Guard (`thread_local! static HOLDING_SHARD_LOCK: Cell<bool>`) mit `debug_assert!(!HOLDING_SHARD_LOCK.get())` beim Erwerb jedes Shard-Locks.

2. **[MINOR] Direct Clippy Cast Warnings in `tx_buffer.rs`:**
   - *Problem:* `tx_buffer.rs:270` (`u64` to `usize`) und `tx_buffer.rs:278` triggern Clippy-Warnungen unter den neuen Workspace-Lints (`clippy::cast_possible_truncation`).
   - *Empfehlung:* Umstellung auf explicit `usize::try_from` oder Invariante per Maskierung absichern.

---

## VERDICT & EVIDENCE

VERDICT: PASSED_WITH_FINDINGS

EVIDENCE_COLLECTION:
- P1_VISIBILITY_FORMULA: OK (crates/contextra-mvcc/src/seq_log.rs:41)
- P2_O1_COMPLEXITY: OK (crates/contextra-mvcc/src/seq_log.rs:36-41, bench: 1.37 ns)
- P3_SNAPSHOT_REGISTRY: OK (crates/contextra-mvcc/src/snapshot.rs:33, 87-89, bench: 1.16 ns flatline)
- P4_NO_LOST_UPDATE: OK (crates/contextra-mvcc/src/seq_log.rs:183-189, 198-204)
- P5_DETERMINISM_P28: PASSED_WITH_NOTE (Instant::now() in convenience defaults, 0 SystemTime/thread_rng)
- P6_CONCURRENCY_TEST: OK (crates/contextra-mvcc/tests/loom_snapshot_registry.rs:1-125)
- P7_TX_BUFFER_SHARDING: MAJOR_FINDING (Comment only, no type or debug_assert deadlock guard for multi-shard lock acquisition)
- P8_ORPHAN_REAPER: OK (crates/contextra-mvcc/src/tx_buffer.rs:563-625, snapshot.rs:194)
- PROPERTY_TESTS: OK (10,000 cases in visibility_proptest.rs, 0 failures)
- BENCHMARKS: OK (Criterion benches present and verified)
