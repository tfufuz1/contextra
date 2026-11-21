# CLAUDE.md — Einstieg für Claude-Sitzungen im Contextra-Repo

Stand: 21.11.2026 · Repo-HEAD beim Audit: `2fd0994` · Maßgebliche Spezifikation: `docs/SPEC.md` (v4.1, Nachtrag v4.1.1 vom 21.11.2026, **Soll-Zustand**)

Das Projekt ist mitten im Umbau. Die Spec beschreibt das Ziel, nicht den Ist-Stand. Vor jeder Änderung den betreffenden Spec-Abschnitt lesen und Anhang B (Messungen) mit dem Ist vergleichen.

**Schnellstart in jeder Sitzung:**
```bash
cd /home/claude/contextra
tools/quickcheck.sh sync layering   # Spec-Anhänge und Schichtregeln
tools/quickcheck.sh build           # Workspace bauen (stable-Toolchain)
```

---

## 0. Arbeitsregeln (verbindlich)

- **Subagenten nur als Haiku.** Beim `Agent`-Tool immer `model: "haiku"` setzen. Sonnet nie verwenden. Opus/Fable nur, wenn der Nutzer es ausdrücklich verlangt.
- **Spec ist Quelle.** Die TOML-Blöcke in Anhang D und E der Spec sind normativ. `benchmarks/budgets.toml` und `capabilities.toml` werden nie von Hand geändert: erst die Spec ändern, dann `python3 tools/check_spec_sync.py sync`, dann im selben Commit committen (SYNC-1).
- **Sprache:** Antworten an den Nutzer und Kommentare/Doku im Repo auf Deutsch (Bestand ist deutsch; Rust-Bezeichner englisch).
- **Push:** Nur über den GitHub-Zugang der Sitzung. Der Ablauf steht in Abschnitt 7 und hat am 21.11.2026 funktioniert.
- **Commit-Trailer (Pflicht ab jetzt):** Jede Commit-Message endet mit
  ```
  Co-Authored-By: Claude Haiku 5.5 <noreply@anthropic.com>
  Claude-Session: https://claude.ai/code/session_01BByRVn81CFexSvyN4AheTv
  ```
  Pull-Request-Beschreibungen enden mit `🤖 Generated with [Claude Code](https://claude.com/claude-code)` und dem Session-Link.
- **Fallback ZIP:** Ist kein Push möglich, liefert du dem Nutzer im Chat eine ZIP-Datei mit allen Änderungen vom Repo-Root aus (`SendUserFile`). Müssen Dateien gelöscht werden, lieferst du sie im Paket als leere Hüllen zum Überschreiben.
- **Bei Push-Fehler `access denied … not in this session's authorized repository set`:** Das Repo ist nicht freigegeben. Dann `mcp__claude-code-remote__add_repo` mit `access: "push"` aufrufen. Vorher nicht per curl/gh prüfen, das liefert falsche 404/403.
- **Keine Selbstdeklarationen neu anlegen** (ADR-19/REM-14): kein `ANCHOR[`- oder `FILE-CONTEXT`-Kommentar, keine `contract.rs`, keine `INVENTAR.md`/`AGENTS.md`/`SPEC.md` in `crates/`.
- **Keine erfundenen Zahlen:** Messwerte mit einem Tool oder `grep` belegen, nicht schätzen.
- **Dokumentationssprache (REQ-DOC-2):** Keine Begriffe wie „DSGVO-konform“ oder „unwiderruflich“ in Doku, Oberflächen oder Werbung, außer sie sind an einen konkreten Messwert gebunden. `tools/doc_lint.py` prüft das.
- **Ausgaben nur nach `evidence/`** (gitignored). Nichts in `docs/` oder den Repo-Baum schreiben, was nicht committet werden soll.

## 1. Umgebung (geprüft 21.11.2026)

| Thema | Befund | Konsequenz |
|---|---|---|
| Benutzer | `root` (UID 0). `sudo -n` funktioniert ohne Passwort. | Pakete können direkt installiert werden (Abschnitt 9). |
| Repo-Pfad | Klon unter `/home/claude/contextra` | Alle Befehle aus dem Repo-Root ausführen |
| Python | 3.13 (`python3`). Standardbibliothek reicht für die Tools. | Tools laufen ohne Zusatzpakete |
| Rust-Toolchain | `rust-toolchain.toml` pinnt **1.89.0**. Der Download von `static.rust-lang.org` ist blockiert (`unsuccessful tunnel`). | **Immer `RUSTUP_TOOLCHAIN=stable cargo …`** (stable = 1.97.0). `cargo metadata` ebenso, sonst versucht Cargo den Download. |
| Rust-Komponenten | `clippy`, `rustfmt` für stable installiert. `cargo-deny`, `cargo-audit`, `cargo-nextest`, `cargo-mutants` **nicht** installiert. | Siehe Abschnitt 9 für Installation. |
| Werkzeuge | `rg`, `jq` vorhanden. `shellcheck` und `ruff` nachinstalliert (Abschnitt 9). | Für Suche und Lint nutzen. |
| crates.io | erreichbar (`cargo check`, `cargo install` getestet) | Abhängigkeiten und Cargo-Tools laufen normal |
| apt | `sudo -n apt-get install` funktioniert (getestet mit `shellcheck`) | Systempakete direkt installieren |
| pip | `python3 -m pip install --break-system-packages <pkg>` funktioniert (getestet mit `ruff`) | Python-Werkzeuge installieren |
| npm | 10.9.9 vorhanden | Node-Werkzeuge bei Bedarf |
| GitHub | `git clone` ok, `git push` ohne Freigabe 403 | Abschnitt 7 |
| `gh` | nicht eingeloggt (`GH_TOKEN` abgelehnt) | nicht verwenden |
| Beliebige Hosts | Downloads von anderen Websites werden durch den Proxy blockiert. | Benötigte Dateien vom Nutzer anhängen lassen. |
| Scratchpad | Sitzungsspezifisch, nicht im Repo | Temporäre Dateien dorthin, nicht nach `/tmp` |

