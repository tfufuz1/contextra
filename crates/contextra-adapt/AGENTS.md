# AGENTS.md — `contextra-adapt`

> **Ring 0** · Klasse `[K]` laut SPEC-Katalog (SPEC §0.5 archiviert es; Root-§10 K5: aktiv behandeln) · Reifegrad `beta` · Unsafe: nein · Rolle: Adaptive Algorithmen: PID-Latenzregler, Bandits, Drift-/Lyapunov-Wächter, Off-Policy.
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`; diese Datei nennt nur Crate-Spezifisches. Stand 2026-11-07, statisch gelesen, **nicht kompiliert**; Zahlen sind grep-Heuristiken.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-adapt --no-default-features --locked` |
| Contextra-Abhängigkeiten **Ist** (Cargo) | `ports`, `types` |
| Contextra-Abhängigkeiten **Soll** (PLAN §4) | `ports`, `types` |
| Features | `bandit-routing`, `flow-corrected-thompson`, `rie-greedy-personalization`, `sketched-bandit` |
| Lints | `#![forbid(unsafe_code)]`, `warn(missing_docs)` |

## 2. Abhängigkeiten & Schulden

- Ist = Soll, autark.
- `pid_latency_controller.rs`, `homeostat.rs`, `decay_controller.rs` erscheinen auch in `db`/`engine` (Re-Exports/Kopien prüfen, DRY).

## 3. Modul-Karte (am Code verifiziert)

| Pfad | Verantwortung |
|---|---|
| `pid.rs`, `pid_latency_controller.rs`, `homeostat.rs` | Regler |
| `bandit.rs`, `flow_thompson.rs`, `rie_greedy.rs`, `off_policy.rs` **und** `offpolicy.rs` | Bandit-/Policy-Lernen (zwei Off-Policy-Dateien: Dopplung klären) |
| `drift.rs`, `lyapunov.rs`, `shadow_mode.rs`, `decay_controller.rs` | Drift, Stabilität, Schattenmodus |

## 4. Verträge & Invarianten

1. **Determinismus (P28):** Zufall nur über `Rng`, Zeit nur über `Clock` (Heuristik: 1 `Instant::now`/`SystemTime::now`-Treffer prüfen).
2. **Wirkungsgrenze:** Regler liefern Empfehlungen/Parameter; sie schreiben nie selbst in Speicher.
3. Alle Regler haben Grenzen (Clamp), Anti-Windup und einen Fallback auf feste Startwerte (SPEC §4.7).

## 5. Ports

Implementiert `AdaptiveController`-artige Rollen; konsumiert `Clock`, `Rng`.

## 6. Nebenläufigkeit & Performance

- Zustand je Regler in der Instanz, kein globaler Zustand; Aufruf aus einem Thread oder hinter Nachrichtenkanal.

## 7. Anti-Patterns

- Globale Singleton-Regler.
- Stille Anpassung der Ranking-Gewichte ohne Schattenmodus und Messung.

## 8. Tests & Messung

```bash
cargo test -p contextra-adapt --locked
cargo clippy -p contextra-adapt --all-targets -- -D warnings
```

- Konvergenz-/Stabilitäts-Property-Tests, deterministischer Replay mit `SeededRng`.

## 9. Offene Arbeit

| WP | Inhalt |
|---|---|
| 5.3 | `graph → adapt`-Kante ist in `capabilities.toml` noch deklariert, in Cargo nicht mehr: bereinigen |
| DRY | `offpolicy.rs` vs `off_policy.rs` zusammenführen |


---


# Kommentar-Syntax (verbindlich)

| Form | Zweck |
|---|---|
| `//!` | Modul-/Crate-Doku am Dateianfang (vor jedem Item, nach `#![...]`-Attributen der Datei; nie mitten in der Datei) |
| `///` | Rustdoc für das **direkt folgende** Item (Struct, Enum, Variante, Feld, Trait, Methode, Funktion, Konstante). Nie vor `let`, Ausdrücken, Match-Armen oder Makroaufrufen; dort `//` verwenden. |
| `// NOTE:` | Warum etwas so gebaut ist (nicht was) |
| `// TODO:` / `// FIXME:` / `// STUB:` | Offene Arbeit / bekanntes Risiko / Platzhalter; immer mit konkreter Aussage |
| `// SAFETY:` | Pflicht direkt über jedem unsafe-Konstrukt (Invariante, Herstellungsort, Folge bei Verletzung) |
| `// ANCHOR[contextra-adapt:<BEREICH>-<NNN>]` | Sprungmarke; vor dem Item/Abschnitt, auf den sie zeigt |
| `// FILE-CONTEXT` + `// ZWECK:` / `// INVARIANTEN:` / `// HOTSPOTS: <von>-<bis>` / `// STAND: <Datum>` | Repo-Konvention für den Dateikopf. Vorhandene Blöcke ergänzen; `HOTSPOTS`-Zeilenbereiche nach dem Kommentieren auf die neuen Zeilennummern aktualisieren. `STAND` auf das heutige Datum setzen, wenn du den Block änderst. |

