# Contextra – Agenten-Betriebsanleitung (AGENTS.md)

<!-- doc-stamp: 2026-10-30 | Basis: docs/SPEC.md Fassung 4.0 -->

Stand: 30.10.2026 · Basis: Gesamtspezifikation Fassung 4.0 (`docs/SPEC.md`) · Umsetzungsplan 4.0 (`docs/PLAN.md`)

---

## 1. Geltungsbereich und Rangfolge der Informationsquellen

Diese Datei regelt die Arbeit aller autonomen Agenten im Repository. Bei Konflikten gilt diese Rangfolge:

1. **Verfassung und Vetos**: `docs/CONSTITUTION.md`, `docs/VETOES.md`
2. **Gesamtspezifikation Fassung 4.0**: `docs/SPEC.md` (insbesondere §2 bis §4 und §16)
3. **Umsetzungsplan Fassung 4.0**: `docs/PLAN.md`
4. **Quelltext und Manifeste**: `Cargo.toml`, `crates/*/Cargo.toml`, `crates/*/src`
5. **Governance-Dateien**: `capabilities.toml`, `governance/*.toml` (Angleichung in PLAN R2)
6. **Agenten-Betriebsanleitung**: diese Datei

Quality Gates dürfen niemals durch Ausnahmen, Sonderregeln oder lokale Overrides geschwächt oder umgangen werden (REQ-QA-010).

> **Agenten-spezifische Anleitungen:**
> - `Claude` / Antigravity: Abschnitt 2 lesen, danach `CLAUDE.md`.
> - Alle anderen Agenten: Abschnitt 2 reicht.
> - `@Jules`: Abschnitt 2a und `.agents/jules/SESSION_BOOTSTRAP.md`. Die Jules-Infrastruktur ist an Pfade gebunden, die die Reduktion teilweise entfernt hat. Vor Nutzung Abschnitt 8 lesen.

---

## 2. Start

### 2.1 Build-Status vor jeder Arbeit prüfen

Stand 30.10.2026 ist der Build **nicht grün** (SPEC §16, B1 bis B4). Vor jeder Änderung:

```bash
cargo metadata --no-deps --format-version 1 > /dev/null   # Workspace lädt?
cargo check --workspace --all-targets                    # Kompiliert?
```

Schlägt einer der Befehle fehl, ist die Aufgabe nur dann zulässig, wenn sie zu Phase R0 gehört (`docs/PLAN.md` §2). Andere Arbeit wird bis zum grünen Build zurückgestellt.

### 2.2 Lesereihenfolge

1. `docs/SPEC.md`, §0 (Schnellstart) und §3 (Regeln)
2. `docs/PLAN.md`, die Phase, zu der die Aufgabe gehört
3. Den Crate-Ordner der Aufgabe und dessen `AGENTS.md`

### 2.3 Ablauf je Aufgabe

1. Aufgabe einem Arbeitspaket (WP) aus `docs/PLAN.md` zuordnen. Ohne Zuordnung: nachfragen.
2. Existenz von APIs, Dateien und Typen im Code prüfen. Nichts aus dem Gedächtnis annehmen.
3. Kleinste mögliche Änderung durchführen. Ein WP = ein PR = höchstens ein Crate-Schnitt (Regel 12).
4. Lokal prüfen: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` (ab R0).
5. Dokumentation im selben PR anpassen, wenn sich Ist-Zustand, Ring, Abhängigkeit oder Befund ändert.

### 2.4 Gate-Status

Der Gate-Runner `cargo xtask` wurde mit Commit `068261abd` entfernt. Die Gates in `governance/gates.toml` sind derzeit nicht ausführbar. Bis zur Entscheidung D-R2 gelten die `cargo`-Befehle aus 2.3 als Mindestprüfung. Befehle wie `cargo xtask check` aus älteren Dokumenten sind ungültig.

---

## 3. Verifizieren, Rebase und Gate-Integrität

- „Fertig“ heißt: Alle relevanten Prüfungen auf dem Merge-Ergebnis sind grün (REQ-QA-010).
- **Rebase-Pflicht:** Vor jedem PR `git fetch origin && git rebase origin/main`. Danach Prüfungen erneut ausführen.
- Es ist verboten, Lints, Gates, Ratchets oder den Toolchain-Pin abzuschwächen oder durch Ausnahmen zu umgehen. Das gilt auch für `#[allow(...)]`-Erweiterungen auf Modulebene ohne ADR.
- **Geschützte Pfade:** `.github/**`, `justfile`, `rust-toolchain.toml`, `deny.toml`, `capabilities.toml`, `AGENTS.md`, `CLAUDE.md`, `governance/**`, `tests/layering.rs`, `tests/unsafe_islands.rs`.

---

## 4. Die zwölf Regeln für jede Änderung (SPEC §3.2)

