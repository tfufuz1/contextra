# Rust Rules

This document consolidates rules for error handling, async I/O, dependency management, and dependency audits.

---

## Error Handling Rules

> Origin: `rules/error-handling.md`

### Single Error Type

All crates use `contextra_core::ContextraError`. No crate-local error enums.

### Variant Policy

- **Append-only**: new variants go at the end of the enum (binary compat)
- **Structured over String**: prefer `WalCorruption { offset, reason }` over `Storage(String)` for machine-parseable errors
- **`From` impls**: only in `error.rs` itself — no wildcard `From<E>` in other modules

### Real Examples (from this repo)

```rust
// ✅ Storage error with context (crates/contextra-store/src/lsm.rs:289)
let file = File::create(path_ref).await
    .map_err(|e| ContextraError::Storage(format!("Failed to create SSTable: {}", e)))?;

// ✅ Structured error (crates/contextra-store/src/sstable.rs:712)
return Err(ContextraError::ChecksumMismatch {
    path: path_buf.to_string_lossy().to_string(),
    block_id: bloom_offset,
});

// ❌ Wrong: swallowing error
let _ = file.sync_all().await;  // ONLY acceptable for best-effort cleanup

// ❌ Wrong: panic on missing value
let val = map[&key];  // Use map.get(&key).ok_or_else(|| ...)?
```

### Available Variants (as of 2026-07)

| Variant | When to use |
|---|---|
| `Internal(String)` | Logic bugs that should be unreachable |
| `InvalidInput(String)` | Caller-provided data fails validation |
| `NotFound(String)` | Key/doc/entity lookup miss |
| `Storage(String)` | Disk I/O, SSTable, WAL generic failures |
| `Io(io::Error)` | Raw I/O (auto-converted via `From`) |
| `WalCorruption { offset, reason }` | WAL integrity check failure |
| `ChecksumMismatch { path, block_id }` | CRC/hash verification failure |
| `Transaction(String)` | Tx lifecycle errors |
| `TransactionTimeout { tx_id, elapsed_ms }` | Tx exceeded TTL |
| `Index(String)` | HNSW/vector index errors |
| `Text(String)` | BM25/text index errors |
| `Crypto(String)` | Encryption/decryption failures |
| `ParseError(String)` | Deserialization failures |
| `ModelLoad { path, reason }` | GGUF/model file loading or parsing errors |
| `OrphanedVectorReference { doc_id, index_id }` | Structural split-brain: vector index entry points to non-existent document |

---

## Async I/O Rules

> Origin: `rules/async-io.md`

### Decision Tree

```
Is it WAL append / MemTable flush / directory create?
  → tokio::fs (sequential async writes)

Is it SSTable random-access read (pread at offset)?
  → std::fs::File inside tokio::task::spawn_blocking
  → Reason: tokio::fs has no equivalent to FileExt::read_exact_at

Is it file delete / rename / metadata?
  → tokio::fs::remove_file / tokio::fs::rename
```

### The spawn_blocking Pattern (SSTable reads)

```rust
// From crates/contextra-store/src/sstable.rs:542-551
let (file, file_size) =
    tokio::task::spawn_blocking(move || -> std::io::Result<(std::fs::File, u64)> {
        let file = std::fs::File::open(&path)?;
        let metadata = file.metadata()?;
        Ok((file, metadata.len()))
    })
    .await
    .map_err(|e| ContextraError::Storage(format!("Join error: {}", e)))?
    .map_err(|e| ContextraError::Storage(format!("File open failed: {}", e)))?;
```

Note the double `?` — first for `JoinError` (task panic), then for the inner `io::Error`.

### Invariant

`lib.rs` states: "Alle Disk-I/O via tokio::fs (zero std::fs imports)."
This invariant is **documented but intentionally violated** for SSTable random reads.
The violation is tracked as `TODO[STABILIZE]` in `lib.rs`.

---

## Dependency Rules

> Origin: `rules/dependencies.md`

