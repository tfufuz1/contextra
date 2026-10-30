# AGENTS.md — `contextra-testkit`

> **Status 30.10.2026 (Reduktion):** Dieses Crate gehört zum Kern v1. Ring und Abhängigkeiten stimmen mit `docs/SPEC.md` §4 überein. Verweise auf entfernte Crates oder Funktionen sind veraltet. Enthält einen Graph-Index-Testdouble, der zur entfernten Graph-Crate gehört (SPEC §7). Maßgeblich ist der Quelltext. Bereinigung: `docs/PLAN.md` Phase R0 bis R3.

## 1. Rolle
`contextra-testkit` stellt Test-Mocks, In-Memory Reference Stores und Injektions-Fixtures ausschließlich für Testumgebungen bereit.

## 2. Ring, Klasse, Reifegrad & `may_use`
- **Ring:** Werkzeug / Tooling
- **Klasse:** Werkzeug (`—`)
- **Reifegrad:** `—` (Werkzeug-Crate)
- **`may_use` (Obergrenze):** Beliebig, **nur** als `dev-dependency` zugelassen (INV-EMBED-1, Spec §6.9)

## 3. Garantien & Anforderungen (§33, §6.6)
- **INV-EMBED-1:** Darf unter keinen Umständen als normale Abhängigkeit in Produktions-Features oder Release-Binaries eingebunden werden.
- **REQ-TST-001:** Deterministische `ManualClock`, `ReferenceStore` und `TenantMatrixRunner` Test-Fixtures.
- **Durchsetzungsmechanismus (§27.6):** `#![forbid(unsafe_code)]`, `check-ring-layering` Cargo-Dependency Checks.

## 4. Verbotene Muster
- Einbindung in `[dependencies]` von Nicht-Testkit Crates.
- Kein `unsafe` (`#![forbid(unsafe_code)]`).

## 5. Implementierte & konsumierte Ports (Anhang A)
- Implementiert: `ManualClock`, Mock Storage & Engine Adapters.

## 6. Tests & Gates
- **Self-Tests:** `erasure_harness_selftest.rs`.

## 7. Offene Punkte
- keine.
