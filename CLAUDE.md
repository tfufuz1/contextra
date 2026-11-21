# CLAUDE.md — Einstieg für Claude-Sitzungen im Contextra-Repo

Stand: 21.11.2026 · Repo-HEAD beim Audit: `2fd0994` · Maßgebliche Spezifikation: `docs/SPEC.md` (v4.1, **Soll-Zustand**)

Das Projekt ist mitten im Umbau. Vieles in `docs/SPEC.md` beschreibt das Ziel, nicht den Ist-Stand. Vor jeder Änderung: `docs/SPEC.md` Abschnitt betreffend lesen und Anhang B (Messungen) mit dem Ist vergleichen.

---

## 0. Arbeitsregeln (verbindlich)

- **Subagenten nur als Haiku.** Beim `Agent`-Tool immer `model: "haiku"` setzen. Sonnet nie verwenden. Opus/Fable nur, wenn der Nutzer es ausdrücklich verlangt.
- **Sprache:** Antworten an den Nutzer und Kommentare/Doku im Repo auf Deutsch (Bestand ist deutsch; Rust-Bezeichner englisch).
- **Push:** Nur über den GitHub-Zugang der Sitzung. Der Ablauf steht in Abschnitt 7 und hat am 21.11.2026 funktioniert.
- **Commit-Trailer (Pflicht ab jetzt):** Jede Commit-Message endet mit
  ```
  Co-Authored-By: Claude Haiku 5.5 <noreply@anthropic.com>
  Claude-Session: https://claude.ai/code/session_01BByRVn81CFexSvyN4AheTv
  ```
  Pull-Request-Beschreibungen enden mit `🤖 Generated with [Claude Code](https://claude.com/claude-code)` und dem Session-Link.
- **Fallback ZIP:** Ist kein Push möglich, liefert du dem Nutzer im Chat eine ZIP-Datei mit allen Änderungen vom Repo-Root aus (`SendUserFile`). Müssen Dateien gelöscht werden, lieferst du sie im Paket als leere Hüllen zum Überschreiben.
- **Bei Push-Fehler `access denied … not in this session's authorized repository set`:** Das Repo ist nicht freigegeben. Dann `mcp__claude-code-remote__add_repo` mit `access: "push"` aufrufen (nicht vorher per curl/gh prüfen, das liefert falsche 404/403).
- **Keine Selbstdeklarationen neu anlegen** (ADR-19/REM-14): kein `ANCHOR[`-, `FILE-CONTEXT`-Kommentar, keine `contract.rs`, keine `INVENTAR.md`/`AGENTS.md`/`SPEC.md` in `crates/`.
- **Keine erfundenen Zahlen:** Messwerte mit einem Tool oder `grep` belegen, nicht schätzen.
- **Dokumentationssprache (REQ-DOC-2):** Keine Begriffe wie „DSGVO-konform“ oder „unwiderruflich“ in Doku, Oberflächen oder Werbung, außer sie sind an einen konkreten Messwert gebunden. `tools/doc_lint.py` meldet sie.

## 1. Umgebung (getestet 21.11.2026)

| Thema | Befund | Konsequenz |
|---|---|---|
| Repo-Pfad | Klon unter `/home/claude/contextra` | Alle Befehle aus dem Repo-Root ausführen |
| Python | 3.13 (`python3`), nur Standardbibliothek nötig | Tools laufen ohne `pip install` |
| Rust-Toolchain | `rust-toolchain.toml` pinnt **1.89.0**. Der Download von `static.rust-lang.org` ist im Sandbox blockiert (`unsuccessful tunnel`). | **Immer `RUSTUP_TOOLCHAIN=stable cargo …` verwenden** (stable = 1.97.0, funktioniert). Alternativ `tools/quickcheck.sh build`. |
| crates.io | erreichbar | Abhängigkeiten laufen normal |
| GitHub | `git clone` ok, `git push` ohne Freigabe 403 | siehe Abschnitt 0 |
| `gh` | nicht eingeloggt (`GH_TOKEN` abgelehnt) | nicht verwenden |
| Scratchpad | Sitzungsspezifisch, ist nicht im Repo | Temporäre Dateien dorthin, nicht nach `/tmp` |
| Ausgaben | `evidence/` ist in `.gitignore` | Tool-Ausgaben nur dorthin oder ins Scratchpad |

