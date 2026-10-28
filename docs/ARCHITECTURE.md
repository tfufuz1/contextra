<!-- doc-stamp: 2026-10-28 1b8b790 -->

# Contextra – Architekturübersicht

> **Normative Quelle:** Dieses Dokument ist ein **Lese-Hilfsmittel** und kein normatives Dokument.
> Die alleinige normative Quelle für Architektur, Verhalten, Konstanten, Ringe, Klassen und Kantenregeln ist
> **`docs/SPEC.md` (Gesamtspezifikation Fassung 3.0, Baukasten-Modell)**. Bei jedem Widerspruch zwischen dieser Datei
> und `docs/SPEC.md` gilt `docs/SPEC.md`.
>
> Stand: **28.10.2026** | Basis: Gesamtspezifikation 3.0 + Umsetzungsplan 3.0 (HEAD 1b8b790)

---

## 1. Einleitung

Contextra ist ein **lokaler, eingebetteter Wissens- und Gedächtnisspeicher in Rust** (Single-Node,
ein Prozess hält ein Datenverzeichnis). Er speichert Dokumente, findet sie hybrid (Vektor + BM25 +
RRF-Fusion, mandantengetrennt) und löscht gemessen (physische Bereinigung über sechs Datenschichten,
Messung am Speicher, Löschbericht mit ausgewiesenen Grenzen).

Dieses Dokument beschreibt:
- das **Ring-Modell** mit allen Crates und ihren Klassen
- den **Crate-Katalog** mit LOC-Budget, Unsafe-Markierung und Umbauaufgaben
- die **Kantenregeln** (Ring-Disziplin P1/P5/P30)
- die **Unsafe-Inseln** (exakt drei)
- die **TODO-Liste** für ausstehende Archivierungs-Arbeitspakete (Phase 2)

Die Zuordnung von Ring und Klasse je Crate ist maschinenlesbar in [`capabilities.toml`](../capabilities.toml)
festgelegt (INV-DOC-SINGLE-SOURCE-1). Diese Datei ist die Single Source of Truth für Gates und Skripte.

---

## 2. Ring-Modell (SPEC.md §4.1)

Der Datenfluss ist unidirektional: **Ring 4 → Ring 3 → Ring 2 → Ring 1 → Ring 0**.
Eine Kante darf nur auf gleichen oder niedrigeren Ring zeigen (P1). Aufwärtskanten sind verboten (P5).
`[K]`-Crates dürfen nie von `[M]`/`[E]`-Crates abhängen (P30, INV-CORE-ISOLATION-1).

```
Ring 4 │ Fassade/Schnittstellen
       │  [K] contextra (Composition Root)
       │  [M] contextra-mcp
       │  [M] contextra-py
       │  [E] contextra-license
       │
Ring 3 │ Orchestrierung
       │  [K] contextra-engine
       │  [K] contextra-db
       │  [K] contextra-erasure (geplant, Ring 3)
       │  [M] contextra-privacy
       │  [M] contextra-cognition
       │  [M] contextra-router
       │  [M] contextra-agent
       │
Ring 2 │ Isolation / Inferenz
       │  [M] contextra-infer-candle
       │  [M] contextra-infer-onnx
       │  [M] contextra-infer-ollama
       │  [M] contextra-sandbox
       │
Ring 1 │ Persistenz
       │  [K] contextra-store
       │  [K] contextra-checkpoint
       │  [M] contextra-kvcache
       │
Ring 0 │ Fundament
       │  [K] contextra-types
       │  [K] contextra-ports
       │  [K] contextra-mvcc
       │  [K] contextra-crypto
       │  [K] contextra-vector
       │  [K] contextra-text
       │  [K] contextra-rank
       │  [M] contextra-graph
       │  [M] contextra-adapt
       │  [E] contextra-audit-export
       │  [E] contextra-avv-generator
       │  Unsafe-Inseln: [K] contextra-simd · [K] contextra-sys · [K] contextra-wire
       │
Werkzeug (kein Ring)
       │  contextra-testkit · xtask · bench / contextra-bench · *-fuzz
       │
Legacy / Auslaufend
       │  contextra-core (Kompatibilitätsfassade, entfällt in WP 2.8)
```

### Wirkung der Ringe

Das Ring-Gate erlaubt jede **Abwärtskante**. Ein Ringverstoß liegt vor, wenn eine Kante aufwärts
zeigt. Jedoch ist auch jede Abwärtskante `[K] → [R]/[E]/[A]` ein **Klassenverstoß** (P30), der
über `may_depend_on` in `capabilities.toml` und das Klassen-Gate durchgesetzt wird.

