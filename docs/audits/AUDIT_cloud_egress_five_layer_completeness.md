# Audit-Report: Cloud-Egress Privacy Gateway Fünf-Schichten-Vollständigkeitsprüfung (§12)

**Datum:** 2026-09-17
**Auditor:** Google-Jules (Principal Rust Systems Engineer)
**Ziel-Crate:** `crates/memfuse-crypto` (Layer 2) & Orchestrierung in `crates/memfuse-mcp` (Layer 8)
**Prüfobjekt:** `crates/memfuse-crypto/src/egress_vault.rs` (448 Zeilen, Stand FINAL_11)
**Referenz-Spezifikation:** `docs/MEMFUSE_ENDPRODUKT_SPEZIFIKATION.md` §2.3 Pkt 7, §12.2.1–§12.2.5
**Issue / Claim:** `WELLE5-AUDIT-EGRESS-FIVELAYER`

---

## 1. Executive Summary

Gemäß Spezifikation §12 ist für Cloud-Egress-Anfragen ein **fünfschichtiges Privacy-Gateway** vorgeschrieben:
1. **Layer 1:** Deterministisches Token-Vaulting (Aho-Corasick + Regex + ONNX-NER + sitzungsstabile Surrogat-Erzeugung `[USER_ENTITY_<blake3...>]`).
2. **Layer 2:** Lokale Vorabstraktion (via `SegmentSynthesizer` mit `SynthesisMode::PrivacyAbstraction` und Latenz-Budget-Fail-Open).
3. **Layer 3:** Graph-Generalisierung ($k$-Anonymitäts-Generalisierung via Community-Zugehörigkeit).
4. **Layer 4:** Bulk-Exfiltrations-Erkennung via `EgressGuard` (HNSW-k-NN-Ähnlichkeitsprüfung mit Fail-Closed).
5. **Layer 5:** Inbound Re-Hydration mit bidirektionalem Zero-Trust (Rücksubstitution + Prompt-Injection-Guard).

### Hauptbefund
Die aktuelle Implementierung in `crates/memfuse-crypto/src/egress_vault.rs` und `crates/memfuse-mcp/src/egress_gateway.rs` deckt **ausschließlich eine vereinfachte Teilmenge von Layer 1** (Regex-Pattern-Matching mit harten Timeouts und Fail-Closed-Semantik) ab.
* **Layer 1:** Teilweise vorhanden (Regex-Pattern-Set, Timeout, Fail-Closed), aber Aho-Corasick-Pre-Filter, ONNX-NER und Surrogat-Tokenisierung fehlen.
* **Layer 2 bis Layer 5:** **Vollständig fehlend**.
* **Symbol `EgressGuard`:** **Fehlt komplett** in `crates/memfuse-crypto` und `crates/memfuse-mcp`.

---

## 2. Fünf-Schichten-Zuordnungstabelle

| Schicht (§12.2) | Bezeichnung | Status | Zeilenreferenz(en) | Befund / Lücke | Fail-Closed / Fail-Open Risikobewertung |
|---|---|---|---|---|---|
| **Layer 1** | Deterministisches Token-Vaulting | **Teilweise** | `egress_vault.rs:78-202` | Vorhanden: Regex-Pattern-Set (`RegexSet`), `spawn_blocking` Timeout, Payload-Größenlimit, Fail-Closed.<br>Fehlt: Aho-Corasick O(n) Pre-Filter, ONNX-NER-Entitätserkennung, Surrogat-Generierung (`[USER_ENTITY_<blake3>]`), Vault-Ersetzungstabelle. | **Sicher (Fail-Closed):** Treffer blockieren die Anfrage vollständig. Unstrukturierte PII ohne Regex-Muster wird jedoch mangels NER durchgelassen. |
| **Layer 2** | Lokale Vorabstraktion | **Fehlend** | N/A (`egress_vault.rs` / `egress_gateway.rs`) | Keine Anbindung an `SegmentSynthesizer` (`SynthesisMode::PrivacyAbstraction`) oder `PidLatencyController`. | **Risiko (Fail-Open bypassed):** Semantische Vorabstraktion findet nicht statt; Payload fließt unkorrigiert weiter. |
| **Layer 3** | Graph-Generalisierung | **Fehlend** | N/A (`egress_vault.rs` / `egress_gateway.rs`) | Keine $k$-Anonymitäts-Generalisierung von Graph-Entitäten oder Community-Zugehörigkeiten (`memfuse-graph::community`). | **Risiko:** Subgraphen und Kantenbeziehungen werden unskaliert im Egress-Payload übertragen. |
| **Layer 4** | Bulk-Exfiltrations-Erkennung (`EgressGuard`) | **Fehlend** | N/A (Symbol `EgressGuard` existiert nicht) | Kein HNSW-k-NN-Ähnlichkeits-Scan gegen gespeicherte Memory-Chunks; Schwellenwert- & Mindestlängenprüfung fehlt. | **HOCHRISIKO (Stiller Fail-Open):** Nahezu 1:1 kopierte vertrauliche Memory-Chunks werden unbemerkt an Cloud-LLMs gesendet! |
| **Layer 5** | Inbound Re-Hydration & Zero-Trust | **Fehlend** | `egress_gateway.rs:64-70` | Inbound Cloud-Antworten werden als unmodifizierter Text durchgereicht; keine Surrogat-Rücksubstitution, kein Prompt-Injection-Guard. | **Risiko:** Cloud-Antworten enthalten Platzhalter un-hydriert; schädliche Cloud-Antworten umgehen Inbound-Sicherheitsprüfungen. |

