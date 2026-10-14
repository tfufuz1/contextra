# ADR-E19: mcp-dispatcher-spawn-blocking

* **Status:** Akzeptiert (aus Spezifikation §13 E19)
* **Datum:** 2026-10-14

## Entscheidung
MCP-Dispatcher nutzt `spawn_blocking` mit begrenztem Pool; Löschpfad nach Commit nicht abbrechbar.

## Begründung
Synchroner Kern; Cancellation-Sicherheit.

## Verworfen
`block_in_place` im async-Kontext
