# AGENTS.md — `contextra-kvcache`

> **Ring 1** · Klasse `[K]` im SPEC-Katalog (SPEC §0.5 archiviert; Root-§10 K5) · Reifegrad `beta` · Unsafe: nein · Rolle: Verschlüsselter KV-Prefix-Cache für LLM-Zustände (Radix, Eviction, KIVI-Quantisierung).
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`; diese Datei nennt nur Crate-Spezifisches. Stand 2026-11-07, statisch gelesen, **nicht kompiliert**; Zahlen sind grep-Heuristiken.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-kvcache --no-default-features --locked` |
| Contextra-Abhängigkeiten **Ist** (Cargo) | `crypto`, `ports`, `types` |
| Contextra-Abhängigkeiten **Soll** (PLAN §4) | `ports`, `types`, `crypto` |
| Features | `kv-encryption`, `kivi-quantization`, `content-addressed-kv-cache`, `kvcache-attention-eviction`, `kvcache-kivi-quant` |
| Lints | `#![forbid(unsafe_code)]` |

## 2. Abhängigkeiten & Schulden

- Kanten Ist = Soll.
- 6 Treffer mit Shredding-Bezug (`kv_shredding`): Beziehung zu `crypto::kv_shredding` klären, bevor `crypto` bereinigt wird (WP 3.1).
- Abhängig von `crypto::kv_cipher`/`kv_segment` (Zuordnung ADR).

## 3. Modul-Karte (am Code verifiziert)

| Pfad | Verantwortung |
|---|---|
| `store.rs`, `service.rs`, `prefix_store.rs`, `adapters.rs` | Cache-Dienst, Port-Adapter `KvPrefixStore` |
| `radix.rs`, `segment.rs`, `eviction_worker.rs` | Radix-Baum, Segmente, Eviction |
| `quantize_kivi.rs`, `attention_score.rs`, `scratchpad_invalidation.rs` | Quantisierung, Aufmerksamkeits-Score, Invalidierung |

## 4. Verträge & Invarianten

1. **Mandantentrennung:** Inhalts-Adressierung darf nie tenantübergreifend treffen (`content_addressed_cross_tenant_isolation.rs`).
2. **AEAD vor Quantisierung-Reihenfolge** wie getestet (`kivi_aead_order_enforced.rs`).
3. Fehlt die Fähigkeit, ist der Cache **inaktiv** (kein stilles Fallback); Löschschicht `KvCache` ist `FeatureDisabled` (SPEC §4.7).
4. Pinned-Basis bleibt bei Eviction erhalten (`clm_pinned_base_preserved.rs`).

## 5. Ports

**Implementiert:** `KvPrefixStore`. **Konsumiert:** `ports`, `crypto`.

## 6. Nebenläufigkeit & Performance

- Eviction-Worker als Hintergrundakteur mit Abbruch (`cancellation.rs`); keine geteilten Veränderlichen außerhalb der Store-Struktur.

## 7. Anti-Patterns

- Cache-Treffer über Tenant-Grenzen.
- Eviction ohne Pinning-Prüfung.

## 8. Tests & Messung

```bash
cargo test -p contextra-kvcache --locked
cargo clippy -p contextra-kvcache --all-targets -- -D warnings
```

- 15 Testdateien (Isolation, Eviction, Golden-Decode, Roundtrip).

## 9. Offene Arbeit

| WP | Inhalt |
|---|---|
| 3.x | Shredding-Bezug klären |
| 1.1 | Klasse/Reifegrad in `capabilities.toml` verbindlich setzen |


---


# Kommentar-Syntax (verbindlich)

| Form | Zweck |
|---|---|
| `//!` | Modul-/Crate-Doku am Dateianfang (vor jedem Item, nach `#![...]`-Attributen der Datei; nie mitten in der Datei) |
| `///` | Rustdoc für das **direkt folgende** Item (Struct, Enum, Variante, Feld, Trait, Methode, Funktion, Konstante). Nie vor `let`, Ausdrücken, Match-Armen oder Makroaufrufen; dort `//` verwenden. |
| `// NOTE:` | Warum etwas so gebaut ist (nicht was) |
| `// TODO:` / `// FIXME:` / `// STUB:` | Offene Arbeit / bekanntes Risiko / Platzhalter; immer mit konkreter Aussage |
| `// SAFETY:` | Pflicht direkt über jedem unsafe-Konstrukt (Invariante, Herstellungsort, Folge bei Verletzung) |
| `// ANCHOR[contextra-kvcache:<BEREICH>-<NNN>]` | Sprungmarke; vor dem Item/Abschnitt, auf den sie zeigt |
| `// FILE-CONTEXT` + `// ZWECK:` / `// INVARIANTEN:` / `// HOTSPOTS: <von>-<bis>` / `// STAND: <Datum>` | Repo-Konvention für den Dateikopf. Vorhandene Blöcke ergänzen; `HOTSPOTS`-Zeilenbereiche nach dem Kommentieren auf die neuen Zeilennummern aktualisieren. `STAND` auf das heutige Datum setzen, wenn du den Block änderst. |

Weitere Tags führst du nicht ein. Beim Kommentieren markierter Altlasten gilt: `// NOTE: Deprecated Debt DEBT-NN – <Beschreibung>`.

## ANCHOR-Ids (Kollisionsschutz)

