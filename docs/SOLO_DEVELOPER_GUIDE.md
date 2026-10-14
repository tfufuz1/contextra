# Contextra — Das Solo-Entwickler-Handbuch (Bare-Metal Laptop Edition)

> **Stand:** Oktober 2026  
> **Zielgruppe:** Der alleinige menschliche Entwickler von Contextra.  
> **Umgebung:** Echter Bare-Metal-Laptop (keine VM, kein Docker-Overhead, vollständiger direkter Hardwarezugriff).

---

## 1. System- & Hardware-Profil deines Laptops

Hier sind alle tatsächlichen Hardware- und Umgebungskennzahlen deines Systems, die direkt auf deinem Rechner ermittelt und geprüft wurden:

| Komponente | Tatsächlicher Hardware-Wert | Bedeutung für Contextra & LLM |
|---|---|---|
| **Betriebssystem** | Debian GNU/Linux 13 (trixie) 13.7 (x86_64) | Stabile, moderne Basis mit aktuellem glibc- und Linux-Kernel |
| **Prozessor (CPU)** | **AMD Ryzen 5 3500U with Radeon Vega Mobile Gfx** | 4 physische Kerne, 8 Threads (Zen+ Architektur, 2.1 GHz Basis, bis zu 3.7 GHz Boost) |
| **CPU-Vektorbefehle** | **AVX2, FMA3, BMI1, BMI2, SHA-NI, SSE4.2** | Vollständige Hardwarebeschleunigung für `contextra-simd` und BLAS/Tensor-Matmul |
| **Grafik (GPU)** | **AMD Radeon Vega Series (Picasso/Raven 2 Mobile iGPU)** | Integrierte Grafikeinheit (nutzt Shared System RAM). **Kein Nvidia CUDA!** |
| **Arbeitsspeicher (RAM)** | **~9.65 GiB physisch (10.121.016 kB)** | Ausreichend für RAG + Rust-Builds; Vorsicht bei parallelen `cargo build -j8` + 7B-Modellen! |
| **Swap-Speicher** | **~13.3 GiB (13.281.276 kB)** | Großzügiger NVMe-Swap-Puffer verhindert OOM-Kills bei speicherintensiven Builds |
| **Festplatte (NVMe)** | **NVMe SSD (`/dev/nvme0n1p3`)** | 384 GB Gesamt, **109 GB frei** auf `/home`. Exzellente I/O-Performance für WAL & SSTables |
| **Rust Toolchain (Pin)** | **Rust 1.89.0** (`rust-toolchain.toml`) | Aktive Compiler-Version im Workspace |
| **Nightly Toolchain** | `nightly-x86_64-unknown-linux-gnu` vorhanden | Notwendig für `miri`, `loom`, `cargo-fuzz` |
| **Lokale LLM-Engine** | **Ollama** aktiv (`http://localhost:11434`) | Installierte Modelle: `qwen2.5-coder:1.5b`, `qwen:latest`, `nomic-embed-text`, `deepseek-r1:1.5b` |

> [!TIP]
> **Ressourcen-Empfehlung für deinen Ryzen 5 3500U:**
> - Bei großen Cargo-Kompilierungen empfiehlt es sich, `CARGO_BUILD_JOBS=4` oder `CARGO_BUILD_JOBS=6` zu setzen. Das verhindert, dass das System durch parallele LLVM-Codegen-Phasen in den Swap gedrängt wird oder die CPU thermisch drosselt.
> - Bei lokalen LLMs ist das Modell **`qwen2.5-coder:1.5b`** (ca. 986 MB RAM) auf deiner 4-Core CPU unschlagbar schnell (ca. 25–40 Tokens/s) und beansprucht kaum Speicher!

---

## 2. Status der Toolchain & Sofort-Setup (Pre-Flight)

Die automatische Attestierung (`cargo xtask env-attest`) hat dein System geprüft. Folgende Werkzeuge sind **bereits voll einsatzbereit**:
- `git`, `rg` (ripgrep), `jq`
- `cargo-nextest` (High-Speed-Testrunner)
- `cargo-fuzz` (LLVM libFuzzer)
- `cargo-mutants` (Mutation-Testing)
- `xtask` (alle 50+ Harness-Kommandos kompiliert)

### ⚠️ Zwei kleine Werkzeuge fehlen noch für vollständige Grüne Gates:

Führe diese beiden Befehle einmalig im Terminal aus, um die Toolchain zu vervollständigen:

```bash
# 1. Miri für Undefined-Behavior- & Unsafe-Insel-Prüfungen installieren
rustup component add --toolchain nightly-x86_64-unknown-linux-gnu miri

# 2. Cargo-Audit für Security-Scans installieren
cargo install cargo-audit

# (Optional: Snapshot-Tests & Workflow-Linter)
cargo install cargo-insta
```

