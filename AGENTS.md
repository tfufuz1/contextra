# Contextra — Agenten-Betriebsanleitung (AGENTS.md)

## 1. Geltungsbereich und Rangfolge
Diese Datei regelt die Arbeit aller autonomen Agenten im Repository.
Bei Konflikten gilt stets folgende Rangfolge: Code + grüne Gates > AGENTS.md > docs/spec > alle sonstigen Vorgaben.

## 2. Start
Arbeitskontext zu Beginn der Session laden:
- `just start` (Zielzustand, siehe Folgeaufgabe)

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
- Geschützte Pfade: `.github/**`, `xtask/**`, `justfile`, `rust-toolchain.toml`, `deny.toml`, `capabilities.toml`, `AGENTS.md`, `.jules/setup/**`. Änderungen an diesen Pfaden erfordern gesonderte Freigabe.

## 5. Scope
- Ausschließlich Dateien im explizit freigegebenen Scope der Task-Karte bearbeiten.
- Mängel außerhalb des Scopes nicht direkt beheben, sondern im PR-Text unter „Out-of-scope Findings" melden.
- Keine Artefakte committen, die sich bei jedem Commit dynamisch ändern.

## 6. Invarianten
- **Zero-Panic:** Kein `unwrap()`, `expect()` oder `panic!()` in Produktionspfaden; Fehler per `Result` propagieren.
- **Prozessaufrufe:** Niemals über `sh -c`; Parameter via `shlex` parsen und als Argument-Array übergeben.
- **Unsafe-Isolierung:** `unsafe` ist streng isoliert auf `contextra-simd`, `contextra-sys` und `contextra-wire`. Alle anderen Crates erzwingen `#![forbid(unsafe_code)]`.
- **Sync-Kern:** Ring 0 (`contextra-text`, `contextra-graph`, `contextra-adapt`, `contextra-crypto`) enthält keine Async-Runtime (`tokio`).
- **Nichtdeterminismus:** Zeit, Zufall und IDs injizieren; `TxId` über `collection.allocate_tx()` erzeugen.
- **Speicher & Locks:** SIMD-Puffer (`contextra-simd`) korrekt ausrichten; keine Locks über `.await`-Punkte halten.

## 7. Nicht tun
- Kein partielles HNSW-Rewiring oder Teilgraph-Rebuilding (F-02) durchführen wegen Recall-Kollaps und RwLock-Contention; zulässig ist ausschließlich reines Tombstone-Pruning.
- Keine mandantenübergreifenden Datenflüsse, Knowledge-Sharing oder Cross-Tenant-Aggregationen (F-10) herstellen; Mandantenisolation (`TenantId`) ist absolut zur Wahrung von DSGVO-Löschgarantien und KV-Cache-Sicherheit.
- Keine Realtime-Audio-, Speech-to-Text-, Voice- oder Jarvis-Assistenten-Funktionen (OP-03) integrieren, da Audio-Streaming nicht zum bi-temporalen Speichersubstrat gehört.
- Keine Veto-Sperren oder Isolationsgrenzen ohne explizites ADR in `DECISIONS.md` umgehen.
