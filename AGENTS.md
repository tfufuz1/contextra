# Contextra — Agenten-Betriebsanleitung (AGENTS.md)

Stand: 2026-10-06 · Gilt für alle Agenten (Jules, Claude Code, Codex, manuelle Sessions)

## 1. Rangfolge

1. Grüne Gates und Code-Realität
2. `AGENTS.md` (diese Datei)
3. `docs/spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md` (bzw. `docs/spec/CONTEXTRA_SPEZIFIKATION_v15.md`)
4. Übrige Vorgaben in `docs/`

## 2. Werkzeug-Regel (verbindlich)

**`cargo xtask` ist die einzige Quelle der Wahrheit für Analyse, Prüfung und Diagnose.**

- Keine Python-, Shell- oder Ad-hoc-Skripte zur Repo-Analyse schreiben oder ausführen.
- Kein `grep` als Ersatz für einen vorhandenen Befehl. `grep` ist nur für gezielte Lektüre erlaubt.
- Kein `sh -c`. Prozessaufrufe immer als Argument-Array.
- Fehlt ein Werkzeug, wird es als xtask-Befehl implementiert (siehe Abschnitt 9), nicht als Skript.
- Befehl unbekannt? `cargo xtask harness-list` aufrufen — dies ist die verbindliche Befehlswahrheit.

## 3. Session-Lebenszyklus

```bash
cargo xtask jules start --card <datei>    # Arbeitskontext laden; Preflight-Validierung
cargo xtask jules check                   # Gates Phase check (guard + symbols + arch)
cargo xtask jules verify                  # Gates Phase verify (integrity + ratchet + diff-budget)
cargo xtask jules submit                  # Gates Phase submit: Abschluss-Prüfung + Session-Report
cargo xtask jules stop                    # Session beenden
```

Preflight ohne Session-Karte: `cargo xtask jules-preflight`.
Direkte Gate-Befehle (für manuelle Nutzung): `cargo xtask gate-guard`, `cargo xtask gate-arch`, `cargo xtask gate-integrity`, `cargo xtask session-report`.

## 4. Befehlsübersicht

**Status-Legende:** ✅ vorhanden · 🟡 vorhanden, aber nicht vollständig · 🔲 geplant (noch nicht implementiert)

Geplante Befehle sind mit `<!-- harness:planned -->` markiert. Ein Agent darf sie nicht aufrufen und nicht als vorhanden behandeln.
**Verbindliche Wahrheit**: `cargo xtask harness-list` — immer zuerst prüfen.

### 4.1 Tier 0 — schnell, ohne Domain-Build

| Befehl | Status | Zweck |
|---|---|---|
| `cargo xtask harness-list [--json]` | ✅ | Alle registrierten Harness-Befehle auflisten |
| `cargo xtask harness-help <name>` | ✅ | Hilfe zu einem einzelnen Harness-Befehl |
| `cargo xtask fast-diff-lint` | ✅ | Build-freier Diff-Scope-Linter für häufige Doktrin-Verstöße |
| `cargo xtask repo-doctor` | ✅ | Session-Start-Dashboard: Git, Workspace, Unsafe/Panic, Branch |
| `cargo xtask env-validate` | ✅ | Toolchain- und Umgebungsvalidierung |
| `cargo xtask env-attest` | ✅ | Environment Attestation: Toolchain-Pinning und Build-Tools |
| `cargo xtask workspace-verify` | ✅ | Workspace-Vollständigkeit und Konsistenz |
| `cargo xtask registry-check` | ✅ | Vollständigkeit und Konsistenz aller xtask-Befehle gegen registry.toml |
| `cargo xtask explain <gate>` | ✅ | Qualitätsgates und Invarianten erklären |
| `cargo xtask search <stichwort>` | 🔲 `<!-- harness:planned -->` | Befehle nach Name/Docstring suchen |

### 4.2 Tier 1 — Workspace-Scan, ohne Domain-Build

