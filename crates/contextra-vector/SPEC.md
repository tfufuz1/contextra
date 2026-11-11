# contextra-vector — Spezifikation (v3)

> **Geltung:** Crate-lokale Spezifikation. Normativ bleibt `docs/SPEC.md` (v3, Stand 10.11.2026). Dieses Dokument überträgt die Anforderungen für dieses Crate, ergänzt sie um verifizierte Ist-Fakten und markiert Abweichungen.
> **Faktenbasis:** Statische Analyse mit `tools/` am geklonten Repo-Stand (`main`, HEAD `5408826f8` / v3).
> **Beleg-Marken:** *[V]* am Quelltext/Manifest verifiziert, *[T]* aus einem tools/-Bericht, *[S]* aus docs/SPEC.md übernommen, *[E]* Festlegung dieser Spezifikation.

**Pfad:** `crates/contextra-vector/` · **Umsetzungsplan:** `PLAN.md` (im selben Verzeichnis)

## 1. Zweck

Vektorindex (HNSW mit SIMD-Beschleunigung, SQ8-Quantisierung, ACORN-Filterung). DiskANN und RaBitQ wandern in Experimente bzw. separates Crate (`contextra-vector-diskann`). *[S]*

## 2. Zielzustand (docs/SPEC.md §3.1) [S]

| Merkmal | Ziel |
|---|---|
| Gruppe | retrieval |
| Ebene (Kern-Closure) | **E1b** (Kern-Closure ⊆ E0 ∪ E1a, höchstens 4 Mitglieder) |
| Kern-Closure | `ids`, `env`, `mvcc` (`sys` nur für `mmap`, `simd` nur für `simd`) |
| Default-Features | `std` (kein `ports-compat` im Default) |
| Fehlertyp (R3) | `VectorError` (`thiserror`, `#[non_exhaustive]`) |
| Brücke (hinter Feature `workspace`, §1.1) | `VectorIndex`-Impls, Wartungs-Purger/Scanner, `From`-Konvertierungen |

Ebenen (SPEC §1.1): E0 = Kern-Closure leer · E1a = Kern-Closure ⊆ E0 · E1b = Kern-Closure ⊆ E0 ∪ E1a, mindestens ein E1a-Mitglied (`mvcc`). Der Kern ist der Code mit `--no-default-features` ohne Feature `workspace`.

## 3. Ist-Zustand (tools/)

| Messgröße | Wert |
|---|---|
| Lines of Code (Produktion, `loc.py`) | ~13.800 Zeilen in 56 Dateien |
| Version / publish | workspace-vererbt / <unset> |
| Interne Normal-Abhängigkeiten (Ist, Cargo.toml) | types (optional), ports (optional), mvcc (optional), sys (optional), simd (optional) |
| Interne Dev-Abhängigkeiten (Ist) | keine |
| Features (Ist, Anzahl) | 16 |
| Default-Features (Ist) | `std`, `ports-compat` |
| zero-panic: Produktionsbefunde | 0 echte Panics im HNSW-Kern (strenges `#![forbid(unsafe_code)]` und Zero-Panic-Lints) |
| unsafe: Vorkommen / ohne SAFETY / Insel | 0 / 0 / False (`forbid(unsafe_code)`) |
| Error-Enums (Ist) | `VectorError`, `Error` (in contract.rs), `AcornError` |

## 4. Ist-vs-Ziel-Abgleich [V]

- **Ist-Kanten, die das Ziel im Kern nicht vorsieht:** `types` (hinter `workspace`/`ports-compat`), `ports` (hinter `workspace`/`ports-compat`)
- **Ziel-Kanten, die im Ist fehlen:** `ids`, `env` als direkte E0-Abhängigkeiten
- **Default-Features weichen ab:** Ist `std, ports-compat` → Ziel `std` (R10: Feature-Regel verbietet `*-compat` im Default).
- **V-VETO-Schutz:** `PartialRebuildConfig` bleibt deaktiviert; V-VETO-F02-Guard sichert die Konnektivitäts- und Slot-Nullungs-Invariante.

## 5. Modul-Architektur & Entkopplungsrichtlinie

```
crates/contextra-vector/src/
├── lib.rs                   <- Facade & Crate-Root (#![forbid(unsafe_code)])
├── types.rs                 <- Crate-lokale Typen (DocId, DistanceMetric, ScoredDocument, Clock/Rng)
├── error.rs                 <- VectorError (thiserror, Ring 1 Error Domain)
├── contract.rs              <- Autarke Service-Verträge (Index, Maintain, DistanceFn, Transform)
├── distance.rs              <- Distanzfunktionen (Kosinus, Euklidisch, Dot) mit SIMD-Fallback
├── compute_pool.rs          <- Parallele Distanzberechnung ohne Tokio-Runtime-Kopplung
├── fixed_hasher.rs          <- Determinismus-Absicherung (SipHasher13/FixedState gegen DoS/Hash-Drift)
├── casting.rs               <- Checked Casts (usize <-> u32) ohne Panic
├── candidate_stream.rs      <- Zero-Allocation Streaming von Such-Kandidaten
├── partial_rebuild.rs       <- Lokale Reparatur- und Maintenance-Routinen (hinter Feature-Gate)
├── quantize/                <- Skalare Quantisierung (SQ8) mit SIMD-Beschleunigung
├── quantize_rabitq.rs       <- Experimentelle RaBitQ 1-Bit/Quantisierung
├── acorn/                   <- ACORN: Prädikat-augmentierte Vektorsuche
├── persistence/             <- Mmap- und dateibasierte HNSW-Index-Serialisierung
├── hnsw/                    <- HNSW Core (Graph, Hierarchie, Insertion, Search, Deletion)
├── diskann/                 <- Out-of-Core Graphindex (Kandidat für S-80 contextra-vector-diskann)
└── compat/                  <- Workspace- & Ports-Brücke (nur aktiv mit Feature "ports-compat"/"workspace")
```

## 6. Persistenz-, Fehler- und Verhaltensvertrag

### 6.1 Formatversionen (Golden-Korpus)
- **HNSW:** `persistence/header.rs`: Magic `0x484E5357`, Version `HNSW_VERSION = 2` (Übergang zu Version 3 für ID-Breitenprüfung `id_width_bits`).
- **DiskANN:** `diskann/format.rs`: `DANN` v1, `DANF`, `PWAL` v1, `TWAL` v1.

### 6.2 Fehlerabbildung
- Lokaler Fehlertyp: `VectorError`.
- Fehler-Brücke: `From<VectorError> for contextra_types::ContextraError` in `compat/error.rs`. Keine Panics im Produktionspfad.

## 7. Invarianten (verbindlich)
1. **INV-VEC-ZEROIZE-1 & INV-VEC-CONNECTIVITY-1:** Löschen = Tombstone + Slot-Nullung + Konnektivitätserhalt.
2. **INV-TENANT-IDX-1:** Strikte Tenant-Isolation je Indexinstanz.
3. **INV-INDEX-APPLY-1:** Deterministische Level-Zuweisung via `hash(doc_id, seed)`.
4. **INV-INDEX-SNAPSHOT-1:** MVCC-Sichtbarkeit über Sequenznummern (`born_seq`/`died_seq`).
5. **INV-ZERO-PANIC:** Absolute Panic-Freiheit in allen Produktionspfaden.
