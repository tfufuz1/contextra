# AGENTS.md — `contextra-mvcc`

## 1. Rolle
`contextra-mvcc` bietet deterministische Multi-Version Concurrency Control (SequenceLog, SnapshotRegistry, TxBuffer, ReadSet, SSI-Validator) für atomare Transaktionsisolation.

## 2. Ring, Klasse, Reifegrad & `may_use`
- **Ring:** Ring 0 (Transaktionsisolation)
- **Klasse:** Kern `[K]`
- **Reifegrad:** `stable`
- **`may_use` (Obergrenze):** `contextra-types`, `contextra-ports` (Spec §6.9, `capabilities.toml`)

## 3. Garantien & Anforderungen (§33)
- **REQ-MVCC-001:** Atomare SSI-Validierung und Write-Skew Konflekterkennung (`SsiCommitGate`).
- **REQ-MVCC-002:** RAII `SnapshotLease` Guarantees zur automatischen Freigabe alter Sequenznummern.
- **Durchsetzungsmechanismus (§27.6):** Unit- & Integrationstests (`ssi_tests.rs`), Loom-Verifikation.

## 4. Verbotene Muster
- Kein `unsafe` (`#![forbid(unsafe_code)]`).
- Keine Halten von Synchronisations-Locks über `.await`-Punkte hinweg.
- Kein Umgehen der `SnapshotLease` RAII-Guards.

## 5. Implementierte & konsumierte Ports (Anhang A)
- Konsumiert: `Clock` (über `contextra-ports` / Typen).
- Stellt MVCC-Isolationsprüfungen für Storage & Index-Kerne bereit.

## 6. Tests & Gates
- **Loom Concurrency Check (REQ-MVCC-005):** Atomare Lock-Freiheits- und Dynamic Thread Concurrency Tests.
- **Mutation-Score (REQ-QA-005):** Ziel >= 85 %.

## 7. Offene Punkte
- keine.
