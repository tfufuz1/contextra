> **Anpassungsvermerk (Stand 2026-11-02, HEAD `c41afc15`):** Dieses Dokument wurde gegen den Repo-Stand geprüft und automatisch angepasst. ⚠ markiert Crate-Namen, die im Repo nicht existieren. `⚠NICHT-IM-REPO(...)` markiert Befehle, deren `cargo xtask`-Target fehlt. Vorgenommene Änderungen:
> - 32 `cargo xtask`-Aufrufe markiert (xtask existiert nicht, kein passendes just-Target)
> - Kontextblöcke G-CTX-J und G-CTX-L durch repo-verifizierte Fassungen ersetzt
> - Prompts entfernt, weil ihr Gegenstand (Non-Default-Members sandbox/py/ollama/onnx; Inferenz, Python-Bindings, WASM, MCP) am HEAD nicht existiert: J-19, L-19, L-20, L-21, L-22
> - 24 Prompts mit Ist-Stand-Hinweis markiert (nennen nicht existierende Crates): J-11, J-12, J-16, J-17, J-23, J-26, J-27, J-30, J-31, J-32, J-40, J-41, J-43, J-44, L-00, L-01, L-10, L-11, L-12, L-13, L-16, L-17, L-18, L-23

# Contextra — Prompt-Katalog: Testen, Benchmarken, Auditieren

**Zielsysteme:** (A) Google Jules (isolierte Cloud-VM, PR-basiert) · (B) Lokal mit Gemini Antigravity-CLI (eigene Hardware, zsh, optional Nix)
**Repo:** `github.com/tfufuz1/contextra` · Toolchain `1.89.0` (gepinnt) · 14 Workspace-Mitglieder + tests/arch, kein xtask
**Referenzstand:** HEAD `c41afc15` (angepasst am 2026-11-02). Die Vor-Analysen bezogen sich auf ältere HEADs (e5bbb44d, 2026-10-03); Abweichungen sind Befunde.
**Quellen dieses Katalogs:** `CONTEXTRA_DEV_GUIDE.md`, `CONTEXTRA_AUDIT_PROMPT_PLAYBOOK_JULES.md`, `JULES-PROMPTS-ANALYSE-PRODUKTE-ERWEITERT.md`, `CONTEXTRA_FEATURE_KATALOG_ERWEITERT.md`, `CONTEXTRA_4KERNSYSTEME_TIEFENSPEZIFIKATION.md`, `PRODUKT-1…4-*-ANALYSE.md`

> **Abgrenzung zum bestehenden Playbook:** `CONTEXTRA_AUDIT_PROMPT_PLAYBOOK_JULES.md` deckt Kompilier-Diagnose und Reparatur ab. Dieser Katalog setzt **danach** an: massive Test-Kampagnen, Benchmarks und Audits — inklusive gezielter Beweis-Tests für jeden Befund aus den vier Produktanalysen (S1/S2-Bugs, Phantom-Anomalie, `DocId`-Truncation, Audit-Ketten-Rollback usw.).

---

## Inhaltsverzeichnis

