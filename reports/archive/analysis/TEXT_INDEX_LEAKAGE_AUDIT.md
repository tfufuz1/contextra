# Forensischer Audit-Report: `contextra-text`
**Information-Retrieval-Architektur & Searchable Encryption Leakage-Profile**

> **Datum:** 2026-11-02
> **Rolle:** Information-Retrieval-Ingenieur mit Kenntnis von Searchable-Encryption-Leakage-Profilen
> **Crate:** `contextra-text` (Ring 2, Ist-Kopf `lib.rs` nennt fehlerhaft Ring 0)
> **Normative Basis:** `docs/SPEC.md` v > `docs/VETOES.md` > `docs/CONSTITUTION.md` > `AGENTS.md`
> **Evidenzstufen:** EV0 (Behauptung) → EV1 (Quelltext) → EV2 (Lokaler Lauf) → EV3 (Mutationsprüfung) → EV4 (Lab/Beweis)

---

## 1. Urteil (Executive Summary)

1. `contextra-text` verletzt die Identitätsentkopplung (S1/D-08/D-70): Postings speichern rohe 64/128-Bit `DocId` statt geblindeter Tags (`DocHandle`/`DocTag`), wodurch Segmentinhalte Dokumentbeziehungen und Identitäten leaken.
2. Der Index ist als termgeordneter KV-Store im LSM realisiert (`meta:stats`, `StorageEngine`), was D-20 (keine Postings im LSM) und E-L5 (kein Zählerüberleben nach Löschung) fundamental bricht.
3. `tests/alloc_profiler.rs` verletzt AR-8/T-UNSAFE-1 durch `unsafe impl GlobalAlloc` außerhalb der Unsafe-Inseln (`sys`/`simd`), da das `#![forbid(unsafe_code)]` in `src/lib.rs` Integrationstests in `tests/` nicht bindet.

---

## 2. Befundtabelle

| ID | Datei:Zeile | Soll (SPEC-ID) | Ist | Evidenz | Folgetest / Maßnahme |
|---|---|---|---|---|---|
| **TXT-01** | `src/posting_list.rs:31-62` | SPEC 9.3, D-08, D-70 (`DocHandle`, `doc_tag`) | Rohe `DocId`/`DocIdRaw` (u64/u128) in Postings und BlockInfo | EV1 (Code), EV2 (`cargo check`) | T-DOCREF-1, Lab-Scan auf Byte-Muster roher DocIds |
| **TXT-02** | `src/inverted/index_struct.rs:18-50` | SPEC 9.7, D-20 (LSM hält keine Postings) | `InvertedIndex<S: StorageEngine>` speichert Postings direkt im LSM | EV1 (Code), EV2 (Test) | T-TEXT-1, Grep nach `StorageEngine` in `src/` = 0 |
| **TXT-03** | `src/inverted/index_struct.rs:245-270` | SPEC 9.7, E-L5 (Exakte Neuberechnung, kein Zählerüberleben) | Globale Statistik `meta:stats` (`total_docs`, `avg_doc_len_x1000`) wird transaktional mutiert statt neu berechnet | EV1 (Code), EV2 (Test) | T-TEXT-3, E-L5-Proptest für Statistik-Neuberechnung |
| **TXT-04** | `tests/alloc_profiler.rs:40-66` | SPEC §3.2 R6, AR-8, T-UNSAFE-1 (`unsafe` nur in `sys`/`simd`) | `#[allow(unsafe_code)]` und `unsafe impl GlobalAlloc` in Integrationstest | EV1 (Code), EV2 (`cargo test -p contextra-arch-tests`) | Verschiebung nach Tooling oder sys-Hilfe |
| **TXT-05** | `src/bm25.rs:25` | SPEC 9.7, D-81 (`k1 = 1.2`, `b = 0.75`) | Standard `BM25_K1 = 1.5` statt 1.2 | EV1 (Code), EV2 (Test) | Referenztest `bm25_reference_values.rs` anpassen |
| **TXT-06** | `src/posting_list.rs:23` | SPEC 9.7, D-81 (128er-Blöcke, Block-Max WAND) | Feste Blockgröße `BLOCK_SIZE = 64` statt 128 | EV1 (Code), EV2 (Test) | T-FMT-1, WAND-Benchmark BM-TEXT |
| **TXT-07** | `src/lib.rs:12` | SPEC 5.2, AR-1 (Ring 2 Crate) | Crate-Header und Docs deklarieren fälschlich Ring 0 | EV1 (Code) | T-ARCH-5 Crate-Header-Bereinigung |
| **TXT-08** | `src/domain/*` | SPEC D-81, D-84 (Nur Analyzer `de/1` und `und/1`) | Spezifische Domänenvokabulare (`legal_de`, `medical_de`) ohne SPEC-Verankerung | EV1 (Code) | Rückbau im Rahmen von WP-1.2 / WP-7.1 |

---

## 3. Searchable Encryption Leakage-Inventar je Segmentfeld