Prüfe danach jederzeit den Zustand mit:
```bash
cargo xtask env-attest
```

---

## 3. Architektur-Analyse: Wäre Candle sinnvoller wegen KV-Cache? (Candle vs. Ollama)

Du hast gefragt: **„wäre es sinnvoller mit Candle wegen kv ??“**

Hier ist die detaillierte, ehrliche und architektonische Antwort für dein konkretes System:

### 3.1 Das Dilemma: Was bietet Candle vs. Ollama in Contextra?

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           DEIN LAPTOP (Ryzen 5 3500U)                       │
├──────────────────────────────────────┬──────────────────────────────────────┤
│               OLLAMA                 │                CANDLE                │
│       (contextra-infer-ollama)       │       (contextra-infer-candle)       │
├──────────────────────────────────────┼──────────────────────────────────────┤
│ • Externer Daemon (HTTP :11434)      │ • In-Process Rust-Bibliothek         │
│ • Basiert auf optimiertem C++        │ • Pure-Rust Tensor-Engine            │
│   (llama.cpp mit AVX2-Vektorisierung)│ • Keine C-Bindings                   │
│ • Prompt-Caching / KV-Reuse          │ • Direkter Speicherzugriff auf       │
│   vollautomatisch im Hintergrund     │   `LayerKv`, `KvState`, Attention    │
│ • Extrem schnell auf 3500U CPU       │ • Ermöglicht KV-Bridge & Spill-to-LSM│
│ • KEIN Zugriff auf Tensoren in Rust  │ • Auf reiner CPU langsamer als C++   │
│ • Völlig stabil & sofort nutzbar     │ • Benötigt GGUF-Downloads im Heap    │
└──────────────────────────────────────┴──────────────────────────────────────┘
```

### 3.2 Warum ist KV-Cache in Contextra v2 so besonders?

In Contextra ist der KV-Cache nicht bloß ein temporäres Array, sondern ein eigenständiges Subsystem:
1. **`contextra-kvcache` (Ring 1):** Bietet einen **Prefix-Radix-Tree**, Paged KV-Blöcke, mandantenisolierte AES-256-GCM-SIV-Verschlüsselung (`TenantIsolatedKvStore`) und KIVI 2-Bit/4-Bit Quantisierung.
2. **Die KV-Cache-Bridge (`crates/contextra-infer-candle/src/kv_bridge.rs`):** Verbindet die Retrieval-Suchtreffer (Context-Hits) direkt mit den KV-Tensoren des LLMs, sodass bei wiederholten RAG-Abfragen **0 Tokens neu berechnet (Prefill)** werden müssen, sondern die Tensoren direkt aus dem LSM-Speicher injiziert werden.

### 3.3 Die Wahrheit für deinen Ryzen 5 3500U:

1. **Ollama kann keine rohen KV-Tensoren an Rust übergeben:**
   Ollama ist eine HTTP-Blackbox. Du kannst über JSON Prompts senden und Tokens empfangen. Ollama nutzt zwar internen Prefix-Cache (Prompt-Caching), aber du kannst die KV-Tensoren **nicht** in Contextras LSM-Store speichern, **nicht** kryptografisch beweisen und **nicht** mit Contextras Radix-Tree manipulieren.
2. **Candle hat vollen KV-Zugriff, aber läuft auf deinem Laptop ohne CUDA:**
   Dein Laptop besitzt eine AMD Radeon Vega iGPU. Das Feature `cuda` funktioniert darauf nicht. Candle muss im puren CPU-Modus laufen. Auf einer 4-Kern-CPU ist die pure-Rust-Inferenz von Candle spürbar zäher als die hochoptimierten Assembler/AVX2-Kernel von llama.cpp (Ollama).

### 3.4 Das Urteil & die Empfohlene 2-Phasen-Strategie:

> [!IMPORTANT]
> **Die klare Empfehlung:**
> 
> * **Phase 1 — SOFORT (Dogfooding & RAG zur Contextra v2 Entwicklung):**  
>   👉 **Nimm OLLAMA mit `qwen2.5-coder:1.5b` und `nomic-embed-text`!**  
>   Warum? Weil du damit in 5 Minuten ein rasend schnelles, stabiles lokales RAG-System hast, das die gesamte Contextra-Spezifikation (`docs/spec/`) und deinen Code kennt. Qwen hilft dir beim Schreiben und Verstehen von Contextra v2 Code, ohne dass dein RAM überläuft oder du mit GGUF-Tensoren kämpfen musst.
>
> * **Phase 2 — VERTIEFUNG (Entwicklung des Contextra v2 KV-Cache-Kerns):**  
>   👉 **Nimm CANDLE (`contextra-infer-candle` mit `--features kv-bridge,kv-stage-b`)**, wenn du die kryptografische KV-Bridge, den Paged Radix-Tree und das LSM-Spilling implementierst und testen willst! Für Unit-Tests und Invarianten-Prüfungen mit kleinen Test-Tensoren ist Candle perfekt geeignet.

---

## 3.5 Reifegrad- & Stabilitäts-Analyse: Was geht HEUTE schon vs. Was fehlt noch?

Nach der Gesamtsynthese aller 20 Tiefenaudits ([`docs/analysis/AUDIT_GESAMTSYNTHESE_2026-10-14.md`](file:///home/freddy/Projekte/contextra/docs/analysis/AUDIT_GESAMTSYNTHESE_2026-10-14.md)) und dem [`docs/14-10-26/ARCHITEKTUR_MASTERPLAN_v2.md`](file:///home/freddy/Projekte/contextra/docs/14-10-26/ARCHITEKTUR_MASTERPLAN_v2.md) gliedert sich Contextra in **6 disjunktive Endprodukte (EP1 bis EP6)**:

| ID | Endprodukt | Zielversion | Kernfunktion & Geltungsbereich | Heutiger Status |
|---|---|---|---|---|
| **EP1** | **Contextra Memory** | **Release 1.0** | Lokaler Multi-Tenant-Speicher (Vektor HNSW + Volltext BM25 + KV), LSM-Store, Purge v2 | 🟡 **Alpha-Ready** (Kompiliert, Basisfunktionen laufen; 6 Blocker in Härtung) |
| **EP2** | **Contextra Proof** | Nach K4 | BSI TR-02102 Löschnachweise v4, 7-Schichten-Beweiskette, autarker Offline-Verifier | 🟡 Krypto solide, Crate-Split und Verifier-Extraktion ausstehend |
| **EP3** | **Contextra Retrieval+** | Jahr 2 | Wissensgraph (CSR), Bi-temporale Filter, OpenIE, budgetierte k-Pfad-Diffusion (PathRAG) | 🟡 Graph implementiert, k-Path-Verdrahtung in Engine ausstehend |
| **EP4** | **Contextra Inference** | Jahr 3 | Lokale Embedder, Irminsul Radix-Trie KV-Cache, 2-Bit KIVI-Quantisierung, SnapKV | 🟡 Ollama voll nutzbar; Candle KV-Bridge in Stufe B |
| **EP5** | **Contextra Cognition** | Experimentell | Adaptive Kognition, Sketched LinUCB Bandits, Brand's Incremental SVD Dedup | 🔲 Prototypisch in `contextra-adapt` / `cognition` |
| **EP6** | **Contextra Guard** | Nach Bedarf | Egress-Firewall, PII-Maskierung, WASM-Sandbox-Isolation, MCP-Sicherheitsgate | 🟡 MCP-Server lauffähig; Sandbox-Isolation als opt-in Plugin |

> [!IMPORTANT]
> **Was bedeutet „STABILES SYSTEM“ für den Solo-Entwickler?**  
> Für die lokale Nutzung und Entwicklung ist **ausschließlich EP1 (Contextra Memory)** die maßgebliche Stabilitätsgrenze! Alle höheren Produkte (Proof, Graph, KIVI, WASM) sind modulare Plugins, die für den Kernbetrieb als lokaler RAG-Speicher **nicht** zwingend erforderlich sind.

### ⚠️ Die 8 konkreten Blocker, die aktuell noch für Release 1.0 (EP1) fehlen:

Aus den Logs (`logs/audits/`) und der Gesamtsynthese ([`docs/analysis/AUDIT_GESAMTSYNTHESE_2026-10-14.md`](file:///home/freddy/Projekte/contextra/docs/analysis/AUDIT_GESAMTSYNTHESE_2026-10-14.md)):
1. **Test-Kompilierfehler (`crates/contextra-store/tests/kv_crypto_shred_instant_unrecoverable.rs:34`):**  
   `assert!(revoked)` auf `Result<bool, CryptoError>` schlägt fehl. Blockiert aktuell `cargo xtask check-compile`.
2. **Blocker N-B (Downcast im Löschpfad):**  
   In `crates/contextra-engine/src/collection/crud/delete.rs:79-109`: `(self.storage.as_ref() as &dyn Any).downcast_ref::<LsmStorage>()` schlägt unter `TenantScopedStorage` fehl und wirft `CapabilityUnsupported`. Muss durch den `ErasureBackend`-Port in `contextra-ports` ersetzt werden.
3. **Blocker N-K (Physischer vs. logischer Scope im Store):**  
   In `crates/contextra-store/src/tenant_codec.rs`: Logischer Scope matchte `__col:{id}:` statt der mandantengeprägten Schlüssel `t:{tenant_id}:...`. Der Purge löschte im LSM-Baum 0 Datensätze.
4. **Blocker L1, L4, L5, L6 (Purge v2 Korrektheitslücken im Store):**  
   - **L1:** MemTable-Tombstone wurde vor dem `ErasureLedger`-Append gesetzt (Resurrection-Gefahr bei Crash).
   - **L4:** Obsolete Quell-SSTables blieben auf der SSD, da `PurgeOutput.obsolete_files` leer war (muss `scrub_and_remove` aufrufen).
   - **L5:** `while let Ok(Some(..))` verschluckte SSTable-Korruptionen still (muss 3-Arm-Match sein).
   - **L6:** Sequenzgrenzen und relativer Rang gingen beim SSTable-Rewrite verloren.
5. **Blocker INV-MVCC-SSI-1 (Write-Skew Lücke in MVCC):**  
   In `crates/contextra-mvcc/src/ssi.rs`: Lese-Iteratoren führten kein durchgängiges Read-Set-Tracking. Dadurch bestand Write-Skew-Risiko bei Transaktionen unter Serializable Snapshot Isolation.
6. **Blocker S5 (7 unverdrahtete Builder-Optionen):**  
   In `crates/contextra/src/builder.rs`: Optionen wie `with_privacy`, `with_adapt`, `with_kms` wurden mit `let _ = ...;` ignoriert. `verify_wiring(&self)` muss im Produktionsstartpfad erzwungen werden.
7. **Blocker B4 (MCP-Server liefert `proof: null`):**  
   In `crates/contextra-mcp/src/server_tools.rs`: `contextra_forget` führte die Löschung aus, verwarf aber das Ergebnis und lieferte hardcodiert `proof = null`.
8. **Fehlende Toolchain-Werkzeuge:**  
   `miri` (für Unsafe-Insel-Prüfungen) und `cargo-audit` (für Security-Gates) müssen noch per Terminal nachinstalliert werden.

---

## 3.6 Die normative Roadmap: Wann ist das stabile System lokal nutzbar und getestet?

Basierend auf dem konsolidierten Phasenplan aus [`docs/14-10-26/ARCHITEKTUR_MASTERPLAN_v2.md`](file:///home/freddy/Projekte/contextra/docs/14-10-26/ARCHITEKTUR_MASTERPLAN_v2.md) (§15) und der disjunktiven Spezifikation [`docs/14-10-26/DISJUNKTIVE_MIKRO_IMPLEMENTIERUNGSSPEZIFIKATION_v1.md`](file:///home/freddy/Projekte/contextra/docs/14-10-26/DISJUNKTIVE_MIKRO_IMPLEMENTIERUNGSSPEZIFIKATION_v1.md) (§10):

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                               CONTEXTRA RELEASE-ROADMAP                                │
├────────────────────────────────────────────────────────────────────────────────────────┤
│  M1: Sofort (Tage 1–3) — Dev-Ready & RAG-Dogfooding                                    │
│  ├── Test-Kompilierfehler in kv_crypto_shred fixen                                      │
│  ├── MCP-Server mit --features ollama & stdio anbinden                                 │
│  └── Lauffähiges Standalone-RAG mit Qwen 2.5 Coder & nomic-embed-text                  │
├────────────────────────────────────────────────────────────────────────────────────────┤
│  M2: Woche 1–2 — Store & Purge v2 Härtung (Phase 0c–0e / K1–K2)                        │
│  ├── N-K Mandanten-Scope im Purge (PhysicalScope t:{tenant_id}:...)                    │
│  ├── L1–L6 Streaming Rewrite Purge v2 & physisches SSTable-Scrubbing                  │
│  ├── INV-MVCC-SSI-1 Read-Set Tracking gegen Write-Skew                                 │
│  └── S5 Builder-Wiring Pflichtprüfung (verify_wiring())                                │
├────────────────────────────────────────────────────────────────────────────────────────┤
│  M3: Woche 3–4 — EP1 CONTEXTRA MEMORY RELEASE 1.0 (STABLE)                             │
│  ├── HNSW-Arena RAM-Zeroisierung (GhostFreeVectorIndex)                                │
│  ├── Vollständige Entkopplung von contextra-core (Ring-0 Async-Purity)                 │
│  └── Grüne Kernel-Gates: cargo xtask verify --tier kernel                              │
│  👉 AB HIER: Vollständig stabiler, transaktionssicherer lokaler Speicher!              │
├────────────────────────────────────────────────────────────────────────────────────────┤
│  M4: Woche 5–8 — EP2 CONTEXTRA PROOF                                                   │
│  ├── 7-Schichten-Beweiskette nach BSI TR-02102                                         │
│  └── Standalone Verifier-Crate contextra-proof-verify                                   │
├────────────────────────────────────────────────────────────────────────────────────────┤
│  M5: Monat 3–4 — EP3 RETRIEVAL+ & EP4 INFERENCE                                        │
│  ├── PathRAG budgetierte k-Pfad-Diffusion (KPathDiffusion)                             │
│  └── Irminsul Radix-Trie KV-Cache mit KIVI 2-Bit Quantisierung                         │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

### Die Meilensteine im Detail:

| Meilenstein | Phase / Task | Fokus & Lieferumfang | Wann fertig? | Akzeptanz-Befehl |
|---|---|---|---|---|
| **M1: Dev-Ready & Dogfooding** | **Phase 0a/0b (K0)** | • Test-Compile-Fehler beheben<br>• MCP-Server mit `--features ollama` starten<br>• Lauffähiges RAG-Beispiel mit Qwen 2.5 Coder | **Sofort (1–3 Tage)** | `cargo xtask check-compile`<br>`cargo test -p contextra-infer-ollama` |
| **M2: Store & Purge v2 Härtung** | **Phase 0c–0e (K1–K2)** | • N-K Mandanten-Scope im Purge<br>• L1–L6 Streaming Rewrite & SSTable-Scrubbing<br>• SSI Read-Set Tracking in `contextra-mvcc`<br>• Builder `verify_wiring()` aktiv | **Woche 1–2** | `cargo test -p contextra-store`<br>`cargo test -p contextra-mvcc`<br>`cargo xtask wal-replay-verify` |
| **M3: EP1 CONTEXTRA MEMORY (STABLE 1.0)** | **Phase 1–2 (K3)** | • HNSW-Arena RAM-Zeroisierung (`GhostFreeVectorIndex`)<br>• Vollständige Ring-Entkopplung (`contextra-core` eliminiert)<br>• 100% Grüne Gates für den Speicher- und Retrieval-Kern | **Woche 3–4** | `cargo xtask check-ring-layering-full`<br>`cargo xtask verify --tier kernel`<br>`cargo test -p contextra` |
| **M4: EP2 Contextra Proof** | **Phase 3–4 (K4)** | • 7-Schichten-Löschnachweis nach BSI TR-02102<br>• Standalone Verifier `contextra-proof-verify`<br>• KMS-Provider im Composition Root verdrahtet | **Woche 5–8** | `cargo test -p contextra-crypto`<br>`cargo xtask orphan-symbols` |
| **M5: EP3 Retrieval+ & EP4 Inference** | **Phase 5 (K5)** | • PathRAG budgetierte k-Pfad-Diffusion (`KPathDiffusion`)<br>• Irminsul Radix-Trie KV-Cache & KIVI 2-Bit Quantisierung<br>• Vollständige MCP- & Python-Fassade | **Monat 3–4** | `cargo test -p contextra-graph`<br>`cargo test -p contextra-kvcache`<br>`cargo xtask can-merge` |

---

## 4. Das ultimative Shell-Kompendium (Alle Befehle auf einen Blick)

Alle Befehle werden im Wurzelverzeichnis `/home/freddy/Projekte/contextra` ausgeführt.

### 4.1 Kompilieren & Build

```bash
# 1. Schneller Syntax- & Type-Check (nur Standard-Crates)
cargo check

