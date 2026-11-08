# Historie-, Refactoring- und Cherry-Pick-Audit (Github History & Repository)

Dieses Dokument dokumentiert die lückenlose Rückverfolgbarkeit (Traceability), forensische
Analyse aller Commit-Zustände von Anfang bis heute, die Identifikation des jeweils reifsten
Zustands entfernter und refaktorisierter Funktionen sowie die systematische Katalogisierung
aller 'Shell-Commits' und leeren Commits im GitHub-Verlauf.

---

## 1. Executive Summary & Metriken

- **Gesamtzahl analysierter Commits im Hauptzweig:** 5232
- **Identifizierte 'Shell-Commits':** 241 (vollständig analysiert und mit präzisen Botschaften versehen)
- **Identifizierte leere Commits:** 330 (Tree identisch mit Parent)
- **Forensisch erfasste Modul-Lebenszyklen:** 14

---

## 2. Reifegrad-Matrix entfernter und refaktorisierter Module & Funktionen

Im Zuge der Transformation des Monolithen zur modularen Baukasten-Architektur wurden diverse
Komponenten entkoppelt, refaktorisert oder entfernt. Die folgende Matrix erfasst den jeweils
**reifsten Zustand** (letzter voll funktionsfähiger Stand vor Ausdünnung/Löschung) sowie den
**Schnitt-/Entfernungs-Commit** für nahtlose Wiederherstellung, Inspektion und Cherry-Picks:

