# AUDIT: SnapshotPinGuard / pin_snapshot Lebenszyklus in contextra-index

**Crate:** `crates/contextra-index` (Layer 3)
**Status / Modus:** AUDIT (read-only)
**Datum:** 2026-09-17
**Auditor:** Google-Jules (Principal Rust Systems Engineer)
**VERDICT:** CONDITIONAL (ID: AGT-INDEX-SNAP-01) (TS: 2026-09-17T07:37:29Z) (SESSION: dba1473f) (VERIFIED-BY-SESSION: PENDING)

---

## 1. HINWEIS ZUR DATEI-LOKALISIERUNG & CRATE-STRUKTUR

Die Aufgabenstellung nennt als Ziel-Datei `crates/contextra-index/src/snapshot.rs`.
Eine Analyse des Dateibaums zeigt, dass eine Datei `snapshot.rs` in `crates/contextra-index/src/` **nicht existiert**.
Der gesamte Lebenszyklus von `SnapshotPinGuard`, `pin_snapshot` und `unpin_snapshot` ist in `crates/contextra-index/src/hnsw.rs` implementiert und stützt sich auf die Datenstruktur `SequenceLog` in `crates/contextra-core/src/seq_log.rs`.

---

## 2. Detaillierte Befunde zu den Audit-Fragen

### Frage 1: RAII-Implementierung & Panic-Unwind-Verhalten
**Status:** **BELEGT**

`SnapshotPinGuard` ist in `crates/contextra-index/src/hnsw.rs:335-348` als RAII-Typ realisiert. Die `Drop`-Implementierung ruft bedingungslos (`unconditional`) `unpin_snapshot` auf, was auch bei Panic-Unwinding im gepinnten Scope garantiert ausgeführt wird.

*Fundstelle (`crates/contextra-index/src/hnsw.rs:335-348`):*
```rust
struct SnapshotPinGuard<'a>(&'a HnswIndexCore, u64);

impl<'a> SnapshotPinGuard<'a> {
    fn new(core: &'a HnswIndexCore, seq_no: u64) -> Self {
        core.cold.seq_log.write().pin_snapshot(seq_no);
        Self(core, seq_no)
    }
}

impl<'a> Drop for SnapshotPinGuard<'a> {
    fn drop(&mut self) {
        self.0.cold.seq_log.write().unpin_snapshot(self.1);
    }
}
```

---

### Frage 2: Refcount-Atomarität, Locking & TOCTOU-Fenster
**Status:** **BELEGT**

* **Schutzmechanismus:** Der Pin-Zähler (Refcount) ist als Hash-Map (`pinned_snapshots: ahash::AHashMap<u64, usize>`) innerhalb von `SequenceLog` realisiert (`crates/contextra-core/src/seq_log.rs:66`).
* **Lock-Schutz:** `SequenceLog` ist in `HnswColdCore.seq_log` als `parking_lot::RwLock<SequenceLog>` gekapselt (`crates/contextra-index/src/hnsw.rs:378`).
* **TOCTOU-Bewertung:** Bei `SnapshotPinGuard::new` wird das `RwLock` im Schreibmodus erworben (`seq_log.write()`), um den Refcount in `pinned_snapshots` zu erhöhen (`hnsw.rs:339`). Es existiert kein TOCTOU-Fenster zwischen Existenzprüfung und Pin-Erhöhung im Guard selbst, da `pin_snapshot` die Sequenznummer direkt eintragen bzw. hochzählen lässt. Allerdings prüft `SequenceLog::pin_snapshot` nicht, ob die Sequenznummer bereits kompaktiert wurde (siehe Frage 4).

*Fundstelle (`crates/contextra-core/src/seq_log.rs:83-98`):*
```rust
    /// Pins a historical sequence number to prevent rebuild from purging soft-deleted nodes active at this snapshot.
    pub fn pin_snapshot(&mut self, seq_no: u64) {
        *self.pinned_snapshots.entry(seq_no).or_insert(0) += 1;
    }

    /// Unpins a historical sequence number.
    pub fn unpin_snapshot(&mut self, seq_no: u64) {
        if let std::collections::hash_map::Entry::Occupied(mut entry) =
            self.pinned_snapshots.entry(seq_no)
        {
            if *entry.get() <= 1 {
                entry.remove();
            } else {
                *entry.get_mut() -= 1;
            }
        }
    }
```

---

### Frage 3: Lock-Erwerbsreihenfolge & Deadlock-Prävention
**Status:** **BELEGT**

In allen Pfaden wird das `seq_log` RwLock kurzzeitig (transient) erworben und sofort wieder freigegeben. Es gibt keine geschachtelten (nested) Locks zwischen `seq_log` und anderen Ressourcen (wie `hot.nodes` oder `write_mutex`).