Ein verschlüsselter Index („Index ist verschlüsselt“) garantiert ohne formales Leakage-Profil keine Vertraulichkeit gegen passive oder aktive Angreifer. Unter Zero-Trust-Prämissen analysieren wir die Felder der Segment- und Posting-Strukturen:

| Feld / Metadaten | Ist-Zustand | Soll-Zustand (SPEC F.7 / 9.7) | Leakage-Profil | Relevante Angriffsklasse |
|---|---|---|---|---|
| **Term-Identifikator** | Klartext-Strings im KV-Store (`__txt:...`) | `term_tag = tag("ctx/term/v1", analyzer_id ‖ lang ‖ norm_term)` (16 B) | **Klartext-Leakage:** Vollständige Offenlegung des Vokabulars und aller indizierten Wörter | Direkte Datenbankabfrage, Wörterbuchabgleich |
| **Dokument-Identität** | Rohe `DocId` (u64 / u128) direkt im Posting | `doc_ord` (lokal 0..N) + Envelope-Bindung `doc_ord → (cluster_tag, doc_tag)` unter DEK | **Identitäts-Leakage (S1):** Postings offenbaren, welche Dokumente dieselben Terme teilen; Querverbindung zu Datensätzen | IKK-Angriff (Islam et al.), Graph-Inferenz, Co-Occurrence-Clustering |
| **Termfrequenz (`tf`)** | Rohe `u32` unmaskiert pro Posting | `u32` im Block, `max_tf_norm` je 128er-Block | **Frequenz-Leakage:** Exakte Häufigkeitsverteilungen pro Dokument | Frequency-Analysis-Angriffe, Known-Document-Angriffe |
| **Dokumentlänge (`dl`)** | Rohe `u32` in jedem Posting und in `meta:stats` | `doc_ord → doclen` in Dokumenttabelle | **Längen-Leakage:** Exakte Dokumentlängen erlauben Fingerprinting spezifischer Templates/Akten | Length-Based Filtering Attack, Rekonstruktionsangriff |
| **Positionen** | Nicht unterstützt / fehlt in Posting | Optional im Segment (Collection-Option, Standard an) | **Positions-Leakage:** Nachbarschaftsbeziehungen zwischen Termen | Phrasen-Rekonstruktion, Inferenz von Satzstrukturen |
| **Segment-Header (`bucket`, `bucket_count`)** | Nicht vorhanden (reiner KV-Store) | Big-Endian Header (F.7): `bucket`, `bucket_count`, `analyzer_id` | **Bucket-Partitioning:** Offenbart Cluster-Zugehörigkeit (`cluster_tag[0] mod bucket_count`) | Count-Angriff auf Cluster-Größen |
| **Lösch-Bitmap** | Nicht vorhanden (Löschung mutiert KV-Store) | Roaring/Bit-Bitmap im Segment, Vernichtung via Segment-Rewrite | **Tombstone-Muster:** Bei KV-Store sofortiges Sichtbarwerden gelöschter Schlüssel | Access-Pattern-Leakage, Temporal Deletion Profiling |

### Angriffsszenarien im Detail:
1. **IKK-Angriff & Co-Occurrence-Inferenz:** Durch die Speicherung von roher `DocId` in `Posting` können Angreifer mit Zugriff auf Segment-Dumps (z. B. bei Backup-Exfiltration oder Speicher-Inspektion) identifizieren, welche Dokumente dieselben seltenen Terme enthalten. Bei blindierten Tags mit lokalem `doc_ord` und verschlüsselter Bindung unter DEK (`ctx/tbind/v1 ‖ file_id ‖ doc_ord`) ist diese Inferenz nach Schlüsselvernichtung unmöglich (`unlinked`).
2. **Volumen- und Count-Angriffe:** Wenn nach `erase(Ziel)` Zähler in `meta:stats` nur dekrementiert werden, aber Rundungsartefakte oder historische Spuren verbleiben, kann die genaue Größe des gelöschten Dokuments rekonstruiert werden. SPEC E-L5 fordert die strikte exakte Neuberechnung aus den verbleibenden Segmenten.
3. **Wörterbuchangriff auf gelöschte Tags:** Ist ein Dokument gelöscht, bleiben Postings in Segmenten bis zum physischen Rewrite bestehen. Da die Bindung jedoch mit dem dokumentspezifischen DEK verschlüsselt ist, schlägt der DEK-Lookup im Vault fehl – das Posting wird als `unlinked` verworfen.

---

## 4. Analyse der Allokationsmessung (`tests/alloc_profiler.rs`)

### Befund:
`crates/contextra-text/tests/alloc_profiler.rs` implementiert `CountingAllocator` via `unsafe impl GlobalAlloc` und verwendet mehrere `#[allow(unsafe_code)]`-Attribute sowie direkte `unsafe`-Blöcke.

```rust
// tests/alloc_profiler.rs:40-44
#[allow(unsafe_code)]
// SAFETY: Thread-safe delegation to System allocator.
unsafe impl GlobalAlloc for CountingAllocator {
```

