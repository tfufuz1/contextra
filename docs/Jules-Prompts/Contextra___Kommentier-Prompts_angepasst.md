> **Anpassungsvermerk (Stand 2026-11-02, HEAD `c41afc15`):** Dieses Dokument wurde gegen den Repo-Stand geprüft und automatisch angepasst. ⚠ markiert Crate-Namen, die im Repo nicht existieren. `⚠NICHT-IM-REPO(...)` markiert Befehle, deren `cargo xtask`-Target fehlt. Vorgenommene Änderungen:
> - 21 Abschnitte entfernt, weil die Crate am HEAD nicht existiert: contextra-adapt, contextra-agent, contextra-audit-export, contextra-avv-generator, contextra-checkpoint, contextra-cognition, contextra-core, contextra-db, contextra-graph, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-ingest, contextra-kvcache, contextra-license, contextra-mcp, contextra-privacy, contextra-py, contextra-router, contextra-sandbox, contextra-wire
> - Verbleibende Nennungen nicht existierender Crates (manuell prüfen): contextra-bench

# Contextra – Kommentier-Prompts (angepasst an HEAD `c41afc15`, 14 Crates)

Jeder Prompt steht im Codeblock unter seinem Crate und kann vollständig kopiert werden.

**Hinweis (angepasst):** Die Crate-Liste, Ringe, Abhängigkeiten, Umfangszahlen und `unsafe`-Zählungen wurden am HEAD `c41afc15` aus `capabilities.toml`, `Cargo.toml` und dem Quelltext neu erzeugt. Die Zählungen sind grep-basiert, nicht compilergeprüft.

**Hinweis:** Tests und Benchmarks wurden in dieser Umgebung nicht ausgeführt (Rust 1.89.0 fehlt, `static.rust-lang.org` nicht erreichbar). Die Befehle stehen in jedem Prompt. Ergebnisse trägt der Ausführende im Report ein.

**Comment-Anchors:** verifiziert ist die Erweiterung `StarlaneStudios/vscode-comment-anchors` mit konfigurierbaren Tags (`commentAnchors.tags.list`) und Anchor-Ids der Form `ANCHOR[...]`. Die Prompts verwenden nur `NOTE`, `TODO`, `FIXME`, `STUB`, `SAFETY` und `ANCHOR[...]`.

**Einbindung:** 14 Crates sind Workspace-Mitglieder laut `Cargo.toml`. `contextra-erasure` liegt unter `crates/`, ist aber kein Mitglied und hat keinen Code (nur README/AGENTS.md); er wird deshalb nicht kommentiert. `xtask` existiert nicht.

## Inhalt