---

## 3. Detail-Analyse der einzelnen Schichten

### 3.1 Layer 1 — Deterministisches Token-Vaulting (§12.2.1)
* **Soll:**
  1. Strukturierte PII: Aho-Corasick-Automat ($O(n)$) mit anschließender Regex-Nachvalidierung.
  2. Unstrukturierte PII: ONNX-NER-Modell aus `memfuse-embed` (Reuse P10).
  3. Sitzungsstabiles Surrogat: Ersetzung von Entitäten durch `[USER_ENTITY_<blake3(entity_text ‖ session_salt)[..4]>]` und sichere Speicherung im `EgressVault` (`ZeroizeOnDrop`).
* **Ist:**
  * In `egress_vault.rs` existiert ein `EgressVault` mit standardmäßigen 5 Regex-Mustern (`sk-`, `AKIA`, `api_key`, `password`, E-Mail).
  * Die Prüfung nutzt direkt `regex::RegexSet` in einem `tokio::task::spawn_blocking`-Task mit hartem 100-ms-Timeout.
  * Bei einem Treffer wird die Anfrage mit `EgressClassification::Block(BlockReason::SensitivePattern("R-xxx"))` **vollständig abgebrochen**.
  * Es findet **keine Surrogat-Ersetzung / Tokenisierung** statt (kein Rewrite von Text in Platzhalter, sondern binäres Allow/Block).

### 3.2 Layer 2 — Lokale Vorabstraktion (§12.2.2)
* **Soll:** Integration des `SegmentSynthesizer`-Traits im Modus `SynthesisMode::PrivacyAbstraction`. Kopplung an den `PidLatencyController`: Bei Latenzüberschreitung Fail-Open (Schicht wird übersprungen, da PII bereits in Layer 1 maskiert wurde).
* **Ist:** Weder in `memfuse-crypto` noch in `memfuse-mcp::egress_gateway` ist Code zur Abstraktion oder Latenz-Budgetierung für Egress vorhanden.

### 3.3 Layer 3 — Graph-Generalisierung (§12.2.3)
* **Soll:** $k$-Anonymitäts-Generalisierung für Graph-Strukturen (Kantenlabels → Community-Zugehörigkeit via `memfuse-graph::community.rs`).
* **Ist:** Keine Implementierung vorhanden. Subgraphen-Informationen im Payload bleiben ungeneralisiert.

### 3.4 Layer 4 — Bulk-Exfiltrations-Erkennung via `EgressGuard` (§12.2.4)
* **Soll:**
  * Eigenständige Komponente `EgressGuard`.
  * Wandelt ausgehenden Payload via `memfuse-embed` in Vektoren um.
  * k-NN-Abfrage gegen den lokalen HNSW-Index (`memfuse-index`).
  * Blockiert Egress, falls `max(cosine_similarity) ≥ threshold` **und** `payload_length ≥ min_payload_bytes`.
  * **Fail-Closed bei Index-Fehlern oder Timeouts**.
* **Ist:**
  * **Symbol-Prüfung:** `grep -rn "EgressGuard" crates/` zeigt:
    * Symbol `EgressGuard` existiert **in keinem Quellcode**! (Nur in Spezifikations-Dokumenten und Kommentaren).
  * **Auswirkung:** Ein Nutzer, der lange, vertrauliche Dokumente/Memory-Chunks in die Cloud schickt, wird von Layer 1 nicht geblockt (da keine Keys/Emails enthalten sind). Mangels Layer 4 fließt der unverschlüsselte Chunk direkt zum Cloud-Anbieter.

