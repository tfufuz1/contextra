# Divergenzprotokoll: Chunker (`contextra-engine::chunker` vs. `contextra-ingest::chunking`) (S-61)

- **Untersuchte Module:**
  - `crates/contextra-engine/src/chunker/**` (`MarkdownChunker`, `ChunkerConfig`)
  - `crates/contextra-ingest/src/chunking/**` (`ProseChunker`, `draft_prose`, `DocumentIr`, `ChunkDraft`)
- **Referenzkorpus:** `crates/contextra-engine/tests/legacy_reference/chunker_legacy_corpus.rs` (210 Dokumente: 10 Randfälle + 200 prozedurale Dokumente)
- **Härtungstest:** `crates/contextra-engine/tests/proof_chunker_properties.rs` (2 passed, inkl. 500 Proptest-Fälle für UTF-8-Grenzen, Determinismus und Konfigurationslimits)
- **Differenztest:** `crates/contextra-engine/tests/proof_chunker_vs_ingest.rs` (1 passed)

## Gemessene Snapshot-Kennzahlen auf dem 210-Dokumente-Korpus

| Metrik | Gemessener Wert | Erwartungswert (eingefroren) | Status |
|---|---|---|---|
| Dokumente im Testkorpus | 210 | 210 | Exakt |
| Dokumente mit Chunks | 209 | 209 | Exakt (nur `edge_empty` liefert 0 Chunks) |
| Gesamtzahl Chunks (`MarkdownChunker`) | 629 | 629 | Exakt |
| Gesamtzahl Token-Schätzung (`CHARS_PER_TOKEN`) | 257.858 | 257.858 | Exakt |
| Chunks mit Brotkrumen (`breadcrumb`) | 628 | 628 | Exakt (alle außer reinem Whitespace) |
| Token-Schätzer Parität Prosa (`estimate_tokens`) | identisch | identisch | Parität bewiesen |

## Strukturelle und architektonische Divergenzen
Zwischen `contextra-engine::chunker` und `contextra-ingest::chunking` bestehen wesentliche Architekturunterschiede:
1. **Eingabe- und Datenmodell:**
   - `contextra-engine::chunker`: Erwartet direkt `&str` (Markdown-Text) und liefert `Vec<contextra_types::ContextChunk>` mit JSON-Metadaten (`breadcrumb`, `source_span`).
   - `contextra-ingest::chunking`: Erwartet `DocumentIr` und `CanonicalText`, operiert auf AST-Knotenstrukturen und liefert interne `ChunkDraft`-Instanzen mit `Range<usize>`-Offsets.
2. **Abhängigkeitsrichtung & Manifesteigentum:**
   - `contextra-ingest` ist in `contextra-engine` ausschließlich als `dev-dependency` deklariert.
   - Ein Ersatz von `MarkdownChunker` durch einen Adapter über `contextra-ingest::chunking` würde eine reguläre Abhängigkeit (`dependencies`) erzwingen. `Cargo.toml` gehört jedoch ENG-A und darf in diesem Auftrag nicht geändert werden.
3. **Fehlerbehandlung:**
   - `MarkdownChunker` ist panikfrei und unfehlbar (`chunk(&self, ...) -> Vec<ContextChunk>`).
   - `contextra-ingest::chunking::draft_prose` gibt `Result<Vec<ChunkDraft>, IngestError>` zurück.

## Härtung und Invarianten
- In `proof_chunker_properties.rs` wurde nachgewiesen:
  - Kein Panic bei beliebigen UTF-8-Sequenzen (Steuerzeichen, Nullbytes, Emojis, ungeschlossene Codeblöcke).
  - Deterministische Idempotenz: Zweifaches Segmentieren liefert exakt dieselben Chunks.
  - Alle `source_span`-Start- und Endpositionen liegen zwingend auf gültigen UTF-8-Zeichengrenzen (`is_char_boundary`).
  - Keine blockierenden Fehler im Chunker identifiziert; alle bestehenden Golden- und Robustheitstests laufen fehlerfrei.

## Voraussetzungen für einen zukünftigen Ingest-Chunker-Adapter (Übergabe an ENG-A)
1. Aufnahme von `contextra-ingest` in `[dependencies]` von `crates/contextra-engine/Cargo.toml` (oder Bereitstellung eines minimalen Ingest-SPI-Crates).
2. Adapter-Schicht, die flachen Markdown-Text (`&str`) in `DocumentIr` überführt und `ChunkDraft` in `contextra_types::ContextChunk` konvertiert.
3. Klärung der Brotkrumen-Semantik (String-Format `> ` vs. hierarchische Pfadlisten).
