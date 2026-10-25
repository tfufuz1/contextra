# contextra-testkit

Test Fixtures, Mock Engines und Chaos Matrix Harness.

Normativitätsklasse: **Werkzeug** (Tooling, `—`; §6.6, §6.9, `capabilities.toml`).

## Rolle in einem Satz
`contextra-testkit` stellt Test-Mocks, In-Memory Reference Stores und Injektions-Fixtures ausschließlich für Testumgebungen bereit.

## Ring, Klasse & Reifegrad
- **Ring:** Werkzeug / Tooling
- **Klasse:** Werkzeug (`—`)
- **Reifegrad:** `—` (Werkzeug-Crate)
- **Erlaubte Abhängigkeiten (`may_use` Obergrenze):** Beliebig, nur als `dev-dependency` zugelassen (INV-EMBED-1, Spec §6.9)
