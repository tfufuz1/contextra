# contextra-testkit

Test Fixtures, Mock Engines, In-Memory Reference Implementations und Port-Konformitätssuiten.

Normativitätsklasse: **Werkzeug** (Tooling, `—`; §6.6, §6.9, `capabilities.toml`).

## Rolle in einem Satz
`contextra-testkit` stellt Test-Mocks, In-Memory Reference Stores, Injektions-Fixtures und wiederverwendbare Vertragstests (Konformitätssuiten) ausschließlich für Testumgebungen bereit.

## Ring, Klasse & Reifegrad
- **Ring:** Werkzeug / Tooling
- **Klasse:** Werkzeug (`—`)
- **Reifegrad:** `—` (Werkzeug-Crate)
- **Erlaubte Abhängigkeiten (`may_use` Obergrenze):** Beliebig, nur als `dev-dependency` zugelassen (INV-EMBED-1, Spec §6.9)

## Generic Port Conformance Suites (`contextra_testkit::conformance`)

Das Modul `conformance` stellt generische Konformitätssuiten für Port-Traits bereit.
Downstream-Adapter-Crates (z. B. LSM Storage, HNSW Vector Index, Inverted Text Index, CSR Graph) nutzen diese Suiten in ihren eigenen Integrationstests (`#[cfg(test)] mod conformance`), um korrekte Vertragserfüllung abzusichern.

### Verfügbare Suiten

1. **`run_storage_engine_suite`**: Put/Get/Delete, Overwrites, Lexikographische Scan-Reihenfolge, Mandanten-Präfix-Trennung, 1 MB Werte, Leerschlüssel/-werte, Atomare Batches, Uncommitted Tx Isolation.
2. **`run_vector_index_suite`**: Insert & Search Distanz-Sortierung, Deletion/Tombstone-Konsistenz (`all_doc_ids`), Upsert, Dimension-Mismatch Fehler, NaN-Rejection, $k > \text{size}$, DocId Handling.
3. **`run_text_index_suite`**: Indexierung & Suche, Deletion (nie wieder auffindbar), Umlaute/ß-Normalisierung, Leeranfrage-Behandlung.
4. **`run_graph_index_suite`**: Kante einfügen/löschen, Nachbarschaft (`neighbors`), Graph-Traversierung (BFS-Hops), Selbstschleifen-Behandlung.
5. **`run_clock_suite`**: Monotonie (`monotonic_nanos`), Gültige Unix-Nanosekunden.
6. **`run_rng_suite`**: Entropie-Verteilung, `fill_bytes` Abdeckung, `next_unit_f64` im Intervall $[0.0, 1.0)$.
7. **`run_metrics_sink_suite`**: Fehlerfreie Erfassung von Counter, Gauge und Histogramm Metrics.

### Einbindungsbeispiel in ein Adapter-Crate

Füge `contextra-testkit` in `[dev-dependencies]` Deines Crates ein:

```toml
[dev-dependencies]
contextra-testkit = { workspace = true }
```

Integriere die Vertragstests in `tests/conformance.rs` oder in ein Testmodul:

```rust
#[cfg(test)]
mod conformance {
    use contextra_testkit::assert_conformance;
    use contextra_testkit::conformance::*;

    #[tokio::test]
    async fn test_storage_conformance() {
        let res = run_storage_engine_suite(|| async {
            // Instanziiere Deinen Storage-Adapter
            MyLsmEngine::new()
        }).await;
        assert_conformance!(res);
    }

    #[tokio::test]
    async fn test_vector_conformance() {
        let res = run_vector_index_suite(|| async {
            // Instanziiere Deinen Vektor-Index-Adapter
            MyHnswIndex::new(128)
        }).await;
        assert_conformance!(res);
    }

    #[tokio::test]
    async fn test_text_conformance() {
        let res = run_text_index_suite(|| async {
            MyInvertedIndex::new()
        }).await;
        assert_conformance!(res);
    }

    #[tokio::test]
    async fn test_graph_conformance() {
        let res = run_graph_index_suite(|| async {
            MyCsrGraph::new()
        }).await;
        assert_conformance!(res);
    }

    #[test]
    fn test_clock_conformance() {
        let clock = MyClock::new();
        let res = run_clock_suite(&clock);
        assert_conformance!(res);
    }
}
```