### 3.5 Layer 5 — Inbound Re-Hydration & Bidirektionaler Zero-Trust (§12.2.5)
* **Soll:**
  * Exakte Rücksubstitution von `[USER_ENTITY_[0-9a-f]{8}]`-Tokens aus der Cloud-Antwort über die `EgressVault`-Tabelle.
  * Durchlauf der Cloud-Antwort durch denselben Prompt-Injection-Guard wie bei lokalen Suchergebnissen.
* **Ist:** In `egress_gateway.rs` (Zeilen 64–70) gibt `handle_cloud_query` bei `EgressClassification::Allow` das unveränderte `request.query` mit `abstracted: false` und `results: vec![]` zurück. Es gibt keine Inbound-Verarbeitung.

---

## 4. Analysis of Existing Tests (§4.6 / FINAL_9)

In `crates/memfuse-crypto/src/egress_vault.rs` sind 12 Modultests im `mod tests` definiert:
1. `test_allow_happy_path`: Prüft `Allow` bei unverdächtigem Text.
2. `test_payload_with_abstract_and_sensitive_pattern_is_blocked`: Prüft Blockieren bei `sk-` Key.
3. `test_block_sensitive_pattern`: Prüft Blockieren bei E-Mail-Adresse.
4. `test_opaque_rule_ids_do_not_leak_raw_regex`: Verifiziert opake Regel-IDs ("R-001").
5. `test_invalid_pattern_returns_error`: Prüft Fehlerbehandlung bei ungültigen Regex.
6. `test_oversized_payload_blocked_before_task_spawn`: Prüft Längenlimit (>64 KiB).
7. `test_redos_and_timeout_fail_closed`: Verifiziert Fail-Closed bei extrem kurzem Timeout (1 ns).
8. `test_trait_implementation_e2e`: Testet `EgressClassifier`-Trait-Anbindung.
9. `test_egress_guard_fail_closed_on_index_unavailable`: Testet Fail-Closed bei Timeout (Namensträger-Test, testet aber `classify_layer1` Timeout, nicht den HNSW `EgressGuard`).
10. `test_egress_vault_default_implementation`: Testet Standard-Vault-Muster.
11. `test_egress_vault_accessors_and_with_timeout`: Testet Builder/Accessor.
12. `test_exact_payload_boundary_allowed`: Testet exakte Payload-Grenze (65.536 Bytes).

**Fazit der Testanalyse:** Die Tests decken **ausschließlich Layer 1** (Regex-Pattern-Matching, Timeout, Payload-Limits) ab. Für Layer 2–5 existieren **keine Tests**.

---

## 5. Code-Qualität & Zero-Panic Audit

* **`#![cfg_attr(not(test), forbid(unsafe_code))]`:** Compliant in `crates/memfuse-crypto/src/lib.rs`.
* **Zero-Panic-Doctrine in Production Code (`egress_vault.rs`):**
  * `unwrap()` / `expect()` kommen in Produktionscode in `EgressVault::default()` vor:
    * Zeile 219: `Self::try_default().unwrap_or_else(...)` — Abgefangen via Fallback, paniziert nicht.
    * Zeile 221: `regex::RegexSet::new(Vec::<&str>::new()).unwrap_or_else(...)` — Abgefangen via `RegexSet::empty()`, paniziert nicht.
  * **Befund:** Produktionscode in `egress_vault.rs` ist **Zero-Panic-konform**.
* **Laufzeitverhalten:** All blocking regex evaluations happen inside `tokio::task::spawn_blocking` bounded by `tokio::time::timeout`, preventing CPU starvation on the async reactor.

---

## 6. Sicherheits- & Fail-Closed-Risikobewertung

