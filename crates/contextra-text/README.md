# contextra-text

Volltextsuche und Inverted-Index (BM25 Engine).

Normativitätsklasse: **Baustein Klasse A** (SPEC v4.0 §2, §5.1, §11.3).

## Rolle & Zweck
`contextra-text` ist der autarke Volltext-Baustein des Contextra-Ökosystems. Er implementiert eine performante Inverted-Index-Engine mit BM25-Scoring, Block-Max WAND dynamischem Pruning und konfigurierbarer Textanalyse inklusive deutscher Morphologie (Komposita-Zerlegung, Umlaut-Normalisierung, Stoppwort-Filterung).

Der Crate kann unabhängig von anderen Speicher- oder Vektorkernen als eigenständige Textsuchmaschine verwendet werden.

## Baukasten-Einordnung & Status (Klasse A)
- **Klasse:** A (0 interne Pflicht-Abhängigkeiten im Standalone-Build)
- **Ring:** Ring 0 / Ring 1
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`
- **Zero-Panic-Doktrin:** Kein `unwrap()`, `expect()`, `panic!` im Produktionspfad.
- **Fehlermodell:** Typisiertes `TextError` (`#[non_exhaustive]`) mit `Result<T>`.
- **Index-Schnittstelle (SPEC §4.3):** Implementiert `Index<DocId, str>` und `Maintain::compact()` auf `MemoryTextIndex`.

## Features
- `default`: Transitionale Workspace-Kompatibilität (`types-compat`).
- `bm25f`: Feldgewichtete BM25-Berechnung ([`BM25F`]).
- `inspect`: Öffnet Rohsicht auf interne Posting-Listen und Segmentdaten für Analyse- und Scan-Adapter.
- `types-compat`: Transitionale Konvertierungen zu `contextra-types` und `contextra-ports` (bis Welle 6).
- `ports-compat`: Transitionale Anbindung an `contextra-ports` (bis Welle 6).
- `docid-128`: Aktiviert 128-Bit `DocId`-Unterstützung.

## Export & Standalone-Verifikation
Der Crate kann mit dem Exporter-Tool vollständig isoliert exportiert und gebaut werden:

```bash
python3 tools/export_crate.py contextra-text --out /tmp/export-contextra-text
```

## Autarkes Einbettungsbeispiel
Ein vollständiges, autarkes Beispiel befindet sich unter `examples/standalone_text.rs` und kann direkt ausgeführt werden:

```bash
cargo run -p contextra-text --example standalone_text
```

```rust
use contextra_text::memory_index::MemoryTextIndex;
use contextra_text::traits::{Index, Maintain};
use contextra_text::types::DocId;

let mut index = MemoryTextIndex::new();
index.insert(DocId::new(1), b"Donaudampfschifffahrt und Volltextsuche").unwrap();

let hits = index.search("Volltextsuche", 10).unwrap();
println!("Gefundene Treffer: {}", hits.len());

index.remove(&DocId::new(1)).unwrap();
index.compact().unwrap();
```
