# CLAUDE.md — Einstieg für Claude-Sitzungen im Contextra-Repo

**FOKUS (verbindlich):** `contextra-store` wird durch `contextra-storage` ersetzt, das Fundament von Contextra. Bis Gate G-6 (Löschung, §22) gilt ausschließlich der Storage-Kern. Alles andere im Repository wird ignoriert und nicht verändert. Normativ ist `docs/SPEC.md` (R7, Gate G-0 erfüllt, G-1 setzt O-1 und O-12 voraus). Arbeitskontext und Regeln für den Kern: `AGENTS.md` im Repo-Root. Vorfassungen und Konzepte liegen in `docs/archive/` und sind keine Anforderungsquelle.


---

## 0. Arbeitsregeln (verbindlich)

- **Subagenten nur als Haiku.** Beim `Agent`-Tool immer `model: "haiku"` setzen. Sonnet nie verwenden. Opus/Fable nur, wenn der Nutzer es ausdrücklich verlangt.
- **Spec ist Quelle.** `docs/SPEC.md` (R7) ist normativ für den Storage-Kern. Erst die Spec ändern, dann den Code (Spec §1.5). Die frühere Regel zu TOML-Blöcken in Anhang D/E, `benchmarks/budgets.toml`, `capabilities.toml` und `tools/check_spec_sync.py` betrifft die ältere Gesamtspezifikation; das Tool existiert im Repository nicht. Diese Dateien werden bis G-6 nicht angefasst.
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

## 5. Bekannte Stolpersteine

1. **Toolchain 1.89.0 nicht ladbar** → `RUSTUP_TOOLCHAIN=stable` setzen, auch bei `cargo metadata` und `cargo install`. Nicht versuchen, `static.rust-lang.org` per curl zu erreichen.
2. **Facade `contextra` baut nicht.** Vor Arbeit an `contextra-db`-Schnittstellen zuerst prüfen, ob `contextra` darauf zugreift.
3. **Anhänge sind Quelle.** `budgets.toml` und `capabilities.toml` nie von Hand ändern. Abweichungen zeigt `check`.
4. **Zahlen aus der Spec sind Soll-Werte.** Anhang B und diese Datei sind Messungen vom 20./21.11.2026.
5. **Historische Dokumente** (`tools/baseline/README.md`, `benchmarks/contextra-bench/PLAN.md|SPEC.md|AGENTS.md`, `docs/archive/`) sind nicht normativ und können auf entfernte Tools verweisen.
6. **`ruff` und `shellcheck` sind nicht überall sauber.** Die Legacy-Tools `agent_tooling_common.py`, `test_coverage_matrix.py` und `doc_lint.py` haben Stilbefunde. Neue Dateien sauber halten, Altbestand nur anfassen, wenn die Aufgabe es verlangt.


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
