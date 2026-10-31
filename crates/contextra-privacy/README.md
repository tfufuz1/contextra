# contextra-privacy

Cloud Egress Security, Data Loss Prevention (DLP), Pseudonymisierung/Surrogate-Vaulting und Bulk-Exfiltrations-Schutz für Contextra.

## Zweck & Einordnung (Ring 3)

`contextra-privacy` liegt in **Ring 3** (Domain / Application Services). Es schützt ausgehende Datenflüsse (Cloud Egress) vor unbeabsichtigtem Datenabfluss und Exfiltration, pseudonymisiert sensible Entitäten über Surrogat-Token und stellt revisionssichere Audit-Spuren bereit.

- **Eingehend:** Wird eingebunden von `contextra`, `contextra-mcp` und `contextra-router`.
- **Ausgehend:** Basiert auf `contextra-types`, `contextra-ports` und `contextra-avv-generator`.
- **Sicherheitsdoktrin:** `#![forbid(unsafe_code)]`, Zero-Panic in Produktionspfaden, striktes Fail-Closed.

---

## Komponenten (aus `src/`)

| Modul | Datei | Hauptaufgabe |
|---|---|---|
| `egress_vault` | `src/egress_vault.rs` | DLP-Klassifikation über Regex-Sets und `SurrogateVault`-Pseudonymisierung (deterministische Surrogat-Token pro Session). |
| `entity_recognizer` | `src/entity_recognizer.rs` | Regex-freier, regelbasierter Entitäten-Erkenner (`RuleBasedEntityRecognizer`) für PII-Kategorien und Wörterbücher (`Gazetteer`). |
| `egress_guard` | `src/egress_guard.rs` | Semantische k-NN Vektorähnlichkeitsprüfung gegen Datenabfluss unter Nutzung indizierter Dokumente. |
| `egress_gateway` | `src/egress_gateway.rs` | Zentrales Schutzgateway (`EgressGateway`), das alle Prüfstufen bündelt und Anfragen vor Weiterleitung in externe Cloud-Dienste säubert (`CloudResponseRehydrator`). |
| `bulk_exfiltration_detector` | `src/bulk_exfiltration_detector.rs` | Sitzungsbasiertes Rate-Limiting und Sliding-Window-Volumenzähler zur Erkennung von Massendatenabflüssen. |
| `context_edit_audit` | `src/context_edit_audit.rs` | Kryptografisch mit BLAKE3 verkettete Audit-Protokolle von Kontextmodifikationen (`ContextEditAuditRecord`). |
| `processing_registry` | `src/processing_registry.rs` | Verzeichnis von Verarbeitungstätigkeiten (VVT) nach Art. 30 DSGVO und Export nach Markdown. |
| `guarded_payload` | `src/guarded_payload.rs` | Type-State-Muster (`Unsanitized` -> `Sanitized`), das unbereinigte Egress-Weiterleitungen zur Compile-Zeit verhindert. |
| `audit_trace` | `src/audit_trace.rs` | Deterministische BLAKE3-Prüfsummenbildung für Audit-Traces ohne Klartext-PII-Leckage. |
| `avv_generator` | `src/avv_generator.rs` | Re-Export und Brücke zur Erstellung von Auftragsverarbeitungsverträgen (AVV) nach Art. 28 DSGVO. |

---

## Datenfluss Egress & Surrogate Vault

```
Eingehende Abfrage (TenantScoped<CloudQueryRequest>)
   │
   ▼
[1. Größen- & Validierungsprüfung] (Empty Query, Max Payload 64 KiB)
   │
   ▼
[2. Bulk-Exfiltrations-Detektor] (Rate Limit & Byte-Volumen pro Session)
   │
   ▼
[3. DLP-Klassifikation & Richtlinienprüfung] (EgressVault Policy Rules)
   │
   ▼
[4. Entitätserkennung & Pseudonymisierung] (RuleBasedEntityRecognizer -> SurrogateVault)
   │  Sensible Entitäten (z. B. IBAN, E-Mail) werden durch Surrogat-Token
   │  wie [USER_ENTITY_1_0123456789abcdef] ersetzt.
   ▼
[5. Layer-4 Vektorähnlichkeitsprüfung] (EgressGuard k-NN Ähnlichkeit gegen Sperrinhalte)
   │
   ▼
Bereinigte Cloud-Abfrage (Sanitized Payload) -> Externe Cloud-Inferenz
   │
   ▼
[6. Antwort-Rehydrierung] (CloudResponseRehydrator: Ersetzt Surrogat-Token zurück in Klartext)
```