## 2. Ist-Zustand (Messung, nicht Soll)

**Build:** `cargo check --workspace` **schlägt fehl**.
- Crate `contextra` (Facade, lib): **20 Fehler** (E0425/E0432/E0433). Ursache: `crates/contextra/src/*.rs` importiert Typen aus `contextra_db`, die dort nicht (mehr) existieren, u. a. `Contextra`, `ContextraConfig`, `Collection`, `SearchResult`, `EmbeddingBackend`, `TextEmbeddingEngine`, `DurabilityMode`, `QueryRewriter`, `chunker`. Betroffen: `agent_memory.rs`, `builder.rs`, `query_rewriter.rs`, `lib.rs`, `performance_profile.rs`.
- Crate `contextra-mcp`: Fehler nur transitiv über die Fassade.
- Crate `contextra-bench` (Bin `scale-100k`): 2 Fehler, Ursache nicht geprüft.
- Alle anderen 19 Crates kompilieren einzeln.

**Schichtprüfung** (`tools/quickcheck.sh layering`, 21.11.2026): **49 Befunde** gegen Anhang E. Verteilung in Abschnitt 8. Die Befunde entsprechen REM-27 (`ports→types`, `store→crypto/erasure/mvcc`, `simd→types`, `db→mvcc/durable-fs/types`, `mcp→crypto/rank/types`, `contextra→types`), dazu R7-Reste (`compat`-Features und Verzeichnisse) und die verbotenen Namen `contextra-types` und `contextra-version`.

**Fehlende Dateien, die die Spec referenziert:**

| Datei | Verweis in | Folge |
|---|---|---|
| `crates/<name>/README.md` | SPEC §3.6 (alle 21 Crates) | **fehlt** (offen, siehe §6 Punkt 7) |
| `crates/<name>/PLAN.md` | SPEC §3.6, §13.5 | **vorhanden, aber veraltet** und nicht mit SPEC v4.1 abgeglichen (offen, siehe §6 Punkt 6). Die Verweise auf „PLAN.md (10×)“ betreffen diese Crate-Dateien, nicht `docs/PLAN.md`. Es gibt kein `docs/PLAN.md` und es wird keins erwartet. |

*Erledigt am 21.11.2026:* `crates/contextra-testkit/tests/layering.rs` (G-03, Ratchet), `.unwrap-baseline.json` (G-13, Ratchet), `layering_baseline.txt` (G-03, Ratchet).

*Neu angelegt am 21.11.2026:* `benchmarks/budgets.toml` (Anhang D) und `capabilities.toml` (Anhang E), beide identisch zur Spec (`check` grün).

**Gate-Status** (`tools/quickcheck.sh gates`, Messung 21.11.2026):

| Gate | Soll | Ist |
|---|---|---|
| G-03 Schichtprüfung (`check_spec_sync.py layering`) | 0 Befunde | **49** |
| G-04 `MAX_SEARCH_K` definiert | 1 | **6** |
| G-08 Dokumentationslint (`doc_lint.py`) | 0 Befunde | **0** |
| G-09 ANCHOR/FILE-CONTEXT-Dateien | 0 | **651** (Selbstdeklarationen noch da) |
| G-09 `contract.rs` | 0 | **9** |
| G-10 `budgets.toml` vorhanden und synchron zu Anhang D | ja | **ja** (`check` grün) |
| G-11 `compat/`-Verzeichnisse | 0 | **7** |
| G-12 `Instant::now`/`SystemTime::now` außerhalb `contextra-env` | 0 | **77** |
| G-13 `unwrap`/`expect`/`panic!` in `src/` (grob) | 0 | **2942** (inkl. Tests; Produktcode separat zählen) |
| G-02 `unsafe` nur in `contextra-sys`, `contextra-simd` | nur diese | zusätzlich `contextra-graph` (1 Datei), `contextra-text` (1 Datei), beide nur in Testdateien |

