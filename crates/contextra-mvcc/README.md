# contextra-mvcc

> **Status 30.10.2026 (Reduktion):** Dieses Crate gehört zum Kern v1. Ring und Abhängigkeiten stimmen mit `docs/SPEC.md` §4 überein. Verweise auf entfernte Crates oder Funktionen sind veraltet. Maßgeblich ist der Quelltext. Bereinigung: `docs/PLAN.md` Phase R0 bis R3.

Multi-Version Concurrency Control und Isolation-Mechanismen.

Normativitätsklasse: **Kern `[K]`** (Ring 0, `stable`; §6.1, §6.9, `capabilities.toml`).

## Rolle in einem Satz
`contextra-mvcc` bietet deterministische Multi-Version Concurrency Control (SequenceLog, SnapshotRegistry, TxBuffer, ReadSet, SSI-Validator) für atomare Transaktionsisolation.

## Ring, Klasse & Reifegrad
- **Ring:** Ring 0 (Transaktionsisolation)
- **Klasse:** Kern `[K]`
- **Reifegrad:** `stable`
- **Erlaubte Abhängigkeiten (`may_use` Obergrenze):** `contextra-types`, `contextra-ports` (Spec §6.9, `capabilities.toml`)
