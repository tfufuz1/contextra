# contextra-vector — Implementierungsplan zum Soll-Zustand

**Normative Basis:** `docs/SPEC.md` §§5–7 und `docs/crates/contextra-vector.md`. **Ausgangslage:** 104 Quelldateien mit DiskANN/ACORN/RaBitQ/partial rebuild sowie 105 Nicht-Test-Panikmustern; die deklarierte E1-Closure wird durch höhere Kompatibilitäten verletzt.

1. Friere HNSW-Persistenzformat und grundlegende Insert/Search-Recall-Tests ein; entferne oder isolieren experimentelle Systeme, die nicht Sollumfang sind, hinter nicht-default Features ohne E2-Abhängigkeit.
2. Halte die Closure `ids, env, simd, durable-fs`; konsumiere die sichere SIMD-API, nie eigene Intrinsics, und entferne Ports/Types/MVCC/Sys-Kompatibilitätsbrücken.
3. Implementiere Kandidaten-Gating in Layer 0 mittels atomarer Erasure-Maske plus synchronem `ReadGate`; überfetch/retry, bis k lebende Ergebnisse in deterministischer `DocId`-Tie-Break-Reihenfolge vorliegen.
4. Implementiere `on_gate`, Purge und Witness: Node-Kanten trennen/relinken, Vektorslot zeroizen, Indexgeneration `g+1` atomisch aktivieren, alte Generation erst nach Pin-Freigabe entfernen.
5. Ersetze alle Nicht-Test-Paniken durch `VectorError`; validiere Dimensionen, Metriken, Quantisierung, Persistenz und Rebuild-Grenzen.

**Abnahme:** Gate-/Auffülltest, Delete→Search→Recovery, Konnektivitäts-/Recall-Regression, Slot-Zeroize/Witness, Scalar-SIMD-Differential und Fuzz-Corpus.
