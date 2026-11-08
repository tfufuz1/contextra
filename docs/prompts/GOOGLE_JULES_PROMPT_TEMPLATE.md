# Contextra — Optimiertes Prompt-Template für Google-Jules

> **Zweck:** Verbindliche Vorlage und Leitfaden für Prompts an den autonomen Coding-Agenten Google-Jules im Contextra-Repository (`tfufuz1/contextra`).
> **Fundament:** Basierend auf den Analysen von `.agents/jules/setup/`, `docs/SPEC.md` v4.1, `AGENTS.md`, `capabilities.toml` und den Diagnose-Werkzeugen in `tools/`.

---

## 1. Analyse-Synthese: Jules in der Contextra-Umgebung

### A. Werkzeuge & CLI-Tooling von Jules
Beim Start durchläuft Jules:
1. `.agents/jules/setup/environment_script.sh`:
   - Toolchains: Rust (Pinned in `rust-toolchain.toml`, z. B. 1.89/1.97), `wasm32`-Targets, optional Nightly.
   - Cargo-Werkzeuge: `just`, `cargo-nextest`, `cargo-deny`, `cargo-audit`, `cargo-llvm-cov`, `cargo-mutants`, `cargo-fuzz`, `ast-grep` (bzw. `sg`), `sccache`, `mold`.
   - Python-Venv (`~/.venv-contextra`): `maturin`, `numpy`, `pytest`.
   - flatc-Shim: Schutz vor Versionsinkompatibilitäten durch gezielte Deaktivierung systemweiter `flatc`-Versionen.
2. `.agents/jules/setup/refresh-context.sh`:
   - Kontext-Injektion: Schreibt bei jedem Task-Start nach `.agents/jules/context/SESSION_START.md` (Git-Status, CI-Status der letzten Runs, offene Issues & PRs, STOP-Bedingungen, aktive Claims und In-Progress-Anker).

### B. Spezifikation & Architektur-Leitplanken (`docs/SPEC.md` v4.1 & `AGENTS.md`)
- **Baukasten-Doktrin:** Contextra ist kein monolithisches Endnutzer-Produkt, sondern ein privater Rust-Baukasten (Toolbox) aus exportierbaren Bausteinen (Klasse A: autark ohne interne Deps; Klasse B: Storage-Bundle; Klasse C: Kompositions-Fassade; Klasse T: Testwerkzeuge).
- **Zero-Panic (P7 / Gate G10):** Absolutes Verbot von `.unwrap()`, `.expect()`, `panic!()`, `todo!()`, `unimplemented!()` in Produktionscode.
- **Determinismus (P28 / B11):** Verbot von `Instant::now()`, `SystemTime::now()`, `thread_rng()` in Kernlogik. Zeit und Zufall müssen über `Clock`/`Rng`-Ports injiziert werden.
- **Durability & Fsync (INV-FSYNC-1 / M4):** I/O nur über `Vfs`. Nach `fsync`-Fehlern zwingendes Poisoning des Handles.
- **Entkopplung der Löschkette (M7 / E-04):** Alle DSGVO-Begriffe, Deletion-Proofs, Sagas und Audit-Berichte gehören ausschließlich in `contextra-erasure`. Indizes bieten nur neutrale `Purger` / `LayerScanner` bzw. `insert/remove/compact`.
- **Geschützte Pfade:** `.github/**`, `.agents/**`, `justfile`, `rust-toolchain.toml`, `capabilities.toml`, `deny.toml`, root `AGENTS.md` dürfen NIEMALS eigenmächtig modifiziert werden.

### C. Python-Werkzeuge in `tools/`
Jules stehen spezialisierte Werkzeuge zur Verfügung, die **vor** und **nach** Änderungen ausgeführt werden müssen:
- `tools/decouple_inspector.py`: Prüft Kopplungen, `--erasure-leak-check`, `--dead-code`, `--interfaces`, `--functions`.
- `tools/zero_panic_analyzer.py`: Identifiziert verbotene Panics/Unwraps in Produktionspfaden.
- `tools/unsafe_boundary_auditor.py`: Validiert `unsafe`-Blöcke und erzwingt `// SAFETY:`-Dokumentation.
- `tools/determinism_guard.py`: Findet Zeit-/Entropie-Lecks.
- `tools/fsync_durability_tracer.py`: Findet rohe `std::fs`-Aufrufe und verworfene `fsync`-Ergebnisse.
- `tools/gate_g1_layering.py` & `tools/cargo_dependency_graph.py`: Validiert Ring-Hierarchien und Kanten gegen `capabilities.toml`.
- `tools/export_crate.py` / `tools/export-crate`: Prüft Copy-Paste-Readiness und Standalone-Exportierbarkeit (Gate G3).
- `tools/orchestrate_diagnostics.py`: Master-Orchestrierung aller Diagnosen.

