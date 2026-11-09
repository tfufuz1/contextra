# contextra-privacy

Autarker Baustein (**Klasse A**, SPEC v4.0 Welle 1) für Cloud-Egress-Sicherheit, Data Loss Prevention (DLP), PII-Erkennung, Surrogat-Token-Vaulting und Schutz vor Datenexfiltration. Schaltbar vor jedes LLM- oder Cloud-Gateway.

## Baukasten-Eigenschaften (Klasse A)

- **Vollkommen autark (B1, B2):** Keine internen Normal- oder Dev-Dependencies zu anderen Contextra-Crates.
- **Eigene & Standardtypen (B3):** Alle Schnittstellen basieren auf Standardtypen oder crate-eigenen Typen (`Clock`, `TenantId`, `TenantScoped`, `EgressPolicy`, `EgressDecision`, `EntityMatch`).
- **Typisierte Fehler (B4):** Hauptfehlertyp `EgressError` / `Error` (`thiserror`, `#[non_exhaustive]`) mit Standard-Varianten (`InvalidInput`, `CapacityExceeded`, `Io`, `StateConflict`, `Corrupt`).
- **Autarker Export (B5):** Vollständig lauffähig außerhalb des Workspace-Repos via `tools/export_crate.py` und `tools/isolation_proof.py`.
- **Feature-Minimalismus (B6):** Standard-Features sind minimal (`default = ["std"]`, `serde`, `tracing`, `inspect`, `experimental`).
- **Keine Fremdfachlogik (B7):** Reine Maskierung und Egress-Sicherheit; keine Löschketten-, Saga- oder Shredding-Logik (gehört exklusiv in `contextra-erasure`).
- **Zero-Panic & Sicherheit (B9, B10):** `#![forbid(unsafe_code)]`, striktes Fail-Closed, null Panics auf beliebigen UTF-8-Grenzen.
- **P28-Determinismus (B11):** Zeitmessung über den `Clock`-Port; interne HashMaps nutzen deterministische BLAKE3-Hasher (`DeterministicBuildHasher`).
- **Lizenz (B12):** Lizenziert unter MIT und Apache 2.0 (`LICENSE-MIT`, `LICENSE-APACHE`), MSRV 1.89.

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
| `processing_registry` | `src/processing_registry.rs` | Verzeichnis von Verarbeitungstätigkeiten (VVT) und Export nach Markdown. |
| `guarded_payload` | `src/guarded_payload.rs` | Type-State-Muster (`Unsanitized` -> `Sanitized`), das unbereinigte Egress-Weiterleitungen zur Compile-Zeit verhindert. |
| `audit_trace` | `src/audit_trace.rs` | Deterministische BLAKE3-Prüfsummenbildung für Audit-Traces ohne Klartext-PII-Leckage. |

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

## Minimalbeispiel (Eigenständig ohne Contextra-Workspace)

```rust
use contextra_privacy::{EgressVault, RuleBasedEntityRecognizer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let vault = EgressVault::try_default()?;
    let recognizer = RuleBasedEntityRecognizer::new();

    let outbound_text = "Bitte überweisen an IBAN DE89370400440532013000 für max@example.com";
    let (sanitized, count) = vault.sanitize_and_vault(outbound_text, &recognizer)?;

    println!("Maskierter Text: {sanitized}");
    println!("Ersetzte Entitäten: {count}");
    Ok(())
}
```

## Export in ein eigenständiges Projekt

```bash
python3 tools/export_crate.py contextra-privacy --out /pfad/zu/deinem/projekt/libs/contextra-privacy
```

---

## Grenzen des Crates

1. **Synthetische Testdaten vs. Reale Daten:** Die Erkennungsqualität wurde ausschließlich auf einem synthetischen Korpus (N = 600) gemessen. Dies stellt **keine Aussage über reale Daten**, reale Sprachvarianz, Dialekte oder komplexe Formatierungsfehler dar.
2. **Keine vollständige OCR-/Layout-Analyse:** `RuleBasedEntityRecognizer` operiert zeichen- und bytebasiert auf UTF-8-Strings. Formatierungen in Tabellen, PDFs oder Bildern werden nicht erfasst.
3. **Mehrdeutigkeit kurzer Nummern:** Postleitzahlen (PLZ) und Teilabschnitte von Telefonnummern weisen systembedingt Überlappungen mit Bestellnummern, Versionsnummern oder Geldbeträgen auf.
4. **Kein Echtzeit-Schutz gegen alle Inferenz-Modelle:** `EgressGuard` prüft semantische Vektorähnlichkeit gegen gespeicherte Schwellenwerte, schützt jedoch nicht lückenlos gegen Prompt-Injection-Evasion oder Steganographie.

---

## Rechtliche Hinweise & Pflichtangaben

- **Erkennungsqualität:** Heuristischer Erkennungskern; die Erkennungsleistung bei realen Daten kann variieren.
- **Fachprüfung offen:** Verweise auf Datenschutzvorschriften in diesem Crate und seinen Begleitdokumenten sind **fachlich nicht geprüft ("Fachprüfung offen")**.
- **Keine Rechtsberatung:** Die Software, die Dokumentation und die enthaltenen Vorlagen stellen keine Rechtsberatung dar und ersetzen keine juristische Prüfung im Einzelfall.