**Entfernte Crates (dürfen nicht mehr referenziert werden, G-07/Liste 12):** `contextra-engine` (REM-19), `contextra-vault` (REM-01, in `exclude`), `contextra-license` (der Build-Fehler verweist darauf), `contextra-ingest`, `contextra-infer-*`, `contextra-wire`, `contextra-py`. `contextra-engine` steht noch in Kommentaren.

**Crate-Anzahl:** 21 Crates unter `crates/` plus `benchmarks/contextra-bench` = 22 Workspace-Mitglieder. `default-members` in `Cargo.toml` listet 20 davon. 21 Crates haben ein `PLAN.md` (veraltet, siehe oben); `README.md` pro Crate fehlt.

**Code-Umfang** (`python3 tools/loc.py`): ca. 104.000 produktive Zeilen gesamt, 68.187 in den 14 Kern-Crates. Anhang-B-Messung inkl. Tests: 233.961 Zeilen in `crates/`.

## 3. Wo finde ich was

| Pfad | Inhalt |
|---|---|
| `docs/SPEC.md` | **Maßgebliche Spezifikation v4.1 (+ Nachtrag v4.1.1).** §1 Produkt, §2 Entscheidungen, §3 Architektur & erlaubte Abhängigkeiten, §4 Löschung/Versionierung, §5 Datenmodell, §6 RAG-Pipeline, §7 Löschkette, §8 Betrieb, §9 Konfiguration, §10 Leistungsziele, §11 Tests & Gates (§11.6 = Build-Gates G-01…G-14), §12 Entfernungen (Liste 12), §13 Umsetzung, Doku und **§13.6 Werkzeuge und Synchronität (SYNC-1…4)**, §14 Befundauflösung, §15 Crate-Verträge, Anhang B Messungen, Anhang D Budgets, Anhang E Schichtregeln. |
| `docs/archive/` | Alte Spezifikationen (v3, v4.0, Löschspez. v2.2). Nicht normativ. |
| `README.md` | Marketing-/Produkttext auf Deutsch. Nicht technisch maßgeblich. |
| `Cargo.toml` | Workspace: `members` (22), `default-members` (20), `exclude` (vault), `[workspace.dependencies]`, Lints. |
| `rust-toolchain.toml` | Pin 1.89.0 (siehe Abschnitt 1). |
| `capabilities.toml` | Schichtregeln, 1:1 aus Anhang E (`[allowed]`, `[rules]`). Generiert, nicht von Hand ändern. |
| `benchmarks/budgets.toml` | Startwerte, 1:1 aus Anhang D. Generiert, nicht von Hand ändern. |
| `crates/<name>/` | Quellcode; je Crate `src/`, `tests/`, `PLAN.md`, `CLAUDE.md` (Crate-Kontext, Abschnitt 8). |
| `crates/contextra-types`, `-ports`, `-ids`, `-env` | Basistypen, Ports (Traits), IDs, Zeit/Umgebung (einziger Ort für `Instant::now`). |
| `crates/contextra-sys`, `-simd` | Einzige Crates mit erlaubtem `unsafe` (Unsafe-Inseln, G-02). |
| `crates/contextra-store`, `-db`, `-mvcc`, `-erasure`, `-checkpoint` | Persistenz, Transaktionen, Löschkette. `contextra-db` ist die Orchestrierung (Ist 412 Zeilen, Ziel 6.000). |
| `crates/contextra-vector`, `-text`, `-rank`, `-graph`, `-privacy`, `-crypto`, `-durable-fs` | Suche (Vektor/Text/Rang/Graph), Datenschutz, Kryptographie, dauerhafte Dateisysteme. |
| `crates/contextra`, `-mcp` | Öffentliche Fassade (**baut aktuell nicht**) bzw. MCP-Server. |
| `crates/contextra-testkit` | Testhilfen, Verhaltenstests. Dort soll `tests/layering.rs` (G-03) liegen. |
| `tests/unsafe_islands.rs` | G-02: erwartet genau die zwei Unsafe-Inseln (Übergangsausnahmen für vector, store, db, crypto). Importiert `walkdir` (Dev-Dependency-Status nicht geprüft). |
| `benchmarks/` | `contextra-bench` (Workspace-Mitglied). Dessen `PLAN.md`, `SPEC.md`, `AGENTS.md` sind historisch, nicht normativ. `data/` mit `fetch_beir.sh`. |
| `tools/` | Werkzeuge und Bibliothek, siehe Abschnitt 4. |
| `tools/baseline/` | Historisch (Stand 30.10.2026), `run_baseline.py`. Nicht normativ. |
| `evidence/` | Gitignored. Ziel für Tool-Ausgaben. |
| `.cargo/` | `config.toml` enthält nur Kommentare (xtask-Alias entfernt). `audit.toml`, `mutants.toml` vorhanden. |
| `deny.toml` | `cargo deny`-Regeln. |

## 4. Tools in `tools/`

Der Bestand ist bewusst klein. Entfernt wurden am 21.11.2026 24 Auditor- und Generator-Skripte, die die Spec in REM-14 zur Entfernung vorsieht. Sie sind über die Git-Historie erreichbar: `git show 0ec4de767:tools/<name>.py`.