# 2. Gesamten Workspace inkl. aller 33 Crates prüfen
cargo check --workspace

# 3. Contextra Facade mit Ollama-Unterstützung kompilieren
cargo build -p contextra --features ollama

# 4. Contextra Facade mit Candle & KV-Bridge kompilieren
cargo build -p contextra --features candle,kv-stage-b

# 5. Contextra MCP-Server kompilieren (für Claude / Cursor / CLI-Tools mit Ollama)
cargo build --release -p contextra-mcp --features ollama

# 6. Release-Build mit maximalen Optimierungen
cargo build --release
```

---

### 4.2 Testen & Beispiele ausführen

```bash
# 1. High-Speed Nextest Runner (empfohlen auf deinem Multi-Core Laptop!)
cargo nextest run

# 2. Lauffähige Beispiele ausführen (direkt zum Ausprobieren):
cargo run -p contextra-db --example quickstart       # Vektorspeicher & semantische Suche
cargo run -p contextra-db --example collections      # Multi-Collection Management
cargo run -p contextra-db --example hybrid_search    # Hybrides Retrieval mit Filtern

# 3. Gezielter Test einzelner Kern-Crates:
cargo test -p contextra-types          # Kerntypen & TenantId-Invarianten
cargo test -p contextra-store          # LSM-Tree, WAL, SSTables, Compaction
cargo test -p contextra-vector         # HNSW Vektor-Indexierung
cargo test -p contextra-mvcc           # SSI & Sequence-Log
cargo test -p contextra-crypto         # Ed25519 DeletionProof & AES-GCM
cargo test -p contextra-infer-ollama   # Ollama RAG-Client & Prompt Injection Evasion
cargo test -p contextra-kvcache        # Prefix-Radix-Tree & KV-Cache
cargo test -p contextra                # Facade & Integrations-Tests

