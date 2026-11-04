# Concurrency-Architektur & Lock-Hierarchie (SPEC §5.4, §11.3, §11.4, D-57, D-78, D-128)

## 1. Systemübersicht & Design-Prinzipien

Contextra ist eine eingebettete Single-Node-Bibliothek für Agenten- und RAG-Speicher mit belegter, gemessener Löschbarkeit. Die Nebenläufigkeits-Architektur trennt Foreground-Latenzen strikt von blockierenden Hintergrund-I/O-Operationen.

### 1.1 Grundlegende Invarianten
- **Keine Sperre über fremdes `.await`:** `await_holding_lock = deny`. Keine synchronen Mutexe oder Guards dürfen über Yield- bzw. Await-Punkte gehalten werden.
- **Cancel-Safety (SPEC I-14):** Mutierende Operationen laufen in separaten Tasks; der Aufrufer wartet auf ein `oneshot`-Signal. Ein Abbruch des Aufrufer-Futures ändert den Commit-Ausgang nicht.
- **O(1)-Speicher:** Dokumentlocks und Barrieren besitzen $O(1)$ Speicheraufwand unabhängig von der Dokumentenanzahl.
- **Backpressure & begrenzte Worker-Pools:** Blockierende Datei-I/O läuft über begrenzte OS-Worker-Queues (`Critical` vor `Normal` gemäß D-128), nicht auf Tokio-Worker-Threads.

---

## 2. Kanonische Lock-Hierarchie

Gemäß **SPEC §5.4**, **§14.2**, **D-57**, **D-78** und **MV-08** gilt die strikte, lineare Lock-Ordnung:

```text
1. Dokumentlock (gestreift, 1024 Shards via doc_tag Hash)
        │
        ▼
2. Commit-Barriere (CommitBarrier: shared für Commit, exclusive für Gate)
        │
        ▼
3. Purge-Barriere (PurgeBarrier: shared für Kompaktion, exclusive für Purge)
        │
        ▼
4. Journal-Lock (exklusives Append-Log-Lock)
```

### 2.1 Beschreibung der Stufen
1. **Dokumentlock (gestreift, D-128):**
   - 1024 asynchrone Shards (`ahash` über `doc_tag`).
   - Gilt nur für Einzel-Dokument-Mutationen (`insert`, `update`, `soft_delete`, `erase(Document)`).
   - Für Subject-, Collection- oder Tenant-Ziele gibt es kein Dokument-Lockset ($O(1)$ Invariante nach D-57).
   - Verhindert Schreib-/Lösch-Races auf Einzeldokument-Ebene (TOCTOU-Schutz).

2. **Commit-Barriere (`CommitBarrier`, SPEC §11.3, D-57, D-78):**
   - Gruppen-Commit-Leader akquiriert `CommitBarrier::shared`.
   - Gate-Gruppen-Commit (`ErasedSet::add_batch`) akquiriert `CommitBarrier::exclusive`.
   - Schließt das TOCTOU-Fenster zwischen Gate-Aktivierung und nachfolgenden Writes: Jeder Commit prüft nach Erwerb das `ReadGate` für alle Keys des Batches und weist Schreibzugriffe auf gegatete Bereiche mit `Error::Erased` ab.

3. **Purge-Barriere (`PurgeBarrier`, SPEC §11.3, §11.4, D-78, I-16):**
   - Hintergrund-Kompaktion hält `PurgeBarrier::compaction` (`shared`).
   - Physischer Purge hält `PurgeBarrier::purge` (`exclusive`).
   - Kooperativer Abbruch: Die Kompaktion prüft pro Block `purge_waiting()`; ist ein Purge anhängig, bricht sie ab und verwirft Teilergebnisse (keine Blockierung > 1 s).

4. **Journal-Lock (SPEC §11.2, D-58):**
   - Schützt das lineare Löschjournal (`journal.log`) für atomare Statusübergänge (`Requested` -> `KeyDestroyed` -> `Completed`).

---

## 3. RCU & Arc-Swap Semantik für Gate und Konfiguration

- **ReadGate (ER-11, PO-07):**
  - Read-Pfade und Commit-Leader erfordern $O(\log n)$ bzw. $O(1)$ Synchronisationskosten ohne blockierende I/O.
  - Das Gate nutzt RCU-ähnliche Zeiger-Swaps (`Arc<RwLock<ErasedSet>>` bzw. atomare Versionszeiger), sodass Leser ohne Wartezeit den aktuellen Intervallbaum inspizieren können.
  - Nach `CommitBarrier::exclusive` wird der neue Intervallbaum publiziert und ist sofort für alle nachfolgenden Leser und Commits sichtbar.

- **Dynamische Konfiguration:**
  - Konfigurationsupdates werden über atomare Zeigeraustausche (`ArcSwap`/`Arc<T>`) eingespielt, sodass laufende Abfragen auf einer unveränderlichen Momentaufnahme arbeiten.

---

## 4. Loom-Verifikation (T-CONC-1, T-LOCK-2)

Die Modelle in `crates/contextra-mvcc/tests/mv_loom_tests.rs` verifizieren unter allen möglichen Thread-Interleavings:
1. `t_conc_1_loom_seq_allocator_concurrent_monotonic`: Deterministisch strikt monotone Sequenzvergabe ohne Lücken oder Überläufe.
2. `t_lock_loom_lock_order_hierarchy_full_spec_5_4`: Vollständige Einhaltung der 4-stufigen Lock-Hierarchie ohne Zyklen oder Deadlocks.
3. `t_snap_loom_history_horizon_never_exceeds_active_snapshots`: Purge-Barriere und History-Horizon übersteigen niemals die kleinste aktive Snapshot-Sequenz (`min_active_seqno`).
4. `t_lock_loom_commit_vs_purge_barrier`: Mutual Exclusion zwischen Commit- und Purge-Barrieren.
5. `t_conc_loom_write_vs_purge_cluster_race`: Nachweis der wechselseitigen Exklusion und deterministischen Abweisung von Schreibern nach Cluster-Gate/Purge.

---

## 5. Latenzmessung: p50/p99/p999 mit und ohne Purge (BM-WRITE-DURING-ERASE, BM-PURGE-WAIT)

Empirisch erhobene Latenzwerte (`crates/contextra-mvcc/tests/bench_latency_report.rs`):

| Szenario | p50 | p99 | p999 | SLA-Kriterium (SPEC §11.3 & BM-WRITE-DURING-ERASE) |
|---|---|---|---|---|
| **Ruhezustand (ohne Purge)** | 0.17 µs | 0.19 µs | 0.49 µs | Referenzlinie |
| **Unter Purge-Last (BM-WRITE-DURING-ERASE)** | 0.17 µs | 0.19 µs | 0.70 µs | $p99_{\text{purge}} / p99_{\text{quiet}} = 1.00\times \le 3.0\times$ (Bestanden) |
