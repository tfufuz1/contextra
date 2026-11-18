# contextra-text — Crate-Spezifikation (Invertierter Volltextindex)

**Ebene:** E1a (Kern-Closure: `ids`, `env`)  
**Status:** Normativ  
**Version:** 1.0  
**Geltungsbereich:** `crates/contextra-text`  
**Referenz:** `docs/SPEC.md` §2, §3, §5, §6, §7

---

## 1. Zweck & Systemrolle

`contextra-text` implementiert die Volltextsuche von Contextra mit speziellem Fokus auf die deutsche Sprache:
- **Deutsche Linguistik:** Exakte Normalisierung (ä/ö/ü/ß, Gross-/Kleinschreibung), Stoppwort-Filterung, Porter/Snowball-Stemming und deterministische Zerlegung deutscher Komposita (z.B. „Haftpflichtversicherungsvertrag“ → [haft, pflicht, versicherung, vertrag]).
- **Posting-Listen & Relevanz:** Kompaktierte Inverted Lists mit Block-Max-Metadaten für schnelles Pruning via Block-Max-WAND (BMW).
- **BM25 & BM25F Scoring:** Relevanzbewertung mit konfigurierbaren Parametern ($k_1$, $b$) und Feldgewichtungen (Titel, Body, Tags).
- **Löschung & Segment-Rewrite:** Physisches Entfernen von Term-Einträgen aus Inverted Segments im Rahmen der DSGVO-Löschkette.

---

## 2. Architektur & Datenmodell

```
┌─────────────────────────────────────────────────────────────┐
│                        contextra-text                       │
│                                                             │
│   ┌─────────────────────────────────────────────────────┐   │
│   │                    InvertedIndex                    │   │
│   │                                                     │   │
│   │   ┌───────────────────┐     ┌───────────────────┐   │   │
│   │   │ GermanTokenizer   │     │ Decompounder      │   │   │
│   │   │ (Unicode Segment.)│     │ (Dictionary-based)│   │   │
│   │   └─────────┬─────────┘     └─────────┬─────────┘   │   │
│   │             │                         │             │   │
│   │             ▼                         ▼             │   │
│   │   ┌─────────────────────────────────────────────┐   │   │
│   │   │ Inverted Segments (Disk / Mmap)             │   │   │
│   │   │ Term: "kuendigung" ──► [doc_1, doc_4, ...]  │   │   │
│   │   │ Term: "frist"      ──► [doc_1, doc_9, ...]  │   │   │
│   │   └─────────────────────────────────────────────┘   │   │
│   │                                                     │   │
│   │   ┌───────────────────┐     ┌───────────────────┐   │   │
│   │   │ Block-Max-WAND    │     │ ErasureLayer Impl │   │   │
│   │   │ Query Evaluator   │     │ (Segment-Rewrite) │   │   │
│   │   └───────────────────┘     └───────────────────┘   │   │
│   └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

---

## 3. Schutz personenbezogener Daten (Klasse D)

- **Posting-Listen als Ableitungen:** Ein Volltextindex speichert Wortstämme und Positionen, die direkt aus Texten stammen. Da Namen, Telefonnummern oder E-Mail-Adressen als Terme im Index indiziert werden können, gelten Posting-Listen als schutzbedürftige abgeleitete Daten (Klasse D).
- **Lineage & Kaskadierung:** Jeder Eintrag in einer Posting-Liste verweist auf eine `DocId` oder `PersonalRef`.

---

## 4. DSGVO-Löschkette: ErasureLayer-Implementierung

`contextra-text` implementiert `contextra_ports::erasure::ErasureLayer` (Priorität 4b — Text-Segment-Rewrite):

```rust
impl ErasureLayer for InvertedIndex {
    fn id(&self) -> LayerId {
        LayerId(*b"CTX_TEXT_BM25_01")
    }

    fn key_coverage(&self) -> KeyCoverage {
        KeyCoverage::Full
    }

    fn resolve(&self, target: &EraseTarget) -> BoxFuture<'_, Result<LayerScope, String>>;
    fn on_gate(&self, scope: &LayerScope) -> BoxFuture<'_, Result<(), String>>;
    fn purge(&self, scope: &LayerScope, cx: &PurgeCx) -> BoxFuture<'_, Result<PurgeOutcome, String>>;
    fn witness(&self, scope: &LayerScope, sc: &Scanner<Armed>) -> BoxFuture<'_, Result<LayerWitness, String>>;
}
```

### 4.1 Die 4 Phasen in `contextra-text`

#### 1. `resolve(target)`
- Mappt das `EraseTarget` auf die Menge der betroffenen `DocId`s.
- Ermittelt die Segment-Dateien (`.idx`, `.post`), in denen diese Dokumente vorkommen.

#### 2. `on_gate(scope)`
- Fügt die gelöschten `DocId`s einer In-Memory `RoaringBitmap` (`deleted_doc_filter`) hinzu.
- Ab diesem Zeitpunkt werden Treffer dieser Dokumente bei Block-Max-WAND Auswertungen sofort übersprungen.

#### 3. `purge(scope, cx)` — Physisches Segment-Rewrite
Unter exklusivem Halt der `PurgeBarrier`:
- **Iterativer Segment-Scan:** Für jedes betroffene Segment wird eine neue, temporäre Posting-Datei geschrieben.
- **Posting-Filterung:** Alle Postings, die zu gelöschten `DocId`s gehören, werden vollständig verworfen.
- **Term-Statistik-Korrektur:** Dokumentfrequenzen ($df$) und Dokumentlängen ($dl$) werden neu berechnet, um Korpus-Statistiken konsistent zu halten.
- **Atomarer Austausch:** Atomares Rename der bereinigten Segmente und Synchronisation via `fsync` (`contextra-durable-fs`).
- **Dateiverschrottung:** Alte Index-Segmente werden mit Nullen überschrieben und gelöscht.

#### 4. `witness(scope, sc)` — Verifikation & Messung
- Scannt die Posting-Listen nach verbliebenen Vorkommen der gelöschten `DocId`s.
- Verifiziert: `residue_matches == 0`.

---

## 5. Synchroner Lesepfad: ReadGate-Integration

```rust
pub fn search_bm25(&self, query_terms: &[Term], top_k: usize) -> Result<Vec<TextHit>, ContextraError> {
    // 1. Initialisiere Block-Max-WAND Scorer
    // 2. Iteriere über Kandidatenblöcke
    while let Some(candidate_doc) = wand_scorer.next_candidate() {
        // Prüfe In-Memory-Maske & ReadGate
        if self.deleted_filter.contains(candidate_doc.id) {
            continue;
        }
        if self.read_gate.check_doc_id(candidate_doc.id) == GateDecision::Erased {
            continue;
        }
        // Score berechnen und in Top-k Heap einfügen
    }
    // 3. Auffüllen auf top_k garantieren
}
```

---

## 6. Invarianten

1. **`#![forbid(unsafe_code)]`:** Vollständig sichere Rust-Implementierung.
2. **Determinismus (INV-5):** Exakt gleiche Score-Berechnung und Sortierung bei gleichen Eingaben. Tie-Break nach `DocId` aufsteigend.
3. **Keine Panics:** Fehlerhafte UTF-8-Sequenzen werden strikt behandelt (`InvalidInput`), ohne Abstürze.