**Einstieg:** `tools/quickcheck.sh <Befehl>`

| Befehl | Was er tut | Typische Dauer |
|---|---|---|
| `build` | `cargo check --workspace --keep-going` mit stable | Sekunden bis Minuten |
| `sync` | Anhänge D/E der Spec gegen `budgets.toml`/`capabilities.toml` vergleichen | < 1 s |
| `layering` | Cargo-Abhängigkeiten gegen Anhang E, Ratchet gegen `layering_baseline.txt` (nur neue Befunde sind Fehler) | < 10 s |
| `ratchet` | Unwrap/Panic-Zähler je Crate gegen `.unwrap-baseline.json` (G-13, R6; Testdateien ausgenommen) | < 5 s |
| `deps` | `cargo deny` (bans, licenses, sources) und G-05 (Netzwerk-Crates in E1-Crates) | 10–60 s |
| `gates` | Build-Gates aus SPEC §11.6 per grep mit Soll/Ist | < 5 s |
| `lint` | `doc_lint` (G-08) und LOC je Crate | < 5 s |
| `coverage` | `test_coverage_matrix` → `evidence/coverage/` | < 30 s |
| `all` | Alles oben in dieser Reihenfolge | Minuten |

| Datei | Zweck | Direktaufruf |
|---|---|---|
| `quickcheck.sh` | Einstieg, Wrapper für alle Prüfungen. Schreibt nur nach `evidence/`. | `tools/quickcheck.sh <Befehl>` |
| `check_spec_sync.py` | SYNC: Anhänge der Spec gegen TOML-Dateien (`check`, `sync`) und Schichtprüfung (`layering`). Exit 1 bei Abweichung. | `python3 tools/check_spec_sync.py {check\|sync\|layering\|all}` |
| `unwrap_ratchet.py` | G-13 / R6: zählt `unwrap()`, `expect(`, `panic!(`, `todo!(`, `unimplemented!(`, `unreachable!(` je Crate in Produktcode. `--update` schreibt die Baseline neu (nur nach bewusstem Abbau), `--detail` listet Zeilen. | `python3 tools/unwrap_ratchet.py [--detail\|--update]` |
| `doc_lint.py` | G-08 / REQ-DOC-2: verbotene Compliance-Formulierungen. Zitierte oder verneinte Begriffe werden nicht gemeldet. | `python3 tools/doc_lint.py --root . [--json]` |
| `loc.py` | Produktive LOC je Crate (ohne Kommentare und Leerzeilen). | `python3 tools/loc.py [--crate X] [--json]` |
| `test_coverage_matrix.py` | Test-Matrix: `#[test]`, `proptest`, Traceability-Marker `T-…`, Lücken. | `python3 tools/test_coverage_matrix.py --workspace . --output-dir <dir>` |
| `agent_tooling_common.py` | Bibliothek für `test_coverage_matrix.py`. Kein eigenständiges Werkzeug. | — |
| `baseline/run_baseline.py` | Voller Build-/Test-Lauf je Crate mit Logs. Historisch, lange Laufzeit. | `--self-test` läuft grün. Voller Lauf braucht ein gültiges Workspace-Build (aktuell rot). |

**Abläufe (so arbeiten, damit Sitzungen synchron bleiben):**

1. **Sitzungsstart:** `tools/quickcheck.sh sync layering build`. Ergebnisse dem Nutzer knapp nennen.
2. **Spec ändern:** Anhang D oder E in `docs/SPEC.md` ändern → `python3 tools/check_spec_sync.py sync` → `tools/quickcheck.sh sync` muss grün sein → Spec und TOML im selben Commit.
3. **Crate umbauen (z. B. REM-27):** Vorher `tools/quickcheck.sh layering` (Befundliste), nach dem Umbau erneut. Ziel: weniger Befunde, kein neuer.
4. **Vor jedem Commit:** `tools/quickcheck.sh sync lint`. Bei Änderungen an Gates oder Crate-Struktur zusätzlich `gates`.
5. **Neue Prüfung nötig:** zuerst prüfen, ob sie in `quickcheck.sh` als Befehl passt. Eigenständige Skripte nur, wenn SYNC-3 es erlaubt.

**Hinweise:**
- `doc_lint.py` meldet 0 Befunde, nachdem Fehlalarme in zitierten und verneinten Formulierungen gefiltert wurden. Der Filter ist eine Heuristik: Eine Formulierung wie „ist nicht DSGVO-konform, aber …“ würde durchrutschen. Bei Zweifel im Text prüfen.
- `check_spec_sync.py layering` prüft nur normale Abhängigkeiten, keine dev-dependencies (REM-27: „außerhalb von dev-dependencies“). Optionale Features zählen mit.
- Vor Löschungen oder Wiederherstellungen aus der Historie: REM-14 in `docs/SPEC.md` lesen.

## 5. Bekannte Stolpersteine

