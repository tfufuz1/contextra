# Implementierungsplan: contextra-ports

**Ebene:** L1 · **Ziel-Zustand:** Verträge in klar getrennten Modulen: `kernel` (synchron, Fundament), `retrieval` (Lesen), `policy`, `system`, `legacy` (bis G-CUTOVER, Feature).
**Basis:** SPEC R14 §2.1, §2.2, INV-PORT-1, §13.3; R15 §2.1 (B-02, B-08) und §2.3.

## 1. Ist-Befund (gemessen)

Dies ist der zentrale Befund des Workspaces. Die SPEC beschreibt `contextra-ports::kernel`. **Dieses Modul existiert nicht.**

- 40 Dateien, 7 255 Zeilen. Oberste Module: `attention`, `checkpoint`, `clock`, `conformance`, `embedding`, `env`, `erasure`, `fs`, `graph`, `graph_index`, `id_gen`, `index`, `inference`, `kv`, `kv_bridge_port`, `ledger`, `license`, `lifecycle`, `metrics`, `observability`, `observer`, `plugin`, `policy`, `query_rewriter`, `reranker`, `rng`, `scanner`, `storage`, `text_index`, `vector_index`, `vfs`, `barrier`, `gate`, `keys`.
- **Kein** Trait `Fs`, `Cipher`, `Mac`, `Kdf`, `Checksum`, `Anchor`, `KeyProvider`, `MemoryBudget`, `HostProbe`.
- `contextra-storage` importiert aus `contextra_storage_ports::traits::` (42 Stellen) die Namen `Anchor`, `AnchorStrength`, `AnchorValue`, `Authority`, `Checksum`, `Cipher`, `Clock`, `ClockReading`, `ClockSkew`, `Entropy`, `File`, `FileRole`, `FourEyesProof`, `Fs`, `FsCaps`, `FsLock`, `IoCtx`, `Kdf`, `Mac`, `MemoryBudget`, `MetricLabel`, `MetricLabelKey`, `MetricName`, `MetricSample`, `Principal`, `ResourceClass`, `ResourcePermit`. Dazu `limits::MAX_GROUP_BYTES`, `limits::REDO_SEGMENT_BYTES`.
  **Diese Crate existiert nicht.** Das ist die direkte Ursache dafür, dass `contextra-storage` nicht baut.
- `ports` hängt von `blake3` und `futures-util` ab. Beides ist laut SPEC für den Kernel verboten.
- `ports/src/fs.rs` und `env.rs` existieren. Inhalt noch nicht gelesen; vor Umbau prüfen, ob sie den gesuchten Kernel-Verträgen entsprechen.
- `ports/src/vfs.rs` und `erasure.rs` definieren die Altverträge `Vfs`, `StorageBackend`, `ErasureLayer`.

## 2. Entfernen

| Datei/Modul | Aktion | Grund |
|---|---|---|
| `kv.rs`, `kv_bridge_port.rs`, `attention.rs`, `inference.rs` | löschen | NZ-18, NZ-23, DEC-38 |
| `embedding.rs` (`TextEmbeddingEngine`, `LlmTextGenerator`, `LlmTextGeneratorStreaming`) | auf `TextGenerator` (schmal, ohne Streaming) reduzieren | §2.2 |
| `query_rewriter.rs` | in `TextGenerator` integrieren | §2.2 |
| `storage.rs` (`StorageEngine`) | nach `legacy/` verschieben | §13.3 |
| `barrier.rs` (`CommitBarrier`, `PurgeBarrier`) | nach `legacy/` verschieben | §13.3 |
| `vfs.rs` (`Vfs`, `VfsFile`) | nach `legacy/` verschieben | ersetzt durch `Fs` |
| `erasure.rs` (`ErasureLayer`, `StorageBackend`, `ErasureTarget`) | nach `legacy/` verschieben | §13.3 |
| `scanner/` (`Purger`, `LayerScanner`) | nach `legacy/` verschieben | §13.3 (`PartitionSink::rewrite_without` ersetzt) |
| `blake3`, `futures-util` in `Cargo.toml` | entfernen | verboten im Kernel; `futures` nur in `retrieval`/`policy`/`system` (§2.1 Regel 7) |
| `TxId`-Verweise | entfernen | §13.3 |

## 3. Ändern

- **`lib.rs`:** Re-Exports (`pub use … ::*`) entfernen. Jedes Modul wird explizit referenziert. Grund: Flaches Re-Export-Netz verdeckt Altnamen.
- **`index.rs`, `vector_index.rs`, `text_index.rs`, `graph_index.rs`:** auf reinen Lesevertrag reduzieren: `search`, `search_at(epoch)`, `traverse*`, `stats`. Schreibmethoden entfernen. Persistenz-Verträge leben in den `contract`-Modulen der Retrieval-Crates (FIX-06).
- **`anchor`:** neu, aus `ledger.rs`/`checkpoint.rs` herauslösen. `Anchor::strength() -> Strong | Weak`, `read()`, `advance(expect, next)` als CAS.
- **`metrics.rs`:** geschlossener Katalog. Labels nur aus den Enums in §15 (`durability`, `result`, `file_role`, `resource_class`, `phase`, `stage`, `owner_kind`, `ErrorClass`).
- **`observer.rs`:** `Observer`/`Metrics` nicht blockierend, nie unter Sperren.
- **`policy.rs`:** `EgressPolicy` mit Standard `DenyExternal` für `Personal` (DEC-37).
- **`license.rs`:** `LicenseGate` darf keine Kerninvariante abschwächen (§1.5). Die Schnittstelle erhält einen Kommentar und einen Test, der prüft, dass kein Kernpfad sie aufruft.

