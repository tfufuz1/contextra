# AGENTS.md — `contextra-simd`

> **Ring 0 · Unsafe-Insel** · Klasse `[K]` · Reifegrad `beta` · Rolle: SIMD-Distanzkerne (AVX2, AVX-512, NEON) mit skalarem Fallback und sicherer Dispatch-API.
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`. Stand 2026-11-07, statisch gelesen, nicht kompiliert.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-simd --locked` |
| Contextra-Abhängigkeiten (Ist = Soll) | `contextra-types` (für `Result`/Fehler) |
| Features | keine |
| Lints | `#![deny(unsafe_op_in_unsafe_fn)]`; Insel erlaubt `unsafe` |
| Unsafe-Umfang | ~230 `unsafe`-Treffer in `kernels/` |
| Konsument | `contextra-vector` |

## 2. Abhängigkeiten

Nur `contextra-types`. Keine Ports, kein `std::fs`, keine Uhr, kein Runtime-Bezug.

## 3. Modul-Karte (am Code verifiziert)

| Datei | Verantwortung |
|---|---|
| `lib.rs` | sichere API: `validate_vector`, `compute_distance`, `compute_distance_trusted`, `compute_distance_f32_bytes_trusted` |
| `dispatch.rs` | Laufzeit-CPU-Erkennung, injizierbarer `Dispatcher` (`SimdTarget`) und Kernel-Auswahl |
| `kernels/scalar.rs` | **Referenzimplementierung** (sicher, Orakel für alle anderen Kernel) |
| `kernels/avx2.rs`, `avx512.rs`, `neon.rs` | handgeschriebene SIMD-Kernel (`unsafe`, `#[target_feature]`) |
| `benches/`, `tests/` | `simd_bench.rs`; `differential_tests.rs`, `simd_cross_validation.rs`, `edge_cases_tests.rs`, `quantized_and_mixed_precision_tests.rs`, `building_block_conformance.rs` |

## 4. Verträge & Invarianten

1. **Skalar ist die Wahrheit.** Jeder SIMD-Kernel muss gegen `kernels/scalar.rs` differenziell getestet sein, über zufällige Längen (inkl. Rest-Elemente `len % lane != 0`), Null-Vektoren, `NaN`, `±Inf` und Denormale.
2. **Eingaben validieren.** `validate_vector` prüft Endlichkeit (`NaN`/`±Inf` ⇒ `InvalidInput`); die „untrusted“-API `compute_distance` prüft zusätzlich Längengleichheit und Metrik und liefert `Err`. Nur die `*_trusted`-Varianten setzen validierte Eingaben voraus; ihr Aufrufer trägt die Beweislast (dokumentiert im Doc-Kommentar).
3. **Dispatch ist sicher.** Ein Kernel wird nur gewählt, wenn die CPU-Fähigkeit zur Laufzeit erkannt wurde; `#[target_feature]`-Funktionen werden nie direkt aus sicherem Code ohne Erkennung aufgerufen.
4. **Alignment.** Alle Annahmen über Ausrichtung (laut Regel `ARENA_ALIGNMENT_BYTES = 64` für Vektor-Slabs, `.agents/rules/simd_safety.md`) werden geprüft oder durch unaligned Loads (`loadu`) vermieden. Kein ungeprüfter `align_to`.
5. **Determinismus (INV-RING0-DETERMINISM-1).** Float-Reduktionen haben eine **feste** Summationsreihenfolge, unabhängig von Thread-Scheduling. Ergebnisse dürfen sich zwischen Kernen nur in einer dokumentierten Toleranz unterscheiden (ULP-Grenze im Test festschreiben).
6. **Kein `unsafe` im öffentlichen Vertrag** (M9): keine `pub unsafe fn`, die Aufrufer-Beweise verlangt, außer klar markiert und `// SAFETY:`-dokumentiert.

## 5. Ports

Keine.

## 6. Nebenläufigkeit & Performance

Zustandslos und `Send + Sync`. Hot Path: keine Allokation, keine dynamische Dispatch-Indirektion pro Element; die Kernel-Wahl wird **einmal** gecacht (Funktionszeiger) und nicht pro Aufruf neu ermittelt. Komplexität: Distanz `O(d)` Zeit, `O(1)` Zusatzspeicher. Belege Änderungen mit `cargo bench -p contextra-simd` (Benchmarks in `benches/`) und `simd-cross-validation.yml`.

## 7. Anti-Patterns

- SIMD-Kernel **ohne** Differentialtest gegen `scalar`.
- Annahme `len % 8 == 0` ohne Rest-Behandlung.
- `NaN`-Verhalten (`<`-Vergleich mit `NaN`) stillschweigend ändern: es beeinflusst Ranking-Reihenfolge und Recall.
- CPU-Erkennung pro Distanzaufruf.
- Neue Kernel-Datei ohne `// SAFETY:` je Block und ohne Miri-fähigen Skalarpfad.