1. **Toolchain 1.89.0 nicht ladbar** → `RUSTUP_TOOLCHAIN=stable` setzen, auch bei `cargo metadata` und `cargo install`. Nicht versuchen, `static.rust-lang.org` per curl zu erreichen.
2. **Facade `contextra` baut nicht.** Vor Arbeit an `contextra-db`-Schnittstellen zuerst prüfen, ob `contextra` darauf zugreift.
3. **Anhänge sind Quelle.** `budgets.toml` und `capabilities.toml` nie von Hand ändern. Abweichungen zeigt `check`.
4. **Zahlen aus der Spec sind Soll-Werte.** Anhang B und diese Datei sind Messungen vom 20./21.11.2026.
5. **Historische Dokumente** (`tools/baseline/README.md`, `benchmarks/contextra-bench/PLAN.md|SPEC.md|AGENTS.md`, `docs/archive/`) sind nicht normativ und können auf entfernte Tools verweisen.
6. **`ruff` und `shellcheck` sind nicht überall sauber.** Die Legacy-Tools `agent_tooling_common.py`, `test_coverage_matrix.py` und `doc_lint.py` haben Stilbefunde. Neue Dateien sauber halten, Altbestand nur anfassen, wenn die Aufgabe es verlangt.

## 6. Empfohlene Reihenfolge für die nächste Sitzung (Stand 21.11.2026, Sitzung 3)

1. `tools/quickcheck.sh sync layering build` ausführen. Das ist der Ist-Stand.
2. Mit dem Nutzer klären, ob der Umbau der Facade `contextra` (Anpassung an `contextra-db`) Priorität hat, oder ob Stub/Feature-Gating gewünscht ist.
3. **REM-27 abarbeiten** (Ziel: `check_spec_sync.py layering` ohne Befunde). Reihenfolge nach Abhängigkeiten:
   - R7 zuerst, da mechanisch: `compat`-Features und `src/compat/` in `simd`, `text`, `rank`, `vector`, `graph`, `checkpoint`, `mvcc` entfernen (CMP-Auflösung, Spec 12.5). Ergebnis ist ohne Fachentscheidung prüfbar.
   - `ports → env/types` und `ports → types`-Abhängigkeit auflösen (R1, Spec 3.3).
   - `simd → types` entfernen (Spec 3.1).
   - `store → crypto/erasure/mvcc` entfernen (R2, REM-27). Das betrifft die Schlüsselvernichtung über Ports (ADR-32).
   - `db`, `mcp`, `contextra` anpassen, sobald ihre Ziel-Abhängigkeiten stehen.