# 4. Testausgaben mit stdout/tracing anzeigen
cargo test -p contextra -- --nocapture
```

---

### 4.3 Doktrin, Lints & Architektur-Gates (`cargo xtask`)

Contextra erzwingt strikte Architektur-Doktrinen (Zero-Panic, kein unbedachtes Unsafe, Ring-Hierarchie P5/P26).

```bash
# 1. Schneller Repository-Zustandsbericht (Dashboard)
cargo xtask repo-doctor --quick

# 2. Doktrin-Scan: Zero-Panic, unzulässige Unsafe-Blöcke, Allow-Durchbrechungen
cargo xtask doctrine-scan

# 3. Schneller Git-Diff Linter (vor Commits ausführen)
cargo xtask fast-diff-lint

# 4. Vollständige Ring-Layering- & DAG-Analyse (P5 / capabilities.toml)
cargo xtask check-ring-layering-full

# 5. Ring-0 Async-Reinheit (P26: kein Tokio in Ring 0!)
cargo xtask check-ring0-async-purity

# 6. Determinismus-Check (P28: Clock/Rng nur über injizierte Ports)
cargo xtask determinism-check

# 7. Unsafe-Inseln auditieren (Unsafe nur in deklarierten Inseln erlaubt)
cargo xtask check-unsafe-islands