0. [Nutzung & Aufteilung Jules ↔ Lokal](#0-nutzung--aufteilung-jules--lokal)
1. [Gemeinsame Bausteine (Kontext, Regeln, Berichtsvertrag)](#1-gemeinsame-bausteine)
2. [Zusammenbau-Skript für fertige Prompts](#2-zusammenbau-skript)
3. [Teil A — Google Jules (VM)](#teil-a--google-jules-vm)
   - A.0 Abhängigkeiten & Setup-Skript
   - A.1 Baseline & Gates (J-00, J-01)
   - A.2 Test-Kampagnen allgemein (J-10 … J-19)
   - A.3 Produkt-Kampagnen mit Beweis-Tests (J-20 … J-27)
   - A.4 Benchmarks in der VM (J-30 … J-32)
   - A.5 Audits (J-40 … J-44)
4. [Teil B — Lokal mit Gemini Antigravity-CLI](#teil-b--lokal-mit-gemini-antigravity-cli)
   - B.0 Umgebung & Hygiene (L-00, L-01)
   - B.1 Crash-, Fehlerinjektions- und Nebenläufigkeits-Kampagnen (L-10 … L-16)
   - B.2 Coverage, Soak, Integration, Forensik (L-17 … L-23)
   - B.3 Benchmarks unter Hardware-Hygiene (L-30 … L-35)
   - B.4 Release-Gate-Audits (L-40 … L-42)
5. [Befund → Prompt-Matrix](#5-befund--prompt-matrix)
6. [Empfohlene Reihenfolge](#6-empfohlene-reihenfolge)

---

## 0. Nutzung & Aufteilung Jules ↔ Lokal

### 0.1 Warum zwei Umgebungen

| Eigenschaft | Jules-VM | Lokal (Antigravity-CLI) |
|---|---|---|
| Laufzeit pro Auftrag | begrenzt (Minuten bis wenige Stunden), Session kann abbrechen | frei wählbar (Stunden/Tage) |
| Netzwerk | typischerweise nur Checkout + `crates.io`; Nix-Cache oft unerreichbar | voll (Nix, Ollama-Modelle, Datensätze) |
| Hardware | geteilt, unbekannte CPU, laute Nachbarn | bekannt, pinnbar (`taskset`, Governor) |
| Dateisystem/Disk | Overlay/virtuell, `fsync`-Verhalten nicht repräsentativ | echte Disk, echtes `fsync` |
| Root/`sudo` | nicht interaktiv, oft nur `sudo -n` | nach Rückfrage möglich |
| Ergebnis-Form | Pull Request + Bericht | Branch/Working-Tree + Bericht |
| Stärke | deterministische, hardware-unabhängige Prüfungen, viele parallele Sessions | Crash-/Kill-Tests, Syscall-Traces, Sanitizer, Langläufer, Benchmarks, FFI/WASM/Ollama |

### 0.2 Zuordnung der Aufgaben

| Aufgabe | Jules | Lokal | Begründung |
|---|:-:|:-:|---|
| Kompilier-/Gate-Sweep, Feature-Matrix | ✅ | ✅ (Gegenprobe) | deterministisch |
| Property-/Differential-/Grenzwert-Tests | ✅ | — | rein logisch, parallelisierbar |
| Beweis-Tests für Produktbefunde (S1/S2) | ✅ | ✅ (Crash-Teile) | Logik in Jules, echte Kills lokal |
| Loom (Modellprüfung) | ✅ | ✅ (tief) | Loom ist zeit-unabhängig |
| Stress mit hoher Taskzahl | ✅ (mittel) | ✅ (groß) | Skalierung hängt an Kernen |
| Fuzzing | ✅ (60 s/Target) | ✅ (Stunden) | Dauer |
| Mutation-Testing | ✅ (Shards) | ✅ (komplett) | Dauer |
| TSan/ASan/Miri | ⚠️ nur falls Nightly vorhanden | ✅ | Nightly + `rust-src` |
| SIGKILL-/Power-Loss-Kampagnen | ❌ | ✅ | echte Prozesse, echte Disk |
| `strace`-Fehlerinjektion (EIO, ENOSPC, Dir-fsync-Audit) | ❌ | ✅ | ptrace/Disk |
| Multi-Prozess-Locking (`DirLock`) | ⚠️ | ✅ | Prozess-Isolation |
| Absolute Benchmarks (ns, MB/s, QPS) | ❌ (nur relativ) | ✅ | Hardware-Hygiene |
| Recall-/Qualitäts-Benchmarks | ✅ | ✅ | hardware-unabhängig |
| Ollama/ONNX/Candle/CUDA real | ❌ | ✅ | externe Laufzeit |
| PyO3, WASM-Sandbox-Escape | ⚠️ (Build) | ✅ (Escape-Suite) | Laufzeit |
| Speicherrest-/Core-Dump-Forensik | ❌ | ✅ | `/proc/<pid>/mem` |
| Supply-Chain-, Tech-Debt-, Doku-Wahrheits-Audit | ✅ | — | statisch |

### 0.3 Konventionen

- **Prompt-IDs:** `J-xx` = Jules, `L-xx` = Lokal. Jeder Prompt steht in einem eigenen ```` ```text ````-Block unter einer Überschrift `### <ID> — Titel`.
- **Platzhalter:** Jeder Prompt beginnt mit `[[CTX]]` und endet mit `[[REPORT]]`. Diese werden durch die gemeinsamen Bausteine aus Kapitel 1 ersetzt — entweder von Hand (Copy-Paste) oder mit dem Skript aus Kapitel 2.
- **Parameter-Zeilen** (`PARAMETER: ...`) am Anfang eines Prompts vor dem Absenden anpassen.
- **Schreibzugriffe sind dateidisjunkt:** Jeder Prompt schreibt genau einen Bericht `docs/analysis/kampagne/<ID>.md` plus neue Testdateien `crates/<crate>/tests/campaign_*.rs`. Dadurch können viele Sessions parallel laufen, ohne Merge-Konflikte.
- **Beweis-Tests für offene Bugs** werden mit `#[ignore = "<BUG-ID>: beweist Defekt, entfernen sobald behoben"]` eingecheckt und im Bericht mit `-- --ignored` gezeigt (rot = Bug bestätigt). So bleibt CI grün, der Beweis bleibt reproduzierbar.
- **Ehrlichkeitsregel:** Alle Prompts formulieren Befunde aus den Vor-Analysen als **Hypothesen zur Verifikation**. Der Agent darf sie widerlegen — „bereits behoben" ist ein gültiges Ergebnis.

### 0.4 Was dieser Katalog **nicht** weiß (bitte selbst prüfen)

- Die genauen **CLI-Flags deiner Antigravity-Version** sind in den Projektdateien nicht dokumentiert. Die Prompts sind reiner Text und funktionieren interaktiv (Einfügen) oder als Datei (Zusammenbau-Skript → Datei → an die CLI übergeben; Aufruf per `--help` deiner Version nachsehen).
- Der **exakte Inhalt von `justfile`, `capabilities.toml`, `AGENTS.md`, `.jules/setup/**`** liegt nur indirekt vor. Prompts verlangen deshalb, Testbefehle und Gate-Namen **aus diesen Dateien zu lesen**, statt sie zu raten.
- Die Jules-Setup-Oberfläche (Initial-Setup-Skript pro Repository) kann sich ändern; prüfe die aktuelle Jules-Dokumentation.

---

## 1. Gemeinsame Bausteine

### G-CTX-J — Kontextblock für Jules (wird an jeden `J-`-Prompt vorangestellt)

```text
KONTEXT (gilt für die gesamte Session, nicht verhandelbar)

1. Repository: https://github.com/tfufuz1/contextra — Rust-Workspace, resolver = "2", 14 Workspace-Mitglieder + tests/arch (Stand HEAD c41afc15). contextra-erasure liegt unter crates/, ist aber kein Mitglied und leer. Ein xtask existiert nicht. Quelle der Wahrheit für Ring, Reifegrad und Testbefehle je Crate ist capabilities.toml (Felder ring, maturity, test). Lies sie, rate nichts. Vor-Analysen bezogen sich auf ältere HEADs; melde jede Abweichung.
2. Toolchain hart gepinnt (rust-toolchain.toml): 1.89.0 mit rustfmt + clippy. Prüfe ZUERST rustc --version / cargo --version. Toolchain-Drift ist eine eigene Fehlerklasse und wird nie mit Code-Fehlern vermischt.
3. Einstieg: `just` (Targets: check, gates, pre-push, bootstrap, coverage, loom-store, session-context). Befehle und Gates in docs/COMPILATION.md. Einzelne Crates: cargo test -p <crate> --locked.
4. Ring-Modell (SPEC 5.2, Abhängigkeiten nur nach unten): Ring 4 contextra · Ring 3 contextra-engine (contextra-erasure ist Soll, leer) · Ring 2 contextra-mvcc, -store, -text, -vector, -rank · Ring 1 contextra-sys, -simd, -durable-fs, -crypto · Ring 0 contextra-types, -ports · dev contextra-testkit, tests/arch.
5. Invarianten (nie aufweichen): Zero-Panic (kein panic!/unwrap/expect außerhalb #[cfg(test)]). unsafe nur in contextra-sys und contextra-simd (AR-8, capabilities.toml, AGENTS.md stimmen überein); alle anderen Crates tragen #![forbid(unsafe_code)]. Tests unter tests/ und Test-Module zählen nach der bisherigen Prüfregel; Abweichungen sind Befunde (tests/arch/src/lib.rs und crates/contextra-text/tests/alloc_profiler.rs enthalten unsafe). Determinismus P28 (Zeit/Zufall/IDs über injizierte Ports Clock/Rng/IdGen; Krypto-Schlüssel/Salt braucht echten CSPRNG). Sync-Kern P26 (kein tokio in Ring 0). TenantId(0) ist reserviert.
6. Geschützte Pfade NICHT ändern: .github/**, justfile, rust-toolchain.toml, deny.toml, capabilities.toml, AGENTS.md, crates/*/AGENTS.md, .agents/**. Neue Dateien sind erlaubt: Bericht unter docs/analysis/kampagne/ und neue Tests unter crates/<crate>/tests/campaign_*.rs. Untersagt AGENTS.md neue Testdateien, Testcode als Patch im Bericht ablegen und das als Befund melden.
7. Clippy ist workspace-weit mit -D warnings (unwrap_used, expect_used, panic, todo, unimplemented, unreachable = deny; siehe clippy.toml). Füge KEIN crate-weites #[allow] hinzu.
8. Jules-VM: bash (nicht zsh), nicht interaktiv (nur sudo -n), Netz meist nur crates.io. Lange Befehle mit timeout (z. B. timeout 25m ...), Ausgabe per tee nach /tmp/<ID>/*.log. Schlägt ein Befehl wegen Netz/Sandbox fehl: vollständigen Fehler berichten, NICHT durch schwächeren Ersatz maskieren.
9. Befehle, die als ⚠NICHT-IM-REPO markiert sind, existieren nicht. Ersatz nur mit Begründung im Bericht; niemals als bestanden melden.
```

### G-CTX-L — Kontextblock für Lokal / Antigravity-CLI (wird an jeden `L-`Prompt vorangestellt)

```text
KONTEXT (gilt für die gesamte Session, nicht verhandelbar)

1. Repository: lokaler Klon von https://github.com/tfufuz1/contextra (Rust-Workspace, 14 Workspace-Mitglieder + tests/arch, Stand HEAD c41afc15). contextra-erasure ist leer und kein Mitglied. Ein xtask existiert nicht. Quelle der Wahrheit für Ring, Reifegrad und Testbefehle: capabilities.toml. Vor-Analysen beziehen sich auf ältere HEADs; melde Abweichungen.
2. Shell ist zsh. Toolchain gepinnt auf 1.89.0 (rust-toolchain.toml, nicht ändern). Einstieg: just (check, gates, pre-push, coverage, loom-store, session-context), Befehle in docs/COMPILATION.md. Nightly-Befehle (TSan, ASan, Fuzz, Miri) nur über die dort beschriebenen Wege.
3. Netzwerk ist verfügbar, aber Contextra ist ein Air-Gap-/Souveränitäts-Produkt: Produktionscode darf zur Laufzeit KEINE Cloud-Verbindung voraussetzen. Jede Beobachtung, die dem widerspricht, ist ein Befund.
4. Ring-Modell (SPEC 5.2): Ring 4 contextra · Ring 3 contextra-engine · Ring 2 contextra-mvcc, -store, -text, -vector, -rank · Ring 1 contextra-sys, -simd, -durable-fs, -crypto · Ring 0 contextra-types, -ports · dev contextra-testkit, tests/arch.
5. Invarianten (nie aufweichen): Zero-Panic im Produktionscode; unsafe nur in contextra-sys und contextra-simd (Widerspruch zwischen Dateien ist ein Befund, keine Entscheidung für dich); Determinismus P28 (Clock/Rng/IdGen injiziert; Krypto-Material braucht echten CSPRNG); Sync-Kern P26; TenantId(0) reserviert.
6. Arbeitsweise lokal: Branch anlegen (git switch -c campaign/<ID>). Logs nach /tmp/contextra-campaign⚠/<ID>/ (nicht ins Repo). Wegwerf-Harnesses in einem eigenen Cargo-Projekt unter /tmp/contextra-campaign⚠/<ID>/ mit path-Dependency auf das Repo. Ins Repo kommen nur: der Bericht docs/analysis/kampagne/<ID>.md und neue Tests crates/<crate>/tests/campaign_*.rs.
7. Sicherheit: Arbeite ausschließlich in /tmp und im Repo. Keine Änderungen an Home-Verzeichnis, Systemkonfiguration oder Kernel-Parametern. Kein sudo ohne ausdrückliche Rückfrage; wenn eine Prüfung Root braucht, nenne den exakten Befehl und frage, sonst überspringe und berichte die Lücke. Kill-/Fault-Tests nur gegen eigene Kindprozesse und Wegwerf-Verzeichnisse (mktemp -d).
8. Ausgabe-Disziplin: Lange Logs nicht in den Chat kippen. In Dateien schreiben, nur Exit-Code, letzte ~40 Zeilen und Fundstellen zitieren. Messdaten als CSV ablegen.
9. Lange Läufe: vor dem Start Schätzdauer und Parameter ausgeben; nach jedem Teilschritt Stabilität prüfen (Speicher, Platte); bei weniger als 5 GB freiem Plattenplatz abbrechen.
10. Befehle, die als ⚠NICHT-IM-REPO markiert sind, existieren nicht. Ersatz nur mit Begründung im Bericht; niemals als bestanden melden.
```

### G-RULES — Verbindliche Regeln für alle Prompts

```text
VERBINDLICHE REGELN (für jeden Auftrag)

R1 Diagnose vor Reparatur. Wenn der Auftrag nicht ausdrücklich eine Reparatur verlangt, änderst du KEINEN Produktionscode. Du schreibst Tests und Berichte.
R2 Keine Abschwächung. Kein Gate, Lint, Schwellenwert, Toolchain-Pin oder Testfall wird gelöscht, ignoriert (ohne begründetes #[ignore]), mit großflächigem #[allow] unterdrückt oder in den Assertions aufgeweicht, um Grün zu erzwingen. Ein falsch-positives Gate meldest du als eigenen Befund.
R3 Beweisstufen. Jede Aussage trägt genau ein Etikett: [GEMESSEN] = von dir ausgeführt, Exit-Code und Ausgabe liegen vor; [BELEGT] = im Quellcode gelesen, mit Datei:Zeile; [VERMUTET] = Schlussfolgerung ohne Beleg. Ohne Etikett gilt eine Aussage als unzulässig. Formulierungen wie „sollte", „vermutlich", „dürfte" sind nur mit [VERMUTET] erlaubt.
R4 Anti-Mirroring. Erwartungswerte in Tests stammen aus einer UNABHÄNGIGEN Quelle: Referenzmodell (z. B. BTreeMap), Brute-Force, veröffentlichte Testvektoren, mathematische Invariante oder handgerechnetes Beispiel. Ein Test, der die Produktionsformel nachrechnet, ist wertlos. Nenne im Bericht je neuem Test die Orakel-Quelle.
R5 Keine erfundenen Konstanten. Kryptografische Testvektoren, Hash-Werte, Schwellenwerte oder Literaturwerte dürfen NICHT aus dem Gedächtnis eingetippt werden. Kopiere sie aus Dateien (z. B. ~/.cargo/registry/src/*/ der Abhängigkeiten, RFC-Text, Repo-Dateien) und nenne den Fundort. Ist keine Quelle verfügbar: berichte „Vektor nicht verfügbar" statt zu raten.
R6 Reproduzierbarkeit. Zufallsbasierte Tests loggen ihren Seed; jeder Befund enthält einen Ein-Zeilen-Reproduktionsbefehl. Flaky-Verdacht: 3 Wiederholungen, davon eine mit --test-threads=1, bevor du klassifizierst.
R7 Mustersuche. Findest du einen Fehler, suche aktiv nach Geschwistern (gleiche Funktion, gleicher Trait, gleiches Copy-Paste-Muster) im gesamten Workspace und liste sie auf.
R8 Misstraue allem. Das Projekt wurde vollständig von KI entwickelt. Code, Tests, Audit-Dokumente („VERIFIED") und Kommentare gelten erst nach eigener Prüfung als wahr. Behauptungen aus Vor-Analysen sind Hypothesen.
R9 Vollständigkeit. Melde auch, was du NICHT ausführen konntest (fehlendes Tool, Netz, Zeit) in einem eigenen Abschnitt „Nicht ausgeführt / Umgebungslücken", getrennt von Code-Befunden.
R10 Keine Selbstbestätigung. Ein Test, den du selbst geschrieben hast und der sofort grün ist, beweist nichts, solange du nicht gezeigt hast, dass er bei einem absichtlich eingebauten Fehler (Mutation im Wegwerf-Branch oder gedanklich begründeter Gegenfall mit [BELEGT]) rot würde. Führe bei den 3 wichtigsten neuen Tests diese Gegenprobe durch.
```

### G-REPORT — Berichtsvertrag (wird an jeden Prompt angehängt)

```text
BERICHT (einzige Schreibaktion außer neuen Testdateien): docs/analysis/kampagne/<ID>.md — neue Datei, Deutsch, genau in dieser Gliederung:

1. Zusammenfassung — Gesamturteil in max. 10 Zeilen, Ampel je Teilziel (GRÜN/GELB/ROT/NICHT GEPRÜFT), Reifegrad-Einschätzung R0–R4 für das Geprüfte (R0 nicht lauffähig/ohne Evidenz, R1 funktioniert im Happy Path, R2 robust unter Fehlern, R3 produktionsreif mit Evidenz, R4 unabhängig auditiert).
2. Ausgeführte Befehle — Tabelle: Befehl | Exit-Code | Dauer | Logdatei | Kurzergebnis. Jeder Befehl, der in der Zusammenfassung zitiert wird, steht hier.
3. Ergebnistabelle — je Prüfpunkt: Status (BESTÄTIGT / WIDERLEGT / TEILWEISE / NICHT GEPRÜFT), Beweisstufe, Fundstelle.
4. Befunde — je Befund: ID (<ID>-F01 …), Schwere (S1 Datenverlust/Sicherheitsbruch, S2 Korrektheits-/Integritätsfehler, S3 Robustheits-/Wahrheitslücke, S4 kosmetisch), Datei:Zeile, Beweisstufe, Erwartet vs. Tatsächlich, Ein-Zeilen-Reproduktionsbefehl, Geschwister-Fundstellen (R7), Fix-Skizze (nur Text, kein Code-Eingriff).
5. Neue Tests — Tabelle: Pfad | Testname | was er beweist | Orakel-Quelle (R4) | Status (grün / #[ignore] mit Bug-ID) | Gegenprobe (R10).
6. Nicht ausgeführt / Umgebungslücken — getrennt von Befunden; je Eintrag: Grund, exakter Befehl zum Nachholen, benötigtes Tool/Setup.
7. Offene Fragen an den Projektleiter — nur echte Entscheidungsfragen.
8. Selbstprüfung — Checkliste: Alle Befehle mit Exit-Code belegt? Alle Aussagen etikettiert (R3)? Keine erfundenen Konstanten (R5)? Kein Gate abgeschwächt (R2)? Keine Produktionsdatei geändert (R1)? Geschützte Pfade unberührt? Jede Abweichung ehrlich genannt?

Antworte im Chat nur mit: Pfad des Berichts, Gesamturteil (3 Zeilen), Anzahl Befunde je Schwere, Anzahl neuer Tests.
```

---

## 2. Zusammenbau-Skript

Das Skript ersetzt `[[CTX]]` durch den passenden Kontextblock (`G-CTX-J` für `J-…`, `G-CTX-L` für `L-…`) plus `G-RULES` und `[[REPORT]]` durch `G-REPORT` (mit eingesetzter Prompt-ID). Es liest diese Markdown-Datei selbst.

```python
#!/usr/bin/env python3
"""Nutzung:
  python3 assemble_prompt.py CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT.md --list
  python3 assemble_prompt.py CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT.md J-20 > /tmp/J-20.txt
  python3 assemble_prompt.py CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT.md J-20 | pbcopy   # macOS
  python3 assemble_prompt.py CONTEXTRA_PROMPT_KATALOG_TEST_BENCHMARK_AUDIT.md J-20 | wl-copy  # Wayland
"""
import re, sys

if len(sys.argv) < 3:
    sys.exit(__doc__)
md = open(sys.argv[1], encoding="utf-8").read().splitlines()

def block(bid):
    pat = re.compile(r"^###\s+" + re.escape(bid) + r"(\s|$)")
    for i, line in enumerate(md):
        if pat.match(line):
            j = i + 1
            while j < len(md) and md[j].strip() != "```text":
                j += 1
            k, out = j + 1, []
            while k < len(md) and md[k].strip() != "```":
                out.append(md[k])
                k += 1
            return "\n".join(out)
    sys.exit(f"Block {bid} nicht gefunden")

if sys.argv[2] == "--list":
    for line in md:
        m = re.match(r"^###\s+([JL]-\d+)\s+—\s+(.*)$", line)
        if m:
            print(f"{m.group(1):6} {m.group(2)}")
    sys.exit(0)

pid = sys.argv[2]
env = "G-CTX-J" if pid.startswith("J-") else "G-CTX-L"
text = block(pid)
ctx = block(env) + "\n\n" + block("G-RULES")
rep = block("G-REPORT").replace("<ID>", pid)
print(text.replace("[[CTX]]", ctx).replace("[[REPORT]]", rep))
```

---

# Teil A — Google Jules (VM)

## A.0 Abhängigkeiten & Setup-Skript

### A.0.1 Abhängigkeitsmatrix

| Abhängigkeit | Zweck | Benötigt von | Installation in der VM | Wenn nicht möglich |
|---|---|---|---|---|
| Rust **1.89.0** + `rustfmt`, `clippy`, `llvm-tools-preview` | gepinnte Toolchain, Coverage | alle | `rustup` (Skript unten) | nicht ersetzbar → Session abbrechen, ENV-Befund |
| Rust **nightly** + `rust-src`, `llvm-tools-preview` | `cargo fuzz`, TSan, ASan (`-Zbuild-std`) | J-14, J-15 | `rustup toolchain install nightly` | Prompts melden „Umgebungslücke" |
| `just` | alle Standard-Workflows | alle | `cargo +stable install just --locked` | Befehle direkt über `cargo`/`cargo xtask` |
| `cargo-deny`, `cargo-audit` | Supply-Chain | J-40 | `cargo +stable install … --locked` | `just security-audit` meldet Lücke |
| `cargo-llvm-cov` | Coverage | J-16 | `cargo +stable install cargo-llvm-cov --locked` | Coverage-Prompt wird NICHT GEPRÜFT |
| `cargo-mutants` | Mutation-Testing | J-17 | `cargo +stable install cargo-mutants --locked` | NICHT GEPRÜFT |
| `cargo-fuzz` | Fuzzing | J-15 | `cargo +stable install cargo-fuzz --locked` | NICHT GEPRÜFT |
| `ast-grep` (`sg`) | Tech-Debt-/Lock-Nesting-Scan (`just debt-audit`) | J-18, J-41 | `cargo +stable install ast-grep --locked` | Fallback `grep -rn`/`rg` (Lücke berichten) |
| `flatc` (FlatBuffers) | `check-flatbuffers-drift`, `regenerate-flatbuffers` | J-01 | `apt-get install flatbuffers-compiler` | Gate meldet ENV |
| C-Toolchain, `pkg-config`, `libssl-dev`, `clang`, `lld`, `cmake` | native Crates, Linker | alle | `apt-get install` | Linker-Fehler = ENV |
| `python3`, `venv`, `pip`, `maturin`, `numpy`, `pytest` | `contextra-py⚠` (`just py-test`) | J-19 | `apt` + `pip` im venv | `py-test` NICHT GEPRÜFT |
| WASM-Target (`wasm32-wasip1`, ggf. `wasm32-unknown-unknown`) | `contextra-sandbox⚠` | J-19 | `rustup target add …` | ENV-Befund |
| ONNX Runtime (nativ) | `contextra-infer-onnx⚠` | J-19 | abhängig von der `ort`-Konfiguration im `Cargo.toml` (Download beim Build oder `ORT_DYLIB_PATH`) — **prüfen** | ENV-Befund |
| `jq`, `strace`, `valgrind`, `gdb` | Auswertung, Syscall-/Speicheranalyse | J-00, J-14 | `apt-get install` | Teilprüfungen entfallen |
| Ollama | `contextra-infer-ollama⚠` real | **nicht in Jules** | — | Mock/Stub (`CONTEXTRA_EMBEDDING_PROVIDER=mock`) |
| Nix | reproduzierbare Dev-Shell | optional | **nicht installieren** | Fallback `nix develop -c … \|\| …` |

> **Wichtig (Toolchain-Falle):** Installiere `cargo-*`-Werkzeuge **außerhalb des Repos** mit `cargo +stable` (Verzeichnis `/tmp`), weil `rust-toolchain.toml` im Repo sonst `1.89.0` erzwingt und aktuelle Versionen der Werkzeuge eine neuere `rustc` verlangen können.

### A.0.2 Setup-Skript (in die Jules-Repository-Einstellungen als Initial-Setup eintragen)

> `.jules/setup/**` ist ein geschützter Pfad: Jules selbst ändert ihn nicht. Dieses Skript trägst **du** in der Jules-Oberfläche ein oder mergest es manuell in das vorhandene Setup. Prüfe vorher, was `.jules/setup/` bereits installiert, und ergänze nur Fehlendes.

```bash
#!/usr/bin/env bash
# Contextra – Jules-VM Setup für Test-/Benchmark-/Audit-Kampagnen
set -uo pipefail
export DEBIAN_FRONTEND=noninteractive
LOG=/tmp/contextra-setup⚠.log
step() { echo "=== $* ===" | tee -a "$LOG"; }
try()  { "$@" >>"$LOG" 2>&1 || echo "WARN: fehlgeschlagen: $*" | tee -a "$LOG"; }

# 0) sudo-Fähigkeit prüfen (nur nicht-interaktiv)
SUDO=""
if [ "$(id -u)" != "0" ]; then
  if sudo -n true 2>/dev/null; then SUDO="sudo -n"; else echo "WARN: kein sudo -n — apt-Schritte entfallen" | tee -a "$LOG"; fi
fi

# 1) Systempakete
if [ "$(id -u)" = "0" ] || [ -n "$SUDO" ]; then
  step "apt"
  try $SUDO apt-get update -y
  try $SUDO apt-get install -y --no-install-recommends \
    build-essential pkg-config libssl-dev clang lld cmake git curl ca-certificates \
    jq strace valgrind gdb ripgrep \
    python3 python3-venv python3-pip python3-dev \
    flatbuffers-compiler
fi

# 2) Rust: gepinnte Toolchain + stable (für cargo-Tools) + nightly (Fuzz/Sanitizer)
step "rustup"
if ! command -v rustup >/dev/null 2>&1; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain none >>"$LOG" 2>&1
  . "$HOME/.cargo/env"
fi
. "$HOME/.cargo/env" 2>/dev/null || true
try rustup toolchain install 1.89.0 --profile minimal -c rustfmt -c clippy -c llvm-tools-preview
try rustup toolchain install stable  --profile minimal
try rustup toolchain install nightly --profile minimal -c rust-src -c llvm-tools-preview
try rustup target add wasm32-wasip1 --toolchain 1.89.0
try rustup target add wasm32-unknown-unknown --toolchain 1.89.0

# 3) Cargo-Werkzeuge (außerhalb des Repos, mit stable)
step "cargo tools"
cd /tmp
for t in just cargo-deny cargo-audit cargo-llvm-cov cargo-mutants cargo-fuzz ast-grep; do
  command -v "${t/cargo-/cargo }" >/dev/null 2>&1 || command -v "$t" >/dev/null 2>&1 || try cargo +stable install "$t" --locked
done

# 4) Python-Umgebung für contextra-py⚠
step "python venv"
try python3 -m venv /tmp/venv
try /tmp/venv/bin/pip install --upgrade pip maturin numpy pytest

# 5) Umgebungsvariablen für alle Sessions
cat >> "$HOME/.bashrc" <<'ENVEOF'
export CARGO_TERM_COLOR=never
export CARGO_INCREMENTAL=0
export CARGO_NET_RETRY=5
export RUST_BACKTRACE=1
export CONTEXTRA_EMBEDDING_PROVIDER=mock
export PATH="/tmp/venv/bin:$HOME/.cargo/bin:$PATH"
ENVEOF

step "fertig – Protokoll: $LOG"
```

---

## A.1 Baseline & Gates

### J-00 — Umgebungs-Verifikation und Abhängigkeits-Lückenbericht

**Typ:** Audit (Umgebung) · **Dauer:** 10–20 min · **Schreibt:** nur Bericht · **Zuerst ausführen, blockiert alle anderen J-Prompts**

```text
[[CTX]]

AUFTRAG: Umgebungs-Baseline und Abhängigkeits-Lückenbericht. KEINE Code-Änderung, KEINE Installation außer den in Schritt 4 genannten.

SCHRITT 1 — Toolchain und Werkzeuge (jede Ausgabe vollständig protokollieren, stdout UND stderr):
  rustc --version --verbose; cargo --version --verbose; rustup show; cat rust-toolchain.toml
  rustup toolchain list; rustup target list --installed; rustup component list --installed
  cargo --list | rg -i "deny|audit|llvm-cov|mutants|fuzz|fmt|clippy"
  command -v just nix flatc ast-grep sg jq strace valgrind gdb python3 pip maturin clang lld cc pkg-config
  python3 -c "import numpy, sys; print(numpy.__version__)"; python3 -m maturin --version
  ldconfig -p | rg -i onnx || echo "libonnxruntime nicht im Linker-Cache"
  nproc; free -m; df -h .; ulimit -a; cat /proc/cpuinfo | rg -m1 "model name|flags"

SCHRITT 2 — Netzwerk-Sonden (nur HEAD-Requests, jeweils mit Timeout 10 s, Ergebnis je Host: erreichbar / blockiert):
  index.crates.io, static.crates.io, github.com, raw.githubusercontent.com, cache.nixos.org, pypi.org, files.pythonhosted.org, huggingface.co

SCHRITT 3 — Repo-Gesundheit:
  git status; git rev-parse HEAD; git log -3 --oneline; ls -la Cargo.lock
  cargo metadata --format-version=1 --locked > /tmp/J-00/metadata.json
  cargo fetch --locked   (prüft, ob alle Abhängigkeiten ladbar sind)
  just --list; ⚠NICHT-IM-REPO(cargo xtask env-validate); just env-validate
  Lies capabilities.toml und liste je Crate: Ring, Reifegrad, test-Befehl. Vergleiche die Crate-Anzahl mit "14 Workspace-Mitglieder + tests/arch, kein xtask" und melde Abweichungen.
  Lies AGENTS.md und rust-toolchain.toml und .jules/setup/** (nur lesen) und fasse zusammen, welche Werkzeuge das vorhandene Setup bereits bereitstellt.

SCHRITT 4 — Fehlende Werkzeuge: Versuche NUR für fehlende Werkzeuge aus dieser Liste die Installation (nicht interaktiv, außerhalb des Repos mit cargo +stable bzw. sudo -n apt-get): just, cargo-deny, cargo-audit, cargo-llvm-cov, cargo-mutants, cargo-fuzz, ast-grep, flatbuffers-compiler. Messe die Dauer je Installation. Scheitert eine, ist das ein Eintrag im Abschnitt "Umgebungslücken", kein Abbruch.

SCHRITT 5 — Abhängigkeits-Matrix als Tabelle im Bericht: Werkzeug | benötigt von Prompt-IDs (J-xx) | vorhanden (ja/nein/Version) | Konsequenz | exakter Installationsbefehl. Leite die Prompt-IDs aus folgender Zuordnung ab: llvm-cov→J-16, mutants→J-17, fuzz+nightly→J-14/J-15, ast-grep→J-18/J-41, flatc→J-01, maturin/numpy/python→J-19, wasm-Target→J-19, onnxruntime→J-19, deny/audit→J-40.

SCHRITT 6 — Verdikt: Ist die VM "bereit", "teilweise bereit (Liste)" oder "nicht bereit (Grund)"? Nenne, welche J-Prompts mit dem aktuellen Zustand sinnvoll ausführbar sind und welche zuerst ein erweitertes Setup-Skript (Kapitel A.0) brauchen.

[[REPORT]]
```

### J-01 — Vollständiger Gate-Sweep (alle `xtask`-Gates mit Exit-Code-Matrix)

**Typ:** Audit/Test · **Dauer:** 30–90 min · **Schreibt:** nur Bericht

```text
[[CTX]]

AUFTRAG: Führe ALLE deterministischen Architektur-, Governance- und Qualitäts-Gates einzeln aus und erstelle eine Gate-Matrix. Reiner Diagnose-Lauf: keine Fixes, keine Gate-Änderung.

VORBEREITUNG: Lies justfile (nur lesen) und xtask/src/main.rs (nur lesen), um die tatsächlich vorhandenen Subcommands zu ermitteln. Die Liste unten stammt aus dem Dev-Guide; Subcommands, die in main.rs nicht existieren, meldest du als "Doku-Drift" (Befund S4), Subcommands in main.rs, die unten fehlen, nimmst du zusätzlich auf.

Schreibe ein Wegwerf-Skript /tmp/J-01/run_gates.sh (NICHT ins Repo), das jedes Gate so ausführt:
    for g in <GATE>; do s=$(date +%s); timeout 20m cargo xtask "$g" >"/tmp/J-01/$g.log" 2>&1; rc=$?; echo "$g|$rc|$(( $(date +%s)-s ))s" >> /tmp/J-01/matrix.txt; done
und danach die just-Gates: just dag-check; just check-arch-gates; just registry-check; just harness-list; just sync-docs-check; just check-agents-integrity; just check-consistency.

GATE-LISTE (Reihenfolge einhalten, Gruppen getrennt auswerten):
 Gruppe 1 Compile/Lint: cargo fmt --all -- --check; cargo clippy --all-targets -- -D warnings; cargo check --all-targets --workspace; cargo check --examples
 Gruppe 2 Hot-Path-Gates (Teil von just check): check-max-results-unbound, check-toctou-defaults, check-nan-hot-loop, check-result-dropped-io, check-coverage-gate (nur falls coverage.json existiert)
 Gruppe 3 Architektur: check-dag, check-ring-layering, check-ring-layering-full, check-ring-capabilities-consistency, check-duplicate-core-primitives, check-duplicate-intent, check-duplicate-symbols, check-duplicate-symbols-cross-file, check-module-reachability, check-orphan-modules, check-phantom-files, check-placeholder-refs, check-crate-references, check-manifest-completeness, check-type-registry, check-unsafe-islands, lint-unsafe-slices, check-ffi-panic-boundary, check-flatbuffers-drift, check-recall-stability, check-bandit-latency-budget
 Gruppe 4 Doku/Governance: check-doc-references, check-toc-integrity, check-marker-drift, check-stale-tags, validate-tags, tag-health, check-agents-freshness, check-agents-integrity, check-jules-context-freshness, check-adr-deadlines, check-veto-deadlines, check-vetoes, check-action-pinning, check-workflow-commands, check-audit-duplication, check-audit-tool-evidence, check-audit-verdict-independence, audit-integrity-check, check-review-coverage
 Gruppe 5 Git-abhängig (benötigen ggf. vollständige Historie; bei flachem Checkout als ENV klassifizieren): check-commit-messages, check-commit-diff-integrity, commit-health, check-branch-overlap, shell-commit-audit
 Gruppe 6 Auswertung: workspace-verify --mode=audit; hotspot-report; panic-inventory; feature-matrix

KLASSIFIKATION je Gate (Spalte "Klasse"): PASS | FAIL-CODE (Gate zeigt echten Befund; nenne Datei:Zeile aus der Ausgabe) | FAIL-ENV (Netz, Historie, fehlendes Werkzeug) | FAIL-GATE (Gate selbst fehlerhaft/falsch-positiv — mit Begründung) | NICHT VORHANDEN.

ZUSATZPRÜFUNG "Gate-Wirksamkeit": Wähle 3 Gates aus Gruppe 2/3 und beweise im Wegwerf-Zustand (git stash-frei, auf einer Kopie des Verzeichnisses unter /tmp), dass sie bei einem absichtlich eingebauten Verstoß (z. B. ein .unwrap() in Produktionscode, ein neuer Aufwärts-Import zwischen Ringen, ein unbegrenztes max_results) tatsächlich ROT werden. Ein Gate, das den Verstoß nicht erkennt, ist Befund S2 (toter Wächter). Repository selbst bleibt unverändert.

AUSGABE zusätzlich zum Berichtsvertrag: die vollständige Gate-Matrix (Gate | Gruppe | rc | Dauer | Klasse | erste Fehlzeile) als Tabelle und eine Rangliste der FAIL-CODE-Gates nach Schwere.

[[REPORT]]
```

---

## A.2 Test-Kampagnen (allgemein)

### J-10 — Ring-weiser Volltest mit Flaky-Detektion, Release-Gegenprobe und Ignore-Inventur

**Typ:** Test · **Dauer:** 30–120 min je Ring · **Schreibt:** nur Bericht

```text
PARAMETER: RING = 0     (setze 0, 1, 2, 3, 4 oder TOOLING; eine Session je Ring, aufsteigend ausführen)

[[CTX]]

AUFTRAG: Vollständiger, wiederholter Testlauf für alle Crates des Rings RING. Ziel: Flaky-Tests, debug-/release-abhängiges Verhalten, vergessene #[ignore]-Tests und Doc-Test-Fehler aufdecken. Keine Änderung an Produktions- oder bestehendem Testcode.

Für JEDEN Crate des Rings (Liste aus capabilities.toml, Reihenfolge wie dort):
  a) Offizieller Testbefehl exakt aus capabilities.toml (Feld test), 3 Läufe: tee /tmp/J-10/<crate>.run<N>.log
  b) Ein Lauf mit --test-threads=1 (Deadlock-/Reihenfolge-Abhängigkeit aufdecken)
  c) Ein Lauf im Release-Profil: cargo test -p <crate> --release --locked  (Overflow-Wrap-, debug_assert- und as-Cast-Unterschiede; Fehler, die NUR im Release auftreten, sind S2)
  d) cargo test -p <crate> --doc --locked  (Doc-Tests separat)
  e) Ignore-Inventur: cargo test -p <crate> -- --list --ignored; je ignoriertem Test: Grund (#[ignore = "..."]), ob der Grund noch zutrifft, und führe ihn einmal mit -- --ignored aus, sofern die Laufzeit < 10 min bleibt.
  f) cargo check -p <crate> --no-default-features  (hermetischer Feature-Gate)

AUSWERTUNG:
 - Flaky-Tabelle: Test | Läufe bestanden/gesamt | Muster (nur bei --test-threads=1 grün? zeit-/seed-abhängig?). Ein Test, der in 3 Läufen nicht identisch besteht, ist S2 (Flaky), auch wenn der Mittelwert grün ist.
 - Debug-vs-Release-Differenzen als eigene Tabelle.
 - Für jeden fehlschlagenden Test: Klasse (a) Produktionsbug, (b) Test veraltet/falsch, (c) Flaky/nebenläufig — mit Beleg. Prüfe zusätzlich nach Anti-Mirroring (R4), ob der Test eine unabhängige Referenz nutzt oder die Produktionsformel wiederholt; melde Letzteres als Testqualitätsmangel, auch bei grünem Test.
 - Zähle je Crate: Anzahl Tests gesamt, ignoriert, Doc-Tests. Crates mit weniger als 1 Test pro 200 LOC (grobe Näherung, Zeilen via wc/tokei-Ersatz) sind "dünn getestet" — Liste.

[[REPORT]]
```

### J-11 — Feature-Matrix-Kampagne (alle Flags einzeln, Kombinationen, bekannte Inkompatibilitäten)

> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-adapt`, `contextra-graph`, `contextra-infer-candle`, `contextra-kvcache`, `contextra-mcp`, `contextra-router`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.

**Typ:** Test/Audit · **Dauer:** 60–150 min · **Schreibt:** nur Bericht

```text
[[CTX]]

AUFTRAG: Prüfe systematisch, dass jedes Cargo-Feature und die wichtigsten Kombinationen kompilieren, lint-sauber sind und ihre dokumentierten Invarianten einhalten. Keine Code-Änderung.

SCHRITT 1 — Inventar: Ermittle je Crate alle Features aus den Manifesten (cargo metadata --no-deps --format-version=1 | jq). Vergleiche mit dieser Soll-Liste aus dem Feature-Katalog (Abweichungen = Doku-Drift):
 contextra (Facade): default=["fast","candle"] (Abweichung von der ursprünglichen Spezifikation!), fast, sovereign, compliance, audit-export, avv-generator, ollama, candle, onnx, router
 contextra-store: encryption-at-rest, wal-integrity, deletion-proof, memory-only-storage, fault-injection, block-cache-v2, sieve-cache
 contextra-vector: experimental-diskann, experimental-rabitq, experimental-predicate-augmented-search, partial-index-rebuild, docid-128
 contextra-text: bm25f · contextra-graph⚠: ppr-forward-push, k-path-diffusion, graph-connectivity-health, edge-reinforcement-learning, docid-128
 contextra-adapt⚠: bandit-routing, flow-corrected-thompson, egress-sherman-morrison, sketched-bandit, rie-greedy-personalization
 contextra-router⚠: bandit-routing, egress-sherman-morrison, flow-corrected-thompson, cloud-egress-guard · contextra-rank: dibud
 contextra-kvcache⚠: content-addressed-kv-cache, kv-encryption, kvcache-attention-eviction, kvcache-kivi-quant, kivi-quantization
 contextra-infer-candle⚠: kv-bridge, kv-stage-b, cuda · contextra-crypto: cloud-egress-guard · contextra-mcp⚠: agent-workflows, kv-bridge, test-utils
 contextra-engine/db/cognition: coherence-bonus-fusion, adaptive-candidate-pool-sizing, volatile-vault, entity-extraction, auto-extraction-opt-out, background-maintenance, graph-connectivity-health, edge-reinforcement-learning, docid-128

SCHRITT 2 — Für JEDEN Crate und JEDES Feature einzeln: cargo check -p <crate> --no-default-features --features <f> UND cargo clippy -p <crate> --no-default-features --features <f> --all-targets -- -D warnings. Zusätzlich je Crate: --all-features (check + clippy + cargo test --no-run). Das Feature cuda und onnx nur wenn die VM es erlaubt (sonst ENV).

SCHRITT 3 — Gezielte Hypothesen (jede einzeln bestätigen oder widerlegen):
 H1: cargo check -p contextra-vector --all-features --tests scheitert mit E0308 in tests/filtered_selectivity_recall.rs (DocId::new(u64) vs u128) — Vor-Analyse BUG-VEC-02. Prüfe, ob "just check"/CI diese Kombination überhaupt baut (wenn nein: Gate-Lücke S2).
 H2: partial-index-rebuild ist architekturell vetoed und darf nicht Teil von default, sovereign, compliance oder all-features-Pfaden eines Release-Profils sein. Belege per cargo tree -e features.
 H3: memory-only-storage ist mit deletion-proof und dem sovereign-Ring inkompatibel (INV-DURABILITY-RING). Baue die Kombination in contextra-store und contextra: erwartet wird Compile-Fehler ODER ein deterministischer Err beim Öffnen — NICHT stilles Akzeptieren. Teste auch die Laufzeitseite mit einem Minimal-Test.
 H4: fault-injection darf in keinem Release-/Default-Feature-Pfad aktiv sein. Belege per cargo tree -e features -p contextra --release.
 H5: docid-128 ist über contextra-types, -vector, -graph, -cognition, -engine, -db konsistent durchgereicht; eine Kombination, in der ein Crate DocId als u128 und ein anderes als u64 kompiliert, ist S1. Prüfe cargo check --workspace mit und ohne docid-128.
 H6: sovereign und compliance ziehen contextra-crypto/-privacy bzw. audit-export/avv-generator wirklich ein (cargo tree -p contextra --features compliance -e normal).
 H7: Feature-Unification-Leck — cargo tree -d und cargo tree -e features -i <crate>: aktiviert ein Dev-Dependency oder ein Nachbar-Crate ein Feature, das im Einzelbau fehlt?

SCHRITT 4 — Kombinationsmatrix (nur Facade): {fast, sovereign, compliance} x {candle, ollama, onnx, keiner} x {router an/aus}: cargo check -p contextra --no-default-features --features "<Kombi>". Nicht bauende Kombinationen sind Befunde, sofern die Doku sie als unterstützt nennt.

[[REPORT]]
```

### J-12 — Property- und Differential-Test-Kampagne (ein Crate pro Session)

> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-graph`, `contextra-wire`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.

**Typ:** Test · **Dauer:** 60–180 min · **Schreibt:** Bericht + `crates/<crate>/tests/campaign_prop_*.rs`

```text
PARAMETER: CRATE = contextra-store     (eine Session je Crate; sinnvolle Reihenfolge: types, wire, mvcc, text, graph, rank, vector, store)

[[CTX]]

AUFTRAG: Baue für CRATE eine Property-/Differential-Testsuite gegen UNABHÄNGIGE Referenzmodelle und führe sie massiv aus. Bestehende Tests nicht ändern.

SCHRITT 1 — Bestand: cargo test -p CRATE --locked mit PROPTEST_CASES=20000 (nur vorhandene Proptests; just prop-tests zusätzlich ausführen). Logge Seeds. Neue proptest-regressions-Dateien: NICHT verwerfen, sondern im Bericht listen.

SCHRITT 2 — Neue Differential-Tests (Datei crates/CRATE/tests/campaign_prop_<thema>.rs; Muster der vorhandenen Proptests im Crate übernehmen). Wähle die Zeile deines Crates:
 contextra-types: Roundtrip/Ordnung/Hash-Konsistenz für DocId (inkl. Werte nahe 2^64 und u128::MAX), TxId, TenantId (try_new(0) muss Err liefern), Grenzwerte u64::MAX.
 contextra-wire⚠: FlatBuffers Encode→Decode-Roundtrip für zufällige Nachrichten; Decode beliebiger Bytes darf nie paniken (nur Err); Stabilität: gleiche Eingabe → gleiche Bytes (Determinismus).
 contextra-mvcc: Sichtbarkeit (SeqLogEntry::is_visible, SequenceLog::is_visible) gegen ein naives Referenzmodell (Vec aller Versionen, lineare Suche); min_retention_seq/min_active_seqno monoton; compact(min_active_seqno) darf nie eine für einen aktiven Snapshot sichtbare Version entfernen; changes_since vollständig.
 contextra-text: BM25-Ranking gegen eine unabhängig aus der Literaturformel (Robertson/Spärck Jones, mit explizit gesetztem k1 und b) geschriebene Naiv-Implementierung (Toleranz epsilon); Scores >= 0; Monotonie in der Termfrequenz; Löschen+Neuinsert ergibt identische Rankings; Unicode/leer/ein Token.
 contextra-graph⚠: CSR-Aufbau gegen Adjazenz-Map-Referenz (Nachbarn, Grade, Pfade/BFS-Distanzen); PageRank/PPR: Masse bleibt ~1, alle Werte >= 0, deterministisch; Selbstschleifen, Mehrfachkanten, isolierte Knoten, leerer Graph; TxId::is_valid_origin() wird auf allen Graph-Operationen erzwungen (AGT-GRAPH-001).
 contextra-rank: RRF-Eigenschaften (Permutationsinvarianz bei Gleichstand, Monotonie, Determinismus); Kalibrierung (Isotonic/Platt): Ausgabe monoton nichtfallend und in [0,1]; NaN/Inf-Eingaben dürfen nie zu NaN-Scores in der Ausgabe führen.
 contextra-vector: Distanzfunktionen (Cosine, Euclidean, Dot; f32 und u8-quantisiert) SIMD-Pfad gegen skalare Eigen-Referenz über Dimensionen {1,2,3,7,8,15,16,17,31,32,33,127,128,129,768,1536}, unaligned Slices, Subnormals, NaN/±Inf, Nullvektoren; HNSW-Suche gegen NaiveReferenceIndex (recall@10 je Seed berichten); search_at(seq) gegen Referenz-Snapshot.
 contextra-store: LSM-Ops (put, delete, put_batch, delete_prefix, get, get_at_seq, scan_prefix, scan_range, commit, rollback, rollback_to_tx) als zufällige Operationsfolge gegen BTreeMap-Referenz mit Versionsliste; mit eingestreutem Close→Reopen; scan_*-Reihenfolge und Grenzen (inkl. Präfix 0xFF…, leerer Key, Key-Maximallänge via validate_key); MemTable-Sharding darf Scan-Reihenfolge nicht verändern.

SCHRITT 3 — Pflicht-Grenzwerte in JEDER neuen Property-Strategie: leer, genau ein Element, Duplikate, u64::MAX/usize::MAX als Längen, f32::INFINITY/NEG_INFINITY/NaN, maximale Key-/Value-Größen.

SCHRITT 4 — Ausführung: PROPTEST_CASES=50000 (Schätzzeit loggen; bei > 20 min auf 10000 reduzieren und das begründen). Alle Fehlschläge: Shrink-Ergebnis (minimales Gegenbeispiel) im Bericht, Klassifikation Produktbug vs. Testbug, Reproduktionsbefehl mit PROPTEST_RNG_SEED bzw. dem geloggten Seed.

SCHRITT 5 — Gegenprobe (R10) für die 3 wichtigsten neuen Tests: zeige durch begründeten Gegenfall oder temporäre Mutation in einer Kopie unter /tmp, dass der Test einen Fehler erkennen würde.

[[REPORT]]
```

### J-13 — Grenzwert- und Fehlerpfad-Lückenaudit mit Testgenerierung (ein Crate pro Session)

**Typ:** Test · **Dauer:** 60–150 min · **Schreibt:** Bericht + `crates/<crate>/tests/campaign_boundary_*.rs`

```text
PARAMETER: CRATE = contextra-engine     (bekannte Testlücke: nur ~24 von 75 Dateien mit Tests; danach store, db, router, agent, cognition, privacy, mcp)

[[CTX]]

AUFTRAG: Prüfe für CRATE die Einhaltung der verbindlichen Pflichtabdeckung (CONTRIBUTING.md): Happy Path, leere Eingaben, Einzelelemente, Grenzwerte (u64::MAX, f32::INFINITY), Fehlerpfade, Concurrency. Schließe die größten Lücken mit neuen Tests.

SCHRITT 1 — Öffentliche Oberfläche inventarisieren: alle pub fn / pub trait-Methoden des Crates (ast-grep oder rg; Ergebnis nach /tmp/J-13/pub_api.txt). Gruppiere nach Modul.

SCHRITT 2 — Abdeckungsmatrix: Für jede pub fn ermittle per Suche in tests/ und #[cfg(test)]-Modulen, welche der 6 Pflichtkategorien bereits getestet sind. Erzeuge die Matrix Funktion x Kategorie (✓/✗) als CSV /tmp/J-13/matrix.csv und fasse im Bericht die Top-30-Lücken zusammen, priorisiert nach Risiko (Persistenz, Konkurrenz, Parsing unvertrauter Daten, Löschpfade, Mandantentrennung, Speichergrößen aus externen Längenfeldern).

SCHRITT 3 — Neue Tests (crates/CRATE/tests/campaign_boundary_<modul>.rs) für die Top-Lücken. Pflicht-Fälle: leere Eingabe; Einzelelement; Duplikate; Key-/Value-/Batch-Maximalgrößen und +1; TenantId(0) (muss abgelehnt werden); DocId oberhalb 2^64; usize::MAX/u64::MAX als Limit- oder Längenparameter (darf weder allozieren noch paniken); f32::NAN/INFINITY in Scores und Vektoren; max_results = 0 und sehr groß (check-max-results-unbound); Fehlerpfade: jede Err-Variante des Crates mindestens einmal gezielt auslösen; Cancel-Sicherheit async (Future droppen/abbrechen mitten in der Operation, danach muss der Zustand konsistent sein); Concurrency: >= 16 Tasks, deterministisch via Barrier statt sleep.

SCHRITT 4 — Panic-Freiheit: Suche im Nicht-Test-Code des Crates nach Indexzugriffen (x[i]), as-Casts auf schmalere Typen, Slicing mit unvertrauten Offsets, Rekursion ohne Tiefenlimit, Vec::with_capacity(untrusted). Je Fund: Fundstelle + ein Test, der den Pfad mit bösartiger Eingabe trifft (oder [VERMUTET], falls nicht erreichbar).

SCHRITT 5 — Lauf: cargo test -p CRATE --locked sowie cargo clippy -p CRATE --all-targets -- -D warnings. Rote Beweis-Tests mit #[ignore = "<Bug-ID>: ..."] versehen und mit -- --ignored zeigen.

[[REPORT]]
```

### J-14 — Nebenläufigkeits-Kampagne: Loom, Chaos-Matrix, Stress, Lock-Hierarchie

**Typ:** Test · **Dauer:** 60–180 min · **Schreibt:** Bericht + `crates/contextra-store/tests/campaign_stress_*.rs`

```text
[[CTX]]

AUFTRAG: Prüfe Nebenläufigkeits-Korrektheit von contextra-store (und Konsumenten) mit Modellprüfung, Fehlerinjektion und Stress. Loom findet Interleavings, keine Zeiten: JEDER Loom-Fehlschlag ist ein echter, reproduzierbarer Defekt (kein Flake).

SCHRITT 1 — Loom: just loom-store; RUSTFLAGS="--cfg loom" cargo test -p contextra-store --test loom_group_commit_handoff -- --nocapture; ⚠NICHT-IM-REPO(cargo xtask loom-run). Variiere LOOM_MAX_PREEMPTIONS in {2,3,4} (Laufzeit loggen; bei Explosion begrenzen und begründen). Bei Fehlschlag: Interleaving Zeile für Zeile analysieren; unterscheide Datenrennen vs. Lock-Ordering/Deadlock.

SCHRITT 2 — Chaos/Fault-Injection: just chaos-test; zusätzlich cargo test -p contextra-store --test chaos_matrix -- --ignored --test-threads=1 (Test-Name aus dem Repo entnehmen, falls abweichend). Notiere Iterationszahl, Killpunkte, Fehlerarten; bewerte, ob die Matrix tatsächlich fsync-Fehler, Kurz-Writes und ENOSPC abdeckt oder nur Happy-Path-Variationen.

SCHRITT 3 — Lock-Hierarchie statisch prüfen: Dokumentierte Reihenfolgen (AGENTS.md: collections → kv_locks → embedder; Store laut Analyse: commit_mutex, state, sstables, wal, flush_mutex, truncate_lock). Finde mit ast-grep/rg alle Stellen, an denen zwei dieser Locks verschachtelt gehalten werden (just debt-audit nutzt dasselbe Muster) und erstelle den Lock-Graphen als Mermaid-Diagramm im Bericht. Jede Kante gegen die Soll-Reihenfolge prüfen; Zyklus oder Umkehr = S1 (Deadlock-Potenzial). Prüfe außerdem: wird ein std::sync-Lock über einen .await gehalten (Deadlock/Send-Probleme)?

SCHRITT 4 — Neue Stress-Tests (crates/contextra-store/tests/campaign_stress_*.rs), alle mit Watchdog (tokio::time::timeout bzw. Thread-Join mit Frist), damit ein Deadlock als Fehlschlag statt als Hänger erscheint:
 a) Group-Commit: >= 200 parallele Tasks committen je 50 Batches mit künstlicher I/O-Verzögerung (FaultVfs/fault-injection-Feature, falls vorhanden). Nach dem Lauf: Anzahl bestätigter Commits == sichtbare Commits, HMAC-Kette lückenlos (verify_wal_chain_completeness bzw. Replay). Suche aktiv nach Fällen, in denen commit_mutex während Disk-I/O gehalten wird (Test test_group_commit_leader_releases_commit_mutex_during_disk_io als Ausgangspunkt, mit mehr Last).
 b) Commit/Rollback/Flush-Interleaving: zufällige Mischung aus put, put_if_absent, delete, commit, rollback, rollback_to_tx, flush, maybe_compact_for_test über 32 Tasks; Invariante: nach Quiescence stimmt der Zustand mit einem seriellen Referenzmodell überein (nur committete Tx sichtbar).
 c) Cancel-Sicherheit: Tasks mit put_if_absent/commit zufällig per JoinHandle::abort bzw. timeout abbrechen, danach darf kein Key dauerhaft gesperrt bleiben (Intent-Locks) — diese Hypothese stammt aus Befund S2-03 der Vor-Analyse; formuliere sie als Test.
 d) Observer-Stress: ein WalObserver, dessen on_commit nie zurückkehrt, plus set_max_observer_latency: Commit-Durchsatz darf nicht einbrechen (Fail-Open); dropped_count steigt.

SCHRITT 6 — TSan/ASan: Falls Nightly + rust-src vorhanden: just tsan und just asan, sonst als Umgebungslücke melden (NICHT als Code-Fehler).

[[REPORT]]
```

### J-15 — Fuzzing-Kampagne (kurz) mit Crash-Triage und Fuzz-Lücken-Analyse

**Typ:** Test · **Dauer:** 60–120 min · **Schreibt:** Bericht + Regressionstests

```text
PARAMETER: SEKUNDEN_PRO_TARGET = 60

[[CTX]]

AUFTRAG: Führe alle vorhandenen Fuzz-Targets aus, triagiere jeden Crash und finde Parser, die kein Fuzz-Target haben.

SCHRITT 1 — Voraussetzung: Nightly + cargo-fuzz vorhanden? Wenn nicht: Abbruch mit Umgebungslücken-Bericht (nicht raten, nicht ersetzen).
SCHRITT 2 — Targets ermitteln: just harness-list und Verzeichnisse crates/*/fuzz/fuzz_targets/. Erwartet laut Dev-Guide: 11 Targets (u. a. fuzz_wal_replay in contextra-store). Weicht die Zahl ab: Befund S4.
SCHRITT 3 — Lauf: just fuzz-all <SEKUNDEN_PRO_TARGET>. Falls das Rezept an der VM scheitert: je Target cargo +nightly fuzz run <target> -- -max_total_time=<SEKUNDEN_PRO_TARGET> -rss_limit_mb=2048 -print_final_stats=1. Logge Ausführungen/s, Coverage-Features und neue Korpus-Dateien je Target.
SCHRITT 4 — Crash-Triage je Crash-Artefakt: just fuzz-minimize <crate> <target> bzw. cargo fuzz tmin; Stack-Trace zitieren; Klassifikation: Panic (Verstoß gegen Zero-Panic, S2), Out-of-Memory/Timeout (unvertraute Längenfelder, S2/S3), UB/Sanitizer (S1), Fuzz-Harness-Fehler. Wandle jedes minimierte Artefakt in einen deterministischen Regressionstest um (crates/<crate>/tests/campaign_fuzzregress_<target>.rs, Eingabe als eingebettetes Byte-Array, mit #[ignore = "<Bug-ID>..."] solange offen).
SCHRITT 5 — Lückenanalyse: Welche Parser/Decoder unvertrauter Bytes haben KEIN Fuzz-Target? Prüfe mindestens: WAL-Frame (wal/encode.rs), WAL-Replay stream vs. mmap (Paritäts-Differential-Target fehlt vermutlich), SSTable-Footer/Block/Index (sstable/reader.rs), Bloom::from_bytes, Manifest::load/ManifestEntry::from_bytes, FlatBuffers-Wire-Decoder, HnswHeader/NodeRecord::from_bytes, DiskAnnHeader/Footer::try_from_bytes, Pending-/Tombstone-WAL, KdfHeader::from_bytes, DeletionProof/AuditChain-Deserialisierung, MCP-JSON-RPC-Eingabe, Prompt-Injection-normalize_text. Liste je Lücke: Funktion, Datei:Zeile, Risiko, Skizze des Targets (Text). Schreibe neue Targets nur, wenn just registry-check danach grün bleibt und KEIN geschützter Pfad berührt wird; sonst nur Empfehlung.

[[REPORT]]
```

### J-16 — Coverage-Kampagne mit Lückenschluss für kritische Funktionen

> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-license`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.

**Typ:** Test · **Dauer:** 60–150 min · **Schreibt:** Bericht + Tests

```text
[[CTX]]

AUFTRAG: Miss die Abdeckung, identifiziere ungetestete sicherheits- und korrektheitskritische Funktionen und schließe die fünf gefährlichsten Lücken.

SCHRITT 1 — just coverage-json (erzeugt coverage.json) und just coverage-html; ⚠NICHT-IM-REPO(cargo xtask check-coverage-gate). Tabelle je Crate: Zeilen-, Funktions- und (falls verfügbar) Branch-Abdeckung. Ist cargo-llvm-cov/llvm-tools nicht verfügbar: Abbruch mit Umgebungslücke.
SCHRITT 2 — Lückenliste: Dateien mit 0 % und die 30 größten ungedeckten Funktionen. Markiere die folgenden kritischen Kandidaten und prüfe jeweils die Abdeckung explizit: Wal::recover_from_poison, wal/replay.rs (replay_mmap vs replay), Wal::truncate/truncate_uncommitted_tail, rewrite_as_v3 und migrate_legacy_wal, manifest rollover, compaction retain_key_versions, validate_tombstone_safety, SequenceLogSsiValidator::coarsen_*, diagnose_pruning_blocker, TxBuffer::reap_orphans_bounded, HNSW rebuild_phase2_merge_and_swap, DiskANN init_hnsw_fallback/recover_pending_delta, Sq8Bias::calibrate, RaBitQ generate_orthogonal_matrix, RevocationLog::verify_integrity, DeletionProof::verify_external, AuditChain::verify_chain/verify_head_signature, EgressGuard fail-closed-Zweige, BulkExfiltrationDetector Zeit-Regression, contextra-license⚠ Prüfpfade, contextra-engine (Testlücke).
SCHRITT 3 — Zeilenabdeckung ist kein Beweis: Für die 5 gefährlichsten Lücken schreibe Tests (crates/<crate>/tests/campaign_cov_*.rs) mit unabhängigem Orakel (R4). Zeige danach die neue Abdeckung dieser Funktionen. Berichte außerdem 3 Funktionen, die zwar "abgedeckt" sind, aber nur über Assertions-lose Aufrufe (Test ruft auf, prüft nichts) — das ist Scheinabdeckung.
SCHRITT 4 — Gate-Bewertung: Ist der Schwellenwert im Coverage-Gate (xtask) plausibel zur gemessenen Abdeckung? Nur berichten, nichts ändern (geschützter Pfad).

[[REPORT]]
```

### J-17 — Mutation-Testing in Shards (Tier-0 Krypto, SSI, WAL-HMAC, Egress)

> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-privacy`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.

**Typ:** Test · **Dauer:** 60–180 min je Shard · **Schreibt:** Bericht + Killer-Tests

```text
PARAMETER: ZIEL = contextra-crypto     SHARD = 1/4     (weitere Ziele: contextra-mvcc, contextra-store --file 'src/wal/**', contextra-privacy⚠)

[[CTX]]

AUFTRAG: Mutation-Testing zeigt, ob die Tests Fehler überhaupt bemerken. Führe einen Shard aus, klassifiziere überlebende Mutanten und schreibe Tests, die die wichtigsten töten.

SCHRITT 1 — Voraussetzung: cargo-mutants vorhanden (sonst Umgebungslücke und Abbruch). cargo mutants --list --package <ZIEL> | wc -l  (Gesamtzahl Mutanten loggen).
SCHRITT 2 — Lauf: just mutants-crypto für contextra-crypto, sonst cargo mutants --package <ZIEL> [--file '<glob>'] --shard <SHARD> --timeout 120 -j 2 --output /tmp/J-17. Bei Zeitüberschreitung der Session: Teilergebnis mit Anzahl getesteter/ausstehender Mutanten berichten, nicht schönrechnen.
SCHRITT 3 — Auswertung: gefangen / überlebt / Timeout / unbrauchbar. Mutation-Score = gefangen / (gefangen + überlebt). Liste die 20 überlebenden Mutanten mit der höchsten Sicherheitsrelevanz (Vergleichsoperatoren in Auth-/Verifikationspfaden, ersetzte Konstanten, entfernte Prüfungen, vertauschte Rückgabewerte in verify_*, Nonce-/Zähler-Inkremente, Längenpräfix-Berechnung bei Hash-Eingaben, Schwellen in SSI/Coarsening, Grenzen in Replay).
SCHRITT 4 — Je überlebendem Mutant: Klasse (a) äquivalent (Verhalten identisch — begründe [BELEGT]), (b) Testlücke. Für die 10 wichtigsten Testlücken schreibe Killer-Tests (crates/<crate>/tests/campaign_mut_*.rs) mit unabhängigem Orakel; zeige durch erneuten Lauf nur dieser Mutanten (cargo mutants --file ... --regex '<mutant>'), dass sie jetzt gefangen werden.
SCHRITT 5 — ⚠NICHT-IM-REPO(cargo xtask check-mutation-score-gate) (nur ausführen/berichten); ⚠NICHT-IM-REPO(cargo xtask record-mutation-score) NICHT ausführen, wenn es Repo-Dateien verändert — beschreibe stattdessen, was es schreiben würde.

[[REPORT]]
```

### J-18 — Test-Qualitäts- und Orakel-Audit (statisch, ohne Änderung)

**Typ:** Audit · **Dauer:** 45–90 min · **Schreibt:** nur Bericht

```text
[[CTX]]

AUFTRAG: Das Projekt wurde zu 100 % von KI entwickelt; die Vor-Analyse zeigt, dass viele als "VERIFIED" deklarierte Garantien auf Orakel-Tests beruhen, die die Implementierung spiegeln. Bewerte die Aussagekraft der vorhandenen Tests, ohne sie zu ändern.

SCHRITT 1 — Statische Muster (ast-grep bzw. rg; Fundzahlen je Crate):
 - Tests ohne jede Assertion (kein assert!/assert_eq!/assert_ne!/matches!/should_panic/?-Rückgabe mit Prüfung)
 - assert!(x.is_ok()) / assert!(x.is_some()) als einzige Prüfung (Ergebnis nie inspiziert)
 - assert_eq!(f(x), f(x)) und Tests, die den Erwartungswert über dieselbe Produktionsfunktion berechnen (Spiegel-Orakel)
 - #[ignore] ohne Begründung; #[should_panic] ohne expected = "..."; Tests mit thread::sleep/Zeitabhängigkeit; Tests mit SystemTime::now/thread_rng (Verstoß P28 auch in Tests?)
 - #[cfg(test)]-Code, der Produktionsinvarianten per Test-Hook umgeht (z. B. *_for_test-Funktionen, die Prüfungen auslassen)
SCHRITT 2 — Stichproben-Tiefenprüfung: Wähle je Kern-Crate (store, mvcc, crypto, vector, privacy) die 10 Tests, die in docs/audits/** oder Feature-Katalogen als Beleg für eine Garantie zitiert werden. Lies Test UND getestete Funktion. Bewerte: unabhängiges Orakel? würde der Test bei einem realistischen Fehler rot? Skala: STARK / SCHWACH / SPIEGEL. Begründe jeden Wert mit Fundstelle.
SCHRITT 3 — Doku-Wahrheit: Finde in docs/audits/** Aussagen "VERIFIED"/"bewiesen"/"100 %". Prüfe stichprobenartig 10 davon: existiert der zitierte Test noch, läuft er, prüft er die Aussage wirklich? Abweichungen = Befund S3 ("Doku-Wahrheit").
SCHRITT 4 — Ergebnis: Orakel-Score je Crate (Anteil STARK/SCHWACH/SPIEGEL in der Stichprobe), Top-20 schwächste Tests mit konkretem Umschreibevorschlag (Text), Liste der Garantien, die aktuell NICHT durch einen starken Test belegt sind.
Schreibe KEINE Änderungen an bestehenden Tests.

[[REPORT]]
```

## A.3 Produkt-Kampagnen mit Beweis-Tests

> Diese Prompts übersetzen die Befunde der vier Produktanalysen in **ausführbare Beweise**. Jede Hypothese ist ein Test: ROT auf unverändertem Code = Befund bestätigt (Test mit `#[ignore = "<Bug-ID>…"]` einchecken), GRÜN = Befund behoben oder widerlegt (dann Regressionstest behalten).

### J-20 — Produkt 1 (LSM-Storage), Teil A: WAL, Recovery, Durability, SSTable-Härtung

**Typ:** Test · **Dauer:** 90–180 min · **Schreibt:** Bericht + `crates/contextra-store/tests/campaign_lsm_*.rs`

```text
[[CTX]]

AUFTRAG: Beweise oder widerlege die folgenden Hypothesen zu contextra-store (WAL, Recovery, Durability, SSTable) durch Tests. Quellen der Hypothesen: PRODUKT-1-LSM-STORAGE-ANALYSE.md. Lies vor jedem Test die betroffene Produktionsfunktion. Keine Produktionsänderung.

H1 [S1-01, lsm/ops/write.rs ~Z. 240]: WalOp::TxEnd erhält die Sequenznummer der Vor-Operation statt einer eigenen. Erwartung der Analyse: nach put → commit → close → reopen bricht das Replay mit WalCorruption "Duplicate or non-monotonic sequence number" ab; 16 Tests der Vor-Analyse schlugen deshalb fehl.
   Test: (a) put+commit, Drop, Reopen: alle Daten sichtbar, kein Fehler; (b) mit rollback_to_tx dazwischen; (c) Gruppen-Commit mit 50 Tx; (d) Reopen ZWEIMAL hintereinander. Zusätzlich Invariante über die Roh-WAL-Einträge: Sequenznummern streng monoton steigend (lies die Einträge mit scan_entries_with_callback oder dem Replay-Sink). Führe zuerst cargo test -p contextra-store --locked aus und notiere, welche Tests aktuell fehlschlagen (checkpoint_systematic_crash.rs wurde in der Analyse genannt).
H2 [S2-01, write.rs ~Z. 290]: Rollback eines fehlgeschlagenen MemoryOnly-Commits setzt last_hmac nicht zurück; Folge-Appends erzeugen eine ungültige HMAC-Kette.
   Test: Fehlerinjektion über simulate_wal_append_failure_for_test (bzw. FaultVfs, Feature fault-injection) für Durability-Modi (ermittle die Varianten des Durability-Enums per grep, inkl. MemoryOnly); danach erfolgreicher Commit, Schließen, Reopen: HMAC-Kette gültig (Replay ok, verify_wal_chain_completeness ok), kein doppelter und kein fehlender Eintrag.
H3 [Poison, wal/mod.rs ~Z. 596–638]: Nach fsync-Fehler markiert is_poisoned() das WAL; recover_from_poison stellt den letzten gültigen HMAC-Anker her.
   Test: fsync-Fehler erzwingen → is_poisoned()==true → weitere Schreibversuche müssen scheitern (kein stilles Weiterschreiben auf unbestimmtem Page-Cache-Zustand, "fsyncgate") → recover_from_poison → (a) kein bestätigter Commit verloren, (b) WAL wieder schreibbar, (c) Kette intakt, (d) ein Commit, der VOR dem Fehler Ok zurückgab, ist nach Reopen vorhanden; ein Commit, der Err zurückgab, ist NICHT sichtbar.
H4 [Torn Tail vs. Korruption]: WAL-Reader unterscheidet abgeschnittenen Schwanz (tolerierbar) von Mitten-Korruption (muss hart fehlschlagen).
   Test: Erzeuge ein WAL mit N=30 Commits. (a) Schneide die Datei an JEDEM Byte-Offset der letzten 3 Frames ab → Reopen muss entweder den längsten gültigen Präfix wiederherstellen oder klar fehlschlagen, nie paniken, nie Daten VOR dem Abschnitt verlieren. (b) Kippe je ein Bit an einer Stichprobe von mindestens 500 Offsets in der Mitte → Fehler muss erkannt werden (Checksumme/HMAC), kein stilles Falschlesen. (c) Zero-Fill-Schwanz anhängen. (d) Paritätstest replay vs. replay_mmap vs. replay_stream: identische Ergebnisse (Daten UND Fehler) für alle obigen Eingaben (Differential-Test).
H5 [Frame-Limits]: Längenfelder in WAL-Frames/SSTable-Blöcken/Manifest-Einträgen aus unvertrauten Bytes dürfen keine riesige Allokation oder Überlauf auslösen.
   Test: Präpariere Dateien mit Längenfeld u32::MAX/u64::MAX/Länge > Restdatei; Reopen muss mit Err scheitern, in begrenzter Zeit und Speicher (messe Peak-RSS über /usr/bin/time -v; Grenze: nicht mehr als 256 MB über Baseline). Gleiches für SSTable-Footer (Magic/Version falsch), Bloom::from_bytes (absurde Parameter), BlockBuilder-Indexoffsets.
H6 [Manifest-Atomarität, manifest/*]: Rollover/Rewrite ist crash-sicher (neu schreiben → fsync → rename → Verzeichnis-fsync).
   Test (logisch, ohne echte Kills): Fehler an jeder I/O-Stelle der Rollover-Sequenz injizieren (FaultVfs); danach muss Reopen entweder den alten oder den neuen Manifest-Zustand sehen, nie einen gemischten; reconstruct_valid_sstables/reconstruct_dead_sstables konsistent. Dokumentiere, welche Injektionspunkte die Test-Infrastruktur tatsächlich anbietet — fehlende Punkte sind ein Befund (Testbarkeit).
H7 [SSTable-Reader]: Block-Cache-Alias nach Dateiersatz — gleiche (Datei-ID, Offset)-Schlüssel nach Compaction mit wiederverwendetem Pfad liefern keine veralteten Blöcke.
   Test: SSTable lesen, Compaction ersetzt Datei, erneut lesen: Ergebnis muss dem neuen Inhalt entsprechen; Gleiches für die drei Cache-Backends (SIEVE, LRU, QuickCache), soweit per Feature wählbar.
H8 [L5 MergeOperator-Verdrahtung]: Trait existiert, wird im CompactionEngine-Merge-Pfad aber nicht aufgerufen (Analyse: compaction/engine.rs ~Z. 410, S3).
   Test: open_with_merge_operator mit zählendem MergeOperator; überlappende Keys über mindestens 3 SSTables erzeugen; maybe_compact_for_test; Assertion: merge() wurde aufgerufen UND der Endwert entspricht dem erwarteten Merge-Ergebnis (handgerechnetes Beispiel, z. B. Zähler-Addition). Fehlerpfad: merge() liefert Err → beide Versionen bleiben erhalten (fail-safe), nichts geht verloren. Belege zusätzlich die Aufrufkette von CompactionEngine::with_merge_operator bis merge_sstables per Datei:Zeile.
H9 [Tombstone-/Retention-Sicherheit]: Compaction darf keine Version entfernen, die ein aktiver Snapshot noch sieht (retain_key_versions, validate_tombstone_safety, MVCC-Floor).
   Test: Snapshot pinnen, überschreiben/löschen, Compaction erzwingen: get_at_seq(snapshot) liefert weiter den alten Wert; nach unpin und erneuter Compaction ist der Platz freigegeben. Zusätzlich: adaptive Planung (CostBasedAdaptivePlanner::plan_compaction, WorkloadMetrics) mit read-heavy vs. write-heavy Eingaben ergibt tatsächlich unterschiedliche CompactionStrategy-Entscheidungen, und keine davon verletzt die Tombstone-Sicherheit.

Führe alle neuen Tests aus, sowohl normal als auch mit -- --ignored für rote Beweis-Tests. Berichte je Hypothese: BESTÄTIGT/WIDERLEGT/TEILWEISE, Beweisstufe, Fundstelle.

[[REPORT]]
```

### J-21 — Produkt 1 (LSM-Storage), Teil B: Nebenläufigkeit, Intent-Locks, Mandantentrennung, Modell-Differential

**Typ:** Test · **Dauer:** 90–180 min · **Schreibt:** Bericht + `crates/contextra-store/tests/campaign_lsm_*.rs`

```text
[[CTX]]

AUFTRAG: Prüfe Intent-Locks, Mandantenisolation, Löschnachweis-Bindung und das Gesamtverhalten gegen ein Referenzmodell. Quellen: PRODUKT-1-LSM-STORAGE-ANALYSE.md (S2-02, S2-03, L6, L7, L9), CONTEXTRA_4KERNSYSTEME_TIEFENSPEZIFIKATION.md (1.2, 1.8, 1.10).

H1 [S2-03/L9, write.rs ~Z. 50–90 und ~75]: Abgebrochenes put_if_absent hinterlässt verwaiste Intent-Locks; der Key bleibt dauerhaft für Schreibzugriffe gesperrt.
   Test: 1000 Iterationen: put_if_absent in einer Tx starten und an zufälligen Punkten abbrechen (Future droppen, JoinHandle::abort, rollback, Fehlerinjektion im Commit-Pfad, Guard-Drop), danach MUSS ein neues put_if_absent/put auf denselben Key in einer frischen Tx gelingen. Prüfe cleanup_intent_locks_where, clear_intent_locks_for_tx und clear_intent_locks_above_tx einzeln: bleibt nach jedem Pfad die Intent-Lock-Tabelle leer (über eine Diagnosefunktion, falls vorhanden; sonst indirekt über Verhalten)?
H2 [L9/K1, TOCTOU]: put_if_absent nutzt TxBuffer::staged_status zusammen mit intent_locks; zwei konkurrierende put_if_absent auf denselben Key: genau EINES gewinnt.
   Test: 64 Tasks mit Barrier starten gleichzeitig put_if_absent(key, value_i); Erwartung: genau ein Ok(erstellt), alle anderen Ok(bereits vorhanden)/Err(Konflikt) — niemals zwei Gewinner; nach Commit enthält der Key den Wert des Gewinners. 200 Wiederholungen mit variierter Verzögerung.
H3 [L6, tenant_codec.rs ~Z. 320–410]: TenantScopedStorage isoliert strikt (Präfix t:{tenant_id}:).
   Test: Mandanten 1, 10, 100, 11 und u64::MAX legen überlappend benannte Keys an. scan_prefix/scan_prefix_bounded/scan_bounded/delete_prefix/scan_prefix_at eines Mandanten darf NIE Keys eines anderen liefern oder löschen. Grenztests für upper_bound_for_prefix: Präfixe, die auf 0xFF enden, leerer Präfix, Präfix == Key. Gleicher Testsatz gegen eine direkte LsmStorage-Instanz und gegen TenantScopedStorage (Trait-Äquivalenz über StorageEngineHandle). TenantId(0) muss abgelehnt werden.
H4 [L7/S2-02, kv/segment.rs ~Z. 120–160]: KvSegmentManager::generate_deletion_proof erzeugt einen Proof, ohne dass das KV-Segment physisch gelöscht/geshreddet wurde (verletzt INV-DELETION-1 und INV-KV-DELETE-1).
   Test: Segment anlegen → generate_deletion_proof OHNE vorheriges Löschen: erwartet wird Err. Danach Löschen (KvDeleteMode::CryptoShred; teste auch den nicht erlaubten Modus) → Proof gelingt und verifiziert extern (verify_external). Verifiziere zusätzlich, dass der Proof an den physischen Zustand gebunden ist: nach Wiedereinfügen desselben Segments muss ein alter Proof ungültig/abgelehnt sein.
H5 [S1-02/L10, util.rs ~Z. 15–60]: DirLock nutzt nur create_new(true), keine POSIX-Locks. Logischer Teil hier, echte Mehrprozess-Prüfung in L-12.
   Test (im Prozess): zweites LsmStorage::open auf dasselbe Verzeichnis muss mit Err scheitern; nach Drop/close muss ein erneutes Öffnen gelingen; STALE-Lock: LOCK-Datei von Hand zurücklassen (simuliert SIGKILL) — dokumentiere das Verhalten (Reopen blockiert für immer? überschreibt blind?). Beide Ausgänge sind Befunde mit Bewertung.
H6 [Observer, lsm/observer.rs ~Z. 240–280]: Circuit-Breaker öffnet bei Observer-Timeout; Commit bleibt Fail-Open.
   Test: Observer, dessen on_commit nie zurückkehrt, mit set_max_observer_latency(50 ms): 1000 Commits dürfen den Durchsatz nicht um mehr als den Faktor 2 gegenüber ohne Observer senken (relative Messung); is_circuit_breaker_open()==true; dropped_count/dropped_count_for steigen; clear_circuit_breaker setzt zurück.
H7 [Gesamtmodell]: Differential-Test über Operationsfolgen mit Crash-Simulation.
   Test: Zufällige Folgen aus put, put_batch, delete, delete_prefix, put_if_absent, commit, rollback, flush, maybe_compact_for_test, Close→Reopen (Drop ohne close = simulierter Crash NACH bestätigtem Commit) gegen ein BTreeMap-Referenzmodell, das nur committete Transaktionen kennt. Invariante: nach jedem Reopen entspricht der sichtbare Zustand exakt dem Modell. Mindestens 5000 Folgen à 100 Operationen; Seeds loggen; Fehlschlag schrumpfen.
H8 [Pressure/Backpressure, system_pressure.rs ~Z. 110–160]: compute_pressure eskaliert monoton; kein Flapping an Grenzwerten.
   Test: synthetische Lastsequenzen (Budget nahe Limit, tiefe WAL-Queue, hohe Blocking-Pool-Last) einzeln und kombiniert; prüfe Monotonie und Hysterese. Prüfe, ob run_with_metrics_sink tatsächlich in Compaction-/WAL-Backpressure verdrahtet ist (Aufrufkette Datei:Zeile) oder nur teilweise.

Alle Beweis-Tests: auf unverändertem Code ausführen (auch -- --ignored). Je Hypothese Status + Beweisstufe + Fundstelle.

[[REPORT]]
```

### J-22 — Produkt 2 (MVCC/SSI): Phantom, Write-Skew, unabhängiger Serialisierbarkeits-Checker

**Typ:** Test · **Dauer:** 90–180 min · **Schreibt:** Bericht + `crates/contextra-mvcc/tests/campaign_ssi_*.rs`, `crates/contextra-store/tests/campaign_ssi_*.rs`

```text
[[CTX]]

AUFTRAG: Prüfe, ob Contextra wirklich Serializable Snapshot Isolation bietet. Quelle: PRODUKT-2-MVCC-SSI-ANALYSE.md. Gesamturteil der Vor-Analyse: Punkt-Reads/Writes serialisierbar (0 Zyklen in 10.000 Historien), Präfix-Reads degenerieren auf Snapshot Isolation mit Phantomen, weil scan_prefix_tracked das ReadSet::record_prefix nicht aufruft. Verifiziere unabhängig. Hinweis: Der Feature-Katalog führt SSI (MVC-006, INV-MVCC-SSI-1) noch als SOLL, die Tiefenspezifikation als implementiert — kläre den Status per Code und melde den Katalog-Drift.

H1 [BUG-MVCC-01, contextra-store lsm/ops/read.rs ~Z. 47]: scan_prefix_tracked registriert nur gefundene Einzel-Keys im ReadSet, nicht das Präfix.
   Test (contextra-store, über StorageEngineHandle mit supports_ssi_tracking()): Tx A: scan_prefix_tracked("acct:") liest 2 Keys. Tx B: put("acct:new") und commit. Tx A: schreibt "summary" (abhängig vom Scan) und commit. SSI-Erwartung: A MUSS mit Serialisierungskonflikt scheitern. Scheitert A nicht = Phantom bestätigt. Variante: Delete eines gescannten Keys (soll gefangen werden); Variante: Insert AUSSERHALB des Präfixes (darf KEINEN Konflikt auslösen — Falsch-Positiv-Kontrolle).
H2 [Write-Skew]: Klassisches Beispiel zweier Ärzte.
   Test: Zwei Tx lesen beide Keys x und y (Punkt-Reads), jede schreibt einen anderen Key, jede unter der Bedingung "x+y>=1". Nur eine darf committen.
H3 [Unabhängiger Beweis, Serialisierbarkeits-Checker]: Schreibe in einer Testdatei einen eigenen Checker (NICHT aus der Produktion abgeleitet): Jede Transaktion protokolliert Reads (Key + gelesene Version/Schreiber-Tx) und Writes; baue den Abhängigkeitsgraphen mit wr-, ww- und rw-Kanten über committete Transaktionen und prüfe auf Zyklen (Serialisierbarkeit ⇔ azyklisch). Erzeuge >= 10.000 zufällige Historien (3–8 nebenläufige Clients, 2–6 Keys für hohe Konkurrenz, Mischung aus Punkt-Reads, Writes, Deletes und Präfix-Scans), mit festem Seed. Berichte: Anzahl Historien, Anzahl abgebrochener Tx, Anzahl gefundener Zyklen — getrennt für Historien OHNE und MIT Präfix-Scans (die Vor-Analyse erwartet Zyklen nur mit Scans).
H4 [K2, ssi.rs Coarsening]: SequenceLogSsiValidator::new_with_bounds(max_tracked_keys) vergröbert Keys zu Präfixen (coarsen_keys_to_prefixes, coarsen_oldest_buckets, merge_adjacent_buckets) ohne Falsch-Negative.
   Test: Differential gegen einen unbegrenzten Referenz-Validator (naive Menge aller committeten Keys je Sequenz): für jede Folge muss GILT: Referenz meldet Konflikt ⇒ gebundener Validator meldet Konflikt (keine Falsch-Negative). Messe zusätzlich die Falsch-Positiv-Rate. Adversariale Keys: "a","ab","abc"; leerer Key b""; Key == Präfix eines anderen; sehr lange Keys; Kapazität 1, 2, 3.
H5 [BUG-MVCC-03, ssi.rs ~Z. 270]: Das Commit-Register ist rein flüchtig; nach Neustart ist pruned_through = 0.
   Test: Validator mit prune_through(n) erzeugen; ReadSet mit min_snapshot_seq < pruned_through validieren: ein Commit mit Snapshot älter als die Prune-Grenze darf NICHT blind durchgewunken werden (konservativ abbrechen). Belege per Test, was tatsächlich passiert; ein stiller Erfolg ist S2.
H6 [BUG-MVCC-02, ssi.rs ~Z. 592]: validate() und record_commit_key() setzen exklusiven Aufrufer-Lock (commit_mutex) voraus.
   Test: Zeige per Thread-Test, dass zwei parallele validate_and_record-Aufrufe ohne äußeren Lock einen Konflikt verpassen können UND belege (Datei:Zeile), dass alle Aufrufer im Workspace den Lock halten (rg nach validate_and_record/record_commit_key). Gibt es einen Aufrufer ohne Lock: S2.
H7 [K3/K4, seq_log.rs, snapshot.rs]: SnapshotGuard hält min_active_seqno; ein abgelaufener SequenceLog-Pin löscht keine Live-Daten.
   Test: Guard erstellen, panic in einem Thread mit catch_unwind: Guard-Drop muss release auslösen. compact(min_active_seqno) entfernt nie eine für einen aktiven Snapshot sichtbare Version. pin_snapshot_at/expired_pins_at/max_pin_duration mit injizierter Clock (kein SystemTime::now). diagnose_pruning_blocker meldet bei mehreren Pins den ältesten. min_active_seqno ist monoton nichtfallend, solange keine neuen älteren Snapshots registriert werden.
H8 [K1/K5, tx_buffer.rs]: staged_status liefert bei mehreren Tx den Status der jüngsten; stage_bounded/reap_orphans_bounded respektieren Grenzen.
   Test: Matrix 2–4 konkurrierende Tx x {Insert, Delete, Insert+Delete} auf denselben Key; reap_orphans_bounded(max=30) bei 100 verwaisten Tx entfernt genau 30 und holt den Rest nachträglich ohne Speicherleck; stage_bounded über der Max-Ops-Grenze liefert Err; is_key_staged_for_tx ist Tx-spezifisch (nicht global approximativ).
H9 [Doku-Wahrheit, BUG-MVCC-04]: Spezifikation behauptet "Row-Level Locks", tatsächlich OCC mit Backward-Validation. Suche alle Stellen in docs/** und README, die Locks/Pessimismus behaupten (rg -i "row-level|zeilen.?sperre|pessimistic"), und liste sie.

Führe alle Tests aus (auch -- --ignored). Je Hypothese Status + Beweisstufe + Fundstelle. Reifegrad-Urteil begründen.

[[REPORT]]
```

### J-23 — Produkt 3 (Vektor-Index), Teil A: HNSW, ACORN, DocId-Truncation, Deletes, Rebuild, Recall

> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-cognition`, `contextra-graph`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.

**Typ:** Test · **Dauer:** 90–180 min · **Schreibt:** Bericht + `crates/contextra-vector/tests/campaign_hnsw_*.rs`

```text
[[CTX]]

AUFTRAG: Prüfe contextra-vector (HNSW + ACORN) gegen die Befunde aus PRODUKT-3-VEKTOR-INDEX-ANALYSE.md. Schreibe Tests mit unabhängigen Orakeln (NaiveReferenceIndex = Brute-Force, eigene skalare Distanz).

H1 [BUG-VEC-01, S1, hnsw/types.rs ~Z. 142 und diskann/types.rs]: DocId (u128) wird per `as u64`/`as usize` gehasht bzw. abgeschnitten; zwei DocIds, die sich nur in den oberen 64 Bit unterscheiden, kollidieren.
   Test (mit Feature docid-128): DocId A = 1, DocId B = (1<<64)+1 und C = (2<<64)+1 mit verschiedenen Vektoren einfügen. Beide/alle müssen einzeln auffindbar (search), einzeln löschbar (delete/remove_with_graph_repair) und mit korrekter all_doc_ids/len-Statistik sein. Löschen von B darf A und C nicht beeinflussen. Gleicher Test für DiskANN (sofern per Feature baubar), contextra-graph⚠ (CSR/Knoten-IDs), contextra-text (Dokument-IDs) und contextra-cognition⚠. Danach Mustersuche (R7): rg nach `as u64`, `as usize`, `as u32`, `.0 as`, `inner() as` in allen Crates und liste JEDE Stelle, an der eine DocId-/u128-Größe verengt wird, mit Bewertung (harmlos/Kollision/Datenverlust).
H2 [BUG-VEC-02, S2]: tests/filtered_selectivity_recall.rs kompiliert unter --all-features nicht (DocId::new(u64) vs u128, E0308).
   Prüfe cargo test -p contextra-vector --all-features --no-run und berichte die exakte Fehlermeldung. KEINE Korrektur. Schlage als Text vor, wie die Feature-Kombination in ein Gate (J-11/L-01) aufgenommen wird.
H3 [BUG-VEC-03/K1, S2, hnsw/adaptive_ef.rs ~Z. 114]: AdaptiveEfStateMachine::step erhöht ef bei Distanz-Gleichstand (identische Distanzen) bis max_ef, weil is_stable() dauerhaft false liefert; 2–4x Latenz-Spitzen bei duplizierten Vektoren.
   Test: (a) Datensatz mit vielen identischen und fast identischen Vektoren (z. B. 30 % Duplikate): protokolliere die ef-Trajektorie je Suche (über search_adaptive_with_stats); Erwartung für korrektes Verhalten: ef steigt nicht weiter, wenn die TOP-K-MENGE (als Menge, nicht als Reihenfolge) zwischen Runden unverändert bleibt. (b) Vergleich ef_final und Distanzberechnungen mit/ohne Duplikate; berichte das Verhältnis als relative Zahl [GEMESSEN]. (c) Teste is_terminated() garantiert Abbruch bei ef_search >= max_ef.
H4 [K2, Zweiphasen-Rebuild]: rebuild_phase1_snapshot_and_build + rebuild_phase2_merge_and_swap integrieren parallele Inserts/Deletes vollständig; search_at(seq) ist deterministisch.
   Test: Während Phase 1 laufen parallele Inserts und Deletes; nach dem Swap: kein Dokument fehlt/doppelt (Vergleich mit Referenzmenge), search_at(seq_vor_rebuild) liefert vor, während und nach dem Swap IDENTISCHE Ergebnisse; wait_for_rebuild_with_timeout hält die Frist; RebuildGuard/SnapshotPinGuard werden bei Panic im Rebuild freigegeben (catch_unwind).
H5 [K3, arena.rs]: Nach free_node und Slot-Wiederverwendung können veraltete Backlinks bestehen bleiben, wenn ein Leser die alte NodeId hält.
   Test: Churn-Test 50.000 Operationen (Insert, Delete, Re-Insert mit zufälligen IDs): danach verify_no_ghost_pointers() und check_connectivity()/connectivity_score(); zusätzlich Thread-Test: Leser hält NodeId, Schreiber löscht und realloziert den Slot, Leser darf nie einen FALSCHEN Vektor als Treffer zurückgeben (Vergleich Treffer-Vektor mit Referenz). Prüfe try_relink_pruned_neighbors auf Reziprozität.
H6 [K4/BUG-VEC-04, partial_rebuild.rs, VETO-F02]: rebuild_region führt nur Tombstone-Pruning aus, kein Subgraph-Rewiring; find_oversaturated_regions rechnet Dichten, wird aber nicht wirksam genutzt.
   Test: Zeige das tatsächliche Verhalten von rebuild_region/should_trigger_partial_rebuild/hot_path_nodes (verändert sich die Graphstruktur außer durch Tombstone-Entfernung?). Belege den Feature-Gate (experimental-partial-rebuild / partial-index-rebuild) per Code. Bewerte, ob Namen und Doku Erwartungen wecken, die der Code nicht einlöst (S3 Doku-Wahrheit).
H7 [Recall-Suite, Hardware-unabhängig]: Messe recall@10 und recall@100 gegen NaiveReferenceIndex für (a) Gauß-Zufallsdaten, (b) geclusterte Daten, (c) adversarielle Daten (Duplikate, kollineare Vektoren, Nullvektoren, Norm ~1e-30 bei Cosine, sehr große Werte), Metriken Cosine/Euclidean/Dot, ef_search in {16,32,64,128,256}, M in {8,16,32}, N in {1k,10k,50k}, Dimension {16,128,768}. Seeds loggen. Berichte die Zahlen als Tabelle. Übernimm vorhandene Schwellenwerte aus bestehenden Tests; neue Schwellen NUR als Vorschlag kennzeichnen, nicht als Assertion.
H8 [ACORN, acorn/ und acorn_filtered.rs]: search_knn_acorn mit Prädikaten verschiedener Selektivität (0,1 %, 1 %, 10 %, 50 %, 100 %, 0 %).
   Test: Ergebnis gegen Brute-Force-gefilterte Referenz (exakter Filter, dann Top-K). Berichte recall je Selektivität; Selektivität 0 % muss leeres Ergebnis ohne Hänger liefern; compute_gamma_edge_budget(base_degree, selectivity) für Grenzwerte (0, 1, sehr klein, NaN) ohne Panic/Überlauf; Prädikat, das paniken würde, darf den Index nicht beschädigen.
H9 [Limits]: max_elements erreicht, Dimension 0, Dimension != Index-Dimension, NaN/Inf in Vektor, leerer Index, k = 0, k > len, doppelte Insert-IDs: je Fall ein Test; erwartet Err, nie Panic.
H10 [VectorIndex-Trait, vector_index_impl.rs]: commit/rollback/rollback_to_tx und search_at(seq) im MVCC-Zusammenspiel: Insert in Tx, Rollback → Vektor nirgends auffindbar; Commit → auffindbar; search_at(alter seq) sieht den Vektor nicht.

Ergebnisse je Hypothese mit Status + Beweisstufe + Fundstelle. Rote Beweis-Tests mit #[ignore = "<Bug-ID>..."] einchecken.

[[REPORT]]
```

### J-24 — Produkt 3 (Vektor-Index), Teil B: Quantisierung, SIMD, Persistenz, DiskANN

**Typ:** Test · **Dauer:** 90–180 min · **Schreibt:** Bericht + `crates/contextra-vector/tests/campaign_quant_*.rs`, `crates/contextra-simd/tests/campaign_simd_*.rs`

```text
[[CTX]]

AUFTRAG: Prüfe Quantisierung, SIMD-Numerik, Persistenzformate und DiskANN. Quelle: PRODUKT-3-VEKTOR-INDEX-ANALYSE.md (K5–K8, BUG-VEC-05). Features: experimental-diskann, experimental-rabitq, experimental-predicate-augmented-search müssen aktiviert werden (cargo test -p contextra-vector --features ...).

H1 [SQ8, quantize/]: ScalarQuantizer::try_train/try_train_with_percentiles auf degenerierten Daten.
   Test: alle Vektoren identisch (Varianz 0 je Dimension), genau 1 Vektor, weniger Vektoren als Dimensionen, Ausreißer (1e30), NaN/Inf im Trainingsset, konstante Dimension neben variabler. Erwartet: Err oder wohldefiniertes Ergebnis, NIE Division durch Null mit NaN-Distanzen. quantize→dequantize Fehlerschranke (Rundungsfehler <= halbe Schrittweite je Dimension, unabhängig berechnet). asymmetric_dist/symmetric_dist gegen exakte f32-Distanz auf derselben Datenmenge: berichte mittleren und maximalen relativen Fehler.
H2 [Drift, check_drift/drift_ratio/is_rebuild_required]: Verschiebe die Datenverteilung nach dem Training (Mittelwert um 3 Sigma): drift_ratio muss steigen und is_rebuild_required(threshold) auslösen; recall vor/nach Rebuild berichten. Sq8Bias::calibrate: Bias vor/nach Kalibrierung gegen unquantisierte Referenz.
H3 [RaBitQ, quantize_rabitq.rs]: generate_orthogonal_matrix liefert R mit R·Rᵀ ≈ I (Fehler < 1e-4, unabhängig mit eigener Matrixmultiplikation prüfen); asymmetric_distance ist ein im Mittel erwartungstreuer Schätzer der exakten Distanz (Mittelwert des Fehlers über >= 10.000 Paare nahe 0 relativ zur Streuung; berichte Bias und Streuung); try_train auf degenerierten Daten wie in H1.
H4 [SIMD, contextra-simd und vector/distance.rs]: Pfad-Korrektheit über alle verfügbaren Dispatch-Pfade.
   Ermittle per /proc/cpuinfo, welche Pfade die VM ausführen kann (scalar, AVX2, AVX-512, NEON) und wie der Dispatch erzwungen/übersteuert werden kann (grep nach is_x86_feature_detected, cfg(target_feature), Feature-Flags). Teste jeden VERFÜGBAREN Pfad gegen eine skalare Eigen-Referenz (f64-Akkumulation) für Cosine, Euclidean, Dot, auch u8-quantisiert: Dimensionen {1,2,3,7,8,15,16,17,31,32,33,63,64,65,127,128,129,255,768,1536}, unaligned Slices (Offset 1,3,5 in einem größeren Puffer), Subnormals, ±0.0, NaN, ±Inf, Werte nahe f32::MAX (Überlauf). Toleranz aus der Fehlerfortpflanzung ableiten (nicht beliebig). Nicht ausführbare Pfade: als "NICHT GEPRÜFT (CPU-Feature fehlt)" listen, nicht als grün. Prüfe die unsafe-Insel: jeder unsafe-Block mit // SAFETY-Kommentar? check-unsafe-islands-Ergebnis zitieren; Widerspruch der Insel-Listen (capabilities.toml vs. AGENTS.md/Dev-Guide) als Befund.
H5 [K7, get_dist_mixed/get_vec_mixed]: RAM-Pfad (f32) vs. Mmap-Pfad (ggf. quantisiert) liefern Rundungsunterschiede < 1e-6, die bei Gleichstand die Top-K-Reihenfolge ändern können.
   Test: Gleicher Datensatz im RAM- und im Mmap-Modus: Top-K-MENGE muss identisch sein; Reihenfolgeabweichungen nur bei Distanzdifferenz < 1e-5; berichte Anzahl und Ursache. Prüfe, ob die Ergebnisse für search_at deterministisch sind.
H6 [Persistenz HNSW, persistence/]: save → load_mmap Roundtrip (Graph, Vektoren, last_tx_id, Quantisierung, v1 und v2/v2_with_bias): Suchergebnisse identisch vor/nach Roundtrip. Header-Korruption: kippe JEDES der ersten 128 Header-Bytes einzeln (und Magic/Version/Dimension/node_count/entry_point auf Extremwerte): load_mmap muss Err liefern, nie panic, nie Out-of-Bounds-Zugriff, nie riesige Allokation (Peak-RSS messen). NodeRecord::from_bytes mit zu kurzem/zu langem Puffer.
H7 [K5, DiskANN init_hnsw_fallback]: Korrupter DiskANN-Header → Fallback auf RAM-HNSW ohne Datenverlust (DiskAnnFallbackPolicy). Test: Header-Byte-Flips und abgeschnittene Datei; jede Policy-Variante; Vergleich der Suchergebnisse mit dem Original-Index (Referenz: NaiveReferenceIndex). Footer-HMAC: manipulierter Footer wird erkannt.
H8 [K6/BUG-VEC-05, diskann/build.rs ~Z. 310]: Pending-WAL/Tombstone-WAL haben keine gemeinsame Sequenznummer mit dem LSM-WAL; kombinierter Crash kann doppelte Inserts erzeugen.
   Test (logisch): append_to_pending_wal mehrfach, "Crash" (Drop ohne persist_delta), recover_pending_delta: jeder Vektor genau einmal vorhanden; wiederholtes recover_pending_delta ist idempotent. Doppeleinspielung derselben Pending-WAL-Einträge (simuliert gleichzeitige LSM-Recovery) erzeugt keine Duplikate in all_doc_ids. Tombstone-WAL: Soft-Delete überlebt Reopen; prune_in_memory/prune_streaming entfernen nur markierte Knoten; TombstoneSet-Konsistenz.
H9 [K8, ComputePool]: Sättigung mit 1000 Aufgaben bei 2 Workern, Aufgaben mit Panic: catch_unwind liefert Err, Worker bleibt lebendig, nachfolgende Aufgaben laufen; Drop während Last hängt nicht (Watchdog).
H10 [VectorCandidateStream]: next_batch/next_doc/refill: Streaming-Top-K entspricht der materialisierten Top-K-Suche (Mengenvergleich), auch bei Löschungen zwischen Batches.

Je Hypothese Status + Beweisstufe + Fundstelle; rote Beweise mit #[ignore = "<Bug-ID>..."].

[[REPORT]]
```

### J-25 — Produkt 4 (Krypto), Teil A: Primitive, Known-Answer-Tests, AEAD, KDF, Nonces

**Typ:** Test · **Dauer:** 90–150 min · **Schreibt:** Bericht + `crates/contextra-crypto/tests/campaign_prim_*.rs`

```text
[[CTX]]

AUFTRAG: Prüfe die kryptografischen Primitive von contextra-crypto gegen UNABHÄNGIGE Quellen. Quelle: PRODUKT-4-KRYPTO-COMPLIANCE-ANALYSE.md. Die Vor-Analyse bescheinigt hohe Qualität; bestätige das mit eigenen Beweisen statt es zu übernehmen.

WICHTIG (R5): Erfinde KEINE Testvektoren aus dem Gedächtnis. Beschaffe sie aus Dateien: ~/.cargo/registry/src/*/ (Test-Verzeichnisse der Abhängigkeiten, z. B. ed25519-dalek, aes-gcm-siv, argon2, blake3, hmac/sha2), RFC-Texte falls erreichbar, oder Dateien im Repo, die NICHT von Contextra selbst erzeugt wurden. Nenne je Vektor den Fundort (Pfad:Zeile). Ist keine Quelle verfügbar: "Vektor nicht verfügbar" berichten.

H1 [Known-Answer-Tests]: Wandle offizielle Vektoren in Tests um, die ÜBER die Contextra-API laufen (nicht nur die Abhängigkeit direkt): Ed25519 (RFC 8032), AES-256-GCM-SIV (RFC 8452), Argon2id (RFC 9106), BLAKE3 (offizielle test_vectors), HMAC-SHA256 (RFC 4231). Wo Contextra eine eigene Verpackung nutzt (derive_*_key, Domänentrennung, längenpräfixiertes Hashing), prüfe die Verpackung durch Gegenrechnung mit der Rohbibliothek in einem unabhängig geschriebenen Hilfscode (andere Struktur als Produktionscode).
H2 [AEAD-Manipulation]: encrypt_auto_nonce/decrypt_auto_nonce: Kippe JEDES Bit von Ciphertext+Tag (bei Klartext <= 64 Byte vollständig, sonst Stichprobe >= 2000 Positionen), AAD-Änderung, Abschneiden, Anhängen, leerer Klartext, 1 Byte, 1 MiB: jede Manipulation muss Err liefern; Klartext wird bei Fehler nie (auch nicht teilweise) zurückgegeben.
H3 [Nonce-Disziplin]: Erzeuge >= 1.000.000 (bei Zeitüberschreitung 200.000) Verschlüsselungen unter einem Schlüssel; erkenne Nonce-Wiederholungen (HashSet). Prüfe, wie encrypt_auto_nonce Nonces bildet (Zufall vs. Zähler) und ob Zufalls-Nonces bei der erlaubten Nachrichtenzahl pro Schlüssel das Geburtstagsrisiko vertragen (GCM-SIV ist nonce-missbrauchsresistent, aber Wiederholung leakt Gleichheit identischer Klartexte — belege per Test, dass gleiche Klartexte bei Nonce-Wiederholung gleiche Chiffrate ergeben, und bewerte die Auswirkung). Prüfe, ob die Nonce-Quelle ein echter CSPRNG ist (P28-Ausnahme) und NICHT der deterministische Rng-Port (Mustersuche rg: thread_rng, SystemTime, Rng-Port im Krypto-Pfad).
H4 [Schlüsselhierarchie, KeyManager]: derive_file_key, derive_segment_key, derive_kv_key, derive_kv_key_scoped (Tenant), cipher_for/cipher_for_scoped (Modell-Fingerprint), derive_deletion_proof_key, integrity_key: Schlüssel sind (a) deterministisch bei gleichem Kontext, (b) paarweise verschieden bei verschiedenem Kontext (Tenant 1 vs. 2, Modell A vs. B, Segment 1 vs. 2, Zweck file vs. kv) über >= 10.000 Kontexte ohne Kollision, (c) ein mit Modell-A-Schlüssel erzeugtes Chiffrat lässt sich mit Modell-B-Schlüssel NICHT entschlüsseln (Cross-Model-Key-Reuse-Schutz).
H5 [KDF, kdf.rs]: KdfParams::new und KdfHeader::from_bytes mit absurden Werten (m_cost = u32::MAX, t_cost = 0, p = 0, p sehr groß, Salt-Länge 0 oder zu lang): from_bytes darf keine Argon2-Berechnung mit Terabyte-Speicher anstoßen (DoS durch manipulierten Header) — erwartet Err VOR der Berechnung. Mindestparameter (Untergrenzen) vorhanden? KdfHeader to_bytes/from_bytes Roundtrip. Zeit für die Default-Parameter relativ messen (nur als Größenordnung).
H6 [Zeroize-Logik, anti_tamper.rs]: VolatileEncryptionKey: Debug-Ausgabe enthält keine Schlüsselbytes (Test mit format!("{:?}")); emergency_wipe macht danach jede Nutzung zu einem Err/wohldefiniert; KeyManager::emergency_wipe. (Der Speicher-Beweis im Prozessabbild läuft lokal in L-23, weil unsafe_code in Tests nicht erlaubt ist.)
H7 [Konstante Zeit]: Finde alle Vergleiche von MACs/Tags/Hashes/Signaturen (rg: "==" auf [u8; 32]/Vec<u8> in crypto/privacy/store-HMAC-Code, verify_*, ct_eq). Jeder nicht konstantzeitige Vergleich geheimer Werte = Befund (S2). Belege je Fund Datei:Zeile. (Statistische Zeitmessung lokal in L-34.)
H8 [WAL-Verschlüsselung, wal_crypto.rs]: EncryptedWal::encrypt_chunk/decrypt_chunk Roundtrip; Chunk-Reordering/Duplikation/Weglassen (verify_wal_chain_completeness, IntegrityVerifier::verify_and_update_v2/_v3) muss erkannt werden; skip_hmac_verify_legacy ist niemals implizit aktivierbar (INV-WAL-LEGACY-KEY-1): Test, dass der Legacy-Pfad nur bei explizitem Opt-in erreichbar ist.
H9 [TenantIsolatedKvStore, kv_segment/]: Mandant A kann Segmente von Mandant B weder lesen (get_segment_bytes/get_decrypted_segment) noch durch Eviction verdrängen (evict_lru_fair vs. evict_lru_global: Fairness-Test mit einem "lauten" Mandanten); Rollback-Cleanup (on_rollback/remove_segments_for_rollback) entfernt genau die Segmente der betroffenen Tx; EvictionWorker::emergency_wipe leert den Store.
H10 [Duplikat-Invariante INV-CRYPTO-DUPLICATE-1]: Suche doppelte Hash-/KDF-/Tag-Berechnungen in contextra-crypto, -store und -privacy (rg nach blake3::, Sha256, Hmac::new): zwei Implementierungen derselben Funktion = Befund S3.

Je Hypothese Status + Beweisstufe + Fundstelle; Quellen aller Vektoren nennen.

[[REPORT]]
```

### J-26 — Produkt 4 (Krypto), Teil B: Crypto-Shredding, Revocation-Log, Audit-Kette, Löschnachweis

> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-agent`, `contextra-privacy`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.

**Typ:** Test · **Dauer:** 90–180 min · **Schreibt:** Bericht + `crates/contextra-crypto/tests/campaign_shred_*.rs`

```text
[[CTX]]

AUFTRAG: Prüfe Löschung durch Schlüsselvernichtung, Widerrufsprotokoll, Audit-Kette und Löschnachweis auf die in PRODUKT-4-KRYPTO-COMPLIANCE-ANALYSE.md genannten Schwächen. Jede Hypothese ist ein Test; ROT auf unverändertem Code = Befund bestätigt.

H1 [SEC-03/K3, S2, kv_shredding.rs ~Z. 188]: KeyRegistry::revoke_group/revoke_subkey/revoke_record löschen den Schlüssel im RAM, schreiben aber keinen Eintrag in das persistierte RevocationLog (Split-Brain nach Absturz).
   Test: KeyRegistry und RevocationLog (open_or_create auf Temp-Datei) gemeinsam nutzen; revoke_group aufrufen; ohne manuelles append: "Absturz" (Drop), Neuaufbau aus dem Log: ist der Widerruf rekonstruierbar (is_revoked)? Erwartung korrekt: JA. Prüfe, ob irgendein Aufrufer im Workspace (rg: revoke_group|revoke_record|revoke_subkey) das Log zuverlässig mitführt oder ob es nur in Tests geschieht. Belege die Kette Datei:Zeile.
H2 [Kaskade]: revoke_group macht ALLE abgeleiteten RecordDeks unbrauchbar (decrypt_record schlägt fehl), ohne andere Gruppen zu berühren; is_group_revoked/is_group_active/is_key_active/is_record_active konsistent; revoke_record betrifft nur diesen Datensatz; ein widerrufener Schlüssel kann nicht über get_or_derive "wiederauferstehen" (Re-Derivation aus dem Master nach Widerruf darf nicht funktionieren).
H3 [RevocationLog-Integrität]: verify_integrity/verify_chain_entries gegen: Bit-Flip in jedem Eintrag (Stichprobe >= 500), Eintrag löschen (Mitte/Ende/Anfang), Einträge vertauschen, Eintrag duplizieren, Datei abschneiden, falscher Schlüssel: jede Manipulation muss erkannt werden; Verhalten fail-closed (unlesbares Log ⇒ is_revoked darf NICHT "nicht widerrufen" antworten).
H4 [SEC-01/K1, S1, audit_chain.rs ~Z. 252]: AuditChain::verify_chain prüft nur interne Konsistenz; Einspielen eines älteren, intakten Backups (Rollback der Kette) wird nicht erkannt, solange kein externer Head-Anker existiert.
   Test: Kette mit 20 Einträgen, speichere Zustand nach Eintrag 10; füge bis 20 fort; ersetze Zustand durch den Stand 10 (inkl. head_hash) und rufe verify_chain/verify_head_signature: erwartet wird Erfolg (Befund bestätigt). Teste zusätzlich: Eintrag verändern, Eintrag entfernen/vertauschen, falscher HMAC-Schlüssel, Kopf-Signatur austauschen, Signatur eines anderen Schlüssels — alle diese Fälle müssen erkannt werden. Dokumentiere, welche externe Verankerung (Zeitstempel, Hash an anderem Ort, Zähler im Manifest) eine Rollback-Erkennung ermöglichen würde (Text).
H5 [K2, compute_record_commitment]: bindend und verbergend: verschiedene Attribute ⇒ verschiedene Commitments (>= 100.000 zufällige Paare, keine Kollision), gleiches Attribut mit verschiedenem Salt ⇒ verschiedene Commitments; ohne Salt ist das Attribut nicht per Wörterbuch aus dem Commitment ableitbar (Test: Angreifer kennt Wörterbuch aber nicht den Salt). EncryptedCommitmentSalt encrypt/decrypt Roundtrip und Manipulation.
H6 [DeletionProof v3, deletion_proof.rs/ed25519_proof.rs]: (a) Längenpräfix-Kollision: Eingaben (a, bc) und (ab, c) ergeben verschiedene Hashes/Signaturen. (b) K4 Downgrade: verify_external lehnt signature_version 1 und 2 sowie unbekannte Versionen ab (CryptoError::UnsupportedProofVersion bzw. äquivalent); SignatureVersion::try_from(u8) für alle 256 Werte. (c) Manipulation je Feld (Scope, Layer, Zeitstempel, DocId, TenantId, WAL-Receipt, Audit-Position): Verifikation schlägt fehl. (d) Replay: Proof von Mandant A für Dokument X ist für Mandant B/Dokument Y ungültig. (e) LayerCleanupProof::new_after_verified_empty/verify_and_create: nur nach nachweislich leerer Schicht; Versuch bei NICHT leerer Schicht muss scheitern. (f) compute_wal_delete_receipt/verify_wal_delete_receipt Roundtrip und Manipulation. (g) export_for_audit() enthält keinen Schlüssel- oder Klartext.
H7 [INV-DELETION-1, Ende-zu-Ende]: DeletionProof::create (und create_v3 …) darf nur NACH physischer Bereinigung erzeugbar sein. Prüfe alle Erzeuger-Aufrufstellen im Workspace (rg: DeletionProof::create|create_v3|create_full_v3|generate_deletion_proof) und bewerte je Aufrufstelle, ob die Bereinigung davor erzwungen wird (Typ-Zustand, Parameter-Beweis, Laufzeitprüfung) oder nur "per Konvention". Konvention ohne Erzwingung = Befund S2 (vgl. S2-02 in Produkt 1).
H8 [Drei Audit-Ketten, SEC-05/K8]: AuditChain (contextra-crypto), ContextEditAuditRecord (contextra-privacy⚠, verify_audit_chain), AgentAudit (contextra-agent⚠): Dokumentiere Genesis-Hash, Schlüssel und Verifikationsweg jeder Kette; prüfe, ob irgendeine Querverbindung existiert. Liste Redundanzen als Text mit Datei:Zeile (reiner Architekturbefund S3).
H9 [Revocation + Lesepfad]: Nach revoke_* darf KEIN Lesepfad (get_decrypted_segment, KV-Cache-Lookup, decrypt_with_group) Klartext liefern; Fehler ist ein wohldefinierter Err. Gleiches nach emergency_wipe.

Je Hypothese Status + Beweisstufe + Fundstelle; rote Beweis-Tests mit #[ignore = "<SEC-ID>: ..."].

[[REPORT]]
```

### J-27 — Produkt 4, Teil C: Privacy-Egress, Prompt-Injection, MCP, DSGVO-Generatoren

> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-audit-export`, `contextra-avv-generator`, `contextra-mcp`, `contextra-privacy`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.

**Typ:** Test · **Dauer:** 90–180 min · **Schreibt:** Bericht + `crates/contextra-privacy⚠/tests/campaign_egress_*.rs`, `crates/contextra-mcp⚠/tests/campaign_mcp_*.rs`, `crates/contextra-audit-export⚠/tests/campaign_*.rs`

```text
[[CTX]]

AUFTRAG: Prüfe die Egress-Firewall (contextra-privacy⚠), den MCP-Server (contextra-mcp⚠) und die DSGVO-Generatoren (contextra-audit-export⚠, contextra-avv-generator⚠). Quellen: PRODUKT-4-KRYPTO-COMPLIANCE-ANALYSE.md (SEC-04, K6, K7), CONTEXTRA_4KERNSYSTEME_TIEFENSPEZIFIKATION.md (4.7, 4.8, Hinweis zum offenen MAJOR-Befund normalize_text), CONTEXTRA_FEATURE_KATALOG_ERWEITERT.md (Teil VI).

H1 [SEC-04/K6, S2, egress_gateway.rs ~Z. 43]: CloudResponseRehydrator::rehydrate ersetzt jedes [USER_ENTITY_xxxx]-Token im Antworttext blind durch den Vault-Klartext; eine bösartige Cloud kann fremde Entitäten per Token-Injektion provozieren.
   Test: Vault mit zwei Entitäten (E1, E2) aus ZWEI verschiedenen Sessions/Anfragen; Cloud-Antwort enthält (a) ein gültiges Token von E1 aus DIESER Anfrage (soll ersetzt werden), (b) ein Token von E2 aus einer ANDEREN Session (soll NICHT ersetzt werden), (c) ein nie ausgegebenes Token, (d) Token in veränderter Schreibweise/Unicode/Zero-Width-getrennt. Erwartet korrekt: nur (a). Zeigt der Test, dass (b) ersetzt wird: Befund bestätigt. Prüfe auch Mandantengrenzen (sanitize_and_vault_scoped, classify_scoped, check_scoped, handle_cloud_query_scoped).
H2 [Layer-1-Pattern-Vault, egress_vault.rs]: Korpus personenbezogener und geheimer Daten: E-Mail, IBAN (DE, mit/ohne Leerzeichen), Telefonnummern (national/international), Steuer-ID, Kreditkartennummern (Luhn-gültig), IPv4/IPv6, API-Schlüssel typischer Formate, JWT, PEM-Privatschlüssel, AWS-Zugangsdaten-Muster. Erwartet: blockiert oder surrogiert; berichte Erkennungsquote je Kategorie. Evasion-Varianten: Unicode-Homoglyphen (kyrillisches а statt lateinischem a), Zero-Width-Zeichen, Fullwidth-Ziffern, Leerzeichen/Bindestriche mittendrin, Base64/Hex-kodiert, auf zwei Sätze verteilt, RTL-Override. Jede umgangene Variante = Befund mit Schwere nach Tragweite. Erfinde keine echten Zugangsdaten; nutze offensichtlich synthetische Platzhalter im jeweils gültigen Format.
H3 [Prompt-Injection, contextra-mcp⚠::prompt_injection, normalize_text — offener MAJOR-Befund laut Analysebericht]: Homoglyphen-Bypass.
   Test: Injektions-Sätze ("ignore previous instructions", "System prompt:", deutsche Varianten) im Klartext (muss erkannt werden) und mit Homoglyphen/Zero-Width/Fullwidth/Leetspeak/Zeichentrennung/HTML-Entities/Markdown-Verschleierung (Erkennung?). Berichte je Variante erkannt/nicht erkannt. Prüfe, ob normalize_text eine Unicode-Normalisierung (NFKC) UND eine Confusables-Abbildung nutzt (Datei:Zeile). Nicht erkannte Varianten = Befund bestätigt.
H4 [Layer 4 EgressGuard, egress_guard.rs]: EgressGuard::check/check_scoped ist fail-closed bei Timeout und Indexfehler.
   Test: Mock-TextSearchEngine, der (a) Err liefert, (b) hängt (Timeout), (c) leere Treffer liefert, (d) Treffer mit hoher Ähnlichkeit liefert: erwartet blockieren in (a), (b), (d); (c) nur wenn das Konzept es erlaubt (dokumentiere). Kein Panic bei Timeout.
H5 [BulkExfiltrationDetector, bulk_exfiltration_detector.rs]: record_and_check: Sliding-Window-Volumenlimit pro Session; fail-closed bei Zeit-Regression.
   Test mit injizierter Clock: Limit exakt erreicht/überschritten; Fensterrand; Clock springt rückwärts (muss blockieren); 32 Threads parallel (Race-Sicherheit: Gesamtzahl erlaubter Anfragen darf das Limit nie überschreiten); Sessions isoliert.
H6 [Type-State, guarded_payload.rs]: GuardedPayload<Unsanitized> kann nicht versendet werden. Schreibe Compile-Fail-Tests als Doc-Tests (Rust-Doc-Kommentar-Codeblock mit dem Attribut compile_fail) oder per trybuild, falls als Dev-Dependency vorhanden. Beleg: der Versandweg akzeptiert nur Sanitized.
H7 [K7, pii_vault_forces_crypto_shred / resolve_effective_kv_delete_mode]: Wahrheitstabelle {PII-Treffer ja/nein} x {konfigurierter KvDeleteMode} x {DeletionProof aktiv ja/nein}: PII-Treffer erzwingt CryptoShred; INV-KV-DELETE-1 (DeletionProof-KV-Seite nur mit CryptoShred); INV-COLLECTION-PROFILE-1/2 (aktiver DeletionProof erzwingt CryptoShred); INV-PERF-PROFILE-1 (BareMetal/Balanced mit DeletionProof inkompatibel); INV-LICENSE-2 (Fast-Ring ohne Lizenz prüfbar). Jede unzulässige Kombination muss abgelehnt werden.
H8 [Audit-Spur, audit_trace.rs/context_edit_audit.rs]: compute_audit_trace ist deterministisch (gleiche Eingabe → gleiche Spur) und ändert sich bei jeder Änderung von Payload, Rule-ID oder Zeitstempel; verify_audit_chain erkennt Manipulation; render_audit_line enthält keinen Klartext-PII.
H9 [MCP-Werkzeuge, contextra-mcp⚠, Feature test-utils]: 11 aktive Werkzeuge (contextra_search, _insert, _get, _forget, _collections, _consolidate, _cloud_query, _relate, _relate_n_ary, _plugin_status, _explain) und 4 "Phantom"-Namen (contextra_upsert, _delete, _create_collection, _drop_collection; im Katalog 🔴 Phantom). Test über den In-Prozess-Dispatch: jedes aktive Werkzeug mit gültiger und ungültiger Eingabe; die 4 Phantom-Namen müssen einen sauberen JSON-RPC-Fehler liefern, nie Panic; prüfe, ob die Sandbox-Klassifikation Namen kennt, die der Dispatch nicht kennt (Katalog-Befund P0-4 bestätigen). Grenzen: relate_n_ary mit 63/64/65 Teilnehmern; k/limit = 0, 1, u32::MAX; überlange Strings; ungültiges UTF-8 (falls über die API erreichbar); tief verschachteltes JSON (Rekursionsgrenze, Stack-Überlauf?); contextra_forget liefert bei aktivem Sovereign-Ring einen Löschbeweis; Mandantentrennung bei contextra_search/_get.
H10 [Art. 30 DSGVO, processing_registry.rs, contextra-audit-export⚠]: generate_registry/render_markdown/render_register_json/render_register_markdown: Ausgabe ist deterministisch; JSON ist gültig (parse mit serde_json im Test); enthält die Pflichtangaben des Verarbeitungsverzeichnisses nach Art. 30 Abs. 1 lit. a–g (Verantwortlicher, Zwecke, Kategorien Betroffener und Daten, Empfängerkategorien, Drittlandübermittlung, Löschfristen, technische/organisatorische Maßnahmen) — prüfe als technische VOLLSTÄNDIGKEITSPRÜFUNG, nicht als Rechtsberatung. Canary-Test: ein eingeschleuster Geheimniswert (z. B. "CANARY-SECRET-1234") in Eingabedaten taucht in keiner Ausgabe auf. bsi_mapping_table/render_bsi_mapping_markdown: jede referenzierte Maßnahme verweist auf existierenden Code (Datei:Zeile) oder wird als unbelegt markiert.
H11 [Art. 28 DSGVO, contextra-avv-generator⚠]: render_avv_markdown/render mit AvvContext/default_technical_measures: prüfe technische Vollständigkeit gegen die Pflichtinhalte nach Art. 28 Abs. 3 lit. a–h (Gegenstand/Dauer, Art/Zweck, Datenarten, Betroffenenkategorien, Weisungsgebundenheit, Vertraulichkeit, Maßnahmen nach Art. 32, Unterauftragsverarbeiter, Unterstützungspflichten, Rückgabe/Löschung, Kontrollrechte). Jede im generierten Text behauptete technische Maßnahme (z. B. "Ende-zu-Ende-Verschlüsselung", "Crypto-Shredding") gegen den Code belegen: nicht belegte Zusage = Befund S3 (Wahrheit der Compliance-Aussage). Keine rechtliche Bewertung.

Je Hypothese Status + Beweisstufe + Fundstelle; rote Beweise mit #[ignore = "<SEC-ID>: ..."].

[[REPORT]]
```

---

## A.4 Benchmarks in der VM

> **Wichtig:** Eine Jules-VM hat geteilte CPUs, unbekannte Taktung und laute Nachbarn. Absolute Zeiten (ns, MB/s, QPS) aus der VM sind **nicht belastbar** und werden nie als Produktkennzahl berichtet. Belastbar sind: **Recall/Qualitätsmetriken** (hardware-unabhängig), **Skalierungsexponenten** (relative Steigung über mehrere Größen), **Allokations-/Speicherzahlen**, und **relative Vergleiche innerhalb derselben Session**. Absolute Messungen laufen lokal (Teil B.3).

### J-30 — Criterion-Smoke und Regressions-Gate (relativ, mit Rauschmessung)

> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-checkpoint`, `contextra-db`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.

**Typ:** Benchmark · **Dauer:** 60–150 min · **Schreibt:** nur Bericht

```text
[[CTX]]

AUFTRAG: Führe die Criterion-Benchmarks im Release-Profil als Smoke- und Regressionsprüfung aus und bestimme zuerst das Messrauschen der VM. Berichte KEINE absoluten Leistungsversprechen.

SCHRITT 1 — Rauschmessung: Führe einen kleinen, schnellen Benchmark (z. B. aus contextra-crypto crypto_benchmarks) DREIMAL nacheinander mit identischer Konfiguration aus (--save-baseline run1/run2/run3) und berechne die Streuung zwischen den Läufen (Median-Differenz in Prozent). Ist die Streuung > 10 %, dann ist das ±5 %-Regressions-Gate (just bench-delta) in dieser VM statistisch bedeutungslos — halte das ausdrücklich im Bericht fest und bewerte Regressionen nur oberhalb der gemessenen Streuung.
SCHRITT 2 — Ausführung (nur Release; Criterion-Targets sind optimiert; alle Befehle mit timeout 30m und Logdatei):
  cargo bench -p contextra-db⚠ --bench scale_bench -- --warm-up-time 1 --measurement-time 3 --sample-size 20
  cargo bench -p contextra-db⚠ --bench rrf_scale_bench -- (gleiche Zeitparameter)
  cargo bench -p contextra-db⚠ --bench competitive_bench -- (gleiche Zeitparameter; benötigt ggf. Vergleichs-Engines als Dev-Dependencies — Netz/Build-Fehler als ENV melden)
  cargo bench -p contextra-db⚠ --bench migration_benchmarks -- (gleiche Zeitparameter)
  cargo bench -p contextra-checkpoint⚠ --bench checkpoint_bench -- (gleiche Zeitparameter)
  cargo bench -p contextra-crypto --bench crypto_benchmarks -- (gleiche Zeitparameter)
  cargo bench -p contextra-crypto --bench deletion_proof_latency_bench -- (gleiche Zeitparameter)
  Weitere Benches: ermittle alle [[bench]]-Einträge im Workspace (rg "\[\[bench\]\]" -g Cargo.toml) und ergänze Fehlendes (z. B. relate_bench).
SCHRITT 3 — Gate: just bench-delta bzw. ⚠NICHT-IM-REPO(cargo xtask bench-gate) --tolerance 0.05 --baseline benchmarks/results/criterion-baseline.json --current criterion/current. Gilt die Baseline-Datei noch? (Alter, Toolchain, andere Hardware ⇒ Vergleich ist nur mit Vorbehalt gültig.) NICHT just bench-set-baseline ausführen.
SCHRITT 4 — Auswertung: je Benchmark: Median, Konfidenzintervall, Abweichung zur Baseline, Einstufung (innerhalb Rauschen / mögliche Regression / mögliche Verbesserung). Ausreißer und instabile Benchmarks (CI-Breite > 20 % des Medians) separat listen. Prüfe Benchmark-Qualität: misst der Bench wirklich die Operation (black_box um Ergebnis und Eingabe? Setup außerhalb der Messschleife? Datensatz groß genug, um den Cache zu überschreiten? Zufallsdaten mit festem Seed?). Schwache Benchmarks = Befund S3 mit konkretem Verbesserungsvorschlag (Text).

[[REPORT]]
```

### J-31 — Skalierungsexponenten und Speicherbedarf (relativ, hardware-unabhängig interpretierbar)

> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-graph`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.

**Typ:** Benchmark · **Dauer:** 60–150 min · **Schreibt:** Bericht (+ optional Wegwerf-Messcode in `/tmp`)

```text
[[CTX]]

AUFTRAG: Bestimme das empirische Skalierungsverhalten zentraler Operationen über mindestens 4 Größenstufen und leite Exponenten ab. Exponenten und Speicher je Element sind weitgehend hardware-unabhängig und deshalb in der VM belastbar. Alles als Wegwerf-Messprogramm unter /tmp/J-31 (eigenes Cargo-Projekt mit path-Dependency auf die Crates, Release-Profil). Keine Repo-Änderung außer dem Bericht.

MESSGRÖSSEN (jede Stufe 3x wiederholen, Median verwenden; Stufen so wählen, dass die größte Stufe < 5 min dauert):
 a) contextra-vector HNSW: Insert-Zeit pro Element und Suchzeit (k=10, ef=64) bei N = 1k, 4k, 16k, 64k (Dimension 128, M=16). Erwartet: Suche ~ O(log N), Insert ~ O(log N). Passe log-log-Regression (Steigung) an; berichte Steigung und R². Speicher je Knoten (RSS-Differenz / N) und vergleiche mit der Analyse-Aussage ~3,4 KB/Knoten bei 768d f32, M=16 (miss auch 768d bei kleineren N).
 b) contextra-text BM25: Indexierungszeit und Abfragezeit bei 10k, 40k, 160k, 640k kurzen Dokumenten (Zipf-verteilte synthetische Terme); Speicher je Dokument.
 c) contextra-graph⚠ CSR: Aufbauzeit, Nachbarschaftsabfrage, PPR bei 10k, 40k, 160k, 640k Kanten.
 d) contextra-store: Put-Durchsatz (relativ) und Lese-Latenz bei 100k, 400k, 1,6M, 6,4M Schlüsseln (Wertgröße 128 B), Write-Amplification = geschriebene Bytes (über /proc/self/io write_bytes oder VFS-Zähler) / Nutzdatenbytes; Anzahl SSTables/Level; Compaction-Verhalten.
 e) Löschung P24 ("Kosten proportional zur lokalen Nachbarschaft, nicht zur Indexgröße"): HNSW remove_with_graph_repair bei N = 4k, 16k, 64k: Zeit pro Löschung muss ~konstant (bzw. log) sein. Wächst sie linear mit N: Befund S2 (verletzt P24).
 f) contextra-mvcc: SequenceLog/SnapshotRegistry/SSI-Validator-Operationen bei wachsender Anzahl aktiver Snapshots und Commit-Register-Größe (max_tracked_keys): O(1)-Sichtbarkeitsprüfung wirklich konstant?
AUSWERTUNG: Tabelle Operation | Soll-Komplexität (aus Spezifikation/Analyse) | gemessene Steigung | R² | Urteil. Abweichungen > 0,3 in der Steigung mit Beschreibung. Speicher je Element als Tabelle. Die Behauptung "Milliarden Vektoren" prüfen: rechne aus dem gemessenen Speicher je Knoten den RAM-Bedarf für 10^6, 10^8, 10^9 Knoten hoch und kennzeichne die Hochrechnung als [VERMUTET] (Extrapolation), nicht als Messung.

[[REPORT]]
```

### J-32 — Recall- und Qualitäts-Benchmarks (hardware-unabhängig)

> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-bench`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.

**Typ:** Benchmark · **Dauer:** 60–120 min · **Schreibt:** nur Bericht

```text
[[CTX]]

AUFTRAG: Führe die Qualitäts-Benchmarks aus (Recall, Ranking-Güte, Stabilität), die unabhängig von der Hardware aussagekräftig sind.

SCHRITT 1 — just bench-regression (LongMemEval-Regressionssuite; cargo run -p contextra-bench --release) und just bench-external (synthetische BEIR-/ANN-Benchmarks ohne Download). ⚠NICHT-IM-REPO(cargo xtask bench-compile); ⚠NICHT-IM-REPO(cargo xtask bench-download) NUR falls das Netz es erlaubt (sonst Umgebungslücke). ⚠NICHT-IM-REPO(cargo xtask check-recall-stability) und check-bandit-latency-budget (letzteres ist zeitabhängig: 3 Läufe, Streuung berichten, in der VM nur eingeschränkt belastbar).
SCHRITT 2 — Ergebnisse als Tabellen: recall@k, nDCG@10 bzw. die von der Suite ausgegebenen Metriken, je Datensatz. Vergleiche mit vorhandenen Referenzwerten/Baselines im Repo (benchmarks/results/**): Abweichungen und Trends (⚠NICHT-IM-REPO(cargo xtask bench-trend)).
SCHRITT 3 — Determinismus: Führe jede Suite zweimal mit identischem Seed aus; Metriken müssen bitgleich oder innerhalb eines dokumentierten Epsilons sein. Abweichungen = Befund S3 (Nichtdeterminismus verletzt P28).
SCHRITT 4 — Güte der Benchmarks selbst: Ist der Datensatz "synthetisch" in einer Weise, die den Algorithmus begünstigt (z. B. gut separierte Cluster)? Gibt es Datenleck zwischen Index- und Anfragemenge? Wird recall gegen exakte Brute-Force-Wahrheit berechnet oder gegen sich selbst? Kennzeichne Aussagen zur Übertragbarkeit auf reale Daten als [VERMUTET].
SCHRITT 5 — Router/Rank: Prüfe, falls Metriken vorhanden, den Einfluss von RRF vs. kalibrierten Scores (Isotonic/Platt) und der Bandit-Routing-Entscheidungen auf die Qualität (Ablationsvergleich, sofern die Suite Schalter anbietet). Keine Hyperparameter-Änderung im Repo.

[[REPORT]]
```

---

## A.5 Audits

### J-40 — Supply-Chain-, Lizenz- und Unsafe-Audit

> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-wire`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.

**Typ:** Audit · **Dauer:** 30–90 min · **Schreibt:** nur Bericht

```text
[[CTX]]

AUFTRAG: Prüfe Lieferkette, Lizenzen und unsafe-Nutzung. Keine Änderung an deny.toml oder Cargo.toml/Cargo.lock.

SCHRITT 1 — just security-audit (cargo-audit gegen RUSTSEC, --deny warnings) und cargo deny check (advisories, bans, licenses, sources), ⚠NICHT-IM-REPO(cargo xtask security-scan), ⚠NICHT-IM-REPO(cargo xtask gen-sbom) (nur ausführen, Ergebnis in /tmp ablegen; erzeugt es Repo-Dateien: berichten, nicht committen). Ist die Advisory-DB offline nicht ladbar: Umgebungslücke, nicht "grün".
SCHRITT 2 — Abhängigkeitsbild: cargo tree -d (doppelte Versionen), cargo tree --edges normal --depth 1 pro Ring-0-Crate (Ring 0 soll möglichst wenige und keine Netzwerk-/Async-Abhängigkeiten haben: P26 kein tokio in Ring 0 — belege per cargo tree -p <ring0-crate> -i tokio für jeden Ring-0-Crate), Crates mit build.rs (potenzielle Build-Zeit-Codeausführung), Crates aus git/path statt crates.io, yanked-Versionen, unmaintained-Hinweise. Netzwerk-fähige Abhängigkeiten in Ring 0–1 (reqwest, hyper, ureq, curl): Befund S2 für den Air-Gap-Anspruch, falls vorhanden.
SCHRITT 3 — unsafe-Inventur: rg -n "unsafe" crates -g '*.rs' (Zählung je Crate und je Datei). Gegen die erlaubten Inseln abgleichen: capabilities.toml nennt contextra-simd, contextra-sys, contextra-wire; Dev-Guide/AGENTS.md nennen zusätzlich contextra-vector/src/distance.rs und sprechen von #![forbid(unsafe_code)] bzw. unsafe_code = "deny". Ermittle, welche Variante der Code tatsächlich lebt (Crate-Attribute #![forbid]/#![deny], Workspace-Lints in Cargo.toml) und liste JEDEN Widerspruch als Befund (S3). ⚠NICHT-IM-REPO(cargo xtask check-unsafe-islands) und lint-unsafe-slices zitieren. Je unsafe-Block: hat er einen // SAFETY-Kommentar, der die Invariante nennt? Blöcke ohne: Liste.
SCHRITT 4 — Geheimnisse im Repo: Suche nach eingecheckten Schlüsseln/Tokens (rg auf typische Muster, .env-Dateien, *.pem, *.key, hartcodierte Salt/Key-Konstanten in Produktionscode, Standardpasswörter in Beispielen/Tests, die in Produktion landen könnten). Test-Schlüssel in tests/ sind erlaubt, aber Produktionspfade mit festem Schlüssel sind S1.
SCHRITT 5 — CI-Härtung (nur lesen): ⚠NICHT-IM-REPO(cargo xtask check-action-pinning) und check-workflow-commands; sind GitHub-Actions auf Commit-SHA gepinnt? Bewertung der Berechtigungen der Workflows (permissions:).

[[REPORT]]
```

### J-41 — Tech-Debt-, Zero-Panic-, Determinismus- und Cast-Audit

> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-checkpoint`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.

**Typ:** Audit · **Dauer:** 45–120 min · **Schreibt:** nur Bericht

```text
[[CTX]]

AUFTRAG: Statischer Tiefenscan nach Verstößen gegen die Projektinvarianten und nach der Fehlerklasse, die den S1-Befund BUG-VEC-01 erzeugt hat (stille Zahlen-Verengung). Keine Code-Änderung.

SCHRITT 1 — Vorhandene Werkzeuge: just debt-audit (unwrap/unsafe/std::fs/Lock-Nesting via ast-grep), ⚠NICHT-IM-REPO(cargo xtask panic-inventory), check-result-dropped-io, check-nan-hot-loop, check-max-results-unbound, check-toctou-defaults. Ergebnisse zitieren.
SCHRITT 2 — Eigene Mustersuche (Fundzahlen je Crate, jeder Fund mit Datei:Zeile in /tmp/J-41/*.txt, im Bericht nur Top-Funde und Zählungen):
 a) Zero-Panic im Nicht-Test-Code: unwrap(), expect(, panic!, todo!, unimplemented!, unreachable!, Indexzugriffe auf Slices mit unvertrauten Indizes, Division durch möglicherweise 0, Slicing &x[a..b] mit externen Werten. Trenne Test-/Bench-/Example-Code sauber ab (#[cfg(test)], tests/, benches/, examples/, Features test-utils/fault-injection).
 b) Zahlen-Verengung: `as u8|u16|u32|u64|usize|i32|i64` auf Größen, die aus u128/u64/usize/Dateiinhalten/Netzwerkdaten stammen; besonders alles, was DocId, TxId, Sequenznummern, Offsets und Längenfelder betrifft. Klassifiziere jeden Fund: sicher (begründet) / Truncation-Risiko / Datenverlust-Risiko. Empfehle als Text try_from statt as.
 c) Determinismus P28: SystemTime::now, Instant::now, thread_rng, rand::random, Uuid::new_v4, std::env::var im Produktionscode außerhalb der Port-Implementierungen (Clock/Rng/IdGenerator). Ausnahme: Krypto-Schlüssel/Salt braucht CSPRNG — prüfe, dass dort NICHT der deterministische Port benutzt wird (Umkehrfehler).
 d) Sync-Kern P26: tokio/async in Ring 0 (rg "tokio|async fn|\.await" je Ring-0-Crate; contextra-vector ComputePool nutzt Threads, kein tokio: bestätigen).
 e) Dateizugriff am VFS vorbei: std::fs::, File::open, OpenOptions außerhalb der VFS-/FaultVfs-Abstraktion in contextra-store und contextra-checkpoint⚠. Jeder direkte Zugriff ist ein Blindfleck der Fehlerinjektion (Testbarkeit) und ein Befund S3.
 f) Ressourcengrenzen: unbeschränkte Kanäle (unbounded_channel, crossbeam unbounded), Vec::with_capacity/vec![0; n] mit n aus unvertrauter Eingabe, Rekursion ohne Tiefenlimit in Parsern, Hash-DoS (HashMap mit nutzerkontrollierten Keys und Standard-Hasher vs. fixer Hasher), Schleifen ohne Obergrenze.
 g) Verworfene Fehler: `let _ =` und `.ok();` auf Result bei I/O, Lock-Poisoning ignoriert, Fehlerklassifikation per String-Vergleich (err.to_string().contains).
 h) #[allow(...)]-Inventar: Anzahl und Lints; großflächige Unterdrückung (crate-weites #![allow]) ist Befund; liste unwrap_used/expect_used/panic-Allows mit Begründung.
 i) Veraltete/tote Teile: check-orphan-modules, check-phantom-files, check-placeholder-refs, TODO/FIXME/XXX/HACK-Zählung mit Datum falls vorhanden; Schein-Code (Funktionen, die berechnen, aber nirgends wirksam aufgerufen werden — Beispiel aus der Vor-Analyse: partial_rebuild.rs find_oversaturated_regions; MergeOperator ohne Aufruf in compaction/engine.rs): finde WEITERE pub-Funktionen/Traits ohne produktiven Aufrufer (rg + Aufrufgraph-Näherung), Muster "implementiert aber nicht verdrahtet".
SCHRITT 3 — Hotspots: ⚠NICHT-IM-REPO(cargo xtask hotspot-report) und die Änderungshäufigkeit (git log --since=... --name-only | sort | uniq -c | sort -rn | head -30). Verbinde Hotspots mit Fundstellen aus Schritt 2 und priorisiere die 10 riskantesten Dateien.

[[REPORT]]
```

### J-42 — Doku-Wahrheits- und Spezifikations-Abgleich (Katalog vs. Code)

**Typ:** Audit · **Dauer:** 60–120 min · **Schreibt:** nur Bericht

```text
[[CTX]]

AUFTRAG: Prüfe, ob Dokumentation, Feature-Katalog und Marketing-Aussagen dem Code entsprechen. Bei Konflikten gilt: capabilities.toml schlägt andere Doku; der Code schlägt die Doku. Keine Änderung an Dokumenten außer dem Bericht.

SCHRITT 1 — Behauptungsregister: Extrahiere aus docs/**, README.md, AGENTS.md, CONTEXTRA_FEATURE_KATALOG(_ERWEITERT).md, docs/SPEC.md ** und den Crate-READMEs alle überprüfbaren Behauptungen (Zahl der Crates, Zeilenzahlen, Features, Garantien, Invarianten INV-*, Leistungsaussagen, Compliance-Aussagen). Tabelle: Behauptung | Quelle (Datei:Zeile) | Prüfmethode | Ergebnis (WAHR / FALSCH / VERALTET / NICHT PRÜFBAR) | Beleg.
SCHRITT 2 — Gezielt prüfen (jeweils mit Beleg):
 a) Status-Konflikte SOLL/IST: INV-MVCC-SSI-1 und MVC-006 (SSI) — Katalog SOLL vs. Code implementiert; INV-WAL-TRUNCATION-1, INV-COMPACTION-ADAPTIVE-1, INV-EGRESS-AUDIT-1, INV-CALIBRATION-CONFORMAL-1, INV-PID-ANTIWINDUP-1, INV-DEDUP-TRANSITIVITY-1, INV-TTL-1: je Invariante IST/SOLL im Code belegen (Tests? Aufruf? Feature?).
 b) "Row-Level Locks" (Spezifikation) vs. OCC mit Backward-Validation (Code).
 c) "Milliarden Vektoren" vs. RAM-Modell ohne DiskANN (Analyse: ~3,4 KB/Knoten bei 768d, M=16).
 d) "Zero-Panic verifiziert": belege oder widerlege mit den Zählungen aus J-41-Methodik (nur Zählung der Produktions-unwraps je Crate).
 e) Crate-/LOC-Zahlen: 14 Workspace-Mitglieder + tests/arch, kein xtask; widersprüchliche LOC-Angaben (z. B. contextra-mvcc 1.955 vs. 3.431, contextra-store 26.311 vs. 29.371) — prüfe, welche aktuell stimmt (wc/tokei-Ersatz) und welche Dokumente veraltet sind (check-consistency, sync-docs-check ausführen).
 f) MCP-Werkzeuge: 11 aktive vs. 4 "Phantom"-Namen — prüfe den Dispatch in server_tools.rs und die Sandbox-Klassifikation; ist die Katalogaussage noch wahr?
 g) Feature-Defaults: Facade default = ["fast","candle"] (Katalog weist auf Abweichung von der Spezifikation hin) — Cargo.toml prüfen.
 h) Drei Audit-Ketten (AuditChain, ContextEditAuditRecord, AgentAudit) — stimmt die Aussage, dass sie unverbunden sind?
 i) Luftlinie/Souveränität: "keine Cloud-Laufzeitannahmen" — suche im Produktionscode nach fest eingebauten Hostnamen/URLs (rg "https?://" außerhalb von Tests/Doku) und nach Telemetrie/Update-Checks.
 j) DeploymentTier-Presets (ENG-024): EdgeMinimal 64 MB, PowerUserLocal 512 MB, EnterpriseShared 4 GB, EnterpriseRegulated 4 GB mit vollständigem Löschbeweis — belege die Konfigurationswerte im Code und ob sie durchgesetzt werden (Budgetprüfung) oder nur Voreinstellungen sind (Messung lokal in L-35).
SCHRITT 3 — Doku-Gates ausführen: ⚠NICHT-IM-REPO(cargo xtask check-doc-references), check-toc-integrity, check-crate-references, check-type-registry, sync-docs --check (nicht ohne --check!), check-agents-integrity, check-agents-freshness, check-jules-context-freshness.
SCHRITT 4 — Ergebnis: Rangliste der 15 gefährlichsten falschen/veralteten Aussagen (Gefahr = Irreführung von Auditoren, Anwendern, Förderstellen) mit Korrekturvorschlag als Text.

[[REPORT]]
```

### J-43 — DSGVO-Ende-zu-Ende-Löschpfad (Canary-Test über alle Schichten)

> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-checkpoint`, `contextra-cognition`, `contextra-graph`, `contextra-kvcache`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.

**Typ:** Test/Audit · **Dauer:** 90–180 min · **Schreibt:** Bericht + `crates/contextra/tests/campaign_erasure_*.rs`

```text
[[CTX]]

AUFTRAG: Beweise, ob "Löschen" (contextra_forget / Collection-Löschung mit Löschbeweis) tatsächlich alle Schichten erreicht. Hypothese aus PRODUKT-4 (SEC-02, S1): Crypto-Shredding vernichtet den Schlüssel in O(1), aber Klartext kann in Memtable, Block-Cache, HNSW-Graph, CSR-Graph, Text-Index und KV-Cache überleben, solange kein synchroner Purge erfolgt. Der RAM-Teil wird lokal in L-23 geprüft; hier der Festplatten- und Index-Teil.

AUFBAU: Test in crates/contextra/tests/campaign_erasure_*.rs (Facade, Features sovereign/compliance, Preset EnterpriseRegulated mit vollständigem Löschbeweis; encryption-at-rest und deletion-proof aktiv; Temp-Verzeichnis). Verwende einen eindeutigen, synthetischen Canary-Text (z. B. "CANARY-ERASE-7F3A-PERSON-ANNA-TESTFALL") als Dokumenteninhalt UND als Metadaten UND als Teil eines Graph-Tripels UND (falls Entity-Extraktion aktiv) als extrahierte Entität, jeweils unter eigenem Tenant.

ABLAUF je Konfiguration (A: encryption-at-rest AN; B: AUS; C: AN + Checkpoint vor dem Löschen):
 1) Dokument einfügen (Embedding via mock-Provider), Daten flushen, Compaction auslösen, Checkpoint erstellen (Konfiguration C).
 2) Vorher-Scan: durchsuche ALLE Dateien im Datenverzeichnis (rekursiv, Bytes) nach dem Canary (UTF-8, UTF-16LE, Hex, Base64). Erwartung A: nie im Klartext auf der Platte; Erwartung B: darf vorkommen.
 3) Löschen (contextra_forget bzw. Rust-API) → Löschbeweis erzeugen und extern verifizieren (verify_external).
 4) Nachher-Prüfung in jeder Schicht über die ÖFFENTLICHE API: get, search (Vektor, Text, Graph, hybrid), consolidate, explain, KV-Cache-Lookup — der Canary darf nirgends mehr erscheinen.
 5) Nachher-Scan der Platte nach flush + Compaction + neuem Checkpoint: Canary-Fund in B ist erwartbar bis Compaction ("Tombstone"); dokumentiere, WANN er verschwindet (nach Compaction? nie? Checkpoint behält Kopie?). Fund in A ist S1.
 6) Zeitliche Bindung: erzeuge den Löschbeweis VOR der tatsächlichen Löschung (falls API es zulässt) — muss scheitern (INV-DELETION-1).
 7) Wiederherstellungs-Test: Checkpoint aus Konfiguration C zurückspielen NACH dem Löschen: das gelöschte Dokument darf nicht wieder auftauchen (Löschung muss über Checkpoint-Restore hinweg Bestand haben, z. B. über das RevocationLog) — falls es auftaucht: Befund S1 (Wiedergeburt gelöschter Daten).
 8) Mandantenisolation: Löschen in Tenant A berührt Tenant B nicht (Canary in B bleibt auffindbar).
 9) Geschwister-Schichten einzeln: HNSW (Vektor weg + keine Geisterzeiger: verify_no_ghost_pointers, INV-DELETION-2), contextra-text (Posting-Listen und Dokumentstatistik/IDF aktualisiert?), contextra-graph⚠ (Knoten/Kanten/Hyperkanten und abgeleitete Community-/Konsolidierungsergebnisse), contextra-cognition⚠ (konsolidierte Zusammenfassungen oder Embeddings, die den Inhalt abgeleitet enthalten — hier kann personenbezogene Information fortleben, obwohl das Original gelöscht ist: Befund S2, falls so), contextra-kvcache⚠ (gecachte Tensoren zum Prompt, kv-encryption), contextra-checkpoint⚠ (Blake3-Manifest enthält Hashes, keine Klartexte?).
 10) Löschbeweis-Inhalt: export_for_audit() enthält keinen Klartext und nennt den Umfang (DeletionScope/DeletionLayer/ExcludedScope) wahrheitsgemäß: Welche Schichten sind als "excluded" ausgewiesen? Eine nicht ausgewiesene, aber nicht bereinigte Schicht = S1 (falscher Löschbeweis).

