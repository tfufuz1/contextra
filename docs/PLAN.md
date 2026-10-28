# Contextra – Implementierungsspezifikation IST → SOLL (Baukasten-Fassung 3.0)

Stand 28.10.2026 · Basis-Commit `e2f7573f2` (Merge PR #5360) · Ziel: Gesamtspezifikation 3.0 (Zielzustand v1.0) · **Fassung 3.0: Baukasten-Architektur, Entkopplung über Ports & Features, Durchsetzung der Modulprinzipien M1–M8, Behebung der Befunde U-01 bis U-12.**

> **Zweck.** Dieses Dokument plant die schrittweise Transformation des Repositories zum entkoppelten Baukasten nach Gesamtspezifikation 3.0 (im Folgenden **Spec**). Die bisherigen Baukasten-Crates (`graph`, `cognition`, `router`, `agent`, `kvcache`, `infer-onnx`, `infer-ollama`, `adapt`) sind gem. ADR-D01 **aktive Baukasten-Module** und werden strikt über Ports, `MODULE.md`-Manifeste, isolierte Audit-Kommandos (`cargo xtask module-audit`) und Evidence-Packs entkoppelt und qualifiziert.

---

## 0. Belegbasis und Grenzen

| Quelle | Stufe |
| --- | --- |
| Spec 3.0 (`docs/SPEC.md`), vollständig gelesen | gelesen |
| Workspace-`Cargo.toml` (35 Mitglieder) & Crate-Manifeste am HEAD `e2f7573f2` | gemessen |
| `capabilities.toml` (26 `default-members`, Reifegrade `beta`/`experimental`) | gelesen |
| `SYSTEMATIC_CRATE_AUDIT.md` (Fassung 2.3, 28.10.2026) | Hypothese (E0), in Spec §1.3 (U-01 bis U-12) evaluiert |

**Grenzen:** In der aktuellen Prüfumgebung steht keine Rust-Toolchain bereit (`cargo` nicht verfügbar). Jede Zeilen- und Statusangabe basiert auf statischen Zählungen am Klon `e2f7573f2`. Das Arbeitspaket WP 0.1 beschafft die verifizierte CI-Build-Baseline.

---

## 1. Kernbefunde & Transformation nach SPEC 3.0

| ID | Befund (Spec §1.3, U-01 bis U-12) | Ursache | Maßnahme im Baukasten-Plan | WP |
| --- | --- | --- | --- | --- |
| U-01 | Reifegrad-Diskrepanz: Audit behauptet "Stable/Produktionsreif", `capabilities.toml` führt Crates als `beta`/`experimental`. | Reifegrade aus Audits statt Evidence-Packs. | Reifegrade werden nur über Evidence-Packs (`evidence/<crate>/`) und Gates vergeben (EP17). | WP 3.1 |
| U-02 | Panic in `text`: `tokenizer.rs:30` enthält `panic!` hinter `#[allow(clippy::panic)]`. | `OnceLock`-Regex-Init schlägt fehl. | Ersetzen durch panic-freie Fehlerbehandlung / `OnceLock<Result<Regex>>`. | WP 1.1 |
| U-03 | `vector`: `VectorScanner` fehlt; nur `HnswScanner` vorhanden. | Unvollständige S2-Abdeckung. | Implementierung von `VectorScanner` inkl. Snapshot- und Quantisierungscode-Messung. | WP 5.2 |
| U-04 | Löschkette unvollständig: Saga, `ErasedSet`, KeyTable, Epochen fehlen. | Fehlende Saga/Key-Module. | Erstellung des neuen Ring-3-Crates `erasure` (ADR-D04) & Entkopplung aller 10 Schichten. | WP 5.1–5.8 |
| U-05 | `contextra-core` hat 41 Verwendungen; Kompatibilitätsfassade nicht trivial. | Zweite Fassade im Workspace. | Schrittweiser Abbau: deprecate → Verwendungen umstellen → Crate entfernen (ADR-D06). | WP 2.8 |
| U-06 | Fehlende Baukasten-Entkopplung: harte Kanten `engine`→`graph`, `db`→`cognition`/`adapt`, `mcp`→`router`/`privacy`. | Direkte Crate-Importe statt Ports. | Umstellung aller Modul-Kanten auf Ports (`RelationIndex` etc.) & optionale Features. | WP 2.1–2.6 |
| U-07 | Stiller Netzverkehr: `infer-onnx` lädt Modeller per `reqwest` von HuggingFace herunter; `infer-ollama` nutzt HTTP. | VETO-NET1 / INV-NET-1 Verstoß. | Downloads nur hinter Egress-Policy & Feature `model-download`; Ollama nur Loopback. | WP 2.7 |
| U-08 | Unsafe-Inseln: `wire` nutzt `deny(unsafe_code)` und `.unwrap()` im FlatBuffers-Generat. | Generat-Gating unzureichend. | Isolierung des Generats, Panic-Scan & `#![forbid(unsafe_code)]` für handgeschriebenen Code. | WP 1.4 |
| U-09 | Workspace-`tokio` nutzt `features = ["full"]`. | Überblähter Build. | Minimierung der Tokio-Features je Crate auf das notwendige Minimum (ADR-D10). | WP 2.9 |
| U-10 | `default-members` enthält nur 26 von 35 Mitgliedern. | Unvollständige CI-Abdeckung. | Aufnahme aller 35 Mitglieder in isolierte Prüf-Pipelines & CI-Baseline. | WP 0.1 |
| U-11 | Fachfremde KV-Cache-Logik in `crypto` (`kv_cipher.rs`, `kv_segment/`, `kv_shredding.rs`). | Falsche Crate-Verantwortung. | Verlagerung aller KV-Cache-Krypto-Funktionen von `crypto` nach `kvcache`. | WP 2.5 |
| U-12 | `xtask` überbläht (~32.600 LOC); regex-basierte Prüfungen. | Hohes Wartungsrisiko. | Inventarisierung aller Gates, Meta-Tests und Rückbau auf wirksame Prüfungen (ADR-D09). | WP 3.6 |

---

## 2. Grundsatzentscheidungen (ADR-D01 bis ADR-D10)

| ID | Entscheidung | Auswirkung & Begründung |
| --- | --- | --- |
| **ADR-D01** | Graph, Kognition, Router, Agent, KV-Cache, ONNX, Ollama, Adapt sind **aktive Baukasten-Module**, nicht Archiv. | Aufhebung der alten Archivierungsentscheidung. Module werden entkoppelt, erhalten Verträge (`MODULE.md`) und eigene Evidence-Packs. |
| **ADR-D02** | Aufnahme neuer Module in den Baukasten über Modulvertrag (M1–M8) und Evidence-Pack. | Der Kern ist nicht mehr geschlossen; Baukasten wächst qualitätsgesichert. |
| **ADR-D03** | Entkopplung über Ports: Modul-Crates kennen `engine`/`db`/`store` nicht. | Jedes Modul ist isoliert baubar (`cargo build -p <crate> --no-default-features`) und testbar (M1, M2). |
| **ADR-D04** | Neues Crate `erasure` (Ring 3) für Löschsaga, `ErasedSet` und Berichtsbildung. | Trennung der Lösch-Orchestrierung von `engine`; `engine` schrumpft; Löschkette wird als eigenständiger Baustein prüfbar. |
| **ADR-D05** | Domänen-Ports (Graph, KV) wandern aus `ports` in die besitzenden Crates. | `contextra-ports` bleibt schlank und stabil (Ring 0 Fundament). |
| **ADR-D06** | `contextra-core` entfällt schrittweise (deprecate → migrieren → entfernen). | Eliminierung redundanter Fassaden und unklarer Kanten. |
| **ADR-D07** | Netz-Standard „aus“ (Egress-Tabelle, INV-NET-1, G8). | Datenschutz als Bauprinzip; Standard ist Loopback; externe Aufrufe nur mit expliziter Freigabe. |
| **ADR-D08** | Reifegrade entstehen ausschließlich aus Evidence-Packs; Audit-Berichte sind Hypothesen. | Verhindert falsche Reife-Behauptungen (U-01, EP17). |
| **ADR-D09** | `xtask` wird nach Wirksamkeit inventarisiert und schlank umgebaut. | Reduziert Komplexität von `xtask` (~32.600 LOC) auf belegbar wirksame Quality-Gates. |
| **ADR-D10** | `tokio` ohne `features = ["full"]`; Features je Crate minimal. | Beschleunigt Bauzeiten und minimiert Angriffsfläche. |

---

## 3. Ziel-Crate-Matrix & Modulverträge

### 3.1 Baukasten-Katalog (35 Workspace-Mitglieder)

| Crate | Ring | Familie | Klasse | Soll-Reife | Transitive Last (Soll) | Modul-Pflichten |
| --- | --- | --- | --- | --- | --- | --- |
| `types` | 0 | Fundament | `[K]` | `stable` | 0 | `RequestContext`, `DocId`, `TenantId`, SemVer-API, keine Panics |
| `ports` | 0 | Fundament | `[K]` | `stable` | 1 | Grundlegende Austausch-Traits, Domänen-Ports ausgelagert |
| `sys` | 0 | Fundament | Insel | `stable` | 1 | Durability-Primitive, `// SAFETY:`, Plattform-Matrix |
| `simd` | 0 | Fundament | Insel | `stable` | 1 | AVX2/AVX-512/NEON Kerne, Skalar-Fallback Standard |
| `wire` | 0 | Fundament | Insel | `stable` | 1 | FlatBuffers Layout, Generat-Panic-Scan, Fuzzing |
| `mvcc` | 0 | Speicher/Suche | `[K]` | `stable` | 2 | Snapshots, SSI-Validator, Write-Skew-Tests |
| `store` | 1 | Speicher/Suche | `[K]` | `stable` | 5 | WAL v3, LSM, Kompaktion, `LsmScanner`, Crash-Matrix |
| `checkpoint` | 1 | Speicher/Suche | `[K]` | `stable` | 4 | Restore-Guard, `CheckpointScanner`, keine `core`-Kante |
| `vector` | 0 | Speicher/Suche | `[K]` | `stable` | 5 | HNSW, SQ8, Slot-Nullung, `HnswScanner` / `VectorScanner` |
| `text` | 0 | Speicher/Suche | `[K]` | `stable` | 2 | BM25, Block-Max-WAND, Komposita-Zerlegung, Panic-frei |
| `rank` | 0 | Speicher/Suche | `[K]` | `stable` | 2 | RRF-Fusion (`fusion/`), Explain/Provenienz |
| `engine` | 3 | Speicher/Suche | `[K]` | `stable` | 11 | Planner, Chunking, Bounded-Compute, `graph` via Port |
| `db` | 3 | Speicher/Suche | `[K]` | `stable` | 12 | Embedded-API, Vault; keine direkten Rand-Kanten |
| `crypto` | 0 | Löschung/Nachweis | `[K]` | `stable` | 3 | AEAD, KDF, Ed25519, `DeletionProof` V5, AuditChain |
| `erasure` (neu) | 3 | Löschung/Nachweis | `[K]` | `beta` | 6 | Löschsaga, `ErasedSet`, Lesesperre, Planung |
| `audit-export` | 0 | Löschung/Nachweis | `[E]` | `experimental` | 2 | Art. 30 DSGVO Export, JSON-Schema |
| `avv-generator` | 0 | Löschung/Nachweis | `[E]` | `experimental` | 2 | Art. 28 DSGVO Vorlagen, versioniert |
| `infer-candle` | 2 | Inferenz | `[M]` | `beta` | 6 | Lokale GGUF-Inferenz, Mandanten-KV-Cache |
| `infer-onnx` | 2 | Inferenz | `[M]` | `experimental` | 5 | ONNX Embeddings, Offline-Modelle, Egress-Schutz |
| `infer-ollama` | 2 | Inferenz | `[M]` | `beta` | 4 | REST-Client, nur Loopback-Default |
| `kvcache` | 1 | Inferenz | `[M]` | `beta` | 4 | Quantisierter KV-Cache, Zeroize beim Drop, KV-Krypto |
| `adapt` | 0 | Inferenz | `[M]` | `beta` | 2 | PID-Latenzregler, Sherman-Morrison, Catoni, Bandit |
| `graph` | 0 | Agenten/Gedächtnis | `[M]` | `beta` | 3 | CSR-Graph, PageRank/PPR, `RelationIndex`-Port |
| `cognition` | 3 | Agenten/Gedächtnis | `[M]` | `beta` | 6 | Konsolidierung, Sleep-Cycle, Zugriff via Ports |
| `router` | 3 | Agenten/Gedächtnis | `[M]` | `beta` | 4 | Kontextueller Bandit, Drift-Erkennung, Fallback |
| `agent` | 3 | Agenten/Gedächtnis | `[M]` | `beta` | 10 | Orchestrator, State-Graph, DLQ, Checkpointing |
| `privacy` | 3 | Schutz/Schnittstellen | `[M]` | `stable` | 3 | PII-Klassifikation & Schwärzung, Egress-Gateway |
| `mcp` | 4 | Schutz/Schnittstellen | `[M]` | `beta` | 16 | MCP stdio Server, JSON-RPC Fuzzing, Werkzeuge |
| `py` | 4 | Schutz/Schnittstellen | `[M]` | `beta` | 12 | PyO3-Bindings, FFI-Panic-Boundary |
| `sandbox` | 2 | Schutz/Schnittstellen | `[M]` | `beta` | 1 | Wasmtime-Sandbox, Fuel-Limits, Escape-Tests |
| `license` | 4 | Schutz/Schnittstellen | `[E]` | `experimental` | 2 | Offline Ed25519-Lizenzprüfung via Port |
| `contextra` | 4 | Fassade | `[K]` | `stable` | 14 | Composition Root, Profile, Default `embedded` |
| `testkit` | – | Werkzeug | Werkzeug | `stable` | – | Fixtures, `ManualClock`, `FaultVfs`, Chaos-Matrix |
| `bench` | – | Werkzeug | Werkzeug | `stable` | – | Profil- & Crate-Benchmarks |
| `xtask` | – | Werkzeug | Werkzeug | `stable` | – | Modul-Audit, Quality Gates, Evidence-Packs |

---

## 4. Phasenplan & Arbeitspakete

### Phase 0 – Baseline & Governance (Sprint 1)
- **WP 0.1: Build- & CI-Baseline herstellen**
  - Einbindung aller 35 Workspace-Mitglieder in die CI. Erzeugung des Baseline-Evidence-Packs unter `evidence/baseline/`.
- **WP 0.2: Dokumentation & Requirements-Sync**
  - Überführung des historischen REQ-Katalogs nach `docs/spec/requirements.toml`.
  - Stempelung und Verifizierung aller Dokumente bezüglich SPEC 3.0.
- **WP 0.3: ADR-Dokumentation (ADR-D01 bis ADR-D10)**
  - Ausformulierung und Ablage der Richtrichtungsentscheidungen in `docs/decisions/`.

### Phase 1 – Sofortige Qualitäts- & Sicherheits-Fixes (Sprint 1–2)
- **WP 1.1: Panic-Eliminierung in `text` (U-02)**
  - Entfernung von `panic!` in `text/src/tokenizer.rs:30`.
- **WP 1.2: Entschärfung von Egress-Verstößen in `infer-onnx` (U-07)**
  - Entfernung des automatischen Modell-Downloads von HuggingFace.
  - Erzwingung lokaler, prüfsummengesicherter Modelledateien; Egress nur bei explizitem Feature `model-download`.
- **WP 1.3: HNSW-Slot-Messung in `vector` (U-03)**
  - Vervollständigung der HNSW-Slot-Nullung (INV-HNSW-SLOT-1) und Implementierung der Vollmessung.
- **WP 1.4: Unsafe-Generat-Isolierung in `wire` (U-08)**
  - Isolierung des FlatBuffers-Generats; `#![forbid(unsafe_code)]` auf handgeschriebenen Code in `wire` anwenden; Panic-Checks im Generat.

### Phase 2 – Baukasten-Entkopplung & Architektur-Schnitt (Sprint 2–4)
- **WP 2.1: Crate-Entkopplung `engine` → `graph`**
  - Auslagerung der direkten `graph`-Abhängigkeit in `engine/src/collection/relate.rs` auf den Port `RelationIndex`.
- **WP 2.2: Crate-Entkopplung `db` → `cognition`, `adapt`**
  - Entfernung harter Kanten; Re-Exports und Spezialfunktionen in Fassade/`cognition` verschieben.
- **WP 2.3: Crate-Entkopplung `mcp` → `router`, `privacy`, `crypto`, `rank`**
  - Umstellung harter Importe in `mcp` auf optionale Features und Fassaden-Re-Exports.
- **WP 2.4: Bereinigung von `contextra-ports`**
  - Verschiebung domänenspezifischer Ports (`graph.rs`, `kv.rs`, `attention.rs`) aus `ports` in die besitzenden Crates (`graph`, `kvcache`).
- **WP 2.5: KV-Cache Krypto-Refactoring (U-11)**
  - Verlagerung von `kv_cipher.rs`, `kv_segment/`, `kv_shredding.rs` aus `crypto` nach `kvcache`. `crypto` bleibt I/O- und domänenfrei.
- **WP 2.6: Entkopplung von `agent` & `cognition`**
  - Beseitigung direkter Zugriffe auf `db`/`store`/`vector`/`text` in `agent` und `cognition`; Zugriff ausschließlich über Ports.
- **WP 2.7: Egress-Sicherung für `infer-ollama` & Gateway**
  - Erzwingung von Loopback-Defaults und Host-Allowlists für REST-Aufrufe.
- **WP 2.8: Schrittweiser Abbau von `contextra-core` (U-05, ADR-D06)**
  - Umstellung der 41 Verwendungen im Workspace auf direkte Crate-Importe; Deprecation und finale Löschung von `contextra-core`.
- **WP 2.9: Tokio-Feature-Minimierung (U-09, ADR-D10)**
  - Entfernen von `features = ["full"]` im Workspace; Crate-spezifische, minimale Feature-Sets deklarieren.

### Phase 3 – Tooling, Gates & Evidence-Infrastruktur (Sprint 4–5)
- **WP 3.1: Isolierter Modul-Audit-Befehl**
  - Implementierung von `cargo xtask module-audit <crate>`.
- **WP 3.2: Evidence-Pack Automatisierung**
  - Automatische Generierung von `evidence/<crate>/` (`manifest.json`, `deps.json`, `api.txt`, `unsafe.json`, `panic.json`, `tests.json`, `depth.json`, `bench.json`, `claims.json`).
- **WP 3.3: Neue Modul-Gates (M1–M8)**
  - Implementierung und Aktivierung von `module-isolation`, `transitive-budget`, `no-cycles`, `may-depend-on`, `allow-register` (EP16), `egress-inventory`, `evidence-pack`, `claims-consistency` (EP17).
- **WP 3.4: Baukasten-Kompositionsprofile**
  - Implementierung von `cargo xtask profile-audit <profil>` für Search, Local-RAG, Agent, Compliance.
- **WP 3.5: Crate-Granulare Benchmarks**
  - Aufteilung von `contextra-bench` in Crate-interne Benchmarks unter `crates/<crate>/benches/`.
- **WP 3.6: `xtask`-Sanierung & Meta-Tests (U-12, ADR-D09)**
  - Inventarisierung aller ~32.600 LOC in `xtask`, Meta-Tests für Prüfskripte, Rückbau redundanter Skripte.

### Phase 4 – Neue Crates & Baustein-Vervollständigung (Sprint 5–6)
- **WP 4.1: Neues Ring-3-Crate `erasure` (ADR-D04)**
  - Implementierung des entkoppelten Löschketten-Orchestrators (`Löschsaga`, `ErasedSet`, Lesesperre, Berichtsbildung).
- **WP 4.2: Port `DurableLog` & Sidecar-Entkopplung**
  - Bereitstellung des `DurableLog`-Ports in `ports` für Sidecar-Logs in `crypto`.

### Phase 5 – Verifikation, Audit & Release 1.0 (Sprint 7)
- **WP 5.1: Modul-Verifikationsleiter (V1–V7)**
  - Durchlauf aller Verifikationsstufen für Kern- `[K]` und Rand-Module `[M]`.
- **WP 5.2: Profil-Verifikation & Beispiel-Anwendungen**
  - Erstellung lauffähiger Beispiele (`examples/<profil>/`) und deutschsprachiger Anleitungen für alle vier KMU-Profile.
- **WP 5.3: Vorbereitung des externen Krypto-Audits**
  - Aufbereitung des Evidence-Packs für `crypto` und `erasure` zur Vorbereitung der Zertifizierung (`certified-ready`).

---

## 5. Abnahmekriterien (DoD für Version 1.0)

1. **Grüne Baseline:** Alle 35 Workspace-Mitglieder bauen, testen und bestehen Clippy isoliert auf der CI.
2. **Modul-Isoliertheit:** `cargo xtask module-audit <crate>` besteht für jedes Crate im Baukasten (M1–M8 erfüllt).
3. **Egress-Konformität:** Kein Crate sendet unberechtigt Daten ins Netz; T-EGR-1 besteht.
4. **Soll-Graph erreicht:** Keine Zyklen, keine unerlaubten Kanten, transitive Last je Crate innerhalb des Budgets.
5. **Profile auditierbar:** Alle Baukasten-Profile (Search, Local-RAG, Agent, Compliance) bestehen `profile-audit`.
6. **Löschkette entkoppelt:** `erasure` steuert die Löschsaga; `DeletionProof` V5 und Mutationskatalog (G7) sind grün.
7. **Ehrliche Dokumentation:** README, Dokumente und Berichte stimmen exakt mit den Evidence-Packs überein (INV-CLAIMS-1, EP17).
