# AUDIT: memfuse-graph — csr.rs compact() Lock-Pfad & Kontention

## 1. Executive Summary & Audit-Kontext
* **Datum:** 2026-08-30
* **Audit-Typ:** READ-ONLY LOCK & KONTENTIONS-ANALYSE
* **Ziel-Crate:** `crates/memfuse-graph` (Layer 2)
* **Analysierter Bereich:** `crates/memfuse-graph/src/csr.rs` (insbesondere `GraphInner::compact()`, `CsrGraph::compact()`, `CsrGraph::compact_async()`, `InnerWriteGuard`, sowie Einbindung in `personalized_page_rank`)
* **Status:** AUDIT COMPLETED — KEIN PRODUKTIONSCODE GEÄNDERT.
* **Parallelitäts-Hinweis:** Claim `cargo xtask claim --crate memfuse-graph --mode audit-readonly` registriert. J3 (Welle 1) analysiert parallel `ppr.rs`/`csr.rs` mit Fokus auf die Fusion-Grenze. Da beide Vorgänge ausschließlich lesend operieren, bestehen keine Schreibkonflikte.

---

## 2. Abgleichs-Ergebnisse & Detaillierte Befunde

### Frage 1: Schreib-Lock (`write()`) vs. Allokation der 8 neuen Vektoren
**Frage:** Wird der exklusive Schreib-Lock VOR Beginn der Allokation der 8 neuen Vektoren erworben, oder erst kurz vor dem finalen Swap (z. B. via `mem::swap`/`ArcSwap::store`)?
* **Antwort / Befund:** Der exklusive Schreib-Lock (`write_state.lock()`, als MutexGuard verpackt in `InnerWriteGuard` bzw. `inner_writer`) wird **VOR** Beginn der Allokation der 8 neuen Vektoren erworben.
* **Details:**
  - Sowohl in `CsrGraph::compact()` (`csr.rs:1090`) als auch in `CsrGraph::compact_async()` (`csr.rs:1135`) wird vor dem Aufruf von `inner.compact()` der Mutex-Lock `write_state.lock()` erworben.
  - Erst innerhalb von `GraphInner::compact()` (`csr.rs:280-406`) werden in den Zeilen `csr.rs:293-308` die 8 neuen `Vec::with_capacity`-Arrays alloziert (`new_offsets`, `new_targets`, `new_weights`, `new_tx_valid_froms`, `new_tx_valid_tos`, `new_business_valid_froms`, `new_business_valid_tos`, `new_source_doc_ids`).
  - Die Allokationsphase ist somit **nicht lock-frei** und findet vollständig unter dem exklusiven Schreib-Lock statt. Leser lesen zwar lock-frei den bestehenden RCU-Snapshot via `ArcSwap`, aber parallele Schreiber/Kompaktierer blockieren am Lock vor Beginn der Speicherallokation.
* **Status:** **BELEGT** (`csr.rs:293-308`, `csr.rs:1090`, `csr.rs:1135`)

---

### Frage 2: `try_write()`-Pfad vs. blockierendes Warten
**Frage:** Gibt es einen `try_write()`-Pfad mit Fallback (z. B. "compact überspringen, wenn bereits gesperrt"), oder blockiert jeder `compact()`-Aufruf zwingend, bis der Lock frei wird?
* **Antwort / Befund:** Es existiert **kein** `try_write()` oder `try_lock()`-Pfad. Jeder `compact()`- und `compact_async()`-Aufruf blockiert zwingend, bis der Lock frei wird.
* **Details:**
  - In `CsrGraph::compact()` (`csr.rs:1090`) wird `self.inner_write()` aufgerufen, was intern `self.write_state.lock()` ausführt.
  - In `CsrGraph::compact_async()` (`csr.rs:1135`) wird innerhalb von `spawn_blocking` direkt `write_state.lock()` aufgerufen.
  - Es gibt an keiner Stelle im Rebuild-/Kompaktierungspfad einen `try_lock()`-Fallback-Mechanismus.
* **Status:** **BELEGT** (`csr.rs:1090`, `csr.rs:1135`)

---

