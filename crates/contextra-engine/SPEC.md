# contextra-engine — Spezifikation (v4.1 Baukasten-Architektur)

> **Geltung:** Crate-lokale Spezifikation. Normativ ist `docs/SPEC.md` (v3/v4.1). Dieses Dokument definiert die Architektur- und Entkopplungsrichtlinien für `contextra-engine` als autarker Class-C-Baustein im Baukastensystem.
> **Faktenbasis:** Statische Analyse mit `tools/` (`decouple_inspector.py`, `zero_panic_analyzer.py`, `determinism_guard.py`).

**Pfad:** `crates/contextra-engine/` · **Umsetzungsplan:** `PLAN.md` (im selben Verzeichnis)

## 1. Zweck & Rolle im Baukastensystem

`contextra-engine` fungiert als primäre **Ausführungsschicht und Orchestrierungs-Baustein** (Ring 3, Klasse C) für mehrdimensionale Datenhaltungs-, Vektor- und Graphabfragen:
- Koordination von Speichermodulen (`StorageEngine`), Vektorindizes (`VectorIndex`), Textindizes (`TextIndex`) und Graphindizes (`GraphIndex`) über das entkoppelte Trait-Bündel `Backends`.
- Transaktionsabwicklung mit Snapshot-Isolation (SSI), Intent-Protokollierung und kompensierenden Aktionen (`DbTransaction`).
- Multi-Tenant-Isolation über isolierte Namespaces und mandantenspezifische Guards (`TenantId`).
- Zero-Copy Hybrid-Retrieval und Score-Fusion unter deterministischer Sortierung (`INV-RANK-DETERMINISM-1`).

## 2. Zielzustand (docs/SPEC.md §3.1 / v4.1)

| Merkmal | Ziel |
|---|---|
| Gruppe / Klasse | integration / Klasse C (Ring 3) |
| Backend-Bindung | Vollständig entkoppelt über `trait Backends` (Generisch oder per `DefaultBackends`) |
| Default-Features | Keine (`default = []`) |
| Fehlertyp (R3) | `ContextraError` (`contextra-types`) |
| DSGVO-/Löschkette | 0 Relikte im Kern; Auslagerung in `contextra-erasure` / `contextra-erasure-adapters` |
| Zero-Panic (R8) | 0 Panics/Unwraps in Produktionspfaden |
| Determinismus (B11) | Strikte Sortierung bei Score-Gleichstand (Tie-Break nach ID aufsteigend), deterministische Zeitabstraktion |

## 3. Architektur- & Entkopplungsprinzipien

### 3.1 Entkopplung durch das `Backends`-Trait
Um die harte Verdrahtung früherer Monolithen aufzulösen, interagieren `Collection`, Transaktionen und Operatoren primär über das Trait-Bündel `Backends`:
```rust
pub trait Backends: Send + Sync + 'static {
    type Storage: StorageEngine;
    type Vector: VectorIndex;
    type Text: TextIndex;
    type Graph: GraphIndex;
}
```
Damit können Entwickler eigene Speicherkerne (z. B. reine In-Memory-Mocks, verteilte Backends oder relationale Brücken) unkompliziert einstecken, ohne den Engine-Code zu modifizieren.

### 3.2 Invarianten & Sperrhierarchie
1. **Lock-Order-Invariante:** `collections` (RwLock) -> `kv_locks` -> `embedder` (RwLock). Aufrufe externer Schnittstellen (Embedder, LLM, VFS) erfolgen niemals unter einem synchronen Lock Guard.
2. **TxId-Monotonie:** `TxId` wird ausschließlich über atomare Allokationsroutinen (`allocate_tx`) vergeben.
3. **Fail-Closed-Invariante:** Jeder Fehler bei I/O, Index-Sync oder Integritätsprüfung führt zum sofortigen Transaktionsabbruch und Rollback kompensierender Aktionen.
4. **Neutrale Integritätssemantik:** Keine Erwähnung von DSGVO- oder Löschkettenbegriffen im Engine-Kern. Integritäts- und Persistenzmodi werden neutral über `integrity_proof_active` und Durability-Optionen parametrisiert.

## 4. Schnittstellen-Inventar

- **Facade:** `Contextra`, `ContextraConfig`, `Collection`
- **Transaktion:** `DbTransaction`, `CommitIntent`, `CompensatingAction`
- **Abstraktion:** `Backends`, `DefaultBackends`
- **Erweiterungen:** `WasmMergeOperator` (hinter Feature `sandbox`), `AdaptiveDecayController`

## 5. Abnahme- & Qualitätskriterien

```bash
# Modularitäts- und Entkopplungsprüfung (0 DSGVO/Lösch-Relikte)
python3 tools/decouple_inspector.py -c contextra-engine --erasure-leak-check

# Zero-Panic-Audit
python3 tools/zero_panic_analyzer.py

# Determinismus-Prüfung
python3 tools/determinism_guard.py
```
