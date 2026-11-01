# Contextra – Typ-Registry & Kanonische Typen

<!-- doc-stamp: 2026-11-01 | Normativ: docs/SPEC.md §8.1, §8.6, §9.1 | PLAN.md Phase 3 -->

> **Regel 11 (Eine Typfamilie je Konzept, SPEC §0.2, §8.1):** Jeder Domänenbegriff, Identifikator und Trait existiert genau einmal an seinem kanonischen Ort im DAG. Doppelte Typdefinitionen über Crate-Grenzen hinweg sind Architektur-Schulden und verboten.

---

## 1. Kanonische Typ-Verteilung nach Ringen

| Konzept / Typ | Kanonischer Ort | Erlaubte Re-Exports | Status laut SPEC |
|---|---|---|---|
| `TenantId`, `CollectionName`, `DocKey`, `SubjectId` | `contextra-types::ids` | `contextra-ports`, `contextra-engine`, `contextra` | ✅ Kanonisch (Newtypes mit Validierung) |
| `TenantTag`, `CollTag`, `ClusterTag`, `DocTag`, `TermTag` | `contextra-types::ids` | Workspace-weit | ✅ Kanonisch (`[u8; 16]`, blind) |
| `Seq(NonZeroU64)`, `TxId(u64)`, `ErasureId([u8; 16])` | `contextra-types::ids` | `mvcc`, `store`, `erasure`, `engine` | ✅ Kanonisch (kein Sentinel) |
| `KeyBuf`, `KeyRange`, `Matcher`, `Region`, `Family` | `contextra-types::key` | `store`, `erasure`, `engine` | ✅ Kanonisch (Löschcluster-Schlüsselraum SPEC §9.1) |
| `EraseTarget`, `ErasureState`, `LayerId`, `LayerStatus`, `ErasureClass` | `contextra-types::erasure` | `erasure`, `engine`, `contextra` | ✅ Kanonisch (Löschpipeline) |
| `ErasureReport`, `RetentionAttestation`, `NeedleSet`, `LayerScan`, `Evidence` | `contextra-types::erasure` | `erasure`, `engine`, `contextra`, `testkit` | ✅ Kanonisch (Nachweis & Bericht) |
| `SecretBytes`, `SecretKey32` | `contextra-types::secret` | `crypto`, `store`, `erasure`, `engine` | ✅ Kanonisch (`Zeroizing`, kein Clone/Debug) |
| `ContextraError` | `contextra-types::error` | Workspace-weit | ✅ Kanonisch |
| `KeySet`, `KdfParams`, `KdfHeader` | `contextra-crypto::keys`, `kdf` | `store`, `erasure`, `engine` | ✅ Kanonisch (Argon2id + HKDF) |
| `StorageEngine`, `ErasureLayer`, `ReadGate`, `RecordSealer`, `LedgerSink` | `contextra-ports` | Workspace-weit | ✅ Kanonisch (Port-Traits) |
| `Vfs`, `VfsFile` | `contextra-ports` | Workspace-weit | ✅ Kanonisch (Konsolidiert) |
| `Clock`, `Rng`, `IdGen`, `Metrics` | `contextra-ports` | Workspace-weit | ✅ Kanonisch (Determinismus-Ports) |

---

## 2. Entfernte und ersetzte historische Typen

| Historischer Typ | Früherer Ort | Grund der Entfernung / Ersatz |
|---|---|---|
| `DocId(u64\|u128)` | `contextra-types` | Ersetzt durch `DocKey` (extern) und `DocTag` (intern blind, SPEC §8.1). |
| `AutoExtractionMode` | `contextra-types` | Vollständig entfernt in v1 (`D-27`). |
| `KeyManager` | `contextra-crypto` | Ersetzt durch das deterministische `KeySet` (`D-05`, SPEC §8.6). |
| `DocDek`, `GroupKek`, `KeyRegistry` | `contextra-crypto` (Ebene B) | Ebene B entfällt in v1 (`D-01`, ADR-001). `DocDek` existiert nur als lokaler Ableitungstyp im `RecordSealer`. |
| `TenantPolicy::Optional` | `contextra-engine` | Vollständig gestrichen (`D-07`); Mandant ist ausnahmslos Pflicht. |
| `IsotonicCalibrator`, `PlattScaling` | `contextra-rank` | Vollständig entfernt (`D-26`); `contextra-rank` bietet ausschließlich `rrf_fuse`. |
