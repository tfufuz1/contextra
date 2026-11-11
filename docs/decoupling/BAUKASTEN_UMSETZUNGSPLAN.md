# Contextra-Baukasten: Umsetzungsplan in Arbeitspaketen

**Bezug:** `docs/SPEC.md` v3 (normativ) · **Ausgangsbasis dieses Plans:** Klon `b02bbe0` (11.11.2026), statisch geprüft
**Zweck:** Die Ziele der SPEC (Entkopplung, Gates, Score) und die Produktionsreife der Bausteine in ausführbare Arbeitspakete (WP) zerlegen.
Beleg-Marken wie in der SPEC: **[V]** gemessen · **[A]** abgeleitet · **[S]** Schätzung. Nichts hier wurde gebaut oder ausgeführt.

---

## 1. Verhältnis zur SPEC und zu meinem ersten Bericht

### 1.1 Rangfolge
Die SPEC v3 ist verbindlich (§0.2 der SPEC). Dieser Plan **operationalisiert** sie, ergänzt nur, was die SPEC offen lässt (Produktionsreife, Release, Referenzkompositionen), und weicht nirgends ab, außer über einen ADR (siehe §1.3).

### 1.2 Korrektur meines ersten Berichts
In AD-4 hatte ich behauptet, `impl From<VectorError> for ContextraError` sei wegen der Orphan-Regel in einem Drittcrate unmöglich, und deshalb einen `IntoContextra`-Extension-Trait vorgeschlagen. **Das war falsch für den Fall der SPEC.** Die Impl liegt im Crate, dem `VectorError` gehört. Dann ist `VectorError` ein lokaler Typ als Trait-Parameter von `From<_>`, und die Regel ist erfüllt. SPEC §4.3 c) (`From` in `compat/error.rs`) ist korrekt und wird so umgesetzt. Der Extension-Trait entfällt.

### 1.3 Abgleich meiner Architekturentscheidungen mit der SPEC

| Mein Vorschlag (erster Bericht) | SPEC v3 | Entscheidung im Plan |
|---|---|---|
| AD-1 vier Ebenen | E0, E1a, E1b, E2, E3 (feiner) | **SPEC** |
| AD-2 `ids`/`env` als einzige Seams | identisch (R4, R7, Mikro-Kerne) | **SPEC** |
| AD-3 Ports gehören dem Konsumenten | `contract`-Modul je Baustein, `ports` gibt ab (S-50/51) | **SPEC** |
| AD-4 Fehler je Crate | R3 + §5.3, `ContextraError` schrumpft erst in S-70 | **SPEC** (ohne Extension-Trait) |
| AD-6 ein `contextra-bridge`-Crate | `compat/` je Crate hinter Feature `workspace` | **SPEC**. Nachteil bleibt: jedes Crate trägt Brückencode. Kompensation durch Gate G-L2 |
| AD-7 `default = []` | R10 (nur `std` und wesensnotwendige) | **SPEC** |
| AD-8 `graph-core`/`graph-rag`, `store-lsm`/`store-wal`, `agent-core`, `cognition-core`, `mcp-core`, `audit-chain` | nur `vector-diskann`, `graph-research` (S-80/81), Inversion von `cognition`/`agent` per Brücke in `db` (S-63) | **SPEC zuerst.** Meine Splits sind **ADR-Kandidaten** (WP-ADR, nach Welle 6, nur wenn der Score M es nach dem SPEC-Pfad noch verlangt) |
| AD-10 Referenzimplementierung + Beispiel je Baustein | nicht gefordert | **Ergänzung** als Reifegrad R2 (§2) |

Befunde aus meiner Analyse, die die SPEC nicht oder nur am Rand führt und die der Plan aufnimmt: doppelter `TokenBudget` (`agent::budget` ↔ `types`), drei BLAKE3-Hash-Ketten, `contextra-vault` aus den Workspace-Members ausgeschlossen, Spezifikationslücke „Context Compaction (Truncate/Summary/Selective)“, `cognition`-Shims (`engine_local.rs`, `graph_local.rs`), `mcp`-Kern ohne eigene Protokollschicht.

### 1.4 Delta SPEC ↔ Klon `b02bbe0` [V]

| Punkt | SPEC v3 | Klon | Folge |
|---|---|---|---|
| HEAD | `5408826f8` | `b02bbe0` | WP-00 misst neu, keine Zahl der SPEC wird ungeprüft übernommen |
| `env` Default | `std, test-utils` (K13) | `std` | WP-11 schrumpft auf Verifikation |
| `from_key`-Aufrufe außerhalb Tests | 86 | 82 (andere Zählung) | WP-10 klassifiziert exakt neu |
| `xtask` in `protected-paths.toml` | 2 Globs | 2 | WP-01 wie geplant |
| `not_run_baseline` | 30+ Gates | ≈38 Einträge (grob) | WP-01 wie geplant |
| `crypto`/`mvcc`/`kvcache`: ungegatete `types`/`ports`-Treffer | 34 / 12 / 11 | 0 außerhalb `compat/` (grobe Zählung) | S-30/S-31 evtl. weitgehend erledigt, **in WP-00 belegen, nicht annehmen** |
| Dateien mit Treffer außerhalb `compat/` | `vector` 35, `graph` 52, `store` 65, `text` 6, `checkpoint` 2 | 39, 59, 66, 10, 5 | Metriken sind nicht identisch (meine Zählung enthält gegatete Dateien); Größenordnung bestätigt |
| `tests/golden/`, `gate_bridge_null.py`, `modularity_score.py`, `doc_lint.py` | fehlen | fehlen (`tests/` enthält nur `layering.rs`, `unsafe_islands.rs`, 2 Negativtests) | WP-02, WP-00, WP-03 |

---

## 2. Reifegrade (was „produktionsreif“ heißt)

Die SPEC definiert „entnehmbar“ (Gates G-L2 → G-L1 → G-L3 → G-L4). Produktionsreife verlangt mehr. Jedes Crate erhält einen Reifegrad; `tools/certify_crate.py` (WP-04) prüft ihn und schreibt `docs/decoupling/certs/<crate>.md`.

