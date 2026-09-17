# Audit-Report: `memfuse-db` — `fusion.rs` BoundedTopK vs. HeapEntry (Dead-Code & Selektions-Algorithmik)

**Stand:** 2026-09-16
**Ziel-Crate:** `crates/memfuse-db` (Layer 5)
**Ziel-Datei:** `crates/memfuse-db/src/fusion.rs`
**Task-Typ:** AUDIT (read-only)
**Claim-Status:** `cargo xtask claim --crate memfuse-db --mode audit-readonly` (aktiv)
**Parallelitäts-Hinweis:** Parallel-Lesen auf `fusion.rs` durch synchrone Audit-Tasks (z. B. J4, J5) ist konfliktfrei, da alle beteiligten Tasks im Read-Only-Modus operieren.

---

## 1. Übersicht & Audit-Gegenstand

Dieser Audit-Report untersucht die Selektions- und Aggregations-Algorithmik im Modul `crates/memfuse-db/src/fusion.rs` mit Schwerpunkt auf:
1. Das Schicksal des in älteren Spezifikationen/Drafts erwähnten `HeapEntry` (`#[cfg(test)]`) im Vergleich zu `BoundedTopK`.
2. Die Laufzeit- und Speicherkomplexität der Top-K-Selektion via `BoundedTopK`.
3. Die korrekte Übernahme des Parameter-Limits `k` (`max_results`) aus Query-Aufrufen.
4. Die Abwesenheit bzw. Nutzung von `select_nth_unstable`.
5. Den Determinismus des Tie-Breaking-Mechanismus bei RRF-Score-Gleichstand.

---

## 2. Detaillierte Befunde zu den Protokoll-Fragen

### Frage 1: Existenz & Sichtbarkeit von `HeapEntry`
* **Frage:** Wird `HeapEntry` AUSSCHLIESSLICH unter `#[cfg(test)]` kompiliert, oder gibt es einen Pfad, in dem es auch im Produktions-Build (non-test) referenziert wird?
* **Befund:** **BELEGT**
* **Detail-Analyse:**
  - Eine Volltextsuche (`git grep "HeapEntry"`) im Repository zeigt, dass `HeapEntry` in `crates/memfuse-db/src/fusion.rs` **überhaupt nicht mehr existiert** (weder im Produktionscode noch im `#[cfg(test)]`-Block).
  - In historischen Spezifikationsentwürfen (z. B. `docs/specs/02_PROJEKT_SPEZIFIKATION_v2_archived.md`) war `HeapEntry` als Hilfs-Struct dokumentiert. Im aktuellen Produktionsstand von `fusion.rs` wurde `HeapEntry` jedoch vollständig durch `TopKCandidate<'a>` (`fusion.rs:514–541`) in Kombination mit `BoundedTopK<T>` (`fusion.rs:110–152`) ersetzt.
  - Die im Soll-Referenz-Anhang genannten historischen Zeilenbereiche (164–191, 1549, 1558, 1567, 1596, 1605, 2728–2739) enthalten im aktuellen Code andere Funktions- und Teststrukturen (`MetadataMergePriority`, `ProvenanceBuilder`, `score_normalized_fusion_with_options` sowie Unit-Tests).
  - **Fazit:** Es existiert weder ein Produktionspfad noch ein Test-Pfad für `HeapEntry`, da das Struct aus dem Quellcode von `fusion.rs` entfernt und durch `TopKCandidate` abgelöst wurde.

---

### Frage 2: Funktionalität & Redundanz (`HeapEntry` vs. `BoundedTopK`)
* **Frage:** Ist `HeapEntry` funktional redundant zu `BoundedTopK` (gleiche Vergleichslogik/Ordering), oder unterscheiden sich die Sortier-/Tie-Breaking-Regeln zwischen beiden Implementierungen? Falls Unterschiede bestehen: könnten Tests mit `HeapEntry` ein anderes Ranking validieren als der Produktionscode mit `BoundedTopK` tatsächlich liefert (Test-Prod-Divergenz-Risiko)?
* **Befund:** **BELEGT**
* **Detail-Analyse:**
  - Da `HeapEntry` aus der Codebasis entfernt wurde, nutzen **sowohl Produktions- als auch Test-Pfade** ausschließlich den kanonischen Container `BoundedTopK<TopKCandidate<'a>>`.
  - Der für `BoundedTopK` genutzte Kandidat-Typ `TopKCandidate<'a>` implementiert `Ord` in `fusion.rs:527–535` wie folgt:
    ```rust
    impl<'a> Ord for TopKCandidate<'a> {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            cmp_scores(other.score, self.score).then_with(|| self.id.cmp(other.id))
        }
    }
    ```
  - `cmp_scores` (`fusion.rs:88–94`) verwendet `f32::total_cmp` mit expliziter Behandlung von `NaN`-Werten.
  - Ein Test-Prod-Divergenz-Risiko durch zwei parallele Heap-Implementierungen **besteht somit nicht**, da alle Aufrufe (Produktion und Tests) dieselbe Datenstruktur `BoundedTopK` und dieselbe Vergleichsfunktion `TopKCandidate::cmp` verwenden.