**Falle:** Mehrere Tools schreiben standardmäßig nach `docs/diagnostics/` (z. B. `orchestrate_diagnostics.py`, Default `--output-dir docs/diagnostics`). Immer `--output-dir` setzen, sonst verschmutzt das Repo.

## 2. Ist-Zustand (Messung, nicht Soll)

**Build:** `cargo check --workspace` **schlägt fehl**.
- Crate `contextra` (Facade, lib): **20 Fehler** (E0425/E0432/E0433). Ursache: `crates/contextra/src/*.rs` importiert Typen aus `contextra_db`, die dort nicht (mehr) existieren, u. a. `Contextra`, `ContextraConfig`, `Collection`, `SearchResult`, `EmbeddingBackend`, `TextEmbeddingEngine`, `DurabilityMode`, `QueryRewriter`, `chunker`. Betroffen: `agent_memory.rs`, `builder.rs`, `query_rewriter.rs`, `lib.rs`, `performance_profile.rs`.
- Crate `contextra-bench` (Bin `scale-100k`): 1 Fehler, Ursache nicht geprüft.
- Alle anderen Crates kompilieren (`cargo check -p contextra-ids -p contextra-types` ok). Warnungen gibt es viele (u. a. unused imports, fehlende Doku).

**Fehlende Dateien, die die Spec referenziert:**

| Datei | Verweis in | Folge |
|---|---|---|
| `capabilities.toml` (Schema v3) | SPEC §2, G1 | `tools/gate_dependencies.py` meldet FAIL; `agent_tooling_common.py` liest es |
| `benchmarks/budgets.toml` | SPEC G-10, Anhang D | fehlt |
| `crates/contextra-testkit/tests/layering.rs` | SPEC G-03 | fehlt |
| `docs/PLAN.md` | SPEC (10×) | fehlt, nur `docs/SPEC.md` und `docs/archive/` existieren |
| `.unwrap-baseline.json` | `.gitignore` | fehlt |
| `tests/test_gate_g8_negative.py` | — | Import `gate_g1_layering` existiert nicht → Test bricht bei Collection ab |

**Gate-Status** (`tools/quickcheck.sh gates`, Messung 21.11.2026):

| Gate | Soll | Ist |
|---|---|---|
| G-04 `MAX_SEARCH_K` definiert | 1 | **6** |
| G-09 ANCHOR/FILE-CONTEXT-Dateien | 0 | **651** (Selbstdeklarationen noch da) |
| G-09 `contract.rs` | 0 | **9** |
| G-11 `compat/`-Verzeichnisse | 0 | **7** |
| G-12 `Instant::now`/`SystemTime::now` außerhalb `contextra-env` | 0 | **77** |
| G-13 `unwrap`/`expect`/`panic!` in `src/` (grob) | 0 | **2942** (inkl. Tests; Produktcode separat zählen) |
| G-02 `unsafe` nur in `contextra-sys`, `contextra-simd` | nur diese | zusätzlich `contextra-graph` (1 Datei), `contextra-text` (1 Datei) |
| G-10 / G-03 / G1 | Datei vorhanden | **fehlt** (siehe oben) |

**Entfernte Crates (dürfen nicht mehr referenziert werden, G-07/Liste 12):** `contextra-engine` (REM-19), `contextra-vault` (REM-01, in `exclude`), `contextra-license` (der Build-Fehler verweist darauf), `contextra-ingest`, `contextra-infer-*`, `contextra-wire`, `contextra-py`. `contextra-engine` steht noch in Kommentaren und `tools/crate_inventory.py`, `tools/loc.py`, `tools/generate_interface_spec.py`.

**Crate-Anzahl:** 21 Crates unter `crates/` plus `benchmarks/contextra-bench` = 22 Workspace-Mitglieder. `default-members` in `Cargo.toml` listet 20 davon. 21 Crates haben ein `PLAN.md`; `README.md` pro Crate fehlt.

**Code-Umfang** (`python3 tools/loc.py`): ca. 104.000 produktive Zeilen gesamt, 68.187 in den 14 Kern-Crates. Anhang-B-Messung inkl. Tests: 233.961 Zeilen in `crates/`.

## 3. Wo finde ich was