| Grad | Name | Kriterien (alle MÜSSEN erfüllt sein) |
|---|---|---|
| **R0** | Ist | – |
| **R1** | Entkoppelt | G-L2, G-L1, G-L3 (`--no-default-features`, `--each-feature`), G-L4 grün; R10 erfüllt; eigener Fehlertyp (R3); Score M ≥ Zielwert der Ebene (§8.6 SPEC) |
| **R2** | Einsetzbar | R1 **und** (a) `contract`-Modul mit allen Umwelt-Traits, (b) Referenzimplementierung ohne I/O (Muster `MemoryTextIndex`), (c) `examples/standalone.rs` ≤ 30 Zeilen, in CI gebaut und ausgeführt, (d) `#![deny(missing_docs)]` für die öffentliche API, README mit Quickstart und Feature-Tabelle, (e) Doctests grün |
| **R3** | Release-fähig | R2 **und** (f) `cargo public-api`-Snapshot + `cargo semver-checks` ohne ungeplanten Bruch, (g) Property-/Differenz-/Golden-Tests je nach Crate-Art (§3.2), (h) MSRV-Job `1.89`, (i) `cargo deny` + `cargo audit` grün, (j) Cargo-Metadaten vollständig (`description`, `license`, `repository`, `keywords`, `categories`, `readme`, `rust-version`, `include`), `publish`-Entscheidung dokumentiert, (k) `CHANGELOG.md`, (l) `cargo publish --dry-run` grün (E0/E1) bzw. bewusst `publish = false` (E2/E3), (m) `tools/export_crate.py` baut das Crate isoliert |

**Zielzustände:** alle E0 und E1 → **R3**; E2 → **R1**; E3 → R1. Die Tier-Einstufung `frozen` (`erasure`, `license`, `audit-export`, `avv-generator`, `sandbox`, `infer-onnx`, `infer-candle`) senkt das Ziel auf **R1**, weil dort laut `AGENTS.md` nicht weiterentwickelt wird; Entkopplung gilt trotzdem.

---

## 3. Querschnittsvorgaben

### 3.1 Standard-Abnahme (Makros, in WPs referenziert)

| Makro | Prüfung |
|---|---|
| **GL2** | `grep -rnE 'contextra_(types\|ports\|testkit)' crates/contextra-X/src --include=*.rs \| grep -v '/compat/'` ist leer (Doc-Kommentare und `#[cfg(test)]` zählen mit, R2) |
| **GL1** | `cargo tree -p contextra-X -e normal --no-default-features --prefix none \| grep '^contextra-' \| sort -u` ⊆ Kern-Closure der Tabelle SPEC §3.1 |
| **GL3** | `cargo check -p contextra-X --no-default-features --locked` und `cargo hack check -p contextra-X --each-feature --locked`; `cargo check --workspace --all-targets --locked` (INV-4) |
| **GL4** | `python3 tools/isolation_proof.py` und `python3 tools/export_crate.py contextra-X` |
| **GF** | Golden-Prüftests des Formats (G-F1), für Versionsanhebung zusätzlich v-alt-Lesetest |
| **GD** | Differenztest alt ↔ neu nach SPEC §5.4 |
| **GE** | Fehlerabbildung: tabellengetriebener Display-Test je Variante (SPEC §5.3) |
| **GC** | Concurrency: Loom/TSan-Suite des Crates unverändert grün |
| **RATCHET** | Score M, `types_fan_in`, `ports_fan_in` nicht schlechter |
| **REVERT** | `git revert` des Commit-Paars auf Wegwerf-Zweig, `cargo check` grün |

### 3.2 Testarten je Crate-Art

| Art | Pflichttests für R3 |
|---|---|
| Numerik/Algorithmus (`simd`, `rank`, `adapt`, `vector`, `graph`) | Property-Tests (Invarianten), Differenztest gegen Skalar-/Brute-Force-Orakel, Determinismus (INV-5), Benchmark mit Regressionsschwelle |
| Persistenz (`store`, `mvcc`, `checkpoint`, `text`, `vector`) | Golden-Dateien v-alt/v-neu, Crash-/Recovery-Suite, Fuzzing der Dekoder (SSTable, WAL, HNSW-Header) |
| Parser/Protokoll (`ingest`, `wire`, `mcp`) | Fuzz-Ziele, Limits-Tests, Eingabe-Korpus |
| Sicherheit (`crypto`, `erasure`, `privacy`, `prompt-guard`, `vault`) | Known-Answer-Tests, Zeroize-/Drop-Tests, Mutation-Score ≥ Baseline (G-C1) |
| Nebenläufigkeit (`mvcc`, `store`, `graph`, `kvcache`) | Loom für Sequenzlog/Snapshots, TSan-Lauf |

### 3.3 Commit- und Review-Regeln
Commit-Paar `WP-xx A` (verschieben/benennen) und `WP-xx B` (umbauen), einzeln revertierbar (INV-3). Änderungen an `governance/*` nur mit ADR und Label `protected-path` (INV-10). Jedes WP endet mit aktualisierter crate-lokaler `SPEC.md`/`PLAN.md`.

---

## 4. Wellenplan und kritischer Pfad

```
W0 Fundament ─┬─► W1 Seams/Features ─┬─► W3 E1a ──► W4 E1b ──► W5 KV/infer ──► W6 Ports/E2/E3 ──► W7 Aufräumen ──► W8 Release
              │                      │      (mvcc, crypto, text, rank, graph, checkpoint)  (vector, store)
              └─► W2 E0-Härtung (parallel zu W1/W3, keine Abhängigkeit zum Kern)
```

**Kritischer Pfad:** WP-00 → WP-01/02 → WP-10 → WP-30 (`mvcc`) → WP-36 (`vector`) / WP-31 (`crypto`) → WP-37 (`store`) → WP-38 (KV-Paar) → WP-50/51 → WP-63/64 → WP-70 → WP-90.
**Parallel:** W2 (E0-Bausteine) hat keine Kern-Abhängigkeit und liefert **früh nutzbare Bausteine**. `rank`, `graph`, `text`, `checkpoint` laufen nach W1 unabhängig voneinander.

**Aufwand [S]** (Personentage, ±50 %, abgeleitet aus LOC und Referenzzahlen): Gesamt ≈ **212 PT**. Bei drei parallelen Spuren ≈ 14 Kalenderwochen plus 30 % Review/Risiko-Puffer.

| Welle | Inhalt | PT |
|---|---|---|
| W0 | Fundament | 22 |
| W1 | Seams und Features | 12 |
| W2 | E0-Härtung | 23 |
| W3 | E1a | 38 |
| W4 | E1b | 40 |
| W5 | KV-Paar, infer-* | 14 |
| W6 | Ports/Types abgeben, E2/E3 | 35 |
| W7 | Aufräumen, Experimente-Crates | 17 |
| W8 | Release, Kompositionen | 11 |

