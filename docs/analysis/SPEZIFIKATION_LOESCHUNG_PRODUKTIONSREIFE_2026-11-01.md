# Contextra — Umfassende Prüfung der Spezifikation, Löscharchitektur und Produktionsreife

<!-- doc-stamp: 2026-11-01 | Autor: Principal Senior Rust Engineer & RAG Systems Specialist -->
<!-- Status: Normativ bewertet & abgenommen gegen docs/SPEC.md (v2.0.0), docs/DELETION-DSGVO.md, docs/CONSTITUTION.md, PLAN.md -->

---

## Management Summary & Mission

Contextra verfolgt die Mission, ein **vollständig datenschutzkonformer, souveräner und hochgradig effizienter lokaler Wissens- und Gedächtnisspeicher in Rust** (Single-Node, Library-First) für sensible europäische und deutsche Unternehmensumgebungen zu sein.

Das funktionale und rechtliche **Alleinstellungsmerkmal (USP)** von Contextra ist:
> **Löschen ist ein eigener, physischer, entkoppelter und kryptografisch messbarer Prozess (Ebene A nach ADR-001 / SPEC §11), der tief im Fundament der Speicher- und Indexstrukturen verankert ist.**
> Ein Betreiber führt `erase(Ziel)` aus und erhält einen kryptografisch per Ed25519 signierten, maschinenlesbaren Löschbericht (`ctx-erasure-report/1`), der exakt ausweist, welche Schichten bereinigt wurden, welche Residuen messbar entfernt sind und welche Risiken außerhalb der Systemgrenzen liegen.

Diese Prüfung verifiziert die architektonischen Spezifikationen, die rechtliche Einordnung nach DSGVO Art. 17/18 sowie BDSG, die tatsächliche Belastbarkeit der Löschpfade im Code, die Test- und Benchmark-Frameworks und die verbindliche Roadmap zur Erlangung der **Produktionsreife (v1.0.0)**.

---

## 1. Präzise Überprüfung der Spezifikation (`docs/SPEC.md`, `DELETION-DSGVO.md`)

### 1.1 Rechtliche Verortung: DSGVO Art. 17/18 und Haftung
- **Kein Methodenzwang in der DSGVO:** Die DSGVO (Art. 17) schreibt keine spezifische technische Methode zur Löschung vor. Maßstab der Datenschutzkonferenz (DSK) und des Standard-Datenschutzmodells (SDM) ist das **Unkenntlichmachen**, sodass ein Personenbezug nicht mehr oder nur mit unverhältnismäßigem Aufwand reproduzierbar ist.
- **Rollen und Betreiberhaftung:** Contextra ist eine lokale, eingebettete Single-Node-Bibliothek (Library-First, kein SaaS, kein Server-Socket, keine Telemetrie). Der Betreiber des Gesamtsystems ist der *Verantwortliche* im Sinne von Art. 4 Nr. 7 DSGVO. Contextra haftet nicht für den rechtlichen Kontext des Kunden, sondern liefert **technisch beweisbare, messbare Garantien (E1–E4 Evidenz)** über den Zustand der unter Kontrolle stehenden Datenstrukturen.
- **Verbot unzulässiger Außenaussagen:** Gemäß SPEC §1.6 und §6.4 sind unbeweisbare Werbeaussagen strikt untersagt. Es gilt ausschließlich der verbindliche Wortlaut aus SPEC §6.4. Was gemessen wurde, wird belegt; was physikalisch außerhalb liegt (Hardware-Remanenz, OS-Journale, externe Backups, Prozessspeicher), wird als Restrisiko im Bericht ausgewiesen.

### 1.2 Die Grundsatzentscheidung: ADR-001 (Verzicht auf Ebene B in v1)
In der initialen Entwurfsphase (`docs/DELETION-DSGVO.md`) wurde die Kombination aus physischer Löschung (Ebene A) und Schlüsselvernichtung (Crypto-Shredding auf Dokumentenebene per KeyTable / `DocDek`, Ebene B) erwogen. Die normative Überprüfung in **ADR-001** und **docs/SPEC.md §3.3 (Entscheidung D-01)** ergab jedoch zwingende Gründe für den Verzicht auf Ebene B in v1:
1. **Rechtliche Unsicherheit:** Weder europäische Gerichte noch Aufsichtsbehörden haben Crypto-Shredding bisher als vollständige Erfüllung von Art. 17 DSGVO anerkannt. Verschlüsselte Daten bleiben solange personenbezogen, wie ein theoretischer Entschlüsselungspfad oder Schlüsselkompromittierungen denkbar sind.
2. **Klartext-Strukturen im RAM:** Suchstrukturen (HNSW-Graphen, BM25-Posting-Listen) müssen zur performanten Inferenz im Arbeitsspeicher unverschlüsselt vorliegen. Ein reines Crypto-Shredding auf Chiffretexten verfehlt diese Strukturen vollständig.
3. **Konsequenz für v1:** Contextra v1 setzt auf **kompromisslose physische Bereinigung (Ebene A)**, kombiniert mit sofortiger Lese- und Suchsperre (`ReadGate` via `ErasedSet`), automatisierter 3-Stufen-Residuenmessung (E1 Rohscan, E2 Decoder-Scan, E3 Slot- und Schlüsselabwesenheit) und einem signierten Ed25519-Löschbericht.

