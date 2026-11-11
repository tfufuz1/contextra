# contextra-ports — Spezifikation (SPEC v4.1)

> **Geltung:** Crate-lokale Spezifikation. Normativ ist `docs/SPEC.md` (v4.1). Dieses Dokument überträgt die Anforderungen für dieses Crate, ergänzt sie um verifizierte Ist-Fakten und markiert die Architektur des Baukastensystems.
> **Faktenbasis:** Statische Analyse und Tooling-Verifikation (`tools/loc.py`, `tools/decouple_inspector.py`, `tools/zero_panic_analyzer.py`, `tools/determinism_guard.py`, `tools/unsafe_boundary_auditor.py`).
> **Beleg-Marken:** *[V]* am Quelltext/Manifest verifiziert, *[T]* aus einem tools/-Bericht, *[S]* aus docs/SPEC.md übernommen, *[E]* Festlegung dieser Spezifikation.

**Pfad:** `crates/contextra-ports/` · **Umsetzungsplan:** `PLAN.md` (im selben Verzeichnis)

## 1. Zweck & Baukasten-Architektur

Port-Traits (Storage, Vector-, Text-, Graph-Index, Vfs, Embedding, Reranker, Lifecycle, Observability u. a.) als zentrale Vertragsschicht des Baukastensystems. *[S]*
`contextra-ports` bildet zusammen mit den Kernspeicher-Crates das **Class B Storage Bundle** (Ring 0 / Verträge). Es ermöglicht modulare, lose gekoppelte Subsysteme, bei denen Implementierungen (`contextra-store`, `contextra-vector`, `contextra-text`, `contextra-graph`, `contextra-infer-*` etc.) austauschbar sind, ohne zyklische Abhängigkeiten zu erzeugen.

## 2. Zielzustand (docs/SPEC.md §3.1 & §11) [S]

| Merkmal | Ziel |
|---|---|
| Baukasten-Klasse | Class B (Storage Bundle / Canonical Ports) |
| Ring | Ring 0 |
| Kern-Closure | `contextra-types`, `contextra-env` |
| Default-Features | leer (`default = []`) |
| Unsafe-Code | `#![forbid(unsafe_code)]` strikt durchgesetzt |
| Dyn-Safety | Alle primären Austauschports sind vollständig dyn-kompatibel (`BoxFuture`, `BoxStream`, keine generischen Methoden ohne `where Self: Sized`) |

## 3. Ist-Zustand (verifiziert via tools/)

| Messgröße | Wert |
|---|---|
| Lines of Code (Produktion, `loc.py`) | 3900 Zeilen in 42 Dateien |
| Version / publish | workspace-vererbt / nicht für crates.io |
| Interne Normal-Abhängigkeiten (Cargo.toml) | `contextra-types`, `contextra-env` |
| Features | `conformance`, `contract-tests`, `testkit` (Default: leer) |
| zero-panic: Verstöße in Produktion (`tools/zero_panic_analyzer.py`) | 0 |
| Unsafe-Code (`tools/unsafe_boundary_auditor.py`) | 0 (Verboten via `#![forbid(unsafe_code)]`) |
| Determinismus-Befunde (`tools/determinism_guard.py`) | 0 unzulässige Befunde in Produktion |
| Schnittstellenkatalog (`tools/rust_ast_inspector.py`) | 43 Traits, 47 Structs, 20 Enums |

## 4. Modul-Katalog & Verantwortlichkeiten

| Subsystem | Modul-Pfade | Enthaltene Port-Traits & Schlüsseltypen |
|---|---|---|
| **Storage & Key-Value** | `src/storage.rs`, `src/kv.rs`, `src/kv_bridge_port.rs`, `src/key_material.rs` | `StorageEngine`, `KvPrefixStore`, `KvBridgeStorage`, `KeyMaterialSource` |
| **Index-Subsysteme** | `src/vector_index.rs`, `src/text_index.rs`, `src/graph_index.rs`, `src/graph.rs`, `src/index.rs` | `VectorIndex`, `TextIndex`, `GraphIndex`, `GraphCollectionMutation`, `HybridSearchProvider`, `CommunityResolver` |
| **Virtual Filesystem** | `src/vfs.rs`, `src/fs.rs` | `Vfs`, `VfsFile`, `VfsOpenOptions`, `VerifiedVia` (Synchron per B08) |
| **Environment Injection** | `src/clock.rs`, `src/rng.rs`, `src/id_gen.rs`, `src/metrics.rs`, `src/env.rs` | `Clock`, `Rng`, `IdGen`, `MetricsSink` (Re-Exports aus `contextra-env` für Rückwärtskompatibilität) |
| **Inference & RAG** | `src/embedding.rs`, `src/reranker.rs`, `src/query_rewriter.rs`, `src/attention.rs`, `src/inference.rs` | `TextEmbeddingEngine`, `EmbeddingProvider`, `TextGenerator`, `Reranker`, `QueryRewriter`, `AttentionExporter` |
| **Lifecycle & Observability** | `src/lifecycle.rs`, `src/observability.rs`, `src/plugin.rs` | `GroundingValidator`, `ResponseGroundingValidator`, `ContextPreparer`, `DriftStatusProvider`, `PluginManifest` |
| **Governance & Policy** | `src/license.rs`, `src/policy.rs` | `LicenseGate`, `EgressPolicy`, `EgressDecision` |
| **Scanner & Maintenance** | `src/scanner/` (`layer`, `session`, `witness`, `mod`) | `Purger`, `LayerScanner`, `ScanSession`, `LayerWitness`, `ScanTarget` (Physische Reclaim- & Prüf-Schnittstellen) |
| **Conformance & Testing** | `src/conformance.rs`, `src/contract_tests.rs` | Contract-Test-Harnesses zur Validierung von Drittanbieter- und Crate-Implementierungen |

## 5. Invarianten & Qualitätsregeln

1. **INV-PORT-DYN-1 (Dyn-Kompatibilität):** Alle zentralen Port-Traits müssen als `Box<dyn PortTrait>` oder `Arc<dyn PortTrait>` instanziierbar und thread-sicher (`Send + Sync`) sein.
2. **INV-PORT-UNSAFE-1 (Zero-Unsafe):** Für das gesamte Crate gilt `#![forbid(unsafe_code)]`.
3. **INV-PORT-ZERO-PANIC-1 (Zero-Panic):** Öffentliche Schnittstellen und Implementierungen dürfen in Produktionspfaden keine unkontrollierten Panics, `unwrap()` oder `expect()` aufrufen.
4. **INV-PORT-DETERMINISM-1 (Determinismus):** Keine direkten Aufrufe von Systemzeit oder unkontrolliertem Zufall außerhalb expliziter Uhrenprovider (`SystemClock`).
5. **INV-PORT-DECOUPLING-1 (Entkopplung):** Ports definieren ausschließlich Schnittstellenverträge und minimale Basistypen. Sie dürfen keine konkreten Implementierungen fremder Crates referenzieren.