| Pfad | Inhalt |
|---|---|
| `docs/SPEC.md` | **Maßgebliche Spezifikation v4.1.** §1 Produkt, §2 Entscheidungen, §3 Architektur & erlaubte Abhängigkeiten, §4 Löschung/Versionierung, §5 Datenmodell, §6 RAG-Pipeline, §7 Löschkette, §8 Betrieb, §9 Konfiguration, §10 Leistungsziele, §11 Tests & Gates (§11.6 = Build-Gates G-01…G-14), §12 Entfernungen (Liste 12), §13 Umsetzung/Doku, §15 Crate-Verträge, Anhang B Messungen, Anhang E Schichtregeln. |
| `docs/archive/` | Alte Spezifikationen (v3, v4.0, Löschspez. v2.2). Nicht normativ. |
| `README.md` | Marketing-/Produkttext auf Deutsch. Nicht technisch maßgeblich. |
| `Cargo.toml` | Workspace: `members` (22), `default-members` (20), `exclude` (vault), `[workspace.dependencies]`, Lints. |
| `rust-toolchain.toml` | Pin 1.89.0 (siehe Umgebung). |
| `crates/<name>/` | Quellcode; je Crate `src/`, `tests/`, `PLAN.md`. |
| `crates/contextra-types`, `-ports`, `-ids`, `-env` | Basistypen, Ports (Traits), IDs, Zeit/Umgebung (einziger Ort für `Instant::now`). |
| `crates/contextra-sys`, `-simd` | Einzige Crates mit erlaubtem `unsafe` (Unsafe-Inseln, G-02). |
| `crates/contextra-store`, `-db`, `-mvcc`, `-erasure`, `-checkpoint` | Persistenz, Transaktionen, Löschkette. `contextra-db` ist die Fassade der Persistenz (Ursache der Build-Fehler). |
| `crates/contextra-vector`, `-text`, `-rank`, `-graph`, `-privacy`, `-crypto`, `-durable-fs` | Suche (Vektor/Text/Rang/Graph), Datenschutz, Kryptographie, dauerhafte Dateisysteme. |
| `crates/contextra`, `-mcp` | Öffentliche Fassade (**baut aktuell nicht**) bzw. MCP-Server. |
| `crates/contextra-testkit` | Testhilfen, Verhaltenstests. Dort soll `tests/layering.rs` (G-03) liegen. |
| `tests/unsafe_islands.rs` | G-02: erwartet genau die zwei Unsafe-Inseln (Übergangsausnahmen für vector, store, db, crypto). Importiert `walkdir` (Dev-Dependency-Status nicht geprüft). |
| `tests/test_gate_g8_negative.py` | Negativtest G1 — derzeit defekt (siehe oben). |
| `benchmarks/` | `contextra-bench` (Workspace-Mitglied, eigene PLAN/SPEC/AGENTS.md — historisch), `competitive_bench.rs`, `data/`. |
| `tools/` | Analyse-Skripte (siehe Abschnitt 4). |
| `tools/baseline/` | Historisch (Stand 30.10.2026), `run_baseline.py`. Nicht normativ. |
| `evidence/` | Gitignored. Ziel für Tool-Ausgaben. |
| `.cargo/` | `config.toml` enthält nur Kommentare (xtask-Alias entfernt). `audit.toml`, `mutants.toml` vorhanden. |
| `deny.toml` | `cargo deny`-Regeln. |

## 4. Tools in `tools/`

**Schnellstart:**
```bash
tools/quickcheck.sh build        # cargo check (stable-Toolchain)
tools/quickcheck.sh gates        # Build-Gates per grep, mit Soll/Ist
tools/quickcheck.sh diagnostics  # lauffähige Analysen -> evidence/diagnostics/
tools/quickcheck.sh all
```

**Verifiziert lauffähig** (mit `--workspace . --output-dir <dir>`; Stand 21.11.2026, alle rc=0):
`async_boundary_scanner`, `concurrency_lock_auditor`, `contract_diff_detector`, `dead_code_microprobe`, `determinism_guard`, `erasure_leak_prober`, `error_topology_mapper`, `fsync_durability_tracer`, `pattern_diagnostic`, `port_contract_auditor`, `rust_ast_inspector`, `test_coverage_matrix`, `test_matrix_advisor`, `type_registry_sync`, `unsafe_boundary_tracer`, `zero_panic_analyzer`, `agentic_context_pack` (vorher KeyError auf `contextra-engine`, behoben), `orchestrate_diagnostics`.

**Eigene CLI** (nicht `--workspace`/`--output-dir`):

