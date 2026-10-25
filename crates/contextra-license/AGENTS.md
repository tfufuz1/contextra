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