**Wellen-Exit:** Eine Welle gilt erst als abgeschlossen, wenn alle ihre WPs ihre Abnahme erfüllen, `cargo check --workspace --all-targets --locked` grün ist und der Ratchet seit Wellenbeginn nicht gestiegen ist.

---

## 5. Arbeitspakete

Format: **Voraussetzung** · **Aufwand** · Aufgaben · **Abnahme** · Risiko/Rollback.

### Welle 0: Fundament (Messbarkeit vor Umbau)

#### WP-00 · Werkzeuge und Neuvermessung (SPEC S-00R)
**Vorauss.:** – · **Aufwand:** 4 PT
1. Werkzeugliste bereitstellen (reproduzierbares Dev-Image/CI-Image): Rust `1.89.0`, `cargo-hack`, `cargo-public-api` (Toolchain-Konflikt `nightly-2024-11-01` ↔ MSRV klären und als ADR festhalten), `cargo-semver-checks`, `cargo-deny`, `cargo-audit`, `cargo-nextest`, `cargo-mutants`, `ripgrep`, `jq`.
2. `tools/gate_bridge_null.py` (GL2 als Skript) und `tools/modularity_score.py` (Score M nach SPEC §8.6) anlegen.
3. Vollmessung: `cargo metadata`, `cargo tree` je Crate (Default und `--no-default-features`), `cargo check` je Crate beider Varianten, `count_bridge_refs.py`, Score. **Ergebnis je Crate: baut `--no-default-features` ja/nein.** Das belegt oder widerlegt die Hypothese B18 (SPEC) bzw. meinen Befund F2.
4. Delta-Bericht gegen §1.4 dieses Plans und gegen SPEC §2.
5. Klassifikation der `from_key`-Aufrufstellen (mechanisch / nicht-mechanisch) als CSV.
**Abnahme:** `docs/decoupling/baseline/` aktualisiert; je Crate eine Zeile „baut standalone: ja/nein + Fehlertext“; jede Zahl trägt [T] statt [V]/[A].
**Risiko:** Ergebnis widerlegt die Annahme „B18“ (dann entfallen Teile von WP-32/34/36); kein Rollback nötig.

#### WP-01 · Gate-Reparatur und Ratchet (S-01R)
**Vorauss.:** WP-00 · **Aufwand:** 5 PT
1. Die zwei `xtask/**`-Globs aus `governance/protected-paths.toml` und den Kommentar im `justfile` entfernen (ADR + Label `protected-path`).
2. Jeden Eintrag in `ratchet.toml › not_run_baseline` triagieren: **CI-Job** oder **ADR mit Frist** (kein dritter Zustand).
3. CI: `cargo check --workspace --all-targets --locked`, Matrix mit `--features docid-128`, G-L1 blockierend; `gate_g1_layering.py` entfernen, sobald G-L1 läuft.
4. Ratchet `types_fan_in` und `ports_fan_in` anlegen (Start 28/22 laut Klon [V], nach WP-00 bestätigen), nur sinkende Werte.
**Abnahme:** `tools/test_gate_layering_negative.py` schlägt bei absichtlichem Verstoß fehl; G-L1 am Baseline-Stand grün; kein Eintrag in `not_run_baseline` ohne ADR-Verweis.
**Rollback:** CI-Konfiguration revertierbar; Ratchet-Werte nur per ADR.

#### WP-02 · Golden-Korpus und Differenz-Harness (S-02R)
**Vorauss.:** WP-00 · **Aufwand:** 6 PT
1. `tests/golden/<format>/` mit `MANIFEST.sha256` und `tools/golden_regen.sh` (Erzeugertests `#[ignore]`, Prüftests laufen immer). Formate laut SPEC §5.1: Manifest `MFMN`, SSTable `MFSX`/`MFST`, WAL inkl. HMAC-Kette, Rollback-Intent, HNSW (Header `0x484E5357`, v2), DiskANN, KDF `MFKD`, KV-Segment v2, Text-Postings, Graph (bincode), Checkpoint.
2. `mvcc/tests/golden/` ins Wurzel-Manifest einhängen.
3. Differenz-Harness (`tests/differential/`): Muster „alte und neue Implementierung gleichzeitig kompiliert, gleiche Eingaben“.
**Abnahme:** GF am **unveränderten** Stand grün; Manifest-Hash ändert sich nur durch bewusste Regeneration.
**Risiko:** Golden-Erzeugung ist nicht deterministisch (Zeitstempel, Zufall) → Clock/Rng aus `env` mit festem Seed injizieren, sonst ist das Format-Gate wertlos.

#### WP-03 · Dokumentation und Metadaten-Wahrheit (S-03R)
**Vorauss.:** WP-01 · **Aufwand:** 3 PT
1. `types/lib.rs`-Verwenderliste durch generierten Verweis ersetzen.
2. Falsche Docstrings korrigieren (`privacy`, `cognition`, `kvcache`, `avv-generator`, `sys/lib.rs:19`, `vector/types.rs:3–6`: dort steht „autark“, was F2 widerspricht).
3. `tools/doc_lint.py` (G-DOC1: keine Compliance-Aussagen, INV-7).
4. `capabilities.toml` um `sublevel` (E1a/E1b) erweitern.
**Abnahme:** `capabilities-consistency` und G-DOC1 grün.

#### WP-04 · Zertifizierungs-Tooling (Ergänzung)
**Vorauss.:** WP-00 · **Aufwand:** 4 PT
1. `tools/certify_crate.py <crate>`: führt GL1–GL4, `deny_missing_docs`-Prüfung, Beispiel-Build, Doctests, `public-api`-Snapshot, `semver-checks`, MSRV-Check aus und schreibt `docs/decoupling/certs/<crate>.md` mit Reifegrad R0–R3.
2. CI-Job `certify` je Crate (informativ bis zur jeweiligen Welle, danach blockierend).
**Abnahme:** Lauf für ein bereits entkoppeltes Crate (z. B. `rank`) liefert nachvollziehbar R1 und listet fehlende R2-Punkte.

---

### Welle 1: Seams und Feature-Hygiene

