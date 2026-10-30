# Contextra — Agenten-Betriebsanleitung (AGENTS.md)

<!-- doc-stamp: 2026-10-29 10cc17922 | Spec-Fassung: 3.0 -->

Stand: 2026-10-29 · Basis: Gesamtspezifikation Fassung 3.0 (`docs/SPEC.md`)

---

## 1. Geltungsbereich und Rangfolge der Informationsquellen

Diese Datei regelt die Arbeit aller autonomen Agenten im Repository.
Bei Konflikten gilt zwingend folgende strikte Rangfolge:

1. **Verfassung und Vetos** (`docs/CONSTITUTION.md`, `docs/VETOES.md`)
2. **Normative Gesamtspezifikation Fassung 3.0** (`docs/SPEC.md`; §0, §1) — Baukasten-Modell
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

# 3. Build-Grundlage prüfen
cargo check --workspace
```

> Vollständiger Ablauf: `.agents/jules/SESSION_BOOTSTRAP.md`
> GitHub-Konventionen (Labels, Protokollpflicht): `.agents/jules/GITHUB_WORKFLOW.md`

### @Jules — Ablauf je Task

1. **GitHub-Issue lesen** (`gh issue view <N>`) — Task, Akzeptanzkriterien, WP-Nummer, Scope.
2. Existenz von APIs, Dateien und Typen im Code prüfen (niemals aus dem Gedächtnis annehmen).
3. Plan mit Dateiliste, WP-Nummer und Akzeptanzbefehlen erstellen.
4. Kleinste mögliche Änderung durchführen (ein WP = ein PR = höchstens ein Crate-Schnitt, REQ-CHG-2).
5. Gates lokal grün: `cargo fmt --check && cargo check --workspace && cargo clippy --workspace && cargo test --workspace && cargo deny check`
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
cargo check --workspace
```

Dann: `docs/SPEC.md` §0 lesen (LLM-Schnellstart) und `capabilities.toml` für Ring/Klassen-Zuordnung prüfen.

---

## 3. Verifizieren, Rebase & Gate-Integrität (§27.5, §27.6, REQ-QA-010)

- „Fertig" bedeutet ausnahmslos: Alle relevanten Gates, Lints und Tests auf dem Merge-Ergebnis sind grün (REQ-QA-010).
- **Rebase-Pflicht (hochfrequentierte Entwicklung):** Vor jedem PR zwingend `git fetch origin && git rebase origin/main` ausführen. Nach dem Rebase Gates erneut prüfen. Kein PR ohne sauberen Rebase auf main.
- Es ist streng verboten, Gates, Lints, Ratchets oder den Toolchain-Pin einzuschränken, abzuschwächen oder durch Ausnahmen zu umgehen (REQ-QA-010).
- **Geschützte Pfade:** `.github/**`, `justfile`, `rust-toolchain.toml`, `deny.toml`, `capabilities.toml`, `AGENTS.md`, `.agents/jules/setup/**`, `governance/**`.

---

## 4. Die zwölf Regeln für jede Änderung (SPEC §0.2)

> Diese Regeln sind normativ. Verletzungen sind Gate-Fehler.