---

## 3. Normativitätsklassen

| Kürzel | Klasse       | Bedeutung |
|--------|-------------|-----------|
| `[K]`  | **Kern**    | Gilt uneingeschränkt für v1.0; alle Gates und Garantien gelten vollständig. |
| `[R]`  | **Rand**    | Gilt nur bei aktiviertem Feature (Release-Default **aus**); darf Kern-Garantien nie schwächen. |
| `[E]`  | **Experimentell** | Nicht normativ; kein Bestandteil des Versprechens. |
| `[A]`  | **Archiv**  | Aus dem Workspace entfernt (Tag `archiv/v0`), nicht normativ (SPEC.md Anhang B). |

Zielreife: `[K]` → `stable` (erst bei Evidenzstufe E4), `[R]`/`[E]` → `experimental` (SPEC.md §4.1).

---

## 4. Crate-Katalog mit LOC-Budget

Budgets sind **Code-Zeilen** (ohne Leer-/Kommentarzeilen, ohne Tests, Werkzeug `loc.py`,
SPEC.md §1.3). Sie dürfen nur sinken, außer eine Phasenabnahme nennt ausdrücklich Zuwachs.
IST-Werte stammen aus Commit `c4f7299b6` (SPEC.md §4.2) bzw. Scan-Commit `a547a42` (PLAN.md §3.1).

### 4.1 Ring 0 – Fundament

| Crate | Ring | Kl. | Unsafe | IST (LOC) | Budget v1.0 | Umbauaufgaben |
|---|---|---|---|---|---|---|
| `contextra-types` | 0 | K | nein | 2.611 | 2.700 | `RequestContext`, `LayerWitness`-Grundtypen, `ModelIdentity`, IDs, Fehlerklassen |
| `contextra-ports` | 0 | K | nein | 1.526 | 1.500 | Ports archivierter Crates entfernen, Mocks raus |
| `contextra-sys` | 0 | K | **ja** | 428 | 450 | Unsafe-Insel; SAFETY-Lücken schließen; Durability-Primitive, `mlock`, `madvise`, Dateisperren |
| `contextra-simd` | 0 | K | **ja** | 1.113 | 1.200 | Unsafe-Insel; Kernel `pub(crate)`, SAFETY, Miri; Abhängigkeit von `core` entfernen |
| `contextra-wire` | 0 | K | **ja** | 190 (+1.201 gen.) | 200 | Unsafe-Insel; Lint-`allow` auf generierte Datei beschränken |
| `contextra-mvcc` | 0 | K | nein | 1.585 | 1.600 | Zeitaufrufe (14) über `Clock`; SSI-Nachweis (T-SSI-1) |
| `contextra-crypto` | 0 | K | nein | 2.992 | 3.200 | AEAD, KDF, Signaturen, `KmsProvider`-Impl (K0/K1), Löschbericht V5, `EffaceableKeySlot`; I/O-frei |
| `contextra-vector` | 0 | K | nein | 6.701 | 6.800 | HNSW, SQ8, ACL-Bitset; DiskANN ins Archiv; `async` entfernen (Ratchet); `VectorScanner` |
| `contextra-text` | 0 | K | nein | 2.329 | 2.400 | BM25, Block-Max-WAND; Panic in `tokenizer.rs:30` entfernen; `TextScanner` |
| `contextra-rank` | 0 | K (reklass.) | nein | 1.709 (`fusion/`) | 1.750 | **Nur `fusion/`** behalten (RRF, Normalisierung, Explain); Rest ins Archiv |

### 4.2 Ring 1 – Persistenz

| Crate | Ring | Kl. | Unsafe | IST (LOC) | Budget v1.0 | Umbauaufgaben |
|---|---|---|---|---|---|---|
| `contextra-store` | 1 | K | nein | 13.066 | 13.400 | `Vfs`-Impl, `DurableLog`-Impl, Sidecars, Record-Umschlag, Purge-Kompaktion, `LsmScanner`; `core`-Abhängigkeit entfernen |
| `contextra-checkpoint` | 1 | K | nein | 1.630 | 1.700 | Spur D/K, Restore-Guard, `CheckpointScanner`; `core`-Abhängigkeit entfernen |

### 4.3 Ring 2 – Isolation / Inferenz (Rand)

