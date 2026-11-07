# AGENTS.md — `contextra-router`

> **Ring 3** · Klasse Rand `[R]` (SPEC-Katalog: Archiv, Root-§10 K5) · Reifegrad `stable` · Unsafe: nein · Rolle: Adaptives Query-/Modell-Routing (Bandits, Conformal Calibration, Dispatch).
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`; diese Datei nennt nur Crate-Spezifisches. Stand 2026-11-07, statisch gelesen, **nicht kompiliert**; Zahlen sind grep-Heuristiken.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-router --no-default-features --locked` |
| Contextra-Abhängigkeiten **Ist** (Cargo) | `adapt`, `ports`, `types`, `wire` |
| Contextra-Abhängigkeiten **Soll** (PLAN §4) | `ports`, `types`, `wire`, `adapt` |
| Features | `bandit-routing`, `flow-corrected-thompson`, `egress-sherman-morrison`, `cloud-egress-guard` |
| Lints | `#![forbid(unsafe_code)]`, `deny(dead_code)` |

## 2. Abhängigkeiten & Schulden

- Normalkanten = Soll. `capabilities.toml` nennt zusätzlich `privacy` (nicht in Cargo). Dev-Kanten `db`, `graph`, `text`, `vector` (M6).
- Egress soll über den Port `EgressPolicy` laufen (WP 2.4), nicht über eine Bindung an `privacy`.

## 3. Modul-Karte (am Code verifiziert)

| Pfad | Verantwortung |
|---|---|
| `router/` (`dispatch_core`, `lifecycle`, `outcomes`) | `RouterEngine` |
| `profile.rs`, `arm_registry.rs`, `routing_strategy.rs` | Profile, Arme, Strategien |
| `dispatch.rs`, `fc_ts_dispatch.rs`, `transport.rs`, `outcome.rs` | Dispatch, Transport, Ergebnisse |

## 4. Verträge & Invarianten

1. Ein Modell wird nie gewählt, dessen Kontextgrenze das Fenster überschreitet; sonst vorher kompaktieren.
2. Kalibrierung wird nach jeder Interaktion aktualisiert (Conformal).
3. Zufall nur über `Rng`, Zeit über `Clock`; Entscheidungen sind reproduzierbar (`id_gen_determinism_test`).
4. Cloud-Egress ist standardmäßig aus und budgetiert (INV-EGRESS-BUDGET-1).

## 5. Ports

Konsumiert `Clock`, `Rng`, `IdGen`, `EgressPolicy` (Soll).

## 6. Nebenläufigkeit & Performance

- Profile/Statistiken hinter kurzem `parking_lot::RwLock`; Updates synchron und kurz.

## 7. Anti-Patterns

- SLM ohne Profil aufrufen.
- Dispatch ohne Egress-Prüfung.

## 8. Tests & Messung

```bash
cargo test -p contextra-router --locked
cargo clippy -p contextra-router --all-targets -- -D warnings
```

- 7 Testdateien; Regret-Tests für Bandits.

## 9. Offene Arbeit

| WP | Inhalt |
|---|---|
| 2.4 | `EgressPolicy` einführen und nutzen |
| M6 | Dev-Kanten entfernen |


---


# Kommentar-Syntax (verbindlich)

