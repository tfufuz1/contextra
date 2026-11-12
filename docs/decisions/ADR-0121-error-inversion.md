# ADR-0121: Inversion der Fehler-Konvertierung: E0/E1-Bausteine bleiben ContextraError-frei

**Status:** Akzeptiert  
**Datum:** 2026-11-12  
**Autor:** Principal Rust Engineer  
**Bezug:** SPEC(11).md §4.1 R3, §5.3c, §6.3, INV-12, S-51

---

## Kontext

`ContextraError` (definiert in `contextra-types/src/error.rs`) hat am HEAD `fcd387021`
505 Aufrufstellen workspace-weit. Bisher implementieren E0/E1-Bausteine in ihrer jeweiligen
`compat/error.rs`:

```rust
impl From<LocalError> for contextra_types::ContextraError { ... }
```

Dadurch müssen E0/E1-Crates `contextra-types` als optionale Abhängigkeit im Kern-Feature
`workspace` führen. Sobald deren `default`-Features `workspace` aktivieren (wie bei
`contextra-graph`, `contextra-mvcc`, `contextra-crypto`, `contextra-checkpoint`), leckt
`contextra-types` in die Kern-Closure (F-07).

SPEC(11).md §5.3c schreibt vor: „Kern-Trait-Methoden enthalten kein `ContextraError`."  
SPEC(11).md §6.3 schreibt vor: `impl From<LocalError> for ContextraError` liegt nur in `compat/`.

Frage: Soll jeder Baustein einen eigenen Fehler behalten und nur E2-Crates `ContextraError` nutzen?

## Entscheidung

**Ja.** Volle Fehlertrennung nach SPEC §5.3c:

1. **Jeder E0/E1-Baustein führt genau einen eigenen, geschlossenen Fehlertyp** mit `thiserror`
   und `#[non_exhaustive]`:
   - `contextra-graph`: `GraphError`
   - `contextra-vector`: `VectorError`
   - `contextra-mvcc`: `MvccError`
   - `contextra-store`: `StoreError`
   - `contextra-text`: `TextError`
   - `contextra-infer-candle`: `CandleError`
   - etc.

2. **`ContextraError` verbleibt ausschließlich in E2/E3** (`contextra-types`, `contextra-engine`,
   `contextra-db`, `contextra`, `contextra-mcp`, ...).

3. **Inversion der `From`-Implementierungen:**  
   Da `ContextraError` ein lokaler Typ in `contextra-types` (E2) ist, darf `contextra-types`
   per Orphan-Regel `From<Fremdfehler>` implementieren:
   ```rust
   // In crates/contextra-types/src/error.rs
   impl From<contextra_graph::GraphError> for ContextraError { ... }
   impl From<contextra_vector::VectorError> for ContextraError { ... }
   impl From<contextra_mvcc::MvccError> for ContextraError { ... }
   impl From<contextra_store::StoreError> for ContextraError { ... }
   impl From<contextra_text::TextError> for ContextraError { ... }
   ```
   Diese Konvertierungen wandern vollständig aus den Baustein-`compat/`-Verzeichnissen nach
   `contextra-types/src/error.rs` (oder compat-Modul in `types`).

4. **Display-Bitgleichheit (§6.3 Punkt 2):**  
   Jede `From`-Implementierung muss den Display-Text des konvertierten Fehlers bitgleich
   zum alten Fehler reproduzieren. Absicherung durch tabellengetriebene Tests je Variante.

5. **Keine Sammelvariante `Other(String)`** (§6.3 Punkt 3).

6. **Reihenfolge der Migration:** Zuerst Bausteine, die den größten Fan-in verursachen
   (graph mit 136 Brückenreferenzen, dann vector mit 24, dann infer-candle mit 7).

## Konsequenzen

- Positiv: G-L2 wird nach vollständiger Migration auf 0 Fundstellen gesenkt.
- Positiv: `default = ["workspace"]`-Lecks (F-07) werden durch Cargo.toml-Sanierung beseitigt.
- Positiv: E0/E1-Bausteine werden echte Bausteine im Sinne von §8.4 (entnehmbar).
- Negativ: `contextra-types/src/error.rs` erhält transitive Abhängigkeiten auf alle Bausteine.
  Mitigierung: Diese Kanten sind reguläre E2→E1-Kanten (INV-8 konform).
- INV-12: Keine neuen `unwrap`/`expect`/`panic` in den `From`-Implementierungen.

## Abgrenzung zu S-30B

S-30B behandelt `tombstone.rs` und `TOMBSTONE_BIT`. Die Fehler-Inversion ist orthogonal
und wird in S-51/S-52/S-53 umgesetzt. ADR-0120 gilt nur für `TOMBSTONE_BIT`.

## Prüfverfahren

```
# Tabellen-getriebener Display-Test
cargo nextest run -p contextra-types --filter-expr 'test(error_display)'
# G-E1
python3 tools/error_topology_mapper.py --verify
# G-L2: Kein Baustein darf ContextraError im Kern verwenden
python3 tools/gate_bridge_null.py
```