---

## Erkennungsqualität & Messung (RuleBasedEntityRecognizer)

Die Erkennungsqualität wurde mithilfe eines synthetischen Korpus (`tests/fixtures/pii_corpus_synthetic.jsonl`) mit **N = 600** Datensätzen (je Kategorie 25 positive und 25 negative Testfälle) im Regressionstest `tests/recognizer_quality.rs` gemessen.

### Gemessene Werte (Span-genauer Exact Match)

| Kategorie | TP | FP | FN | Precision | Recall | F1-Score |
|---|---:|---:|---:|---:|---:|---:|
| `BIC` | 25 | 2 | 0 | 92.59% | 100.00% | 0.96 |
| `CREDIT_CARD` | 25 | 2 | 0 | 92.59% | 100.00% | 0.96 |
| `EMAIL` | 25 | 0 | 0 | 100.00% | 100.00% | 1.00 |
| `GAZETTEER` | 25 | 0 | 0 | 100.00% | 100.00% | 1.00 |
| `IBAN` | 25 | 1 | 0 | 96.15% | 100.00% | 0.98 |
| `IPV4` | 25 | 5 | 0 | 83.33% | 100.00% | 0.91 |
| `IPV6` | 24 | 2 | 1 | 92.31% | 96.00% | 0.94 |
| `LICENSE_PLATE` | 25 | 3 | 0 | 89.29% | 100.00% | 0.94 |
| `PHONE` | 25 | 4 | 0 | 86.21% | 100.00% | 0.93 |
| `PLZ` | 25 | 15 | 0 | 62.50% | 100.00% | 0.77 |
| `PLZ_ORT` | 25 | 1 | 0 | 96.15% | 100.00% | 0.98 |
| `STEUER_ID` | 25 | 1 | 0 | 96.15% | 100.00% | 0.98 |
| **GESAMT** | **299** | **36** | **1** | **89.25%** | **99.67%** | **0.94** |

---

## Grenzen des Crates

1. **Synthetische Testdaten vs. Reale Daten:** Die Erkennungsqualität wurde ausschließlich auf einem kleinen synthetischen Korpus (N = 600) gemessen. Dies stellt **keine Aussage über reale Daten**, reale Sprachvarianz, Dialekte oder komplexe Formatierungsfehler dar.
2. **Keine vollständige OCR-/Layout-Analyse:** `RuleBasedEntityRecognizer` operiert zeichen- und bytebasiert auf UTF-8-Strings. Formatierungen in Tabellen, PDFs oder Bildern werden nicht erfasst.
3. **Mehrdeutigkeit kurzer Nummern:** Postleitzahlen (PLZ) und Teilabschnitte von Telefonnummern weisen systembedingt Überlappungen mit Bestellnummern, Versionsnummern oder Geldbeträgen auf (niedrigere Precision bei isolierten 5-Ziffern-Folgen).
4. **Kein Echtzeit-Schutz gegen alle Inferenz-Modelle:** `EgressGuard` prüft semantische Vektorähnlichkeit gegen gespeicherte Schwellenwerte, garantiert jedoch keinen vollständigen Schutz gegen Prompt-Injection-Evasion oder Steganographie.

---

## Rechtliche Hinweise & Pflichtangaben

- **Erkennungsqualität:** Die Erkennungsqualität wurde nur auf einem kleinen synthetischen Korpus (N = 600, gemessene Gesamt-Precision: 89.25%, Recall: 99.67%) gemessen. Dies ist keine Aussage über die Erkennungsleistung bei realen Daten.
- **Fachprüfung offen:** Die DSGVO-Aussagen und Verweise auf Rechtsvorschriften (z. B. Art. 15, Art. 28, Art. 30 DSGVO) in diesem Crate und seinen Begleitdokumenten sind **fachlich nicht geprüft ("Fachprüfung offen")**.
- **Keine Rechtsberatung:** Die Software, die Dokumentation und die enthaltenen Vorlagen stellen keine Rechtsberatung dar und ersetzen keine juristische Prüfung im Einzelfall.
