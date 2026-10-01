# Contextra — Agenten-Betriebsanleitung (AGENTS.md)

Stand: 2026-09-30 (Systemspezifikation v15)

## 1. Geltungsbereich und Rangfolge
Diese Datei regelt die Arbeit aller autonomen Agenten im Repository.
Bei Konflikten gilt stets folgende Rangfolge: Code + grüne Gates > AGENTS.md > docs/spec (`docs/spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md` / `docs/spec/CONTEXTRA_SPEZIFIKATION_v15.md`) > alle sonstigen Vorgaben.

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

## 4. Verifizieren
- „Fertig" bedeutet ausnahmslos: Alle relevanten Gates, Lints und Tests sind grün.
- Es ist verboten, Gates, Lints oder den Toolchain-Pin abzuschwächen oder zu umgehen, um Testergebnisse zu erzwingen.
- Geschützte Pfade: `.github/**`, `xtask/**`, `justfile`, `rust-toolchain.toml`, `deny.toml`, `capabilities.toml`, `AGENTS.md`, `.jules/setup/**`.

## 5. Scope
- Ausschließlich Dateien im explizit freigegebenen Scope der Task-Karte bearbeiten.
- Mängel außerhalb des Scopes nicht direkt beheben, sondern im PR-Text unter „Out-of-scope Findings" melden.

## 6. Invarianten (verbindlich, v15 Teil F)
- **Zero-Panic (P7):** Kein `unwrap()`, `expect()` oder `panic!()` in Produktionspfaden; Fehler per `Result` propagieren.
- **Prozessaufrufe:** Niemals über `sh -c`; Parameter via `shlex` parsen und als Argument-Array übergeben.
- **Unsafe-Isolierung:** `unsafe` ist streng isoliert auf die drei Unsafe-Inseln laut `capabilities.toml` (`contextra-simd`, `contextra-sys`, `contextra-wire`). Alle anderen Crates erzwingen `#![forbid(unsafe_code)]`.
- **Single-Node & Ein-Prozess-Garantie:** Contextra ist ein `cargo add`, kein Server. Multi-Node Scaling, Sharding, Peer-to-Peer-Sync oder Cluster-Logik sind verboten.
- **Pure Rust Inferenz:** GGUF/Candle Inferenz via `contextra-infer-candle` ist das Standard-Backend.
- **Sync-Kern (P26):** Ring 0 (`contextra-types`, `contextra-ports`, `contextra-vector`, `contextra-text`, `contextra-graph`, `contextra-rank`, `contextra-adapt`, `contextra-crypto`, `contextra-wire`, `contextra-sys`, `contextra-simd`, `contextra-core`, `contextra-avv-generator`, `contextra-audit-export`) enthält keine Async-Runtime (`tokio`).
- **Determinismus (P28):** Zeit, Zufall und IDs ausschließlich über injizierte Ports (`Clock`, `Rng`, `IdGenerator`); `TxId` über `collection.allocate_tx()` erzeugen; kein direktes `SystemTime::now()`/`thread_rng()` im Produktionscode.
- **Speicher & Locks:** SIMD-Puffer (`contextra-simd`) korrekt ausrichten; Sperrenhierarchie einhalten (`collections` → `kv_locks` → `embedder`); keine Locks über `.await`-Punkte halten.
- **Spec-Sync:** Jede API-Änderung MUSS `docs/spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md` spiegeln.

## Anti-Gaming
Das Abschwächen, Umgehen oder Deaktivieren von Qualitäts-Gates, Lints, Schwellenwerten oder Toolchain-Pins ist streng verboten. Gate-Schutz erfolgt über `protected-paths` <!-- harness:planned -->, `gate-weakening` <!-- harness:planned --> und `ratchet` <!-- harness:planned -->.

## 7. Nicht tun (Explizite Scope-Ausschlüsse v15 Teil 0.5)
- Kein unkontrolliertes, globales Teilgraph-Rebuilding im HNSW-Index durchführen (Ausnahme: ADR-097).
- Keine mandantenübergreifenden Datenflüsse oder Cross-Tenant-Aggregationen herstellen (`TenantId` isolation).
- Keine Realtime-Audio- oder Voice-Funktionen integrieren.
- Keine Veto-Sperren ohne explizites ADR in `docs/decisions/` umgehen.
- Keine Framework-Adapter (LangChain/LlamaIndex), kein horizontales Sharding, Peer-to-Peer-Sync, EU-AI-Act-Risikomapper oder externe HTTP-Inferenz als Standardabhängigkeit einfügen.
