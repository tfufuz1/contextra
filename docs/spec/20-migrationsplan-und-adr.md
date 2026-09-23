---
source: CONTEXTRA_SPEC_v4_MASTER.md
chapter: "20"
---
## 20. Migrationsplan v2 und ADR-Übersicht (neu)

Dieser Abschnitt operationalisiert Teil A2 (§A2) und §4.2–§4.3. Er beschreibt, **wie** vom archivierten
Layer-0–5-Zustand (§4.0) in das Ring-Modell (§4.2) überführt wird — als Ergänzung, nicht als Ersatz der in §17/§18
beschriebenen Stabilisierungs- und Optimierungs-Roadmap. Beide Roadmaps laufen nebeneinander: §17/§18 adressiert
Korrektheit/Performance am bestehenden Code, §20 adressiert den Strukturumbau. Wo beide denselben Code betreffen,
gilt Teil A (Ground Truth vor Feature-Ausbau) vor §20 vor §17/§18.

### 20.1 Strangler-Regel (verbindlich)

Neue Crates entstehen **neben** den alten. Alte Crates (`contextra-core`, `contextra-db`, `contextra-candle`,
`contextra-ollama`, `contextra-embed`, `contextra-calibration`) re-exportieren die neuen Symbole mit `#[deprecated]`
für einen vollen Release-Zyklus, bis alle internen und die dokumentierte externe API (`cargo add contextra-db`,
§2.2) umgestellt sind. Der neue, öffentlich beworbene Fassaden-Crate heißt `contextra` (`cargo add contextra`);
`contextra-db` bleibt als Kompatibilitäts-Re-Export bestehen, bis der Deprecation-Zyklus abgeschlossen ist.

### 20.2 Phasenplan

Umfang: **S** < 1 Woche, **M** 1–3 Wochen, **L** > 3 Wochen (Schätzung nach betroffenen LOC, nicht kalibriert).
Jede Phase ist einzeln auslieferbar; das Exit-Kriterium ist zugleich das Merge-Gate.

| Phase | Umfang | Inhalt | Exit-Kriterium |
|---|---|---|---|
| **0R** Rest der Leitplanken | S | `contextra-py`-Panic-Test gegen das `maturin`-Release-Wheel; `contextra-bench` ohne `onnx`-Zwang (bereits umgesetzt); Root-Lints zuerst setzen, danach Vererbung in allen 20+ Crates; `unwrap`/`expect`/`panic`-Prod-Stellen (≈ 11) im selben PR beheben; `.unwrap-baseline.json`-Ratchet löschen; `deny.toml` erweitern (nicht neu anlegen); `tests/layering.rs` im Warnmodus; `audit.toml` bereinigen; `prefill_skip_count`-Fix (§9.2); `contextra-testkit`-Skelett (`ManualClock`, In-Memory-`StorageEngine`) | `cargo clippy --workspace --all-targets --locked -- -D warnings` grün mit den neuen Lints; 20/20 Crates erben `[workspace.lints]`; `cargo fetch --locked && cargo check --workspace --locked --offline` grün (Netzfreiheit über `default-members`, nicht über `--workspace`, §A2.1 D8); `cargo tree --workspace -e normal -i ort-sys` ohne Treffer; Layering-Test läuft (Warnmodus) |
| **1a** Aufwärtskanten auflösen | M | Fassade `contextra` + Builder entstehen; `contextra-db`/`contextra-engine` erhält `Arc<dyn Embedder>` statt eigener Backend-Konstruktion; `Weak`-Setter → `MetricsSink`/Ports; Kante `router → db` über Ports lösen | `cargo tree -p contextra-db -e normal` ohne `candle`, `ollama`, `embed`; Layering-Test scharf für Ring 3 |
| **1b** `core`-Zerlegung | M (≈ 10,3k LOC verschoben) | `contextra-core` → `contextra-types`/`contextra-ports`/`contextra-mvcc`; `contextra-core` wird `#[deprecated]`-Re-Export; Entscheidung über `saos.rs` (Löschkandidat) | Kein Crate importiert `contextra_core::` außer dem Re-Export-Test |
| **1c** Unsafe-Inseln zuerst | M | `contextra-sys`, `contextra-simd` extrahieren; `contextra-core-ipc-gen` → `contextra-wire`; `contextra-checkpoint` ohne globalen Zustand (P29) | `tests/unsafe_islands.rs` scharf; `contextra-vector`, `contextra-store`, `contextra-db`/`contextra-engine` tragen `#![forbid(unsafe_code)]` |
| **2** Sync-Kerne | L | `StorageRead` (sync) / `StorageWrite` (async) trennen; Persistenzaufrufe aus `csr.rs`, `inverted.rs`, `diskann.rs` in die Engine verschieben; begrenzter `ComputePool` statt unbegrenztem `spawn_blocking` | `cargo tree -e normal -p contextra-{vector,text,graph}` ohne `tokio`; Benchmark-p99 ≤ +3 % oder ≤ 2σ der vorher eingefrorenen Baseline |
| **3a** Konsistenz-Spike | S | ADR-N06-Prototyp: Crash-Injektion über `contextra-testkit`-Fault-VFS (≥ 10⁴ Läufe, deterministischer Seed) | Kriterien: rekonstruierte Indizes = Orakel; `visible_lsn` monoton; keine sichtbare Teilmenge eines Commits |
| **3b** `db` zerlegen | L | Zerlegung in `engine`/`cognition`/`rank`/`adapt`/`router`/`privacy`; acht (real sechs) `hybrid_search_*`-Varianten → `search(SearchRequest)`; `docid-128` entfernt (ADR-N05) | `contextra-db` nur noch Re-Export; Fassade `contextra` ≤ 20 `pub fn` (heute 50 in `contextra-db`) |
| **4** KV echt | L | Stufe A messen → optional B (eigenes Llama-Modell mit `KvState`) → optional C (Segment-Spill) | Gates gemäß §9.2-Tabelle; Spec-Status wechselt erst nach grünem Golden-Test von 🔴 auf 🟡/🟢 |
| **5** Hygiene | M | God-Files zerlegen (`csr`, `hnsw`, `fusion`, `sstable`, `compaction`, `diskann`, `ollama`-Client); Governance-Tag-Kommentare aus dem Quellcode; `xtask` auf < 3.000 LOC; `results/`-Verzeichnis (≈ 25 MB) aus dem Repo; Reifegrad-Marker und Feature-Katalog aus `capabilities.toml` generieren (P12) | Keine Datei > 1.000 Zeilen außerhalb Generat/Tests; alle Spec-Marker generiert, nicht handgepflegt |

