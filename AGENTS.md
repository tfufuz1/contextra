# Contextra — Agenten-Betriebsanleitung (AGENTS.md)

<!-- doc-stamp: 2026-10-29 10cc17922 | Spec-Fassung: 3.1 -->

Stand: 2026-10-29 · Basis: Gesamtspezifikation Fassung 3.1 (`docs/SPEC.md`)

---

## 1. Geltungsbereich und Rangfolge der Informationsquellen

Diese Datei regelt die Arbeit aller autonomen Agenten im Repository.
Bei Konflikten gilt zwingend folgende strikte Rangfolge:

1. **Verfassung und Vetos** (`docs/CONSTITUTION.md`, `docs/VETOES.md`)
2. **Normative Gesamtspezifikation Fassung 3.1** (`docs/SPEC.md`; §0, §1) — Baukasten-Modell
3. **Umsetzungsplan** (`docs/PLAN.md`)
4. **Capabilities Manifest** (`capabilities.toml`; INV-DOC-SINGLE-SOURCE-1)
5. **Governance Gate-Definitionen** (`governance/gates.toml`; §17.3)
6. **Agenten-Betriebsanleitung** (`AGENTS.md`)
7. **Produktionscode und grüne Quality Gates** (REQ-QA-010)

Quality Gates dürfen niemals durch Ausnahmen, Sonderregeln oder lokale Overrides geschwächt oder umgangen werden (REQ-QA-010).

> **Agenten-spezifische Anleitungen:**
> - `@Jules` → lies Abschnitt 2a und folge `.agents/jules/SESSION_BOOTSTRAP.md`
> - `Claude` / Antigravity → ignoriere Abschnitt 2a, lies stattdessen `CLAUDE.md`
> - Alle anderen Agenten → Abschnitt 2b reicht

---

## 2a. Start — @Jules

> **Nur für Jules. Alle anderen Agenten: diesen Abschnitt überspringen.**

```bash
# 1. Vorgefertigten Kontext lesen (durch refresh-context.sh beim Task-Start erzeugt)
cat .agents/jules/context/SESSION_START.md

# 2. Konkreten Task von GitHub holen
gh issue list --state open --label "jules" --json number,title,body,milestone
gh issue view <N>

# 3. Build-Grundlage prüfen (ADR-D11 / SPEC §0.2)
cargo check --workspace --all-targets --locked
```

> Vollständiger Ablauf: `.agents/jules/SESSION_BOOTSTRAP.md`
> GitHub-Konventionen (Labels, Protokollpflicht): `.agents/jules/GITHUB_WORKFLOW.md`

### @Jules — Ablauf je Task

1. **GitHub-Issue lesen** (`gh issue view <N>`) — Task, Akzeptanzkriterien, WP-Nummer, Scope.
2. Existenz von APIs, Dateien und Typen im Code prüfen (niemals aus dem Gedächtnis annehmen).
3. Plan mit Dateiliste, WP-Nummer und Akzeptanzbefehlen erstellen.
4. Kleinste mögliche Änderung durchführen (ein WP = ein PR = höchstens ein Crate-Schnitt, REQ-CHG-2).
5. Gates lokal grün: `cargo check -p <crate> --no-default-features --locked` && `cargo check -p <crate> --locked` && `cargo test -p <crate> --locked` (oder `just crate <crate>`) sowie `just check` / `just baseline`.
6. **Rebase auf origin/main** (`git fetch origin && git rebase origin/main`) — PFLICHT vor jedem PR.
7. Gates nach Rebase erneut prüfen, dann PR erstellen, Issue schließen.

### @Jules — CI/CD Fixer

In Pull Requests kann Jules durch Kommentare aktiviert werden:

```
@jules fix-ci           → Jules analysiert fehlgeschlagene CI-Jobs und öffnet Fix-PR
@jules check-gates      → Jules führt alle Gates und meldet Befunde als Issue-Kommentar
@jules recheck          → Jules wiederholt den Submit-Gate-Check
```

> **Wichtig:** Jules ändert niemals Gates, Lints, Ratchets oder den Toolchain-Pin, um Rot zu beseitigen.
> Jules behebt die Ursache — nicht das Symptom.

---

## 2b. Start — Alle anderen Agenten

```bash
cargo check --workspace --all-targets --locked
```

Dann: `docs/SPEC.md` §0 lesen (LLM-Schnellstart) und `capabilities.toml` für Ring/Klassen-Zuordnung prüfen.

