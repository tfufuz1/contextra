# ADR 0002: Adapter-Crate `contextra-ingest` für Format-Extraktion & Chunking

> **Status-Update 30.10.2026:** Überholt. Das Crate `contextra-ingest` ist entfernt (SPEC §5.2).

## Status
Accepted

## Datum
2026-10-31

## Kontext
Entsprechend SPEC §C-10.3 und §C-11 gehören Dokumentformat-Extraktoren und strukturbewusstes Chunking außerhalb des Datenbank-Kerns in ein dediziertes Adapter-Crate in Ring 4. Die bisherige Implementierung im Kern beschränkte sich auf den reinen Markdown-Chunker (`crates/contextra-engine/src/chunker.rs`). Um ca. 15 neue Ingestion-Formate und erweiterte Chunking-Strategien sicher und erweiterbar zu unterstützen, wird mit `contextra-ingest` die grundlegende Adapter-Architektur geschaffen.

## Entscheidungen (A1–A9)

1. **A1: Entkopplung in ein dediziertes Ring-4 Adapter-Crate (`contextra-ingest`)**
   - Das neue Crate wird isoliert in Ring 4 angesiedelt.
   - Es hat keine I/O- oder Netzwerk-Berechtigungen (reines Byte-in / IngestOutput-out Modell).
   - Es darf ausschließlich von Ring-0/Ring-3-Schnittstellen (`contextra-types`, `contextra-engine`) abhängen, ohne zirkuläre Rückkopplungen.

2. **A2: Zweistufige IR-Architektur (`DocumentIr` -> `CanonicalText`)**
   - Extraktion trennt Quellcode-Parsing von Text-Kanonisierung.
   - `DocumentIr` repraesentiert die flache Struktur des Dokuments (Ueberschriften, Absaetze, Tabellen, Boundary-Marker, Anhaenge).
   - `CanonicalText` erzeugt die deterministische String-Repraesentation mit exakten Byte-Spans je Block für nahtloses Span-Slicing.

3. **A3: Metadatenschema `contextra.chunk.v1` statt `ContextChunk`-Aenderung**
   - Um den Typ `ContextChunk` in Ring 0 (`contextra-types`) stabil zu halten, werden alle erweiterten Ingestion-Metadaten (z. B. `chunk_id`, `source_span`, `locator`, `strategy`, `breadcrumb`, `heading_level`) strukturiert im JSON-Feld `metadata` unter dem Schema `"contextra.chunk.v1"` abgelegt.

4. **A4: Strikte `source_span`-Semantik auf dem kanonischen Text**
   - `source_span` (`Range<usize>`) bezieht sich exakt auf den Byte-Bereich im `CanonicalText`.
   - Bei verbatim weitergereichtem Text (z. B. Raw-Markdown) gilt: kanonischer Text == Originaltext.
   - Es gilt die Kontextfreiheits-Invariante: Das Rendering einer Teilblockliste liefert identische Substrings im Elterndokument.

5. **A5: Fail-Closed Panik-Isolierung via `std::panic::catch_unwind`**
   - Unbehandelte Panics externer Format-Parser werden an der Crate-Grenze in `ExtractorRegistry::extract` mittels `catch_unwind` abgefangen und strikt auf `IngestError::ParserPanic` gemappt.
   - Produktionscode enthält kein `unwrap`, `expect`, `panic!`, `todo!` oder Array-Indexing.

6. **A6: Versteckter Inhalt und Datenschutz-Policies (`ExtractPolicy`)**
   - Versteckte Inhalte (Kommentare, Redaktionsnotizen, nachverfolgte Löschungen) werden standardmäßig gefiltert (`include_hidden = false`).
   - Personenbezogene Dokument-Metadaten (Autoren) werden nur bei explizitem `keep_personal_metadata = true` übernommen.

7. **A7: Ressourcenbudgets und Zip-Bomb-Schutz (`ResourceBudget` & `IngestLimits`)**
   - Dekompression, XML-Events, Zell- und Seitenzahlen werden über ein atomares `ResourceBudget` vorab geprüft.
   - Überschreitungen führen sofort zu `IngestError::LimitExceeded`.

8. **A8: Einbettung von Anhängen als flache Boundaries**
   - Anhänge und verschachtelte Dokumente werden als flache `Boundary::Attachment`-Blöcke in die hierarchische Blockliste eingebettet und erben entsprechende `attachment`-Locators.

9. **A9: Abhängigkeit zu `contextra-engine` für Fallback-Chunking**
   - `contextra-ingest` nutzt `contextra_engine::chunker::MarkdownChunker` als Fallback-Engine für unvollständige oder reine Fließtext-Formate, um vollständige Verhaltenskompatibilität zu gewährleisten.

## Konsequenzen
- Das Crate `contextra-ingest` bildet das stabile Fundament für alle zukünftigen Format-Extraktoren.
- Ring-0-Typen bleiben unberührt.
- Fuzzing und defensive Parser-Isolierung sichern das System gegen bösartige Eingabedateien ab.