AUSWERTUNG: Matrix Schicht x Konfiguration (A/B/C) x Zeitpunkt (sofort / nach flush / nach Compaction / nach Checkpoint) mit Canary gefunden/nicht gefunden und Beweisstufe. Bewerte die Belastbarkeit des Löschbeweises technisch (keine Rechtsberatung).

[[REPORT]]
```

### J-44 — Befund-Regressionsmatrix (alle bekannten Befunde einzeln nachprüfen)

> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-mcp`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.

**Typ:** Audit/Test · **Dauer:** 90–180 min · **Schreibt:** Bericht (+ Mini-Tests, wo nötig)

```text
[[CTX]]

AUFTRAG: Prüfe JEDEN der folgenden bekannten Befunde einzeln gegen den aktuellen Checkout und bestimme den heutigen Status: OFFEN (reproduziert) · BEHOBEN (nicht mehr reproduzierbar, Regressionstest vorhanden/geschrieben) · TEILWEISE · NICHT REPRODUZIERBAR · CODE UMGEBAUT (Fundstelle existiert nicht mehr). Nutze, wo vorhanden, die Beweis-Tests aus J-20 bis J-27 (docs/analysis/kampagne/J-2x.md); fehlen sie, schreibe Minimaltests. Behebe nichts.

LISTE (Quelle in Klammern):
 Produkt 1 LSM: S1-01 TxEnd-Sequenznummer/Replay-Bruch (write.rs ~240) · S1-02 DirLock ohne POSIX-Lock (util.rs ~35) · S2-01 MemoryOnly-Rollback setzt last_hmac nicht zurück (write.rs ~290) · S2-02 generate_deletion_proof ohne vorherige physische Löschung (kv/segment.rs ~135) · S2-03 verwaiste Intent-Locks bei put_if_absent-Abbruch (write.rs ~75) · S3-01 MergeOperator nicht verdrahtet (compaction/engine.rs ~410) · S4-01 unklare Fehlermeldung (compaction/retention.rs ~63) · Testfehler in checkpoint_systematic_crash.rs · Doku docs/audits/lsm-core_DEEP_AUDIT_2026-09-27.md zu S1-01 · Hinweis ANALYSEBERICHT_8TAGE: Race-Fixes um Commit/Rollback/Flush (put_if_absent-TOCTOU, Group-Commit-Leader-Panics, HNSW/LSM compute-then-commit-Nichtatomarität).
 Produkt 2 MVCC: BUG-MVCC-01 Phantom bei Präfix-Scans (store read.rs ~47) · BUG-MVCC-02 validate() setzt externen Lock voraus (ssi.rs ~592) · BUG-MVCC-03 flüchtiges Commit-Register (ssi.rs ~270) · BUG-MVCC-04 Doku "Row-Level Locks" · B-16 Total-Outage bei großem Register (soll durch Coarsening behoben sein) · Fall A TOCTOU is_key_staged_globally → is_key_staged_for_tx.
 Produkt 3 Vektor: BUG-VEC-01 DocId-Truncation (hnsw/types.rs ~142) · BUG-VEC-02 Kompilierfehler filtered_selectivity_recall.rs ~90 · BUG-VEC-03 AdaptiveEf-Oszillation (adaptive_ef.rs ~114) · BUG-VEC-04 partial_rebuild.rs Schein-Code (~180) · BUG-VEC-05 entkoppelte Sequenz-IDs DiskANN-WAL vs. LSM-WAL (diskann/build.rs ~310) · K3 Backlink-Race bei Slot-Wiederverwendung · K7 RAM/Mmap-Rundungsdifferenz.
 Produkt 4 Krypto/DSGVO: SEC-01 Audit-Kette ohne externen Head-Anker (audit_chain.rs ~252) · SEC-02 Klartext im RAM nach Crypto-Shredding (kv_shredding.rs ~188; RAM-Teil lokal L-23) · SEC-03 Split-Brain KeyRegistry/RevocationLog (kv_shredding.rs ~188) · SEC-04 Rehydrator-Token-Injektion (egress_gateway.rs ~43) · SEC-05 drei getrennte Audit-Ketten · MAJOR normalize_text-Homoglyphen-Bypass (contextra-mcp⚠::prompt_injection).
 Katalog: MCP-Phantom-Werkzeuge contextra_upsert/_delete/_create_collection/_drop_collection (P0-4) · Facade-Default-Features candle statt nur fast · MVC-006/INV-MVCC-SSI-1 Statuskorrektur.

AUSGABE: Matrix Befund-ID | heutiger Status | Beweis (Test/Befehl, Beweisstufe) | Fundstelle heute | Anmerkung. Danach: Anzahl OFFEN je Schwere, Rangliste der 10 gefährlichsten OFFENEN Befunde, und eine Liste von Befunden, deren Fundstelle sich verschoben hat (Zeilenangaben der Vor-Analysen sind Näherungswerte — suche per Symbolname).

[[REPORT]]
```