---

## 3. Verifizieren, Rebase & Gate-Integrität (§27.5, §27.6, REQ-QA-010)

- „Fertig" bedeutet ausnahmslos: Alle relevanten Gates, Lints und Tests auf dem Merge-Ergebnis sind grün (REQ-QA-010).
- **Rebase-Pflicht (hochfrequentierte Entwicklung):** Vor jedem PR zwingend `git fetch origin && git rebase origin/main` ausführen. Nach dem Rebase Gates erneut prüfen. Kein PR ohne sauberen Rebase auf main.
- Es ist streng verboten, Gates, Lints, Ratchets oder den Toolchain-Pin einzuschränken, abzuschwächen oder durch Ausnahmen zu umgehen (REQ-QA-010).
- **Geschützte Pfade:** `.github/**`, `xtask/**`, `justfile`, `rust-toolchain.toml`, `deny.toml`, `capabilities.toml`, `AGENTS.md`, `.agents/jules/setup/**`, `governance/**`.

---

## 4. Die sechzehn Regeln für jede Änderung (SPEC §0.2)

> Diese Kurzreferenz verweist auf die normativen Definitionen in SPEC §0.2 (INV-DOC-SINGLE-SOURCE-1).

1. **Ring und Klasse zuerst.** Kante nur auf gleichen/niedrigeren Ring (§4.1) und in `may_depend_on` (§4.4). Kern [K] hängt nie von Rand/Experim./Archiv ab (P30).
2. **Jedes Crate ein Modul mit Vertrag.** Modul-Manifest, Evidence-Pack, isoliertes Prüfkommando (§7).
3. **Kein Panic in Produktion.** Kein `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!`, `unreachable!`, ungeprüftes Indexieren (P7, §3.2).
4. **Zeit, Zufall, IDs nur über Ports** `Clock`, `Rng`, `IdGen` (P28). Ausnahme: Krypto-Schlüssel/-Salts aus System-RNG.
5. **Alles Persistente geht durch WAL** (P24). Zustand liegt nur in Sidecar-Logs und Keystore (§8.5).
6. **I/O nur über `Vfs`**; `fsync`-Fehler werden propagiert, nie mit `let _ =` verworfen.
7. **Unsafe-Inseln (drei Inseln)** (`simd`, `sys`, `wire`), jeder Block mit `// SAFETY:` (P2, §4.6).
8. **Mandant aus RequestContext**, nie aus Literalen, Defaults oder Nutzerparametern (INV-TENANT-1).
9. **Sicherheit ist kein Feature-Schalter.** Krypto, Messung, Bericht immer aktiv; kein `cfg(not(feature))` für Sicherheit (A1).
10. **Messen vor Behaupten.** Restbestand-Zähler entsteht nur aus Scanner der besitzenden Schicht (INV-CLAIMS-1).
11. **Fail-Closed.** Unbekannter Zustand, Formatversion, fehlender Port: Ablehnung mit Grund.
12. **Kein Netz ohne Erlaubnis.** Ausgehender Verkehr nur laut Egress-Tabelle (§10.6), Default nur Loopback.
13. **Eine Typfamilie je Konzept.** Löschtypen nur in `crypto`/`ports`; Ports nur in `ports` (§6).
14. **Ein PR = ein WP = höchstens ein Crate-Schnitt**; danach Build, Modulprüfkommando, Zeilenzählung (REQ-CHG-2).
15. **Keine Test-Hooks in öffentlicher Produktions-API.** `*_for_test`, `Mock*`, `Fake*` hinter `cfg(test)` oder `test-hooks`/`testkit` (§12.4).
16. **Kein blockierendes I/O in `async`.** Dateizugriff über `Vfs` oder `spawn_blocking`; kein `std::fs`/`std::thread::sleep` in `async fn`.

---

## 5. Baukasten-Crates [K] / [M] / [E] — Baukasten-Modell (SPEC §5)

### 5.1 Kern-Crates [K]

