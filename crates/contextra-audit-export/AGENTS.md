# AGENTS.md — `contextra-audit-export`

## 1. Rolle
`contextra-audit-export` exportiert Audit-Trails und DSGVO Art. 30 Verarbeitungsübersichten in standardisierten Formaten.

## 2. Ring, Klasse, Reifegrad & `may_use`
- **Ring:** Ring 4 (Erweiterung)
- **Klasse:** Erweiterung `[E]`
- **Reifegrad:** `experimental`
- **`may_use` (Obergrenze):** `contextra-types`, `contextra-crypto`, `contextra-ports` (Spec §6.9, `capabilities.toml`)

## 3. Garantien & Anforderungen (§33)
- **REQ-AUD-001:** Formatstabiler Export von Audit-Log-Einträgen.
- **Durchsetzungsmechanismus (§27.6):** `#![forbid(unsafe_code)]`, Export Integrationstests.

## 4. Verbotene Muster
- Kein `unsafe` (`#![forbid(unsafe_code)]`).
- Keine Modifikation von Audit-Einträgen während des Exports.

## 5. Implementierte & konsumierte Ports (Anhang A)
- Konsumiert: `contextra-types` und `contextra-crypto` AuditChain Primitiven.

## 6. Tests & Gates
- **Export Format Verification:** Schema-Validierung der exportierten Audit-Dokumente.

## 7. Offene Punkte
- keine.