1. **Pinning Pfad (`search_at`):**
   * `SnapshotPinGuard::new` holt `core.cold.seq_log.write()`, pinned `seq_no`, und gibt den Lock am Ende der Anweisung sofort frei (`hnsw.rs:339`).
   * Anschließend fordert `search_at` `self.inner.cold.seq_log.read().clone()` an, um den Snapshot für die Sichtbarkeitsprüfung zu klonen (`hnsw.rs:3562`). Das Lock wird direkt nach dem Klonen freigegeben.
2. **Kompaktierungs- / Rebuild-Pfad (`trigger_rebuild_async` / `compact_seq_log`):**
   * `compact_seq_log` holt `self.inner.cold.seq_log.write().compact(...)` transient (`hnsw.rs:832`).
   * `rebuild` liest `self.cold.seq_log.read().min_retention_seq()` transient (`hnsw.rs:2945-2946`).

Die Lock-Erwerbsreihenfolge ist konsistent und frei von zirkulären Abhängigkeiten.

*Code-Beleg (`crates/contextra-index/src/hnsw.rs:3560-3567`):*
```rust
    /// Searches for nearest neighbors at a specific snapshot sequence number.
    async fn search_at(&self, query: &[f32], k: usize, seq_no: u64) -> Result<Vec<ScoredDocument>> {
        let _pin_guard = SnapshotPinGuard::new(&self.inner, seq_no);
        let log = self.inner.cold.seq_log.read().clone();
        let filter_fn = move |doc_id: DocId| -> bool { log.is_visible(doc_id, seq_no) };
        self.search_filtered_internal(query, k, Some(&filter_fn), Some(seq_no))
            .await
    }
```

---

### Frage 4: Verhalten bei Pinning bereits gelöschter/kompaktierter Snapshots
**Status:** **BELEGT**

* **Verhalten:** Wenn `pin_snapshot` für eine `seq_no` aufgerufen wird, die kleiner als `compacted_below` ist, gibt es **keinen Fehler (`Err`)** und keinen Panik-Zustand. `pin_snapshot` fügt die `seq_no` bedingungslos in `pinned_snapshots` ein.
* **Sicherheits- / Korrektheitsauswirkung:** Da Einträge mit `delete_seq < min_active_seqno` bereits aus `seq_log.entries` entfernt wurden, liefert `is_visible(doc_id, seq_no)` für diese gelöschten Elemente `false`. Die Query liefert nur die verbleibenden Einträge (bzw. ein leeres Ergebnis) zurück. Es kommt zu **keinem undefinierten Verhalten (UB)**, keinem Out-of-bounds-Zugriff und keinem Speicherleck.

---

### Frage 5: Timeout & Obergrenze für Pin-Dauer
**Status:** **BELEGT**

* **Ergebnis:** Es existiert **KEIN Timeout** und **KEINE Obergrenze** for die Pin-Dauer.
* **Risiko:** Wenn eine Query im ungepinnten Scope hängt oder wenn ein Aufrufer die publizierte Methode `index.pin_snapshot(seq_no)` manuell aufruft und das spätere `unpin_snapshot(seq_no)` versäumt, bleibt `min_retention_seq()` dauerhaft auf oder unter `seq_no` blockiert. Dadurch kann die GC-Kompaktierung von `seq_log` (`compact`) für alte gelöschte Einträge dauerhaft blockiert werden.

---

### Frage 6: `.unwrap()` / `.expect()`-Scan im Scope von `hnsw.rs`
**Status:** **BELEGT**

Ein vollständiger Scan des Produktionscodes in `crates/contextra-index/src/hnsw.rs` ergab:

* **Produktionspfad (`SnapshotPinGuard`, `pin_snapshot`, `unpin_snapshot`, `search_at`):**
  **0 Vorkommen** von `.unwrap()` oder `.expect()`.
* **Test- und Benchmark-Scope (`#[cfg(test)]` & Test-Module):**
  Alle Fundstellen von `.unwrap()`/`.expect()` befinden sich ausschließlich in Unit-Tests und Property-Tests (z. B. `test_hnsw_search_at_snapshot_isolation`, `prop_hnsw_search_at_consistency`).

---

## 3. ZUSAMMENFASSUNG & EMPFEHLUNGEN

1. **Dateiname-Dokumentation:** Die Modul-Dokumentation / Invarianten-Taxonomie sollte klarstellen, dass das Snapshot-Pinning in `hnsw.rs` (Layer 3) und `seq_log.rs` (Layer 0) verankert ist und kein separates `snapshot.rs` existiert.
2. **GC-Blockade-Schutz (Empfehlung):** Zur Vermeidung dauerhafter GC-Blockaden durch unendliches Pinning sollte in `SequenceLog` ein optionales TTL/Lease-Konzept oder ein Schwellenwert für maximale Pin-Dauer in Erwägung gezogen werden.
3. **Verifizierung:** Die RAII-Guards verhindern zuverlässig Memory Leaks und verwaiste Pins bei Panic Unwinding.
