# Contextra — Agenten-Betriebsanleitung (AGENTS.md)

Stand: 2026-10-25 (Spezifikation 1.2)

## 1. Geltungsbereich und Rangfolge der Quellen (§28.1)
Diese Datei regelt die Arbeit aller autonomen Agenten im Repository.
Bei Konflikten gilt streng folgende Rangfolge (§28.1):
1. **Spezifikation 1.2** (`docs/spec/CONTEXTRA_SPEZIFIKATION_v15.md`, §0, §1)
2. **Vetos und Prinzipien** (§4.1, §5.0, P1–P30)
3. **Crate-Inventar & Reifegrade** (`capabilities.toml`)
4. **Agenten-Betriebsanleitung** (`AGENTS.md`)
5. **Quellcode & grüne Gates**

Quality Gates dürfen unter keinen Umständen durch Ausnahmen oder Sonderbehandlungen geschwächt oder umgangen werden (REQ-QA-010).

## 2. Start
Arbeitskontext zu Beginn der Session laden:
- `cargo run --manifest-path xtask/Cargo.toml -- jules-preflight`
- Künftiger Start: `cargo xtask jules start --card <datei>` <!-- harness:planned -->

## 3. Ablauf
Jeder Task folgt diesem iterativen Ablauf:
1. Task lesen und Ziel verstehen.
2. Existenz von APIs, Dateien und Typen im Code prüfen (niemals Schnittstellen aus dem Gedächtnis annehmen).
3. Plan mit Dateiliste und Akzeptanzbefehlen erstellen.
4. Kleinste mögliche Änderung durchführen.
5. Verifizieren und PR-Text erstellen.

## 4. Verifizieren (§27.0, REQ-QA-010)
- „Fertig" bedeutet ausnahmslos: Alle relevanten Gates, Lints und Tests sind grün.
- Es ist verboten, Gates, Lints oder den Toolchain-Pin abzuschwächen oder zu umgehen, um Testergebnisse zu erzwingen (REQ-QA-010).
- Geschützte Pfade: `.github/**`, `xtask/**`, `justfile`, `rust-toolchain.toml`, `deny.toml`, `capabilities.toml`, `AGENTS.md`, `.jules/setup/**`.

### Pflichttabellen der Qualitäts-Gates
| Gate | Phase | Lokal Pflicht | CI Pflicht | Risiko-Klasse | Owner |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `scope-guard` | check | ja | ja | hoch | governance |
| `protected-paths` | check | ja | ja | kritisch | governance |
| `symbol-exists` | check | ja | ja | mittel | architecture |
| `ratchet` | verify | ja | ja | hoch | governance |
| `gate-weakening` | submit | ja | ja | kritisch | governance |
| `test-integrity` | submit | ja | ja | hoch | quality |
| `session-report` | submit | ja | ja | niedrig | agent-ops |

## 5. Scope
- Ausschließlich Dateien im explizit freigegebenen Scope der Task-Karte bearbeiten.
- Mängel außerhalb des Scopes nicht direkt beheben, sondern im PR-Text unter „Out-of-scope Findings" melden.

## 6. Systemregeln (Kurzform mit Spezifikationsverweisen, §0–§34)
- **Zero-Panic (P7, §27.6):** Kein `unwrap()`, `expect()` oder `panic!()` in Produktionspfaden (`src/`); Fehler per `Result` propagieren.
- **Prozessaufrufe:** Niemals über `sh -c`; Parameter via `shlex` parsen und als Argument-Array übergeben.
- **Unsafe-Isolierung (§6.7):** `#![forbid(unsafe_code)]` in allen Nicht-Insel-Crates. `unsafe` ist streng isoliert auf die drei Unsafe-Inseln laut `capabilities.toml` (`contextra-simd`, `contextra-sys`, `contextra-wire`).
- **Single-Node & Ein-Prozess-Garantie (§1.0, REQ-ARCH-001):** Contextra ist ein `cargo add`, kein Server. Multi-Node Scaling, Sharding, Peer-to-Peer-Sync oder Cluster-Logik sind verboten (§4.4).
- **Pure Rust Inferenz (§14.1):** GGUF/Candle Inferenz via `contextra-infer-candle` (`[R]`, stable) ist das Standard-Backend.
- **Sync-Kern (§6.8, P26):** Ring 0 enthält keine Async-Runtime (`tokio`).
- **Determinismus (§6.8, P28):** Zeit, Zufall und IDs ausschließlich über injizierte Ports (`Clock`, `Rng`, `IdGenerator`); kein direktes `SystemTime::now()`/`thread_rng()` im Produktionscode.
- **Speicher & Locks (§5.0, INV-LOCK-ORDER-1):** SIMD-Puffer (`contextra-simd`) korrekt ausrichten; Sperrenhierarchie einhalten (`storage.wal` → `pending_commit_queue` → `index_finalizer`); keine Locks über `.await`-Punkte halten.
- **Löschaussagen (INV-CLAIMS-1, §4.3, §20.12):** Aussagen zur Löschung folgen der Außenformulierung (§4.3, §20.12). Pauschale Zusagen wie „100 % gelöscht“ oder „DSGVO-konform“ sind verboten.
- **Spec-Sync:** Jede API-Änderung MUSS `docs/spec/CONTEXTRA_SPEZIFIKATION_v15.md` spiegeln.

## Anti-Gaming (REQ-QA-010)
Das Abschwächen, Umgehen oder Deaktivieren von Qualitäts-Gates, Lints, Schwellenwerten oder Toolchain-Pins ist streng verboten (REQ-QA-010). Gate-Schutz erfolgt über `protected-paths` <!-- harness:planned -->, `gate-weakening` <!-- harness:planned --> und `ratchet` <!-- harness:planned -->.

## 7. Nicht tun (Explizite Scope-Ausschlüsse, §0.5)
- Kein unkontrolliertes, globales Teilgraph-Rebuilding im HNSW-Index durchführen (Ausnahme: ADR-097).
- Keine mandantenübergreifenden Datenflüsse oder Cross-Tenant-Aggregationen herstellen (`TenantId` isolation, REQ-TEN-001).
- Keine Realtime-Audio- oder Voice-Funktionen integrieren (§0.5).
- Keine Veto-Sperren ohne explizites ADR in `docs/decisions/` umgehen.
- Keine Framework-Adapter (LangChain/LlamaIndex), kein horizontales Sharding, Peer-to-Peer-Sync, EU-AI-Act-Risikomapper oder externe HTTP-Inferenz als Standardabhängigkeit einfügen (§0.5).
