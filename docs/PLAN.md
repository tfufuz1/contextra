# Contextra v1 – Umsetzungsplan (PLAN)

| Feld | Wert |
|---|---|
| Dokument | Verbindliche Umsetzungsspezifikation zur SPEC 5.1.0 (`docs/SPEC.md`, D-99) |
| Fassung | **5.1.0** (Release 1.0.0 Fahrplan) |
| Stand | 2026-11-03 |
| Normative Referenz | [`docs/SPEC.md`](SPEC.md) Version 5.1.0 (einzige Quelle der Wahrheit) |
| Status | Verbindlicher Arbeits- und Phasenplan für Entwicklung, Gates, Refactoring und Release |

> **Normative Bindung (D-99, T-TRACE-2):** Dieses Dokument ist strikt von [`docs/SPEC.md`](SPEC.md) abgeleitet. Es darf Verhalten, Datenformate, Sicherheitsanforderungen, Schnittstellen und Gates der Spezifikation weder ändern noch aufweichen. Jedes Arbeitspaket referenziert ausschließlich existierende Gate-, Test-, Invarianten- und Benchmark-IDs der Spezifikation 5.1.0.

---

## 1. Strategie und Architektur-Ziele

Contextra löst das fundamentale Dilemma moderner KI- und RAG-Systeme im europäischen Rechtsraum: **Rechtssicheres, messbares und auditierbares Löschen personenbezogener Daten (DSGVO Art. 17 & 18) als Kernarchitektur-Prinzip.**

### 1.1 Kernprinzipien der Umsetzung
1. **Drei-Säulen-Löschkette (D-01, SPEC §11):**
   - **Säule 1 (Gate & physischer Purge):** O(1)-Sofortsperre (`ReadGate`) für alle aktiven und historischen Versionen, gefolgt von physischem Rewrite ohne Ziel-Records.
   - **Säule 2 (Schlüsselvernichtung / Crypto-Erase, D-79, D-111):** 16-Shard-Tresor (`FileVault`) mit 64-Byte-Slots, atomarer Slot-Vernichtung (Status `0x02` mit MAC) und O(1)-Vernichtung von Zielschlüsseln (`T`, `C`, `S`, `D`), wodurch Dokumentspeicher und Vektor-Payload unlesbar und Textpostings unlinked werden.
   - **Säule 3 (Unabhängige Messung & kryptografischer Nachweis):** Verifikation auf Medium/Cache (`verified_via`, D-113), Erstellung signierter Berichte mit Ed25519 (`ReportSigner`, D-130) und transparenter Ausweisung von Restrisiken (Anhang D).

2. **Entkoppeltes Multi-Crate-Ökosystem (SPEC §5):**
   - 14 Produktions-Crates in strikter Ringhierarchie (Ring 0 bis 4) plus Tooling/Dev-Crates (`contextra-testkit`, `contextra-testmacros`, `contextra-lab`, `ctx-verify`, `tests/arch`).
   - Keine Zyklen, keine unerlaubten Kanten (AR-1 bis AR-14, SPEC §5.3).

3. **Deterministische Durability & Fail-Closed-Verhalten:**
   - Single-Node ohne Netzwerk und Server (VETO-SN01, SPEC HR-1).
   - RealVfs mit Handle-Poisoning bei I/O-Fehlern (DF-03) und strenger Startprobe für Dateisysteme (D-56, D-114).
   - Keine stillen Fallbacks oder korrupten Zustände (VETO-FAILCLOSED).

---

## 2. Gate-Reihenfolge (Normativ nach SPEC §15.3)

Die Gates müssen zwingend in folgender linearer Abfolge durchlaufen werden:
$$\text{G-A} \longrightarrow \text{G-B} \longrightarrow \text{G0} \longrightarrow \text{G-C} \longrightarrow \text{G-D} \longrightarrow \text{G-F} \longrightarrow \text{G-E} \longrightarrow \text{G-G} \longrightarrow \text{G-H}$$