---

# Teil B — Lokal mit Gemini Antigravity-CLI

## B.0 Umgebung & Hygiene

### B.0.1 Abhängigkeitsmatrix (lokal)

| Abhängigkeit | Zweck | Benötigt von | Bezug |
|---|---|---|---|
| Rust **1.89.0** (gepinnt) + Nightly + `rust-src`, `llvm-tools-preview`, optional `miri` | Build, TSan/ASan/Fuzz/Miri | alle | `nix develop` oder `rustup` |
| `just`, `cargo-deny`, `cargo-audit`, `cargo-llvm-cov`, `cargo-mutants`, `cargo-fuzz`, `ast-grep` | Standard-Workflows | L-01, L-14, L-16, L-17 | `nix develop` / `cargo +stable install --locked` (außerhalb des Repos) |
| `flatc` | FlatBuffers-Drift-Gates | L-01 | Nix / Paketmanager |
| `strace` (≥ 5.x für `inject=`) | Syscall-Audit, Fehlerinjektion (EIO, ENOSPC, EMFILE) | L-11, L-12, L-31 | Paketmanager |
| `gdb` (`gcore`), Python 3 | Speicher-Forensik | L-23 | Paketmanager |
| `taskset`, `numactl` (optional), `perf` (optional), `hyperfine` (optional), `fio` (optional) | Benchmark-Hygiene, I/O-Referenz | L-00, L-30–L-35 | Paketmanager |
| `unshare` (util-linux) | Netzwerk-Namespace → Air-Gap-Beweis | L-19 | Paketmanager; benötigt User-Namespaces |
| `systemd-run` (optional) | Speicher-/CPU-Limits per cgroup | L-35 | systemd |
| Ollama + ein lokales Embedding-Modell | `contextra-infer-ollama⚠` real | L-19 | `ollama list` muss ein Embedding-Modell zeigen |
| ONNX Runtime (nativ) + Testmodell | `contextra-infer-onnx⚠` | L-19 | je nach `ort`-Konfiguration |
| CUDA-Toolkit (optional) | Feature `cuda` von `contextra-infer-candle⚠` | L-19 | nur wenn GPU vorhanden |
| Python 3 + `maturin`, `numpy`, `pytest`, `matplotlib` | `contextra-py⚠`, Diagramme | L-20, L-18 | `nix develop` (PyO3-Toolchain) oder venv |
| WASM-Targets | `contextra-sandbox⚠` | L-21 | `rustup target add …` |