---

### Frage 3: Instanziierung, Befüllung & `k`-Parameter-Übernahme in `BoundedTopK`
* **Frage:** An welcher exakten Stelle im finalen RRF-Merge wird `BoundedTopK` instanziiert und befüllt? Wird `k` korrekt aus dem Nutzer-Query-Parameter (`limit`/`top_k`) übernommen, oder existiert ein Hardcoded-Default, der bei kleinen `k`-Werten ineffizient wäre?
* **Befund:** **BELEGT**
* **Detail-Analyse:**
  1. **Instanziierung & Befüllung in `weighted_reciprocal_rank_fusion_with_options` (RRF-Merge):**
     - **Instanziierung:** `fusion.rs:882`
       ```rust
       let mut top_k = BoundedTopK::new(max_results);
       ```
     - **Befüllung:** `fusion.rs:883–890`
       ```rust
       for idx in 0..scores.len() as u32 {
           let i = idx as usize;
           top_k.push(TopKCandidate {
               idx,
               score: scores[i],
               id: id_table[i],
           });
       }
       ```
  2. **Instanziierung & Befüllung in `score_normalized_fusion_with_options` (Score-Normalized Merge):**
     - **Instanziierung:** `fusion.rs:1265`
       ```rust
       let mut top_k = BoundedTopK::new(max_results);
       ```
     - **Befüllung:** `fusion.rs:1266–1272`
  3. **Parameter-Übernahme & Kapazitäts-Allokation:**
     - `BoundedTopK::new(capacity)` (`fusion.rs:117–123`):
       ```rust
       pub fn new(capacity: usize) -> Self {
           let capacity = capacity.min(memfuse_core::MAX_SEARCH_K);
           Self {
               heap: std::collections::BinaryHeap::with_capacity(capacity.saturating_add(1)),
               capacity,
           }
       }
       ```
     - Der Parameter `max_results` aus dem Aufrufer (`reciprocal_rank_fusion`, `weighted_reciprocal_rank_fusion`, `fuse_search_results_with_strategy`) wird **direkt** als Kapazität an `BoundedTopK::new` übergeben.
     - Es existiert **kein** vorgegebener Hardcoded-Default (wie z. B. 1000), der den Speicherbedarf bei kleinen $k$-Anfragen (z. B. $k=5$ oder $k=10$) künstlich aufblähen würde.
     - *Hinweis zur Abgrenzung:* Die Konstante `let k = 60;` (`fusion.rs:563`, `777`) ist ausschließlich der mathematische RRF-Glättungsfaktor ($1 / (k + \text{rank})$ nach Cormack et al.), nicht das Resultat-Limit $k$.

---

### Frage 4: Analyse der Nutzung von `select_nth_unstable`
* **Frage:** Wird `select_nth_unstable` irgendwo im selben Modul verwendet? Falls ja: an welcher Stelle, und operiert es nachweislich auf einer bereits vollständig materialisierten `Vec` (kein Stream-Kontext)? Falls es doch in einem Streaming-/Iterator-Kontext verwendet wird: als Abweichung vom Design vermerken.
* **Befund:** **BELEGT**
* **Detail-Analyse:**
  - Eine Suche nach `select_nth_unstable` in `crates/memfuse-db/src/fusion.rs` (sowie in der gesamten Crate `crates/memfuse-db`) ergibt **0 Treffer**.
  - Für die Selektion der Top-K-Ergebnisse wird im gesamten Modul `fusion.rs` konsequent `BoundedTopK` (ein gecappter Min-Heap mit maximal $k+1$ Elementen) verwendet.
  - Dadurch bleibt die Komplexität während der Iteration über $N$ kandidierende Dokumente strikt bei $\mathcal{O}(N \log k)$ Zeitaufwand und $\mathcal{O}(k)$ zusätzlichem Speicherplatz.
  - Eine fehlerhafte Nutzung von `select_nth_unstable` im Streaming-Kontext liegt somit **nicht** vor.