| Crate | Ring | Status | WP-Aufgabe |
|-------|------|--------|------------|
| `contextra-types` | 0 | aktiv | Grundlegende Typen, IDs, Fehlerklassen |
| `contextra-ports` | 0 | aktiv | Port-Schnittstellen (Entkopplung) |
| `contextra-sys` | 0 | aktiv | Unsafe-Insel; Systemnahe Bindings, Low-Level-I/O |
| `contextra-simd` | 0 | aktiv | Unsafe-Insel; SIMD-Distanzkerne |
| `contextra-wire` | 0 | aktiv | Unsafe-Insel; FlatBuffers Wire-Protokoll |
| `contextra-mvcc` | 0 | aktiv | Snapshots, SSI, Sequenzen |
| `contextra-crypto` | 0 | aktiv | AEAD, KDF, Signaturen, Löschbericht V5 |
| `contextra-vector` | 0 | aktiv | HNSW, SQ8, ACL-Bitset |
| `contextra-text` | 0 | aktiv | BM25, Tokenizer |
| `contextra-rank` | 0 | aktiv | RRF-Fusion, Normalisierung |
| `contextra-store` | 1 | aktiv | WAL, LSM, SSTables, Kompaktion |
| `contextra-checkpoint` | 1 | aktiv | Persistent Checkpoint Engine |
| `contextra-engine` | 3 | aktiv | Planner, Collections, Entkopplung von Rand-Crates |
| `contextra-db` | 3 | aktiv | Main DB Abstraktion |
| `contextra` | 4 | aktiv | Composition Root, öffentliche Facade |

### 5.2 Rand-Module [M] — Baukasten-Module mit Vertrag (SPEC §4.2, §5)

> **Gemäß SPEC 3.1 / ADR-D01 sind Inferenz-, Agenten- und Schutz-Crates aktive Baukasten-Module [M] mit isoliertem Vertrag, KEIN Archiv.**

| Crate | Ring | Familie | Aufgabe / Entkopplungs-WP |
|-------|------|---------|---------------------------|
| `contextra-infer-candle` | 2 | F3 | Lokales GGUF Embedding- & Inferenz-Backend |
| `contextra-infer-onnx` | 2 | F3 | ONNX Embedding- & Cross-Encoder-Backend (WP 2.7: Download aus) |
| `contextra-infer-ollama` | 2 | F3 | Ollama Client (WP 2.7: Loopback-Default) |
| `contextra-kvcache` | 1 | F3 | Quantisierter KV-Cache |
| `contextra-adapt` | 0 | F3 | PID-Latenzregelung |
| `contextra-graph` | 0 | F4 | CSR-Graph, PathGraphing (WP 2.2: entkoppeln von `engine`) |
| `contextra-cognition` | 3 | F4 | Sleep-Cycle-Konsolidierung (WP 2.3: entkoppeln von `db`) |
| `contextra-router` | 3 | F4 | Lyapunov Drift Control, Routing |
| `contextra-agent` | 3 | F4 | Agent Execution Pipeline (WP 2.6: entkoppeln über Ports) |
| `contextra-privacy` | 3 | F5 | PII-Filter, Egress-Gateway |
| `contextra-mcp` | 4 | F5 | MCP-Server, stdio IPC (WP 2.7: Entkopplung) |
| `contextra-py` | 4 | F5 | PyO3 Python-Bindings |
| `contextra-sandbox` | 2 | F5 | WASM Execution Boundary |

### 5.3 Experimentell [E] & Auslaufend

- **Experimentell [E]:** `contextra-license`, `contextra-audit-export`, `contextra-avv-generator`
- **Auslaufend:** `contextra-core` (WP 2.8: schrittweise Entfernung der Kompatibilitätsfassade)

---

## 6. Veto-Register (SPEC §3.1, VETOES.md)

| Veto | Status | Regel |
|------|--------|-------|
| **VETO-F10** — Keine mandantenübergreifenden Datenflüsse | `permanent_rejected` | Kein Cache, Index, Embedding, Statistik oder Audit-Eintrag überschreitet Tenant-Grenzen. Tenant kommt aus der Transaktion, nie aus Nutzerparametern. |
| **VETO-F02** — HNSW-Löschung nur per Tombstone+Slot-Nullung+Rebuild | `bestätigt` | Kein Rewiring der Nachbarn. Slot-Nummern werden erst nach Rebuild-Abschluss wiederverwendet. |
| **VETO-OP03** — Kein Echtzeit-Audio/STT | `bestätigt (geschlossen)` | Kein Audio-Streaming im Kern. |
| **Single-Node-Doktrin** | `permanent` | Kein Sharding, kein P2P, keine verteilte Replikation. |

---

## 7. Sicherheitsports (INV-WIRING-2)