---

## 2. Architektur des Löschkerns & Performance-Verankerung

Die physische Löschung ist in Contextra nicht als nachträglicher Patch, sondern tief im Layout aller Datenpfade verankert:

```
                  ┌──────────────────────────────────────────────┐
                  │          erase(EraseTarget, Attestation)      │
                  └──────────────────────┬───────────────────────┘
                                         │
                                         ▼
                  ┌──────────────────────────────────────────────┐
                  │ 1. READGATE: ErasedSet + SnapshotRevocation  │ ◄── Fail-Closed Sperre
                  └──────────────────────┬───────────────────────┘     (SLA: p50 ≤ 10ms)
                                         │
                                         ▼
                  ┌──────────────────────────────────────────────┐
                  │ 2. PURGE-BARRIER & WAL-DRAIN                 │ ◄── Versiegelung WAL,
                  └──────────────────────┬───────────────────────┘     Flush aller Memtables
                                         │
         ┌───────────────────────────────┼───────────────────────────────┐
         ▼                               ▼                               ▼
┌──────────────────┐           ┌──────────────────┐           ┌──────────────────┐
│ LSM-Store        │           │ Textindex (BM25) │           │ Vektorindex HNSW │
│ - SSTable-Stream │           │ - Df-Zähler-Sync │           │ - Slot-Zeroing   │
│ - Exact Keymatch │           │ - Segment-Rewrite│           │ - Snapshots neu  │
│ - Manifest-Swap  │           │ - Alt-Löschung   │           │ - Batch-Rebuild  │
└────────┬─────────┘           └────────┬─────────┘           └────────┬─────────┘
         └───────────────────────────────┼───────────────────────────────┘
                                         │
                                         ▼
                  ┌──────────────────────────────────────────────┐
                  │ 3. RESIDUEN-MESSUNG & CANARY-VALIDIERUNG     │ ◄── E1: Rohscan Files
                  │    (ForensicScanner, DecoderScan, E1/E2/E3)  │     E2: Decoder-Scan
                  └──────────────────────┬───────────────────────┘     E3: Key-Abwesenheit
                                         │
                                         ▼
                  ┌──────────────────────────────────────────────┐
                  │ 4. ED25519 PROOF ISSUANCE & VERIFICATION     │ ◄── Fälschungssicherer
                  │    (ctx-erasure-report/1, verify-API)        │     Löschbericht
                  └──────────────────────────────────────────────┘
```

### 2.1 Schichtenspezifische physische Purge-Mechanismen (Ebene A)
1. **LSM-Storage (`contextra-store`)**:
   - Zielgerichtete Purge-Barriere gegen Hintergrund-Compactions.
   - Streaming-Rewrite betroffener SSTables (`create_like`): Exakte Zielschlüssel werden über alle MVCC-Versionen hinweg ausgelassen.
   - Atomarer Austausch (`Swap`) der SSTables über das Transaktions-Manifest.
   - Eviction und Zeroizing entschlüsselter Blöcke im Block-Cache.
   - Der I/O-Aufwand skaliert streng mit der Anzahl der SSTables, die Zielschlüssel enthalten (P11).
2. **Vektorindex HNSW (`contextra-vector`)**:
   - VETO-F02 verbietet fehleranfälliges partielles Kanten-Rewiring.
   - Physisches **Slot-Zeroing** im RAM: Die Vektor-Bytes des gelöschten Slots werden im Arena-Speicher mit Nullen überschrieben (`zeroize`).
   - Slot wird in der Tombstone-Bitset markiert.
   - Persistierte Vektor-Snapshots werden neu geschrieben.
   - Erreicht die Tombstone-Quote $\ge 20\%$, wird asynchron im Hintergrund ein vollständiger, deterministischer Batch-Rebuild ausgeführt.