Parallel laufende externe Prüfgates:
- **G-LEGAL (SPEC §6.7):** Fachanwaltliche Prüfung (Fragen 1 bis 17). Blockiert nie das technische Release, sondern limitiert Löschaussagen strikt auf SPEC §6.4 bis zur formalen Freigabe (D-103).
- **G-K (D-130, T-AES-REVIEW):** Externe Validierung der Kryptografie-Verwendung (Nonce-Schema, AAD, KDF-Schranken, Keyslots). Muss vor Freigabe von **G-H** grün sein.

| Gate | Kriterien & Prüfgegenstand | Verifizierende Tests / Belege |
|---|---|---|
| **G-A** | Kompilierbarkeit des gesamten Workspace: `cargo check --workspace --all-targets --locked` und `cargo fmt --check` grün (D-93). | T-BUILD-1, T-FMT-1 |
| **G-B** | Vollständiger Rückbau nicht spezifizierter Crates und Features (D-84, D-105); Bereinigung Tokio-Features (D-54); genau ein VFS-Trait (`ports::Vfs`), 0 direkte FS-Zugriffe außerhalb `durable-fs`. | T-ARCH-3, T-ARCH-4, T-CFG-1, T-UNSAFE-1 |
| **G0** | Speicherentscheid (SPEC §11.9): Prototyp mit D-70, D-71 und D-101 (Clusterschnitt); Messung von Write Amplification; Festlegung der Fallbackstufe mechanisch nach D-76. | BM-ERASE-WA, BM-ERASE-WA-TEXT |
| **G-C** | Code-Hygiene: 0 `unwrap`, 0 `expect`, 0 `panic`, 0 `let _ =` auf `Result`, 0 unzulässige `allow` im Produktionscode (D-107); `cargo clippy --workspace --all-targets -- -D warnings`; Snapshot-Sentinel- und Delete-Race-Fixes; Pflichtverschlüsselung (D-05). | T-LINT-2, T-LINT-3, T-ARCH-8, T-ZERO-1 |
| **G-D** | Löschkern S0 (`contextra-erasure`): Saga-Zustandsmaschine, Tresor-Sharding (D-111), Gate-Log, Replace (D-115), Supersede (D-117), Restricted (D-116), Messung MK1–MK3, Mutationstests ohne Überlebende. | T-ISO-*, T-GATE-*, T-VAULT-*, T-KEYCOV-*, T-FK-*, T-CRASH-*, T-RAW-*, T-PROOF-*, T-RESTORE-*, T-VERIFY-*, T-REPL-*, T-SUP-*, T-ERID-*, T-OBS-*, T-RESTR-*, T-EPOCH-1, BM-ERASE-LAT, BM-ERASE-O1 |
| **G-F** | Plattformintegrität & KDF: Keyslot-Hierarchie, `HEADER.a`/`.b`, Startprobe P1–P7 (D-56, D-114), Argon2id-Schranken (D-06). | T-KEYSLOT-*, T-HDR-*, T-KDF-*, T-PLAT-* |
| **G-E** | S1 Textindex (`contextra-text`): Segmentindex mit Block-AEAD (D-20), Analyzer `de/1` & `und/1` (D-81), Block-Max WAND, Rewrite im ersten Purge-Fenster (D-69, D-71). | T-TEXT-*, BM-TEXT, BM-ERASE-WA-TEXT |
| **G-G** | S2 Vektorindex (`contextra-vector`): Arena-Layout (D-86), Surrogat-Injektion vor `Gated` (D-77), Batch-Kantenreparatur, deterministischer Rebuild (D-122). | T-VEC-*, BM-VEC, BM-VEC-DEL, BM-VEC-REBUILD |
| **G-H** | Release-Reife (v1.0.0 = S0 + S1 nach Regelfall D-129 bzw. S0–S3 wenn G-G vor dem RC grün ist): Alle T-* und BM-*, T-TRACE-1/2, 72h-Soak-Test (T-SOAK-1, D-109), SBOM, signierte Artefakte, Dokumenten-Konsistenz. | T-SOAK-1, T-TRACE-1, T-TRACE-2, T-DOC-1 |