### 20.3 ADR-Übersicht (Kontext · Entscheidung · Konsequenzen · Alternativen · Exit-Kriterium je ADR-Dokument unter `docs/decisions/`)

| ADR | Gegenstand | Status |
|---|---|---|
| N01 | Layer-Regeln maschinell (`cargo_metadata`-Test + `cargo-deny`-`wrappers`) | beschlossen, Warnmodus in Phase 0R, scharf ab 1a |
| N02 | Sync-Kern: `StorageRead` sync / `StorageWrite` async, begrenzter `ComputePool` | beschlossen; Benchmark-Gate vor Phase 2 |
| N03 | Drei Unsafe-Inseln (`sys`, `simd`, `wire`), Mechanik `deny`+`forbid` pro Crate | beschlossen; ersetzt die Sechs-Ausnahmen-Regel der Vorfassung vollständig |
| N04 | Panic-Profile: Root `unwind`, `release-abort` nur für Binaries ohne FFI | Profil bereits umgesetzt; Release-Wheel-Test (contextra-py) offen |
| N05 | Identifikatoren: externes `DocId` (128 Bit) + internes dichtes `DocIdx(u32)` (Option C) | Empfehlung, gekoppelt an N06; **offen** (§A2.4 Nr. 3) |
| N06 | Konsistenzmodell: WAL als einzige Wahrheit, abgeleiteter Zustand statt 2PC-Härtung (Option B) | Empfehlung, Spike-Gate; **offen** (§A2.4 Nr. 1) |
| N07 | KV in Stufen A/B/C, Segmentdateien statt LSM-Spill | Stufe A beschlossen; B/C nach Messung |
| N08 | Generierte Spec und Capability-Manifest (`capabilities.toml`, Testverweis pro Zeile) | beschlossen |
| N09 | Crate-Schnittkriterien I/U/C/S/D (P30) | neu, beschlossen |
| N10 | Kein globaler veränderlicher Zustand (P29) | neu, beschlossen |

### 20.4 Was aus der Vorfassung unverändert beibehalten wird

Um Missverständnisse auszuschließen: Diese Überarbeitung ist ein **Struktur**-Umbau, kein Widerruf der
bewährten Kern-Invarianten. Unverändert bleiben: WAL-First mit HMAC-Kette (P2), MVCC, RCU-Snapshots im
CSR-Graph, Tenant-Isolation, Sandbox mit getrennten Fuel-/Wall-Clock-Budgets (P23), deterministische Recovery
(P3), und die DAG-Doktrin selbst (P5) — sie war immer richtig, sie wurde nur bislang nicht maschinell
erzwungen, was diese Fassung nachholt.

---


---

<a id="21-sota"></a>
