# SPEC: `contextra-infer-ollama`

> **Normative Baustein-Spezifikation (Klasse A, Ring 2)**
> **Stand:** Welle 3 / SPEC v4.0

---

## 1. Zweck & Abgrenzung

`contextra-infer-ollama` ist ein autarker REST/HTTP-Adapter für Ollama-Inferenzinstanzen. Er dient der Einbindung lokaler oder entfernter LLM- und Embedding-Modelle in RAG-Pipelines.

### Nicht-Ziele
- Keine persistente Speicherung oder Indizierung von Vektoren (Aufgabe von `contextra-vector`).
- Keine Transaktionskoordination (`TxId`, MVCC).
- Keine Löschlogik oder DSGVO-Verwaltung (Aufgabe von `contextra-erasure`).

---

## 2. Architektonische Invarianten & Schnittstellen

1. **Baustein-Eigenschaften B1–B12 (Klasse A):**
   - 0 interne Abhängigkeiten im Core-Build (`default = []`).
   - Ein typisiertes Fehler-Enum: `OllamaError` (`#[non_exhaustive]`).
   - Keine Fremdfachlogik; Standardtypen und eigene Typen in der öffentlichen API.
2. **Feature-Gates:**
   - `http`: Aktiviert `reqwest`, `tokio`, `futures` sowie den asynchronen `OllamaClient` und `OllamaEmbedder`.
   - `ports-compat`: Transitionales Adapter-Feature (Ablauf in Welle 6), das `contextra-ports` und `contextra-types` bindet (`EmbeddingProvider`, `LlmTextGenerator`, `QueryRewriter`).
3. **Resilienz & Egress:**
   - Alle Requests laufen über konfigurierbare Timeouts (Connect, Read, Request).
   - Exponential Backoff mit Jitter für transiente 5xx- und Netzwerkfehler. Clientfehler 4xx werden niemals wiederholt.
   - Circuit-Breaker schützt vor kaskadierenden Ausfällen und öffnet nach Schwellwertüberschreitung.
   - Endpoint-Policy prüft Adressen vor dem Verbindungsaufbau (Standard: `LocalOnly`).

---

## 3. Fehlermodell (`OllamaError`)

- `InvalidInput(String)`: Validierungsfehler (leere Texte, überlange Eingaben).
- `ModelNotFound(String)`: Modell nicht auf dem Ollama-Server installiert.
- `Network(String)`: Verbindungsabbruch oder Verbindungsverweigerung.
- `Timeout(String)`: Zeitüberschreitung bei Verbindung oder Datenübertragung.
- `Server(String)`: HTTP 5xx Fehler des Ollama-Servers.
- `Deserialization(String)`: Ungültige JSON-Antworten oder Schema-Abweichungen.
- `EgressViolation(String)`: Verstoß gegen die Egress-Richtlinie.
- `CircuitBreakerOpen`: Unterbrechung infolge offener Schutzschaltung.
- `Internal(String)`: Unerwartete interne Zustände.

---

## 4. Benchmarking & Performance

Criterion-Benchmarks messen deterministische Operationen ohne Server-Abhängigkeit:
- RAG-Prompt-Generierung (`build_rag_prompt`)
- XML-Escaping (`xml_escape`)
- LLM-Antwort-Parsing (`parse_importance_score_response`)
EOF