| Crate | Ring | Kl. | Unsafe | IST (LOC) | Budget v1.0 | Umbauaufgaben |
|---|---|---|---|---|---|---|
| `contextra-infer-candle` | 2 | R | nein | 2.660 | 2.660 | Embedding-Backend; optional; im Release-Default aus |
| `contextra-sandbox` | 2 | R | nein | 1.105 | 1.105 | WASM-Isolierung; optional; im Release-Default aus |

### 4.4 Ring 3 – Orchestrierung

| Crate | Ring | Kl. | Unsafe | IST (LOC) | Budget v1.0 | Umbauaufgaben |
|---|---|---|---|---|---|---|
| `contextra-engine` | 3 | K | nein | 10.114 | 8.500 Ziel / 10.200 Grenze | Entkopplung von Archiv-Crates; Saga, Lesesperre, Boot; Stub-Löschung |
| `contextra-db` | 3 | K | nein | 585 | 600 | Entkopplung von `cognition`, `graph`, `adapt`; keine eigene Logik |
| `contextra-privacy` | 3 | R | nein | 1.331 | 1.331 | Egress-Filterung; optional; im Release-Default aus |

### 4.5 Ring 4 – Fassade / Schnittstellen

| Crate | Ring | Kl. | Unsafe | IST (LOC) | Budget v1.0 | Umbauaufgaben |
|---|---|---|---|---|---|---|
| `contextra` | 4 | K | nein | 729 | 800 | Composition Root, Boot B0–B9, Default `embedded`; Rand nur optional |
| `contextra-mcp` | 4 | K | nein | 3.314 | 3.400 | Drei Werkzeuge (`memory_remember`, `memory_recall`, `memory_forget`), Pipeline, IPC |
| `contextra-py` | 4 | R | nein | 1.426 | 1.426 | PyO3-Bindings; optional |
| `contextra-license` | 4 | E | nein | 307 | 307 | Lizenzprüfung; experimentell |
| `contextra-audit-export` | 4 | E | nein | 214 | 214 | Audit-Log-Export; experimentell |
| `contextra-avv-generator` | 4 | E | nein | 196 | 196 | AVV-Generator Art. 28 DSGVO; experimentell |

### 4.6 Werkzeug-Crates (kein Ring, kein Produktionscode)

| Crate | Aufgabe |
|---|---|
| `contextra-testkit` | Fixtures, Chaos-Harness, `FaultVfs`, `ManualClock`, Fake-Verwahrer; nur `dev-dependency` |
| `xtask` | Gates, CI-Skripte (31.991 Zeilen; **Schutzpfad**, keine Kürzung ohne Inventar) |
| `xtask-heavy` | Fuzz, Miri, Mutation, Chaos |
| `bench` / `*-fuzz` | Benchmarks und Fuzz-Targets |

---

## 5. Archiv-Crates

<!-- TODO(WP-2.1): Archiv-Tag `archiv/v0` auf den Stand nach WP 1.x setzen und folgende Crates
     aus dem Workspace entfernen. Diese Crates sind NICHT mehr Teil von Contextra v1.0.
     Kein anderes Crate darf neu von ihnen abhängen. -->

Die folgenden Crates werden per Git-Tag `archiv/v0` archiviert und danach aus dem Workspace entfernt
(PLAN.md §3.2, D-04). Gemeinsam umfassen sie ca. 25.000 LOC Produktionscode.

| Crate | LOC (Scan) | Grund der Archivierung |
|---|---|---|
| `contextra-graph` | 8.708 | Graph-Reasoning ist kein Ziel v1.0 (SPEC.md §2.3) |
| `contextra-cognition` | 3.191 | Kognition/Konsolidierung ist kein Ziel v1.0 |
| `contextra-adapt` | 2.193 | Online-Adaption ist kein Ziel v1.0 |
| `contextra-kvcache` | 2.434 | Persistenter Prefix-Cache (M1) archiviert (SPEC.md §2.2) |
| `contextra-router` | 1.465 | Router/Agentenlaufzeit ist kein Ziel v1.0 |
| `contextra-agent` | 2.413 | Agent-Pipeline ist kein Ziel v1.0 |
| `contextra-infer-onnx` | 952 | ONNX-Backend wird nicht ausgeliefert |
| `contextra-infer-ollama` | 1.661 | Ollama-Backend wird nicht ausgeliefert |
| `contextra-core` | 106 | Nur Kompatibilitätsfassade; entfällt (D-05) |
| `vector/diskann` | – | DiskANN ist kein Ziel v1.0 (§2.3) |
| `rank` ohne `fusion/` | ~1.263 | Kalibrierung, DiBuD, Drift: nicht Teil von Kern-`rank` |

