# contextra-mcp — Crate-Spezifikation

**Ebene:** E3 (Endpunkte / Anwendungen)  
**Bindung:** `docs/SPEC.md` §1, §2, §3, §6

---

## 1 Zweck & Domäne

`contextra-mcp` stellt den Model Context Protocol (MCP) Server bereit, über den LLMs und externe Agenten (z.B. Claude Desktop, agy) via standardisiertem JSON-RPC über `stdio` auf Contextra zugreifen können:
- Tools für Suche (`search`, `hybrid_search`, `graph_query`)
- Tools für Dokumentenverwaltung (`insert`, `get`, `delete`)
- Tools für Auditierung und Löschnachweise (`erase_subject`, `verify_erasure`)

## 2 Datenschutz & Löschdurchsetzung im MCP-Server

- **Absorbierte Prompt-Guard Validierung:** `contextra-mcp` validiert eingehende Parameter auf PII-Muster und Zweckbindungen vor der Weitergabe an den Kern.
- **Lesesperre:** Alle Tool-Aufrufe nutzen die regulären lesenden Methoden von `contextra`, womit das `ReadGate` zwingend greift. Ein gelöschtes Dokument liefert auch über MCP `Document not found`.
- **Lösch-Tools:** Erlauben Administratoren das Auslösen der DSGVO-Löschkette mit Attestierung (`RetentionAttestation`) und Rückgabe des signierten Löschbelegs.

## 3 Invarianten

- Strikte Trennung über stdio (keine unautorisierten offenen Netzwerk-Ports).
- Keine Klartextausgabe von Schlüsseln oder internen Audit-Hashes.
- `#![forbid(unsafe_code)]`.
