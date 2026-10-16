# Contextra — Agenten-Betriebsanleitung (AGENTS.md)

Stand: 2026-10-04 (Systemspezifikation v15)

## 1. Rangfolge
Code + grüne Gates > `AGENTS.md` > `docs/spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md` (`docs/spec/CONTEXTRA_SPEZIFIKATION_v15.md`) > sonstige Vorgaben.

## 2. Start und Ende
Session-Lebenszyklus: `cargo xtask jules start|check|verify|submit|stop`.
Start: `cargo xtask jules start --card <datei>` (Arbeitskontext laden; Preflight-Fallback: `cargo run --manifest-path xtask/Cargo.toml -- jules-preflight`).

## 3. Ablauf
1. Task-Karte lesen und Ziel/Scope verstehen.
2. Existenz von APIs, Dateien und Typen im Code prüfen (`capabilities.toml`, `docs/TYPE_REGISTRY.md`).
3. Plan mit Dateiliste und Akzeptanzbefehlen erstellen.
4. Kleinste mögliche Änderung durchführen; Änderungen vor/nach Modifikation verifizieren.
5. Verifizieren (`jules verify`), pre-commit durchführen und einreichen (`jules submit`).

## 4. Gates
| Gate | Befehl | bündelt | Phase |
| :--- | :--- | :--- | :--- |
| `guard` | `cargo xtask gate-guard` <!-- harness:planned --> | Scope- & Protected-Paths-Prüfung | check |
| `symbols` | `cargo xtask symbol-exists` | Existenzprüfung aller Symbol-Pfade | check |
| `arch` | `cargo xtask gate-arch` <!-- harness:planned --> | Ring-Layering, Unsafe-Inseln & DAG-Integrität | check |
| `integrity` | `cargo xtask gate-integrity` <!-- harness:planned --> | Test-Integrity, Ratchet, Diff-Budget & Gate-Weakening | verify |
| `report` | `cargo xtask session-report` | Erstellung des Session-Reports für PR-Dokumentation | submit |

Zusatzprüfungen werden risikobasiert durch `gate-integrity` anhand der `risk`-Einstufung der Task-Karte gewählt. Agenten müssen keine Einzel-Gates manuell aufrufen.
Alle weiteren Qualitäts-Gates aus `governance/gates.toml` laufen automatisiert in CI und Nightly-Runs.
Anti-Gaming: Das Abschwächen, Umgehen oder Deaktivieren von Qualitäts-Gates, Lints, Schwellenwerten, Baselines oder Toolchain-Pins ist streng verboten.

## 5. Scope
- Ausschließlich Dateien im explizit freigegebenen `scope` der Task-Karte bearbeiten.
- Mängel außerhalb des Scopes nicht direkt beheben, sondern im PR-Text unter „Out-of-scope Findings" dokumentieren.

## 6. Invarianten (verbindlich)
| ID | Regel | Prüfung |
| :--- | :--- | :--- |
| P2 | `clippy::unwrap_used/expect_used/panic` sind deny in Produktionscode | `cargo clippy --all-targets` |
| P12 | `capabilities.toml` ist einzige Ring-Quelle | `cargo xtask check-ring-capabilities-consistency` |
| P28 | Determinismus via injizierte Ports (`Clock`, `Rng`, `IdGenerator`); CSPRNG Pflicht für Key/Salt-Material | `cargo xtask determinism-check` |
| INV-TENANT-1 | `TenantId::try_new(0)` -> Err; `0` ist SYSTEM | `cargo test -p contextra-types` |
| INV-DELETION-1/2 | `DeletionProof::create()` nur nach physischer Bereinigung; kein Zeiger auf gelöschte IDs nach `remove_with_graph_repair` | `cargo test -p contextra-crypto` |
| INV-DURABILITY-RING | `MemoryOnly` inkompatibel mit DeletionProof/Sovereign | `cargo test -p contextra` |
| INV-PERF-PROFILE-1 | `BareMetal`/`Balanced` inkompatibel mit Löschbeweis | `cargo test -p contextra` |
| INV-COLLECTION-PROFILE-1/2 | Aktiver Proof erfordert `CryptoShred`; Presets konsistent | `cargo test -p contextra` |
| INV-WAL-LEGACY-KEY-1 | Legacy-HMAC-Fallback nie implizit aktiv | `cargo test -p contextra-store` |
| INV-WAL-TRUNCATION-1 | WAL-Truncation-Schutz über Manifest-High-Water-Mark | `cargo test -p contextra-store` |
| INV-MVCC-SSI-1 | SSI Read-Set Tracking & Write-Skew Validierung aktiv | `cargo test -p contextra-mvcc` |
| INV-EGRESS-AUDIT-1 | Egress-Audit-Trace hash-chained ohne Plaintext-Leakage | `cargo test -p contextra-privacy` |
| INV-MCP-CLASSIFY-1 | Jedes gelistete MCP-Tool explizit klassifiziert | `cargo test -p contextra-mcp` |
| INV-CHECKPOINT-DETERMINISM-1 | Checkpointing ohne direkte Wall-Clock/Random-Aufrufe | `cargo test -p contextra-checkpoint` |
| AGT-GRAPH-001 | CSR-Graph-Aktionen deterministisch und thread-safe | `cargo test -p contextra-graph` |
| INV-VAULT-1/2/3 | Zeroize, Surrogate-Kollisionsschutz, Key-Isolation | `cargo test -p contextra-crypto` |
| VETO-F02/F10/OP03 | HNSW-Pruning ohne Rewire; KV-Scratchpad-Invalidierung nur unter Prefix; Memory-Limits einhalten | `cargo test -p contextra-vector` |
| Snapshot-Isolation | Alle Signale einer Anfrage lesen denselben Sequenzstand (`max_seq`) | `cargo test -p contextra-text` |
| Fail-Closed Default | Fail-closed als Fallback für SSI, Sandbox, Egress-Timeout & Egress-Indexausfall | `cargo test -p contextra-mcp` |

## 7. Architektur-Karte
- `capabilities.toml`: Verbindliches Crate- & Ring-Verzeichnis (Ring 0 Sync-Purity: `cargo xtask check-ring0-async-purity`).
- `docs/TYPE_REGISTRY.md`: Zentrale Dokumentation aller Domänen-Typen.
- `docs/decisions/README.md`: Verzeichnis aller Architektur-Entscheidungen (ADRs) und Veto-Sperren.
- `crates/<name>/AGENTS.md`: Lokale Crate-Betriebsanleitungen und crate-spezifische Invarianten. <!-- doc-ref-ignore -->

## 8. Nicht tun
- Kein `sh -c` für Prozessaufrufe; immer `shlex` Argument-Arrays nutzen.
- Single-Node & Ein-Prozess-Garantie: Keine Multi-Node-, Sharding-, P2P-Sync- oder Cluster-Logik.
- Sperrenhierarchie strikt einhalten: `collections` -> `kv_locks` -> `embedder`; keine Locks über `.await` halten.
- Spec-Sync: Jede API-Änderung muss in `docs/spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md` gespiegelt werden.
- Scope-Ausschlüsse: Kein globales HNSW-Rebuilding, keine Cross-Tenant-Flüsse, keine Realtime-Audio/Voice-Features, keine Veto-Bypasses ohne ADR.
