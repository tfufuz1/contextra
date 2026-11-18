# AGENTS.md — Contextra Workspace Richtlinie

**Status:** Normativ (Quelle der Wahrheit: `docs/SPEC.md` v1.0)
**Datum:** 18. November 2026
**Zielgruppe:** KI-Assistenten, autonome Agenten und Principal Engineers

---

## 1. Arbeitsanweisung & Geltung

Alle Agenten und Entwickler, die am Repository `contextra` arbeiten, sind strikt an die Festlegungen in `docs/SPEC.md` und den zugehörigen Crate-Spezifikationen in `docs/crates/` gebunden. Jedes Crate verfügt zusätzlich über eine lokale `AGENTS.md` im Crate-Verzeichnis, die crate-spezifische Regeln, Abhängigkeiten und Testpflichten definiert.

### Rangfolge der Dokumente:
1. `docs/SPEC.md` (Gesamtspezifikation — Single Source of Truth)
2. `docs/crates/*.md` (Spezifikationen pro Crate — normativ)
3. Diese `AGENTS.md` (Entwickler- & Agenten-Verhaltensrichtlinien)
4. `crates/*/AGENTS.md` (Crate-lokale Agentenrichtlinien)
5. `CLAUDE.md` (AI-Kurzreferenz)

---

## 2. Konsolidierte Architektur (23 Crates)

Der Workspace ist streng hierarchisch in vier Ringe/Ebenen gegliedert. Abhängigkeiten dürfen **ausschließlich abwärts** zeigen (E3 → E2 → E1 → E0).

```
┌──────────────────────── E3: Endpunkte ──────────────────────┐
│  mcp          testkit          bench                        │
└──────────────────────────┬──────────────────────────────────┘
                           │
┌──────────────────────── E2: Integration ────────────────────┐
│  contextra ──► db ──► engine (deprecated, = pub use db::*)  │
│  types        ports                                         │
└──────────────────────────┬──────────────────────────────────┘
                           │
┌──────────────────────── E1: Domäne ─────────────────────────┐
│  store    vector    text    graph    rank                    │
│  mvcc     crypto    checkpoint                              │
└──────────────────────────┬──────────────────────────────────┘
                           │
┌──────────────────────── E0: Fundament ──────────────────────┐
│  ids   env   sys   durable-fs   simd   privacy   erasure    │
└─────────────────────────────────────────────────────────────┘
```

### Abhängigkeitsregeln (R1–R6)

| Regel | Inhalt |
|-------|--------|
| **R1** | Kanten zeigen nur abwärts (E3→E2→E1→E0) |
| **R2** | Dev-Abhängigkeiten auf höhere Ebenen sind verboten |
| **R3** | Default-Features MÜSSEN leer sein |
| **R4** | Ein Konzept hat genau einen Besitzer — keine Typ-Dubletten |
| **R5** | `#![forbid(unsafe_code)]` in allen Crates außer `sys` und `simd` |
| **R6** | Kein `unwrap`, `expect`, `panic!`, `todo!` in Nicht-Test-Code |

### Entfernte Crates (Scope-Reset)

Die folgenden 13 Crates wurden zur Stabilisierung aus dem Workspace entfernt:
- `contextra-cognition`, `contextra-agent`, `contextra-adapt`
- `contextra-wire`, `contextra-py`, `contextra-prompt-guard`
- `contextra-ingest`, `contextra-infer-onnx`, `contextra-infer-ollama`
- `contextra-infer-candle`, `contextra-sandbox`, `contextra-router`
- `contextra-kv-cache`

> [!CAUTION]
> **Reaktivierungsverbot:** Kein entferntes Crate darf ohne Bestehen des formalen Gates `G-REACTIVATE` (siehe `docs/SPEC.md` §2.2) reaktiviert oder als Workspace-Member eingetragen werden. `contextra-vault` verbleibt dormant in `exclude`.

---

## 3. Datenklassifikation & DSGVO-Löschkette

### 3.1 Vier Datenklassen (SPEC §3)

| Klasse | Name | Verschlüsselung | Löschbar pro Betroffenen |
|--------|------|-----------------|-------------------------|
| **K** | Unternehmenswissen | Collection-KEK | Nein (nur Collection-Drop) |
| **P** | Personenbezogene Daten | SubjectKey → SubjectDocDEK | Ja, atomar O(1) via Crypto-Shredding |
| **D** | Abgeleitete Daten | Mit Lineage zum Quelldatum | Ja (kaskadiert über Lineage) |
| **E** | Löschmaterial | Separater ErasureKey | Nach Aufbewahrungsfrist |