| Befehl | Status | Zweck | Exit |
|---|---|---|---|
| `cargo xtask orphan-symbols [--json]` | ✅ | Öffentliche Symbole ohne Verwendung (`ORPHAN`, `TEST_ONLY`, `SUSPECT`) | 2 bei `ORPHAN` |
| `cargo xtask unwired-ports [--only-unwired]` | ✅ | Traits ohne Implementierung (`UNWIRED`, `MOCK_ONLY`, `SOLO`) | 2 bei `UNWIRED` |
| `cargo xtask stub-impls [--min-severity hoch]` | ✅ | Stub-Implementierungen (`HOCH`, `MITTEL`, `NIEDRIG`) | 2 bei `HOCH` |
| `cargo xtask doctrine-scan [--crate <name>] [--json]` | ✅ | Zero-Panic-Verstöße, Unsafe-Insel-Verstöße, `#[allow]`-Durchbrechungen | 2 bei Verstoß |
| `cargo xtask dependency-graph-audit [--json]` | ✅ | DAG-Invariante (P5), Ring-0-`tokio` (P26), Zyklen | 2 bei Verstoß |
| `cargo xtask check-ring-layering` | ✅ | Ring-Layering (Kurzfassung) | 2 bei Verstoß |
| `cargo xtask check-ring-layering-full` | ✅ | Ring-Layering (vollständige Analyse) | 2 bei Verstoß |
| `cargo xtask check-ring0-async-purity` | ✅ | Ring-0-Async-Reinheit (P26) | 2 bei Verstoß |
| `cargo xtask check-unsafe-islands` | ✅ | Unsafe nur in deklarierten Inseln | 2 bei Verstoß |
| `cargo xtask check-ring-capabilities-consistency` | ✅ | `capabilities.toml` als einzige Ring-Quelle (P12) | 2 bei Verstoß |
| `cargo xtask determinism-check` | ✅ | Determinismus via Ports (P28) | 2 bei Verstoß |
| `cargo xtask check-duplicate-symbols` | ✅ | Doppelte Funktions-/Struktursymbole über Dateigrenzen | 2 bei Verstoß |
| `cargo xtask check-result-dropped-io` | ✅ | Ignorierte I/O-Fehler-Results (INV-DURABILITY-RING) | 2 bei Verstoß |
| `cargo xtask check-nan-hot-loop` | ✅ | NaN-Validierung in Vektorberechnungs-Hotpaths | 2 bei Verstoß |
| `cargo xtask symbol-exists <pfad>` | ✅ | Einzelnes Symbol-Pfad-Existenz prüfen | 2 bei fehlendem Symbol |
| `cargo xtask claim` | ✅ | Claim-Integrität für Agent-Gates | 2 bei Verstoß |

Hinweis: Die Scan-Befehle nutzen den gemeinsamen `WorkspaceIndex` (geplant in `xtask/src/workspace_index.rs`). Bis dieser vollständig ist, arbeiten einige 🟡-Befehle mit eigener Logik. Ihre Ergebnisse sind trotzdem verbindlich.

### 4.3 Tier 2 — Build-Abhängigkeit

| Befehl | Status | Zweck |
|---|---|---|
| `cargo xtask check-compile` | ✅ | Workspace kompiliert fehlerfrei |
| `cargo xtask check-dag` | ✅ | DAG-Prüfung über Cargo-Graph |
| `cargo xtask bench-gate --tolerance 0.05` | ✅ | Benchmark-Gate (`xtask-heavy`) |
| `cargo xtask bench-trend` | ✅ | Benchmark-Trendanalyse |
| `cargo xtask feature-matrix` | ✅ | Feature-Kombinationen prüfen |
| `cargo xtask loom-run` | ✅ | Concurrency-Tests mit loom |
| `cargo xtask panic-inventory` | ✅ | Vollständiges Panic-Inventar |
| `cargo xtask security-scan` | ✅ | Sicherheitsscan (audit, Advisories) |
| `cargo xtask audit-integrity-check` | ✅ | Audit-Ketten-Integrität |
| `cargo xtask check-fbs-drift` | ✅ | FlatBuffers-Schema-Drift-Erkennung |
| `cargo xtask reproducible-build-check` | ✅ | Reproduzierbarkeit des Builds |
| `cargo xtask fuzz-smoke` | ✅ | Fuzz-Smoke über berührte Crates mit Fuzz-Targets |
| `cargo xtask mutants-diff` | ✅ | Diff-scopierter Mutation-Testing-Score |
| `cargo xtask wal-replay-verify` | ✅ | WAL-Replay Crash-Konsistenz-Verifikation (INV-WAL-*) |
| `cargo xtask blast-radius` | ✅ | Auswirkungsbereich geänderter Crates berechnen |
| `cargo xtask risk` | ✅ | Risikobewertung der Änderungen |
| `cargo xtask can-merge` | ✅ | Merge-Entscheidung (aggregiertes Urteil) |

### 4.4 Gates (Jules-Lebenszyklus)

| Gate | Befehl | Bündelt | Phase |
|---|---|---|---|
| `guard` | `cargo xtask gate-guard` | Scope- und Protected-Paths-Prüfung | check |
| `symbols` | `cargo xtask symbol-exists` | Existenz aller Symbol-Pfade in der Task-Karte | check |
| `arch` | `cargo xtask gate-arch` | Ring-Layering, Unsafe-Inseln, DAG | check |
| `integrity` | `cargo xtask gate-integrity` | Test-Integrität, Ratchet, Diff-Budget, Gate-Weakening | verify |
| `report` | `cargo xtask session-report` | Session-Report für PR | submit |

