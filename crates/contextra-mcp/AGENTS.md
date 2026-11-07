# AGENTS.md — `contextra-mcp`

> **Ring 4** · Klasse `[K]` laut SPEC (Oberfläche) · Reifegrad `beta` · Unsafe: nein · Rolle: MCP-Server (stdio, JSON-RPC 2.0), Prompt-Injection-Guard, Sandbox-Policy, Egress.
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`; diese Datei nennt nur Crate-Spezifisches. Stand 2026-11-07, statisch gelesen, **nicht kompiliert**; Zahlen sind grep-Heuristiken.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-mcp --no-default-features --locked` |
| Contextra-Abhängigkeiten **Ist** (Cargo) | `contextra`, `crypto`, `ports`, `privacy`, `rank`, `router`, `types`, `wire` (+ optional `infer-candle`/`-ollama`/`-onnx`) |
| Contextra-Abhängigkeiten **Soll** (PLAN §4) | `contextra`, `types`, `ports` |
| Features | `candle`, `ollama`, `onnx` (optional), `test-utils` |
| Lints | `#![forbid(unsafe_code)]`, `deny(dead_code)` |

## 2. Abhängigkeiten & Schulden

- **Fünf harte Kanten über das Soll** (`crypto`, `privacy`, `rank`, `router`, `wire`); die drei `infer-*`-Kanten sind bereits optionale Features (gut, M5). Soll: nur über die Fassade (SPEC §4.4 Regel 4).
- **Löschreste:** Werkzeuge `contextra_forget`, `contextra_delete`, `contextra_drop_collection` in `server_tools.rs`; `proof_key.rs`, `proof_key_env.rs` (WP 3.7).
- `egress_*`/`bulk_exfiltration_detector.rs` sind dünne Re-Exports aus `privacy`.

## 3. Modul-Karte (am Code verifiziert)

| Pfad | Verantwortung |
|---|---|
| `server.rs`, `server_dispatch.rs`, `server_tools.rs`, `tools_crud.rs` | Transport, Dispatch, Werkzeuge (`contextra_search`, `_insert`, `_upsert`, `_get`, `_collections`, …) |
| `protocol.rs`, `validation.rs`, `io.rs`, `config.rs` | JSON-RPC, Eingabevalidierung |
| `prompt_injection/` (`guard`, `policy`, `audit`) | Quarantäne |
| `sandbox.rs`, `egress_*.rs`, `explain.rs`, `routing.rs` | Policy, Egress, Erklärungen |
| `bin/contextra-mcp-server.rs` | Binär |

## 4. Verträge & Invarianten

1. **Nur stdio (INV-STDOUT-1):** `stdout` gehört ausschließlich dem Protokoll; Logs nach `stderr`/Datei. Kein HTTP/`axum`/`hyper`.
2. **Schreibende Werkzeuge und Cloud-Egress sind standardmäßig aus** (Opt-in); jede Tool-Ausführung läuft durch Sandbox mit Timeout.
3. **Prompt-Injection-Guard** vor Embeddings/Graph; Treffer ⇒ Quarantäne und Audit.
4. **Recall liefert Daten, nie Anweisungen** (INV-TRUST-1: `trust`-Label).
5. **Breite Löschungen** brauchen Bestätigung über einen Kanal **außerhalb** des Tool-Aufrufs (INV-FORGET-GUARD-1); Anfrageabbruch wirkt kooperativ (INV-CANCEL-1).
6. Kein `unsafe`, auch keine Ausnahme (unvertraute Eingaben).

## 5. Ports

Konsumiert die Fassade; Sicherheits-Ports kommen von dort.

## 6. Nebenläufigkeit & Performance

- Volatile Ergebnisse in einem einzigen, nicht verschachtelten Mutex; Tokio an der Kante. Heuristik: `thread_rng` 2 Treffer (Allowlist ADR-098 prüfen).

## 7. Anti-Patterns

- HTTP-Schnittstelle einführen.
- Werkzeug ohne Sandbox.
- Lösch-Werkzeug ohne Außenkanal-Bestätigung.

## 8. Tests & Messung

```bash
cargo test -p contextra-mcp --locked
cargo clippy -p contextra-mcp --all-targets -- -D warnings
```