**Regeln:**
- **K-Schreibpfade** akzeptieren nur `Validated<T>` aus `contextra-privacy`, um unerkannte PII auszuschließen
- **P-Daten** werden ausschließlich über `SubjectRef` (128-Bit-Zufall) adressiert, verschlüsselt mit individuellem `SubjectKey`
- **D-Daten** tragen verpflichtend `LineageRef` zum Quelldatum für kaskadierte Bereinigung
- **K-Daten** DÜRFEN NICHT `SubjectRef` enthalten — bei Personenbezug → Klasse P oder `PersonalRef`-Verweis

### 3.2 Die 6-Schritte-Löschsaga (`contextra-erasure`)

1. **Intent & Validierung** (fsync #1) — `RetentionAttestation` prüfen, Duplikaterkennung, Journal-Append
2. **Gate setzen** (fsync #2) — `CommitBarrier::exclusive()`, `ReadGate` aktivieren, alle Lesepfade liefern `Erased`
3. **Schlüsselvernichtung** (fsync #3) — Layer `on_gate`, DEK/KEK/SubjectKey mit Zufall+Null überschreiben, `verify_key_absent`. **Ab hier kryptographisch unlesbar!**
4. **Physische Bereinigung (Purge)** — Asynchron unter `PurgeBarrier`:
   - a) Snapshot-Widerruf (`mvcc`)
   - b) Text-Segment-Rewrite (`text`)
   - c) WAL-Drain & Memtable-Flush (`store`)
   - d) SSTable-Rewrite (`store`)
   - e) HNSW-Graph-Reparatur (`vector`)
   - f) Graph-Bereinigung (`graph`)
   - g) Slot-Bereinigung (`crypto`)
   - h) Cache-Invalidierung (alle Layer)
5. **Messung & Positivkontrolle** — `Scanner<Armed>` mit Canary-Muster, alle `LayerWitness.residue_matches == 0`
6. **Bericht & Ledger** — Ed25519-signierter Bericht, Hash-verkettetes Ledger außerhalb des Datenverzeichnisses

### 3.3 ReadGate-Invariante (SPEC §6)

Jeder Lesepfad MUSS synchron O(1) gegen das `ReadGate` prüfen. Bei `Erased` → sofortiger Abbruch, Fail-Closed:

| Lesepfad | Gate-Check-Stelle |
|----------|-------------------|
| `get`, `get_at_snapshot` | Vor Entschlüsselung |
| `scan`, `scan_prefix` | Je Eintrag vor Ausgabe |
| Vektor-/Textsuche | Auf Kandidatenebene vor Fusion/Rerank |
| Hybrid-Suche | Vor Fusion, mit Auffüllen auf `k` |
| Graph-Traversal | Je Kante und Zielknoten |
| Export | Je Eintrag; gesperrte werden ausgelassen |
| MCP-Tools | Über darunterliegende Pfade |
| Kompaktionen | `ContainsErased` → Range nicht in Ergebnis |

### 3.4 Schlüsselhierarchie (SPEC §4)

```
MasterKey (Passphrase → Argon2id → 256 Bit)
  ├── HKDF → TenantKEK(tenant_id)
  │            ├── HKDF → CollectionKEK(collection_id, epoch)
  │            │            ├── HKDF → DocumentDEK (Klasse K)
  │            │            └── HKDF → SubjectKey(subject_ref) → SubjectDocDEK (Klasse P)
  │            └── Epochenrotation bei Collection-Neuanlage nach Drop
  └── HKDF → ErasureKey (Journal, Ledger, Berichte)
```

**Algorithmen:** AES-256-GCM-SIV, HKDF-SHA-256, Argon2id (m=256MB, t=4, p=1), Ed25519 (Berichte), BLAKE3 (IDs)

---

## 4. Entwicklungsstandards & Invarianten

### 4.1 Code-Regeln

