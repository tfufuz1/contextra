# SPEC-05 — Indizes: Adressraum, Manifest, Vektor, Volltext, Graph, Filter

**Version 3.0 · 26. November 2026 · Status: Final**
**Querverweise:** [Index](SPEC-00-INDEX.md) · [Datenmodell](SPEC-02-DATA-MODEL.md) · [Versionierung](SPEC-04-VERSIONING.md) · [Context Engine](SPEC-06-CONTEXT-ENGINE.md)

Alle Indizes sind segmentbasiert, unveränderlich, content-addressed und **nicht** Teil des `truth_hash`. Verlust führt nie zu Datenverlust.

---

## 1. IndexManifest und Abdeckung

```rust
pub struct IndexManifestRef {              // Manifest-Log, je Branch
    pub manifest_id: ChunkId,
    pub covers: CommitId,
    pub covers_truth_hash: Digest,
    pub watermark_seq: Seq,
}

pub struct IndexManifest {
    pub covers: CommitId,
    pub catalog_digest: Digest,
    pub segments: Vec<SegmentRef>,          // kind, space/analyzer, gid_range, rows, bytes
    pub watermarks: BTreeMap<LaneKey, Seq>, // je (Art, Space|Analyzer)
    pub pending: BTreeMap<LaneKey, Roaring64Ref>,  // Gids ohne verfügbares Derivat
    pub fts_stats: Vec<FtsCorpusStats>,
    pub deletion_vectors: Vec<DeletionVectorRef>,
}
```

**Warum Watermark allein nicht genügt:** Ein Embedding kann lange nach seinem Commit eintreffen. Eine Cell unterhalb der Sequenzmarke ohne Vektor steht deshalb in `pending` und wird beim Eintreffen nachindiziert. Die Abdeckung eines Lanes ist:

```
lane_coverage = durchsuchbare sichtbare Kandidaten / sichtbare Kandidaten, für die der Lane anwendbar ist
```

Zähler und Nenner werden **innerhalb der ACL-Sichtmaske** gebildet (INV-5).

**Ist `covers` kein Vorfahr des Snapshot-Heads** (Rollback, fremder Branch), bleiben Segmente nutzbar, weil jeder Treffer validiert wird (§2.3); Zeilen aus der Differenz werden über den Prolly-Diff als Delta ergänzt.

Compaction ersetzt ausschließlich Manifeste. Die Publikation ist ein CAS gegen das vorherige Manifest.

---

## 2. Adressraum, Filter, Validierung

### 2.1 Filter im `Gid`-Raum

```
allow = ACL(principal) ∩ NS ∩ Kind ∩ Trust ∩ Grounding ∩ ¬TaintBlock
        ∩ ValidAt(v) ∩ Attrs ∩ Live(snapshot) [∩ GraphFrontier]
```

Basis sind `Roaring64`-Bitmaps aus Segmenten, überlagert mit dem RAM-Delta. Bitmaps für häufig geänderte Prädikate dürfen **Obermengen** sein, ausgenommen ACL: Die ACL-Maske MUSS exakt oder eine Untermenge sein; fehlt Information, ist die Cell unsichtbar.

### 2.2 Übersetzung `Gid → DocId`

Jedes Segment speichert `gids[N]`, sortiert nach `(gid, born_seq)`; `DocId` ist die Position. Mehrere Versionen derselben `Gid` sind zulässig. Übersetzung durch Galloping-Intersection, Aufwand O(m·log(n/m)) mit m = kleinere Seite; Ergebnis `Roaring32`, gecacht je (Query, Segment). Dieselbe `DocSet` schränkt Graph-Beam, Postings und Flat-Scan **während** der Suche ein.

### 2.3 Treffervalidierung (verbindlich)

Jeder Index-Eintrag trägt `(gid, input_digest[16])`. Ein Treffer ist gültig, wenn

1. die `Gid` im Snapshot lebt und in `allow` liegt, **und**
2. `input_digest` dem aktuellen Eingabedigest dieser Cell für diesen Producer entspricht.

Damit sind veraltete Versionen unabhängig von Deletion Vectors ausgeschlossen. Deletion Vectors (je Manifest, also je Branch) sind nur Beschleuniger. Verwirft die Validierung so viele Treffer, dass weniger als `k` bleiben, wird mit verdoppeltem Over-Fetch einmal nachgesucht.