---

## 3. Ist-Zustand und Status pro Crate

| Crate | Ring | Status Ist-Zustand | Primäre Arbeitspakete & Aufgaben |
|---|---|---|---|
| `contextra-types` | 0 | Vorhanden, stabil, reine Datentypen. | Bereinigung Altbestand Sitzungsmarken, `DocRef`-Angleichung (D-70). |
| `contextra-ports` | 0 | Vorhanden, ZV-Ports bereinigt. | Löschports `ErasureObserver` (D-118), `ReportSigner` (D-130), `PreGate` (D-115) konsolidieren. |
| `contextra-sys` | 1 | Vorhanden, Unsafe-Insel gehärtet. | Prüfung `fsync`, `mlock`, `PR_SET_DUMPABLE`, Entfernung Rest-Dateizugriffe. |
| `contextra-simd` | 1 | Vorhanden, AVX2/AVX-512/NEON Kerne. | Skalare Referenz und Distanzkerne verifizieren; T-SIMD-1. |
| `contextra-durable-fs` | 1 | Vorhanden, RealVfs. | Startprobe P1–P7 (D-114), atomare Rename-/Sync-Operationen absichern. |
| `contextra-crypto` | 1 | Vorhanden, RFC-konforme Primitive. | Argon2id-Bounds (D-06), Slot-AAD (D-52), Nonce-Schema (D-60), abgeleitete Schlüssel (D-80). |
| `contextra-mvcc` | 2 | Vorhanden, SSI & SnapshotLease. | Snapshot-Registry nach `tenant_tag` (D-61), Sentinel `u64::MAX` strikt verboten (KF-11, D-127). |
| `contextra-store` | 2 | Vorhanden, LSM-Speicher. | WAL nie umschreiben (D-73), `WalFloor`, Manifest, SSTable-Clusterschnitt (D-101), Dateischlüssel (D-85). |
| `contextra-text` | 2 | Vorhanden, BM25-Segmente. | Analyzer `de/1` und `und/1` (D-81), Envelope-Bindung `doc_ord` unter DEK (D-69, D-121), Rewrite. |
| `contextra-vector` | 2 | Vorhanden, HNSW-Arena. | Surrogat vor `Gated` (D-77), Kantenreparatur, deterministischer Rebuild (D-122). |
| `contextra-rank` | 2 | Vorhanden, Festkomma-RRF. | RRF-Fusion mit $k=60$ (D-89), Tie-Breaker deterministisch nach `doc_tag`. |
| `contextra-erasure` | 3 | Skelett vorhanden. | Vollständiger Löschkern: 16-Shard-Tresor (`FileVault`, D-111), Saga, Gate, Journal, Messung, Report. |
| `contextra-engine` | 3 | Vorhanden, Orchestrierung. | Entkoppelung Fremdmodule, `TenantScopedStorage`, Lifecycle, Transaktionen, Suche mit Nachladen (D-62). |
| `contextra` | 4 | Vorhanden, reine Fassade. | Nur Re-Exporte und `system_defaults()` (D-89). Keine eigene Logik. |
| `contextra-testkit` | dev | Vorhanden. | `FaultVfs` mit deterministischer Crash-Matrix, In-Memory-Ports, Konformitäts-Suiten. |
| `contextra-testmacros`| dev | Vorhanden. | Attribute `#[ctx_test]` und `#[ctx_bench]` für statische Traceability (T-TRACE-1). |
| `contextra-lab` | dev | Vorhanden. | Unabhängige Prüfinstanz ohne Produktions-Decoder für MK1–MK3 (AR-4). |
| `ctx-verify` | dev | Vorhanden. | Standalone CLI-Verifizierer für Löschberichte und Feldscans (D-119). |
| `tests/arch` | dev | Vorhanden. | Architektur-, Ring-, Namens- und Lint-Validierung (T-ARCH-1..8, T-DOC-1). |

