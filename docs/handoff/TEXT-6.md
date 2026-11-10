# Handoff TEXT-6 — contextra-text: PostingStore, compat/, TextError

- **Crate:** `crates/contextra-text`
- **Commit-Kennung:** `TX6`
- **Branch:** `jules/text-6-decouple`
- **Status:** Abgeschlossen, mergefähig

---

## 1. Ergebnisse je Arbeitspaket

| WP | Titel | Status | Prüfbefehl | Ergebnis |
|---|---|---|---|---|
| **TX6.0** | Baseline, Kopplungs- & Feature-Befund | GRÜN | `cargo check -p contextra-text --all-targets` | Baseline unter `docs/decoupling/baseline/text/` erfasst. Nullprüfzahlen und StorageEngine-Inventar (11 Methoden) erhoben. |
| **TX6.1** | Golden- und Orakel-Korpus | GRÜN | `cargo test -p contextra-text --test golden_oracle_tests` | Golden-Dateien mit `MANIFEST.sha256` unter `tests/golden/` angelegt, Re-Encoding, Backward-Read und Bit-Gleichheit bewiesen. |
| **TX6.2** | Fehlerabbildung (TextError) | GRÜN | `cargo test -p contextra-text --lib error::tests` | `docs/decoupling/errors/text.md` angelegt. `TextError` (thiserror) mit Table-Driven-Test; `bm25.rs`, `tokenizer.rs`, `posting_list.rs` auf `TextError` umgestellt. |
| **TX6.3** | Brücke `compat/` und `PostingStore` | GRÜN | `cargo check -p contextra-text --no-default-features --all-targets` | `PostingStore` Trait (exakt 11 Methoden) in `storage.rs` definiert. `compat/storage.rs` implementiert `PostingStore` für `StorageEngine`. `purge.rs`, `scanner.rs`, `morph_index.rs`, `Bm25Scorer` nach `compat/` verschoben; Pfadkompatibilität gewahrt. |
| **TX6.4** | Differenztest & Nullprüfungen | GRÜN | `cargo test -p contextra-text --test golden_oracle_tests` | Autarker Bau ohne Default-Features und mit Einzel-Features verifiziert. `Bm25CandidateStream` auf `PostingStore` migriert. |
| **TX6.5** | Feature-Hygiene | GRÜN | `cargo check -p contextra-text --features workspace --all-targets --locked` | Alle bestehenden Weiterleitungen geschützt. Additives Feature `workspace` eingeführt. |
| **TX6.6** | Härtung & Determinismus | GRÜN | `python3 tools/zero_panic_analyzer.py --crate contextra-text --fail-on-prod` | 0 Prod-Verstöße bei Zero-Panic; verbotene Formulierungen bereinigt (0 Befunde); Bitflip-, Truncation- und Permutations-Determinismus nachgewiesen. |
| **TX6.7** | Dokumentation & Handoff | GRÜN | `cargo fmt -p contextra-text -- --check` | `crates/contextra-text/AGENTS.md` und Handoff aktualisiert. |

---

## 2. API-Freeze-Tabelle

| Typ / Funktion | Vorheriger Pfad / Signatur | Neuer interner Pfad | Kompatibilitätsstatus |
|---|---|---|---|
| `Bm25Scorer` | `contextra_text::Bm25Scorer<S: StorageEngine>` | `contextra_text::compat::scorer::Bm25Scorer` | Re-Export in Root unter `types-compat`: Quellkompatibel |
| `BM25MorphIndex` | `contextra_text::BM25MorphIndex<S: StorageEngine>` | `contextra_text::compat::morph_index::BM25MorphIndex` | Re-Export in Root unter `types-compat`: Quellkompatibel |
| `TextPurger` | `contextra_text::TextPurger<S: StorageEngine>` | `contextra_text::compat::purge::TextPurger` | Re-Export in Root unter `types-compat`: Quellkompatibel |
| `TextScanner` | `contextra_text::TextScanner<S: StorageEngine>` | `contextra_text::compat::scanner::TextScanner` | Re-Export in Root unter `types-compat`: Quellkompatibel |
| `PostingStore` | Neu | `contextra_text::storage::PostingStore` | Neu / Additiv, exportiert in Root |
| `InvertedIndex` | `InvertedIndex<S: StorageEngine>` | `InvertedIndex<S: PostingStore>` | `StorageEngine` implementiert blanket `PostingStore`: Quellkompatibel |
| `Bm25CandidateStream` | `Bm25CandidateStream<S: StorageEngine>` | `Bm25CandidateStream<S: PostingStore>` | Quellkompatibel |

---

## 3. Messwerte Vorher / Nachher

| Metrik | Vorher (TX6.0) | Nachher (TX6.7) | Delta |
|---|---|---|---|
| `zero_panic_analyzer` (Prod-Verstöße) | 0 | 0 | 0 |
| `pattern_diagnostic` (Forbidden Wording) | 5 | 0 | -5 (vollständig neutralisiert) |
| Standalone Build (`--no-default-features`) | Grün | Grün | stabil |
| Golden Tests | 0 Tests | 4 Tests (byteweise & Determinismus) | +4 Tests |

---

## 4. Abweichungen von SPEC-v2

- Keine unvorhergesehenen Abweichungen. Die Vorgaben aus dem Auftrag zur Unabhängigkeit (kein Verlass auf separate `contextra-ids`/`contextra-env`-Migration, kein Aufbrechen von Verbraucher-Manifesten) wurden strikt eingehalten.

---

## 5. Koordinationsanforderungen (Übergaben an Folgeschritt)

1. **`contextra-store`:** Pre-existing Compile-Fehler E0201 (doppelte Methoden in `block_cache.rs`) und E0509/E0599 in `crates/contextra-store` blockieren weiterhin workspace-weite Builds; ist außerhalb des Eigentums von `contextra-text`.
2. **Default-Flip:** Sobald die übrigen Crates auf autarke Text-Traits migriert sind, kann `default = ["std"]` (Entfernen von `types-compat` aus default) im Integrationsschritt geschaltet werden.
