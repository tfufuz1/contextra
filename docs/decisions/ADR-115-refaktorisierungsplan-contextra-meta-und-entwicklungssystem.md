# ADR-115: Refaktorisierungsplan Contextra Meta- & Entwicklungssystem (xtask / .jules / CI)

* **Datum**: 2026-10-10
* **Status**: 🟢 Accepted / Decided
* **Betroffene Komponenten**: `xtask/`, `xtask-heavy/`, `.jules/`, `.github/workflows/`, `justfile`, `.githooks/`, `governance/`
* **Spezifikationsreferenzen**: `CONSTITUTION.md`, `AGENTS.md`, `governance/gates.toml`, `.jules/harness-phases.toml`

---

## 1. Ausgangslage & Befund

Das Meta- und Entwicklungssystem von Contextra ist durch organisches Wachstum auf **~58.500 Zeilen / 238 Dateien** in `xtask`/`xtask-heavy` angewachsen.
Folgende Kernschwachstellen wurden identifiziert:

1. **Zwei parallele Dispatch-Mechanismen**: `xtask/build.rs` auto-generiert Subcommands aus `xtask/src/harness/*.rs` (46 Module), während `xtask/src/cli/mod.rs` eine manuelle `COMMAND_DISPATCH_TABLE` (~90 Einträge) führt.
2. **Redundante Submit-Gates**: `harness/jules.rs` Phase `submit` existiert unabhängig von `jules_submit_gate.rs` (Worktree-Hash Ledger), was zu zwei Torwächtern führt, die sich gegenseitig nicht kennen.
3. **Mehrfache Veto-/Deadline-Logik**: `check_vetoes.rs`, `veto_deadline_gate.rs` und `check_adr_deadlines.rs` duplizieren Structs und Ablauflogik.
4. **Duplikation von CI- und Lokal-Checks**: Ca. 20–25 CI-Jobs führen reine `cargo`/`xtask`-Checks aus, die auch lokal in der Jules-VM laufen können.

---

## 2. Architekturentscheidung

Es wird eine schrittweise Konsolidierung des Meta-Systems beschlossen:

1. **Einheitliche Single Source of Truth**: `governance/gates.toml` wird zur alleinigen Quelle der Wahrheit für Qualitäts-Gates, Phasen (`check`, `verify`, `submit`, `ci-confirm`) und Ausführungsorte (`local-vm`, `ci-only`).
2. **Generierter Phasenlebenszyklus**: `.jules/harness-phases.toml` und CI-Gate-Konfigurationen werden aus `governance/gates.toml` generiert.
3. **Kryptografische Jules-VM-Eintrittskarte**: Der Worktree-Hash Ledger aus `jules_submit_gate.rs` wird vollständig in `cargo xtask jules submit` integriert. CI-Workflows verifizieren primär diesen Ledger-Eintrag für Tier-0/1-Checks.
4. **Verallgemeinerter GateCheck Trait**: Die Gate-Ergebnistypen in `xtask/src/` werden auf eine gemeinsame Abstraktion (`GateFinding` / `GateResult` / `GateCheck`) umgestellt.

---

## 3. Inkrementeller Phasenplan

* **Phase 0**: Fixieren der Diagnose durch ADR-115 und Nachtragen fehlender Crate-Inventar-Einträge (`contextra-durable-fs` in `ARCHITECTURE.md`).
* **Phase 1**: Verallgemeinerung des `GateCheck`-Traits über alle Gate-Module.
* **Phase 2**: Konsolidierung von Submit-Gate, Preflight und Jules-Lebenszyklus.
* **Phase 3**: Vereinheitlichung der Veto- und Deadline-Checker.
* **Phase 4**: Konsolidierung des Dispatch-Mechanismus und automatische Generierung von `.jules/harness-phases.toml`.
* **Phase 5**: Entlastung der CI-Pipelines durch Verifikation des VM-Worktree-Ledgers.
* **Phase 6**: Aufräumarbeiten & Härtung.

---

## 4. Konsequenzen

* **Positiv**: Massive Reduktion der Code-Duplikation in `xtask`, signifikant schnellere CI-Laufzeiten durch Vermeidung doppelter Check-Ausführungen.
* **Negativ/Risiko**: Erfordert sorgfältiges Testen der Migration, um Unterbrechungen aktiver Jules-Sessions zu vermeiden.