---

## 4. Arbeitspaket-Katalog (WP-0.1 bis WP-9.2)

### Phase 0: Governance & Build-Fundament (Gate G-A)
- **WP-0.1: Governance-Infrastruktur**
  - Umsetzung: `contextra-testmacros`, `ratchet.toml`, `tests/arch`-Grundprüfungen, T-TRACE-1.
  - Abnahme: T-TRACE-1, T-ARCH-5, T-ARCH-6.
- **WP-0.2: Toolchain- und Lieferketten-Sicherheit**
  - Umsetzung: Toolchain-Pinning nach D-82/D-104 (`rust-toolchain.toml` 1.97.0, MSRV 1.89), `cargo-semver-checks`, SBOM-Pipeline.
  - Abnahme: T-SUPPLY-1, T-SUPPLY-2.
- **WP-0.3: Workspace-Kompilierung & Build-Konsistenz (Gate G-A)**
  - Umsetzung: Bereinigung doppelter Manifest-Einträge in `contextra-store`, `Cargo.lock` konsolidieren, `mmap`-Reste auf `Vfs` umstellen.
  - Abnahme: `cargo check --workspace --all-targets --locked`, `cargo fmt --check` (Gate G-A).

### Phase 1: Rückbau & Dekomposition (Gate G-B)
- **WP-1.1: Rückbau archivierter Crates & ungenutzter Ports**
  - Umsetzung: Bereinigung toter Verweise auf 20 Alt-Crates (`graph`, `db`, `kvcache`, etc.), Entfernung der Ports `embedding`, `reranker`, `query_rewriter` (D-24).
  - Abnahme: T-ARCH-3, T-ARCH-4.
- **WP-1.2: Bereinigung toter Features und Plattform-Reste**
  - Umsetzung: Entfernung von `acl_win32.rs`, Windows-Pfade (D-84), Feature-Schalter wie `legacy-wal-key`, `memory-only-storage`, `cloud-egress-guard`.
  - Abnahme: T-CFG-1, T-UNSAFE-1.
- **WP-1.3: Abhängigkeiten & Tokio-Hygiene**
  - Umsetzung: Durchsetzung der Whitelist `governance/deps.toml` (D-105); Bereinigung Tokio-Features (D-54); `deny.toml` ohne ungeprüfte Ausnahmen.
  - Abnahme: T-ARCH-4, T-SUPPLY-1.
- **WP-1.4: Konsolidierung auf kanonisches `ports::Vfs`**
  - Umsetzung: `RealVfs` in `contextra-durable-fs` als einzige I/O-Schicht; Beseitigung direkter `std::fs`-Aufrufe in `store` und `vector`.
  - Abnahme: T-LINT-2, T-FSYNC-1.
- **WP-1.5: Architektur-Struktur & Namensregeln**
  - Umsetzung: Fassadenschnitt nach D-89, Namensregeln D-90 (`mod.rs`-Bereinigung, keine `util`/`helpers`), Zwecksätze D-97 in allen Crates, Bereinigung von Sitzungsmarken.
  - Abnahme: T-ARCH-7, T-ARCH-8, T-DOC-1.

### Phase 2: Code-Hygiene & Speicher-Entscheid (Gates G0 & G-C)
- **WP-2.1: Zero-Panic & Unsafe-Audit**
  - Umsetzung: Beseitigung aller `unwrap`, `expect`, `panic` im Produktionscode (D-107); `clippy -- -D warnings`.
  - Abnahme: T-LINT-3, T-ZERO-1 (Gate G-C).
- **WP-2.2: Speicherentscheid G0-Prototyp**
  - Umsetzung: Messung der Write Amplification bei Clusterschnitt nach D-101 (`BM-ERASE-WA`); mechanische Bestimmung der Fallbackstufe (D-76, D-101).
  - Abnahme: Gate G0-Protokoll, T-DOC-1.