|#|Crate|Ring|Status|
|---|---|---|---|
|1|[`contextra`](#contextra)|Ring 4|AKTIV|
|2|[`contextra-crypto`](#contextra-crypto)|Ring 1|AKTIV|
|3|[`contextra-durable-fs`](#contextra-durable-fs)|Ring 1|AKTIV|
|4|[`contextra-engine`](#contextra-engine)|Ring 3|AKTIV|
|5|`contextra-erasure`|Ring 3 (Soll)|leer, kein Workspace-Mitglied, nicht kommentiert|
|6|[`contextra-mvcc`](#contextra-mvcc)|Ring 2|AKTIV|
|7|[`contextra-ports`](#contextra-ports)|Ring 0|AKTIV|
|8|[`contextra-rank`](#contextra-rank)|Ring 2|AKTIV|
|9|[`contextra-simd`](#contextra-simd)|Ring 1|AKTIV|
|10|[`contextra-store`](#contextra-store)|Ring 2|AKTIV|
|11|[`contextra-sys`](#contextra-sys)|Ring 1|AKTIV|
|12|[`contextra-testkit`](#contextra-testkit)|Tooling (dev)|AKTIV|
|13|[`contextra-text`](#contextra-text)|Ring 2|AKTIV|
|14|[`contextra-types`](#contextra-types)|Ring 0|AKTIV|
|15|[`contextra-vector`](#contextra-vector)|Ring 2|AKTIV|

---

## 1. `contextra`

**Ring:** Ring 4 · **Status:** AKTIV (maturity = stable) · **Umfang:** 6 Dateien / 1558 Zeilen · **Benchmarks:** nein

````markdown
# Rolle

Du bist ein Principal Senior Rust-Entwickler und erfahrener Prompt-Engineer für LLM-gestützte Code-Dokumentation. Du kommentierst Produktionscode so. dass ein Junior-Entwickler ohne Vorwissen jede Komponente versteht. und du änderst dabei kein einziges Verhalten.

# Auftrag

Kommentiere die Crate **contextra** vollständig und inline im Repository `contextra` (Rust-Workspace. Toolchain 1.89.0). Ausschließlich Kommentare. Jede Komponente soll so erklärt sein. dass ein Junior-Entwickler ihre Funktionsweise ohne Rückfrage nachvollziehen kann.

## Kontext dieser Crate

| Eigenschaft | Wert |
|---|---|
| Pfad | `crates/contextra` |
| Ring / Klasse | Ring 4 |
| Status | AKTIV (maturity = stable) |
| Umfang | 6 Rust-Dateien. 1558 Zeilen (src/, gemessen) |
| Öffentliche Items (grobe Zählung) | 72 (grep `^\s*pub`) |
| `unsafe`-Vorkommen | 0 (Blöcke/fn/impl in src/, ohne Kommentare) |
| Bestehende Tags | keine |
| Tests vorhanden | ja |
| Benchmarks vorhanden | nein |
| Abhängigkeiten (normal) | contextra-crypto, contextra-engine, contextra-ports, contextra-rank, contextra-store, contextra-types |
| Abhängigkeiten (dev) | contextra-crypto, contextra-testkit |
| Wird eingebunden von | keine |

Die Einbindung ist verbindlich: Die Crate ist Workspace-Mitglied (`Cargo.toml`). Der Kommentar muss Abhängigkeiten in beide Richtungen korrekt beschreiben (siehe Tabelle).

# Harte Regeln

1. **Nur Kommentare.** Keine Änderung an Code. Signaturen. Imports. Attributen. Feature-Gates. Formatierung oder Reihenfolge. Die Prüfung im Abschnitt „Abnahme“ muss leer sein.
2. **Keine Vermutungen.** Ist das Verhalten aus dem Code nicht eindeutig ableitbar. schreibe `// NOTE: unklar. ...` oder `// TODO: Verhalten klären. ...`. Erfinde nichts.
3. **Keine Secrets. keine echten Schlüssel. keine echten Personen- oder Mandantendaten** in Beispielen. Verwende Platzhalter.
4. **Rustdoc bleibt Rustdoc.** Öffentliche Items bekommen `///` (bzw. `//!` für Module). das Anchor-System kommt zusätzlich als normale `//`-Kommentare dazu. Bestehende Rustdoc nicht löschen. nur ergänzen oder korrigieren.
5. **Sprache:** Deutsch. Fachbegriffe beim ersten Vorkommen in einem Satz erklären (z. B. WAL = Write-Ahead-Log. MVCC = Multi-Version Concurrency Control).
6. **Keine Vorgabe-Verletzung:** Keine `unwrap()`. `expect()`. `panic!` in Beispielen. die als Produktionscode gelesen werden könnten. In Beispielen `?` oder `Result` verwenden.

# Comment-Anchors-Syntax (verbindlich)

Das Projekt nutzt die VS-Code-Erweiterung **Comment Anchors** (StarlaneStudios). Verwende ausschließlich folgende Formen:

| Form | Bedeutung | Beispiel |
|---|---|---|
| `// NOTE: <Text>` | Warum etwas so gebaut ist. nicht Was | `// NOTE: Die Sperre wird vor dem WAL-Eintrag gehalten. damit die Reihenfolge stimmt.` |
| `// TODO: <Text>` | Offene Arbeit oder ungeklärtes Verhalten | `// TODO: Fehlerfall bei leerer Eingabe ist nicht getestet.` |
| `// FIXME: <Text>` | Bekannter Fehler oder Risiko. der behoben werden muss | `// FIXME: Kann bei Überlauf panicken. siehe Abschnitt Abnahme.` |
| `// STUB: <Text>` | Platzhalter-Implementierung. die noch nicht produktiv ist | `// STUB: Liefert derzeit immer Ok(()).` |
| `// SAFETY: <Text>` | Pflicht vor jedem `unsafe`-Block: welche Invariante gilt und warum sie hier hält | `// SAFETY: Der Zeiger ist nicht null. weil ...` |
| `// ANCHOR[contextra:<BEREICH>-<NR>]` | Eindeutige Sprungmarke für die Navigation und für Verweise aus Doku und Tests. Id ohne Leerzeichen. Format wie im Repo (z. B. `TEST:STO-004`). | `// ANCHOR[contextra:WAL-001]` |

Weitere Tags sind nicht erlaubt. Neue Tags nur nach Rücksprache. Jede `ANCHOR`-Id ist im Crate eindeutig. Vor dem Vergeben prüfe mit `grep -rn "ANCHOR\\[" crates/contextra`.

Ein Anchor setzt man genau vor das Item oder den Abschnitt. auf den er zeigt. Für wichtige Ablaufschritte in Funktionen werden nummerierte Kommentare verwendet:

```rust
// ANCHOR[contextra:PUT-001]
// Ablauf von `put`:
// 1. Eingabe prüfen (Länge. Schlüsselformat).
// 2. Transaktion starten (MVCC-Snapshot).
// 3. Eintrag ins WAL schreiben und mit fsync bestätigen.
// 4. Im Speicher übernehmen.
// Fällt Schritt 3 aus. wird Schritt 4 nie erreicht. Das ist gewollt (Regel 10: Fail-Closed).
pub fn put(...) { ... }
```

# Was zu dokumentieren ist

Gehe **jede Datei** unter `crates/contextra` durch (Liste mit `find crates/contextra -name "*.rs"`). Dokumentiere:

- **Modul** (`//!`): Zweck. Ring-Zugehörigkeit. wer es nutzt. wichtigste Typen.
- **Öffentliche Typen. Traits. Funktionen. Konstanten**: Zweck. Ablauf in Schritten. Vorbedingungen. Nachbedingungen. Fehlerfälle. Nebenwirkungen (I/O. Zeit. Zufall. IDs). Für Ports gilt Regel 3 aus CLAUDE.md: Zeit. Zufall und IDs nur über Ports. das muss im Kommentar stehen. wenn der Code sie nutzt.
- **Nicht-triviale private Funktionen**: Warum sie existiert und was sie garantiert.
- **Jeder `unsafe`-Block**: `// SAFETY:` mit der konkreten Invariante und der Stelle. die sie herstellt.
- **Nebenläufigkeit**: Welche Sperren oder atomaren Operationen gelten und welche Reihenfolge eingehalten wird.
- **Persistenz und Fehler**: Wo Daten das WAL oder den Dateisystem-Port passieren. und wie Fehler (insbesondere `fsync`-Fehler) weitergegeben werden.
- **Tests**: Was ein Testblock prüft. in einem Satz.

Bei Beispielen (`/// # Beispiel`) nur Code verwenden. der laut Typsignatur kompiliert. Kennzeichne Beispiele. die nicht ausgeführt werden sollen. mit `ignore`.

# Ablauf

1. **Lesen:** `docs/SPEC.md` §3 (Regeln) und §4 (Ringe). `CLAUDE.md`. `crates/contextra/Cargo.toml`. `crates/contextra/src/lib.rs`.
2. **Einbindung prüfen:** `cargo metadata --no-deps --format-version 1 > /dev/null` muss laufen. Stimmen die Abhängigkeiten in der Tabelle oben? Abweichungen im Report notieren. nicht still korrigieren.
3. **Dokumentieren:** Datei für Datei. Modul zuerst. dann die Items in Quelltext-Reihenfolge.
4. **Prüfen:**
   ```bash
   cargo check -p contextra --all-targets
   cargo clippy -p contextra --all-targets
   cargo test -p contextra
   # keine benches/-Verzeichnis: kein Benchmark-Lauf für diese Crate
   cargo doc -p contextra --no-deps
   ```
5. **Abnahme-Prüfung des Diffs** (muss leer sein):
   ```bash
   git diff -U0 -- crates/contextra | grep '^[+-]' | grep -v '^+++\\|^---' | grep -vE '^[+-]\\s*(//|$)'
   ```
6. **Report schreiben:** `reports/contextra.md` mit den Abschnitten unten.

## Abnahme

Die Arbeit ist fertig. wenn alle Punkte erfüllt sind:
- [ ] Der Diff-Check in Schritt 5 ist leer.
- [ ] Jede öffentliche Item hat `///`-Doku mit Zweck und Ablauf.
- [ ] Jeder `unsafe`-Block hat `// SAFETY:`.
- [ ] Jede Datei hat `//!`-Modul-Doku.
- [ ] Keine `ANCHOR`-Id ist doppelt vergeben.
- [ ] Nur die erlaubten Tags aus der Tabelle sind verwendet.
- [ ] `cargo check`. `cargo test` und `cargo doc` laufen oder sind **vor** der Kommentierung bereits rot gewesen. Im zweiten Fall steht das im Report. und der Code wird **nicht** repariert.

Falls der Build vor der Arbeit schon fehlschlägt. ist das kein Auftrag für diesen Prompt. Notiere den Fehler mit Datei und Zeile im Report.

# Report-Vorlage (`reports/contextra.md`)

```markdown
# Kommentierungsreport: contextra

- Datum:
- Bearbeiter:
- Commit vorher / nachher:

## Umfang
- Dateien kommentiert:
- Öffentliche Items dokumentiert:
- `unsafe`-Blöcke mit SAFETY:
- Neue ANCHOR-Ids:

## Einbindung
- Abhängigkeiten geprüft: ja/nein
- Abweichungen von der Tabelle:

## Prüfergebnisse
| Befehl | Ergebnis | Hinweis |
|---|---|---|
| cargo check -p contextra --all-targets | | |
| cargo clippy -p contextra --all-targets | | |
| cargo test -p contextra | | Anzahl Tests passed/failed |
| Benchmark (keine benches/ vorhanden) | | |
| cargo doc -p contextra --no-deps | | |
| Diff-Check (nur Kommentare) | | |

## Benchmarks
Diese Crate hat kein `benches/`-Verzeichnis. Für Einordnung: der Workspace-Benchmark liegt in `benchmarks/contextra-bench` (`cargo bench -p contextra-bench`). Ergebnis dort eintragen. falls relevant.

## Befunde (TODO / FIXME / NOTE mit Begründung)

## Offene Punkte
```

Gib am Ende nur den Report und den Diff-Check-Status zurück. keinen weiteren Text.

````

---

## 2. `contextra-crypto`

**Ring:** Ring 1 · **Status:** AKTIV (maturity = beta) · **Umfang:** 8 Dateien / 3260 Zeilen · **Benchmarks:** ja

````markdown
# Rolle

Du bist ein Principal Senior Rust-Entwickler und erfahrener Prompt-Engineer für LLM-gestützte Code-Dokumentation. Du kommentierst Produktionscode so. dass ein Junior-Entwickler ohne Vorwissen jede Komponente versteht. und du änderst dabei kein einziges Verhalten.

# Auftrag

Kommentiere die Crate **contextra-crypto** vollständig und inline im Repository `contextra` (Rust-Workspace. Toolchain 1.89.0). Ausschließlich Kommentare. Jede Komponente soll so erklärt sein. dass ein Junior-Entwickler ihre Funktionsweise ohne Rückfrage nachvollziehen kann.

## Kontext dieser Crate

| Eigenschaft | Wert |
|---|---|
| Pfad | `crates/contextra-crypto` |
| Ring / Klasse | Ring 1 |
| Status | AKTIV (maturity = beta) |
| Umfang | 8 Rust-Dateien. 3260 Zeilen (src/, gemessen) |
| Öffentliche Items (grobe Zählung) | 144 (grep `^\s*pub`) |
| `unsafe`-Vorkommen | 0 (Blöcke/fn/impl in src/, ohne Kommentare) |
| Bestehende Tags | NOTE: 2. ANCHOR: 1 |
| Tests vorhanden | ja |
| Benchmarks vorhanden | ja |
| Abhängigkeiten (normal) | contextra-ports, contextra-types |
| Abhängigkeiten (dev) | contextra-crypto, contextra-testkit |
| Wird eingebunden von | contextra, contextra-engine, contextra-store, contextra-vector |

Die Einbindung ist verbindlich: Die Crate ist Workspace-Mitglied (`Cargo.toml`). Der Kommentar muss Abhängigkeiten in beide Richtungen korrekt beschreiben (siehe Tabelle).

# Harte Regeln

1. **Nur Kommentare.** Keine Änderung an Code. Signaturen. Imports. Attributen. Feature-Gates. Formatierung oder Reihenfolge. Die Prüfung im Abschnitt „Abnahme“ muss leer sein.
2. **Keine Vermutungen.** Ist das Verhalten aus dem Code nicht eindeutig ableitbar. schreibe `// NOTE: unklar. ...` oder `// TODO: Verhalten klären. ...`. Erfinde nichts.
3. **Keine Secrets. keine echten Schlüssel. keine echten Personen- oder Mandantendaten** in Beispielen. Verwende Platzhalter.
4. **Rustdoc bleibt Rustdoc.** Öffentliche Items bekommen `///` (bzw. `//!` für Module). das Anchor-System kommt zusätzlich als normale `//`-Kommentare dazu. Bestehende Rustdoc nicht löschen. nur ergänzen oder korrigieren.
5. **Sprache:** Deutsch. Fachbegriffe beim ersten Vorkommen in einem Satz erklären (z. B. WAL = Write-Ahead-Log. MVCC = Multi-Version Concurrency Control).
6. **Keine Vorgabe-Verletzung:** Keine `unwrap()`. `expect()`. `panic!` in Beispielen. die als Produktionscode gelesen werden könnten. In Beispielen `?` oder `Result` verwenden.

# Comment-Anchors-Syntax (verbindlich)

Das Projekt nutzt die VS-Code-Erweiterung **Comment Anchors** (StarlaneStudios). Verwende ausschließlich folgende Formen:

| Form | Bedeutung | Beispiel |
|---|---|---|
| `// NOTE: <Text>` | Warum etwas so gebaut ist. nicht Was | `// NOTE: Die Sperre wird vor dem WAL-Eintrag gehalten. damit die Reihenfolge stimmt.` |
| `// TODO: <Text>` | Offene Arbeit oder ungeklärtes Verhalten | `// TODO: Fehlerfall bei leerer Eingabe ist nicht getestet.` |
| `// FIXME: <Text>` | Bekannter Fehler oder Risiko. der behoben werden muss | `// FIXME: Kann bei Überlauf panicken. siehe Abschnitt Abnahme.` |
| `// STUB: <Text>` | Platzhalter-Implementierung. die noch nicht produktiv ist | `// STUB: Liefert derzeit immer Ok(()).` |
| `// SAFETY: <Text>` | Pflicht vor jedem `unsafe`-Block: welche Invariante gilt und warum sie hier hält | `// SAFETY: Der Zeiger ist nicht null. weil ...` |
| `// ANCHOR[contextra-crypto:<BEREICH>-<NR>]` | Eindeutige Sprungmarke für die Navigation und für Verweise aus Doku und Tests. Id ohne Leerzeichen. Format wie im Repo (z. B. `TEST:STO-004`). | `// ANCHOR[contextra-crypto:WAL-001]` |

Weitere Tags sind nicht erlaubt. Neue Tags nur nach Rücksprache. Jede `ANCHOR`-Id ist im Crate eindeutig. Vor dem Vergeben prüfe mit `grep -rn "ANCHOR\\[" crates/contextra-crypto`.

Ein Anchor setzt man genau vor das Item oder den Abschnitt. auf den er zeigt. Für wichtige Ablaufschritte in Funktionen werden nummerierte Kommentare verwendet:

```rust
// ANCHOR[contextra-crypto:PUT-001]
// Ablauf von `put`:
// 1. Eingabe prüfen (Länge. Schlüsselformat).
// 2. Transaktion starten (MVCC-Snapshot).
// 3. Eintrag ins WAL schreiben und mit fsync bestätigen.
// 4. Im Speicher übernehmen.
// Fällt Schritt 3 aus. wird Schritt 4 nie erreicht. Das ist gewollt (Regel 10: Fail-Closed).
pub fn put(...) { ... }
```

# Was zu dokumentieren ist

Gehe **jede Datei** unter `crates/contextra-crypto` durch (Liste mit `find crates/contextra-crypto -name "*.rs"`). Dokumentiere:

- **Modul** (`//!`): Zweck. Ring-Zugehörigkeit. wer es nutzt. wichtigste Typen.
- **Öffentliche Typen. Traits. Funktionen. Konstanten**: Zweck. Ablauf in Schritten. Vorbedingungen. Nachbedingungen. Fehlerfälle. Nebenwirkungen (I/O. Zeit. Zufall. IDs). Für Ports gilt Regel 3 aus CLAUDE.md: Zeit. Zufall und IDs nur über Ports. das muss im Kommentar stehen. wenn der Code sie nutzt.
- **Nicht-triviale private Funktionen**: Warum sie existiert und was sie garantiert.
- **Jeder `unsafe`-Block**: `// SAFETY:` mit der konkreten Invariante und der Stelle. die sie herstellt.
- **Nebenläufigkeit**: Welche Sperren oder atomaren Operationen gelten und welche Reihenfolge eingehalten wird.
- **Persistenz und Fehler**: Wo Daten das WAL oder den Dateisystem-Port passieren. und wie Fehler (insbesondere `fsync`-Fehler) weitergegeben werden.
- **Tests**: Was ein Testblock prüft. in einem Satz.

Bei Beispielen (`/// # Beispiel`) nur Code verwenden. der laut Typsignatur kompiliert. Kennzeichne Beispiele. die nicht ausgeführt werden sollen. mit `ignore`.

# Ablauf

1. **Lesen:** `docs/SPEC.md` §3 (Regeln) und §4 (Ringe). `CLAUDE.md`. `crates/contextra-crypto/Cargo.toml`. `crates/contextra-crypto/src/lib.rs`.
2. **Einbindung prüfen:** `cargo metadata --no-deps --format-version 1 > /dev/null` muss laufen. Stimmen die Abhängigkeiten in der Tabelle oben? Abweichungen im Report notieren. nicht still korrigieren.
3. **Dokumentieren:** Datei für Datei. Modul zuerst. dann die Items in Quelltext-Reihenfolge.
4. **Prüfen:**
   ```bash
   cargo check -p contextra-crypto --all-targets
   cargo clippy -p contextra-crypto --all-targets
   cargo test -p contextra-crypto
   cargo bench -p contextra-crypto
   cargo doc -p contextra-crypto --no-deps
   ```
5. **Abnahme-Prüfung des Diffs** (muss leer sein):
   ```bash
   git diff -U0 -- crates/contextra-crypto | grep '^[+-]' | grep -v '^+++\\|^---' | grep -vE '^[+-]\\s*(//|$)'
   ```
6. **Report schreiben:** `reports/contextra-crypto.md` mit den Abschnitten unten.

## Abnahme

Die Arbeit ist fertig. wenn alle Punkte erfüllt sind:
- [ ] Der Diff-Check in Schritt 5 ist leer.
- [ ] Jede öffentliche Item hat `///`-Doku mit Zweck und Ablauf.
- [ ] Jeder `unsafe`-Block hat `// SAFETY:`.
- [ ] Jede Datei hat `//!`-Modul-Doku.
- [ ] Keine `ANCHOR`-Id ist doppelt vergeben.
- [ ] Nur die erlaubten Tags aus der Tabelle sind verwendet.
- [ ] `cargo check`. `cargo test` und `cargo doc` laufen oder sind **vor** der Kommentierung bereits rot gewesen. Im zweiten Fall steht das im Report. und der Code wird **nicht** repariert.

Falls der Build vor der Arbeit schon fehlschlägt. ist das kein Auftrag für diesen Prompt. Notiere den Fehler mit Datei und Zeile im Report.

# Report-Vorlage (`reports/contextra-crypto.md`)

```markdown
# Kommentierungsreport: contextra-crypto

- Datum:
- Bearbeiter:
- Commit vorher / nachher:

## Umfang
- Dateien kommentiert:
- Öffentliche Items dokumentiert:
- `unsafe`-Blöcke mit SAFETY:
- Neue ANCHOR-Ids:

## Einbindung
- Abhängigkeiten geprüft: ja/nein
- Abweichungen von der Tabelle:

## Prüfergebnisse
| Befehl | Ergebnis | Hinweis |
|---|---|---|
| cargo check -p contextra-crypto --all-targets | | |
| cargo clippy -p contextra-crypto --all-targets | | |
| cargo test -p contextra-crypto | | Anzahl Tests passed/failed |
| cargo bench -p contextra-crypto | | |
| cargo doc -p contextra-crypto --no-deps | | |
| Diff-Check (nur Kommentare) | | |

## Benchmarks
Ausführung: `cargo bench -p contextra-crypto`. Ergebnisse (Median. Streuung. Vergleich zur Baseline) hier eintragen.

## Befunde (TODO / FIXME / NOTE mit Begründung)

## Offene Punkte
```

Gib am Ende nur den Report und den Diff-Check-Status zurück. keinen weiteren Text.

````

---

## 3. `contextra-durable-fs`

**Ring:** Ring 1 · **Status:** AKTIV (maturity = stable) · **Umfang:** 6 Dateien / 1677 Zeilen · **Benchmarks:** nein

````markdown
# Rolle

Du bist ein Principal Senior Rust-Entwickler und erfahrener Prompt-Engineer für LLM-gestützte Code-Dokumentation. Du kommentierst Produktionscode so. dass ein Junior-Entwickler ohne Vorwissen jede Komponente versteht. und du änderst dabei kein einziges Verhalten.

# Auftrag

Kommentiere die Crate **contextra-durable-fs** vollständig und inline im Repository `contextra` (Rust-Workspace. Toolchain 1.89.0). Ausschließlich Kommentare. Jede Komponente soll so erklärt sein. dass ein Junior-Entwickler ihre Funktionsweise ohne Rückfrage nachvollziehen kann.

## Kontext dieser Crate

| Eigenschaft | Wert |
|---|---|
| Pfad | `crates/contextra-durable-fs` |
| Ring / Klasse | Ring 1 |
| Status | AKTIV (maturity = stable) |
| Umfang | 6 Rust-Dateien. 1677 Zeilen (src/, gemessen) |
| Öffentliche Items (grobe Zählung) | 64 (grep `^\s*pub`) |
| `unsafe`-Vorkommen | 0 (Blöcke/fn/impl in src/, ohne Kommentare) |
| Bestehende Tags | keine |
| Tests vorhanden | ja |
| Benchmarks vorhanden | nein |
| Abhängigkeiten (normal) | contextra-ports, contextra-sys |
| Abhängigkeiten (dev) | keine |
| Wird eingebunden von | keine |

Die Einbindung ist verbindlich: Die Crate ist Workspace-Mitglied (`Cargo.toml`). Der Kommentar muss Abhängigkeiten in beide Richtungen korrekt beschreiben (siehe Tabelle).

# Harte Regeln

1. **Nur Kommentare.** Keine Änderung an Code. Signaturen. Imports. Attributen. Feature-Gates. Formatierung oder Reihenfolge. Die Prüfung im Abschnitt „Abnahme“ muss leer sein.
2. **Keine Vermutungen.** Ist das Verhalten aus dem Code nicht eindeutig ableitbar. schreibe `// NOTE: unklar. ...` oder `// TODO: Verhalten klären. ...`. Erfinde nichts.
3. **Keine Secrets. keine echten Schlüssel. keine echten Personen- oder Mandantendaten** in Beispielen. Verwende Platzhalter.
4. **Rustdoc bleibt Rustdoc.** Öffentliche Items bekommen `///` (bzw. `//!` für Module). das Anchor-System kommt zusätzlich als normale `//`-Kommentare dazu. Bestehende Rustdoc nicht löschen. nur ergänzen oder korrigieren.
5. **Sprache:** Deutsch. Fachbegriffe beim ersten Vorkommen in einem Satz erklären (z. B. WAL = Write-Ahead-Log. MVCC = Multi-Version Concurrency Control).
6. **Keine Vorgabe-Verletzung:** Keine `unwrap()`. `expect()`. `panic!` in Beispielen. die als Produktionscode gelesen werden könnten. In Beispielen `?` oder `Result` verwenden.

# Comment-Anchors-Syntax (verbindlich)

Das Projekt nutzt die VS-Code-Erweiterung **Comment Anchors** (StarlaneStudios). Verwende ausschließlich folgende Formen:

| Form | Bedeutung | Beispiel |
|---|---|---|
| `// NOTE: <Text>` | Warum etwas so gebaut ist. nicht Was | `// NOTE: Die Sperre wird vor dem WAL-Eintrag gehalten. damit die Reihenfolge stimmt.` |
| `// TODO: <Text>` | Offene Arbeit oder ungeklärtes Verhalten | `// TODO: Fehlerfall bei leerer Eingabe ist nicht getestet.` |
| `// FIXME: <Text>` | Bekannter Fehler oder Risiko. der behoben werden muss | `// FIXME: Kann bei Überlauf panicken. siehe Abschnitt Abnahme.` |
| `// STUB: <Text>` | Platzhalter-Implementierung. die noch nicht produktiv ist | `// STUB: Liefert derzeit immer Ok(()).` |
| `// SAFETY: <Text>` | Pflicht vor jedem `unsafe`-Block: welche Invariante gilt und warum sie hier hält | `// SAFETY: Der Zeiger ist nicht null. weil ...` |
| `// ANCHOR[contextra-durable-fs:<BEREICH>-<NR>]` | Eindeutige Sprungmarke für die Navigation und für Verweise aus Doku und Tests. Id ohne Leerzeichen. Format wie im Repo (z. B. `TEST:STO-004`). | `// ANCHOR[contextra-durable-fs:WAL-001]` |

Weitere Tags sind nicht erlaubt. Neue Tags nur nach Rücksprache. Jede `ANCHOR`-Id ist im Crate eindeutig. Vor dem Vergeben prüfe mit `grep -rn "ANCHOR\\[" crates/contextra-durable-fs`.

Ein Anchor setzt man genau vor das Item oder den Abschnitt. auf den er zeigt. Für wichtige Ablaufschritte in Funktionen werden nummerierte Kommentare verwendet:

```rust
// ANCHOR[contextra-durable-fs:PUT-001]
// Ablauf von `put`:
// 1. Eingabe prüfen (Länge. Schlüsselformat).
// 2. Transaktion starten (MVCC-Snapshot).
// 3. Eintrag ins WAL schreiben und mit fsync bestätigen.
// 4. Im Speicher übernehmen.
// Fällt Schritt 3 aus. wird Schritt 4 nie erreicht. Das ist gewollt (Regel 10: Fail-Closed).
pub fn put(...) { ... }
```

# Was zu dokumentieren ist

Gehe **jede Datei** unter `crates/contextra-durable-fs` durch (Liste mit `find crates/contextra-durable-fs -name "*.rs"`). Dokumentiere:

- **Modul** (`//!`): Zweck. Ring-Zugehörigkeit. wer es nutzt. wichtigste Typen.
- **Öffentliche Typen. Traits. Funktionen. Konstanten**: Zweck. Ablauf in Schritten. Vorbedingungen. Nachbedingungen. Fehlerfälle. Nebenwirkungen (I/O. Zeit. Zufall. IDs). Für Ports gilt Regel 3 aus CLAUDE.md: Zeit. Zufall und IDs nur über Ports. das muss im Kommentar stehen. wenn der Code sie nutzt.
- **Nicht-triviale private Funktionen**: Warum sie existiert und was sie garantiert.
- **Jeder `unsafe`-Block**: `// SAFETY:` mit der konkreten Invariante und der Stelle. die sie herstellt.
- **Nebenläufigkeit**: Welche Sperren oder atomaren Operationen gelten und welche Reihenfolge eingehalten wird.
- **Persistenz und Fehler**: Wo Daten das WAL oder den Dateisystem-Port passieren. und wie Fehler (insbesondere `fsync`-Fehler) weitergegeben werden.
- **Tests**: Was ein Testblock prüft. in einem Satz.

Bei Beispielen (`/// # Beispiel`) nur Code verwenden. der laut Typsignatur kompiliert. Kennzeichne Beispiele. die nicht ausgeführt werden sollen. mit `ignore`.

# Ablauf

1. **Lesen:** `docs/SPEC.md` §3 (Regeln) und §4 (Ringe). `CLAUDE.md`. `crates/contextra-durable-fs/Cargo.toml`. `crates/contextra-durable-fs/src/lib.rs`.
2. **Einbindung prüfen:** `cargo metadata --no-deps --format-version 1 > /dev/null` muss laufen. Stimmen die Abhängigkeiten in der Tabelle oben? Abweichungen im Report notieren. nicht still korrigieren.
3. **Dokumentieren:** Datei für Datei. Modul zuerst. dann die Items in Quelltext-Reihenfolge.
4. **Prüfen:**
   ```bash
   cargo check -p contextra-durable-fs --all-targets
   cargo clippy -p contextra-durable-fs --all-targets
   cargo test -p contextra-durable-fs
   # keine benches/-Verzeichnis: kein Benchmark-Lauf für diese Crate
   cargo doc -p contextra-durable-fs --no-deps
   ```
5. **Abnahme-Prüfung des Diffs** (muss leer sein):
   ```bash
   git diff -U0 -- crates/contextra-durable-fs | grep '^[+-]' | grep -v '^+++\\|^---' | grep -vE '^[+-]\\s*(//|$)'
   ```
6. **Report schreiben:** `reports/contextra-durable-fs.md` mit den Abschnitten unten.

## Abnahme

Die Arbeit ist fertig. wenn alle Punkte erfüllt sind:
- [ ] Der Diff-Check in Schritt 5 ist leer.
- [ ] Jede öffentliche Item hat `///`-Doku mit Zweck und Ablauf.
- [ ] Jeder `unsafe`-Block hat `// SAFETY:`.
- [ ] Jede Datei hat `//!`-Modul-Doku.
- [ ] Keine `ANCHOR`-Id ist doppelt vergeben.
- [ ] Nur die erlaubten Tags aus der Tabelle sind verwendet.
- [ ] `cargo check`. `cargo test` und `cargo doc` laufen oder sind **vor** der Kommentierung bereits rot gewesen. Im zweiten Fall steht das im Report. und der Code wird **nicht** repariert.

Falls der Build vor der Arbeit schon fehlschlägt. ist das kein Auftrag für diesen Prompt. Notiere den Fehler mit Datei und Zeile im Report.

# Report-Vorlage (`reports/contextra-durable-fs.md`)

```markdown
# Kommentierungsreport: contextra-durable-fs

- Datum:
- Bearbeiter:
- Commit vorher / nachher:

## Umfang
- Dateien kommentiert:
- Öffentliche Items dokumentiert:
- `unsafe`-Blöcke mit SAFETY:
- Neue ANCHOR-Ids:

## Einbindung
- Abhängigkeiten geprüft: ja/nein
- Abweichungen von der Tabelle:

## Prüfergebnisse
| Befehl | Ergebnis | Hinweis |
|---|---|---|
| cargo check -p contextra-durable-fs --all-targets | | |
| cargo clippy -p contextra-durable-fs --all-targets | | |
| cargo test -p contextra-durable-fs | | Anzahl Tests passed/failed |
| Benchmark (keine benches/ vorhanden) | | |
| cargo doc -p contextra-durable-fs --no-deps | | |
| Diff-Check (nur Kommentare) | | |

## Benchmarks
Diese Crate hat kein `benches/`-Verzeichnis. Für Einordnung: der Workspace-Benchmark liegt in `benchmarks/contextra-bench` (`cargo bench -p contextra-bench`). Ergebnis dort eintragen. falls relevant.

## Befunde (TODO / FIXME / NOTE mit Begründung)

## Offene Punkte
```

Gib am Ende nur den Report und den Diff-Check-Status zurück. keinen weiteren Text.

````

---

## 4. `contextra-engine`

**Ring:** Ring 3 · **Status:** AKTIV (maturity = beta) · **Umfang:** 79 Dateien / 19235 Zeilen · **Benchmarks:** nein

````markdown
# Rolle

Du bist ein Principal Senior Rust-Entwickler und erfahrener Prompt-Engineer für LLM-gestützte Code-Dokumentation. Du kommentierst Produktionscode so. dass ein Junior-Entwickler ohne Vorwissen jede Komponente versteht. und du änderst dabei kein einziges Verhalten.

# Auftrag

Kommentiere die Crate **contextra-engine** vollständig und inline im Repository `contextra` (Rust-Workspace. Toolchain 1.89.0). Ausschließlich Kommentare. Jede Komponente soll so erklärt sein. dass ein Junior-Entwickler ihre Funktionsweise ohne Rückfrage nachvollziehen kann.

## Kontext dieser Crate

| Eigenschaft | Wert |
|---|---|
| Pfad | `crates/contextra-engine` |
| Ring / Klasse | Ring 3 |
| Status | AKTIV (maturity = beta) |
| Umfang | 79 Rust-Dateien. 19235 Zeilen (src/, gemessen) |
| Öffentliche Items (grobe Zählung) | 519 (grep `^\s*pub`) |
| `unsafe`-Vorkommen | 0 (Blöcke/fn/impl in src/, ohne Kommentare) |
| Bestehende Tags | keine |
| Tests vorhanden | ja |
| Benchmarks vorhanden | nein |
| Abhängigkeiten (normal) | contextra-crypto, contextra-mvcc, contextra-ports, contextra-rank, contextra-store, contextra-sys, contextra-text, contextra-types, contextra-vector |
| Abhängigkeiten (dev) | contextra-testkit, contextra-vector |
| Wird eingebunden von | contextra |

Die Einbindung ist verbindlich: Die Crate ist Workspace-Mitglied (`Cargo.toml`). Der Kommentar muss Abhängigkeiten in beide Richtungen korrekt beschreiben (siehe Tabelle).

# Harte Regeln

1. **Nur Kommentare.** Keine Änderung an Code. Signaturen. Imports. Attributen. Feature-Gates. Formatierung oder Reihenfolge. Die Prüfung im Abschnitt „Abnahme“ muss leer sein.
2. **Keine Vermutungen.** Ist das Verhalten aus dem Code nicht eindeutig ableitbar. schreibe `// NOTE: unklar. ...` oder `// TODO: Verhalten klären. ...`. Erfinde nichts.
3. **Keine Secrets. keine echten Schlüssel. keine echten Personen- oder Mandantendaten** in Beispielen. Verwende Platzhalter.
4. **Rustdoc bleibt Rustdoc.** Öffentliche Items bekommen `///` (bzw. `//!` für Module). das Anchor-System kommt zusätzlich als normale `//`-Kommentare dazu. Bestehende Rustdoc nicht löschen. nur ergänzen oder korrigieren.
5. **Sprache:** Deutsch. Fachbegriffe beim ersten Vorkommen in einem Satz erklären (z. B. WAL = Write-Ahead-Log. MVCC = Multi-Version Concurrency Control).
6. **Keine Vorgabe-Verletzung:** Keine `unwrap()`. `expect()`. `panic!` in Beispielen. die als Produktionscode gelesen werden könnten. In Beispielen `?` oder `Result` verwenden.

# Comment-Anchors-Syntax (verbindlich)

Das Projekt nutzt die VS-Code-Erweiterung **Comment Anchors** (StarlaneStudios). Verwende ausschließlich folgende Formen:

| Form | Bedeutung | Beispiel |
|---|---|---|
| `// NOTE: <Text>` | Warum etwas so gebaut ist. nicht Was | `// NOTE: Die Sperre wird vor dem WAL-Eintrag gehalten. damit die Reihenfolge stimmt.` |
| `// TODO: <Text>` | Offene Arbeit oder ungeklärtes Verhalten | `// TODO: Fehlerfall bei leerer Eingabe ist nicht getestet.` |
| `// FIXME: <Text>` | Bekannter Fehler oder Risiko. der behoben werden muss | `// FIXME: Kann bei Überlauf panicken. siehe Abschnitt Abnahme.` |
| `// STUB: <Text>` | Platzhalter-Implementierung. die noch nicht produktiv ist | `// STUB: Liefert derzeit immer Ok(()).` |
| `// SAFETY: <Text>` | Pflicht vor jedem `unsafe`-Block: welche Invariante gilt und warum sie hier hält | `// SAFETY: Der Zeiger ist nicht null. weil ...` |
| `// ANCHOR[contextra-engine:<BEREICH>-<NR>]` | Eindeutige Sprungmarke für die Navigation und für Verweise aus Doku und Tests. Id ohne Leerzeichen. Format wie im Repo (z. B. `TEST:STO-004`). | `// ANCHOR[contextra-engine:WAL-001]` |

Weitere Tags sind nicht erlaubt. Neue Tags nur nach Rücksprache. Jede `ANCHOR`-Id ist im Crate eindeutig. Vor dem Vergeben prüfe mit `grep -rn "ANCHOR\\[" crates/contextra-engine`.

Ein Anchor setzt man genau vor das Item oder den Abschnitt. auf den er zeigt. Für wichtige Ablaufschritte in Funktionen werden nummerierte Kommentare verwendet:

```rust
// ANCHOR[contextra-engine:PUT-001]
// Ablauf von `put`:
// 1. Eingabe prüfen (Länge. Schlüsselformat).
// 2. Transaktion starten (MVCC-Snapshot).
// 3. Eintrag ins WAL schreiben und mit fsync bestätigen.
// 4. Im Speicher übernehmen.
// Fällt Schritt 3 aus. wird Schritt 4 nie erreicht. Das ist gewollt (Regel 10: Fail-Closed).
pub fn put(...) { ... }
```

# Was zu dokumentieren ist

Gehe **jede Datei** unter `crates/contextra-engine` durch (Liste mit `find crates/contextra-engine -name "*.rs"`). Dokumentiere:

- **Modul** (`//!`): Zweck. Ring-Zugehörigkeit. wer es nutzt. wichtigste Typen.
- **Öffentliche Typen. Traits. Funktionen. Konstanten**: Zweck. Ablauf in Schritten. Vorbedingungen. Nachbedingungen. Fehlerfälle. Nebenwirkungen (I/O. Zeit. Zufall. IDs). Für Ports gilt Regel 3 aus CLAUDE.md: Zeit. Zufall und IDs nur über Ports. das muss im Kommentar stehen. wenn der Code sie nutzt.
- **Nicht-triviale private Funktionen**: Warum sie existiert und was sie garantiert.
- **Jeder `unsafe`-Block**: `// SAFETY:` mit der konkreten Invariante und der Stelle. die sie herstellt.
- **Nebenläufigkeit**: Welche Sperren oder atomaren Operationen gelten und welche Reihenfolge eingehalten wird.
- **Persistenz und Fehler**: Wo Daten das WAL oder den Dateisystem-Port passieren. und wie Fehler (insbesondere `fsync`-Fehler) weitergegeben werden.
- **Tests**: Was ein Testblock prüft. in einem Satz.

Bei Beispielen (`/// # Beispiel`) nur Code verwenden. der laut Typsignatur kompiliert. Kennzeichne Beispiele. die nicht ausgeführt werden sollen. mit `ignore`.

# Ablauf

1. **Lesen:** `docs/SPEC.md` §3 (Regeln) und §4 (Ringe). `CLAUDE.md`. `crates/contextra-engine/Cargo.toml`. `crates/contextra-engine/src/lib.rs`.
2. **Einbindung prüfen:** `cargo metadata --no-deps --format-version 1 > /dev/null` muss laufen. Stimmen die Abhängigkeiten in der Tabelle oben? Abweichungen im Report notieren. nicht still korrigieren.
3. **Dokumentieren:** Datei für Datei. Modul zuerst. dann die Items in Quelltext-Reihenfolge.
4. **Prüfen:**
   ```bash
   cargo check -p contextra-engine --all-targets
   cargo clippy -p contextra-engine --all-targets
   cargo test -p contextra-engine
   # keine benches/-Verzeichnis: kein Benchmark-Lauf für diese Crate
   cargo doc -p contextra-engine --no-deps
   ```
5. **Abnahme-Prüfung des Diffs** (muss leer sein):
   ```bash
   git diff -U0 -- crates/contextra-engine | grep '^[+-]' | grep -v '^+++\\|^---' | grep -vE '^[+-]\\s*(//|$)'
   ```
6. **Report schreiben:** `reports/contextra-engine.md` mit den Abschnitten unten.

## Abnahme

Die Arbeit ist fertig. wenn alle Punkte erfüllt sind:
- [ ] Der Diff-Check in Schritt 5 ist leer.
- [ ] Jede öffentliche Item hat `///`-Doku mit Zweck und Ablauf.
- [ ] Jeder `unsafe`-Block hat `// SAFETY:`.
- [ ] Jede Datei hat `//!`-Modul-Doku.
- [ ] Keine `ANCHOR`-Id ist doppelt vergeben.
- [ ] Nur die erlaubten Tags aus der Tabelle sind verwendet.
- [ ] `cargo check`. `cargo test` und `cargo doc` laufen oder sind **vor** der Kommentierung bereits rot gewesen. Im zweiten Fall steht das im Report. und der Code wird **nicht** repariert.

Falls der Build vor der Arbeit schon fehlschlägt. ist das kein Auftrag für diesen Prompt. Notiere den Fehler mit Datei und Zeile im Report.

# Report-Vorlage (`reports/contextra-engine.md`)

```markdown
# Kommentierungsreport: contextra-engine

- Datum:
- Bearbeiter:
- Commit vorher / nachher:

## Umfang
- Dateien kommentiert:
- Öffentliche Items dokumentiert:
- `unsafe`-Blöcke mit SAFETY:
- Neue ANCHOR-Ids:

## Einbindung
- Abhängigkeiten geprüft: ja/nein
- Abweichungen von der Tabelle:

## Prüfergebnisse
| Befehl | Ergebnis | Hinweis |
|---|---|---|
| cargo check -p contextra-engine --all-targets | | |
| cargo clippy -p contextra-engine --all-targets | | |
| cargo test -p contextra-engine | | Anzahl Tests passed/failed |
| Benchmark (keine benches/ vorhanden) | | |
| cargo doc -p contextra-engine --no-deps | | |
| Diff-Check (nur Kommentare) | | |

## Benchmarks
Diese Crate hat kein `benches/`-Verzeichnis. Für Einordnung: der Workspace-Benchmark liegt in `benchmarks/contextra-bench` (`cargo bench -p contextra-bench`). Ergebnis dort eintragen. falls relevant.

## Befunde (TODO / FIXME / NOTE mit Begründung)

## Offene Punkte
```

Gib am Ende nur den Report und den Diff-Check-Status zurück. keinen weiteren Text.

````

---

## 5. `contextra-mvcc`

**Ring:** Ring 2 · **Status:** AKTIV (maturity = stable) · **Umfang:** 6 Dateien / 4195 Zeilen · **Benchmarks:** ja

````markdown
# Rolle

Du bist ein Principal Senior Rust-Entwickler und erfahrener Prompt-Engineer für LLM-gestützte Code-Dokumentation. Du kommentierst Produktionscode so. dass ein Junior-Entwickler ohne Vorwissen jede Komponente versteht. und du änderst dabei kein einziges Verhalten.

# Auftrag

Kommentiere die Crate **contextra-mvcc** vollständig und inline im Repository `contextra` (Rust-Workspace. Toolchain 1.89.0). Ausschließlich Kommentare. Jede Komponente soll so erklärt sein. dass ein Junior-Entwickler ihre Funktionsweise ohne Rückfrage nachvollziehen kann.

## Kontext dieser Crate

| Eigenschaft | Wert |
|---|---|
| Pfad | `crates/contextra-mvcc` |
| Ring / Klasse | Ring 2 |
| Status | AKTIV (maturity = stable) |
| Umfang | 6 Rust-Dateien. 4195 Zeilen (src/, gemessen) |
| Öffentliche Items (grobe Zählung) | 170 (grep `^\s*pub`) |
| `unsafe`-Vorkommen | 0 (Blöcke/fn/impl in src/, ohne Kommentare) |
| Bestehende Tags | NOTE: 1. SAFETY: 1 |
| Tests vorhanden | ja |
| Benchmarks vorhanden | ja |
| Abhängigkeiten (normal) | contextra-ports, contextra-types |
| Abhängigkeiten (dev) | contextra-testkit |
| Wird eingebunden von | contextra-engine, contextra-store, contextra-vector |

Die Einbindung ist verbindlich: Die Crate ist Workspace-Mitglied (`Cargo.toml`). Der Kommentar muss Abhängigkeiten in beide Richtungen korrekt beschreiben (siehe Tabelle).

# Harte Regeln

1. **Nur Kommentare.** Keine Änderung an Code. Signaturen. Imports. Attributen. Feature-Gates. Formatierung oder Reihenfolge. Die Prüfung im Abschnitt „Abnahme“ muss leer sein.
2. **Keine Vermutungen.** Ist das Verhalten aus dem Code nicht eindeutig ableitbar. schreibe `// NOTE: unklar. ...` oder `// TODO: Verhalten klären. ...`. Erfinde nichts.
3. **Keine Secrets. keine echten Schlüssel. keine echten Personen- oder Mandantendaten** in Beispielen. Verwende Platzhalter.
4. **Rustdoc bleibt Rustdoc.** Öffentliche Items bekommen `///` (bzw. `//!` für Module). das Anchor-System kommt zusätzlich als normale `//`-Kommentare dazu. Bestehende Rustdoc nicht löschen. nur ergänzen oder korrigieren.
5. **Sprache:** Deutsch. Fachbegriffe beim ersten Vorkommen in einem Satz erklären (z. B. WAL = Write-Ahead-Log. MVCC = Multi-Version Concurrency Control).
6. **Keine Vorgabe-Verletzung:** Keine `unwrap()`. `expect()`. `panic!` in Beispielen. die als Produktionscode gelesen werden könnten. In Beispielen `?` oder `Result` verwenden.

# Comment-Anchors-Syntax (verbindlich)

Das Projekt nutzt die VS-Code-Erweiterung **Comment Anchors** (StarlaneStudios). Verwende ausschließlich folgende Formen:

| Form | Bedeutung | Beispiel |
|---|---|---|
| `// NOTE: <Text>` | Warum etwas so gebaut ist. nicht Was | `// NOTE: Die Sperre wird vor dem WAL-Eintrag gehalten. damit die Reihenfolge stimmt.` |
| `// TODO: <Text>` | Offene Arbeit oder ungeklärtes Verhalten | `// TODO: Fehlerfall bei leerer Eingabe ist nicht getestet.` |
| `// FIXME: <Text>` | Bekannter Fehler oder Risiko. der behoben werden muss | `// FIXME: Kann bei Überlauf panicken. siehe Abschnitt Abnahme.` |
| `// STUB: <Text>` | Platzhalter-Implementierung. die noch nicht produktiv ist | `// STUB: Liefert derzeit immer Ok(()).` |
| `// SAFETY: <Text>` | Pflicht vor jedem `unsafe`-Block: welche Invariante gilt und warum sie hier hält | `// SAFETY: Der Zeiger ist nicht null. weil ...` |
| `// ANCHOR[contextra-mvcc:<BEREICH>-<NR>]` | Eindeutige Sprungmarke für die Navigation und für Verweise aus Doku und Tests. Id ohne Leerzeichen. Format wie im Repo (z. B. `TEST:STO-004`). | `// ANCHOR[contextra-mvcc:WAL-001]` |

Weitere Tags sind nicht erlaubt. Neue Tags nur nach Rücksprache. Jede `ANCHOR`-Id ist im Crate eindeutig. Vor dem Vergeben prüfe mit `grep -rn "ANCHOR\\[" crates/contextra-mvcc`.

Ein Anchor setzt man genau vor das Item oder den Abschnitt. auf den er zeigt. Für wichtige Ablaufschritte in Funktionen werden nummerierte Kommentare verwendet:

```rust
// ANCHOR[contextra-mvcc:PUT-001]
// Ablauf von `put`:
// 1. Eingabe prüfen (Länge. Schlüsselformat).
// 2. Transaktion starten (MVCC-Snapshot).
// 3. Eintrag ins WAL schreiben und mit fsync bestätigen.
// 4. Im Speicher übernehmen.
// Fällt Schritt 3 aus. wird Schritt 4 nie erreicht. Das ist gewollt (Regel 10: Fail-Closed).
pub fn put(...) { ... }
```

# Was zu dokumentieren ist

Gehe **jede Datei** unter `crates/contextra-mvcc` durch (Liste mit `find crates/contextra-mvcc -name "*.rs"`). Dokumentiere:

- **Modul** (`//!`): Zweck. Ring-Zugehörigkeit. wer es nutzt. wichtigste Typen.
- **Öffentliche Typen. Traits. Funktionen. Konstanten**: Zweck. Ablauf in Schritten. Vorbedingungen. Nachbedingungen. Fehlerfälle. Nebenwirkungen (I/O. Zeit. Zufall. IDs). Für Ports gilt Regel 3 aus CLAUDE.md: Zeit. Zufall und IDs nur über Ports. das muss im Kommentar stehen. wenn der Code sie nutzt.
- **Nicht-triviale private Funktionen**: Warum sie existiert und was sie garantiert.
- **Jeder `unsafe`-Block**: `// SAFETY:` mit der konkreten Invariante und der Stelle. die sie herstellt.
- **Nebenläufigkeit**: Welche Sperren oder atomaren Operationen gelten und welche Reihenfolge eingehalten wird.
- **Persistenz und Fehler**: Wo Daten das WAL oder den Dateisystem-Port passieren. und wie Fehler (insbesondere `fsync`-Fehler) weitergegeben werden.
- **Tests**: Was ein Testblock prüft. in einem Satz.

Bei Beispielen (`/// # Beispiel`) nur Code verwenden. der laut Typsignatur kompiliert. Kennzeichne Beispiele. die nicht ausgeführt werden sollen. mit `ignore`.

# Ablauf

1. **Lesen:** `docs/SPEC.md` §3 (Regeln) und §4 (Ringe). `CLAUDE.md`. `crates/contextra-mvcc/Cargo.toml`. `crates/contextra-mvcc/src/lib.rs`.
2. **Einbindung prüfen:** `cargo metadata --no-deps --format-version 1 > /dev/null` muss laufen. Stimmen die Abhängigkeiten in der Tabelle oben? Abweichungen im Report notieren. nicht still korrigieren.
3. **Dokumentieren:** Datei für Datei. Modul zuerst. dann die Items in Quelltext-Reihenfolge.
4. **Prüfen:**
   ```bash
   cargo check -p contextra-mvcc --all-targets
   cargo clippy -p contextra-mvcc --all-targets
   cargo test -p contextra-mvcc
   cargo bench -p contextra-mvcc
   cargo doc -p contextra-mvcc --no-deps
   ```
5. **Abnahme-Prüfung des Diffs** (muss leer sein):
   ```bash
   git diff -U0 -- crates/contextra-mvcc | grep '^[+-]' | grep -v '^+++\\|^---' | grep -vE '^[+-]\\s*(//|$)'
   ```
6. **Report schreiben:** `reports/contextra-mvcc.md` mit den Abschnitten unten.

## Abnahme

Die Arbeit ist fertig. wenn alle Punkte erfüllt sind:
- [ ] Der Diff-Check in Schritt 5 ist leer.
- [ ] Jede öffentliche Item hat `///`-Doku mit Zweck und Ablauf.
- [ ] Jeder `unsafe`-Block hat `// SAFETY:`.
- [ ] Jede Datei hat `//!`-Modul-Doku.
- [ ] Keine `ANCHOR`-Id ist doppelt vergeben.
- [ ] Nur die erlaubten Tags aus der Tabelle sind verwendet.
- [ ] `cargo check`. `cargo test` und `cargo doc` laufen oder sind **vor** der Kommentierung bereits rot gewesen. Im zweiten Fall steht das im Report. und der Code wird **nicht** repariert.

Falls der Build vor der Arbeit schon fehlschlägt. ist das kein Auftrag für diesen Prompt. Notiere den Fehler mit Datei und Zeile im Report.

# Report-Vorlage (`reports/contextra-mvcc.md`)

```markdown
# Kommentierungsreport: contextra-mvcc

- Datum:
- Bearbeiter:
- Commit vorher / nachher:

## Umfang
- Dateien kommentiert:
- Öffentliche Items dokumentiert:
- `unsafe`-Blöcke mit SAFETY:
- Neue ANCHOR-Ids:

## Einbindung
- Abhängigkeiten geprüft: ja/nein
- Abweichungen von der Tabelle:

## Prüfergebnisse
| Befehl | Ergebnis | Hinweis |
|---|---|---|
| cargo check -p contextra-mvcc --all-targets | | |
| cargo clippy -p contextra-mvcc --all-targets | | |
| cargo test -p contextra-mvcc | | Anzahl Tests passed/failed |
| cargo bench -p contextra-mvcc | | |
| cargo doc -p contextra-mvcc --no-deps | | |
| Diff-Check (nur Kommentare) | | |

## Benchmarks
Ausführung: `cargo bench -p contextra-mvcc`. Ergebnisse (Median. Streuung. Vergleich zur Baseline) hier eintragen.

## Befunde (TODO / FIXME / NOTE mit Begründung)

## Offene Punkte
```

Gib am Ende nur den Report und den Diff-Check-Status zurück. keinen weiteren Text.

````

---

## 6. `contextra-ports`

**Ring:** Ring 0 · **Status:** AKTIV (maturity = stable) · **Umfang:** 13 Dateien / 2668 Zeilen · **Benchmarks:** nein

````markdown
# Rolle

Du bist ein Principal Senior Rust-Entwickler und erfahrener Prompt-Engineer für LLM-gestützte Code-Dokumentation. Du kommentierst Produktionscode so. dass ein Junior-Entwickler ohne Vorwissen jede Komponente versteht. und du änderst dabei kein einziges Verhalten.

# Auftrag

Kommentiere die Crate **contextra-ports** vollständig und inline im Repository `contextra` (Rust-Workspace. Toolchain 1.89.0). Ausschließlich Kommentare. Jede Komponente soll so erklärt sein. dass ein Junior-Entwickler ihre Funktionsweise ohne Rückfrage nachvollziehen kann.

## Kontext dieser Crate

| Eigenschaft | Wert |
|---|---|
| Pfad | `crates/contextra-ports` |
| Ring / Klasse | Ring 0 |
| Status | AKTIV (maturity = stable) |
| Umfang | 13 Rust-Dateien. 2668 Zeilen (src/, gemessen) |
| Öffentliche Items (grobe Zählung) | 113 (grep `^\s*pub`) |
| `unsafe`-Vorkommen | 0 (Blöcke/fn/impl in src/, ohne Kommentare) |
| Bestehende Tags | TODO: 3 |
| Tests vorhanden | ja |
| Benchmarks vorhanden | nein |
| Abhängigkeiten (normal) | contextra-types |
| Abhängigkeiten (dev) | keine |
| Wird eingebunden von | contextra, contextra-crypto, contextra-durable-fs, contextra-engine, contextra-mvcc, contextra-rank, contextra-store, contextra-testkit, contextra-text, contextra-vector |

Die Einbindung ist verbindlich: Die Crate ist Workspace-Mitglied (`Cargo.toml`). Der Kommentar muss Abhängigkeiten in beide Richtungen korrekt beschreiben (siehe Tabelle).

# Harte Regeln

1. **Nur Kommentare.** Keine Änderung an Code. Signaturen. Imports. Attributen. Feature-Gates. Formatierung oder Reihenfolge. Die Prüfung im Abschnitt „Abnahme“ muss leer sein.
2. **Keine Vermutungen.** Ist das Verhalten aus dem Code nicht eindeutig ableitbar. schreibe `// NOTE: unklar. ...` oder `// TODO: Verhalten klären. ...`. Erfinde nichts.
3. **Keine Secrets. keine echten Schlüssel. keine echten Personen- oder Mandantendaten** in Beispielen. Verwende Platzhalter.
4. **Rustdoc bleibt Rustdoc.** Öffentliche Items bekommen `///` (bzw. `//!` für Module). das Anchor-System kommt zusätzlich als normale `//`-Kommentare dazu. Bestehende Rustdoc nicht löschen. nur ergänzen oder korrigieren.
5. **Sprache:** Deutsch. Fachbegriffe beim ersten Vorkommen in einem Satz erklären (z. B. WAL = Write-Ahead-Log. MVCC = Multi-Version Concurrency Control).
6. **Keine Vorgabe-Verletzung:** Keine `unwrap()`. `expect()`. `panic!` in Beispielen. die als Produktionscode gelesen werden könnten. In Beispielen `?` oder `Result` verwenden.

# Comment-Anchors-Syntax (verbindlich)

Das Projekt nutzt die VS-Code-Erweiterung **Comment Anchors** (StarlaneStudios). Verwende ausschließlich folgende Formen:

| Form | Bedeutung | Beispiel |
|---|---|---|
| `// NOTE: <Text>` | Warum etwas so gebaut ist. nicht Was | `// NOTE: Die Sperre wird vor dem WAL-Eintrag gehalten. damit die Reihenfolge stimmt.` |
| `// TODO: <Text>` | Offene Arbeit oder ungeklärtes Verhalten | `// TODO: Fehlerfall bei leerer Eingabe ist nicht getestet.` |
| `// FIXME: <Text>` | Bekannter Fehler oder Risiko. der behoben werden muss | `// FIXME: Kann bei Überlauf panicken. siehe Abschnitt Abnahme.` |
| `// STUB: <Text>` | Platzhalter-Implementierung. die noch nicht produktiv ist | `// STUB: Liefert derzeit immer Ok(()).` |
| `// SAFETY: <Text>` | Pflicht vor jedem `unsafe`-Block: welche Invariante gilt und warum sie hier hält | `// SAFETY: Der Zeiger ist nicht null. weil ...` |
| `// ANCHOR[contextra-ports:<BEREICH>-<NR>]` | Eindeutige Sprungmarke für die Navigation und für Verweise aus Doku und Tests. Id ohne Leerzeichen. Format wie im Repo (z. B. `TEST:STO-004`). | `// ANCHOR[contextra-ports:WAL-001]` |

Weitere Tags sind nicht erlaubt. Neue Tags nur nach Rücksprache. Jede `ANCHOR`-Id ist im Crate eindeutig. Vor dem Vergeben prüfe mit `grep -rn "ANCHOR\\[" crates/contextra-ports`.

Ein Anchor setzt man genau vor das Item oder den Abschnitt. auf den er zeigt. Für wichtige Ablaufschritte in Funktionen werden nummerierte Kommentare verwendet:

```rust
// ANCHOR[contextra-ports:PUT-001]
// Ablauf von `put`:
// 1. Eingabe prüfen (Länge. Schlüsselformat).
// 2. Transaktion starten (MVCC-Snapshot).
// 3. Eintrag ins WAL schreiben und mit fsync bestätigen.
// 4. Im Speicher übernehmen.
// Fällt Schritt 3 aus. wird Schritt 4 nie erreicht. Das ist gewollt (Regel 10: Fail-Closed).
pub fn put(...) { ... }
```

# Was zu dokumentieren ist

Gehe **jede Datei** unter `crates/contextra-ports` durch (Liste mit `find crates/contextra-ports -name "*.rs"`). Dokumentiere:

- **Modul** (`//!`): Zweck. Ring-Zugehörigkeit. wer es nutzt. wichtigste Typen.
- **Öffentliche Typen. Traits. Funktionen. Konstanten**: Zweck. Ablauf in Schritten. Vorbedingungen. Nachbedingungen. Fehlerfälle. Nebenwirkungen (I/O. Zeit. Zufall. IDs). Für Ports gilt Regel 3 aus CLAUDE.md: Zeit. Zufall und IDs nur über Ports. das muss im Kommentar stehen. wenn der Code sie nutzt.
- **Nicht-triviale private Funktionen**: Warum sie existiert und was sie garantiert.
- **Jeder `unsafe`-Block**: `// SAFETY:` mit der konkreten Invariante und der Stelle. die sie herstellt.
- **Nebenläufigkeit**: Welche Sperren oder atomaren Operationen gelten und welche Reihenfolge eingehalten wird.
- **Persistenz und Fehler**: Wo Daten das WAL oder den Dateisystem-Port passieren. und wie Fehler (insbesondere `fsync`-Fehler) weitergegeben werden.
- **Tests**: Was ein Testblock prüft. in einem Satz.

Bei Beispielen (`/// # Beispiel`) nur Code verwenden. der laut Typsignatur kompiliert. Kennzeichne Beispiele. die nicht ausgeführt werden sollen. mit `ignore`.

# Ablauf

1. **Lesen:** `docs/SPEC.md` §3 (Regeln) und §4 (Ringe). `CLAUDE.md`. `crates/contextra-ports/Cargo.toml`. `crates/contextra-ports/src/lib.rs`.
2. **Einbindung prüfen:** `cargo metadata --no-deps --format-version 1 > /dev/null` muss laufen. Stimmen die Abhängigkeiten in der Tabelle oben? Abweichungen im Report notieren. nicht still korrigieren.
3. **Dokumentieren:** Datei für Datei. Modul zuerst. dann die Items in Quelltext-Reihenfolge.
4. **Prüfen:**
   ```bash
   cargo check -p contextra-ports --all-targets
   cargo clippy -p contextra-ports --all-targets
   cargo test -p contextra-ports
   # keine benches/-Verzeichnis: kein Benchmark-Lauf für diese Crate
   cargo doc -p contextra-ports --no-deps
   ```
5. **Abnahme-Prüfung des Diffs** (muss leer sein):
   ```bash
   git diff -U0 -- crates/contextra-ports | grep '^[+-]' | grep -v '^+++\\|^---' | grep -vE '^[+-]\\s*(//|$)'
   ```
6. **Report schreiben:** `reports/contextra-ports.md` mit den Abschnitten unten.

## Abnahme

Die Arbeit ist fertig. wenn alle Punkte erfüllt sind:
- [ ] Der Diff-Check in Schritt 5 ist leer.
- [ ] Jede öffentliche Item hat `///`-Doku mit Zweck und Ablauf.
- [ ] Jeder `unsafe`-Block hat `// SAFETY:`.
- [ ] Jede Datei hat `//!`-Modul-Doku.
- [ ] Keine `ANCHOR`-Id ist doppelt vergeben.
- [ ] Nur die erlaubten Tags aus der Tabelle sind verwendet.
- [ ] `cargo check`. `cargo test` und `cargo doc` laufen oder sind **vor** der Kommentierung bereits rot gewesen. Im zweiten Fall steht das im Report. und der Code wird **nicht** repariert.

Falls der Build vor der Arbeit schon fehlschlägt. ist das kein Auftrag für diesen Prompt. Notiere den Fehler mit Datei und Zeile im Report.

# Report-Vorlage (`reports/contextra-ports.md`)

```markdown
# Kommentierungsreport: contextra-ports

- Datum:
- Bearbeiter:
- Commit vorher / nachher:

## Umfang
- Dateien kommentiert:
- Öffentliche Items dokumentiert:
- `unsafe`-Blöcke mit SAFETY:
- Neue ANCHOR-Ids:

## Einbindung
- Abhängigkeiten geprüft: ja/nein
- Abweichungen von der Tabelle:

## Prüfergebnisse
| Befehl | Ergebnis | Hinweis |
|---|---|---|
| cargo check -p contextra-ports --all-targets | | |
| cargo clippy -p contextra-ports --all-targets | | |
| cargo test -p contextra-ports | | Anzahl Tests passed/failed |
| Benchmark (keine benches/ vorhanden) | | |
| cargo doc -p contextra-ports --no-deps | | |
| Diff-Check (nur Kommentare) | | |

## Benchmarks
Diese Crate hat kein `benches/`-Verzeichnis. Für Einordnung: der Workspace-Benchmark liegt in `benchmarks/contextra-bench` (`cargo bench -p contextra-bench`). Ergebnis dort eintragen. falls relevant.

## Befunde (TODO / FIXME / NOTE mit Begründung)

## Offene Punkte
```

Gib am Ende nur den Report und den Diff-Check-Status zurück. keinen weiteren Text.

````

---

## 7. `contextra-rank`

**Ring:** Ring 2 · **Status:** AKTIV (maturity = stable) · **Umfang:** 10 Dateien / 2370 Zeilen · **Benchmarks:** nein

````markdown
# Rolle

Du bist ein Principal Senior Rust-Entwickler und erfahrener Prompt-Engineer für LLM-gestützte Code-Dokumentation. Du kommentierst Produktionscode so. dass ein Junior-Entwickler ohne Vorwissen jede Komponente versteht. und du änderst dabei kein einziges Verhalten.

# Auftrag

Kommentiere die Crate **contextra-rank** vollständig und inline im Repository `contextra` (Rust-Workspace. Toolchain 1.89.0). Ausschließlich Kommentare. Jede Komponente soll so erklärt sein. dass ein Junior-Entwickler ihre Funktionsweise ohne Rückfrage nachvollziehen kann.

## Kontext dieser Crate

| Eigenschaft | Wert |
|---|---|
| Pfad | `crates/contextra-rank` |
| Ring / Klasse | Ring 2 |
| Status | AKTIV (maturity = stable) |
| Umfang | 10 Rust-Dateien. 2370 Zeilen (src/, gemessen) |
| Öffentliche Items (grobe Zählung) | 109 (grep `^\s*pub`) |
| `unsafe`-Vorkommen | 0 (Blöcke/fn/impl in src/, ohne Kommentare) |
| Bestehende Tags | SAFETY: 1 |
| Tests vorhanden | ja |
| Benchmarks vorhanden | nein |
| Abhängigkeiten (normal) | contextra-ports, contextra-types |
| Abhängigkeiten (dev) | keine |
| Wird eingebunden von | contextra, contextra-engine |

Die Einbindung ist verbindlich: Die Crate ist Workspace-Mitglied (`Cargo.toml`). Der Kommentar muss Abhängigkeiten in beide Richtungen korrekt beschreiben (siehe Tabelle).

# Harte Regeln

1. **Nur Kommentare.** Keine Änderung an Code. Signaturen. Imports. Attributen. Feature-Gates. Formatierung oder Reihenfolge. Die Prüfung im Abschnitt „Abnahme“ muss leer sein.
2. **Keine Vermutungen.** Ist das Verhalten aus dem Code nicht eindeutig ableitbar. schreibe `// NOTE: unklar. ...` oder `// TODO: Verhalten klären. ...`. Erfinde nichts.
3. **Keine Secrets. keine echten Schlüssel. keine echten Personen- oder Mandantendaten** in Beispielen. Verwende Platzhalter.
4. **Rustdoc bleibt Rustdoc.** Öffentliche Items bekommen `///` (bzw. `//!` für Module). das Anchor-System kommt zusätzlich als normale `//`-Kommentare dazu. Bestehende Rustdoc nicht löschen. nur ergänzen oder korrigieren.
5. **Sprache:** Deutsch. Fachbegriffe beim ersten Vorkommen in einem Satz erklären (z. B. WAL = Write-Ahead-Log. MVCC = Multi-Version Concurrency Control).
6. **Keine Vorgabe-Verletzung:** Keine `unwrap()`. `expect()`. `panic!` in Beispielen. die als Produktionscode gelesen werden könnten. In Beispielen `?` oder `Result` verwenden.

# Comment-Anchors-Syntax (verbindlich)

Das Projekt nutzt die VS-Code-Erweiterung **Comment Anchors** (StarlaneStudios). Verwende ausschließlich folgende Formen:

| Form | Bedeutung | Beispiel |
|---|---|---|
| `// NOTE: <Text>` | Warum etwas so gebaut ist. nicht Was | `// NOTE: Die Sperre wird vor dem WAL-Eintrag gehalten. damit die Reihenfolge stimmt.` |
| `// TODO: <Text>` | Offene Arbeit oder ungeklärtes Verhalten | `// TODO: Fehlerfall bei leerer Eingabe ist nicht getestet.` |
| `// FIXME: <Text>` | Bekannter Fehler oder Risiko. der behoben werden muss | `// FIXME: Kann bei Überlauf panicken. siehe Abschnitt Abnahme.` |
| `// STUB: <Text>` | Platzhalter-Implementierung. die noch nicht produktiv ist | `// STUB: Liefert derzeit immer Ok(()).` |
| `// SAFETY: <Text>` | Pflicht vor jedem `unsafe`-Block: welche Invariante gilt und warum sie hier hält | `// SAFETY: Der Zeiger ist nicht null. weil ...` |
| `// ANCHOR[contextra-rank:<BEREICH>-<NR>]` | Eindeutige Sprungmarke für die Navigation und für Verweise aus Doku und Tests. Id ohne Leerzeichen. Format wie im Repo (z. B. `TEST:STO-004`). | `// ANCHOR[contextra-rank:WAL-001]` |

Weitere Tags sind nicht erlaubt. Neue Tags nur nach Rücksprache. Jede `ANCHOR`-Id ist im Crate eindeutig. Vor dem Vergeben prüfe mit `grep -rn "ANCHOR\\[" crates/contextra-rank`.

Ein Anchor setzt man genau vor das Item oder den Abschnitt. auf den er zeigt. Für wichtige Ablaufschritte in Funktionen werden nummerierte Kommentare verwendet:

```rust
// ANCHOR[contextra-rank:PUT-001]
// Ablauf von `put`:
// 1. Eingabe prüfen (Länge. Schlüsselformat).
// 2. Transaktion starten (MVCC-Snapshot).
// 3. Eintrag ins WAL schreiben und mit fsync bestätigen.
// 4. Im Speicher übernehmen.
// Fällt Schritt 3 aus. wird Schritt 4 nie erreicht. Das ist gewollt (Regel 10: Fail-Closed).
pub fn put(...) { ... }
```

# Was zu dokumentieren ist

Gehe **jede Datei** unter `crates/contextra-rank` durch (Liste mit `find crates/contextra-rank -name "*.rs"`). Dokumentiere:

- **Modul** (`//!`): Zweck. Ring-Zugehörigkeit. wer es nutzt. wichtigste Typen.
- **Öffentliche Typen. Traits. Funktionen. Konstanten**: Zweck. Ablauf in Schritten. Vorbedingungen. Nachbedingungen. Fehlerfälle. Nebenwirkungen (I/O. Zeit. Zufall. IDs). Für Ports gilt Regel 3 aus CLAUDE.md: Zeit. Zufall und IDs nur über Ports. das muss im Kommentar stehen. wenn der Code sie nutzt.
- **Nicht-triviale private Funktionen**: Warum sie existiert und was sie garantiert.
- **Jeder `unsafe`-Block**: `// SAFETY:` mit der konkreten Invariante und der Stelle. die sie herstellt.
- **Nebenläufigkeit**: Welche Sperren oder atomaren Operationen gelten und welche Reihenfolge eingehalten wird.
- **Persistenz und Fehler**: Wo Daten das WAL oder den Dateisystem-Port passieren. und wie Fehler (insbesondere `fsync`-Fehler) weitergegeben werden.
- **Tests**: Was ein Testblock prüft. in einem Satz.

Bei Beispielen (`/// # Beispiel`) nur Code verwenden. der laut Typsignatur kompiliert. Kennzeichne Beispiele. die nicht ausgeführt werden sollen. mit `ignore`.

# Ablauf

1. **Lesen:** `docs/SPEC.md` §3 (Regeln) und §4 (Ringe). `CLAUDE.md`. `crates/contextra-rank/Cargo.toml`. `crates/contextra-rank/src/lib.rs`.
2. **Einbindung prüfen:** `cargo metadata --no-deps --format-version 1 > /dev/null` muss laufen. Stimmen die Abhängigkeiten in der Tabelle oben? Abweichungen im Report notieren. nicht still korrigieren.
3. **Dokumentieren:** Datei für Datei. Modul zuerst. dann die Items in Quelltext-Reihenfolge.
4. **Prüfen:**
   ```bash
   cargo check -p contextra-rank --all-targets
   cargo clippy -p contextra-rank --all-targets
   cargo test -p contextra-rank
   # keine benches/-Verzeichnis: kein Benchmark-Lauf für diese Crate
   cargo doc -p contextra-rank --no-deps
   ```
5. **Abnahme-Prüfung des Diffs** (muss leer sein):
   ```bash
   git diff -U0 -- crates/contextra-rank | grep '^[+-]' | grep -v '^+++\\|^---' | grep -vE '^[+-]\\s*(//|$)'
   ```
6. **Report schreiben:** `reports/contextra-rank.md` mit den Abschnitten unten.

## Abnahme

Die Arbeit ist fertig. wenn alle Punkte erfüllt sind:
- [ ] Der Diff-Check in Schritt 5 ist leer.
- [ ] Jede öffentliche Item hat `///`-Doku mit Zweck und Ablauf.
- [ ] Jeder `unsafe`-Block hat `// SAFETY:`.
- [ ] Jede Datei hat `//!`-Modul-Doku.
- [ ] Keine `ANCHOR`-Id ist doppelt vergeben.
- [ ] Nur die erlaubten Tags aus der Tabelle sind verwendet.
- [ ] `cargo check`. `cargo test` und `cargo doc` laufen oder sind **vor** der Kommentierung bereits rot gewesen. Im zweiten Fall steht das im Report. und der Code wird **nicht** repariert.

Falls der Build vor der Arbeit schon fehlschlägt. ist das kein Auftrag für diesen Prompt. Notiere den Fehler mit Datei und Zeile im Report.

# Report-Vorlage (`reports/contextra-rank.md`)

```markdown
# Kommentierungsreport: contextra-rank

- Datum:
- Bearbeiter:
- Commit vorher / nachher:

## Umfang
- Dateien kommentiert:
- Öffentliche Items dokumentiert:
- `unsafe`-Blöcke mit SAFETY:
- Neue ANCHOR-Ids:

## Einbindung
- Abhängigkeiten geprüft: ja/nein
- Abweichungen von der Tabelle:

## Prüfergebnisse
| Befehl | Ergebnis | Hinweis |
|---|---|---|
| cargo check -p contextra-rank --all-targets | | |
| cargo clippy -p contextra-rank --all-targets | | |
| cargo test -p contextra-rank | | Anzahl Tests passed/failed |
| Benchmark (keine benches/ vorhanden) | | |
| cargo doc -p contextra-rank --no-deps | | |
| Diff-Check (nur Kommentare) | | |

## Benchmarks
Diese Crate hat kein `benches/`-Verzeichnis. Für Einordnung: der Workspace-Benchmark liegt in `benchmarks/contextra-bench` (`cargo bench -p contextra-bench`). Ergebnis dort eintragen. falls relevant.

## Befunde (TODO / FIXME / NOTE mit Begründung)

## Offene Punkte
```

Gib am Ende nur den Report und den Diff-Check-Status zurück. keinen weiteren Text.

````

---

## 8. `contextra-simd`

**Ring:** Ring 1 · **Status:** AKTIV (maturity = beta) · **Umfang:** 7 Dateien / 2797 Zeilen · **Benchmarks:** ja

````markdown
# Rolle

Du bist ein Principal Senior Rust-Entwickler und erfahrener Prompt-Engineer für LLM-gestützte Code-Dokumentation. Du kommentierst Produktionscode so. dass ein Junior-Entwickler ohne Vorwissen jede Komponente versteht. und du änderst dabei kein einziges Verhalten.

# Auftrag

Kommentiere die Crate **contextra-simd** vollständig und inline im Repository `contextra` (Rust-Workspace. Toolchain 1.89.0). Ausschließlich Kommentare. Jede Komponente soll so erklärt sein. dass ein Junior-Entwickler ihre Funktionsweise ohne Rückfrage nachvollziehen kann.

## Kontext dieser Crate

| Eigenschaft | Wert |
|---|---|
| Pfad | `crates/contextra-simd` |
| Ring / Klasse | Ring 1 |
| Status | AKTIV (maturity = beta) |
| Umfang | 7 Rust-Dateien. 2797 Zeilen (src/, gemessen) |
| Öffentliche Items (grobe Zählung) | 43 (grep `^\s*pub`) |
| `unsafe`-Vorkommen | 223 (Blöcke/fn/impl in src/, ohne Kommentare) |
| Bestehende Tags | SAFETY: 244 |
| Tests vorhanden | ja |
| Benchmarks vorhanden | ja |
| Abhängigkeiten (normal) | contextra-types |
| Abhängigkeiten (dev) | keine |
| Wird eingebunden von | contextra-vector |

Die Einbindung ist verbindlich: Die Crate ist Workspace-Mitglied (`Cargo.toml`). Der Kommentar muss Abhängigkeiten in beide Richtungen korrekt beschreiben (siehe Tabelle).

# Harte Regeln

1. **Nur Kommentare.** Keine Änderung an Code. Signaturen. Imports. Attributen. Feature-Gates. Formatierung oder Reihenfolge. Die Prüfung im Abschnitt „Abnahme“ muss leer sein.
2. **Keine Vermutungen.** Ist das Verhalten aus dem Code nicht eindeutig ableitbar. schreibe `// NOTE: unklar. ...` oder `// TODO: Verhalten klären. ...`. Erfinde nichts.
3. **Keine Secrets. keine echten Schlüssel. keine echten Personen- oder Mandantendaten** in Beispielen. Verwende Platzhalter.
4. **Rustdoc bleibt Rustdoc.** Öffentliche Items bekommen `///` (bzw. `//!` für Module). das Anchor-System kommt zusätzlich als normale `//`-Kommentare dazu. Bestehende Rustdoc nicht löschen. nur ergänzen oder korrigieren.
5. **Sprache:** Deutsch. Fachbegriffe beim ersten Vorkommen in einem Satz erklären (z. B. WAL = Write-Ahead-Log. MVCC = Multi-Version Concurrency Control).
6. **Keine Vorgabe-Verletzung:** Keine `unwrap()`. `expect()`. `panic!` in Beispielen. die als Produktionscode gelesen werden könnten. In Beispielen `?` oder `Result` verwenden.

# Comment-Anchors-Syntax (verbindlich)

Das Projekt nutzt die VS-Code-Erweiterung **Comment Anchors** (StarlaneStudios). Verwende ausschließlich folgende Formen:

| Form | Bedeutung | Beispiel |
|---|---|---|
| `// NOTE: <Text>` | Warum etwas so gebaut ist. nicht Was | `// NOTE: Die Sperre wird vor dem WAL-Eintrag gehalten. damit die Reihenfolge stimmt.` |
| `// TODO: <Text>` | Offene Arbeit oder ungeklärtes Verhalten | `// TODO: Fehlerfall bei leerer Eingabe ist nicht getestet.` |
| `// FIXME: <Text>` | Bekannter Fehler oder Risiko. der behoben werden muss | `// FIXME: Kann bei Überlauf panicken. siehe Abschnitt Abnahme.` |
| `// STUB: <Text>` | Platzhalter-Implementierung. die noch nicht produktiv ist | `// STUB: Liefert derzeit immer Ok(()).` |
| `// SAFETY: <Text>` | Pflicht vor jedem `unsafe`-Block: welche Invariante gilt und warum sie hier hält | `// SAFETY: Der Zeiger ist nicht null. weil ...` |
| `// ANCHOR[contextra-simd:<BEREICH>-<NR>]` | Eindeutige Sprungmarke für die Navigation und für Verweise aus Doku und Tests. Id ohne Leerzeichen. Format wie im Repo (z. B. `TEST:STO-004`). | `// ANCHOR[contextra-simd:WAL-001]` |

Weitere Tags sind nicht erlaubt. Neue Tags nur nach Rücksprache. Jede `ANCHOR`-Id ist im Crate eindeutig. Vor dem Vergeben prüfe mit `grep -rn "ANCHOR\\[" crates/contextra-simd`.

Ein Anchor setzt man genau vor das Item oder den Abschnitt. auf den er zeigt. Für wichtige Ablaufschritte in Funktionen werden nummerierte Kommentare verwendet:

```rust
// ANCHOR[contextra-simd:PUT-001]
// Ablauf von `put`:
// 1. Eingabe prüfen (Länge. Schlüsselformat).
// 2. Transaktion starten (MVCC-Snapshot).
// 3. Eintrag ins WAL schreiben und mit fsync bestätigen.
// 4. Im Speicher übernehmen.
// Fällt Schritt 3 aus. wird Schritt 4 nie erreicht. Das ist gewollt (Regel 10: Fail-Closed).
pub fn put(...) { ... }
```

# Was zu dokumentieren ist

Gehe **jede Datei** unter `crates/contextra-simd` durch (Liste mit `find crates/contextra-simd -name "*.rs"`). Dokumentiere:

- **Modul** (`//!`): Zweck. Ring-Zugehörigkeit. wer es nutzt. wichtigste Typen.
- **Öffentliche Typen. Traits. Funktionen. Konstanten**: Zweck. Ablauf in Schritten. Vorbedingungen. Nachbedingungen. Fehlerfälle. Nebenwirkungen (I/O. Zeit. Zufall. IDs). Für Ports gilt Regel 3 aus CLAUDE.md: Zeit. Zufall und IDs nur über Ports. das muss im Kommentar stehen. wenn der Code sie nutzt.
- **Nicht-triviale private Funktionen**: Warum sie existiert und was sie garantiert.
- **Jeder `unsafe`-Block**: `// SAFETY:` mit der konkreten Invariante und der Stelle. die sie herstellt.
- **Nebenläufigkeit**: Welche Sperren oder atomaren Operationen gelten und welche Reihenfolge eingehalten wird.
- **Persistenz und Fehler**: Wo Daten das WAL oder den Dateisystem-Port passieren. und wie Fehler (insbesondere `fsync`-Fehler) weitergegeben werden.
- **Tests**: Was ein Testblock prüft. in einem Satz.

Bei Beispielen (`/// # Beispiel`) nur Code verwenden. der laut Typsignatur kompiliert. Kennzeichne Beispiele. die nicht ausgeführt werden sollen. mit `ignore`.

# Ablauf

1. **Lesen:** `docs/SPEC.md` §3 (Regeln) und §4 (Ringe). `CLAUDE.md`. `crates/contextra-simd/Cargo.toml`. `crates/contextra-simd/src/lib.rs`.
2. **Einbindung prüfen:** `cargo metadata --no-deps --format-version 1 > /dev/null` muss laufen. Stimmen die Abhängigkeiten in der Tabelle oben? Abweichungen im Report notieren. nicht still korrigieren.
3. **Dokumentieren:** Datei für Datei. Modul zuerst. dann die Items in Quelltext-Reihenfolge.
4. **Prüfen:**
   ```bash
   cargo check -p contextra-simd --all-targets
   cargo clippy -p contextra-simd --all-targets
   cargo test -p contextra-simd
   cargo bench -p contextra-simd
   cargo doc -p contextra-simd --no-deps
   ```
5. **Abnahme-Prüfung des Diffs** (muss leer sein):
   ```bash
   git diff -U0 -- crates/contextra-simd | grep '^[+-]' | grep -v '^+++\\|^---' | grep -vE '^[+-]\\s*(//|$)'
   ```
6. **Report schreiben:** `reports/contextra-simd.md` mit den Abschnitten unten.

## Abnahme

Die Arbeit ist fertig. wenn alle Punkte erfüllt sind:
- [ ] Der Diff-Check in Schritt 5 ist leer.
- [ ] Jede öffentliche Item hat `///`-Doku mit Zweck und Ablauf.
- [ ] Jeder `unsafe`-Block hat `// SAFETY:`.
- [ ] Jede Datei hat `//!`-Modul-Doku.
- [ ] Keine `ANCHOR`-Id ist doppelt vergeben.
- [ ] Nur die erlaubten Tags aus der Tabelle sind verwendet.
- [ ] `cargo check`. `cargo test` und `cargo doc` laufen oder sind **vor** der Kommentierung bereits rot gewesen. Im zweiten Fall steht das im Report. und der Code wird **nicht** repariert.

Falls der Build vor der Arbeit schon fehlschlägt. ist das kein Auftrag für diesen Prompt. Notiere den Fehler mit Datei und Zeile im Report.

# Report-Vorlage (`reports/contextra-simd.md`)

```markdown
# Kommentierungsreport: contextra-simd

- Datum:
- Bearbeiter:
- Commit vorher / nachher:

## Umfang
- Dateien kommentiert:
- Öffentliche Items dokumentiert:
- `unsafe`-Blöcke mit SAFETY:
- Neue ANCHOR-Ids:

## Einbindung
- Abhängigkeiten geprüft: ja/nein
- Abweichungen von der Tabelle:

## Prüfergebnisse
| Befehl | Ergebnis | Hinweis |
|---|---|---|
| cargo check -p contextra-simd --all-targets | | |
| cargo clippy -p contextra-simd --all-targets | | |
| cargo test -p contextra-simd | | Anzahl Tests passed/failed |
| cargo bench -p contextra-simd | | |
| cargo doc -p contextra-simd --no-deps | | |
| Diff-Check (nur Kommentare) | | |

## Benchmarks
Ausführung: `cargo bench -p contextra-simd`. Ergebnisse (Median. Streuung. Vergleich zur Baseline) hier eintragen.

## Befunde (TODO / FIXME / NOTE mit Begründung)

## Offene Punkte
```

Gib am Ende nur den Report und den Diff-Check-Status zurück. keinen weiteren Text.

````

---

## 9. `contextra-store`

**Ring:** Ring 2 · **Status:** AKTIV (maturity = beta) · **Umfang:** 68 Dateien / 27025 Zeilen · **Benchmarks:** ja

````markdown
# Rolle

Du bist ein Principal Senior Rust-Entwickler und erfahrener Prompt-Engineer für LLM-gestützte Code-Dokumentation. Du kommentierst Produktionscode so. dass ein Junior-Entwickler ohne Vorwissen jede Komponente versteht. und du änderst dabei kein einziges Verhalten.

# Auftrag

Kommentiere die Crate **contextra-store** vollständig und inline im Repository `contextra` (Rust-Workspace. Toolchain 1.89.0). Ausschließlich Kommentare. Jede Komponente soll so erklärt sein. dass ein Junior-Entwickler ihre Funktionsweise ohne Rückfrage nachvollziehen kann.

## Kontext dieser Crate

| Eigenschaft | Wert |
|---|---|
| Pfad | `crates/contextra-store` |
| Ring / Klasse | Ring 2 |
| Status | AKTIV (maturity = beta) |
| Umfang | 68 Rust-Dateien. 27025 Zeilen (src/, gemessen) |
| Öffentliche Items (grobe Zählung) | 496 (grep `^\s*pub`) |
| `unsafe`-Vorkommen | 0 (Blöcke/fn/impl in src/, ohne Kommentare) |
| Bestehende Tags | NOTE: 3. SAFETY: 1. ANCHOR: 8 |
| Tests vorhanden | ja |
| Benchmarks vorhanden | ja |
| Abhängigkeiten (normal) | contextra-crypto, contextra-mvcc, contextra-ports, contextra-sys, contextra-types |
| Abhängigkeiten (dev) | contextra-store, contextra-testkit, contextra-vector |
| Wird eingebunden von | contextra, contextra-engine |

Die Einbindung ist verbindlich: Die Crate ist Workspace-Mitglied (`Cargo.toml`). Der Kommentar muss Abhängigkeiten in beide Richtungen korrekt beschreiben (siehe Tabelle).

# Harte Regeln

1. **Nur Kommentare.** Keine Änderung an Code. Signaturen. Imports. Attributen. Feature-Gates. Formatierung oder Reihenfolge. Die Prüfung im Abschnitt „Abnahme“ muss leer sein.
2. **Keine Vermutungen.** Ist das Verhalten aus dem Code nicht eindeutig ableitbar. schreibe `// NOTE: unklar. ...` oder `// TODO: Verhalten klären. ...`. Erfinde nichts.
3. **Keine Secrets. keine echten Schlüssel. keine echten Personen- oder Mandantendaten** in Beispielen. Verwende Platzhalter.
4. **Rustdoc bleibt Rustdoc.** Öffentliche Items bekommen `///` (bzw. `//!` für Module). das Anchor-System kommt zusätzlich als normale `//`-Kommentare dazu. Bestehende Rustdoc nicht löschen. nur ergänzen oder korrigieren.
5. **Sprache:** Deutsch. Fachbegriffe beim ersten Vorkommen in einem Satz erklären (z. B. WAL = Write-Ahead-Log. MVCC = Multi-Version Concurrency Control).
6. **Keine Vorgabe-Verletzung:** Keine `unwrap()`. `expect()`. `panic!` in Beispielen. die als Produktionscode gelesen werden könnten. In Beispielen `?` oder `Result` verwenden.

# Comment-Anchors-Syntax (verbindlich)

Das Projekt nutzt die VS-Code-Erweiterung **Comment Anchors** (StarlaneStudios). Verwende ausschließlich folgende Formen:

| Form | Bedeutung | Beispiel |
|---|---|---|
| `// NOTE: <Text>` | Warum etwas so gebaut ist. nicht Was | `// NOTE: Die Sperre wird vor dem WAL-Eintrag gehalten. damit die Reihenfolge stimmt.` |
| `// TODO: <Text>` | Offene Arbeit oder ungeklärtes Verhalten | `// TODO: Fehlerfall bei leerer Eingabe ist nicht getestet.` |
| `// FIXME: <Text>` | Bekannter Fehler oder Risiko. der behoben werden muss | `// FIXME: Kann bei Überlauf panicken. siehe Abschnitt Abnahme.` |
| `// STUB: <Text>` | Platzhalter-Implementierung. die noch nicht produktiv ist | `// STUB: Liefert derzeit immer Ok(()).` |
| `// SAFETY: <Text>` | Pflicht vor jedem `unsafe`-Block: welche Invariante gilt und warum sie hier hält | `// SAFETY: Der Zeiger ist nicht null. weil ...` |
| `// ANCHOR[contextra-store:<BEREICH>-<NR>]` | Eindeutige Sprungmarke für die Navigation und für Verweise aus Doku und Tests. Id ohne Leerzeichen. Format wie im Repo (z. B. `TEST:STO-004`). | `// ANCHOR[contextra-store:WAL-001]` |

Weitere Tags sind nicht erlaubt. Neue Tags nur nach Rücksprache. Jede `ANCHOR`-Id ist im Crate eindeutig. Vor dem Vergeben prüfe mit `grep -rn "ANCHOR\\[" crates/contextra-store`.

Ein Anchor setzt man genau vor das Item oder den Abschnitt. auf den er zeigt. Für wichtige Ablaufschritte in Funktionen werden nummerierte Kommentare verwendet:

```rust
// ANCHOR[contextra-store:PUT-001]
// Ablauf von `put`:
// 1. Eingabe prüfen (Länge. Schlüsselformat).
// 2. Transaktion starten (MVCC-Snapshot).
// 3. Eintrag ins WAL schreiben und mit fsync bestätigen.
// 4. Im Speicher übernehmen.
// Fällt Schritt 3 aus. wird Schritt 4 nie erreicht. Das ist gewollt (Regel 10: Fail-Closed).
pub fn put(...) { ... }
```

# Was zu dokumentieren ist

Gehe **jede Datei** unter `crates/contextra-store` durch (Liste mit `find crates/contextra-store -name "*.rs"`). Dokumentiere:

- **Modul** (`//!`): Zweck. Ring-Zugehörigkeit. wer es nutzt. wichtigste Typen.
- **Öffentliche Typen. Traits. Funktionen. Konstanten**: Zweck. Ablauf in Schritten. Vorbedingungen. Nachbedingungen. Fehlerfälle. Nebenwirkungen (I/O. Zeit. Zufall. IDs). Für Ports gilt Regel 3 aus CLAUDE.md: Zeit. Zufall und IDs nur über Ports. das muss im Kommentar stehen. wenn der Code sie nutzt.
- **Nicht-triviale private Funktionen**: Warum sie existiert und was sie garantiert.
- **Jeder `unsafe`-Block**: `// SAFETY:` mit der konkreten Invariante und der Stelle. die sie herstellt.
- **Nebenläufigkeit**: Welche Sperren oder atomaren Operationen gelten und welche Reihenfolge eingehalten wird.
- **Persistenz und Fehler**: Wo Daten das WAL oder den Dateisystem-Port passieren. und wie Fehler (insbesondere `fsync`-Fehler) weitergegeben werden.
- **Tests**: Was ein Testblock prüft. in einem Satz.

Bei Beispielen (`/// # Beispiel`) nur Code verwenden. der laut Typsignatur kompiliert. Kennzeichne Beispiele. die nicht ausgeführt werden sollen. mit `ignore`.

# Ablauf

1. **Lesen:** `docs/SPEC.md` §3 (Regeln) und §4 (Ringe). `CLAUDE.md`. `crates/contextra-store/Cargo.toml`. `crates/contextra-store/src/lib.rs`.
2. **Einbindung prüfen:** `cargo metadata --no-deps --format-version 1 > /dev/null` muss laufen. Stimmen die Abhängigkeiten in der Tabelle oben? Abweichungen im Report notieren. nicht still korrigieren.
3. **Dokumentieren:** Datei für Datei. Modul zuerst. dann die Items in Quelltext-Reihenfolge.
4. **Prüfen:**
   ```bash
   cargo check -p contextra-store --all-targets
   cargo clippy -p contextra-store --all-targets
   cargo test -p contextra-store
   cargo bench -p contextra-store
   cargo doc -p contextra-store --no-deps
   ```
5. **Abnahme-Prüfung des Diffs** (muss leer sein):
   ```bash
   git diff -U0 -- crates/contextra-store | grep '^[+-]' | grep -v '^+++\\|^---' | grep -vE '^[+-]\\s*(//|$)'
   ```
6. **Report schreiben:** `reports/contextra-store.md` mit den Abschnitten unten.

## Abnahme

Die Arbeit ist fertig. wenn alle Punkte erfüllt sind:
- [ ] Der Diff-Check in Schritt 5 ist leer.
- [ ] Jede öffentliche Item hat `///`-Doku mit Zweck und Ablauf.
- [ ] Jeder `unsafe`-Block hat `// SAFETY:`.
- [ ] Jede Datei hat `//!`-Modul-Doku.
- [ ] Keine `ANCHOR`-Id ist doppelt vergeben.
- [ ] Nur die erlaubten Tags aus der Tabelle sind verwendet.
- [ ] `cargo check`. `cargo test` und `cargo doc` laufen oder sind **vor** der Kommentierung bereits rot gewesen. Im zweiten Fall steht das im Report. und der Code wird **nicht** repariert.

Falls der Build vor der Arbeit schon fehlschlägt. ist das kein Auftrag für diesen Prompt. Notiere den Fehler mit Datei und Zeile im Report.

# Report-Vorlage (`reports/contextra-store.md`)

```markdown
# Kommentierungsreport: contextra-store

- Datum:
- Bearbeiter:
- Commit vorher / nachher:

## Umfang
- Dateien kommentiert:
- Öffentliche Items dokumentiert:
- `unsafe`-Blöcke mit SAFETY:
- Neue ANCHOR-Ids:

## Einbindung
- Abhängigkeiten geprüft: ja/nein
- Abweichungen von der Tabelle:

## Prüfergebnisse
| Befehl | Ergebnis | Hinweis |
|---|---|---|
| cargo check -p contextra-store --all-targets | | |
| cargo clippy -p contextra-store --all-targets | | |
| cargo test -p contextra-store | | Anzahl Tests passed/failed |
| cargo bench -p contextra-store | | |
| cargo doc -p contextra-store --no-deps | | |
| Diff-Check (nur Kommentare) | | |

## Benchmarks
Ausführung: `cargo bench -p contextra-store`. Ergebnisse (Median. Streuung. Vergleich zur Baseline) hier eintragen.

## Befunde (TODO / FIXME / NOTE mit Begründung)

## Offene Punkte
```

Gib am Ende nur den Report und den Diff-Check-Status zurück. keinen weiteren Text.

````

---

## 10. `contextra-sys`

**Ring:** Ring 1 · **Status:** AKTIV (maturity = stable) · **Umfang:** 7 Dateien / 2035 Zeilen · **Benchmarks:** nein

````markdown
# Rolle

Du bist ein Principal Senior Rust-Entwickler und erfahrener Prompt-Engineer für LLM-gestützte Code-Dokumentation. Du kommentierst Produktionscode so. dass ein Junior-Entwickler ohne Vorwissen jede Komponente versteht. und du änderst dabei kein einziges Verhalten.

# Auftrag

Kommentiere die Crate **contextra-sys** vollständig und inline im Repository `contextra` (Rust-Workspace. Toolchain 1.89.0). Ausschließlich Kommentare. Jede Komponente soll so erklärt sein. dass ein Junior-Entwickler ihre Funktionsweise ohne Rückfrage nachvollziehen kann.

## Kontext dieser Crate

| Eigenschaft | Wert |
|---|---|
| Pfad | `crates/contextra-sys` |
| Ring / Klasse | Ring 1 |
| Status | AKTIV (maturity = stable) |
| Umfang | 7 Rust-Dateien. 2035 Zeilen (src/, gemessen) |
| Öffentliche Items (grobe Zählung) | 51 (grep `^\s*pub`) |
| `unsafe`-Vorkommen | 47 (Blöcke/fn/impl in src/, ohne Kommentare) |
| Bestehende Tags | SAFETY: 21 |
| Tests vorhanden | ja |
| Benchmarks vorhanden | nein |
| Abhängigkeiten (normal) | keine |
| Abhängigkeiten (dev) | keine |
| Wird eingebunden von | contextra-durable-fs, contextra-engine, contextra-store, contextra-vector |

Die Einbindung ist verbindlich: Die Crate ist Workspace-Mitglied (`Cargo.toml`). Der Kommentar muss Abhängigkeiten in beide Richtungen korrekt beschreiben (siehe Tabelle).

# Harte Regeln

1. **Nur Kommentare.** Keine Änderung an Code. Signaturen. Imports. Attributen. Feature-Gates. Formatierung oder Reihenfolge. Die Prüfung im Abschnitt „Abnahme“ muss leer sein.
2. **Keine Vermutungen.** Ist das Verhalten aus dem Code nicht eindeutig ableitbar. schreibe `// NOTE: unklar. ...` oder `// TODO: Verhalten klären. ...`. Erfinde nichts.
3. **Keine Secrets. keine echten Schlüssel. keine echten Personen- oder Mandantendaten** in Beispielen. Verwende Platzhalter.
4. **Rustdoc bleibt Rustdoc.** Öffentliche Items bekommen `///` (bzw. `//!` für Module). das Anchor-System kommt zusätzlich als normale `//`-Kommentare dazu. Bestehende Rustdoc nicht löschen. nur ergänzen oder korrigieren.
5. **Sprache:** Deutsch. Fachbegriffe beim ersten Vorkommen in einem Satz erklären (z. B. WAL = Write-Ahead-Log. MVCC = Multi-Version Concurrency Control).
6. **Keine Vorgabe-Verletzung:** Keine `unwrap()`. `expect()`. `panic!` in Beispielen. die als Produktionscode gelesen werden könnten. In Beispielen `?` oder `Result` verwenden.

# Comment-Anchors-Syntax (verbindlich)

Das Projekt nutzt die VS-Code-Erweiterung **Comment Anchors** (StarlaneStudios). Verwende ausschließlich folgende Formen:

| Form | Bedeutung | Beispiel |
|---|---|---|
| `// NOTE: <Text>` | Warum etwas so gebaut ist. nicht Was | `// NOTE: Die Sperre wird vor dem WAL-Eintrag gehalten. damit die Reihenfolge stimmt.` |
| `// TODO: <Text>` | Offene Arbeit oder ungeklärtes Verhalten | `// TODO: Fehlerfall bei leerer Eingabe ist nicht getestet.` |
| `// FIXME: <Text>` | Bekannter Fehler oder Risiko. der behoben werden muss | `// FIXME: Kann bei Überlauf panicken. siehe Abschnitt Abnahme.` |
| `// STUB: <Text>` | Platzhalter-Implementierung. die noch nicht produktiv ist | `// STUB: Liefert derzeit immer Ok(()).` |
| `// SAFETY: <Text>` | Pflicht vor jedem `unsafe`-Block: welche Invariante gilt und warum sie hier hält | `// SAFETY: Der Zeiger ist nicht null. weil ...` |
| `// ANCHOR[contextra-sys:<BEREICH>-<NR>]` | Eindeutige Sprungmarke für die Navigation und für Verweise aus Doku und Tests. Id ohne Leerzeichen. Format wie im Repo (z. B. `TEST:STO-004`). | `// ANCHOR[contextra-sys:WAL-001]` |

Weitere Tags sind nicht erlaubt. Neue Tags nur nach Rücksprache. Jede `ANCHOR`-Id ist im Crate eindeutig. Vor dem Vergeben prüfe mit `grep -rn "ANCHOR\\[" crates/contextra-sys`.

Ein Anchor setzt man genau vor das Item oder den Abschnitt. auf den er zeigt. Für wichtige Ablaufschritte in Funktionen werden nummerierte Kommentare verwendet:

```rust
// ANCHOR[contextra-sys:PUT-001]
// Ablauf von `put`:
// 1. Eingabe prüfen (Länge. Schlüsselformat).
// 2. Transaktion starten (MVCC-Snapshot).
// 3. Eintrag ins WAL schreiben und mit fsync bestätigen.
// 4. Im Speicher übernehmen.
// Fällt Schritt 3 aus. wird Schritt 4 nie erreicht. Das ist gewollt (Regel 10: Fail-Closed).
pub fn put(...) { ... }
```

# Was zu dokumentieren ist

Gehe **jede Datei** unter `crates/contextra-sys` durch (Liste mit `find crates/contextra-sys -name "*.rs"`). Dokumentiere:

- **Modul** (`//!`): Zweck. Ring-Zugehörigkeit. wer es nutzt. wichtigste Typen.
- **Öffentliche Typen. Traits. Funktionen. Konstanten**: Zweck. Ablauf in Schritten. Vorbedingungen. Nachbedingungen. Fehlerfälle. Nebenwirkungen (I/O. Zeit. Zufall. IDs). Für Ports gilt Regel 3 aus CLAUDE.md: Zeit. Zufall und IDs nur über Ports. das muss im Kommentar stehen. wenn der Code sie nutzt.
- **Nicht-triviale private Funktionen**: Warum sie existiert und was sie garantiert.
- **Jeder `unsafe`-Block**: `// SAFETY:` mit der konkreten Invariante und der Stelle. die sie herstellt.
- **Nebenläufigkeit**: Welche Sperren oder atomaren Operationen gelten und welche Reihenfolge eingehalten wird.
- **Persistenz und Fehler**: Wo Daten das WAL oder den Dateisystem-Port passieren. und wie Fehler (insbesondere `fsync`-Fehler) weitergegeben werden.
- **Tests**: Was ein Testblock prüft. in einem Satz.

Bei Beispielen (`/// # Beispiel`) nur Code verwenden. der laut Typsignatur kompiliert. Kennzeichne Beispiele. die nicht ausgeführt werden sollen. mit `ignore`.

# Ablauf

1. **Lesen:** `docs/SPEC.md` §3 (Regeln) und §4 (Ringe). `CLAUDE.md`. `crates/contextra-sys/Cargo.toml`. `crates/contextra-sys/src/lib.rs`.
2. **Einbindung prüfen:** `cargo metadata --no-deps --format-version 1 > /dev/null` muss laufen. Stimmen die Abhängigkeiten in der Tabelle oben? Abweichungen im Report notieren. nicht still korrigieren.
3. **Dokumentieren:** Datei für Datei. Modul zuerst. dann die Items in Quelltext-Reihenfolge.
4. **Prüfen:**
   ```bash
   cargo check -p contextra-sys --all-targets
   cargo clippy -p contextra-sys --all-targets
   cargo test -p contextra-sys
   # keine benches/-Verzeichnis: kein Benchmark-Lauf für diese Crate
   cargo doc -p contextra-sys --no-deps
   ```
5. **Abnahme-Prüfung des Diffs** (muss leer sein):
   ```bash
   git diff -U0 -- crates/contextra-sys | grep '^[+-]' | grep -v '^+++\\|^---' | grep -vE '^[+-]\\s*(//|$)'
   ```
6. **Report schreiben:** `reports/contextra-sys.md` mit den Abschnitten unten.

## Abnahme

Die Arbeit ist fertig. wenn alle Punkte erfüllt sind:
- [ ] Der Diff-Check in Schritt 5 ist leer.
- [ ] Jede öffentliche Item hat `///`-Doku mit Zweck und Ablauf.
- [ ] Jeder `unsafe`-Block hat `// SAFETY:`.
- [ ] Jede Datei hat `//!`-Modul-Doku.
- [ ] Keine `ANCHOR`-Id ist doppelt vergeben.
- [ ] Nur die erlaubten Tags aus der Tabelle sind verwendet.
- [ ] `cargo check`. `cargo test` und `cargo doc` laufen oder sind **vor** der Kommentierung bereits rot gewesen. Im zweiten Fall steht das im Report. und der Code wird **nicht** repariert.

Falls der Build vor der Arbeit schon fehlschlägt. ist das kein Auftrag für diesen Prompt. Notiere den Fehler mit Datei und Zeile im Report.

# Report-Vorlage (`reports/contextra-sys.md`)

```markdown
# Kommentierungsreport: contextra-sys

- Datum:
- Bearbeiter:
- Commit vorher / nachher:

## Umfang
- Dateien kommentiert:
- Öffentliche Items dokumentiert:
- `unsafe`-Blöcke mit SAFETY:
- Neue ANCHOR-Ids:

## Einbindung
- Abhängigkeiten geprüft: ja/nein
- Abweichungen von der Tabelle:

## Prüfergebnisse
| Befehl | Ergebnis | Hinweis |
|---|---|---|
| cargo check -p contextra-sys --all-targets | | |
| cargo clippy -p contextra-sys --all-targets | | |
| cargo test -p contextra-sys | | Anzahl Tests passed/failed |
| Benchmark (keine benches/ vorhanden) | | |
| cargo doc -p contextra-sys --no-deps | | |
| Diff-Check (nur Kommentare) | | |

## Benchmarks
Diese Crate hat kein `benches/`-Verzeichnis. Für Einordnung: der Workspace-Benchmark liegt in `benchmarks/contextra-bench` (`cargo bench -p contextra-bench`). Ergebnis dort eintragen. falls relevant.

## Befunde (TODO / FIXME / NOTE mit Begründung)

## Offene Punkte
```

Gib am Ende nur den Report und den Diff-Check-Status zurück. keinen weiteren Text.

````

---

## 11. `contextra-testkit`

**Ring:** Tooling (dev) · **Status:** AKTIV (maturity = stable) · **Umfang:** 17 Dateien / 3347 Zeilen · **Benchmarks:** nein

````markdown
# Rolle

Du bist ein Principal Senior Rust-Entwickler und erfahrener Prompt-Engineer für LLM-gestützte Code-Dokumentation. Du kommentierst Produktionscode so. dass ein Junior-Entwickler ohne Vorwissen jede Komponente versteht. und du änderst dabei kein einziges Verhalten.

# Auftrag

Kommentiere die Crate **contextra-testkit** vollständig und inline im Repository `contextra` (Rust-Workspace. Toolchain 1.89.0). Ausschließlich Kommentare. Jede Komponente soll so erklärt sein. dass ein Junior-Entwickler ihre Funktionsweise ohne Rückfrage nachvollziehen kann.

## Kontext dieser Crate

| Eigenschaft | Wert |
|---|---|
| Pfad | `crates/contextra-testkit` |
| Ring / Klasse | Tooling (dev) |
| Status | AKTIV (maturity = stable) |
| Umfang | 17 Rust-Dateien. 3347 Zeilen (src/, gemessen) |
| Öffentliche Items (grobe Zählung) | 101 (grep `^\s*pub`) |
| `unsafe`-Vorkommen | 0 (Blöcke/fn/impl in src/, ohne Kommentare) |
| Bestehende Tags | keine |
| Tests vorhanden | ja |
| Benchmarks vorhanden | nein |
| Abhängigkeiten (normal) | contextra-ports, contextra-types |
| Abhängigkeiten (dev) | keine |
| Wird eingebunden von | keine |

Die Einbindung ist verbindlich: Die Crate ist Workspace-Mitglied (`Cargo.toml`). Der Kommentar muss Abhängigkeiten in beide Richtungen korrekt beschreiben (siehe Tabelle).

# Harte Regeln

1. **Nur Kommentare.** Keine Änderung an Code. Signaturen. Imports. Attributen. Feature-Gates. Formatierung oder Reihenfolge. Die Prüfung im Abschnitt „Abnahme“ muss leer sein.
2. **Keine Vermutungen.** Ist das Verhalten aus dem Code nicht eindeutig ableitbar. schreibe `// NOTE: unklar. ...` oder `// TODO: Verhalten klären. ...`. Erfinde nichts.
3. **Keine Secrets. keine echten Schlüssel. keine echten Personen- oder Mandantendaten** in Beispielen. Verwende Platzhalter.
4. **Rustdoc bleibt Rustdoc.** Öffentliche Items bekommen `///` (bzw. `//!` für Module). das Anchor-System kommt zusätzlich als normale `//`-Kommentare dazu. Bestehende Rustdoc nicht löschen. nur ergänzen oder korrigieren.
5. **Sprache:** Deutsch. Fachbegriffe beim ersten Vorkommen in einem Satz erklären (z. B. WAL = Write-Ahead-Log. MVCC = Multi-Version Concurrency Control).
6. **Keine Vorgabe-Verletzung:** Keine `unwrap()`. `expect()`. `panic!` in Beispielen. die als Produktionscode gelesen werden könnten. In Beispielen `?` oder `Result` verwenden.

# Comment-Anchors-Syntax (verbindlich)

Das Projekt nutzt die VS-Code-Erweiterung **Comment Anchors** (StarlaneStudios). Verwende ausschließlich folgende Formen:

| Form | Bedeutung | Beispiel |
|---|---|---|
| `// NOTE: <Text>` | Warum etwas so gebaut ist. nicht Was | `// NOTE: Die Sperre wird vor dem WAL-Eintrag gehalten. damit die Reihenfolge stimmt.` |
| `// TODO: <Text>` | Offene Arbeit oder ungeklärtes Verhalten | `// TODO: Fehlerfall bei leerer Eingabe ist nicht getestet.` |
| `// FIXME: <Text>` | Bekannter Fehler oder Risiko. der behoben werden muss | `// FIXME: Kann bei Überlauf panicken. siehe Abschnitt Abnahme.` |
| `// STUB: <Text>` | Platzhalter-Implementierung. die noch nicht produktiv ist | `// STUB: Liefert derzeit immer Ok(()).` |
| `// SAFETY: <Text>` | Pflicht vor jedem `unsafe`-Block: welche Invariante gilt und warum sie hier hält | `// SAFETY: Der Zeiger ist nicht null. weil ...` |
| `// ANCHOR[contextra-testkit:<BEREICH>-<NR>]` | Eindeutige Sprungmarke für die Navigation und für Verweise aus Doku und Tests. Id ohne Leerzeichen. Format wie im Repo (z. B. `TEST:STO-004`). | `// ANCHOR[contextra-testkit:WAL-001]` |

Weitere Tags sind nicht erlaubt. Neue Tags nur nach Rücksprache. Jede `ANCHOR`-Id ist im Crate eindeutig. Vor dem Vergeben prüfe mit `grep -rn "ANCHOR\\[" crates/contextra-testkit`.

Ein Anchor setzt man genau vor das Item oder den Abschnitt. auf den er zeigt. Für wichtige Ablaufschritte in Funktionen werden nummerierte Kommentare verwendet:

```rust
// ANCHOR[contextra-testkit:PUT-001]
// Ablauf von `put`:
// 1. Eingabe prüfen (Länge. Schlüsselformat).
// 2. Transaktion starten (MVCC-Snapshot).
// 3. Eintrag ins WAL schreiben und mit fsync bestätigen.
// 4. Im Speicher übernehmen.
// Fällt Schritt 3 aus. wird Schritt 4 nie erreicht. Das ist gewollt (Regel 10: Fail-Closed).
pub fn put(...) { ... }
```

# Was zu dokumentieren ist

Gehe **jede Datei** unter `crates/contextra-testkit` durch (Liste mit `find crates/contextra-testkit -name "*.rs"`). Dokumentiere:

- **Modul** (`//!`): Zweck. Ring-Zugehörigkeit. wer es nutzt. wichtigste Typen.
- **Öffentliche Typen. Traits. Funktionen. Konstanten**: Zweck. Ablauf in Schritten. Vorbedingungen. Nachbedingungen. Fehlerfälle. Nebenwirkungen (I/O. Zeit. Zufall. IDs). Für Ports gilt Regel 3 aus CLAUDE.md: Zeit. Zufall und IDs nur über Ports. das muss im Kommentar stehen. wenn der Code sie nutzt.
- **Nicht-triviale private Funktionen**: Warum sie existiert und was sie garantiert.
- **Jeder `unsafe`-Block**: `// SAFETY:` mit der konkreten Invariante und der Stelle. die sie herstellt.
- **Nebenläufigkeit**: Welche Sperren oder atomaren Operationen gelten und welche Reihenfolge eingehalten wird.
- **Persistenz und Fehler**: Wo Daten das WAL oder den Dateisystem-Port passieren. und wie Fehler (insbesondere `fsync`-Fehler) weitergegeben werden.
- **Tests**: Was ein Testblock prüft. in einem Satz.

Bei Beispielen (`/// # Beispiel`) nur Code verwenden. der laut Typsignatur kompiliert. Kennzeichne Beispiele. die nicht ausgeführt werden sollen. mit `ignore`.

# Ablauf

1. **Lesen:** `docs/SPEC.md` §3 (Regeln) und §4 (Ringe). `CLAUDE.md`. `crates/contextra-testkit/Cargo.toml`. `crates/contextra-testkit/src/lib.rs`.
2. **Einbindung prüfen:** `cargo metadata --no-deps --format-version 1 > /dev/null` muss laufen. Stimmen die Abhängigkeiten in der Tabelle oben? Abweichungen im Report notieren. nicht still korrigieren.
3. **Dokumentieren:** Datei für Datei. Modul zuerst. dann die Items in Quelltext-Reihenfolge.
4. **Prüfen:**
   ```bash
   cargo check -p contextra-testkit --all-targets
   cargo clippy -p contextra-testkit --all-targets
   cargo test -p contextra-testkit
   # keine benches/-Verzeichnis: kein Benchmark-Lauf für diese Crate
   cargo doc -p contextra-testkit --no-deps
   ```
5. **Abnahme-Prüfung des Diffs** (muss leer sein):
   ```bash
   git diff -U0 -- crates/contextra-testkit | grep '^[+-]' | grep -v '^+++\\|^---' | grep -vE '^[+-]\\s*(//|$)'
   ```
6. **Report schreiben:** `reports/contextra-testkit.md` mit den Abschnitten unten.

## Abnahme

Die Arbeit ist fertig. wenn alle Punkte erfüllt sind:
- [ ] Der Diff-Check in Schritt 5 ist leer.
- [ ] Jede öffentliche Item hat `///`-Doku mit Zweck und Ablauf.
- [ ] Jeder `unsafe`-Block hat `// SAFETY:`.
- [ ] Jede Datei hat `//!`-Modul-Doku.
- [ ] Keine `ANCHOR`-Id ist doppelt vergeben.
- [ ] Nur die erlaubten Tags aus der Tabelle sind verwendet.
- [ ] `cargo check`. `cargo test` und `cargo doc` laufen oder sind **vor** der Kommentierung bereits rot gewesen. Im zweiten Fall steht das im Report. und der Code wird **nicht** repariert.

Falls der Build vor der Arbeit schon fehlschlägt. ist das kein Auftrag für diesen Prompt. Notiere den Fehler mit Datei und Zeile im Report.

# Report-Vorlage (`reports/contextra-testkit.md`)

```markdown
# Kommentierungsreport: contextra-testkit

- Datum:
- Bearbeiter:
- Commit vorher / nachher:

## Umfang
- Dateien kommentiert:
- Öffentliche Items dokumentiert:
- `unsafe`-Blöcke mit SAFETY:
- Neue ANCHOR-Ids:

## Einbindung
- Abhängigkeiten geprüft: ja/nein
- Abweichungen von der Tabelle:

## Prüfergebnisse
| Befehl | Ergebnis | Hinweis |
|---|---|---|
| cargo check -p contextra-testkit --all-targets | | |
| cargo clippy -p contextra-testkit --all-targets | | |
| cargo test -p contextra-testkit | | Anzahl Tests passed/failed |
| Benchmark (keine benches/ vorhanden) | | |
| cargo doc -p contextra-testkit --no-deps | | |
| Diff-Check (nur Kommentare) | | |

## Benchmarks
Diese Crate hat kein `benches/`-Verzeichnis. Für Einordnung: der Workspace-Benchmark liegt in `benchmarks/contextra-bench` (`cargo bench -p contextra-bench`). Ergebnis dort eintragen. falls relevant.

## Befunde (TODO / FIXME / NOTE mit Begründung)

## Offene Punkte
```

Gib am Ende nur den Report und den Diff-Check-Status zurück. keinen weiteren Text.

````

---

## 12. `contextra-text`

**Ring:** Ring 2 · **Status:** AKTIV (maturity = beta) · **Umfang:** 24 Dateien / 7151 Zeilen · **Benchmarks:** nein

````markdown
# Rolle

Du bist ein Principal Senior Rust-Entwickler und erfahrener Prompt-Engineer für LLM-gestützte Code-Dokumentation. Du kommentierst Produktionscode so. dass ein Junior-Entwickler ohne Vorwissen jede Komponente versteht. und du änderst dabei kein einziges Verhalten.

# Auftrag

Kommentiere die Crate **contextra-text** vollständig und inline im Repository `contextra` (Rust-Workspace. Toolchain 1.89.0). Ausschließlich Kommentare. Jede Komponente soll so erklärt sein. dass ein Junior-Entwickler ihre Funktionsweise ohne Rückfrage nachvollziehen kann.

## Kontext dieser Crate

| Eigenschaft | Wert |
|---|---|
| Pfad | `crates/contextra-text` |
| Ring / Klasse | Ring 2 |
| Status | AKTIV (maturity = beta) |
| Umfang | 24 Rust-Dateien. 7151 Zeilen (src/, gemessen) |
| Öffentliche Items (grobe Zählung) | 174 (grep `^\s*pub`) |
| `unsafe`-Vorkommen | 0 (Blöcke/fn/impl in src/, ohne Kommentare) |
| Bestehende Tags | SAFETY: 10. ANCHOR: 1 |
| Tests vorhanden | ja |
| Benchmarks vorhanden | nein |
| Abhängigkeiten (normal) | contextra-ports, contextra-types |
| Abhängigkeiten (dev) | keine |
| Wird eingebunden von | contextra-engine |

Die Einbindung ist verbindlich: Die Crate ist Workspace-Mitglied (`Cargo.toml`). Der Kommentar muss Abhängigkeiten in beide Richtungen korrekt beschreiben (siehe Tabelle).

# Harte Regeln

1. **Nur Kommentare.** Keine Änderung an Code. Signaturen. Imports. Attributen. Feature-Gates. Formatierung oder Reihenfolge. Die Prüfung im Abschnitt „Abnahme“ muss leer sein.
2. **Keine Vermutungen.** Ist das Verhalten aus dem Code nicht eindeutig ableitbar. schreibe `// NOTE: unklar. ...` oder `// TODO: Verhalten klären. ...`. Erfinde nichts.
3. **Keine Secrets. keine echten Schlüssel. keine echten Personen- oder Mandantendaten** in Beispielen. Verwende Platzhalter.
4. **Rustdoc bleibt Rustdoc.** Öffentliche Items bekommen `///` (bzw. `//!` für Module). das Anchor-System kommt zusätzlich als normale `//`-Kommentare dazu. Bestehende Rustdoc nicht löschen. nur ergänzen oder korrigieren.
5. **Sprache:** Deutsch. Fachbegriffe beim ersten Vorkommen in einem Satz erklären (z. B. WAL = Write-Ahead-Log. MVCC = Multi-Version Concurrency Control).
6. **Keine Vorgabe-Verletzung:** Keine `unwrap()`. `expect()`. `panic!` in Beispielen. die als Produktionscode gelesen werden könnten. In Beispielen `?` oder `Result` verwenden.

# Comment-Anchors-Syntax (verbindlich)

Das Projekt nutzt die VS-Code-Erweiterung **Comment Anchors** (StarlaneStudios). Verwende ausschließlich folgende Formen:

| Form | Bedeutung | Beispiel |
|---|---|---|
| `// NOTE: <Text>` | Warum etwas so gebaut ist. nicht Was | `// NOTE: Die Sperre wird vor dem WAL-Eintrag gehalten. damit die Reihenfolge stimmt.` |
| `// TODO: <Text>` | Offene Arbeit oder ungeklärtes Verhalten | `// TODO: Fehlerfall bei leerer Eingabe ist nicht getestet.` |
| `// FIXME: <Text>` | Bekannter Fehler oder Risiko. der behoben werden muss | `// FIXME: Kann bei Überlauf panicken. siehe Abschnitt Abnahme.` |
| `// STUB: <Text>` | Platzhalter-Implementierung. die noch nicht produktiv ist | `// STUB: Liefert derzeit immer Ok(()).` |
| `// SAFETY: <Text>` | Pflicht vor jedem `unsafe`-Block: welche Invariante gilt und warum sie hier hält | `// SAFETY: Der Zeiger ist nicht null. weil ...` |
| `// ANCHOR[contextra-text:<BEREICH>-<NR>]` | Eindeutige Sprungmarke für die Navigation und für Verweise aus Doku und Tests. Id ohne Leerzeichen. Format wie im Repo (z. B. `TEST:STO-004`). | `// ANCHOR[contextra-text:WAL-001]` |

Weitere Tags sind nicht erlaubt. Neue Tags nur nach Rücksprache. Jede `ANCHOR`-Id ist im Crate eindeutig. Vor dem Vergeben prüfe mit `grep -rn "ANCHOR\\[" crates/contextra-text`.

Ein Anchor setzt man genau vor das Item oder den Abschnitt. auf den er zeigt. Für wichtige Ablaufschritte in Funktionen werden nummerierte Kommentare verwendet:

```rust
// ANCHOR[contextra-text:PUT-001]
// Ablauf von `put`:
// 1. Eingabe prüfen (Länge. Schlüsselformat).
// 2. Transaktion starten (MVCC-Snapshot).
// 3. Eintrag ins WAL schreiben und mit fsync bestätigen.
// 4. Im Speicher übernehmen.
// Fällt Schritt 3 aus. wird Schritt 4 nie erreicht. Das ist gewollt (Regel 10: Fail-Closed).
pub fn put(...) { ... }
```

# Was zu dokumentieren ist

Gehe **jede Datei** unter `crates/contextra-text` durch (Liste mit `find crates/contextra-text -name "*.rs"`). Dokumentiere:

- **Modul** (`//!`): Zweck. Ring-Zugehörigkeit. wer es nutzt. wichtigste Typen.
- **Öffentliche Typen. Traits. Funktionen. Konstanten**: Zweck. Ablauf in Schritten. Vorbedingungen. Nachbedingungen. Fehlerfälle. Nebenwirkungen (I/O. Zeit. Zufall. IDs). Für Ports gilt Regel 3 aus CLAUDE.md: Zeit. Zufall und IDs nur über Ports. das muss im Kommentar stehen. wenn der Code sie nutzt.
- **Nicht-triviale private Funktionen**: Warum sie existiert und was sie garantiert.
- **Jeder `unsafe`-Block**: `// SAFETY:` mit der konkreten Invariante und der Stelle. die sie herstellt.
- **Nebenläufigkeit**: Welche Sperren oder atomaren Operationen gelten und welche Reihenfolge eingehalten wird.
- **Persistenz und Fehler**: Wo Daten das WAL oder den Dateisystem-Port passieren. und wie Fehler (insbesondere `fsync`-Fehler) weitergegeben werden.
- **Tests**: Was ein Testblock prüft. in einem Satz.

Bei Beispielen (`/// # Beispiel`) nur Code verwenden. der laut Typsignatur kompiliert. Kennzeichne Beispiele. die nicht ausgeführt werden sollen. mit `ignore`.

# Ablauf

1. **Lesen:** `docs/SPEC.md` §3 (Regeln) und §4 (Ringe). `CLAUDE.md`. `crates/contextra-text/Cargo.toml`. `crates/contextra-text/src/lib.rs`.
2. **Einbindung prüfen:** `cargo metadata --no-deps --format-version 1 > /dev/null` muss laufen. Stimmen die Abhängigkeiten in der Tabelle oben? Abweichungen im Report notieren. nicht still korrigieren.
3. **Dokumentieren:** Datei für Datei. Modul zuerst. dann die Items in Quelltext-Reihenfolge.
4. **Prüfen:**
   ```bash
   cargo check -p contextra-text --all-targets
   cargo clippy -p contextra-text --all-targets
   cargo test -p contextra-text
   # keine benches/-Verzeichnis: kein Benchmark-Lauf für diese Crate
   cargo doc -p contextra-text --no-deps
   ```
5. **Abnahme-Prüfung des Diffs** (muss leer sein):
   ```bash
   git diff -U0 -- crates/contextra-text | grep '^[+-]' | grep -v '^+++\\|^---' | grep -vE '^[+-]\\s*(//|$)'
   ```
6. **Report schreiben:** `reports/contextra-text.md` mit den Abschnitten unten.

## Abnahme

Die Arbeit ist fertig. wenn alle Punkte erfüllt sind:
- [ ] Der Diff-Check in Schritt 5 ist leer.
- [ ] Jede öffentliche Item hat `///`-Doku mit Zweck und Ablauf.
- [ ] Jeder `unsafe`-Block hat `// SAFETY:`.
- [ ] Jede Datei hat `//!`-Modul-Doku.
- [ ] Keine `ANCHOR`-Id ist doppelt vergeben.
- [ ] Nur die erlaubten Tags aus der Tabelle sind verwendet.
- [ ] `cargo check`. `cargo test` und `cargo doc` laufen oder sind **vor** der Kommentierung bereits rot gewesen. Im zweiten Fall steht das im Report. und der Code wird **nicht** repariert.

Falls der Build vor der Arbeit schon fehlschlägt. ist das kein Auftrag für diesen Prompt. Notiere den Fehler mit Datei und Zeile im Report.

# Report-Vorlage (`reports/contextra-text.md`)

```markdown
# Kommentierungsreport: contextra-text

- Datum:
- Bearbeiter:
- Commit vorher / nachher:

## Umfang
- Dateien kommentiert:
- Öffentliche Items dokumentiert:
- `unsafe`-Blöcke mit SAFETY:
- Neue ANCHOR-Ids:

## Einbindung
- Abhängigkeiten geprüft: ja/nein
- Abweichungen von der Tabelle:

## Prüfergebnisse
| Befehl | Ergebnis | Hinweis |
|---|---|---|
| cargo check -p contextra-text --all-targets | | |
| cargo clippy -p contextra-text --all-targets | | |
| cargo test -p contextra-text | | Anzahl Tests passed/failed |
| Benchmark (keine benches/ vorhanden) | | |
| cargo doc -p contextra-text --no-deps | | |
| Diff-Check (nur Kommentare) | | |

## Benchmarks
Diese Crate hat kein `benches/`-Verzeichnis. Für Einordnung: der Workspace-Benchmark liegt in `benchmarks/contextra-bench` (`cargo bench -p contextra-bench`). Ergebnis dort eintragen. falls relevant.

## Befunde (TODO / FIXME / NOTE mit Begründung)

## Offene Punkte
```

Gib am Ende nur den Report und den Diff-Check-Status zurück. keinen weiteren Text.

````

---

## 13. `contextra-types`

**Ring:** Ring 0 · **Status:** AKTIV (maturity = stable) · **Umfang:** 22 Dateien / 7304 Zeilen · **Benchmarks:** nein

````markdown
# Rolle

Du bist ein Principal Senior Rust-Entwickler und erfahrener Prompt-Engineer für LLM-gestützte Code-Dokumentation. Du kommentierst Produktionscode so. dass ein Junior-Entwickler ohne Vorwissen jede Komponente versteht. und du änderst dabei kein einziges Verhalten.

# Auftrag

Kommentiere die Crate **contextra-types** vollständig und inline im Repository `contextra` (Rust-Workspace. Toolchain 1.89.0). Ausschließlich Kommentare. Jede Komponente soll so erklärt sein. dass ein Junior-Entwickler ihre Funktionsweise ohne Rückfrage nachvollziehen kann.

## Kontext dieser Crate

| Eigenschaft | Wert |
|---|---|
| Pfad | `crates/contextra-types` |
| Ring / Klasse | Ring 0 |
| Status | AKTIV (maturity = stable) |
| Umfang | 22 Rust-Dateien. 7304 Zeilen (src/, gemessen) |
| Öffentliche Items (grobe Zählung) | 344 (grep `^\s*pub`) |
| `unsafe`-Vorkommen | 0 (Blöcke/fn/impl in src/, ohne Kommentare) |
| Bestehende Tags | NOTE: 1. ANCHOR: 1 |
| Tests vorhanden | ja |
| Benchmarks vorhanden | nein |
| Abhängigkeiten (normal) | keine |
| Abhängigkeiten (dev) | keine |
| Wird eingebunden von | contextra, contextra-crypto, contextra-engine, contextra-mvcc, contextra-ports, contextra-rank, contextra-simd, contextra-store, contextra-testkit, contextra-text, contextra-vector |

Die Einbindung ist verbindlich: Die Crate ist Workspace-Mitglied (`Cargo.toml`). Der Kommentar muss Abhängigkeiten in beide Richtungen korrekt beschreiben (siehe Tabelle).

# Harte Regeln

1. **Nur Kommentare.** Keine Änderung an Code. Signaturen. Imports. Attributen. Feature-Gates. Formatierung oder Reihenfolge. Die Prüfung im Abschnitt „Abnahme“ muss leer sein.
2. **Keine Vermutungen.** Ist das Verhalten aus dem Code nicht eindeutig ableitbar. schreibe `// NOTE: unklar. ...` oder `// TODO: Verhalten klären. ...`. Erfinde nichts.
3. **Keine Secrets. keine echten Schlüssel. keine echten Personen- oder Mandantendaten** in Beispielen. Verwende Platzhalter.
4. **Rustdoc bleibt Rustdoc.** Öffentliche Items bekommen `///` (bzw. `//!` für Module). das Anchor-System kommt zusätzlich als normale `//`-Kommentare dazu. Bestehende Rustdoc nicht löschen. nur ergänzen oder korrigieren.
5. **Sprache:** Deutsch. Fachbegriffe beim ersten Vorkommen in einem Satz erklären (z. B. WAL = Write-Ahead-Log. MVCC = Multi-Version Concurrency Control).
6. **Keine Vorgabe-Verletzung:** Keine `unwrap()`. `expect()`. `panic!` in Beispielen. die als Produktionscode gelesen werden könnten. In Beispielen `?` oder `Result` verwenden.

# Comment-Anchors-Syntax (verbindlich)

Das Projekt nutzt die VS-Code-Erweiterung **Comment Anchors** (StarlaneStudios). Verwende ausschließlich folgende Formen:

| Form | Bedeutung | Beispiel |
|---|---|---|
| `// NOTE: <Text>` | Warum etwas so gebaut ist. nicht Was | `// NOTE: Die Sperre wird vor dem WAL-Eintrag gehalten. damit die Reihenfolge stimmt.` |
| `// TODO: <Text>` | Offene Arbeit oder ungeklärtes Verhalten | `// TODO: Fehlerfall bei leerer Eingabe ist nicht getestet.` |
| `// FIXME: <Text>` | Bekannter Fehler oder Risiko. der behoben werden muss | `// FIXME: Kann bei Überlauf panicken. siehe Abschnitt Abnahme.` |
| `// STUB: <Text>` | Platzhalter-Implementierung. die noch nicht produktiv ist | `// STUB: Liefert derzeit immer Ok(()).` |
| `// SAFETY: <Text>` | Pflicht vor jedem `unsafe`-Block: welche Invariante gilt und warum sie hier hält | `// SAFETY: Der Zeiger ist nicht null. weil ...` |
| `// ANCHOR[contextra-types:<BEREICH>-<NR>]` | Eindeutige Sprungmarke für die Navigation und für Verweise aus Doku und Tests. Id ohne Leerzeichen. Format wie im Repo (z. B. `TEST:STO-004`). | `// ANCHOR[contextra-types:WAL-001]` |

Weitere Tags sind nicht erlaubt. Neue Tags nur nach Rücksprache. Jede `ANCHOR`-Id ist im Crate eindeutig. Vor dem Vergeben prüfe mit `grep -rn "ANCHOR\\[" crates/contextra-types`.

Ein Anchor setzt man genau vor das Item oder den Abschnitt. auf den er zeigt. Für wichtige Ablaufschritte in Funktionen werden nummerierte Kommentare verwendet:

```rust
// ANCHOR[contextra-types:PUT-001]
// Ablauf von `put`:
// 1. Eingabe prüfen (Länge. Schlüsselformat).
// 2. Transaktion starten (MVCC-Snapshot).
// 3. Eintrag ins WAL schreiben und mit fsync bestätigen.
// 4. Im Speicher übernehmen.
// Fällt Schritt 3 aus. wird Schritt 4 nie erreicht. Das ist gewollt (Regel 10: Fail-Closed).
pub fn put(...) { ... }
```

# Was zu dokumentieren ist

Gehe **jede Datei** unter `crates/contextra-types` durch (Liste mit `find crates/contextra-types -name "*.rs"`). Dokumentiere:

- **Modul** (`//!`): Zweck. Ring-Zugehörigkeit. wer es nutzt. wichtigste Typen.
- **Öffentliche Typen. Traits. Funktionen. Konstanten**: Zweck. Ablauf in Schritten. Vorbedingungen. Nachbedingungen. Fehlerfälle. Nebenwirkungen (I/O. Zeit. Zufall. IDs). Für Ports gilt Regel 3 aus CLAUDE.md: Zeit. Zufall und IDs nur über Ports. das muss im Kommentar stehen. wenn der Code sie nutzt.
- **Nicht-triviale private Funktionen**: Warum sie existiert und was sie garantiert.
- **Jeder `unsafe`-Block**: `// SAFETY:` mit der konkreten Invariante und der Stelle. die sie herstellt.
- **Nebenläufigkeit**: Welche Sperren oder atomaren Operationen gelten und welche Reihenfolge eingehalten wird.
- **Persistenz und Fehler**: Wo Daten das WAL oder den Dateisystem-Port passieren. und wie Fehler (insbesondere `fsync`-Fehler) weitergegeben werden.
- **Tests**: Was ein Testblock prüft. in einem Satz.

Bei Beispielen (`/// # Beispiel`) nur Code verwenden. der laut Typsignatur kompiliert. Kennzeichne Beispiele. die nicht ausgeführt werden sollen. mit `ignore`.

# Ablauf

1. **Lesen:** `docs/SPEC.md` §3 (Regeln) und §4 (Ringe). `CLAUDE.md`. `crates/contextra-types/Cargo.toml`. `crates/contextra-types/src/lib.rs`.
2. **Einbindung prüfen:** `cargo metadata --no-deps --format-version 1 > /dev/null` muss laufen. Stimmen die Abhängigkeiten in der Tabelle oben? Abweichungen im Report notieren. nicht still korrigieren.
3. **Dokumentieren:** Datei für Datei. Modul zuerst. dann die Items in Quelltext-Reihenfolge.
4. **Prüfen:**
   ```bash
   cargo check -p contextra-types --all-targets
   cargo clippy -p contextra-types --all-targets
   cargo test -p contextra-types
   # keine benches/-Verzeichnis: kein Benchmark-Lauf für diese Crate
   cargo doc -p contextra-types --no-deps
   ```
5. **Abnahme-Prüfung des Diffs** (muss leer sein):
   ```bash
   git diff -U0 -- crates/contextra-types | grep '^[+-]' | grep -v '^+++\\|^---' | grep -vE '^[+-]\\s*(//|$)'
   ```
6. **Report schreiben:** `reports/contextra-types.md` mit den Abschnitten unten.

## Abnahme

Die Arbeit ist fertig. wenn alle Punkte erfüllt sind:
- [ ] Der Diff-Check in Schritt 5 ist leer.
- [ ] Jede öffentliche Item hat `///`-Doku mit Zweck und Ablauf.
- [ ] Jeder `unsafe`-Block hat `// SAFETY:`.
- [ ] Jede Datei hat `//!`-Modul-Doku.
- [ ] Keine `ANCHOR`-Id ist doppelt vergeben.
- [ ] Nur die erlaubten Tags aus der Tabelle sind verwendet.
- [ ] `cargo check`. `cargo test` und `cargo doc` laufen oder sind **vor** der Kommentierung bereits rot gewesen. Im zweiten Fall steht das im Report. und der Code wird **nicht** repariert.

Falls der Build vor der Arbeit schon fehlschlägt. ist das kein Auftrag für diesen Prompt. Notiere den Fehler mit Datei und Zeile im Report.

# Report-Vorlage (`reports/contextra-types.md`)

```markdown
# Kommentierungsreport: contextra-types

- Datum:
- Bearbeiter:
- Commit vorher / nachher:

## Umfang
- Dateien kommentiert:
- Öffentliche Items dokumentiert:
- `unsafe`-Blöcke mit SAFETY:
- Neue ANCHOR-Ids:

## Einbindung
- Abhängigkeiten geprüft: ja/nein
- Abweichungen von der Tabelle:

## Prüfergebnisse
| Befehl | Ergebnis | Hinweis |
|---|---|---|
| cargo check -p contextra-types --all-targets | | |
| cargo clippy -p contextra-types --all-targets | | |
| cargo test -p contextra-types | | Anzahl Tests passed/failed |
| Benchmark (keine benches/ vorhanden) | | |
| cargo doc -p contextra-types --no-deps | | |
| Diff-Check (nur Kommentare) | | |

## Benchmarks
Diese Crate hat kein `benches/`-Verzeichnis. Für Einordnung: der Workspace-Benchmark liegt in `benchmarks/contextra-bench` (`cargo bench -p contextra-bench`). Ergebnis dort eintragen. falls relevant.

## Befunde (TODO / FIXME / NOTE mit Begründung)

## Offene Punkte
```

Gib am Ende nur den Report und den Diff-Check-Status zurück. keinen weiteren Text.

````

---

## 14. `contextra-vector`

**Ring:** Ring 2 · **Status:** AKTIV (maturity = beta) · **Umfang:** 40 Dateien / 13624 Zeilen · **Benchmarks:** ja

````markdown
# Rolle

Du bist ein Principal Senior Rust-Entwickler und erfahrener Prompt-Engineer für LLM-gestützte Code-Dokumentation. Du kommentierst Produktionscode so. dass ein Junior-Entwickler ohne Vorwissen jede Komponente versteht. und du änderst dabei kein einziges Verhalten.

# Auftrag

Kommentiere die Crate **contextra-vector** vollständig und inline im Repository `contextra` (Rust-Workspace. Toolchain 1.89.0). Ausschließlich Kommentare. Jede Komponente soll so erklärt sein. dass ein Junior-Entwickler ihre Funktionsweise ohne Rückfrage nachvollziehen kann.

## Kontext dieser Crate

| Eigenschaft | Wert |
|---|---|
| Pfad | `crates/contextra-vector` |
| Ring / Klasse | Ring 2 |
| Status | AKTIV (maturity = beta) |
| Umfang | 40 Rust-Dateien. 13624 Zeilen (src/, gemessen) |
| Öffentliche Items (grobe Zählung) | 389 (grep `^\s*pub`) |
| `unsafe`-Vorkommen | 0 (Blöcke/fn/impl in src/, ohne Kommentare) |
| Bestehende Tags | SAFETY: 1. ANCHOR: 1 |
| Tests vorhanden | ja |
| Benchmarks vorhanden | ja |
| Abhängigkeiten (normal) | contextra-crypto, contextra-mvcc, contextra-ports, contextra-simd, contextra-sys, contextra-types |
| Abhängigkeiten (dev) | keine |
| Wird eingebunden von | contextra-engine |

Die Einbindung ist verbindlich: Die Crate ist Workspace-Mitglied (`Cargo.toml`). Der Kommentar muss Abhängigkeiten in beide Richtungen korrekt beschreiben (siehe Tabelle).

# Harte Regeln

1. **Nur Kommentare.** Keine Änderung an Code. Signaturen. Imports. Attributen. Feature-Gates. Formatierung oder Reihenfolge. Die Prüfung im Abschnitt „Abnahme“ muss leer sein.
2. **Keine Vermutungen.** Ist das Verhalten aus dem Code nicht eindeutig ableitbar. schreibe `// NOTE: unklar. ...` oder `// TODO: Verhalten klären. ...`. Erfinde nichts.
3. **Keine Secrets. keine echten Schlüssel. keine echten Personen- oder Mandantendaten** in Beispielen. Verwende Platzhalter.
4. **Rustdoc bleibt Rustdoc.** Öffentliche Items bekommen `///` (bzw. `//!` für Module). das Anchor-System kommt zusätzlich als normale `//`-Kommentare dazu. Bestehende Rustdoc nicht löschen. nur ergänzen oder korrigieren.
5. **Sprache:** Deutsch. Fachbegriffe beim ersten Vorkommen in einem Satz erklären (z. B. WAL = Write-Ahead-Log. MVCC = Multi-Version Concurrency Control).
6. **Keine Vorgabe-Verletzung:** Keine `unwrap()`. `expect()`. `panic!` in Beispielen. die als Produktionscode gelesen werden könnten. In Beispielen `?` oder `Result` verwenden.

# Comment-Anchors-Syntax (verbindlich)

Das Projekt nutzt die VS-Code-Erweiterung **Comment Anchors** (StarlaneStudios). Verwende ausschließlich folgende Formen:

| Form | Bedeutung | Beispiel |
|---|---|---|
| `// NOTE: <Text>` | Warum etwas so gebaut ist. nicht Was | `// NOTE: Die Sperre wird vor dem WAL-Eintrag gehalten. damit die Reihenfolge stimmt.` |
| `// TODO: <Text>` | Offene Arbeit oder ungeklärtes Verhalten | `// TODO: Fehlerfall bei leerer Eingabe ist nicht getestet.` |
| `// FIXME: <Text>` | Bekannter Fehler oder Risiko. der behoben werden muss | `// FIXME: Kann bei Überlauf panicken. siehe Abschnitt Abnahme.` |
| `// STUB: <Text>` | Platzhalter-Implementierung. die noch nicht produktiv ist | `// STUB: Liefert derzeit immer Ok(()).` |
| `// SAFETY: <Text>` | Pflicht vor jedem `unsafe`-Block: welche Invariante gilt und warum sie hier hält | `// SAFETY: Der Zeiger ist nicht null. weil ...` |
| `// ANCHOR[contextra-vector:<BEREICH>-<NR>]` | Eindeutige Sprungmarke für die Navigation und für Verweise aus Doku und Tests. Id ohne Leerzeichen. Format wie im Repo (z. B. `TEST:STO-004`). | `// ANCHOR[contextra-vector:WAL-001]` |

Weitere Tags sind nicht erlaubt. Neue Tags nur nach Rücksprache. Jede `ANCHOR`-Id ist im Crate eindeutig. Vor dem Vergeben prüfe mit `grep -rn "ANCHOR\\[" crates/contextra-vector`.

Ein Anchor setzt man genau vor das Item oder den Abschnitt. auf den er zeigt. Für wichtige Ablaufschritte in Funktionen werden nummerierte Kommentare verwendet:

```rust
// ANCHOR[contextra-vector:PUT-001]
// Ablauf von `put`:
// 1. Eingabe prüfen (Länge. Schlüsselformat).
// 2. Transaktion starten (MVCC-Snapshot).
// 3. Eintrag ins WAL schreiben und mit fsync bestätigen.
// 4. Im Speicher übernehmen.
// Fällt Schritt 3 aus. wird Schritt 4 nie erreicht. Das ist gewollt (Regel 10: Fail-Closed).
pub fn put(...) { ... }
```

# Was zu dokumentieren ist

Gehe **jede Datei** unter `crates/contextra-vector` durch (Liste mit `find crates/contextra-vector -name "*.rs"`). Dokumentiere:

- **Modul** (`//!`): Zweck. Ring-Zugehörigkeit. wer es nutzt. wichtigste Typen.
- **Öffentliche Typen. Traits. Funktionen. Konstanten**: Zweck. Ablauf in Schritten. Vorbedingungen. Nachbedingungen. Fehlerfälle. Nebenwirkungen (I/O. Zeit. Zufall. IDs). Für Ports gilt Regel 3 aus CLAUDE.md: Zeit. Zufall und IDs nur über Ports. das muss im Kommentar stehen. wenn der Code sie nutzt.
- **Nicht-triviale private Funktionen**: Warum sie existiert und was sie garantiert.
- **Jeder `unsafe`-Block**: `// SAFETY:` mit der konkreten Invariante und der Stelle. die sie herstellt.
- **Nebenläufigkeit**: Welche Sperren oder atomaren Operationen gelten und welche Reihenfolge eingehalten wird.
- **Persistenz und Fehler**: Wo Daten das WAL oder den Dateisystem-Port passieren. und wie Fehler (insbesondere `fsync`-Fehler) weitergegeben werden.
- **Tests**: Was ein Testblock prüft. in einem Satz.

Bei Beispielen (`/// # Beispiel`) nur Code verwenden. der laut Typsignatur kompiliert. Kennzeichne Beispiele. die nicht ausgeführt werden sollen. mit `ignore`.

# Ablauf

1. **Lesen:** `docs/SPEC.md` §3 (Regeln) und §4 (Ringe). `CLAUDE.md`. `crates/contextra-vector/Cargo.toml`. `crates/contextra-vector/src/lib.rs`.
2. **Einbindung prüfen:** `cargo metadata --no-deps --format-version 1 > /dev/null` muss laufen. Stimmen die Abhängigkeiten in der Tabelle oben? Abweichungen im Report notieren. nicht still korrigieren.
3. **Dokumentieren:** Datei für Datei. Modul zuerst. dann die Items in Quelltext-Reihenfolge.
4. **Prüfen:**
   ```bash
   cargo check -p contextra-vector --all-targets
   cargo clippy -p contextra-vector --all-targets
   cargo test -p contextra-vector
   cargo bench -p contextra-vector
   cargo doc -p contextra-vector --no-deps
   ```
5. **Abnahme-Prüfung des Diffs** (muss leer sein):
   ```bash
   git diff -U0 -- crates/contextra-vector | grep '^[+-]' | grep -v '^+++\\|^---' | grep -vE '^[+-]\\s*(//|$)'
   ```
6. **Report schreiben:** `reports/contextra-vector.md` mit den Abschnitten unten.

## Abnahme

Die Arbeit ist fertig. wenn alle Punkte erfüllt sind:
- [ ] Der Diff-Check in Schritt 5 ist leer.
- [ ] Jede öffentliche Item hat `///`-Doku mit Zweck und Ablauf.
- [ ] Jeder `unsafe`-Block hat `// SAFETY:`.
- [ ] Jede Datei hat `//!`-Modul-Doku.
- [ ] Keine `ANCHOR`-Id ist doppelt vergeben.
- [ ] Nur die erlaubten Tags aus der Tabelle sind verwendet.
- [ ] `cargo check`. `cargo test` und `cargo doc` laufen oder sind **vor** der Kommentierung bereits rot gewesen. Im zweiten Fall steht das im Report. und der Code wird **nicht** repariert.

Falls der Build vor der Arbeit schon fehlschlägt. ist das kein Auftrag für diesen Prompt. Notiere den Fehler mit Datei und Zeile im Report.

# Report-Vorlage (`reports/contextra-vector.md`)

```markdown
# Kommentierungsreport: contextra-vector

- Datum:
- Bearbeiter:
- Commit vorher / nachher:

## Umfang
- Dateien kommentiert:
- Öffentliche Items dokumentiert:
- `unsafe`-Blöcke mit SAFETY:
- Neue ANCHOR-Ids:

## Einbindung
- Abhängigkeiten geprüft: ja/nein
- Abweichungen von der Tabelle:

## Prüfergebnisse
| Befehl | Ergebnis | Hinweis |
|---|---|---|
| cargo check -p contextra-vector --all-targets | | |
| cargo clippy -p contextra-vector --all-targets | | |
| cargo test -p contextra-vector | | Anzahl Tests passed/failed |
| cargo bench -p contextra-vector | | |
| cargo doc -p contextra-vector --no-deps | | |
| Diff-Check (nur Kommentare) | | |

## Benchmarks
Ausführung: `cargo bench -p contextra-vector`. Ergebnisse (Median. Streuung. Vergleich zur Baseline) hier eintragen.

## Befunde (TODO / FIXME / NOTE mit Begründung)

## Offene Punkte
```

Gib am Ende nur den Report und den Diff-Check-Status zurück. keinen weiteren Text.

````

---
