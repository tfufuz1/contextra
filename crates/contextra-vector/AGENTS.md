Modulübersicht: hnsw (inkl. erasure), diskann (experimentell), distance, quantize und quantize_rabitq, persistence (HnswHeader, MmapIndex), acorn (prädikat-gefilterte Suche), candidate_stream, compute_pool, partial_rebuild, types, contract.

Harte Invarianten:
- INV-HNSW-SLOT-1: Slots werden erst nach Rebuild wiederverwendet.
- INV-VEC-ZEROIZE-1: Gelöschte Vektoren werden sofort genullt.
- INV-VEC-CONNECTIVITY-1: Teilweiser Neuaufbau erhält die Konnektivität.
- VETO-F02: Das Feature `partial-index-rebuild` darf nicht Default werden, bevor `test_partial_rebuild_recall_regression()` über mehr als 30 Tage stabil grün war und ein ADR die Freigabe bestätigt.

Pflichten: Jede Änderung an Graph-Konstruktion oder Persistenz wird mit Recall-Messungen belegt (`baseline_recall.json`). Regressionen beim Recall gelten als Blocker. Das Format von `HnswHeader` ist versioniert. Ports-Kompatibilität über das Feature `ports-compat`.

Verifikation: `cargo test -p contextra-vector --locked`, `cargo test -p contextra-vector --features experimental`, `cargo bench -p contextra-vector`, Recall-Vergleich gegen `baseline_recall.json`.

Optimierungsfokus: Parameter `M`, `efConstruction`, `efSearch` nach Recall-Latenz-Kurve, Speicherlayout für Cache-Lokalität, Quantisierung mit Rerank-Strategie.