### Regelkonflikt:
- **AR-8 / SPEC §3.2 Regel 6 / T-UNSAFE-1:** `unsafe` ist **ausschließlich** in `contextra-sys` und `contextra-simd` erlaubt. Alle anderen Crates erzwingen `#![forbid(unsafe_code)]`.
- **Lücke:** Rust verbietet `#![forbid(unsafe_code)]` aus `src/lib.rs` nicht für Integrationstests in `tests/`, weshalb der Test isoliert kompiliert. Der Test `tests/arch/src/lib.rs:check_unsafe` scannt aktuell nur `crates/*/src/` und übersieht daher `tests/alloc_profiler.rs`.

### Lösungsvorschläge (ohne unsafe im Workspace):
1. **Verschiebung in Tooling / Benchmarks außerhalb des Workspaces:** Das Profiling-Harness wird als externes Tool in `tools/alloc_profiler` oder unter `benches/` geführt, wo spezielle Memory-Profiling-Tools (wie `dhat`, `valgrind` oder OS-Profiling über `/proc/self/statm`) zum Einsatz kommen.
2. **Delegation an `contextra-sys`:** Falls programmatische Allokationszählung im Workspace erforderlich ist, stellt `contextra-sys` (die autorisierte Unsafe-Insel) eine sichere Abstraktion bereit (z. B. `contextra_sys::alloc::CountingAllocator`), die in `contextra-text` ohne eigenes `unsafe` oder `#[allow(unsafe_code)]` eingebunden werden kann.
3. **OS-Syscall-Messung via /proc (rein sicheres Rust):** Messung des Peak-RSS und der Page-Faults über `contextra-sys::fs_probe` oder `/proc/self/status` ohne globalen Custom Allocator.

---

## 5. Messungen und Benchmarks (BM-TEXT & BM-ERASE-WA-TEXT)

### Lokale Messung der aktuellen Testsuite:
- **Unit-Tests:** 106 passed in 95.48s (inkl. Langläufer `test_high_frequency_term_resident_index_performance`).
- **Alloc Budget Gate:** `alloc_budget_gate` passed in 29.48s (500 Dokumente).
  - Gemessene Allocs Tokenizer: <= 22.000.000
  - Gemessene Bytes Tokenizer: <= 300.000.000
- **WAND-Tie-Breaking:** Differential-Tests gegen Naive-BM25-Referenz bestehen deterministisch (`wand_tie_break_threshold.rs`, `wand_differential.rs`).

### Anforderungen für Zielarchitektur (SPEC 9.7, D-71, G-E):
1. **BM-TEXT:** Top-10-Abfrage auf 1 Mio. Chunks mit deutschem Analyzer `de/1` und Block-Max WAND:
   - Soll: $p99 \le 30\,\text{ms}$.
2. **BM-ERASE-WA-TEXT:**
   - Rewrite-Aufwand bei Subject-Erase (1.000 Dokumente über 12 Monate) bei Segmentgrößen 8/32 MiB:
   - Soll: Aufwand $\le 2 \times \sum \text{Größe der berührten Segmente}$.
   - Entscheidung über `bucket_count > 1`: Falls der Median-Betroffene > 25 % der Segmente berührt (D-71d).

---

## 6. Nicht geprüft / Restrisiko

1. **Nicht geprüft (EV1):** Physische Segmentdateien nach F.7 (Dateiaufbau `text/<tenant>/<coll>/<file_id>.seg`) existieren im aktuellen HEAD noch nicht; der bestehende Code basiert auf dem transienten KV-Store `InvertedIndex<S: StorageEngine>`.
2. **Restrisiko `TextIndexUntilPurge` (SPEC D-69, E-L15):** Bis zum physischen Neuschreiben des Segments im Purge-Fenster verbleiben Postings im Segment. Zwar ist die Bindung nach DEK-Zerstörung unlesbar, jedoch leakt die Häufigkeitsverteilung des gelöschten Dokuments für Angreifer mit Kenntnis von $K_{idx}$.
3. **Restrisiko Allokations-Drift:** Da `alloc_profiler.rs` 10k Dokumente im Gate ignoriert (`run_profile_10k_documents #[ignore]`), werden Allokationsregressionen im Großkorpus nur verzögert erkannt.

---

## 7. Getroffene Entscheidungen mit SPEC-Verweis

- **ENT-TXT-01 (SPEC D-08, D-70):** Rohe `DocId`/`DocIdRaw` in Postings ist ein definierter S1-Defekt. Die Zielarchitektur erfordert `DocHandle { cluster_tag, doc_tag }` mit segmentlokaler `doc_ord`.
- **ENT-TXT-02 (SPEC D-20, D-49, D-71):** Der KV-Index (`inverted/*`) ist als technische Altlast zu verwerfen; die Schicht `text` migriert vollständig auf autonome, verschlüsselte Segmente mit Segment-Rewrite.
- **ENT-TXT-03 (SPEC AR-8, T-UNSAFE-1):** Der Verstoß in `tests/alloc_profiler.rs` darf nicht durch Lockern von Lints oder Ignorieren gelöst werden, sondern erfordert die Auslagerung der Allokationszählung aus dem Crate-Testbereich.