# 8. Unwired Ports (sucht nach Traits ohne Implementierung)
cargo xtask unwired-ports

# 9. Clippy Linter über alle Targets mit strengen Deny-Regeln
cargo clippy --all-targets
```

---

### 4.4 Spezial-Prüfungen: Loom, Miri, Fuzzing & Mutation

```bash
# ── LOOM (Concurrency, Race-Conditions & Lock-Free Verifikation) ──────────
# 1. Scanner für unzulässige Loom-Mocks
cargo xtask check-loom-mocks

# 2. Loom Concurrency Runner
cargo xtask loom-run

# ── MIRI (Undefined Behavior & Speicher-Integrität) ────────────────────────
# Hinweis: Miri simuliert Rust-Code und prüft Pointer-Aliasing & Stacked Borrows
cargo +nightly miri test -p contextra-simd
cargo +nightly miri test -p contextra-crypto
cargo +nightly miri test -p contextra-sys

# ── FUZZING & MUTATION TESTING ───────────────────────────────────────────
# 1. Fuzz-Smoke über alle Crates mit Fuzz-Targets
cargo xtask fuzz-smoke

# 2. Diff-Mutation-Testing (erkennt ungetesteten Code im Diff)
cargo xtask mutants-diff

# ── CRASH-KONSISTENZ (WAL-Replay) ────────────────────────────────────────
cargo xtask wal-replay-verify
```

---

### 4.5 Benchmarks

```bash
# 1. Standard Criterion Benchmarks ausführen
cargo bench -p contextra-bench

