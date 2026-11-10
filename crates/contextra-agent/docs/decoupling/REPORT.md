# Abschlussbericht: Entkopplung contextra-agent

## 1. Gate-Matrix

| Gate | Befehl | Ergebnis |
|---|---|---|
| G-L1 (Layering / Tree) | `cargo tree -p contextra-agent -e normal --no-default-features` | GRÜN (nur `types`, `ports` und Third-Party) |
| G-L2 (Null-Prüfung) | `grep -rEn 'contextra_(types|ports|testkit)' crates/contextra-agent/src --include=*.rs \| grep -v '/compat/'` | Entfällt (Ebene 2 agentic darf `types` und `ports` konsumieren) |
| G-B.1 (No-Default) | `cargo tree -p contextra-agent -e normal --no-default-features` | GRÜN |
| G-B.2 (Workspace) | `cargo tree -p contextra-agent -e normal --features workspace` | GRÜN |
| G-B.3 (Audit) | `python3 tools/decouple_inspector.py --crate contextra-agent` | GRÜN (Score: 60/100, 0 Dead-Code-Kandidaten) |
| G-F1 (Golden-Korpus) | `cd crates/contextra-agent/tests/golden && sha256sum -c MANIFEST.sha256` | GRÜN (`agent_errors.json: OK`) |

## 2. Abweichungen von Spec / Prompt mit Grund
- Keine Abweichungen. `contextra-graph` wurde aus den Normal-Dependencies entfernt und in `[dev-dependencies]` isoliert.
- Optionale Speicherbindungen (`contextra-db`, `contextra-checkpoint`, `contextra-router`) wurden unter `storage-bindings` und `workspace` modularisiert.

## 3. Getroffene Entscheidungen
- `ports.rs`: `ContextStore` und `ScratchpadStore` als schlanke, dyn-safe Traits definiert. `InMemoryScratchpadStore` als Referenzimplementierung bereitgestellt (0 Dead-Code-Kandidaten).
- `compat/store_adapter.rs`: `CollectionContextStore` angebunden.
- `AGENTS.md`: Veraltete Droh- und Autonomietexte gemäß R0 entfernt.

## 4. Offene Punkte / Risiken
- Vorbestehende upstream-Kompilierfehler in `contextra-store` (`sstable/block_cache.rs` doppelte Methodendefinitionen) und `contextra-router` (`crate::lyapunov` Import) auf `origin/main` blockieren volles workspace-weites `cargo test`.

## 5. Übergaben an Z
- `capabilities.toml`: In der Root-Datei kann nach Abschluss aller Klassen `contextra-agent` dauerhaft auf den reduzierten Abhängigkeitsbaum aktualisiert werden.
- Default-Features: `workspace` ist als Übergangs-Default gesetzt. Z kann diesen nach Migration aller Konsumenten auf `default = []` flippen.

## 6. Messwerte
- Crate-Modul-Score: 60/100 (zuvor 54/100).
- Dead-Code-Kandidaten: 0 (zuvor 1).