- 10 Testdateien; Fuzz: `fuzz_jsonrpc_parsing`, `fuzz_prompt_injection_guard`.

## 9. Offene Arbeit

| WP | Inhalt |
|---|---|
| 3.7 | Löschwerkzeuge herauslösen, Lizenz-Gates entkoppeln |
| 5.3 | Kanten auf Fassade reduzieren |


---


# Kommentar-Syntax (verbindlich)

| Form | Zweck |
|---|---|
| `//!` | Modul-/Crate-Doku am Dateianfang (vor jedem Item, nach `#![...]`-Attributen der Datei; nie mitten in der Datei) |
| `///` | Rustdoc für das **direkt folgende** Item (Struct, Enum, Variante, Feld, Trait, Methode, Funktion, Konstante). Nie vor `let`, Ausdrücken, Match-Armen oder Makroaufrufen; dort `//` verwenden. |
| `// NOTE:` | Warum etwas so gebaut ist (nicht was) |
| `// TODO:` / `// FIXME:` / `// STUB:` | Offene Arbeit / bekanntes Risiko / Platzhalter; immer mit konkreter Aussage |
| `// SAFETY:` | Pflicht direkt über jedem unsafe-Konstrukt (Invariante, Herstellungsort, Folge bei Verletzung) |
| `// ANCHOR[contextra-mcp:<BEREICH>-<NNN>]` | Sprungmarke; vor dem Item/Abschnitt, auf den sie zeigt |
| `// FILE-CONTEXT` + `// ZWECK:` / `// INVARIANTEN:` / `// HOTSPOTS: <von>-<bis>` / `// STAND: <Datum>` | Repo-Konvention für den Dateikopf. Vorhandene Blöcke ergänzen; `HOTSPOTS`-Zeilenbereiche nach dem Kommentieren auf die neuen Zeilennummern aktualisieren. `STAND` auf das heutige Datum setzen, wenn du den Block änderst. |

Weitere Tags führst du nicht ein. Beim Kommentieren markierter Altlasten gilt: `// NOTE: Deprecated Debt DEBT-NN – <Beschreibung>`.

## ANCHOR-Ids (Kollisionsschutz)

Im Repo existieren ca. 3.900 Anchors; Ids sind workspace-weit eindeutig zu halten. Format: `ANCHOR[contextra-mcp:<BEREICH>-<NNN>]` mit `<BEREICH>` in Großbuchstaben/Ziffern/Bindestrichen, `<NNN>` dreistellig. Vorgehen:

1. Vor jeder neuen Id: `grep -rnF "ANCHOR[contextra-mcp:<BEREICH>-<NNN>]" crates benchmarks` muss leer sein.
2. Nummeriere ab dem Maximum des Bereichs weiter. Höchste bestehende Nummern in dieser Crate:

| Bereich | Höchste Nr. | Nächste freie Id |
|---|---:|---|
| – | – | – |

3. Neue Bereiche nach dem Modul benennen (z. B. `WAL`, `PUT`, `GATE`); fange bei `001` an, nach Prüfung 1. Setze Anchors sparsam: pro Modul, pro öffentlichem Einstiegspunkt und pro nicht-trivialem Ablauf, nicht pro Zeile.
4. Bereits vorhandene doppelte Ids anderer Stellen nicht „reparieren“; nur im Report melden.

Für Abläufe in Funktionen nutze nummerierte Schritte:

```rust
// ANCHOR[contextra-mcp:PUT-001]
// Ablauf von `put`:
// 1. Eingabe prüfen (Länge, Schlüsselformat).
// 2. Snapshot der Transaktion nehmen.
// 3. Eintrag ins WAL schreiben und mit fsync bestätigen; ein Fehler wird zurückgegeben (Fail-Closed).
// 4. Erst danach im Speicher übernehmen.
```

# Was zu dokumentieren ist

Gehe **jede** Rust-Datei unter `crates/contextra-mcp` durch (`find crates/contextra-mcp -name "*.rs"`, ohne die generierten Dateien oben) einschließlich `tests/`, `benches/`, `examples/` und `build.rs`:

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
