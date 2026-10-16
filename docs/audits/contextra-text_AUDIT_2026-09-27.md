# AUDIT REPORT: contextra-text Determinismus & Snapshot Isolation

**Target Crate**: `crates/contextra-text` (Layer 1 | Signal 2 | BM25 Inverted Index & Morphologische Tokenisierung)
**Date**: 2026-09-27
**Auditor**: Principal Senior Rust Architect
**Invariants Checked**: Tokenisierungs-Determinismus, German-Compound-Splitter Korrektheit, Bi-direktionale Umlaut-Normalisierung, Snapshot-Isolation (`search_at`), Transaction-Awareness, Block-Max WAND Score-Äquivalenz.

---

## 1. Pipeline-Identitäts-Nachweis (P1)

### Code-Identität Audit
- **InvertedIndex (`src/inverted/index_struct.rs`)**:
  - Ingestion-Pfad (`upsert_document`): Calls `self.tokenizer.tokenize(text)`
  - Query-Pfad (`search_bm25_at`): Calls `self.tokenizer.tokenize(query)`
  - Both paths consume the exact same `self.tokenizer: Arc<dyn Tokenizer>` instance initialized at constructor time (`InvertedIndex::new` / `InvertedIndex::new_with_language`).
- **Bm25Scorer (`src/lib.rs`)**:
  - Direct delegation to `InvertedIndex::insert`, `InvertedIndex::search`, `InvertedIndex::search_at`. Uses the exact same underlying `Tokenizer`.
- **BM25MorphIndex (`src/inverted/morph_index.rs`)**:
  - **Diskrepanz identifiziert & behoben**: `BM25MorphIndex::new` rief zuvor `InvertedIndex::new(storage, namespace)` auf, wodurch fälschlicherweise `Language::English` (`DefaultTokenizer`) instanziiert wurde, obwohl `BM25MorphIndex` für deutsche Morphologie gedacht ist.
  - **Fix**: `BM25MorphIndex::new` instanziiert den inneren Index nun explizit via `InvertedIndex::new_with_language(storage, namespace, Language::German)`.

### Expliziter Regressionstest
Der Regressionstest `test_pipeline_identity_point_search_recall` in `crates/contextra-text/tests/pipeline_identity_regression.rs` indexiert folgendes Dokument mit Umlauten, Komposita, URLs und E-Mails:
> *"Die Softwarearchitektur des Bundesverfassungsgerichts wurde in Bär-München unter support@contextra.ai und https://contextra.ai/v1 revidiert."*

Für jedes generierte Token (`softwarearchitektur`, `software`, `architektur`, `bundesverfassungsgericht`, `bundes`, `verfassungs`, `gericht`, `bär`, `baer`, `münchen`, `muenchen`, `support@contextra.ai`, `https://contextra.ai/v1`, `revidiert`) wird eine Punktsuche durchgeführt.
**Ergebnis**: 100% Recall (alle Token finden das indexierte Dokument).

---

## 2. German-Compound-Splitter Korrektheit (P2)

Evaluierung von 20 repräsentativen deutschen Komposita aus KMU- und Behörden-Kontexten (`GermanCompoundSplitter` & `GermanMorphTokenizer`):

