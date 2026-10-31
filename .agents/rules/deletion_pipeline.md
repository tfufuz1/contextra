# Governance Rule: Deletion Pipeline & DSGVO Compliance (`rules/deletion_pipeline.md`)

> **Normative Vorgabe nach docs/SPEC.md §12 und PLAN.md §10.**

## Gilt für
- `crates/contextra-engine`
- `crates/contextra-store`
- `crates/contextra-crypto`
- `crates/contextra-vector`
- `crates/contextra-text`
- `crates/contextra-mcp`

## Pflichtregeln
1. **Zweistufige Bereinigung (Ebene A & Ebene B)**:
   - Löschung wirkt ausschließlich durch die Kombination aus physischer Klartextbereinigung (Ebene A: HNSW-Slot-Zeroing, BM25-Posting-Rewrite, Memtable-Purge, Cache-Drop) und kryptografischer Schlüsselvernichtung (Ebene B: Vernichtung des `DocDek` im `KeyTable`).
   - Schlüsselvernichtung allein gilt niemals als ausreichende Löschung (SPEC §12.1).
2. **Reihenfolge-Invariante (INV-DEL-ORDER-1)**:
   - Ebene A MUSS zwingend vor Ebene B abgeschlossen und verifiziert sein (`PlaintextPurged` -> `PlaintextMeasured` -> `KeyDestroyed`).
3. **Strikter Scope-Typ (Kein freies String-Präfix-Matching)**:
   - Der Löschauftrag wird typisiert über `ErasureScope { tenant, collection, doc }` abgebildet.
   - Es ist strengstens verboten, Sammelpräfixe wie `t:{tenant}:` für Einzeldokument-Löschungen anzusetzen (Schutz vor Bug F01).
4. **Kein HNSW-Rewiring (VETO-F02)**:
   - HNSW-Löschung erfolgt über Tombstone, Slot-Nullung (`zeroize_and_free_node`) und periodischen Rebuild. Kein dynamisches Neuverbinden der Graphkanten.
5. **Fail-Closed Lesesperre (ErasedSet)**:
   - Ab Zustand `Frozen` blockiert der `ErasedSet` fail-closed alle Lese- und Suchzugriffe auf den Scope.
6. **Externe Prüfbarkeit (Ed25519 Proof v4/v5)**:
   - Löschnachweise dürfen extern ausschließlich asymmetrisch per Ed25519-Signatur verifiziert werden.
   - Der Bericht darf keine Klartext-IDs, personenbezogenen Daten oder Chunk-Inhalte enthalten.

## Verweise
- `docs/SPEC.md` §12
- `PLAN.md` §10
- `crates/contextra-engine/src/contextra_impl/erasure.rs`
- `crates/contextra-crypto/src/deletion_proof/`