Diese Regeln sind normativ. Verletzungen sind Fehler.

1. **Ring und Klasse zuerst.** Abhängigkeiten zeigen nur auf gleichen oder niedrigeren Ring (SPEC §4.1). Die Ringzuordnung steht in SPEC §4, nicht in einzelnen Dateien.
2. **Kein Panic in Produktion.** Kein `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!`, `unreachable!`, `process::exit`, ungeprüftes Indexieren. Fehler sind typisiert (`ContextraError`).
3. **Zeit, Zufall und IDs nur über Ports** (`Clock`, `Rng`, `IdGen`). Verboten in Kernlogik: `Instant::now`, `SystemTime::now`, `thread_rng`, `std::env::var`, `HashMap`/`HashSet` mit zufälligem Hash, sofern die Reihenfolge nach außen sichtbar ist. Ausnahme: Schlüssel- und Salt-Material aus dem System-RNG.
4. **Persistentes läuft über das WAL.** Sicherheitszustand wird über WAL und Manifest persistiert.
5. **I/O nur über `Vfs`.** Commit ist ein durabler Flush. `fsync`-Fehler werden propagiert, nie mit `let _ =` verworfen.
6. **Unsafe nur in zwei Inseln:** `contextra-sys` und `contextra-simd`. Jeder Block mit `// SAFETY:`. Alle anderen Crates haben `#![forbid(unsafe_code)]`.
7. **Mandant kommt aus dem `RequestContext`**, nie aus Literalen, Defaults oder Nutzerparametern (VETO-F10).
8. **Sicherheit ist kein Feature-Schalter.** Verschlüsselung und Integritätsprüfung sind in jedem Build aktiv. Kein `cfg(feature)`-Ersatz für Sicherheitscode.
9. **Messen vor Behaupten.** Löschaussagen und Leistungsangaben brauchen Messung oder Test.
10. **Fail-Closed.** Unbekannter Zustand, unbekannte Formatversion, gebrochene Kette: Ablehnung mit benanntem Grund, nie Raten.
11. **Eine Typfamilie je Konzept.** Keine Doppeldefinitionen öffentlicher Typen.
12. **Ein PR = ein Arbeitspaket.** Höchstens ein Crate-Schnitt.

---

## 5. Crates

### 5.1 Verbliebene Crates (Kern v1)

| Crate | Ring | Klasse | Kurzbeschreibung |
|---|---|---|---|
| `contextra-types` | 0 | [K] | Typen, IDs, Budgets, Filter, Fehler |
| `contextra-ports` | 0 | [K] | Trait-Verträge |
| `contextra-sys` | 0 | [K] (U) | Systemnahe Schicht, FFI-Insel |
| `contextra-simd` | 0 | [K] (U) | SIMD-Distanzkerne |
| `contextra-mvcc` | 0 | [K] | MVCC, SSI |
| `contextra-crypto` | 0 | [K] | Verschlüsselung, Schlüsselableitung, WAL-Integrität |
| `contextra-vector` | 0 | [K] | HNSW, SQ8, partieller Rebuild |
| `contextra-text` | 0 | [K] | BM25, Tokenizer, Morphologie |
| `contextra-rank` | 0 | [K] | Fusion, Kalibrierung, Drift |
| `contextra-store` | 1 | [K] | LSM-Speicher, WAL, SSTable, Kompaktion |
| `contextra-engine` | 2 | [K] | Collections, Hybridsuche, Transaktionen |
| `contextra-db` | 3 | [K] | Anwendungs-API (`prelude`, `advanced`) |
| `contextra` | 4 | [K] | Composition Root, öffentliche Fassade |
| `contextra-testkit` | Tooling | [T] | Testwerkzeuge |
| `contextra-bench` | Tooling | [T] | Benchmarks (`benchmarks/`) |

### 5.2 Entfernte Crates (nicht im Workspace)

Entfernt in der Reduktion vom 30.10.2026. Wiederherstellung nur über Git (`git checkout 3c1f144fa -- <Pfad>`) und nur über einen Plan nach SPEC §5.2 und PLAN R6.

`contextra-adapt`, `contextra-agent`, `contextra-audit-export`, `contextra-avv-generator`, `contextra-checkpoint`, `contextra-cognition`, `contextra-core`, `contextra-graph`, `contextra-infer-candle`, `contextra-infer-ollama`, `contextra-infer-onnx`, `contextra-ingest`, `contextra-kvcache`, `contextra-license`, `contextra-mcp`, `contextra-privacy`, `contextra-py`, `contextra-router`, `contextra-sandbox`, `contextra-wire`.

**Nicht neu anlegen:** Keine Stubs oder Platzhalter-Crates für entfernte Bausteine. Ein Stub mit Standardimplementierung verletzt Regel 8 und den Grundsatz „Messen vor Behaupten“.

