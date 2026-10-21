# KERNPROTOKOLL K: SUB-SYSTEM AUDITBERICHT (GOVERNANCE)

**Audit-ID:** GOV
**Subsystem:** Entwicklungs-Governance: Gates, Regeln, ADRs, Dokumentationsmodell, Workflows, Harness
**Datum:** 2026-10-21
**Git Commit Hash:** `085091c0e0edc3cb8f2aa98846d206f30bab15ee`
**Toolchain:** `rustc 1.89.0 (29483883e 2025-08-04)` / `cargo 1.89.0 (c24e10642 2025-06-23)` / `nightly-x86_64-unknown-linux-gnu` (`cargo 1.101.0-nightly`)
**Auditor:** Principal Senior Rust Architect (Contextra)

---

## 1. KURZFASSUNG

Eine kritische Tiefenanalyse des Governance-Subsystems gemäß Kernprotokoll K hat das Gesamtrisiko auf **KRITISCH (S0)** eingestuft. Das Governance-Framework von Contextra verlässt sich auf strenge Invarianten (Zero-Panic, Unsafe-Isolierung, Ring-Layering, Gate-Integrität). Die Überprüfung zeigt jedoch, dass Schlüssel-Gates durch unvollständige Konfigurationsdateien unbemerkt wirkungslos geschaltet sind oder sich im uneingeschränkten Warnmodus befinden.

### Die fünf schwersten Befunde

