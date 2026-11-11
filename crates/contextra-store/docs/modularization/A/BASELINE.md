# BASELINE — contextra-store Modul A (WAL)

## 1. Ausgangszustand vor Beginn
- Toolchain: `rustc 1.89.0`, `cargo 1.89.0`
- Commit HEAD: `922ee269471eb76eb4be9370c0eb0e03b0e2a3ea` (gleich `origin/main`)
- Arbeitsverzeichnis: sauber (`git status` leer)

## 2. Vorbedingung & Baseline-Befunde
- **Blocker Manifest**: In `crates/contextra-store/Cargo.toml` (eingeführt in Merge-Commit `57b0955cb`) ist das Feature `workspace = ["dep:contextra-types", "dep:contextra-ports"]` deklariert, während `contextra-types` und `contextra-ports` unter `[dependencies]` als reguläre, nicht-optionale Workspace-Abhängigkeiten aufgeführt sind.
- Dadurch scheitert jedes Cargo-Kommando (`cargo check -p contextra-store`) global bereits auf Manifest-Ebene:
  `feature 'workspace' includes 'dep:contextra-types', but 'contextra-types' is not an optional dependency`
- Gemäß den strikten Eigentumsregeln (EIGENTUM: `crates/contextra-store/Cargo.toml` ist read-only und gehört Agent G) darf dieser Defekt von Agent A nicht eigenmächtig in `Cargo.toml` behoben werden, sondern ist als externer Blocker in `FINDINGS.md` und unter "Übergaben" mit exaktem Diff-Vorschlag zu melden.
- **Lib.rs Baseline-Vorbedingung**: `src/lib.rs` deklariert `mod budget;` und `mod stats;` nicht. Die Vorbedingung zur Baseline-Reparatur greift, sobald der Manifest-Parser den Crate laden kann.

## 3. Quellbestand WAL (Eigentum Agent A)
- `src/wal/mod.rs`: 1050 Zeilen
- `src/wal/io.rs`: 1008 Zeilen
- `src/wal/flusher.rs`: 771 Zeilen (+ `flusher/command.rs`: 65 Zeilen)
- `src/wal/replay.rs`: 729 Zeilen
- `src/wal/hmac.rs`: 570 Zeilen
- `src/wal/encode.rs`: 447 Zeilen
- `src/wal/open_heal_tests.rs`: 361 Zeilen
- Tests unter `src/wal/tests/`: 3281 Zeilen (`replay_tests.rs` 1621, `io_tests.rs` 572, `hmac_tests.rs` 418, `flusher_tests.rs` 406, `encode_tests.rs` 264)
- Gesamter Quellbestand WAL: 8303 Zeilen
