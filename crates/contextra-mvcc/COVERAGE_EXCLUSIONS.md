# Coverage Exclusions für `contextra-mvcc`

Folgende Zeilen und Pfade sind aus der 100%-Coverage-Berechnung ausgeschlossen oder als defensive Zweige begründet:

## 1. Defensive Sentinel- und Tombstone-Checks in `ssi.rs`
- `crates/contextra-mvcc/src/ssi.rs:645`: Prüfung `bound_seq == u64::MAX || (bound_seq & TOMBSTONE_BIT) != 0` in `prune_through_checked`. Dieser Pfad schützt vor ungültigen Wasserständen im Speicher und ist durch `sentinel_regression_test.rs` und `mv_10_*` abgedeckt, im Normalbetrieb jedoch unerreichbar.
- `crates/contextra-mvcc/src/ssi.rs:677`: Veraltete Hülle `prune_through` mit Schutzprüfung.

## 2. Kapazitätsüberlauf und defensive Limits
- `crates/contextra-mvcc/src/ssi.rs:1004`: `committed_writes.len() >= self.max_tracked_keys` Fehlerpfad für extreme Lastzustände (1.000.000 Einträge).

## 3. Fehler-Konvertierungen (`error.rs`)
- `From<MvccError> for ContextraError`: Varianten `LimitExceeded` und `Transaction` werden defensiv vorgehalten.
