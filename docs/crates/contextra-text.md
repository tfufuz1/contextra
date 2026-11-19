> **Löschpfad v3.0:** Die löschrelevanten Abschnitte dieser Spezifikation sind mit Stand 19.11.2026 durch `docs/SPEC.md` v3.0 ersetzt bzw. präzisiert (§5, §6, §7, §9, §12, §13, §14, §16 der SPEC). Widerspricht ein Abschnitt unten der SPEC, gilt die SPEC. Abschnitte, die so markiert sind, sind überholt. Weitere Verweise: §23 SPEC (Ersetzungstabelle).

# contextra-text — Implementierungsspezifikation (Soll-Zustand)

**Ebene:** E1 (Domänenbaustein / Ring 1)  
**Kern-Closure:** `ids`, `env`, `durable-fs` (E0-Fundamente, max. 5)  
**Status:** Normativ (Soll-Zustand für Entwickler)  
**Version:** 2.0  
**Geltungsbereich:** `crates/contextra-text`  
**Referenzdokumente:** `docs/SPEC.md` §2, §3, §5.3 (Schritt 4b), §6, §7; `ANALYSIS-E0-CONTRADICTIONS.md`; `ALTLASTEN-AUDIT.md` (AP-21)  

---

## 1. Zweck & Architekturgarantien

`contextra-text` implementiert die hochperformante Volltext-Suchmaschine von Contextra mit speziellem Fokus auf die deutsche Sprache und Rechts-/Medizinkorpora:
- **Deutsche Morphologie & Kompositazerlegung:** Deterministische Zerlegung deutscher Bandwurmwörter (z. B. „Schadensersatzanspruch“ → [schaden, ersatz, anspruch]), Umlaut-Normalisierung und Stemming.
- **Invertierte Indizes & Block-Max WAND (BMW):** Speichereffiziente Posting-Listen mit dynamischem Pruning für Sub-Millisekunden-Top-K-Suchen.
- **BM25 / BM25F Relevanzbewertung:** Konfigurierbare Parameter ($k_1$, $b$) und Feldgewichtungen (Titel, Body, Annotationen).
- **Physisches Segment-Rewrite (DSGVO Phase 4b):** Vollständiges Umschreiben von Posting-Dateien ohne gelöschte `DocId`s und Bereinigung von Term-Statistiken.

### Beseitigung von Ebenenverstößen & Altlasten (R1, R3)
1. **Keine E2-Abhängigkeiten:** Die optionalen Abhängigkeiten zu `contextra-types` und `contextra-ports` sowie die Features `workspace`, `types-compat` und `ports-compat` werden vollständig entfernt.
2. **Leere Default-Features (R3):** `default = []`.
3. **Altlasten-Bereinigung (`AP-21`):** In `src/lib.rs` Zeile 25 werden alle Bezüge zu `contextra-router` und `contextra-agent` gelöscht.

### Architekturgarantien (R1–R6, INV-1, INV-3, INV-4, INV-5)
1. **`#![forbid(unsafe_code)]`:** 100 % speichersicher.
2. **Zero-Panic-Policy:** Alle Operationen liefern `TextError`.
3. **Deterministisches Ranking (INV-5):** Exakt gleiche Scores und Tie-Breaking nach `DocId` aufsteigend.
4. **Schutz abgeleiteter Daten (Klasse D):** Posting-Listen unterliegen als abgeleitete Daten der kaskadierten Löschung.

---

## 2. Architektur & Komponenten

```
┌─────────────────────────────────────────────────────────────┐
│                        contextra-text                       │
│                                                             │
│   ┌─────────────────────────────────────────────────────┐   │
│   │                    InvertedIndex                    │   │
│   │                                                     │   │
│   │   ┌───────────────────┐     ┌───────────────────┐   │   │
│   │   │  GermanTokenizer  │     │ CompoundSplitter  │   │   │
│   │   │ (Unicode Segment.)│     │ (Wörterbuch-Trie) │   │   │
│   │   └─────────┬─────────┘     └─────────┬─────────┘   │   │
│   │             │                         │             │   │
│   │             ▼                         ▼             │   │
│   │   ┌─────────────────────────────────────────────┐   │   │
│   │   │ Inverted Segments (Disk / Memmap)           │   │   │
│   │   │ Term: "haftung"   ──► [doc_1, doc_4, ...]   │   │   │
│   │   │ Term: "ausschluss"──► [doc_1, doc_9, ...]   │   │   │
│   │   └─────────────────────────────────────────────┘   │   │
│   │                                                     │   │
│   │   ┌───────────────────┐     ┌───────────────────┐   │   │
│   │   │  Block-Max-WAND   │     │  SegmentRewriter  │   │   │
│   │   │  Query Evaluator  │     │ (DSGVO Phase 4b)  │   │   │
│   │   └───────────────────┘     └───────────────────┘   │   │
│   └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

---

## 3. DSGVO-Löschkette: Text-Segment-Rewrite (Phase 4b)

Posting-Listen dürfen nach einer Löschung keine Reste der gelöschten Dokumente enthalten:

```rust
use contextra_ids::DocId;
use std::path::{Path, PathBuf};

