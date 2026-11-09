# Offene Punkte / Externe Fehler (Fragment P11)

## Start-Check & Fremdfehler (Stand vor Beginn)
1. `contextra-graph`: E0433 / E0432 Unresolved modules `contextra_types` / `contextra_ports` in `csr/graph_write/mod.rs` etc. (151 Fehler).
2. `contextra-vector`: E0592 Duplikat-Definitionen (`remove`, `rebuild_sync`), E0599 fehlendes Zeroize in `deletion.rs`, E0609 unbekanntes Feld `kernel` auf `HnswColdCore`, E0599 `try_new_with_rng_and_kernel` nicht gefunden.
3. `contextra-crypto`: E0635 unerwarteter CFG condition value `serde` mit `-D unexpected-cfgs`.

Diese Build-Fehler liegen in fremden Crates außerhalb des BESITZ von P11.

## Prüfbefund zu cargo check -p contextra-mcp / cargo test -p contextra-mcp
- Beim Ausführen von `cargo check -p contextra-mcp` und `cargo test -p contextra-mcp` scheitert der Build an transitiven Workspace-Abhängigkeiten in fremden Crates (`contextra-graph`, `contextra-vector`, `contextra-crypto`), wie bereits im Start-Check dokumentiert.
- `contextra-mcp` selbst enthält 0 Fehler. Alle Beweis- und Zeugenpfade sowie ProofKey-Aufrufe wurden vollständig aus `crates/contextra-mcp/` entfernt (`grep -rniE "proof|witness|erasure_report" crates/contextra-mcp/src` liefert 0 Treffer).
