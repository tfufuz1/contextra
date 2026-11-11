# contextra-privacy

Autarker Baustein für Cloud-Egress-Sicherheit, Data Loss Prevention (DLP), Erkennung sensibler Entitäten, Surrogat-Token-Vaulting und Schutz vor Datenexfiltration. Schaltbar vor jedes LLM- oder Cloud-Gateway.

## 1. Zweck und Rolle im Baukasten

`contextra-privacy` ist ein autarker Baustein (Ebene E0, Tier `stable`) des Contextra-Ökosystems. Das Crate hat keine internen Abhängigkeiten zu anderen Contextra-Crates und kann isoliert gebaut, getestet und exportiert werden.

### Abgrenzung zu Nachbarbausteinen

- **`contextra-privacy` vs. `contextra-vault`:**
  - `contextra-privacy`: Verantwortlich für Egress-Klassifikation, DLP-Regeln, Erkennung von Entitäten (IBAN, E-Mail, Telefon, Kreditkarten) und kurzlebige Surrogat-Ersetzung im Egress-Datenstrom.
  - `contextra-vault`: Flüchiger, kryptografischer Schlüssel- und Geheimspeicher (Secret Management, ephemerer Key-Store). Enthält keine DLP-Erkenner oder Exfiltrationsprüfungen.
- **`contextra-privacy` vs. `contextra-prompt-guard`:**
  - `contextra-privacy`: Überwacht strukturierte Daten und Datenvolumina gegen Abfluss (DLP und Bulk-Exfiltration).
  - `contextra-prompt-guard`: Analysiert Prompts auf semantische Jailbreaks, Injection-Muster und manipulative System-Anweisungen.
- **`contextra-privacy` vs. `contextra-mcp`:**
  - `contextra-privacy`: Stellt die domänenspezifischen Schutzprimitive bereit.
  - `contextra-mcp`: Protokolladapter, der MCP-Serverfunktionen bereitstellt und `contextra-privacy` als Egress-Schutzschild einbindet.

---

## 2. Tatsächliche Abhängigkeiten

Ermittelt über `cargo tree -p contextra-privacy -e normal --no-default-features`:

```text
contextra-privacy v0.1.0
├── blake3 v1.8.7
├── parking_lot v0.12.5
├── rand v0.8.8
├── regex v1.13.1
├── scc v2.4.0
├── serde v1.0.229
├── serde_json v1.0.151
├── thiserror v2.0.20
├── tokio v1.53.1
├── tracing v0.1.44
└── zeroize v1.9.0
```

Es existieren **keine internen Abhängigkeiten** im regulären Bausteinbetrieb (`allowed_internal = []`).

---

## 3. Feature-Matrix

| Feature | Standard | Beschreibung |
|---|---|---|
| `std` | Ja | Aktiviert die Standardbibliothek. |
| `serde` | Nein | Ermöglicht Serialisierung und Deserialisierung für Audit- und VVT-Datensätze. |
| `tracing` | Nein | Aktiviert strukturierte Instrumentierung und Logging ohne Klartext-Leaks. |

---

## 4. Quickstart-Minimalbeispiel

Das Beispiel ist autark und benötigt keine externen Workspace-Crates:

```rust
use contextra_privacy::{EgressVault, RuleBasedEntityRecognizer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let recognizer = RuleBasedEntityRecognizer::new();
    let vault = EgressVault::try_default()?;

    let text = "Mitarbeiter Max (max@corp.de) nutzt IBAN DE89370400440532013000.";
    let (sanitized, count) = vault.sanitize_and_vault(text, &recognizer)?;

    println!("Ersetzungen: {count}");
    println!("Sanitized: {sanitized}");
    Ok(())
}
```

Ausführung:
```bash
cargo run -p contextra-privacy --example standalone --no-default-features
```

---

## 5. Komponenten & Modulübersicht

| Modul | Datei | Hauptaufgabe |
|---|---|---|
| `contract` | `src/contract.rs` | Minimale Umwelt- und Port-Traits (`Clock`, `EgressPolicy`, `TextSearchEngine`, `EntityRecognizer`). |
| `egress_vault` | `src/egress_vault.rs` | DLP-Klassifikation über Regex-Sets und `SurrogateVault`-Pseudonymisierung. |
| `entity_recognizer` | `src/entity_recognizer.rs` | Regelbasierter Entitäten-Erkenner (`RuleBasedEntityRecognizer`) für PII-Kategorien und Wörterbücher (`Gazetteer`). |
| `egress_guard` | `src/egress_guard.rs` | Semantische k-NN Vektorähnlichkeitsprüfung gegen Datenabfluss. |
| `egress_gateway` | `src/egress_gateway.rs` | Schutzgateway (`EgressGateway`), das alle Prüfstufen bündelt (`CloudResponseRehydrator`). |
| `bulk_exfiltration_detector` | `src/bulk_exfiltration_detector.rs` | Sitzungsbasiertes Rate-Limiting zur Erkennung von Massendatenabflüssen. |
| `context_edit_audit` | `src/context_edit_audit.rs` | Kryptografisch mit BLAKE3 verkettete Audit-Protokolle von Kontextmodifikationen. |
| `processing_registry` | `src/processing_registry.rs` | Verzeichnis von Verarbeitungstätigkeiten (VVT) und Export nach Markdown. |
| `guarded_payload` | `src/guarded_payload.rs` | Type-State-Muster (`Unsanitized` -> `Sanitized`), das unbereinigte Egress-Weiterleitungen verhindert. |
| `audit_trace` | `src/audit_trace.rs` | Deterministische BLAKE3-Prüfsummenbildung für Audit-Traces ohne Klartext-PII-Leckage. |

---

## 6. Datenfluss Egress & Surrogate Vault

```text
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

## 7. Export und Isolation

Das Crate kann jederzeit als eigenständiges Projekt exportiert werden:

```bash
python3 tools/export_crate.py contextra-privacy --out /tmp/export-contextra-privacy
```

---

## 8. Hinweise & Grenzen

- **Heuristischer Erkennungskern:** `RuleBasedEntityRecognizer` operiert zeichen- und musterorientiert auf UTF-8-Zeichenketten.
- **Keine Rechtsberatung:** Diese Software, die Dokumentation und die Vorlagen stellen keine Rechts- oder Compliance-Beratung dar (INV-7).
- **Entkopplung:** Löschketten-, Saga- oder Archiv-Funktionalitäten sind neutral über Schnittstellen angebunden und gehören funktional in `contextra-erasure`.