### 2.4 Flat-Fallback

| Zustand | Verhalten |
|---|---|
| Nicht indizierte Zeilen ≤ `flat_fallback_max_rows` (Default 200 000 je Lane) | exakte Ergänzung |
| darüber | zeitbegrenzter Scan, neueste zuerst; `lane_coverage < 1`, Warnung `PartialIndex`, Index-Job mit höchster Priorität |
| Aufrufer verlangt `require_complete` | unbegrenzter exakter Scan bis Query-Deadline, sonst `index_incomplete` |

Ein vollständiger Indexverlust auf großen Beständen ist damit **verfügbar, aber degradiert**; Latenzziele gelten dann nicht. FTS-Fallback tokenisiert Rohtext und ist entsprechend teuer.

---

## 3. Volltext (`contextra-text`)

### 3.1 Segment

1. FST-Term-Dictionary (exakt, Präfix, Fuzzy per Levenshtein-Automat, Distanz ≤ 2, ≤ 64 Expansionen).
2. Postings in 128er-Blöcken, Delta + SIMD-Bitpacking; optional Positionen.
3. Je Block die **Impact-Grenzen** `(tf, fieldnorm)` als Pareto-Menge je Feld.
4. Feldnormen (1 B je Feld und Dokument).

Block-Max-Schranken werden **zur Abfragezeit** aus den Impact-Grenzen und den aktuellen globalen Statistiken berechnet. Zur Bauzeit berechnete Scores sind unzulässig: Ändern sich `avgdl` oder `df`, wären sie keine oberen Schranken mehr, und Block-Max-WAND lieferte falsche Top-k.

### 3.2 BM25F

`k1 = 1.2`; je Feld `b = 0.75`. Gewichte: `title`/`zoom.L2` 3,0 · `tags` 2,0 · `heading_path` 1,5 · `body` 1,0 · `edge_labels` 0,5. IDF in der nichtnegativen Form `ln(1 + (N − df + 0.5)/(df + 0.5))`.

`(N, avgdl_feld, df_term)` werden über alle Segmente des Manifests plus Delta aggregiert. Bis zur Compaction zählen ersetzte Dokumente mit (dokumentierte Näherung). Scores sind je (Manifest, Delta) deterministisch.

### 3.3 Statistik-Isolation

Globale IDF-Werte verraten indirekt Termvorkommen in unsichtbaren Dokumenten. Daher je Namespace:

| `idf_isolation` | Statistikbasis | Eigenschaft |
|---|---|---|
| `namespace` (Default) | Namespace; Zugriff nur, wenn der Principal den Namespace lesen darf | innerhalb gemischter ACLs ist ein Score-Seitenkanal möglich und dokumentiert |
| `acl-class` | getrennte Statistiken je `AclRef`-Klasse, aggregiert über sichtbare Klassen | kein Einfluss unsichtbarer Dokumente; höherer Speicher- und CPU-Bedarf; ≤ 1 024 Klassen |

Namespaces mit vertraulichen, gemischten ACLs MÜSSEN `acl-class` verwenden. INV-5-Tests laufen in diesem Modus auf Score-Ebene.

### 3.4 Analyzer

UAX-29-Segmentierung → NFC/Lowercasing → Umlaut-/ß-Faltung (zusätzlicher Token, Original bleibt) → Spracherkennung je Chunk → Snowball (`de`, `en`) → deutsche Komposita-Zerlegung (DP mit Fugenelementen, Wörterbuch per Digest gepinnt) → CamelCase/snake_case und N-Gramme für Bezeichner. Analyzer-ID inklusive Datendigest ist Teil des Segmentschlüssels; Änderung ⇒ Rebuild. Abfragen nutzen den Analyzer des Segments.

---

## 4. Vektor (`contextra-vector`)

### 4.1 Stufen

```
Commit/Derivation → Branch-Delta (Flat-SIMD bis 20 000, darüber RAM-HNSW)
   → Index-Job → Segment: Flat (< 50 000) | HNSW | Vamana
   → größengestufte Compaction (Faktor 4–8, ≤ 8·10⁶ je Segment)
```

### 4.2 Parameter und Layout

