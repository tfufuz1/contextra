# AGENTS.md — contextra-privacy
> Ring 3 · stable · Quelle: capabilities.toml · Spec: III.16, K.20, L.5, Teil 10

1. Zweck
`contextra-privacy` bietet Cloud Egress Security, Data Loss Prevention (DLP), Anonymisierung über Surrogate-Vaulting und Schutz vor Bulk-Exfiltration in Ring 3. Die Crate führt Regex-Klassifikation, k-NN-Ähnlichkeitsprüfungen sowie Inbound-Rehydration durch und generiert DSGVO-konforme Verarbeitungsverzeichnisse und AVV-Entwürfe. Sämtliche Egress-Entscheidungen werden kryptografisch auditiert und unter Fail-closed-Garantien abgewickelt.

2. Modul-Karte
| Pfad | Verantwortung |
|---|---|
| `src/lib.rs` | Public API & Re-Exports für Egress-Security und DLP |
| `src/audit_trace.rs` | Erzeugung deterministischer BLAKE3 Hash-Traces (`EgressClassifierTrace`, `INV-EGRESS-AUDIT-1`) |
| `src/avv_generator.rs` | Generierung von Auftragsverarbeitungsverträgen (`AvvContext`, `generate_avv_draft`) |
| `src/bulk_exfiltration_detector.rs` | Volume- und Zeitfenster-basiertes Rate-Limiting pro `SessionId` (Layer 4) |
| `src/context_edit_audit.rs` | Auditierung und Nachverfolgung von Kontext-Bearbeitungen |
| `src/egress_gateway.rs` | Gateway-Bündelung, Handling von Cloud-Queries und Inbound-Rehydration (`CloudResponseRehydrator`) |
| `src/egress_guard.rs` | Layer-4 Bulk-Exfiltrations-Prüfung mittels k-NN-Ähnlichkeitssuche |
| `src/egress_vault.rs` | Layer-1 DLP-Regex-Klassifikation (`classify_layer1`) & `SurrogateVault` Anonymisierung |
| `src/error.rs` | Crate-lokale Fehlerdefinitionen (`EgressError`) |
| `src/guarded_payload.rs` | Type-State Kapselung (`GuardedPayload<S>`) gegen ungesanitisierten Egress |
| `src/processing_registry.rs` | Maschinell erzeugtes Verzeichnis von Verarbeitungstätigkeiten (Art. 30 DSGVO) |

3. Invarianten
- **INV-EGRESS-AUDIT-1**: Jede Egress-Entscheidung protokolliert einen deterministischen 32-Byte BLAKE3-Hash-Trace via `compute_audit_trace` (`cargo test -p contextra-privacy`).
- **APM-EGRESS-BYPASS**: Jede externe Anfrage MUSS `EgressClassifier::classify()` durchlaufen (`cargo test -p contextra-privacy`).
- **Fail-Closed-Doktrin**: Bei Timeouts, Index-Ausfällen oder internen Fehlern liefert Layer-1 DLP strikt `Block(...)` (`cargo test -p contextra-privacy`).

4. Verboten / Anti-Patterns
- **Keine Klartext-Logs**: Personenbezogene Daten oder unbehandelte Egress-Nutzlasten dürfen niemals in `tracing`-Logs geschrieben werden.
- **Unsanitierter Egress**: `GuardedPayload<Unsanitized>` darf nicht direkt an externe Endpunkte übermittelt werden; Umwandlung in `GuardedPayload<Sanitized>` ist erforderlich.
- **Panic in Egress-Pfaden**: Verwende niemals `.unwrap()` oder `.expect()` in Produktionspfaden; Fehler müssen sauber in `Block` bzw. `EgressError` konvertiert werden.

5. Nebenläufigkeit, Async- und Lock-Regeln
- Hot Paths nutzen `parking_lot::Mutex` oder lockfreie Datenstrukturen (`scc`).
- Synchronisations-Sperren dürfen niemals über `.await`-Grenzen hinweg gehalten werden (P26).
- Asynchroner I/O erfolgt strikt über Tokio-Tasks; Timeouts in `classify_layer1` sind auf `DEFAULT_TIMEOUT` (100ms) begrenzt.

6. Verifikation
- `cargo test -p contextra-privacy`

7. Bekannte Lücken / SOLL
- **Standardmuster-Ablösung**: `EgressVault::default_patterns()` verwendet statische Teilstring-Muster (`sk-`, `AKIA`, `api_key`, `password`, E-Mail-Regex), die keine allgemeinen PII-Formen (z. B. Namen, Adressen) ohne konfigurierte `EntityRecognizer` erfassen.
- **Kollisionsrisiko bei Surrogate-Keys**: `SurrogateVault` generiert 16-Bit (4 Hex-Zeichen) Hash-Tokens (`[USER_ENTITY_xxxx]`). Bei sehr vielen Entitäten pro Sitzung können Kollisionen auftreten, bei denen neuere Mappings frühere überschreiben.