Zusatzprüfungen wählt `gate-integrity` risikobasiert anhand der `risk`-Einstufung der Task-Karte.
**Agenten rufen Einzel-Gates nie manuell auf** — ausschließlich über den Lebenszyklus (Abschnitt 3).

Weitere Qualitäts-Gates aus `governance/gates.toml` laufen automatisiert in CI und Nightly-Runs.

### 4.5 Dokumentation und Governance

| Befehl | Status | Zweck |
|---|---|---|
| `cargo xtask sync-docs [--check]` | ✅ | Generierte Doku synchron halten |
| `cargo xtask check-type-registry` | ✅ | `docs/TYPE_REGISTRY.md` vollständig |
| `cargo xtask check-vetoes` | ✅ | Veto-Sperren |
| `cargo xtask check-adr-deadlines` | ✅ | ADR-Fristen |
| `cargo xtask check-veto-deadlines` | ✅ | Veto-Fristen |
| `cargo xtask check-agents-integrity` | ✅ | Diese Datei ist konsistent |
| `cargo xtask check-agents-freshness` | ✅ | Diese Datei ist aktuell |
| `cargo xtask check-doc-references` | ✅ | Querverweise in Doku gültig |
| `cargo xtask check-workflow-commands` | ✅ | Workflow-Befehle existieren |
| `cargo xtask check-stale-tags` | ✅ | Veraltete Tags erkennen |
| `cargo xtask check-toc-integrity` | ✅ | Inhaltsverzeichnis-Integrität |
| `cargo xtask consolidate-adrs` | ✅ | ADRs zusammenführen |
| `cargo xtask gen-arch-docs` | ✅ | Architektur-Dokumentation generieren |
| `cargo xtask context-pack` | ✅ | Agent-Kontext-Paket erstellen |
| `cargo xtask crate-context` | ✅ | Crate-Kontext-Report |

### 4.6 Agent-Kontext (Lesen vor Beginn)

```bash
cargo xtask crate-context <crate-name>    # Kontext eines Crates laden
cargo xtask context-pack                  # Vollständiges Kontext-Paket
cargo xtask check-jules-context-freshness # Kontext noch aktuell?
```

## 5. Vor jeder Änderung

1. Task-Karte lesen, Ziel und Scope verstehen.
2. Existenz prüfen: `capabilities.toml`, `docs/TYPE_REGISTRY.md`, `cargo xtask symbol-exists <pfad>`.
3. Verfügbare Befehle prüfen: `cargo xtask harness-list` — **niemals** Befehle aus Abschnitt 4 nutzen ohne vorherige Bestätigung.
4. Plan mit Dateiliste und Akzeptanzbefehlen erstellen. Akzeptanzbefehle **nur** aus Abschnitt 4 mit Status ✅ oder 🟡.
5. Kleinste mögliche Änderung. Vor und nach der Änderung mit xtask verifizieren.

## 6. Scope

- Nur Dateien im `scope` der Task-Karte ändern.
- Befunde außerhalb des Scopes nicht direkt beheben. Im PR unter „Out-of-scope Findings" dokumentieren.

## 7. Invarianten (verbindlich)

