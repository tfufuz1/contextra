# CLAUDE.md — contextra-db

> Crate-Kontext. Gilt zusätzlich zur Root-`CLAUDE.md` und zu `docs/SPEC.md` (maßgeblich bei Widerspruch).

## Rolle
Orchestrierung der Hybridsuche: Schreibpfad, Lesepfad, Scheduler, Wiederanlauf, Verdrahtung der Ports. SPEC §13.1 Meilenstein M2. **Hier fehlt am meisten gegenüber der Spec**: Ist 412 Zeilen, Ziel 6.000.

## Eckdaten (Messung 21.11.2026)
- Ebene: **E2** (SPEC §3.1). Ziel laut Spec: ~6.000 Zeilen.
- Umfang: **412 Zeilen `src`**, keine Testdatei, 1 `#[test]`.
- Features: `default`.
- Build: `cargo check -p contextra-db` **ok**.
- Abhängigkeiten (Ist, 15 interne): checkpoint, crypto, durable-fs, env, erasure, graph, ids, mvcc, ports, privacy, rank, store, text, types, vector.
- Genutzt von: `contextra` (Fassade), `contextra-bench`.

## Befehle
```bash
RUSTUP_TOOLCHAIN=stable cargo check -p contextra-db
RUSTUP_TOOLCHAIN=stable cargo test -p contextra-db
```

## Wichtig: Ursache des Facade-Build-Fehlers
Die Fassade `contextra` importiert aus `contextra_db` Typen, die hier **nicht existieren**: `Contextra`, `ContextraConfig`, `ContextraStats`, `Collection`, `CollectionConfig`, `SearchResult`, `ProvenanceRecord`, `EmbeddingBackend`, `TextEmbeddingEngine`, `DriftStatusProvider`, `DurabilityMode`, `QueryRewriter`, `chunker`. Dieses Crate muss diese API liefern (oder die Fassade muss angepasst werden). Entscheidung mit dem Nutzer vorher klären, siehe Root-`CLAUDE.md` §6.

## Ist-Abweichungen zur Spec
- Umfang 412 vs. Ziel 6.000. Schreib- und Lesepfad sind hier praktisch nicht implementiert.
- 0 `unwrap`/`expect`/`panic`-Treffer im Produktcode (Ist).
- Die Orchestrierung ist der einzige Ort, an dem die Ports verdrahtet werden (SPEC §3.4). Keine Port-Implementierungen in `db` selbst, nur Wiring.

## Sitzungsnotizen
<!-- Nur anhängen. Format: ### YYYY-MM-DD · Branch · Kurzfassung · Offen: … -->