### Frage 3: Warteschlangen-Verhalten, Debouncing & AtomicBool
**Frage:** Können mehrere `compact()`-Aufrufe gleichzeitig in die Warteschlange geraten (z. B. durch periodischen Trigger UND expliziten Aufruf)? Existiert eine Dedup-/Debounce-Logik (z. B. ein `AtomicBool`-Flag "compaction läuft bereits")?
* **Antwort / Befund:**
  - Ja, mehrere `compact()`- bzw. `compact_async()`-Aufrufe können gleichzeitig in die Mutex-Warteschlange von `write_state` geraten.
  - Es existiert **kein** `AtomicBool`-Debounce-Flag (wie z. B. `is_compacting: AtomicBool`), das bereits beim Einstieg verhindert, dass ein neuer Aufruf in den Lock gerät oder ein `spawn_blocking`-Task gestartet wird.
  - **ABER:** Es existiert ein Double-Checked-Locking-Muster:
    1. Lock-freie Vorabprüfung (`csr.rs:1083-1088` in `compact()`, `csr.rs:1103-1108` in `compact_async()`): Wenn `!is_dirty` und keine `pending_edges`/`tombstoned_edges` vorhanden sind, bricht die Funktion sofort ab.
    2. Nach Lock-Erwerb erfolgt eine erneute Prüfung (`csr.rs:1091-1095` in `compact()`, `csr.rs:1136-1140` in `compact_async()`): Sobald der erste wartende Thread `inner.compact()` beendet hat, hat er `is_dirty = false` gesetzt sowie `pending_edges` und `tombstoned_edges` geleert. Der nachfolgend aus der Mutex-Warteschlange frei werdende Thread stellt fest, dass die Bedingungen nicht mehr erfüllt sind, und überspringt den eigentlichen `inner.compact()`-Rebuild.
* **Status:** **BELEGT** (`csr.rs:1082-1096`, `csr.rs:1102-1142`)

---

### Frage 4: Sperrdauer des exklusiven Locks & Exakte Zeilengrenzen
**Frage:** Wie lange hält der exklusive Lock im Verhältnis zur Gesamtlaufzeit von `compact()` — die gesamte Funktion, oder nur den finalen Swap-Schritt? Exakte Zeilen-Grenzen des gesperrten Abschnitts.
* **Antwort / Befund:** Der exklusive Lock wird für die **gesamte** Dauer von `GraphInner::compact()` gehalten (100% der Kompaktierungslaufzeit), nicht nur während des finalen Swaps.
* **Exakte Zeilen-Grenzen:**
  - **Synchroner Pfad (`CsrGraph::compact()`):**
    - Lock-Erwerb: `csr.rs:1090` (`let mut inner = self.inner_write();`)
    - Ausführung Rebuild: `csr.rs:1096` (`inner.compact();` -> führt `GraphInner::compact()` Zeilen `280-406` aus)
    - Lock-Freigabe + Snapshot-Publish: `csr.rs:1097` (Scope-Ende von `inner`; `InnerWriteGuard::drop` in `csr.rs:480-485` ruft `arc_swap.store(Arc::new((*self.guard).clone()))` auf und gibt danach den Mutex-Guard frei).
  - **Asynchroner Pfad (`CsrGraph::compact_async()`):**
    - Lock-Erwerb: `csr.rs:1135` (`let mut inner_writer = write_state.lock();`)
    - Ausführung Rebuild: `csr.rs:1140` (`inner_writer.compact();` -> Zeilen `280-406`)
    - Lock-Freigabe + Snapshot-Publish: `csr.rs:1141` (`arc_swap.store(Arc::new(inner_writer.clone()));` gefolgt vom Schließen des Scopes `csr.rs:1142`).
* **Status:** **BELEGT** (`csr.rs:280-406`, `csr.rs:1090-1097`, `csr.rs:1135-1142`, `csr.rs:480-485`)

---

### Frage 5: Verhalten von `personalized_page_rank_with_context` während gehaltenem Lock
**Frage:** Was passiert mit parallel laufenden `personalized_page_rank_with_context`-Aufrufen, während der Lock gehalten wird — blockieren sie synchron (Thread-Stall) oder wird ein `Err`/"busy"-Signal zurückgegeben?
* **Antwort / Befund:**
  - Sie blockieren **nicht** synchron die Tokio-Worker-Threads (kein Thread-Stall der async Runtime), geben aber **kein** `Err`/"busy"-Signal zurück.
  - **Ablauf:**
    1. `personalized_page_rank_with_context_async` (`csr.rs:1690`) ruft zuerst `self.compact_async().await?` auf.
    2. Wenn eine Kompaktierung erforderlich ist, lagert `compact_async()` die Arbeit via `tokio::task::spawn_blocking` in den Tokio-Blocking-ThreadPool aus.
    3. Der Tokio-Blocking-Task versucht `write_state.lock()` zu erwerben. Wenn eine andere Kompaktierung läuft, wartet der Blocking-Task auf den Mutex.
    4. Die PPR-Future `await`ed das Ergebnis des `spawn_blocking`-Tasks.
    5. Sobald die Kompaktierung abgeschlossen ist, ruft PPR `self.inner_read()` (`csr.rs:1698`, bzw. `1990` in `personalized_page_rank`) auf und lädt den RCU-Snapshot völlig lock-frei (`ArcSwap::load`).
  - **Fazit:** Parallele PPR-Aufrufe warten asynchron auf das Ende der laufenden Kompaktierung, bevor sie ihren lock-freien RCU-Snapshot beziehen. Es erfolgt weder ein Abbruch mit `Err` noch ein synchroner Block der Tokio-Async-Worker-Threads.
