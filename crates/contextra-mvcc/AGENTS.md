# AGENTS.md — `contextra-mvcc`

## 1. Rolle & Zweck (Baukasten-Prinzip)
`contextra-mvcc` stellt ein autarkes, deterministisches Multi-Version Concurrency Control (MVCC) Subsystem (Ring 0) bereit.
Es kapselt `SequenceLog`, `SnapshotRegistry`, `TxBuffer`, `ReadSet` und `SsiCommitGate` (Serializable Snapshot Isolation) für atomare Transaktionsisolation ohne globale Locks.
Als wiederverwendbarer Lego-Baustein kann es unabhängig in jede Storage-Engine oder jedes Transaktionssystem integriert werden.

## 2. Ring, Klasse, Reifegrad & `may_use`
- **Ring:** Ring 0 (Transaktionsisolation & Nebenläufigkeit)
- **Klasse:** Kern `[K]`
- **Reifegrad:** `stable`
- **`may_use` (Obergrenze):** `contextra-types`, `contextra-ports` (Spec §6.9, `capabilities.toml`).

## 3. Garantien & Architektur-Invarianten
- **REQ-MVCC-001:** Atomare SSI-Validierung und Write-Skew-Konflikterkennung (`SsiCommitGate`).
- **REQ-MVCC-002:** RAII `SnapshotLease` Guarantees zur deterministischen Freigabe alter Sequenznummern und Verhinderung von Pin-Leaks.
- **Clock-Injektion:** Alle Zeitstempel basieren auf der injizierten `Clock`-Port-Abstraktion (Monotonic Nanoseconds); kein direkter Aufruf von `Instant::now()` im Produktionscode.
- **Zero-Panic-Doktrin:** Vollständige Fehlerfortpflanzung; keine unkontrollierten Panics oder Unwraps.

## 4. Verbotene Muster
- Kein `unsafe` (`#![forbid(unsafe_code)]` strikt erzwungen).
- **Kein Halten von Synchronisations-Locks über `.await`-Punkte** (Verhinderung von Deadlocks und Executor-Starvation).
- Kein Umgehen der `SnapshotLease` RAII-Guards.
- Kein Shared Mutable State ohne wohldefinierte Nebenläufigkeitsgrenzen.

## 5. Schnittstellen & Datenfluss (Ports & Abstraktionen)
- **Konsumiert:** `Clock`-Port aus `contextra-ports`.
- **Bereitgestellt für:** `contextra-store`, `contextra-vector`, `contextra-engine` für Snapshot-Isolation und Versionsabgleiche.

## 6. Tests, Sabotage & Qualitätssicherung
- **Loom Concurrency Check (REQ-MVCC-005):** Exhaustive Validierung aller atomaren Operationen, Memory-Orderings und Lock-Freiheit unter nebenläufiger Last.
- **SSI Write-Skew Tests:** Verifikation der Konflikterkennung bei konkurrierenden Transaktionen (`ssi_tests.rs`).
- **Mutations-Testing (REQ-QA-005):** Ziel-Mutation-Score >= 85 %.

## 7. Offene Punkte & Technische Schulden
- Keine.
