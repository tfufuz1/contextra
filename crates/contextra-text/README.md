# contextra-text

Autarker, thread-sicherer Volltextsuche- und Inverted-Index-Baustein (BM25 Engine).

Normativitätsklasse: **Baustein Klasse A** (SPEC v4.0 §2, §5.1, §11.3).

## Rolle & Zweck
`contextra-text` ist ein autarker Klasse-A-Baustein des Contextra-Ökosystems. Er implementiert eine performante Inverted-Index-Engine mit BM25-Scoring, Copy-on-Write Snapshot-Isolation, Block-Max WAND dynamischem Pruning und konfigurierbarer Textanalyse inklusive deutscher Morphologie (Komposita-Zerlegung, Umlaut-Normalisierung, Stoppwort-Filterung).

Der Baustein besitzt im autarken Standalone-Betrieb null interne Abhängigkeiten (`contextra-*`) und kann vollständig isoliert in Fremdprojekte eingebettet werden.

## Baukasten-Einordnung & Status (Klasse A)
- **Klasse:** A (0 interne Pflicht-Abhängigkeiten im Standalone-Build)
- **Ring:** Ring 0 / Ring 1
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`
- **Zero-Panic-Doktrin:** Kein `unwrap()`, `expect()`, `panic!` im Produktionspfad.
- **Fehlermodell:** Eingefrorener Vertrag in `contract.rs` mit typisiertem `TextError` / `Error` (`#[non_exhaustive]`) und `Result<T>`.
- **Index-Schnittstelle (SPEC §4.3):** Implementiert `Index<DocKey, str>` und `Maintain::compact()` auf `MemoryTextIndex`.
- **Snapshot-Isolation:** Leser greifen auf unveränderliche `IndexSnapshot`-Instanzen zu. Leser blockieren Schreiber nicht länger als O(1).
- **Löschung:** Neutral über Tombstones und physische Freigabe via `compact`.

## Features
- `std` (Default): Standardbibliotheksunterstützung.
- `serde`: Serialisierung und Deserialisierung für Kernstrukturen.
- `tracing`: Diagnostische Telemetrie.
- `inspect`: Öffnet Rohsicht auf interne Posting-Listen, Segmentdaten und Schlüsselinspektion.
- `experimental`: Experimentelle Such- und Indexierungsfeatures.
- `bm25f`: Feldgewichtete BM25-Berechnung ([`BM25F`]).
- `docid-128`: Aktiviert 128-Bit `DocId`-Unterstützung.
- `types-compat`: Transitionale Konvertierungen zu `contextra-types` und `contextra-ports`.
- `ports-compat`: Transitionale Anbindung an `contextra-ports`.

## Export & Standalone-Verifikation
Der Crate kann mit dem Exporter-Tool vollständig isoliert exportiert und gebaut werden:

```bash
python3 tools/export_crate.py contextra-text --out /tmp/export-contextra-text
```

## Copy-Paste-Anleitung für Fremdprojekte
Füge `contextra-text` zu deiner `Cargo.toml` hinzu:

```toml
[dependencies]
contextra-text = { version = "0.1", default-features = false, features = ["std"] }
```

## Minimalbeispiel (Klasse A / Autark)
Ein vollständiges Beispiel befindet sich unter `examples/standalone_text.rs`:

```rust
use contextra_text::bm25::BM25;
use contextra_text::contract::DocKey;
use contextra_text::memory_index::MemoryTextIndex;
use contextra_text::traits::{Index, Maintain};

// 1. Initialisiere den Index
let mut index = MemoryTextIndex::new();
let doc1 = DocKey::from(1u64);
let doc2 = DocKey::from(2u64);

// 2. Dokumente einfügen
Index::<DocKey, str>::insert(&mut index, doc1, b"Donaudampfschifffahrt und Volltextsuche").unwrap();
Index::<DocKey, str>::insert(&mut index, doc2, b"Volltextsuche in Rust mit BM25").unwrap();

// 3. Suche ausführen
let hits = Index::<DocKey, str>::search(&index, "Volltextsuche", 10).unwrap();
println!("Gefundene Treffer: {}", hits.len());

// 4. Neutrale Löschung per Tombstone und physische Kompaktierung
Index::<DocKey, str>::remove(&mut index, &doc1).unwrap();
let stats = Maintain::compact(&mut index).unwrap();
println!("Kompaktierung: {} bereinigt, {} aktiv.", stats.purged_docs, stats.active_docs);
```
