# Contextra — Agenten-Betriebsanleitung (AGENTS.md)

Stand: 14.10.2026 · Gilt für alle Agenten (Jules, Claude Code, Codex, manuelle Sessions)
**Ziel-Spezifikation (Single Source of Truth):** [`docs/SOLL_ZUSTAND_SPEZIFIKATION_1.0.md`](docs/SOLL_ZUSTAND_SPEZIFIKATION_1.0.md)

---

## 1. Rangfolge bei Konflikten (§0.2)

1. Harte Governance-Dateien des Repos (`CONSTITUTION.md`, `AGENTS.md`, `VETOES.md`). Widersprüche erfordern ein vorab beschlossenes ADR (§13).
2. Endprodukt-Spezifikation (SOLL-Zustand): [`docs/SOLL_ZUSTAND_SPEZIFIKATION_1.0.md`](docs/SOLL_ZUSTAND_SPEZIFIKATION_1.0.md).
3. Code (IST).
4. `capabilities.toml` (maschinenlesbare Ring-Tabelle, synchronisiert per Gate G-04).

---

## 2. Verbindliche Leitregeln für Agenten (REG-01 bis REG-17)

| ID | Regel |
|---|---|
| **REG-01** | **Zero-Panic.** Kein `unwrap`, `expect`, `panic!`, `unreachable!` oder unchecked Indexing im Produktivcode von Ring 0 und Ring 1. Test-, Bench- und Fuzz-Code ist ausgenommen. Die Ratsche `governance/ratchet.toml:unwrap_exact` ist das einzige Gate und darf nur sinken. |
| **REG-02** | **Ring-DAG.** Abhängigkeiten laufen nur von höheren auf niedrigere Ringe (gleicher Ring nur laut Ordnung §4.2). `capabilities.toml` ist die Kopie der Ring-Tabelle. |
| **REG-03** | **Determinismus über Ports.** Zeit, Zufall und IDs kommen ausschließlich über `Clock`, `Rng`, `IdGenerator` aus `contextra-ports`. Direkte Aufrufe von `Instant::now`, `SystemTime::now`, `thread_rng`, `rand::random` sind außerhalb der Port-Implementierungen und außerhalb von Tests/Benches verboten (Gate G-09). |
| **REG-04** | **Drei Unsafe-Inseln:** `contextra-sys`, `contextra-simd`, `contextra-wire`. Kein `unsafe` anderswo. Jedes `unsafe`-Fragment hat einen `// SAFETY:`-Kommentar. Für generierten Code in `wire` gilt eine explizite, dokumentierte Ausnahme (Datei-Header + Generator-Test). Alle übrigen Crates tragen `#-[#![forbid(unsafe_code)]`. |
| **REG-05** | **Single-Node.** Keine Multi-Node-, Sharding-, Raft- oder Cluster-Logik. „Leader" bedeutet ausschließlich den Group-Commit-Leader im Store. Es gibt keine Replikas. |
| **REG-06** | **Wirksamkeit.** Kein `pub`-Symbol ohne Produktionsaufrufer und ohne End-to-End-Test über die Fassade. Entweder verdrahten, auf `pub(crate)` reduzieren oder löschen. |
| **REG-07** | **Opt-in-Kosten.** Nicht aktivierte Features verursachen weder Code im Binary noch CPU-Kosten. Jedes Feature kompiliert isoliert (Gate G-06). |
| **REG-08** | **Fail-closed.** Fehler, Unbekanntes und Unvollständiges führen zu Ablehnung, nie zu stillem Erfolg. Das gilt für Scans, Resume, Lizenz, Mandantenbindung, Speicherplatzprobe, Manifest-Prüfung. |
| **REG-09** | **Builder-Ehrlichkeit.** Jeder Builder-Schalter hat eine nachweisbare Wirkung. `build()` validiert die Verdrahtung gegen den tatsächlichen Zustand, nie gegen hartcodierte Werte. |
| **REG-10** | **Snapshot-Konsistenz.** Alle Signale einer Anfrage lesen denselben Sequenzstand über ein `ReadSnapshot`-Token. |
| **REG-11** | **Sperrenordnung.** Die globale Lock-Reihenfolge (§7.1) gilt crate-übergreifend. Synchrone Locks werden nie über `.await` gehalten. |
| **REG-12** | **Tool-Klassifikation.** Jedes MCP-Tool ist klassifiziert und feature-gated. Ein Tool ohne Klassifikation wird nicht registriert. |
| **REG-13** | **Erasure-Opt-in und Ports-Only.** Kern-Crates importieren nie Erasure-Plugin- oder Evidence-Crates. Sie sprechen nur Ports aus `contextra-ports`. |
| **REG-14** | **Fehlerfreie Evidenz.** Ein Zeuge oder Beweis entsteht nur aus einem real ausgeführten Scan. Es gibt keine Konstanten-, Default- oder `None`-Zeugen. Fehler ergeben keinen Beweis. |
| **REG-15** | **Tenant-Matrix.** Jeder Löschtest läuft über `LsmStorage` **und** über `TenantScopedStorage`. |
| **REG-16** | **Ehrliche Claims.** Dokumentation, Fehlermeldungen und API-Namen behaupten nie mehr als technisch bewiesen ist (§5.14). |
| **REG-17** | **Formate nie downgraden.** Jedes persistente Format ist versioniert. Öffnen upgradet, ein Downgrade wird abgelehnt. |

