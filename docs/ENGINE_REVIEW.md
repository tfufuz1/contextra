# contextra-engine – Technischer Review-Report

> **Datum:** 2026-11-02  
> **Crate:** `contextra-engine` (Ring 3)  
> **Reviewer-Rolle:** Datenbank-Kernel-Ingenieur (Orchestrierung, Hintergrund-Worker, Lebenszyklen)  
> **Basis:** Vollständige Quell-Lektüre aller Produktionsdateien in `src/`

---

## 1. Executive Summary

Die `contextra-engine` orchestriert LSM-Store, HNSW-Vektor-Index, BM25-Text-Index und (nominell) Graph-Index in einem zweiphasigen Commit-Protokoll. Die Grundstruktur ist solide, aber es gibt **drei kritische Korrektheits-Defekte**, mehrere mittelschwere Konsistenzprobleme und ein Bündel Optimierungslücken, die unter Last sichtbar werden.

| Schwere | Anzahl | Beschreibung |
|---------|--------|-------------|
| 🔴 Kritisch | 3 | Korrektheitsfehler; Datenverlust oder Gate-Daten-Leak möglich |
| 🟠 Hoch | 6 | Konsistenz-/Sicherheitsprobleme unter Concurrent-Last |
| 🟡 Mittel | 8 | Architektur- und API-Lücken |
| 🟢 Gering | 5 | Optimierungspotenzial |

---

## 2. Kritische Defekte (🔴)

### DEF-01 — Delete-Lock-Race: KV-Lock wird sofort nach Erwerb freigegeben

