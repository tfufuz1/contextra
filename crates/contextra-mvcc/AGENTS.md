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

## Storage-Umbau (`contextra-storage`)

Normativ: `docs/SPEC.md` (R8). Umsetzungsplan und Verträge: `docs/IMPLEMENTATION_SPEC.md` (nicht normativ). Bis Gate G-5 ändert sich dieses Crate nicht (Spec §22); die Angaben hier sind Dokumentation, kein Auftrag zur Codeänderung.

- Rolle: bleibt obere Schicht (SSI, Transaktionspuffer, Snapshot-Register); der Stamp des Kerns ist die einzige Ordnung.
- Vertrag IF-5: nutzt `View::stamp`, `view_at(stamp)` und `Guard::{Version, Unchanged, RangeUnchangedSince}`. Die Abbildung `TxId ↔ Stamp` liegt im Adapter, nicht im Kern.
- Verboten: eigene Zeitordnung neben dem Stamp; Eingriffe in den Kern. Stand: `contextra_store` kommt nur in Kommentaren vor (2 Dateien).
- Arbeitspakete: WP-5.5 (Adapter).
