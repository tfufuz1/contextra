# contextra-text

Volltextsuche und Inverted-Index (BM25 Engine).

Normativitätsklasse: **Kern `[K]`** (Ring 0 / Ring 1 gemäß SPEC §4.1, `capabilities.toml`).

## Rolle & Zweck
`contextra-text` ist der autarke Volltext-Baustein des Contextra-Ökosystems. Er implementiert eine performante Inverted-Index-Engine mit BM25-Scoring, Block-Max WAND dynamischem Pruning und konfigurierbarer Textanalyse inklusive deutscher Morphologie (Komposita-Zerlegung, Umlaut-Normalisierung, Stoppwort-Filterung).

Der Crate kann unabhängig von anderen Speicher- oder Vektorkernen als eigenständige Textsuchmaschine verwendet werden.

## Baukasten-Einordnung & Status (M1–M11)
- **Ring:** Ring 1 (Volltextsuche / Index-Schicht)
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`
- **Zero-Panic-Doktrin:** Kein `unwrap()`, `expect()`, `panic!` im Produktionspfad.
- **Erlaubte Abhängigkeiten:** Ausschließlich `contextra-types` und `contextra-ports`.
- **Umwelt-Injektion (M4):** Persistenz rein über den generischen [`StorageEngine`]-Port aus `contextra-ports`.

## Features
- `default`: Minimaler, leichtgewichtiger Build.
- `bm25f`: Feldgewichtete BM25-Berechnung ([`BM25F`]).
- `docid-128`: Aktiviert 128-Bit `DocId`-Unterstützung (weitergeleitet an `contextra-types/docid-128`).

## Architektur & Deprecated Debt (Entscheidung D-E1)
- **Löschsemantik:** Löschpfade (`TextPurger`, `TextScanner`, `PurgeReport`, `ParsedTextTarget`) sind als `#[deprecated]` gekennzeichnet. Sobald das Interface-Paket `contextra-erasure-spi` vorliegt, werden Lösch-Satelliten (`contextra-erasure-text`) diese Aufgaben übernehmen. Der Textindex beschränkt sich künftig auf neutrale Wartungsoperationen (Postinglisten neu schreiben, Terme kompaktieren).
- **Sprach-Enum:** `Language` wird perspektivisch nach `contextra-types` migriert (Paket B21).

## Autarkes Einbettungsbeispiel
Ein vollständiges, autarkes Beispiel befindet sich unter `examples/standalone_text.rs` und kann direkt ausgeführt werden:

```bash
cargo run -p contextra-text --example standalone_text
```

```rust
use contextra_text::bm25::BM25;
use contextra_text::tokenizer::{GermanMorphTokenizer, Tokenizer};

let tokenizer = GermanMorphTokenizer::default();
let tokens = tokenizer.tokenize("Donaudampfschifffahrt und Volltextsuche");

let bm25 = BM25::default();
let score = bm25.score_term(2, 45, 40.0, 5, 500);
println!("Score: {score}");
```