### B.0.2 Umgebungs-Check (zsh; installiert nichts, meldet nur)

```zsh
#!/usr/bin/env zsh
# Nutzung: zsh local_env_check.zsh   (im Repo-Root ausführen)
setopt no_unset 2>/dev/null
ok()   { print -P "%F{green}OK%f    $1" }
miss() { print -P "%F{red}FEHLT%f $1  →  $2" }
chk()  { command -v "$1" >/dev/null 2>&1 && ok "$1 ($(command -v $1))" || miss "$1" "$2" }

print "== Toolchain =="; rustc --version; cargo --version
rustup toolchain list | grep -q nightly && ok "nightly" || miss "nightly" "rustup toolchain install nightly -c rust-src -c llvm-tools-preview"
rustup component list --toolchain nightly --installed 2>/dev/null | grep -q miri && ok "miri" || miss "miri (optional)" "rustup +nightly component add miri"

print "== Werkzeuge =="
for t in just flatc strace gdb gcore python3 taskset unshare ast-grep sg jq rg; do chk $t "Paketmanager / nix develop"; done
for t in cargo-deny cargo-audit cargo-llvm-cov cargo-mutants cargo-fuzz; do chk $t "cargo +stable install $t --locked (im Verzeichnis /tmp ausführen)"; done
for t in perf hyperfine fio numactl systemd-run; do chk $t "optional"; done

print "== Nix =="; command -v nix >/dev/null && (nix develop -c true >/dev/null 2>&1 && ok "nix develop" || miss "nix develop" "Flake-Fehler prüfen") || miss nix "optional, empfohlen"

print "== Ollama =="
command -v ollama >/dev/null && ollama list 2>&1 | head -5 || miss ollama "optional, nur für L-19"

print "== Hardware / Hygiene =="
lscpu | grep -E "Model name|^CPU\(s\)|Thread|MHz" 
for f in /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor /sys/devices/system/cpu/intel_pstate/no_turbo /sys/devices/system/cpu/smt/control; do [[ -r $f ]] && print "$f: $(cat $f)"; done
free -h | head -2; df -hT . /tmp | cat
lsblk -d -o NAME,ROTA,MODEL,SIZE 2>/dev/null
print "ptrace_scope: $(cat /proc/sys/kernel/yama/ptrace_scope 2>/dev/null || echo n/a)"
print "user namespaces: $(cat /proc/sys/user/max_user_namespaces 2>/dev/null || echo n/a)"
```