pub struct SegmentRewritePlan {
    pub segment_id: u64,
    pub doc_ids_to_purge: Vec<DocId>,
}

pub struct SegmentRewriteOutcome {
    pub segment_id: u64,
    pub postings_removed: u64,
    pub bytes_reclaimed: u64,
}

pub struct TextSegmentRewriter;

impl TextSegmentRewriter {
    /// Führt das physische Umschreiben eines Posting-Segments aus (Phase 4b).
    ///
    /// 1. Schreibt neue Posting-Listen in temporäre Datei (ohne doc_ids_to_purge).
    /// 2. Passt Dokumentfrequenzen (df) und Feldlängen an.
    /// 3. Führt atomares Rename und Directory-Fsync aus (contextra-durable-fs).
    /// 4. Überschreibt alte Segment-Datei mit Nullen (Scrubbing).
    pub fn rewrite_segment(
        segment_path: &Path,
        docs_to_purge: &[DocId],
    ) -> Result<SegmentRewriteOutcome, TextError> {
        // Implementierung des atomaren Rewrites
        let parent = segment_path.parent().ok_or(TextError::InvalidPath)?;
        let tmp_path = parent.join(format!(".{}.rewrite.tmp", segment_path.file_name().unwrap().to_str().unwrap()));

        // Iteration über Postings, Ausschluss von docs_to_purge, Schreiben nach tmp_path
        // Atomares Rename via contextra-durable-fs
        Ok(SegmentRewriteOutcome {
            segment_id: 1,
            postings_removed: docs_to_purge.len() as u64,
            bytes_reclaimed: 0,
        })
    }
}
```

---

## 4. Relevanz-Scoring (BM25)

$$BM25(D, Q) = \sum_{t \in Q} IDF(t) \cdot \frac{f(t, D) \cdot (k_1 + 1)}{f(t, D) + k_1 \cdot \left(1 - b + b \cdot \frac{|D|}{\text{avgdl}}\right)}$$

- $k_1 = 1.2$: Term-Frequenz-Sättigung.
- $b = 0.75$: Dokumentlängen-Normalisierung.
- Tie-Breaking: Bei identischem BM25-Score gewinnt stets die kleinere `DocId`.

---

## 5. Fehlerbehandlung (`TextError`)

```rust
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum TextError {
    #[error("Segment file corrupted: {0}")]
    CorruptedSegment(String),

    #[error("Tokenizer error: {0}")]
    Tokenizer(String),

    #[error("Invalid segment path")]
    InvalidPath,

    #[error("I/O error during index operation: {0}")]
    Io(#[from] std::io::Error),
}
```

---

## 6. `Cargo.toml` Spezifikation

```toml
[package]
name = "contextra-text"
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
description = "Inverted fulltext index, BM25, Block-Max WAND, and German morphological decompounding for Contextra"

[lints]
workspace = true

[dependencies]
contextra-ids = { workspace = true }
contextra-env = { workspace = true }
contextra-durable-fs = { workspace = true }
bytes = { workspace = true }
unicode-segmentation = "1.11.0"
regex = "1.10"
ahash = { workspace = true }
serde = { workspace = true }
thiserror = { workspace = true }
parking_lot = { workspace = true }
tracing = { workspace = true, optional = true }

[dev-dependencies]
tempfile = { workspace = true }
proptest = "1.4"
criterion = { workspace = true }

[features]
default = []
std = ["contextra-ids/std", "contextra-env/std"]
tracing = ["dep:tracing"]
```

---

## 7. Implementierungs-Arbeitspakete für Entwickler

1. **`Cargo.toml` Bereinigung:**
   - Entfernen von `contextra-types` und `contextra-ports`.
   - Features `workspace`, `types-compat`, `ports-compat` entfernen.
   - `default = []` setzen (R3).
2. **Altlasten-Bereinigung (`AP-21`):**
   - Veraltete Kommentare zu `contextra-router` und `contextra-agent` in `src/lib.rs` Zeile 25 bereinigen.
3. **DSGVO Phase 4b Integration:**
   - Segment-Rewrite Engine gegen `contextra-durable-fs` absichern.
4. **Verifikation:**
   - `cargo test -p contextra-text`
