# ERRORS — contextra-engine (S-64)

Dokumentation der Fehlerabbildung und Fehler-Durchreichung an der Integrationsschicht `contextra-engine`.

## 1. Fehlerdomäne der Engine

Gemäß `docs/SPEC.md` §3.1 verwendet `contextra-engine` als Ebene E2 (Integrationsschicht) die zentrale Fehlerdomäne `contextra_types::ContextraError`.

## 2. Lokale Fehlererzeugung in `contextra-engine`

Die folgenden Varianten von `ContextraError` werden in `contextra-engine` direkt erzeugt:

| Variante | Quelle / Anwendungsfall | Display-Muster |
|---|---|---|
| `ContextraError::InvalidInput(msg)` | Validierung von Dimension, Parametern, Query-Limits, Tenant-Scopes | `Ungültige Eingabe: {msg}` |
| `ContextraError::PolicyViolation(msg)` | Lizenz-Autorisierung (`FeatureRing`), TenantPolicy-Verletzung | `Richtlinienverletzung: {msg}` |
| `ContextraError::Transaction(msg)` | Transaktionskonflikte, Commit-Intent-Serialisierung, Intent-Rollback | `Transaktionsfehler: {msg}` |
| `ContextraError::Serialization(msg)` | Import/Export, JSON/Bincode-Deserialisierung | `Serialisierungsfehler: {msg}` |
| `ContextraError::LimitExceeded { limit, context }` | Extraktionslimits, Entitäts-/Tripel-Beschränkungen | `Limit überschritten: {limit} ({context})` |
| `ContextraError::ParseError(msg)` | Parsing von Modell-Antworten / Extraktionsmustern | `Parsing-Fehler: {msg}` |
| `ContextraError::Storage(msg)` | Initialisierungsprüfung, VFS-Repair-Fehler beim Booten | `Speicherfehler: {msg}` |
| `ContextraError::Sandbox(msg)` | WASM-Sandbox Validierungs- und Ausführungsfehler (Feature `sandbox`) | `Sandbox-Fehler: {msg}` |
| `ContextraError::Internal(msg)` | Unerwartete System-/Thread-Fehler | `Interner Fehler: {msg}` |

## 3. Fehler-Durchreichung aus den Backend-Subsystemen

Backend-Fehler aus den Speicher-, Index- und Port-Modulen werden über den `Result<T>`-Typ (`contextra_types::Result<T>`) transparent und unverändert an Aufrufer propagiert:

| Backend | Quell-Fehlertyp | Durchgereichte Varianten | Verhalten an Fassade |
|---|---|---|---|
| `contextra-store` | `StoreError` -> `ContextraError` | `ContextraError::Storage`, `ContextraError::Corrupt`, `ContextraError::Io` | Bitgleich durchgereicht mit unveränderter Meldung |
| `contextra-vector` | `VectorError` -> `ContextraError` | `ContextraError::Index`, `ContextraError::InvalidInput` | 1:1 durchgereicht |
| `contextra-text` | `TextIndexError` -> `ContextraError` | `ContextraError::Index`, `ContextraError::InvalidInput` | 1:1 durchgereicht |
| `contextra-graph` | `GraphError` -> `ContextraError` | `ContextraError::Index`, `ContextraError::InvalidInput` | 1:1 durchgereicht |
| `contextra-rank` | `RankError` -> `ContextraError` | `ContextraError::InvalidInput`, `ContextraError::Index` | 1:1 durchgereicht |
| `contextra-checkpoint` | `CheckpointError` -> `ContextraError` | `ContextraError::Storage`, `ContextraError::Corrupt` | 1:1 durchgereicht |
