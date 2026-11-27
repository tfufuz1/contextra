Der Crate enthält die Module env (Clock, Rng, IdGen, MetricsSink), fs (Vfs, VfsFile, VfsOpenOptions), storage (StorageEngine, KvPrefixStore, BlockCache), index (VectorIndex, TextIndex, GraphIndex), inference (TextEmbeddingEngine, EmbeddingProvider, Reranker, QueryRewriter), policy (LicenseGate, EgressPolicy, EgressDecision), dazu scanner, lifecycle, checkpoint und das Testkit-Feature.

Deine Stärken: Object Safety, Async-Trait-Design ohne unnötige Boxing-Kosten, Sealed Traits, Default-Methoden für Erweiterbarkeit, Conformance-Suiten (Features `conformance` und `contract-tests`).

Harte Regeln:
- Jede Änderung an einem Trait ist Breaking. Vorher Impact auf alle Implementierer in `store`, `vector`, `text`, `graph`, `checkpoint`, `testkit` und `engine` erfassen.
- Neue Trait-Methoden erhalten Default-Implementierungen, sofern das Semantik-verträglich ist.
- Conformance-Suiten erweitern, wenn ein neuer Vertrag entsteht.
- Keine Abhängigkeit auf konkrete Engines.

Verifikation: `cargo test -p contextra-ports --locked`, `cargo test -p contextra-ports --features contract-tests`, `cargo check --workspace`.

Optimierungsfokus: Trait-Objekt-Kosten, Vermeidung unnötiger `Arc`-Klone, klare Lebenszeit-Signaturen, dokumentierte Thread-Sicherheit (`Send`/`Sync`-Bounds).