| | HNSW (RAM) | Vamana (SSD) |
|---|---|---|
| Graph | `M=16`, `efC=200` | `R=64`, `L_build=125`, `α=1.2` |
| Suche | `efSearch` 16–256 adaptiv | Beam 8, `L_search` 100 (Start) |
| RAM je Vektor | Code + Graph | **Code aller Knoten** + `gid` |
| Auf SSD | — | je Knoten: Vollvektor `f16` + Grad + `R` Nachbar-`DocId`s |
| Rerank | Top-N gegen `emb.full` | exakt aus dem gelesenen Sektor |

Beam-Search bewertet Nachbarn über die **RAM-residenten Codes** und liest Sektoren nur für expandierte Knoten. Sektorbelegung bei R = 64 (260 B Adjazenz): 768 Dim ⇒ 1 796 B ⇒ 2 Knoten je 4 KiB; 1 024 Dim ⇒ 2 308 B ⇒ 1 Knoten; 3 072 Dim ⇒ 6 404 B ⇒ 2 Sektoren je Knoten. Vollvektoren im Segment sind eine Kopie zur I/O-Vermeidung.

### 4.3 Quantisierung

| Verfahren | RAM bei 768 Dim | Einsatz |
|---|---|---|
| SQ8 (0,5/99,5-Perzentile) | 768 B | wenn Budget reicht |
| PQ M=64, 8 bit | 64 B | große Bestände |
| RaBitQ 1 bit + Korrekturfaktoren | ≈ 96 B + ≤ 16 B | große Bestände |

Der Segment-Builder wählt anhand des verbleibenden `fixed`-Budgets (Reihenfolge SQ8 → RaBitQ/PQ) und MUSS die Recall-Prüfung bestehen, sonst bleibt das alte Manifest aktiv. Parameter und Seed stehen im Segmentheader; keine Online-Rekalibrierung. Optionaler Matryoshka-Präfix (128–256 Dim) als Vorfilter, nur für Producer, die ihn unterstützen.

Spaces werden nie gemischt oder verglichen. Modellwechsel: neuer Space, Doppelbetrieb, Backfill nach Salience; Abfragen fusionieren **Ränge**, nicht Distanzen.

### 4.4 Gefilterte Suche

`S` = erlaubte `DocId`s, `σ = |S|/N`.

| Strategie | Bedingung (in dieser Reihenfolge geprüft) | Verfahren |
|---|---|---|
| **A** exakt | `|S| ≤ 20 000` oder `σ < 0.01` mit `|S| ≤ 200 000` | SIMD über Codes, exakter Rerank |
| **B** In-Filter | `σ < 0.5` | Beam mit Prädikat; `σ < 0.1` mit Zwei-Hop-Expansion (ACORN-Prinzip) |
| **C** Post-Filter | sonst | Over-Fetch `ceil(k/σ)` plus Marge |

Fallback: Liefert B weniger als `min(k, |S|)` gültige Treffer innerhalb des Expansionsbudgets, wird A ausgeführt (deadlinegebunden). Unsichtbare Knoten dürfen als Routing-Knoten dienen, erscheinen aber nie in Ergebnissen oder Zählern. Parallele Segmentsuchen teilen die k-te Score-Schranke nur zwischen Scores **derselben Metrik und desselben Space**.

### 4.5 Validierung der Eingaben, Löschungen, Compaction

NaN, ±∞, falsche Dimension, Nullvektor bei Cosine ⇒ `vector_invalid`. Gelöschte Knoten bleiben Routing-Knoten. Compaction bei > 8 Segmenten je Stufe und Space, > 30 % toten Einträgen oder Übergröße. Sichtbarer Fan-out je Query ≤ 24; darüber erhält Compaction Priorität.

---

## 5. Graph (`contextra-graph`)

### 5.1 Truth und CSR

Wahrheit ist der Baum `graph`. CSR-Segmente (vorwärts und rückwärts): Elias-Fano-Offsets, delta-bitgepackte Nachbarn, Spalten für Art, Gewicht, Gültigkeit, Trust; Skip-Index für Grad > 10⁴. Bau ab > 10 000 Kanten oder > 20 % Änderung; Delta-Overlay bis dahin.

Eine Kante ist sichtbar, wenn sie selbst und **beide Endpunkte** im Snapshot sichtbar sind. `retire` kaskadiert nicht; G1 meldet hängende Kanten. Traversierungen prüfen die Sichtmaske **bei jeder Expansion**: Pfade führen nie durch unsichtbare Knoten.

