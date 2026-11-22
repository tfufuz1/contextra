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

## Storage-Umbau (`contextra-storage`)

Normativ: `docs/SPEC.md` (R8). Umsetzungsplan und Verträge: `docs/IMPLEMENTATION_SPEC.md` (nicht normativ). Bis Gate G-5 ändert sich dieses Crate nicht (Spec §22); die Angaben hier sind Dokumentation, kein Auftrag zur Codeänderung.

- Rolle: Konsument von Ledger-Bytes und `ErasureReceipt`; Politik bleibt hier.
- Vertrag IF-7: liest die Ledger-Kette (unkeyed `Hasher`, D-12) und den `ledger_head` aus dem Anker; schreibt nie in Kerndateien.
- Verboten: Inhalte oder Nutzerschlüssel in Exporten (INV-KEY-3).
- Arbeitspakete: nach G-6.