# 2. Offizielles Benchmark-Gate (xtask-heavy mit Toleranzprüfung)
cargo xtask bench-gate --tolerance 0.05

# 3. Benchmark-Trendanalyse
cargo xtask bench-trend
```

---

## 5. Schritt-für-Schritt-Anleitung: Contextra v1 als RAG nutzen, um v2 zu entwickeln

Du möchtest Contextra nutzen, um die eigene Spezifikation und den Code zu indexieren, damit dir **Qwen (lokal via Ollama)** beim Entwickeln von Contextra v2 hilft.

Es gibt **zwei Wege**:
1. **Weg A (Sofort & Interaktiv):** Über den integrierten **Contextra MCP-Server** (kann mit jedem MCP-Client wie Cursor, Claude Desktop, Antigravity oder einem CLI genutzt werden).
2. **Weg B (Direkt in Rust / Standalone):** Ein 100% autarkes Dogfooding-Skript.

---

### Weg A: Der Contextra MCP-Server mit Ollama & Qwen

Contextra besitzt einen vollständig implementierten Stdio-MCP-Server in [`crates/contextra-mcp`](file:///home/freddy/Projekte/contextra/crates/contextra-mcp)!

#### Schritt 1: Stelle sicher, dass Ollama läuft
Prüfe im Terminal:
```bash
ollama list
```
Du hast bereits:
- `nomic-embed-text` (Vektor-Embeddings, 768 Dimensionen)
- `qwen2.5-coder:1.5b` (Code-Generierung & RAG)

#### Schritt 2: Starte den Contextra MCP-Server (mit `--features ollama`)
> [!IMPORTANT]
> Da `ollama` in `contextra-mcp` ein optionales Feature ist, **muss `--features ollama` zwingend übergeben werden**, damit der Provider einkompiliert wird:

```bash
cargo run --release -p contextra-mcp --features ollama --bin contextra-mcp-server -- \
  --provider ollama \
  --ollama-url http://localhost:11434 \
  --embed-model nomic-embed-text \
  --db-path ./target/contextra_rag_db \
  --allow-write