4. **Freigabe einholen:** `contextra-version` → `contextra-version` (ADR-06) und REM-30 (`contextra-types` auflösen). Beides ändert viele `Cargo.toml`. Nicht eigenmächtig.
5. **Sitzung 3 (21.11.2026): nach SPEC wiederhergestellt.** Die SPEC verlangt `check_spec_sync.py` (SYNC-1/2/3), `budgets.toml`, `capabilities.toml` und `tests/layering.rs` (G-03). Umgesetzt: `tools/check_spec_sync.py` mit `check`, `sync`, `layering [--ratchet|--update-baseline]` (nur Standardbibliothek); `benchmarks/budgets.toml` und `capabilities.toml` per `sync` 1:1 aus Anhang D/E; `layering_baseline.txt` mit 56 Befunden (Zählung inkl. optionaler Abhängigkeiten, wie SYNC-2 es verlangt; CLAUDE.md alt: 49); `crates/contextra-testkit/tests/layering.rs` (grün); `quickcheck.sh` mit `sync` und `layering`. **Erledigt (Sitzung 3):** `tools/unwrap_ratchet.py` und `.unwrap-baseline.json` (SYNC-5b, G-13). Messung: 19 Panik-Stellen im Produktcode, alle in `contextra-graph`. Dazu gehört `pub mod conformance` in `crates/contextra-graph/src/storage.rs`. Es ist kein `#[cfg(test)]` und gehört in die Testkit-Crate (Soll). Die Gate-Tabelle oben nennt 2942, das war eine grobe Zählung mit Tests und ist nicht der Gate-Wert.
6. **Erledigt 21.11.2026: `crates/<name>/PLAN.md` gegen SPEC v4.1 abgeglichen (21 Crates, Vorlage 13.5).** Je Crate ein Commit `docs(<name>): PLAN.md auf SPEC v4.1 abgeglichen`. Ist-Abweichungen stehen in Abschnitt 3 bzw. 4, offene Entscheidungen unter „Offene Punkte (Entscheidung nötig)“ in Abschnitt 6 der jeweiligen PLAN.md. Ausnahme: `contextra-version` enthält nur den Status, die Umbenennung wartet auf Freigabe (Punkt 4). Die Qualität ist uneinheitlich: `contextra-types` ist ein Auflösungsplan, `sys`, `simd`, `durable-fs` und `crypto` sind kürzer als der Rest. Eine fachliche Durchsicht mit dem Nutzer steht aus.
7. **Erledigt 21.11.2026: `README.md` je Crate nach SPEC 3.6** (21 Crates plus `benchmarks/contextra-bench`). Zweck aus der Rollenzeile der Crate-`CLAUDE.md`, öffentliche API wörtlich aus `src/lib.rs` (Ist-Stand, kann laut PLAN entfallende Module enthalten), Features aus `Cargo.toml`. Die bestehende README des Benchmark-Crates war als historisch markiert und wurde ersetzt (Git-Historie). Commits je Gruppe.
8. **Erledigt 21.11.2026: SYNC-5 als Notiz in §13.6** (`docs(spec): SYNC-5 …`). Keine Norm geändert, `check_spec_sync.py check` grün.
9. Die übrigen Entfernungen aus §12 (ANCHOR, FILE-CONTEXT, `contract.rs`) sind eine eigene Aufgabe. Erst nach Freigabe durch den Nutzer.
10. **Entschieden in Sitzung 3 (21.11.2026) nach SPEC:** Die SPEC enthält kein tokio und kein REQ-ASYNC-1. Anhang E prüft nur normale Abhängigkeiten, und zu Dev-Abhängigkeiten heißt es „nur auf contextra-testkit geprüft“. G-05 sperrt Netzwerk-Crates, tokio gehört nicht dazu. Die Dev-Abhängigkeit von `contextra-erasure` ist daher spec-konform. Umgesetzt: `contextra-erasure` als Wrapper in `deny.toml` mit Kommentar. Am Code von erasure nichts geändert. Die übrigen TRANSITION-Verweise auf REQ-ASYNC-1 in `deny.toml` sind ohne Norm und sollten bei Gelegenheit bereinigt werden (nicht Teil dieser Entscheidung). Befund aus Sitzung 2, unverändert bestätigt: `cargo tree -p contextra-erasure -e normal -i tokio` ist leer, `-e dev` zeigt tokio 1.53.1 unter `[dev-dependencies]`. REQ-ASYNC-1 ist in der SPEC nicht vorhanden. Zwei Optionen für den Nutzer: (a) Dev-Abhängigkeit entfernen, Tests umschreiben. (b) Wrapper `contextra-erasure` mit TRANSITION-Vermerk in `deny.toml`. Nicht geändert, bis entschieden. Befund korrigiert: `tokio` 1.53.1 ist bei `contextra-erasure` eine **Dev-Abhängigkeit** (`Cargo.toml`, Zeile 39, Abschnitt `[dev-dependencies]`), keine normale. Der normale Abhängigkeitsbaum enthält `contextra-erasure` nicht (`cargo tree -e normal`). `deny.toml` erfasst die Dev-Abhängigkeit trotzdem, und für `contextra-erasure` steht dort kein Wrapper. Normale tokio-Nutzung gibt es zusätzlich in `contextra-privacy`, `contextra-checkpoint`, `contextra-store` und `contextra`, dort jeweils mit Wrapper-Eintrag. Die Begründung „REQ-ASYNC-1“ steht in `deny.toml`, ist in SPEC v4.1 aber nicht zu finden. Optionen für den Nutzer (siehe PLAN von `contextra-erasure`, O-2): (a) Dev-Abhängigkeit entfernen und die Tests umschreiben, (b) Wrapper für `contextra-erasure` mit TRANSITION-Vermerk in `deny.toml`. Die Duplikat- und Lizenzwarnungen aus dem früheren Befund sind nicht erneut gemessen. **Entscheidung ausstehend.** Sobald entschieden, hier festhalten.

## 7. Push-Workflow (verifiziert 21.11.2026)

Ablauf, der funktioniert hat. Bitte in dieser Reihenfolge abarbeiten.

1. **Klon prüfen, nicht blind neu klonen.**
   `git -C /home/claude/contextra rev-parse HEAD`. Wenn das klappt, ist der Klon da und wird weiterverwendet. Ein Neu-Klon würde lokale Commits löschen.
   Nur wenn der Klon fehlt: `git clone --depth 1 https://github.com/tfufuz1/contextra /home/claude/contextra`, mit langem Timeout (ca. 10 min). Ein Standard-Timeout von 2 min bricht den Pack-Download ab. Der Proxy erlaubt max. 2 parallele Git-Operationen pro Repo, bei 429 einmal nach 5 s wiederholen.
2. **Freigabe prüfen.** Bei `access denied by the git proxy: … not in this session's authorized repository set` (HTTP 403) das Repo freigeben:
   `mcp__claude-code-remote__add_repo` mit `owner: "tfufuz1"`, `repo: "contextra"`, `access: "push"`.
   Die Antwort kann „Clone it NOW“ und „register_repo_root“ enthalten. Den Klon nur dann neu anlegen, wenn Schritt 1 fehlschlägt. `register_repo_root` war in dieser Sitzung nicht verfügbar. Dann CLAUDE.md direkt lesen.