| Tool | Aufruf | Hinweis |
|---|---|---|
| `loc.py` | `python3 tools/loc.py [--crate X] [--json]` | produktive LOC je Crate |
| `crate_inventory.py` | `python3 tools/crate_inventory.py --root . overview` | Subcommands: `overview, crate, matrix, hubs, cycles, json, agents, check` |
| `feature_audit.py` | `python3 tools/feature_audit.py --root .` | `--fail-on-unused` für CI |
| `doc_lint.py` | `python3 tools/doc_lint.py --root . [--json]` | G-08. **Fehlalarme** in SPEC/PLAN, wo verbotene Begriffe als Verbot zitiert werden |
| `abhaengigkeits_budget.py` | `python3 tools/abhaengigkeits_budget.py [--crate X] [--json]` | nutzt `cargo metadata` oder Cargo.toml-Fallback |
| `isolation_proof.py` | `python3 tools/isolation_proof.py --crate X --modes default` | `--crate` Pflicht |
| `generate_interface_spec.py` | `… --repo-root . --out-dir <dir> --no-timestamp-only` | erzeugt Schnittstellen-Spec (3.578 öffentliche Symbole gemessen) |
| `generate_function_inventory.py` | `… --head --out-csv <f> --out-md <f>` | beide Pfade Pflicht |

**Defekt bzw. blockiert:**
- `gate_dependencies.py` (G1): meldet FAIL, weil `capabilities.toml` fehlt. Erwartetes Verhalten, bis die Datei existiert.
- `agent_tooling_common.py`: Bibliothek, wird von mehreren Tools importiert. Liest `capabilities.toml` (fehlt → leere Metadaten, kein Crash).
- `orchestrate_diagnostics.py`: läuft, ruft aber `gate_dependencies.py` auf (FAIL dort wird gemeldet).
- `tests/test_gate_g8_negative.py`: ImportError (`gate_g1_layering` existiert nicht; das Gate heißt `gate_dependencies.py`).
- `tools/baseline/run_baseline.py`: `--self-test` läuft grün. Der volle Lauf (`cargo` über alle Crates) dauert lange und ist auf den Build-Fehler der Facade angewiesen.

**Die Spec sieht vor, `tools/` drastisch zu verkleinern** (REM-14, §13.4): Zielbestand nur `doc_lint.py`, `loc.py`, `test_coverage_matrix.py`, `baseline/` und die Skripte aus §11.6. Die übrigen Auditor-Skripte sind laut Spec zu entfernen. **Nicht eigenmächtig löschen:** das ist eine Projektentscheidung. Erst mit dem Nutzer klären.

## 5. Bekannte Stolpersteine

1. **Toolchain 1.89.0 nicht ladbar** → `RUSTUP_TOOLCHAIN=stable`. Nicht versuchen, `static.rust-lang.org` per curl zu erreichen. Die alte Anleitung (conda-forge/apt) ist unnötig, solange stable baut.
2. **Facade `contextra` baut nicht.** Vor Arbeit an `contextra-db`-Schnittstellen zuerst prüfen, ob `contextra` darauf zugreift.
3. **Tool-Ausgaben in `docs/diagnostics/`** verschmutzen das Repo. Immer `--output-dir`.
4. **Zahlen aus der Spec sind Soll-Werte.** Anhang B und diese Datei sind Messungen vom 20./21.11.2026.
5. **Historische Dokumente** (`tools/baseline/README.md`, `benchmarks/contextra-bench/*`, `docs/archive/`) sind nicht normativ.

## 6. Empfohlene Reihenfolge für die nächste Sitzung

1. `tools/quickcheck.sh build` ausführen → Ist-Stand des Builds bestätigen.
2. Mit dem Nutzer klären, ob der Umbau der Facade `contextra` (Anpassung an `contextra-db`) Priorität hat, oder ob Stub/Feature-Gating gewünscht ist.
3. Fehlende Artefakte (`capabilities.toml`, `budgets.toml`, `layering.rs`) anlegen oder die Spec anpassen. Das ist eine Entscheidung, keine Routine.
4. Entfernungen aus §12 (REM-14 Selbstdeklarationen, Auditor-Skripte) nur nach Freigabe.

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

Jedes Crate hat eine eigene `CLAUDE.md` (Liste unten). Zweck: Eine Sitzung kann auf ein Crate beschränkt werden, ohne den ganzen Workspace zu laden.