---

## 6. Veto-Register (SPEC §3.1, `docs/VETOES.md`)

| Veto | Status | Regel |
|---|---|---|
| **VETO-F10**: Keine mandantenübergreifenden Datenflüsse | permanent abgelehnt | Kein Cache, Index, Embedding, Statistik oder Audit-Eintrag überschreitet Mandantengrenzen. |
| **VETO-F02**: HNSW-Löschung nur per Tombstone, Slot-Nullung und Rebuild | bestätigt | Kein Rewiring der Nachbarn. Slots werden erst nach Rebuild wiederverwendet. |
| **VETO-OP03**: Kein Echtzeit-Audio und kein STT | bestätigt, geschlossen | Kein Audio-Streaming im Kern. |
| **Single-Node-Doktrin** | permanent | Kein Sharding, kein P2P, keine verteilte Replikation. |

---

## 7. Sicherheitsports (Ist-Stand)

Die Port-Liste aus Fassung 3.1 (`KmsProvider`, `DurableLog`, `ErasureJournal`, `LayerScanner`, `Purger`, `EffaceableKeySlot`, `ErasedSetMirror`) bezieht sich auf die entfernte Löschkette und ist **nicht mehr verpflichtend**.

Verpflichtend in v1 bleibt: `Vfs` (Regel 5) und die Determinismus-Ports `Clock`, `Rng`, `IdGen` (Regel 3). Konkrete Implementierungen liegen in `contextra-store` bzw. `contextra-sys`. Null-, Test- oder Accept-All-Implementierungen dürfen nicht als Standard in den Produktpfad.

---

## 8. Hinweise für Jules und andere autonome Agenten

Die Dateien unter `.agents/jules/` beschreiben einen Ablauf mit `gh`-Issues, `cargo xtask` und einer Gate-Matrix aus dem Vorzustand. Diese Teile sind **nicht aktuell**:

- Alle Verweise auf `cargo xtask` sind ungültig (Befund B1).
- Routinen wie `unsafe-pruefung`, `unwrap-allow-abbau` und `dependency-hygiene` verweisen auf entfernte Crates.
- Die Session-Bootstrap-Datei referenziert Kontext-Dateien, die in dieser Revision nicht erzeugt werden.

Bis `.agents/jules/` in Phase R3 bereinigt ist, gilt für alle Agenten: **Diese Datei und `docs/SPEC.md` haben Vorrang vor den Jules-Dokumenten.**

Keine Agenten-Aktion darf Gates, Lints oder den Toolchain-Pin ändern, um einen roten Build grün zu machen. Die Ursache wird behoben, nicht das Symptom.

---

## 9. Nicht-Ziele v1 (SPEC §2.3)

Diese Fähigkeiten sind in v1 **nicht** implementiert und dürfen ohne ADR nicht hinzugefügt werden:

- Graph-Reasoning, Konsolidierung über Schlafzyklen, Kognition
- Router, Agenten-Laufzeit, Online-Adaption
- Inferenz-Backends (Candle, ONNX, Ollama), KV-Cache-Verwaltung
- MCP-Server, Python-Bindings, WASM-Sandbox
- Dokumenten-Ingestion, PII-Filter, Egress-Gateway
- DiskANN
- Verteilter Betrieb (kein Sharding, kein P2P)
- Echtzeit-Audio und STT (VETO-OP03)
- Löschbeweis als Rechtsnachweis (SPEC §9)
- Windows als Garantieplattform

**Offen:** RaBitQ ist im Code vorhanden, widerspricht aber diesem Katalog. Entscheidung D-R4.

---

## 10. Referenzen

| Dokument | Zweck |
|---|---|
| `docs/SPEC.md` | Normative Gesamtspezifikation 4.0 |
| `docs/PLAN.md` | Umsetzungsplan 4.0, Phasen R0 bis R6 |
| `docs/ARCHITECTURE.md` | Architekturübersicht (nicht normativ) |
| `docs/VETOES.md` | Vetos |
| `docs/CONSTITUTION.md` | Grundlegende Governance-Prinzipien |
| `docs/TYPE_REGISTRY.md` | Doppeldefinitionen öffentlicher Typen |
| `capabilities.toml` | Ring- und Reifegrad-Zuordnung (Angleichung in R2) |
| `governance/gates.toml` | Gate-Katalog (Angleichung in R2) |
| `governance/ratchet.toml` | Qualitäts-Baselines (nach R0 neu zu setzen) |
| `governance/protected-paths.toml` | Geschützte Pfade |
| `CLAUDE.md` | Arbeitsregeln für Claude und Antigravity |
| `docs/decisions/` | ADR-Index (Status einzelner ADRs in PLAN R3.4) |