Folgende Ports MÜSSEN in Produktion real implementiert sein. Null-/Test-/Accept-All-Implementierungen als Standard sind unzulässig:

| Port | Crate | Pflicht-Impl | WP |
|------|-------|-------------|-----|
| `KmsProvider` | `ports` | `crypto::kms::K1Provider` | WP 5.1 |
| `Vfs` | `ports` | `store::vfs::StdVfs` | WP 4.1 |
| `DurableLog` | `ports` | `store::sidecar::DurableLogImpl` | WP 4.2 |
| `ErasureJournal` | `ports` | `store::sidecar::ErasureJournalImpl` | WP 4.2 |
| `LayerScanner` | `ports` | je Schicht (WP 6.x) | WP 6.1–6.7 |
| `Purger` | `ports` | `store::compaction::RangePurger` | WP 6.1 |
| `EffaceableKeySlot` | `ports` | K2/K3: `CapabilityUnsupported` (D-10) | WP 5.1 |
| `ErasedSetMirror` | `ports` | `engine::erasure::MirrorImpl` | WP 7.3 |

---

## 8. Aktueller Implementierungsstatus & Gate-Lage

Gemäß **ADR-D12 / SPEC §1.3** stammen alle verbindlichen Metriken, Zähler und Gate-Befunde aus `cargo xtask metrics` bzw. den dokumentierten Baselines unter `evidence/baseline-*`. Das Repository führt keine unüberprüften manuellen Altwerte im Dokumententext.

Automatisierte Audits und Entwicklungs-Gates werden per CI sowie lokal über `just check`, `just baseline` und `cargo xtask check` validiert.

---

## 9. Nicht-Ziele v1.0 (SPEC §2.3)

**Diese Fähigkeiten sind in v1.0 NICHT implementiert** und dürfen nicht hinzugefügt werden ohne ADR:

- Graph-Reasoning, Kognition/Konsolidierung
- Router/Agentenlaufzeit, Online-Adaption
- KV-Cache-Verwaltung, DiskANN/RaBitQ
- Windows für Server und MCP (Best-Effort, keine Garantieplattform)
- Verteilter Betrieb (kein Sharding, kein P2P)
- Echtzeit-Audio/STT (VETO-OP03)
- Löschbeweis als Rechtsnachweis
- Python-Wheels in v1.0
- Schlüsselverwahrung K2/K3 als ausgelieferte Stufe (Port vorhanden, `CapabilityUnsupported`)

---

## 10. Referenzen

| Dokument | Zweck |
|----------|-------|
| `docs/SPEC.md` | **Einzige normative Quelle** für Architektur, Verhalten, Konstanten, Tests, Gates, Plan |
| `docs/PLAN.md` | Implementierungsplan IST→SOLL, Arbeitspakete (WP 0.1–9.4) |
| `docs/ARCHITECTURE.md` | Lese-Hilfsmittel (nicht normativ; normativ ist SPEC §4) |
| `capabilities.toml` | Single Source of Truth für Ring, Klasse, Reifegrad je Crate |
| `docs/VETOES.md` | Veto-Register (VETO-F02, VETO-F10, VETO-OP03) |
| `docs/CONSTITUTION.md` | Grundlegende Governance-Prinzipien |
| `governance/gates.toml` | Gate-Katalog — Single Source of Truth aller Gate-Definitionen (§17.3) |
| `governance/ratchet.toml` | Qualitäts-Baselines (dürfen nur sinken, REQ-QA-005) |
| `governance/security_ports.toml` | Pflicht-Sicherheitsports (INV-WIRING-2, WP 0.4) |
| `governance/key_purposes.toml` | Schlüssel-Zweckbindung HKDF (INV-KEYPURPOSE-1, §12.3) |
| `governance/clippy.toml` | Workspace-Clippy: P28-Verbotsliste (disallowed-methods/-types) |
| `governance/determinism-allow.toml` | Erlaubte Determinismus-Ausnahmen (ADR-098) |
| `governance/protected-paths.toml` | Geschützte Pfade (REQ-GOV-002, §18.2) |
| `.agents/jules/SESSION_BOOTSTRAP.md` | Jules Session-Ablauf (primärer Pfad) |
| `.agents/jules/GITHUB_WORKFLOW.md` | GitHub-Konventionen für Jules |
| `CLAUDE.md` | Claude / Antigravity Agent Guide |
| `docs/decisions/` | ADR-Index | <!-- doc-ref-ignore -->