1. **Ring und Klasse zuerst.** Jede Kante zeigt auf gleichen oder niedrigeren Ring (§4.1 P1/P5); Kern `[K]` hängt nie von Rand `[R]`/Experimentell `[E]`/Archiv `[A]` ab (P30).
2. **Kein Panic in Produktion.** Kein `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!`, `unreachable!`, `assert!`, `process::exit`, ungeprüftes Indexieren (P7). Fehler sind typisiert (`ContextraError`, §13.6).
3. **Zeit, Zufall, IDs nur über Ports** `Clock`, `Rng`, `IdGen` (P28). Verboten in Kernlogik: `Instant::now`, `SystemTime::now`, `thread_rng`, `std::env::var`, `HashMap/HashSet` <!-- doc-ref-ignore --> mit RandomState. Ausnahme: Schlüssel-/Salt-Material aus dem System-RNG.
4. **Alles Persistente geht durch das WAL** (P24). Sicherheitszustand liegt ausschließlich in den fünf Sidecar-Logs (§7.9).
5. **I/O nur über `Vfs`**; Commit = durabler Flush mit plattformgerechter Primitive (§7.7); `fsync`-Fehler werden propagiert, nie mit `let _ =` verworfen.
6. **Unsafe-Inseln** (`contextra-simd`, `contextra-sys`), jeder Block mit `// SAFETY:` (P2).
7. **Mandant kommt aus dem `RequestContext`**, nie aus Literalen, Defaults oder Nutzerparametern (INV-TENANT-1, VETO-F10).
8. **Sicherheit ist kein Feature-Schalter.** Verschlüsselung, Messung und Löschbericht sind in jedem Build aktiv; kein `cfg(not(feature))`-Ersatz für Sicherheitscode (A1).
9. **Messen vor Behaupten.** Ein Restbestand-Zähler entsteht nur aus einem Scanner der besitzenden Schicht; Konstanten oder Aufrufer-Werte sind nie Messung (§12.7, §12.10).
10. **Fail-Closed.** Unbekannter Zustand, unbekannte Formatversion, fehlender Sicherheitsport, gebrochene Kette: Ablehnung mit benanntem Grund, nie Raten (INV-FAILCLOSED-1).
11. **Eine Typfamilie je Konzept.** Löschtypen existieren nur in `crypto`/`ports`; Ports nur in `ports` (A2, §5).
12. **Ein PR = ein Arbeitspaket (WP) = höchstens ein Crate-Schnitt**; nach jedem Schnitt Build, Löschtest, Zeilenzählung (REQ-CHG-2).

---

## 5. Baukasten-Crates — Baukasten-Modell (SPEC §5)

### Reduktion 1.0

Das Repository wurde auf 14 Crates unter `crates/` reduziert (`contextra`, `contextra-crypto`, `contextra-db`, `contextra-engine`, `contextra-mvcc`, `contextra-ports`, `contextra-rank`, `contextra-simd`, `contextra-store`, `contextra-sys`, `contextra-testkit`, `contextra-text`, `contextra-types`, `contextra-vector`).
Entfernte Crates (`graph`, `cognition`, `adapt`, `kvcache`, `router`, `agent`, `infer-*`, `sandbox`, `py`, `license`, `core`, `ingest`, `mcp`, `wire`, `checkpoint`, `privacy`, `audit-export`, `avv-generator`) liegen in der Git-Historie (Tag `snapshot/pre-cut` bzw. Commit `eb10652f`) und werden nicht neu angelegt.
`cargo xtask` wurde entfernt; Validierungen erfolgen ausschließlich über standardisierte Cargo-Befehle (`cargo fmt`, `cargo check`, `cargo clippy`, `cargo test`, `cargo deny`).

### 5.1 Verbliebene 14 Crates

| Crate | Ring | Status | Aufgabe |
|-------|------|--------|---------|
| `contextra-types` | 0 | aktiv | Grundlegende Typen, IDs, Fehlerklassen |
| `contextra-ports` | 0 | aktiv | Port-Schnittstellen (Entkopplung) |
| `contextra-sys` | 0 | aktiv | Unsafe-Insel; Systemnahe Bindings, Low-Level-I/O |
| `contextra-simd` | 0 | aktiv | Unsafe-Insel; SIMD-Distanzkerne |
| `contextra-mvcc` | 0 | aktiv | Snapshots, SSI, Sequenzen |
| `contextra-crypto` | 0 | aktiv | AEAD, KDF, Signaturen |
| `contextra-vector` | 0 | aktiv | HNSW, SQ8, ACL-Bitset |
| `contextra-text` | 0 | aktiv | BM25, Tokenizer |
| `contextra-rank` | 0 | aktiv | RRF-Fusion, Normalisierung |
| `contextra-testkit` | 0 | aktiv | Testkit-Infrastruktur |
| `contextra-store` | 1 | aktiv | WAL, LSM, SSTables, Kompaktion |
| `contextra-engine` | 3 | aktiv | Planner, Collections |
| `contextra-db` | 3 | aktiv | Main DB Abstraktion |
| `contextra` | 4 | aktiv | Composition Root, öffentliche Facade |

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

## 8. Aktueller Implementierungsstatus (IST-Befunde)

> Basierend auf PLAN.md §1 (Kernbefunde) und Commit `a547a42`.
> Aktuelle Validation-Checks laufen auf main via CI.

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