### Phase 3: Fundament & Kryptografie (Gate G-F)
- **WP-3.1: Kanonische Typen & Ports**
  - Umsetzung: `DocRef`, `DocHandle`, `cluster_tag` (D-70); Löschtypen `LayerWitness`, `KeyCoverage`, `EraseTarget`; `erasure`-Crate im Workspace.
  - Abnahme: T-ARCH-1, T-DOCREF-1.
- **WP-3.2: Kryptografische Härtung**
  - Umsetzung: Argon2id Bounds (D-06), Keyslot-AAD (D-52), Nonce-Schema (D-60), abgeleitetes `K_sign` via `ReportSigner` (D-80, D-130).
  - Abnahme: T-KDF-*, T-KEYSLOT-*, T-ENC-*.
- **WP-3.3: LSM-Speicher & MVCC-Integration**
  - Umsetzung: WAL ohne Rewrite (D-73), `WalFloor` im Manifest, Dateischlüssel-Vernichtung Slot `F` (D-85), SnapshotLease mit Widerruf nach `tenant_tag` (D-61, D-127).
  - Abnahme: T-WAL-*, T-ISO-*, T-FK-*, T-SNAP-*.

### Phase 4: Löschkern S0 (Gate G-D)
- **WP-4.1: 16-Shard-Schlüsseltresor (`FileVault`)**
  - Umsetzung: 16 Shards mit 64-Byte-Slots (D-111, F.3), atomare Slot-Vernichtung (Status `0x02` mit MAC), abbrechbarer Rehash (`λ > 0.60`), Trennung von `data_dir` (D-112).
  - Abnahme: T-VAULT-1..9, BM-VAULT-REHASH, BM-ERASE-O1.
- **WP-4.2: Gate-Log, Saga & Protokolle**
  - Umsetzung: `ReadGate` Intervallbaum (D-10), Saga-Zustandsmaschine, `replace_erasing` (D-115), Berichtigung `update_with(..., Purge)` via `PrefixSeq` (D-117), `Restricted`-Pfad nach Art. 18 (D-116).
  - Abnahme: T-GATE-*, T-REPL-*, T-SUP-*, T-RESTR-*.
- **WP-4.3: Löschmessung & kryptografischer Nachweis**
  - Umsetzung: `Scanner<Armed>`, Verifikation am Medium (`verified_via`, D-113), Ed25519-Signatur, Ledger und Restore-Guard mit Bereichs-Tags (D-74, D-126).
  - Abnahme: T-VERIFY-*, T-PROOF-*, T-RESTORE-*, T-ERID-*.
- **WP-4.4: Unabhängige Verifikation (`contextra-lab`, `ctx-verify`)**
  - Umsetzung: Autonome Decoder für Formate nach Anhang F; Standalone-Verifikation von Berichten und Rohscans (MK1–MK3).
  - Abnahme: T-RAW-*, T-VER-*.

### Phase 5: Text- und Vektorindizes (Gates G-E & G-G)
- **WP-5.1: S1 Textindex (`contextra-text`)**
  - Umsetzung: Verschlüsselte Segmente mit Block-AEAD (D-20), `doc_ord`-Entkopplung unter DEK (D-69, D-121), Analyzer `de/1` & `und/1` (D-81), Segment-Rewrite im ersten Purge-Fenster.
  - Abnahme: T-TEXT-*, BM-TEXT, BM-ERASE-WA-TEXT (Gate G-E).
- **WP-5.2: S2 Vektorindex (`contextra-vector`)**
  - Umsetzung: Arena mit allen Repräsentationen (D-86), Surrogat-Injektion vor `Gated` (D-77), Batch-Kantenreparatur, deterministischer Rebuild (D-122).
  - Abnahme: T-VEC-*, BM-VEC, BM-VEC-DEL, BM-VEC-REBUILD (Gate G-G).