> **Ist-Zustand (26.10.2026):** Alle oben genannten Crates befinden sich noch im Workspace
> (`Cargo.toml`). Die Archivierung beginnt mit WP 2.1 (nach Abschluss von Phase 1).

---

## 6. Unsafe-Inseln (exakt drei)

Nur die folgenden drei Crates dürfen `unsafe`-Code im Produktionspfad enthalten (SPEC.md §4.6, P2):

| Insel | Crate | Aufgabe |
|---|---|---|
| `simd` | `contextra-simd` | SIMD-beschleunigte Distanzberechnungen (AVX-512 > AVX2 > NEON > skalar) |
| `sys` | `contextra-sys` | Systemnahe Bindings, Low-Level-I/O, Durability-Primitive (`fdatasync`, `F_FULLFSYNC`) |
| `wire` | `contextra-wire` | FlatBuffers Wire-Protokolle, Zero-Copy-Deserialisierung |

**Regeln für alle Inseln:**
1. Die Insel exponiert ausschließlich eine **sichere** öffentliche API.
2. Jeder `unsafe`-Block trägt `// SAFETY:` mit Begründung.
3. Miri- und Property-Tests gegen eine sichere Referenzimplementierung.
4. **Alle anderen Crates** setzen `#![forbid(unsafe_code)]`; workspaceweit gilt `unsafe_code = "deny"`.
5. Generierter Code folgt P2; Generator und Version sind gepinnt; Ausgabe in der CI reproduzierbar.
6. Das Gate weist generierten und handgeschriebenen `unsafe`-Code getrennt aus.

---

## 7. Kantenregeln (Ring-Disziplin)

### P1 – Ring-Disziplin
Kanten (normal und build) zeigen ausschließlich auf gleichen oder niedrigeren Ring.

### P5 – DAG-Integrität
Keine Aufwärtskanten, keine Zyklen (normal + build, **mit aufgelösten Features**).
Dev-Aufwärtskanten nur als in `capabilities.toml` deklarierte Ausnahme.

### P30 – Kern-Isolation
`[K]`-Crates hängen weder normal noch build (im Release-Default-Feature-Set) von `[R]`/`[E]`/`[A]`-Crates ab.
Rand-Fähigkeiten erreichen den Kern ausschließlich über Ports, die die Fassade injiziert.

### Erlaubte Abhängigkeiten (strikt, SPEC.md §4.4)

| Crate | darf abhängen von |
|---|---|
| `types` | – |
| `ports` | `types` |
| `sys`, `simd` | `types` (nur falls nötig) |
| `wire` | `types` |
| `mvcc` | `types`, `ports` |
| `crypto` | `types`, `ports`, `sys` |
| `vector` | `types`, `ports`, `simd` |
| `text` | `types`, `ports` |
| `rank` | `types`, `ports` |
| `store` | `types`, `ports`, `mvcc`, `crypto`, `sys`, `wire` |
| `checkpoint` | `types`, `ports`, `crypto` |
| `engine` | `types`, `ports`, `mvcc`, `crypto`, `store`, `vector`, `text`, `rank`, `checkpoint` |
| `db` | `types`, `ports`, `engine` |
| `mcp` | `types`, `ports`, `wire`, `db`, `contextra` (nur Fassaden-API) |
| `contextra` | alle; `[R]`/`[E]` nur als `optional = true` |

Vollständige Regeln und verbotene Kanten: SPEC.md §4.4.

---

## 8. Reifegrad-Kriterien (SPEC.md §4.1)

`maturity = stable` setzt voraus:
1. Alle Sicherheits-Ports des Crates sind produktiv verdrahtet (INV-WIRING-2).
2. Alle Gates (SPEC.md §17.3) sind grün.
3. Fuzz-, loom- und Miri-Abdeckung nach SPEC.md §17.1.

Ein Crate mit ausschließlich Null-/Test-Implementierungen seiner Sicherheits-Ports gilt als `experimentell`.

---

## 9. Crate-Konzept-Karte (SPEC.md §0.3)