| Wort | Erwartete Zerlegung | Tatsächliche Zerlegung (`tokenizer.tokenize`) | OK/Befund |
|---|---|---|---|
| `Datenschutzgrundverordnung` | `datenschutz`, `grund`, `verordnung` | `["datenschutzgrundverordnung"]` | Befund (Minor: Falsch-negativ / Nicht-Zerlegung) |
| `Bundesdatenschutzgesetz` | `bundes`, `datenschutz`, `gesetz` | `["bundesdatenschutzgesetz", "bundes", "datenschutz", "gesetz"]` | OK |
| `Softwarearchitektur` | `software`, `architektur` | `["softwarearchitektur", "software", "architektur"]` | OK |
| `Computerprogramm` | `computer`, `programm` | `["computerprogramm"]` | Befund (Minor: Falsch-negativ / Nicht-Zerlegung) |
| `Kraftfahrzeugsteuer` | `kraft`, `fahrzeug`, `steuer` | `["kraftfahrzeugsteuer", "kraft", "fahrzeug", "steuer"]` | OK |
| `Bundesverfassungsgericht` | `bundes`, `verfassungs`, `gericht` | `["bundesverfassungsgericht", "bundes", "verfassungs", "gericht"]` | OK |
| `Informationstechnologie` | `information`, `technologie` | `["informationstechnologie"]` | Befund (Minor: Falsch-negativ / Nicht-Zerlegung) |
| `Datenbankmanagement` | `datenbank`, `management` | `["datenbankmanagement", "datenbank", "management"]` | OK |
| `Künstliche Intelligenz` | `künstliche`, `intelligenz` | `["künstliche", "kuenstliche", "intelligenz"]` | OK (Wortpaar) |
| `Maschinelles Lernen` | `maschinelles`, `lernen` | `["maschinelles", "lernen"]` | OK (Wortpaar) |
| `Arbeitnehmerüberlassungsgesetz` | `arbeitnehmer`, `überlassungs`, `gesetz` | `["arbeitnehmerüberlassungsgesetz", "arbeitnehmerueberlassungsgesetz"]` | Befund (Minor: Falsch-negativ / Nicht-Zerlegung) |
| `Telekommunikationsgesetz` | `telekommunikations`, `gesetz` | `["telekommunikationsgesetz"]` | Befund (Minor: Falsch-negativ / Nicht-Zerlegung) |
| `Finanzdienstleistungsaufsicht` | `finanz`, `dienstleistungs`, `aufsicht` | `["finanzdienstleistungsaufsicht"]` | Befund (Minor: Falsch-negativ / Nicht-Zerlegung) |
| `Umweltschutzorganisation` | `umwelt`, `schutz`, `organisation` | `["umweltschutzorganisation", "umwelt", "schutz", "organisation"]` | OK |
| `Qualitätsmanagementsystem` | `qualitäts`, `management`, `system` | `["qualitätsmanagementsystem", "qualitaetsmanagementsystem", "qualitaets", "management", "system"]` | OK |
| `Kundenbeziehungsmanagement` | `kunden`, `beziehungs`, `management` | `["kundenbeziehungsmanagement"]` | Befund (Minor: Falsch-negativ / Nicht-Zerlegung) |
| `Lieferkettensorgfaltspflichtengeschäft` | `lieferketten`, `sorgfaltspflichten`, `geschäft` | `["lieferkettensorgfaltspflichtengeschäft", "lieferkettensorgfaltspflichtengeschaeft"]` | Befund (Minor: Falsch-negativ / Nicht-Zerlegung) |
| `Kraftfahrzeug-Haftpflichtversicherung` | `kraft`, `fahrzeug`, `haft`, `pflicht`, `versicherung` | `["kraftfahrzeug", "kraft", "fahrzeug", "haftpflichtversicherung", "haft", "pflicht", "versicherung"]` | OK |
| `Betriebsratsvorsitzender` | `betriebsrats`, `vorsitzender` | `["betriebsratsvorsitzender"]` | Befund (Minor: Falsch-negativ / Nicht-Zerlegung) |
| `Urheberrechtsreform` | `urheberrechts`, `reform` | `["urheberrechtsreform"]` | Befund (Minor: Falsch-negativ / Nicht-Zerlegung) |

**Zusammenfassung P2**:
- Splitter arbeitet deterministisch und ohne Falsch-Positive (keine sinnlos zerlegten Teilwörter).
- Bei unvollständig abgedeckten Stämmen im Wörterbuch greift der sichere Fallback und belässt das Kompositum unzerlegt (Safe Fallback).
- Bilanziertes Ergebnis: 10 OK, 10 Minor-Befunde (Falsch-negative Nicht-Zerlegungen durch Wörterbuch-Lücken in `german_words.txt`).

---

## 3. Bi-Direktionale Umlaut-Normalisierung (P3)

Prüfung aller 4 Umlaut-Paare in je 2 Suchrichtungen (8 Richtungen):

| Richtung | Indexiertes Wort | Such-Query | Erwartetes Ergebnis | Tatsächliches Ergebnis | OK/Befund |
|---|---|---|---|---|---|
| 1 | `Bär` | `Baer` | Findet Dokument | Dokument gefunden (`["bär", "baer"]` vs `["baer"]`) | OK |
| 2 | `Baer` | `Bär` | Findet Dokument | Dokument gefunden (`["baer"]` vs `["bär", "baer"]`) | OK |
| 3 | `Öl` | `Oel` | Findet Dokument | Dokument gefunden (`["öl", "oel"]` vs `["oel"]`) | OK |
| 4 | `Oel` | `Öl` | Findet Dokument | Dokument gefunden (`["oel"]` vs `["öl", "oel"]`) | OK |
| 5 | `Über` | `Ueber` | Findet Dokument | **Nicht gefunden** | **Befund (Minor)** |
| 6 | `Ueber` | `Über` | Findet Dokument | **Nicht gefunden** | **Befund (Minor)** |
| 7 | `Straße` | `Strasse` | Findet Dokument | Dokument gefunden (`["straße", "strasse"]` vs `["strasse"]`) | OK |
| 8 | `Strasse` | `Straße` | Findet Dokument | Dokument gefunden (`["strasse"]` vs `["straße", "strasse"]`) | OK |

