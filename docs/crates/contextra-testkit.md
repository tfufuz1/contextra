# contextra-testkit — Crate-Spezifikation

**Ebene:** E3 (Endpunkte / Testwerkzeuge)  
**Bindung:** `docs/SPEC.md` §2, §12

---

## 1 Zweck & Domäne

`contextra-testkit` enthält Testwerkzeuge, Mock-Infrastrukturen, Fehlersimulatoren und Generator-Routinen für Unit-, Integrations- und Crash-Tests:
- Fehlersimulation für Two-Phase-Commit (2PC) und I/O-Unterbrechungen
- Synthetische Dokument- und Korpus-Generatoren mit bekannter Ground-Truth
- Positivkontrollen-Kalibrierung für Scanner-Tests (Canary-Injektion)
- Loom- und Concurrency-Testfixtures

## 2 Regeln & Richtungen (R1 / R2)

- `contextra-testkit` ist eine **reine Test-Hilfsbibliothek**.
- Kein Kern-Crate (E0 bis E2) darf eine reguläre Abhängigkeit zu `testkit` führen.
- Dev-Abhängigkeiten auf `testkit` sind streng auf Test-Ziele beschränkt und dürfen nicht in Produktions-Exports gelangen.
- `#![forbid(unsafe_code)]`.
