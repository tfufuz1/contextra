# Contextra – Implementierer-Kompilier- und Bauanleitung (Stufe 0)

<!-- doc-stamp: 2026-10-31 | Normativ: docs/SPEC.md Fassung 2.4, PLAN.md §12.2 -->

Dieses Dokument dient als operative Anleitung für Implementierer, um den Cargo-Workspace von Contextra schrittweise zu kompilieren, Fehlerquellen zu beheben und die Testsuite auszuführen.

---

## 1. Systemvoraussetzungen

- **Rust:** Version 1.89+ (laut `rust-toolchain.toml`, Edition 2021).
- **Cargo:** Resolver v2.
- **Plattform:** Linux (x86_64 / aarch64), macOS, Windows.
- **Build-Tooling:** `just` (optional, siehe [`justfile`](../justfile)), `nix develop` (optional, siehe [`flake.nix`](../flake.nix)).
- **Umgebungsvariablen:** Keine verpflichtenden Secrets für Standard-Builds. Für Tests mit Mock-KMS/Proof-Key: `CONTEXTRA_PROOF_KEY` (mindestens 32 Bytes).

---

## 2. Aktueller Workspace-Zustand & Crate-Topologie

Der Workspace besteht nach SPEC §4 und PLAN §11 aus folgenden Schichten (Ringen 0–4):

### Ring 0: Fundament (Keine internen Abhängigkeiten oder nur Ring 0)
- `contextra-types`: Kanonische Datenstrukturen, RequestContext, Fehler (`ContextraError`).
- `contextra-sys` *(Unsafe-Insel)*: POSIX/Win32-FFI, `mmap_readonly`, Durability-Primitive (`durable_flush_file`, `durable_flush_dir`).
- `contextra-simd` *(Unsafe-Insel)*: AVX-512 / AVX2 / NEON / Skalar-Kernel für Distanzmetriken.
- `contextra-wire` *(Unsafe-Insel)*: FlatBuffers IPC, Zero-Copy Serialisierung.
- `contextra-ports`: Trait-Schnittstellen (`StorageEngine`, `Clock`, `Rng`, `IdGen`, `KmsProvider`, `Vfs`).
- `contextra-mvcc`: Multi-Version Concurrency Control (SequenceLog, TxBuffer, SSI).
- `contextra-crypto`: AEAD (ChaCha20-Poly1305 / AES-GCM-SIV), Argon2id / HKDF KDF, KeyRegistry, RevocationLog.
- `contextra-text`: BM25/BM25F, Tokenizer, Block-Max WAND.
- `contextra-vector`: HNSW, SQ8-Quantisierung, Vektor-Suche.
- `contextra-rank`: RRF-Fusion (`fusion/`), Normalisierung, Re-Ranking.
- `contextra-durable-fs`: Transaktionales VFS und Durability-Wrapper.
- `contextra-core`: Deprecated Ring-0-Modul (wird in `types`/`ports` konsolidiert).

### Ring 1: Persistenz
- `contextra-store`: LSM-Tree, Write-Ahead-Log (WAL), SSTables, Compaction, Block-Cache, Segment-Verschlüsselung.
- `contextra-checkpoint`: Snapshots, Backup-Export, Restore-Guard.

### Ring 2: Isolation & Inferenz
- `contextra-infer-ollama`: Lokale REST-basierte Ollama-Inferenz.
- `contextra-infer-onnx` *(eingefroren)*: ONNX-Runtime Embeddings/Reranking.
- `contextra-infer-candle` *(eingefroren)*: Candle-Inferenz.
- `contextra-sandbox` *(eingefroren)*: WASI/WASM-Sandbox für Filter & Merge-Operatoren.

### Ring 3: Orchestrierung & Kernlogik
- `contextra-engine`: Abfrage-Planner, Fusion, Hybrid-Suche, ErasureSaga, Background-Reaper.
- `contextra-db`: Collections, Transaktions-Fassade, Metadaten-Filter, Tenant-Scoping.
- `contextra-privacy` *[Rand]*: PII-Erkennung, Prompt-Injection-Filter, Egress-Firewall.
- `contextra-agent` *[eingefroren]*: Agenten-Laufzeit.
- `contextra-cognition` *[eingefroren]*: Semantische Aggregation & Memory-Konsolidierung.
- `contextra-graph` *[eingefroren]*: GraphRAG Wissensgraph.
- `contextra-router` *[eingefroren]*: Multi-Armed-Bandit Routing.

### Ring 4: Fassade & Schnittstellen
- `contextra`: Offizielle Library-Fassade und einzige Composition Root (`ContextraBuilder`).
- `contextra-mcp`: Model Context Protocol Server (stdio, JSON-RPC 2.0).
- `contextra-py` *[Rand]*: Python-Bindings via PyO3.
- `contextra-ingest` *[Rand]*: Dokument-Parser (PDF, DOCX, EPUB, Mail, Markdown).
- `contextra-license` *[Experimentell]*: Ed25519-Signaturprüfung.
- `contextra-audit-export` *[Experimentell]*: DSGVO-Auskunfts- und Löschexport.
- `contextra-avv-generator` *[Experimentell]*: AVV-Markdown-Generator.

### Tooling & Benchmarks
- `contextra-testkit`: Test-Doubles, `FaultVfs`, Generatoren.
- `contextra-bench`: Criterion-Benchmarks.

---

