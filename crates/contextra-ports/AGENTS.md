# AGENTS.md — `contextra-ports`

## 1. Rolle
`contextra-ports` definiert sämtliche abstrakten Port-Schnittstellen und Trait-Definitionen zur sauberen Entkopplung von Kern und Außenwelt.

## 2. Ring, Klasse, Reifegrad & `may_use`
- **Ring:** Ring 0 (Port-Fundament)
- **Klasse:** Kern `[K]`
- **Reifegrad:** `stable`
- **`may_use` (Obergrenze):** `contextra-types` (Spec §6.9, `capabilities.toml`)

## 3. Garantien & Anforderungen (§33)
- **REQ-PRT-001:** Reine Trait-Schnittstellen (Anhang A.1, A.2) ohne konkrete Implementierungen.
- **REQ-PRT-002:** Injektionsfähige Zeit- (`Clock`) und Zufallsprimitiven (`EntropyRng`, `Rng`, `SeededRng`).
- **Durchsetzungsmechanismus (§27.6):** Statische Trait-Überprüfung, `#![forbid(unsafe_code)]`.

## 4. Verbotene Muster
- **Keine konkreten Port-Implementierungen** (außer Mocks in Tests).
- Kein `unsafe` (`#![forbid(unsafe_code)]`).
- Keine direkten Systemzeit- oder OS-Zufallsaufrufe (`Instant::now`, `thread_rng` verboten per `clippy.toml`).

## 5. Implementierte & konsumierte Ports (Anhang A)
- **Definiert sämtliche Ports (Anhang A.1 / A.2):** `StorageEngine`, `VectorIndex`, `TextIndex`, `GraphIndex`, `Clock`, `EntropyRng`, `DurableLog`, `ErasureParticipant`, `KeyCustodian` etc.

## 6. Tests & Gates
- **Workspace Clippy Gate:** Verhinderung verbotener Methoden (`Instant::now`, `thread_rng`).

## 7. Offene Punkte
- keine.