---

### L-00 — Umgebungs- und Benchmark-Eignungs-Baseline

> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-campaign`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.

**Typ:** Audit (Umgebung) · **Dauer:** 20–40 min · **Schreibt:** nur Bericht · **Zuerst ausführen**

```text
[[CTX]]

AUFTRAG: Stelle fest, was diese Maschine kann und taugt. KEINE Systemänderung, KEIN sudo. Alles, was Root bräuchte, nur als Vorschlag mit exaktem Befehl auflisten.

SCHRITT 1 — Toolchain/Werkzeuge: führe den Umgebungs-Check aus (Skript aus Kapitel B.0.2 des Katalogs, falls die Datei vorhanden ist; sonst gleichwertige Einzelbefehle): rustc/cargo-Version, rust-toolchain.toml, Nightly + Komponenten, nix develop -c true, just --version, cargo-Subcommands, flatc, strace, gdb, python3, unshare, ollama list. ⚠NICHT-IM-REPO(cargo xtask env-validate) und just env-validate.
SCHRITT 2 — Repo: git status, git rev-parse HEAD (Abgleich mit Referenz `c41afc15`), cargo fetch --locked, cargo metadata --locked, capabilities.toml lesen (Crate-Anzahl, Ringe, Testbefehle).
SCHRITT 3 — Hardware & Benchmark-Hygiene (nur lesen): CPU-Modell, Kerne/Threads, SMT-Status, Turbo/Boost-Status, Frequenz-Governor, Cache-Größen (lscpu), RAM, Swap, NUMA, Hintergrundlast (uptime, top -bn1 | head -20), thermische Drosselung (falls /sys/class/thermal lesbar), Energiesparmodus/Akkubetrieb (upower oder /sys/class/power_supply), Dateisystemtyp und Mount-Optionen von /tmp und vom Repo-Verzeichnis (df -T; findmnt -T .), ROTA/NVMe-Modell (lsblk), I/O-Scheduler. Bestimme ob /tmp ein tmpfs ist (dann taugt es NICHT für Durability-/fsync-Aussagen).
SCHRITT 4 — fsync-Realität: Miss die fsync-Latenz des Datenträgers, auf dem die Kampagnen-Verzeichnisse liegen sollen, mit einem Wegwerfprogramm (z. B. 1000x write 4 KiB + fsync mit Zeitmessung in Rust oder Python unter /tmp/contextra-campaign⚠/L-00; ist /tmp ein tmpfs, nutze ein Verzeichnis auf der echten Platte unter dem Repo-Verzeichnis, das in .gitignore steht, z. B. target/). Berichte Median, p99, Maximum. Eine fsync-Latenz unter ~20 µs deutet auf tmpfs, Schreib-Cache ohne Flush-Durchreichung oder Virtualisierungs-Cache hin: Durability-Messungen sind dann unzuverlässig — kennzeichne das ausdrücklich.
SCHRITT 5 — Verdikt: (a) "Crash-/Durability-Kampagnen sinnvoll?" (b) "Absolute Benchmarks sinnvoll?" mit Begründung und der Liste behebbarer Störfaktoren (Governor, Turbo, Hintergrundprozesse, Energiesparen). (c) Welche L-Prompts sind ausführbar, welche brauchen zuerst ein Werkzeug (Tabelle mit Installationsbefehl).

