# AGENTS.md — `contextra-license`

## 1. Rolle
`contextra-license` prüft kryptografische Lizenzschlüssel zur Freischaltung optionaler Enterprise- und Feature-Ringe.

## 2. Ring, Klasse, Reifegrad & `may_use`
- **Ring:** Ring 4 (Erweiterung)
- **Klasse:** Erweiterung `[E]`
- **Reifegrad:** `experimental`
- **`may_use` (Obergrenze):** `contextra-types`, `contextra-crypto`, `contextra-ports` (Spec §6.9, `capabilities.toml`)

## 3. Garantien & Anforderungen (§33)
- **REQ-LIC-001:** Kryptografische Signaturprüfung von Lizenzschlüsseln ohne Netzwerkverbindung.
- **Durchsetzungsmechanismus (§27.6):** `#![forbid(unsafe_code)]`, Lic-Gate Tests.

## 4. Verbotene Muster
- Kein `unsafe` (`#![forbid(unsafe_code)]`).
- Keine Netzwerkaufrufe / Phone-Home Logik.

## 5. Implementierte & konsumierte Ports (Anhang A)
- Implementiert: `LicenseGate` Port.
- Konsumiert: `Clock` Port.

## 6. Tests & Gates
- **License Gate Verification:** Signatur- & Ablaufdatumstests.

## 7. Offene Punkte
- keine.

## Storage-Umbau (`contextra-storage`)

Normativ: `docs/SPEC.md` (R8). Umsetzungsplan und Verträge: `docs/IMPLEMENTATION_SPEC.md` (nicht normativ). Bis Gate G-5 ändert sich dieses Crate nicht (Spec §22); die Angaben hier sind Dokumentation, kein Auftrag zur Codeänderung.

- Rolle: Konsument von `Eraser`/`ErasureReceipt`; Politik bleibt hier.
- Vertrag IF-7: wie `contextra-privacy`.
- Verboten: Politik im Kern.
- Arbeitspakete: nach G-6.
