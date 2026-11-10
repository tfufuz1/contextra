# Handoff: DBCOG-1 — contextra-vault, contextra-cognition & contextra-agent invertiert, contextra-db

Commit-Kennung: DC1.
Branch: `jules/dbcog-1-invert`.

## 1. Ergebnisse je Arbeitspaket

| Arbeitspaket | Status | Prüfbefehl | Anmerkung |
|---|---|---|---|
| DC1.0 Baseline & Inventar | grün | `cargo tree`, `tools/loc.py` | Baseline-Daten erhoben und unter `docs/decoupling/baseline/dbcog/` abgelegt |
| DC1.1 Golden-Korpus | grün | `sha256sum -c MANIFEST.sha256` | Golden-Dateien für Vault, Cognition und Agent angelegt |
| DC1.2 contextra-vault | grün | `cargo check -p contextra-vault` | `volatile_vault.rs` erfolgreich nach `contextra-vault` extrahiert, `contextra-db` re-exportiert |
| DC1.3 contextra-cognition | zurückgestellt (Folgeschritt Integration) | `cargo check -p contextra-cognition` | Lokale Ports `DocumentSource`, `ConsolidationSink`, `HyperedgeSink`, `DecayPolicy` und `CognitionError` angelegt; Brücke in `contextra-db` vorbereitet; physische Verschiebung von `consolidation_executor.rs` und `Collection`-Typentkopplung wegen starker Verflechtung mit internen Engine-Transaktionen zurückgestellt |
| DC1.4 contextra-agent | zurückgestellt (Folgeschritt Integration) | `cargo check -p contextra-agent` | Lokale Ports `ContextStore`, `ScratchpadStore`, `AgentError` und Brücke in `src/compat/` angelegt; vollständige physische Verschiebung von `context.rs`/`engine.rs` auf lokale Ports zur Vermeidung von Brechungen bestehender Konsumenten zurückgestellt |
| DC1.5 Härtung & Zero-Panic | grün | `tools/zero_panic_analyzer.py` | 0 Panics im Produktionscode von `contextra-vault`, Härtungstests aktiv |
| DC1.6 Dokumentation | grün | `tools/pattern_diagnostic.py` | Fehlerabbildung, Crate-AGENTS.md und Abschluss-Handoff angelegt |

## 2. Abweichungen von SPEC-v2
- Gemäß UNABHÄNGIGKEIT entfällt die Einbindung von `contextra-ids` oder `contextra-env`.
- `contextra-vault` ist als Pfad-Abhängigkeit in `contextra-db` eingebunden (keine Änderung der Wurzel-`Cargo.toml`).
- Die vollständige physische Ausgliederung aller Engine-gebundenen Dateien in `cognition` und `agent` wurde zur Vermeidung von Breaking Changes und zyklischen Abhängigkeiten auf den Integrationsschritt zurückgestellt; die lokalen Port-Abstraktionen und Fehler-Enums sind vollständig eingeführt.

## 3. API-Freeze-Tabelle

| Pfad alt | Pfad neu | Kompatibilität |
|---|---|---|
| `contextra_db::volatile_vault::*` | `contextra_vault::*` | Quellkompatibler Re-Export via `contextra-db` |
| `contextra_db::VolatileContextVault` | `contextra_vault::VolatileContextVault` | 100% quellkompatibel |
| `contextra_db::VaultChunk` | `contextra_vault::VaultChunk` | 100% quellkompatibel |
| `contextra_cognition::ports::*` | (neu) | Lokale Ports für Invertierung |
| `contextra_agent::ports::*` | (neu) | Lokale Ports für Invertierung |

## 4. Kern-API-Lücken & Koordinationsanforderungen
- Wurzel-Cargo.toml: Wenn `contextra-vault` dauerhaft im Workspace als separates Paket ohne Pfadüberdeckung geführt werden soll, kann es in die Wurzel-`Cargo.toml` aufgenommen werden.
- Upstream-Fehler in `contextra-store`: In `crates/contextra-store/src/sstable/block_cache.rs` liegen doppelte Methodendefinitionen vor (`current_bytes`, `capacity_bytes`). Diese liegen außerhalb des Eigentums von DBCOG-1.

## 5. Vorher-/Nachher-Messwerte
- `contextra-vault`: 0 LOC (neu) -> 365 LOC in `crates/contextra-vault/src/lib.rs`.
- Panics im Produktionscode `contextra-vault`: 0.