```

Der Server startet und lauscht über `stdio` auf JSON-RPC-Nachrichten. Er stellt folgende Tools bereit:
- `contextra_insert` / `contextra_upsert`: Dokumente mit automatischen Ollama-Embeddings speichern.
- `contextra_ingest_markdown`: Ganze Markdown-Dokumente automatisch chunking-gerecht indexieren!
- `contextra_search`: Hybride Suche (Vektor + Volltext) über den Wissensbestand.
- `contextra_forget`: Dokumente nach Mandant löschen.

#### Schritt 3: Den MCP-Server lokal testen und prüfen

1. **Integritäts- & Datenbank-Doctor:**
   ```bash
   cargo run -p contextra-mcp --bin contextra-mcp-cli -- doctor --db-path ./target/contextra_rag_db
   ```
2. **Schnelltest via Stdio JSON-RPC (Tools abfragen):**
   ```bash
   echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | cargo run -q -p contextra-mcp --features ollama --bin contextra-mcp-server -- --db-path ./target/contextra_rag_db
   ```
3. **Anbindung an Antigravity / Claude Desktop / Cursor (`mcpServers` Config):**
   ```json
   {
     "mcpServers": {
       "contextra": {
         "command": "cargo",
         "args": [
           "run", "--release", "-q", "-p", "contextra-mcp", "--features", "ollama",
           "--bin", "contextra-mcp-server", "--",
           "--provider", "ollama",
           "--ollama-url", "http://localhost:11434",
           "--embed-model", "nomic-embed-text",
           "--db-path", "/home/freddy/Projekte/contextra/target/contextra_rag_db",
           "--allow-write"
         ]
       }
     }
   }
   ```

---

### Weg B: Das Standalone-RAG Dogfooding Tool (Eigenbau in Rust)

Wenn du Contextra als Rust-Bibliothek direkt ausführen willst, nutzt du `contextra-db` und `contextra-infer-ollama`.

#### 1. Die bereits vorhandenen lauffähigen Beispiele testen:
Contextra enthält in `crates/contextra-db/examples/` bereits fertige Beispiele für Speicher, Suche und Collections:
```bash
# Basis-Vektorspeicher & Suche (4D-Vektoren, In-Memory/Disk):
cargo run -p contextra-db --example quickstart

# Multi-Collection Management:
cargo run -p contextra-db --example collections

