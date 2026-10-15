# Contextra — Endprodukt-Spezifikation (SOLL-Zustand, Single Source of Truth)

**Version:** 1.4 · **Stand:** 14.10.2026 · **IST-Basis:** `tfufuz1/contextra` @ `f00bb2a` (Spec-Basis), Re-Verifikation @ `90b23431` (HEAD des Klons, Merge PR #4791, 14.10.2026 23:40 +0200)
**Rolle:** Principal Senior Rust Architect (finale Entscheidungsinstanz)
**Sprache der Spezifikation:** Deutsch · Code-Bezeichner, Pfade und Invarianten-IDs bleiben unverändert.

> Dieses Dokument ist in sich geschlossen. Es verweist auf kein anderes Planungsdokument. Verweise auf Quellcode-Pfade beziehen sich auf das Repository. Alle Aussagen über den IST-Zustand sind als solche markiert und beziehen sich auf den oben genannten Commit.

---

## 0. Gebrauchsanweisung (für Menschen und Agenten)

### 0.1 Zweck
Dieses Dokument legt fest, **was Contextra sein soll**, **warum**, **was dafür am Code zu ändern ist** und **in welcher Reihenfolge**. Es wird jeder Agentenanfrage beigelegt. Es ersetzt alle früheren Architektur-, Algorithmen-, Löschnachweis- und Produktpläne.

### 0.2 Rangordnung bei Konflikten
1. Harte Governance-Dateien des Repos (`CONSTITUTION.md`, `AGENTS.md`, `VETOES.md`) gelten weiter. Widerspricht diese Spezifikation einer davon, wird zuerst ein ADR (§13) beschlossen, das die Governance-Datei ändert. Kein Agent löst einen solchen Konflikt eigenmächtig.
2. Diese Spezifikation (SOLL).
3. Der Code (IST). Weicht der Code von der Spezifikation ab, ist das entweder ein Bug im Code oder eine fehlende Spezifikationsänderung. Beides wird per PR behoben. Der Code gilt nie stillschweigend als Wahrheit.
4. `capabilities.toml` ist die maschinenlesbare Kopie der Ring-Tabelle (§4.1). Sie wird aus dieser Spezifikation synchronisiert und per Gate geprüft (G-04).

### 0.3 Regeln für Agenten
- **R-A1:** Pro PR genau ein Arbeitspaket (AP, §12). Der AP nennt Dateien, Akzeptanzkriterien und Gates. Außerhalb des AP wird nichts geändert.
- **R-A2:** Ein AP ist erst fertig, wenn seine Akzeptanzkriterien als Test oder Gate laufen. Beschreibung ohne Test zählt nicht.
- **R-A3:** Jeder neue `pub`-Symbol braucht einen Produktionsaufrufer und einen Test über die Fassade (REG-06).
- **R-A4:** Wo diese Spezifikation „Re-Verifikation" verlangt, wird der IST-Zustand zuerst gemessen und das Ergebnis im PR festgehalten. Erst danach wird geändert.
- **R-A5:** Offene Punkte (§14) werden nicht geraten. Sie werden dem Principal Architect als Frage vorgelegt.
- **R-A6:** Rechtliche und normative Aussagen (DSGVO, Beweiskraft, BSI) dürfen nur in der Form aus §5.14 in Doku, Fehlermeldungen oder Marketing erscheinen.

### 0.4 Verifikationsstatus dieser Spezifikation
- Geprüft durch Lesen des Quellcodes: Ring-Zuordnung, Feature-Definitionen, `KvDeleteMode`, `PhysicalScope`, `erase_document`, Builder-Wiring, MCP-Abhängigkeiten, MCP-Mandantenfallback, Mutex-Typ des Group-Commit, Gate-Registry, Unwrap-Ratsche.
- Als gemessen übernommen (bei M0 erneut zu messen, AP-M0.3): `cargo check --locked` grün für `contextra-engine` und `contextra`; Hüllengröße 26; `unsafe`-Verteilung; Zählungen zu `Instant::now`, FMA, `partial_cmp`.
- **Nicht** ausgeführt: Tests, Loom, Miri, Fuzzing, Mutation, Benchmarks, Clippy, NEON-Pfad, `--all-features`.
- Mathematische Aussagen (§9) sind analytisch aus den Formeln abgeleitet und noch nicht durch Experimente belegt.

---

### 0.5 Änderungsprotokoll v1.1 (Integration der Anhänge)

Diese Version integriert die Planungsdokumente `ADVANCED_ALGORITHMS_SPEC_v1`, `ARCHITECTURE_PLAN_v1`, `ARCHITEKTUR_MASTERPLAN_v2`, `ARCHITEKTUR_OPTIMIERUNG_v1`, `IMPLEMENTIERUNGSSPEZIFIKATION` und `SOTA_ALGORITHMIC_FEINSPEZIFIKATION_2026` sowie die Recherche „Algorithmische Spezifikation für deterministische Retrieval-Systeme“. Übernommen wurde nur, was gegen die Rangordnung (§0.2) und die Leitregeln (§2) geprüft ist. Jede Übernahme, Korrektur und Ablehnung ist in §13 als E38 bis E52 dokumentiert. Die Begründungen der Ablehnungen und die Detailbefunde stehen in der Begleitdatei `ANHANG_ANALYSE_v1.md`.

**Baseline-Konflikt.** Die Anhänge beziehen sich auf die Commits `500e9246` und `16764583`, diese Spezifikation auf `f00bb2a`. Compile- und Zählbefunde der Anhänge sind deshalb Hypothesen. M0.3 misst am gepinnten Commit.

**Wesentliche Änderungen gegenüber v1.0:**
- Layer 40 (Egress-Audit, EP6) wird in §6.3 geführt. v1.0 verwendete diese ID in §5.10, führte sie aber nicht in §6.3 auf. Das war ein interner Widerspruch.
- Ringzuordnung (§4.1) ist die einzige Quelle. Abweichende Ringe in Anhängen werden verworfen (E51).
- Conformal (§9.3): Ranggrenze ohne Clamp, Kalibrierset nur aus relevanten Dokumenten.
- DiBud (§9.2): korrekte obere Schranke für unerschöpfte Kanäle, kein Tie-Break auf Schranken.
- KV (§9.4): Namensraum `Irminsul` nur für die Eviction-Strategie. Positionsunabhängiges Caching ist ein eigener, mandantengebundener Kandidat.
- OpenIE (§9.5): kein `temperature=0`-Determinismus, Ereignis-Cache im Löschinventar.
- Bandit (§9.7): keine Regret-Zusage.
- Serialisierbarkeit (§9.9a): ESSN als Kandidat hinter Benchmark-Gate, Default bleibt SSI.
- Latenz-Drift (§9.9b): Catoni-Kandidat für EP5.
- Spektral (§9.9c): Wächter und Provenienzpflicht.
- Kryptographie (§5.13): „unwiderruflich" erst nach Stufe 2 (E50).
- Offene Punkte O-9 bis O-13 (§14.1).

### 0.6 Änderungsprotokoll v1.2 (Re-Verifikation am Klon)

Die Spezifikation v1.1 wurde gegen den tatsächlichen Klon `tfufuz1/contextra` @ `90b23431` geprüft. Der Klon erfolgte flach (`--depth 1`, `GIT_LFS_SKIP_SMUDGE=1`). Ergebnisse:

- **Baseline-Drift (F-37).** Der HEAD hat sich seit `f00bb2a` verändert. Die Anhänge verwenden drei verschiedene Commits (`500e9246`, `16764583`, `f00bb2a`). Ab v1.2 gilt: Jeder Befund trägt einen Commit-Pin, und Re-Verifikationen nennen den Hash.
- **Bestätigt, weiter offen am HEAD:** F-01 (Zeugen-Fabrikation in `erase_document`, `delete.rs:186–201`), F-05 (Downcasts), F-13 (MCP-Mandanten-Fallback, `validation.rs:24`; `block_in_place` in `routing.rs:255`), F-16 (`KvDeleteMode::default() = CryptoShred`, `store/src/kv/delete_mode.rs:18`), F-17 (`store` hängt regulär von `contextra-core` ab, `Cargo.toml:41`), F-23 (fehlende Einträge, siehe F-35).
- **Überholt oder teilweise behoben:** F-29 (siehe Befundtabelle), F-31 (`graph → store` ist entfallen; `store → graph` nur noch Dev-Kante, `Cargo.toml:68`), F-03 (Felder von `PhysicalScope` sind privat, Konstruktor aber hinter Feature `backend-impl` gegated, siehe F-33).
- **Neu gefunden:** F-32 bis F-37 (§3.2).
- **Nicht ausgeführt:** `cargo check`, Tests, Loom, Clippy und Hüllenmessung. Die Toolchain `1.89.0` (`rust-toolchain.toml`) war beim Klon noch nicht vollständig synchronisiert. Diese Messungen bleiben AP-M0.3 vorbehalten. Die Hülle 26 ist daher weiter nur übernommen, nicht gemessen.
- **Fehlende Begleitdatei.** `ANHANG_ANALYSE_v1.md` (auf die v1.1 in §0.5 verweist) liegt im Klon unter `docs/14-10-26/` nicht vor. Die Begründungen der Ablehnungen in §13 sind damit nur über die Spezifikation selbst nachvollziehbar. Die Datei ist nachzureichen oder der Verweis zu streichen (AP-M0.1).
- **Präzisierungen:** Die Crate-Zahl in `crates/` beträgt 34. Die Zahl „44 Workspace-Mitglieder" in §3.1 ist nicht reproduzierbar: `Cargo.toml` führt 37 `members` und keine `exclude`-Einträge. Die Angabe wird auf 37 korrigiert (siehe §3.1).

---

### 0.7 Änderungsprotokoll v1.3 (Architekturlücken geschlossen)

Die Architekturentscheidungen E01 bis E52 wurden einzeln gegen Spezifikation und Code geprüft. v1.3 schließt die dabei gefundenen Lücken:

- **Garantien mit Cargo-Durchsetzung:** Feature-Unifikation macht `backend-impl` über den Workspace hinweg sichtbar (E53, G-22).
- **Übergangsregeln für Ringe:** befristete, gezählte Ausnahmen statt stiller Verletzung (§4.3a, E54, G-21).
- **Verbindliche Port-Namen:** `Rng` entfällt zugunsten `EntropyRng`/`SeededRng` (§7.4, E55, G-23).
- **Betrieb:** Einzelprozess-Sperre (E56), Plattform-Tiers (E57, vorläufig), Zeitbasis mit Monotonie-Anker (E59).
- **Bedrohungsmodell** A0 bis A4 als Grundlage aller Claims (§5.16, E58).
- **Integritätsschlüssel** mit ID und Rotation (E60).
- **clippy-Durchsetzung** vervollständigt, Ausnahmen über gezähltes `#[allow]` (E61, G-09).
- **Logs und Tracing** als Nicht-Garantie, Doc-IDs nur keyed hash (E62, G-29).
- **Port-API-Stabilität** vor 1.0 mit semver-Gate (E63, G-26).
- **Layer-Zählung** klargestellt: 15 normative IDs in §6.3, 13 Code-Varianten (E64, O-18).
- Gates G-11 und G-12 sind spezifiziert, aber am HEAD nicht implementiert (F-40).
- Neue Befunde F-39 bis F-51 (§3.2), Arbeitspakete N0.1 bis N6.1 (§12.2), offene Punkte O-17 bis O-19.

---

### 0.8 Änderungsprotokoll v1.4 (Abgleich mit sechs Planungsdokumenten)

Abgeglichen wurden: die Endprodukt-Spezifikation (SOLL-Zustand, `f00bb2a`), die Finalen Architekturentscheidungen FE-01 bis FE-17, die Widerspruchsanalyse, das Entscheidungsregister, die Roadmap und die Umstrukturierungs-Roadmap.

**Korrekturen an v1.3:**
- **Layer-Zählung:** `DeletionLayer` hat 14 Varianten (IDs 1–7 und 20–26). Die Zahl 13 aus v1.3 war ein Zählfehler, die Struct-Variante `WalAllSegments { … }` wurde übersehen. E64, O-18 und F-51 sind korrigiert.
- **Ledger:** Das IST-Ledger speichert nur Keyed-Hashes (`erasure_ledger/entry.rs`). Der in v1.1 bis v1.3 beschriebene Klartext-Intent und die Kompaktierung entfallen (E21 überholt, E65).
- **HNSW:** Die vorgesehene Kantenreparatur bei Löschung kollidiert mit VETO-F02 (bedingt akzeptiert, Review 2027-10-12). Sie entfällt (E68, O-10 erledigt).
- **Zustandsnummern:** Übernahme der Nummerierung 0 bis 8 aus FE-10 (E67). Die Werte 1 bis 9 aus v1.1 entfallen. `Scanning` ist kein eigener Zustand mehr.
- **Rangordnung (§0.2):** Die Governance-Dateien stehen über der Spezifikation. Bedingte Vetos gelten als verbindliche Vorgabe für den betroffenen Pfad.

**Übernommen aus den Dokumenten:**
- Versiegeltes Scope-Journal für Resume (FE-18), E66, §5.18.
- Sichtbarkeit gelöschter Schlüssel während des Hintergrund-Purge und Dispose-Policy `RequireScrub` (E76).
- Eigentümer je Doppeldefinition (FE §3.7, E70, §4.11).
- Stub-Verbot als Doktrin REG-18 (E69), aus Roadmap R7-15, R7-16, R7-18.
- Ack-Schlüssel per HKDF aus dem Integritätsschlüssel (E75). Damit ist der Downgrade-Schutz spezifiziert.
- Kapselung von `TenantId` und `TxId` (E74, R2-02).
- Canary-Abdeckung als Gate G-32 (R1-07).
- Ehrlichkeitsklausel um Journaling, CoW und OverlayFS erweitert (R3-08, §5.14).

**Ergänzt, aber noch nicht entschieden:** Roadmap-Führung (E71), Owner und Kalender (E72), Loom mit drei Szenarien (E73; FE-12 nennt zwei), Tenant-ID-Herkunft außerhalb von stdio (E77), Checkpoint-Determinismus (MP2 §4.9) und Cascade-Invalidierung (GL-G3), die bisher keine Phase haben.

**Verworfen:** `LogicalScope::Keys` und `Prefixes` aus dem FE-Entwurf. Sie widersprechen F-03, weil die Mandantenbindung fehlt. Das Issuer-Token für `PhysicalScope` (FE §5.5) bleibt eine Alternative. Wir behalten das Feature-Gate (E35).

**Nicht geprüft:** VETO-F02-Volltext (nur der Kopf wurde gelesen, O-21), Block-Cache-Stubs (N6.4), Canary-Abdeckung im Detail (N6.2) und EP6-Stubs (nur aus der Roadmap übernommen).

---

## 1. Produktdefinition und Release-Schnitt

### 1.1 Leitmotiv
> **Jede Fähigkeit ist ein Plugin. Was nicht aktiviert ist, existiert weder im Binary noch zur Laufzeit. Der Löschnachweis ist das Kernversprechen, sein Preis ist opt-in.**

### 1.2 Endprodukte

| ID | Produkt | Inhalt | Release |
|---|---|---|---|
| **EP1** | Contextra Memory | Lokaler Mandanten-Speicher, Hybridsuche (Vektor + BM25 + RRF), MCP-Server, Löschung in den Modi `Off` und `Assured` | **v0.1.0**, danach Gate-basiert 1.0.0 |
| **EP2** | Contextra Proof | Modus `Attested`: Layer-Scans, signierter Löschbeweis v4, unabhängiger Verifier, Audit-Export, AVV-Generator | nach EP1 |
| **EP3** | Contextra Retrieval+ | Wissensgraph, PathRAG, OpenIE-Ingestion, Graph-Löschung | nach EP2 |
| **EP4** | Contextra Inference | Lokale Embedder (candle/onnx/ollama), KV-Cache (KIVI, SnapKV, Prefix-Trie, Tier-2) | nach EP3 |
| **EP5** | Contextra Cognition | Adapt (PID, Bandit), Router, Cognition (Konsolidierung), Agent | experimentell |
| **EP6** | Contextra Guard | Privacy/Egress-Schutz, Sandbox (WASI-Merge), Lizenzprüfung | nach Bedarf |

### 1.3 Versionierung
- Erstes Release: **v0.1.0** (Workspace-Version ist `0.1.0`).
- **1.0.0 ist ein Gate, kein Meilenstein.** Es wird vergeben, wenn alle vier Bedingungen erfüllt sind:
  1. Upgrade-Test mit Golden-Store-Fixtures (altes Manifest, `purge/`-Intents, Ledger) grün.
  2. Crash-Matrix CRASH-01 bis CRASH-08 (§5.12) grün.
  3. Rechtliche Formulierungen nach §5.14 geprüft und freigegeben.
  4. Formatstabilitätsregel (§5.9) beschlossen und durch Tests abgesichert.
- Formatstabilität wird erst ab 1.0.0 zugesagt. Vorher gilt: Upgrade-on-open, nie Downgrade.

### 1.4 Was EP1 verspricht und was nicht
**EP1 verspricht:** schnelle lokale Hybridsuche, strikte Mandantentrennung, deterministisches Verhalten, im Modus `Assured` logische und physische Löschung im Speicher des Stores (Tombstone, SSTable-Rewrite, Überschreiben und Entfernen der Altdateien, Anti-Resurrection-Ledger, Restore-Guard).
**EP1 verspricht nicht:** einen signierten Löschbeweis (das ist EP2), Löschung auf SSD-Controller-Ebene, in Backups, Swap, Core-Dumps oder im Prozessspeicher, rechtsverbindliche Beweiskraft.

---

## 2. Verbindliche Leitregeln

Diese Regeln gelten für jeden Code und jede Entscheidung. Ein Verstoß ist ein Merge-Blocker.

| ID | Regel |
|---|---|
| **REG-01** | **Zero-Panic.** Kein `unwrap`, `expect`, `panic!`, `unreachable!` oder unchecked Indexing im Produktivcode von Ring 0 und Ring 1. Test-, Bench- und Fuzz-Code ist ausgenommen. Die Ratsche ist das einzige Gate und darf nur sinken. **Zählregel offen (F-34):** `governance/ratchet.toml` führt drei Zähler (`unwrap_exact` = 502, `unwrap_legacy_grep` = 1175, `expect_exact` = 242); eine naive Suche am HEAD `90b23431` findet 825 `.unwrap()` und 575 `.expect(` außerhalb von Testpfaden, ohne `#[cfg(test)]`-Module auszuschließen. Bis M0.3 eine normierte, parserbasierte Zählregel festlegt, gilt `unwrap_exact` als maßgeblich, und jede Abweichung wird im PR begründet. |
| **REG-02** | **Ring-DAG.** Abhängigkeiten laufen nur von höheren auf niedrigere Ringe (gleicher Ring nur laut Ordnung §4.2). `capabilities.toml` ist die Kopie der Ring-Tabelle. |
| **REG-03** | **Determinismus über Ports.** Zeit, Zufall und IDs kommen ausschließlich über `Clock`, `EntropyRng`/`SeededRng`, `IdGenerator` aus `contextra-ports` (Namen verbindlich nach E55). Direkte Aufrufe von `Instant::now`, `SystemTime::now`, `thread_rng`, `rand::random` sind außerhalb der Port-Implementierungen und außerhalb von Tests/Benches verboten (Gate G-09). |
| **REG-04** | **Drei Unsafe-Inseln:** `contextra-sys`, `contextra-simd`, `contextra-wire`. Kein `unsafe` anderswo. Jedes `unsafe`-Fragment hat einen `// SAFETY:`-Kommentar. Für generierten Code in `wire` gilt eine explizite, dokumentierte Ausnahme (kein Kommentar je Block, dafür ein Datei-Header und ein Generator-Test). Alle übrigen Crates tragen `#![forbid(unsafe_code)]`. |
| **REG-05** | **Single-Node.** Keine Multi-Node-, Sharding-, Raft- oder Cluster-Logik. „Leader" bedeutet ausschließlich den Group-Commit-Leader im Store. Es gibt keine Replikas. |
| **REG-06** | **Wirksamkeit.** Kein `pub`-Symbol ohne Produktionsaufrufer und ohne End-to-End-Test über die Fassade. Entweder verdrahten, auf `pub(crate)` reduzieren oder löschen. |
| **REG-07** | **Opt-in-Kosten.** Nicht aktivierte Features verursachen weder Code im Binary noch CPU-Kosten. Jedes Feature kompiliert isoliert (Gate G-06). |
| **REG-08** | **Fail-closed.** Fehler, Unbekanntes und Unvollständiges führen zu Ablehnung, nie zu stillem Erfolg. Das gilt für Scans, Resume, Lizenz, Mandantenbindung, Speicherplatzprobe, Manifest-Prüfung. |
| **REG-09** | **Builder-Ehrlichkeit.** Jeder Builder-Schalter hat eine nachweisbare Wirkung. `build()` validiert die Verdrahtung gegen den tatsächlichen Zustand, nie gegen hartcodierte Werte. |
| **REG-10** | **Snapshot-Konsistenz.** Alle Signale einer Anfrage lesen denselben Sequenzstand über ein `ReadSnapshot`-Token. |
| **REG-11** | **Sperrenordnung.** Die globale Lock-Reihenfolge aus §7.1 gilt crate-übergreifend. Synchrone Locks werden nie über `.await` gehalten. |
| **REG-12** | **Tool-Klassifikation.** Jedes MCP-Tool ist klassifiziert und feature-gated. Ein Tool ohne Klassifikation wird nicht registriert. |
| **REG-13** | **Erasure-Opt-in und Ports-Only.** Kern-Crates importieren nie Erasure-Plugin- oder Evidence-Crates. Sie sprechen nur Ports aus `contextra-ports`. |
| **REG-14** | **Fehlerfreie Evidenz.** Ein Zeuge oder Beweis entsteht nur aus einem real ausgeführten Scan. Es gibt keine Konstanten-, Default- oder `None`-Zeugen. Fehler ergeben keinen Beweis. |
| **REG-15** | **Tenant-Matrix.** Jeder Löschtest läuft über `LsmStorage` **und** über `TenantScopedStorage`. |
| **REG-16** | **Ehrliche Claims.** Dokumentation, Fehlermeldungen und API-Namen behaupten nie mehr als technisch bewiesen ist (§5.14). |
| **REG-17** | **Formate nie downgraden.** Jedes persistente Format ist versioniert. Öffnen upgradet, ein Downgrade wird abgelehnt. |
| **REG-18** | **Keine Stubs mit Erfolgsrückgabe.** Eine Funktion ohne Implementierung darf in Produktion nie `Ok(leer)`, eine unveränderte Eingabe oder `None` als Erfolg zurückgeben. Sie liefert einen Fehler (fail-closed, REG-08). Gate G-30 (E69). |

---

## 3. IST-Zustand und Befundregister

### 3.1 Strukturdaten (IST @ `f00bb2a`)
- 34 Crates in `crates/` (IST @ `90b23431`), dazu Tooling-Mitglieder (`xtask`, `xtask-heavy`, `benchmarks/contextra-bench`). `Cargo.toml` führt 37 `members`, keine `exclude`-Einträge. Die frühere Angabe „44 Workspace-Mitglieder" (inkl. 7 Fuzz-Crates) ist am HEAD nicht reproduzierbar und gilt als korrigiert. Rund 380k Zeilen Rust (Schätzung, nicht gemessen).
- Hülle des EP1-Binaries (`contextra-mcp`, Default-Features): 26 Crates. Mit `--no-default-features` ebenfalls 26.
- `contextra-core` ist eine tote Fassade: 139 Dateien in `src/`, 404 Dateien im Workspace referenzieren `contextra_core`, 11 `Cargo.toml` hängen davon ab. Viele Symbole kommen per `pub use contextra_ports::*` hindurch.
- Fassade `contextra`: `default = ["fast", "candle"]`. Engine: `default = ["graph","rank","adapt","kvcache","candle"]`.
- Das Löschpfad-Gerüst ist am HEAD `90b23431` **teilweise implementiert** (korrigiert gegenüber v1.1, F-38): `contextra-ports/src/erasure.rs` hat 514 Zeilen (v1.1: 223). Implementiert sind `ErasureBackend` für `LsmStorage` (`contextra-store/src/lsm/erasure_backend.rs`) und `RestoreGuard` (`StreamingRestoreGuard`, `contextra-store/src/restore_guard.rs`). Ohne Implementierung bleiben `LayerScan`, `ErasureHooks` und `WriteGate`. `FreeSpaceProbe` ist doppelt definiert (F-32). `Attested` ist nur eine Enum-Variante.
- Store enthält bereits: `erasure_ledger/`, `lsm/purge/` (`intent.rs`, `resume.rs`, `orchestrator.rs`), `lsm/fence.rs`, `lsm/witness.rs`, `ManifestFormatTracker` mit `FormatError::DowngradeAttempt`.
- Ring laut `capabilities.toml` (maßgeblich für IST): Ring 0: `types, ports, wire, sys, durable-fs, simd, mvcc, text, rank, vector, graph, crypto, adapt, audit-export, avv-generator, core`. Ring 1: `store, checkpoint, kvcache`. Ring 2: `infer-candle, infer-ollama, infer-onnx, sandbox`. Ring 3: `engine, db, privacy, cognition, router, agent`. Ring 4: `contextra, mcp, py, license`. Tooling: `testkit, xtask-heavy`. **Nicht eingetragen** (F-35): `xtask` und `benchmarks/contextra-bench`. Die Fuzz-Crates sind nicht per `capabilities.toml` erfasst.

### 3.2 Befundregister

Schweregrad: 🔴 Blocker (verletzt das Kernversprechen oder Sicherheitsinvarianten), 🟠 hoch, 🟡 mittel. „Re-V." = Befund stammt aus einer älteren Messung und muss bei M0 am HEAD neu verifiziert werden (AP-M0.3), bevor er behoben wird.

| ID | Sev | Befund (IST) | Soll-Behebung | AP |
|---|---|---|---|---|
| **F-01** | 🔴 | `erase_document` (`contextra-engine/src/collection/crud/delete.rs`) ruft **kein** `ledger_append` auf (nur Kommentare), stellt aber einen `ErasureLedger`-Zeugen mit `entry_confirmed: true` und `decision_chain_mac: [0u8; 32]` aus. Text- und Relations-Zeugen werden mit `None` gebaut, Fehler per `if let Ok` verschluckt. | Zeugen-Fabrikation ersatzlos entfernen. Ohne echten Scan kein Zeuge (REG-14). | M0.2 |
| **F-02** | 🔴 | Keine API-Semantik für Teilerfolg: Nach Ledger-Eintrag setzt Recovery den Tombstone durch, auch wenn der Aufrufer zuvor ein `Err` sah (SSI-Konflikt, I/O, Cancellation). Im Code kann `index.delete(..).await?` nach erfolgreichem `db_tx.commit()` noch `Err` liefern. | Commit-Punkt-Semantik und `ErasureIndeterminate` (§5.3). Cancellation-sicherer Löschpfad. | M2.6 |
| **F-03** | 🔴 | `PhysicalScope` (`ports/src/erasure.rs`) hat `pub`-Felder, `pub fn new`, `Default`, `Deserialize`. Leerer Prefix matcht jeden Key. `LogicalScope::Keys/Prefixes` tragen keine `TenantId`. `matches_key` ist linear je Key. | Opaker Typ mit Invarianten (§5.4). | M1.1 |
| **F-04** | 🔴 | Port-Typen erfüllen die Zielinvarianten nicht: `LayerObservation` ohne Digest (nur `details: String`); `WriteGate::check_allowed(key)` je Key; `HookToken` ist `Copy` und öffentlich; `purged(scans)` akzeptiert leere Liste; `FreeSpaceProbe` fehlt im Port; Zeitstempel sind Wall-Clock; `layer_id` ist `u8`. | Port-Neudefinition (§5.5). Es gibt keine Implementierungen, die Änderung ist billig. | M1.2 |
| **F-05** | 🔴 | Tenant-Pfad umgeht Fence und Zeugen: `delete.rs` castet per `downcast_ref::<LsmStorage>()`; unter `TenantScopedStorage` ergibt das `None`. `tenant_id()` liefert per `unwrap_or_default()` still einen Default-Mandanten, `kv_hooks.remove_doc_segments` kann den falschen Mandanten treffen. Am HEAD `90b23431` außerhalb von Tests 15 `downcast_ref`-Stellen, davon 5 im Engine-Pfad (`collection/mod.rs` ×3, `collection/crud/delete.rs`, `transaction/db_transaction_commit.rs`). Weiter offen. | `ErasureBackend` über `as_erasure_backend()`, fail-closed Mandantenauflösung (§5.11). | M2.3 |
| **F-06** | 🔴 | Purge-Korrektheit (Re-V.): obsolete SSTables bleiben unscrubbt (L4), Lesefehler werden verschluckt (L5), Rang/Seq-Grenzen gehen beim Rewrite verloren (L6), `None`-Probe wird als 0 Byte frei gelesen (L8), Purge trifft logische statt physische Schlüssel (N-K), eine Intent-Datei je Store kollidiert (N-E), `Vec`+Doppellesen (N-F), Orphan-GC ohne Scrub (N-G), Failpoints per Umgebungsvariable (N-J). | Purge v2 (§5.7). | M2.1 |
| **F-07** | 🔴 | Abgeleitete Daten liegen außerhalb des Löschumfangs (§5.10): HNSW-mmap-Knoten (im Code als „explicit residual risk" markiert, `core_rebuild.rs`), `quantize`, `rabitq`, `diskann`, KV-Cache-Tier-2, Prefix-Trie, Graph-Merge, inkrementelle SVD, Conformal-Kalibrierset, Bandit-Zustand, LLM-Triples, aktives WAL-Segment (Klartext bis Rotation), Snapshot-Pins (ein langer Reader blockiert Zeroisierung unbegrenzt). | Inventar mit Strategie je Speicher (§5.10), Pin-Policy, WAL-Rotation. | M5.7–M5.9 |
| **F-08** | 🟠 | Beweisformat inkonsistent: 7 Layer (`u8`, `[LayerProof;7]`) gegen LayerIds 1..7/20/21/26/40; Public Key liegt im Beweis (Selbstzertifizierung); `proof_v3.rs` enthält `construct_v4_payload` mit bincode unter dem v4-Domain-Tag (zwei Serialisierungen, ein Tag); `verify` statt `verify_strict`; `bincode` im Signaturpfad. | Beweis v4 normativ (§6), v3-Erzeugung löschen. | M1.4, M7 |
| **F-09** | 🟠 | Einzel-Dokument-Löschung läuft über Rewrite und Zeroing, den auf SSD/CoW schwächsten Mechanismus; Crypto-Erase nur auf Collection/Tenant. Schlüssel-Backup mit Wartefrist würde Shredding aushebeln. | Zwei-Stufen-Crypto-Erase, kein DEK-Backup, ehrliche Claims (§5.13, §5.14). | M5.6 |
| **F-10** | 🟠 | Keine Hochwassermarke (`erasure_high_water_mark` 0 Treffer). Manifest-Integrität (MAC) unspezifiziert. Verhalten bei Lizenzablauf unklar. | §5.9, §5.15. | M5.1 |
| **F-11** | 🔴 | Fence-TOCTOU: `ErasureFence` (`parking_lot::RwLock`) wird nur am Eingang von `put`/`delete` geprüft, vor dem Group-Commit. `commit_mutex` ist `tokio::sync::Mutex` und wird vor dem WAL-I/O freigegeben und für den Memtable-Publish neu erworben. Zwischen Check und Publish kann eine Fence aktiv werden. | Fence-Protokoll B mit Re-Check im Publish-Fenster (§5.8). | M4.1 |
| **F-12** | 🟠 | Lock-Hierarchie unvollständig dokumentiert; Regel „keine Locks über `.await`" ist falsch formuliert (`erase_document` hält tokio-`kv_locks` über mehrere Awaits inkl. Commit). | §7.1. | M4.3 |
| **F-13** | 🔴 | MCP: `validation.rs` liest `CONTEXTRA_TENANT_ID` und fällt auf `TenantId::SYSTEM` zurück (fail-open). `server_dispatch.rs` nutzt `block_in_place`+`block_on`. `contextra-ports` mit `test-utils` steht in `[dependencies]`. Harte Abhängigkeiten auf `contextra[router, encryption-at-rest]`, `adapt`, `crypto`, `privacy`. Features `fast/guard/graph` fehlen. | §8. | M3.6 |
| **F-14** | 🟠 | Hülle 26 statt ≤ 17: `candle` im Default, Engine-Defaults ziehen `graph/adapt/kvcache/candle`, MCP bindet Parkplatz-Crates hart. | §4.4, §4.5. | M3.2–M3.4 |
| **F-15** | 🟠 | `verify_wiring` existiert (`wiring_selftest.rs`), ist aber tautologisch: `builder.rs` übergibt `has_license_gate = true` hartcodiert und verwirft das Ergebnis (`let _report`). Sieben Builder-Schalter (`with_privacy`, `with_adapt`, `with_kms`, `with_kv_shredding`, `with_checkpoint`, `with_avv_generator`, `with_audit_export`) haben keine Wirkung. | §4.7. | M3.5 |
| **F-16** | 🟠 | `KvDeleteMode::default()` ist `CryptoShred` (`store/src/kv/delete_mode.rs`), Test `test_kv_delete_mode_default` zementiert das. | Default `TombstoneOnly`; Builder setzt `CryptoShred` bei `encryption-at-rest`. Re-V. @ `90b23431`: Default weiterhin `CryptoShred` (`store/src/kv/delete_mode.rs:18`). | M1.5 |
| **F-17** | 🟠 | `contextra-core` tote Fassade (s. 3.1). | Ersatzlos entfernen per Skript-PRs (§4.9). | M3.1 |
| **F-18** | 🟠 | Vier parallele Ringmodelle (Tabellen in Plänen, `capabilities.toml`, Layer 0–10 in `SOURCE_OF_TRUTH.md`). 15 von 21 vergleichbaren Crates widersprechen `capabilities.toml`. Plan nennt Unsafe-Inseln `vector/kvcache/store`, die kein `unsafe` haben. | Ein Modell (§4.1–§4.2). | M0.4, M4.5 |
| **F-19** | 🟡 | Alle 46 `unsafe` in `wire` liegen in `contextra_generated.rs` ohne `// SAFETY`. Ein Plan wollte `unsafe` in `kvcache` (`aligned_alloc.rs`). | REG-04; `kvcache` bleibt `forbid(unsafe_code)`, ausgerichtete Allokation als sichere Fassade in `sys`. | M4.6 |
| **F-20** | 🟠 | Determinismus nicht durchgesetzt: `Instant::now` ~81, `SystemTime::now` 7, `thread_rng` 6 Treffer (grob, inkl. Inline-Tests); kein Workspace-`clippy.toml`; Workspace-Lints ohne `indexing_slicing`, `arithmetic_side_effects`, `float_cmp`; `partial_cmp` 38× in `src/`; 25 FMA-Stellen in `simd` (u. a. NEON `vfmaq_f32`); `ReadSnapshot` plante `Instant`. | §7.4–§7.6, Gate G-09. | M4.7 |
| **F-21** | 🟠 | 512 `pub fn` ohne Aufrufer, 1.057 undokumentierte Schnittstellen. Vier Reaper (`start_expiry_reaper`, `start_orphan_reaper`, `start_thermostat_reaper`, `start_kv_cache_eviction`) werden im `open`-Pfad nie gestartet. | §10. | M3.7, M6.4 |
| **F-22** | 🟠 | Einzelbefunde: `apply_temporal_validity_filter_at` castet Unix-Zeit in `TxId`; `PendingDeleteJournal::process` verwirft `let _ = self.flush()`; `kv_shredding.rs:247` offenes TODO (lockfreie Gruppen-Revokation); `AgentContext.status` ist `pub`; `engine`/`store` binden `sandbox` (Ring-Verletzung). | §10.2. | M3.7 |
| **F-23** | 🟡 | Governance: `deletion-guarantee-gate` fehlt in `xtask/registry.toml`; doppelte/abweichende Dokumentvarianten; absolute `file:///`-Links; `contextra-bench` und Fuzz-Crates fehlen in `capabilities.toml`. | M0.4, §11. | M0.4 | <!-- doc-ref-ignore -->
| **F-24** | 🟠 | `docid-128` ändert das On-Disk-Format und verträgt sich nicht mit „jedes Feature isoliert". | Format-Feature mit Manifest-Eintrag (§4.5). | M4.4 |
| **F-25** | 🟡 | 1-%-p99-Gate auf CI-Hardware ist statistisch nicht messbar. | Instruktionszähler plus Toleranz (§11.3). | M4.2 |
| **F-26** | 🟠 | Mathematische Fehler und Lücken in Algorithmenbeschreibungen (§9). | §9 normativ. | M8–M10 |
| **F-27** | 🟠 | Ledger: IST-Format weicht vom Zielformat ab (`ErasureLedger`/`ledger_append` im Store vorhanden, Zielformat nicht). Unbegrenztes Wachstum, Verhalten bei fehlendem MAC-Schlüssel und Kompaktierung ungeklärt. | §5.9. | M5.2 |
| **F-28** | 🟠 | Krypto-Aussagen widersprüchlich (AES-GCM-SIV gegen AES-256-GCM; „deterministisches CSPRNG-Schlüsselmaterial"; „BSI-TR-02102-konform"; „rechtsverbindlich nach § 371a ZPO"). | §5.13, §5.14. | M0.5 |
| **F-29** | 🟡 | Annahmen über Kompilierfehler (Engine/Fassade), ein stets `CapabilityUnsupported` liefernden `Collection::delete()` und ein noch fehlendes `erasure.rs` treffen am HEAD nicht zu: `delete()` übergibt `assurance: S0`, `erasure.rs` existiert (223 Zeilen), `cargo check --locked` ist grün. | Messen statt annehmen. **Re-V. @ `90b23431` (v1.2):** Die Aussage „keine `impl`" ist überholt; `erasure.rs` hat 514 Zeilen; Implementierungen siehe §3.1 (F-38). | M0.3 |
| **F-30** | 🟠 | `contextra-crypto` (7.669 Zeilen) bündelt Schlüsselverwaltung, KV-Verschlüsselung, Löschbeweis, Audit-Kette. Store, Vector, Engine, Kvcache u. a. hängen daran; Beweiscode landet im EP1-Binary. Fuzz-Targets (`deletion_proof_tamper` u. a.) liegen in `crypto`. | Crate-Split (§4.8). | M5.5 |

| **F-31** | 🟠 | Ring-Verletzungen im IST (aus `Cargo.toml` gelesen; reguläre und Dev-Kanten in M0.3 trennen): `store`(1)→`sandbox`(2); `graph`(0)→`store`(1) und `store`→`graph` (Zyklus-Verdacht über Dev-Kante); `engine`(3)→`license`(4); `engine`(3)→`sandbox`(2) ist erlaubt, aber ungeplant hart; `db`(3)→`cognition`(3) gegen die Ordnung; `checkpoint`→`store` im selben Ring. | Auflösung per Ports und Ordnung (§4.3). **Re-V. @ `90b23431` (v1.2):** `graph → store` entfallen; `store → graph` nur noch Dev-Kante (`contextra-store/Cargo.toml:68`); `store → sandbox` nicht mehr vorhanden; `store → core` regulär vorhanden (siehe F-36). Dev-Kante auf Zyklusfreiheit prüfen (M0.3). | M4.5 |
| **F-32** | 🟠 | `FreeSpaceProbe` ist doppelt definiert: als Port in `contextra-ports/src/erasure.rs` und als store-lokaler Trait `contextra-store/src/compaction/purge_filter.rs:14` mit eigener Impl `StatvfsSpaceProbe`. Zwei Traits gleichen Namens ziehen Aufrufer in verschiedene Richtungen. | Store-lokalen Trait entfernen; `StatvfsSpaceProbe` als Adapter des Port-Traits in `store` (oder `sys`) implementieren. Fail-closed bei `None` (REG-08). | M1.2 |
| **F-33** | 🟠 | `PhysicalScope` (`ports/src/erasure.rs:100–112`): Felder privat, aber der validierende Konstruktor ist hinter Feature `backend-impl` gegated. Die Invarianten (nicht leer, Mandantenpräfix, kanonisch) existieren daher nur in Builds mit diesem Feature. Das widerspricht REG-07 (Feature-Isolation) und dem Ziel, dass Invarianten nicht von Feature-Sätzen abhängen. `Serialize` ist abgeleitet; `Deserialize` ist am HEAD zu prüfen. | Konstruktor und Invariantenprüfung immer verfügbar. Nur die Backend-Implementierung bleibt hinter dem Feature. `Deserialize` nur über den validierenden Pfad (`TryFrom`). Tests in Default-Build und in Feature-Build. | M1.1 |
| **F-34** | 🟠 | Unwrap-Zählung hat drei Regeln: `unwrap_exact` = 502, `unwrap_legacy_grep` = 1175, `expect_exact` = 242 (`governance/ratchet.toml`). Naive Zählung am HEAD `90b23431`: 825 `.unwrap()` und 575 `.expect(` außerhalb von Testpfaden; `#[cfg(test)]`-Module inline sind dort mitgezählt. REG-01 nennt nur einen Wert. Die Ratsche ist damit nicht reproduzierbar. | Ein parserbasierter Zähler (z. B. `syn`-Visitor) mit definierten Ausschlüssen (`#[cfg(test)]`, Doc-Kommentare, `tests/`, `benches/`, `fuzz/`). Alle drei Werte im Gate ausweisen. Zahl erst nach Messung festschreiben. | M0.3, M4 |
| **F-35** | 🟠 | `capabilities.toml` erfasst `xtask` (Standardbinary, Workspace-Member) und `benchmarks/contextra-bench` (Workspace-Member) nicht. Präzisiert F-23. Ohne Eintrag gibt es für diese Crates keinen Ring und keine Maturity. | Beide Einträge als Tooling anlegen; G-04 erweitern, sodass jedes Workspace-Member einen Eintrag braucht. | M0.4 |
| **F-36** | 🟠 | `contextra-store` hängt regulär von `contextra-core` ab (`contextra-store/Cargo.toml:41`). Die tote Fassade (F-17) kann daher nicht einfach gelöscht werden; die Abhängigkeit ist zuerst zu entkoppeln. Die Kante `graph → store` ist entfallen, deshalb ist `graph` jetzt frei von Store-Abhängigkeiten. | Reihenfolge: (1) `store` von `core` entkoppeln, (2) `core`-Symbole über Ports oder `types` führen, (3) Crate löschen. Jeder Schritt mit eigenem PR und Test (R-A1). | M3.1 |
| **F-37** | 🟡 | Baseline-Drift: Spezifikation (`f00bb2a`), Anhänge (`500e9246`, `16764583`) und HEAD des Klons (`90b23431`) stimmen nicht überein. Befunde ohne Commit-Pin sind nicht prüfbar. | Jede Befundtabelle trägt einen Commit-Pin. Ein Gate `baseline-pin` schreibt den Hash der Messung in `docs/baseline/` und bricht bei Abweichung ab. | M0.3 |
| **F-38** | 🟠 | Re-Verifikation @ `90b23431`: Die Aussage „keine `impl` für Erasure-Ports" aus v1.1 ist überholt. Implementiert sind `ErasureBackend` für `LsmStorage` und `RestoreGuard` (`StreamingRestoreGuard`). Nicht implementiert sind `LayerScan`, `ErasureHooks` und `WriteGate`. F-01, F-05, F-13, F-16, F-17 bestehen unverändert. | Die Befundtabelle ist bei M0.3 mit diesem Stand zu aktualisieren. Die Arbeitspakete M1–M2 werden auf die tatsächlichen Lücken zugeschnitten. | M0.3 |
| **F-39** | 🟠 | Port-Namen divergieren: Spezifikation nennt `Rng`, Code hat `EntropyRng` und `SeededRng` (`contextra-ports/src/rng.rs`). Die Mehrdeutigkeit betrifft die Trennung von Entropie und Test (§5.13). | Namen nach §7.4 verbindlich machen (E55). | N1.1 |
| **F-40** | 🟠 | Gates G-11 (`check-no-downcast`) und G-12 (`check-port-features`) sind spezifiziert, aber in `xtask` am HEAD nicht implementiert. | Implementieren oder Status in `xtask/registry.toml` ehrlich führen (N0.3). | N0.3, N1.1 |
| **F-41** | 🟠 | Feature-Unifikation: `backend-impl` wird über die Selbst-Dev-Dependency in `contextra-ports/Cargo.toml:30` aktiviert. G-12 ist so nicht durchsetzbar. | Feature-Baum-Gate (E53, G-22). | N1.1 |
| **F-42** | 🟠 | Ringverletzungen ohne Ausnahmeregel: `store → contextra-core` (`Cargo.toml:41`), `store → contextra-crypto` (`Cargo.toml:45`). §4.3 verbietet die zweite Kante, der Split liegt in M5. | Befristete Ausnahmen (§4.3a, E54, G-21). | N0.2, N3.1, N5.2 |
| **F-43** | 🔴 | Keine Einzelprozess-Sperre im Store-Open-Pfad (kein `flock`, keine `LOCK`-Datei gefunden). Zwei Prozesse auf einem Verzeichnis können WAL und Manifest beschädigen. | `flock` auf `<store>/LOCK`, Fehler `StoreLocked` (§5.17, E56, G-24). | N4.1 |
| **F-44** | 🟠 | Plattformannahmen nur für Linux im Code (`cfg(target_os = "linux")` in `durable-fs` und `sys`). Der Claim `Assured` hat keinen Geltungsbereich. | Plattform-Tiers (§5.17, E57, G-25). | N6.1 |
| **F-45** | 🟠 | Zeitbasis: Quarantäne und Ledger-Zeit hängen an der Wanduhr. Eine Rücksetzung ist nicht erkennbar; der Monotonie-Anker fehlt. | Zeitanker und Quarantäne in Ticks (§5.9, E59, G-28). | N5.1 |
| **F-46** | 🟠 | Integritätsschlüssel ohne ID und Rotation. Verlust oder Wechsel ist nicht spezifiziert. | ID, Rotation, verify-only-Keyring (§5.9, E60). | N5.1 |
| **F-47** | 🟠 | Kein Bedrohungsmodell. Claims in §5.14 sind nicht an Angreiferklassen gebunden. | Modell A0 bis A4 (§5.16, E58). | N0.1 |
| **F-48** | 🟡 | clippy-Verbote unvollständig: `SystemTime::now` und `rand::random` fehlen in `clippy.toml`. Der Ausnahmemechanismus ist ungeklärt. | Ergänzung und `#[allow]`-Regel (§7.5, E61, G-09). | N4.2 |
| **F-49** | 🟡 | Logs und Tracing liegen außerhalb des Löschumfangs, ohne Festlegung. Doc-IDs stehen im Klartext. | Nicht-Garantie und keyed hash (§5.2, E62, G-29). | N6.1 |
| **F-50** | 🟡 | Keine Port-API-Stabilitätsregel und kein semver-Gate. | E63, G-26. | N1.2 |
| **F-51** | ✅ | Layer-Zählung: v1.3 nannte 13 Varianten. Der Code hat 14 (`contextra-crypto/src/deletion_proof/types.rs`). | E78, O-18 erledigt. | — |
| **F-52** | 🟠 | Ledger-Spezifikation (v1.1 bis v1.3) widersprach dem IST: Klartext-Intent im Ledger, Kompaktierung, neues Binärformat. | E65 | N5.3 |
| **F-53** | 🟠 | HNSW-Soll mit Kantenreparatur widersprach VETO-F02 (bedingt akzeptiert, Review 2027-10-12). | E68, O-10 | N0.5 |
| **F-54** | 🟡 | Zustandsnummern in v1.1 und FE-10 unterschiedlich (1 bis 9 gegen 0 bis 8). | E67 | §5.6 (erledigt) |
| **F-55** | 🟠 | Doppeldefinitionen ohne Eigentümer (u. a. `KvSegment`, `KeyManager`, `CopyRegistry`, `GcFloor`, `egress_gateway`, PID-Regler). | E70, §4.11 | N2.3 |
| **F-56** | 🟠 | Stubs mit Erfolgsrückgabe: DLP-Sanitizer, `merge()` mit `Ok(Vec::new())`, unsalted Surrogat-Hash. Aus der Roadmap übernommen, nicht selbst geprüft. | E69 | N1.3 |
| **F-57** | 🟠 | Resume ungeregelt: Hintergrund-Purge, Leser-Sichtbarkeit, Dispose-Policy (Produktion `RequireScrub`, Blueprint `OrxHooks::default()`). | E76 | N4.3 |
| **F-58** | 🟠 | `TenantId(pub u64)` und `TxId(pub u64)` sind öffentlich (`contextra-types/src/types/domain/ids.rs`). | E74 | N2.2 |
| **F-59** | 🟡 | Canary laut Umstrukturierungs-Roadmap nur SSTable-Rohbytes. Kompression und Verschlüsselung erzeugen falsch negative Ergebnisse. | G-32 | N6.2 |
| **F-60** | 🟡 | Keine Owner, kein Kalender. Phasensummen widersprechen sich (39, ca. 12, ca. 9 Wochen). | E72 | N0.4 |
| **F-61** | 🟡 | Vier parallele Roadmaps ohne Führung. | E71 | §12 |

---


---

## 4. Zielarchitektur

### 4.1 Ring-Modell (SOLL, einziges gültiges Modell)

Es gibt genau ein Ringsystem (Ring 0 bis 4 plus Tooling). Andere Schichtnummerierungen (z. B. „Layer 0–10") heißen „Topo-Ebene" und sind nur eine Sortierhilfe der Dokumentation.

| Crate | Ring | Unsafe-Insel | Produkt | Rolle |
|---|---|---|---|---|
| `contextra-types` | 0 | nein | EP1 | Basistypen, IDs, `ContextraError`, `DeterministicTimestamp` |
| `contextra-ports` | 0 | nein | EP1 | Alle Port-Traits (Clock, Rng, IdGenerator, StorageEngine, Erasure-Ports, ReadSnapshot, MergeOperatorFactory, LicenseGate, …) |
| `contextra-wire` | 0 | **ja** (generierter Code) | EP1 | Wire-Format (FlatBuffers-Ableitung) |
| `contextra-sys` | 0 | **ja** | EP1 | OS-Schnittstellen, `SystemClock`, `SystemRng`, sichere ausgerichtete Allokation, mmap-Hilfen |
| `contextra-durable-fs` | 0 | nein | EP1 | fsync-Disziplin, `scrub_and_remove`, `sync_dir`, Journal |
| `contextra-simd` | 0 | **ja** | EP1 | SIMD-Kernel (AVX2, NEON, Scalar) |
| `contextra-crypto-core` | 0 | nein | EP1 | KeyManager, KDF, AEAD, HMAC, SecureBuf, WAL-Crypto (**neu durch Split**) |
| `contextra-mvcc` | 0 | nein | EP1 | Snapshots, SSI-Validator |
| `contextra-text` | 0 | nein | EP1 | BM25, invertierter Index |
| `contextra-rank` | 0 | nein | EP1 | RRF-Fusion, Kalibrator (Conformal), DiBud |
| `contextra-vector` | 0 | nein | EP1 | HNSW und Quantisierung; Delegation an `simd` |
| `contextra-adapt` | 0 | nein | EP5 | PID, Bandit |
| `contextra-graph` | 0 | nein | EP3 | CSR-Graph, PPR, PathRAG (rein algorithmisch, **kein** Store-Import) |
| `contextra-crypto-kv` | 0 | nein | EP1 (Feature) | KV-Segmentverschlüsselung, Revocation-Log (**neu**, Feature `kv-encryption`) |
| `contextra-proof-verify` | 0 | nein | EP2 | Unabhängiger Verifier, nur `ed25519-dalek` und `blake3` (**neu**) |
| `contextra-audit-export`, `contextra-avv-generator` | 0 | nein | EP2 | Export und AVV-Generator |
| `contextra-store` | 1 | nein | EP1 | LSM, WAL, Manifest, Ledger, Purge, Fence |
| `contextra-checkpoint` | 1 | nein | EP1 | Checkpoints (nutzt `Clock`/`IdGenerator`) |
| `contextra-kvcache` | 1 | nein (`forbid(unsafe_code)`) | EP4 | KV-Cache |
| `contextra-infer-candle`, `-ollama`, `-onnx` | 2 | nein | EP4 | Inferenz-Backends |
| `contextra-sandbox` | 2 | nein | EP6 | WASI-Merge-Operatoren; implementiert `MergeOperatorFactory` |
| `contextra-privacy` | 3 | nein | EP6 | DLP, Egress-Filter |
| `contextra-engine` | 3 | nein | EP1 | Collections, Hybridsuche, Löschpfad |
| `contextra-db` | 3 | nein | EP1 | Mehrmandanten-API über Engine |
| `contextra-erasure-evidence` | 3 | nein | EP2 | Beweisausstellung, Policy, Zeugen (**neu durch Split**) |
| `contextra-cognition`, `-router`, `-agent` | 3 | nein | EP5 | Kognition, Routing, Agent |
| `contextra` | 4 | nein | EP1 | Fassade und Composition Root |
| `contextra-mcp` | 4 | nein | EP1 | MCP-Server (stdio) |
| `contextra-py` | 4 | nein | EP1/EP5 | Python-Bindings |
| `contextra-license` | 4 | nein | EP6 | Lizenzprüfung (optional, nicht in EP1) |
| `contextra-testkit`, Bench, Fuzz, `xtask*` | Tooling | nein | — | In `capabilities.toml` eingetragen |

**Entfernt:** `contextra-core` (§4.9), `contextra-crypto` (§4.8). Es gibt **kein** Crate `contextra-errors` und kein Crate `contextra-erasure` (E38). Ringzuordnungen aus den Anhängen, die von dieser Tabelle abweichen, sind verworfen (E51). Offen bleibt allein die Ringzuordnung von `contextra-audit-export` und `contextra-avv-generator` (G5, O-12). Ein eigenes Crate `contextra-erasure` entsteht in EP1 **nicht**: Ledger, Purge, Intent, Resume und RestoreGuard bleiben im Store, der Koordinator in Store und Engine hinter den Ports (ADR-S12).

### 4.2 Ordnung innerhalb eines Rings
Innerhalb eines Rings darf ein Crate nur von Crates abhängen, die in der folgenden Ordnung **vor** ihm stehen:

```
Ring 0:  types < ports < wire < sys < durable-fs < simd < crypto-core < mvcc < text < rank
              < vector < adapt < graph < crypto-kv < proof-verify < audit-export < avv-generator
Ring 1:  store < checkpoint < kvcache
Ring 2:  sandbox, infer-candle, infer-ollama, infer-onnx   (untereinander unabhängig)
Ring 3:  privacy < engine < erasure-evidence < db < cognition < router < agent
Ring 4:  license < contextra < mcp, py
```

### 4.3 Abhängigkeitsmatrix und Auflösung der Verletzungen

| Crate | Darf abhängen von | Darf **nicht** abhängen von |
|---|---|---|
| `store` | `types, ports, sys, durable-fs, crypto-core, mvcc, vector` (+ `graph` nur mit Store-Feature `graph`) | `crypto-kv` (außer Feature `kv-encryption`), `erasure-evidence`, `sandbox`, `license`, `engine` |
| `vector` | `types, ports, sys, durable-fs, simd, crypto-core` | `store`, `erasure-evidence` |
| `graph` | `types, ports, adapt` | `store`, `engine`, `crypto-*` |
| `checkpoint` | `types, ports, store` | `engine`, `crypto-kv` |
| `kvcache` | `types, ports, sys, crypto-core` | `store`, `engine` |
| `engine` | `types, ports, store, vector, text, rank, mvcc, checkpoint` (+ optional `graph`, `adapt`, `kvcache`, Inferenz) | `crypto-*` direkt, `erasure-evidence`, `license`, `sandbox` direkt |
| `db` | `engine, store, ports, types` | `cognition` (regulär) |
| `erasure-evidence` | `types, ports, crypto-core, durable-fs` | `store`, `engine`, `license` |
| `mcp` | `contextra` (Fassade), `types`, `ports` | `crypto*`, `adapt`, `privacy`, `wire`, `rank` direkt |

Auflösungen:
- `store → sandbox`: Port `MergeOperatorFactory` in `ports`; `sandbox` implementiert ihn; die Fassade verdrahtet.
- `graph → store`: Persistenz über Port `GraphPersistence` in `ports`; der Store (mit Feature `graph`) oder die Engine implementiert ihn. `graph` importiert `store` nie.
- `store → graph`: nur hinter Store-Feature `graph`, im EP1-Build aus.
- `engine → license`: Port `LicenseGate` in `ports` (Default `Open`). `license` bleibt Ring 4 und optional.
- `db → cognition`: Kante entfällt; die Fassade (Ring 4) verdrahtet `cognition` über Ports.
- `checkpoint → store`: erlaubt durch Ordnung `store < checkpoint`.

### 4.3a Übergangsausnahmen zur Ringmatrix (E54)

Der IST-Stand verletzt die Matrix in §4.3 an zwei Stellen, deren Auflösung an späte Meilensteine gebunden ist. Verletzungen werden nicht stillschweigend geduldet. Sie stehen als Ausnahme in `capabilities.toml` (Feld `exceptions`) mit Ablauf-Meilenstein:

| Ausnahme | Verletzt | Ablauf | Abbau |
|---|---|---|---|
| E-1 | `store → contextra-core` (`contextra-store/Cargo.toml:41`) | N3.1 | mit M3.1 |
| E-2 | `store → contextra-crypto` (monolithisch, `Cargo.toml:45`) | N5.2 | Crate-Split §4.8 |

Regeln:
1. Gate G-21 (`check-exception-ledger`) schlägt fehl, wenn ein Ablauf-Meilenstein erreicht ist und die Ausnahme noch besteht.
2. Die Anzahl der Ausnahmen darf nur sinken. Neue Ausnahmen erfordern ein ADR.
3. Ausnahmen gelten nur für die genannte Kante, nicht für das ganze Crate.

### 4.4 Hülle und Produkt-Manifest EP1

Metrik: Anzahl der Workspace-Crates in `cargo tree -p contextra-mcp -e normal` mit dem Build-Feature-Satz des Produkts. Das Gate misst den **Default-Build** und berichtet alle anderen Feature-Sätze.

```toml
# products/ep1.toml
[product]
id = "EP1"
name = "Contextra Memory"
status = "release-0.1"
target_binary = "contextra-mcp"

[crates]
allowed = [
  "contextra-types", "contextra-ports", "contextra-sys", "contextra-wire",
  "contextra-simd", "contextra-durable-fs", "contextra-crypto-core",
  "contextra-mvcc", "contextra-text", "contextra-rank", "contextra-store",
  "contextra-vector", "contextra-checkpoint", "contextra-engine",
  "contextra-db", "contextra", "contextra-mcp",
]   # 17 Crates
forbidden = [
  "contextra-core", "contextra-crypto", "contextra-graph", "contextra-cognition",
  "contextra-adapt", "contextra-router", "contextra-agent", "contextra-privacy",
  "contextra-sandbox", "contextra-license", "contextra-kvcache",
  "contextra-infer-candle", "contextra-infer-ollama", "contextra-infer-onnx",
  "contextra-erasure-evidence", "contextra-crypto-kv", "contextra-proof-verify",
]

[build]
features = ["fast"]        # Release-Profil zusätzlich: "erasure", "encryption-at-rest"

[gate]
command = "cargo tree -p contextra-mcp -e normal --no-default-features --features fast"
max_crates = 18             # Ziel 17
```

Hüllen-Ziel: IST 26 → ≤ 18 (Ziel 17) nach M3. Weitere Manifeste `products/ep2.toml` bis `ep6.toml` folgen dem Muster (§4.10).

### 4.5 Feature-Matrix (SOLL)

**Fassade `contextra`:**
```toml
[features]
default = ["fast"]
fast = ["contextra-engine/fast"]                  # SIMD-Dispatch; ohne: Scalar-Pfad

# Erweiterungen (einzeln aktivierbar)
graph      = ["contextra-engine/graph"]
adapt      = ["contextra-engine/adapt"]
kvcache    = ["contextra-engine/kvcache"]
cognition  = ["dep:contextra-cognition"]
router     = ["dep:contextra-router"]
agent      = ["dep:contextra-agent"]

# Inferenz (opt-in; candle ist NICHT mehr Default)
candle = ["dep:contextra-infer-candle"]
ollama = ["dep:contextra-infer-ollama"]
onnx   = ["dep:contextra-infer-onnx"]

# Sicherheit
privacy            = ["dep:contextra-privacy"]
sandbox            = ["dep:contextra-sandbox"]
license            = ["dep:contextra-license"]
dev-open-gate      = ["license", "contextra-license/dev-open-gate"]
encryption-at-rest = ["contextra-store/encryption-at-rest", "dep:contextra-crypto-kv"]

# Löschgarantie
erasure          = ["contextra-store/erasure", "contextra-engine/erasure"]       # Assured
erasure-evidence = ["erasure", "dep:contextra-erasure-evidence", "license"]       # Attested (EP2)

# Format-Features (ändern das On-Disk-Format, siehe unten)
docid-128 = ["contextra-types/docid-128"]

# Bündel
sovereign  = ["privacy", "erasure-evidence", "encryption-at-rest"]
compliance = ["sovereign", "contextra-audit-export", "contextra-avv-generator"]
```

**Engine:** `default = []`. Optionale Features: `graph, adapt, kvcache, candle, onnx, reranking, encryption-at-rest, erasure, fast, bench, sandbox`. `rank` ist Teil der Hybridsuche und daher **Pflichtabhängigkeit**, kein Feature.
**Store:** `erasure` (Ledger, Gate, Purge-Treiber, Resume, RestoreGuard, Backend-Impl), `encryption-at-rest`, `graph`, `fault-injection` (Failpoints), `loom` (nur Tests).
**Ports:** `test-utils` ausschließlich in `[dev-dependencies]`; `backend-impl` schaltet die Konstruktoren für `PhysicalScope` frei und darf **nur** von `contextra-store` aktiviert werden (Gate G-12). Cargo vereinigt Features über alle Ziele eines Builds. Deshalb reicht eine Deklaration nicht: G-22 prüft den Feature-Baum jedes Produkt-Manifests (`cargo tree -e features -i contextra-ports`). Die Selbst-Abhängigkeit in `crates/contextra-ports/Cargo.toml:30` wird entfernt. Konstruktor-Tests liegen in `contextra-store`, wo das Feature aktiv ist (E53).
**MCP:** Features `fast`, `guard`, `graph`, `router`, `agent-workflows`, `candle`, `ollama`, `onnx`, `encryption-at-rest`, `erasure` leiten an die Fassade weiter. Ohne Feature fehlt der zugehörige Code und das zugehörige Tool vollständig.

**Format-Features (`docid-128`):** Das Feature ändert die ID-Breite im On-Disk-Format. Das Manifest speichert die ID-Breite. Öffnen eines Stores mit abweichender Breite ergibt `FormatError::IdWidthMismatch`. Format-Features sind von der Isolationsprüfung (G-06) ausgenommen und werden stattdessen als eigene Matrix-Achse getestet.

**Performance-Budgets je Feature** (Messung per Instruktionszähler plus Wall-Clock-Toleranz, §11.3):

| Feature | Schreibpfad | Lesepfad | Startup |
|---|---|---|---|
| `erasure` (Modus Off) | 0 (Gate nicht injiziert) | 0 | 0 |
| `erasure` (Assured, idle) | ≤ 3 % Abweichung | 0 | gemessen, Ziel ≤ 250 ms bei 1 Mio. Ledger-Einträgen |
| `erasure` (Assured, Purge aktiv) | ≤ 5 % p99 | 0 | — |
| `encryption-at-rest` | ≤ 5 % p99 | ≤ 5 % p99 | ≤ 50 ms |
| `graph` | 0 | ≤ 5 % p99 | ≤ 200 ms |
| `adapt` | 0 | ≤ 1 % (Instruktionen) | ≤ 10 ms |

Alle Prozentwerte sind **Startwerte**, die in M4 mit Baseline-Messungen kalibriert werden. Die Zusagen „0 ns" oder „< 1 ns" werden nirgends gemacht.

### 4.6 DurabilityMode × ErasureMode

Zwei getrennte Konzepte, durchgesetzt im Builder (Build-Zeit) und im Manifest (Open-Zeit):

| DurabilityMode ↓ / ErasureMode → | `Off` | `Assured` | `Attested` |
|---|---|---|---|
| `Full` | ✅ | ✅ | ✅ |
| `WalNoHmac` | ✅ | ✅ (Health-Warnung `W-DURABILITY-WAL-NOHMAC`; Ledger hat eigenen MAC) | ❌ `INV-DURABILITY-RING` |
| `MemoryOnly` | ✅ | ❌ `INV-DURABILITY-RING` | ❌ `INV-DURABILITY-RING` |

### 4.7 Builder-Ehrlichkeit und Wiring

1. `ContextraBuilder::build()` ruft `verify_wiring()` **zwingend** auf. Das Ergebnis wird nie verworfen (`let _report` ist verboten).
2. `verify_wiring` prüft **den tatsächlichen Zustand**: jeder gesetzte Schalter muss einen realen, nicht-`None`-Port besitzen; jeder Port, der laut Feature-Satz aktiv sein muss, muss vorhanden sein. Hartcodierte Werte (wie `has_license_gate = true`) sind verboten. Das Ergebnis `WiringReport` listet je Schalter `Wired | NotRequested`, und `build()` liefert `BuildError::IncompleteWiring { flag, missing }` bei jedem Widerspruch.
3. Für jeden der sieben bisher wirkungslosen Schalter (`with_privacy`, `with_adapt`, `with_kms`, `with_kv_shredding`, `with_checkpoint`, `with_avv_generator`, `with_audit_export`) gilt **eine** der Optionen, je Schalter im AP M3.5 entschieden: (a) verdrahten und mit Test belegen; (b) entfernen. Wirkungslose Schalter bleiben nicht stehen. Voreinstellung: `with_kms`, `with_kv_shredding`, `with_checkpoint` verdrahten; `with_privacy`, `with_adapt` hinter ihre Features stellen; `with_avv_generator`, `with_audit_export` nach EP2 verschieben und im EP1-Builder entfernen.
4. Neue Builder-API: `with_erasure_mode(ErasureMode)`, `with_erasure_hooks(Arc<dyn ErasureHooks>)`, `with_license_gate(Arc<dyn LicenseGate>)`. `with_erasure_mode(Assured)` ohne Feature `erasure` ergibt `BuildError::FeatureMissing`.
5. `KvDeleteMode::default()` ist `TombstoneOnly`. Der Builder setzt `CryptoShred`, sobald `encryption-at-rest` aktiv ist. `INV-KV-DELETE-1` wird um den Satz ergänzt: *Ohne Verschlüsselung ist kein KV-Löschbeweis möglich.* Der Test `test_kv_delete_mode_default` wird entsprechend angepasst.
6. Gate G-05 (`check-builder-wiring`): kein `let _ =` und kein `_`-Präfix auf Builder-Feldern, jeder Builder-Schalter hat mindestens einen Test, der seine Wirkung beobachtet.

### 4.8 Crate-Split `contextra-crypto`

`contextra-crypto` (IST 7.669 Zeilen) wird zerlegt. **Es bleibt kein Shim** (vor v0.1.0 gibt es keine externen Konsumenten); alle internen Imports werden per Skript-PR umgestellt.

| Neues Crate | Ring | Inhalt |
|---|---|---|
| `contextra-crypto-core` | 0 | `KeyManager`, KDF (Argon2id, HKDF-SHA-256), AEAD (AES-256-GCM-SIV), HMAC-SHA-256, `SecureBuf`, WAL-Crypto, Keystore, KMS-Port-Implementierung `LocalKmsProvider` |
| `contextra-crypto-kv` | 0 | `RevocationLog`, `kv_cipher`, `kv_segment`, `kv_shredding` (Feature `kv-encryption`) |
| `contextra-erasure-evidence` | 3 | `deletion_proof*`, `deletion_witness`, `ed25519_proof`, `audit_chain`, `anti_tamper`, `wal_completeness` (EP2) |
| `contextra-proof-verify` | 0 | Unabhängiger Verifier (EP2) |

Fuzz-Targets `deletion_proof_tamper` und `..._verify_external_fuzz` ziehen mit nach `evidence`/`proof-verify` um. Governance-Aufwand (Treffer auf `contextra-crypto`: 35 in `.github`, 27 in `xtask`, 18 in `rules`, 12 in `capabilities.toml`, 10 in `mutants-guarantees.toml`, 6 im `justfile`) ist Teil von AP M5.5 und wird in **einem** PR je Zielcrate abgearbeitet.

Reihenfolge-Vorgabe: Der Split in `crypto-core` und `evidence` erfolgt in M5 (nach der ehrlichen Löschkette), weil erst dann feststeht, welche Beweistypen die Engine tatsächlich nutzt.

### 4.9 Entfernung von `contextra-core`

`contextra-core` wird ersatzlos entfernt. Vorgehen:
1. Per Skript die `pub use`-Liste von `contextra-core/src/lib.rs` auslesen und daraus die Mapping-Tabelle **generieren**. Handgepflegte Tabellen sind nicht zulässig, weil Symbole wie `ResourceBudget`, `ResourceTracker`, `TOMBSTONE_BIT`, `MAX_SEARCH_K`, `DistanceMetric` über `pub use contextra_ports::*` hindurchgereicht werden.
2. Mapping-Beispiele: `DocId, TxId, ContextraError` → `contextra_types`; `TxBuffer` → `contextra_mvcc`; `WireFormat` → `contextra_wire`; Budget- und Distanztypen → `contextra_ports`.
3. Umstellung per Skript-PR je Crate-Gruppe (3 bis 4 PRs) über das vorhandene Kartenbudget (`files`, `lines`, `crates`) mit Reviewer-Prüfsumme. Das Standardbudget (15 Dateien, 600 Zeilen, 2 Crates) reicht nicht, eine neue Ausnahmeklasse wird nicht geschaffen. `AGENTS.md` bleibt unverändert. Der Eintrag zu `contextra-core` in der Scope-Freeze-Entscheidung (Klasse K „Strangler-Fassade") wird angepasst.
4. Danach `contextra-core` aus Workspace, `capabilities.toml`, Dokumentation löschen. Gate: `cargo check --workspace` ohne das Crate.

### 4.10 Manifeste der übrigen Produkte (Kurzfassung)

| Produkt | Zusätzliche Crates gegenüber EP1 | Zusätzliche Features |
|---|---|---|
| EP2 | `crypto-core` (bereits), `crypto-kv`, `erasure-evidence`, `proof-verify`, `audit-export`, `avv-generator`, `license` | `erasure-evidence`, `encryption-at-rest`, `compliance` |
| EP3 | `graph` (+ Engine-Feature `graph`) | `graph` |
| EP4 | `kvcache`, `infer-candle`, `infer-onnx`, `infer-ollama` | `kvcache`, `candle`, `onnx`, `ollama` |
| EP5 | `adapt`, `router`, `cognition`, `agent` | `adapt`, `router`, `cognition`, `agent` |
| EP6 | `privacy`, `sandbox`, `license` | `privacy`, `sandbox`, `guard`, `license` |

EP2 ist ein Produkt innerhalb desselben Repositories. Eine Auslagerung in ein eigenes Repository ist eine Organisationsentscheidung und bleibt offen (§14).

---

### 4.11 Eigentum bei Doppeldefinitionen (E70)

Diese Typen existieren doppelt. Vor dem Crate-Schnitt bekommt jeder Typ genau einen Eigentümer. Der andere wird gelöscht oder als Re-Export geführt. Die Tabelle stammt aus FE §3.7. Der Einzelnachweis je Typ ist Teil von N2.3.

| Typ | Orte heute | Eigentümer SOLL |
|---|---|---|
| `KvSegment`, `EvictionWorker`, `SpillHandler` | `crypto`, `kvcache` | `kvcache` |
| `EncryptedSegmentPayload` | `crypto`, `kvcache` | `crypto-kv` |
| `KeyManager` | `crypto`, `store/wal` | `crypto-core` |
| `CopyRegistry` | `checkpoint`, `store` | `checkpoint` |
| `GcFloor` | `mvcc/floor.rs`, `store/lsm/gc_floor.rs` | `mvcc` (`SnapshotFloorTracker`) |
| `AvvContext` | `privacy`, `avv-generator` | `avv-generator` |
| `ProofKeySource` | `crypto`, `mcp` | `erasure-evidence` (EP2) |
| `egress_gateway` | `privacy`, `mcp` | `privacy` |
| PID-Regler | `adapt/pid.rs`, `adapt/pid_latency_controller.rs` | eine Implementierung (R7-13) |
| `CsrGraph`, `HyperEdge`, `RoleId` in `engine/no_graph_stubs.rs` | `engine`, `graph` | Ports in `contextra-ports::graph`, Stubs entfallen |

## 5. Löscharchitektur (Kernversprechen)

### 5.1 Drei Modi

`ErasureMode` ist ein Laufzeitmodus mit Compile-Zeit-Feature. Standard ist `Off`.

| Eigenschaft | `Off` | `Assured` | `Attested` (EP2) |
|---|---|---|---|
| Löschmethode | Tombstone + normale Compaction | Ledger → Tombstone → Purge (Rewrite + Scrub), Crypto-Erase nur mit `encryption-at-rest` | wie `Assured` |
| Ledger (Anti-Resurrection) | — | ✅ mit fsync | ✅ |
| Write-Fence | — | ✅ Präfix-Fence | ✅ |
| Resume nach Crash | — | ✅ Scope aus Ledger | ✅ |
| Restore-Guard | — (dokumentierte Nicht-Garantie) | ✅ streamend | ✅ |
| Layer-Scans / Zeugen | — | — | ✅ |
| Signierter Beweis | — | — | ✅ (v4) |
| Cargo-Feature | Kern | `erasure` | `erasure` + `erasure-evidence` |
| Lizenz | — | — | `Sovereign` (nur zum Ausstellen neuer Beweise, §5.15) |
| Laufzeitkosten | **keine** (kein Gate injiziert, Code nicht kompiliert) | Ledger-Append je Löschung, Gate-Check je Batch | + Scans + Signatur |

### 5.2 Garantien je Modus (Vertrag)

Nach `Ok(DeleteOutcome::Assured)` und abgeschlossenem Purge (`PurgeState::Completed`) gilt innerhalb des Stores:
1. Kein lesbarer Datensatz des Scopes existiert in Memtable, SSTables, Blob-Dateien und Manifest-Schlüsselgrenzen.
2. Alle Altdateien wurden überschrieben (Nullung), `fsync`-ed, entfernt, das Verzeichnis ist synchronisiert.
3. WAL-Segmente, deren Inhalt vollständig in SSTables übernommen wurde und die Scope-Daten enthielten, sind gescrubbt und entfernt (§5.10).
4. Ein Wiedereinspielen älterer Checkpoints oder WAL-Replays stellt den Scope nicht wieder her (RestoreGuard + Ledger).
5. Abgeleitete Indizes (Vektor, Text) enthalten keinen Eintrag zum Scope (§5.10, Scans in EP2 belegen es).

**Nicht garantiert (in jedem Modus):** SSD-Controller-Remanenz (Wear-Leveling, Spare Area), CoW-Dateisysteme und Snapshots des Dateisystems, Prozessspeicher, Swap, Core-Dumps, externe Backups, Kopien außerhalb des Stores, Logs und Tracing (E62), Plattformen ohne Dauerhaftigkeitsbeleg (§5.17), Angreifer mit Schreibzugriff auf Store-Dateien (§5.16, Klasse A2).

### 5.3 Commit-Punkt und API-Semantik

**Commit-Punkt** einer Löschung ist das erfolgreiche, durable Anhängen des `ErasureIntent`-Records an das Ledger (inkl. `fsync`). Das ist ein Linearisierungspunkt:

- **Vor dem Commit-Punkt** scheitert die Operation mit `EraseError::NotStarted(cause)`. Es gibt keine Wirkung, ein Retry ist sicher.
- **Nach dem Commit-Punkt** wird die Löschung **zwingend abgeschlossen** (Roll-forward). Der Aufrufer erhält entweder `Ok(DeleteOutcome::Assured { .. })` oder, wenn danach etwas schiefging (Tombstone-Commit-Konflikt, I/O, Shutdown, Cancellation), `Err(EraseError::Indeterminate { purge_id, cause })`. `Indeterminate` bedeutet ausdrücklich „wird gelöscht", nicht „fehlgeschlagen".
- **Idempotenz:** Dieselbe Löschung erneut anzufordern liefert denselben `PurgeHandle`.
- **Cancellation-Sicherheit:** Nach dem Commit-Punkt läuft der Rest in einem vom `ErasureSupervisor` (im `EngineLifecycle`) besessenen Task. Das Verwerfen des Aufrufer-Futures bricht ihn nicht ab. Vor dem Commit-Punkt bedeutet Abbruch `NotStarted`.
- **Kein `Err` nach Commit:** Fehler beim Aktualisieren abgeleiteter Indizes nach dem Tombstone-Commit (z. B. `index.delete(..)`) dürfen nie als `Err` an den Aufrufer zurückgehen. Sie setzen `PurgeState::PurgePending { reason: IndexRetry }` und einen Health-Eintrag. Die Suche filtert Tombstones per MVCC ohnehin heraus, es gibt keine sichtbare Wiederauferstehung.

```rust
pub enum DeleteOutcome {
    /// Modus Off: Tombstone committed.
    Logical { tx: TxId },
    /// Modus Assured: Ledger durable, Tombstone committed, Purge geplant oder laufend.
    Assured { tx: TxId, purge: PurgeHandle },
}

pub enum EraseError {
    NotStarted(ContextraError),
    Indeterminate { purge_id: PurgeId, cause: ContextraError },
}
```

### 5.4 Scopes

```rust
/// Logischer Scope, vom Aufrufer angegeben. Jede Variante trägt die TenantId.
pub enum LogicalScope {
    Document   { tenant: TenantId, collection: CollectionName, doc: DocId },
    Collection { tenant: TenantId, collection: CollectionName },
    Tenant     { tenant: TenantId },
    // Keys/Prefixes existieren NICHT als LogicalScope (waren mandantenlos und damit unsicher).
}

/// Physischer Scope: opak. Nur das Storage-Backend kann ihn erzeugen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalScope { /* private Felder */ }
```

Invarianten von `PhysicalScope` (im Konstruktor erzwungen, Konstruktor nur mit Port-Feature `backend-impl`, das nur `contextra-store` aktiviert):
1. Nicht leer: mindestens ein Präfix oder Schlüssel.
2. **Jedes** Präfix und jeder Schlüssel beginnt mit dem physischen Mandantenpräfix `t:<tenant_id>:`. Ein leeres Präfix ist unzulässig (kein Wildcard-Purge).
3. Kanonisch: sortiert, dedupliziert, redundante Präfixe (von einem kürzeren überdeckt) entfernt.
4. `matches_key` arbeitet in O(|key|) über eine Präfix-Struktur (sortierter Vektor mit Binärsuche oder Trie), nicht linear über alle Einträge.
5. Kein `Default`, kein `Deserialize`. Persistenz im Ledger erfolgt über eine eigene, validierende `decode`-Funktion im Store, die dieselben Invarianten prüft.
6. `scope_digest() -> [u8;32]` = BLAKE3 über die kanonische Kodierung, Domain-Tag `contextra/physical-scope/v1`.

### 5.5 Port-Spezifikation `contextra-ports::erasure` (SOLL-Signaturen)

Alle Traits sind synchron (der Kern ist synchron; asynchrone Aufrufer nutzen `spawn_blocking`).

```rust
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub enum ErasureMode { #[default] Off, Assured, Attested }

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PurgeId(pub [u8; 16]);            // Zufalls-ID über IdGenerator-Port

/// Lineares Token: weder Copy noch Clone; wird von finish/abort verbraucht.
#[must_use]
pub struct HookToken { id: u64, _no_copy: () }

pub trait ErasureBackend: Send + Sync {
    fn physical_scope(&self, scope: &LogicalScope) -> Result<PhysicalScope, ContextraError>;
    fn acquire_fence(&self, scope: &PhysicalScope, id: PurgeId) -> Result<FenceGuard, ContextraError>;
    fn purge_physical(&self, scope: &PhysicalScope, as_of: TxId,
                      probe: &dyn FreeSpaceProbe) -> Result<PurgeOutput, PurgeError>;
}

/// Ersetzt alle Downcasts auf LsmStorage.
pub trait StorageErasureExt { fn as_erasure_backend(&self) -> Option<&dyn ErasureBackend>; }

pub trait ErasureHooks: Send + Sync + 'static {
    fn on_begin(&self, ctx: &EraseContext<'_>) -> Result<HookToken, ContextraError>;       // vor Commit-Punkt, schreibt Ledger
    fn on_tombstoned(&self, tok: &HookToken, tx: TxId) -> Result<(), ContextraError>;
    fn on_purged(&self, tok: &HookToken, out: &PurgeOutput) -> Result<(), ContextraError>;
    fn finish(&self, tok: HookToken) -> Result<FinishArtifact, ContextraError>;            // verbraucht das Token
    fn abort(&self, tok: HookToken);                                                        // verbraucht das Token
}
pub enum FinishArtifact { None, Proof(Vec<u8>) }  // Proof nur in Attested

/// Schreib-Gate. Fast-Path: ein einziger Acquire-Load je Batch.
pub trait WriteGate: Send + Sync {
    fn is_active(&self) -> bool;                                      // #[inline], AtomicBool::load(Acquire)
    fn check_batch(&self, keys: &mut dyn Iterator<Item = &[u8]>) -> Result<(), WriteFenced>;
}

pub trait FreeSpaceProbe: Send + Sync {
    /// None = unbekannt. Darf NIE als 0 Byte interpretiert werden.
    fn available_bytes(&self) -> Result<Option<u64>, ProbeError>;
}

pub trait RestoreGuard: Send + Sync {
    fn admit(&self, key: &[u8], source_tx: TxId) -> RestoreVerdict;   // Admit | Reject
}

/// Layer-Scan (EP2). Synchron, fail-closed. layer_id ist u16.
pub trait LayerScan: Send + Sync {
    fn layer_id(&self) -> u16;
    fn scan(&self, scope: &PhysicalScope) -> Result<LayerObservation, ScanError>;
}
pub struct LayerObservation {
    pub layer_id: u16,
    pub clean: bool,
    pub state_digest: [u8; 32],       // BLAKE3, Domain je Layer
    pub item_count: u64,              // gescannte Einheiten (Beleg gegen Leer-Scans)
}
```

Weitere Vorgaben:
- `purged(...)`/Attested-Abschluss verlangt eine **nicht leere** Scan-Liste; die Menge der Pflicht-Layer ist je Produkt in einer Konstante `REQUIRED_LAYERS` festgelegt, und ein Beweis ohne alle Pflicht-Layer wird nie ausgestellt.
- Zeitstempel in Port-Typen sind `DeterministicTimestamp` aus dem `Clock`-Port (`wall_ms` plus monotoner Zähler), nie `SystemTime`.
- `ReadSnapshot` enthält **keinen** `Instant`; es trägt `max_seq: TxId` und eine Snapshot-ID (§7.2).
- `PurgeOutput { records_purged, bytes_reclaimed, sstables_replaced, obsolete_files, duration_ticks }`; `duration` stammt aus dem `Clock`-Port.
- `PurgeError { InsufficientDiskSpace{needed, available: Option<u64>}, FenceConflict, ReadError{file, offset, source}, Io, Manifest, ProbeFailed, SnapshotPinned{oldest_pin_age} }`.

### 5.6 Löschpfad und Zustandsmaschine

**Modus `Off`:** `delete()` → Tombstone → `DeleteOutcome::Logical`. Keine Hooks, kein Ledger, kein Gate, keine Scans.

**Modus `Assured`:**
```
1. Auflösung: Mandant fail-closed bestimmen (§5.11); LogicalScope bilden.
2. backend.physical_scope(scope)                                  (Invarianten §5.4)
3. hooks.on_begin(ctx)  → Ledger.append(ErasureIntent) + fsync    ← COMMIT-PUNKT
4. Tombstone-Commit (MVCC); Retry bei SSI-Konflikt                 → on_tombstoned
5. Purge-Intent-Datei schreiben (Zufalls-PurgeId, nie fester Name)
6. Antwort an Aufrufer: Ok(Assured{purge})      (Purge läuft im ErasureSupervisor-Task)
7. Fence installieren (Drain, §5.8) → Memtable flushen → Purge v2 (§5.7)
8. Abgeleitete Speicher bereinigen (§5.10) → WAL-Scrub
9. Ledger.append(ErasurePurged) + fsync → Intent-Datei löschen → Fence freigeben
```
Bei `Attested` folgen nach Schritt 8 die Layer-Scans, die Beweisausstellung und `finish` (EP2, §6).

**Persistierte Zustände** (`#[repr(u8)]`-Enum `PurgeState` in `lsm/purge/intent.rs`, mit Golden-Test der Byte-Kodierung):

| Wert | Zustand | Bedeutung |
|---|---|---|
| 0 | `Begun` | `hooks.begin()` aufgerufen |
| 1 | `LedgerAppended` | Commit-Punkt: Ledger-Eintrag fsync-fest |
| 2 | `Tombstoned` | Tombstone committed |
| 3 | `PurgePending` | wartet (Fence, Speicherplatz, Pin, Retry) mit `reason` und `retry_at` (Ticks) |
| 4 | `Purging` | Fence gehalten, Rewrite läuft |
| 5 | `Purged` | Rewrite, Scrub und abgeleitete Speicher fertig |
| 6 | `KeyQuarantined{until}` | Crypto-Erase Stufe 1 erfolgt (§5.13) |
| 7 | `Completed` | Assured abgeschlossen (Endzustand EP1) |
| 8 | `Attested` | (EP2) Scans und Beweis fertig; Scans laufen in diesem Zustand |

Zustandsnummern nach E67 (FE-10). Die Werte aus v1.1 bis v1.3 entfallen.

Kanten: `0→1→2→3→4→5→7`; `4→3` bei Fehler (Retry); `3→3` bei Crash (Resume); `5→6→7` bei Crypto-Erase; `5→8→7` in `Attested` (EP2). Jede Kante schreibt vor der Wirkung den Zielzustand in die Intent-Datei (write-ahead). Es gibt **keinen** Zustand, der ohne persistierte Intent erreichbar ist.

---

### 5.7 Purge v2 (Streaming-Rewrite)

Ort: `contextra-store/src/lsm/ops/purge.rs` (Treiber) und `lsm/purge/*`. Ziel: jede der Ursachen aus F-06 strukturell beseitigen.

**Ablauf (alle Schritte unter dem Fence aus §5.8):**
1. **Vorprüfung Speicherplatz:** `probe.available_bytes()`. `Ok(Some(n))` und `n < Bedarf` → `InsufficientDiskSpace`, Zustand `PurgePending{DiskSpace}`. `Ok(None)` (unbekannt) → **nicht blockieren**; Purge startet und rollt bei `ENOSPC` sauber zurück. `Err` → `PurgeError::ProbeFailed`, Zustand `PurgePending`. Bedarf = Summe der Größen der zu schreibenden Ersatzdateien plus Reserve (Startwert 10 MiB, FE-Vorgabe). Eine `None`-Probe blockiert nicht.
2. **Memtable:** aktive und immutable Memtables flushen, damit alle Scope-Daten in SSTables stehen.
3. **Je betroffener SSTable** (Vorfilter über Key-Range und Bloom-Filter): Entry-Strom lesen und direkt in eine Ersatz-SSTable schreiben (O(1) zusätzlicher RAM, kein `Vec` aller Records, kein Doppellesen). Lesefehler: **3-Arm-Match** `Ok(Some)`/`Ok(None)`/`Err`. `Err` bricht den Writer ab, löscht die Teildatei (`.purge_tmp`) und liefert `PurgeError::ReadError`. Ein Lesefehler wird **nie** als Dateiende behandelt.
4. **Verwerfungsregel:** Ein Record wird verworfen, wenn `scope.matches_key(key)` und `seq <= as_of` gilt. Das umfasst alle Versionen **und** Tombstones des Scopes. Spätere Schreibzugriffe (`seq > as_of`) bleiben erhalten; sie können nur nach Fence-Ende entstehen.
5. **Positionsverlust verhindern:** Die Ersatz-SSTable übernimmt Level/Rang sowie `min_seq` und `max_seq` der Quelle (`finalize_with_bounds`); die Reihenfolge der Level in der Merge-Logik bleibt erhalten.
6. **Blob-Dateien und Referenzregister** (`store/src/blob`, `copy_registry`): Werte und Referenzeinträge des Scopes werden entfernt oder die Blobs neu geschrieben. (Re-V.: genaue Struktur beim AP lesen und beschreiben.)
7. **Manifest:** Atomarer Austausch (`commit_sstable_exchange`) von Alt- gegen Ersatzdateien in **einer** Manifest-Transaktion. Schlüsselgrenzen im Manifest werden neu berechnet. **Alte Manifest-Versionen und Manifest-Logsegmente** werden nach dem Austausch gescrubbt.
8. **Scrub:** `contextra_durable_fs::scrub_and_remove(path)` je Altdatei (Nullen überschreiben, `fsync`, unlink), danach `sync_dir`. Fehler dabei sind **nicht** still: Zustand bleibt `Purging`, Retry, Health-Eintrag.
9. **Orphan-GC:** Verwaiste `.purge_tmp`- und Altdateien werden beim Start über denselben Scrub entfernt (kein bloßes `remove_file`).
10. **Intent-Dateien:** Dateiname `purge/<purge_id>.intent` mit Zufalls-ID aus dem `IdGenerator`-Port; mehrere gleichzeitige Intents sind möglich.
11. **Failpoints** (für Chaos-Tests) existieren nur unter `cfg(feature = "fault-injection")`, nie über Umgebungsvariablen.

**Koaleszenz:** Löschanfragen innerhalb eines Fensters (`erase_coalesce_window`, Startwert 1 s) werden zu einem Purge mit vereinigtem Scope zusammengefasst, weil Memtable-Flush, Rewrite und Index-Rebuild Gemeinkosten haben. Der Ledger-Eintrag je Anfrage bleibt einzeln.

**Differenztest (Pflicht):** Ein Referenzmodell (`BTreeMap<Key, Vec<(seq, Value|Tombstone)>>`) wird mit zufälligen Schreib-/Lösch-/Purge-Sequenzen verglichen (proptest). Zusätzlich ein **Rohsweep**: nach dem Purge werden alle Dateien des Store-Verzeichnisses bytegenau nach den Scope-Schlüsseln und -Werten durchsucht; ein Treffer ist ein Testfehler.

### 5.8 Write-Fence, Protokoll B (Drain-Fence)

**Problem im IST (F-11):** Der Check liegt am Eingang von `put`/`delete`. Zwischen Check und Publish im Group-Commit (zwei `commit_mutex`-Fenster mit WAL-I/O dazwischen) kann eine Fence aktiv werden.

**Strukturen** (`contextra-store/src/lsm/fence.rs`, nur mit Feature `erasure`):
```rust
pub struct ErasureGate {
    active:   AtomicBool,                         // Fast-Path
    table:    arc_swap::ArcSwap<Vec<PrefixFence>>, // Präfixe mit PurgeId und Refcount
    inflight: AtomicU32,                          // Batches zwischen Check und Publish
    drained:  tokio::sync::Notify,                // oder Condvar im synchronen Pfad
    install:  parking_lot::Mutex<()>,             // serialisiert Install/Release
}
```
`arc-swap` wird in `deny.toml` und `supply-chain/` aufgenommen und geprüft.

**Writer/Leader-Seite** (Group-Commit-Leader):
```
[commit_mutex lock]                    // Fenster 1
    if gate.is_active() { gate.check_batch(batch_keys)?  }   // Reject → WriteFenced
    gate.inflight += 1                 // im selben kritischen Abschnitt wie der Check
    Sequenznummern vergeben
[commit_mutex unlock]
WAL-Append + fsync                     // ohne Lock
[commit_mutex lock]                    // Fenster 2
    Memtable-Publish
    gate.inflight -= 1; falls 0 und aktiv: drained.notify_all()
[commit_mutex unlock]
```
Der Check am Eingang von `put`/`delete` bleibt als Frühabbruch (Optimierung, keine Sicherheitsgrundlage).

**Installer-Seite** (`acquire_fence`):
```
1. install.lock()
2. table ← table + neue Präfixe (ArcSwap::store); active.store(true, Release)
3. Drain: wiederhole { commit_mutex lock; if inflight == 0 {unlock; break}; unlock; warte auf drained mit Timeout }
4. Timeout → Fence wieder entfernen, Fehler FenceConflict (Retry mit Backoff)
5. install.unlock(); FenceGuard zurückgeben
```
**Korrektheitsargument:** Jeder Leader-Abschnitt in Fenster 1 liegt entweder vor oder nach dem ersten Lock-Erwerb des Installers in Schritt 3 (Mutex-Ausschluss). Davor: der Batch ist gezählt und wird vor `inflight == 0` publiziert, bevor der Purge den Manifest-Snapshot liest. Danach: `active == true` ist über die Mutex-Happens-before-Beziehung sichtbar, ein konfligierender Batch wird abgewiesen. Es entsteht kein WAL-Record für abgewiesene Batches, also ist kein WAL-Format-Eintrag „void" nötig.

**Freigabe:** `FenceGuard::drop` entfernt die Präfixe, setzt `active = false`, wenn die Tabelle leer ist. Beim Start (Resume) werden Fences **vor** der Rückkehr von `open()` installiert.

**Loom-Modell (Merge-Blocker, `crates/contextra-store/tests/loom_fence.rs`):** 3 Threads (Installer, Leader, Writer) und **3** Szenarien: (a) Fence vor Fenster 1, (b) Fence zwischen Fenster 1 und Fenster 2 (WAL-I/O), (c) Fence nach Fenster 2. Zu zeigen: kein konfligierender Record mit `seq > as_of_snapshot` wird nach Drain publiziert; kein Deadlock; Seq-Vergabe bleibt dicht. Leader-Eigen-Ops (Seq-Vergabe, Rückbau bei eigenem Reject) sind Teil des Modells.

### 5.9 Ledger, Manifest-Marke, Formatversionierung

**Integritätsschlüssel.** Assured erfordert einen Store-Integritätsschlüssel (32 Byte, echte Entropie über den `EntropyRng`-Port, kein deterministischer Generator). Ablage: `<store>/integrity.key` (Modus 0600). Mit `encryption-at-rest` wird er stattdessen per HKDF aus dem KMS-Masterschlüssel abgeleitet (Info `contextra/integrity/v1`). Fehlt der Schlüssel, obwohl die Marke ≥ `Assured` verlangt, scheitert `open()` mit `PolicyViolation::IntegrityKeyMissing`. Es gibt keine automatische Neuerzeugung. **Ehrlichkeit:** Der Schlüssel schützt vor Konfigurationsdrift und versehentlichem Downgrade, nicht vor einem Angreifer mit Dateizugriff (er liegt neben den Daten).

**Rotation und Schlüssel-IDs (E60).** Jeder Ledger-Record und das Manifest tragen `integrity_key_id: u32`. Rotation erzeugt einen neuen Schlüssel mit neuer ID. Alte Schlüssel bleiben im Keyring als verify-only. Rotation schreibt einen Record `KeyRotated{from, to}` mit MAC unter dem alten Schlüssel, damit die Kette unter dem neuen Schlüssel nachweisbar anschließt. Ein verlorener alter Schlüssel macht Records unverifizierbar. Das führt zu fail-closed `open()`, nie zu einem stillen Neustart.

**Zeitanker (E59).** Das Manifest speichert `mono_anchor = (wall_ms_max, mono_ctr_max)`, MAC-geschützt, und aktualisiert es bei jedem Ledger-Append. `open()` vergleicht die Wanduhr mit dem Anker. Eine Rücksetzung über die Toleranz (Startwert 60 s) ergibt `PolicyViolation::ClockRegression`, fail-closed. Die Quarantäne (`KeyQuarantined{until}`) wird in monotonen Ticks ab dem Anker gemessen, nicht in der Wanduhr. **Ehrlichkeit:** Ein Angreifer mit Schreibzugriff (Klasse A2, §5.16) kann Anker und Manifest zusammen zurücksetzen. Dagegen hilft nur der externe Anker (`ledger_anchor_export`).

**Ledger-Format (IST-Basis, E65):** Das Ledger bleibt beim bestehenden Record-Format aus `erasure_ledger/entry.rs` und erhält einen Datei-Header. Ein Eintrag enthält `ledger_seq`, `kind` (Key oder Prefix), `keyed_hash` (32 Byte), `erased_at_tx`, `proof_id` (16 Byte). Schlüsselbytes werden nie gespeichert. Die Kettenabdeckung ist BLAKE3-keyed. Das ist eine dokumentierte Ausnahme zur HMAC-Regel (FE-05).
```
Datei-Header:    MAGIC "CTRALEDG"(8) | version u32   (Version 0 = header-los, wird beim Öffnen angehoben)
Record (IST):    ledger_seq u64 | kind u8 | keyed_hash [32] | erased_at_tx u64 | proof_id [16]   (Big-Endian, ≤ 1024 Byte)
  chain_hash  =  BLAKE3(prev_chain_hash || record)
  mac         =  BLAKE3-keyed(Schlüssel aus integrity_key, Domain "contextra/erasure-ledger/v1", chain_hash)
  Typen v1:      Pending · Purged · KeyRevoked · KeyDestroyed · ModeChange
```
- **Kein Klartext-Intent im Ledger** (E65). Der Scope liegt im versiegelten Scope-Journal (§5.18, E66). Das Ledger bleibt append-only, Einträge werden nie gelöscht.
- `Purged`-Eintrag: nur Keyed-Hashes der Schlüssel und Präfixe, kein Klartext.
- **Keine Kompaktierung nötig,** weil das Ledger nie Klartext enthält. Das Wachstum liegt bei ca. 100 Byte je Löschung. Eine Kompaktierung braucht ein eigenes ADR.
- Das Ledger wächst nur um Digests (ca. 100 Byte je Löschung); der Start liest einen **Digest-Index** (Hash-Set), nicht alle Klartexte. Ziel Startzeit: gemessen in M5.2, Startwert ≤ 250 ms bei 1 Mio. Einträgen.
- **Migration (E65):** Das IST-Format wird unverändert gelesen. Upgrade-on-open ergänzt den Header und schreibt Version 1. Ein Golden-Fixture des IST-Formats ist Pflicht (G-16).

**Manifest-Header** (neue/erweiterte Einträge, MAC mit dem Integritätsschlüssel):
`erasure_high_water_mark: ErasureMode`, `ledger_head_hash`, `ledger_record_count`, `id_width`, Formatversionen je Dateityp. Ein Ledger-Kürzen am Ende (Truncation) wird durch Abgleich von `ledger_head_hash`/`ledger_record_count` erkannt und führt zu fail-closed `open()`. Optionaler externer Anker: `ledger_anchor_export()` liefert Kettenkopf und Zähler zur externen Ablage.

**Hochwassermarke (monoton):**
1. `open(mode)` mit `mode < high_water_mark` → `PolicyViolation`.
2. Eine Erhöhung der Marke ist jederzeit erlaubt und wird im Manifest (MAC) und im Ledger (`ModeChange`-Record ohne Token) festgehalten.
3. Ein Downgrade erfordert einen `AcknowledgeToken` (HMAC über `from|to|nonce|wall_ms` mit dem Ack-Schlüssel, erzeugt durch ein Admin-Kommando. Der Ack-Schlüssel wird per HKDF-SHA-256 aus dem Integritätsschlüssel abgeleitet (Info `contextra/ack/v1`, E75)) und erzeugt einen `ModeChange`-Record. Ohne Token: `PolicyViolation`.
4. Der Modus ist je geöffneter Instanz unveränderlich.

**Formatversionierung:** Der vorhandene `ManifestFormatTracker` (`FormatVersionRecord{max_version, migrated_from}`, `FormatError::DowngradeAttempt`) wird erweitert, nicht ersetzt. WAL, SSTable-Header, Ledger, Manifest und Beweis folgen **einer** Regel: Upgrade-on-open, Downgrade immer abgelehnt. Vor jedem Merge, der ein Format ändert, läuft der Upgrade-Test mit Golden-Store-Fixtures.

**RestoreGuard (streamend)** — `StreamingRestoreGuard { ledger_index, revocation_index }`: `admit(key, source_tx)` lehnt ab, wenn ein Ledger-Eintrag existiert, dessen Scope-Digest den Schlüssel oder eines seiner strukturellen Präfixe (Mandant, Collection, Dokument) deckt und dessen `tx > source_tx` ist (Löschung nach Erstellung der Quelle), oder wenn der zugehörige Schlüssel widerrufen ist. Der Restore verarbeitet Quelldaten als Strom, ohne sie vollständig zu laden. In Modus `Off` gibt es keine Barriere (dokumentierte Nicht-Garantie).

### 5.10 Inventar abgeleiteter und redundanter Daten

Jedes Speicherartefakt mit Dokumentinhalt oder Dokument-Bezug hat eine Löschstrategie. Ein neues Artefakt darf nur eingeführt werden, wenn diese Tabelle (und der Scan-Layer in EP2) erweitert wird.

| Speicher | Crate | Enthält | Strategie bei Löschung | Scan-Layer | Meilenstein |
|---|---|---|---|---|---|
| Memtable (aktiv, immutable) | store | Records | Flush, dann Purge der SSTables | 1 | M2 |
| SSTables, Bloom, Blob, `copy_registry` | store | Records, Werte | Purge v2 §5.7 | 3 | M2 |
| Manifest (Schlüsselgrenzen, Altversionen) | store | Key-Ranges | Neuberechnung, Alt-Versionen scrubben | 22 | M2 |
| WAL, aktives Segment | store | Klartext-Records | Memtable flushen, danach alle Segmente, deren Records vollständig in SSTables liegen **und** Scope-Daten enthielten, per `scrub_and_remove` entfernen; aktives Segment rotieren, dann dasselbe | 4 | M5 |
| Erasure-Ledger | store | Keyed-Hashes (kein Klartext, E65) | Keine Löschung und keine Kompaktierung (append-only); Scope liegt nur im versiegelten Scope-Journal (§5.18, E66) | 26 | M5 |
| Checkpoints | checkpoint | Vollständiger Zustand | Collection/Tenant-Löschung: betroffene Checkpoints scrubben und entfernen. Dokument-Löschung: nur Restore-Barriere (Daten bleiben physisch), Option `checkpoint_scrub_on_erase` | 5 | M5 |
| HNSW (Arena, Graph, mmap) | vector | Vektoren, Kanten | Payload-Slot der Arena nullen, Kanten ohne Umverdrahtung entfernen (VETO-F02 in `VETOES.md`, bedingt akzeptiert bis 2027-10-12; E68). `remove_with_graph_repair` (`contextra-vector/src/hnsw/deletion.rs`) wird für Löschungen nicht verwendet. **Nicht in-place nullbare mmap-Knoten:** Segmentdatei ohne die Knoten neu schreiben, alte scrubben | 3 | M5 |
| `quantize`, `rabitq`, `diskann` | vector | Codes/Kopien | Codes des Dokuments nullen bzw. Segment neu schreiben; Zuordnung `doc_id → slot` vollständig | 3 | M5 |
| Text-Index (Postings, Doc-Länge, Stored Fields) | text | Terme je Doc | Postings entfernen, Segment neu schreiben, Alt-Segment scrubben | 20 | M5 |
| Embedding-/Query-Cache | engine | Vektoren je Doc | Einträge je Doc entfernen | 24 | M5 |
| KV-Cache Tier 1/2, Prefix-Trie | kvcache | Attention-Blöcke | Blöcke mit Doc-Bezug verwerfen und scrubben (bei Prefix-Teilung den ganzen Block), Trie-Knoten entfernen | 6, 7 | EP4 |
| Graph (Kanten, Knoteneigenschaften, Hyperkanten) | graph | Triples | Kaskade mit Zyklenschutz; Kanten behalten Provenienz-Menge, Gewicht wird neu berechnet | 21 | EP3 |
| LLM-extrahierte Triples (Cache) | graph/engine | Triples | Einträge je Quelle entfernen | 21 | EP3 |
| OpenIE-Ereignisse (Chunk-Hash, Modell-ID, Prompt-Hash, Ergebnis) | engine | Extrahierte Entitäten und Relationen aus Dokumenten | Einträge je Quell-Chunk entfernen; der Cache-Schlüssel enthält den Dokumentbezug | 21 | EP3 |
| Relink-Graph (ephemer) | engine | Query-bezogene Kanten | Nur Prozessspeicher, nach Request verworfen; kein Persistieren ohne Eintrag in dieser Tabelle | — | EP3 |
| Kanten-Attributmasken (EMA-Kandidat) | vector | Prädikatsbits je Kante, Attributbezug zum Dokument | Bits beim Segment-Rewrite neu schreiben; Dokumentbezug über `doc_id → slot` | 3 | nur nach ADR |
| SVD-Faktoren (Brand) | cognition | Spaltenräume | Kein Downdate möglich → Markierung `dirty`, Rebuild aus Inzidenzmatrix bei Konsolidierung; bis dahin Faktoren für betroffene Mandanten gesperrt | 23 | EP5 |
| Conformal-Kalibrierset, Bandit-Zustand | rank/adapt | Scores je Doc/Query | Kalibrierscores speichern **ohne Dokumentinhalt**, Schlüssel `doc_id`-Digest; bei Löschung Einträge entfernen. Bandit-Zustand enthält keine Dokumentdaten, Reset bei Mandanten-Löschung | — | EP5 |
| Audit-Kette (Egress u. a.) | privacy/evidence | Ereignisse | Nur Hashes/Pseudonyme in die Kette, nie Klartext; Kette bleibt unveränderlich | 40 | EP6 |
| Logs, Tracing, Health | alle | IDs | Doc-IDs nur pseudonymisiert (keyed hash) | — | M6 |
| Snapshot-Pins | store/mvcc | Referenzen auf alte Versionen | Pin-Policy unten | — | M5 |
| Prozessspeicher, Swap, Core-Dump | — | alles | Nicht abgedeckt; `sys::harden_process()` setzt `RLIMIT_CORE=0` und `PR_SET_DUMPABLE=0` (Linux) beim Start von `contextra-mcp` | — | M5 |

**Snapshot-Pin-Policy:** `max_snapshot_age` (Startwert 15 min, konfigurierbar). Ein Reader, dessen Snapshot älter ist, erhält beim nächsten Zugriff `SnapshotExpired`. Der Purge wartet höchstens `pin_grace` (Startwert 30 s) auf Pins mit `snapshot_seq <= as_of`, danach werden sie zwangsweise ablaufen gelassen, weil die Löschpflicht Vorrang vor Reader-Komfort hat. Solange Pins warten: Zustand `PurgePending{PinnedSnapshot}` und Health-Eintrag.

### 5.11 Mandantenpfad

1. Der Löschpfad nutzt ausschließlich `storage.as_erasure_backend()`. `TenantScopedStorage` implementiert `StorageErasureExt` und **delegiert** an das innere Backend mit seinem Mandantenpräfix. Es gibt keinen `downcast_ref::<LsmStorage>()` mehr im Workspace (Gate G-11: 0 Treffer).
2. Ist ein Backend ohne `ErasureBackend` aktiv und der Modus ≥ `Assured`, scheitert `delete()` fail-closed mit `EraseError::NotStarted(CapabilityUnsupported)`. Im Modus `Off` greift das nicht.
3. `tenant_id()` liefert `Result<TenantId, ContextraError>`. `unwrap_or_default()` ist verboten. `kv_hooks.remove_doc_segments` erhält die aufgelöste `TenantId` als Pflichtparameter.
4. `purge_tenant` löscht physisch (Scope `Tenant`), oder wird in `forget_tenant_logical` umbenannt. Ein Name darf keine Garantie behaupten, die nicht gilt.
5. Tests laufen als Matrix über `LsmStorage` und `TenantScopedStorage` (REG-15).
6. `TenantId(pub u64)` und `TxId(pub u64)` (`contextra-types/src/types/domain/ids.rs`) werden gekapselt. Das Feld wird privat, Erzeugung nur über `try_new` oder `TryFrom`. So entsteht kein Mandanten- oder Transaktionswert ohne Prüfung (E74, R2-02).

### 5.12 Crash- und Chaos-Matrix

Jede Zeile ist ein Test. Fehlerinjektion über `fault-injection`-Failpoints.

| ID | Absturzzeitpunkt | Erwarteter Zustand nach Neustart | Test |
|---|---|---|---|
| CRASH-01 | Nach Ledger-Append, vor Tombstone-Commit | Recovery rollt vor: Tombstone wird gesetzt, Purge läuft. Ein dem Aufrufer gemeldetes `Err` war `Indeterminate`. | `chaos_ledger_orphan` |
| CRASH-02 | Während Rewrite | `.purge_tmp` wird gescrubbt entfernt, altes Manifest aktiv, Intent-Resume | `chaos_sst_rewrite_crash` |
| CRASH-03 | Nach neuer SSTable, vor Manifest | Verwaiste neue SSTable verworfen (gescrubbt), Purge startet neu | `chaos_pre_manifest_crash` |
| CRASH-04 | Nach Manifest, vor Scrub | Resume führt Scrub der Altdateien aus | `chaos_post_manifest_scrub` |
| CRASH-05 | `ENOSPC` im Rewrite | `PurgePending{DiskSpace}`, keine Teildatei übernommen | `chaos_disk_full` |
| CRASH-06 | Lesefehler (Bit-Rot) | `ReadError`, kein `Ok`, kein Zeuge, Health-Alarm | `chaos_bitrot_fail_closed` |
| CRASH-07 | Schreibzugriff während Purge | `WriteFenced`; Loom-Szenarien (a)–(c) | `chaos_fence_contention`, `loom_fence` |
| CRASH-08 | Restore aus altem Checkpoint | `RestoreGuard` lehnt gelöschte Schlüssel ab | `chaos_anti_resurrection` |
| CRASH-09 | Nach Manifest und Scrub, vor `Purged`-Ledger-Record | Resume erkennt abgeschlossenen Rewrite (idempotent), schreibt Record | `chaos_pre_purged_record` |
| CRASH-10 | Aufrufer-Future wird nach Commit-Punkt verworfen | Löschung läuft zu Ende | `chaos_cancel_after_commit` |
| CRASH-11 | Ledger-Ende abgeschnitten | `open()` schlägt fail-closed fehl (Zähler/Kettenkopf stimmen nicht) | `chaos_ledger_truncate` |
| CRASH-12 | Pin hält alte Version | `PurgePending{PinnedSnapshot}`, nach Grace Zwangsablauf, Purge läuft | `chaos_pinned_snapshot` |

### 5.13 Crypto-Erase (nur mit `encryption-at-rest`)

- **Schlüsselhierarchie:** Master-Key (Argon2id aus Passphrase oder KMS) → KEK je Mandant → DEK je Collection **und optional je Dokument** → verschlüsselt Chunks, Embeddings, KV-Segmente.
- **AEAD:** AES-256-GCM-SIV. Schlüsseltypen: `Zeroize + ZeroizeOnDrop`, **kein `Clone`**, kein `Debug`-Ausdruck des Materials.
- **Entropie:** Schlüsselgenerierung nutzt den `EntropyRng`-Port mit echter Entropiequelle (`SystemRng`, E55). Deterministische Generatoren sind nur in Tests zulässig und dort typseitig getrennt.
- **Granularität:** Primärmechanismus ist Crypto-Erase auf **Collection und Tenant**. Für **Einzel-Dokumente** gilt: Mit `encryption-at-rest` und aktivierter Option `per_document_dek` ist Crypto-Erase des Dokument-DEK der Primärmechanismus, der Rewrite ist ergänzend. Ohne `per_document_dek` oder ohne Verschlüsselung ist der Rewrite (§5.7) der einzige Mechanismus, und der Claim fällt auf „physisch überschrieben, Medien-Remanenz nicht abgedeckt" zurück.
- **Zwei Stufen:** Stufe 1: Schlüssel sofort widerrufen (`KeyRevoked`), danach ist keine Entschlüsselung möglich; Zustand `KeyQuarantined{until}`. Stufe 2: nach Ablauf zeroisieren (`KeyDestroyed`). Fenster konfigurierbar: Standard **72 h nur für Collection/Tenant**, **0 h für Dokumente**. Während der Quarantäne verlangt Claim-Text „widerrufen, Material bis T in Quarantäne".
- **Kein DEK-Backup, kein Schlüssel-Escrow** für gelöschte Scopes. Ein Backup würde die Löschung umkehrbar machen.
- **Schlüsselverlust** ist ein Datenverlust-Risiko des Betreibers, das in der Betriebsdokumentation steht; es wird nicht durch Schlüsselkopien gelöst.
- **Lock-freie Revocation:** `kv_shredding` ersetzt das offene TODO durch atomare Bitmasken (`AtomicU64`-Slices) je Key-ID-Generation, O(1) ohne Lock im Lesepfad.

### 5.14 Zulässige Claims (REG-16)

| Thema | Zulässig | Unzulässig |
|---|---|---|
| Löschung `Assured` | „Daten des Scopes wurden im Store überschrieben und entfernt; Wiederherstellung aus Checkpoints wird verhindert." | „unwiederbringlich gelöscht", „DSGVO-konform gelöscht" |
| Medien | „SSD-Remanenz, CoW-Dateisysteme, Backups, Swap und Prozessspeicher sind nicht abgedeckt." | „garantiert SSD-sicher" |
| Crypto-Erase | „Schlüssel widerrufen/vernichtet; verschlüsselte Daten sind nicht mehr entschlüsselbar" (nur bei Verschlüsselung) | gleiche Aussage ohne Verschlüsselung |
| Beweis (EP2) | „Löschnachweis nach Stand der Technik, unabhängig prüfbar" | „rechtsverbindlich", „mathematisch unanfechtbar", „§ 371a ZPO" |
| Normen | „orientiert an BSI TR-02102-1 mit Abweichungsliste" | „BSI-TR-02102-konform" |
| Dateisystem | „Dauerhaftigkeit nur auf Tier-1-Plattformen und bei unterstützten Dateisystemen (§5.17)“ | „Daten sind auf dem Medium sicher gelöscht“, „Löschung wirkt auf Snapshots und Journal“ |
| Algorithmik | „Ergebnis reproduzierbar je ISA-Pfad" (§7.6) | „bit-identisch über alle Plattformen" für Fließkomma |

Begründung: BSI TR-02102 ist eine Empfehlung zu Algorithmen und Schlüssellängen, kein Löschmodell; ob BLAKE3 und AES-GCM-SIV dort aufgeführt sind, ist ungeprüft. Ein selbstverwalteter Ed25519-Schlüssel ist keine qualifizierte elektronische Signatur. Jede juristische Aussage braucht eine Rechtsprüfung (§14).

### 5.15 Lizenz und Löschmodi
- Die Lizenz (`Sovereign`) steuert ausschließlich das **Ausstellen neuer Beweise** (`Attested`).
- Lizenzablauf oder -fehlen **hebt weder die Hochwassermarke an noch verhindert es `open()`** und senkt den Modus nicht. Löschungen laufen weiter in `Assured`-Qualität; die Beweisausstellung scheitert mit `LicenseExpired`, der Beweis bleibt im Zustand `Scanning`/ausstehend. Es entsteht nie ein Beweis ohne gültige Lizenz und nie eine stille Herabstufung.
- Engine und Store kennen die Lizenz nur über den Port `LicenseGate`.

---

### 5.16 Bedrohungsmodell (E58)

Die Claims in §5.14 gelten nur gegen die Angreiferklassen, die hier als abgedeckt markiert sind. Jede andere Klasse ist Nicht-Garantie.

| Klasse | Fähigkeit des Angreifers | Abdeckung | `Off` | `Assured` | `Attested` (EP2) |
|---|---|---|---|---|---|
| A0 | Legitimer Client, falscher Mandant | Mandantenbindung (§8.2), Tenant-Matrix (REG-15) | nein | ja, fail-closed | ja |
| A1 | Lesender Zugriff auf Store-Dateien ohne Root | Verschlüsselung (`encryption-at-rest`) | nein | nur mit Verschlüsselung | nur mit Verschlüsselung |
| A2 | Schreibender Zugriff auf Store-Dateien (Rollback, Manipulation) | MAC-Kette, Hochwassermarke, Zeitanker, externer Anker | nein | Erkennung nur mit externem Anker | Erkennung über Beweis; externer Anker Pflicht |
| A3 | Forensik auf Medien, Backups, Snapshots, CoW | — | nein | nein (§5.2) | nein |
| A4 | Prozess- oder Root-Zugriff, Speicherauslesen, Core-Dumps | `harden_process` (nur Linux, teilweise) | nein | nein | nein |

**Folge:** Der Integritätsschlüssel liegt neben den Daten (§5.9). Gegen A2 wirkt er nur gegen Konfigurationsdrift, nicht gegen einen Angreifer mit Schreibzugriff. Diese Grenze gehört in Doku und Fehlermeldungen (`PolicyViolation`, REG-16).

### 5.17 Betriebsannahmen (Sperre, Plattform, Zeit)

1. **Einzelprozess-Sperre (E56).** `open()` nimmt eine exklusive `flock`-Sperre auf `<store>/LOCK`. Die Sperre gilt für die Lebensdauer des Store-Handles. Scheitert sie, liefert `open()` `StoreLocked`. Es gibt keinen Fallback auf „read-only trotzdem". Test: Zwei Prozesse, der zweite `open()` scheitert (G-24).
2. **Plattform-Tiers (E57, vorläufig, O-17).** Tier 1 (Pflicht für `Assured`-Claims): Linux mit einem Dateisystem, das `fsync` auf Verzeichnisse verlässlich umsetzt (ext4, XFS). Tier 2: macOS nur mit `F_FULLFSYNC` in `durable-fs` und eigenem Crash-Test; ohne Test kein Claim. Nicht unterstützt in v0.1.0: Windows, da kein Dauerhaftigkeitsbeleg vorliegt. CoW-Dateisysteme (btrfs, ZFS, APFS-Snapshots) sind Nicht-Garantie nach §5.14.
3. **Zeitbasis (E59).** Die Wanduhr dient nur der Anzeige. Ordnung und Fristen laufen über Ticks und den Zeitanker (§5.9).
4. **Medienannahme.** SSD-Remanenz ist Nicht-Garantie. `scrub_and_remove` ist eine Überschreibung auf dem logischen Blockgerät (§5.2).
5. **Ausnahmen zu Gates** gelten nur über die befristete Ausnahmeliste (§4.3a, E54, G-21).

---

### 5.18 Resume, Sichtbarkeit und Scope-Journal (E66, E76)

1. **Versiegeltes Scope-Journal** `purge/scope-<intent_id>.sealed`. Inhalt ist der `PhysicalScope`, verschlüsselt mit AES-256-GCM-SIV. Der Schlüssel wird per HKDF-SHA-256 abgeleitet (Info `contextra/purge-scope/v1`). Die Datei wird fsync-fest vor `LedgerAppended` geschrieben und nach `Completed` mit `scrub_and_remove` vernichtet. Fehlt sie bei vorhandenem Intent, scheitert `open()` fail-closed.
   **Ehrlichkeit:** Ohne `encryption-at-rest` liegt der Schlüssel neben den Daten. Dann gilt die Schutzgrenze A1 aus §5.16 nicht. Mit Verschlüsselung leitet sich der Schlüssel aus dem KMS-Master ab.
2. **Resume-Reihenfolge in `open()`:** Storage aufbauen, Manifest und WAL-Replay mit Ledger-Filter. Danach Ledger laden und prüfen, offene Intents lesen, Scope aus dem Journal rekonstruieren und Fences setzen. Erst dann gibt `open()` `Ok` zurück. Der Rest des Purge läuft im Hintergrund weiter.
3. **Sichtbarkeit:** Solange ein Ledger-Eintrag ohne `Purged` besteht, sind die betroffenen Schlüssel für Leser unsichtbar. Der Filter greift im Lesepfad und ist über `ReadSnapshot` konsistent (PV3).
4. **Dispose-Policy:** In Produktion ist `RequireScrub` gesetzt (`recovery.rs:546`). No-op-Hooks sind nur im Modus `Off` zulässig und werden durch G-31 geprüft.
5. **Grenze:** Die Sichtbarkeitsregel gilt für den Store. Kopien in Checkpoints sind über den RestoreGuard (§5.9) abgedeckt, nicht über den Filter.

## 6. Evidenz und Löschbeweis (EP2, Modus `Attested`)

Dieses Kapitel ist für EP2 normativ. EP1 enthält davon nur die Typen im Port (§5.5) und die Entfernung der fabrizierten Zeugen. Die Formatentscheidungen stehen jetzt fest, damit EP1 nichts baut, was EP2 brechen müsste.

### 6.1 Kryptografische Suite (verbindlich)

| Aufgabe | Festlegung |
|---|---|
| AEAD | AES-256-GCM-SIV (`aes-gcm-siv`) |
| Master-Key | Argon2id |
| Subkeys | HKDF-SHA-256 mit Domain-Info-Strings |
| WAL- und Ledger-MAC | HMAC-SHA-256 |
| Beweis-, Merkle- und Kettenhash | **BLAKE3** |
| Signatur | Ed25519, Verifikation ausschließlich mit `verify_strict` |

ChaCha20, plain AES-256-GCM, HKDF-SHA-512 und SHA-256 als Beweis-Digest sind gestrichen. `bincode` darf nie im Signaturpfad stehen (nur für Legacy-Deserialisierung).

### 6.2 Beweisformat v4 (normativ)

- **Serialisierung:** TLV mit `u16`-Tag und `u32`-Länge, beides **Big-Endian**. Critical-Bit `0x8000` im Tag: Ein unbekannter kritischer Tag führt zum Abbruch der Verifikation, ein unbekannter nicht-kritischer Tag wird übersprungen, aber mitsigniert.
- **Domain-Tag:** `contextra/deletion-proof/v4` plus ein NUL-Byte (28 Byte) steht als Präfix vor den signierten Bytes. Die frühere Darstellung „32 Byte Magic" und alle festen Offset-Tabellen (236/381 Byte) gelten nicht. Die Tag-Tabelle wird aus dem Code **generiert** (`cargo xtask gen-proof-tags`) und mit **Golden-Testvektoren** abgesichert.
- **Kein CBOR**, keine Little-Endian-Tabellen.
- **Felder (mindestens):** `version=4`, `tenant_id`, `scope_digest` (BLAKE3), `erased_at_tx`, `issued_wall_ms` + `issued_mono_ctr` (aus `Clock`), `stage` (`0x8004`: 1 Interim, 2 Final), `supersedes` (`0x8005`), `layers[]` (je `layer_id: u16`, `status`, `state_digest [32]`, `item_count`), `chain` (Kettenhash), `wal_chain_receipt`, `signer_key_id`, `signature`.
- `signature` ist in `Attested` **kein `Option`**.
- `LayerStatus`: `Clean`, `Dirty` (führt nicht zu einem Beweis), `NotApplicable{reason}`. `Pending` darf in einem signierten Beweis nicht vorkommen.
- **Signierter Bereich:** Domain-Tag plus alle TLV-Felder in aufsteigender Tag-Reihenfolge außer `signature`. Der eingebettete Public Key dient nur als Hinweis (`signer_key_id`), **nie** als Vertrauensanker.
- Zeitstempel: Ausstellungszeit ist informativ; Reihenfolge und Beweisführung stützen sich auf `erased_at_tx` und die Kette.

### 6.3 Layer-Modell
Die `DeletionLayer`-Varianten (IDs 1–7, 20–26 und 40, jeweils `u16`; Layer 40 nur in EP6, E52) sind die **normative Menge der Scan-Layer**. Für Auditoren gibt es eine abgeleitete Gruppierung in sieben Beweisklassen, die als Tabelle im Verifier liegt, nicht im Beweisformat:

| Klasse | Inhalt | Layer-IDs |
|---|---|---|
| 1 Logisch | Tombstone, Ledger | 1, 26 |
| 2 Physisch | Datenträger, Manifest | 2, 22 |
| 3 Indizes | Vektor, Graph, Text, Relationen | 3, 5, 20, 21 |
| 4 WAL | Log-Segmente | 4 |
| 5 Kopien und Caches | KV-Cache, Embedding-Cache, Checkpoints, Prozess-Caches | 6, 7, 23, 24 |
| 6 Krypto | Schlüsselvernichtung | 25 |
| 7 Audit | Hash-Kette, WAL-Receipt | kein Scan, abgeleitet aus `chain` und `wal_chain_receipt` |
| 7b Egress-Audit (EP6) | Egress-Auditkette, nur Hashes | 40 |

Neue Layer werden ergänzt, ohne das Format zu brechen. Jeder Layer hat ein eigenes Digest-Domain-Tag (`contextra/layer/<name>/v1`). „Replica-ACK" entfällt (REG-05, keine Replikas); Layer 5 meint Checkpoints. Ein Pflicht-Layer mit Teil- oder `Uncovered`-Status erzeugt keinen Beweis, sondern einen Fehler mit Layer-ID (E42).

### 6.4 Verifier `contextra-proof-verify`
- Abhängigkeiten ausschließlich `ed25519-dalek` und `blake3`; `no_std`-fähig; **kein** Zugriff auf `crypto-core`.
- Prüft Domain-Tag, TLV-Struktur und Critical-Bits, Layer-Pflichtmenge, Kettenhash und `verify_strict`.
- **Vertrauensanker:** Der Verifier akzeptiert einen Beweis nur gegen einen vom Auditor bereitgestellten Public Key oder eine Key-Registry. Ablage, Rotation, Verlust und Registry der Signierschlüssel werden zu Beginn von EP2 spezifiziert (§14, offen). Bis dahin liefert der Verifier ausdrücklich „Signatur gültig für Schlüssel X", nie „vertrauenswürdig".
- Legacy: v2/v3-Beweise sind **verify-only**. Beweise, die mit dem v4-Tag aus dem alten bincode-Pfad stammen, werden als „Legacy, nicht vertrauenswürdig" gekennzeichnet.
- Die v3-Erzeugung (`construct_v4_payload` und alle `create`-Funktionen in `proof_v3.rs`) wird gelöscht; es gibt kein Feature `legacy-proofs`.

### 6.5 Tampering- und Mutationstests (Pflicht in EP2)
- **Tampering-Matrix:** Mutation jedes Feldes (Domain-Tag, jeder TLV-Tag, jede Layer-Statusbyte, Digests, Kette, Signatur, Public Key, `stage`, `supersedes`) → Verifikation lehnt ab.
- **Mutationstest:** Ein No-Op-Scanner (immer `clean`, `item_count = 0`) → kein Beweis. Ein Scan mit `item_count = 0` für einen Pflicht-Layer, der Daten haben müsste, wird als Fehler gewertet.
- **Testvektoren:** Grenzfälle (leere Layerliste, doppelte Tags, unbekannter kritischer Tag, Überlänge) im Verifier.

---

## 7. Concurrency, MVCC, Determinismus

### 7.1 Globale Lock-Reihenfolge (REG-11)

```
Engine-Ebene (async, tokio):  collections → kv_locks → embedder
Erasure-Ebene:                ledger_append_lock → fence_install_lock
Store-Ebene:                  commit_mutex → pending_commit_queue → truncate_lock → state
```
Die Ebenen werden strikt in dieser Reihenfolge genommen; ein Lock einer früheren Ebene wird nie unter einem späteren erworben.

**Regel zu `.await`:** *Synchrone* Locks (`parking_lot`, `std`) werden nie über ein `.await` gehalten. *Asynchrone* Locks (`tokio::sync`) dürfen über `.await` gehalten werden, wenn sie in obiger Liste stehen und die Reihenfolge eingehalten wird. `erase_document` hält `kv_locks` (async) über mehrere Awaits inklusive Commit; das ist zulässig. Das `ledger_append`-`fsync` geschieht **außerhalb** von `commit_mutex`. Mehrschlüssel-Löschungen erwerben Shard-Locks in aufsteigender Shard-Reihenfolge, um Kollisionen zu vermeiden.

Gate G-10 (`check-lock-order`) prüft per statischer Analyse und Loom-Tests, dass keine umgekehrte Reihenfolge vorkommt.

### 7.2 Snapshot-Isolation (REG-10)
```rust
// contextra-ports
pub struct ReadSnapshot { pub id: SnapshotId, pub max_seq: TxId }   // kein Instant, nicht Copy
pub trait SnapshotReader: Send + Sync {
    fn snapshot(&self) -> Result<ReadSnapshot, ContextraError>;
    fn read_at(&self, snap: &ReadSnapshot, key: &[u8]) -> Result<Option<Value>, ContextraError>;
}
```
Store, Vector, Text und (EP3) Graph erhalten `&ReadSnapshot` als **Pflichtparameter** aller Lesefunktionen der Hybridsuche. Das Snapshot-Alter wird über den `Clock`-Port (Ticks) gemessen. Gate G-08 prüft, dass Multi-Signal-Reads dasselbe Token nutzen. Snapshots sind durch die Pin-Policy (§5.10) zeitlich begrenzt.

### 7.3 MVCC-Bereinigung
- `register()`/`register_at()` werden entfernt, alle nutzen `acquire()`.
- `recover_pending_intents` wird in `repair_on_open()` verdrahtet.
- **SSI:** Der Commit-Pfad nutzt den SSI-Validator bereits; das **Read-Set-Tracking im Lesepfad** wird ergänzt (ohne dieses ist die Schnittstelle unvollständig). SSI bleibt der Standard; ein Serial Safety Net wird nur nach Benchmark-Entscheidung geprüft (§9.9).

### 7.4 Zeit, Zufall, IDs (REG-03)
- `Clock`, `EntropyRng`, `SeededRng`, `IdGenerator` liegen in `contextra-ports` (Namen nach E55). Produktivimplementierungen (`SystemClock`, `SystemRng`, `SystemIdGen`) liegen in `contextra-sys`; Testimplementierungen (`ManualClock`, `SeededRng`) in `contextra-testkit`.
- **Zeitmodell:** `DeterministicTimestamp { wall_ms: u64, mono_ctr: u64 }`. Dauermessungen sind Differenzen monotoner Ticks aus dem `Clock`-Port, nie `Instant`.
- `contextra-checkpoint` erhält `Arc<dyn Clock>` und `Arc<dyn IdGenerator>` (kein `SystemTime::now`, kein `rand::random`). Das erfüllt `INV-CHECKPOINT-DETERMINISM-1`.
- **Schlüssel- und Nonce-Erzeugung** nutzt nie den deterministischen Test-Rng; die Typen sind getrennt (`EntropyRng` vs. `SeededRng`).
- **Namensverbindlichkeit (E55):** Die Namen gelten einheitlich in allen Dokumenten und im Code. Neue Port-Traits brauchen einen Eintrag hier, G-23 prüft den Abgleich.

| Verbindlicher Name | Ort | Zweck |
|---|---|---|
| `Clock` | `contextra-ports/src/clock.rs` | Monotone Ticks und Wanduhr (`DeterministicTimestamp`) |
| `EntropyRng` | `contextra-ports/src/rng.rs` | Schlüssel, Nonces; Impl `SystemRng` in `contextra-sys` |
| `SeededRng` | `contextra-ports/src/rng.rs` | Nur Tests und Benches; Impl in `contextra-testkit` |
| `IdGenerator` | `contextra-ports/src/id_gen.rs` | Zufalls-IDs (PurgeId, Intent-Dateien) |

`Rng` als alleiniger Name entfällt. Die Stellen in v1.1 (REG-03, §5.9, §5.13) sind entsprechend korrigiert (F-39).

### 7.5 Clippy-Durchsetzung (Gate G-09)
Workspace-`clippy.toml` mit `disallowed-methods` für `std::time::Instant::now`, `std::time::SystemTime::now`, `rand::thread_rng`, `rand::random`, `f32/f64::partial_cmp` in `rank`/`vector`. Ausnahmen nur in `contextra-sys` (Port-Implementierungen), Tests, Benches, Fuzz.

**Stand v1.3 (IST am HEAD `90b23431`, F-48):** `clippy.toml` enthält nur `Instant::now`, `thread_rng` und `partial_cmp`. `SystemTime::now` und `rand::random` fehlen. **Ausnahmemechanismus (E61):** `clippy.toml` gilt workspaceweit und lässt keine Ausnahme pro Crate zu. Ausnahmen erfolgen deshalb über `#[allow(clippy::disallowed_methods)]` mit Kommentar `// DETERMINISM-EXCEPTION: <Grund>`. Die Ratsche zählt diese Stellen. Ihre Zahl darf nur sinken.
Workspace-Lints (Warnung in Ring 2+, **Fehler in Ring 0 und Ring 1**, schrittweise per Ratsche): `indexing_slicing`, `arithmetic_side_effects` (für Index-/Größenrechnung), `float_cmp`, `unwrap_used`, `expect_used`, `panic`.

### 7.6 Fließkomma-Politik (verbindlich)

1. **Reproduzierbarkeit je ISA-Pfad, nicht über ISAs.** FMA (AVX2, NEON `vfmaq_f32`) ändert die Rundung gegenüber Nicht-FMA; Scalar, AVX2 und NEON sind daher nicht bit-identisch. Kahan-Summation stellt diese Gleichheit nicht her und ist **nicht** Teil der Politik.
2. **Bit-Identität über ISAs** wird nur für Integer- und Festkomma-Pfade zugesagt: RRF/DiBud (Q64.64), Beweise, Wire-Format, Ledger, Hashing.
3. **Reduktionsordnung:** Jeder SIMD-Kernel dokumentiert seine Reduktionsordnung (feste Lane-Zuordnung und festen Baum); der Scalar-Referenzkernel bildet dieselbe Ordnung ohne FMA nach. Ein Test vergleicht SIMD gegen Scalar-Referenz mit expliziter, dokumentierter Toleranz (ULP-Grenze), nicht auf Gleichheit.
4. **HNSW-Aufbau:** Distanzen beim Einfügen in den Index werden mit dem **kanonischen Kernel** (feste Reduktionsordnung, ohne FMA, auf allen ISAs identisch) berechnet; dadurch ist die Graphstruktur ISA-unabhängig. Abfragen dürfen schnelle Kernel nutzen.
5. **NaN/Inf:** Vektoren und Scores mit NaN oder ±Inf werden **am Eingang abgelehnt** (`InvalidInput`), nie gespeichert. Hinweis: `total_cmp` ordnet ein NaN mit gesetztem Vorzeichenbit **vor** −∞; die Behauptung „NaN ans Ende" ist falsch, daher die Eingangskanonisierung.
6. **Vergleiche** in Sortierungen: ausschließlich `total_cmp` (nach der Eingangsprüfung); Gleichstände werden deterministisch über die `DocId` aufgelöst.
7. **Arithmetik:** Index- und Größenrechnungen mit `checked_*`/`saturating_*`; Zugriffe über `.get()` mit Fehlerpropagation.
8. **Keine Kahan-Politik** (E43): Gleitkomma-Summen folgen der Reduktionsordnung aus Punkt 3. Kahan-Summation ist nicht Teil der Politik. Die Aussage „NaN wird ans Ende sortiert" ist falsch, Punkt 5 gilt.

### 7.7 Lifecycle und Reaper
Ein `EngineLifecycle`-Manager bündelt `start_expiry_reaper`, `start_orphan_reaper`, `start_thermostat_reaper` und `start_kv_cache_eviction`. `Contextra::open` startet sie (nur die zum Feature-Satz gehörenden) mit einem gemeinsamen Shutdown-Signal; `close`/`Drop` stoppt sie geordnet. Der `ErasureSupervisor` (§5.3) gehört demselben Manager.

---

## 8. Fassade und MCP

### 8.1 Composition Root
`contextra` (Ring 4) ist die einzige Stelle, die Ports mit Implementierungen verdrahtet (Clock, Rng, IdGen, Storage, `MergeOperatorFactory`, `LicenseGate`, `ErasureHooks`, KMS). `contextra-mcp` delegiert an die Fassade und enthält **keine** doppelte Verarbeitungslogik für `remember`/`recall`.

### 8.2 Mandantenbindung (fail-closed)
- Der Transport ist stdio (Ein-Prozess-Modell). Die **Tenant-ID wird beim Prozessstart gebunden** (Startargument oder `CONTEXTRA_TENANT_ID`) und ist danach unveränderlich. Es gibt **keinen** `SYSTEM`-Fallback; fehlt die Bindung, startet der Server nicht.
- Die Tenant-ID darf **nie** aus Tool-Argumenten kommen. Ein Tool-Argument `tenant` wird abgelehnt.
- Ein Prozess bedient genau einen Mandanten. Mehrmandanten-Betrieb erfolgt über mehrere Prozesse oder über die Bibliotheks-API (`contextra-db`), nicht über MCP.

### 8.3 Dispatcher und Nebenläufigkeit
Der synchrone Kern wird über `spawn_blocking` (begrenzter Pool: `bounded-compute-pool`) aufgerufen. `block_in_place` + `block_on` im async-Kontext entfällt. Nach dem Commit-Punkt eines Löschvorgangs ist der Request nicht mehr abbrechbar (§5.3); vor dem Commit-Punkt ist Abbruch folgenlos.

### 8.4 Tool-Registrierung und Klassifikation (REG-12)
Jedes Tool trägt eine `Classification` (`ReadOnly`, `WriteLocal`, `EgressExternal`). Die Registrierung ist feature-gated (nur `#[cfg(feature = ...)]`, nicht zusätzlich zur Laufzeit):

| Tool | Klasse | Bedingung |
|---|---|---|
| `contextra_search`, `contextra_recall` | `ReadOnly` | immer |
| `contextra_remember` | `WriteLocal` | immer |
| `contextra_forget` | `WriteLocal` | immer |
| `contextra_relate` | `WriteLocal` | Feature `graph` |
| `contextra_explain` | `ReadOnly` | Feature `graph` (nutzt `find_path`) |
| `contextra_cloud_query` | `EgressExternal` | Feature `guard` (sonst nicht sichtbar, fail-closed) |

Neue Tools ohne Klassifikation werden nicht registriert (Test `INV-MCP-CLASSIFY-1`).

### 8.5 `contextra_forget`-Antwort
Das Ergebnis von `erase_document` wird nicht verworfen. Die JSON-Antwort enthält je Modus: `Off` → `{status: "logically_deleted", tx}`; `Assured` → `{status: "erasure_accepted", tx, purge_id, state}` und, falls `Indeterminate`, `{status: "erasure_accepted_indeterminate", purge_id}` (mit Text „wird gelöscht"); `Attested` → zusätzlich `proof` (Base64) nach Abschluss oder `proof_pending`. Das Feld `proof: null` als Dauerzustand ist nicht zulässig; der Claim-Text folgt §5.14.

### 8.6 Abhängigkeitsbereinigung MCP
`contextra-ports` mit `test-utils` wandert in `[dev-dependencies]`. `adapt`, `crypto`, `privacy`, `rank`, `wire`, `router` sind keine direkten oder harten Abhängigkeiten von `contextra-mcp`; benötigte Funktionalität kommt über die Fassade und Features. Das Gate-Kommando aus dem Produkt-Manifest (`--features fast`) muss ausführbar sein, `fast`, `guard`, `graph` existieren als MCP-Features.

---

## 9. Algorithmen (normativ, korrigiert)

Alle Algorithmen laufen unter REG-01/REG-03 und der Fließkomma-Politik (§7.6). Mathematische Aussagen sind analytisch abgeleitet und **vor Produktivfreigabe experimentell zu belegen**; das ist je Abschnitt als Gate angegeben.

### 9.1 Hybridsuche und RRF (EP1, `contextra-rank`)
- Kanäle: dichte Vektorsuche (HNSW), BM25, später Graph. Reciprocal Rank Fusion mit `k = 60`, Rang 1-basiert: `score(d) = Σ_c 1 / (k + r_c(d))`.
- Berechnung in **Festkomma** (`FixedRankScore(u128)`, Q64.64), damit das Ergebnis auf allen ISAs bit-identisch ist. Gleichstände werden über `DocId` aufsteigend aufgelöst.
- Alle Kanäle lesen über dasselbe `ReadSnapshot` (§7.2).

### 9.2 DiBud (budgetierte RRF-Zertifizierung, `contextra-rank`)
Ziel: Ein Top-`k`-Ergebnis über mehrere **materialisierte** Kanallisten zertifizieren, ohne alle Listen vollständig zu lesen.
- Kanalliste `c` hat Länge `n_c`; `p_c` Positionen wurden gelesen. Untere Schranke: `L(d) = Σ_{c: d in gelesenem Präfix} 1/(k + r_c(d))`.
- Restwert eines Kanals: `U_c = 1/(k + p_c + 1)` (0, falls der Kanal erschöpft ist). Obere Schranke eines bekannten Dokuments: `U(d) = L(d) + Σ_{c: d noch nicht in c gesehen} U_c`. Obere Schranke für noch ungesehene Dokumente: `U_unseen = Σ_c U_c`.
- **Zertifizierung** von `d` für den nächsten Rang: `L(d) > max( max_{e ≠ d, bekannt} U(e), U_unseen )` — **strikt `>`**. Mit `≥` entstünde ein Gleichstand mit einem ungesehenen Dokument. Gleichstände werden erst nach Erschöpfung aller lesbaren Positionen über `DocId` aufgelöst.
- **Schrankennormierung (E47):** Für einen nicht erschöpften Kanal `c` gilt `U_c = 1/(k + p_c + 1)`. Die Form `1/(k + n_c)` mit Listenlänge `n_c` ist keine obere Schranke und ist verboten. Ein Gleichstand auf Schranken wird nicht per Dokument-ID entschieden, sondern erst nach Erschöpfung aufgelöst.
- **HNSW ist nicht präfix-stabil** (die Rangliste ändert sich mit wachsendem `ef`). Daher wird die HNSW-Liste **einmal** mit festem `ef` materialisiert; DiBud spart Fusionsarbeit, nicht HNSW-Suchaufwand. Das gilt als Ehrlichkeitsklausel in der Dokumentation.
- **Gate:** Property-Test: das zertifizierte Top-`k` ist **identisch** zum Top-`k` der vollständigen RRF auf zufälligen Listen; synthetischer Ausreißertest; Messung der eingesparten Leseschritte. Der Algorithmus ist eine Eigenableitung und gilt bis zum Bestehen dieser Tests als experimentell (Feature `dibud`, Default aus).

### 9.3 Conformal-Kalibrierung und ACI (`contextra-rank`)
- Split Conformal, **klassenbedingt** auf relevante Dokumente: Aus dem Kalibrierset der als relevant gelabelten Dokumente werden Scores `s_i = 1 − f(d_i)` mit fusioniertem Score `f ∈ [0,1]` gebildet. Schwelle `q̂` = der `⌈(n+1)(1−α)⌉`-te kleinste Wert der `n` Scores (die Ceiling gilt für den Index `⌈(n+1)(1−α)⌉`, **nicht** um den Bruch durch `n`). Ist der Index `> n`, ist `q̂ = +∞` (alle Kandidaten bleiben).
- Vorhersagemenge für eine Anfrage: `C = { d : 1 − f(d) ≤ q̂ }`. Garantie (Austauschbarkeit vorausgesetzt): `P(relevantes Dokument ∈ C) ≥ 1 − α`.
- **Ranggrenze ohne Clamp (E45):** Ist `⌈(n+1)(1−α)⌉ > n`, gilt `q̂ = +∞`, alle Kandidaten bleiben. Ein Clamp auf `n` ist verboten. Das Kalibrierset enthält nur als relevant gelabelte Dokumente. Irrelevante Kandidaten erzeugen keinen zweiten Score-Typ.
- **Label-Quelle:** explizites Feedback (Übernahme/Ablehnung); gespeichert als `(query_digest, doc_digest, f, label)` ohne Dokumentinhalt.
- **Kaltstart:** Bei `n < n_min` (Startwert 30) keine Filterung; Ergebnis als „unkalibriert" markiert.
- **ACI** bei Drift: `α_{t+1} = clamp(α_t + γ·(α − err_t), 0, 1)` mit `err_t ∈ {0,1}`; `γ` als Festkomma-Zweierpotenz (Startwert `2^-8`).
- **Löschung:** Kalibrierscores tragen den `doc_digest`; bei Löschung des Dokuments werden sie entfernt (§5.10).
- **Gate:** Abdeckungstest auf synthetischen Daten (empirische Abdeckung ≥ 1 − α − Toleranz) und Driftszenario.

### 9.4 KV-Cache (EP4, `contextra-kvcache`, `forbid(unsafe_code)`)
- **KIVI-Quantisierung:** Keys pro Kanal, Values pro Token; Standard 2 Bit (Keys 4 Bit konfigurierbar); Gruppengröße `g = 64` (Bereich 32–128); Residual-Fenster der jüngsten 128–512 Tokens in FP16/BF16.
- Quantisierung je Block: `scale = max((x_max − x_min)/(2^b − 1), ε_scale)`; ist `x_max == x_min`, dann `scale = 1`, `zp = 0` (Schutz gegen Division durch null). `zp = clamp(round(−x_min/scale), 0, 2^b − 1)`; `X_q = clamp(round(X/scale) + zp, 0, 2^b − 1)`; `X_deq = scale·(X_q − zp)`.
- Die Behauptung „87,5 % Ersparnis" entfällt; Speicherersparnis wird gemessen.
- **SnapKV-Eviction** braucht Attention-Gewichte und ist daher nur mit dem candle-Backend (Feature `kv-bridge`) möglich. Ollama/ONNX nutzen eine deterministische Recency-Eviction (logische Uhr). Beobachtungsfenster `W ≈ 32`; je Head Top-`k` der gemittelten Attention plus Gleitfenster der jüngsten `L_recent` Tokens.
- **Namensraum (E44):** `Irminsul` bezeichnet ausschließlich die Eviction-Strategie dieses Abschnitts (SnapKV-Scoring, Prefix-Schutz, Budget). Positionsunabhängiges Caching mit Inhalts-Hash und RoPE-Korrektur ist ein eigener Kandidat `PIC` für EP4. `PIC` verwendet nur mandantengebundene Schlüssel und braucht einen ADR und eine Messung der RoPE-Latenz beim Prefill.
- **Scores:** NaN- und ±Inf-Scores werden am Eingang der Eviction abgelehnt (§7.6, Punkt 5). Eine Sortierung ohne Eingangsprüfung ist unzulässig.
- **Pins:** Die 50-%-Grenze der Refcount-Regel gilt auch für Irminsul-Schutz. ADVANCED §1.2 nennt keine Grenze und wird insoweit nicht übernommen.
- **Tier-2-Scrub:** Beim Löschen eines Dokuments werden betroffene Tier-2-Blöcke gescrubbt (§5.10).
- **Prefix-Radix-Baum je Mandant:** Ein Wurzelknoten je Mandant; **keine mandantenübergreifende Teilung** (sonst Timing-Seitenkanal).
- **Refcount-Regel:** Einträge mit `refcount > 1` werden nicht evictet, aber die gepinnte Menge ist auf ≤ 50 % des Budgets begrenzt. Wird das Budget dennoch überschritten, liefert der Einfügeversuch `EvictionBlocked` (Backpressure), nie unbeschränktes Wachstum.
- **Tier 2:** Evictete Blöcke werden mit zstd (Level 1) komprimiert und in mmap-Segmente geschrieben; verschlüsselt, wenn `encryption-at-rest` aktiv ist. Rückholung kostet `O(Blockgröße)` (Dekompression + Dequantisierung), nicht `O(1)`.
- Ausgerichtete Allokation (64 Byte) kommt aus `contextra-sys::AlignedBuf` (sichere API).
- Jeder Block trägt Doc-Bezug für die Löschung (§5.10).

### 9.5 Graph-Ingestion und OpenIE (EP3, `contextra-graph`/`engine`)
Pipeline: Chunk → Satz-Splitting → regelbasierte NER → LLM-Validierung über `LlmExtractionPort` → Kanonisierung → CSR-Einfügung.
- **Kein Determinismus-Claim für `temperature = 0`** (gilt über Hardware und Bibliotheksversionen nicht). Stattdessen: Extraktionsergebnisse werden als **Ereignis persistiert** (Schlüssel: Chunk-Hash + Modell-ID + Prompt-Hash) und bei Wiederholung **aus dem Cache abgespielt**, nie neu vom LLM erzeugt.
- Externe Modelle laufen nur hinter dem Egress-Gate (`guard`).
- Tripel tragen Provenienz (Quell-Chunk); Hyperkanten für `n > 2` Teilnehmer in der CSR-Struktur.
- Kanonische Sortierung: Entitäten `(canon_id, span_start)`, Tripel `(subject_id, predicate, object_id)`. Updates inkrementell ohne Voll-Rebuild.
- **Kein `temperature=0`-Determinismus (E46):** Die Angabe `temperature=0, fixed seed` aus ADVANCED §2.2 wird nicht übernommen. Die Ereignis-Persistenz ist die einzige Determinismusquelle. Die Ingestion-Orchestrierung liegt in `contextra-engine` (Ring 3). `contextra-graph` bleibt Ring 0 ohne Store-Import.

### 9.6 PathRAG (EP3)
- Budgetierte Best-First-Variante von k-kürzesten Pfaden (heuristisch, **nicht** exaktes Yen): Knotenbudget `B` über die Vereinigung aller Pfade; Relevanz über Personalized PageRank (Forward Push, Schwelle ε); Diversität über minimale Jaccard-Distanz zu gewählten Pfaden; Score `Rel·w_rel + Div·w_div`. Tie-Break deterministisch `(total_cmp(score), node_id)`.
- `find_path` (Einzelpfad, Dijkstra) wird über Feature `graph` exponiert; `find_k_paths` bleibt `pub(crate)` bis ein Benchmark-Gate in EP3 bestanden ist.
- `AGT-GRAPH-001` wird als **Laufzeitprüfung** (Fehler statt `debug_assert!`) erzwungen.
- Kaskaden-Invalidierung (`child_edge_ids`) mit Zyklenschutz.

### 9.7 Sketched Contextual Bandit (EP5, `contextra-adapt`)
- Sparse-JL-Projektion (Achlioptas): `S ∈ R^{k×d}`, Einträge `√(3/k)·{+1 (p=1/6), 0 (p=2/3), −1 (p=1/6)}`, erzeugt aus einem persistierten Seed über den `Rng`-Port. `z = S·x`. Sherman-Morrison auf `k×k`: `O(k²)` statt `O(d²)`.
- `k` ist ein Konfigurationswert (Startwert 32, Bereich 16–128). Eine Formel `k = O(√d·log d)` wird **nicht** behauptet. Für naive Zufallsprojektion wird **keine Regret-Garantie** zugesagt; die Qualität wird per Benchmark gegen volles LinUCB auf synthetischen Daten belegt.
- **Keine Regret-Zusage (E48):** Die Schranke `O(k√T log T)` und die Dimensionsformel aus ADVANCED §4.4 werden nicht übernommen. Die Qualität wird per Benchmark gegen volles LinUCB belegt.
- **Re-Sketching bei Drift:** Die Kontext-Historie wird nicht gehalten, daher ist ein „Re-Projizieren" der Kovarianz nicht möglich. Stattdessen: neuer Seed, **Reset** (`A = λI`, `b = 0`). Count-Sketch ist verworfen.

### 9.8 Spektrale Hyperkanten-Deduplizierung (EP5, `contextra-cognition`)
- Inzidenzmatrix `H ∈ {0,1}^{|V|×|E|}`, SVD-Rang `k ≤ k_max` (Startwert 64). Brand-Update einer neuen Spalte `c ∈ R^{|V|}`: `p = U_kᵀc`, `r = c − U_k p`, `q = r/‖r‖` (falls `‖r‖ > ε`), `K = [[Σ_k, p],[0, ‖r‖]]`, SVD von `K` → `U'`, `Σ'`, `V'`; `U_{k+1} = [U_k, q]·U'`. **Danach Truncation** auf `k_max` (kleinste Singulärwerte verwerfen); ohne sie wüchse `k` mit jedem Update.
- **Einbettung** einer Hyperkante: `z_e = U_kᵀ c_e ∈ R^k` (nicht `V_kᵀc`: Dimensionsfehler). Duplikat bei Cosinus-Ähnlichkeit `> θ` (Startwert 0,92) **innerhalb desselben Mandanten**.
- **Provenienz:** Beim Zusammenführen bleibt die Menge der Quell-Dokumente erhalten; Gewichte werden je Quelle geführt. Eine Löschung entfernt die Quelle, Kanten ohne Quelle werden entfernt, die Faktoren werden als `dirty` markiert und bei der Konsolidierung aus `H` neu berechnet (kein Downdate möglich).

### 9.9 Serialisierbarkeit
SSI (`SequenceLogSsiValidator`) ist Standard. Ein Serial Safety Net wird in EP1 nicht implementiert; die Entscheidung wird nach einem Benchmark über die Abbruchrate bei ≥ 8 Schreib-Threads getroffen (Startschwelle 5 %).

### 9.9a Extended Serial Safety Net (ESSN): Kandidat, kein Default

Status: **Kandidat** hinter dem Gate aus §9.9. Quelle: Recherche Feld F (Preprint). Die Herkunft der Bezeichnung „SSN (Wang et al. 2016)" in ARCH_PLAN §19.2 ist nicht geprüft.

- **Form:** Commit-Zertifizierung über Zeitstempel je Version (`sstamp`, `pstamp`) mit Abbruch bei Erfüllen der Ausschlussbedingung. Die exakte Bedingung (π, η) ist aus der Originalliteratur zu übernehmen und im Konfliktgraph-Test zu belegen. Die Gleichsetzung „π ≤ η ⇔ sstamp ≤ pstamp" aus ARCH_PLAN §19.2 ist ungeprüft.
- **Voraussetzungen:** (a) serieller Commit-Sequencer unter Mutex mit monotoner `tx_seq`. Latch-freie Varianten sind verboten (P28). (b) Versionsmetadaten je Version, persistiert in Store und MVCC. Das ist eine Formatänderung nach REG-17. (c) Korrektheitstests: der Write-Skew-Fall und ein Read-Only-Anomaliefall müssen abgebrochen werden. Zyklenfreiheit ist über den Konfliktgraph zu prüfen.
- **Gate (O-11):** Abbruchrate und Commit-Overhead bei ≥ 8 Schreib-Threads auf Zielhardware mit 4 Kernen. Schwelle 5 %. Die Angabe „−50 % Abbrüche" aus der Recherche ist ungemessen und zählt nicht.
- Bis zum Gate-Ergebnis gilt Standard-SSI.

### 9.9b Latenz-Drift-Erkennung (Catoni): Kandidat für EP5

- **Zweck:** Drift-Erkennung auf Latenz- und Reward-Reihen mit Heavy-Tail-Ausreißern. CUSUM ist dafür ungeeignet (§9.10).
- **Einflussfunktion:** `ψ(x) = sign(x)·log(1 + |x| + x²/2)`. Die in der Recherche angegebene Form `x/(1+|x|)` ist keine Catoni-Funktion und wird nicht übernommen.
- **Ablauf:** Mittelwertschätzer per Bisektion mit hartem Iterationslimit, damit Pfade deterministisch bleiben. Nichtkonvergenz führt zu einem Fallback mit Health-Eintrag, nicht zu einem Alarm.
- **Gate:** Falsch-Alarm-Rate und Erkennungsverzug gegen CUSUM auf synthetischen Heavy-Tail-Daten mit GC-Pausen-Modell. Ohne bestandenes Gate kein Default.
- **Ring:** `contextra-adapt`, Ring 0 nach §4.1.

### 9.9c Spektral-Deduplizierung: Wächter und Provenienz

- Ergänzend zu §9.8: Singulärwerte unterhalb `ε_svd` werden abgeschnitten. Eingabevektoren werden mit `is_finite()` geprüft, sonst `InvalidInput` (§7.6).
- Rangobergrenze `k_max` mit Truncation nach jedem Brand-Update. Eine vollständige Neukonstruktion aus der Inzidenzmatrix ist nur als Konsolidierungsschritt zulässig.
- **Provenienzpflicht:** Varianten mit Oblivious-Vertex-Sampling oder Effective-Resistance-Sparsifier verlieren die Zuordnung zu Quelldokumenten. Sie sind deshalb für dieses Feature verboten (§9.10).

### 9.10 Verworfen
BMSSP/FindPivots (Parameter degenerieren bei kleinen und mittleren Graphen), HyperQuant (für LLM-Gewichte entworfen), Mixed-Precision-Fließkomma (verletzt ISA-Reproduzierbarkeit), ML-basierte Compaction (CPU-Varianz), Count-Sketch für Bandits, CBOR für Beweise, SHA-256 als Beweis-Digest, `unsafe` in `kvcache`, jede Form von Replikation oder Clustering. Zusätzlich verworfen (Recherche-Ablehnungstabelle, bestätigt): Motif-Clustering O(k·deg³), Inference-free Sparse Retriever, CUSUM als Drift-Detektor, Read Promotion, GraphPlanner (Multi-Agent), H-Mem (Multi-Tenant-Cloud), Kahan-Summation als Politik (E43), Replica-Layer (E40), CBOR-Export (E41), Oblivious-Sparsifier ohne Provenienz (§9.9c).

---

## 10. Verdrahtungs-Doktrin und Behandlung unverdrahteter Funktionen

### 10.1 Regel
Aus dem Befund „512 öffentliche Funktionen ohne Aufrufer, 1.057 undokumentierte Schnittstellen" folgt für **jedes** Symbol genau eine der drei Entscheidungen: (a) im Composition Root (`contextra::builder` oder `contextra-mcp`) produktiv verdrahten und per Fassadentest belegen; (b) auf `pub(crate)`/`pub(in …)` reduzieren; (c) ersatzlos löschen. Ein Symbol, das „später gebraucht wird", bekommt keine Sonderstellung. Die Liste wird per `cargo xtask find-unwired-gaps` erzeugt und als Ratsche geführt (Gate G-14: Zahl darf nur sinken, Ziel 0 in EP1).
Öffentliche Symbole erhalten vollständige `///`-Dokumentation mit Fehler-, Panik- und Sicherheitsvertrag (Lint `missing_docs` als Fehler in Ring 0 und Ring 1 ab M6).

### 10.2 Konkrete Entscheidungen

| Symbol/Befund | Entscheidung |
|---|---|
| `DpiPurgeDriver`, `DpiPurgeDriver::new/run_physical` | In den Purge-v2-Orchestrator im Store integrieren, `pub(crate)` |
| `reverse_sweep` | Als Verifikations-Rohsweep in den Gate-/Testpfad integrieren, falls es Rückverweise auf gelöschte Dokumente sucht (Re-V.); sonst löschen |
| `DpiDeletionRuntime::open` | Nach EP2 in `erasure-evidence` |
| `forget_document` (Konstantenzeugen), `DeletionProof::create` (v3), `deletion_proof_active` | Löschen (M0.2, M1.4, M3.3) |
| `ForgetOptions { assurance: S1 }` | Aus dem Kern entfernen; Assurance ist Plugin-Konfiguration, der Parameter `_opts` in `erase_document` entfällt |
| `find_k_paths` | `pub(crate)` bis Benchmark-Gate EP3 |
| `KmsProvider`/`LocalKmsProvider` | Im Builder als Standard-KMS-Port per `with_kms` verdrahten |
| Vier Reaper | `EngineLifecycle` (§7.7) |
| `recover_pending_intents` | in `repair_on_open()` |
| `apply_temporal_validity_filter_at` | Signatur trennen: `as_of_tx: TxId` (Systemzeit) und `as_of_business: Option<u64>` (Geschäftszeit); kein Cast Unix-Zeit → `TxId` |
| `PendingDeleteJournal::process` | `self.flush()?`, Fehler propagieren (INV-FSYNC-1) |
| `kv_shredding.rs:247` (TODO) | Atomare Bitmaske (§5.13) |
| `AgentContext.status` (`pub`) | Privates Feld, Übergänge über typisierte Methoden der Engine (EP5) |
| `sandbox`-Bindung in `engine`/`store` | Port `MergeOperatorFactory` (§4.3) |
| Legacy-WAL | Entfernen |
| Builder-Schalter (7) | §4.7 |

---

## 11. Qualitätssicherung

### 11.1 Gates
Alle Gates laufen als `cargo xtask <name>` und sind in `xtask/registry.toml` registriert. Ein Gate ohne Registry-Eintrag existiert nicht.

| Gate | Name | Prüfung | Ab |
|---|---|---|---|
| G-01 | `check-build-baseline` | `cargo check --locked` je Paket und je Eintrag der Feature-Matrix | M0 |
| G-02 | `deletion-guarantee-gate` | Rohsweep nach Purge, Marker-/Claim-Prüfung, Differenztest | M0 (Stub), M2 |
| G-03 | `check-product-hull --product <ID>` | Hülle ≤ Manifest, verbotene Crates fehlen | M3 |
| G-04 | `sync-docs --check` | `capabilities.toml` stimmt mit §4.1 überein | M0 |
| G-05 | `check-builder-wiring` | Kein `let _ =`/`_`-Feld im Builder, Wirkungstest je Schalter | M3 |
| G-06 | `check-feature-isolation` | Jedes Feature kompiliert isoliert (Format-Features eigene Achse) | M3 |
| G-07 | `check-ring-layering --strict` | Ring-DAG und Ordnung §4.2 (reguläre Kanten) | M4 |
| G-08 | `check-snapshot-consistency` | Multi-Signal-Reads nutzen `ReadSnapshot` | M4 |
| G-09 | `check-determinism` | `clippy.toml`-Verbote und Lints | M1 (warn), M4 (deny) |
| G-10 | `check-lock-order` | Lock-Reihenfolge (§7.1) + Loom | M4 |
| G-11 | `check-no-downcast` | 0 Treffer `downcast_ref::<LsmStorage>` | M2 |
| G-12 | `check-port-features` | `backend-impl` nur von `store`; `test-utils` nur dev | M1 |
| G-13 | `check-unsafe-islands` | `unsafe` nur in `sys/simd/wire`, `SAFETY`-Kommentare, Ausnahme generierter Code | M4 |
| G-14 | `find-unwired-gaps --ratchet` | Zahl unverdrahteter `pub fn` sinkt | M3 |
| G-15 | `unwrap-ratchet` | `governance/ratchet.toml:unwrap_exact` (IST 502) sinkt; `.github/unwrap_baseline.txt` wird daraus generiert | laufend |
| G-16 | `upgrade-test` | Golden-Store-Fixtures öffnen und upgraden | M4 |
| G-17 | `bench-gate` | §11.3 | M4 |
| G-18 | `crash-matrix` | CRASH-01..12 | M5 |
| G-19 | `check-claims` | Verbotene Formulierungen (§5.14) in Doku, Fehlertexten, MCP-Antworten | M0 |
| G-20 | `check-mcp-classification` | Jedes Tool klassifiziert und gated | M3 |
| G-21 | `check-exception-ledger` | Ausnahmen in `capabilities.toml` (§4.3a) haben Ablauf-Meilenstein; abgelaufen = rot; Anzahl sinkt monoton | M0 |
| G-22 | `check-feature-tree` | `backend-impl` nur im Feature-Baum von `contextra-store` (`cargo tree -e features -i contextra-ports`), je Produkt-Manifest | M1 |
| G-23 | `check-port-names` | Namenstabelle §7.4 gegen `pub`-Traits in `contextra-ports` | M1 |
| G-24 | `check-store-lock` | Zweiter `open()` auf demselben Verzeichnis scheitert mit `StoreLocked` (Zwei-Prozess-Test) | M4 |
| G-25 | `check-platform-tiers` | CI-Jobs je Tier; `Assured`-Claims nur auf Tier 1/2 mit grünem Crash-Test | M6 |
| G-26 | `check-api-semver` | `cargo-semver-checks` auf `contextra-ports`; vor 1.0 Warnung, ab 1.0 blockierend | M1 (warn), 1.0 (deny) |
| G-27 | `check-tenant-resolution` | 0 Treffer `unwrap_or_default` in der Mandantenauflösung; `tenant_id()` liefert nur `Result` | M2 |
| G-28 | `check-clock-anchor` | Test: Wanduhr zurück → `ClockRegression`; Quarantäne in Ticks | M5 |
| G-29 | `check-log-redaction` | Doc-IDs in Log- und Tracing-Ausgaben nur als keyed hash | M6 |
| G-30 | `check-no-stubs` | Keine Erfolgsrückgabe ohne Implementierung in Produktion (REG-18); Erkennung über Liste bekannter Stubs und `todo!`-artiger Muster | M1 (warn), M6 (deny) |
| G-31 | `check-noop-hooks` | No-op-Hooks nur bei `ErasureMode::Off`; Produktions-Dispose ist `RequireScrub` | M4 |
| G-32 | `check-canary-coverage` | Canary-Marker in Memtable, SSTable, WAL, HNSW und Textindex, auch komprimiert und verschlüsselt | M5 |
| G-33 | `check-doc-links` | Keine lokalen absoluten Pfade (`file:///`), alle internen Verweise existieren | M0 | <!-- doc-ref-ignore -->

**Status v1.3:** G-11 (`check-no-downcast`) und G-12 (`check-port-features`) sind spezifiziert, aber am HEAD `90b23431` nicht in `xtask` implementiert (F-40). Ihr Status ist in `xtask/registry.toml` zu prüfen. Jeder Gate-Status wird in N0.3 gemessen.

Diff-Budget je PR: Standard 15 Dateien, 600 Zeilen, 2 Crates (vorhandenes Verfahren); größere Vorhaben (Core-Cut, Crate-Split) nutzen das Kartenbudget mit Reviewer-Prüfsumme.

### 11.2 Testarten (Pflicht je Bereich)
- **Differenztest** gegen BTreeMap-Referenz (Purge, Compaction).
- **Rohsweep** der Store-Dateien (§5.7).
- **Tenant-Matrix** (REG-15).
- **Loom** für Fence (3 Szenarien) und Lock-Ordnung.
- **Crash-Matrix** (§5.12) mit Failpoints.
- **Property-Tests** (proptest) für Scope-Invarianten, RRF/DiBud, Quantisierung.
- **Fuzzing** für Parser (Ledger, Manifest, TLV, Wire) und Verifier.
- **Mutationstests** für Löschgarantien (`mutants-guarantees.toml`): jede Mutation an Purge, Fence, Ledger, RestoreGuard muss einen Test brechen.
- **Tampering-Matrix** (EP2, §6.5).
- **E2E über die Fassade** je Modus (REG-06).

### 11.3 Benchmark-Politik
- Ein Gate auf „≤ 1 % p99" ist auf CI-Hardware nicht messbar und gilt nicht. Stattdessen: (1) **Instruktionszähler** (z. B. `iai-callgrind`) als deterministisches Gate mit Toleranz (Startwert 2 %), (2) Wall-Clock-Messung mit Criterion nur auf einem dedizierten Runner und mit fester Baseline, Toleranz 5 %, (3) Budgets aus §4.5 sind Startwerte und werden in M4 kalibriert.
- Kommando: `cargo xtask bench-gate --tolerance 0.05` mit festgehaltener Criterion-Baseline im Repo.

---

## 12. Roadmap und Arbeitspakete

### 12.1 Lesehinweise
- Meilensteine laufen **sequenziell**; innerhalb eines Meilensteins sind APs parallel, soweit „Abh." nichts anderes sagt.
- Aufwand: S ≤ 1 Tag, M 2–3 Tage, L ≈ 1 Woche, XL > 1 Woche (Eigenschätzung, ±25 %, ohne Puffer; EP1 gesamt ≈ 18–20 Wochen nominal).
- **Baseline-Konflikt (v1.1):** Die Anhänge verwenden die Commits `500e9246` und `16764583`, diese Spezifikation `f00bb2a`. Befunde der Anhänge sind Hypothesen bis M0.3. Die Roadmap der IMPLEMENTIERUNGSSPEZIFIKATION (Phasen 0 bis 6) wird nicht übernommen. Maßgeblich bleiben M0 bis M11.
- Jeder AP nennt: **Was**, **Warum** (Begründung/Befund), **Dateien**, **Akzeptanz**. Ein AP ohne Akzeptanz ist nicht fertig.
- Reihenfolgeprinzip: erst Wahrheit herstellen (M0), dann billige Vertragsänderungen solange es keine Implementierungen gibt (M1), dann Korrektheit (M2), dann Struktur (M3), dann Nebenläufigkeit (M4), dann die vollständige Garantie (M5), dann Release (M6).

### M0 — Governance, Baseline, Hotfix (≈ 1 Woche)

| AP | Aufgabe | Warum | Dateien | Akzeptanz | Aufw. |
|---|---|---|---|---|---|
| M0.1 | Planungsdokumente konsolidieren: diese Spezifikation als maßgebliche Quelle ablegen; alle früheren Pläne in `docs/archive/` mit Kopfzeile „superseded by"; byte-identische Duplikate löschen; je EP genau eine lebende Spezifikation; absolute `file:///`-Links durch relative ersetzen | Parallele, widersprüchliche Wahrheiten (F-18, F-23) | `docs/14-10-26/**`, `docs/archive/` | Genau eine Quelle je Thema; `grep file:///` = 0 | S | <!-- doc-ref-ignore -->
| M0.2 | **Hotfix F-01:** Fabrikation von Zeugen in `erase_document` entfernen (Ledger-Witness mit `entry_confirmed:true`/MAC-Nullen, `None`-Zeugen, `if let Ok`-Verschlucken). Anfragen mit Assurance > S0 liefern bis M5 ehrlich `CapabilityUnsupported`. MCP meldet `logically_deleted` ohne Beweisbehauptung | Das Kernversprechen wird sonst aktiv verletzt; schnellste Risikoreduktion | `contextra-engine/src/collection/crud/delete.rs`, `contextra-mcp/src/server_tools.rs` | Test: kein Zeuge ohne Scan; Test: Assurance>S0 → `CapabilityUnsupported`; G-19 grün | S |
| M0.3 | Baseline messen (Re-V.): `cargo check --locked` je Paket, Hülle (Default und `--no-default-features`), `unsafe`-Zählung, `Instant::now`/`SystemTime::now`/`thread_rng`/FMA/`partial_cmp`-Zählung, **Ring-Kanten getrennt nach regulär/dev**, `find-unwired-gaps`; F-06-Einzelbefunde (L4, L5, L6, L8, N-K, N-E, N-F, N-G, N-J) am HEAD einzeln bestätigen oder als behoben markieren. Ergebnis in `docs/baseline/2026-10.md` | Frühere Befunde sind teils veraltet (F-29); Behebung nur auf gemessener Basis (R-A4) | `docs/baseline/`, `xtask` | Datei mit Messwerten und Befundstatus; G-01 läuft | M |
| M0.4 | `capabilities.toml` an §4.1 angleichen (Bench-/Fuzz-Crates als Tooling, geplante neue Crates), „Layer 0–10" → „Topo-Ebene", G-04 aktivieren | Ein Ringmodell (F-18) | `capabilities.toml`, `docs/SOURCE_OF_TRUTH.md`, `xtask` | `sync-docs --check` grün | S |
| M0.5 | Claims-Bereinigung nach §5.14; G-19 anlegen | Unzulässige Rechts-/Normaussagen (F-28) | Doku, Fehlertexte, MCP-Antworten | G-19 grün | S |
| M0.6 | ADR-Dateien aus §13 in `docs/decisions/` anlegen (Präfix `ADR-E`, weil Nummern belegt) | Entscheidungen müssen im Repo nachvollziehbar sein | `docs/decisions/` | Je ADR eine Datei | S |
| M0.7 | `deletion-guarantee-gate` (G-02) in `xtask/registry.toml` registrieren (zunächst mit Rohsweep-Stub) | Das Gate fehlt (F-23) | `xtask/registry.toml`, `xtask/src` | Gate läuft in CI | M |
| M0.8 | **Baseline-Pin und Zählregeln (v1.2):** Commit-Hash in `docs/baseline/` festschreiben (F-37); die drei Unwrap-Zähler der Ratsche durch eine parserbasierte Regel ersetzen und dokumentieren (F-34); `Instant::now`/`thread_rng`/`SystemTime::now`-Zählregel ebenfalls parserbasiert (Gap: 6 vs. 12 `thread_rng` außerhalb Tests je nach Methode) | Messungen ohne reproduzierbare Regel sind nicht gate-fähig (R-A4) | `governance/ratchet.toml`, `xtask`, `docs/baseline/` | Gate meldet alle Zähler mit Pin; zwei aufeinanderfolgende Läufe identisch | M |
| M0.9 | **Registrierung der Lücken (v1.2):** `xtask` und `contextra-bench` in `capabilities.toml` (F-35); `FreeSpaceProbe`-Duplikat dokumentieren (F-32) und Entfernung als M1.2-Vorbedingung vermerken | Vollständigkeit des Ringmodells (REG-02) | `capabilities.toml`, `docs/baseline/` | G-04 grün mit vollständiger Crate-Liste | S |

### M1 — Vertragstypen und billige Korrekturen (≈ 1 Woche)

Begründung: Für `ErasureBackend`, `LayerScan`, `WriteGate`, `RestoreGuard`, `ErasureHooks` gibt es **keine Implementierung**. Typänderungen sind jetzt billig und später Breaking Changes.

| AP | Aufgabe | Dateien | Akzeptanz | Aufw. |
|---|---|---|---|---|
| M1.1 | `PhysicalScope` opak mit Invarianten (§5.4), `LogicalScope` ohne `Keys/Prefixes`, jede Variante mit `TenantId`; Port-Feature `backend-impl` | `contextra-ports/src/erasure.rs` | Property-Tests der Invarianten; leerer Präfix und fremdes Mandantenpräfix werden abgelehnt; G-12 | M |
| M1.2 | Port-Neudefinition nach §5.5: lineares `HookToken`, `WriteGate::is_active/check_batch`, `FreeSpaceProbe`, `LayerScan` mit `u16`, `LayerObservation` mit `state_digest` und `item_count`, `DeterministicTimestamp`; `test-utils` nur in dev-Abhängigkeiten | `contextra-ports` | `cargo test -p contextra-ports`; Compile-Fail-Tests (trybuild): doppeltes `finish` unmöglich | M |
| M1.3 | `DeleteOutcome`, `EraseError`, `PurgeState` (`#[repr(u8)]`) samt Golden-Test der Byte-Kodierung | `contextra-ports`, `lsm/purge/intent.rs` | Golden-Test | S |
| M1.4 | v3-Beweiserzeugung löschen (inkl. `construct_v4_payload`); v2/v3 verify-only; Legacy-Kennzeichnung | `contextra-crypto/src/deletion_proof/proof_v3.rs` | Test: keine v3-Erzeugung möglich; Bestandsbeweise lesbar oder klar als ungültig markiert | S |
| M1.5 | `KvDeleteMode::default()` → `TombstoneOnly`; Builder setzt `CryptoShred` bei `encryption-at-rest`; Test und Invariantendoku anpassen | `contextra-store/src/kv/delete_mode.rs`, `builder.rs` | Test: Default = kein CryptoShred; mit Feature = CryptoShred | S |
| M1.6 | `Clock`/`Rng`/`IdGenerator`-Ports vervollständigen (`EntropyRng` vs. `SeededRng`), Workspace-`clippy.toml` im Warnmodus (G-09 warn) | `contextra-ports`, `contextra-sys`, `clippy.toml` | Neue Verstöße werden gemeldet | M |

### M2 — Purge v2 und ehrlicher Löschpfad (≈ 3 Wochen)

| AP | Aufgabe | Warum | Dateien | Akzeptanz | Abh. | Aufw. |
|---|---|---|---|---|---|---|
| M2.1 | Purge v2 nach §5.7: Streaming, 3-Arm-Match, Rang/Seq-Erhalt, Scrub, Orphan-GC mit Scrub, Zufalls-Intent-IDs, Probe-Semantik (`None` ≠ 0), Failpoints nur per Feature | F-06 | `lsm/ops/purge.rs`, `lsm/purge/*`, `durable-fs` | Differenztest grün; Rohsweep leer; CRASH-02..06 | M1 | L |
| M2.2 | Test-Harness: BTreeMap-Referenz, Rohsweep, Tenant-Matrix, Chaos-Failpoints | REG-15, §11.2 | `store/tests`, `contextra-testkit` | Harness nutzbar für alle folgenden APs | — | M |
| M2.3 | `ErasureBackend` im Store implementieren; `StorageErasureExt`; `TenantScopedStorage` delegiert; alle 5 Downcasts entfernen; `tenant_id()` fail-closed; `remove_doc_segments` mit Pflicht-`TenantId` | F-05 | `store`, `engine/src/collection/**`, `kv_hooks` | G-11 = 0; Tenant-Matrix grün | M1.1/M1.2 | L |
| M2.4 | Resume-Fix: Storage zuerst aufbauen, Scope aus Intent (Ledger-Scope folgt in M5.2), Fence **vor** der Rückkehr von `open()`, Rewrite als Hintergrund-Task | Fail-open-Resume (N-A) | `lsm/purge/resume.rs`, `lsm/recovery.rs` | CRASH-01..04 mit Resume | M2.1 | M |
| M2.5 | Löschpfad im Engine: `Off` = reiner Tombstone ohne Hooks/Scans; `Assured` über den vorhandenen Store-Ledger (`ledger_append` + `fsync` **vor** Tombstone), `ErasureHooks`-Aufrufe | F-01 Folgearbeit | `engine/src/collection/crud/delete.rs` | E2E-Test über Fassade: Off und Assured; Zähler-Mock zeigt 0 Overhead bei Off | M2.3 | M |
| M2.6 | Commit-Punkt-Semantik: `EraseError::{NotStarted, Indeterminate}`, Idempotenz, `ErasureSupervisor`, kein `Err` nach Commit (Indexfehler → `PurgePending`) | F-02 | `engine`, `EngineLifecycle` | CRASH-10; Test: `Err` nach Commit nur als `Indeterminate` | M2.5 | M |
| M2.7 | `PendingDeleteJournal::process` propagiert `flush()?`; Orphan-GC über Scrub | F-22 (Teil) | `contextra-durable-fs` | Fehlerinjektionstest | — | S |

### M3 — Core-Cut, Features, Builder, MCP (≈ 3 Wochen)

| AP | Aufgabe | Warum | Dateien | Akzeptanz | Aufw. |
|---|---|---|---|---|---|
| M3.1 | `contextra-core` entfernen (§4.9), Skript-PRs je Crate-Gruppe | F-17 | 404 Dateien, 11 `Cargo.toml` | `cargo check --workspace` ohne Crate | L |
| M3.2 | Engine: `default = []`; `graph`, `adapt`, `kvcache`, `candle` hinter `cfg`; `rank` Pflicht | F-14 | `contextra-engine` | `cargo check -p contextra-engine --no-default-features` grün; G-06 | M |
| M3.3 | Fassade: `default = ["fast"]`, Features nach §4.5, `LicenseGate`-Port statt `license`-Abhängigkeit in der Engine, `deletion_proof_active` und Feature `deletion-proof` entfernen | F-14, F-31 | `contextra`, `contextra-engine`, `ports` | Hülle ≤ 18; 0 `cfg(feature="deletion-proof")` | M |
| M3.4 | Ring-Abhängigkeiten für die Hülle: `store→sandbox` über `MergeOperatorFactory`; `engine→sandbox/license` optional; `graph→store` über `GraphPersistence`; `db→cognition` entfernen | F-31 | `ports`, `store`, `engine`, `graph`, `db` | G-03 grün | L |
| M3.5 | Builder-Ehrlichkeit (§4.7): echtes `verify_wiring`, 7 Schalter entscheiden, `with_erasure_mode`, `with_erasure_hooks`, `with_license_gate` | F-15 | `contextra/src/builder.rs`, `wiring_selftest.rs` | G-05; Wirkungstest je Schalter | M |
| M3.6 | MCP-Härtung (§8): Mandantenbindung beim Start (kein `SYSTEM`), `spawn_blocking`, `test-utils` nach dev, Features `fast/guard/graph`, Abhängigkeiten bereinigen, Tool-Klassifikation | F-13 | `contextra-mcp` | G-20; Test: Start ohne Mandant scheitert; Tool-Argument `tenant` abgelehnt | L |
| M3.7 | Einzelbefunde: `EngineLifecycle` startet die Reaper; `ENG-FILTER-001`-Signatur; Legacy-WAL entfernen | F-21, F-22 | `engine`, `store` | Reaper-Test über Fassade; Filtertest trennt System-/Geschäftszeit | M |
| M3.8 | Produkt-Manifest `products/ep1.toml` und G-03/G-06/G-14 aktivieren | §4.4 | `products/`, `xtask` | Gates grün | S |

### M4 — Fence, Locks, Ringe, Determinismus (≈ 3 Wochen)

| AP | Aufgabe | Warum | Dateien | Akzeptanz | Aufw. |
|---|---|---|---|---|---|
| M4.1 | Fence-Protokoll B (§5.8): `ErasureGate` mit `AtomicBool`, `ArcSwap`, `inflight`-Drain; Integration in Group-Commit-Leader (Fenster 1/2); `arc-swap` in `deny.toml` und `supply-chain/` | F-11 | `lsm/fence.rs`, `lsm/group_commit.rs`, `lsm/ops/write.rs` | **Loom (3 Szenarien) grün — Merge-Blocker**; CRASH-07 | L |
| M4.2 | Bench-Infrastruktur (§11.3): `iai-callgrind`, Baseline, `bench-gate` | F-25 | `benches`, `xtask` | Baseline im Repo; Gate läuft | M |
| M4.3 | Lock-Reihenfolge (§7.1) dokumentieren und G-10 | F-12 | `docs`, `xtask`, Loom-Tests | G-10 grün | M |
| M4.4 | Formatversionierung: `ManifestFormatTracker` erweitern, `IdWidthMismatch`, Manifest-Einträge, Upgrade-Test mit Golden-Fixtures (G-16) | F-24, REG-17 | `store/src/manifest`, `tests/fixtures` | Upgrade-Test grün | L |
| M4.5 | Ring-Gate strikt (G-07), Ordnung §4.2, restliche Verletzungen auflösen | F-18, F-31 | `xtask`, `capabilities.toml` | G-07 grün | M |
| M4.6 | Unsafe-Inseln: `wire` generierter Code mit Datei-Header und Generator-Test; `AlignedBuf` sicher in `sys`; G-13 | F-19 | `wire`, `sys` | G-13 grün | M |
| M4.7 | Determinismus: alle `Instant::now`/`SystemTime::now`/`thread_rng` über Ports; G-09 auf `deny`; Lints in Ring 0/1 | F-20 | workspace | G-09 grün | L |
| M4.8 | `ReadSnapshot`/`SnapshotReader` als Pflichtparameter der Hybridsuche; SSI-Read-Set-Tracking; `register`/`register_at` entfernen | REG-10, §7.3 | `ports`, `store`, `vector`, `text`, `mvcc`, `engine` | G-08 grün | L |
| M4.9 | Fließkomma-Politik (§7.6): NaN/Inf-Ablehnung am Eingang, kanonischer Kernel beim HNSW-Aufbau, dokumentierte Reduktionsordnung, SIMD-vs-Scalar-Test mit ULP-Grenze | F-20 | `simd`, `vector`, `rank` | Tests grün; keine `partial_cmp` in `rank`/`vector` | M |

### M5 — Assured vollständig (≈ 4 Wochen)

| AP | Aufgabe | Warum | Dateien | Akzeptanz | Aufw. |
|---|---|---|---|---|---|
| M5.1 | Integritätsschlüssel, MAC im Manifest, Hochwassermarke, `AcknowledgeToken`, `ModeChange` (§5.9) | F-10 | `store/src/manifest`, `engine`, Admin-Kommando | Downgrade ohne Token → `PolicyViolation`; Schlüssel fehlt → `open` scheitert | L |
| M5.2 | Ledger v1 (§5.9, E65): IST-Format mit Datei-Header `CTRALEDG`, Migration vom IST-Format (Re-V. des Altformats), Digest-Index, Truncation-Erkennung (keine Kompaktierung) | F-27 | `store/src/erasure_ledger` | Fixture-Migration; CRASH-11 | L |
| M5.3 | Hook-Implementierung und `PurgeState`-Zustandsmaschine in Store/Engine (§5.6), Scope aus Ledger im Resume | F-02 | `store`, `engine` | CRASH-01..04, 09 | L |
| M5.4 | `StreamingRestoreGuard`; Checkpoint-Restore-Integration; Checkpoint-Scrub bei Collection/Tenant-Löschung | CRASH-08 | `store`, `checkpoint` | CRASH-08 grün | M |
| M5.5 | Crate-Split §4.8: `crypto-core`, `crypto-kv`, `erasure-evidence` (Quellcode verschoben, Kompilierung bleibt Ziel), Governance-Anpassungen in einem PR je Zielcrate | F-30 | `crates/contextra-crypto`, `.github`, `xtask`, `rules`, `capabilities.toml`, `mutants-guarantees.toml`, `justfile`, Fuzz | `cargo tree` des EP1-Binaries ohne `deletion_proof`; Hülle ≤ 17 | XL |
| M5.6 | Crypto-Erase zweistufig, `per_document_dek` (Option), `KeyQuarantined`, Revocation-Bitmaske | F-09 | `crypto-kv`, `store` | Tests Stufe 1/2; Claim-Texte nach §5.14 | L |
| M5.7 | Abgeleitete Daten: HNSW (Slot-Nullung, Segment-Rewrite für mmap), `quantize/rabitq/diskann`, Text-Index, Embedding-/Query-Cache; ein Scan-Test (Rohsweep) je Speicher | F-07 | `vector`, `text`, `engine` | Rohsweep über **alle** Dateien des Store-Verzeichnisses leer | XL |
| M5.8 | WAL-Scrub mit Rotation, Manifest-Historie-Scrub, Koaleszenz-Fenster | F-07 | `store/src/wal`, `manifest` | Rohsweep inkl. WAL leer | L |
| M5.9 | Snapshot-Pin-Policy (§5.10), `sys::harden_process()` | F-07 | `mvcc`, `store`, `sys`, `mcp` | CRASH-12 | M |
| M5.10 | `DurabilityMode × ErasureMode` im Builder und beim Open durchsetzen (§4.6) | REG-09 | `contextra/builder.rs`, `store` | 9 Matrixfälle als Tests | S |

### M6 — Release-Härtung v0.1.0 (≈ 2–3 Wochen)

| AP | Aufgabe | Akzeptanz |
|---|---|---|
| M6.1 | E2E-Suiten über Fassade und MCP für `Off` und `Assured`, jeweils mit Tenant-Matrix | Beide Suiten grün |
| M6.2 | Crash-Matrix CRASH-01..12 vollständig (G-18) | Alle grün |
| M6.3 | Upgrade-Test mit Altstore (Manifest, `purge/`-Intents, Ledger) | G-16 grün |
| M6.4 | Verdrahtungs-Triage der 512 `pub fn` und Dokumentation der verbleibenden öffentlichen API; `missing_docs` als Fehler in Ring 0/1 | G-14 = Ziel; Lint grün |
| M6.5 | Benchmarks gegen Budgets (§4.5), Kalibrierung der Startwerte, Ergebnis in `docs/BENCHMARKS.md` | G-17 grün |
| M6.6 | Pseudonymisierte Logs (keyed hash für Doc-IDs), Claims-Review, Betriebsdokumentation (Schlüsselverlust, Medien-Remanenz) | G-19 grün |
| M6.7 | Mutationstests für Löschgarantien | Alle Mutanten getötet |
| M6.8 | Release-Checkliste, Tag **v0.1.0** | Alle Gates grün |

### M7 — EP2 Contextra Proof, Modus `Attested` (≈ 5–6 Wochen)

| AP | Aufgabe | Akzeptanz |
|---|---|---|
| M7.0 | **Voraussetzung:** Entscheidung zu Signierschlüssel-Ablage, Rotation, Verlust, Key-Registry und Trust-Anker (offener Punkt O-1) | ADR beschlossen |
| M7.1 | `erasure-evidence` ausbauen: Beweis v4 (§6.2), generierte Tag-Tabelle, Golden-Vektoren, `verify_strict`, kein `bincode` im Signaturpfad | Golden-Tests |
| M7.2 | `contextra-proof-verify` (nur `ed25519-dalek`, `blake3`), Trust-Anker-Prüfung, Legacy-Kennzeichnung | Verifier ohne Contextra-Abhängigkeiten baubar |
| M7.3 | Layer-Scans (echt, fail-closed, mit `item_count`): 1, 2, 3, 4, 5, 20, 21, 22, 23, 24, 25, 26; `REQUIRED_LAYERS` | Mutationstest: No-Op-Scanner → kein Beweis |
| M7.4 | `ErasureHooks`-Attested-Implementierung, `stage`/`supersedes`-Auswertung im Verifier, Lizenzgate (§5.15) | E2E `Attested` |
| M7.5 | Tampering-Matrix, Testvektoren, Fuzzing des Verifiers | Alle Mutationen abgelehnt |
| M7.6 | `audit-export`, `avv-generator` verdrahten; Egress-Audit-Layer (nur Hashes) | REG-06 erfüllt |
| M7.7 | Rechtsprüfung der Formulierungen (O-2, O-3); Hülle EP2 messen | Freigabe für 1.0.0-Gate |

### M8 — EP3 Retrieval+ (≈ 4 Wochen)
`GraphPersistence`-Port und Implementierung; Ingestion-Ereignisse mit Cache-Replay (§9.5); Graph-Löschung mit Kaskade und Layer 21; `find_path` über `graph`-Feature und MCP-Tools `relate`/`explain`; `find_k_paths`-Benchmark-Gate, danach Entscheidung `pub`; `AGT-GRAPH-001` als Laufzeitprüfung; Löschumfang für Graph-Kanten (O-6) abschließen.

### M9 — EP4 Inference (≈ 4 Wochen)
Embedder-Backends hinter Features; `kvcache` mit KIVI, SnapKV (candle `kv-bridge`), mandantengetrenntem Prefix-Trie, Refcount-Grenze, Tier-2 (verschlüsselt); Doc-Bezug je Block und Löschung; `AlignedBuf` aus `sys`; Messung statt „87,5 %".

### M10 — EP5 Cognition (experimentell)
`adapt` (PID, sketched Bandit mit Reset), `cognition` (Spektral-Dedup mit Provenienz, `dirty`-Rebuild), `router`, `agent` (`status` privat); Kalibrierset und Bandit-Zustand im Löschumfang; DiBud-Gate (§9.2).

### M11 — EP6 Guard (nach Bedarf)
`privacy` (DLP, Surrogate-Vault, Egress-Gateway fail-closed), `sandbox` (WASI-Merge-Operatoren mit Fuel/Timeout; implementiert `MergeOperatorFactory`), `license` (Online/Offline-Entscheidung O-5), Tool `contextra_cloud_query` hinter `guard`.

---

### 12.2 Nachträge v1.3 (Architekturlücken)

| AP | Aufgabe | Dateien | Akzeptanz | Aufw. |
|---|---|---|---|---|
| N0.1 | Bedrohungsmodell §5.16 und Betriebsannahmen §5.17 als Pflichttext; Claims §5.14 gegen A0–A4 prüfen | Spec, Doku | G-19 grün mit angepassten Formulierungen | S |
| N0.2 | Ausnahmeliste §4.3a in `capabilities.toml` eintragen; G-21 aktivieren | `capabilities.toml`, `xtask` | G-21 grün | S |
| N0.3 | Befundstand F-39 bis F-51 und Gate-Status (G-11, G-12 u. a.) am Commit-Pin messen | `docs/baseline/` | Datei mit Status je Befund und Gate | S |
| N1.1 | Port-Namen nach §7.4 vereinheitlichen (`Rng` entfällt); Feature-Baum-Gate; Selbst-Dev-Dependency entfernen; Konstruktor-Tests nach `contextra-store` verlegen | `contextra-ports`, `contextra-store`, `contextra-testkit` | G-22 und G-23 grün; Ports-Tests ohne `backend-impl` grün | M |
| N1.2 | `contextra-ports`-Semver-Baseline setzen | `xtask`, CI | G-26 (warn) grün | S |
| N2.1 | `unwrap_or_default` in der Mandantenauflösung ersetzen | `contextra-engine`, `contextra-store` | G-27 grün | M |
| N3.1 | Übergangsausnahme E-1 (`store → core`) zusammen mit M3.1 abbauen | `contextra-store/Cargo.toml`, `capabilities.toml` | G-21 zählt E-1 ab | S |
| N4.1 | Einzelprozess-Sperre `<store>/LOCK` mit `flock`; Fehler `StoreLocked`; Zwei-Prozess-Test | `contextra-store` (open) | G-24 grün | M |
| N4.2 | clippy-Verbote um `SystemTime::now` und `rand::random` ergänzen; Ausnahmen über `#[allow]` mit Kommentar (E61) | `clippy.toml`, `xtask` | G-09 (deny) grün | S |
| N5.1 | Integritätsschlüssel-ID, Rotation, Zeitanker, Quarantäne in Ticks (E59, E60) | `contextra-store` (Manifest, Ledger, Keyring) | G-28 grün; Rotationstest | L |
| N5.2 | Übergangsausnahme E-2 (`store → crypto`) beim Crate-Split abbauen | `Cargo.toml`, `capabilities.toml` | G-21 zählt E-2 ab | M |
| N6.1 | Plattform-CI-Jobs (G-25) und Log-Redaktion (G-29) | CI, `contextra-mcp` (Logging) | G-25 und G-29 grün | M |
| N0.4 | Owner je Arbeitspaket verpflichtend; Kalender je Meilenstein festlegen (E72). Die Phasensummen (39, ca. 12, ca. 9 Wochen) müssen vereinheitlicht werden | §12 | Jeder AP hat Owner; Kalender entschieden | S |
| N0.5 | VETO-F02 prüfen: 6 Treffer in `contextra-vector/src/hnsw/`. Volltext lesen (O-21), Prüfung vor Frist 2027-10-12 | `VETOES.md`, `contextra-vector` | `check-vetoes` ohne offene Treffer | S |
| N0.6 | Dev- und normale Kanten mit `cargo tree -e normal` und `-e dev` belegen (Widerspruch Agent/Pack, R0-07). `engine → license` als DAG-Verstoß bestätigen | `Cargo.toml` je Crate | G-07 mit Belegdatei | S |
| N1.3 | Stubs entfernen: `sanitize_payload` (DLP, R7-15), `merge()` (R7-16), Surrogat-Hash keyed (R7-18). Vorläufig über G-30 absichern (E69) | `contextra-privacy`, `contextra-sandbox` | G-30 grün | M |
| N2.2 | `TenantId` und `TxId` kapseln (E74, R2-02) | `contextra-types` | Compile-Test: kein `pub`-Feld | S |
| N2.3 | Doppeldefinitionen nach §4.11 auflösen. Einzelnachweis je Typ | je Typ | Grep-Gate ohne Doppeldefinition | M |
| N4.3 | Resume: Scope-Journal, Sichtbarkeitsfilter, Dispose `RequireScrub` (E66, E76) | `contextra-store/src/lsm/purge/`, `recovery.rs:546` | CRASH-02 bis CRASH-04 grün; Leser-Test: gelöschter Key während Purge unsichtbar | M |
| N5.3 | Ledger auf IST-Format mit Header (E65). Golden-Fixture des IST-Formats | `erasure_ledger/` | Upgrade-Test (G-16) grün | M |
| N5.4 | Checkpoint-Determinismus (MP2 §4.9) und rekursive Cascade-Invalidierung (GL-G3) bekommen eine Phase | `contextra-checkpoint` | Entscheidung und Tests | M |
| N6.2 | Canary auf alle Speicher erweitern, Kompression und Verschlüsselung abdecken (R1-07, LS §13.1). Basis: `contextra-testkit/src/deletion_oracle/` | `contextra-testkit` | G-32 grün | M |
| N6.3 | Migrationspfad für bestehende Stores und Beweise v2 und v3 (Umstrukturierung §7) | `tests/` | G-16 grün; alte Beweise klar als ungültig oder prüfbar markiert | M |
| N6.4 | Block-Cache-Invalidierung prüfen (O-20). Die Stubs aus dem FE-Dokument sind nicht bestätigt | `contextra-store` (Block-Cache) | Messung dokumentiert; Rohsweep nach Purge | S |

## 13. Entscheidungsregister (ADR-E)

Alle Entscheidungen sind final, sofern §14 sie nicht ausdrücklich als offen führt. Jede ist als Datei `docs/decisions/ADR-E<nn>-<slug>.md` anzulegen (M0.6). „Verworfen" nennt die Alternative und den Grund, damit die Frage nicht erneut gestellt wird.

| ID | Entscheidung | Begründung | Verworfen |
|---|---|---|---|
| **E01** | EP1 liefert `Off` und `Assured`; `Attested` ist EP2. Erstes Release v0.1.0; 1.0.0 ist ein Gate (§1.3). | `Attested` existiert nur als Enum-Variante, kein Port hat eine Implementierung; spart ≈ 4 Wochen und vermeidet ein Versprechen ohne Deckung. | „Release 1.0 mit Attested" |
| **E02** | Genau ein Ringmodell (Ring 0–4 + Tooling), maßgeblich §4.1, gespiegelt in `capabilities.toml`; Ordnung innerhalb eines Rings (§4.2). | Vier parallele Modelle verhindern ein eindeutiges Gate (F-18). | „Layer 0–10" als zweites Ringsystem |
| **E03** | Genau drei Unsafe-Inseln: `sys`, `simd`, `wire`. `kvcache` bleibt `forbid(unsafe_code)`; ausgerichtete Allokation als sichere Fassade in `sys`. | Auditierbarkeit; kein `unsafe` in Ring 1. | `unsafe` in `kvcache`/`vector`/`store` |
| **E04** | Kryptosuite nach §6.1 (AES-256-GCM-SIV, Argon2id, HKDF-SHA-256, HMAC-SHA-256, BLAKE3, Ed25519 `verify_strict`). Kein Claim „BSI-konform". | Entspricht dem Code; TR-02102 ist keine Löschnorm. | SHA-256 als Beweisdigest, ChaCha20, plain GCM |
| **E05** | Beweisformat v4: TLV, `u16`-Tag/`u32`-Länge Big-Endian, Critical-Bit `0x8000`, Domain-Tag 28 Byte; Tag-Tabelle generiert; `layer_id: u16`; Signatur Pflicht in `Attested`; zwei Beweisstufen über `stage`/`supersedes`. | Entspricht `proof_v4.rs`/`canon.rs`; Offset-Tabellen veralten, Golden-Vektoren nicht. | CBOR, Little-Endian-Tabellen, Festlayouts |
| **E06** | v3-Beweiserzeugung wird gelöscht, v2/v3 bleiben verify-only; kein Feature `legacy-proofs`. | Zwei Serialisierungen unter einem Domain-Tag verletzen die Domain-Trennung. | `deprecated`-Markierung |
| **E07** | Normativ ist die Layer-ID-Liste aus §6.3 (15 IDs: 1–7, 20–26, 40; Layer 40 nur EP6, E52). Der IST-Code hat 13 `DeletionLayer`-Varianten, die Differenz ist E64/O-18. Sieben Beweisklassen sind eine Auditor-Sicht (§6.3). | Kein „7-Layer"-Modell deckt sich mit dem Code; Erweiterbar ohne Formatbruch. | Festes `[LayerProof;7]` |
| **E08** | Crypto-Erase zweistufig; 72 h Quarantäne nur für Collection/Tenant, 0 h für Dokumente; **kein DEK-Backup**; Dokument-DEK optional (§5.13). | Ein Backup macht die Löschung umkehrbar; Art.-17-Unverzüglichkeit. | „Backup plus 72 h" |
| **E09** | Commit-Punkt = durables Ledger-Append; danach Roll-forward; `Indeterminate`-Semantik; Cancellation-sicher (§5.3). | Ohne Vertrag widerspricht Recovery dem gemeldeten `Err` (F-02). | Roll-back nach Commit |
| **E10** | `PurgeState` als persistiertes `#[repr(u8)]`-Enum mit Golden-Test (§5.6). | Zustand muss crash-fest und formatstabil sein. | Implizite Zustände aus Dateien |
| **E11** | `KvDeleteMode::default() = TombstoneOnly`; Builder setzt `CryptoShred` bei `encryption-at-rest`. | Verborgene Kopplung des Defaults an Verschlüsselung. | Default `CryptoShred` |
| **E12** | **Fence-Protokoll B als Drain-Fence** mit `inflight`-Zähler (§5.8), Loom mit 3 Szenarien als Merge-Blocker. | Der Check am Eingang plus „Atomic-Load im Leader" ließe ein TOCTOU-Fenster zwischen WAL-I/O und Publish; der Drain beweist die Lückenfreiheit ohne WAL-Formatänderung. | Reiner Eingangs-Check; Void-Records im WAL |
| **E13** | Kein Crate `contextra-erasure` in EP1; Ledger, Purge, Resume, RestoreGuard bleiben im Store. Crate-Schnitt von Crypto in M5, Evidence/Verifier in EP2. | Ring 1 darf nur auf Ring 0 hängen; ein Plugin mit Store-Abhängigkeit müsste Ring 3 sein und brächte in EP1 keinen Nutzen. | Eigenes Erasure-Crate vor Attested |
| **E14** | `facade.default = ["fast"]`, `engine.default = []`; `candle` opt-in; `rank` Pflicht. | Hülle ≤ 18 ist sonst unerreichbar. | Default mit `candle` |
| **E15** | `contextra-core` wird ersatzlos entfernt, ohne Shim; Skript-PRs über Kartenbudget. | Tote Fassade mit 404 Referenzen. | Dauerhafte Strangler-Fassade |
| **E16** | `contextra-crypto` wird ohne Shim gesplittet (§4.8). | Vor v0.1.0 keine externen Konsumenten; Beweiscode gehört nicht ins EP1-Binary. | Rückwärtskompatibler Shim |
| **E17** | Formatversionierung: Upgrade-on-open, Downgrade immer abgelehnt; Hochwassermarke monoton; `AcknowledgeToken`; Integritätsschlüssel (§5.9). | Vorhandener `ManifestFormatTracker` wird erweitert; ehrliche Schutzgrenze dokumentiert. | Freier Moduswechsel |
| **E18** | MCP: Mandant beim Prozessstart gebunden, kein `SYSTEM`-Fallback, nie aus Tool-Argumenten. | Transport ist stdio; Header/TLS-Token sind dort unmöglich. | Session-Header, `SYSTEM`-Default |
| **E19** | MCP-Dispatcher nutzt `spawn_blocking` mit begrenztem Pool; Löschpfad nach Commit nicht abbrechbar. | Synchroner Kern; Cancellation-Sicherheit. | `block_in_place` im async-Kontext |
| **E20** | DurabilityMode × ErasureMode-Matrix (§4.6). | Zwei Konzepte, definierte Interaktion. | Ein vereintes Konzept |
| **E21** | *(überholt durch E65, v1.4)* Ledger speichert nur Keyed-Hashes im IST-Format; keine Kompaktierung, kein Klartext-Intent (§5.9). | Das Ledger darf nach Abschluss keine Klartextschlüssel gelöschter Daten tragen. | Dauerhafter Klartext-Scope |
| **E22** | Fließkomma-Politik: Reproduzierbarkeit je ISA, Bit-Identität nur für Festkomma/Integer, kanonischer Kernel beim HNSW-Aufbau, NaN-Ablehnung am Eingang (§7.6). | FMA bricht ISA-Gleichheit; Kahan hilft nicht; `total_cmp` ordnet negatives NaN vor −∞. | „Bit-Identität über alle ISAs", Kahan-Pflicht |
| **E23** | DiBud nur auf materialisierten Listen, strikt `>`; Feature `dibud` bleibt aus, bis Gates bestanden (§9.2). | HNSW nicht präfix-stabil; Gleichstandslücke bei `≥`. | DiBud direkt auf wachsendem `ef` |
| **E24** | Bandit: Sparse JL, Reset bei Re-Sketch, keine Regret-Zusage; Count-Sketch verworfen (§9.7). | Historie wird nicht gehalten. | Re-Projektion der Kovarianz |
| **E25** | OpenIE-Ergebnisse als Ereignis cachen und wiedergeben; kein `temperature=0`-Determinismusclaim (§9.5). | Keine Hardware-/Versionsinvarianz. | LLM-Replay |
| **E26** | KV-Cache: Prefix-Trie je Mandant; Refcount-Pin ≤ 50 %; SnapKV nur mit candle; Rückholung `O(Block)` (§9.4). | Timing-Seitenkanal; Eviction-Blockade; Attention-Zugriff nur in candle. | Mandantenübergreifender Trie; „O(1)" |
| **E27** | `find_k_paths` bleibt `pub(crate)` bis Benchmark-Gate in EP3. | Fertiger Code ohne Aufrufer. | Sofortige Exposition |
| **E28** | Lock-Reihenfolge global (§7.1); sync-Locks nie über `.await`, async-Locks erlaubt, wenn gelistet. | Frühere Formulierung war falsch. | „Keine Locks über `.await`" |
| **E29** | Lizenz steuert nur das Ausstellen neuer Beweise; kein Downgrade, kein `open`-Blocker bei Ablauf (§5.15). | Löschpflicht darf nicht an Lizenzstatus hängen. | Fail-closed `open` bei Ablauf |
| **E30** | Snapshot-Pin-Policy mit `max_snapshot_age` und `pin_grace` (§5.10). | Ein Langläufer-Reader blockiert sonst die Zeroisierung unbegrenzt. | Unbegrenztes Warten |
| **E31** | Checkpoints: bei Collection/Tenant-Löschung scrubben; bei Dokument-Löschung Restore-Barriere und Option `checkpoint_scrub_on_erase`. | Kosten-Nutzen; ehrlicher Claim. | Immer alle Checkpoints neu schreiben |
| **E32** | `LicenseGate`-Port; `contextra-license` nicht in EP1. | Hülle und Ringregel (`engine`→Ring 4 verboten). | Hartes `license`-Dependency der Engine |
| **E33** | Benchmark-Gate über Instruktionszähler plus Wall-Clock auf dediziertem Runner (§11.3). | 1 % p99 auf CI-Hardware nicht messbar. | 1-%-Gate |
| **E34** | Unwrap-Ratsche `governance/ratchet.toml:unwrap_exact` ist die einzige Quelle; `.github/unwrap_baseline.txt` wird generiert. | Zwei Zahlen (502 vs. 786) widersprachen sich. | Zwei unabhängige Baselines |
| **E35** | `PhysicalScope`-Konstruktor nur mit Port-Feature `backend-impl`, nur vom Store aktivierbar (G-12). | Echte Opazität über Crate-Grenzen ohne `unsafe`. | Bloße Doc-Kommentar-Opazität |
| **E36** | Rangordnung der Quellen: Governance-Dateien > diese Spezifikation > Code (§0.2). | Eine Wahrheit pro Thema. | Mehrere gleichrangige Pläne |
| **E37** | Löschumfang umfasst alle abgeleiteten Speicher (§5.10); ein neues Artefakt braucht Eintrag und Scan-Layer. | Kernversprechen gilt nur, wenn Kopien erfasst sind. | Löschung nur im LSM |
| **E38** | Kein Crate `contextra-erasure`. Coordinator, Purge-Treiber und RestoreGuard bleiben in Store und Engine hinter den Ports (E13 bestätigt). | Ein Plugin-Crate hat in EP1 keinen Nutzen und vergrößert die Hülle (§4.4). | Plugin-Crate `contextra-erasure` (IMPLEMENTIERUNGSSPEZIFIKATION §3.2) |
| **E39** | Keine Deprecation-Shims bei Crate-Umzügen (E16 bleibt). | Shims halten alte Pfade in Ring 0 und 1 und erhöhen die Hülle. | `pub use`-Shims mit `#[deprecated]` (IMPLEMENTIERUNGSSPEZIFIKATION §3.4, ARCHITEKTUR_MASTERPLAN_v2 §10.3) |
| **E40** | Kein Replica-Layer und kein Replica-ACK. Layer 5 bezeichnet Checkpoints. | REG-05 (Single-Node). | Layer 5 als „Replica/Backup ACK" (ARCHITEKTUR_MASTERPLAN_v2 §7.6) und als „Replikat" (ADVANCED §7.1) |
| **E41** | Beweisserialisierung bleibt TLV v4 (E05). Kein CBOR-Export. | Ein zweites Export-Format verdoppelt Kanonisierung und Testvektoren. | CBOR-Export (ARCHITEKTUR_MASTERPLAN_v2 §7.8, ADVANCED §7.4) |
| **E42** | Kein Beweis mit `Uncovered`- oder Teilstatus für einen Pflicht-Layer. Fehlt ein Layer, entsteht ein Fehler mit Layer-ID. | REG-14, §6.2 (`Pending` verboten), Fail-closed. | `Uncovered(reason)` im signierten Beweis (Recherche Feld J) |
| **E43** | Keine Kahan-Summation als Politik. Gleitkomma-Reduktionen folgen §7.6 Punkt 3. | Kahan ersetzt keine feste Reduktionsordnung. „NaN wird ans Ende sortiert" ist falsch (§7.6 Punkt 5). | Kahan-Politik und NaN-Aussage (ADVANCED §8.1) |
| **E44** | `Irminsul` bezeichnet die Eviction-Strategie (EP4). Positionsunabhängiges Caching ist Kandidat `PIC`, nur mandantengebunden und nur nach ADR. | Derselbe Name steht in ADVANCED §1 und in der Recherche für Verschiedenes. Mandantenübergreifende Treffer verletzen §9.4 (Timing-Seitenkanal). | Mandantenübergreifender Inhalts-Cache (Recherche Feld M) |
| **E45** | Conformal: Ranggrenze ohne Clamp. Kalibrierset nur aus relevanten Dokumenten. | Clamp auf `n` verletzt die Abdeckung bei kleinem `n`. Gemischte Scores ergeben keine einheitliche Nonkonformitätsfunktion. | `clamp(rank,1,n)` und gemischte Scores (ADVANCED §3.2) |
| **E46** | OpenIE ist nicht durch `temperature=0` deterministisch. Determinismus entsteht nur durch Ereignis-Persistenz (E25). | Modell- und Hardware-Varianz. | `temperature=0, fixed seed` (ADVANCED §2.2, ARCHITEKTUR_MASTERPLAN_v2) |
| **E47** | DiBud: `U_c = 1/(k+p_c+1)`, Zertifizierung strikt `>`, kein Tie-Break auf Schranken. | Nur so ist die Schranke eine obere Schranke. Ein Tie-Break auf Schranken ist ohne echte Scores unsound. | `U_c = 1/(c+n_c)` und `≥`-Variante (ARCHITECTURE_PLAN_v1 §19.3) |
| **E48** | Kein Regret-Ausdruck und keine Formel `k = O(√d log d)` für den Bandit. E24 bleibt. | Keine Herleitung für die verwendete Projektion. Qualität nur per Benchmark. | `O(k√T log T)` (ADVANCED §4.4) |
| **E49** | ESSN ist Kandidat hinter Benchmark-Gate. Default bleibt SSI. Latch-freie Varianten sind verboten. | Beleg fehlt, Commit-Overhead auf 4 Kernen ungemessen, P28. | ESSN als Default (Recherche Feld F), latch-freie SSN (ARCHITECTURE_PLAN_v1 §19.2 verbietet sie selbst) |
| **E50** | „Unwiderruflich zerstört" gilt erst nach Stufe 2 der Krypto-Löschung. Vorher gilt „widerrufen, Material in Quarantäne". | §5.13 (E08), §5.14. | Sofortige Vernichtung bei Collection-Drop (ARCHITEKTUR_MASTERPLAN_v2 §7.2, IMPLEMENTIERUNGSSPEZIFIKATION §10.2) |
| **E51** | Ringzuordnung nach §4.1. Abweichende Ringe aus Anhängen werden verworfen. Ausnahme: G5 bleibt offen (O-12). | Eine einzige Ringquelle (P12). Die Anhänge widersprechen sich untereinander. | ADVANCED (kvcache, adapt, rank, crypto Ring 2), SOTA (kvcache, rank, crypto Ring 0; adapt Ring 1; cognition Ring 2), ARCHITEKTUR_MASTERPLAN_v2 (graph Ring 3) |
| **E52** | Egress-Audit ist Scan-Layer 40 (EP6) und steht in §6.3. Das Scan-Feature heißt `erasure-evidence`, nicht `erasure-scan`. | §5.10 verwendet 40 bereits. Der Feature-Name gehört zur Produktgrenze EP2. | `erasure-scan` als Feature-Name (ARCHITEKTUR_MASTERPLAN_v2 §4.4, IMPLEMENTIERUNGSSPEZIFIKATION §8.3) |
| **E53** | Feature-Unifikation wird durch ein Feature-Baum-Gate abgesichert (G-22). Konstruktor-Tests von `PhysicalScope` liegen in `contextra-store`. | Cargo vereinigt Features; eine Deklaration allein erzwingt die Grenze nicht. Die Selbst-Dev-Dependency in `contextra-ports` aktiviert das Feature für jeden Testbuild. | Selbst-Dev-Dependency; reine Doku-Regel |
| **E54** | Ringverletzungen werden nur als befristete Ausnahmen in `capabilities.toml` geduldet (§4.3a, G-21). | Der Crate-Split (§4.8) und der Core-Abbau (§4.9) können nicht in M1 erfolgen. Stille Verletzung würde G-04 und G-07 entwerten. | Stille Duldung bis M5 |
| **E55** | Verbindliche Port-Namen nach §7.4: `Clock`, `EntropyRng`, `SeededRng`, `IdGenerator`, `SystemRng` (Impl). `Rng` entfällt. | Der Begriff `Rng` ist im Code mehrdeutig (Entropie gegen Test). Die Trennung ist sicherheitsrelevant (§5.13). | Generischer Name `Rng` |
| **E56** | Einzelprozess-Sperre über `flock` auf `<store>/LOCK`; Fehler `StoreLocked` (§5.17, G-24). | Zwei Prozesse auf einem Store zerstören WAL und Manifest. Ohne Sperre gilt das Kernversprechen nicht. | Nur Dokumentation; read-only-Fallback |
| **E57** | Plattform-Tiers, vorläufig (§5.17, O-17): Tier 1 Linux; Tier 2 macOS mit `F_FULLFSYNC` und Crash-Test; Windows nicht in v0.1.0. | Die Dauerhaftigkeit (§5.3) hängt am `fsync`-Verhalten des Betriebssystems. Ohne Beleg gibt es keinen Claim. | Implizite Unterstützung aller Plattformen |
| **E58** | Bedrohungsmodell A0 bis A4 (§5.16) ist Grundlage aller Claims (§5.14). | Verstreute Schutzgrenzen (E17, E50) sind nicht prüfbar. Ein Modell macht Claims gegen Angreiferklassen testbar. | Verstreute Einzelaussagen |
| **E59** | Zeitbasis: MAC-geschützter Monotonie-Anker im Manifest; Quarantäne in Ticks; Rücksetzung fail-closed (§5.9). | Reine Wanduhr erlaubt eine Verkürzung der 72-h-Quarantäne (F-45). | Reine Wanduhr |
| **E60** | Integritätsschlüssel mit ID und Rotation; alte Schlüssel bleiben verify-only (§5.9). | Ohne Rotation ist ein Schlüsselwechsel nicht ohne Bruch der Kette möglich. | Schlüsseltausch ohne Kette |
| **E61** | clippy-Ausnahmen nur über `#[allow]` mit Kommentar; die Ratsche zählt sie (§7.5, G-09). | `clippy.toml` gilt workspaceweit und kann nicht pro Crate ausnehmen. | Crate-spezifische `clippy.toml` (nicht möglich) |
| **E62** | Logs und Tracing liegen außerhalb des Löschumfangs; Doc-IDs nur als keyed hash (§5.2, G-29). | Logs enthalten Doc-IDs im Klartext. Eine Löschung, die Logs einschließt, ist mit vertretbarem Aufwand nicht zu leisten. | Logs im Löschumfang |
| **E63** | API-Stabilität: vor 1.0 frei mit semver-Warnung; ab 1.0 blockierend (G-26). | Ports sind der Vertrag zwischen Ringen. Ungeprüfte Änderungen brechen Konsumenten still. | Keine Stabilitätsregel |
| **E64** | Normativ sind die 14 Layer-IDs 1–7 und 20–26 (Code: 14 Varianten). ID 40 (EP6) bleibt bis zum Start von EP6 außerhalb des Enums (E52). | Die Zählung 13 aus v1.3 war ein Zählfehler (E78). §6.3 nennt 15 IDs, weil ID 40 erst mit EP6 hinzukommt. | „14 Varianten" (E07 alt) |
| **E65** | Ledger behält das IST-Record-Format mit Keyed-Hashes und ergänzt einen Datei-Header `CTRALEDG`. BLAKE3-keyed ist eine dokumentierte Ausnahme zur HMAC-Regel. Kein Klartext-Intent, keine Kompaktierung (§5.9). | Das IST-Ledger speichert bereits nur Keyed-Hashes (`erasure_ledger/entry.rs`). Ein Klartext-Intent gehört nicht ins Ledger. | Neues Ledger-Format v1 mit Klartext-Intent und Kompaktierung (v1.1 bis v1.3) |
| **E66** | Versiegeltes Scope-Journal für Resume: AES-256-GCM-SIV, vor `LedgerAppended` fsync-fest, nach `Completed` vernichtet (§5.18). | Resume braucht den Scope. Das Ledger kennt nur Hashes. Ohne Journal ist Resume nicht möglich (FE-18). | Scope aus dem Ledger; Scope aus Hashes rekonstruieren |
| **E67** | Zustandsnummern 0 bis 8 nach FE-10 (`#[repr(u8)]`, Golden-Test). `Scanning` entfällt. | Die Nummern in v1.1 widersprachen FE-10. | Werte 1 bis 9 aus v1.1 |
| **E68** | HNSW-Löschung ohne Kantenumverdrahtung. `remove_with_graph_repair` wird für Löschungen nicht verwendet. O-10 ist erledigt. Die Veto-Bedingung gilt, bis ein ADR sie ausdrücklich aufhebt. | VETO-F02 ist bedingt akzeptiert. Governance hat Vorrang (§0.2). | Kantenreparatur bei Löschung (v1.1 bis v1.3) |
| **E69** | REG-18: Keine Stubs mit Erfolgsrückgabe in Produktion (G-30). | DLP-Sanitizer und `merge()` geben heute unveränderte oder leere Erfolge zurück (R7-15, R7-16). Das ist fail-open. | Stubs bis zur Umsetzung hinnehmen |
| **E70** | Eigentümer je Doppeldefinition vor dem Crate-Schnitt (§4.11). | Ohne Eigentümer driften Kopien auseinander (FE §3.7). | Doppelte Typen dulden |
| **E71** | Führende Roadmap ist §12 dieser Spezifikation. Roadmap.md, die Umstrukturierungs-Roadmap, IS und MP2 liefern Arbeitspakete, werden aber nicht als eigene Pläne fortgeführt. | Vier parallele Pläne sind nicht prüfbar (Roadmap §7). | Eigene Roadmap je Quelle |
| **E72** | Jeder Arbeitspaket-Eintrag braucht einen Owner. Der Kalender wird pro Meilenstein festgelegt. | Ohne Owner und Kalender gibt es keine Verantwortlichkeit und keine Prüfung des Zeitplans. | Schätzung ohne Owner |
| **E73** | Loom-Modell mit drei Threads und drei Szenarien: Fence vor Fenster 1, zwischen den Fenstern, nach Fenster 2. | FE-12 nennt zwei Szenarien. Drei decken die Lücke zwischen Check und Publish ab. | Zwei Szenarien (FE-12) |
| **E74** | `TenantId` und `TxId` sind gekapselt. Erzeugung nur mit Prüfung (§5.11). | Öffentliche Felder erlauben Mandanten- und Transaktionswerte ohne Prüfung (`ids.rs`). | Öffentliche Felder mit Konvention |
| **E75** | Ack-Schlüssel für den Downgrade-Schutz per HKDF aus dem Integritätsschlüssel (Info `contextra/ack/v1`). | Der Schlüssel war unspezifiziert (FE-15, O-4). Eine Ableitung braucht keinen neuen Speicherort. | Eigenes Signaturschlüsselpaar für den Ack |
| **E76** | Resume: Gelöschte Schlüssel sind für Leser unsichtbar, bis `Purged` erreicht ist. Die Produktions-Dispose ist `RequireScrub`. No-op-Hooks sind nur bei `Off` zulässig (G-31). | Ein Hintergrund-Purge hätte sonst Lesezugriff auf noch vorhandene Daten (FE §4.9). | Blockierendes `open()` bis Ende des Purge |
| **E77** | Tenant-ID-Herkunft: stdio mit Bindung beim Prozessstart (E18). Andere Transporte brauchen ein eigenes ADR. | MP2 §19.1 ist nicht entworfen. Der Transport ist heute stdio. | Bindung aus Tool-Argumenten |
| **E78** | Die Layer-Zählung ist 14 Varianten (IDs 1 bis 7 und 20 bis 26). ID 40 (EP6) folgt mit EP6. Die Zählung 13 aus v1.3 war ein Zählfehler. | Der Code wurde nachgezählt (`contextra-crypto/src/deletion_proof/types.rs`). | 13 oder 15 Varianten |

---

## 14. Offene Punkte, Annahmen, Risiken

### 14.1 Offen (nicht raten, Rückfrage an den Principal Architect)

| ID | Punkt | Auslöser |
|---|---|---|
| O-1 | Signierschlüssel: Ablage, Rotation, Verlust, Key-Registry, Trust-Anker. Der Verifier darf dem eingebetteten Schlüssel nie allein vertrauen. | Beginn M7 (M7.0) |
| O-2 | Rechtliche Formulierung zu DSGVO Art. 17 und § 371a ZPO; ob ein selbstverwalteter Ed25519-Schlüssel eine geeignete Beweisform ist | Vor 1.0.0 und vor jeder rechtlichen Aussage |
| O-3 | BSI-Konformitätsmatrix gegen die aktuelle TR-02102 (BLAKE3, GCM-SIV) | Vor Verwendung des Worts „konform" |
| O-4 | EP2 als eigenes Repository (Produkt-/Organisationsentscheidung) | Vor M7 |
| O-5 | Lizenzprüfung online oder offline; Aussteller der Lizenzen | Vor M11 |
| O-6 | Löschumfang für Graph-Kanten, KV-Blöcke, Vektor-Slots **und Inhalte von MCP-Tool-Calls** (Audit/Logging) im Detail | Vor M8 und M9 |
| O-7 | Kalibrierung der quantitativen Startwerte (§4.5, §5.7, §5.10, §9) | M4/M6 |
| O-8 | Genaue IST-Struktur von `blob/` und `copy_registry` sowie des IST-Ledger-Formats | M2.1, M5.2 (Re-V.) |
| O-9 | Numerischer Typ für Kalibrierung und Spektral-Dedup: f32 (§7.6, Recherche) oder f64 (ADVANCED §3, §5). Bestimmt die Reproduzierbarkeit je ISA. | Vor M4.9 |
| O-10 | VETO-F02 gegen HNSW-Graphreparatur `remove_with_graph_repair` (§5.10, Recherche Feld C, ARCHITEKTUR_MASTERPLAN_v2 §4.4). Konflikt zwischen Governance (Vorrang nach §0.2) und Löschanforderung. Entschieden in v1.4 (E68): keine Umverdrahtung beim Löschen. Das Veto bleibt bis zu einem ADR bestehen, das es ausdrücklich aufhebt. | Erledigt |
| O-11 | ESSN-Benchmark: Abbruchrate und Sequencer-Overhead bei ≥ 8 Schreib-Threads auf 4 Kernen, Schwelle 5 %. | Vor Entscheidung §9.9a |
| O-12 | Ringzuordnung von `contextra-audit-export` und `contextra-avv-generator` (G5). Ferner Masterplan-Zuordnung `graph` Ring 3 gegen §4.1 Ring 0. | Vor M4.5 |
| O-13 | Compile- und Zählbefunde der Anhänge am gepinnten Commit messen: „Engine kompiliert nicht" (IMPLEMENTIERUNGSSPEZIFIKATION §1.2), unverdrahtete `pub fn` (512 in §3.1, 324 im Teamdokument, 101 in ARCHITEKTUR_OPTIMIERUNG_v1 und ARCHITEKTUR_MASTERPLAN_v2), Crate-Zahl (37 in ARCHITECTURE_PLAN, 34 am HEAD). | M0.3 |
| O-14 | Zählregel für die Ratsche (F-34): Welche Muster zählen (`unwrap()`, `expect(`, Inline-Tests)? Die Antwort bestimmt, ob `unwrap_exact` = 502 oder eine andere Zahl die Basis ist. | Vor M0.8 |
| O-15 | Entkopplungsreihenfolge `contextra-store → contextra-core` (F-36): Dürfen `core`-Symbole in `types` verschoben werden, oder sollen sie über Ports laufen? | Vor M3.1 |
| O-16 | Ob die Dev-Kante `contextra-store → contextra-graph` (`Cargo.toml:68`) bestehen bleiben darf. Sie erzeugt bei Dev-Builds eine Zyklusprüfung (F-31). | Vor M4.5 |
| O-17 | Plattform-Tier 2 (macOS) und Windows: Produktentscheidung, ob macOS in v0.1.0 unterstützt wird (E57). | Vor N6.1 |
| O-18 | Geschlossen (E78): Der Code hat 14 Varianten, passend zu den IDs 1 bis 7 und 20 bis 26. ID 40 folgt mit EP6. | — |
| O-19 | Stufe des Angreifermodells für Produkt-Claims: Ist A2 (Schreibzugriff) ausgeschlossen oder nur mit externem Anker abgedeckt (§5.16)? | Vor M5 und Freigabe der Claims (Gate 3 in §1.3) |
| O-20 | Block-Cache-Invalidierung: Stubs laut FE-Dokument nicht bestätigt. Messung nötig (N6.4). | Vor M5 |
| O-21 | VETO-F02-Volltext und ADR-VETO-REVIEW-F02-OP03-2026-10 lesen. E68 gegen den Volltext prüfen. | Vor M5.7 |

### 14.2 Annahmen (änderbar, mit Folge)
- **A1:** `Attested` ist für das erste Release nicht nötig. Ist der Löschbeweis das Verkaufsargument, wandert er in den Umfang: ≈ +4–6 Wochen, M7 wird vorgezogen, der Crate-Schnitt aus E13/E16 entsprechend.
- **A2:** 72 Stunden Quarantäne für Collection/Tenant sind für Kunden tragbar. Der Wert ist konfigurierbar, 0 ist jederzeit möglich; die rechtliche Bewertung steht aus (O-2).
- **A3:** Ein Prozess bedient genau einen Mandanten im MCP (E18). Brauchen Kunden Mehrmandanten-MCP in einem Prozess, ist ein neuer Authentifizierungsweg nötig.

### 14.3 Risiken

| Risiko | W | Auswirkung | Mitigation |
|---|---|---|---|
| Fence/Drain: Deadlock oder Starvation bei hoher Schreiblast | M | Purge stockt | Loom (3 Szenarien), Drain-Timeout mit Backoff, Health-Eintrag |
| Purge v2 erzeugt neue Fehler | M | Datenverlust | Differenztest, Rohsweep, Crash-Matrix, Mutationstests |
| Crate-Split (M5.5) bricht Governance und CI | H | Verzug | Ein PR je Zielcrate, Skripte, vorab Inventar aller Treffer |
| Feature-Kombinations-Explosion | H | CI-Kosten | Matrix `{off, erasure} × {default, graph}` plus Isolation je Feature |
| HNSW-Löschung unvollständig (mmap) | M | Kernversprechen | Segment-Rewrite statt In-Place; Rohsweep über Store-Verzeichnis |
| Rewrite pro Löschung zu teuer | H | Durchsatz | Koaleszenz-Fenster, Budgets messen |
| Schlüsselverlust bei Crypto-Erase | N | Datenverlust | Betriebsdokumentation; bewusst kein Escrow |
| Pin-Policy bricht Langläufer-Reader | M | Fehler bei Clients | Konfigurierbar, klare Fehlermeldung `SnapshotExpired` |
| Zeitplan ohne Puffer | H | Verzug | ±25 %-Hinweis; M3/M5 enthalten XL-Pakete; wöchentliche Re-Planung |

### 14.4 Ehrlichkeits-Klausel
Nicht abgedeckt und in keinem Modus garantiert: SSD-Remanenz, CoW-Dateisysteme und Dateisystem-Snapshots, Prozessspeicher, Swap, Core-Dumps (nur gehärtet, nicht garantiert), externe Backups und Kopien, rechtliche Beweiskraft. Loom, Crash-Matrix, Benchmarks, Mutationstests und Fuzzing sind zum Zeitpunkt dieser Spezifikation **nicht ausgeführt**; die Roadmap macht sie zu Gates.

---

## Anhang A — Befund → Arbeitspaket

| Befund | AP | | Befund | AP |
|---|---|---|---|---|
| F-01 | M0.2, M2.5 | | F-17 | M3.1 |
| F-02 | M1.3, M2.6, M5.3 | | F-18 | M0.4, M4.5 |
| F-03 | M1.1 | | F-19 | M4.6 |
| F-04 | M1.2 | | F-20 | M1.6, M4.7, M4.9 |
| F-05 | M2.3 | | F-21 | M3.7, M6.4 |
| F-06 | M0.3, M2.1, M2.4, M2.7 | | F-22 | M2.7, M3.7, M5.6 |
| F-07 | M5.7, M5.8, M5.9 | | F-23 | M0.1, M0.4, M0.7 |
| F-08 | M1.4, M7.1 | | F-24 | M4.4 |
| F-09 | M5.6 | | F-25 | M4.2 |
| F-10 | M5.1 | | F-26 | M4.9, M8–M10 |
| F-11 | M4.1 | | F-27 | M5.2 |
| F-12 | M4.3 | | F-28 | M0.5 |
| F-13 | M3.6 | | F-29 | M0.3 |
| F-14 | M3.2, M3.3, M3.4 | | F-30 | M5.5 |
| F-15 | M3.5 | | F-31 | M3.4, M4.5 |
| F-16 | M1.5 | | | |

## Anhang B — Glossar

| Begriff | Bedeutung |
|---|---|
| Commit-Punkt | Durables Anhängen des `ErasureIntent` an das Ledger; ab hier Roll-forward (§5.3) |
| Drain-Fence | Fence mit Zählung laufender Batches, installiert erst nach deren Abschluss (§5.8) |
| Group-Commit-Leader | Der Thread im Store, der Schreibbatches bündelt; **kein** Replikationsbegriff |
| Hochwassermarke | Höchster je im Store verwendeter `ErasureMode`; nur erhöhbar (§5.9) |
| Hülle | Anzahl der Workspace-Crates im Abhängigkeitsbaum eines Produkt-Binaries |
| Rohsweep | Bytegenaue Suche aller Store-Dateien nach Scope-Schlüsseln und -Werten |
| Scrub | Überschreiben mit Nullen, `fsync`, Entfernen, Verzeichnis-Sync (`scrub_and_remove`) |
| Topo-Ebene | Reine Sortierhilfe der Doku; **kein** Ring |
| Re-V. | Befund muss am HEAD neu gemessen werden, bevor er behoben wird |

## Anhang C — PR-Checkliste für Agenten

1. Genau ein AP; AP-ID im PR-Titel.
2. IST vor Änderung gemessen (bei „Re-V.") und im PR festgehalten.
3. Akzeptanzkriterien als Test oder Gate vorhanden und grün.
4. Keine neuen `unwrap`/`expect`/`panic` in Ring 0/1; keine direkten `Instant::now`/`SystemTime::now`/`thread_rng`.
5. Jedes neue `pub`-Symbol hat Produktionsaufrufer, `///`-Doku und Fassadentest.
6. Persistente Formate: Version erhöht, Upgrade-Test mit Golden-Fixture.
7. Löschbezogene Änderung: Tenant-Matrix, Rohsweep, betroffene CRASH-Zeilen.
8. Keine Claims außerhalb von §5.14 in Doku, Fehlertexten oder MCP-Antworten.
9. Diff-Budget eingehalten; Ausnahme nur über Kartenbudget.
10. Offene Punkte (§14) nicht eigenmächtig entschieden.

---
*Ende der Spezifikation.*
