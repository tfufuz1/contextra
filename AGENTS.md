# Contextra — Betriebsanleitung für Autonome AI-Agenten (AGENTS.md)

---

## 1. Zweck
Diese `AGENTS.md` ist die maßgebliche, operative Betriebsanleitung für autonome Coding-Agenten (insbesondere Google-Jules) im **Contextra Cognitive OS** Repository. Bei Konflikten zwischen Prompt-Texten, Spezifikationen und Code gilt stets: **Code-Befund > `AGENTS.md` > Spezifikationen**.

---

## 2. Crate-Claiming & Session-Zustand (Single-Agent-Modus)
Im Standardbetrieb läuft das Contextra Cognitive OS im **Single-Agent-Modus** (`CONTEXTRA_SINGLE_AGENT_MODE=1`, Default).
* **Crate-Claiming:** Der bisher verpflichtende Claim-Schritt via `cargo xtask claim --crate <crate-name>` entfällt im Single-Agent-Modus. Der Session-Zustand wird stattdessen in `.jules/SESSION.md` gehalten.
* **Multi-Agent-Reversibilität & `check-duplicate-intent`:** Der Claim-Mechanismus und das `check-duplicate-intent`-Gate bleiben inaktiv bzw. optional erhalten und können jederzeit reaktiviert werden, falls künftig wieder mehrere Agenten parallel an unterschiedlichen Crates arbeiten.

---

## 3. Pflicht-Workflow
Jeder Agent befolgt strikt den iterativen 5-Phasen-Workflow:
1. **Phase 1: Exploration & Session:** Session-Zustand in `.jules/SESSION.md` prüfen/führen (Claim-Schritt entfällt im Single-Agent-Modus `CONTEXTRA_SINGLE_AGENT_MODE=1`), Workspace-Status via `git status`, `read_file` und `bash` erforschen.
2. **Phase 2: Plan & Review:** Gliederung/Plan verfassen, `set_plan` setzen, Review via `request_plan_review` einholen.
3. **Phase 3: Act:** Code/Dokumentation präzise und ununterbrochen bearbeiten, dabei ausschließlich den erlaubten Scope anfassen.
4. **Phase 4: Verify:** Qualitätssicherung durchführen (siehe Verify-Pflichtbefehle unten).
5. **Phase 5: Reflect & Submit:** Git-Diff prüfen (`git diff --stat`), Commits ggf. konsolidieren (`git commit --amend`), Pre-Commit-Steps durchführen und via `submit`-Tool einreichen.

---

## 4. Scope-Disziplin
* **Strikter Scope:** Jeder Agent darf AUSSCHLIESSLICH Dateien bearbeiten, die in seinem spezifischen Arbeitsauftrag (Task Scope) explizit freigegeben sind.
* **Umgang mit Nebenbefunden:** Werden während der Arbeit Fehler oder Mängel außerhalb des eigenen Scopes entdeckt (z. B. fehlerhafte Nachbar-Crates, veraltete Doku), dürfen diese **NICHT selbständig behoben** werden. Sie sind stattdessen im Abschlusskommentar / PR-Review-Notes als *Out-of-Scope Findings* präzise zu dokumentieren.

---

## 5. Verify-Pflichtbefehle
Vor jedem Commit und Submit müssen folgende Verifikationsschritte in der Sandbox ausgeführt werden:
* `just check` — Formatierung, Clippy-Lints und Grundkompilation aller Workspace-Crates prüfen.
* `just test -p <crate>` — Unit- und Integrationstests für das bearbeitete Crate ausführen.
* `just dag-check` — Richtungs- und Schichtenintegrität des Ring-0–4-Modells verifizieren (keine verbotenen Upward- oder Cross-Ring-Abhängigkeiten).
* `cargo xtask jules-preflight` — Umfassendes Preflight-Gate: prüft Unerreichbare Module (`check-orphan-modules`), Unsafe-Inseln (`check-unsafe-islands`), Intent-Duplikate (`check-duplicate-intent`), AGENTS.md-Integrität (`check-agents-integrity`) und Dokumentations-Frische.

---

