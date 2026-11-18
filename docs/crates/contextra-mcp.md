# contextra-mcp — Crate-Spezifikation (Model Context Protocol Server)

**Ebene:** E3 (Endpunkte / Anwendungen)  
**Status:** Normativ  
**Version:** 1.0  
**Geltungsbereich:** `crates/contextra-mcp`  
**Referenz:** `docs/SPEC.md` §1, §2, §3, §6

---

## 1. Zweck & Systemrolle

`contextra-mcp` implementiert den offiziellen Model Context Protocol (MCP) Server für Contextra:
- **LLM-Integration:** Bereitstellung von standardisierten Tools und Ressourcen für KI-Assistenten (Claude Desktop, Cursor, agy-cli) über `stdio` via JSON-RPC 2.0.
- **Suchwerkzeuge:** Werkzeuge für semantische Vektorsuche, Volltextsuche und Graph-Traversierung.
- **DSGVO-Verwaltung:** Spezielle Tools zur Auslösung und Überprüfung von Löschsagas (`erase_subject`, `verify_erasure_status`).
- **Prompt-Guard & Egress-Filtering:** Absorbiert die Validierungslogik des früheren `contextra-prompt-guard` zur Erkennung von Prompt-Injection-Angriffen und PII-Leaks in Tool-Rückgaben.

---

## 2. MCP-Tools Schnittstelle

### 2.1 Retrieval Tools
1. `search_hybrid`: Führt eine hybride Suche über Vektor-, Volltext- und Graph-Indizes durch.
   - Parameter: `query: string`, `collection: string`, `top_k: int`, `as_of: optional int`.
   - **Garantie:** Alle Treffer durchlaufen zwingend das `ReadGate`. Gelöschte Dokumente sind unsichtbar.
2. `get_document`: Ruft ein bestimmtes Dokument ab.
   - Parameter: `collection: string`, `doc_id: string`.
   - Liefert bei gelöschten Dokumenten `Document not found` (Fail-Closed).

### 2.2 DSGVO-Löschtools
1. `erase_subject`: Startet die atomare Löschung eines Betroffenen.
   - Parameter: `subject_ref: string`, `attestation_code: string`, `operator_ref: string`.
   - Führt die 6-Schritte-Löschsaga aus und liefert ein signiertes `EraseReceipt` zurück.
2. `verify_erasure`: Prüft den Status einer Löschung anhand der `ErasureId`.

---

## 3. Sicherheits- & Egress-Architektur

```
LLM / MCP-Client (Claude Desktop / Cursor)
                 │  stdio (JSON-RPC 2.0)
                 ▼
┌─────────────────────────────────────────────────────────────┐
│                        contextra-mcp                        │
│                                                             │
│   ┌─────────────────────────────────────────────────────┐   │
│   │                 PromptInjectionGuard                │   │
│   │         (Input Sanitization / Pattern Scan)         │   │
│   └──────────────────────────┬──────────────────────────┘   │
│                              │                              │
│                              ▼                              │
│   ┌─────────────────────────────────────────────────────┐   │
│   │                    Tool Router                      │   │
│   └──────────────────────────┬──────────────────────────┘   │
│                              │                              │
│                              ▼                              │
│   ┌─────────────────────────────────────────────────────┐   │
│   │                 ReadGate Consultation               │   │
│   │          (Fail-Closed vor Datenauslieferung)        │   │
│   └──────────────────────────┬──────────────────────────┘   │
│                              │                              │
│                              ▼                              │
│   ┌─────────────────────────────────────────────────────┐   │
│   │                 Egress PII Filter                   │   │
│   │       (Verhindert unbeabsichtigte PII-Ausgabe)      │   │
│   └─────────────────────────────────────────────────────┘   │
└──────────────────────────────┬──────────────────────────────┘
                               │
                               ▼
                          contextra-db
```

---

## 4. Invarianten

1. **`#![forbid(unsafe_code)]`:** Vollständig sicher.
2. **Reine `stdio`-Kommunikation:** Keine unautorisierten Netzwerk-Sockets oder offenen HTTP-Ports.
3. **Keine PII in Logs:** MCP-Logs (über stderr) enthalten ausschließlich `ErasureId`, Zeitstempel und Tool-Namen, niemals Payloads oder Schlüsselbytes.