Im Repo existieren ca. 3.900 Anchors; Ids sind workspace-weit eindeutig zu halten. Format: `ANCHOR[contextra-kvcache:<BEREICH>-<NNN>]` mit `<BEREICH>` in Großbuchstaben/Ziffern/Bindestrichen, `<NNN>` dreistellig. Vorgehen:

1. Vor jeder neuen Id: `grep -rnF "ANCHOR[contextra-kvcache:<BEREICH>-<NNN>]" crates benchmarks` muss leer sein.
2. Nummeriere ab dem Maximum des Bereichs weiter. Höchste bestehende Nummern in dieser Crate:

| Bereich | Höchste Nr. | Nächste freie Id |
|---|---:|---|
| `KIVI` | 11 | `ANCHOR[contextra-kvcache:KIVI-012]` |
| `RADIX` | 10 | `ANCHOR[contextra-kvcache:RADIX-011]` |
| `ATTN` | 6 | `ANCHOR[contextra-kvcache:ATTN-007]` |
| `SEG` | 6 | `ANCHOR[contextra-kvcache:SEG-007]` |
| `ADAPT` | 5 | `ANCHOR[contextra-kvcache:ADAPT-006]` |
| `SERV` | 5 | `ANCHOR[contextra-kvcache:SERV-006]` |
| `SCRATCH` | 4 | `ANCHOR[contextra-kvcache:SCRATCH-005]` |
| `STORE` | 4 | `ANCHOR[contextra-kvcache:STORE-005]` |
| `EVICT` | 3 | `ANCHOR[contextra-kvcache:EVICT-004]` |
| `PREFIX` | 3 | `ANCHOR[contextra-kvcache:PREFIX-004]` |
| `LIB` | 1 | `ANCHOR[contextra-kvcache:LIB-002]` |
| `TEST-PIN` | 1 | `ANCHOR[contextra-kvcache:TEST-PIN-002]` |

3. Neue Bereiche nach dem Modul benennen (z. B. `WAL`, `PUT`, `GATE`); fange bei `001` an, nach Prüfung 1. Setze Anchors sparsam: pro Modul, pro öffentlichem Einstiegspunkt und pro nicht-trivialem Ablauf, nicht pro Zeile.
4. Bereits vorhandene doppelte Ids anderer Stellen nicht „reparieren“; nur im Report melden.

Für Abläufe in Funktionen nutze nummerierte Schritte:

```rust
// ANCHOR[contextra-kvcache:PUT-001]
// Ablauf von `put`:
// 1. Eingabe prüfen (Länge, Schlüsselformat).
// 2. Snapshot der Transaktion nehmen.
// 3. Eintrag ins WAL schreiben und mit fsync bestätigen; ein Fehler wird zurückgegeben (Fail-Closed).
// 4. Erst danach im Speicher übernehmen.
```

# Was zu dokumentieren ist

Gehe **jede** Rust-Datei unter `crates/contextra-kvcache` durch (`find crates/contextra-kvcache -name "*.rs"`, ohne die generierten Dateien oben) einschließlich `tests/`, `benches/`, `examples/` und `build.rs`:

- **Modul** (`//!`): Zweck, Ring-Zugehörigkeit, Nutzer, wichtigste Typen, Invarianten, Verweis auf Nachbarmodule.
- **Öffentliche Items:** Zweck in einem Satz, dann bei Bedarf `# Errors` (welcher Fehler wann), `# Panics` nur wenn der Code abbrechen kann (sollte nach Zero-Panic-Regel nicht vorkommen: dann `FIXME`), `# Safety` bei `unsafe fn`, Vorbedingungen, Nachbedingungen, Nebenwirkungen (I/O, Zeit, Zufall, IDs). Zeit, Zufall und IDs laufen nur über `Clock`/`Rng`/`IdGen`; steht es im Code, steht es im Kommentar.
- **Nicht-triviale private Funktionen:** Warum sie existieren und was sie garantieren.
- **Nebenläufigkeit:** Sperren, atomare Operationen, Reihenfolge, Warum kein Deadlock.
- **Persistenz und Fehler:** Wo Daten WAL oder Dateisystem-Port passieren; `fsync`-Fehler werden propagiert.
- **Tests:** Ein Satz je Testfunktion oder -block: welche Invariante oder welches Verhalten bewiesen wird.
- **Architektur:** Welche Ports die Crate nutzt/anbietet, welche Abhängigkeitsrichtung gilt (DAG, `may_depend_on`), was bewusst *nicht* hier liegt.

## Rustdoc-Fallen (jeder Punkt bricht sonst `cargo clippy -D warnings` oder `cargo doc`)

- Kein `///` vor Nicht-Items (`unused_doc_comments`); keine Leerzeile zwischen `///` und Item (`empty_line_after_doc_comments`).
- Fortsetzungszeilen in Listen und Zitaten korrekt einrücken (`doc_lazy_continuation`). **Vier Leerzeichen Einrückung in Doc-Text erzeugen einen Doctest**; Code immer in Fences mit Sprache. Beispiele, die nicht kompilieren oder laufen sollen: `ignore` oder `text`, begründet.
- Intra-Doc-Links `[`Typ`]` nur auf existierende, erreichbare Items; spitze Klammern im Text (`Vec<u8>`) in Backticks setzen, sonst „unclosed HTML tag“.
- `// SAFETY:` ohne Leerzeile und ohne dazwischenliegende Zeilen direkt über dem unsafe-Ausdruck.
- Doc-Beispiele müssen gegen die tatsächliche Signatur kompilieren; im Zweifel `no_run`/`ignore` mit Grund.
