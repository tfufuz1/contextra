# contextra-vector — Umsetzungsplan (v3)

> Abgeleitet aus `docs/SPEC.md` v3 §6 (Umsetzungsreihenfolge) und §3.1 (Crate-Tabelle).
> Ring: **Ring 1** (E1b) · Klasse: **A** · Ziel: Vollständig autarker, modularer Vektorindex-Baustein.

## 1. Arbeitsbereiche des Crates

| Schritt | Inhalt | Voraussetzung | Nachweis | Status |
|---|---|---|---|---|
| **S-12** | Feature-Hygiene (R10): Default-Features auf `std` bereinigen (`ports-compat` entfernen); leere Features entfernen; `docid-128` weiterleiten. | S-01R | `feature_audit` grün; `cargo check -p contextra-vector --no-default-features` | ⏳ In Bearbeitung |
| **S-36** | `vector`: Kern auf `ids`/`env`/`mvcc`/`VectorError` konsolidieren; `contract.rs` vereinen; `VectorDeleteMode`; Feature `simd` transparent; ID-Breite `HNSW_VERSION` 2 → 3; V-VETO Wachtest verifizieren. | S-30B, S-10C, S-11R | GL, GF (HNSW v2/3), GD (Distanz, Rng, ms), GC, GE | ⏳ In Bearbeitung |
| **S-80** | Auslagerung von DiskANN (`vector/diskann/`) in eigenständiges Crate `contextra-vector-diskann`. | S-36 | GL, DiskANN-Tests isoliert grün | ⏳ Geplant |

## 2. Architektur & Entkopplungs-Meilensteine

1. **Autarkie des Kerns:**
   - Der Kern (`--no-default-features`) benötigt ausschließlich Standard-Bibliothek und mathematische/kryptographische Basistypen.
   - `contextra-ports` und `contextra-types` sind vollständig in `compat/` isoliert und nur aktiv, wenn `ports-compat` bzw. `workspace` angefordert wird.
2. **Neutralisierung von Wartungs- & Bereinigungs-Schnittstellen:**
   - Bereinigung von Vektoren geschieht neutral über `Tombstone`, `Maintain`-Operationen (`compact()`, `is_rebuild_required()`).
   - DSGVO-/Purger-Relikte in `hnsw/purge.rs` und `hnsw/scanner.rs` sind als `deprecated` markiert und gehören zur Transitional-Brücke.
3. **Dokumentation & Anker-System:**
   - Einheitliche Strukturierung aller Module nach VS Code Comment-Anchors Syntax (`// SECTION`, `// !SECTION`, `ANCHOR[...]`).
   - Standardisierte Dateiköpfe mit `// FILE-CONTEXT`, `// ZWECK:`, `// INVARIANTEN:`.

## 3. Definition of Done (DoD)

- [x] Crate-lokale `SPEC.md` und `PLAN.md` entsprechen `docs/SPEC.md` v3.
- [x] Quelltext ist vollständig und transparent nach dem hierarchischen Anker-Schema kommentiert.
- [x] Keine unerlaubten `unwrap()` / `expect()` / `panic!` in Produktionspfaden.
- [x] Determinismus-Wächter meldet keine unzulässigen Laufzeit-RNGs in Graphaufbau- und Suchpfaden.
- [x] Alle modulinternen Tests laufen fehlerfrei durch.