| Konzept | Crate (Ring) | Modul / Ort |
|---|---|---|
| IDs, Fehlerklassen, `RequestContext`, `LayerWitness`-Grundtypen | `types` (0) | – |
| Alle Traits (Ports) | `ports` (0) | SPEC.md §5 |
| Snapshots, SSI, Sequenzen | `mvcc` (0) | SPEC.md §8 |
| AEAD, KDF, Signatur, `KmsProvider`-Impl, Löschbericht, AuditChain, RevocationLog | `crypto` (0) | SPEC.md §12 |
| HNSW, SQ8, ACL-Bitset | `vector` (0) | SPEC.md §9 |
| BM25, Analyzer, Block-Max-WAND | `text` (0) | SPEC.md §10 |
| RRF-Fusion, Normalisierung, Provenienz/Explain | `rank` (0, nur `fusion/`) | SPEC.md §11.5 |
| SIMD-Kernel / OS-Primitive / Wire-Layout | `simd` / `sys` / `wire` (0, Unsafe-Inseln) | SPEC.md §4.6 |
| WAL, Memtable, SSTable, Manifest, Kompaktion, `Vfs`, Journal | `store` (1) | SPEC.md §7 |
| Interne Checkpoints, Restore-Guard | `checkpoint` (1) | SPEC.md §12.9, §12.17, §13.8 |
| Planner, Chunking, Löschsaga, Epoch Seal, Lesesperre, Startsequenz | `engine` (3) | SPEC.md §11, §12.8, §12.2.4 |
| Collections, Transaktionen, Embedded-API | `db` (3) | SPEC.md §13.9 |
| MCP-Server, IPC-Proxy, CLI | `mcp` (4) | SPEC.md §13 |
| Composition Root, öffentliche Rust-API | `contextra` (4) | SPEC.md §13.4 |

---

## 10. Locking-Disziplin

Vollständige, geordnete Sperrenhierarchie; keine Sperr-Guards über `.await`
(INV-LOCK-ORDER-1, `clippy::await_holding_lock = deny`). Details: SPEC.md §8.

---

## 11. TODO-Liste: Archivierungs-Arbeitspakete (PLAN.md Phase 2)

<!-- Diese Arbeitspakete sind noch offen und müssen in der angegebenen Reihenfolge abgearbeitet werden. -->

```
TODO(WP-2.1): Archiv-Tag `archiv/v0` auf Stand nach WP 1.x setzen.
              Folgende Crates werden zuerst aus `default-members` entfernt, nach Entkopplung
              aus dem Workspace:
              - contextra-graph
              - contextra-cognition
              - contextra-adapt
              - contextra-kvcache
              - contextra-router
              - contextra-agent
              - contextra-infer-onnx
              - contextra-infer-ollama
              - contextra-core
              - vector/diskann
              - rank ohne fusion/
              Vorher MIGRATION-EDGES.md aus `cargo metadata` erzeugen.

TODO(WP-2.2): `contextra-engine` von Graph/Adapt/KVCache/InferCandle entkoppeln.
              Je Kante ein Commit, `cargo check` nach jedem Schritt.
              Betroffene Module: kv_cache_integration/, extraction/, background_workers/,
              collection/search/hybrid/, collection/maintenance.rs, etc.

TODO(WP-2.3): `contextra-db`, `contextra-mcp`, `contextra` entkoppeln;
              `contextra-core` vollständig entfernen (286 Dateien referenzieren `contextra_core`
              — Migrationsskript mit Prüfung je Crate erforderlich).
              Default-Feature auf `embedded` setzen.

TODO(WP-2.4): Rand-Crates optional machen; Brücken über Ports sicherstellen.
              `contextra` darf Rand-Crates nur über `optional = true` einbinden.

TODO(WP-2.5): `capabilities.toml` auf Klassen (`class`-Feld) und strikte `may_depend_on`
              umstellen (WP 3.6 der Spec). Alle 66 nicht deklarierten Kanten bereinigen;
              Warnungen = Fehler setzen.
```

---

## 12. Referenzen

| Dokument | Zweck |
|---|---|
| [`docs/SPEC.md`](SPEC.md) | **Einzige normative Quelle** für alle Architektur-, Verhaltens- und Gate-Aussagen |
| [`capabilities.toml`](../capabilities.toml) | Single Source of Truth für Ring, Klasse, Reifegrad je Crate (INV-DOC-SINGLE-SOURCE-1) |
| [`AGENTS.md`](../AGENTS.md) | Agenten-Betriebsanleitung und Rangfolge der Informationsquellen |
| [`docs/PLAN.md`](PLAN.md) | Implementierungsplan IST → SOLL (Arbeitsunterlage, nicht normativ) |

> [!IMPORTANT]
> Verweise auf frühere Dokumente wie `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md`,
> `GESAMTSPEZIFIKATION.md`, `CONTEXTRA_SPEZIFIKATION.md` oder `spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md`
> sind **obsolet**. Diese Fassungen sind durch SPEC.md Fassung 2.3 vollständig ersetzt
> und dürfen nicht mehr herangezogen werden (SPEC.md §1.1).