#### WP-10 · `ids` Commit C (S-10C)
**Vorauss.:** WP-02 · **Aufwand:** 4 PT
1. Jede der ≈82 Aufrufstellen von `::from_key(` gemäß WP-00-Liste anpassen: mechanisch über `From<IdError> for ContextraError` und `?`; nicht-mechanische Stellen (Musterabgleich auf `ContextraError::InvalidInput`) einzeln umbauen.
2. Differenztest SPEC §4.1: 10.000 zufällige Schlüssel plus Randfälle (leer, Unicode, 4 KiB) gegen alle `DocId`-Kopien (`erasure`, `graph`, `ingest`, `rank`, `text`, `vector`); Golden-Vektoren in 64 **und** 128 Bit.
**Abnahme:** GF, GD, GE, GL; **Rückfall nach SPEC §9.2, wenn > 20 nicht-mechanische Stellen**.
**Risiko:** serde/`bincode`-Ausgabe weicht zwischen Kopien ab → vor jedem Umbau messen.

#### WP-11 · `env`-Verifikation (S-11R)
**Vorauss.:** WP-02 · **Aufwand:** 1 PT
1. Prüfen: `default = ["std"]` [V] bereits erfüllt; `test-utils` nur bei Verbrauchern aktiv, die es nutzen (`cargo tree -p contextra-env -e features`).
2. `unix_ms`-Differenztest gegen die `now_unix_ms`-Kopien in `text`, `graph`, `vector/contract.rs` (`ManualClock`).
**Abnahme:** GD; kein Verbraucher aktiviert `test-utils` im Normalbuild.

#### WP-12 · Feature-Hygiene R10 (S-12R, S-24)
**Vorauss.:** WP-01 · **Aufwand:** 4 PT
1. Pro Crate Defaults nach SPEC §3.1 setzen: `crypto` leer, `graph`, `kvcache`, `rank`, `text`, `vector`, `testkit`, `license`, `audit-export` ohne `workspace`/`*-compat`/`exp-*`/`test-utils`.
2. `erasure`: `[features]` einführen (`signing`, `vault`, `async`, `conformance`); Datei→Abhängigkeit-Zuordnung per Erhebung festschreiben; Default behält heutiges Verhalten.
3. `feature_audit.py` entrauschen (Defaults und Weiterleitungsfeatures nicht melden), Meldungen > 0 sind Fehler.
4. Gate `error-domain` aktivieren (Basis `error_topology_mapper.py`).
**Abnahme:** `cargo hack check --each-feature -p <crate>` je Crate; `feature_audit` grün; **Konsumenten-Check:** `cargo check --workspace` zeigt, dass kein Verbraucher stillschweigend ein entferntes Default-Feature verloren hat (explizit in deren Manifesten ergänzen).
**Risiko:** Cargo-Feature-Unification macht `docid-128` workspace-weit wirksam; Matrixlauf mit und ohne Feature ist Pflicht.

#### WP-13 · Seam-Deduplizierung (Ergänzung zu SPEC B20)
**Vorauss.:** WP-10, WP-11 · **Aufwand:** 3 PT
1. Entscheidung je Crate festschreiben: E0 behält lokale Mini-Traits (R7), E1+ nutzt `env`/`ids` direkt. Konkret zu bereinigen: `text` (hängt zwingend an `env`/`ids`, definiert aber `Clock`, `Rng`, `DocId` selbst), `vector` (2× `Clock`, 2× `Rng`, `MetricsSink`), `graph`, `rank`.
2. `DistanceMetric` (3 Kopien): R4-Ausnahme nutzen, Konformitätstest (serde-Namen und Reihenfolge identisch) in `compat/`.
3. Blanket-Brücken nach SPEC §4.3 a/b; Konfliktregel e) umsetzen, **Test beider Features gleichzeitig** gegen E0119-Überlappung.
**Abnahme:** `grep -rn 'trait \(Clock\|Rng\)' crates` nur in `env` und E0-Crates; GD.

---

### Welle 2: E0-Bausteine auf R3 (parallel, kein Kernbezug)

> Diese Welle liefert die ersten produktionsreifen Bausteine. Alle Crates sind laut Messung bereits entkoppelt; die Arbeit ist Härtung, nicht Umbau.

#### WP-20 · `simd`, `wire`, `sys`, `durable-fs` (Unsafe-Inseln und Basis)
**Vorauss.:** WP-04 · **Aufwand:** 6 PT
1. `simd`: Property-Tests gegen Skalarorakel (SPEC §5.4, rel. ≤ 1e-5), Dim 4/17/128/768, NaN/Inf-Verhalten; Benchmark-Schwelle; `simsimd` nur als Dev-Orakel; `workspace`-Feature nur Brücke.
2. `wire`: Fuzz-Ziele für FlatBuffers-Dekodierung; Schema-Hash-Job für `wire/schemas/contextra.fbs.sha256` (SPEC B21).
3. `sys`: `unsafe`-Inventar (`unsafe_islands.rs`) aktuell, `mlock`/`VirtualLock`-Pfade mit Fehlerfall-Tests (`RLIMIT_MEMLOCK`); `lib.rs:19`-Docstring korrigieren.
4. R2/R3-Punkte (Beispiel, Docs, Metadaten) je Crate.
**Abnahme:** `certify_crate.py` → R3; `unsafe`-Inventar deckt alle Stellen (simd 269, wire 45, sys 22 [V]) ab.

#### WP-21 · `rank`, `adapt`
**Vorauss.:** WP-04, WP-12 · **Aufwand:** 4 PT
1. `adapt`: Features umbenennen (`exp-flow-thompson`, `exp-rie-greedy`), `offpolicy.rs` gegen `off_policy.rs` abgleichen, je ein `Clock`/`Rng`-Trait (S-23R).
2. `rank`: Property-Tests (PAVA monoton, Platt-Wertebereich, Conformal-Abdeckung, RRF-Determinismus mit ID-Tie-Break), `fusion/mod.rs: use crate::compat` auf Gate prüfen.
3. Beispiele: `rank` (zwei Rangfolgen → RRF, < 30 Zeilen), `adapt` (LinUCB-Wahl mit Seed).
**Abnahme:** R3; `grep -rn 'trait \(Clock\|Rng\)' crates/contextra-adapt/src` je 1.