- **WP-5.3: Hybridsuche & Ranking (`contextra-rank`)**
  - Umsetzung: Festkomma-RRF ($k=60$), deterministischer Tie-Breaker nach `doc_tag`, Paginierung mit Nachladen (`ReadGate`-Filter, D-62).
  - Abnahme: T-HYB-1, T-SEARCH-2.

### Phase 6: Orchestrierung & Plattformhärtung (Gate G-F)
- **WP-6.1: Plattformprobe & Systemhärtung**
  - Umsetzung: Startprobe P1–P7 (D-56, D-114), `fsync_p50`-Plausibilität, `RLIMIT_CORE=0` via `contextra-sys`.
  - Abnahme: T-PLAT-1..2, T-OPS-3.
- **WP-6.2: Engine-Orchestrierung**
  - Umsetzung: `EngineBuilder`, `TenantScopedStorage`, Lifecycle, Hintergrund-Tasks (`vault_scrub`, Purge).
  - Abnahme: T-API-*, T-SOAK-1.

### Phase 7: Benchmarks & Robustheitsnachweis (Gate G-H)
- **WP-7.1: Benchmark-Harness & Basislinien**
  - Umsetzung: `ctx-bench` mit SPEC §14.8 Hardware-Metadaten, Erfassung aller BM-* Baselines.
  - Abnahme: BM-Reports im Evidenzbündel.
- **WP-7.2: Dauer- und Robustheitsnachweis (D-109)**
  - Umsetzung: 72h-Crash-Schleife mit `FaultVfs` (T-SOAK-1), Parser-Fuzzing (T-FMT-1), Mutationstests ohne überlebende Mutanten in `erasure` (T-MUT-1).
  - Abnahme: T-SOAK-1, T-MUT-1.

### Phase 8: Dokumentation & Release-Reife
- **WP-8.1: Dokumentation & Betreiber-Lieferdokumente**
  - Umsetzung: `README.md` nach SPEC §6.4, `docs/OPERATIONS.md`, Runbooks RB-1 bis RB-10, `docs/DSFA-BAUSTEIN.md` (SPEC Anhang G), `docs/SECURITY.md`.
  - Abnahme: T-DOC-1, T-LINT-1, T-LINT-4.
- **WP-8.2: Release-Abschluss (Gate G-H)**
  - Umsetzung: SBOM (CycloneDX), signierte Artefakte, Evidenzbündel (EV2–EV4), Kryptoprüfbericht G-K.
  - Abnahme: Gate G-H Freigabe.

---

## 5. Rückverfolgbarkeitsmatrix (Traceability nach T-TRACE-2)

