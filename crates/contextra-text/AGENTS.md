# AGENTS.md — `contextra-text`

> **Normative Vorgabe gemäß [docs/SPEC.md §8.11](file:///home/freddy/Projekte/contextra/docs/SPEC.md) und [PLAN.md](file:///home/freddy/Projekte/contextra/PLAN.md)**

## 1. Rolle & Zweck
`contextra-text` stellt eine segmentbasierte Volltextsuchmaschine mit BM25-Scoring und Block-Max-WAND bereit. Es implementiert ein unveränderliches, AEAD-verschlüsseltes Segment-Format (SPEC §9.7), das bei Löschungen durch Rewrite und Swap physisch bereinigt wird.

## 2. Ring, Abhängigkeiten & Verbote
- **Ring:** Ring 2 (Volltextindexierung)
- **Erlaubte Abhängigkeiten:** `contextra-types`, `contextra-ports`, `contextra-crypto`
- **Verbotene Abhängigkeiten:** Keine Abhängigkeit zu `contextra-store`, `contextra-vector` oder `contextra-engine`.
- **Verbotene Muster:**
  - **Strikt kein `unsafe`** (`#![forbid(unsafe_code)]`).
  - **Kein Term-Klartext auf der Festplatte:** Auf Datenträgern existieren ausschließlich blinde `term_tag`-Hashes.
  - Keine Speicherung von Postinglisten im LSM-Store (`plb`, `fw`, `tbs` sind verboten; D-20).

## 3. Modul-Inventar (Soll-Zustand laut SPEC §8.11)

| Modul | Inhalt & Verantwortung |
|---|---|
| `tokenizer` | Deterministische Tokenisierung: NFKC-Normalisierung, Kleinschreibung, Unicode-Wortgrenzen nach UAX #29. |
| `buffer` | Textpuffer für neu ankommende Dokumente vor dem Segment-Flush. |
| `segment` | Unveränderliche Segmente (SPEC §9.7, `SegmentWriter`, `SegmentReader`): AEAD-verschlüsselt (Zielgröße 32 MiB). |
| `merge` | Hintergrund-Zusammenfassung kleiner Segmente. |
| `bm25` | Deterministisches BM25-Scoring basierend auf dynamisch aggregierten Statistiken. |
| `wand` | Block-Max WAND zur Beschleunigung von Top-K-Abfragen (verifiziert gegen exhaustives BM25). |
| `layer` | Implementierung des `ErasureLayer`-Ports für Volltextindizes. |
| `docseg` | Abbildung `doc_tag ↔ segment_id` zum Auffinden betroffener Segmente bei Löschvorgängen. |

> [!IMPORTANT]
> **Bereinigung veralteter Module (SPEC §8.11):**
> Die Module `morphology`, `morph_index`, `data`, `domain` sowie alle alten LSM-Postinglisten-Typen werden vollständig entfernt.

## 4. Kritische Invarianten
1. **Kein Termklartext auf Platte (SPEC §8.11 Invariante a):** Terme werden ausschließlich als `term_tag` (`blind_tag`) in Postings indiziert.
2. **Segment-Verschlüsselung:** Jedes Segment ist mit AES-256-GCM-SIV verschlüsselt; der Schlüssel wird via `HKDF(K_enc, "ctx/text/seg/v1" ‖ segment_id)` abgeleitet.
3. **Statistik-Berechnung ohne Drift:** Globale Kennzahlen ($N, \Sigma\text{len}, df$) werden deterministisch aus den Segmentheadern berechnet. Es gibt keine unbereinigbaren globalen Zähler.
4. **Segment-Purge-Ablauf:**
   - Dokumente im In-Memory-Puffer werden sofort verworfen.
   - Betroffene Segmente werden gestreamt und ohne das Zieldokument neu geschrieben.
   - Das alte Segment wird nach Manifest-Aktualisierung gelöscht und das Fehlen via `Vfs::exists` verifiziert.

## 5. Crate-Arbeitspakete zum SOLL-Zustand (PLAN.md)

- **[ ] WP-1.2: Veraltete Morphologie- und NLP-Module entfernen**
  - Rückbau von `morphology`, `domain` und historischem Sprachmodellcode.
- **[ ] WP-4.1: Textindex auf Segmentformat umstellen (D-20)**
  - Implementierung von `SegmentWriter` und `SegmentReader` gemäß SPEC §9.7.
  - Implementierung des `ErasureLayer`-Ports für `contextra-text`.
- **[ ] WP-4.3: Segment-Rewrite & Fehlen-Prüfung**
  - Purge-Pipeline: Segment-Rewrite ohne Ziel-Tags, atomarer Austausch und Unlink-Prüfung.

## 6. Verifikation & Benchmarks
- **T-TEXT-1:** Validierung des BM25-Rankings gegen eine mathematische Referenz.
- **T-TEXT-2:** Beweis, dass Block-Max WAND exakt dieselben Treffer wie exhaustives BM25 liefert.
- **T-TEXT-3:** Löschprüfung: Segment-Rewrite, Rohscan (E1/E2) findet keine Spuren, Term-Statistik bleibt mathematisch konsistent.
- **Benchmarks:** B-TEXT-BUILD, B-TEXT-QUERY, B-TEXT-ERASE.