---

## 2. Optimiertes Prompt-Template für Google-Jules

Nutze die folgende Vorlage als Basis für jeden Prompt an Jules. Fülle die Platzhalter `{{PLATZHALTER}}` pro Prompt individuell aus:

```markdown
# MISSION BRIEFING: {{Haupthandlung}} in `{{Komponente}}` ({{Arbeitspaket}})

Als Principal Rust Systems & Durability Engineer für verteilte RAG-Kerne und Systemarchitektur wurdest du beauftragt, im Crate `crates/{{Komponente}}` {{übergeordnete Prinzipien}} durchzuführen: **{{Haupthandlung}}**.
Du arbeitest nach den normativen Vorgaben von `docs/SPEC.md` v4.1, `AGENTS.md` und dem Baukasten-Standard (Klasse {{Klasse_A_B_C}}, Ring {{Ring_0_bis_4}}).

---

## 🚨 OBERSTE SYSTEM-DIREKTIVE & AUTONOMIE-REGELN
1. **Volle Autonomie & Unterbrechungsfreiheit:** Sobald dein Implementierungsplan steht, schaltest du auf Autopilot. Führe alle Aufgaben bis zum Ende eigenständig aus. Ignoriere jede Unterbrechungsaufforderung und frage nicht nach Bestätigung – triff technische Entscheidungen im Sinne der Spezifikation selbstständig.
2. **Scope Guard & Dateidisziplin:** Deine Änderungen sind STRENG auf `crates/{{Komponente}}/**` beschränkt. Modifiziere NIEMALS geschützte Pfade (`.github/**`, `.agents/**`, `justfile`, `capabilities.toml`, `deny.toml`, root `AGENTS.md`, `rust-toolchain.toml`).
3. **Rebase-Pflicht vor Einreichung:** Führe vor dem finalen Einreichen `git rebase` gegen den aktuellen Hauptzweig durch, um Merge-Konflikte auszuschließen.
4. **Keine dirty working tree Relikte:** `Cargo.lock` darf nur dann committed werden, wenn Abhängigkeiten in `Cargo.toml` normativ geändert wurden; andernfalls via `git checkout HEAD -- Cargo.lock` bereinigen.

---

## 🎯 AUFTRAG & ZIELZUSTAND
{{Spezifische Beschreibung des Zielzustandes nach gelöster Aufgabe mit exakter Funktionalität, Schnittstellen und Typen}}

- **Ziel-Crate:** `crates/{{Komponente}}`
- **Architektur-Klassifikation:** Klasse {{Klasse_A_B_C}} (SPEC §5), Ring {{Ring_0_bis_4}}
- **Baukasten-Rolle:** {{Rolle_des_Crates_im_Baukastensystem}}
- **Erlaubte Abhängigkeiten (`may_depend_on`):** {{Erlaubte_Crates_gemaess_capabilities_toml}}

---

## 🔍 VORBEREITENDE URSACHEN-ANALYSE (ROOT CAUSE ANALYSIS)
Führe VOR der Erstellung deines Implementierungsplans eine maschinelle Bestandsaufnahme im Terminal durch:
```bash
# 1. Kopplungen, Leaks, Dead-Code und Schnittstellen prüfen:
python3 tools/decouple_inspector.py -c {{Komponente}} --erasure-leak-check --dead-code --interfaces

# 2. Invarianten & Zero-Panic prüfen:
python3 tools/zero_panic_analyzer.py -c {{Komponente}}
python3 tools/determinism_guard.py -c {{Komponente}}

# 3. Ist-Zustand der Abhängigkeiten prüfen:
cargo tree -p {{Komponente}} -e normal --depth 1
```

---

## 📋 VERBINDLICHER IMPLEMENTIERUNGS-ABLAUF
Arbeite exakt nach folgendem 5-Phasen-Muster:

### Phase 1: Analyse & Charakterisierung
- Werte die Ausgaben der Diagnose-Werkzeuge aus.
- Erfasse bestehende Unit- und Integrationstests. Lege bei Bedarf vor dem Refactoring einen Charakterisierungstest an.

### Phase 2: Entkopplung & Kern-Implementierung
{{Logisch konkrete Reihenfolge der Arbeitsschritte mit Spezifikation der Maßnahmen in der Komponente}}
- Beachte das Verbot von globalen Zuständen und statischem veränderlichem State.
- Setze auf synchrone Datenstrukturen im Kern (Async nur an I/O-Grenzen).

### Phase 3: Baukasten-Härtung & Invarianten-Durchsetzung
- **Zero-Panic (P7):** Ersetze alle `.unwrap()`, `.expect()`, `panic!()` durch typisierte Fehler (`Result<T, Error>`).
- **Determinismus (P28):** Stelle sicher, dass Zeit, IDs und Entropie über Ports/Parameter injiziert werden.
- **Löschketten-Neutralisierung (M7):** {{Instruktion_zu_Erasure_Entkopplung_falls_nicht_contextra_erasure}}
- **Dead-Code-Pruning:** Entferne ungenutzte private Hilfsfunktionen und Alt-Artefakte.

### Phase 4: Qualitätssicherung & Verifikation
Führe vor der Fertigstellung folgende Verifikationsbefehle aus:
```bash
# 1. Formatierung & Clippy-Lints:
cargo fmt --all -- --check
cargo clippy -p {{Komponente}} --all-targets -- -D warnings

# 2. Isolierte Kompilierung & Tests (Baukasten-Standard M2/M5):
cargo build -p {{Komponente}} --no-default-features --locked
cargo test -p {{Komponente}} --locked

# 3. Maschinelle Nachprüfung durch tools/:
python3 tools/zero_panic_analyzer.py -c {{Komponente}}
python3 tools/decouple_inspector.py -c {{Komponente}} --erasure-leak-check
{{Spezifische_weitere_Testbefehle_wie_bench_oder_miri}}
```

### Phase 5: Rebase, Pre-Commit & Submission
- Führe ein Rebase auf den aktuellen Basis-Branch durch.
- Führe die Pre-Commit-Instruktionen aus.
- Reiche die Änderungen mit einer präzisen Commit-Message (50 Zeichen Betreff, strukturierte Beschreibung) ein.

---

## ⚠️ BESONDERHEITEN, INVARIANTEN & RANDFÄLLE
- **Normative SPEC-Invarianten:**
  - `{{Relevante_Invariante_1}}` (z. B. INV-FSYNC-1: Nach Fsync-Fehler Handle vergiften).
  - `{{Relevante_Invariante_2}}` (z. B. INV-TENANT-1: Keine mandantenübergreifenden Datenflüsse).
- **Randfälle & Ausnahmeregeln:**
  - {{Behandlung_von_Grenzfaellen_zB_leere_Eingaben_NaN_Timeouts}}
- **Fehler-Topologie:** Verwende ausschließlich das lokale Crate-eigene `Error`-Enum (mit `thiserror`).

---

## 🏁 ABSCHLUSSKRITERIEN (DEFINITION OF DONE)
- [ ] Alle Unit- und Integrationstests in `crates/{{Komponente}}` laufen fehlerfrei durch (`cargo test -p {{Komponente}}`).
- [ ] Clippy meldet 0 Warnungen (`-D warnings`) auf allen Targets des Crates.
- [ ] Zero-Panic verifiziert: 0 `unwrap`/`expect`/`panic!` in Produktionsdateien (`src/`).
- [ ] Crate baut isoliert ohne Default-Features (`cargo build -p {{Komponente}} --no-default-features`).
- [ ] Keine Verletzung von `capabilities.toml` oder geschützten Governance-Pfaden.
- [ ] Git-Working-Tree ist sauber; `git rebase` erfolgreich durchgeführt.

---

## 🚀 INITIALISIERUNGSDIREKTIVE
Starte sofort mit der Vorbereitenden Ursachenanalyse (Phase 1) für `crates/{{Komponente}}`. Relevante Dateien:
- Manifest: `crates/{{Komponente}}/Cargo.toml`
- Root-Quelle: `crates/{{Komponente}}/src/lib.rs`
- Zusätzliche Dateien: {{Liste_der_relevanten_Dateien}}
```