Weitere Tags führst du nicht ein. Beim Kommentieren markierter Altlasten gilt: `// NOTE: Deprecated Debt DEBT-NN – <Beschreibung>`.

## ANCHOR-Ids (Kollisionsschutz)

Im Repo existieren ca. 3.900 Anchors; Ids sind workspace-weit eindeutig zu halten. Format: `ANCHOR[contextra-adapt:<BEREICH>-<NNN>]` mit `<BEREICH>` in Großbuchstaben/Ziffern/Bindestrichen, `<NNN>` dreistellig. Vorgehen:

1. Vor jeder neuen Id: `grep -rnF "ANCHOR[contextra-adapt:<BEREICH>-<NNN>]" crates benchmarks` muss leer sein.
2. Nummeriere ab dem Maximum des Bereichs weiter. Höchste bestehende Nummern in dieser Crate:

| Bereich | Höchste Nr. | Nächste freie Id |
|---|---:|---|
| `BANDIT-CONST` | 1 | `ANCHOR[contextra-adapt:BANDIT-CONST-002]` |
| `BANDIT-ERROR` | 1 | `ANCHOR[contextra-adapt:BANDIT-ERROR-002]` |
| `BANDIT-ALIGNED` | 1 | `ANCHOR[contextra-adapt:BANDIT-ALIGNED-002]` |
| `BANDIT-IMPL-ENUM` | 1 | `ANCHOR[contextra-adapt:BANDIT-IMPL-ENUM-002]` |
| `BANDIT-SKETCH` | 1 | `ANCHOR[contextra-adapt:BANDIT-SKETCH-002]` |
| `BANDIT-SKETCH-SEED` | 1 | `ANCHOR[contextra-adapt:BANDIT-SKETCH-SEED-002]` |
| `BANDIT-SKETCH-FROMSEED` | 1 | `ANCHOR[contextra-adapt:BANDIT-SKETCH-FROMSEED-002]` |
| `BANDIT-SKETCH-PROJECT` | 1 | `ANCHOR[contextra-adapt:BANDIT-SKETCH-PROJECT-002]` |
| `BANDIT-POLICY-TRAIT` | 1 | `ANCHOR[contextra-adapt:BANDIT-POLICY-TRAIT-002]` |
| `BANDIT-PROFILE-STATE` | 1 | `ANCHOR[contextra-adapt:BANDIT-PROFILE-STATE-002]` |
| `BANDIT-COLD-START` | 1 | `ANCHOR[contextra-adapt:BANDIT-COLD-START-002]` |
| `BANDIT-ENSURE-SKETCH` | 1 | `ANCHOR[contextra-adapt:BANDIT-ENSURE-SKETCH-002]` |

3. Neue Bereiche nach dem Modul benennen (z. B. `WAL`, `PUT`, `GATE`); fange bei `001` an, nach Prüfung 1. Setze Anchors sparsam: pro Modul, pro öffentlichem Einstiegspunkt und pro nicht-trivialem Ablauf, nicht pro Zeile.
4. Bereits vorhandene doppelte Ids anderer Stellen nicht „reparieren“; nur im Report melden.

Für Abläufe in Funktionen nutze nummerierte Schritte:

```rust
// ANCHOR[contextra-adapt:PUT-001]
// Ablauf von `put`:
// 1. Eingabe prüfen (Länge, Schlüsselformat).
// 2. Snapshot der Transaktion nehmen.
// 3. Eintrag ins WAL schreiben und mit fsync bestätigen; ein Fehler wird zurückgegeben (Fail-Closed).
// 4. Erst danach im Speicher übernehmen.
```

# Was zu dokumentieren ist

Gehe **jede** Rust-Datei unter `crates/contextra-adapt` durch (`find crates/contextra-adapt -name "*.rs"`, ohne die generierten Dateien oben) einschließlich `tests/`, `benches/`, `examples/` und `build.rs`:

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