---

### Frage 5: Tie-Breaking bei gleichem RRF-Score
* **Frage:** Tie-Breaking bei gleichem RRF-Score: Ist das Verhalten deterministisch (z. B. Sekundärsortierung nach `DocId`), oder hängt die Reihenfolge von der Einfügereihenfolge in den Heap ab (nicht-deterministisches Ranking bei Score-Gleichstand)? Exakte Fundstelle.
* **Befund:** **BELEGT**
* **Detail-Analyse:**
  - Das Tie-Breaking ist **100% deterministisch**.
  - **Exakte Fundstelle:** `crates/memfuse-db/src/fusion.rs`, Zeilen 527–535:
    ```rust
    impl<'a> Ord for TopKCandidate<'a> {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            // BinaryHeap is max-heap by default. We want peek() to return the candidate with
            // the worst score (or highest ID on tie).
            // Therefore, lower score => Greater priority in max-heap.
            cmp_scores(other.score, self.score).then_with(|| self.id.cmp(other.id))
        }
    }
    ```
  - In `cmp_scores` (`fusion.rs:88–94`) werden primär die f32-Scores verglichen (`f32::total_cmp`).
  - Bei identischem RRF-Score (`cmp_scores` liefert `Ordering::Equal`) greift der sekundäre Lexikographische Vergleich der Dokument-IDs: `.then_with(|| self.id.cmp(other.id))`.
  - Wenn `other.id > self.id`, ergibt `self.id.cmp(other.id)` `Ordering::Less`. Im Max-Heap wird das Element mit kleinerer ID somit als "schlechteres Max-Heap-Element" (d. h. als bevorzugter/besserer Suchkandidat) behandelt.
  - Ergänzend ist derselbe Determinismus in `apply_resonance_bonus` verankert (`fusion.rs:78`):
    ```rust
    b.score.total_cmp(&a.score).then_with(|| a.id.cmp(&b.id))
    ```
  - **Nachweis im Testset:** In `fusion.rs` verifizieren die Tests `test_rrf_identical_ranks` (`fusion.rs:2800–2853`) und `test_rrf_tie_breaking_multiple_documents_same_score` (`fusion.rs:3058–3100`), dass Dokumente mit identischem Score über wiederholte Durchläufe hinweg strikt deterministisch nach `DocId` sortiert werden.

---

## 3. Zusammenfassung & Empfehlungen

| Prüfpunkt | Ist-Zustand | Bewertung |
|---|---|---|
| **HeapEntry Status** | Aus `fusion.rs` entfernt; durch `TopKCandidate` ersetzt | Kein Dead-Code-Müll im Quellcode vorhanden |
| **Test-Prod-Divergenz** | Produktion und Tests nutzen `BoundedTopK<TopKCandidate>` | Vollständige Konsistenz |
| **Top-K-Komplexität** | `BoundedTopK` mit $\mathcal{O}(N \log k)$ Zeit & $\mathcal{O}(k)$ Speicher | Optimal für kandidatenweise Aggregation |
| **`max_results`-Handling** | `capacity` = `max_results` (gecappt auf `MAX_SEARCH_K`) | Kein ineffizienter Hardcoded-Default |
| **`select_nth_unstable`** | 0 Aufrufe | Kein Missbrauch im Stream-Kontext |
| **Tie-Breaking** | `cmp_scores(...).then_with(|| self.id.cmp(other.id))` | 100% deterministisches Ranking |

**Empfehlung für Folgetasks:**
Da der Quellcode in `fusion.rs` bezüglich `BoundedTopK`, `TopKCandidate` und Determinismus bereits optimal umgesetzt ist, sind keine Code-Anpassungen an `fusion.rs` erforderlich. Etwaige historische Doku-Referenzen auf `HeapEntry` in älteren Archiv-Spezifikationen können bei zukünftigen Dokumentations-Cleanups bereinigt werden.
