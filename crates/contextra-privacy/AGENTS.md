# AGENTS.md — `contextra-privacy`

## Modulübersicht
`contextra-privacy` ist ein Ring 3 Crate für Cloud-Egress-Sicherheit, Data Loss Prevention (DLP), Anonymisierung/Surrogate-Vaulting und Bulk-Exfiltrations-Schutz.

| Modul | Hauptaufgabe |
|---|---|
| `egress_vault.rs` | Regex-Set DLP Klassifikation & `SurrogateVault` Anonymisierung |
| `bulk_exfiltration_detector.rs` | Rate-Limiting & Sliding-Window Volumenzähler pro Session |
| `egress_guard.rs` | Layer-4 k-NN Vektor-Ähnlichkeitsprüfungen gegen Exfiltration |
| `egress_gateway.rs` | Gateway-Bündelung & Egress-Shield Abwicklung für Cloud Query Flows |

## Invarianten
1. **Zero-Panic-Doctrine**: Niemals `.unwrap()` / `.expect()` in Production Code.
2. **Fail-Closed**: Bei Timeouts, Index-Fehlern oder Laufzeitfehlern gilt strikt `Block(...)`.
3. **No Unsafe**: `#![forbid(unsafe_code)]` wird strikt durchgesetzt.

## Storage-Umbau (`contextra-storage`)

Normativ: `docs/SPEC.md` (R8). Umsetzungsplan und Verträge: `docs/IMPLEMENTATION_SPEC.md` (nicht normativ). Bis Gate G-5 ändert sich dieses Crate nicht (Spec §22); die Angaben hier sind Dokumentation, kein Auftrag zur Codeänderung.

- Rolle: Konsument von `Eraser`, `ErasureReceipt` und Ledger-Bytes; Politik (Rechtsgründe, Fristen, Mandanten) bleibt hier (NZ-14, NZ-16).
- Vertrag IF-7: `purge_deadline` ist Pflichtparameter (SD-06); der Kern trifft keine Rechtsaussage (O-2).
- Verboten: Rechtsbegriffe wie „DSGVO-konform“ ohne Messwert (REQ-DOC-2); Löschaussagen nennen Stufe ER0–ER7 und Prüfschritt.
- Arbeitspakete: nach G-6.
