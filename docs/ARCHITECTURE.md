# Contextra – Architektur-Dokumentation

| Feld | Wert |
|---|---|
| Stand | 31.10.2026 |
| Normative Referenz | [`docs/SPEC.md`](SPEC.md) (Fassung 2.4, Zielzustand v1.0) & [`PLAN.md`](../PLAN.md) |
| Ring-Definition | [`capabilities.toml`](../capabilities.toml) |
| Sicherheits-Doktrin | Single-Node, Safe Rust, Zero Panic, Fail-Closed, absolute Mandantentrennung (VETO-F10) |

---

## 1. Überblick & Grundprinzipien

Contextra ist ein lokaler, eingebetteter Wissens- und Gedächtnisspeicher in Rust. Das Gesamtsystem ist als strikter, azyklischer Graph (**DAG**) organisiert. 

### Architektur-Invarianten
1. **Strikte Schichtung (Ring 4 → 0):** Ein Crate in Ring $N$ darf nur von Crates in Ringen $\le N$ abhängen.
2. **Klassen-Schutz:** Kern-Crates (`[K]`) dürfen niemals von Rand- (`[R]`), experimentellen (`[E]`) oder eingefrorenen Crates abhängen.
3. **Single-Node-Doktrin:** Ein Prozess hält das Datenverzeichnis. Keine verteilten Knoten, kein Sharding, kein P2P, keine Netzwerk-Listener im Kern.
4. **Zero-Panic-Doktrin (P7):** Öffentliche APIs und Produktionspfade dürfen niemals paniken (`unwrap`, `expect`, `panic!`, ungeprüftes Indexieren sind verboten).
5. **P28 Determinismus:** Zeit, Zufall und IDs werden über Ports (`Clock`, `Rng`, `IdGen`) injiziert.
6. **WAL First (P24):** Zustandsänderungen werden vor der In-Memory-Modifikation dauerhaft im WAL synchronisiert (`fsync`).
7. **Absolute Mandantentrennung (VETO-F10):** Jede Abfrage und Datenstruktur erzwingt `TenantId` aus dem `RequestContext`.

---

## 2. Ringmodell (SPEC §4)

