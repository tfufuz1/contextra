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