| Arbeitspaket | Relevante SPEC-Entscheidungen | Beteiligte Crates | Zugeordnete Gates & Tests |
|---|---|---|---|
| **WP-0.1** | D-91, D-97, D-100, D-107 | `testmacros`, `tests/arch` | T-TRACE-1, T-ARCH-5, T-ARCH-8 |
| **WP-0.2** | D-82, D-104 | Workspace, CI | T-SUPPLY-1, T-SUPPLY-2, Gate G-A |
| **WP-0.3** | D-93, D-94 | `store`, `sys`, Workspace | T-BUILD-1, T-FMT-1, Gate G-A |
| **WP-1.1** | D-24, D-45, D-84 | `ports`, `engine`, `rank` | T-ARCH-3, T-ARCH-4, Gate G-B |
| **WP-1.2** | D-22, D-84 | `sys`, `types`, Workspace | T-CFG-1, T-UNSAFE-1, Gate G-B |
| **WP-1.3** | D-54, D-105, D-106 | Workspace, CI | T-ARCH-4, T-SUPPLY-1, Gate G-B |
| **WP-1.4** | D-56, D-84, D-94 | `durable-fs`, `store`, `vector`| T-LINT-2, T-FSYNC-1, Gate G-B |
| **WP-1.5** | D-88, D-89, D-90, D-91, D-97, D-98 | Alle Crates | T-ARCH-5..8, T-DOC-1, Gate G-B |
| **WP-2.1** | D-107 | Alle Produktions-Crates | T-LINT-3, T-ZERO-1, Gate G-C |
| **WP-2.2** | D-70, D-71, D-76, D-101 | `store`, `text` | BM-ERASE-WA, Gate G0 |
| **WP-3.1** | D-08, D-10, D-21, D-70, D-87, D-96 | `types`, `ports`, `erasure` | T-ARCH-1, T-DOCREF-1, Gate G-F |
| **WP-3.2** | D-06, D-52, D-60, D-80, D-130 | `crypto`, `ports` | T-KDF-*, T-KEYSLOT-*, T-ENC-*, Gate G-F |
| **WP-3.3** | D-61, D-73, D-85, D-127 | `store`, `mvcc` | T-WAL-*, T-ISO-*, T-FK-*, T-SNAP-*, Gate G-C |
| **WP-4.1** | D-79, D-111, D-112, D-114 | `erasure`, `crypto` | T-VAULT-1..9, BM-VAULT-REHASH, BM-ERASE-O1, Gate G-D |
| **WP-4.2** | D-10, D-115, D-116, D-117 | `erasure`, `engine` | T-GATE-*, T-REPL-*, T-SUP-*, T-RESTR-*, Gate G-D |
| **WP-4.3** | D-74, D-113, D-119, D-126, D-130 | `erasure`, `ports` | T-VERIFY-*, T-PROOF-*, T-RESTORE-*, Gate G-D |
| **WP-4.4** | D-109, AR-4, Anhang F | `lab`, `ctx-verify` | T-RAW-*, T-VER-*, Gate G-D |
| **WP-5.1** | D-20, D-69, D-71, D-81, D-121 | `text`, `crypto` | T-TEXT-*, BM-TEXT, BM-ERASE-WA-TEXT, Gate G-E |
| **WP-5.2** | D-77, D-86, D-122 | `vector`, `simd` | T-VEC-*, BM-VEC, BM-VEC-DEL, BM-VEC-REBUILD, Gate G-G |
| **WP-5.3** | D-62, D-89 | `rank`, `engine` | T-HYB-1, T-SEARCH-2, Gate G-G |
| **WP-6.1** | D-56, D-63, D-114 | `sys`, `durable-fs`, `engine` | T-PLAT-1..2, T-OPS-3, Gate G-F |
| **WP-6.2** | D-62, D-89, D-95 | `engine`, `contextra` | T-API-*, T-SOAK-1, Gate G-H |
| **WP-7.1** | D-109, SPEC §14.8 | `testkit`, Benchmarks | BM-*, Gate G-H |
| **WP-7.2** | D-109 | `testkit`, `erasure`, `store` | T-SOAK-1, T-MUT-1, Gate G-H |
| **WP-8.1** | D-67, D-98, D-110, SPEC §6.4 | Dokumentation, Runbooks | T-DOC-1, T-LINT-1, T-LINT-4, Gate G-H |
| **WP-8.2** | D-102, D-129, D-130, SPEC §17 | Release Artefakte | Gate G-K, Gate G-H |

---

## 6. Risikomanagement und Abweichungskontrolle

1. **Format- und Abwärtskompatibilität:**
   - Vor Release 1.0.0 gilt P3 (keine stillen Upgrades, explizite Migration). Alle Formatänderungen (Anhang F) werden direkt synchron mit `contextra-lab` und `ctx-verify` umgesetzt.
2. **Kompilierbarkeit und Schnittstellen:**
   - Bei Schnittstellenbrüchen gilt strikt: Kein Crate darf unvollständige Ports oder falsche Typen exportieren. Änderungen an Schnittstellen in `contextra-ports` ziehen zwingend synchrone Updates in den Implementierungs-Crates nach sich.
3. **Konfliktvermeidung mit SPEC:**
   - Jede Diskrepanz zwischen Code/Dokumentation und `docs/SPEC.md` ist per Definition ein Defekt des Codes oder des abgeleiteten Dokuments (D-98, D-99).
