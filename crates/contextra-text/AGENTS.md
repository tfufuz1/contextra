# AGENTS.md — `contextra-text`

> **Ring 1 (`capabilities.toml`: Ring 0 ≠)** · Klasse `[K]` · Reifegrad `beta` · Unsafe: nein · Rolle: Inverted Index: BM25/BM25F, Block-Max-WAND, Tokenizer, Morphologie. Implementiert `TextIndex`.
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`; diese Datei nennt nur Crate-Spezifisches. Stand 2026-11-07, statisch gelesen, **nicht kompiliert**; Zahlen sind grep-Heuristiken.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-text --no-default-features --locked` |
| Contextra-Abhängigkeiten **Ist** (Cargo) | `ports`, `types` |
| Contextra-Abhängigkeiten **Soll** (PLAN §4) | `ports`, `types` |
| Features | `bm25f`, `docid-128` |
| Lints | `#![forbid(unsafe_code)]`, `deny(dead_code)` |

## 2. Abhängigkeiten & Schulden

- Kanten Ist = Soll; **bereits frei von Löschlogik** (0 Treffer). `inverted/purge.rs` (`TextPurger`) und `inverted/scanner.rs` (`TextScanner`) implementieren die Ports.
- Datentyp `Language` liegt in `inverted/types.rs`, Soll: `contextra-types` (WP 4.5).
- 43 `async fn` (Ratchet: nicht steigern, Ziel synchroner Kern).
- Ring in `capabilities.toml` 0, Soll 1.

## 3. Modul-Karte (am Code verifiziert)

| Pfad | Verantwortung |
|---|---|
| `inverted/` (`index_struct`, `types`, `morph_index`, `purge`, `scanner`) | Index, Port-Implementierungen |
| `bm25.rs`, `wand.rs`, `posting_list.rs`, `stream.rs` | Scoring, Block-Max-WAND, Postings, Kandidatenstrom |
| `tokenizer.rs`, `morphology/`, `domain/legal_de.rs`, `domain/medical_de.rs` | Analyse, Komposita, Domänenvokabular |

## 4. Verträge & Invarianten

1. **Statistik je Tenant** (IDF, Dokumentlängen): keine mandantenübergreifende Posting-Liste oder Statistik (INV-TENANT-IDX-1).
2. **BM25-Referenzwerte** sind Test-Orakel (`tests/bm25_reference_values.rs`); WAND muss Differentialtest gegen Brute-Force bestehen (`wand_differential.rs`).
3. **Tokenizer ohne Panics:** Fehler statt `panic!`; ungültiges UTF-8 ist Eingabefehler.
4. Tombstone-Auflösung mit Read-Your-Writes (`tombstone_read_your_writes.rs`).
5. Gleichstände deterministisch (`wand_tie_break_threshold.rs`).

## 5. Ports

**Implementiert:** `TextIndex`, `Purger`, `LayerScanner`. **Konsumiert:** `StorageEngine` (generisch `TextPurger<S>`).

## 6. Nebenläufigkeit & Performance

- Posting-Listen unveränderlich je Segment; Updates über Delta/Tombstone; kurzes `parking_lot`-Locking.
- Komplexität: WAND sublinear in Postings; Allokationsprofil in `tests/alloc_profiler.rs`, Schreibverstärkung in `write_amplification.rs`.

## 7. Anti-Patterns

- Stoppwörter/Analyzer pro Aufruf neu bauen.
- Dokumentstatistik tenantübergreifend cachen.
- `Language`-Typ weiter in `text` erweitern.

## 8. Tests & Messung

```bash
cargo test -p contextra-text --locked
cargo clippy -p contextra-text --all-targets -- -D warnings
cargo test -p contextra-text --locked --features bm25f
(cd crates/contextra-text && cargo +nightly fuzz run fuzz_bm25_tokenize -- -max_total_time=60)
```

- Property-Tests (BM25-Invarianten, Purge), Fehlerinjektion (`purge_error_injection_tests.rs`), Fuzz-Korpus vorhanden.

## 9. Offene Arbeit

| WP | Inhalt |
|---|---|
| 4.5 | Autarker Index, `Language` nach `types`, BM25F mit Block-Max-WAND |
| 3.4 | Index-Bereinigung als eigenständiger Baustein |
| 1.1 | Ring auf 1 |