1. **Unsafe-Inseln:** `#![forbid(unsafe_code)]` im gesamten Workspace. Einzige Ausnahmen: `contextra-sys` (OS-Primitive) und `contextra-simd` (SIMD-Intrinsics). Jeder `unsafe`-Block benötigt `// SAFETY:`-Kommentar.
2. **Zero-Panic-Policy:** Keine Verwendung von `unwrap()`, `expect()`, `panic!()`, `todo!()` oder `unimplemented!()` in Nicht-Test-Code. Alle Fehlerpfade verwenden `Result` mit `ContextraError` bzw. crate-spezifischen Fehlertypen.
3. **Keine stillen Fallbacks:** Fehlt ein Schlüssel oder scheitert eine kryptographische Operation, MUSS ein Fehler zurückgegeben werden. Keine Fallback-Schlüssel, Null-Schlüssel oder deterministische Notfall-Passphrasen.
4. **Zeroize:** Sämtliches Schlüsselmaterial MUSS `ZeroizeOnDrop` implementieren. Kein `Clone`, kein `Debug`, kein `Serialize` für Schlüssel.
5. **Determinismus (INV-5):** Gleiche Eingaben → identische Such- und Ranking-Ergebnisse (Tie-Breaking nach `DocId` aufsteigend).
6. **Feature-Hygiene:** `default`-Features müssen in allen Crates leer sein.
7. **Fehlbare RNG:** `OsRng` über `try_fill_bytes` aufrufen. Fehler des Zufallsgenerators dürfen nie zu einem Chiffretext führen.

### 4.2 Workspace-Lints (Cargo.toml)

```toml
[workspace.lints.clippy]
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
todo = "deny"
unimplemented = "deny"
await_holding_lock = "deny"
indexing_slicing = "deny"
```

### 4.3 Invarianten-Katalog

| ID | Invariante |
|----|-----------|
| INV-1 | Kein persistiertes Byte ändert sich ohne Formatvermerk |
| INV-2 | Jeder öffentliche Pfad bleibt auflösbar |
| INV-3 | `forbid(unsafe_code)` außerhalb der Inseln (`sys`, `simd`) |
| INV-4 | Kein `unwrap`/`expect`/`panic!`/`todo!` in Nicht-Test-Code |
| INV-5 | Gleiche Eingabe → gleiche Rangfolge, Tie-Break ID aufsteigend |
| INV-6 | WAL, Manifest, Flush, Recovery-Verhalten unverändert bei Umbau |
| INV-7 | Keine Compliance-Aussagen. „Gelöscht" nur mit gemessener Stufe |
| INV-8 | Abhängigkeitskanten nur abwärts |
| INV-9 | `Completed` und `Refused` sind terminale Saga-Zustände |
| INV-10 | Gate wird nie zurückgenommen |
| INV-11 | Schlüsselbytes mit `zeroize` genullt vor Drop |
| INV-12 | Logs/Traces enthalten nie Klartextinhalte des Löschziels |

---

## 5. Crate-Verzeichnis & Zuständigkeiten

### E0 — Autarke Fundamente

| Crate | Zweck | Unsafe | Kern-Closure |
|-------|-------|--------|-------------|
| `contextra-ids` | Identitätstypen: `DocId`, `SubjectRef`, `PersonalRef`, `TenantId`, `ErasureId` | Nein | Leer |
| `contextra-env` | Zeitquellen (`Clock`), virtuelles Dateisystem (`Vfs`), Metriken | Nein | Leer |
| `contextra-sys` | OS-Primitive: `mlock`, `madvise`, I/O-Flags | **Ja** | Leer |
| `contextra-durable-fs` | Crash-sichere I/O: Handle-Poisoning, atomares Rename, Scrubbing | Nein | Leer |
| `contextra-simd` | SIMD-Distanzkerne: Cosine, L2, Dot (AVX2/AVX-512/NEON + Skalar) | **Ja** | Leer |
| `contextra-privacy` | `DataClass`-Enum, `Validated<T>`, PII-Erkennung, Egress-Schutz | Nein | Leer |
| `contextra-erasure` | DSGVO-Löschkern: Saga-Orchestrator, ReadGate, Vault, Messung | Nein | Leer |

### E1 — Autonome Domänenbausteine

| Crate | Zweck | Kern-Closure (max. 5) |
|-------|-------|-----------------------|
| `contextra-store` | LSM-Tree Storage Engine: WAL, MemTable, SSTables | `ids`, `env`, `sys`, `durable-fs`, `crypto` |
| `contextra-vector` | HNSW-Vektorindex: ANN-Suche, SQ8/SQ4-Quantisierung | `ids`, `env`, `simd`, `durable-fs` |
| `contextra-text` | Invertierter Volltextindex: BM25/BM25F, German Tokenizer | `ids`, `env` |
| `contextra-graph` | GraphRAG: Property-Graph, Traversal, Hyperedges | `ids`, `env` |
| `contextra-rank` | Fusion & Reranking: RRF, Score-Normalisierung, Tie-Breaking | `ids`, `env` |
| `contextra-crypto` | Kryptographische Primitive: AES-256-GCM-SIV, HKDF, Argon2id | `ids`, `env` |
| `contextra-mvcc` | Snapshot-Isolation: Pin-Verwaltung, Snapshot-Widerruf | `ids`, `env` |
| `contextra-checkpoint` | Checkpoints & Branching: Hardlink-basiert, Zero-Copy | `ids`, `env` |

