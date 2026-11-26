# IMPL-00 — Implementierungsspezifikationen: Index, Gap-Matrix, Reihenfolge

**Stand:** 26.11.2026 · **Basis-Commit:** `42ab648` · **Bezug:** `docs/SPEC.md` v2.0, `docs/spec/SPEC-00…09` v2.1
**Adressat:** Implementierer. Jede IMPL-Spezifikation ist ein abgeschlossenes Arbeitspaket mit Akzeptanzkriterien.

> **Lesehinweis zum IST-Zustand.** `docs/FUNKTIONSINVENTAR.md` ist gegenüber HEAD teilweise veraltet: `contextra-audit-export` und `contextra-avv-generator` sind aus Workspace und `crates/` bereits entfernt, die Löschkette selbst (`DeletionProof` in 45 Dateien) steckt aber weiterhin im Code. Vor jedem Arbeitspaket MUSS der Implementierer den Ist-Zustand per `grep` erneut verifizieren (Kommandos stehen in den Specs).

---

## 1. Befund in einem Absatz

Der IST-Code ist ein **LSM-Key-Value-Store** (`contextra-store`: WAL → Memtable → SSTable) mit darübergelegten Vektor-/Text-/Graph-Indizes, MVCC/SSI auf Sequenznummern und async `StorageEngine`-Trait (`BoxFuture`). Das SOLL ist ein **content-addressed Objektspeicher** (Packs/PAX-Extents) mit **Prolly-Tree-Truth-Layer** (`RootSet`), getrenntem `IndexManifest`, dichtem `Gid`-Adressraum, MAC-Handles, Trust/Taint, Merge-Gates und deterministisch simulierbarem, runtime-freiem Kernel (L0–L4). Der Abstand ist **architektonisch, nicht kosmetisch**: Storage-Modell, Commit-Modell und ID-Modell ändern sich. Daher Strangler-Migration mit Crate-Weichen, nicht Big-Bang.

## 2. Gemessene IST-Fakten (Grundlage der Specs)

| Fakt | Beleg (Reproduktion) | SOLL-Bezug |
|---|---|---|
| Keine Typen `Gid`, `Handle`, `CellRecord`, `RootSet`, `IndexManifestRef`, `Hlc`, `Seq`, `BeliefBeta`, `TaintFlags`, `TrustTier`, `EvidenceSufficiency`, Knapsack | `grep -rEl 'struct Gid\|RootSet\|…' crates` → 0 Treffer | SPEC §2, §4, §6, §8 |
| `DocId` ist `u64` (Feature `docid-128`: `u128`), Hash aus Key | `crates/contextra-types/src/types/domain/ids.rs` | SOLL: `Gid(u64)` dicht + `DocId(u32)` segmentlokal (C-06) |
| `StorageEngine` ist KV-Trait mit `BoxFuture`, Key `&[u8]` | `contextra-ports/src/storage.rs` | SOLL: sync Kernel, Objekt-/Cell-API |
| `contextra-store` hängt von `contextra-mvcc`, `contextra-core`, `tokio`, `memmap2` ab | `crates/contextra-store/Cargo.toml` | SOLL: store (L2) unter mvcc (L3), runtime-frei, kein mmap (C-16) |
| `tokio` in 25 Crate-Manifesten, u. a. `store`, `crypto`, `ports`, `mvcc`, `text`, `vector`, `graph` | `grep -l tokio crates/*/Cargo.toml` | SOLL: `tokio` verboten in L0–L4 |
| WAL lebt in `contextra-store/src/wal` (≈4 900 Zeilen); `WalOp` = `Put/Delete/TxEnd` (KV) | `wal/encode.rs:31` | SOLL: eigene Crate `contextra-wal`, typisierte Payloads (Anhang B) |
| `contextra-core` ist Deprecated-Fassade, 301 Dateien referenzieren `contextra_core` | `crates/contextra-core/src/lib.rs` | SOLL: kein `core` |
| Löschkette vorhanden: `DeletionProof` 45, `LayerCleanupProof` 23, `KeyRegistry/kv_shredding` 23, `AuditChain` 10, `RevocationLog` 11 Dateien | `grep -rEl` | SOLL C-15: ausgeschlossen |
| Forschungs-Heuristiken vorhanden: `contextra-adapt` (Bandits, FC-TS, Lyapunov, OffPolicy), `contextra-router`, `apprh`, `tl_hfd` in `contextra-graph` | `grep -rEl` | SOLL §1.4: entfallen |
| Kein `contextra-wal`, `-security`, `-server`, `-cli`, `-sqlite`, `-sim` | `ls crates` | SOLL §10.2 |
| `FaultVfs` existiert in `contextra-testkit` und `durable-fs` (`FaultInjectingVfs`) | Inventar | SOLL: ein `FaultVfs`, in `contextra-sim` nutzbar |
| 131 Dateien mit `#![forbid(unsafe_code)]`; unsafe-Inseln `simd`, `sys`, `wire` | Inventar | SOLL §10.3 Regel 3 |

## 3. Paketübersicht

| ID | Titel | Hauptsächlich betroffene Crates | SOLL-Quelle | Abhängig von |
|---|---|---|---|---|