#### WP-22 · Sicherheitsbausteine: `privacy`, `prompt-guard`, `erasure`, `sandbox`, `license`, `audit-export`, `avv-generator`
**Vorauss.:** WP-12 · **Aufwand:** 6 PT
1. `license`, `audit-export`: `workspace` aus Default, Inline-Brücken nach `compat/` (S-20R); `audit-export/testkit.rs` hinter `test-utils`.
2. `privacy`: Known-Answer-Tests für IBAN/E-Mail-Erkennung, Zeroize-Tests; **Abgrenzung zu `contextra-vault`** (Namenskollision) in der Doku.
3. `prompt-guard`: `file-audit` bleibt Feature; Test, dass Logger ohne Feature keinen `std::fs`-Zugriff hat.
4. `erasure`: Features aus WP-12; Konformitätssuite (`conformance`) aus `testkit` hierher (S-50-Vorbereitung).
5. `sandbox`: Fuel-Tests (`consume_fuel`, `set_fuel`), Timeout-Verhalten, `anyhow` aus der Public API entfernen, falls dort sichtbar.
**Abnahme:** R3 für `privacy`, `prompt-guard`; R1 für `frozen`-Crates (`erasure`, `license`, `audit-export`, `avv-generator`, `sandbox`); G-C1 Mutation-Score ≥ Baseline.

#### WP-23 · `ingest`, `infer-onnx`
**Vorauss.:** WP-12 · **Aufwand:** 4 PT
1. `ingest`: Fuzz-Ziele je Format (`fmt-pdf`, `fmt-docx`, `fmt-pptx`, …), `fuzz_support.rs` hinter `fuzz`, `limits.rs`-Tests (Zip-Bomb, Tiefe), `TryFrom` für 128-Bit-`DocId`.
2. `infer-onnx`: `fakes` hinter `test-utils`; nur eine Port-Referenz (`EmbeddingProvider`) in `compat/`.
**Abnahme:** `ingest` R3; `infer-onnx` R1; Fuzzläufe in CI ≥ definierte Mindestdauer ohne Fund.

#### WP-24 · `vault` (S-22R)
**Vorauss.:** WP-12 · **Aufwand:** 3 PT
1. `crates/contextra-vault` aus `exclude` zurück in `members` (und `default-members`); sonst erfasst keine Workspace-CI das zwingend von `db` genutzte Crate.
2. In `db` die Quellen `volatile_vault.rs`, `vault_compat.inc`, `vault_sys.inc` entfernen, nur Re-Export behalten.
3. Lokale `VaultDocId/TxId/TenantId` → `ids` (Mikro-Kern), `From`-Brücke in `compat/`.
**Abnahme:** `grep -rn 'volatile_vault' crates --exclude-dir=contextra-vault` nur Re-Exports; GL; R3.

---

### Welle 3: E1a-Bausteine

#### WP-30 · `mvcc` (S-30B)
**Vorauss.:** WP-10, WP-11 · **Aufwand:** 5 PT
1. **Zuerst** Zustand verifizieren (Klon zeigt `types`/`ports` bereits optional [V], SPEC nennt sie zwingend).
2. `MvccError` und `compat/error.rs` (`impl From<MvccError> for ContextraError`).
3. `types/src/tombstone.rs` ersatzlos entfernen (R9; nur `mvcc` und `types` nutzen ihn [Ü]), `tombstone_diff`-Test bis dahin behalten.
4. Loom-Tests (`loom_sequence_log`, `loom_snapshot_registry`, `loom_tx_buffer`) in `loom-gate.yml` (Pfadfilter) prüfen.
**Abnahme:** GL, GF (`mvcc/tests/golden_corpus.rs`), GC, GD (Tombstone), GE; R3.
**Risiko:** `vector` und `store` hängen an `mvcc` → Importpfade nur über `#[deprecated]`-Re-Export (INV-2).

#### WP-31 · `crypto`-Kern (S-31)
**Vorauss.:** WP-10, WP-11, WP-12 · **Aufwand:** 5 PT
1. `TenantId` → `ids`; `KeyManager::{derive_kv_key, derive_kv_key_scoped, cipher_for, cipher_for_scoped}` auf `[u8; 32]` statt `kv_cipher::ModelFingerprint` (**eigener Commit**, Signaturänderung).
2. `default = []` (kv-encryption raus); `types`/`ports` hinter `workspace`.
3. Known-Answer-Tests für AES-256-GCM-SIV, HKDF, Argon2id; WAL-HMAC-Golden.
**Abnahme:** GL, GF (WAL-HMAC, `MFKD`, KV v2), GC, GE; G-C1 Mutation ≥ Baseline; R3 für den Kern (KV-Teil folgt in WP-38).

#### WP-32 · `text` (S-32)
**Vorauss.:** WP-10, WP-11 · **Aufwand:** 8 PT
1. Lokales Trait `PostingStore` mit **genau** den in `src/` genutzten `StorageEngine`-Methoden (per `grep` erheben); Brücke implementiert es für `StorageEngine`.
2. `wand.rs` (≈20 Referenzen), `scanner.rs`, `index_struct.rs`, `purge.rs`, `morph_index.rs`: Imports auf `ids`, `env`, `TextError`; `ContextraError`, `MAX_SEARCH_K`, `TOMBSTONE_BIT` über lokale Konstanten, Gleichheitstest in `compat/`.
3. `wand`/`inverted`/`stream` **nicht mehr** hinter `types-compat`: Block-Max-WAND muss ohne `types` laufen.
4. Clock → `unix_ms`; ID-Breite im Postings-Header (SPEC §5.2).
5. Beispiel `standalone.rs` mit `MemoryTextIndex`: Dokumente indexieren, BM25-Suche, deutsches Kompositum.
**Abnahme:** GL, GF (Postings v-alt/neu), GD (ms), GE; Property-Test: WAND-Top-k == erschöpfender BM25-Scan; R3.

#### WP-33 · `rank`/Temporalfilter (S-33)
**Vorauss.:** WP-10, WP-11 · **Aufwand:** 2 PT
`engine/src/temporal_filter.rs` → `pub use` auf `rank::temporal` **nach** Differenztest (bitgleich); `types-compat` → `workspace`.
**Abnahme:** GL, GD.

#### WP-34 · `graph` (S-34)
**Vorauss.:** WP-10, WP-11, WP-12 · **Aufwand:** 15 PT
1. Brücke: `provider.rs`, `purger.rs`, `csr/graph_index/*`, alle Importe von `ports::{graph_index, graph}` nach `compat/`.
2. Kern-Importe (`hyperedge.rs:16`, `ppr/forward_push.rs:17`, `csr/types.rs:13`, `community.rs:22`, `path_rag/*`, `apprh/*`) auf `ids::{DocId, TxId, EntityId}` und `GraphError`. `ResourceTracker` lokal oder `env`, Entscheidung im ADR.
3. `PprConfig`, `PprAlgorithm`, `TlHfdParams` als Zielort `graph` vorbereiten (Verschiebung in WP-51).
4. `default = []`; Experimente Klasse A/B hinter `exp-*`; ID-Breite im bincode-Header.
5. Beispiel: CSR bauen, Forward-Push-PPR, Top-Knoten.
**Abnahme:** GL, GF (bincode v-alt/neu), GC (TSan), GE; Property-Test: PPR-Summe ≤ 1 + ε und deterministische Reihenfolge; R3.
**Risiko:** Größtes E1a-Paket (≈59 Dateien). In drei Commit-Paare teilen: Brücke, Kern-Importe, Features.

