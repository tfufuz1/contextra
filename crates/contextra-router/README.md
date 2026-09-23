# contextra-router

`contextra-router` stellt das SLM-Profil-Routing und den Dispatcher bereit (Ring 3).

## Zweck

Führt Anfragen an kleine Sprachmodelle (SLMs) und Dispatcher-Pfade aus, nutzt re-exportierte Bandit-Komponenten zur Strategie-Auswahl.

## Ring-Zugehörigkeit & Status

- **Ring:** Ring 3 (Anwendungskern / Router)
- **Status:** 🟢 Fertig
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`

## Öffentliche API-Übersicht

- **Router Engine:** `RouterEngine`, `DefaultRouterEngine`, `RoutingDecision`
- **SLM Profiles & Dispatch:** `SlmProfile`, `dispatch_to_slm`
- **Routing Outcomes:** `RoutingOutcome`, `DecisionId`
- **Adapt Re-exports:** Re-exports von `contextra_adapt::{bandit, lyapunov, offpolicy}`

## Architektur & Verweise

Details zum SLM-Profil-Routing finden sich in [`ARCHITECTURE.md`](../../ARCHITECTURE.md) (folgt in Kürze) und `README.md` §8.
