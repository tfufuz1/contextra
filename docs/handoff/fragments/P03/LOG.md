# Protokoll AUFTRAG P03: contextra-vector

## Start-Check (Baseline VOR P03)
- [V] Toolchain: Rust 1.89.0 (`cargo 1.89.0 (c24e10642 2025-06-23)`).
- [V] BUILD_LADDER.md: Nicht vorhanden in `docs/handoff/fragments/P00/`.
- [V] Vorkompilierungsstatus `contextra-vector`: Crate baute vor Arbeitsbeginn nicht sauber durch vorherige ungemergte Zwischenstände (E0592 doppelte `remove`/`rebuild_sync`, E0599 fehlendes `Zeroize`, E0609 fehlendes Feld `kernel` auf `HnswColdCore`, E0599 fehlende `try_new_with_rng_and_kernel`).
- [V] Alle anderen Pfade im Workspace sind READ-ONLY.

## Lückenanalyse (recall_vs_brute_force.rs)
- Bestand: `tests/recall_vs_brute_force.rs` prüfte 5 000 Vektoren (128d, Seed 42, 43, 44, 45, 200 Queries) mit Recall@10 >= 0,95 (Cosine, Euclidean, DotProduct) und Recall@1 >= 0,99 (Cosine).
- Ziel laut Auftrag: 10 000 Vektoren, 128 d, fester Seed, Brute-Force-Orakel, Recall@10 >= 0,95 bei dokumentierten `M` und `ef`.
- Lücke: 10 000 Vektoren fehlen; wird in WP-40 ergänzt.

## WP-40: HNSW-Recall Verifikation
- Commit: `WP-40: HNSW-Recall Lückenanalyse und 10k-Vektoren Recall@10-Test`
- Prüfbefehl: `cargo test -p contextra-vector --test recall_vs_brute_force test_hnsw_recall_at_10_target_10k_vectors -- --nocapture` -> grün
- Parameter:
  - Vektoren: 10 000
  - Dimensionen: 128
  - Seed: 42 (StdRng)
  - Distanzmetrik: Cosine
  - M = 32
  - ef_construction = 200
  - ef_search = 150
  - Queries: 100
- Gemessener Recall@10: **0.9760** (Ziel >= 0.95 erfüllt).
- Brute-Force Orakel: O(N) exakter linearer Scan mit nativer skalarer Kosinus-Distanz (`compute_distance_scalar`), unabhängig von HNSW-Implementierung.

## WP-41: Lösch-Konsistenz
- Analyse Vorbestand:
  - `hnsw_scanner_purge_test.rs` prüfte funktionale Aspekte von Tombstones vs. Hard-Purge und Byte-Needle-Abwesenheit auf Serialisierungsebene.
  - `partial_rebuild_recall.rs` prüfte lokalen Recall nach Nachbarschaftslöschung mit 15% Cluster-Löschung.
- Neuer Test: `crates/contextra-vector/tests/deletion_consistency_recall_test.rs`
  - Prüft 2 000 Vektoren (64d, Cosine, Seed 12345), 20% deterministische Löschung (400 Dokumente).
  - Invariante 1: Gelöschte IDs tauchen in den k-NN Suchergebnissen (k=10, 50 Queries) vor und nach Rebuild niemals auf (0 Treffer gelöschter IDs).
  - Invariante 2: Recall der Restmenge bleibt >= 0.95 gegenüber dem gefilterten linearen Brute-Force-Orakel. Gemessen: **1.0000** vor Rebuild, **1.0000** nach Rebuild.
  - Invariante 3: `index.check_connectivity()` bestätigt intakte BFS-Erreichbarkeit auf Layer 0 nach Rebuild.
- Prüfbefehl: `cargo test -p contextra-vector --test deletion_consistency_recall_test -- --nocapture` -> grün

## WP-42: Persistenz-Header & Fuzz-Target
- Commit: `WP-42: HnswHeader Prüfsummen-Validierung und Fuzz-Target fuzz_mmap_header`
- `HnswHeader`:
  - Trägt Version (`HNSW_VERSION = 2`, `version: u16`) und kryptographische Prüfsumme via BLAKE3 (`header.checksum() -> [u8; 32]`, `header.verify_checksum(&expected)`).
  - Test `test_hnsw_header_version_and_checksum_verification` in `src/persistence/tests.rs` verifiziert deterministische Prüfsummenbildung und Fail-Closed-Verhalten bei Manipulation.
- Fuzz-Target:
  - Neues Fuzz-Target `crates/contextra-vector/fuzz/fuzz_targets/fuzz_mmap_header.rs` implementiert.
  - In `crates/contextra-vector/fuzz/Cargo.toml` als `[[bin]]` registriert.
  - Verifikation: `cargo check --manifest-path crates/contextra-vector/fuzz/Cargo.toml` -> grün.
  - Vermerk: `cargo-fuzz` Binary ist in der Sandbox-Umgebung nicht systemweit installiert (`no such command: fuzz`); stattdessen erfolgreicher Build-Check und Syntax/Typ-Verifikation über cargo check für fuzz-Targets.

## WP-43: Verschlankung & Feature-Gating
- Commit: `WP-43: Verschlankung & Feature-Gating Analyse`
- Audit der Features:
  - `acorn`: Notwendig für `src/hnsw/acorn_filtered.rs` (gefilterte Suche).
  - `diskann`: Bleibt als opt-in Feature `experimental-diskann` erhalten.
  - `quantize_rabitq`: Bleibt als opt-in Feature `experimental-rabitq` erhalten.
  - Externe Konsumenten (`contextra-engine`, `contextra-db`) wurden in `PENDING_API_CHANGES.md` dokumentiert.