### Before Adding Any Dependency

1. **Does it exist?** Check crates.io — LLMs hallucinate crate names ("slopsquatting")
2. **Is it maintained?** Last release within 12 months, >1 maintainer preferred
3. **License compatible?** Must be MIT OR Apache-2.0 (see workspace Cargo.toml)
4. **Is it necessary?** If the needed functionality is <20 lines of std code, write it inline
5. **`cargo audit` clean?** No known advisories for the version being added

### Before Using Any API from an Existing Dependency

**Verify the function/trait/method exists in the PINNED version from Cargo.lock.**

LLMs routinely generate calls to functions that existed in a different version or never existed.
Check docs.rs/[crate]/[exact-version] — not from memory.

### Current Workspace Dependencies (2026-07)

```toml
# Verified essential — used extensively
thiserror = "2"          # ContextraError derive
tokio = "1"              # async runtime (full features)
bytes = "1"              # zero-copy buffer management
blake3 = "1"             # hashing (keys, bloom, HMAC)
serde = "1"              # serialization
serde_json = "1"         # JSON for metadata

# Verified justified — specific use cases
crc32fast = "1.3"        # SSTable/WAL integrity checks
memmap2 = "0.9"          # memory-mapped SSTable reads
ahash = "0.8"            # fast HashMap hashing
roaring = "0.10"         # compressed bitmaps (HNSW delete tracking)
parking_lot = "0.12"     # faster Mutex/RwLock (index hot path)
flatbuffers = "24.3"     # IPC serialization

# Review needed — possibly over-specified
bincode = "1.3.3"        # used for WAL entry serialization
rand = "0.8"             # salt generation, HNSW random levels
```

### Slopsquatting Defense

If a dependency name looks unusual or you haven't seen it before:
1. Search crates.io manually
2. Verify the GitHub/repo link in Cargo.toml matches the crate
3. Check download count — very low downloads on a "utility" crate is a red flag

---

## Dependency Audit Rules

> Origin: `rules/dependency_audit.md`

> Referenziert aus `AGENTS.md`. Pflicht bei jeder neuen Abhängigkeit.

### Checkliste (alle Punkte vor `Ask-first`-Freigabe)

- [ ] **Existiert das Paket?** `cargo search <name>` — nicht aus Trainingsdaten annehmen.
- [ ] **Version** aus `Cargo.lock` bestätigen — keine angenommene aktuelle Version nutzen.
- [ ] **Lizenz** MIT oder Apache-2.0 kompatibel? `cargo license` oder crates.io prüfen.
- [ ] **Tatsächliche Nutzung** mehr als eine triviale Funktion, die 5 Zeilen Std-Code wäre?
- [ ] **Maintenance** letzter Release < 12 Monate? Offene kritische Issues?
- [ ] **Advisories** `cargo audit` grün?
- [ ] **Slopsquatting-Check** für unbekannte/neue Pakete: crates.io direkt prüfen, Eigentümer verifizieren.

### Aktuelle Abhängigkeiten mit Risikoanmerkungen

| Crate | Version | Risiko | Anmerkung |
|---|---|---|---|
| `bincode` | 1.3.3 | MINOR | Veraltetes Format — v2 bricht Serde-Kompatibilität. Pinning intentional. |
| `uuid` | 1.23.1 | OK | Nur in `contextra-store` für WAL-UUID. Korrekte Nutzung. |
| `aes-gcm-siv` | 0.11.1 | OK | Letzte v0.11.x — v0.12 existiert noch nicht. Lizenz: Apache/MIT. |
| `tokio-util` | 0.7.18 | MINOR | Nur für `TaskTracker` in `contextra-store`. Evaluieren ob `tokio::task::JoinSet` reicht. |
| `flatbuffers` | 24.12.23 | MINOR | Nur in `contextra-core` für IPC. Generierter Code (`contextra_generated.rs`) enthält `unwrap()` — nicht manuell editieren. |
