# contextra-adapt

`contextra-adapt` stellt adaptive Kontroll-, Bandit-Routing- und Homeostat-Algorithmen für Contextra bereit (Ring 0).

## Zweck

Enthält mathematische Modelle zur adaptiven Auswahl von Retrieval-Strategien, Lyapunov-Drift-Überwachung, PID-Latenzregelung und Homeostat-Komponenten ohne I/O-Abhängigkeiten.

## Ring-Zugehörigkeit & Status

- **Ring:** Ring 0 (Synchroner Kern, kein `tokio`)
- **Status:** 🟢 Fertig
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`

## Öffentliche API-Übersicht

- **Bandit Routing:** `BanditPolicy`, `ShermanMorrisonBandit`, `DiagonalApproximationBandit`, `BanditError`, `BanditImplementation`
- **Lyapunov & Drift:** `LyapunovDriftWatcher`, `LyapunovResult`, `DriftReason`
- **PID Control:** `PidController`, `PidConfig`
- **Homeostat & Decay:** `HomeostatController`, `MemoryDecay`
- **Off-Policy Evaluation:** `OffPolicyEvaluator`, `OffPolicyStats`

## Architektur & Verweise

Details zur Ring-Architektur und den mathematischen Modellen finden sich in [`ARCHITECTURE.md`](../../ARCHITECTURE.md) (folgt in Kürze) sowie `README.md` §8 und §17 in der Root.