## 8. Tests & Messung

```bash
cargo test -p contextra-simd --locked
cargo +nightly miri test -p contextra-simd            # Skalar- und, soweit emulierbar, SIMD-Pfade
cargo bench -p contextra-simd
cargo clippy -p contextra-simd --all-targets -- -D warnings
```

CI-Workflow `simd-cross-validation.yml` vergleicht die Kernel plattformübergreifend. Cross-Arch-Ergebnisse (x86-64 vs. aarch64) gehören in den Differentialtest.

## 9. Offene Arbeit

| WP | Inhalt |
|---|---|
| 5.6 | Unsafe-Audit der ~230 Treffer: `// SAFETY:` je Block, Miri-Lauf dokumentieren |
| 4.4 | `vector` bindet `simd` nur über die sichere API; keine direkten Kernel-Importe |


---


# Kommentar-Syntax (verbindlich)

| Form | Zweck |
|---|---|
| `//!` | Modul-/Crate-Doku am Dateianfang (vor jedem Item, nach `#![...]`-Attributen der Datei; nie mitten in der Datei) |
| `///` | Rustdoc für das **direkt folgende** Item (Struct, Enum, Variante, Feld, Trait, Methode, Funktion, Konstante). Nie vor `let`, Ausdrücken, Match-Armen oder Makroaufrufen; dort `//` verwenden. |
| `// NOTE:` | Warum etwas so gebaut ist (nicht was) |
| `// TODO:` / `// FIXME:` / `// STUB:` | Offene Arbeit / bekanntes Risiko / Platzhalter; immer mit konkreter Aussage |
| `// SAFETY:` | Pflicht direkt über jedem unsafe-Konstrukt (Invariante, Herstellungsort, Folge bei Verletzung) |
| `// ANCHOR[contextra-simd:<BEREICH>-<NNN>]` | Sprungmarke; vor dem Item/Abschnitt, auf den sie zeigt |
| `// FILE-CONTEXT` + `// ZWECK:` / `// INVARIANTEN:` / `// HOTSPOTS: <von>-<bis>` / `// STAND: <Datum>` | Repo-Konvention für den Dateikopf. Vorhandene Blöcke ergänzen; `HOTSPOTS`-Zeilenbereiche nach dem Kommentieren auf die neuen Zeilennummern aktualisieren. `STAND` auf das heutige Datum setzen, wenn du den Block änderst. |

Weitere Tags führst du nicht ein. Beim Kommentieren markierter Altlasten gilt: `// NOTE: Deprecated Debt DEBT-NN – <Beschreibung>`.

## ANCHOR-Ids (Kollisionsschutz)

Im Repo existieren ca. 3.900 Anchors; Ids sind workspace-weit eindeutig zu halten. Format: `ANCHOR[contextra-simd:<BEREICH>-<NNN>]` mit `<BEREICH>` in Großbuchstaben/Ziffern/Bindestrichen, `<NNN>` dreistellig. Vorgehen:

1. Vor jeder neuen Id: `grep -rnF "ANCHOR[contextra-simd:<BEREICH>-<NNN>]" crates benchmarks` muss leer sein.
2. Nummeriere ab dem Maximum des Bereichs weiter. Höchste bestehende Nummern in dieser Crate:

| Bereich | Höchste Nr. | Nächste freie Id |
|---|---:|---|
| – | – | – |

3. Neue Bereiche nach dem Modul benennen (z. B. `WAL`, `PUT`, `GATE`); fange bei `001` an, nach Prüfung 1. Setze Anchors sparsam: pro Modul, pro öffentlichem Einstiegspunkt und pro nicht-trivialem Ablauf, nicht pro Zeile.
4. Bereits vorhandene doppelte Ids anderer Stellen nicht „reparieren“; nur im Report melden.

Für Abläufe in Funktionen nutze nummerierte Schritte:

```rust
// ANCHOR[contextra-simd:PUT-001]
// Ablauf von `put`:
// 1. Eingabe prüfen (Länge, Schlüsselformat).
// 2. Snapshot der Transaktion nehmen.
// 3. Eintrag ins WAL schreiben und mit fsync bestätigen; ein Fehler wird zurückgegeben (Fail-Closed).
// 4. Erst danach im Speicher übernehmen.
```

# Was zu dokumentieren ist

Gehe **jede** Rust-Datei unter `crates/contextra-simd` durch (`find crates/contextra-simd -name "*.rs"`, ohne die generierten Dateien oben) einschließlich `tests/`, `benches/`, `examples/` und `build.rs`:

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