# Hybride Suche mit Filtern:
cargo run -p contextra-db --example hybrid_search
```

#### 2. Vollständiger, lauffähiger Code für dein lokales RAG-Tool:

Hier ist der syntaktisch und typgetreue Code für dein eigenes Dogfooding-RAG-Tool (unter Nutzung von [`ContextraConfig`](file:///home/freddy/Projekte/contextra/crates/contextra-engine/src/lib.rs#L234), [`OllamaClient`](file:///home/freddy/Projekte/contextra/crates/contextra-infer-ollama/src/client/core.rs#L29) und [`build_rag_prompt`](file:///home/freddy/Projekte/contextra/crates/contextra-infer-ollama/src/client/validation.rs#L19)):

```rust
use contextra_db::{Contextra, ContextraConfig};
use contextra_infer_ollama::{build_rag_prompt, OllamaClient, OllamaConfig};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Contextra v1 Dogfooding RAG für Contextra v2 ===");

    // 1. Ollama-Client konfigurieren (nutzt nomic-embed-text + qwen2.5-coder)
    let ollama_config = OllamaConfig {
        base_url: "http://localhost:11434".to_string(),
        model: "qwen2.5-coder:1.5b".to_string(),
        embed_model: "nomic-embed-text".to_string(),
        timeout_secs: 60,
        ..Default::default()
    };
    let ollama = OllamaClient::with_config(ollama_config);

    // Health-Check gegen lokales Ollama
    if let Err(e) = ollama.health_check().await {
        eprintln!("⚠️ Ollama läuft nicht auf http://localhost:11434: {e}");
        return Ok(());
    }

    // 2. Contextra Storage Engine öffnen (768 Dimensionen für nomic-embed-text)
    let config = ContextraConfig {
        dimension: 768,
        consolidation_enabled: false, // Autarkes lokales RAG: kein Background-Consolidation-Worker erforderlich
        ..Default::default()
    };
    let db = Contextra::open_with_config("./target/contextra_dogfood_db", config).await?;
    println!("Contextra DB geladen. Dokumenten-Anzahl: {}", db.len().await?);

    // 3. Beispiel: Ein Kapitel aus der Spezifikation indexieren
    let doc_id = "spec:mvcc_ssi";
    let spec_chunk = r#"
    [INV-MVCC-SSI-1] Serializable Snapshot Isolation:
    ReadSet erfasst gelesene Keys und Range-Präfixe (Phantomschutz via record_prefix).
    SequenceLogSsiValidator erkennt Write-Skew-Konflikte gegen committete Writes nach snapshot_seq.
    Fail-closed: Reads mit snapshot_seq < pruned_through werden sofort abgewiesen.
    "#;

    // Vektor über Ollama (nomic-embed-text) berechnen
    let embedding = ollama.embed("nomic-embed-text", spec_chunk).await?;

    // Im Contextra-Speicher ablegen (HNSW Vektor-Index + LSM Storage)
    db.insert(
        doc_id,
        &embedding,
        Some(json!({
            "text": spec_chunk,
            "source": "docs/spec/v15_mvcc.md",
            "topic": "concurrency"
        })),
    ).await?;
    println!("Spezifikations-Auszug erfolgreich indexiert!");

    // 4. Eine Frage an dein lokales Qwen stellen
    let user_frage = "Wie funktioniert der Phantomschutz und Write-Skew-Schutz in Contextra?";
    println!("\nFrage an Qwen: {}", user_frage);

    // Relevante Chunks aus Contextra abrufen (Hybrid-Retrieval)
    let query_vector = ollama.embed("nomic-embed-text", user_frage).await?;
    let search_results = db.search(&query_vector, 3).await?;

    // Textinhalte aus Metadaten extrahieren
    let mut retrieved_contexts = Vec::new();
    for hit in &search_results {
        if let Some(text) = hit.metadata.as_ref().and_then(|m| m.get("text")).and_then(|t| t.as_str()) {
            retrieved_contexts.push(text);
        }
    }
    let combined_context = retrieved_contexts.join("\n---\n");

    // 5. RAG-Prompt mit XML-Escaping & Halluzinationsschutz bauen (3 Parameter)
    let system_context = "Du bist der leitende System-Architekt für Contextra v2.";
    let rag_prompt = build_rag_prompt(system_context, &combined_context, user_frage);

    // 6. Antwort von Qwen 2.5 Coder generieren
    println!("Generiere Antwort via Qwen 2.5 Coder (1.5b)...\n");
    let antwort = ollama.generate_text("qwen2.5-coder:1.5b", &rag_prompt).await?;
    println!("--- Qwen Antwort ---\n{}", antwort);

    // 7. Datenbank sauber schließen
    db.close().await?;
    Ok(())
}
```

---

## 6. Zusammenfassung & Nächste Schritte für dich

1. **Einmaliges Toolchain-Setup vervollständigen:**
   ```bash
   rustup component add --toolchain nightly-x86_64-unknown-linux-gnu miri
   cargo install cargo-audit
   ```
2. **Workspace-Kompilierung & Repo-Gesundheit prüfen:**
   ```bash
   cargo xtask repo-doctor --quick
   cargo check -p contextra -p contextra-mcp
   ```
3. **Behebungen gemäß Roadmap Meilenstein 1 (M1) umsetzen:**
   - Test-Fehler in `crates/contextra-store/tests/kv_crypto_shred_instant_unrecoverable.rs:34` beheben (`.expect("sub-key revoke")`).
   - N-B Hotfix in `crates/contextra-engine/src/collection/crud/delete.rs` (`ErasureBackend`-Port).
4. **RAG-Dogfooding starten:**
   Nutze Ollama (`qwen2.5-coder:1.5b` + `nomic-embed-text`) für den interaktiven RAG-Workflow zur Entwicklung von Release 1.0. Sobald Meilenstein 3 (EP1 Stable) erreicht ist, ist dein lokaler Speicher 100% crash- und transaktionssicher!

Damit hast du die volle Kontrolle über deinen Bare-Metal-Laptop und eine glasklare Roadmap bis zum stabilen Release 1.0!
