# Fehlerabbildung: contextra-mvcc (`MvccError`)

Gemäß SPEC-v2 §5.3 (B12):
Jede in `contextra-mvcc` erzeugte Fehlervariante wird auf einen lokalen, stark typisierten Fehlertyp `MvccError` abgebildet.
Über die Kompatibilitätsbrücke `compat/error.rs` wird `From<MvccError> for ContextraError` implementiert.
Display-Text und Semantik bleiben bitgleich erhalten.

## Varianten-Mapping

| MvccError-Variante | ContextraError-Variante | Modul | Kontext / Auslöser |
|---|---|---|---|
| `Conflict(String)` | `Conflict(String)` | `ssi.rs` | SSI-Validierungskonflikt (Anti-Dependency, Overwrite, Phantoms) |
| `LimitExceeded { resource, current, limit }` | `LimitExceeded { resource, current, limit }` | `snapshot.rs`, `tx_buffer.rs` | Kapazitätsgrenzen für aktive Snapshots oder ReadSets überschritten |
| `Transaction(String)` | `Transaction(String)` | `tx_buffer.rs` | Transaktion abgebrochen, nicht existent oder ungültiger Zustand |
| `Storage(String)` | `Storage(String)` | `tx_buffer.rs` | Shard-Index außerhalb der Grenzen oder interner Speicherfehler |
| `InvalidInput(String)` | `InvalidInput(String)` | `ssi.rs` | Ungültige Sequenznummer bei `prune_through_checked` (Sentinel/Tombstone-Bit) |
| `Internal(String)` | `Internal(String)` | `ssi.rs` | Interner System- oder Zustandsfehler |