| Crate | Pfad | Ebene | Build (21.11.2026) |
|---|---|---|---|
| contextra-ids | `crates/contextra-ids/CLAUDE.md` | E0 | ok |
| contextra-env | `crates/contextra-env/CLAUDE.md` | E0 | ok |
| contextra-sys | `crates/contextra-sys/CLAUDE.md` | E0 (unsafe) | ok |
| contextra-simd | `crates/contextra-simd/CLAUDE.md` | E0 (unsafe) | ok |
| contextra-durable-fs | `crates/contextra-durable-fs/CLAUDE.md` | E0 | ok |
| contextra-ports | `crates/contextra-ports/CLAUDE.md` | E0 | ok |
| contextra-crypto | `crates/contextra-crypto/CLAUDE.md` | E1 | ok |
| contextra-erasure | `crates/contextra-erasure/CLAUDE.md` | E1 (sicherheitskritisch) | ok |
| contextra-mvcc | `crates/contextra-mvcc/CLAUDE.md` | E1 (→ `version`) | ok |
| contextra-store | `crates/contextra-store/CLAUDE.md` | E1 | ok |
| contextra-vector | `crates/contextra-vector/CLAUDE.md` | E1 | ok |
| contextra-text | `crates/contextra-text/CLAUDE.md` | E1 | ok |
| contextra-graph | `crates/contextra-graph/CLAUDE.md` | E1, optional | ok |
| contextra-rank | `crates/contextra-rank/CLAUDE.md` | E1 | ok |
| contextra-privacy | `crates/contextra-privacy/CLAUDE.md` | E1 | ok |
| contextra-checkpoint | `crates/contextra-checkpoint/CLAUDE.md` | E1 | ok |
| contextra-types | `crates/contextra-types/CLAUDE.md` | (REM-30) | ok |
| contextra-testkit | `crates/contextra-testkit/CLAUDE.md` | E3, dev | ok |
| contextra-db | `crates/contextra-db/CLAUDE.md` | E2 | ok (dünn, 412 Zeilen) |
| contextra | `crates/contextra/CLAUDE.md` | E2 | **FEHLER** (20) |
| contextra-mcp | `crates/contextra-mcp/CLAUDE.md` | E3 | **FEHLER** (transitiv) |
| contextra-bench | `benchmarks/contextra-bench/CLAUDE.md` | E3, bench | **FEHLER** (2) |

**Arbeitsregeln für parallele Sitzungen:**

1. **Sitzung = ein Crate.** Startpunkt ist die Crate-`CLAUDE.md`, danach die Root-`CLAUDE.md` für übergreifende Regeln. Schreibzugriff nur auf das eigene Crate.
2. **Notizen nur in „Sitzungsnotizen“ am Ende der eigenen Crate-Datei anhängen.** Nicht überschreiben, nicht in fremde Dateien schreiben. Format: `### YYYY-MM-DD · Branch · Kurzfassung · Offen: …`.
3. **Root-`CLAUDE.md` nur durch die Koordinations-Sitzung ändern** (also durch den Nutzer bzw. eine dafür beauftragte Sitzung).
4. **Schnittstellenänderungen** (Ports, Typen aus `ids`/`types`, `Clock`/`SeededRng` in `env`): In der eigenen Notiz unter „Offen:“ beschreiben und die betroffenen Dependents nennen (siehe Eckdaten in deren Datei). Fremde Crates nicht selbst anpassen, sondern dort als offen eintragen.
5. **Branch pro Sitzung** (Konvention, noch nicht mit dem Nutzer bestätigt): `crate/<name>/<kurzthema>`, von `main` abzweigen, nur dieses Branch pushen. So kollidieren parallele Sitzungen nicht auf `main`. Vor dem Merge den Nutzer fragen.
6. **Datei-Eigentum:** `contextra`, `contextra-db`, `contextra-mcp`, `contextra-bench` hängen am Build-Fehler. Solange er nicht behoben ist, nur daran arbeiten, wenn die Aufgabe genau das ist.

**Bekannte Zuständigkeitsprobleme** (keine Sitzung darf sie still lösen):
- `contextra-types` (REM-30) muss aufgelöst werden. Betrifft fast alle Crates.
- `contextra-mvcc` → `contextra-version` (ADR-06). Betrifft alle Dependents in `Cargo.toml`.
- `capabilities.toml` fehlt. Solange es fehlt, ist die Schichtprüfung (G1) rot.
