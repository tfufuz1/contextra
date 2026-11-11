# contextra-engine — Spezifikation (v4.1 Baukasten-Architektur)

> **Geltung:** Crate-lokale Spezifikation. Normativ ist `docs/SPEC.md` (v3/v4.1 Gesamtspezifikation). Dieses Dokument definiert die Architektur- und Entkopplungsrichtlinien für `contextra-engine` als autarker Class-C-Baustein auf Ebene E2 (Integration) im Baukastensystem.
> **Faktenbasis:** Statische Analyse mit `tools/` (`decouple_inspector.py`, `zero_panic_analyzer.py`, `determinism_guard.py`).

**Pfad:** `crates/contextra-engine/` · **Umsetzungsplan:** `PLAN.md` (im selben Verzeichnis)

## 1. Zweck & Rolle im Baukastensystem

`contextra-engine` fungiert als primäre **Ausführungsschicht und Orchestrierungs-Baustein** (Ring 3, Klasse C, Ebene E2) für mehrdimensionale Datenhaltungs-, Vektor- und Graphabfragen:
- Koordination von Speichermodulen (`StorageEngine`), Vektorindizes (`VectorIndex`), Textindizes (`TextIndex`) und Graphindizes (`GraphIndex`) über das entkoppelte Trait-Bündel `Backends`.
- Transaktionsabwicklung mit Snapshot-Isolation (SSI), Intent-Protokollierung und kompensierenden Aktionen (`DbTransaction`).
- Multi-Tenant-Isolation über isolierte Namespaces und mandantenspezifische Guards (`TenantId`).
- Zero-Copy Hybrid-Retrieval und Score-Fusion unter deterministischer Sortierung (`INV-RANK-DETERMINISM-1`).

## 2. Zielzustand (docs/SPEC.md §3.1 / v4.1)

| Merkmal | Ziel | Ist-Stand (S-64) |
|---|---|---|
| Gruppe / Klasse / Ebene | integration / Klasse C (Ring 3) / Ebene E2 | Erreicht |
| Kern-Closure | `ids`, `env`, `types`, `ports`, `mvcc`, `store`, `vector`, `text`, `graph`, `rank`, `checkpoint` | Erreicht (`cargo tree`) |
| Default-Features | Keine (`default = []`) | Erreicht |
| Fehlertyp (R3) | `ContextraError` (`contextra-types`) | Erreicht (siehe `docs/decoupling/ERRORS.md`) |
| DSGVO-/Löschkette | 0 Relikte im Kern; Auslagerung in `contextra-erasure` | Erreicht (WP-14 Rückbau; INV-7 Doku-Neutralität) |
| Lizenz-Naht | Gekapselt in `src/license_seam.rs` | Erreicht |
| Zero-Panic (R8) | 0 unbegründete Panics in Neu-Code | Erreicht |
| Konsumenten-Schutz | Schutz der Schnittstellen für `db` & `cognition` | Erreicht (`tests/s64_public_surface.rs`) |

## 3. Architektur- & Entkopplungsprinzipien

### 3.1 Entkopplung durch das `Backends`-Trait
`Collection`, Transaktionen und Operatoren interagieren über das Trait-Bündel `Backends`:
```rust
pub trait Backends: Send + Sync + 'static {
    type Storage: StorageEngine;
    type Vector: VectorIndex;
    type Text: TextIndex;
    type Graph: GraphIndex;
}
```
`DefaultBackends` bindet standardmäßig die Backends `LsmStorage`, `HnswIndex`, `ContextraTextIndex` und `CsrGraphIndex` für nahtlose Abwärtskompatibilität ein.

### 3.2 Invarianten & Sperrhierarchie
1. **Lock-Order-Invariante:** `collections` (RwLock) -> `kv_locks` -> `embedder` (RwLock). Aufrufe externer Schnittstellen (Embedder, LLM, VFS) erfolgen niemals unter einem synchronen Lock Guard.
2. **TxId-Monotonie:** `TxId` wird ausschließlich über atomare Allokationsroutinen (`allocate_tx`) vergeben.
3. **Fail-Closed-Invariante:** Jeder Fehler bei I/O, Index-Sync oder Integritätsprüfung führt zum sofortigen Transaktionsabbruch und Rollback kompensierender Aktionen.
4. **Neutrale Integritätssemantik (INV-7):** Keine Erwähnung von DSGVO- oder Löschkettenbegriffen im Engine-Kern.

## 4. Schnittstellen-Inventar

- **Facade:** `Contextra`, `ContextraConfig`, `Collection`, `TenantCollectionMap`, `TenantPolicy`
- **Transaktion:** `DbTransaction`, `CommitIntent`, `CompensatingAction`
- **Abstraktion:** `Backends`, `DefaultBackends`
- **Erweiterungen:** `WasmMergeOperator` (hinter Feature `sandbox`), `AdaptiveDecayController`
- **Konsumenten-Absicherung:** `tests/s64_public_surface.rs`

## 5. Abnahme- & Qualitätskriterien

```bash
# Modularitäts- und Entkopplungsprüfung
python3 tools/decouple_inspector.py -c contextra-engine --erasure-leak-check

# Zero-Panic-Audit
python3 tools/zero_panic_analyzer.py --crate contextra-engine

# Determinismus-Prüfung
python3 tools/determinism_guard.py --crate contextra-engine
```