| ID | Regel | Prüfbefehl |
|---|---|---|
| P2 | `unwrap_used`, `expect_used`, `panic` sind `deny` in Produktionscode | `cargo clippy --all-targets` · `cargo xtask doctrine-scan` |
| P5 | DAG: Abhängigkeiten nur zu niedrigeren Ringen | `cargo xtask dependency-graph-audit` |
| P12 | `capabilities.toml` ist einzige Ring-Quelle | `cargo xtask check-ring-capabilities-consistency` |
| P26 | Ring 0 bindet `tokio` nicht als Produktions-Abhängigkeit ein | `cargo xtask check-ring0-async-purity` |
| P28 | Determinismus via Ports (`Clock`, `Rng`, `IdGenerator`); CSPRNG für Key- und Salt-Material | `cargo xtask determinism-check` |
| INV-TENANT-1 | `TenantId::try_new(0)` → Err; `0` ist SYSTEM | `cargo test -p contextra-types` |
| INV-DELETION-1/2 | `DeletionProof::create()` nur nach physischer Bereinigung; kein Zeiger auf gelöschte IDs | `cargo test -p contextra-crypto` |
| INV-DURABILITY-RING | `MemoryOnly` inkompatibel mit DeletionProof und Sovereign | `cargo test -p contextra` |
| INV-PERF-PROFILE-1 | `BareMetal` und `Balanced` inkompatibel mit Löschbeweis | `cargo test -p contextra` |
| INV-COLLECTION-PROFILE-1/2 | Aktiver Proof erfordert `CryptoShred`; Presets konsistent | `cargo test -p contextra` |
| INV-WAL-LEGACY-KEY-1 | Legacy-HMAC-Fallback nie implizit aktiv | `cargo test -p contextra-store` |
| INV-WAL-TRUNCATION-1 | WAL-Truncation-Schutz über Manifest-High-Water-Mark | `cargo test -p contextra-store` |
| INV-MVCC-SSI-1 | SSI Read-Set-Tracking und Write-Skew-Validierung aktiv | `cargo test -p contextra-mvcc` |
| INV-EGRESS-AUDIT-1 | Egress-Audit-Trace hash-verkettet, ohne Plaintext | `cargo test -p contextra-privacy` |
| INV-MCP-CLASSIFY-1 | Jedes gelistete MCP-Tool explizit klassifiziert | `cargo test -p contextra-mcp` |
| INV-CHECKPOINT-DETERMINISM-1 | Checkpointing ohne direkte Wall-Clock- oder Random-Aufrufe | `cargo test -p contextra-checkpoint` |
| AGT-GRAPH-001 | CSR-Graph-Aktionen deterministisch und thread-safe | `cargo test -p contextra-graph` |
| INV-VAULT-1/2/3 | Zeroize, Surrogate-Kollisionsschutz, Key-Isolation | `cargo test -p contextra-crypto` |
| VETO-F02/F10/OP03 | HNSW-Pruning ohne Rewire; KV-Scratchpad-Invalidierung nur unter Prefix; Memory-Limits | `cargo test -p contextra-vector` |
| Snapshot-Isolation | Alle Signale einer Anfrage lesen denselben Sequenzstand (`max_seq`) | `cargo test -p contextra-text` |
| Fail-Closed | Fail-closed für SSI, Sandbox, Egress-Timeout und Egress-Indexausfall | `cargo test -p contextra-mcp` |

## 8. Architektur-Karte

- `capabilities.toml`: verbindliches Crate- und Ring-Verzeichnis, einzige Quelle für Ringe, Composition-Root-Menge und Unsafe-Inseln.
- `docs/TYPE_REGISTRY.md`: alle Domänen-Typen.
- `docs/decisions/README.md`: ADRs und Veto-Sperren.
- `xtask/registry.toml`: alle Harness-Befehle mit `owner_tier`, `phase` und `summary` — **verbindliche Befehlswahrheit**.
- `crates/<name>/AGENTS.md`: crate-spezifische Betriebsanleitungen und Invarianten. <!-- doc-ref-ignore -->

## 9. Werkzeuge erweitern

Ein neues Werkzeug entsteht als xtask-Befehl. Kein Skript.

1. Modul in `xtask/src/harness/<name>.rs` mit `//!`-Docstring (Zweck, Tier, Exit-Codes).
2. Eintrag in `xtask/registry.toml` mit `owner_tier`, `summary`, `phase`.
3. Tests in `xtask/tests/<name>.rs` mit Fixtures für Befund und Nicht-Befund.
4. Befund-Kategorien: Muss-Fehler (Exit 2) und Hinweis (Exit 0).
5. `--json` unterstützen.
6. Befehl in Abschnitt 4 dieser Datei eintragen — zuerst als `🔲 <!-- harness:planned -->`, nach Merge und Eintrag in `registry.toml` als `✅`.

Verbindliche Regeln für Werkzeuge:
- Tier 0 und Tier 1 dürfen keine Domain-Crate als Cargo-Dependency haben.
- Rust-Analyse nutzt `syn`, nicht Regex, sofern nicht explizit begründet.
- Test-Code wird strukturell ausgeschlossen (über den WorkspaceIndex), nicht über Textschnitt.
- Kein `sh -c`. Prozessaufrufe als Argument-Array (via `shlex`).

## 10. Nicht tun

- Kein Python-, Shell- oder Ad-hoc-Analyseskript im Repository.
- Keine Gate-, Lint-, Schwellen-, Baseline- oder Toolchain-Pin-Abschwächung.
- Keine neuen `#[allow(...)]` für Sicherheits-Lints ohne expliziten ADR.
- Keine Verwendung von `sh -c`.
- Keine Single-Node-Verletzung: keine Multi-Node-, Sharding-, P2P-Sync- oder Cluster-Logik.
- Sperrenhierarchie einhalten: `collections` → `kv_locks` → `embedder`. Keine Locks über `.await`.
- Jede API-Änderung in `docs/spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md` spiegeln.
- Scope-Ausschlüsse: kein globales HNSW-Rebuilding, keine Cross-Tenant-Flüsse, keine Realtime-Audio- oder Voice-Features, keine Veto-Bypasses ohne ADR.
- Keine Analyse-Befehle erfinden. Nur Befehle aus Abschnitt 4 mit Status ✅ oder 🟡 ausführen — und vorher mit `cargo xtask harness-list` bestätigen.
