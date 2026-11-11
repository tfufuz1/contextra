# Fehlerabbildung: contextra-rank (`RankError` → `ContextraError`)

## Invariante
Jeder von `contextra-rank` definierte Fehlerzustand (`RankError`) wird verlustfrei und semantisch deterministisch auf `contextra_types::ContextraError` abgebildet.

| `RankError`-Variante | Ziel-Variante in `contextra_types::ContextraError` | Semantik |
|---|---|---|
| `RankError::InvalidInput(msg)` | `ContextraError::InvalidInput(msg)` | Parameter-, ID- oder Dimensionsfehler |
| `RankError::Calibration(msg)` | `ContextraError::InvalidInput(msg)` | Fehler bei PAVA-/Platt-Kalibrierungszuständen |
| `RankError::FusionFailed(msg)` | `ContextraError::InvalidInput(msg)` | Fehlerhafte Fusionsmatrix oder Degenerierung |

## Test-Nachweis
Wird über table-driven Test in `tests/hardening_tests.rs` nachgewiesen.