3. **Commit mit Trailer** (Abschnitt 0), dann pushen:
   ```bash
   cd /home/claude/contextra
   git add <Dateien>            # nicht blind -A, wenn fremde Änderungen im Arbeitsbaum liegen
   git -c user.name="Claude" -c user.email="claude@anthropic.invalid" commit -m "<Betreff>

   Co-Authored-By: Claude Haiku 5.5 <noreply@anthropic.com>
   Claude-Session: https://claude.ai/code/session_01BByRVn81CFexSvyN4AheTv"
   git push -u origin HEAD
   ```
   Erfolgsmeldung im Test: `591e07e90..d554c5e39 HEAD -> main`.
4. **Fehlerfälle:**
   - `HTTP 413` (Pack zu groß, Klon ist flach): `git -c push.negotiate=true push -u origin HEAD`.
   - Immer noch abgelehnt: `git fetch --depth=50 origin main`, dann `git rebase --onto origin/main <Start-Commit>`. Nicht ohne Tiefe fetchen, das lädt Gigabytes alter Historie.
   - Tiefer-Klon prüfen: `git rev-parse --is-shallow-repository`.
   - Ein Hook verweigert den Fetch: neues Verzeichnis anlegen, dort Patch per `git format-patch --stdout <Start> | git am` einspielen und von dort pushen.
   - `push negotiation failed`-Warnung allein bedeutet nicht, dass der Push fehlgeschlagen ist.
5. **Fallback ohne Push:** ZIP der Änderungen vom Repo-Root per `SendUserFile` (Abschnitt 0).

**Hinweis:** Auf `main` liegt ein Commit `591e07e90 Shell-Commit`, den nicht diese Sitzung erzeugt hat. Vor Änderungen am Branch-Verlauf nicht umschreiben.

## 8. Crate-Sitzungen und Konfliktvermeidung

Jedes Crate hat eine eigene `CLAUDE.md`. Zweck: Eine Sitzung kann auf ein Crate beschränkt werden, ohne den ganzen Workspace zu laden. Die Spalte „Schicht-Befunde“ stammt aus `check_spec_sync.py layering` (21.11.2026, Summe 49).

| Crate | Pfad | Ebene | Build | Schicht-Befunde |
|---|---|---|---|---|
| contextra-ids | `crates/contextra-ids/CLAUDE.md` | E0 | ok | 0 |
| contextra-env | `crates/contextra-env/CLAUDE.md` | E0 | ok | 0 |
| contextra-sys | `crates/contextra-sys/CLAUDE.md` | E0 (unsafe) | ok | 0 |
| contextra-simd | `crates/contextra-simd/CLAUDE.md` | E0 (unsafe) | ok | 4 |
| contextra-durable-fs | `crates/contextra-durable-fs/CLAUDE.md` | E0 | ok | 0 |
| contextra-ports | `crates/contextra-ports/CLAUDE.md` | E0 | ok | 2 |
| contextra-crypto | `crates/contextra-crypto/CLAUDE.md` | E1 | ok | 0 |
| contextra-erasure | `crates/contextra-erasure/CLAUDE.md` | E1 (sicherheitskritisch) | ok | 0 |
| contextra-version | `crates/contextra-version/CLAUDE.md` | E1 (→ `version`) | ok | 5 |
| contextra-store | `crates/contextra-store/CLAUDE.md` | E1 | ok | 6 |
| contextra-vector | `crates/contextra-vector/CLAUDE.md` | E1 | ok | 4 |
| contextra-text | `crates/contextra-text/CLAUDE.md` | E1 | ok | 5 |
| contextra-graph | `crates/contextra-graph/CLAUDE.md` | E1, optional | ok | 5 |
| contextra-rank | `crates/contextra-rank/CLAUDE.md` | E1 | ok | 5 |
| contextra-privacy | `crates/contextra-privacy/CLAUDE.md` | E1 | ok | 0 |
| contextra-checkpoint | `crates/contextra-checkpoint/CLAUDE.md` | E1 | ok | 5 |
| contextra-types | `crates/contextra-types/CLAUDE.md` | (REM-30) | ok | 1 |
| contextra-testkit | `crates/contextra-testkit/CLAUDE.md` | E3, dev | ok | nicht in Anhang E |
| contextra-db | `crates/contextra-db/CLAUDE.md` | E2 | ok (dünn, 412 Zeilen) | 3 |
| contextra | `crates/contextra/CLAUDE.md` | E2 | **FEHLER** (20) | 1 |
| contextra-mcp | `crates/contextra-mcp/CLAUDE.md` | E3 | **FEHLER** (transitiv) | 3 |
| contextra-bench | `benchmarks/contextra-bench/CLAUDE.md` | E3, bench | **FEHLER** (2) | nicht in Anhang E |

**Arbeitsregeln für parallele Sitzungen:**

