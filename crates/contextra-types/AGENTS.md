# AGENTS.md — `contextra-types`

> **Ring 0** · Klasse `[K]` · Reifegrad `stable` · Unsafe: nein · Rolle: kanonische IDs, Fehlerbaum, `RequestContext` und Basis-Datenmodelle, ohne jede Contextra-Abhängigkeit.
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`. Diese Datei nennt nur Crate-Spezifisches. Stand 2026-11-07, statisch gelesen, nicht kompiliert.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-types --no-default-features --locked` |
| Contextra-Abhängigkeiten (Ist = Soll) | keine |
| Externe Abhängigkeiten | `thiserror`, `serde`, `serde_json`, `bincode`, `ahash`, `blake3`, `sha2`, `bytes` (dev: `proptest`) |
| Features | `docid-128` (breite `DocId`), `auto-extraction-opt-out`; `default = []` |
| Lints im Crate | `#![forbid(unsafe_code)]`, `#![deny(dead_code)]`, `#![warn(missing_docs)]` |
| Konsumenten | alle Crates (transitiv über `ports`) |

## 2. Abhängigkeiten

- **Erlaubt:** nur externe Crates. **Verboten:** jedes `contextra-*` (Ring-0-Wurzel).
- **Keine I/O, keine Async-Runtime, keine Uhr, kein Zufall.** Das Crate ist deterministisch (INV-RING0-DETERMINISM-1).

## 3. Modul-Karte (am Code verifiziert)

| Datei | Verantwortung |
|---|---|
| `error.rs`, `error_dto.rs` | `ContextraError` (≈51 Varianten, einziger Fehlerbaum, A6) und `ContextraErrorDto` für FFI/MCP |
| `ids.rs`, `types/domain/ids.rs` | `DocId` (`u64` oder per `docid-128` `u128`), `TxId` (mit `INTERNAL_BASE`), `TenantId`, `CollectionId` |
| `types/domain/{document,misc}.rs` | Dokument- und Hilfsmodelle |
| `types/{saos,filter,budget,importance}.rs` | Abfrage-, Filter-, Budget- und Gewichtstypen |
| `tenant_scope.rs`, `refs.rs`, `schema.rs`, `model_fingerprint.rs`, `retrieval_strategy.rs` | Mandantenscope, Referenzen, Schema, Embedding-Identität (INV-EMBED-2), Strategie |
| `tombstone.rs`, `types/domain/misc.rs` | `SeqBitTombstone`, `TombstoneSemanticsCheck`; die Konstante `TOMBSTONE_BIT` (`1 << 63`) steht in `types/domain/misc.rs` |
| `witness.rs` | `RequestContext`, `ScanMethod`, `SagaRecord` |
| `erasure.rs` | **Löschdomäne (516 Zeilen):** `EraseRequest`, `EraseReceipt`, `ErasureStatus`, `RetentionAttestation`, `LayerId`, `EraseTarget` |

## 4. Verträge & Invarianten

1. **Ein Fehlerbaum (A6).** Neue Varianten nur am **Ende** von `ContextraError` anfügen (Binärkompatibilität der DTOs). `From`-Impls stehen ausschließlich in `error.rs`. Keine crate-lokalen Fehler-Enums, die nicht in `ContextraError` münden.
2. **Tombstone-Disziplin.** Bit 63 einer Sequenznummer ist `TOMBSTONE_BIT`. Vor jedem `max_seq`-Vergleich maskieren: `seq & !TOMBSTONE_BIT`. Unmaskierte Vergleiche erzeugen Phantom-Sichtbarkeit.
3. **`TxId`-Bereiche.** Reguläre Sequenz vs. interner Systembereich (`INTERNAL_BASE`). Niemals aus `SystemTime` ableiten (P28).
4. **Tenant (VETO-F10).** `RequestContext` trägt den Mandanten. Typen dürfen keinen Default-Mandanten liefern.
5. **`DocId`-Breite.** `docid-128` ist ein **Workspace-weites** Feature (`vector`, `text`, `graph`, `store`, `py` spiegeln es). Ändere die Breite nie in einem Crate allein.
6. **Persistierte Typen** (`bincode`/`serde`) sind Formatvertrag (INV-FORMAT-1): Feldreihenfolge nicht ändern ohne Formatversion.
7. **DbC durch Typen.** Newtypes mit validierenden Konstruktoren (`try_new`), keine öffentlichen Felder dort, wo eine Invariante gilt.

## 5. Ports

Keine. `types` definiert Daten, keine Austauschrollen. Traits gehören nach `contextra-ports`.

## 6. Nebenläufigkeit & Performance

Keine Locks, kein `async`. Alle Typen sind `Send + Sync`, wo möglich. Große Strukturen (`Document`, Filter) sollen `Clone` günstig halten (`Arc`/`Bytes`). Neue Hot-Path-Typen: Größe mit `size_of` im Test festschreiben.

## 7. Anti-Patterns

- **Neue Typfamilie anlegen, ohne `docs/TYPE_REGISTRY.md` und `grep -rn "struct <Name>" crates/` zu prüfen.** `EraseTarget` und `LayerId` existieren bereits doppelt (`types` und `ports`, Root-§10 K3).
- **Erasure-Domänentypen weiter ausbauen.** `erasure.rs` gehört fachlich nach `contextra-erasure`. Nur reine Port-Typen dürfen im Fundament bleiben. Keine neuen Felder, keine neuen Varianten ohne ADR.
- **Variante in der Mitte einfügen** oder umbenennen (bricht DTO/Binärformat).
- **`unwrap()` in `From`-/`TryFrom`-Impls.**

## 8. Tests & Messung

```bash
cargo test -p contextra-types --locked
cargo test -p contextra-types --locked --features docid-128
cargo clippy -p contextra-types --all-targets -- -D warnings
```

- Property-Tests (`proptest`) für `DocId`-Roundtrip, DTO-Konvertierung (verlustfrei) und Tombstone-Maskierung.
- Zielwert Mutation-Score ≥ 85 % (`cargo mutants -p contextra-types`).
- Beide Feature-Varianten (`docid-128` an/aus) gehören in die CI-Matrix.

## 9. Offene Arbeit

| WP | Inhalt |
|---|---|
| 2.2 / 3.9 | Doppelte Löschtypen (`EraseTarget`, `LayerId`) mit `ports` vereinen; Löschdomäne aus `types/erasure.rs` nach `contextra-erasure` verlagern, Port-Typen in `ports` belassen |
| 4.5 | `Language` aus `contextra-text` hierher verlagern (PLAN) |
| 5.5 | Typen, die heute über `contextra-core` re-exportiert werden, direkt hier beziehen (DEBT-03) |
