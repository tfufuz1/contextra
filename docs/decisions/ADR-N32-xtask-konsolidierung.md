# ADR-N32: xtask-Konsolidierung, Verzeichnisbereinigung und Registry-Migration

* **Status:** Accepted
* **Datum:** 2026-10-19
* **Autoren:** Principal Architect (Contextra)
* **Betroffene Komponenten:** `xtask/`, Governance-Konfigurationen (`governance/`), Dokumentation (`docs/gates/`)
* **Spezifikationsreferenzen:** Spezifikation v15 Teil G (xtask & Gate Harness), `governance/protected-paths.toml`

---

## 1. Ist-Zustand & Befund (mit Pfad- und Zeilennachweisen)

Eine unvoreingenommene Analyse der `xtask`-Implementierung und der zugehörigen Governance-Verzeichnisse zeigt erhebliche Code-Umfänge und verstreute Konfigurationspfade:

1. **Massiver Codeumfang in `xtask`:**
   - Das Verzeichnis `xtask/src/` umfasst **131 Quellcodedateien** mit insgesamt **43.324 Zeilen Code**.
   - Die Datei `xtask/src/main.rs` ist als God-Object mit **4.642 Zeilen Code** strukturiert und enthält überlappende CLI-Dispatch-Match-Arme (z. B. doppelte Vorkommen von Commands wie `"check-commit-diff-integrity"` in Zeile 2153 und 2442).

2. **Verstreute Governance- und Gate-Konfigurationen:**
   - Konfigurationen sind uneinheitlich über drei Top-Level-Verzeichnisse verteilt:
     - `governance/`: Enthält `gates.toml` (5.064 Bytes), `protected-paths.toml` (2.820 Bytes), `verdict-required.toml` (1.448 Bytes), `determinism-allow.toml` u. a.
     - `docs/gates/`: Enthält 44 Markdown-Dokumentationen zu einzelnen Gates (insgesamt über 200 KB).
     - `xtask/`: Enthält Crate-Quellcode und eigene Harness-Logik.

3. **Legacy-Registry-Altlasten:**
   - In der internen Gate- und Command-Registry von `xtask` befinden sich **110 als "legacy" eingestufte Registry-Einträge**, die historische oder abgelöste Teil-Checks repräsentieren.

---

## 2. Entscheidung & Soll-Zustand

1. **`xtask/` als einzige Heimat des Gate-Systems:**
   - Das Verzeichnis `xtask/` wird zum alleinigen Speicherort für den gesamten Gate-Code, alle Gate-Konfigurationen und Gate-Dokumentationen konsolidiert.
   - **Neue Zielstruktur:**
     - `xtask/config/governance/`: Beherbergt alle bisherigen `governance/`-Konfigurationen (`gates.toml`, `protected-paths.toml`, `verdict-required.toml` etc.).
     - `xtask/config/`: Beherbergt spartenübergreifende Regelsätze (`registry.toml`, `ring0-async-exceptions.toml`, `mutants-guarantees.toml`, `caller-allowlist.toml`).
     - `xtask/docs/gates/`: Beherbergt alle Gate-Spezifikationen und Beschreibungen (bisher unter `docs/gates/`).

2. **Selbstschutz via `protected-paths.toml`:**
   - Die Datei `protected-paths.toml` (künftig unter `xtask/config/governance/protected-paths.toml`) wird so angepasst, dass sie die neuen Pfade `xtask/config/**` und `xtask/docs/gates/**` lückenlos als geschützte Pfade erfasst.

3. **Modularisierung von `main.rs`:**
   - Das God-Object `xtask/src/main.rs` (4.642 LOC) wird in klare CLI-Submodule (`xtask/src/cli/`) aufgeteilt. Unerreichbare oder doppelte Match-Arme werden eliminiert.

4. **Bereinigung der 110 Legacy-Registry-Einträge:**
   - Die 110 "legacy"-Registry-Einträge werden systematisch analysiert. Veraltete Einträge werden entfernt; verbleibende aktive Registry-Funktionen werden in die konsolidierten Gate-Klassen überführt.

---

## 3. Konsequenzen

* **Positiv:**
  - Aufgeräumte Root-Verzeichnisstruktur (Entfall des Top-Level-Verzeichnisses `governance/`).
  - Einzige Anlaufstelle für Gate-Entwickler und Maintainer ausschließlich in `xtask/`.
  - Höhere Wartbarkeit durch Entlastung der `main.rs` und Eliminierung von Legacy-Code.
* **Negativ:**
  - Pfadänderungen in CI-Workflows und Skripten erforderlich (`governance/` → `xtask/config/governance/`).

---

## 4. Verworfene Alternativen

* **Beibehaltung der verteilten Verzeichnisse (`governance/`, `docs/gates/`):**
  Verworfen, da die Trennung von Gate-Code (`xtask/`) und Gate-Konfiguration (`governance/`) zu Orientierungsproblemen und redundanten Pfad-Prüfungen führte.

---

## 5. Migrationsschritte

1. Verschieben von `governance/` nach `xtask/config/governance/`.
2. Verschieben von `docs/gates/` nach `xtask/docs/gates/`.
3. Verschieben von `registry.toml`, `ring0-async-exceptions.toml` u. a. nach `xtask/config/`.
4. Aktualisierung der Pfad-Referenzen in `protected-paths.toml` und allen `xtask`-Reader-Funktionen.
5. De-Komposition von `xtask/src/main.rs` in modulare CLI-Komponenten und Bereinigung der 110 Legacy-Registry-Einträge.
