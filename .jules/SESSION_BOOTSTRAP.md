# Contextra — Jules Session Bootstrap
> Maschinenausführbare Checkliste für den Sitzungsablauf.
> Für den Spickzettel der Befehle siehe [HARNESS_QUICKSTART.md](HARNESS_QUICKSTART.md). Für den Sitzungsvertrag siehe [PREAMBLE.md](PREAMBLE.md).

- **VETOES.md** (Root): Permanent abgelehnte oder eingeschränkt akzeptierte Features. Vor Arbeitsbeginn prüfen.

---

## 🎯 Kontext-Ladeordnung

Lies NUR `.jules/context/CONTEXT_PACK.md` <!-- doc-ref-ignore --> (enthält Repo-Stand, Baseline, Parallel-Branches, Karte, AGENTS-Auszüge). Bei Bedarf danach die vollständige Crate-`AGENTS.md`; `AGENTS.md`/`WORKING_STATE.md` nur für Fragen, die das Pack offenlassen.

Das Kontextpaket (`.jules/context/CONTEXT_PACK.md` <!-- doc-ref-ignore -->) wird von `cargo xtask jules start` erzeugt und frischt sich selbst auf. Das Skript `refresh-context.sh` darf **nie ohne `--agent` mitten in der Aufgabe aufgerufen werden** (ohne Flag setzt es nichts Destruktives mehr, aber `--agent` ist der vorgesehene Weg).

---

## Phasentabelle (Jules Fassade)

Die Phasen-Runner-Konfiguration (`.jules/harness-phases.toml`) ist die kanonische Quelle aller Gate-Ausführungen.

| Phase | Agent-Gate | Befehl | Zweck |
|---|---|---|---|
| `start` | `guard` / `integrity` | `cargo xtask jules start --card <karte>` | Initialisiert Session, lintet Karte, setzt Claim, erzeugt Kontext (führt intern `session-init` und `context-pack` aus) |
| `check` | `guard` / `symbols` | `cargo xtask jules check` | Prüft Syntax, Lints, Scope, geschützte Pfade & Symbol-Existenz |
| `verify` | `arch` | `cargo xtask jules verify` | Führt Akzeptanz-Tests, DAG-Checks & Architektur-Gates aus |
| `submit` | `integrity` / `report` | `cargo xtask jules submit` | Verifiziert PR-Integrität, Diff-Budget & Session-Report |
| `stop` | `guard` | `cargo xtask jules stop --reason <grund>` | Stoppt die Session und den Loop Guard bei Eskalation |

*Hinweis*: `session-init` und `context-pack` sind interne Teilschritte von `cargo xtask jules start`. Das erzeugte Paket ist `.jules/context/CONTEXT_PACK.md` <!-- doc-ref-ignore -->. Für Details siehe [HARNESS_QUICKSTART.md](HARNESS_QUICKSTART.md).

---

## 🛠️ Wann welches Werkzeug (Pflicht-Gates nach Risk-Klasse)

| Risiko-Klasse (`risk`) / Pfad | Pflichtgate & Regel-Referenz | Befehl / Rezept |
|---|---|---|
| `sec` (Sicherheit / Crypto / Privacy) | `mutants-diff`, `fuzz-smoke` (`rules/fuzzing.md`, `rules/mutation-testing.md`) <!-- doc-ref-ignore --> | `cargo xtask mutants-diff`, `just fuzz-smoke CRATE` |
| `crash` (WAL / Store / Checkpoint / Durabilität) | `wal-replay-verify` (`rules/wal_crypto.md`) <!-- doc-ref-ignore --> | `cargo test -p contextra-store --test wal_*` |
| `simd` (SIMD / Vektorisierung) | SIMD-Sicherheitsanalyse (`rules/simd_safety.md`) <!-- doc-ref-ignore --> | `cargo test -p contextra-simd` |
| Berührt Unsafe-Pfade (`touched_unsafe`) | `unsafe-audit` (`rules/simd_safety.md`) <!-- doc-ref-ignore --> | `cargo xtask check-unsafe-islands` |
| Berührt Cargo-Dateien (`touched_cargo`) | `dependency-gate` (`rules/dependencies.md`) <!-- doc-ref-ignore --> | `cargo xtask dependency-graph-audit` |
| Berührt Geschützte Pfade (`touched_protected` / Loom / Bench) | `protected-paths`, Loom/Miri (`rules/loom-miri.md`), Bench (`rules/benchmarking.md`) <!-- doc-ref-ignore --> | `just loom-store`, `just bench-setup` |
| `none` / `utf8` / `fusion` / `concurrency` / `calib` | Standard-Verifikation (`rules/testing.md`, `rules/test_quality.md`) <!-- doc-ref-ignore --> | `cargo xtask jules verify` |

---

## 🏷️ Kommentarsystem & Tag-Grammatik (rules/tag_taxonomy.md)

Jeder im Code platzierte Tag muss der Grammatik entsprechen:
```rust
// AI-TAG[KATEGORIE][SEVERITY] Kurzbeschreibung (ID: AGT-<CRATE>-<8-hex>) (TS: <ISO-8601-UTC>) (SESSION: <HASH>)
// BEFUND: Detaillierte Analyse.
// RISIKO: Auswirkung.
// EMPFEHLUNG: Konkrete Handlung.

// ANCHOR[TYP:ID] STATUS:OPEN (TS: <ISO-8601-UTC>) (SESSION: <HASH>)
```

---

## 🔍 Review Pass Regel
Bei Peer-Reviews oder Selbstkontrolle muss der Kommentar die explizite Markierung `PRÜFER-KONTEXT: FRESH` tragen.

---

## 🔄 Notfall-Eskalation (Prompt-Thrashing)

Wenn derselbe Compiler- oder Gate-Fehler nach 2 Iterationen nicht behoben ist:
1. **STOPP** — keinen weiteren Code schreiben.
2. Fehler auf minimales Beispiel reduzieren und in Session-Log dokumentieren.
3. Session abbrechen mit `cargo xtask jules stop --reason thrash`.