1. **Sitzung = ein Crate.** Startpunkt ist die Crate-`CLAUDE.md`, danach die Root-`CLAUDE.md` für übergreifende Regeln. Schreibzugriff nur auf das eigene Crate.
2. **Notizen nur in „Sitzungsnotizen“ am Ende der eigenen Crate-Datei anhängen.** Nicht überschreiben, nicht in fremde Dateien schreiben. Format: `### YYYY-MM-DD · Branch · Kurzfassung · Offen: …`.
3. **Root-`CLAUDE.md` nur durch die Koordinations-Sitzung ändern** (also durch den Nutzer bzw. eine dafür beauftragte Sitzung).
4. **Schnittstellenänderungen** (Ports, Typen aus `ids`/`types`, `Clock`/`SeededRng` in `env`): In der eigenen Notiz unter „Offen:“ beschreiben und die betroffenen Dependents nennen (siehe Eckdaten in deren Datei). Fremde Crates nicht selbst anpassen, sondern dort als offen eintragen.
5. **Branch pro Sitzung** (Konvention, noch nicht mit dem Nutzer bestätigt): `crate/<name>/<kurzthema>`, von `main` abzweigen, nur dieses Branch pushen. So kollidieren parallele Sitzungen nicht auf `main`. Vor dem Merge den Nutzer fragen.
6. **Datei-Eigentum:** `contextra`, `contextra-db`, `contextra-mcp`, `contextra-bench` hängen am Build-Fehler. Solange er nicht behoben ist, nur daran arbeiten, wenn die Aufgabe genau das ist.
7. **Schicht-Befunde nur im eigenen Crate beheben.** Eine Abhängigkeit, die auf ein anderes Crate zeigt, wird dort als „Offen“ vermerkt, nicht dort entfernt.

**Bekannte Zuständigkeitsprobleme** (keine Sitzung darf sie still lösen):
- `contextra-types` (REM-30) muss aufgelöst werden. Betrifft fast alle Crates.
- `contextra-version` → `contextra-version` (ADR-06). Betrifft alle Dependents in `Cargo.toml`.

## 9. Effizient arbeiten: Shell, Installation, Abläufe

**Shell-Regeln:**
- Suche mit `rg` (ripgrep), nicht mit `grep -r`. Schneller und ignoriert `.gitignore`.
- JSON-Reports mit `jq` lesen, nicht die ganze Datei ausgeben.
- Cargo immer mit `-p <crate>` ausführen, wenn nur ein Crate betroffen ist. Das spart Zeit und Ausgabe.
- Lange Befehle (`cargo install`, voller `cargo test`) im Hintergrund starten und das Log im Scratchpad lesen:
  ```bash
  S=/tmp/claude-0/-home-claude/e9233f87-4814-5de6-b5ed-2c8d26062fb4/scratchpad
  mkdir -p "$S" && RUSTUP_TOOLCHAIN=stable nohup timeout 900 cargo install --locked cargo-deny > "$S/deny.log" 2>&1 &
  tail -n 3 "$S/deny.log"
  ```
- Lint-Stand per Befehl: `shellcheck tools/quickcheck.sh` und `ruff check <datei>`. Mit `LANG=C.UTF-8` setzen, sonst meldet shellcheck Locale-Fehler.

**Fehlende Abhängigkeiten installieren (geprüft):**

| Was | Befehl | Status 21.11.2026 |
|---|---|---|
| Systempaket (apt) | `sudo -n apt-get update -qq && sudo -n apt-get install -y --no-install-recommends <paket>` | getestet mit `shellcheck` |
| Python-Werkzeug | `python3 -m pip install --break-system-packages <paket>` | getestet mit `ruff` (Binary liegt in `/root/.local/bin`; `export PATH="$HOME/.local/bin:$PATH"`) |
| Cargo-Werkzeug | `RUSTUP_TOOLCHAIN=stable cargo install --locked <tool>` | dauert ca. 5–10 min, im Hintergrund starten. `cargo-deny` v0.22.0 wurde am 21.11.2026 so installiert (`/root/.cargo/bin/cargo-deny`). Noch nicht installiert: `cargo-audit`, `cargo-nextest`, `cargo-mutants`. |
| Node-Werkzeug | `npm install -g <paket>` | npm 10.9.9 vorhanden, nicht getestet |

**Typische Befehle (sofort einsetzbar):**
```bash
tools/quickcheck.sh sync layering      # Sync und Schichten
tools/quickcheck.sh build              # Workspace bauen
RUSTUP_TOOLCHAIN=stable cargo test -p contextra-env          # Ein Crate testen
RUSTUP_TOOLCHAIN=stable cargo clippy -p contextra-env -- -D warnings
rg -n "Instant::now|SystemTime::now" crates --glob '*.rs' | rg -v 'contextra-env/'   # G-12 Fundstellen
python3 tools/loc.py --crate contextra-store
```

**Was nicht geht oder nicht erwünscht ist:**
- Downloads von beliebigen Websites (Proxy blockiert, z. B. `static.rust-lang.org`). Dateien vom Nutzer anfordern.
- Tool-Ausgaben in `docs/` oder in den Repo-Baum schreiben.
- Dateien in `benchmarks/budgets.toml` oder `capabilities.toml` von Hand ändern.
- Neue Auditor-Skripte in `tools/` anlegen (SYNC-3, REM-14).