#### WP-35 · `checkpoint` (S-35)
**Vorauss.:** WP-10, WP-11 · **Aufwand:** 3 PT
`SnapshotSource` = genau die genutzten `StorageEngine`-Methoden; schmale Dateinaht statt `ports::Vfs` (`hardlink_cloner.rs`); `WorkflowState` lokal abbilden; Restreferenzen in `source.rs`, `store.rs`.
**Abnahme:** GL, GF (Meta/Manifest, `id_width_bits` bereits vorhanden [V]); R3.

---

### Welle 4: E1b-Bausteine (höchstes Risiko)

#### WP-36 · `vector` (S-36)
**Vorauss.:** WP-30, WP-10, WP-11 · **Aufwand:** 15 PT
Teilpakete, jeweils eigenes Commit-Paar:
- **36a** V-VETO-Wachtest `partial_rebuild_veto_f02_guard` **anlegen** (existiert nur in `SPEC.md`, SPEC B22) und grün schalten. Voraussetzung für alles Weitere.
- **36b** `VectorError` und `compat/error.rs`; `hnsw/types.rs:11`, `hnsw/arena.rs`, `quantize/mod.rs`, `quantize_rabitq.rs`, `core_rebuild.rs`, `core_search.rs`, `staging.rs`, `candidate_stream.rs` von `contextra_types` lösen (`ContextraError`, `DocId`, `Result`, `DistanceMetric`, `ScoredDocument`, `TxId`, `MAX_SEARCH_K`).
- **36c** `contract.rs` mit `types.rs` vereinen (heute je 2× `Clock`/`Rng`); `Arc<dyn contextra_ports::{Clock,Rng,MetricsSink}>` → `env`. `RwLock<contextra_mvcc::SequenceLog>` bleibt (`mvcc` ist Kern-Abhängigkeit laut SPEC; **kein** eigener `VisibilityLog`).
- **36d** Brücke: `hnsw/vector_index_impl.rs`, `diskann/vector_index_impl.rs`, `hnsw/purge.rs`, `hnsw/scanner.rs`; `HnswConnectivityDegraded` bleibt in `ContextraError` bis WP-70.
- **36e** Feature `simd` ersetzt `compute_distance` durch `contextra_simd::dispatch`; Skalarpfade bleiben als Orakel.
- **36f** ID-Breite: `HNSW_VERSION` 2 → 3 mit `id_width_bits`; erst Leser (v2 und v3), dann Schreiber (SPEC §5.2).
**Abnahme:** GL, GF (HNSW v2/v3), GD (Distanz, Rng, ms), GC, GE; **Recall-Regression:** Recall@10 gegen Brute-Force unverändert (Toleranz 0); HNSW-Benchmarks (`benchmarks/contextra-bench`) innerhalb der Schwelle; R3.
**Risiko:** Hotpath-Regression durch zusätzliche Indirektion (`dyn Rng`) → Benchmark **vor** 36c als Baseline; bei > Schwelle statische Generik statt `dyn` im Hotpath. Rollback je Teilpaket.

#### WP-37 · `store` (S-37)
**Vorauss.:** WP-30, WP-31, WP-10, WP-11 · **Aufwand:** 25 PT
- **37a** `StoreError` mit Abbildung der genutzten Varianten (`Storage`, `Io`, `WalCorruption`, `ChecksumMismatch`, `Transaction*`, `Conflict`, `WalTruncationDetected`, `CrossDeviceLink`), `compat/error.rs`.
- **37b** Brücke `engine/`, `scanner/` nach `compat/`.
- **37c** Imports (≈66 Dateien, u. a. `lsm/engine.rs`, `compaction/engine.rs`, `scanner/mod.rs`, `sstable/bloom.rs`, `wal/*`) → `ids`/`env`/`StoreError`; `tenant_codec` → `ids::TenantId`.
- **37d** `FeatureRing`-Entfernung **erst nach WP-50**; `validate_durability` bleibt.
- **37e** zuletzt Manifest v1 → v2 mit `id_width_bits`.
**Abnahme:** GL, GF, GC (Loom/TSan), **Crash-/Recovery-Suite unverändert grün (INV-6)**, GE; Fuzz-Ziele für SSTable- und WAL-Dekoder; R3 (Ausnahme Budget: 5 Kern-Mitglieder, SPEC §3.1).
**Risiko:** Durability. 37e hat Format-Auswirkung (einzige erlaubte INV-1-Ausnahme) und wird in einem **eigenen Release-Zweig** mit v-alt-Lesetest und Rückwärts-Migrationssperre dokumentiert (X8-Muster).

---

### Welle 5: KV-Paar und Inferenz

#### WP-38 · KV-Umzug als Paar (S-38)
**Vorauss.:** WP-31, WP-11 · **Aufwand:** 8 PT
1. `crypto/{kv_cipher, kv_segment/*, kv_shredding, revocation_log}` → `kvcache`; `kvcache`-Kern, Brücke (`prefix_store.rs`, `adapters.rs`, `KvCachePruner/Scanner`, `scratchpad_invalidation.rs`), Features (`kv-encryption`→crypto, `exp-kivi`, `exp-content-addressed`, `kvcache-attention-eviction` streichen).
2. `infer-candle/kv_bridge` umstellen.
3. **Im selben Commit:** `mutants-guarantees.toml`, `loom-gate.yml`, `CODEOWNERS` anpassen; Konstanten `CURRENT_KV_KEY_DERIVATION_VERSION` und `CURRENT_KV_FORMAT_VERSION` per `const_assert_eq` koppeln.
**Abnahme:** GL, GF (KV-Segment), GC (`loom_kv_deferred_zeroize` am neuen Ort grün), Mutation-Score G2 ≥ Baseline; R3.
**Risiko:** Gate-Pfade vergessen → CI testet stillschweigend nichts mehr. Deshalb alle drei Konfigurationsdateien in **einem** Commit.

