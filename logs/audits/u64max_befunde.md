# Audit-Bericht: `u64::MAX` Befunde (`crates/`)

## Übersicht
Gemäß Teil A des Audit-Auftrags wurden alle Vorkommen von `u64::MAX` in `crates/` (exklusive Integrationstests und Benches) vollständig analysiert und in drei Kategorien klassifiziert:
- **(a) Lesezeitpunkt eines Snapshots**: Aufrufe von Speicherlesefunktionen mit `u64::MAX` als Sequenznummer / Snapshot-Zeitpunkt.
- **(b) Sentinel / Grenzwert**: Nutzung von `u64::MAX` als Initialisierungswert, Maximalwert, Maske, Overflow-Schranke oder Unbounded-Limit.
- **(c) Kommentar / Dokumentation**: `u64::MAX` in Doc-Comments oder Fehlermeldungs-Strings.

---

## 1. Kategorie (a): Snapshot-Lesezeitpunkt

| Datei | Zeile | Code-Ausschnitt | Klassifikation | Beschreibung & Aktion |
|---|---|---|---|---|
| `crates/contextra-engine/src/collection/crud/links.rs` | 65 | `.get_at_seq_tracked(tx, &doc_key, u64::MAX)` | **(a)** | **Ersetzt:** Aufruf im Rahmen der Transaktion `tx` (`link_memories`). `u64::MAX` wurde durch den transaktionalen Snapshot-Zeitpunkt `tx.inner()` ersetzt. |
| `crates/contextra-engine/src/collection/crud/links.rs` | 139 | `.get_at_seq_tracked(tx, &user_key, u64::MAX)` | **(a)** | **Ersetzt:** Aufruf im Rahmen der Transaktion `tx` (`link_memories`). `u64::MAX` wurde durch den transaktionalen Snapshot-Zeitpunkt `tx.inner()` ersetzt. |
| `crates/contextra-engine/src/collection/crud/links.rs` | 191 | `self.storage.get_at_seq(&doc_key, u64::MAX)` | **(a)** | **Dokumentiert (Unverändert):** `get_links(doc_id)` hat keinen Transaktions- oder `ReadSnapshot`-Parameter. Das Lesen von Link-Kanten erfolgt als strukturelle Metadaten-Abfrage stets auf dem neuesten Stand. |
| `crates/contextra-store/src/sstable/reader.rs` | 914 | `self.get_at(key, u64::MAX, u64::MAX)` | **(a)** | **Dokumentiert (Unverändert):** Convenience-Funktion `SstableReader::get(&self, key)` ohne Sequenznummern-Einschränkung (liest die neueste Version). |
| `crates/contextra-text/src/wand.rs` | 293 | `storage.scan_prefix_at(&prefix, u64::MAX)` | **(a)** | **Dokumentiert (Unverändert):** On-demand Posting-List-Laden im WAND-Scorer ohne expliziten Transaktionssnapshot (liest alle aktiven Schlüssel). |
| `crates/contextra-graph/src/cascade.rs` | 626 | `cascade_invalidate_edges_for_superseded_doc(&graph, doc_a, u64::MAX)` | **(a)** | **Dokumentiert (Unverändert):** Kaskadierende Kanteninvalidation beim Ersetzen eines Dokuments; verwendet `u64::MAX` als unbounded validity horizon. |

---

## 2. Kategorie (b): Sentinel / Grenzwert für andere Zwecke

