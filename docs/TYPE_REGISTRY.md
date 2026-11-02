# Contextra – Typ-Registry & Kanonische Typen

<!-- doc-stamp: 2026-11-02 | Normativ: docs/SPEC.md §8.1, §9.1, §9.3 | SPEC 4.0.0 -->

> **Regel 11 (Eine Typfamilie je Konzept, SPEC §0.2, §8.1):** Jeder Domänenbegriff, Identifikator und Trait existiert genau einmal an seinem kanonischen Ort im DAG. Doppelte Typdefinitionen über Crate-Grenzen hinweg sind Architektur-Schulden und verboten.

---

## 1. Kanonische Typ-Verteilung nach Ringen

| Konzept / Typ | Kanonischer Ort | Erlaubte Re-Exports | Status laut SPEC |
|---|---|---|---|
| `TenantId`, `CollectionName`, `DocKey`, `SubjectId` | `contextra-types::ids` | `contextra-ports`, `contextra-engine`, `contextra` | Kanonisch (Newtypes mit Validierung) |
| `TenantTag`, `CollTag`, `ClusterTag`, `DocTag`, `TermTag`, `NameTag` | `contextra-types::ids` | Workspace-weit | Kanonisch (`[u8; 16]`, blind keyed BLAKE3) |
| `DocRef`, `DocHandle` | `contextra-types::ids` | Workspace-weit | Kanonisch (D-70: Betroffener ist Teil der Identität) |
| `Seq(NonZeroU64)`, `TxId(u64)`, `ErasureId([u8; 16])` | `contextra-types::ids` | `mvcc`, `store`, `erasure`, `engine` | Kanonisch (kein Sentinel) |
| `KeyBuf`, `KeyRange`, `Matcher`, `Region`, `Family` | `contextra-types::key` | `store`, `erasure`, `engine` | Kanonisch (Löschcluster-Schlüsselraum SPEC §9.1) |
| `EraseTarget`, `ErasureState`, `LayerId`, `LayerStatus`, `KeyCoverage` | `contextra-types::erasure` | `erasure`, `engine`, `contextra` | Kanonisch (D-69) |
| `ErasureReport`, `RetentionAttestation`, `LayerWitness`, `ResidualRisk` | `contextra-types::erasure` | `erasure`, `engine`, `contextra`, `testkit` | Kanonisch (Nachweis & Bericht) |
| `SecretBytes`, `SecretKey32` | `contextra-types::secret` | `crypto`, `store`, `erasure`, `engine` | Kanonisch (`Zeroizing`, kein Clone/Debug) |
| `ContextraError` | `contextra-types::error` | Workspace-weit | Kanonisch |
| `StorageEngine`, `TextIndex`, `VectorIndex` | `contextra-ports` | Workspace-weit | Kanonisch (statisch dispatchte Traits, D-03) |
| `ErasureLayer`, `ReadGate`, `RecordSealer`, `FileKeyring`, `LedgerSink` | `contextra-ports` | Workspace-weit | Kanonisch (Löschport-Traits) |
| `Vfs`, `VfsFile` | `contextra-ports` | Workspace-weit | Kanonisch (konsolidiertes einziges VFS-Trait, D-04) |
| `Clock`, `Rng`, `IdGen`, `Metrics` | `contextra-ports` | Workspace-weit | Kanonisch (Determinismus-Ports) |

---

## 2. Löschtypen und Schnittstellen (SPEC §8.1, §8.2)

- **`EraseTarget`:** Typisiertes Löschziel:
  - `Document(DocRef)`
  - `Subject { tenant: TenantId, subject: SubjectId }`
  - `Collection { tenant: TenantId, coll: CollectionName }`
  - `Tenant(TenantId)`
- **`KeyCoverage`:** Reichweite der Schlüsselvernichtung je Schicht (D-69):
  - `Full`: Nutzdaten und Metadaten vollständig unlesbar (z. B. Dokumentspeicher).
  - `Unlinked`: Bindungen gelöst (z. B. Textpostings unlinked).
  - `None`: Keine Schlüsselvernichtung, physischer Purge und Gate tragen den Schutz.
- **`LayerWitness`:** Typisierter Nachweis einer Prüfung, der ausschließlich aus einem `Scanner<Armed>` nach erfolgreicher Positivkontrolle erzeugt werden kann (D-72).