| Form | Zweck |
|---|---|
| `//!` | Modul-/Crate-Doku am Dateianfang (vor jedem Item, nach `#![...]`-Attributen der Datei; nie mitten in der Datei) |
| `///` | Rustdoc für das **direkt folgende** Item (Struct, Enum, Variante, Feld, Trait, Methode, Funktion, Konstante). Nie vor `let`, Ausdrücken, Match-Armen oder Makroaufrufen; dort `//` verwenden. |
| `// NOTE:` | Warum etwas so gebaut ist (nicht was) |
| `// TODO:` / `// FIXME:` / `// STUB:` | Offene Arbeit / bekanntes Risiko / Platzhalter; immer mit konkreter Aussage |
| `// SAFETY:` | Pflicht direkt über jedem unsafe-Konstrukt (Invariante, Herstellungsort, Folge bei Verletzung) |
| `// ANCHOR[contextra-router:<BEREICH>-<NNN>]` | Sprungmarke; vor dem Item/Abschnitt, auf den sie zeigt |
| `// FILE-CONTEXT` + `// ZWECK:` / `// INVARIANTEN:` / `// HOTSPOTS: <von>-<bis>` / `// STAND: <Datum>` | Repo-Konvention für den Dateikopf. Vorhandene Blöcke ergänzen; `HOTSPOTS`-Zeilenbereiche nach dem Kommentieren auf die neuen Zeilennummern aktualisieren. `STAND` auf das heutige Datum setzen, wenn du den Block änderst. |

Weitere Tags führst du nicht ein. Beim Kommentieren markierter Altlasten gilt: `// NOTE: Deprecated Debt DEBT-NN – <Beschreibung>`.

## ANCHOR-Ids (Kollisionsschutz)

Im Repo existieren ca. 3.900 Anchors; Ids sind workspace-weit eindeutig zu halten. Format: `ANCHOR[contextra-router:<BEREICH>-<NNN>]` mit `<BEREICH>` in Großbuchstaben/Ziffern/Bindestrichen, `<NNN>` dreistellig. Vorgehen:

1. Vor jeder neuen Id: `grep -rnF "ANCHOR[contextra-router:<BEREICH>-<NNN>]" crates benchmarks` muss leer sein.
2. Nummeriere ab dem Maximum des Bereichs weiter. Höchste bestehende Nummern in dieser Crate:

| Bereich | Höchste Nr. | Nächste freie Id |
|---|---:|---|
| `PRF` | 22 | `ANCHOR[contextra-router:PRF-023]` |
| `OUT` | 9 | `ANCHOR[contextra-router:OUT-010]` |
| `OTC` | 9 | `ANCHOR[contextra-router:OTC-010]` |
| `LCF` | 8 | `ANCHOR[contextra-router:LCF-009]` |
| `RTR` | 6 | `ANCHOR[contextra-router:RTR-007]` |
| `ARM` | 5 | `ANCHOR[contextra-router:ARM-006]` |
| `FCTS` | 3 | `ANCHOR[contextra-router:FCTS-004]` |
| `SERDE` | 3 | `ANCHOR[contextra-router:SERDE-004]` |
| `TRP` | 3 | `ANCHOR[contextra-router:TRP-004]` |
| `DSP` | 2 | `ANCHOR[contextra-router:DSP-003]` |
| `BN-BLAT` | 1 | `ANCHOR[contextra-router:BN-BLAT-002]` |
| `BN-COST` | 1 | `ANCHOR[contextra-router:BN-COST-002]` |

3. Neue Bereiche nach dem Modul benennen (z. B. `WAL`, `PUT`, `GATE`); fange bei `001` an, nach Prüfung 1. Setze Anchors sparsam: pro Modul, pro öffentlichem Einstiegspunkt und pro nicht-trivialem Ablauf, nicht pro Zeile.
4. Bereits vorhandene doppelte Ids anderer Stellen nicht „reparieren“; nur im Report melden.

Für Abläufe in Funktionen nutze nummerierte Schritte:

```rust
// ANCHOR[contextra-router:PUT-001]
// Ablauf von `put`:
// 1. Eingabe prüfen (Länge, Schlüsselformat).
// 2. Snapshot der Transaktion nehmen.
// 3. Eintrag ins WAL schreiben und mit fsync bestätigen; ein Fehler wird zurückgegeben (Fail-Closed).
// 4. Erst danach im Speicher übernehmen.
```

# Was zu dokumentieren ist

Gehe **jede** Rust-Datei unter `crates/contextra-router` durch (`find crates/contextra-router -name "*.rs"`, ohne die generierten Dateien oben) einschließlich `tests/`, `benches/`, `examples/` und `build.rs`:

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
