# PENDING_API_CHANGES (Fragment P11)

Format: Symbol | betroffene Crates | gewünschte Änderung

| Symbol | Betroffene Crates | Gewünschte Änderung |
|---|---|---|
| `contextra_mcp::proof_key` / `proof_key_from_env` | `contextra-mcp` | Entfernt aus `contextra-mcp`. Proof-Key-Handling migriert in die Löschdomäne (`contextra-erasure`) per ERASE-ADAPT-1 / WP-15. |
| `contextra_forget` | MCP Clients | Tool ist nun reiner Alias für `contextra_delete` (Einzeldokument-Löschung ohne Nachweisobjekt). |
| `license_gate` / `license` | `contextra-mcp`, `contextra` | Keine Weiterleitung oder Abhängigkeit mehr auf `contextra/license` in `contextra-mcp`. |