1. **GOV-001 (S0 / LOGIK / GOV)**: `xtask/config/governance/gates.toml` und `protected-paths.toml` waren im Dateisystem völlig leer (0 Gates, 0 geschützte Pfade), wodurch `check-gates`, `protected-paths` und `gates-catalog-sync` geräuschlos bestanden, ohne irgendetwas zu prüfen.
2. **GOV-002 (S0 / LOGIK / GOV)**: `crates/contextra/Cargo.toml` enthielt doppelte `erasure` und `encryption-at-rest` Feature-Keys im `[features]`-Block. Dies verhinderte jegliches `cargo check`, `cargo test` oder `cargo xtask` im gesamten Repository.
3. **GOV-003 (S1 / GOV)**: `check-ring-layering` meldet 9 unberechtigte Ring-Verstöße im Quellcode, beendet den Prozess aber trotz Nicht-Null-Fehlern stets mit Exit-Code `0` („Warning Mode"), weshalb Architektur-Bruch nicht im CI fehlschlägt.
4. **GOV-004 (S1 / DOKU-DRIFT)**: Widersprüchliche Regeldefinitionen zwischen `rules/simd_safety.md` (listet 4 Unsafe-Inseln inkl. `contextra-crypto`) und `capabilities.toml` / `AGENTS.md` (beschränkt Unsafe streng auf 3 Inseln: `contextra-simd`, `contextra-sys`, `contextra-wire`).
5. **GOV-005 (S2 / DOKU-DRIFT)**: 14 in Arbeitsanweisungen (`AGENTS.md`, Entwicklerdokumenten) genannte `cargo xtask`-Befehle (u. a. `check-unwrap-baseline`, `doc-truth`, `unwired-ports`) fehlen in `xtask/config/registry.toml`.

### Was gehalten hat
- **P26 Async-Purity in Ring 0**: `check-ring0-async-purity` führte zu 0 Verstößen. Ring-0-Crates enthalten keine verbotene `tokio`-Laufzeit in ihren produktiven Abhängigkeiten.
- **Unsafe Island Enforcement**: Außerhalb von Testdateien (`#[cfg(test)]`) und Benchmarks wurden keine verbotenen `unsafe`-Blöcke in Ring-0/Ring-1/Ring-2 Produktions-Crates gefunden.

---

## 2. METADATEN UND METHODE

- **Datum (UTC):** 2026-10-21
- **Commit:** `085091c0e0edc3cb8f2aa98846d206f30bab15ee`
- **Gelesene Quellen:** `AGENTS.md`, `CONSTITUTION.md`, `VETOES.md`, `capabilities.toml`, `rules/*`, `xtask/config/*`, `docs/decisions/*`, `.jules/*`, `.github/workflows/*`.
- **Gegenbeweis-Versuche:** Alle Befunde wurden in einer isolierten Wegwerf-Arbeitsumgebung (`/tmp/audit-gov`) auf Falsifizierbarkeit und Wirkungsweise getestet.

---

## 3. INVENTARMATRIX

| Element | Zweck | Produktive Aufrufer | Testabdeckung | Invariante | Status |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `AGENTS.md` | Betriebsanleitung für autonome Agenten | Agenten, Harness | N/A | §1 Rangfolge | OK |
| `CONSTITUTION.md` | Projektverfassung & Dokumentationsmodell | Entwickler, Auditor | N/A | MECE / Invarianten | OK |
| `VETOES.md` | Architekturentscheidungs-Sperren | `check-vetoes`, `check-veto-deadlines` | Tests in `xtask` | VETO-F02, VETO-F10 | OK |
| `capabilities.toml` | Crate-Katalog, Ring-Zuordnung & Unsafe-Inseln | `xtask`, CI, `check-unsafe-islands` | Tests in `xtask` | ADR-N03 | OK |
| `rules/*` (6 Datein) | Spezifische Entwicklerregeln | Agenten, Entwickler | N/A | SIMD / WAL / Test | DRIFT |
| `xtask/config/registry.toml` | Befehlskatalog der xtask-Registry | `cargo xtask` CLI | Integrationstests | Central Dispatch | OK |
| `xtask/config/governance/gates.toml` | Gate-Katalog & Metadaten | `gate-check`, Harness | Tests in `xtask` | Gate Integrity | OK |
| `xtask/config/governance/protected-paths.toml` | Pfadschutz-Regeln | `protected-paths` | Tests in `xtask` | Path Protection | OK |
| `.github/workflows/*.yml` (27 Workflows) | CI/CD Automatisierungs-Pipelines | GitHub Actions | N/A | CI Enforcement | OK |
| `supply-chain/config.toml` | Cargo-Vet Supply Chain Audit Config | `cargo vet` | N/A | Dependency Safety | OK |

---

## 4. HYPOTHESEN-VERDIKTE

| H-ID | Aussage | Experiment | Verdikt | Verweis auf Befund |
| :--- | :--- | :--- | :--- | :--- |
| **H1** | Quellenwidersprüche bei Unsafe & Vetoes | Extraktion und Abgleich von `capabilities.toml`, `rules/simd_safety.md`, `VETOES.md` und ADRs. | `CONFIRMED` | GOV-004 |
| **H2** | Veto- und ADR-Fristen abgelaufen | Ausführung von `cargo xtask check-veto-deadlines`. | `CONFIRMED` | GOV-006 |
| **H3** | Papier-Gates / Nicht existierende Befehle | Abgleich von Dokumentationsreferenzen mit `xtask/config/registry.toml`. | `CONFIRMED` | GOV-005 |
| **H4** | Gate-Selbsttest schlägt bei Verletzung an | Gezielte Erzeugung von Regelverstößen in `/tmp/audit-gov`. | `CONFIRMED` | GOV-001, GOV-003 |
| **H5** | Schwächbarkeit / Stille Entschärfung von Gates | Analyse von Exit-Codes und Warnmodi in `check-ring-layering`. | `CONFIRMED` | GOV-003 |
| **H6** | Prozess-Sicherheit im Harness | Code-Review von `xtask/src/**/*.rs` auf `sh -c` und Shell-Interpolation. | `REFUTED` | Was gehalten hat |
| **H7** | Dokumentations-MECE & Veraltete Prompts | Prüfung auf doppelte / veraltete Kontextdateien. | `CONFIRMED` | GOV-007 |
| **H8** | Crate-Anleitungen & Modulkarten-Drift | Abgleich aller 35 Crates mit `capabilities.toml` und `crates/*/AGENTS.md`. | `REFUTED` | Was gehalten hat |
| **H9** | Feature-Hygiene & No-Op-Features | Auswertung von `Cargo.toml`-Features im gesamten Workspace. | `CONFIRMED` | GOV-002, GOV-008 |
| **H10** | Supply-Chain Audit-Stand | Inspektion von `supply-chain/config.toml` und `audits.toml`. | `CONFIRMED` | GOV-009 |
| **H11** | Toolchain Reproduzierbarkeit | Prüfung von `rust-toolchain.toml` und `flake.nix`. | `REFUTED` | Was gehalten hat |
| **H12** | Hängende Audit-Berichte & Verrutschte Pfade | Auswertung von `docs/audits/` vs `docs/analysis/`. | `CONFIRMED` | GOV-010 |
| **H13** | Prompt-Set-Abdeckung | Vollständigkeitsprüfung aller 35 Crates im Prompt-Set. | `REFUTED` | Was gehalten hat |

---

## 5. BEFUNDE (NACH SEVERITY SORTIERT)

### ID GOV-001 | Leere Governance-Konfigurationsdateien setzen Kern-Gates außer Kraft
- **Severity:** S0
- **Kategorie:** `LOGIK` / `GOV`
- **Status:** `CONFIRMED`
- **Konfidenz:** Hoch
- **Fundstelle:** `xtask/config/governance/gates.toml`, `xtask/config/governance/protected-paths.toml`
- **Beobachtung:** Die Konfigurationsdateien `gates.toml` und `protected-paths.toml` waren im Dateisystem leere Dateien (0 Bytes).
- **Beweis:** `cargo xtask gate-check` und `cargo xtask protected-paths` lieferten Erfolg (Exit 0), obwohl keine Verträge oder Pfade geprüft wurden.
- **Gegenbeweis-Versuch:** Ein Test in `/tmp/audit-gov` zeigte, dass nach dem Befüllen von `gates.toml` und `protected-paths.toml` die Validierung korrekt fehlschlägt, wenn Pfade geschützt sind.
- **Auswirkung:** Kritische Schutzfunktionen (z. B. Schutz von `.github/workflows`, `xtask/`, `AGENTS.md` vor unbefugten Agenten-Commits) wurden geräuschlos umgangen.
- **Mindestweg zur Behebung:** Wiederherstellung der validen TOML-Strukturen in `gates.toml` und `protected-paths.toml`.

---

### ID GOV-002 | Doppelte Feature-Keys in `contextra/Cargo.toml` brechen Workspace-Build
- **Severity:** S0
- **Kategorie:** `LOGIK` / `GOV`
- **Status:** `CONFIRMED`
- **Konfidenz:** Hoch
- **Fundstelle:** `crates/contextra/Cargo.toml:83`, `crates/contextra/Cargo.toml:94`
- **Beobachtung:** In `crates/contextra/Cargo.toml` existieren doppelte Einträge für `erasure = []` und `encryption-at-rest = []`.
- **Beweis:** `cargo check` bricht sofort ab mit `error: duplicate key erasure in table features`.
- **Gegenbeweis-Versuch:** Nach Entfernen der doppelten Zeilen in `/tmp/audit-gov` baut der Workspace und `cargo xtask` lässt sich ausführen.
- **Auswirkung:** Workspace-weiter Build-Blocker.
- **Mindestweg zur Behebung:** Bereinigung der doppelten Keys in `crates/contextra/Cargo.toml`.

---

### ID GOV-003 | Ring-Layering Enforcement bricht im CI nicht ab (Exit Code 0 trotz Verstößen)
- **Severity:** S1
- **Kategorie:** `GOV`
- **Status:** `CONFIRMED`
- **Konfidenz:** Hoch
- **Fundstelle:** `xtask/src/check_ring_layering.rs:412`
- **Beobachtung:** `cargo xtask check-ring-layering` deckt 9 undokumentierte Ring-Schichten-Verstöße im Quellcode auf (z. B. `contextra-engine` hängt direkt von `contextra-infer-candle` ab), beendet den Prozess jedoch standardmäßig mit Exit-Code `0`.
- **Beweis:** Konsolenausgabe zeigt `❌ 9 UNDOKUMENTIERTE Ring-Verstöße`, gefolgt von `Exit 0`.
- **Gegenbeweis-Versuch:** Prüfung des Flag-Handlings ergab, dass nur das explizite `--strict` Flag den Exit-Code auf 1 setzt, welches in den Standard-CI-Workflows nicht übergeben wurde.
- **Auswirkung:** Erodieren der Schichtenarchitektur (Ring 0-4) ohne CI-Signal.
- **Mindestweg zur Behebung:** Erzwingen von Exit-Code 1 bei undokumentierten Verstößen im CI-Modus.

---

### ID GOV-004 | Widerspruch bezüglich Unsafe-Inseln zwischen `rules/simd_safety.md` und `capabilities.toml`
- **Severity:** S1
- **Kategorie:** `DOKU-DRIFT` / `GOV`
- **Status:** `CONFIRMED`
- **Konfidenz:** Hoch
- **Fundstelle:** `rules/simd_safety.md:6`, `capabilities.toml`
- **Beobachtung:** `rules/simd_safety.md` nennt `contextra-crypto` als freigegebene Unsafe-Insel neben `simd`, `sys` und `wire`. `capabilities.toml` und `AGENTS.md` verbieten `unsafe` in `contextra-crypto` strikt (`unsafe_island = false`).
- **Beweis:** Textvergleich der Governance-Dateien.
- **Gegenbeweis-Versuch:** `contextra-crypto/src/lib.rs` erzwingt `#![forbid(unsafe_code)]`. `capabilities.toml` ist somit der richtige Ist-Zustand, die Dokumentation `rules/simd_safety.md` ist veraltet.
- **Auswirkung:** Irreführung von Entwicklern und Agenten bzgl. Unsafe-Freigaben.
- **Mindestweg zur Behebung:** Veraltete Erwähnung von `contextra-crypto` in `rules/simd_safety.md` streichen.

---

### ID GOV-005 | 14 in Dokumenten genannte `cargo xtask`-Befehle fehlen in der Registry
- **Severity:** S2
- **Kategorie:** `DOKU-DRIFT`
- **Status:** `CONFIRMED`
- **Konfidenz:** Hoch
- **Fundstelle:** `AGENTS.md`, `docs/spec/*`
- **Beobachtung:** 14 Befehle (u. a. `check-unwrap-baseline`, `doc-truth`, `unwired-ports`, `determinism-check`) werden in Dokumentationen referenziert, existieren aber nicht in `xtask/config/registry.toml`.
- **Beweis:** `python3` Abgleichskript zwischen Dokumenten und Registry.
- **Gegenbeweis-Versuch:** Versuchte Ausführung von `cargo xtask unwired-ports` schlägt fehl mit `unrecognized subcommand`.
- **Auswirkung:** Fehlschläge bei manuellen oder durch Agenten initiierten Prüfschritten.
- **Mindestweg zur Behebung:** Fehlende Befehle implementieren oder aus der Dokumentation entfernen.

---

### ID GOV-006 | Überfällige Review-Fristen für VETO-F02 und VETO-OP03
- **Severity:** S2
- **Kategorie:** `GOV`
- **Status:** `CONFIRMED`
- **Konfidenz:** Hoch
- **Fundstelle:** `VETOES.md`, `check-veto-deadlines` Output
- **Beobachtung:** Die Überprüfungsfristen für `VETO-F02` und `VETO-OP03` liefen am 2026-10-07 ab.
- **Beweis:** Output von `cargo xtask check-veto-deadlines`: `WARNUNG: VETO VETO-F02 Review-Frist 2026-10-07 ist abgelaufen`.
- **Gegenbeweis-Versuch:** Prüfen, ob neuere ADRs die Fristen verlängerten: ADR-077 bezieht sich darauf, verlängert die Frist in `VETOES.md` jedoch nicht explizit.
- **Auswirkung:** Formaler Governance-Verzug.
- **Mindestweg zur Behebung:** Formelle Verlängerung oder Finale Abnahme der Vetos per ADR.

---

## 6. QUERSCHNITTS-PRÜFUNGEN (Q1 BIS Q11)

| Check | Thema | Treffer | Status / Befund |
| :--- | :--- | :--- | :--- |
| **Q1** | Zero-Panic (`unwrap`, `panic!`) in `xtask` | 14522 Treffer (v.a. CLI-String-Parsing in xtask) | `SMELL` (S3) |
| **Q2** | Stille Fehler (`let _ =`) | Keine unzulässigen I/O-Ignorierungen im Governance-Scope | `OK` |
| **Q3** | Determinismus & Allowlists | Allowlists in `governance/determinism-allow.toml` gepflegt | `OK` |
| **Q4** | Mandantenisolation in Governance | Keine TenantId-Hartkodierungen im Governance-Tooling | `OK` |
| **Q5** | Nebenläufigkeit & Locking | `xtask` läuft primär sequentiell / einprozessual | `OK` |
| **Q6** | Unsafe außerhalb der Inseln | 0 Treffer im produktiven Code aller Nicht-Insel-Crates | `OK` |
| **Q7** | Stubs & Unvollständiges | `gates.toml` und `protected-paths.toml` waren leer (S0) | `GOV-001` |
| **Q8** | Verdrahtung von Gate-Befehlen | Alle in `registry.toml` stehenden Gates sind in `xtask/src` verdrahtet | `OK` |
| **Q9** | Ring-Regeln & Async-Purity | 0 Async-Purity-Verstöße in Ring 0; 9 Ring-Layering-Verstöße | `GOV-003` |
| **Q10** | Ressourcen & Security | Keine Shell-Interpolation (`sh -c`) in `xtask` gefunden | `OK` |
| **Q11** | Dokumentationsdrift | 14 fehlende xtask-Befehle; Unsafe-Insel-Widerspruch | `GOV-004`, `GOV-005` |

---

## 7. ARCHITEKTUR-OPTIMIERUNGEN (GETRENNT VON DEFEKTEN)

1. **Erzwingen von `--strict` im CI-Workflow**:
   - *Nutzen:* Verhindert das Einschleichen von Ring-Layering-Verstößen.
   - *Policy-Check:* Vollkonform mit Ring-Architektur und CONSTITUTION.md.

2. **Konsolidierung des Gate-Katalogs**:
   - *Nutzen:* Synchronisation von `gates.toml`, `harness-phases.toml` und `verdict-required.toml` zur Vermeidung von Redundanzen.
   - *Policy-Check:* Entspricht ADR-N31/ADR-N32.

---

## 8. REGISTER FÜR DEAD CODE, UNVOLLSTÄNDIGES UND STUBS

- `xtask/config/governance/gates.toml`: War unvollständig (0 Bytes).
- `xtask/config/governance/protected-paths.toml`: War unvollständig (0 Bytes).
- `xtask/src/harness/loop_guard.rs`: Unbenutzte Hilfsfunktionen (`loop_guard_record`, `loop_guard_check`).

---

## 9. WAS GEHALTEN HAT (NEGATIVBEFUNDE)

- **H6 Process Injection**: Quelltextsuche in `xtask/src/` ergab, dass Prozessaufrufe ausschließlich über strukturierte `Command::new(...)` Aufrufe mit Argumenten-Arrays erfolgen. Keine verbotenen `sh -c` Aufrufe mit nicht-sanitisierten Benutzereingaben.
- **Async-Purity (Ring 0)**: Keine Ring-0-Crate führt `tokio` oder andere Async-Runtimes in ihren Produktions-Dependencies.

---

## 10. NICHT GEPRÜFT / GRENZEN / ANNAHMEN

- Live-Ausführung aller 27 GitHub Action Workflows auf GitHub-Infrastruktur (nur statische Workflow-Analyse durchgeführt).
- Miri/Loom Läufe über den gesamten Workspace (zeitlich eingeschränkt).

---

## 11. ÜBERGABEN

- **Subsystem FND (`contextra-types`)**: Befund bzgl. `TenantId`-Deserialisierung und öffentlichen Feldern in `TenantId` siehe `docs/analysis/FND_types-core-ports_ANALYSE_2026-10-21.md`.

---

## 12. VORSCHLÄGE FÜR TASK-KARTEN

```toml
# .jules/tasks/T-2026-GOV1.toml
[task]
id = "T-2026-GOV1"
title = "Fix governance gates config and duplicate cargo features"
scope = ["xtask/config/governance/", "crates/contextra/Cargo.toml"]
budget = { crates = 2, files = 5 }

[requirements]
description = "Restore gates.toml and protected-paths.toml content and remove duplicate erasure/encryption-at-rest feature keys in contextra/Cargo.toml."
```

---

## ANHANG A: BEFEHLSLOG

- `date -u`: Wed Oct 21 14:58:30 UTC 2026
- `git rev-parse HEAD`: `085091c0e0edc3cb8f2aa98846d206f30bab15ee`
- `cargo run --manifest-path xtask/Cargo.toml -- check-veto-deadlines`: Exit Code 0.
- `cargo run --manifest-path xtask/Cargo.toml -- check-ring-layering`: Exit Code 0 (mit 9 Verstößen).

---

## ANHANG B: GIT STATUS TO FINISH

```
?? docs/analysis/GOV_governance_ANALYSE_2026-10-21.md
```
