# AGENTS.md — `contextra-adapt`

## 1. Rolle
`contextra-adapt` regelt Laufzeitparameter (wie Search-Budget oder PID-Latenzsteuerung) deterministisch ohne eigene Hintergrund-Threads.

## 2. Ring, Klasse, Reifegrad & `may_use`
- **Ring:** Ring 0 (Parameteradaption)
- **Klasse:** Rand `[R]`
- **Reifegrad:** `experimental`
- **`may_use` (Obergrenze):** `contextra-types`, `contextra-ports` (Spec §6.9, `capabilities.toml`)

## 3. Garantien & Anforderungen (§33)
- **REQ-ADP-001:** Anti-Windup PID Latenz-Regelung und deterministische ef-Parameteranpassung.
- **Durchsetzungsmechanismus (§27.6):** `#![forbid(unsafe_code)]`, PID Controller Integrationstests.

## 4. Verbotene Muster
- **Keine Hintergrund-Threads ohne Port.**
- Kein `unsafe` (`#![forbid(unsafe_code)]`).
- Keine unbegrenzte Integrator-Akkumulation (Anti-Windup Pflicht).

## 5. Implementierte & konsumierte Ports (Anhang A)
- Implementiert: `AdaptiveController` Port.
- Konsumiert: `Clock` Port.

## 6. Tests & Gates
- **PID Controller Tests:** Anti-Windup & Convergence Verification.

## 7. Offene Punkte
- keine.
