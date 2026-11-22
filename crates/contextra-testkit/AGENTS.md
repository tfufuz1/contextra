# AGENTS.md — `contextra-testkit`

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

## Storage-Umbau (`contextra-storage`)

Normativ: `docs/SPEC.md` (R8). Umsetzungsplan und Verträge: `docs/IMPLEMENTATION_SPEC.md` (nicht normativ). Bis Gate G-5 ändert sich dieses Crate nicht (Spec §22); die Angaben hier sind Dokumentation, kein Auftrag zur Codeänderung.

- Rolle: Test-Primitive und Test-`KeyProvider` für den Kern, nur als `dev-dependency`.
- Ist: `ManualClock`, In-Memory-Stores, `FaultVfs` (eigene API, kein Port-`Fs`); **kein** Test-`KeyProvider`, keine Konformitätssuite für `Fs`/`Cipher`/`Mac`/`Kdf`/`KeyProvider`.
- Vertrag IF-4: Test-`KeyProvider`, Test-`Cipher`/`Mac`/`Kdf` (übernimmt `testing/refimpl.rs` aus dem Kern, D-05), `ManualClock` und seedbare `Entropy` als Port-Implementierungen.
- Verboten: Abhängigkeit auf `contextra-store`; Nutzung außerhalb von `dev-dependencies` (INV-EMBED-1).
- Arbeitspakete: WP-2.5.