[[REPORT]]
```

### L-01 — Voll-Gate und Gesamt-Qualitätslauf (lokale Gegenprobe zu J-01)

> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-campaign`, `contextra-infer-ollama`, `contextra-infer-onnx`, `contextra-py`, `contextra-sandbox`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.

**Typ:** Test/Audit · **Dauer:** 1–4 h · **Schreibt:** nur Bericht

```text
[[CTX]]

AUFTRAG: Führe die vollständige Qualitätskette lokal aus und vergleiche mit der Jules-Gate-Matrix (docs/analysis/kampagne/J-01.md, falls vorhanden). Keine Reparaturen, keine Gate-Änderungen. Logs je Schritt nach /tmp/contextra-campaign⚠/L-01/.

SCHRITTE (nacheinander, jeweils mit Exit-Code und Dauer protokollieren; nix develop -c <cmd> || <cmd>):
 1) just bootstrap (nur falls Hooks fehlen) · just check · just dag-check · just check-arch-gates
 2) cargo check --workspace --all-targets · cargo check --examples · cargo build --workspace --release
 3) cargo build -p contextra-sandbox⚠ -p contextra-py⚠ -p contextra-infer-ollama⚠ -p contextra-infer-onnx⚠ (Non-Default-Member; Fehler als ENV/CODE klassifizieren)
 4) just test · just triple-test (3x cargo test --workspace) · just prove-bugs · just prop-tests · just qa-full
 5) Hermetic-Feature-Gate: für jeden Crate cargo check -p <crate> --no-default-features
 6) just security-audit · just debt-audit · ⚠NICHT-IM-REPO(cargo xtask security-scan)
 7) ⚠NICHT-IM-REPO(cargo xtask workspace-verify) --mode=audit (bzw. just full-audit) · ⚠NICHT-IM-REPO(cargo xtask jules-preflight) · ⚠NICHT-IM-REPO(cargo xtask jules-submit-gate)
 8) Lokale Zusatzgates, die in Jules nicht gehen: ⚠NICHT-IM-REPO(cargo xtask check-commit-messages), check-commit-diff-integrity, commit-health, check-branch-overlap (vollständige Git-Historie vorhanden).
AUSWERTUNG: (a) Matrix Schritt | rc | Dauer | Klasse (PASS/FAIL-CODE/FAIL-ENV/FAIL-GATE); (b) Unterschiede zur Jules-Matrix (was ist lokal grün, in Jules rot oder umgekehrt, und warum); (c) triple-test: identische Ergebnisse in allen 3 Läufen? Jede Abweichung ist Flaky-Befund S2; (d) Dauer-Rangliste der langsamsten Schritte.

[[REPORT]]
```

---

## B.1 Crash-, Fehlerinjektions- und Nebenläufigkeits-Kampagnen

> **Ehrliche Einordnung:** `SIGKILL` eines Prozesses beweist **Prozess-Crash-Konsistenz** (der OS-Seitencache überlebt). Er beweist **nicht** Stromausfall-Sicherheit (`fsync`-Korrektheit). Dafür gibt es in diesem Katalog drei Bausteine: den Syscall-Reihenfolge-Audit (L-11), die Fehlerinjektion über das VFS/`strace` (L-11) und — nur bei vorhandener Root-Berechtigung und nach Rückfrage — optional `dm-flakey`/`dm-log-writes` oder eine QEMU-VM, die hart beendet wird (siehe L-11, Schritt 6).

### L-10 — Crash-Konsistenz-Kampagne (SIGKILL-Zyklen mit ACK-Protokoll)

> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-campaign`, `contextra-checkpoint`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.

**Typ:** Test · **Dauer:** 2–8 h · **Schreibt:** Bericht (+ Wegwerf-Harness in `/tmp`)

```text
PARAMETER: ZYKLEN_PRO_MODUS = 1000     (Minimum 200)     DISK_DIR = <Verzeichnis auf echter Platte, kein tmpfs>

[[CTX]]

AUFTRAG: Beweise die Crash-Konsistenz von contextra-store (und contextra-checkpoint⚠) unter wiederholtem SIGKILL. Quelle der Hypothesen: PRODUKT-1-LSM-STORAGE-ANALYSE.md (Reifegrad R1: Datenverlust-/Replay-Bugs S1-01, S2-01). Harness als eigenes Cargo-Projekt unter /tmp/contextra-campaign⚠/L-10/harness mit path-Dependency auf contextra-store (Release-Profil). Nicht ins Repo committen.

HARNESS-DESIGN (Kind- und Elternprozess im selben Binary, Unterkommandos child/verify/parent):
 - Kind: öffnet LsmStorage im Arbeitsverzeichnis (mktemp -d unterhalb DISK_DIR), führt eine deterministische Folge von Transaktionen aus dem Seed aus (put, delete, put_batch, delete_prefix, put_if_absent, gelegentlich rollback). VOR commit() schreibt es "INTENT <tx> <digest>" auf stdout (flush), NACH Rückkehr von commit() mit Ok "ACK <tx>" (flush). Wertgrößen variieren (klein bis 64 KiB), damit Flush/Compaction/Manifest-Rollover stattfinden; Konfiguration so wählen, dass Memtable-Flush und Compaction innerhalb der Laufzeit mehrfach auftreten (suche die Konfigurationsparameter im Code).
 - Eltern: liest stdout, führt Protokoll (ACK-Menge, INTENT-ohne-ACK-Menge), sendet SIGKILL nach zufälliger Verzögerung (gleichverteilt 5–400 ms; zusätzlich 20 % der Zyklen "gezielt": Kill, sobald das Kind einen Phasenmarker meldet — FLUSH_START, COMPACT_START, MANIFEST_ROLLOVER, falls diese über Metriken/Hooks erreichbar sind; sonst nur Zufallskills und das als Einschränkung berichten).
 - Verify (separater Prozess): öffnet das Verzeichnis neu und prüft folgende Invarianten gegen ein aus dem Seed abgeleitetes Referenzmodell:
   I1 Jede ACKed Tx ist vollständig sichtbar (alle ihre Operationen).
   I2 Jede Tx mit INTENT ohne ACK ist ganz oder gar nicht sichtbar (Atomarität), nie teilweise.
   I3 Kein Key/Value existiert, der von keiner Tx geschrieben wurde (keine Garbage-Daten).
   I4 open() gelingt; ein ZWEITES open() nach close gelingt ebenfalls und liefert identischen Zustand (Recovery ist idempotent).
   I5 Nach Recovery gelingen neue Commits; sie überleben den nächsten Zyklus.
   I6 WAL-Kette lückenlos (verify_wal_chain_completeness bzw. Replay ohne Fehler); Sequenznummern streng monoton.
   I7 Zurückgerollte Tx sind nicht sichtbar.
   I8 Keine unerklärten Hilfsdateien (tmp, *.bak) nach erfolgreicher Recovery; Plattenverbrauch begrenzt.
 - Recovery-Kill: In 20 % der Zyklen wird der VERIFY-Prozess selbst (die Recovery) per SIGKILL zu zufälligem Zeitpunkt beendet und danach erneut gestartet (Idempotenz unter wiederholtem Crash).
MODI: Führe die Kampagne für JEDEN Durability-Modus aus (Varianten per grep im Code ermitteln; für MemoryOnly gilt nur Konsistenz/Präfix-Eigenschaft, nicht I1 — lies die Semantik im Code und halte sie fest) und für Konfiguration mit/ohne encryption-at-rest und wal-integrity.
AUSFÜHRUNG: Mindestens ZYKLEN_PRO_MODUS Zyklen je Modus (Seeds und Kill-Zeitpunkte in CSV). Bei jedem Fehlschlag: Seed, Kill-Zeit, vollständige Verify-Ausgabe, und Verzeichnis-Snapshot (tar nach /tmp/contextra-campaign⚠/L-10/failures/) aufheben; danach Minimierung (kleinere Operationsfolge, die denselben Fehler zeigt) und ein deterministischer Repro-Test unter crates/contextra-store/tests/campaign_crash_repro_*.rs mit #[ignore = "<Bug-ID>..."].
ERWARTETE HYPOTHESEN (zu bestätigen/widerlegen, nicht anzunehmen): S1-01 (Reopen nach Commit scheitert mit WalCorruption "Duplicate or non-monotonic sequence number") müsste schon im allerersten Zyklus auftreten, falls noch offen; S2-01 (ungültige HMAC-Kette nach Rollback eines fehlgeschlagenen MemoryOnly-Commits).
Zusätzlich: Wiederhole die Kampagne für contextra-checkpoint⚠ (Checkpoint erstellen, während das Kind weiterschreibt, SIGKILL mitten im Checkpoint; danach muss der letzte VOLLSTÄNDIGE Checkpoint (Blake3-Manifest) verifizierbar sein und ein halber Checkpoint erkannt und verworfen werden).
BERICHT enthält: Zyklen je Modus, Anzahl Verstöße je Invariante I1–I8, Verteilung der Kill-Zeitpunkte, Dauer, erkannte Schwächen der Methode (z. B. keine echte Stromausfall-Simulation).

[[REPORT]]
```

### L-11 — Syscall-Reihenfolge-Audit und Fehlerinjektion (`strace`, VFS)

> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-campaign`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.

**Typ:** Test/Audit · **Dauer:** 2–5 h · **Schreibt:** Bericht (+ Wegwerf-Skripte in `/tmp`)

```text
[[CTX]]

AUFTRAG: Belege anhand der tatsächlichen Systemaufrufe, ob contextra-store seinen Durability-Vertrag einhält, und prüfe das Verhalten unter I/O-Fehlern. Quelle: PRODUKT-1-LSM-STORAGE-ANALYSE.md Prüfkatalog A, B, E, F. Nutze das vorhandene Wegwerf-Harness aus L-10, falls vorhanden (/tmp/contextra-campaign⚠/L-10/harness), sonst baue ein minimales Programm: öffnen, 200 Commits, flush, Compaction, schließen.

SCHRITT 1 — Trace aufnehmen: strace -f -ttt -yy -e trace=openat,creat,write,pwrite64,writev,fsync,fdatasync,sync_file_range,rename,renameat,renameat2,unlink,unlinkat,ftruncate,fallocate,mkdir,fcntl,flock,mmap,munmap,msync,close -o /tmp/contextra-campaign⚠/L-11/trace.txt <harness>. Das Harness schreibt Marker "ACK <tx>" per write(1, …) — diese erscheinen im Trace und erlauben die Zuordnung "Commit bestätigt".
SCHRITT 2 — Regeln automatisch prüfen (Wegwerf-Skript, z. B. Python, das den Trace parst; Regelverletzungen mit Zeilennummern im Trace ausgeben):
 R-A Vor jedem ACK eines Commits im Durability-Modus "synchron" gibt es ein fsync/fdatasync auf dem WAL-Dateideskriptor NACH dem zugehörigen write. (Group-Commit: ein fsync darf mehrere ACKs decken, aber kein ACK darf VOR dem deckenden fsync erscheinen.)
 R-B Neue Dateien (SSTable, Manifest, Salt-Datei, UUID-Sidecar): write → fsync(Datei) → rename → fsync(Elternverzeichnis). Fehlt der Verzeichnis-fsync nach rename/creat/unlink, ist die Dauerhaftigkeit nach Stromausfall nicht garantiert (S2).
 R-C Alte Eingabe-SSTables/Manifest/WAL werden erst nach dauerhaftem Ersatz gelöscht (unlink nach dem fsync des neuen Manifests und des Verzeichnisses).
 R-D ftruncate auf dem WAL (Crash-Recovery-Trunkierung, truncate_uncommitted_tail) wird von fsync gefolgt, bevor neue Schreibvorgänge bestätigt werden.
 R-E Schreibvorgänge mit Teilergebnis (write liefert weniger Bytes als verlangt) werden in einer Schleife vervollständigt (im Normalfall nicht sichtbar; siehe Schritt 3).
SCHRITT 3 — Fehlerinjektion mit strace (nur gegen das eigene Harness-Kind; Optionen deiner strace-Version per strace --help prüfen):
 a) fsync-Fehler: -e inject=fsync:error=EIO:when=<n> und -e inject=fdatasync:error=EIO:when=<n> für verschiedene n (erster, mittlerer, späterer fsync, auch "ab dem n-ten dauerhaft": when=<n>+). Erwartung: Commit liefert Err (nie Ok), WAL gilt als poisoned (is_poisoned), weitere Schreibversuche scheitern bis recover_from_poison; nach Neustart ist KEIN Commit sichtbar, der Err geliefert hat, und jeder Ok-Commit ist sichtbar; kein Panic.
 b) ENOSPC: -e inject=write:error=ENOSPC:when=<n>+ (und für pwrite64/fallocate): Verhalten bei Flush, Compaction, Manifest-Rollover, WAL-Append: Err statt Panic, keine halb geschriebene aber als gültig geführte Datei, Neustart möglich.
 c) EMFILE/ENFILE: -e inject=openat:error=EMFILE:when=<n>+ (alternativ ulimit -n 64 im Subshell des Kindes).
 d) Kurz-Schreibzugriff (falls deine strace-Version retval-Injektion kann): -e inject=write:retval=<kleiner als angefordert>:when=<n>: Wird die Schleife korrekt fortgesetzt oder entsteht ein abgeschnittener Frame?
 e) EINTR/EAGAIN: -e inject=write:error=EINTR:when=<n>: wird wiederholt?
SCHRITT 4 — Fehlerinjektion über das projekteigene VFS (Feature fault-injection, FaultVfs, simulate_wal_append_failure_for_test, restore_wal_file_handle_for_test): welche Fehlerpunkte bietet es? Vergleiche die Abdeckung mit den strace-Punkten aus Schritt 3; fehlende Punkte = Befund (Testbarkeit).
SCHRITT 5 — Direkt-I/O am VFS vorbei: Wurden im Trace Dateizugriffe beobachtet, die NICHT über die FaultVfs-Abstraktion gehen könnten (Vergleich mit J-41, Punkt e)? Liste.
SCHRITT 6 — Stromausfall-Simulation (nur mit Rückfrage an mich, weil Root nötig): Falls ich zustimme, beschreibe und führe aus: dm-log-writes oder dm-flakey auf einem Loop-Device (Wegwerf-Image, mehrere hundert MB) mit Wiedergabe der Schreibfolge bis zu jeder fsync-Grenze und anschließender Verify-Prüfung wie in L-10; ansonsten nenne den exakten Befehlsablauf und markiere den Punkt als NICHT AUSGEFÜHRT. Alternative ohne Root: Prüfe, ob FaultVfs "ungesyncte Schreibvorgänge verwerfen" kann; wenn ja, nutze es für dieselbe Prüfung.
AUSWERTUNG: Regelmatrix R-A…R-E x Datei-Typ (WAL, SSTable, Manifest, Salt/UUID/Marker, Checkpoint) mit BESTÄTIGT/VERLETZT und Trace-Zeilen als Beweis [GEMESSEN]; Fehlerinjektions-Matrix Fehlerart x Operation x beobachtetes Verhalten; Befunde mit Schwere.

[[REPORT]]
```

### L-12 — Mehrprozess-Isolation und `DirLock`

> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-campaign`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.

**Typ:** Test · **Dauer:** 60–120 min · **Schreibt:** Bericht (+ Wegwerf-Harness)

```text
[[CTX]]

AUFTRAG: Prüfe, ob zwei Prozesse dasselbe Datenverzeichnis gleichzeitig öffnen können. Hypothese aus PRODUKT-1 (S1-02, util.rs ~Z. 15–60): DirLock nutzt nur OpenOptions::create_new(true) ohne flock/fcntl; ein zweiter Prozess überschreibt die LOCK-Datei und korrumpiert laufende Flushes/WALs.