| Modul / Funktion | Pfad | Reifster Zustand (Hash) | Reifster Commit | Entfernungs-/Schnitt-Commit | Zweck / Beschreibung |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **contextra-core** | `crates/contextra-core` | `15755fe93` | Merge pull request #4885 from tfufuz1/jules-15928893261243544682-b5b0201b (2026-10-16) | `24862f3c7` ([Entfernung EP1] contextra-core Dateibaum (Strangler-Fassade) (2026-10-16)) | Legacy monolithic core facade, predecessor to decoupled building-block crates |
| **contextra-db** | `crates/contextra-db` | `9cbcf3a78` | Merge pull request #4855 from tfufuz1/jules-w4-01-decouple-inventory-protocol-17519155072974042073 (2026-10-15) | `084b3c7ff` ([Entkopplung contextra-db] Decouple contextra-db dependencies and features (2026-10-15)) | Legacy High-Level Database Wrapper & Session Manager |
| **contextra-privacy** | `crates/contextra-privacy` | `fa8b6a30c` | Merge pull request #4845 from tfufuz1/decouple-store-contextra-core-8951227313492383209 (2026-10-15) | `136298b5e` ([Entfernung EP6] Delete crates/contextra-privacy and crates/contextra-sandbox (2026-10-15)) | GDPR / AVV / Auftragsverarbeitung und Löschungs-Metriken |
| **contextra-graph** | `crates/contextra-graph` | `15755fe93` | Merge pull request #4885 from tfufuz1/jules-15928893261243544682-b5b0201b (2026-10-16) | `d22665d74` ([Entfernung EP3] contextra-graph Dateibaum (2026-10-16)) | GraphRAG, PPR (Personalized PageRank), CSR Matrix, Hyperedge Relate |
| **contextra-cognition** | `crates/contextra-cognition` | `15755fe93` | Merge pull request #4885 from tfufuz1/jules-15928893261243544682-b5b0201b (2026-10-16) | `923636929` ([Entfernung EP5] contextra-cognition Dateibaum (2026-10-16)) | Gedächtnis-Konsolidierung, Memory-Compaction & Background Maintenance |
| **contextra-adapt** | `crates/contextra-adapt` | `15755fe93` | Merge pull request #4885 from tfufuz1/jules-15928893261243544682-b5b0201b (2026-10-16) | `c234dc3f9` ([Entfernung EP5] contextra-adapt Dateibaum (2026-10-16)) | Adapter & Brücken für externe Speicher und Frameworks |
| **contextra-agent** | `crates/contextra-agent` | `15755fe93` | Merge pull request #4885 from tfufuz1/jules-15928893261243544682-b5b0201b (2026-10-16) | `e64df1dc8` ([Entfernung EP5] contextra-agent Dateibaum (2026-10-16)) | Autonome Agenten-Loop, Scratchpad & Goal Reasoning |
| **contextra-router** | `crates/contextra-router` | `15755fe93` | Merge pull request #4885 from tfufuz1/jules-15928893261243544682-b5b0201b (2026-10-16) | `35d588b6c` ([Entfernung EP5] contextra-router Dateibaum (2026-10-16)) | Query Router, semantic query classifier & multi-index dispatcher |
| **contextra-license** | `crates/contextra-license` | `15755fe93` | Merge pull request #4885 from tfufuz1/jules-15928893261243544682-b5b0201b (2026-10-16) | `6a5d12665` ([Entfernung EP2] contextra-license Dateibaum (2026-10-16)) | Signed license gate, hardware fingerprinting, installation binding |
| **contextra-sandbox** | `crates/contextra-sandbox` | `fa8b6a30c` | Merge pull request #4845 from tfufuz1/decouple-store-contextra-core-8951227313492383209 (2026-10-15) | `136298b5e` ([Entfernung EP6] Delete crates/contextra-privacy and crates/contextra-sandbox (2026-10-15)) | Wasmtime execution engine, resource limiting & sandbox isolation |
| **contextra-py** | `crates/contextra-py` | `fa8b6a30c` | Merge pull request #4845 from tfufuz1/decouple-store-contextra-core-8951227313492383209 (2026-10-15) | `eba5c8b41` ([Entfernung EP1] contextra-py entfernen (2026-10-15)) | Python PyO3 bindings & zero-copy vector transfers |
| **xtask** | `xtask` | `73c64919a` | feat(xtask): wire 19 xtask modules in main.rs, update justfile and bootstrap docs (#3880) (2026-09-28) | `068261abd` (Shell-Commit (2026-10-30)) | Workspace governance, linting, forensic checkers, gate matrix |
| **deletion_proof** | `crates/contextra-crypto/src/deletion_proof.rs` | `cc4dfa5ae` | contextra-crypto: harden DeletionProof::issue with V5 verified witness measurement binding (2026-10-28) | `0a0f49ba0` (Delete crates/contextra-crypto/src/deletion_proof.rs (2026-10-29)) | Cryptographic Deletion Proof (BSI TR-02102 / Ed25519 & Falcon) |
| **hnsw_purger** | `crates/contextra-vector/src/hnsw/purger.rs` | `e343f32bf` | contextra-vector: Implement HNSW LayerScanner and Purger [LÖSCHKETTE W2-P01] (2026-10-28) | `7963b3363` (refactor(contextra-vector): modular decoupling and isolated building-block architecture Baukasten-Refaktorisierung für contextra-vector (Paket B12): (2026-11-07)) | HNSW Vector Purger & LayerScanner für graph-basierte Vektorlöschung |

---

## 3. Detaillierter Leitfaden für Cherry-Picks & Rekonstruktion

### `contextra-core` (crates/contextra-core)
- **Aufgabe & Umfang:** Legacy monolithic core facade, predecessor to decoupled building-block crates
- **Reifster Commit (Mature State):** `15755fe936a64293bdead2aa6a79e2d0433dc45a`
  - Titel: *Merge pull request #4885 from tfufuz1/jules-15928893261243544682-b5b0201b (2026-10-16)*
  - Notiz: Reifster Stand vor Ausdünnung/Löschung; enthält frühere Monolith-Typen und traits
- **Entfernungs-/Schnitt-Commit (Removal State):** `24862f3c7e27acc3e80aaa58a97d095f0be36b8a`
  - Titel: *[Entfernung EP1] contextra-core Dateibaum (Strangler-Fassade) (2026-10-16)*
- **Befehl für selektiven Cherry-Pick:**
  ```bash
  git cherry-pick -n 15755fe936a64293bdead2aa6a79e2d0433dc45a
  ```
- **Befehl zum partiellen Auschecken des Moduls:**
  ```bash
  git checkout 15755fe936a64293bdead2aa6a79e2d0433dc45a -- crates/contextra-core
  ```

### `contextra-db` (crates/contextra-db)
- **Aufgabe & Umfang:** Legacy High-Level Database Wrapper & Session Manager
- **Reifster Commit (Mature State):** `9cbcf3a78fb7d3e8917d5b7533f62c0366bc278d`
  - Titel: *Merge pull request #4855 from tfufuz1/jules-w4-01-decouple-inventory-protocol-17519155072974042073 (2026-10-15)*
  - Notiz: Vollständige DB-Fassade inkl. Session-Management vor Entkopplung
- **Entfernungs-/Schnitt-Commit (Removal State):** `084b3c7ffce1b314b7eb9f448b86c53dcbb12a59`
  - Titel: *[Entkopplung contextra-db] Decouple contextra-db dependencies and features (2026-10-15)*
- **Befehl für selektiven Cherry-Pick:**
  ```bash
  git cherry-pick -n 9cbcf3a78fb7d3e8917d5b7533f62c0366bc278d
  ```
- **Befehl zum partiellen Auschecken des Moduls:**
  ```bash
  git checkout 9cbcf3a78fb7d3e8917d5b7533f62c0366bc278d -- crates/contextra-db
  ```

### `contextra-privacy` (crates/contextra-privacy)
- **Aufgabe & Umfang:** GDPR / AVV / Auftragsverarbeitung und Löschungs-Metriken
- **Reifster Commit (Mature State):** `fa8b6a30cfc588f60f026db084ffef7267f1c3a6`
  - Titel: *Merge pull request #4845 from tfufuz1/decouple-store-contextra-core-8951227313492383209 (2026-10-15)*
  - Notiz: Reifste Implementierung der Privacy-/AVV-Mechanismen vor Konsolidierung in contextra-avv-generator
- **Entfernungs-/Schnitt-Commit (Removal State):** `136298b5e590dc47405437b5482ead07d4f83b5f`
  - Titel: *[Entfernung EP6] Delete crates/contextra-privacy and crates/contextra-sandbox (2026-10-15)*
- **Befehl für selektiven Cherry-Pick:**
  ```bash
  git cherry-pick -n fa8b6a30cfc588f60f026db084ffef7267f1c3a6
  ```
- **Befehl zum partiellen Auschecken des Moduls:**
  ```bash
  git checkout fa8b6a30cfc588f60f026db084ffef7267f1c3a6 -- crates/contextra-privacy
  ```

### `contextra-graph` (crates/contextra-graph)
- **Aufgabe & Umfang:** GraphRAG, PPR (Personalized PageRank), CSR Matrix, Hyperedge Relate
- **Reifster Commit (Mature State):** `15755fe936a64293bdead2aa6a79e2d0433dc45a`
  - Titel: *Merge pull request #4885 from tfufuz1/jules-15928893261243544682-b5b0201b (2026-10-16)*
  - Notiz: Vollständige GraphRAG & CSR Algorithmen inkl. n-ary Hyperedge Retrieval
- **Entfernungs-/Schnitt-Commit (Removal State):** `d22665d74ed71c4328e01e1f7937a6e9d6072b60`
  - Titel: *[Entfernung EP3] contextra-graph Dateibaum (2026-10-16)*
- **Befehl für selektiven Cherry-Pick:**
  ```bash
  git cherry-pick -n 15755fe936a64293bdead2aa6a79e2d0433dc45a
  ```
- **Befehl zum partiellen Auschecken des Moduls:**
  ```bash
  git checkout 15755fe936a64293bdead2aa6a79e2d0433dc45a -- crates/contextra-graph
  ```

### `contextra-cognition` (crates/contextra-cognition)
- **Aufgabe & Umfang:** Gedächtnis-Konsolidierung, Memory-Compaction & Background Maintenance
- **Reifster Commit (Mature State):** `15755fe936a64293bdead2aa6a79e2d0433dc45a`
  - Titel: *Merge pull request #4885 from tfufuz1/jules-15928893261243544682-b5b0201b (2026-10-16)*
  - Notiz: Vollständiges episodisches und semantisches Konsolidierungssystem vor Isolierung
- **Entfernungs-/Schnitt-Commit (Removal State):** `923636929b2017ebd585ed6702681d7eae5fabe9`
  - Titel: *[Entfernung EP5] contextra-cognition Dateibaum (2026-10-16)*
- **Befehl für selektiven Cherry-Pick:**
  ```bash
  git cherry-pick -n 15755fe936a64293bdead2aa6a79e2d0433dc45a
  ```
- **Befehl zum partiellen Auschecken des Moduls:**
  ```bash
  git checkout 15755fe936a64293bdead2aa6a79e2d0433dc45a -- crates/contextra-cognition
  ```

### `contextra-adapt` (crates/contextra-adapt)
- **Aufgabe & Umfang:** Adapter & Brücken für externe Speicher und Frameworks
- **Reifster Commit (Mature State):** `15755fe936a64293bdead2aa6a79e2d0433dc45a`
  - Titel: *Merge pull request #4885 from tfufuz1/jules-15928893261243544682-b5b0201b (2026-10-16)*
  - Notiz: Reifster Zustand für Third-Party Integrationen und Speicherbrücken
- **Entfernungs-/Schnitt-Commit (Removal State):** `c234dc3f92ee7d734b919e831c2c880aaceb68ee`
  - Titel: *[Entfernung EP5] contextra-adapt Dateibaum (2026-10-16)*
- **Befehl für selektiven Cherry-Pick:**
  ```bash
  git cherry-pick -n 15755fe936a64293bdead2aa6a79e2d0433dc45a
  ```
- **Befehl zum partiellen Auschecken des Moduls:**
  ```bash
  git checkout 15755fe936a64293bdead2aa6a79e2d0433dc45a -- crates/contextra-adapt
  ```

### `contextra-agent` (crates/contextra-agent)
- **Aufgabe & Umfang:** Autonome Agenten-Loop, Scratchpad & Goal Reasoning
- **Reifster Commit (Mature State):** `15755fe936a64293bdead2aa6a79e2d0433dc45a`
  - Titel: *Merge pull request #4885 from tfufuz1/jules-15928893261243544682-b5b0201b (2026-10-16)*
  - Notiz: Agenten-Orchestrierung vor Zerlegung in autarke Bausteine
- **Entfernungs-/Schnitt-Commit (Removal State):** `e64df1dc8ca89fa236da20add44a8472a63e8026`
  - Titel: *[Entfernung EP5] contextra-agent Dateibaum (2026-10-16)*
- **Befehl für selektiven Cherry-Pick:**
  ```bash
  git cherry-pick -n 15755fe936a64293bdead2aa6a79e2d0433dc45a
  ```
- **Befehl zum partiellen Auschecken des Moduls:**
  ```bash
  git checkout 15755fe936a64293bdead2aa6a79e2d0433dc45a -- crates/contextra-agent
  ```

### `contextra-router` (crates/contextra-router)
- **Aufgabe & Umfang:** Query Router, semantic query classifier & multi-index dispatcher
- **Reifster Commit (Mature State):** `15755fe936a64293bdead2aa6a79e2d0433dc45a`
  - Titel: *Merge pull request #4885 from tfufuz1/jules-15928893261243544682-b5b0201b (2026-10-16)*
  - Notiz: Query-Routing & semantic classification
- **Entfernungs-/Schnitt-Commit (Removal State):** `35d588b6c41b79622cca07d23b1643de5bacd663`
  - Titel: *[Entfernung EP5] contextra-router Dateibaum (2026-10-16)*
- **Befehl für selektiven Cherry-Pick:**
  ```bash
  git cherry-pick -n 15755fe936a64293bdead2aa6a79e2d0433dc45a
  ```
- **Befehl zum partiellen Auschecken des Moduls:**
  ```bash
  git checkout 15755fe936a64293bdead2aa6a79e2d0433dc45a -- crates/contextra-router
  ```

### `contextra-license` (crates/contextra-license)
- **Aufgabe & Umfang:** Signed license gate, hardware fingerprinting, installation binding
- **Reifster Commit (Mature State):** `15755fe936a64293bdead2aa6a79e2d0433dc45a`
  - Titel: *Merge pull request #4885 from tfufuz1/jules-15928893261243544682-b5b0201b (2026-10-16)*
  - Notiz: Kryptografische Lizenzprüfung & Gate-System vor Verselbstständigung
- **Entfernungs-/Schnitt-Commit (Removal State):** `6a5d12665072ac86960c443125720bc5959765bf`
  - Titel: *[Entfernung EP2] contextra-license Dateibaum (2026-10-16)*
- **Befehl für selektiven Cherry-Pick:**
  ```bash
  git cherry-pick -n 15755fe936a64293bdead2aa6a79e2d0433dc45a
  ```
- **Befehl zum partiellen Auschecken des Moduls:**
  ```bash
  git checkout 15755fe936a64293bdead2aa6a79e2d0433dc45a -- crates/contextra-license
  ```

### `contextra-sandbox` (crates/contextra-sandbox)
- **Aufgabe & Umfang:** Wasmtime execution engine, resource limiting & sandbox isolation
- **Reifster Commit (Mature State):** `fa8b6a30cfc588f60f026db084ffef7267f1c3a6`
  - Titel: *Merge pull request #4845 from tfufuz1/decouple-store-contextra-core-8951227313492383209 (2026-10-15)*
  - Notiz: Wasmtime Wasm-Merge-Operator und Sandbox Isolation
- **Entfernungs-/Schnitt-Commit (Removal State):** `136298b5e590dc47405437b5482ead07d4f83b5f`
  - Titel: *[Entfernung EP6] Delete crates/contextra-privacy and crates/contextra-sandbox (2026-10-15)*
- **Befehl für selektiven Cherry-Pick:**
  ```bash
  git cherry-pick -n fa8b6a30cfc588f60f026db084ffef7267f1c3a6
  ```
- **Befehl zum partiellen Auschecken des Moduls:**
  ```bash
  git checkout fa8b6a30cfc588f60f026db084ffef7267f1c3a6 -- crates/contextra-sandbox
  ```

### `contextra-py` (crates/contextra-py)
- **Aufgabe & Umfang:** Python PyO3 bindings & zero-copy vector transfers
- **Reifster Commit (Mature State):** `fa8b6a30cfc588f60f026db084ffef7267f1c3a6`
  - Titel: *Merge pull request #4845 from tfufuz1/decouple-store-contextra-core-8951227313492383209 (2026-10-15)*
  - Notiz: Reifste PyO3 Zero-Copy Anbindung an Contextra
- **Entfernungs-/Schnitt-Commit (Removal State):** `eba5c8b416c72b1719cd33ae249519c86f9bd36c`
  - Titel: *[Entfernung EP1] contextra-py entfernen (2026-10-15)*
- **Befehl für selektiven Cherry-Pick:**
  ```bash
  git cherry-pick -n fa8b6a30cfc588f60f026db084ffef7267f1c3a6
  ```
- **Befehl zum partiellen Auschecken des Moduls:**
  ```bash
  git checkout fa8b6a30cfc588f60f026db084ffef7267f1c3a6 -- crates/contextra-py
  ```

### `xtask` (xtask)
- **Aufgabe & Umfang:** Workspace governance, linting, forensic checkers, gate matrix
- **Reifster Commit (Mature State):** `73c64919a7594ac557c7b895163e358e720b4ac0`
  - Titel: *feat(xtask): wire 19 xtask modules in main.rs, update justfile and bootstrap docs (#3880) (2026-09-28)*
  - Notiz: Reifster Stand des 19-Modul-xtask-Frameworks vor Migration zu Standard Cargo / standalone Tools
- **Entfernungs-/Schnitt-Commit (Removal State):** `068261abdc2f1df34a878ad8e57a48cd54368578`
  - Titel: *Shell-Commit (2026-10-30)*
- **Befehl für selektiven Cherry-Pick:**
  ```bash
  git cherry-pick -n 73c64919a7594ac557c7b895163e358e720b4ac0
  ```
- **Befehl zum partiellen Auschecken des Moduls:**
  ```bash
  git checkout 73c64919a7594ac557c7b895163e358e720b4ac0 -- xtask
  ```

### `deletion_proof` (crates/contextra-crypto/src/deletion_proof.rs)
- **Aufgabe & Umfang:** Cryptographic Deletion Proof (BSI TR-02102 / Ed25519 & Falcon)
- **Reifster Commit (Mature State):** `cc4dfa5aee16ee884874dd975102f1a753055967`
  - Titel: *contextra-crypto: harden DeletionProof::issue with V5 verified witness measurement binding (2026-10-28)*
  - Notiz: Vollständige V5 DeletionProof Implementierung vor Auslagerung nach contextra-erasure
- **Entfernungs-/Schnitt-Commit (Removal State):** `0a0f49ba09ece0778009bb399c3d46886e68c65a`
  - Titel: *Delete crates/contextra-crypto/src/deletion_proof.rs (2026-10-29)*
- **Befehl für selektiven Cherry-Pick:**
  ```bash
  git cherry-pick -n cc4dfa5aee16ee884874dd975102f1a753055967
  ```
- **Befehl zum partiellen Auschecken des Moduls:**
  ```bash
  git checkout cc4dfa5aee16ee884874dd975102f1a753055967 -- crates/contextra-crypto/src/deletion_proof.rs
  ```

### `hnsw_purger` (crates/contextra-vector/src/hnsw/purger.rs)
- **Aufgabe & Umfang:** HNSW Vector Purger & LayerScanner für graph-basierte Vektorlöschung
- **Reifster Commit (Mature State):** `e343f32bf061e69dafd3468244fb764207a25b9c`
  - Titel: *contextra-vector: Implement HNSW LayerScanner and Purger [LÖSCHKETTE W2-P01] (2026-10-28)*
  - Notiz: Reifste Vektor-Purging-Logik vor Vormerkung zur contextra-erasure-spi Auslagerung
- **Entfernungs-/Schnitt-Commit (Removal State):** `7963b33636b22a27d45bfa06ed963b5a9d6503e9`
  - Titel: *refactor(contextra-vector): modular decoupling and isolated building-block architecture Baukasten-Refaktorisierung für contextra-vector (Paket B12): (2026-11-07)*
- **Befehl für selektiven Cherry-Pick:**
  ```bash
  git cherry-pick -n e343f32bf061e69dafd3468244fb764207a25b9c
  ```
- **Befehl zum partiellen Auschecken des Moduls:**
  ```bash
  git checkout e343f32bf061e69dafd3468244fb764207a25b9c -- crates/contextra-vector/src/hnsw/purger.rs
  ```

---

## 4. Vollständiger Katalog der 'Shell-Commits' & Präzise Änderungs-Kommentare

Alle 241 'Shell-Commits' wurden durch Prüfung der Git-Diff-Trees dechiffriert. Nachfolgend sind
die ersten 100 Shell-Commits mit Datum, betroffenen Dateien und dem empfohlenen präzisen Commit-Text
aufgeführt (der komplette Datensatz steht in `docs/HISTORIE_FORENSIK_AUDIT.json` zur Verfügung):

| Hash | Datum | Geänderte Dateien | Bereich | Präziser Kommentar (Ersatz für 'Shell-Commit') |
| :--- | :--- | :--- | :--- | :--- |
| `ebef86535` | 2026-11-07 | 1 | `docs` | add: `docs/ARCHITEKTUR_BAUSTEINE_LLM_OLD.md` |
| `b44c2d99e` | 2026-11-07 | 2 | `tools` | refactor: add 2 file(s) in tools |
| `10dc414c5` | 2026-11-07 | 1 | `crates` | update: `crates/contextra-vector/AGENTS.md` |
| `f6c0a9287` | 2026-11-07 | 1 | `docs` | remove: `docs/BAUKASTEN_ARCHITEKTUR_BERATUNG.md` |
| `ecd2b61d8` | 2026-11-07 | 19 | `Cargo.lock` | refactor: update 19 file(s) in Cargo.lock, Cargo.toml, DEPRECATED_DEBT.md |
| `e6abdef29` | 2026-11-07 | 5 | `docs` | refactor: add 3 file(s), remove 2 file(s) in docs |
| `33f7b6da4` | 2026-11-06 | 1 | `crate-coupling-report.md` | remove: `crate-coupling-report.md` |
| `d297f7df2` | 2026-11-06 | 2 | `crate-coupling-report.md` | refactor: add 2 file(s) in crate-coupling-report.md, docs |
| `1b78ad44f` | 2026-11-06 | 1 | `docs` | remove: `docs/crate-coupling-report.md` |
| `adbeedc9b` | 2026-11-06 | 1 | `AGENTS.md` | update: `AGENTS.md` |
| `1b5683e3d` | 2026-11-06 | 4 | `AGENTS.md` | refactor: add 1 file(s), update 1 file(s), remove 2 file(s) in AGENTS.md, crate-coupling-report.md, docs |
| `5d1a40553` | 2026-11-06 | 3 | `crate-coupling-report.md` | refactor: add 3 file(s) in crate-coupling-report.md, docs |
| `9ce55716f` | 2026-11-06 | 1 | `AGENTS.md` | update: `AGENTS.md` |
| `fd5164fc4` | 2026-11-06 | 1 | `AGENTS.md` | update: `AGENTS.md` |
| `99f69cf00` | 2026-11-06 | 2 | `AGENTS.md` | refactor: add 2 file(s) in AGENTS.md, PLAN.md |
| `adfa04f12` | 2026-11-06 | 19 | `crates` | refactor: remove 19 file(s) in crates |
| `dbe366173` | 2026-11-06 | 47 | `docs` | refactor: remove 47 file(s) in docs, flake.lock, flake.nix |
| `770b603c8` | 2026-11-06 | 2 | `crate-coupling-report_20261105.md` | refactor: remove 2 file(s) in crate-coupling-report_20261105.md, reports.zip |
| `f55c50433` | 2026-11-06 | 10 | `Cargo.lock` | refactor: add 9 file(s), update 1 file(s) in Cargo.lock, crate-coupling-report_20261105.md, docs |
| `88e4cd747` | 2026-10-31 | 24 | `AGENTS.md` | refactor: update 1 file(s), remove 23 file(s) in AGENTS.md, CLAUDE.md, README.md |
| `641dc4bda` | 2026-10-31 | 1 | `Cargo.toml` | update: `Cargo.toml` |
| `ff5010095` | 2026-10-31 | 11 | `README.md` | refactor: add 6 file(s), update 5 file(s) in README.md, docs |
| `9102f2cd2` | 2026-10-30 | 7 | `Cargo.toml` | refactor: add 3 file(s), update 4 file(s) in Cargo.toml, crates, tests |
| `7a379c6a9` | 2026-10-30 | 98 | `.agents` | refactor: update 98 file(s) in .agents, .github, AGENTS.md |
| `2870ec929` | 2026-10-30 | 1 | `main-vor-reset-4c1d3a67.bundle` | add: `main-vor-reset-4c1d3a67.bundle` |
| `068261abd` | 2026-10-30 | 241 | `Cargo.lock` | refactor: update 1 file(s), remove 240 file(s) in Cargo.lock, xtask |
| `ed402e21a` | 2026-10-30 | 991 | `.gitignore` | refactor: add 2 file(s), update 18 file(s), remove 971 file(s) in .gitignore, Cargo.toml, REDUCTION_RESIDUALS.md |
| `3c1f144fa` | 2026-10-29 | 5 | `evidence` | refactor: add 5 file(s) in evidence, scripts |
| `f65cdd776` | 2026-10-29 | 1 | `crates` | remove: `crates/contextra-graph/src/csr/graph_write.rs` |
| `66f19dfa2` | 2026-10-29 | 2 | `docs` | refactor: update 2 file(s) in docs |
| `52d3227f0` | 2026-10-28 | 1 | `crates` | update: `crates/contextra-vector/tests/diskann_docid128_tombstones.rs` |
| `47801f882` | 2026-10-28 | 10 | `docs` | refactor: remove 10 file(s) in docs |
| `1780bd2c0` | 2026-10-28 | 3 | `xtask` | refactor: update 3 file(s) in xtask |
| `8f8fa7cd5` | 2026-10-28 | 3 | `crates` | refactor: update 3 file(s) in crates |
| `33da4458e` | 2026-10-28 | 1 | `crates` | update: `crates/contextra-store/tests/adaptive_compaction_determinism.rs` |
| `0caf9554d` | 2026-10-27 | 1 | `docs` | update: `docs/SPEC.md` |
| `c387884b7` | 2026-10-26 | 1 | `Cargo.lock` | update: `Cargo.lock` |
| `04f2def36` | 2026-10-26 | 5 | `crates` | refactor: update 5 file(s) in crates |
| `b9f73d2b2` | 2026-10-26 | 5 | `docs` | refactor: update 5 file(s) in docs |
| `93a04093c` | 2026-10-26 | 2 | `PLAN.md` | refactor: add 1 file(s), remove 1 file(s) in PLAN.md, docs |
| `7ebd6049f` | 2026-10-26 | 2 | `docs` | refactor: add 2 file(s) in docs |
| `c9279019d` | 2026-10-26 | 179 | `ARCHITECTURE.md` | refactor: add 4 file(s), remove 175 file(s) in ARCHITECTURE.md, CONSTITUTION.md, CONTRIBUTING.md |
| `e6593dbd7` | 2026-10-25 | 1 | `crates` | update: `crates/contextra-wire/src/contextra_generated.rs` |
| `dd748a20b` | 2026-10-16 | 1 | `crates` | update: `crates/contextra-wire/src/contextra_generated.rs` |
| `f9429808b` | 2026-10-16 | 511 | `docs` | refactor: remove 511 file(s) in docs |
| `e40902ac2` | 2026-10-15 | 2 | `docs` | refactor: add 1 file(s), remove 1 file(s) in docs |
| `f00bb2af7` | 2026-10-14 | 39 | `docs` | refactor: add 26 file(s), remove 13 file(s) in docs |
| `426dfead9` | 2026-10-14 | 49 | `AGENTS.md` | refactor: add 27 file(s), update 22 file(s) in AGENTS.md, README.md, crates |
| `ce52bda8c` | 2026-10-14 | 11 | `docs` | refactor: add 10 file(s), update 1 file(s) in docs |
| `b5037b46a` | 2026-10-14 | 27 | `"docs` | refactor: add 9 file(s), remove 18 file(s) in "docs, docs |
| `0c315cbf9` | 2026-10-14 | 4 | `docs` | refactor: add 3 file(s), update 1 file(s) in docs |
| `230372f0d` | 2026-10-14 | 14 | `crates` | refactor: add 2 file(s), update 12 file(s) in crates, docs, rules |
| `4a88c4255` | 2026-10-14 | 11 | `.jules` | refactor: add 4 file(s), update 7 file(s) in .jules, docs |
| `f375ad8ac` | 2026-10-14 | 2 | `docs` | refactor: add 2 file(s) in docs |
| `561c46fbb` | 2026-10-14 | 12 | `"Contextra \342\200\223 Gesamtanalyse Architektur und L\303\266schnachweis (HEAD 16764583).md"` | refactor: add 10 file(s), remove 2 file(s) in "Contextra \342\200\223 Gesamtanalyse Architektur und L\303\266schnachweis (HEAD 16764583).md", "docs, IMPLEMENTIERUNGSSPEZIFIKATION.md |
| `482d0c94c` | 2026-10-14 | 9 | `crates` | refactor: add 1 file(s), update 8 file(s) in crates |
| `83b2cc863` | 2026-10-14 | 16 | `"Contextra \342\200\223 Gesamtanalyse Architektur und L\303\266schnachweis (HEAD 16764583).md"` | refactor: add 3 file(s), update 13 file(s) in "Contextra \342\200\223 Gesamtanalyse Architektur und L\303\266schnachweis (HEAD 16764583).md", IMPLEMENTIERUNGSSPEZIFIKATION.md, benchmarks |
| `180391a09` | 2026-10-13 | 2 | `crates` | refactor: update 2 file(s) in crates |
| `9ad0c80a3` | 2026-10-13 | 11 | `crates` | refactor: add 2 file(s), update 9 file(s) in crates |
| `59420f7da` | 2026-10-13 | 1 | `crates` | update: `crates/contextra-wire/src/contextra_generated.rs` |
| `4b7709e66` | 2026-10-13 | 44 | `crates` | refactor: add 2 file(s), update 42 file(s) in crates, xtask |
| `561dcec9d` | 2026-10-13 | 52 | `.jules` | refactor: add 4 file(s), update 48 file(s) in .jules, SECURITY.md, crates |
| `755624dc6` | 2026-10-13 | 14 | `SECURITY.md` | refactor: add 3 file(s), update 11 file(s) in SECURITY.md, crates |
| `edeb252d0` | 2026-10-13 | 8 | `.jules` | refactor: add 5 file(s), update 3 file(s) in .jules, crates |
| `27830321f` | 2026-10-13 | 4 | `.jules` | refactor: update 4 file(s) in .jules |
| `6ba2898e8` | 2026-10-11 | 5 | `crates` | refactor: add 2 file(s), update 3 file(s) in crates |
| `8cedf9fb7` | 2026-10-11 | 15 | `benches` | refactor: update 15 file(s) in benches, crates |
| `b648f7087` | 2026-10-11 | 3 | `crates` | refactor: update 3 file(s) in crates |
| `b8660984b` | 2026-10-11 | 9 | `.jules` | refactor: update 9 file(s) in .jules, WORKING_STATE.md, benches |
| `a2b063dcd` | 2026-10-11 | 10 | `crates` | refactor: add 1 file(s), update 9 file(s) in crates, docs |
| `240f4f7c0` | 2026-10-11 | 153 | `.github` | refactor: add 71 file(s), update 82 file(s) in .github, .jules, ARCHITECTURE.md |
| `d65505630` | 2026-10-10 | 1 | `crates` | update: `crates/contextra-wire/src/contextra_generated.rs` |
| `19d12ae45` | 2026-10-07 | 3 | `benchmarks` | refactor: update 3 file(s) in benchmarks, crates |
| `52e6ea7e8` | 2026-10-06 | 27 | `crates` | refactor: update 27 file(s) in crates |
| `e31ed5c4c` | 2026-10-05 | 3 | `Cargo.lock` | refactor: update 3 file(s) in Cargo.lock, crates |
| `5a2b348ac` | 2026-10-03 | 2 | `.jules` | refactor: update 2 file(s) in .jules |
| `24cf38527` | 2026-10-03 | 3 | `BENCH-LOG.md` | refactor: add 2 file(s), update 1 file(s) in BENCH-LOG.md, crates, docs |
| `03567e318` | 2026-10-03 | 10 | `.jules` | refactor: add 3 file(s), update 5 file(s), remove 2 file(s) in .jules, LOG.md, WORKING_STATE.md |
| `9aecb84d7` | 2026-10-01 | 4 | `crates` | refactor: update 4 file(s) in crates |
| `519880da2` | 2026-09-28 | 1 | `.jules` | update: `.jules/contextra-Prompter.html` |
| `45d154cb4` | 2026-09-27 | 10 | `xtask` | refactor: add 10 file(s) in xtask |
| `e0a66d335` | 2026-09-27 | 1 | `crates` | update: `crates/contextra-wire/src/contextra_generated.rs` |
| `58a6174ba` | 2026-09-25 | 1 | `crates` | update: `crates/contextra-wire/src/contextra_generated.rs` |
| `ec52eb62d` | 2026-09-25 | 3 | `Cargo.lock` | refactor: update 3 file(s) in Cargo.lock, crates |
| `7e97d9519` | 2026-09-25 | 1 | `crates` | update: `crates/contextra-store/src/lsm/tests.rs` |
| `d2066b3f6` | 2026-09-25 | 6 | `Cargo.lock` | refactor: update 6 file(s) in Cargo.lock, crates |
| `f51de7fa1` | 2026-09-25 | 1 | `xtask` | update: `xtask/Cargo.lock` |
| `150d4caa1` | 2026-09-25 | 326 | `benches` | refactor: add 6 file(s), update 320 file(s) in benches, benchmarks, crates |
| `346edb1c4` | 2026-09-23 | 1 | `docs` | update: `docs/GESAMTSPEZIFIKATION.md` |
| `9300ea21d` | 2026-09-20 | 107 | `.github` | refactor: add 48 file(s), update 27 file(s), remove 32 file(s) in .github, AGENTS.md, Cargo.lock |
| `84bd7e6e5` | 2026-09-20 | 11 | `.github` | refactor: add 3 file(s), update 8 file(s) in .github, crates, deny.toml |
| `b4efbc09f` | 2026-09-19 | 11 | `.cargo` | refactor: add 6 file(s), update 5 file(s) in .cargo, Cargo.lock, Cargo.toml |
| `77bb41f4b` | 2026-09-19 | 2 | `README.md` | refactor: update 2 file(s) in README.md, docs |
| `729b19b1f` | 2026-09-19 | 5 | `docs` | refactor: remove 5 file(s) in docs |
| `3f37ab309` | 2026-09-18 | 70 | `results` | refactor: add 70 file(s) in results |
| `90b057c58` | 2026-09-18 | 6 | `crates` | refactor: update 6 file(s) in crates, xtask |
| `43495ab9f` | 2026-09-18 | 3 | `crates` | refactor: update 3 file(s) in crates |
| `7432e2af2` | 2026-09-18 | 32 | `README.md` | refactor: add 30 file(s), update 2 file(s) in README.md, docs, results |
| `ceee3f402` | 2026-09-18 | 1 | `verify_workspace.sh` | update: `verify_workspace.sh` |
| `a9b2caa23` | 2026-09-17 | 19 | `.jules` | refactor: add 1 file(s), update 18 file(s) in .jules, AGENTS.md, CONSTITUTION.md |
| ... | ... | ... | ... | *Weitere 141 Shell-Commits siehe JSON-Audit.* |

---

## 5. Leere Commits & Redundanz-Audit

Es wurden 330 leere Commits im Verlauf identifiziert. Ein Commit gilt als leer,
wenn der resultierende Dateibaum (`tree`) identisch zu seinem Eltern-Commit (`parent tree`) ist.
Diese Commits erzeugen keinen funktionalen Mehrwert und stammen vor allem aus Rebase-Operationen
oder leeren Synchronisationsläufen.

| Hash | Parent | Ursprüngliche Nachricht | Diagnose |
| :--- | :--- | :--- | :--- |
| `4de18f72a` | `6f19f3e9b` | mvcc: modular decoupling, SSI sentinel hardening, elimi | Redundanter Rebase/Sync-Commit (identischer Tree) |
| `a4edf6e49` | `0269bbc6c` | refactor(contextra-infer-onnx): align modular baukasten | Redundanter Rebase/Sync-Commit (identischer Tree) |
| `0269bbc6c` | `283f555d2` | refactor(contextra-infer-onnx): align modular baukasten | Redundanter Rebase/Sync-Commit (identischer Tree) |
| `34eb20687` | `06c8084c9` | docs(agents): overhaul all crate AGENTS.md files for mo | Redundanter Rebase/Sync-Commit (identischer Tree) |
| `06c8084c9` | `ad40e2bbb` | docs(agents): overhaul all crate AGENTS.md files for mo | Redundanter Rebase/Sync-Commit (identischer Tree) |
| `37bca9540` | `95c78a0b4` | refactor(contextra-erasure): modular decoupling and iso | Redundanter Rebase/Sync-Commit (identischer Tree) |
| `ad40e2bbb` | `284ef8a14` | docs(agents): overhaul all crate AGENTS.md files for mo | Redundanter Rebase/Sync-Commit (identischer Tree) |
| `284ef8a14` | `912d5badb` | docs(agents): overhaul all crate AGENTS.md files for mo | Redundanter Rebase/Sync-Commit (identischer Tree) |
| `95c78a0b4` | `00b3c2f09` | refactor(contextra-erasure): modular decoupling and iso | Redundanter Rebase/Sync-Commit (identischer Tree) |
| `00b3c2f09` | `ba7e3337b` | refactor(contextra-erasure): modular decoupling and iso | Redundanter Rebase/Sync-Commit (identischer Tree) |
| `db86e080a` | `1b655647d` | refactor(agent,engine): modular decoupling and state ma | Redundanter Rebase/Sync-Commit (identischer Tree) |
| `70e89ce0e` | `176b30483` | docs/test(license): KEY_MANAGEMENT.md und Schlüsselwech | Redundanter Rebase/Sync-Commit (identischer Tree) |
| `176b30483` | `0ffe77dfb` | docs/test(license): KEY_MANAGEMENT.md und Schlüsselwech | Redundanter Rebase/Sync-Commit (identischer Tree) |
| `000ee0c5d` | `bb4a743b3` | docs(adr): ADR-0006 bis 0008 als Proposed (Löschkette) | Redundanter Rebase/Sync-Commit (identischer Tree) |
| `83c400031` | `9b66a5817` | docs(spec): legal-matrix.toml aus SPEC §9.12 (WP 5.10,  | Redundanter Rebase/Sync-Commit (identischer Tree) |
| `9b66a5817` | `add305884` | docs(spec): legal-matrix.toml aus SPEC §9.12 (WP 5.10,  | Redundanter Rebase/Sync-Commit (identischer Tree) |
| `b7c9c03d7` | `7e6fd5529` | sandbox: Wasmtime-Upgrade schließt 21 Advisories | Redundanter Rebase/Sync-Commit (identischer Tree) |
| `7e6fd5529` | `890329831` | sandbox: Wasmtime-Upgrade schließt 21 Advisories | Redundanter Rebase/Sync-Commit (identischer Tree) |
| `890329831` | `089fd86b7` | sandbox: Wasmtime-Upgrade schließt 21 Advisories | Redundanter Rebase/Sync-Commit (identischer Tree) |
| `3809b513e` | `5ee5d73e0` | router: Clock-Port, Dublette entschärft, Idempotenz-/Is | Redundanter Rebase/Sync-Commit (identischer Tree) |
| `f94ed2316` | `e462a0635` | kvcache: produktive Port-Implementierungen, KvCacheServ | Redundanter Rebase/Sync-Commit (identischer Tree) |
| `9711df409` | `e462a0635` | infer-candle: GGUF-Härtung, deterministische Generierun | Redundanter Rebase/Sync-Commit (identischer Tree) |
| `5ba5a03c8` | `e462a0635` | infer-ollama: Loopback-Egress-Policy, Resilienz, Dimens | Redundanter Rebase/Sync-Commit (identischer Tree) |
| `fbe688cc8` | `5f6af5429` | graph: ppr/ und csr/ modularisiert, Charakterisierungst | Redundanter Rebase/Sync-Commit (identischer Tree) |
| `57afc696b` | `0796b40ed` | graph: ppr/ und csr/ modularisiert, Charakterisierungst | Redundanter Rebase/Sync-Commit (identischer Tree) |
| `9f3c92286` | `0796b40ed` | audit-export: Art.-30-Vollständigkeit, deterministische | Redundanter Rebase/Sync-Commit (identischer Tree) |
| `6b8447e70` | `28984bfaf` | docs: update SPEC.md and PLAN.md to Fassung 2.5 for dec | Redundanter Rebase/Sync-Commit (identischer Tree) |
| `d946ced76` | `cc4dfa5ae` | contextra-crypto: harden DeletionProof::issue with V5 v | Redundanter Rebase/Sync-Commit (identischer Tree) |
| `67ddccac0` | `ca0573f67` | contextra-ports: scanner V1 & witness mint [LÖSCHKETTE  | Redundanter Rebase/Sync-Commit (identischer Tree) |
| `ca0573f67` | `b27a796c9` | contextra-ports: implement scanner V1 contract and lock | Redundanter Rebase/Sync-Commit (identischer Tree) |
| `eeaaca9d5` | `84823d350` | ports: Scanner-Vertrag V1 und gesperrter Mint-Pfad [LÖS | Redundanter Rebase/Sync-Commit (identischer Tree) |
| `d552acd78` | `01d2a0931` | [Durability-Primitive & SAFETY-Luecken] contextra-sys:  | Redundanter Rebase/Sync-Commit (identischer Tree) |
| `b055aa50c` | `ed13bea7d` | refactor(contextra-wire): restrict allow lints to gener | Redundanter Rebase/Sync-Commit (identischer Tree) |
| `01d2a0931` | `3e3dae16c` | feat(sys): implement SPEC 7.7 durability primitives and | Redundanter Rebase/Sync-Commit (identischer Tree) |
| `9b3a4c541` | `a3c39880f` | refactor(simd): set SIMD kernels to pub(crate) and enca | Redundanter Rebase/Sync-Commit (identischer Tree) |
| `43fac19ce` | `86b60d3e8` | [WP 2.1] Workspace-Archivierung: Kanten-Inventar erstel | Redundanter Rebase/Sync-Commit (identischer Tree) |
| `56093515b` | `2baa40da6` | docs: generate xtask inventory evidence | Redundanter Rebase/Sync-Commit (identischer Tree) |
| `2baa40da6` | `732e526a2` | docs: add xtask inventory evidence under evidence/xtask | Redundanter Rebase/Sync-Commit (identischer Tree) |
| `732e526a2` | `3fc19a1f2` | docs: generate xtask inventory evidence | Redundanter Rebase/Sync-Commit (identischer Tree) |
| `3fc19a1f2` | `596489710` | WP-xtask-inventory: Belegtes Inventar von xtask (db68bd | Redundanter Rebase/Sync-Commit (identischer Tree) |
| `84609aaac` | `16efd2054` | WP-2.3(mcp): Entkopplung contextra-mcp von Archiv-Crate | Redundanter Rebase/Sync-Commit (identischer Tree) |
| `16efd2054` | `347e34934` | WP-2.3(mcp): Entkopplung contextra-mcp von Archiv-Crate | Redundanter Rebase/Sync-Commit (identischer Tree) |
| `008a5f230` | `106c09405` | WP-0.4(docs,tools): Governance und Harness Pfadaudit (P | Redundanter Rebase/Sync-Commit (identischer Tree) |
| `106c09405` | `536c8bbd5` | WP-0.4(docs,tools): Governance und Harness Pfadaudit (P | Redundanter Rebase/Sync-Commit (identischer Tree) |
| `536c8bbd5` | `644ddadf1` | WP-0.4(docs,tools): Governance und Harness Pfadaudit (P | Redundanter Rebase/Sync-Commit (identischer Tree) |
| `644ddadf1` | `b8b2fa434` | WP-0.4(docs,tools): Governance und Harness Pfadaudit (P | Redundanter Rebase/Sync-Commit (identischer Tree) |
| `8cb803ca4` | `f8d2612a9` | WP-0.4(governance,xtask): Lint-Satz vervollstaendigt, R | Redundanter Rebase/Sync-Commit (identischer Tree) |
| `a77c6a7e1` | `7392a560a` | WP-0.4(governance,xtask): Lint-Satz vervollstaendigt, R | Redundanter Rebase/Sync-Commit (identischer Tree) |
| `12adce055` | `6676a5e27` | evidence: baseline a547a42 (WP 0.1) | Redundanter Rebase/Sync-Commit (identischer Tree) |
| `6676a5e27` | `35cfb5fd3` | evidence: baseline a547a42 (WP 0.1) | Redundanter Rebase/Sync-Commit (identischer Tree) |
| ... | ... | ... | *Weitere 280 leere Commits siehe JSON-Audit.* |

---

## 6. Git-Tags & Repository-Markierungen

Zur dauerhaften und bequemen Navigation in GitHub und Git-Clients wurden für die reifsten
Zustände und Schnittpunkte Tags im lokalen Git-Objektspeicher angelegt:

- `git tag cherry-pick/contextra-core-mature 15755fe93` -> Reifster Stand von `contextra-core`
- `git tag forensic/contextra-core-removal 24862f3c7` -> Schnitt-/Entfernungspunkt von `contextra-core`
- `git tag cherry-pick/contextra-db-mature 9cbcf3a78` -> Reifster Stand von `contextra-db`
- `git tag forensic/contextra-db-removal 084b3c7ff` -> Schnitt-/Entfernungspunkt von `contextra-db`
- `git tag cherry-pick/contextra-privacy-mature fa8b6a30c` -> Reifster Stand von `contextra-privacy`
- `git tag forensic/contextra-privacy-removal 136298b5e` -> Schnitt-/Entfernungspunkt von `contextra-privacy`
- `git tag cherry-pick/contextra-graph-mature 15755fe93` -> Reifster Stand von `contextra-graph`
- `git tag forensic/contextra-graph-removal d22665d74` -> Schnitt-/Entfernungspunkt von `contextra-graph`
- `git tag cherry-pick/contextra-cognition-mature 15755fe93` -> Reifster Stand von `contextra-cognition`
- `git tag forensic/contextra-cognition-removal 923636929` -> Schnitt-/Entfernungspunkt von `contextra-cognition`
- `git tag cherry-pick/contextra-adapt-mature 15755fe93` -> Reifster Stand von `contextra-adapt`
- `git tag forensic/contextra-adapt-removal c234dc3f9` -> Schnitt-/Entfernungspunkt von `contextra-adapt`
- `git tag cherry-pick/contextra-agent-mature 15755fe93` -> Reifster Stand von `contextra-agent`
- `git tag forensic/contextra-agent-removal e64df1dc8` -> Schnitt-/Entfernungspunkt von `contextra-agent`
- `git tag cherry-pick/contextra-router-mature 15755fe93` -> Reifster Stand von `contextra-router`
- `git tag forensic/contextra-router-removal 35d588b6c` -> Schnitt-/Entfernungspunkt von `contextra-router`
- `git tag cherry-pick/contextra-license-mature 15755fe93` -> Reifster Stand von `contextra-license`
- `git tag forensic/contextra-license-removal 6a5d12665` -> Schnitt-/Entfernungspunkt von `contextra-license`
- `git tag cherry-pick/contextra-sandbox-mature fa8b6a30c` -> Reifster Stand von `contextra-sandbox`
- `git tag forensic/contextra-sandbox-removal 136298b5e` -> Schnitt-/Entfernungspunkt von `contextra-sandbox`
- `git tag cherry-pick/contextra-py-mature fa8b6a30c` -> Reifster Stand von `contextra-py`
- `git tag forensic/contextra-py-removal eba5c8b41` -> Schnitt-/Entfernungspunkt von `contextra-py`
- `git tag cherry-pick/xtask-mature 73c64919a` -> Reifster Stand von `xtask`
- `git tag forensic/xtask-removal 068261abd` -> Schnitt-/Entfernungspunkt von `xtask`
- `git tag cherry-pick/deletion_proof-mature cc4dfa5ae` -> Reifster Stand von `deletion_proof`
- `git tag forensic/deletion_proof-removal 0a0f49ba0` -> Schnitt-/Entfernungspunkt von `deletion_proof`
- `git tag cherry-pick/hnsw_purger-mature e343f32bf` -> Reifster Stand von `hnsw_purger`
- `git tag forensic/hnsw_purger-removal 7963b3363` -> Schnitt-/Entfernungspunkt von `hnsw_purger`
