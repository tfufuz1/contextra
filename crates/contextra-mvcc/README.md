# contextra-mvcc

Multi-Version Concurrency Control und Isolation-Mechanismen.

Normativitätsklasse: **Kern `[K]`** (Ring 0, `stable`; §6.1, §6.9, `capabilities.toml`).

## Rolle in einem Satz
`contextra-mvcc` bietet deterministische Multi-Version Concurrency Control (SequenceLog, SnapshotRegistry, TxBuffer, ReadSet, SSI-Validator) für atomare Transaktionsisolation.

## Ring, Klasse & Reifegrad
- **Ring:** Ring 0 (Transaktionsisolation)
- **Klasse:** Kern `[K]`
- **Reifegrad:** `stable`
- **Erlaubte Abhängigkeiten (`may_use` Obergrenze):** `contextra-types`, `contextra-ports` (Spec §6.9, `capabilities.toml`)