```
┌────────────────────────────────────────────────────────────────────────┐
│ Ring 4: Fassade & Schnittstellen                                       │
│ contextra (Composition Root) · contextra-mcp · [R] contextra-py        │
│ [E] contextra-license · [E] audit-export · [E] avv-generator           │
├────────────────────────────────────────────────────────────────────────┤
│ Ring 3: Orchestrierung                                                 │
│ contextra-engine · contextra-db · [R] contextra-privacy                │
│ [eingefroren: contextra-agent, contextra-cognition, contextra-graph]    │
├────────────────────────────────────────────────────────────────────────┤
│ Ring 2: Isolation & Inferenz                                           │
│ contextra-infer-ollama · [R] contextra-infer-candle*                   │
│ [eingefroren: contextra-infer-onnx*, contextra-sandbox*]               │
├────────────────────────────────────────────────────────────────────────┤
│ Ring 1: Persistenz                                                     │
│ contextra-store · contextra-checkpoint                                 │
├────────────────────────────────────────────────────────────────────────┤
│ Ring 0: Fundament & Verträge                                           │
│ contextra-types · contextra-ports · contextra-mvcc · contextra-crypto   │
│ contextra-vector · contextra-text · contextra-rank · durable-fs · core │
│ ─── Unsafe-Inseln (Ring 0) ───                                         │
│ contextra-sys · contextra-simd · contextra-wire                       │
├────────────────────────────────────────────────────────────────────────┤
│ Tooling & Benchmarks                                                   │
│ contextra-testkit · contextra-bench                                    │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Die 15 Kern-Crates (Klasse `[K]`)

| Crate | Ring | Unsafe | Aufgabe & Kernkomponenten |
|---|---|---|---|
| `contextra-types` | 0 | Nein | Kanonische Typen, `RequestContext`, Identitäten, `ContextraError`. |
| `contextra-ports` | 0 | Nein | Synchrone Verträge: `StorageEngine`, `Clock`, `Rng`, `IdGen`, `KmsProvider`, `Vfs`. |
| `contextra-sys` | 0 | **Ja** | Low-Level OS-FFI, `mmap_readonly`, Durability-Primitive (`durable_flush_file`, `durable_flush_dir`). |
| `contextra-simd` | 0 | **Ja** | AVX-512 / AVX2 / NEON Distanzkernel; Skalar-Referenz als Orakel. |
| `contextra-wire` | 0 | **Ja** | Zero-Copy FlatBuffers IPC und Serialisierungs-Layouts. |
| `contextra-mvcc` | 0 | Nein | Snapshot-Isolation, SequenceLog, TxBuffer, ReadSet, SSI-Validator. |
| `contextra-crypto` | 0 | Nein | AEAD, KDF (Argon2id/HKDF), KeyRegistry, RevocationLog, Ed25519, Proof-Ausstellung. |
| `contextra-vector` | 0 | Nein | HNSW-Index, SQ8-Quantisierung, gefilterte Vektorsuche, Bitset-Isolation. |
| `contextra-text` | 0 | Nein | BM25/BM25F-Volltextindex, Komposita-Zerlegung, Block-Max WAND. |
| `contextra-rank` | 0 | Nein | Reziproke Rangfusion (RRF in `fusion/`), Score-Normalisierung, Explain-Metadaten. |
| `contextra-store` | 1 | Nein | LSM-Tree, WAL, SSTables, Block-Cache, Compaction, KV-Segmentverschlüsselung. |
| `contextra-checkpoint` | 1 | Nein | Snapshots, Point-in-Time-Recovery, Restore-Guard, Backup-Export. |
| `contextra-engine` | 3 | Nein | Hybrid-Planer, Background-Reaper, ErasureSaga, Lesesperren, Lifecycle. |
| `contextra-db` | 3 | Nein | Collections-API, Multi-Tenant-Scoping, Transaktionsverwaltung. |
| `contextra` | 4 | Nein | **Hauptfassade & einzige Composition Root** (`ContextraBuilder`). |

---

## 4. Zerlegung in die Produktfamilie (PLAN §11)

Um monolithische Querkanten dauerhaft aufzulösen, ist das System in sieben disjunkte Produkte strukturiert:

1. **EP1 Memory (Kern):** Eingebettete, verschlüsselte Hybrid-Suche mit Gedächtnis-Lebenszyklus (`store`, `vector`, `text`, `engine`, `db`, `durable-fs`).
2. **EP2 Proof (Compliance):** Extern prüfbare Löschbeweise und DSGVO-Nachweise (`audit-export`, `avv-generator`, `verify`).
3. **EP3 Graph (Erweiterung):** Wissensgraph und GraphRAG (`graph` – temporär eingefroren).
4. **EP4 Inference (Adapter):** Lokale Inferenzanbindungen (`infer-ollama`, optional `infer-onnx`, `infer-candle`).
5. **EP5 Agent (Laufzeit):** Wiederanlauffähige, budgetierte Agenten-Laufzeit (`agent`, `router` – temporär eingefroren).
6. **EP6 Guard (Firewall):** Fail-Closed-Firewall für LLM-Ein- und Ausgänge mit PII-Erkennung und Sandbox (`privacy`, `sandbox`).
7. **EP7 Ingest (Pipeline):** Lokale Offline-Dokumentenextraktion (PDF, DOCX, EPUB, Mail, Markdown) zu kanonischen Text-Chunks mit Byte-Spans (`ingest`).

---

## 5. Entkopplung von Querkanten (PLAN §11.4)

Bestehende historische Querkanten werden gemäß `PLAN.md` wie folgt bereinigt:

- **Engine → Graph:** Gekoppelt über Trait-Port `GraphEnginePort`; Graph-Code wird hinter `#[cfg(feature = "graph")]` gelegt oder entkoppelt.
- **DB → Cognition:** Konsolidierungslogik wird über Lifecycle-Hooks abstrahiert; kein direkter Import im Standard-Build.
- **MCP → Router:** Dispatcher greift direkt auf die Fassade bzw. Engine zu, nicht hart auf das Router-Crate.
- **Store & Rank Dev-Dependencies:** Entfernung von Zirkelbezügen zu `graph`, `sandbox` und `adapt` in Tests.

---

## 6. Build- und Test-Infrastruktur

- Die Governance-Prüfung erfolgt über standardisierte Cargo- und Just-Workflows:
  - `just check`: Führt `cargo fmt`, `cargo clippy --all-targets` und `cargo check --workspace` aus.
  - `just test`: Führt die Testsuite mit `cargo test --locked` aus.
  - `just gates`: Führt den architektonischen Layering-Test (`tests/layering.rs`) aus.
- Keine versteckten Skripte: Alle Pfade und Gates werden direkt per Cargo geprüft.
