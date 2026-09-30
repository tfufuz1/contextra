# Contextra — Agenten-Betriebsanleitung (AGENTS.md)

Stand: 2026-09-28

## 1. Geltungsbereich und Rangfolge
Diese Datei regelt die Arbeit aller autonomen Agenten im Repository.
Bei Konflikten gilt stets folgende Rangfolge: Code + grüne Gates > AGENTS.md > docs/spec (`docs/spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md`) > alle sonstigen Vorgaben.

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

## 6. Invarianten
- **Zero-Panic:** Kein `unwrap()`, `expect()` oder `panic!()` in Produktionspfaden; Fehler per `Result` propagieren.
- **Prozessaufrufe:** Niemals über `sh -c`; Parameter via `shlex` parsen und als Argument-Array übergeben.
- **Unsafe-Isolierung:** `unsafe` ist streng isoliert auf die drei Unsafe-Inseln laut `capabilities.toml` (`contextra-simd`, `contextra-sys`, `contextra-wire`). Alle anderen Crates erzwingen `#![forbid(unsafe_code)]`.
- **Single-Node & Ein-Prozess-Garantie:** Contextra ist ein `cargo add`, kein Server. Multi-Node Scaling oder Cluster-Logik sind verboten.
- **Pure Rust Candle Inferenz:** GGUF Inferenz via `contextra-infer-candle` ist das Standard-Backend.
- **Sync-Kern:** Ring 0 (`contextra-types`, `contextra-ports`, `contextra-vector`, `contextra-text`, `contextra-graph`, `contextra-rank`, `contextra-adapt`, `contextra-crypto`) enthält keine Async-Runtime (`tokio`).
- **Nichtdeterminismus:** Zeit, Zufall und IDs injizieren; `TxId` über `collection.allocate_tx()` erzeugen.
- **Speicher & Locks:** SIMD-Puffer (`contextra-simd`) korrekt ausrichten; keine Locks über `.await`-Punkte halten.
- **Spec-Sync:** Jede API-Änderung MUSS `docs/spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md` spiegeln.

## Anti-Gaming
Das Abschwächen, Umgehen oder Deaktivieren von Qualitäts-Gates, Lints, Schwellenwerten oder Toolchain-Pins ist streng verboten. Gate-Schutz erfolgt über `protected-paths` <!-- harness:planned -->, `gate-weakening` <!-- harness:planned --> und `ratchet` <!-- harness:planned -->.

## 7. Nicht tun
- Kein unkontrolliertes, globales Teilgraph-Rebuilding im HNSW-Index durchführen (Ausnahme: ADR-097).
- Keine mandantenübergreifenden Datenflüsse oder Cross-Tenant-Aggregationen herstellen.
- Keine Realtime-Audio- oder Voice-Funktionen integrieren.
- Keine Veto-Sperren ohne explizites ADR in `docs/decisions/` umgehen.