* **Status:** **BELEGT** (`csr.rs:1132-1143`, `csr.rs:1690-1708`, `csr.rs:1987-2006`)

---

### Frage 6: Vermeidbarkeit der 8-fachen Vec-Allokation
**Frage:** Ist die 8-fache Vec-Allokation vermeidbar (z. B. durch In-Place-Rebuild oder Wiederverwendung der alten Kapazität via `.clear()` statt Neuallokation)?
* **Beobachtung:**
  - In `GraphInner::compact()` (`csr.rs:293-308`) werden bei jedem Durchlauf 8 neue Vektoren mit `Vec::with_capacity(...)` auf dem Heap alloziert.
  - Nach der Neuerstellung der Spalten werden die alten Vektoren in `csr.rs:360-367` überschrieben.
  - Da `GraphInner` beim Veröffentlichen via `arc_swap.store(Arc::new(inner_writer.clone()))` geclont wird, verbleibt der alte Snapshot für existierende Leser im Speicher, während der Writer-Zustand neu aufgebaut wird.
  - **Optimierungspotenzial (Beobachtung, NICHT zu fixen):** Die Neuerstellung aller 8 Spaltenvektoren unter exklusivem Lock erzeugt unnötigen Allokationsdruck auf den Heap. Ein Aufbau der neuen CSR-Strukturen außerhalb des Locks (z. B. auf einer ungesperrten Arbeitskopie) oder die Wiederverwendung von Puffer-Kapazitäten vor Lock-Erwerb könnte die Lock-Haltezeit erheblich verkürzen.
* **Status:** **BELEGT** (`csr.rs:293-308`, `csr.rs:360-367`, `csr.rs:480-485`)

---

## 3. Strict Invariants & APM Matrix Cross-Check

### Locking & Deadlock-Prävention
* In `crates/memfuse-graph/src/csr.rs` werden weder `NodesGuard` (spezifisch für `session_dag.rs`) noch `ConsolidationNodesGuard` (spezifisch für `memfuse-db`) verwendet.
* `CsrGraph` nutzt eine klare Trennung:
  - **Read-Path (RCU):** `ArcSwap<GraphInner>` via `inner_read()` — garantiert lock-freien Zugriff für Leser (einschließlich PPR).
  - **Write/Compaction-Path:** `parking_lot::Mutex<GraphInner>` (`write_state`) — serialisiert alle Modifikationen und Kompaktierungen.
* Deadlocks zwischen Lesern und Kompaktierern sind durch das RCU-Muster strukturell ausgeschlossen.

### Zero-Copy & Memory Alignment
* `GraphInner` speichert CSR-Arrays kontigu im Speicher (`offsets`, `targets`, `weights`, `tx_valid_froms`, `tx_valid_tos`, `business_valid_froms`, `business_valid_tos`, `source_doc_ids`).
* Die 8-fache Vec-Allokation während `compact()` erfolgt vollständig unter dem Schreib-Lock, stellt jedoch die Memory-Alignment- und Cache-Kontiguitäts-Invarianten der CSR-Arrays nach Abschluss wieder her.

---

## 4. Fazit & Zusammenfassung

| Prüfpunkt | Ergebnis | Relevante Zeilen in `csr.rs` |
| :--- | :--- | :--- |
| **1. Lock-Erwerb vor Allokation** | Lock wird VOR Allokation der 8 Vektoren erworben. | `293-308`, `1090`, `1135` |
| **2. Non-blocking `try_write()`** | NEIN, blockierender Mutex-Lock (`lock()`). | `1090`, `1135` |
| **3. Queueing & Debouncing** | Queuing möglich; kein `AtomicBool`-Debounce, aber Double-Checked-Locking. | `1082-1096`, `1102-1142` |
| **4. Lock-Haltezeit** | 100% der `compact()`-Dauer (O(V+E) Rebuild & Allokation). | `280-406`, `1090-1097`, `1135-1142` |
| **5. PPR-Interaktion** | Wartet asynchron via `spawn_blocking.await`; kein Thread-Stall, kein `Err`. | `1132-1143`, `1690-1708`, `1987-2006` |
| **6. 8x Vec Allokations-Druck** | 8x `Vec::with_capacity` pro Durchlauf unter Lock; Optimierungspotenzial vorhanden. | `293-308`, `360-367` |