---

## 3. Werkzeug-Regel (verbindlich)

**`cargo xtask` ist die einzige Quelle der Wahrheit für Analyse, Prüfung und Diagnose.**
- Keine Python-, Shell- oder Ad-hoc-Skripte zur Repo-Analyse schreiben oder ausführen.
- Kein `grep` als Ersatz für einen vorhandenen Befehl. `grep` ist nur für gezielte Lektüre erlaubt.
- Kein `sh -c`. Prozessaufrufe immer als Argument-Array.
- Fehlt ein Werkzeug, wird es als xtask-Befehl implementiert, nicht als Skript.
- Befehl unbekannt? `cargo xtask harness-list` aufrufen — dies ist die verbindliche Befehlswahrheit.

---

## 4. Zielarchitektur & Ring-Modell (§4.1)

| Crate | Ring | Unsafe-Insel | Produkt | Rolle |
|---|---|---|---|---|
| `contextra-types` | 0 | nein | EP1 | Basistypen, IDs, `ContextraError`, `DeterministicTimestamp` |
| `contextra-ports` | 0 | nein | EP1 | Alle Port-Traits |
| `contextra-wire` | 0 | **ja** (generiert) | EP1 | FlatBuffers Wire-Format |
| `contextra-sys` | 0 | **ja** | EP1 | OS-Schnittstellen, SystemClock/Rng, AlignedBuf |
| `contextra-durable-fs` | 0 | nein | EP1 | fsync-Disziplin, `scrub_and_remove`, sync_dir |
| `contextra-simd` | 0 | **ja** | EP1 | SIMD-Kernel (AVX2, NEON, Scalar) |
| `contextra-crypto-core` | 0 | nein | EP1 | KeyManager, KDF, AEAD, HMAC, SecureBuf |
| `contextra-mvcc` | 0 | nein | EP1 | Snapshots, SSI-Validator |
| `contextra-text` | 0 | nein | EP1 | BM25, invertierter Index |
| `contextra-rank` | 0 | nein | EP1 | RRF-Fusion, Conformal Calibration, DiBud |
| `contextra-vector` | 0 | nein | EP1 | HNSW, Quantisierung |
| `contextra-adapt` | 0 | nein | EP5 | PID, Sketched Bandit |
| `contextra-graph` | 0 | nein | EP3 | CSR-Graph, PPR, PathRAG |
| `contextra-crypto-kv` | 0 | nein | EP1 (Feature) | KV-Segmentverschlüsselung, Revocation-Log |
| `contextra-proof-verify` | 0 | nein | EP2 | Unabhängiger DeletionProof-Verifier |
| `contextra-audit-export`, `contextra-avv-generator` | 0 | nein | EP2 | Export und AVV-Generator |
| `contextra-store` | 1 | nein | EP1 | LSM, WAL, Manifest, Ledger, Purge v2, Fence |
| `contextra-checkpoint` | 1 | nein | EP1 | Checkpoints |
| `contextra-kvcache` | 1 | nein (`forbid(unsafe_code)`) | EP4 | KV-Cache (Irminsul, KIVI 2-Bit) |
| `contextra-infer-candle`, `-ollama`, `-onnx` | 2 | nein | EP4 | Inferenz-Backends |
| `contextra-sandbox` | 2 | nein | EP6 | WASI-Merge-Operatoren |
| `contextra-privacy` | 3 | nein | EP6 | DLP, Egress-Filter |
| `contextra-engine` | 3 | nein | EP1 | Collections, Hybridsuche, Löschpfad |
| `contextra-db` | 3 | nein | EP1 | Mehrmandanten-API über Engine |
| `contextra-erasure-evidence` | 3 | nein | EP2 | Beweisausstellung, Policy, Zeugen |
| `contextra-cognition`, `-router`, `-agent` | 3 | nein | EP5 | Kognition, Routing, Agent |
| `contextra` | 4 | nein | EP1 | Fassade und Composition Root |
| `contextra-mcp` | 4 | nein | EP1 | MCP-Server (stdio) |
| `contextra-py` | 4 | nein | EP1/EP5 | Python-Bindings |
| `contextra-license` | 4 | nein | EP6 | Lizenzprüfung |
| `contextra-testkit`, Bench, Fuzz, `xtask*` | Tooling | nein | — | Testkit, Harness |