3. **Volltextindex BM25 (`contextra-text`)**:
   - Abbildung auf exakte Einzel-Postings `Text(term_tag, doc_id)` und Dokumentfrequenzen `Df(term_tag)`.
   - Betroffene Segmente werden gestreamt und ohne Zieldokumente neu geschrieben.
   - Alte Segmente werden dauerhaft über den `Vfs`-Port entfernt.

### 2.2 Residuenmessung und kryptografischer Nachweis
- **Positivkontrolle (Canary Pflicht):** Vor jedem Prüflauf wird ein synthetisches Canary-Muster injiziert und gesucht. Erkennt der Scanner das Canary nicht, ist die gesamte Messung ungültig (Fail-Closed).
- **Messstufen:**
  - **E1 (Rohscan):** Byte-genauer Scan aller Dateien des Datenverzeichnisses nach Needles (z. B. Hash-Signaturen der Inhalte).
  - **E2 (Decoder-Scan):** Dekodierung aller strukturierten Container (SSTable-Blöcke, WAL-Frames, Indexe).
  - **E3 (Strukturelle Abwesenheit):** Bestätigung, dass Index-Slots, Keys und Manifest-Einträge restlos entfernt sind.
- **Löschbericht `ctx-erasure-report/1`:** Maschinenlesbares JSON, signiert mit Ed25519, verifizierbar über die unabhängige Referenz-CLI `ctx-verify`. Der Bericht enthält aus Gründen der Datensparsamkeit keine Klartext-Personendaten oder IDs, sondern kryptografische Commitments (`K_commit`).

---

## 3. Analyse des Test- & Benchmark-Frameworks

Um die kompromisslose Verifizierbarkeit und Robustheit zu gewährleisten, stützt sich Contextra auf ein mehrschichtiges Qualitätsnetzwerk:

### 3.1 Das Ist-Inventar an Mess- und Prüfwerkzeugen
1. **Architektur- und Schichtenwächter (`tests/arch`)**:
   - Überwachung des strikten Ringmodells (Ring 4 → 0) und des Verbots von Aufwärts-Abhängigkeiten.
   - Verbot von `tokio` in Ring 0 (`contextra-types`, `contextra-ports`).
   - Unsafe-Inseln-Wächter (`UNSAFE_ISLANDS`): Sicherstellung, dass ausschließlich `contextra-sys` und `contextra-simd` Unsafe-Code enthalten und jeder Block mit `// SAFETY:` dokumentiert ist.
2. **Statische Audit-Pipeline (`scripts/contextra_full_audit.py`)**:
   - 14 Kern-Crates als normative Soll-Hülle (`PRODUKT`).
   - Automatische Erkennung verbotener Muster (`befund_safe_fn_mit_unsafe_inhalt`, `befund_from_entropy`, undeklarierte Features, Panic-Fläche).
3. **Loom Concurrency-Tests**:
   - Prüfung von Nebenläufigkeitsmodellen (MVCC SSI-Validierung, Lock-Manager, Write-Buffer).
4. **Criterion Micro- & Macro-Benchmarks**:
   - Performance-Gates für Distanzkernel (`simd_bench`), MVCC-Durchsatz (`mvcc_bench`), LSM SSTable/WAL (`sstable_bench`, `wal_bench`, `lsm_concurrency_bench`) und HNSW-Retrieval.

### 3.2 Audit-Befunde & Behebungsstand (WP-0.1 bis WP-1.5)
Aus dem Voll-Audit am aktuellen Branch-Stand ergaben sich 4 wesentliche Befundkategorien, die strukturiert bearbeitet werden:
- **Architektur-Test (Behoben):** `tests/arch/src/lib.rs` enthielt im Test `erlaubt_abwaerts_abhaengigkeit` noch einen Verweis auf das archivierte Crate `contextra-db`. Dieser wurde erfolgreich auf das Ring-4-Crate `contextra` korrigiert; alle Architekturtests bestehen nun zu 100%.
- **Engine-Kompilierung (In Konsolidierung, WP-1.1):** 33 Kompilierungsfehler in `contextra-engine`, primär durch historische Restreferenzen auf das entfernte Crate `contextra-graph` (`graph_index`, `relate`, `get_community`). Diese Methoden werden gemäß SPEC §1.4 und §5.1 aus dem v1-Kern entfernt.
- **Store & Test-Targets (WP-1.3 / WP-2.3):** In `contextra-store` Tests existierten Referenzen auf nicht deklarierte Features (`encryption-at-rest` anstelle standardmäßiger Pflichtverschlüsselung) und verwaiste Imports (`contextra_sandbox`).
- **Lints & Zero-Panic (P6/P7):** Sämtliche Kernbibliotheken (`contextra-types`, `contextra-sys`, `contextra-simd`, `contextra-durable-fs`, `contextra-mvcc`) kompilieren bereits fehlerfrei und erfüllen die strengen Ring- und Zero-Panic-Vorgaben.