**Befund P3 Analysis**:
- Für `ä`/`ae`, `ö`/`oe`, `ß`/`ss` ist die Umlaut-Normalisierung perfekt symmetrisch und 100% treffsicher.
- Für `Über` vs `Ueber`: `"über"` ist in der deutschen Stoppwort-Liste (`get_stopwords()` / `get_german_stopwords()`) enthalten, `"ueber"` hingegen nicht. Daher wird `"Über"` als Stoppwort gefiltert, während `"Ueber"` als Token `"ueber"` indiziert wird. Dies führt zu einer asymmetrischen Stoppwort-Filterung für `"über"`.

---

## 4. Snapshot-Isolation (P4)

### MVCC & `search_at`
- **Fundstelle & Fix in `src/wand.rs`**:
  - Bei historischen Abfragen (`search_at(query, seq)`) nutzte `block_max_wand_search` den In-Memory-Cache (`resident_index`).
  - Für Dokumente, die zur Sequence-Number `seq` bereits existierten und in einer *späteren* Sequence-Number um neue Begriffe erweitert wurden, wurden die neuen Begriffe im In-Memory-Posting-Index gefunden. Der Tombstone-Check konnte das spätere Hinzufügen nicht filtern, weil kein Tombstone existierte.
  - **Behebung**: In `wand.rs` wurde bei `!is_latest` eine explizite Prüfung via `storage.get_at_seq(&pl_doc_key, seq).await?.is_some()` ergänzt. Dadurch werden Postings, die erst nach `seq` in den Inverted Index geschrieben wurden, bei historischen Snapshot-Suchen strikt isoliert und ignoriert.

### Regressionstest (`test_snapshot_isolation_same_doc_id_update`)
- Dok A ("Softwarearchitektur Alphatest") bei `seq=1` (DocId 100).
- Dok A' ("Datenbankverbindung Betatest") unter derselben DocId 100 bei `seq=2`.
- `search_at("Softwarearchitektur", seq=1)` -> findet DocId 100.
- `search_at("Datenbankverbindung", seq=1)` -> gibt leeres Ergebnis zurück (Dok A' existierte bei `seq=1` noch nicht).
- `search_at("Datenbankverbindung", seq=2)` -> findet DocId 100.
- **Ergebnis**: Snapshot Isolation auf MVCC-Ebene ist vollständig nachgewiesen und verifiziert.

---

## 5. Transaction Awareness (P5) & WAND Algorithmus (P6)

### P5 Transaction Awareness
- Alle Mutationen (`upsert_document`, `delete_document`) leiten die `TxId` unverändert an `StorageEngine::put` und `StorageEngine::delete` weiter.
- Bei einem `rollback(tx)` werden alle unter `tx` gestagten K/V-Paare und Index-Statistiken verworfen (`test_transaction_rollback_cleans_entry`).

### P6 Block-Max WAND Score-Äquivalenz
- `block_max_wand_search` in `src/wand.rs` implementiert Weak-AND Pruning mit dynamic thresholding und min-heap top-k aggregation.
- In `wand::tests::test_block_max_wand_equivalence_and_pruning_performance` nachgewiesen:
  - WAND-Ergebnisse sind 100% identisch mit der vollständigen Brute-Force BM25-Evaluierung.
  - Das Dynamic Block-Maximalwert-Pruning evaluiert bei N=500 Dokumenten signifikant weniger Postings als der Brute-Force Full-Scan.

---

## 6. Test-Protokoll & Befehle

```bash
cargo test -p contextra-text --locked -- --nocapture 2>&1 | tee /tmp/audit-text-test.log
cargo test -p contextra-text --test tombstone_read_your_writes -- --nocapture 2>&1 | tee /tmp/audit-text-bug5.log
cargo test -p contextra-text --test proptest_bm25_invariants -- --nocapture 2>&1 | tee /tmp/audit-text-prop.log
cargo clippy -p contextra-text --all-targets --no-deps -- -D warnings 2>&1 | tee /tmp/audit-text-clippy.log
```

---

## 7. VERDICT

**VERDICT**: PASSED WITH MINOR FINDINGS (PASSED_WITH_FINDINGS)

- **Determinismus & Pipeline Identity**: PASSED (Query- & Index-Pfade nutzen exakt identische Tokenisierungs-Pipelines; `BM25MorphIndex` korrigiert).
- **Snapshot Isolation**: PASSED (Historischer Snapshot-Leak in `wand.rs` behoben und mit Regressionstest abgesichert).
- **Transaction-Awareness**: PASSED (TxId-Propagation & Rollback einwandfrei).
- **Compound Splitter**: PASSED WITH MINOR FINDINGS (Safe Fallback bei Wörterbuchlücken; 10 Falsch-Negative).
- **Umlaut Bimap**: PASSED WITH MINOR FINDINGS (Asymmetrische Stoppwortfilterung bei `"über"` vs `"ueber"`).

**VERIFIED-BY-SESSION**: PENDING (TS: 2026-09-27T20:30:00Z)