#### WP-39 · `infer-ollama`, `infer-candle` (S-39)
**Vorauss.:** WP-11, WP-38 · **Aufwand:** 6 PT
1. `infer-ollama`: `OllamaError`, `Rng`/`Clock` aus `env`; `reqwest` bleibt hinter `client`; Circuit-Breaker-Zustandsautomat als Property-Test.
2. `infer-candle`: `candle-*`, `tokenizers`, `half` optional hinter `candle`; `ConfidenceCalibrator` lokal, `rank` nur noch Brücke (`compat/calibrator.rs`); `ModelFingerprint` als Newtype.
3. **Vorbedingung:** `grep -rn 'contextra_infer_candle::' crates` über alle Verbraucher, damit kein Symbol durch das Gaten verschwindet.
**Abnahme:** GL; `cargo tree -p contextra-infer-candle --no-default-features` ohne `candle-*` und ohne `rank`; `infer-ollama` R3, `infer-candle` R1.

---

### Welle 6: `ports`/`types` abgeben, E2/E3

#### WP-50 · `ports` abgeben (S-50)
**Vorauss.:** WP-22 · **Aufwand:** 4 PT
Lizenz-Traits → `license`; Erasure-Familie → `erasure`; Conformance → `testkit`. Re-Exports bleiben (R9).
**Abnahme:** GL; `grep -rln 'FeatureRing\|LicenseGate' crates/contextra-{store,vector,text,graph,kvcache,rank,checkpoint,mvcc}` leer; `ports_fan_in` sinkt (RATCHET).

#### WP-51 · `types` abgeben (S-51)
**Vorauss.:** WP-50, WP-34 · **Aufwand:** 3 PT
`types/src/erasure.rs` → `erasure`; `PprConfig`, `PprAlgorithm`, `TlHfdParams` → `graph`. Veraltete deprecated Typen (`ErasureId`, `GateId`) mit Fristplan.
**Abnahme:** GL, GE; `types_fan_in` sinkt.

#### WP-60 · Regler (S-60)
**Vorauss.:** WP-21 · **Aufwand:** 4 PT
`adapt` nimmt die fehlenden Methoden auf (decay 5, homeostat 3, pid 5); `engine`/`db` re-exportieren per `pub use` **nach** bitgleichem Differenztest (f64, `ManualClock`).
**Abnahme:** GD.

#### WP-61 · Chunker (S-61)
**Vorauss.:** WP-01 · **Aufwand:** 4 PT
Paritätstest `engine::chunker` ↔ `ingest::chunking::prose` (identische Chunk-Folge und Breadcrumbs), danach Adapter; `contextra::chunker::MarkdownChunker` API-stabil.
**Abnahme:** GD; `mcp`-Tests unverändert.

#### WP-62 · `router` (S-62R)
**Vorauss.:** WP-01 · **Aufwand:** 2 PT
`ports_local.rs` entfernen, nachdem die drei `mcp`-Importe auf `contextra_ports` umgestellt sind; `Vfs`-Stelle auf lokale `&Path`-Funktion.
**Abnahme:** GL.

#### WP-63 · `cognition` und `agent` invertieren (S-63)
**Vorauss.:** WP-37, WP-50 · **Aufwand:** 10 PT
1. `cognition`: Brücke ins Crate `db` (Feature `cognition`); lokale Ports `DocumentSource`, `ConsolidationSink`, `HyperedgeSink`, `DecayPolicy`; `CognitionError`; **`engine_local.rs`/`graph_local.rs` löschen** (verschleiern `Collection`, `AdaptiveDecayController`, `detect_communities`, `CsrGraph`).
2. `agent`: lokale Ports `ContextStore`, `ScratchpadStore`; `try_from_db(&contextra_db::Contextra)`/`from_db` (`engine.rs:89,112`) und `Arc<contextra_router::DefaultRouterEngine>` (`context.rs`) hinter Brücke/Feature `routing`.
3. **`TokenBudget` vereinheitlichen** (`agent::budget` ↔ `contextra_types`): ein Besitzer, Differenztest.
4. **Spezifikationslücke Context Compaction:** Truncate/Summary/Selective sind im Quelltext nicht als Strategien auffindbar. Entscheidung per ADR: implementieren (in `cognition`, rein auf Slices, ohne `Collection`) **oder** aus der Spezifikation streichen.
**Abnahme:** GL; `cargo tree -p contextra-cognition -e normal` ohne `contextra-engine`; Kern-Closure laut SPEC (`types`, `ports`); `detect_near_duplicates`, GMM und `transitivity_veto` per Unit-/Property-Test ohne `Collection`.

#### WP-64 · `engine`, `db`, `contextra`, `mcp`, `py`, `testkit` (S-64)
**Vorauss.:** WP-50, WP-51, WP-33 · **Aufwand:** 8 PT
1. `engine`: Lizenzumstellung (Feature `license`), `extraction/` → `exp-extraction` (ohne Zusammenlegung mit `graph/entity_extraction.rs`).
2. `contextra`: ohne Feature `license` `OpenFastGate`-Semantik per Test.
3. `mcp`: Protokollkern (`io.rs`, `server_dispatch.rs`, `validation.rs`: 0 Workspace-Referenzen [V]) markieren und in eigenem Modul bündeln (Vorbereitung für ADR-Split).
4. `py`: `docid-128 = ["contextra-ids/docid-128"]`, `maturin` mit und ohne `pyo3` getrennt prüfen. `testkit`: `erasure-conformance` aus Default.
**Abnahme:** GL; G-A1 ohne unbeabsichtigte API-Diffs.

---

### Welle 7: Aufräumen und Experimente-Crates

#### WP-70 · Aufräumen (S-70)
**Vorauss.:** WP-30…WP-64 · **Aufwand:** 3 PT
Deprecated Re-Exports entfernen (R9), nicht mehr genutzte `ContextraError`-Varianten entfernen (nur bei grünem G-E1), `archive/`-Verschiebungen bestätigen, „Klasse A/B/C“-Begriffe in `docs/` bereinigen.
**Abnahme:** alle Gates; Fan-in-Ratchet auf Zielwert.

#### WP-70a · Nachprüfung der Erasure-Umzüge (S-70a, B23)
**Vorauss.:** WP-00 · **Aufwand:** 3 PT
Item-Diff `archive/contextra-crypto-proofs` ↔ `erasure`; Mutation-Score der Zieldateien ≥ Quelldateien; sonst Wiederherstellung hinter `legacy-erasure` (nicht Default).
**Abnahme:** G-C1 (G2/G3 ≥ Baseline).

