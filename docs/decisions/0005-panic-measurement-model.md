# Decision Proposal: Panic-Messmodell für den Contextra Workspace

- **Status:** Proposed
- **Datum:** 2026-10-29
- **Betroffene Komponenten:** `governance/ratchet.toml`, `xtask`, `docs/VETOES.md`

## Problemstellung
In bisherigen Analysen wurden drei unterschiedliche Kennzahlen für Panic-/Unwrap-Vorkommen im Codebase-Scan berichtet:
1. `unwrap_exact = 502`: Exakte Treffer von `.unwrap()` Aufrufen im gesamten Workspace (inklusive Tests, Benches, Examples).
2. `expect_exact = 242`: Exakte Treffer von `.expect(...)` Aufrufen.
3. `unwrap_legacy_grep = 1175`: Breiter RegEx/Grep, der auch Test-Fixtures, Kommentare, Dokutests und `unwrap_or`/`unwrap_err` erfasst.

Diese Abweichungen führen zu Verwirrung bei Ratchet-Messungen.

## Analyse & Definition

| Kennzahl | Suchmuster | Zielbereich | Messwert | Bedeutung |
|---|---|---|---|---|
| **K1: Production Surface** | `.unwrap()` / `.expect()` in `crates/*/src/` außerhalb `#[cfg(test)]` | Produktionscode | **0** | **Normative Verbotsebene (P7)** |
| **K2: Exact Workspace Code** | Exaktes `.unwrap()` AST/Regex | Gesamter Workspace | **502** | Test- & Tooling-Codebase |
| **K3: Legacy Grep** | Loose string match `unwrap` | Textuelle Fundstellen | **1175** | Inkl. Comments, Fixtures, Dtos |

## Empfehlung
Als einzige maßgebliche Kennzahl für das Gate `panic-surface` / `ratchet-panic` gilt:

> **Produktionscode-Panic-Surface (K1):** Exakte Summe von `unwrap()`, `expect()`, `panic!()`, `todo!()`, `unreachable!()` in Produktionsdateien (unter `crates/*/src/`, ausgenommen Testmodule `#[cfg(test)]`).
> **Zielwert & Baseline:** **0** (Regel 2 / P7).
