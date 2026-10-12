# Governance Rule: Robustness & Fuzz Testing Standards (`rules/fuzzing.md`)

## Gilt für
- Crates mit `fuzz/`-Verzeichnis (`contextra-crypto`, `contextra-db`, `contextra-mcp`, `contextra-store`, `contextra-text`, `contextra-vector`, `contextra-wire`)
- PRs mit Sicherheitsrelevanz (`risk=sec`) oder Modifikationen an Parsern, Decodern, Deserialisierern und WAL-Repräsentationen

## Pflichtregeln
1. **Fuzzing-Pflicht bei Risk=Sec**: Sobald Ingest-Logik, Netzwerk-Parser, Binärformate oder Krypto-Dekodierung geändert werden, muss das zugehörige Fuzz-Target ausgeführt werden (Quelle: `xtask/src/harness/fuzz_smoke.rs`).
2. **Crash-Artefakt Workflow**: Findet der Fuzzer einen Crash, darf das Binär-Artefakt unter `fuzz/artifacts/` keinesfalls in Git committet werden. Stattdessen wird aus dem Artefakt zuerst ein fehlschlagender Unit-/Regressionstest unter `tests/` erstellt (Red Phase), bevor der Fix implementiert wird.
3. **Corpus-Minimierung**: Nach erfolgreichen Fuzz-Läufen mit neuen Abdeckungen wird der Corpus mittels `cmin` minimiert, um redundante Testfälle zu entfernen.
4. **Toolchain-Installation**: Es darf NIE `cargo install fuzz` ausgeführt werden (falsches Paket). Die korrekte Installation lautet `cargo install cargo-fuzz`.
5. **Automatisierte Smoke-Prüfung**: Das Gate `fuzz-smoke` prüft geänderte Crates automatisch. Fehlt die Nightly-Toolchain oder `cargo-fuzz`, meldet die CI `FUZZ_TOOLCHAIN_MISSING`.

## Fuzz-Crates & Target-Matrix

| Crate | Target | Fokus / Invariante |
|---|---|---|
| `contextra-crypto` | `audit_chain_decode_fuzz` | Dekodiersicherheit der Hash-Kette |
| `contextra-crypto` | `deletion_proof_tamper` | Fälschungssicherheit von Deletion Proofs |
| `contextra-crypto` | `deletion_proof_verify_external_fuzz` | Externe Signaturprüfung ohne KeyManager |
| `contextra-crypto` | `revocation_log_decode_fuzz` | Parsing von Widerrufs-Logeinträgen |
| `contextra-crypto` | `wal_hmac_chain_verify_fuzz` | HMAC-Verifikationskette gegen Bitleitungsfehler |
| `contextra-db` | `rrf_fusion` | Numerische Stabilität von Reciprocal Rank Fusion |
| `contextra-mcp` | `fuzz_jsonrpc_parsing` | JSON-RPC 2.0 Ingest-Sicherheit |
| `contextra-mcp` | `fuzz_prompt_injection_guard` | Robuste Erkennung böswilliger Prompt-Muster |
| `contextra-store` | `fuzz_compaction_interleave` | Nebeneinanderliegende SSTable-Compaction |
| `contextra-store` | `fuzz_manifest_load` | Rebuild-Resilienzen im Manifest (`MFMN`) |
| `contextra-store` | `fuzz_memtable_concurrent` | Parallele Mutationen in der MemTable |
| `contextra-store` | `fuzz_recovery_arbitrary_state` | Crash-Recovery bei unvollständigen Segmenten |
| `contextra-store` | `fuzz_sstable_binary_search` | Binärsuche in SSTable-Indexblöcken |
| `contextra-store` | `fuzz_wal_replay` | Invariante WAL-Wiederherstellung |
| `contextra-store` | `wal_mutation_chaos` | Bit-Rot und Random-Truncation im WAL |
| `contextra-store` | `wal_roundtrip` | Serialisierung / Deserialisierung im WAL |
| `contextra-store` | `witness_reader_sstable` | SSTable Witness-Reader Bounds-Safety |
| `contextra-text` | `fuzz_bm25_tokenize` | Tokenizer-Stabilität bei UTF-8 Extremwerten |
| `contextra-vector` | `fuzz_hnsw_persistence` | HNSW Graph-Deserialisierung aus Mmap |
| `contextra-vector` | `hnsw_insert_search` | HNSW Graph-Integrität unter Zufallseinfügungen |
| `contextra-wire` | `fuzz_flatbuffers_ipc` | Zero-Copy FlatBuffers Ingest-Validierung |

## Befehle

```bash
# Fuzz-Target manuell ausführen (Laufzeit begrenzen)
cd crates/<crate> && cargo +nightly fuzz run <target> -- -max_total_time=60

# Crash-Artefakt reproduzieren
cd crates/<crate> && cargo +nightly fuzz run <target> fuzz/artifacts/<target>/<crash_id>

# Corpus minimieren
cd crates/<crate> && cargo +nightly fuzz cmin <target>

# Smoke-Test über alle geänderten Crates via xtask / just
cargo run -q --manifest-path xtask/Cargo.toml -- fuzz-smoke
just fuzz-all 60
```

## Verbotenes Verhalten
- Einpflegen binärer Crash-Artefakte (`fuzz/artifacts/*`) in das Git-Repository.
- Ausführen von `cargo install fuzz` statt `cargo install cargo-fuzz`.
- Stummschalten von Fuzzing-Crashes durch `catch_unwind` ohne Behebung der Ursache.

## Häufige Fehlerbilder
- `FUZZ_TOOLCHAIN_MISSING`: Nightly Toolchain (`rustup toolchain install nightly`) oder `cargo-fuzz` fehlen im Pfad.
- `FUZZ_CRASH_DETECTED`: Der Fuzzer hat einen Panic, Out-of-bounds Read oder Integer Overflow identifiziert.

## Verweise
- `xtask/src/harness/fuzz_smoke.rs`
- `justfile` (Rezept `fuzz-all`, `fuzz-minimize`)
- `crates/*/fuzz/fuzz_targets/`