### E2 — Integration & Orchestrierung

| Crate | Zweck |
|-------|-------|
| `contextra-types` | `ContextraError`, `EraseTarget`, `SagaState`, `RetentionAttestation` |
| `contextra-ports` | Port-Traits: `ErasureLayer`, `ReadGate`, `CommitBarrier`, `PurgeBarrier`, `LayerWitness` |
| `contextra-db` | Integrationsmotor: Multi-Index-Orchestrierung, Hybrid-Search, 2PC, Zeitreise |
| `contextra-engine` | **Deprecated** — `pub use contextra_db::*` Re-Export |
| `contextra` | Top-Level Façade: `Contextra::open`, High-Level API, Compliance-Profile |

### E3 — Endpunkte & Werkzeuge

| Crate | Zweck |
|-------|-------|
| `contextra-mcp` | MCP-Server (stdio/JSON-RPC 2.0): LLM-Integration, Lösch-Tools, Egress-Filter |
| `contextra-testkit` | Test-Infrastruktur: Fault-Injection, Canary-Orakel, synthetische Korpora |
| `contextra-bench` | Benchmark-Harness: Recall@K, Latenz-Profiling, Regressions-Gates |

---

## 6. Testkatalog (Löschkette)

| Kennung | Prüft |
|---------|-------|
| T-DEL-1 | Nach Insert+Flush: kein Klartext-Canary in Dateien |
| T-DEL-2 | Nach Löschung+Flush+Compaction: Lesepfad liefert `Erased` |
| T-DEL-3 | Löschnachweis schlägt mit falschem Schlüssel fehl |
| T-DEL-4 | Nach Löschung und Neustart bleibt Datum `Erased` |
| T-DEL-5 | `CryptoShred` führt Widerruf aus |
| T-DEL-6 | Weder Volltext- noch Vektorsuche findet gelöschtes Dokument |
| T-DEL-7 | Positivkontrolle: Scanner findet Canary |
| T-DEL-8 | Collection-Drop widerruft CollectionKEK |
| T-DEL-9 | Subject-Löschung: SubjectKey → alle P-Daten `Erased` |
| T-DEL-10 | Lineage-Kaskade: Quelldatum gelöscht → alle D-Ableitungen bereinigt |
| T-DEL-11 | ReadGate blockiert alle Lesepfade (§6.2 vollständig) |
| T-DEL-12 | Crash in Phase 3 → Recovery → Saga wird fortgesetzt |
| T-DEL-13 | Restrict-Modus: Lesezugriff nur mit Audit-Eintrag |

---

## 7. Konfiguration (SPEC §14)

```rust
pub struct ContextraConfig {
    pub data_dir: PathBuf,
    pub encryption: EncryptionConfig,      // Required { passphrase } oder None
    pub guard_policy: GuardPolicy,         // Required oder Optional
    pub data_class_policy: DataClassPolicy, // Enforced oder Legacy
    pub erasure: ErasureConfig,            // ledger_dir, batch_window_secs, disk_reserve_bytes
}
```

---

## 8. Glossar

| Begriff | Definition |
|---------|-----------|
| **Canary** | Bekanntes Byte-Muster zur Scanner-Kalibrierung |
| **Crypto-Shredding** | Schlüsselvernichtung macht Chiffretext unlesbar |
| **DEK** | Data Encryption Key — verschlüsselt ein einzelnes Payload |
| **Gate** | Synchrone Lesesperre gegen gelöschte Daten |
| **KEK** | Key Encryption Key — umhüllt DEKs einer Gruppe |
| **Lineage** | Herkunftsverweis von D-Daten auf ihr Quelldatum |
| **PersonalRef** | 128-Bit-Zufall als Verweis auf P-Daten in K-Dokumenten |
| **Saga** | Die 6-Schritte-Löschsequenz mit Journal und Recovery |
| **SubjectRef** | 128-Bit-Zufall zur Identifikation eines Betroffenen |
| **SubjectKey** | Aus CollectionKEK abgeleiteter Schlüssel für einen Betroffenen |
