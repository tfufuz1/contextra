# Übergabebericht MISSION `VEC-2` · HNSW-Pflege: Löschung, Rebuild, gefilterte Suche (ACORN)

## 1. Soll/Ist-Delta
- **Normwiderspruch aufgelöst:** Gemäß SPEC §11.4 und VETO-F02 erfolgt die Löschung über Tombstone-Markierung + Slot-Nullung (`INV-VEC-ZEROIZE-1`) und inkrementellen bzw. vollständigen Rebuild (`INV-VEC-CONNECTIVITY-1`, `Maintain::compact`). Slot-Nummern werden erst nach abgeschlossener Reparatur wiederverwendet.
- **INV-VEC-ZEROIZE-1:** In `HnswIndex::remove` und bei der Speicherbereinigung werden die Vektorslots im RAM über `zeroize::Zeroize::as_mut_slice().zeroize()` deterministisch und unwiderruflich mit Nullen überschrieben. Benannter Test: `test_inv_vec_zeroize_1_slot_nulling_on_deletion` in `tests/deletion_zeroize_test.rs`.
- **INV-VEC-CONNECTIVITY-1:** `HnswIndex::verify_full_connectivity` verifiziert per vollständiger Graphtraversierung (BFS über alle Schichten), dass nach `Maintain::compact` jeder lebende Knoten vom Einstiegspunkt erreichbar ist und kein Geisterzeiger auf gelöschte Slots zeigt. Benannter Test: `test_inv_vec_connectivity_1_reachable_after_compact` in `tests/hnsw_rebuild_connectivity_test.rs`.
- **Gefilterte Suche (ACORN):** Prädikat-gesteuerte Traversierung mit Selektivitäts-Augmentierung (`compute_gamma_edge_budget`) und Referenz-Orakel-Abgleich (`NaiveReferenceIndex`). Selektivität 0.0 (leere Treffermenge) und 1.0 (vollständige Abdeckung) mathematisch sauber definiert ohne NaN oder Panics. Benannter Test: `test_acorn_selectivity_zero_and_one_and_oracle` in `tests/acorn_selectivity_edge_cases_test.rs`.
- **Port-Brücken & Adapter (E-05):** Erasure-Ports (`HnswPurger`, `hnsw_purger`, `HnswScanner`, `hnsw_scanner`) wurden unter `docs/handoff/VEC-2/adapter_src/hnsw_erasure_adapters.rs` für Klasse C abgelegt.
- **Lesende inspect-Sicht:** `HnswIndex::inspect_residual` liefert einen detaillierten `ResidualReport` (in_id_map, non_zero_vector_bytes, incoming_edge_count, in_staging).

## 2. Bewusste Abweichungen & Annahmen
- **Signaturstabilität in `src/hnsw/mod.rs`:** Um die Vorgabe *"Keine Datei außerhalb deiner OWNS-Pfade geändert"* und *"Ändere keine pub/pub(crate)-Signaturen, die außerhalb deiner OWNS-Pfade genutzt werden, außer rein additiv"* einzuhalten, wurden die bestehenden Re-Exports in `src/hnsw/mod.rs` nicht gelöscht, sondern die Purger/Scanner-Typen in `purge.rs` und `scanner.rs` als `#[deprecated]` beibehalten. Die ungenutzte Duplikatdatei `src/hnsw/purger.rs` wurde gelöscht. Sobald Klasse-C-Integratoren (`contextra-engine`) auf die ausgelagerten Adapter umgestellt sind, können die Re-Exports in `mod.rs` und `lib.rs` restlos entfernt werden.

## 3. Messwerte & Evidenz
- `cargo test -p contextra-vector --lib`: 58 passed; 0 failed
- `tests/deletion_zeroize_test.rs`: 1 passed (INV-VEC-ZEROIZE-1)
- `tests/hnsw_rebuild_connectivity_test.rs`: 1 passed (INV-VEC-CONNECTIVITY-1)
- `tests/acorn_selectivity_edge_cases_test.rs`: 1 passed
- `tests/k02_snapshot_rebuild_determinism.rs`: 1 passed
- `tests/hnsw_rebuild_search_consistency.rs`: 1 passed
- `tests/hnsw_scanner_purge_test.rs`: 5 passed
- `tests/reconstruction_attack_test.rs`: 1 passed
- `tests/deletion_ghost_pointer_proptest.rs`: 1 passed
- `tests/acorn_hnsw_differential.rs`: 4 passed
- `tests/acorn_naive_recall.rs`: 3 passed
- `cargo bench -p contextra-vector --no-run`: Kompiliert fehlerfrei
- `RUSTDOCFLAGS="-D warnings" cargo doc -p contextra-vector --no-deps`: Erfolgreich
