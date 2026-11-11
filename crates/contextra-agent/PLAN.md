# contextra-agent — Umsetzungsplan

> Abgeleitet aus docs/SPEC.md v3 §6.2 (S-63) und §3.1. Normativ gilt die Crate-Entkopplung und Modularisierung für Ebene E2.

## 1. Status und Meilensteine

| Meilenstein | Beschreibung | Status |
|---|---|---|
| M-1: Inventar & Ports | Lokale Ports `ContextStore` und `ScratchpadStore` etabliert, `contextra-graph` aus Normal-Dependencies isoliert | Abgeschlossen |
| M-2: Eigener Fehlertyp | `AgentError` mit `From`-Konvertierungen zu `ContextraError` etabliert | Abgeschlossen |
| M-3: Systematische Inline-Kommentierung | Lückenlose Kommentierung nach Comment-Anchors-Schema, `FILE-CONTEXT`-Header, `ANCHOR`, `SECTION`/`!SECTION`, `NOTE` | In Arbeit |
| M-4: S-63 Invertierung Vorbereitung | Vollständige Kapselung von `contextra-db`, `contextra-checkpoint`, `contextra-router` über Ports und optionale Features | Geplant |

## 2. Detaillierte Schritte dieses Crates

### Schritt S-63: Agent-Entkopplung & Invertierung

1. **Port-Abstraktion:**
   - Alle Workflows nutzen `ContextStore` und `ScratchpadStore`.
   - Direkte Zugriffe auf Storage (`contextra-db`, `contextra-checkpoint`) werden rein über Brückenmodule in `src/compat/` bereitgestellt.

2. **Feature-Closure Hygiene:**
   - Feature `routing` isoliert die direkte Kopplung an `contextra-router`.
   - Default-Features bleiben leer (`default = []`).

3. **Kommentierungs- und Dokumentationsstandard:**
   - Alle Quelldateien erhalten standardisierte Header und Anker zur Navigation und Funktionserklärung.
   - Bereinigung aller historischen Terminologien aus der Dokumentation.

## 3. Definition of Done

- Unveränderte Testkompatibilität über alle bestehenden 23 Testsuiten.
- `#[forbid(unsafe_code)]` strikt eingehalten.
- Keine neuen Panics oder nicht-deterministischen Pfade in Kern-Komponenten (`zero_panic_analyzer.py`, `determinism_guard.py`).
- Quellcode vollständig nach verbindlichem Schema kommentiert.