## 4. Implementieren

### 4.1 `ports::kernel` (neu, vorrangig)

Synchron, ohne `futures`, `serde`, `std::fs` (INV-PORT-1). Inhalt:

- `Fs`: `open_ro`, `create_new`, `read_at`, `write_at`, `read_ranges`, `append_prealloc`, `preallocate`, `sync_data`, `sync_all`, `sync_dir`, `rename`, `lock`, `list`, `free_space`; optional `link`, `reflink`, `punch_hole` über `FsCaps`. `FileRole`, `IoCtx` als Parameter.
- `Clock`: `read() -> ClockReading { monotonic_nanos, utc_nanos }`, `ClockSkew`. Uhr nie für Stamps (AN-09).
- `Entropy`: `fill(&mut [u8]) -> Result`.
- `Anchor`: wie oben. `AnchorValue = (erase_epoch, ledger_head, catalog_epoch)`.
- `KeyProvider`: `create_cluster_key`, `create_cluster_keys` (Batch, ≤ 256, idempotent je `BatchId`, durabel), `destroy_keys`, `keyed_mac`. Quelle: §8.4.
- `Cipher`: `seal(key, nonce: &[u8; 24], aad, plaintext, out)`, `open(...)`. XChaCha20-Poly1305.
- `Mac`: `tag(key, data) -> [u8; 32]`; `verify`.
- `Kdf`: `derive(root, info) -> Key32` (HKDF-SHA256).
- `Checksum`: `crc32c(data) -> u32`.
- `MemoryBudget`: `try_reserve(class, bytes) -> Result<ResourcePermit, Error>`. `ResourcePermit` gibt per `Drop` frei.
- `HostProbe` (optional): Swap, Core-Dumps, memlock, FS-Typ, Geräte-ID. Nur Diagnose.
- `Authority` (optional): prüft `FourEyesProof` für `LiftPolicy::FourEyes`.
- `Principal`: opaque Kennung.
- `Error`: der Fehlertyp kommt aus `contextra-types`.

Damit erfüllt `contextra-storage` alle 42 importierten Namen, wenn der Import auf `contextra_ports::kernel::…` umgestellt wird.

### 4.2 `ports::conformance`

Je Kernel-Port eine generische Testfunktion (`fn fs_conformance<F: Fs>(f: &F)`). Jeder Adapter ruft sie auf. Heute existiert `conformance.rs`, Inhalt vorher prüfen und an Kernel-Ports anpassen.

### 4.3 `legacy`-Feature

`ports::legacy` (Feature `legacy`, Standard aus) enthält alle verschobenen Altverträge. Ziel: nach G-CUTOVER gelöscht.

## 5. Tests

- `INV-PORT-1` als Test: `cargo tree -p contextra-ports -e normal` darf weder `futures`, `serde` noch `blake3` enthalten, außer in `retrieval`/`policy`/`system` (über Feature).
- Konformanzsuite läuft für jeden Adapter (MemFs, FaultFs, echter Dateisystem-Adapter).
- Dyn-Safety-Test: jedes Kernel-Trait ist `dyn`-tauglich (`dyn_safety`-Modul existiert bereits in `lib.rs:139`, auf Kernel erweitern).

## 6. Abhängigkeiten nach Umbau

- `kernel`: `contextra-types`, `thiserror`, `bytes`.
- `retrieval`/`policy`/`system`: zusätzlich `futures-util` (nur dort).
- `legacy`: alles, was bisher galt.
- Verboten im Kernel: `blake3`, `serde`, `tokio`, `std::fs`.

## 7. Gate und Reihenfolge

1. **P0-A:** `ports::kernel` anlegen, `contextra-storage`-Importe umstellen (Phantom-Crate entfernen). **Das ist der Schritt, der das Workspace überhaupt baubar macht.**
2. Altverträge in `legacy` verschieben, Konsumenten (`engine`, `checkpoint`, `mvcc`) hinter Feature `legacy-store` setzen.
3. Konformanzsuite für alle Kernel-Ports.
4. **G-0:** Konformanz `Fs`/`Cipher`/`KeyProvider`/`Anchor` grün, `INV-PORT-1` als Test.

**Risiko:** `contextra-storage` und `contextra-engine` hängen direkt an dieser Crate. Die Umbenennung ist mechanisch, aber breit (42 + 26 Dateien). Ein Skript mit Vorher-/Nachher-Diff ist Pflicht.