### Ordnung innerhalb eines Rings (§4.2)
```
Ring 0:  types < ports < wire < sys < durable-fs < simd < crypto-core < mvcc < text < rank
              < vector < adapt < graph < crypto-kv < proof-verify < audit-export < avv-generator
Ring 1:  store < checkpoint < kvcache
Ring 2:  sandbox, infer-candle, infer-ollama, infer-onnx   (untereinander unabhängig)
Ring 3:  privacy < engine < erasure-evidence < db < cognition < router < agent
Ring 4:  license < contextra < mcp, py
```

---

## 5. Endprodukte (EP1 bis EP6) & EP1 Hüllen-Ziel (§4.4)

EP1 **Contextra Memory** erlaubt im Default-Build maximal 18 Crates (Ziel 17 Crates) in `cargo tree -p contextra-mcp -e normal --no-default-features --features fast`.
Verbotene Crates in EP1: `contextra-core`, `contextra-crypto`, `contextra-graph`, `contextra-cognition`, `contextra-adapt`, `contextra-router`, `contextra-agent`, `contextra-privacy`, `contextra-sandbox`, `contextra-license`, `contextra-kvcache`, `contextra-infer-*`, `contextra-erasure-evidence`, `contextra-crypto-kv`, `contextra-proof-verify`.

---

## 6. Session-Lebenszyklus

```bash
cargo xtask jules start --card <datei>    # Arbeitskontext laden; Preflight-Validierung
cargo xtask jules check                   # Gates Phase check (guard + symbols + arch)
cargo xtask jules verify                  # Gates Phase verify (integrity + ratchet + diff-budget)
cargo xtask jules submit                  # Gates Phase submit: Abschluss-Prüfung + Session-Report
cargo xtask jules stop                    # Session beenden
```

---

## 7. Invarianten & Prüfbefehle

- Zero-Panic (P2): `cargo xtask doctrine-scan`
- Ring-Architektur (P5/P12): `cargo xtask gate-arch`
- Determinismus via Ports (P28): `cargo xtask determinism-check`
- Unsafe nur in sys/simd/wire: `cargo xtask check-unsafe-islands`
- Full Workspace Verification: `cargo xtask verify`