#### WP-80 · `contextra-vector-diskann` (S-80)
**Vorauss.:** WP-36 · **Aufwand:** 6 PT
`vector/diskann/` (`search.rs`, `format.rs`, `integrity.rs`, `candidate_stream.rs`-Anteile) in eigenes Crate; `db`-Feature `experimental-diskann` zeigt dorthin; DiskANN-Formate (`DANN`, `DANF`, `PWAL`, `TWAL`) per Golden abgesichert.
**Abnahme:** GL, DiskANN-Tests isoliert grün.

#### WP-81 · `contextra-graph-research` (S-81)
**Vorauss.:** WP-34 · **Aufwand:** 5 PT
Experimente Klasse A/B (Perkolation, Edge-RL, APPRH nach Messung) auslagern, wenn > 2.000 Zeilen (X4).
**Abnahme:** GL, GD.

#### WP-ADR · Über die SPEC hinausgehende Splits (optional)
**Vorauss.:** WP-64 · **Aufwand:** nur Entscheidung (2 PT)
Meine Kandidaten aus dem ersten Bericht (`store-lsm`/`store-wal`, `audit-chain`, `mcp-core`, `graph-core`/`graph-rag`) nur dann per ADR beantragen, wenn nach WP-64 Score M oder Entnehmbarkeit es **messbar** verlangt. Bis dahin gilt die SPEC.

---

### Welle 8: Release und Zusammenstecken

#### WP-90 · Release-Engineering
**Vorauss.:** WP-70 · **Aufwand:** 5 PT
1. Cargo-Metadaten, `CHANGELOG.md`, Versionierungsregel (Bausteine unabhängig versioniert, `0.x` bis R3 für alle) und `publish`-Entscheidung je Crate dokumentieren (heute `publish = false` bei 1 von 39 Manifesten [V]).
2. Publikationsreihenfolge topologisch (E0a → E0b → E1a → E1b), `cargo publish --dry-run` je Crate.
3. MSRV-, `deny`-, `audit`-, `semver-checks`-Jobs blockierend; docs.rs-Features je Crate (`[package.metadata.docs.rs]`).
**Abnahme:** `certify_crate.py` ergibt R3 für alle E0/E1-Crates (außer `frozen`: R1).

#### WP-91 · Referenzkompositionen und Baustein-Katalog (Ergänzung)
**Vorauss.:** WP-36, WP-32, WP-21 · **Aufwand:** 6 PT
Der Beleg, dass sich Produktivlösungen „mit wenig Implementierung“ zusammenstecken lassen. Drei eigenständige Cargo-Projekte unter `examples/compose/` (nicht im Workspace, nur Path-Abhängigkeiten, `--no-default-features`):
1. **`rag-mini`:** `ids` + `env` + `vector` + `text` + `rank`: indexieren, hybride Suche, RRF. `main.rs` ≤ 80 Zeilen, deterministische Ausgabe (Golden-Stdout).
2. **`privacy-gateway`:** `privacy` + `prompt-guard`: PII ersetzen, Injection prüfen, Antwort rehydrieren.
3. **`agent-routing`:** `adapt` + `agent` (via Port-Fakes): Modellauswahl mit LinUCB und PID-Regler.
Zusätzlich: generierter Katalog `docs/bausteine/README.md` aus `capabilities.toml` und den Zertifikaten (Crate, Ebene, Kern-Closure, Reifegrad, Beispiel, Features).
**Abnahme:** Jede Komposition baut in CI mit **nur** den genannten Crates (`cargo tree` ⊆ Liste), läuft und vergleicht ihre Ausgabe; die Anzahl eigener Zeilen ist ein erfasster Messwert.

---

## 6. Risiken

| Risiko | Wirkung | Gegenmaßnahme |
|---|---|---|
| SPEC-Zahlen veraltet (HEAD-Abstand, §1.4) | Falscher Aufwand, überflüssige WPs | WP-00 misst neu; Aufwand pro WP nach WP-00 nachschätzen |
| Format-Versionsanhebung (WP-36f, WP-37e) | Datenverlust, Nicht-Rückwärtskompatibilität | Leser vor Schreiber, v-alt-Lesetest, Release-Zweig, Hash-Vergleich nach abgelehntem Öffnen (SPEC §5.2) |
| HNSW-/WAND-Hotpath-Regression | Performanceverlust | Benchmark-Baseline vor 36c, Schwellenwert, statische Generik statt `dyn` im Hotpath |
| Blanket-Impl-Überlappung (E0119) | Compilefehler bei zwei Brücken | SPEC §4.3 e), Test mit beiden Features gleichzeitig (WP-13) |
| Cargo-Feature-Unification (`docid-128` nicht additiv) | Falsche ID-Breite im Gesamtbuild | CI-Matrix mit/ohne Feature (WP-01), Breitenprüfung beim Öffnen |
| Gate-Pfade nach Umzug vergessen (WP-38) | CI prüft stillschweigend nichts | Konfigurationsdateien im selben Commit, Negativtest |
| `public-api`/Toolchain-Konflikt | G-A1 nicht ausführbar | ADR in WP-00; Alternative `cargo-semver-checks` als Ersatzgate |
| Mutation-Baselines driften | G-C1 wertlos | Baselines in WP-00 einfrieren, nur per ADR ändern (INV-10) |
| Review-Engpass durch Commit-Paare | Verzug | Spuren nach Crate trennen, CODEOWNERS pro Spur |

## 7. Spuren und Besetzung (Vorschlag)

| Spur | Pakete | Profil |
|---|---|---|
| **A Plattform** | W0, WP-12, WP-13, WP-90, WP-91 | Build/CI, Tooling |
| **B Kern-Retrieval** | WP-30, 32, 33, 34, 35, 36, 80, 81 | Index-/Graph-Algorithmik |
| **C Speicher & Krypto** | WP-31, 37, 38, 70a | Durability, Kryptografie |
| **D Bausteine & Integration** | W2, WP-39, 50, 51, 60–64, 70 | Breite, API-Design |

Spur B und C treffen sich an `mvcc` (WP-30) und `store` (WP-37); dort ist die Abstimmung bindend.

## 8. Nicht belegt

1. Alle Aufwandszahlen sind Schätzungen aus LOC und Referenzzahlen.
2. Ob die Kerncrates ohne Features bauen, ist bis WP-00 nur abgeleitet.
3. Der tatsächliche Fortschritt von S-30/S-31 gegenüber der SPEC ist offen (§1.4).
4. Eine Implementierung der Context-Compaction-Strategien wurde im Quelltext nicht gefunden (WP-63).