SCHRITTE (Wegwerf-Harness unter /tmp/contextra-campaign⚠/L-12, Datenverzeichnis per mktemp -d auf echter Platte):
 1) Statische Sicht: Lies util.rs (DirLock) und alle Stellen, an denen es benutzt wird (LsmStorage::open, Manifest::open_with_dir_lock). Welche Systemaufrufe entstehen? Prüfe per strace -f -e trace=openat,flock,fcntl,unlink,rename, ob flock/fcntl(F_SETLK/F_OFD_SETLK) vorkommen.
 2) Zwei-Prozess-Test: Prozess A öffnet und schreibt kontinuierlich (mit ACK-Protokoll wie in L-10). Prozess B versucht während A läuft zu öffnen. Erwartet: B scheitert SOFORT mit klarer Fehlermeldung, A bleibt unbeschädigt. Beobachte: Gelingt B? Was passiert mit der LOCK-Datei (Inhalt: PID? überschrieben?)? Danach A beenden und per Verify-Prozess (L-10 I1–I8) Integrität prüfen: Korruption = S1 bestätigt.
 3) Wettlauf: 8 Prozesse öffnen gleichzeitig dasselbe leere Verzeichnis (Barrier über Startsignal): genau EIN Prozess darf erfolgreich sein.
 4) Stale Lock: Prozess A wird per SIGKILL beendet (LOCK-Datei bleibt liegen). Kann ein neuer Prozess das Verzeichnis anschließend öffnen? (Ein reines create_new-Verfahren würde für immer blockieren oder muss einen Stale-Erkennungsmechanismus haben.) Teste: Verzeichnis bleibt nach Crash dauerhaft unzugänglich = Befund S2 (Verfügbarkeit); stilles Überschreiben bei lebendem Halter = S1.
 5) PID-Wiederverwendung und Container: Falls PID-basierte Stale-Erkennung genutzt wird: simuliere eine LOCK-Datei mit PID eines fremden lebenden Prozesses; wird der Lock fälschlich als gültig oder als stale gewertet? Dokumentiere das Verhalten (kein Töten fremder Prozesse!).
 6) Lesezugriff: open_read_only (Wal::open_read_only) parallel zu einem Schreiber: Verhalten dokumentieren (konsistenter Snapshot? Fehler?).
 7) Checkpoint-Verzeichnis und Manifest: Werden sie von DirLock mit abgedeckt, oder kann ein zweiter Prozess das Manifest separat öffnen (Manifest::open_with_dir_lock vs. open)?
AUSWERTUNG: Verhaltenstabelle Szenario x Ergebnis x Beweis; Schwere nach Auswirkung (stille Korruption S1; Aussperrung S2). Fix-Skizze nur als Text (z. B. echte Advisory-Locks über flock/fcntl bzw. fs2/fd-lock, Stale-Erkennung über Lock-Besitz statt Dateiexistenz).

[[REPORT]]
```

### L-13 — Sanitizer-, Miri- und Speichersicherheits-Kampagne

> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-core`, `contextra-graph`, `contextra-kvcache`, `contextra-wire`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.

**Typ:** Test · **Dauer:** 2–6 h · **Schreibt:** Bericht

```text
[[CTX]]

AUFTRAG: Prüfe Speicher- und Datenrennen-Sicherheit mit ThreadSanitizer, AddressSanitizer und (ergänzend) Miri. Nightly-Befehle ändern rust-toolchain.toml NICHT (geschützter Pfad; nutze +nightly bzw. die just-Rezepte). Sanitizer brauchen unoptimierte Debug-Builds und -Zbuild-std (rust-src erforderlich).

SCHRITT 1 — just tsan und just asan (gesamt). Schlagen sie wegen Umgebung fehl, nenne den exakten Fehler und behebe nur die Umgebung (rustup component add rust-src --toolchain nightly), nicht den Code. Ergebnisse je Crate.
SCHRITT 2 — Gezielt für die kritischsten Crates einzeln mit höherem Parallelitätsgrad: contextra-store (Group-Commit, Memtable-Sharding, Observer), contextra-vector (HNSW Insert/Search parallel, ComputePool), contextra-mvcc, contextra-engine, contextra-kvcache⚠, contextra-text, contextra-graph⚠. Befehlsmuster: RUSTFLAGS="-Zsanitizer=thread" cargo +nightly test -Zbuild-std --target <host-triple> -p <crate> -- --test-threads=<Anzahl Kerne> (Host-Triple per rustc -vV ermitteln). Wiederhole TSan-Läufe 3x (Datenrennen sind probabilistisch).
SCHRITT 3 — Miri (ergänzend, nicht im Projekt-justfile): cargo +nightly miri test -p <crate> für die reinen Safe-Rust-Crates mit überschaubarer Laufzeit (contextra-types, contextra-mvcc, contextra-ports, contextra-rank, Teile von contextra-core⚠/text/graph). Miri unterstützt Dateisystem, mmap und SIMD-Intrinsics nur eingeschränkt: Tests, die daran scheitern, als "Miri nicht unterstützt" listen — NICHT als Fehler und NICHT durch Code- oder Test-Änderungen umgehen. Setze MIRIFLAGS="-Zmiri-seed=<n>" und wiederhole mit 3 Seeds; für Nebenläufigkeit MIRIFLAGS="-Zmiri-preemption-rate=0.1".
SCHRITT 4 — Unsafe-Inseln besonders: contextra-simd, contextra-sys (Memory-Mapping), contextra-wire⚠ (FlatBuffers), contextra-vector/src/distance.rs. Je unsafe-Block: Welche Invariante muss gelten (Ausrichtung, Länge, Lebensdauer, Aliasing)? Teste sie mit ASan + gezielten Grenzfall-Eingaben (Länge 0, 1, nicht Vielfaches der SIMD-Breite, Slice am Seitenende, mmap-Datei abgeschnitten → SIGBUS-Risiko: dokumentiere, ob Truncation einer gemappten Datei durch einen fremden Prozess das Programm abstürzen lässt (contextra-sys MemoryMap, DiskANN load_node/MmapIndex)).
SCHRITT 5 — Valgrind (falls vorhanden) auf einem kleinen Release-Test von contextra-crypto (Zeroize-Verhalten, uninitialisierte Reads).
AUSWERTUNG: Matrix Crate x Werkzeug (TSan/ASan/Miri) x Ergebnis (sauber/Befund/nicht unterstützt/nicht ausführbar); je Befund Stacktrace, Klassifikation (echtes Datenrennen, UB, Fehlalarm — mit Begründung), Reproduktionsbefehl. Keine Suppression-Dateien anlegen, um Grün zu erzwingen.

[[REPORT]]
```

### L-14 — Fuzzing-Langlauf mit Korpus-Pflege und Coverage

**Typ:** Test · **Dauer:** 3–12 h · **Schreibt:** Bericht + Regressionstests

```text
PARAMETER: SEKUNDEN_PRO_TARGET = 900     PARALLEL = <Anzahl Kerne / 2>

[[CTX]]

AUFTRAG: Lasse alle Fuzz-Targets lange laufen, pflege den Korpus, miss die Fuzz-Abdeckung und triagiere Crashes. Ergänzt J-15 (Jules: 60 s je Target).

SCHRITT 1 — Targets: just harness-list; Verzeichnisse crates/*/fuzz/fuzz_targets/. Erwartet laut Dev-Guide: 11 Targets (u. a. fuzz_wal_replay in contextra-store). Nightly + cargo-fuzz prüfen.
SCHRITT 2 — Lauf: für jedes Target cargo +nightly fuzz run <target> -- -max_total_time=<SEKUNDEN_PRO_TARGET> -jobs=<PARALLEL> -workers=<PARALLEL> -rss_limit_mb=4096 -timeout=30 -print_final_stats=1 (bzw. just fuzz-all <SEKUNDEN_PRO_TARGET>). Logs je Target. Beachte: libFuzzer meldet Crashes als Artefakt (crash-*, oom-*, timeout-*, slow-unit-*).
SCHRITT 3 — Triage: Je Artefakt cargo fuzz tmin bzw. just fuzz-minimize <crate> <target>; Stacktrace; Klassifikation (Panic = Zero-Panic-Verstoß S2; OOM/Timeout = unvertraute Längenfelder bzw. quadratische Laufzeit S2/S3; Sanitizer-Fund S1; Harness-Fehler). Deterministischen Regressionstest erzeugen (crates/<crate>/tests/campaign_fuzzregress_*.rs, Bytes eingebettet, #[ignore = "<Bug-ID>..."] solange offen).
SCHRITT 4 — Abdeckung: cargo +nightly fuzz coverage <target> und Auswertung mit llvm-cov (per Anleitung von cargo-fuzz): Welche Zweige der Parser (WAL-Frame, SSTable-Reader, Manifest, Wire, HnswHeader, DiskAnnHeader, KdfHeader, DeletionProof, MCP-JSON-RPC) bleiben unberührt? Je Target: Prozent der Zielfunktion abgedeckt, Liste großer unerreichter Blöcke und Vorschlag eines Wörterbuchs (-dict=) bzw. strukturierter Eingaben (arbitrary-Ableitung) als Text.
SCHRITT 5 — Differential-Fuzzing (neue Targets nur im Wegwerf-Projekt oder als Empfehlung, falls xtask registry-check sonst bricht): (a) WAL replay (Stream) vs. replay_mmap: gleiche Eingabe muss gleiches Ergebnis oder gleichen Fehlertyp liefern; (b) SSTable-Reader get vs. scan vs. stream: gleiche Daten; (c) HNSW save→load_mmap Roundtrip mit mutierten Bytes: nie Panic/UB; (d) Egress-normalize_text: Idempotenz (normalize(normalize(x)) == normalize(x)) und Erkennung unter Homoglyphen.
SCHRITT 6 — Korpus: Neue interessante Eingaben (cargo fuzz cmin zur Minimierung) — NICHT ins Repo kopieren, außer das Repo checkt Korpora ein (prüfen) und ich bestätige.
AUSWERTUNG: Tabelle Target | Laufzeit | Ausführungen | Abdeckung | Crashes | Klassen; offene Fuzz-Lücken priorisiert.

[[REPORT]]
```

### L-15 — Loom tief, Chaos-Matrix lang, Thread-Skalierung

**Typ:** Test · **Dauer:** 3–10 h · **Schreibt:** Bericht + Tests

```text
[[CTX]]

AUFTRAG: Vertiefe die Nebenläufigkeits-Prüfung aus J-14 auf lokaler Hardware: größere Loom-Modelle, lange Chaos-Läufe, Skalierung über Threadzahlen. Jeder Loom-Fehlschlag ist ein echter Defekt (Loom prüft Interleavings, keine Zeiten).

SCHRITT 1 — Loom-Inventar: rg -n "loom" crates -g '*.rs' -g Cargo.toml: welche Modelle existieren (Group-Commit-Handoff und?). Liste Komponenten MIT komplexer Synchronisation OHNE Loom-Modell: Memtable-Sharding, ObserverRegistry/Circuit-Breaker, SnapshotRegistry (register/release/min_active_seqno), TxBuffer (stage/drain/reap + intent_locks), HNSW apply_insert vs. rebuild_phase2 (Swap), BlockCache (SIEVE-Eviction), ComputePool, TenantIsolatedKvStore-Eviction, BulkExfiltrationDetector.
SCHRITT 2 — Bestehende Modelle tief: RUSTFLAGS="--cfg loom" LOOM_MAX_PREEMPTIONS=3 (dann 4, falls die Laufzeit es zulässt) cargo test -p contextra-store --test loom_group_commit_handoff --release -- --nocapture; LOOM_LOG=trace nur bei Fehlschlag; LOOM_CHECKPOINT_FILE für lange Läufe. ⚠NICHT-IM-REPO(cargo xtask loom-run), just loom-store. Laufzeit und Zustandsraum-Größe berichten.
SCHRITT 3 — Neue Loom-Modelle (nur in neuen Dateien crates/<crate>/tests/campaign_loom_*.rs, hinter cfg(loom)); wähle nach Risiko mindestens 2: (a) SnapshotRegistry: register/release/min_active_seqno — Invariante: min_active_seqno ist nie größer als der älteste lebende Guard; (b) TxBuffer + intent_locks: zwei Tx put_if_absent auf denselben Key plus Abbruch einer Tx — Invariante: höchstens ein Gewinner, nach Abbruch keine verwaisten Locks; (c) HNSW Swap: Reader während rebuild_phase2_merge_and_swap — Invariante: Reader sieht entweder alten oder neuen Graph vollständig. Loom-Modelle spiegeln die Synchronisationsstruktur mit loom-Primitiven; prüfe, ob die Produktion loom-kompatible Typen über eine cfg-Abstraktion nutzt (sonst nur ein vereinfachtes Modell möglich — das ist ein Hinweis auf ein Testbarkeitsdefizit und im Bericht zu benennen).
SCHRITT 4 — Chaos-Matrix lang: just chaos-test, dann cargo test -p contextra-store --test chaos_matrix --release -- --ignored --test-threads=1 wiederholt (mind. 20 Läufe, falls ein Seed-/Iterationsparameter existiert — suche Umgebungsvariablen/Konstanten im Testcode und erhöhe sie per Umgebung, nicht per Codeänderung). Bewertung der Matrix-Abdeckung (Fehlerarten x Operationen).
SCHRITT 5 — Thread-Skalierung: Group-Commit-Durchsatz und Latenzverteilung bei 1, 2, 4, 8, 16, 32, 64, 128, 256 parallelen Schreibern (Wegwerf-Programm unter /tmp, Release, echte Platte). Erwartet: Durchsatz steigt durch Batching (Group Commit) und flacht ab; ein Einbruch bei hoher Parallelität (Lock-Konvoi) = Befund S3. Erfasse fsync-Zahl pro Sekunde (strace -c -f -e trace=fsync,fdatasync) — Group-Commit soll die fsync-Rate je Commit senken.
SCHRITT 6 — Deadlock-Jagd: Stress mit Watchdog (alle 10 s Stack-Dump aller Threads via gdb -p <pid> -batch -ex "thread apply all bt" bei Stillstand >30 s) für die gemischte Last aus J-14 Schritt 4(b) über 30 Minuten.

[[REPORT]]
```

### L-16 — Mutation-Testing vollständig (Tier-0 bis Tier-2)

> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-campaign`, `contextra-privacy`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.

**Typ:** Test · **Dauer:** 4–24 h · **Schreibt:** Bericht + Killer-Tests

```text
PARAMETER: ZIELE = contextra-crypto, contextra-mvcc, contextra-privacy⚠, contextra-store (--file 'src/wal/**' und 'src/compaction/**'), contextra-vector (--file 'src/hnsw/deletion.rs' und 'src/quantize/**')

[[CTX]]

AUFTRAG: Vollständiger Mutation-Lauf; ergänzt die Shards aus J-17. Ziel ist, nachzuweisen, dass die Tests der kritischsten Module Fehler wirklich bemerken.

SCHRITT 1 — cargo mutants --list --package <crate> [--file ...] | wc -l je Ziel (Mutantenzahl und geschätzte Dauer). just mutants-crypto (Tier-0) zuerst.
SCHRITT 2 — Lauf je Ziel: cargo mutants --package <crate> [--file '<glob>'] --timeout 180 -j <Kerne/2> --output /tmp/contextra-campaign⚠/L-16/<crate> (Release-Bauprofil nicht erzwingen; Tests laufen im Standardprofil des Crates). Läuft ein Ziel länger als ~6 Stunden: in Shards (--shard k/n) aufteilen und nacheinander ausführen. Zwischenstände loggen.
SCHRITT 3 — Auswertung: Mutation-Score je Ziel und je Datei; überlebende Mutanten nach Sicherheitsrelevanz sortieren (Verifikations-/Auth-Pfade, Vergleichsoperatoren in verify_*, Zähler/Nonce, Längenpräfixe, Grenzwerte in Replay, SSI-Coarsening, Retention-Floors, Tombstone-Regeln). Je überlebendem Mutant: äquivalent [BELEGT] oder Testlücke.
SCHRITT 4 — Killer-Tests: Schreibe für die 25 wichtigsten Testlücken Tests (crates/<crate>/tests/campaign_mut_*.rs) mit unabhängigem Orakel (R4) und führe NUR diese Mutanten erneut aus (cargo mutants --regex "<Mutantenbeschreibung>" bzw. --file), um zu zeigen, dass sie jetzt gefangen werden.
SCHRITT 5 — Gate: ⚠NICHT-IM-REPO(cargo xtask check-mutation-score-gate) (berichten); wenn es einen Score-Record gibt (record-mutation-score), prüfe ob die dokumentierten Scores zu den gemessenen passen (Doku-Wahrheit); schreibe keine Records ins Repo.

[[REPORT]]
```

---

## B.2 Coverage, Soak, Integration, Forensik

### L-17 — Coverage vollständig inkl. Non-Default-Members und Lückenschluss

> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-campaign`, `contextra-checkpoint`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.

**Typ:** Test · **Dauer:** 2–5 h · **Schreibt:** Bericht + Tests

```text
[[CTX]]

AUFTRAG: Messe die Abdeckung des GESAMTEN Workspaces einschließlich der Non-Default-Member (soweit baubar) und schließe kritische Lücken. Ergänzt J-16.

SCHRITT 1 — just coverage-html, just coverage-json, ⚠NICHT-IM-REPO(cargo xtask check-coverage-gate). Zusätzlich je Non-Default-Member: cargo llvm-cov -p <crate> --html --output-dir /tmp/contextra-campaign/L-17/<crate> (py/onnx/sandbox/ollama nur wenn baubar; sonst Umgebungslücke).
SCHRITT 2 — Berichte: Zeilen-, Funktions-, Regions-/Branch-Abdeckung je Crate (Tabelle) und Top-50-Liste ungedeckter Funktionen nach Größe/Risiko. Prüfe zusätzlich die Abdeckung durch Integrationstests getrennt von Unit-Tests (cargo llvm-cov mit --tests bzw. --lib getrennt): Funktionen, die NUR durch Unit-Tests mit Test-Hooks (*_for_test) erreicht werden, sind nicht echt abgedeckt.
SCHRITT 3 — Kritische-Pfade-Liste aus J-16 Schritt 2 abarbeiten: für jede dort genannte Funktion die tatsächliche Abdeckung nennen. Schreibe Tests für die 10 gefährlichsten Lücken (unabhängiges Orakel, R4) und messe erneut.
SCHRITT 4 — Scheinabdeckung: Finde Tests, die Code ausführen, aber nichts prüfen (J-18 Muster); nenne die 10 auffälligsten.
SCHRITT 5 — Fehlerpfad-Abdeckung speziell: Anteil der Err-Zweige (match-Arme mit Err, ?-Operator-Pfade) die je getroffen werden — nutze die Branch-/Region-Daten für contextra-store/src/wal/**, lsm/recovery.rs, manifest/**, contextra-checkpoint⚠/**.

[[REPORT]]
```

### L-18 — Soak-/Langzeit-Stresstest mit Ressourcen-Zeitreihe

> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-campaign`, `contextra-db`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.

**Typ:** Test · **Dauer:** 2–24 h (Parameter) · **Schreibt:** Bericht + CSV/Diagramme in `/tmp`

```text
PARAMETER: DAUER_STUNDEN = 2     SCHREIBER = 8     LESER = 16

[[CTX]]

AUFTRAG: Prüfe Stabilität unter Dauerlast: Speicherwachstum, Dateideskriptor-Lecks, Thread-Lecks, Compaction-Rückstau, Latenz-Drift, Plattenverbrauch. Wegwerf-Harness unter /tmp/contextra-campaign⚠/L-18 (Release, echte Platte). Nutze contextra-testkit (Fixtures/Mock-Engines), soweit passend.

LAST: gemischter Workload über die Facade bzw. contextra-db⚠: Inserts mit Embedding (mock-Provider), hybride Suche (Vektor + Text + Graph), Updates, Löschungen (inkl. Löschbeweis im sovereign-Ring in separatem zweiten Lauf), contextra_consolidate in Intervallen, Checkpoints alle 10 Minuten, Hintergrundwartung (Feature background-maintenance). Datenmenge wächst bis zu einem definierten Plattenlimit und wird dann durch Löschungen im Gleichgewicht gehalten (steady state).
MESSUNG alle 10 s in CSV: RSS und VSZ, Anzahl offener Dateideskriptoren (/proc/<pid>/fd), Threads, Plattenverbrauch des Datenverzeichnisses, WAL-Größe, Anzahl SSTables/Level, StorageHealth/Pressure-Level (health()), Latenz p50/p95/p99/max je Operationsart, Fehlerzählung je Fehlertyp, Compaction-Rückstand, deleted_ratio und Rebuild-Zähler des Vektorindex.
ANALYSE: (a) RSS-Steigung über die zweite Laufhälfte (lineare Regression): signifikant positive Steigung im Steady State = Speicherleck-Verdacht; (b) FD-/Thread-Zahlen müssen konstant bleiben; (c) p99-Drift (erstes vs. letztes Drittel); (d) Write-Stalls und Backpressure-Ereignisse (PressureLevel-Übergänge, Flapping?); (e) Plattenverbrauch vs. Nutzdaten (Space-Amplification); (f) Anzahl Fehler (jeder unerwartete Fehler ist Befund).
STÖRUNGEN (zweiter Lauf, kürzer): SIGSTOP/SIGCONT-Pausen des Prozesses (Zeitsprünge der Clock-Injektion?), Platte fast voll (Verhalten beim Erreichen von < 1 GB frei: kontrollierter Fehler statt Korruption, dann Erholung nach Freigabe), Speicherlimit per Subshell (ulimit -v) bzw. systemd-run --scope -p MemoryMax=<Wert> (falls verfügbar): Verhalten nahe am Limit (Backpressure statt OOM-Kill?).
AUSGABE: CSV-Dateien, Diagramme (matplotlib, falls vorhanden) unter /tmp/contextra-campaign⚠/L-18/, im Bericht Tabellen und die Regressionskoeffizienten. Nach dem Lauf: Verify (L-10 Invarianten) auf dem Datenverzeichnis.

[[REPORT]]
```

### L-23 — Speicher-Forensik: Klartext-Reste und Schlüssel nach Crypto-Shredding

> ⚠ **Ist-Stand-Hinweis:** Dieser Prompt nennt Crates, die am HEAD `c41afc15` nicht existieren: `contextra-campaign`. Vor Ausführung auf vorhandene Crates umschreiben oder als Soll-Befund führen.

**Typ:** Test · **Dauer:** 90–180 min · **Schreibt:** Bericht (+ Wegwerf-Skripte)

```text
[[CTX]]

AUFTRAG: Beweise oder widerlege SEC-02 (S1, kv_shredding.rs ~Z. 188): Nach Crypto-Shredding (Schlüsselvernichtung) bleiben Klartextdaten im Arbeitsspeicher (Memtable, Block-Cache, HNSW-Graph, CSR-Graph, KV-Cache-Tensoren) erhalten, solange kein synchroner Purge erfolgt. Zusätzlich: Überleben Schlüsselbytes nach emergency_wipe/Drop? Dies ist die Laufzeit-Ergänzung zu J-43 (Platten-/Index-Teil). Nur eigene Prozesse untersuchen.

AUFBAU: Wegwerf-Programm unter /tmp/contextra-campaign⚠/L-23 (Release), das (1) einen Collection mit encryption-at-rest und deletion-proof öffnet, (2) Dokumente mit eindeutigem synthetischen Canary einfügt (z. B. "CANARY-MEM-9C1D-ANNA-TESTFALL"; als UTF-8; zusätzlich in Metadaten und Graph-Tripel), (3) liest und sucht (damit Block-Cache, Index und KV-Cache gefüllt sind), (4) nach jedem Schritt auf ein Signal wartet, damit der Speicher von außen inspiziert werden kann.
INSPEKTION (Wegwerf-Python-Skript): lese /proc/<pid>/maps und /proc/<pid>/mem (gleicher Benutzer; falls ptrace_scope das verbietet: gcore <pid> aus gdb und das Core-File durchsuchen) und zähle Treffer des Canary-Musters (UTF-8 und UTF-16LE) je Speicherregion (Heap, Anon-mmap, Datei-mmap, Stack). Ausgabe: Anzahl Treffer und Regionstyp, NIE die Inhalte fremder Daten ausgeben (nur Canary).
MESSPUNKTE: P1 nach Einfügen · P2 nach Lesen/Suchen · P3 nach contextra_forget/revoke_group/Crypto-Shred · P4 nach flush + Compaction + Checkpoint · P5 nach Cache-Eviction/Trim (malloc_trim bzw. Allokator-Eigenheiten notieren) · P6 nach close()/Drop des Collection-Objekts · P7 nach emergency_wipe. Erwartung eines korrekten Systems: ab P3 keine (oder dokumentiert befristete) Treffer; die Analyse behauptet: Treffer bleiben bis nach P3/P4.
SCHLÜSSEL: Gib dem Programm einen bekannten Master-Schlüssel (Testwert aus eigener Quelle, kein produktiver) und suche dessen Bytes (und KEK/DEK-Ableitungen, soweit das Programm sie über eine Test-API kennt) in denselben Messpunkten; nach VolatileEncryptionKey::emergency_wipe/KeyManager::emergency_wipe/Drop dürfen die Schlüsselbytes nicht mehr auffindbar sein. (Zeroize verhindert Dead-Store-Elimination, kopiert aber nicht frühere Kopien — Stack-/Register-Spills und Vec-Reallokationen können Reste hinterlassen: genau das messen.)
ZUSATZ: Swap/Hibernation/Core-Dump-Policy: setzt der Code mlock/MADV_DONTDUMP/PR_SET_DUMPABLE für Schlüsselseiten (rg)? Wenn nein: Befund S3 (Härtungslücke), kein Beweis für Leck.
AUSWERTUNG: Tabelle Messpunkt x Region x Treffer; Befunde mit Schwere (Klartext nach Löschung nachweisbar = S1 laut Vor-Analyse; Schlüsselreste = S2); Fix-Skizze nur als Text (synchroner Purge-Hook an alle Cache-/Index-Schichten, zeroize der Block-Cache-Einträge, Eviction pro Tenant/Key-Gruppe).

[[REPORT]]
```

---