## 3. Bekannte Bauprobleme & Status der Behebungen

Die in `PLAN.md` §4.3 identifizierten Bau-Blocker werden wie folgt adressiert:

| ID | Status | Komponente | Beschreibung & Maßnahme |
|---|---|---|---|
| **B-01** | ✅ Behoben | `contextra-store/src/kv/segment.rs:152` | `delete_segment` gibt nun `Result<bool, ContextraError>` zurück und reicht Fehler aus `registry.revoke_subkey` sauber durch. Aufrufer in Tests angepasst. |
| **B-02** | ✅ Behoben | Workspace / Tooling | `xtask` wurde entfernt. Workflows und `justfile` rufen stattdessen native Cargo-Befehle (`cargo check`, `cargo test`, `cargo clippy`) auf. |
| **B-03** | ✅ Behoben | `contextra-durable-fs` & `contextra-sys` | Zeilenumbruch-Literal in `journal.rs:183` behoben (`'\n'`). Fehlende Re-Exports (`flush_dir`, `flush_file`, `fs_class`, `rename_durable`, `FlushKind`, `FsClass`) in `contextra-sys` nachgerüstet. |
| **B-04** | 🟡 Offen | `benchmarks/contextra-bench` | Fehlende Features bereinigen. Benchmarks werden bei Standard-Testläufen via `--workspace --exclude contextra-bench` ignoriert. |
| **B-05** | 🟡 Offen | `Cargo.lock` | Bereinigung veralteter `xtask`-Lockeinträge via `cargo update -p contextra-types` (automatisch bei Build). |
| **B-06** | 🟡 Offen | `contextra-ingest` | Module `safe::xml` / `safe::opc` für `fmt-docx` und `fmt-sheets` anpassen; Pfad in `chunking/stubs.rs` korrigieren. |
| **B-07** | 🟡 Offen | `contextra-py` | `FusionWeights` aus `contextra-types` statt `contextra-rank` importieren. |
| **B-09** | 🟡 Offen | `contextra-engine/.../lifecycle.rs:144` | Nicht mehr existierendes Feld `_pid_controller` aus Struct-Initialisierung entfernen. |
| **B-12** | 🟡 Offen | `contextra-db/tests/filter_tests.rs:6` | Importpfad auf `contextra_db::filter::MetadataFilter` anpassen. |

---

## 4. Schrittweise Bauanleitung für Implementierer

Implementierer sollten strikt schichtenweise von Ring 0 aufwärts kompilieren:

### Schritt 1: Workspace-Metadaten prüfen
```bash
cargo metadata --format-version 1 --no-deps > /dev/null
echo "Exit: $?" # Muss 0 sein
```

### Schritt 2: Ring 0 (Fundament) kompilieren & testen
```bash
# Fundamentale Typen & Hardware-Abstraktionen
cargo check -p contextra-types
cargo check -p contextra-sys
cargo check -p contextra-simd
cargo check -p contextra-wire
cargo check -p contextra-ports

# MVCC, Krypto, Indizes
cargo check -p contextra-mvcc
cargo check -p contextra-crypto
cargo check -p contextra-text
cargo check -p contextra-vector
cargo check -p contextra-rank
cargo check -p contextra-durable-fs

# Ring 0 Tests ausführen
cargo test -p contextra-types
cargo test -p contextra-sys
cargo test -p contextra-crypto
cargo test -p contextra-mvcc
```

### Schritt 3: Ring 1 (Speicher) kompilieren & testen
```bash
cargo check -p contextra-store
cargo check -p contextra-checkpoint

# Wichtig: Spezifische Unit-Tests für WAL & Segment-Löschung ausführen
cargo test -p contextra-store --test kv_delete_mode_wiring_test
```

### Schritt 4: Ring 2 & Ring 3 (Orchestrierung & Logik)
```bash
# Inferenz & Orchestrierung
cargo check -p contextra-infer-ollama
cargo check -p contextra-engine
cargo check -p contextra-db
cargo check -p contextra-privacy
```

### Schritt 5: Ring 4 (Fassade & Endnutzer-Schnittstellen)
```bash
cargo check -p contextra
cargo check -p contextra-mcp
```

### Schritt 6: Gesamter Kern-Check
```bash
# Prüft alle Standard-Mitglieder
cargo check --workspace
```

---

## 5. Arbeitsanweisungen bei Kompilierfehlern (Stufe 0)

1. **Kein `unwrap()` oder `panic!` hinzufügen**:
   Jeder neue oder reparierte Pfad MUSS typisierte Fehler (`ContextraError`) zurückgeben (SPEC P7).
2. **Keine Querkanten einführen**:
   Abhängigkeiten fließen ausschließlich von höheren zu niedrigeren Ringen (Ring 4 → 3 → 2 → 1 → 0).
3. **Eingefrorene Crates entkoppeln**:
   Befindet sich Code in `contextra-engine` oder `contextra-db`, der `contextra-graph` oder `contextra-adapt` aufruft, ist dieser hinter optionale Cargo-Features (z. B. `feature = "graph"`) zu kapseln oder durch Trait-Ports aus `contextra-ports` zu abstrahieren.
4. **I/O nur über Vfs**:
   Dateisystem-Operationen dürfen nicht über `std::fs` direkt im Kerncode aufgerufen werden, sondern müssen die durablen Primitiven aus `contextra-sys` bzw. den `Vfs`-Port verwenden.