**Dateien:**  
- [`delete.rs:27-28`](file:///home/freddy/Projekte/contextra/crates/contextra-engine/src/collection/crud/delete.rs#L27-L28)  
- [`update.rs:32-33`](file:///home/freddy/Projekte/contextra/crates/contextra-engine/src/collection/crud/update.rs#L32-L33)  
- [`kv.rs:18-19`](file:///home/freddy/Projekte/contextra/crates/contextra-engine/src/collection/crud/kv.rs#L18-L19)  
- [`maintenance.rs:706-707`](file:///home/freddy/Projekte/contextra/crates/contextra-engine/src/collection/maintenance.rs#L706-L707)

**Befund:**

```rust
// delete.rs
let _guard = self.kv_locks.lock_for(id).await;
drop(_guard);                // ← Lock bereits freigegeben!
let mut db_tx = self.begin_transaction()?;
// ... delete_op läuft OHNE Lock
```

Das ist der bekannte Fehler aus der Aufgabenbeschreibung. Der Lock wird erworben und sofort mit `drop(_guard)` explizit freigegeben, bevor `begin_transaction()` und `delete_op()` laufen. Dadurch ist der gesamte Löschpfad ungeschützt. Ein konkurrierender `insert` oder `update` kann zwischen Lock-Freigabe und tatsächlichem Löschen auf dieselbe ID schreiben — der Löschvorgang überschreibt dann Daten, die nach seiner „Beobachtungs-Sequenz" entstanden sind.

Dasselbe Muster tritt in `update()`, `put_kv()`, `put_kv_if_absent()` und `update_document_importance()` auf.

**Korrekte Lösung:**

```rust
// Der Guard muss die gesamte Operation überspannen
let _guard = self.kv_locks.lock_for(id).await;
let mut db_tx = self.begin_transaction()?;
match self.delete_op(&mut db_tx, id).await {
    Ok(_) => { db_tx.commit().await?; Ok(()) }
    Err(e) => { db_tx.rollback().await.ok(); Err(e) }
}
// Guard wird hier durch Drop freigegeben
```

**Auswirkung:** TOCTOU-Race, Datenverlust unter SSI-Last, Verletzung der Invariante `kv_locks schützt Mutationen`.

---

### DEF-02 — `DbTransaction::Drop` spawnt Fire-and-Forget ohne Garantie

**Datei:** [`db_transaction.rs:150-184`](file:///home/freddy/Projekte/contextra/crates/contextra-engine/src/transaction/db_transaction.rs#L150-L184)

**Befund:**

```rust
impl<S, V> Drop for DbTransaction<S, V> {
    fn drop(&mut self) {
        if !self.committed.load(Ordering::Acquire) {
            // ...
            if let Ok(handle) = tokio::runtime::Handle::try_current() {
                handle.spawn(async move {      // ← Fire-and-Forget, unkontrolliert
                    collection.graph_index.rollback(tx_id).await;
                    collection.text_index.rollback(tx_id).await;
                    collection.index.rollback(tx_id).await;
                    collection.storage.rollback(tx_id).await;
                });
            }
            // Falls kein Runtime-Handle: kein Rollback! Stilles Silentfailure.
        }
    }
}
```

Drei Probleme:
1. **Kein Runtime-Handle → kein Rollback:** Falls die `DbTransaction` außerhalb einer Tokio-Runtime gedroppt wird (z. B. in einem `spawn_blocking`-Thread oder beim Unit-Test), passiert absolut nichts. Kein Rollback, kein Log.
2. **Reihenfolge falsch:** Der Drop-Rollback rollt `graph_index` zuerst zurück, während der saubere `rollback()`-Pfad die umgekehrte Reihenfolge (Graph → Text → Vector → Storage) einhält. Beim Drop ist die Reihenfolge aber Storage-seitig nicht garantiert, weil alle vier Futures nicht sequenziell sind.
3. **Keine Verbindung zum `task_tracker`:** Beim Shutdown kann das System schon heruntergefahren sein, bevor der Drop-Rollback-Task abschließt.

**Korrekte Lösung:** Die Drop-Implementierung sollte statt Fire-and-Forget einen synchronen, best-effort Rollback über die Storage-Schicht auslösen (kein Await im Drop) und alle Fehler ins Tracing schreiben. Die Async-Rollbacks für Text- und Vector-Index müssen im Normalfall über den expliziten `rollback()`-Pfad erfolgen.

---

### DEF-03 — Hardcodierte Tenant-ID im Commit-Pfad bricht Multi-Tenant-Isolation

**Datei:** [`db_transaction_commit.rs:334`](file:///home/freddy/Projekte/contextra/crates/contextra-engine/src/transaction/db_transaction_commit.rs#L334)

**Befund:**

```rust
let tenant_id = TenantId::try_new(1).unwrap_or_default();
ledger.push(CompensateHnswAction::new(
    self.collection.clone(),
    tenant_id,    // ← IMMER Tenant 1, unabhängig vom echten Tenant
    Arc::clone(&doc_ids),
));
```

`CompensateHnswAction` erhält stets `TenantId(1)`, egal welcher Mandant die Transaktion ausführt. Wenn die HNSW-Kompensation Tenant-Isolation erzwingt (Log-Kontext, Metrics-Labels), werden alle Rollback-Aktionen fälschlich Tenant 1 zugeschrieben. In einer Umgebung mit `TenantPolicy::Required` kann das zu falschen Audit-Einträgen oder, falls die Kompensation tatsächlich mandantenabhängig filtert, zu fehlgeschlagenen Kompensationen für Tenant ≠ 1 führen.

**Korrekte Lösung:** Die `TenantId` muss aus dem `collection`-Kontext kommen (z. B. als Feld in `Collection` oder als Parameter in `begin_transaction`).

---

## 3. Hohe Priorität (🟠)

### HIGH-01 — 2PC-Recovery kennt `stages_completed` nicht, bricht Forward-Recovery ab

**Dateien:**  
- [`cleanup.rs`](file:///home/freddy/Projekte/contextra/crates/contextra-engine/src/transaction/cleanup.rs)  
- [`maintenance.rs:220-244`](file:///home/freddy/Projekte/contextra/crates/contextra-engine/src/collection/maintenance.rs#L220-L244)  
- [`intent.rs`](file:///home/freddy/Projekte/contextra/crates/contextra-engine/src/transaction/intent.rs)

**Befund:**

`CommitIntent::Pending` enthält `stages_completed: u8` (0 = Intent geschrieben, 1 = LSM committed). Die Recovery-Logik in `cleanup.rs::recover_pending_intents()` und in `maintenance.rs::recover_pending_transaction_intents()` wertet `stages_completed` **nicht aus**. Beim Forward-Recovery wird immer angenommen, dass entweder alles neu gespielt werden muss oder dass der LSM-Zustand alles entscheidet. 

Konkret:
- Ein Intent mit `stages_completed = 1` (LSM committed, Indizes noch nicht) wird korrekt als "committed" erkannt (weil Daten in LSM vorhanden), aber die Text- und Vektor-Indizes werden **nicht zwangsrekonstruiert** — sie werden nur durch den Fallback-Scan in `fallback_scan_missing_text_index_documents()` repariert, der ausschließlich Text-Index prüft, **nicht HNSW**.
- Ein Intent mit `stages_completed = 0` (LSM commit noch nicht) wird als "aborted" markiert, aber der Vektor-Index könnte bereits partiell committed sein (HNSW hat kein transaktionales Rollback auf Dateisystem-Ebene).

**Risiko:** Nach einem Crash zwischen Phase (b) LSM-Commit und Phase (c) Index-Commit bleiben HNSW-Einträge ohne LSM-Gegenstück (Geisterknoten).

---

### HIGH-02 — `collect::maintenance::repair_internal` hält `consolidation_guard`, ruft dann async `storage.scan_prefix` auf

**Datei:** [`maintenance.rs:169-204`](file:///home/freddy/Projekte/contextra/crates/contextra-engine/src/collection/maintenance.rs#L169-L204)

**Befund:**

```rust
async fn repair_internal(&self) -> Result<()> {
    let _guard = self.consolidation_guard.lock().await;  // tokio::Mutex
    let docs = self.storage.scan_prefix(&self.prefix).await?;  // Kann lange dauern
    // ...
    // Weitere awaits mit gehaltenem Lock
```

`consolidation_guard` ist ein `tokio::sync::Mutex`. Clippy `await_holding_lock` warnt genau vor diesem Pattern: Eine `tokio::sync::Mutex` über `.await` zu halten ist zulässig (anders als `parking_lot::Mutex`), aber konzeptuell problematisch: Während `scan_prefix` wartet, blockiert keine CPU, aber der `consolidation_guard` hält externe Schreibvorgänge fern. Ein langer Storage-Scan bei großen Collections (100k+ Docs) kann Repair-Latenz in den Minuten-Bereich treiben. Kein Timeout ist implementiert.

---

### HIGH-03 — `link_memories` hat kein vollständiges transaktionales Staging

**Datei:** [`links.rs:35-92`](file:///home/freddy/Projekte/contextra/crates/contextra-engine/src/collection/crud/links.rs#L35-L92)

**Befund:**

`link_memories()` allokiert eine TX (`allocate_tx()`), führt Storage-Writes direkt aus und committet. Es nutzt **kein `DbTransaction`-Objekt** und damit kein 2PC, kein CommitIntent-WAL, keinen Ledger für Kompensation. Außerdem: Der `link_memories`-Pfad schreibt in beide Schlüssel (`doc_key` und `user_key`) ohne sie zu `stage_text_insert` oder `stage_graph_edge` hinzuzufügen. Links sind damit nicht in der Text-Index-Reparatur bei Recovery berücksichtigt.

Bei einem Crash nach dem `storage.commit(tx)` in `update_metadata_links()`, aber vor dem `storage.commit(tx)` in der äußeren `link_memories`-Funktion (bei getrennten Schreibvorgängen) entstehen inkonsistente Link-Zustände.

---

### HIGH-04 — `evict_decayed_chunks` und `reap_expired_documents` sammeln IDs in einem Vec, löschen danach — kein Gate gegen paralleles Lesen

**Datei:** [`maintenance.rs:46-143`](file:///home/freddy/Projekte/contextra/crates/contextra-engine/src/collection/maintenance.rs#L46-L143), [`maintenance.rs:419-510`](file:///home/freddy/Projekte/contextra/crates/contextra-engine/src/collection/maintenance.rs#L419-L510)

**Befund:**

Der Ablauf ist:
1. Alle expired/decayed IDs in `Vec<String>` sammeln (Scan über gesamte Collection).
2. Danach sequenziell `self.delete(id)` für jede ID aufrufen.

Zwischen Schritt 1 (Entscheidung „soll gelöscht werden") und Schritt 2 (Löschen) liegt Zeit. Ein laufendes Lese-Query kann noch Suchergebnisse für diese Dokumente liefern. Das entspricht zwar dem MVCC-Modell, aber: Falls ein Dokument zwischen Scan und Delete geupdated wird (Importance-Score-Update), wird es trotzdem gelöscht. Die TTL/Decay-Entscheidung basiert auf einem veralteten Snapshot, der nicht als MVCC-Snapshot abgesichert ist.

**Optimum:** Die gesamte Scan-to-Delete-Sequenz sollte innerhalb eines einzelnen MVCC-Snapshots (`snapshot_seq()`) arbeiten, sodass alle Löschentscheidungen auf demselben konsistenten Zustand basieren.

---

### HIGH-05 — `check_doc_id_collision` (nicht-tracked) vs. `check_doc_id_collision_tracked` (tracked) — TOCTOU zwischen Kollisionsprüfung und `put_if_absent`

**Datei:** [`insert.rs:261-363`](file:///home/freddy/Projekte/contextra/crates/contextra-engine/src/collection/crud/insert.rs#L261-L363)

**Befund:**

```rust
// In insert_op():
self.check_doc_id_collision_tracked(tx, doc_id, id).await?;  // MVCC-tracked Read
// ...
let written = self.storage.put_if_absent(tx, &doc_key, &meta_data).await?;
if !written {
    self.check_doc_id_collision(doc_id, id).await?;  // Nicht-tracked, kein SSI
}
```

Der Fallback `check_doc_id_collision()` (Zeile 362) liest ohne TX-Tracking. Falls zwei konkurrierende Inserts mit unterschiedlichen User-Keys denselben `DocId`-Hash produzieren, können beide die erste Prüfung bestehen, beide `put_if_absent` scheitern, und beide dann die nicht-getracked-e Kollisionsprüfung machen — zu einem Zeitpunkt, an dem keiner von beiden den Lock auf den anderen hält.

---

### HIGH-06 — `collect_for_tenant()` hat ein TOCTOU-Fenster zwischen Read-Lock-Freigabe und Write-Lock-Erwerb

**Datei:** [`collections.rs:52-115`](file:///home/freddy/Projekte/contextra/crates/contextra-engine/src/contextra_impl/collections.rs#L52-L115)

**Befund:**

```rust
let read_guard = self.tenant_collections.read().await;
if let Some(col) = read_guard.get(&cache_key) {
    return Ok(Arc::clone(col));
}
drop(read_guard);

// ← Fenster: Zwei Tasks könnten beide hier ankommen

let mut write_guard = self.tenant_collections.write().await;
if let Some(existing) = write_guard.get(&cache_key) {  // Double-check
    return Ok(Arc::clone(existing));
}
// Erstellt Collection, startet Worker...
```

Das double-checked Locking ist korrekt implementiert (Write-Lock, nochmalige Prüfung). **Aber:** `load_index()`, `load_text_stats()` und `migrate_doc_keys_v1()` laufen **innerhalb des Write-Locks**! Diese Operationen können bei großen Collections lange dauern (Sekunden bis Minuten bei Replay). Während dieser Zeit blockiert der Write-Lock jeden anderen `collection()`-Aufruf — auch für vollständig andere Collection-Namen.

**Korrekte Lösung:** Collection erstellen und initialisieren außerhalb des Write-Locks, dann nur den fertigen Arc eintragen.

---

## 4. Mittlere Priorität (🟡)

### MED-01 — Graph-Index-Reste sind toter Code, aber im Commit-Pfad aktiv

**Dateien:**  
- [`db_transaction_commit.rs:56-128`](file:///home/freddy/Projekte/contextra/crates/contextra-engine/src/transaction/db_transaction_commit.rs#L56-L128)  
- [`db_transaction.rs:26-30`](file:///home/freddy/Projekte/contextra/crates/contextra-engine/src/transaction/db_transaction.rs#L26-L30)

`Collection` (Ring 3) hat kein `graph_index`-Feld (nicht in `mod.rs`). `DbTransaction` referenziert `collection.graph_index` und `collection.graph_index.commit/rollback` — das Projekt baut nur weil ein `graph_index`-Feld an anderer Stelle existiert (vermutlich in Test-Fixtures oder einer separaten Collection-Variante). `staged_graph_entities`, `staged_graph_edges`, `staged_graph_entity_deletes`, `staged_graph_edge_deletes`, `staged_hyperedges` — **5 von 10 Staging-Vektoren** in `DbTransaction` sind Graph-bezogen und werden in der Produktion nie befüllt (weil `Collection.graph_index` ein No-Op ist).

**Empfehlung gemäß Aufgabenbeschreibung:** Graph-Reste vollständig entfernen:
- `staged_graph_*` aus `DbTransaction`
- `commit_graph_staged()` aus `db_transaction_commit.rs`
- `compensating_actions.rs::CompensateGraphAction`
- `staged_hyperedges` (leer, `hyperedge_worker.rs` ist ein leeres Stub-Modul)

---

### MED-02 — `apply_read_gate` ist ein No-Op, aber kritische Sicherheitsabsicht

**Datei:** [`hydrate.rs:21-24`](file:///home/freddy/Projekte/contextra/crates/contextra-engine/src/collection/search/hydrate.rs#L21-L24)

```rust
pub fn apply_read_gate(hits: Vec<crate::SearchResult>) -> Vec<crate::SearchResult> {
    // In Stufe 1: No-Op Durchleitung
    hits
}
```

`apply_read_gate` wird an **4 Stellen** im Hybrid-Such-Pfad aufgerufen und soll laut SPEC §8.12 und D-62 erased Dokumente aus Suchergebnissen filtern. Als No-Op ist das ReadGate sicher (keine Datenverluste), aber **gelöschte Dokumente (soft-deleted via Erasure) können in Suchergebnissen erscheinen**, bis der HNSW-Rebuild sie aus dem Index entfernt. Das ist ein Gate-Daten-Leak.

**Anforderung:** Sobald WP für das Erasure-Pipeline verfügbar ist, muss `apply_read_gate` gegen den `ErasedSet`-Port implementiert werden. Bis dahin ist die Funktion korrekt dokumentiert, aber die Lücke muss im Backlog priorisiert bleiben.

---

### MED-03 — `compensate_lsm` in `db_transaction_rollback.rs` ist duplizierter Code

**Dateien:**  
- [`db_transaction_rollback.rs:67-179`](file:///home/freddy/Projekte/contextra/crates/contextra-engine/src/transaction/db_transaction_rollback.rs#L67-L179)  
- [`compensating_actions.rs:81-193`](file:///home/freddy/Projekte/contextra/crates/contextra-engine/src/transaction/compensating_actions.rs#L81-L193)

`compensate_lsm` in `db_transaction_rollback.rs` und `CompensateLsmAction::execute()` in `compensating_actions.rs` sind nahezu identisch (Retry-Loop, Key-Restore, Aborted-Intent-Schreiben). Beide müssen bei Änderungen synchron gehalten werden.

---

### MED-04 — `scan_prefix` in `read.rs`: Limit-Prüfung nach BATCH passiert zu spät

**Datei:** [`read.rs:140-148`](file:///home/freddy/Projekte/contextra/crates/contextra-engine/src/collection/crud/read.rs#L140-L148)

```rust
if results.len() >= effective_limit {
    return Err(ContextraError::LimitExceeded { ... });
}
results.push((user_key, val));
```

Wenn `effective_limit == 10_000` und der letzte Batch genau 1001 Einträge hat, werden alle 1001 deserialisiert und dann auf das 10.000-ste gestoßen → Error. Effizienter: `break` wenn `results.len() == effective_limit` (kein Error-Return, sondern ordentliche Pagination) oder Prüfung vor dem Einfügen mit Truncation anstatt Error.

---

### MED-05 — Text-Existenz-Check in Recovery via BM25-Suche ist unzuverlässig

**Datei:** [`maintenance.rs:298-312`](file:///home/freddy/Projekte/contextra/crates/contextra-engine/src/collection/maintenance.rs#L298-L312)

```rust
let text_exists = match self.text_index.search_bm25(&text, 1, None).await {
    Ok(hits) => hits.iter().any(|(id, _)| *id == doc_id),
    Err(_) => false,
};
```

BM25-Suche mit `k=1` für exakten Text kann falsche Negative produzieren: Wenn der Text des gesuchten Dokuments nicht der am besten rankende BM25-Treffer ist (bei großen Collections wahrscheinlich), wird das Dokument als fehlend erkannt und erneut indiziert — doppelt indexiert.

**Korrekte Lösung:** Direkte Existenzprüfung über einen Text-Index-Port-Methode `contains(doc_id)` statt über eine BM25-Suche.

---

### MED-06 — `next_tx` wird von allen Collections und der Engine-Facade geteilt

**Dateien:**  
- [`lib.rs`](file:///home/freddy/Projekte/contextra/crates/contextra-engine/src/lib.rs) (`Arc<AtomicU64>`)  
- [`tx.rs:27`](file:///home/freddy/Projekte/contextra/crates/contextra-engine/src/collection/tx.rs#L27)

Alle Collections (und die Engine selbst für Intent-Transaktionen) teilen einen einzigen `Arc<AtomicU64>`-Zähler. Das bedeutet:
- TX-IDs aus verschiedenen Collections sind global monoton — korrekt für globale Ordnung.
- **Aber:** `TxId::MAX_COLLECTION_SEQUENCE` ist ein Collection-bezogenes Limit, das die Engine-Facade und alle Collections gemeinsam erschöpfen. Ein intensiver Batch-Import kann die TX-IDs anderer Collections beschleunigt erschöpfen.
- Die `allocate_tx`-Methode in `Collection` prüft `> TxId::MAX_COLLECTION_SEQUENCE`, aber diese Konstante gilt eigentlich pro Collection, nicht global.

---

### MED-07 — `drop_collection` löscht HNSW-Index nicht aus dem Speicher

**Datei:** [`maintenance.rs:716-738`](file:///home/freddy/Projekte/contextra/crates/contextra-engine/src/collection/maintenance.rs#L716-L738)

`drop_collection()` löscht alle LSM-Daten und Text-Index-Namespace, aber der HNSW-Index (`self.index`) bleibt im Speicher (`Arc<V>` wird nicht invalidiert). Da die Collection in `self.collections`-Cache verbleibt (bis evtl. manuell entfernt), kann der alte Arc noch referenziert werden. Der HNSW-Index ist rein in-memory und enthält nach dem LSM-Drop noch alle alten Vektoren.

---

### MED-08 — `orphan_workers.rs` bricht SPEC-Invariante: Verwendet `SystemClock` direkt statt Port `Clock`

**Datei:** [`orphan_workers.rs:41`](file:///home/freddy/Projekte/contextra/crates/contextra-engine/src/background_workers/orphan_workers.rs#L41)

```rust
let clock = contextra_ports::SystemClock::new();  // ← Direktinstanziierung, kein Port-Inject
```

Laut SPEC P7 muss Zeit ausschließlich über den Port `Clock` injiziert werden. `start_orphan_cleanup_worker_with_config` instanziiert `SystemClock` direkt, was deterministische Tests und Zeitreisen verhindert. Der Clock-Port muss als Parameter übergeben werden.

---

## 5. Optimierungspotenzial (🟢)

### OPT-01 — `reap_expired_documents` und `trigger_expiry_cleanup` — Codeduplizierung: Zwei fast-identische Scan-Loops

**Dateien:**  
- [`maintenance.rs:419-509`](file:///home/freddy/Projekte/contextra/crates/contextra-engine/src/collection/maintenance.rs#L419-L509)  
- [`maintenance.rs:517-636`](file:///home/freddy/Projekte/contextra/crates/contextra-engine/src/collection/maintenance.rs#L517-L636)

Beide Methoden implementieren denselben paginierten Scan-Loop (cursor-basiert, `BATCH_SIZE = 1000`, `String::from_utf8_lossy`, prefix-stripping, `__`-Filter). Der Code sollte in eine gemeinsame `scan_user_keys_paged()`-Methode extrahiert werden.

---

### OPT-02 — Mutex vs. RwLock in `DbTransaction` — unnötige Exklusivität

**Datei:** [`db_transaction.rs:21-31`](file:///home/freddy/Projekte/contextra/crates/contextra-engine/src/transaction/db_transaction.rs#L21-L31)

`DbTransaction` hat 10 `std::sync::Mutex<Vec<...>>`-Felder. Diese sind theoretisch korrekt (Staging-Vektoren werden nur durch Push-Operationen modifiziert), aber:
- Alle Staging-Vektoren werden ausschließlich vom Owner-Task der Transaktion befüllt — ein `Mutex` ist unnötig, da niemals concurrent auf dieselbe `DbTransaction` zugegriffen wird.
- `Arc<Mutex<Vec<DocId>>>` für `staged_doc_ids` + `Arc::make_mut()` ist unnötig komplex.

**Empfehlung:** Da `DbTransaction` nicht `Clone` ist (sondern die zugehörige `Collection` wird geklont), sollten die Staging-Felder einfache `Vec<T>` sein. Der `Arc<Vec<DocId>>` für `staged_doc_ids` sollte durch ein normales `Vec<DocId>` ersetzt werden.

---

### OPT-03 — `insert_inner_unlocked` prüft Embedding-Dimension doppelt

**Datei:** [`insert.rs:196-258`](file:///home/freddy/Projekte/contextra/crates/contextra-engine/src/collection/crud/insert.rs#L196-L258)

`insert()` prüft `embedding.len() != self.dimension` (Z. 201-207), dann `insert_inner_unlocked()` prüft erneut (Z. 231-237), dann `insert_op()` ruft `validate_embedding()` nochmals auf. Dreifache Dimension-Prüfung und zweifache `validate_embedding()`.

---

### OPT-04 — `KvKeyLocks` mit 16 Shards: Hohe Kollisionswahrscheinlichkeit bei parallelen Batch-Ops

**Datei:** [`kv_lock.rs:10`](file:///home/freddy/Projekte/contextra/crates/contextra-engine/src/collection/kv_lock.rs#L10)

`KV_LOCK_SHARDS = 16` bedeutet bei 1000 parallelen Documents im `insert_many()` ≈ 62 Docs pro Shard. Das reduziert Parallelität erheblich. Bei modernen Systemen mit vielen Cores wäre `KV_LOCK_SHARDS = 256` oder `512` sinnvoll, mit `#[repr(align(64))]` auf dem Mutex, um False-Sharing zu vermeiden.

---

### OPT-05 — `compute_default_importance` allokiert `HashSet` bei jedem Insert

**Datei:** [`mod.rs:129`](file:///home/freddy/Projekte/contextra/crates/contextra-engine/src/collection/mod.rs#L129)

```rust
let unique_chars = text.chars().collect::<std::collections::HashSet<_>>().len() as f32;
```

Für jeden Insert mit Text wird ein `HashSet<char>` allokiert. Für kurze Texte (die häufig sind) ist das Overhead. Alternative: Zeichen-Zählung über ein `[bool; 128]` für ASCII oder ein Bit-Array.

---

## 6. Architektur-Bewertung: Was fehlt

### 6.1 Graph-Index: Entfernungspfad

Die Aufgabenbeschreibung fordert: „Halbentfernte Graph-Reste (graph_index, relate, hyperedge_worker) werden entfernt, nicht repariert." Der aktuelle Zustand ist inkonsistent:

- `hyperedge_worker.rs` ist ein leerer Stub (10 Zeilen, kein Code)
- `DbTransaction` staged Graph-Operationen, die nie ausgeführt werden
- `compensating_actions.rs::CompensateGraphAction` existiert produktiv

**Empfohlene Entfernung in einem PR:**
1. `staged_graph_entities`, `staged_graph_edges`, `staged_graph_entity_deletes`, `staged_graph_edge_deletes`, `staged_hyperedges` aus `DbTransaction` löschen
2. `commit_graph_staged()`, `CompensateGraphAction`, `stage_graph_entity()`, `stage_graph_edge()`, `stage_graph_entity_delete()`, `stage_graph_edge_delete()` löschen
3. Graph-Staging in `insert_op()`, `update_op()`, `delete_op()` entfernen
4. `hyperedge_worker.rs` löschen, aus `mod.rs` entfernen
5. `GraphIndex`-Imports aus `db_transaction*.rs` entfernen

### 6.2 Vertikaler Shutdown-Pfad

Der `TaskTracker`-gestützte Shutdown (via `CancellationToken`) ist korrekt implementiert, aber:
- Die im `Drop`-Pfad der `DbTransaction` gespawnten Rollback-Tasks sind **nicht im `TaskTracker` registriert** und können nach `cancel_token.cancel()` + `task_tracker.wait()` noch laufen.
- Empfehlung: Rollback-Tasks in `begin_transaction()` mit einem Arc auf den `TaskTracker` ausstatten und dort spawnen.

### 6.3 Purge-Reihenfolge bei Schichtlöschung

Laut Aufgabenbeschreibung: „Konsistenz über mehrere Indizes (atomares Sichtbarwerden, Purge-Reihenfolge je Schicht)". Die aktuelle Purge-Reihenfolge bei Rollback ist:

```
Graph → Text → Vector → Storage
```

Bei Commit-Compensation (Ledger, LIFO) wird umgekehrt in der Reihenfolge kompensiert, in der Aktionen gepusht wurden:

```
RollbackStaged → CompensateLsm → CompensateHnsw → CompensateText → CompensateGraph
```

Das ist korrekt (LIFO). Aber: **Text-Index wird vor HNSW kompensiert** (weil Text nach HNSW committed wird). Bei einem Crash nach Text-Commit, aber vor HNSW-Commit: Text ist committed, HNSW nicht. Recovery-Pfad kompensiert Text (`CompensateTextAction`), aber HNSW war nie committed — kein Kompensationsbedarf für HNSW. Das ist korrekt.

**Lücke:** Recovery beim Start (cleanup.rs) erkennt diesen partiellen Zustand nicht und führt keine Text-Kompensation durch — nur die Collection-spezifische `repair_internal()` macht einen Text-Fallback-Scan.

---

## 7. Zusammenfassung nach Dateien

| Datei | Schwere | Befunde |
|-------|---------|---------|
| [`delete.rs`](file:///home/freddy/Projekte/contextra/crates/contextra-engine/src/collection/crud/delete.rs) | 🔴 | DEF-01: Lock zu früh freigegeben |
| [`update.rs`](file:///home/freddy/Projekte/contextra/crates/contextra-engine/src/collection/crud/update.rs) | 🔴 | DEF-01: Lock zu früh freigegeben |
| [`kv.rs`](file:///home/freddy/Projekte/contextra/crates/contextra-engine/src/collection/crud/kv.rs) | 🔴 | DEF-01: Lock zu früh freigegeben (2x) |
| [`maintenance.rs`](file:///home/freddy/Projekte/contextra/crates/contextra-engine/src/collection/maintenance.rs) | 🔴🟠🟡 | DEF-01 (L707), HIGH-02, HIGH-04, MED-05, OPT-01 |
| [`db_transaction.rs`](file:///home/freddy/Projekte/contextra/crates/contextra-engine/src/transaction/db_transaction.rs) | 🔴🟡 | DEF-02, MED-01, OPT-02 |
| [`db_transaction_commit.rs`](file:///home/freddy/Projekte/contextra/crates/contextra-engine/src/transaction/db_transaction_commit.rs) | 🔴🟠🟡 | DEF-03, HIGH-01, MED-01 |
| [`db_transaction_rollback.rs`](file:///home/freddy/Projekte/contextra/crates/contextra-engine/src/transaction/db_transaction_rollback.rs) | 🟡 | MED-03 |
| [`compensating_actions.rs`](file:///home/freddy/Projekte/contextra/crates/contextra-engine/src/transaction/compensating_actions.rs) | 🟡 | MED-01, MED-03 |
| [`cleanup.rs`](file:///home/freddy/Projekte/contextra/crates/contextra-engine/src/transaction/cleanup.rs) | 🟠 | HIGH-01 (Recovery ignoriert stages_completed) |
| [`insert.rs`](file:///home/freddy/Projekte/contextra/crates/contextra-engine/src/collection/crud/insert.rs) | 🟠🟢 | HIGH-05, OPT-03 |
| [`collections.rs`](file:///home/freddy/Projekte/contextra/crates/contextra-engine/src/contextra_impl/collections.rs) | 🟠 | HIGH-06 |
| [`links.rs`](file:///home/freddy/Projekte/contextra/crates/contextra-engine/src/collection/crud/links.rs) | 🟠 | HIGH-03 |
| [`hydrate.rs`](file:///home/freddy/Projekte/contextra/crates/contextra-engine/src/collection/search/hydrate.rs) | 🟡 | MED-02: ReadGate No-Op |
| [`orphan_workers.rs`](file:///home/freddy/Projekte/contextra/crates/contextra-engine/src/background_workers/orphan_workers.rs) | 🟡 | MED-08: SystemClock direkt |
| [`read.rs`](file:///home/freddy/Projekte/contextra/crates/contextra-engine/src/collection/crud/read.rs) | 🟡 | MED-04 |
| [`mod.rs`](file:///home/freddy/Projekte/contextra/crates/contextra-engine/src/collection/mod.rs) | 🟠🟢 | MED-06 (shared next_tx), OPT-05 |
| [`kv_lock.rs`](file:///home/freddy/Projekte/contextra/crates/contextra-engine/src/collection/kv_lock.rs) | 🟢 | OPT-04 |
| [`hyperedge_worker.rs`](file:///home/freddy/Projekte/contextra/crates/contextra-engine/src/background_workers/hyperedge_worker.rs) | 🟡 | Stub ohne Funktion, zu entfernen |

---

## 8. Priorisierte Maßnahmen

| Prio | Maßnahme | Dateien |
|------|----------|---------|
| P1 | **DEF-01 fixen:** `drop(_guard)` entfernen, Guard über gesamte Transaktion halten | `delete.rs`, `update.rs`, `kv.rs`, `maintenance.rs` |
| P2 | **DEF-02 fixen:** Drop-Rollback nicht via Fire-and-Forget, Fallback auf sync best-effort | `db_transaction.rs` |
| P3 | **DEF-03 fixen:** `TenantId` aus Collection-Kontext ableiten, nicht hardcoden | `db_transaction_commit.rs` |
| P4 | **MED-01/Graph-Reste entfernen:** `staged_graph_*`, `commit_graph_staged`, Hyperedge-Worker | `db_transaction*.rs`, `compensating_actions.rs`, `hyperedge_worker.rs` |
| P5 | **HIGH-06 fixen:** Collection-Init außerhalb des Write-Locks | `collections.rs` |
| P6 | **HIGH-01 fixen:** Recovery `stages_completed` auswerten | `cleanup.rs`, `maintenance.rs` |
| P7 | **MED-08 fixen:** Clock-Port in Orphan-Worker injizieren | `orphan_workers.rs` |
| P8 | **MED-02 — ReadGate:** Im Erasure-WP implementieren | `hydrate.rs` |
| P9 | **OPT-01:** Scan-Loop extrahieren | `maintenance.rs` |
| P10 | **OPT-04:** KV_LOCK_SHARDS erhöhen | `kv_lock.rs` |

---

*Report erstellt durch systematische Quellcode-Lektüre aller Produktionsdateien in `contextra-engine/src/`. Keine Subagenten verwendet.*