## 6. Architektur-Invarianten (Kurzreferenz)
Detaillierte Spezifikationen siehe [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md).
* **Zero-Panic-Doctrine:** Kein `unwrap()`, `expect()` oder `panic!()` in Produktionspfaden. Fehler werden über `Result<T, ContextraError>` propagiert.
* **Security & Shell Execution:** Prozessaufrufe dürfen niemals über Shell-Wrapper (`sh -c`) erfolgen; Parameter MUSS `shlex`-geparsed und als explizites Argumenten-Array übergeben werden.
* **Zero-Copy & SIMD Alignment:** Zero-Copy Read-Pfade nutzen `bytes::Bytes`. SIMD-Puffer (`contextra-simd`) erfordern 16/32-Byte-Ausrichtung; Bounds-Checks vor Unsafe-SIMD-Loads sind zwingend.
* **Locking-Disziplin:** Hierarchie beachten (z. B. `NodesGuard` vor `ConsolidationNodesGuard`), um Deadlocks zu verhindern. Mutex/RwLock-Guards dürfen nicht über `.await`-Punkte gehalten werden.
* **Ring-Modell & DAG-Regel:** Abweichungsfreie Einhaltung der Ring-0–4-Schichten. Kerne derselben Schicht dürfen sich nicht untereinander referenzieren.
* **Unsafe-Inseln:** `unsafe` Code ist streng isoliert auf `contextra-simd`, `contextra-sys` und `contextra-wire`. Alle anderen Crates erzwingen `#![forbid(unsafe_code)]`.

---

## 7. Crate-Topologie (Ring-0–4-Modell)

| Crate | Ring | Status | Kurzbeschreibung |
|---|---|---|---|
| `contextra-wire` | Ring 0 | ✅ vorhanden | FlatBuffers-Generat (`contextra.fbs`) + IPC-Adapter (Unsafe-Insel) |
| `contextra-sys` | Ring 0 | ✅ vorhanden | Unsafe-Insel: `ReadOnlyMap` (mmap), Win32-ACL |
| `contextra-simd` | Ring 0 | ✅ vorhanden | Unsafe-Insel: SIMD-Distanzkernel, Laufzeit-Dispatch |
| `contextra-crypto` | Ring 0 | ✅ vorhanden | Package `contextra-crypto`: Encryption-at-Rest, Zeroize |
| `contextra-text` | Ring 0 | ✅ vorhanden | BM25 Volltextsuche, deutsche Morphologie & Komposita |
| `contextra-graph` | Ring 0 | ✅ vorhanden | CSR-Graph, PPR, Leiden, Hyperkanten |
| `contextra-adapt` | Ring 0 | ✅ vorhanden | LinUCB-Bandit, Lyapunov, PID Controller |
| `contextra-store` | Ring 1 | ✅ vorhanden | LSM-Tree, WAL (Group Commit, HMAC), MVCC-Pin |
| `contextra-kvcache` | Ring 1 | ✅ vorhanden | In-Memory LRU-Cache, Eviction-Worker, Tenant-Isolation |
| `contextra-checkpoint` | Ring 1 | ✅ vorhanden | RAII-Checkpoint & Persistent Store Management |
| `contextra-sandbox` | Ring 2 | ✅ vorhanden | WASM Execution Boundary, Fuel + Wall-Clock Budgets |
| `contextra-router` | Ring 3 | ✅ vorhanden | SLM-Profil-Routing, MCP-Dispatch |
| `contextra-agent` | Ring 3 | ✅ vorhanden | Multi-Step Persistent Agent Workflow Loop |
| `contextra-mcp` | Ring 4 | ✅ vorhanden | Model Context Protocol stdio JSON-RPC 2.0 Server |
| `contextra-py` | Ring 4 | ✅ vorhanden | PyO3 Python-Bindings, FFI catch_unwind |
| `contextra-testkit` | Tooling | ✅ vorhanden | Fault-VFS, `ManualClock`, In-Memory-`StorageEngine` |
| `contextra-bench` | Tooling | ✅ vorhanden | Reproduzierbare Benchmark-Harness |
| `xtask` | Tooling | ✅ vorhanden | CI/CD Tasks, Preflight, Drift Gates, Layering Checks |

---

## 8. Non-Obvious Decisions & Directives
* **Sync-Kern, async-Schale:** Ring 0 enthält kein `tokio`. Kerne sind synchron; Async-I/O gehört in Ring 1+.
* **Injizierter Nichtdeterminismus:** `Clock`, `Rng`, `IdGen` werden injiziert; direkte Aufrufe von `SystemTime::now()` oder `rand::thread_rng()` in Kernen sind verboten.
* **TxId-Generierung:** Stets über `collection.allocate_tx()`, NIEMALS `SystemTime::as_nanos()`.
* **MCP Transport:** Ausschließlich stdio JSON-RPC 2.0 (axum wurde entfernt via ADR-010).
* **Markdown Chunking:** Dokumente stets via `MarkdownChunker` aufteilen, NIEMALS komplette Fließtexte in einen einzelnen Vektor einbetten.