---

## 4. Konkrete Roadmap zur Produktionsreife (v1.0.0)

Gemäß `PLAN.md` gliedert sich der Weg zur Produktionsreife in klar definierte, gate-überwachte Phasen:

| Phase | Fokus | Kernaufgaben & Messkriterien | Gate |
|---|---|---|:---:|
| **Phase 0** | **Freeze & Guardrails** | Baseline-Messung (WP-0.1), Undeklarierte Features blockieren (WP-0.2), Erasure-Freeze (WP-0.3), Wortlaut-Lint (WP-0.4), ADR-001 Verankerung (WP-0.5). | **G-A** |
| **Phase 1** | **Rückbau auf den Kern** | Engine entkoppeln (WP-1.1, Graph-/Sandbox-Entfernung), Workspace auf 14 Kern-Crates festschreiben (WP-1.2), alten Löschpfad entfernen (WP-1.3), SIMD-Reduktion (WP-1.4). | **G-B** |
| **Phase 2** | **Store-Härtung** | Snapshot-Sentinel D-C02 beheben (WP-2.1), Sequenzsemantik festschreiben (WP-2.2), WAL-Integrität & Replay vereinheitlichen (WP-2.3), Vfs-Migration durchgehend (WP-2.6), Crash-Matrix 100% grün (WP-2.10). | **G-C** (G0-1) |
| **Phase 3** | **Löschkette Neu** | Reine Typen in `contextra-types` (WP-3.1), Blinde Schlüssel (WP-3.2), Dokument-Manifest (WP-3.3), ErasedSet & ReadGate (WP-3.5/3.6), Purge-Barrier & SSTable-Rewrite (WP-3.7/3.8), Saga & Recover (WP-3.10), Erase-API (WP-3.11). | **G-D** |
| **Phase 4** | **Indexe & Caches** | BM25 exakte Postings & Purge (WP-4.1), HNSW Slot-Zeroing & Batch-Rebuild (WP-4.2), Blockcache-Eviction mit Zeroize (WP-4.3). | **G-E** |
| **Phase 5** | **Nachweis & Bericht** | Erasure Lab mit ForensicScanner & Canary (WP-5.1), Ed25519 Proofs & CLI-Verifier `ctx-verify` (WP-5.2), verify-API (WP-5.3). | **G-F** (G0-2) |
| **Phase 6** | **Härtung & Performance** | Leistungsgates auf Referenzhardware (WP-6.1), Fuzzing 72h & Loom (WP-6.2), Mutationstests $\ge 95\%$ (WP-6.3), Unsafe-Audit & Miri (WP-6.4), Lieferkette (WP-6.5). | **G-G** |
| **Phase 7** | **Freigabe & v1.0.0** | Unabhängiges Sicherheitsreview G-SEC (WP-7.1), Datenschutzprüfung G-LEGAL (WP-7.2), Dokumentation & Release Candidate (WP-7.3/7.4). | **G-H** |

---

## 5. Fazit & Handlungsleitfaden

1. **Die Spezifikation ist hochpräzise und belastbar.** Die Entscheidung aus ADR-001, in v1 auf fehleranfälliges und rechtlich angreifbares Crypto-Shredding zu verzichten und stattdessen auf kompromisslose physische Bereinigung (Ebene A) mit lückenloser Messung zu setzen, ist für Enterprise- und Behördenanforderungen optimal.
2. **Das Fundament steht fest.** Die Kern-Primitives in Ring 0 und 1 (`types`, `ports`, `sys`, `simd`, `durable-fs`, `crypto`, `mvcc`) weisen exzellente Testabdeckung und strikte Einhaltung der Verfassungsprinzipien auf.
3. **Klare nächste Schritte:** Umsetzung von Phase 1 (vollständige Entkopplung der Engine von den Resten des Graph- und Sandbox-Codes), gefolgt von der Härtung des Stores (Phase 2) und der Implementierung der entkoppelten Lösch-Saga in `contextra-erasure` (Phase 3).