### 5.2 Operatoren mit Pflichtlimits

| Limit | Default | Hart |
|---|---|---|
| Hops | 3 | 5 |
| Fan-out je Knoten | 50 | 500 |
| Besuchte Knoten | 100 000 | 100 000 |
| Pfade | 10 000 | 10 000 |
| Deadline | 200 ms | 2 s |

Überschreitung ⇒ Teilergebnis mit `truncated = true`. Reihenfolge der Expansion ist deterministisch (Gewicht absteigend, dann `Gid`), damit abgeschnittene Ergebnisse reproduzierbar sind; Deadline-Abbrüche sind davon ausgenommen und gekennzeichnet.

**Forward-Push Personalized PageRank** (einziger PPR-Standard): Dämpfung `d = 0.85` (Teleport 0,15), Residuum-Schwelle `ε = 1e-4` relativ zum Ausgangsgrad, `max_pushes = 10⁶`, `top_k = 100`.

**Pfadkonfidenz** ist eine Min-Schranke: `C(P) = min_i conf(e_i)`; unabhängige Pfade (disjunkte `source_root`s) werden durch das Maximum zusammengefasst. Ein Hop-Dämpfungsfaktor `γ = 0.9` darf das **Ranking** beeinflussen, nicht die ausgewiesene Konfidenz.

**Leiden** läuft als geseedeter, deterministischer Batch-Job; Community-Zugehörigkeit steht im IndexManifest, boostet optional (× 1,2) und speist `Summary`-Cells. Community-Zusammenfassungen unterliegen INV-13; über ACL-Klassen hinweg wird getrennt geclustert oder konjunktiv vererbt.

---

## 6. Temporal, Attribute, Sequenzen

- **Bitmaps** je Manifest: `NsId`, Kind, Trust, Taint, `AclRef`, Tags, Enum-Attribute.
- **Zeitindex:** abgeleitete, nach `valid_from` sortierte Segmente mit Blockzusammenfassung `min/max(valid_to)`. Er ist ein Index, kein achter `RootSet`-Baum.
- **Extent-Bloom** (8 B je Cell) zum frühen Ausschluss; Treffer werden exakt nachgeprüft.
- **Token-Summen:** `window(seq, anchor, budget, dir, tokenizer)` summiert Knotenaggregate in O(log n) und schneidet an Cell-Grenzen. Unsichtbare Cells werden ausgeschlossen: Enthält ein Teilbaum gemischte Sichtbarkeit, wird abgestiegen; ausgegebene Summen zählen nur Sichtbares. Das O(log n)-Ziel gilt für homogene Sichtbarkeit.

---

## 7. Lifecycle

| Struktur | Ort | Aktualisierung | Wiederherstellung |
|---|---|---|---|
| `RootSet`-Bäume | `packs/truth` | Delta → Fold | WAL |
| RAM-Deltas (`vec`, `fts`, `graph`, `bitmap`) | Branch | bei Publikation | aus WAL/Delta |
| Embeddings, Summaries | Derivation Cache | `DerivationPut` | WAL/Pack, sonst neu berechnen |
| Index-Segmente | `packs/index` | Hintergrundjob | jederzeit aus Truth + Derivationen |

Index-Builds laufen im `jobs`-Budget, sind abbrechbar und setzen an Segmentgrenzen fort.

---

## 8. Abnahme

- Recall@10 ≥ 0,95 gegen exakte Suche je Quantisierung **und** je Selektivitätsklasse σ ∈ {0,001; 0,01; 0,1; 0,5; 1}.
- Differenzial: BM25 gegen Referenzimplementierung; Block-Max-WAND gegen erschöpfendes Scoring unter wechselnden Statistiken (identische Top-k); PPR gegen Power-Iteration innerhalb der ε-Schranke.
- Property: `Gid→DocId` gegen naive Referenz; verspätete Derivate werden gefunden.
- Index-Verlust-Test: identische Kandidatenmengen unterhalb der Fallback-Grenze; darüber korrekte Degradationsmeldung; vollständiger Rebuild.
- ACL-Suite: keine unsichtbare `Gid` in Treffern, Pfaden, Zählern.

---

*Weiter: [SPEC-06 Context Engine](SPEC-06-CONTEXT-ENGINE.md)*
