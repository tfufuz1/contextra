# contextra-mvcc — Umsetzungsplan (v3)

> Abgeleitet aus docs/SPEC.md v3 §6 (Umsetzungsreihenfolge) und §6.2 (Schritt S-30B).

## 1. Statusübersicht

| Schritt | Inhalt | Voraussetzung | Nachweis | Status |
|---|---|---|---|---|
| S-30A | `tombstone.rs` aus `types` nach `mvcc` übernehmen | S-10, S-11 | Diff-Test, G-F1 | ✅ erledigt |
| S-30B | Kern auf `ids`/`env`/`MvccError`; `types`/`ports` hinter `workspace` optionalisieren; Brücke in `compat/`; Bereinigung von Re-Exports und Duplikaten | S-10, S-11 | GL, GF, GC, GE, G-L2 (0 Refs) | ✅ erledigt |
| S-30C | Vollständiges "Comment-Anchors"-Kommentierungssystem etablieren | S-30B | `pattern_diagnostic.py`, vollständige Inline-Dokumentation | ✅ erledigt |

## 2. Abnahmekriterien (Definition of Done)

- [x] **G-L1:** Kern-Closure enthält ausschließlich E0-Crates (`contextra-ids`, `contextra-env`).
- [x] **G-L2:** `tools/count_bridge_refs.py -c contextra-mvcc` meldet 0 ungegatete Referenzen auf `types`/`ports`/`testkit`.
- [x] **G-L3:** Erfolgreicher Build für `contextra-mvcc` mit `--no-default-features` und Default (`workspace`).
- [x] **G-L4:** Isolationsnachweis und Testdurchlauf in autarker Umgebung ohne zyklische Workspace-Kopplung.
- [x] **R3:** Eigener lokaler Fehlertyp `MvccError` mit bitgleicher Abbildung auf `ContextraError` in `compat/error.rs`.
- [x] **R8:** Zero-Panic-Invariante im Produktionscode (kein `unwrap()`, `expect()` oder `panic!()` in Nicht-Test-Pfaden).
- [x] **Comment-Anchors:** Vollständige semantische Kommentierung aller Komponenten mit `SECTION`, `!SECTION`, `ANCHOR`, `LINK` und `NOTE`.

## 3. Entnahmerezept für externe Projekte

Um `contextra-mvcc` als autarken Baustein in einem externen Projekt einzubinden:
```toml
[dependencies]
contextra-mvcc = { path = "./crates/contextra-mvcc", default-features = false }
contextra-ids = { path = "./crates/contextra-ids" }
contextra-env = { path = "./crates/contextra-env" }
```