| Szenario | Aktuelles Verhalten | Spezifiziertes Verhalten (§12) | Sicherheits-Auswirkung |
|---|---|---|---|
| **Egress mit vertraulichem Memory-Chunk (ohne API-Key/E-Mail)** | **ALLOW** (Durchgelassen) | **BLOCK** (Layer 4 `EgressGuard` HNSW-Match) | **KRITISCH:** Exfiltration interner Gedächtnis-Inhalte in die Cloud. |
| **Egress mit unstrukturierter Personen-PII ("Max Mustermann besucht Berlin")** | **ALLOW** (Durchgelassen) | **TOKENISIERT** (Layer 1 ONNX-NER Surrogat-Ersetzung) | **HOCH:** DSGVO-PII-Exfiltration an externe LLM-Anbieter. |
| **Egress mit strukturiertem Secret ("sk-12345...")** | **BLOCK** (Fail-Closed) | **TOKENISIERT / BLOCK** | **SICHER:** Wird durch Layer 1 Regex zuverlässig geblockt. |
| **Timeout bei Klassifikation** | **BLOCK** (Fail-Closed) | **BLOCK** (Fail-Closed) | **SICHER:** Garantiert Fail-Closed-Verhalten. |
| **Index / HNSW Ausfall** | **N/A** (Nicht angebunden) | **BLOCK** (Fail-Closed) | **KRITISCH:** Mangels Anbindung wird bei fehlendem/beschädigtem Index nicht geblockt. |

---

## 7. Umsetzungsreife Handlungsempfehlungen (Folge-Prompts)

Da in diesem Audit-Prompt gemäß **ROLE-LOCK REGEL** kein Produktionscode verändert werden darf, werden die notwendigen Erweiterungen hier als konkrete, architekturell isolierte Folge-Aufgaben spezifiziert:

### Empfehlung 1: Implementierung `EgressGuard` (Layer 4)
* **Ziel-Datei:** `crates/memfuse-crypto/src/egress_guard.rs` (oder `crates/memfuse-index/src/egress_guard.rs`).
* **Inhalt:**
  ```rust
  pub struct EgressGuard {
      index: Arc<dyn VectorIndex>,
      embedder: Arc<dyn EmbeddingClient>,
      threshold: f32, // z. B. 0.85
      min_bytes: usize, // z. B. 128
  }
  ```
* **Verhalten:** Erstellt Vektor-Embedding des Outbound-Payloads, führt k-NN Search ($k=1$) aus, blockiert mit `BlockReason::SensitivePattern("Bulk exfiltration detected")`, falls `sim >= threshold` && `len >= min_bytes`. Bei Fehler/Timeout strictly **Fail-Closed**.

### Empfehlung 2: Ergänzung Layer 1 Surrogat-Tokenisierung & NER
* **Ziel-Datei:** `crates/memfuse-crypto/src/egress_vault.rs`
* **Inhalt:**
  * Erweiterung von `EgressVault` um eine In-Memory-Surrogat-Map (`HashMap<String, String>`) mit `session_salt`.
  * Methode `sanitize_and_vault(&self, payload: &str) -> Result<(String, VaultMap), EgressVaultError>`.
  * Anbindung von `memfuse-embed` ONNX-NER für unstrukturierte PII.

### Empfehlung 3: Pipeline-Orchestrierung aller 5 Layer in `memfuse-mcp` / `memfuse-router`
* **Ziel-Datei:** `crates/memfuse-mcp/src/egress_gateway.rs`
* **Inhalt:**
  * Verkettung aller 5 Layer im `handle_cloud_query`-Pfad:
    $$\text{Payload} \xrightarrow{\text{L1}} \text{Vaulted} \xrightarrow{\text{L2}} \text{Abstracted} \xrightarrow{\text{L3}} \text{Generalized} \xrightarrow{\text{L4}} \text{Guarded} \rightarrow \text{Cloud Egress}$$
  * Inbound-Pfad:
    $$\text{Cloud Response} \xrightarrow{\text{Prompt Injection Guard}} \xrightarrow{\text{L5 Re-Hydration}} \text{Client}$$
  * Verwendung des bereitstehenden Typ-State-Protektors `GuardedPayload<Sanitized>` aus `memfuse-router::guarded_payload`.

---

## 8. Fazit & Audit-Verdict

* **STATUS:** **FAIL / INCOMPLETE** (gegenüber Fünf-Schichten-Spezifikation §12)
* **GRUND:** Von 5 spezifizierten Schutzschichten ist nur Layer 1 teilweise (als reiner Regex-Blocker ohne Tokenisierung) vorhanden. Die Komponenten Layer 2, Layer 3, Layer 4 (`EgressGuard`) und Layer 5 fehlen im Quellcode.
* **POSITIV:** Das vorhandene Layer-1-Modul `egress_vault.rs` ist qualitativ hochwertig, Zero-Panic-konform, performant via `spawn_blocking` kapselnd und garantiert striktes **Fail-Closed** bei Timeouts.
