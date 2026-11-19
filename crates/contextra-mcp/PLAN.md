# contextra-mcp — Implementierungsplan zum Soll-Zustand

**Normative Basis:** `docs/SPEC.md` §§5–6 und `docs/crates/contextra-mcp.md`. **Ausgangslage:** Der Server enthält historische Prompt-Injection-, Sandbox-, Routing- und Exfiltrationslogik; Sollumfang ist ein schmaler stdio JSON-RPC-Endpoint.

1. Beschränke Transport auf stdio/JSON-RPC 2.0 und entferne HTTP/Sandbox/Router/Prompt-Guard-Altpfade sowie direkte Datenbankumgehungen.
2. Implementiere nur `search_hybrid`, `get_document`, `erase_subject`, `verify_erasure` und spezifizierte weitere Erasure-Ziele; alle Calls delegieren an die Fassade/DB.
3. Validiere Parameter, Tenant- und Operator-Kontext; serialisiere ausschließlich sichere Fehlercodes. Keine Klartexte des Löschziels, Schlüssel oder vollständige Inhalte in Log/Trace.
4. Stelle sicher, dass Gate-Resultate nie in MCP zu Daten degradiert werden: Dokument gesperrt/ausgelassen, Suche füllt delegiert auf k auf, Verify liefert nur den kanonischen Reportstatus.
5. Leere Default-Features, entferne höhere/entfernte Dependencies und alle Non-Test-Panics in Config/Explain/Sandbox.

**Abnahme:** JSON-RPC-Contract-/Malformed-Input-Tests, Gate-/Erasure-End-to-End, stdout-Protokolltest und Log-Leak-Test.