| Datei | Zeile | Code-Ausschnitt | Klassifikation | Beschreibung |
|---|---|---|---|---|
| `crates/contextra-types/src/types/domain/ids.rs` | 168 | `pub const MAX: Self = Self(u64::MAX);` | **(b)** | `DocId::MAX` Konstante. |
| `crates/contextra-types/src/types/domain/ids.rs` | 409 | `pub const INTERNAL_BASE: u64 = u64::MAX - 1_000_000;` | **(b)** | Basis-TxId für interne Systemtransaktionen (`[INTERNAL_BASE, u64::MAX]`). |
| `crates/contextra-types/src/types/domain/ids.rs` | 480..513 | `INTERNAL_BASE + offset ... overflows u64::MAX` | **(b)** | Bereichs- und Überlaufprüfungen für `TxId`. |
| `crates/contextra-types/src/types/budget.rs` | 512, 513, 518 | `tracker.consume_memory(u64::MAX)` | **(b)** | Memory-Tracker Overflow-Test / Guard. |
| `crates/contextra-mvcc/src/snapshot.rs` | 77, 338 | `AtomicU64::new(u64::MAX)` / `unwrap_or(u64::MAX)` | **(b)** | Initialwert für die minimale aktive Sequenznummer (`min_active_seqno`), wenn keine aktiven Leases vorhanden sind. |
| `crates/contextra-mvcc/src/snapshot.rs` | 121, 142 | `as_nanos().try_into().unwrap_or(u64::MAX)` | **(b)** | Sättigende Konvertierung von Zeitdauern (Ticks). |
| `crates/contextra-mvcc/src/tx_buffer.rs` | 232 | `as_nanos().try_into().unwrap_or(u64::MAX)` | **(b)** | Timeout-Sättigung. |
| `crates/contextra-mvcc/src/seq_log.rs` | 127 | `as_nanos().try_into().unwrap_or(u64::MAX)` | **(b)** | Pinning-Dauer-Sättigung. |
| `crates/contextra-mvcc/src/floor.rs` | 62 | `unwrap_or(u64::MAX)` | **(b)** | GC Floor Berechnungs-Fallback. |
| `crates/contextra-store/src/lsm/gc_floor.rs` | 36 | `unwrap_or(u64::MAX)` | **(b)** | Fallback für minimalen Snapshot. |
| `crates/contextra-store/src/compaction/retention.rs` | 22, 62 | `if floor_seq == u64::MAX` | **(b)** | retention floor check in Compaction. |
| `crates/contextra-store/src/compaction/engine.rs` | 678, 779 | `unwrap_or(u64::MAX)` | **(b)** | Snapshot Floor Boundary in Compaction Engine. |
| `crates/contextra-store/src/compaction/adaptive.rs` | 285, 333 | `let mut best_tier_size = u64::MAX;` | **(b)** | Min-Findung Initialisierungswert. |
| `crates/contextra-store/src/wal/encode.rs` | 136 | `if seq_no == u64::MAX` | **(b)** | WAL Sequenznummern-Überlaufschutz (I-4). |
| `crates/contextra-store/src/wal/flusher.rs` | 480 | `unwrap_or(u64::MAX)` | **(b)** | Sättigende Konvertierung der geschriebenen WAL-Länge. |
| `crates/contextra-store/src/wal/io.rs` | 564 | `fail_tx == u64::MAX` | **(b)** | Fault-Injection Trigger-Sentinel. |
| `crates/contextra-store/src/sstable/builder.rs` | 268, 270 | `min_tx_id: u64::MAX`, `min_seq: u64::MAX` | **(b)** | Sentinel-Initialisierung vor SSTable Block-Akkumulation. |
| `crates/contextra-store/src/sstable/builder.rs` | 387 | `"Block size exceeds u64::MAX"` | **(b)** | Fehlerursachen-Text bei theoretischem Blockgrößen-Überlauf. |
| `crates/contextra-crypto/src/crypto.rs` | 514, 756 | `if val == u64::MAX` / `nonce_counter.store(u64::MAX)` | **(b)** | Nonce-Counter Sättigungs- und Exhaustion-Schutz. |
| `crates/contextra-sys/src/determinism.rs` | 38, 51, 170 | `unwrap_or(u64::MAX)` | **(b)** | Sättigende Zeitstempel-Konvertierung in `SystemClock`. |
| `crates/contextra-graph/src/community.rs` | 332 | `let u64_id = u64::MAX - ...` | **(b)** | Synthetische ID-Generierung für Hierarchie-Ebenen im Leiden Community Detection. |
| `crates/contextra-graph/src/csr/graph_index.rs` | 143, 176, 660, 781 | `INTERNAL_BASE-Bereich [u64::MAX - 1_000_000]` | **(b)** | Invariante-Prüfungs-Meldung für System-TxIds. |
| `crates/contextra-adapt/src/decay_controller.rs` | 186 | `u64::MAX / 2` | **(b)** | Zeitdifferenz-Berechnung für Halbwertszeit-Decay. |
| `crates/contextra-text/src/wand.rs` | 245 | `max_seq == Some(u64::MAX)` | **(b)** | Prüfflag ob unbeschränkte Sequenznummer vorliegt. |
| `crates/contextra-vector/src/hnsw/core_rebuild.rs` | 536 | `unwrap_or(u64::MAX)` | **(b)** | Shift-Multiplier Schutz vor Integer-Überlauf. |
| `crates/contextra-engine/src/export.rs` | 144 | `contextra_types::TxId::new(u64::MAX)` | **(b)** | System-TxId Sentinel für Re-Export Operationen. |
| `crates/contextra-engine/src/transaction/compensating_actions.rs` | 204..207 | `DocId::new(u64::MAX as _)` | **(b)** | DocId Boundary Schutz. |
| `crates/contextra-engine/src/transaction/db_transaction_rollback.rs` | 280 | `(u64::MAX as u128) + 100` | **(b)** | Rollback Boundary / Overflow Test. |
| `crates/contextra-engine/src/background_workers/config.rs` | 38 | `unwrap_or(u64::MAX)` | **(b)** | Exponential Backoff Sättigung. |
| `crates/contextra-cognition/src/memory_consolidation.rs` | 675 | `(chunk_val as f32) / (u64::MAX as f32)` | **(b)** | Floating-Point Normalisierung von Integern. |

---

## 3. Kategorie (c): Kommentare / Dokumentation

| Datei | Zeile | Code-Ausschnitt | Klassifikation | Beschreibung |
|---|---|---|---|---|
| `crates/contextra-engine/src/collection/crud/read.rs` | 19 | `// instead of silently mapping them to u64::MAX (ID: AGT-DB-001).` | **(c)** | Kommentar bezüglich Fehlerbehandlung bei ungültigen Snapshots. |
| `crates/contextra-store/src/lsm/ops/read.rs` | 25, 28 | `/// INV-MVCC-SSI-1: A caller passing snapshot_seq = u64::MAX ...` | **(c)** | Doc-Comment zur Invariante INV-MVCC-SSI-1. |
| `crates/contextra-store/src/compaction/retention.rs` | 54 | `// Versions: 30, 20, 10, 5 for floor = u64::MAX` | **(c)** | Beispielkommentar für Compaction Retention. |
| `crates/contextra-store/src/wal/encode.rs` | 7 | `//! - I-4: Monotone Sequenznummern: seq_no == u64::MAX wird als Überlauf abgelehnt.` | **(c)** | WAL-Moduldokumentation. |
