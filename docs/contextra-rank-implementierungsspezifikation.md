# contextra-rank — Implementierungsspezifikation (SOLL-Zustand)

**Stand:** 17. Nov 2026 · **Geprüfter Klon:** `tfufuz1/contextra`, HEAD `24487fa16` · **Normative Quelle:** `docs/SPEC.md` v6 (16. Nov 2026) · **Zielpfad im Repo (Vorschlag):** `crates/contextra-rank/IMPLEMENTATION_SPEC.md`

---

## 0 Lesehilfe

| Marke | Bedeutung |
| --- | --- |
| **[V]** | Am Klon per Datei, `grep`, `diff` oder `sha256sum` geprüft. Nichts gebaut. |
| **[S]** | Aus `docs/SPEC.md` v6 übernommen (Paragraph in Klammern). |
| **[E]** | Einschätzung. Nicht belegt; das Prüfverfahren steht daneben. |
| **[D]** | Festlegung dieser Spezifikation. Wo eine Entscheidung nach SPEC ein ADR braucht, steht „ADR nötig“. |

Begriffe (Kern, Brücke, Kern-Closure, E1a, Gates GL/GH/GF/GD/GE/GC) wie in `docs/SPEC.md` §2, §7, §9.1. Jede MUSS-Aussage nennt ihr Prüfverfahren (SPEC §1.3).

**Grenzen dieser Arbeit.** `cargo` und `rustc` waren nicht verfügbar; es wurde nichts gebaut, getestet oder ausgeführt. Außerdem fehlt dem Klon die Workspace-Datei `Cargo.toml` (siehe F-R01); ein Build wäre ohnehin nicht möglich gewesen. Alle Zahlen sind Näherungen aus statischer Analyse. Die Schritte RK-00 und RK-01 holen die Messung nach.

---

## 1 Kurzfassung

`contextra-rank` ist laut SPEC ein **E1a-Baustein**: Kern-Closure `{ids, env}`, Default `std`, Features `workspace`, `exp-dibud`, `exp-resonance`, Fehlertyp `RankError`, keine dev-Kante auf `types`/`ports`, `DocId` und `Clock` ohne eigene Kopie [S §4.1, §5, R4, R12, S-33].

Der Crate ist funktional weit: 0 ungegatete Brückenreferenzen in `src/` (die einzige Textstelle ist ein Doc-Kommentar, `lib.rs:25`), kein `unwrap`/`expect`/`panic!` in `src/`, `forbid(unsafe_code)`, vier Golden-Dateien mit gültigen Hashes [V]. Zum SOLL fehlen acht Dinge:

1. **Kern-Closure ist leer statt `{ids, env}`.** `DocId`, `TxId` und gleich **zwei** `Clock`-Traits (`contract.rs`, `calibration/isotonic.rs`) sind lokal kopiert (R4-Verstoß).
2. **Default-Features sind verboten** (`exp-dibud`, `exp-resonance`, `experimental-rank` im Default; R10).
3. **`ResonanceConfig` ist nicht additiv:** ohne `exp-resonance` ist es ein anderer Typ (`{enabled}`) als mit (`{beta, gamma}`), und es steht in den Signaturen der Kern-Fusionsfunktionen (F-R04).
4. **dev-Abhängigkeiten auf `types` und `ports`** (R12), mit Default-Features und ohne Gating.
5. **Feature `dibud` ist ein Altname-Alias**, aber `lib.rs` gated noch auf ihn (`cfg(feature = "dibud")`), nicht auf `exp-dibud`.
6. **Temporalfilter-Ablösung** (S-33) zeigt auf eine Datei, die am HEAD nicht mehr existiert: die Kopie liegt jetzt in `contextra-db`; ein Engine-Test verweist auf den alten Pfad (F-R05).
7. **Dokumentation widerspricht dem Manifest** (README, SPEC, REPORT behaupten `workspace` im Default; `Cargo.toml` sagt etwas anderes), und `docs/decoupling/errors/rank.md` fehlt.
8. **Determinismus-Lücke (Hypothese):** `DocId::from_key` und `ConfigFingerprint::new` hashen mit `ahash::AHasher::default()`. Mit `runtime-rng` sind diese Schlüssel vermutlich pro Prozess zufällig (F-R07, [E], Test in RK-00).

Der Plan hat **12 Schritte (RK-00 bis RK-11)**, je als Commit-Paar A/B (INV-3), in vier Wellen. Zwei Entscheidungen blockieren Schritte und brauchen ein ADR: **D-R01** (`DocId`-Semantik, entspricht D-20) und **D-R02** (additives `ResonanceConfig`).

---

## 2 SOLL-Zustand (normativ, abgeleitet aus SPEC v6)

Jede Zeile ist eine prüfbare Anforderung. „Gate“ nennt das Prüfverfahren.

| ID | Anforderung (MUSS) | Quelle | Gate / Prüfung |
| --- | --- | --- | --- |
| **A-01** | Ebene **E1a**; Kern-Closure ⊆ E0, höchstens 4 Mitglieder; Ziel genau `{contextra-ids, contextra-env}` (2). | §2.2, §4.1 | `cargo tree -p contextra-rank -e normal --no-default-features` |
| **A-02** | Interne Kerndeps erhalten `default-features = false` (sonst zieht deren Default die Brücke in die Closure). | §2.1 (F-07) | G-L4 (`isolation_proof.py`) |
| **B-01** | `default = ["std"]`. Nie im Default: `workspace`, `exp-*`, `experimental-*`, `*-compat`, `legacy-*`. | R10 | G-H1 (`feature_audit.py`) |
| **B-02** | Features: `workspace`; `exp-dibud`; `exp-resonance`; Sammelfeature `experimental-all`. `*-compat` nur als Alias bis S-70. Features ohne `cfg` und ohne Weiterleitung entfallen. | R10, §4.1 | G-H1, `cargo hack check --each-feature --no-dev-deps` |
| **B-03** | Experimente sind additiv: ein Feature fügt Code hinzu, ändert aber keine Typform und keine Signatur. | X1–X3, R5 | `cargo hack`, Test D-04 (§8) |
| **C-01** | Ein Fehlertyp `RankError` (thiserror, `#[non_exhaustive]`); `From<RankError> for ContextraError` nur in `compat/`; keine Sammelvariante `Other(String)` ohne `#[source]`. | R3, §6.3 | G-E1 |
| **C-02** | Tabelle `docs/decoupling/errors/rank.md` mit Variante, `ContextraError`-Entsprechung, Display-String; Display bitgleich. | §6.3 | G-E1, tabellengetriebener Test |
| **D-01** | `contextra_types`, `contextra_ports`, `contextra_testkit` kommen im Kern nirgends vor, auch nicht in Kommentaren oder `#[cfg(test)]`. Brücke nur in `src/compat/` hinter `workspace`. | R2, R7 | G-L2 (`gate_bridge_null.py`) |
| **E-01** | Keine dev-Kante von `rank` auf `types`, `ports`, `testkit`. Solche Tests liegen unter `testkit/tests/rank/` oder hinter `required-features = ["workspace"]`. dev-Kanten auf E0 (`env`, `ids`) und externe Crates sind erlaubt. | R12, D-12, S-12b | G-L1 (normal und dev), G-H1 |
| **F-01** | `DocId` und `TxId` kommen aus `ids`; `Clock` und `SystemClock` aus `env`. Keine zweite Definition in `rank`. | R4, S-33, S-10C, S-11 | `symbol-uniqueness`, Differenztests |
| **F-02** | `rank` führt `id_width_bits` nur, wenn ein persistiertes Format `DocId` enthält. Ob das so ist, entscheidet der Breitentest, nicht eine Liste. | §6.2 | Breitentest (RK-02) |
| **G-01** | `dibud` (`exp-dibud`) und `fusion/resonance.rs` (`exp-resonance`) laufen hinter ihren Features; die Naht liegt in `fusion/ext.rs` ohne `types`/`ports`; je Experiment eine Verhaltens-Baseline im Crate; Tier `experimental` in `capabilities.toml`. | X1–X8, §9.4 | `feature_audit.py`, G-L1 je Feature |
| **G-02** | `drift` bleibt im Crate (keine Dublette zu `adapt::drift`). | §9.4, B16 | Review |
| **H-01** | Bausteinvertrag: `Cargo.toml` (`rust-version`, `publish` nach R14, `version` an Pfad-Deps), `README.md` (Einsatzbeispiel ≤ 10 Zeilen ohne weitere `contextra-*`), `examples/`, `SPEC.md`, `capabilities.fragment.toml` (Tier, Ebene, Unterebene), Kern-Tests ohne Brücke. | §5.4 | G-DOC1, Review |
| **H-02** | `publish = false` nur mit ADR. `rank` hat Entnehmbarkeitsanspruch. | R14 | G-H1 |
| **I-01** | `engine::temporal_filter` wird `pub use` auf `rank::temporal`, nach grünem Differenztest „Filterausgaben bitgleich“. (Am HEAD liegt die Kopie in `db`, siehe F-R05.) | S-33, §6.4 | G-D1 |
| **J-01** | Persistierte Formate (`Calibrator`-Container: `platt_v1`, `isotonic_v1`, `conformal_v1`) haben einen Golden-Korpus; kein Byte ändert sich. | INV-1, §6.1 | G-F1 |
| **J-02** | Rangfolge deterministisch: Score absteigend, Tie-Break `id` aufsteigend; gilt mit `ManualClock`. | INV-5 | G-D1 (`determinism_guard.py`) |
| **K-01** | Kein `unwrap`/`expect`/`panic!`/`todo!` in Nicht-Test-Code; `forbid(unsafe_code)`. | INV-11, INV-12 | `unwrap-gate`, `unsafe_islands.rs` |
| **L-01** | Entnehmbar nur nach G-L2 → G-L1 → G-L3 → G-L4 → G-H1 (in dieser Reihenfolge), M = 100, Beispiel kompiliert ohne weitere `contextra-*`. | §8.4, R6 | `REPORT.md` mit HEAD-Hash |

---

## 3 IST-Zustand und Delta

Alle Angaben [V] am Klon, sofern nicht anders markiert.

### 3.1 Struktur und Umfang

| Merkmal | IST |
| --- | --- |
| Quelltext | 31 Dateien, ca. 4.400 Zeilen in `src/`; größte Datei `fusion/rrf.rs` (753) |
| Module | `calibration/{isotonic,platt,conformal}`, `fusion/{rrf,mrrf,normalized,global,provenance,signal,topk,types,ext,resonance}`, `dibud/{driver,state,types}`, `drift`, `explain`, `metrics`, `temporal`, `local_types`, `contract`, `error`, `compat/{clock,error,types}` |
| Experimente | `dibud/` ≈ 518 Zeilen, `fusion/resonance.rs` 96, `fusion/ext.rs` 37; weit unter X4 (2.000 Zeilen) → bleibt im Crate |
| Tests | 21 Integrations-Testdateien, Golden-Korpus `tests/golden/` mit `MANIFEST.sha256` (4 Einträge, alle `OK`) |
| `unsafe` | `#![forbid(unsafe_code)]`, `#![deny(missing_docs)]` |
| Panics in `src/` | `grep` nach `unwrap()`, `expect(`, `panic!`, `todo!`: **0 Treffer** |
| Nichtdeterminismus | `SystemTime` nur in `SystemClock` (mit `ALLOW-DETERMINISM`) |
| Ausgabereihenfolge | Hash-Map-Iteration wird vor der Ausgabe sortiert (`rrf.rs:114`, Score absteigend, `id` aufsteigend) |

### 3.2 Abweichungsmatrix

| ID | Merkmal | SOLL | IST | Delta |
| --- | --- | --- | --- | --- |
| Δ1 | Kern-Closure | `{ids, env}` | leer (`Cargo.toml`: nur `types`/`ports` optional) | `ids`, `env` ergänzen |
| Δ2 | `Clock` | `env::Clock` | **2 lokale Traits** (`contract.rs:7`, `calibration/isotonic.rs:14`) plus lokale `SystemClock` | beide entfernen, Re-Export |
| Δ3 | `DocId` | `ids::DocId` | lokales `DocId(pub u64)` mit abweichender `from_key` (Zahl-/`doc_`-Präfix-Parsing, sonst `AHasher`) | D-R01 |
| Δ4 | `TxId` | `ids::TxId` | lokales `TxId(pub u64)` (`local_types.rs:187`); `ids::TxId` existiert (`ids/src/lib.rs:407`) | ersetzen |
| Δ5 | Default | `["std"]` | `["std", "exp-dibud", "exp-resonance", "experimental-rank"]` | flippen, Konsument `db` anpassen |
| Δ6 | Feature-Gating | `exp-dibud` | `lib.rs:71,78` gated auf Alias `dibud` | `cfg` auf Zielnamen |
| Δ7 | Additivität | Typ unabhängig vom Feature | `ResonanceConfig` hat zwei Gestalten (`ext.rs:14-29` vs. `resonance.rs`) | D-R02 |
| Δ8 | dev-Kanten | keine auf `types`/`ports` | `[dev-dependencies]` führt beide, ohne `default-features = false` | gaten oder verschieben |
| Δ9 | Fehlertypen | ein `RankError` | `RankError` **und** `ConformalError` (`conformal.rs:14`); `ConformalError` hat keine Abbildung in `compat/error.rs` | integrieren oder dokumentieren |
| Δ10 | Fehlerdoku | `errors/rank.md` | Datei und Verzeichnis `docs/decoupling/errors/` fehlen | anlegen |
| Δ11 | `std` | Weiterleitung oder entfernen | `std = []`, ohne `cfg(feature = "std")` im Quelltext | nach Δ1 auf `ids/std`, `env/std` weiterleiten |
| Δ12 | Aliase | als Alias gekennzeichnet | `dibud`, `types-compat`, `ports-compat`, `experimental-rank` ohne Kennzeichnung | kennzeichnen, in S-70 entfernen |
| Δ13 | `publish` | ADR oder entfernen | `publish = false` ohne ADR | D-R06 |
| Δ14 | `capabilities.fragment.toml` | `level`, `sublevel`, `core_deps`, Tier je Experiment | `sublevel`, `core_deps` fehlen | ergänzen |
| Δ15 | R4 `MAX_SEARCH_K` | ein Besitzer je Konzept | `rank::MAX_SEARCH_K = 10_000`, `ids::MAX_SEARCH_K = 1_000` | D-R03 |

### 3.3 Öffentliche Verbraucher (für die Migrationsmatrix, §9)

| Verbraucher | Nutzung von `rank` [V] | Features |
| --- | --- | --- |
| `contextra-db` | `fusion::{…}`, `FusionWeights`, `IsotonicCalibrator`, `SearchResult`, **`fuse_signals`** (nur mit `exp-resonance`), Temporalfilter-Kopie (383 Zeilen) | `features = ["workspace"]`, verlässt sich auf den Default für `exp-resonance` |
| `contextra-mcp` | `fusion::ProvenanceRecord`, `IsotonicCalibrator` | Default |
| `contextra-cognition` | `pub use fusion::{ProvenanceRecord, SearchResult}` | Default |
| `contextra-infer-candle` | `IsotonicCalibrator`, `ConfigFingerprint::from_bytes` | optional, `default-features = false` (+ dev-Kante mit Default) |
| `contextra-engine` | in `src/` keine Referenz; Test `proof_temporal_filter_vs_rank.rs` | Default |
| `contextra-bench` | `rrf_fuse`, `weighted_reciprocal_rank_fusion`, `score_normalized_fusion_with_options`, `ScoredItem`, `SearchResult` | Default |

Kein Verbraucher importiert `contextra_rank::DocId`, `TxId`, `PortsClockAdapter`, `IsotonicPortsClockExt`, die `*_types_compat`-Funktionen, `dibud` oder `ResonanceConfig` direkt (außer `db` über `fuse_signals`). [V, `grep` über `crates/` und `benchmarks/`]

---

## 4 Befunde

| ID | Befund | Beleg | Folge |
| --- | --- | --- | --- |
| **F-R01** | **Die Workspace-Datei `Cargo.toml` fehlt im Klon.** Der letzte Commit `24487fa16` („Shell-Commit“) löscht sie (236 Zeilen). `Cargo.lock` ist vorhanden. | `git show --stat HEAD -- Cargo.toml`; Vorversion mit `git show HEAD~1:Cargo.toml` lesbar | Kein Build, kein `cargo metadata`, alle Gates scheitern mit Exit 2. **Blockiert RK-00.** Wiederherstellen ist nicht Teil dieses Crates; es ist Vorbedingung. |
| **F-R02** | Zwei `Clock`-Traits im selben Crate. `contract::Clock` wird nirgends im Crate genutzt, `calibration::Clock` ist die gelebte. | `contract.rs:7`, `isotonic.rs:14`; `grep Clock src` | R4-Verstoß; trivial zu bereinigen, aber öffentlicher Pfad `contextra_rank::contract::Clock` bleibt zu halten (INV-2). |
| **F-R03** | `env::Clock` unterscheidet sich vom lokalen Trait: `monotonic_nanos` **ohne Default-Impl**, Bound `Send + Sync + 'static`. SPEC D-10 will `'static` streichen (S-11). `env::SystemClock::monotonic_nanos` nutzt `Instant`, die lokale Fassung die Wanduhr. | `env/src/clock.rs`; `isotonic.rs:19,45` | Eigene Implementierer müssen `monotonic_nanos` ergänzen. Im Crate wird `monotonic_nanos` nur in einem Test aufgerufen (`isotonic.rs:415`), die Produktionspfade nutzen nur `now_unix_nanos`; ein Verhaltensunterschied ist dort nicht zu erwarten [E; bestätigt der Differenztest DT-1]. |
| **F-R04** | **Experiment ist nicht additiv.** `ResonanceConfig` hat ohne `exp-resonance` das Feld `enabled`, mit dem Feature `beta`/`gamma`. Der Typ steht in den Signaturen von `rrf.rs`, `mrrf.rs`, `normalized.rs` (`Option<&ResonanceConfig>`). | `ext.rs:14-29`, `resonance.rs`, `rrf.rs:202,661,742`; Befund 1 in `docs/decoupling/FINDINGS.md` | Ein Default-Flip (Δ5) würde Code brechen, der `ResonanceConfig { beta, gamma }` baut. Verletzt B-03 und `cargo hack --each-feature` für die Integrationstests. |
| **F-R05** | **SPEC-Erratum:** S-33 nennt `engine::temporal_filter` (`temporal_filter.rs`). Am HEAD liegt die Datei in `contextra-db/src/temporal_filter.rs`, `engine/src/` kennt sie nicht mehr. `contextra-engine/tests/proof_temporal_filter_vs_rank.rs` importiert noch `contextra_engine::temporal_filter` (nicht auflösbar). Die db-Kopie unterscheidet sich von `rank::temporal` nur in `TxId`-Herkunft, `Clock`-Trait, Doc-Kopf und zwei `#[allow]`. | `diff crates/contextra-rank/src/temporal.rs crates/contextra-db/src/temporal_filter.rs` | Die Ablösung wird in `db` umgesetzt. Der Engine-Beweistest muss umgezogen oder entfernt werden. Nach RK-07/RK-08 entfallen die Typdivergenzen, die `engine/docs/.../TEMPORAL_PARITY.md` als Hindernis nennt. |
| **F-R06** | Dokumentation widerspricht dem Manifest. README: `workspace` und `dibud` seien Default. Crate-`SPEC.md`: Default `std, types-compat, experimental-rank`. `REPORT.md`: `workspace` bleibe im Default. `Cargo.toml`: `default = ["std", "exp-dibud", "exp-resonance", "experimental-rank"]`, **kein** `workspace`. Außerdem zitiert die Crate-`SPEC.md` „docs/SPEC.md v2“, Ring-Begriffe und die alte HEAD-Zeile `f0eda1cb9`. | Dateien lesen | G-DOC1 würde anschlagen; RK-01. |
| **F-R07** | **Hypothese (Determinismus):** `DocId::from_key` (Fallback) und `ConfigFingerprint::new` hashen mit `ahash::AHasher::default()`. `Cargo.lock` führt für `ahash 0.8.12` die Abhängigkeit `getrandom`, also ist `runtime-rng` aktiv. Dann sind die Schlüssel pro Prozess zufällig. Das widerspräche INV-5 und machte jeden persistierten Wert aus diesen Pfaden unbrauchbar. | `local_types.rs:15-32,78-96`; `Cargo.lock` Zeilen 57-68; `Cargo.toml`: `ahash = { version = "0.8", features = ["serde"] }` | **[E]** Nicht ausgeführt. Prüfung in RK-00: zwei Prozesse, gleicher Schlüssel, Ausgabe vergleichen. Bestätigt sich die Hypothese, vereinfacht das D-R01, weil für nicht-numerische Schlüssel kein stabiler Altwert existiert. |
| **F-R08** | `ConformalError` ist ein zweiter öffentlicher Fehler-Enum und fehlt in `compat/error.rs`. | `conformal.rs:14`; `compat/error.rs` | R3/G-E1: eine Abbildung fehlt oder die Entscheidung „kein eigener Typ“ fehlt (D-R05). |
| **F-R09** | Alle drei `RankError`-Varianten werden in `compat/error.rs` auf **dieselbe** Variante `ContextraError::InvalidInput` abgebildet. Display bleibt bitgleich, die Information „Kalibrierung vs. Fusion“ geht an der Grenze verloren. | `compat/error.rs` | Kein Fehler im Sinne von §6.3 (jede Variante hat eine Entsprechung), aber dokumentationspflichtig in `errors/rank.md`. |
| **F-R10** | `Cargo.toml` führt `include = ["src/**/*", …]` ohne `benches/`, `examples/`, `tests/`, deklariert aber `[[bench]] rank_bench`. | `Cargo.toml` | **[E]** `cargo package` scheitert vermutlich an dem fehlenden Bench-Ziel; relevant für §8.4 Punkt 6 (SOLL). |
| **F-R11** | `MAX_SEARCH_K` existiert in `rank` (10.000) und `ids` (1.000) mit verschiedenen Werten. | `local_types.rs:6`, `ids/src/lib.rs:34` | Das Gate `symbol-uniqueness` meldet das vermutlich [E]; D-R03. |
| **F-R12** | Die Golden-Datei `id_serde.json` prüft die serde-Form von `DocId` und passt nach dem Wechsel auf `ids::DocId` nur bei gleicher Newtype-Serialisierung (Ableitung `Serialize` auf Tupel-Struct, [V] in `ids`). Im 128-Bit-Build ist `DocId` ein `u128`; `ids::DocId::as_u64()` liefert dann nur die unteren 64 Bit. | `ids/src/lib.rs:222-298` | Breitentest in RK-02; `rank` darf `as_u64()` nicht in Logik nutzen (RK-08). |
| **F-R13** | `AGENTS.md` im Crate enthält generierte Inventarwerte (z. B. „Interne Dev/Build-Abhängigkeiten (2)“, 4.440 LOC), die nach RK-04/RK-05 veralten. | `AGENTS.md` | Regenerieren mit `tools/crate_inventory.py agents`. |
| **F-R14** | Der Bericht `docs/decoupling/REPORT.md` meldet „GRÜN“ für Gates, die laut SPEC §7.3 nirgends in CI laufen, und „keine Risiken“. Er nennt keinen HEAD-Hash. | `REPORT.md` | Wird durch den Bericht aus RK-10 ersetzt (§8.4 verlangt HEAD-Hash und Datum). |

---

## 5 Entscheidungen (Vorschläge)

Jede Entscheidung ändert sich nur per ADR in `docs/decisions/` (SPEC §10).

### D-R01 — `DocId`-Semantik (entspricht D-20, **ADR nötig, blockiert RK-08**)

**Lage.** `rank::DocId::from_key` rechnet anders als `ids::DocId::from_key`: erst `u64`-Parsing, dann Präfix `doc_`/`doc-`, sonst `AHasher`. `ids` nimmt BLAKE3-Präfix und liefert `IdError`. Kein Verbraucher im Workspace ruft `rank::DocId::from_key` auf [V]; im Crate selbst wird die Funktion nicht genutzt, nur der Typ in `dibud` und `ProvenanceRecord::synthesized_from`.

| Option | Wirkung | Risiko |
| --- | --- | --- |
| **(a)** Typ **und** Semantik von `ids` übernehmen | `from_key("42")` liefert nicht mehr 42. Der Rückgabetyp wechselt von `Result<_, String>` auf `Result<_, IdError>`. | Bricht externe Aufrufer (im Workspace keine bekannt). Datenneuaufbau nach D-16 nur nötig, wenn extern persistierte Schlüssel aus `rank` stammen. |
| **(b)** Typ aus `ids`, Altfunktion als freie Funktion `rank::legacy::doc_id_from_key(&str) -> Result<ids::DocId, RankError>` | Zahlen- und Präfixfälle bleiben bitgleich; Rest folgt `ids` (deterministisch). | Zwei Wege zum selben Typ; die Funktion lebt bis S-70. |

**Empfehlung [D]: (b), mit Einschränkung.** Die Zahlen- und Präfixzweige bleiben bitgleich zur Legacy-Funktion. Der `AHasher`-Zweig wird **nicht** erhalten, sondern auf `ids::DocId::from_key` umgeleitet, weil er unter F-R07 vermutlich ohnehin nicht reproduzierbar ist. Das ist eine Verhaltensänderung für nicht-numerische Schlüssel und gehört deshalb ausdrücklich ins ADR. Ergibt RK-00, dass F-R07 **nicht** zutrifft (Schlüssel stabil), dann gilt stattdessen: Zweig bitgleich erhalten und `from_legacy_key` im Crate führen (D-20 Option b vollständig).

### D-R02 — Additives `ResonanceConfig` (**ADR nötig, blockiert RK-03**)

`ResonanceConfig { beta: f32, gamma: f32 }` wird ein **Datentyp in `fusion/ext.rs`**, unabhängig vom Feature. Das Feature `exp-resonance` steuert nur noch die **Rechnung**: ohne Feature gibt `apply_resonance_bonus` die Ergebnisse unverändert zurück und schreibt eine `tracing::warn!`-Zeile, wenn eine Konfiguration übergeben wurde.

*Begründung.* Typform und Signaturen bleiben in jeder Featurekombination gleich (B-03); das Experiment bleibt abschaltbar (X2). *Preis.* X1 („Host-Kern kennt keine Experimente“) wird um einen zwei-Feld-Datentyp aufgeweicht. *Verworfen:* `Option<&dyn ResonanceModifier>` in den Signaturen, weil das eine Signaturänderung aller Fusionsfunktionen ist (INV-2). *Folge:* Das Stub-Feld `enabled` entfällt; es ist nur in Builds ohne Default sichtbar und hat keinen Verbraucher [V].

### D-R03 — `MAX_SEARCH_K`

`rank` behält seinen Wert (`BoundedTopK`-Obergrenze), führt aber einen neuen, eindeutigen Namen `MAX_TOPK_CAPACITY = 10_000` ein. `MAX_SEARCH_K` bleibt als `#[deprecated]`-Konstante mit gleichem Wert bis S-70. Kein Wechsel auf `ids::MAX_SEARCH_K`, weil das Verhalten ändern würde (1.000 statt 10.000).

### D-R04 — `ConfigFingerprint::new`

Konstruktor `from_bytes` ist der unterstützte Weg (so nutzt ihn `infer-candle`). `ConfigFingerprint::new(…)` wird `#[deprecated]` mit Warnhinweis „nicht prozessübergreifend stabil“ (falls F-R07 bestätigt) und bis S-70 unverändert gelassen. **Kein** Austausch des Hashers im Rahmen dieser Spezifikation: `ahash` ist auch mit festen Seeds nicht plattformübergreifend stabil (AES- und Fallback-Pfad), ein Austausch bräuchte einen eigenen Entwurf.

### D-R05 — Fehlertyp

`ConformalError` wird als Variante `RankError::Conformal(#[from] ConformalError)` eingebunden; `ConformalError` bleibt öffentlich (INV-2). `compat/error.rs` bildet die neue Variante auf `ContextraError::InvalidInput` ab (wie die übrigen) und `errors/rank.md` führt sie auf.

### D-R06 — `publish`

`publish = false` entfällt (R14: Baustein mit Entnehmbarkeitsanspruch). Voraussetzung ist F-R10 (`include`). Der `cargo package`-Probelauf (§8.4 Punkt 6) bleibt SOLL und scheitert [E] solange `ids`/`env` nicht veröffentlicht sind; er ist in diesem Fall als Befund zu führen, nicht als Blocker.

### D-R07 — `docid-128` als Weiterleitung

`rank` erhält `docid-128 = ["contextra-ids/docid-128"]`. *Begründung:* Das Feature ist für `rank` nur Weiterleitung (R10 erlaubt das), ermöglicht den Breitentest je Crate (`cargo test -p contextra-rank --features docid-128`) und steht nicht im Default. *Abweichung:* §4.1 listet es für `rank` nicht; deshalb ADR-Hinweis oder Nachtrag in §4.1.

---

## 6 Zielbild

### 6.1 `Cargo.toml` (Ziel)

```toml
[package]
name = "contextra-rank"
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version = "1.89"
description = "Rangfusion (RRF/mRRF), Score-Normalisierung, Kalibrierung (Isotonic, Platt, Conformal) und Drift-Erkennung"
readme = "README.md"
repository = "https://github.com/tfufuz1/contextra"
keywords = ["ranking", "rrf", "fusion", "calibration", "retrieval"]
categories = ["algorithms", "science"]
include = ["src/**/*", "benches/**/*", "examples/**/*", "tests/**/*",
           "Cargo.toml", "README.md", "CHANGELOG.md", "LICENSE-*"]
# publish: entfällt nach D-R06

[features]
default = ["std"]
std = ["contextra-ids/std", "contextra-env/std"]
workspace = ["dep:contextra-types", "dep:contextra-ports"]
docid-128 = ["contextra-ids/docid-128"]               # D-R07: nur Weiterleitung
exp-dibud = []
exp-resonance = []
experimental-all = ["exp-dibud", "exp-resonance"]
# Aliase, entfallen in S-70:
dibud = ["exp-dibud"]
experimental-rank = ["experimental-all"]
types-compat = ["workspace"]
ports-compat = ["workspace"]

[dependencies]
contextra-ids = { path = "../contextra-ids", version = "0.1.0", default-features = false }
contextra-env = { path = "../contextra-env", version = "0.1.0", default-features = false }
contextra-types = { path = "../contextra-types", version = "0.1.0", default-features = false, optional = true }
contextra-ports = { path = "../contextra-ports", version = "0.1.0", default-features = false, optional = true }
serde = { version = "1", features = ["derive"] }
serde_json = { workspace = true }
ahash = { version = "0.8", features = ["serde"] }
tracing = { workspace = true }
thiserror = { workspace = true }

[dev-dependencies]
contextra-env = { path = "../contextra-env", version = "0.1.0", default-features = false, features = ["std", "test-utils"] }
proptest = "1"
criterion = { workspace = true }
tokio = { version = "1", features = ["rt", "macros"] }   # nur für exp-dibud-Tests

[[bench]]
name = "rank_bench"
harness = false

# Brücken-Tests (E-01): nur mit Feature workspace
[[test]]
name = "hardening_tests"
required-features = ["workspace"]
[[test]]
name = "legacy_temporal_reference"      # bisher tests/legacy_reference/temporal_filter.rs
path = "tests/legacy_reference/temporal_filter.rs"
required-features = ["workspace"]
```

*Hinweis:* `contextra-types`/`contextra-ports` bleiben **optionale normale** Abhängigkeiten (das erlaubt R12 als „hinter `required-features = ["workspace"]`“). Der Fan-in-Ratchet (R11) wird dadurch nicht erhöht.

### 6.2 Feature-Matrix (Prüfumfang `cargo hack`)

| Kombination | Erwartung |
| --- | --- |
| `--no-default-features` | baut; Kern-Closure `{ids, env}`; Kerntests laufen |
| Default (`std`) | baut; identisch zu oben |
| `--features exp-resonance` | baut; `ResonanceConfig`-Typ identisch, Bonus aktiv |
| `--features exp-dibud` | baut; `dibud`-Tests laufen |
| `--features experimental-all` | baut |
| `--features workspace` | baut; Closure zusätzlich `types`, `ports` (erlaubt: Feature-Closure, §2.1) |
| `--features docid-128` | baut; Breitentest grün |
| `--all-features` | baut; Tests grün |

### 6.3 Modulbaum (Ziel, Änderungen markiert)

```
src/
├── lib.rs                 # cfg(feature="exp-dibud"); Doc ohne "contextra-types/-ports"
├── error.rs               # RankError (+ Conformal-Variante, D-R05)
├── contract.rs            # nur noch: pub use contextra_env::Clock;   (Pfad bleibt, INV-2)
├── local_types.rs         # DocId/TxId → pub use ids; FusionStrategy, SignalFailurePolicy,
│                          #   SignalFusionStrategies, FusionWeights, ConfigFingerprint bleiben (R4-Ausnahme, §6.4)
├── calibration/
│   ├── isotonic.rs        # Clock-Trait und SystemClock entfernt; pub use contextra_env::{Clock, SystemClock}
│   ├── platt.rs, conformal.rs, mod.rs
├── fusion/
│   ├── ext.rs             # ResonanceModifier + ResonanceConfig{beta,gamma} (D-R02) + Dispatch
│   ├── resonance.rs       # exp-resonance: nur noch die Rechnung
│   └── rrf.rs, mrrf.rs, normalized.rs, global.rs, provenance.rs, signal.rs, topk.rs, types.rs
├── dibud/                 # exp-dibud, unverändert
├── drift.rs, explain.rs, metrics.rs, temporal.rs
└── compat/                # nur mit Feature workspace
    ├── clock.rs           # PortsClockAdapter/IsotonicPortsClockExt: #[deprecated], leiten an env weiter
    ├── error.rs           # From<RankError> (inkl. Conformal)
    └── types.rs           # DocId/TxId-From entfallen (Identität); Rest bleibt
```

### 6.4 Gespiegelte Wertetypen (R4-Ausnahme)

`FusionStrategy` und `SignalFailurePolicy` sind kleine geschlossene Enums (je 2 Unit-Varianten) und dürfen nach R4 gespiegelt bleiben, **wenn** ein Konformitätstest die serde-Namen gleichsetzt. Für `SignalFusionStrategies`, `FusionWeights` (Structs) und `ConfigFingerprint` gilt die Ausnahme dem Wortlaut nach nicht. **[D]** `rank` bleibt Besitzer dieser Structs für den Kern; `types` hält sie als Spiegel, `compat/types.rs` führt die `From`-Brücken. Der Konformitätstest (serde-Roundtrip beider Seiten, gleiche Namen und Reihenfolge) liegt hinter `required-features = ["workspace"]` (Schritt RK-05). Das ist ein Nachweis für den R4-Befund im Register, kein Zusammenlegen (D-01 verbietet das).

### 6.5 Code-Skizzen der kritischen Stellen

**Clock (RK-07).**

```rust
// calibration/isotonic.rs  (Ziel)
pub use contextra_env::{Clock, SystemClock};   // Pfad contextra_rank::{Clock, SystemClock} bleibt gültig

fn default_clock() -> Arc<dyn Clock> { Arc::new(SystemClock::new()) }

// contract.rs
pub use contextra_env::Clock;
```

Die Tests, die `FixedTestClock` bauen, ergänzen `monotonic_nanos`; neue Tests verwenden `contextra_env::ManualClock` (Feature `test-utils`, dev-Kante auf E0 ist nach R12 erlaubt).

**Bruecke Clock (compat).** `contextra_ports::Clock` ist laut `ports/clock.rs` ein Re-Export von `env::Clock` [S §5.2]. `Arc<dyn contextra_ports::Clock>` ist dann derselbe Typ wie `Arc<dyn env::Clock>`:

```rust
// compat/clock.rs  (Ziel)
#[deprecated(note = "contextra_ports::Clock ist env::Clock; direkt with_clock verwenden")]
pub trait IsotonicPortsClockExt { fn with_ports_clock(self, c: Arc<dyn contextra_env::Clock>) -> Self; }
```

Ist die Identität nach S-11 nicht erfüllt, bleibt der Adapter unverändert (Rückfall, §10).

**Resonanz-Naht (RK-03).**

```rust
// fusion/ext.rs  (Ziel)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ResonanceConfig { pub beta: f32, pub gamma: f32 }
impl Default for ResonanceConfig { fn default() -> Self { Self { beta: 0.5, gamma: 0.3 } } }

pub trait ResonanceModifier { fn modify_results(&self, r: Vec<SearchResult>, n: usize) -> Vec<SearchResult>; }

pub fn apply_resonance_bonus(r: Vec<SearchResult>, n: usize, cfg: &ResonanceConfig) -> Vec<SearchResult> {
    #[cfg(feature = "exp-resonance")]
    { return super::resonance::apply(r, n, cfg); }
    #[cfg(not(feature = "exp-resonance"))]
    { tracing::warn!("ResonanceConfig übergeben, aber Feature exp-resonance ist aus; Bonus entfällt"); r }
}
```

**`DocId` (RK-08, nach ADR D-R01).**

```rust
// local_types.rs  (Ziel)
pub use contextra_ids::{DocId, TxId};

// legacy.rs  (nur Option (b); lebt bis S-70)
pub fn doc_id_from_key(key: &str) -> Result<DocId, RankError> { /* Zahl, Präfix doc_/doc-, sonst ids */ }
```

`compat/types.rs`: `impl From<contextra_types::DocId> for DocId` und die Gegenrichtung **entfallen**, weil beide Seiten dann derselbe Typ sind (andernfalls Konflikt E0119 durch die reflexive `From<T> for T`). `From<contextra_types::TxId> for TxId` entfällt aus demselben Grund.

---

## 7 Implementierungsplan

### 7.1 Regeln für jeden Schritt

Commit-Paar nach INV-3: `S-33 A: rank RK-nn …` (verschieben/umbenennen) und `S-33 B: rank RK-nn …` (umbauen), beide einzeln revertierbar (D-08). Definition of Done nach SPEC §9.1: Gates grün, `cargo check --workspace --all-targets --locked`, Score nicht gesunken, `REPORT`-Bericht regeneriert, `git revert` baut. Gate-Kürzel: **GL** (G-L1 bis G-L4), **GH**, **GF**, **GD**, **GE**, **GC**.

### 7.2 Reihenfolge

```
Welle 0   RK-00 → RK-01 → RK-02
Welle 1   RK-03 → RK-04 → RK-05 → RK-06        (Features, dev-Kanten, Fehler; vor S-10C/S-11 möglich)
Welle 2   RK-07 (nach S-11) → RK-08 (nach S-10C und ADR D-R01) → RK-09
Welle 3   RK-10 → RK-11                          (Abnahme; Aufräumen mit S-70)
```

Harte Abhängigkeiten: RK-04 nach RK-03 (sonst bricht der Default-Flip den Typ); RK-04 und RK-05 vor RK-10; RK-07 nach S-11 (`env` ohne `'static`, Prüfung der Verbraucher); RK-08 nach S-10C **und** ADR; RK-09 nach RK-07 **und** RK-08 (Typidentität von `TxId` und `Clock`).

### RK-00 — Baseline und Messung (entspricht S-00)

| | |
| --- | --- |
| **Vorbedingung** | **Workspace-`Cargo.toml` ist wiederhergestellt (F-R01).** Ohne sie sind alle Befehle unten nicht ausführbar. |
| **A (nur Messung, kein Code)** | `git show HEAD~1:Cargo.toml > Cargo.toml` prüfen und als eigener Commit durch die Eigentümer bestätigen lassen. Toolchain `1.89.0` (`rust-toolchain.toml`). Auf der Maschine ohne `static.rust-lang.org`: Toolchain über conda-forge oder apt installieren (siehe `CLAUDE.md`). |
| **B (Bericht)** | Baseline unter `docs/decoupling/baseline/`: `cargo check -p contextra-rank --no-default-features --locked`, je Feature (`cargo hack check -p contextra-rank --each-feature --no-dev-deps`), `cargo test -p contextra-rank --locked`, `cargo tree -p contextra-rank -e normal --no-default-features`, `cargo public-api -p contextra-rank` nach `docs/decoupling/public-api.txt`, Testanzahl vorher. |
| **Zusatzmessungen** | (1) **F-R07-Test:** kleines Beispiel (`examples/`-ähnlich, nicht eingecheckt) ruft `DocId::from_key("abc")` und `ConfigFingerprint::new("m","q","t",0.1)` in zwei getrennten Prozessen auf und vergleicht die Ausgabe. (2) `grep -rn 'contextra_rank::' crates benchmarks tests` vollständig als Verbraucherliste. (3) `tools/gate_bridge_null.py` und `tools/count_bridge_refs.py` für `rank`. |
| **Abnahme** | Bericht mit HEAD-Hash und Datum; F-R03 (Clock-Verhalten) und F-R07 mit Ergebnis im Befundregister. |
| **Rollback** | nicht nötig (kein Code). |

### RK-01 — Dokumentation und Metadaten (entspricht Anteilen von S-03)

| | |
| --- | --- |
| **A** | `AGENTS.md` (generierter Block regenerieren), `capabilities.fragment.toml` um `sublevel = "E1a"`, `core_deps = ["contextra-ids", "contextra-env"]` und Tier `experimental` für `exp-dibud`/`exp-resonance` ergänzen (D-17, X7). Das ist Zielzustand und gilt erst nach RK-08; bis dahin Feld `core_deps_target`, falls das Schema es nicht anders vorsieht [E, Schema in S-03 prüfen]. |
| **B** | `README.md` (Feature-Tabelle nach §6.2, Defaults richtigstellen, §4 „interne Abhängigkeiten“ auf Ziel), Crate-`SPEC.md` und `PLAN.md` auf `docs/SPEC.md` v6 umstellen (Rangfolge §1.2), `lib.rs`-Doc-Kopf: `contextra-types`/`contextra-ports` in Zeile 25 durch eine Formulierung **ohne** beide Schreibweisen ersetzen (D-01 strikt), Ring-Terminologie auf Ebenen (E1a) umstellen, `REPORT.md` kennzeichnen als „überholt, ersetzt durch RK-10“. |
| **Neu** | `docs/decoupling/errors/rank.md` mit Tabelle (RK-06 füllt sie vollständig, hier der Rahmen). |
| **Abnahme** | G-DOC1 (`tools/doc_lint.py`) ohne Befund für `rank`; `gate_bridge_null.py` meldet für `src/` ohne `compat/` **0** (strenge Zählung inklusive Kommentare). |
| **Gates** | G-DOC1, GL (G-L2) |
| **Rollback** | Revert (reine Doku). |

### RK-02 — Golden-Korpus, Breitentest, Differenz-Harness (entspricht S-02)

| | |
| --- | --- |
| **A** | Vorhandenen Korpus `tests/golden/` unverändert lassen; im Wurzelkorpus `tests/golden/rank/` referenzieren (SPEC §6.1: crate-lokale Korpora bleiben gültig, Wurzel-Manifest verweist). `tools/golden_regen.sh`-Einträge für `rank` ergänzen. |
| **B** | **Neue Golden-Dateien** (Erzeuger `#[ignore]`, Prüftest läuft immer): `Calibrator` als JSON für **Platt** und **Conformal** (bisher nur Isotonic: `isotonic_calibration.json`), `ProvenanceRecord`-JSON, `FusionWeights`/`SignalFusionStrategies`-JSON, `explain`-Ausgabe. **Breitentest:** Korpus je einmal mit 64 und einmal mit 128 Bit erzeugen (`--features docid-128` nach D-R07); sind die Bytes gleich, braucht kein Format `id_width_bits` (SPEC §6.2). *Erwartung [E]:* gleich, weil keines der Formate `DocId` trägt. **Differenz-Harness:** Test-Binary mit Namensmuster `differential_*` (SPEC §7.3 lässt das Muster zu). |
| **Abnahme** | `sha256sum -c` über das Manifest; Breitentest grün; **kein** bestehender Hash ändert sich. |
| **Gates** | GF |
| **Rollback** | Revert; Korpus ist additiv. |

### RK-03 — Additive Experiment-Naht (neu; Voraussetzung für den Default-Flip)

*Voraussetzung:* ADR D-R02.

| | |
| --- | --- |
| **A** | `fusion/resonance.rs` auf die reine Rechnung reduzieren (`pub(super) fn apply(...)`), `fuse_signals` bleibt dort. `lib.rs`/`fusion/mod.rs`: `cfg(feature = "dibud")` → `cfg(feature = "exp-dibud")` (Δ6). Aliase `dibud`, `experimental-rank` als Kommentar kennzeichnen. |
| **B** | `ResonanceConfig`, `ResonanceModifier`, `apply_resonance_bonus` in `fusion/ext.rs` nach §6.5; Stub-Zweig (`cfg(not(feature = "exp-resonance"))`) entfällt. Die drei `use super::{apply_resonance_bonus, ResonanceConfig}` in `rrf.rs`, `mrrf.rs`, `normalized.rs` zeigen auf `ext`. Tests, die `beta`/`gamma` setzen, brauchen kein Feature-Gate mehr; Tests, die den Bonus **prüfen**, tragen `#[cfg(feature = "exp-resonance")]` (FINDINGS Nr. 1). |
| **Differenztest DT-4** | `rrf_fusion.json` und alle `fusion_*`-Tests bitgleich in: Default alt (mit `exp-resonance`) ↔ `--features exp-resonance` neu. |
| **Additivitätstest DT-6** | Ohne Konfiguration (`None`) sind die Ergebnisse mit und ohne `exp-resonance` bitgleich. |
| **Abnahme** | `cargo hack check -p contextra-rank --each-feature --no-dev-deps` grün; `cargo test` in jeder Kombination aus §6.2 grün. |
| **Gates** | GL, GH, GD |
| **Rollback** | Revert von B; A bleibt, es verschiebt nur. |

### RK-04 — Default-Flip und Konsumenten (entspricht Anteil von S-12a)

| | |
| --- | --- |
| **A** | Konsumenten vorbereiten, **bevor** der Default fällt: `contextra-db/Cargo.toml`: `features = ["workspace", "exp-resonance"]` (erhält `fuse_signals` und das Resonanzverhalten, INV-5). `contextra-mcp`, `cognition`, `infer-candle`, `engine`, `bench` brauchen **keine** Experimente [V, `grep`], werden aber im Commit auf Kompilierbarkeit geprüft. |
| **B** | `rank/Cargo.toml`: `default = ["std"]`; `std` leitet weiter (`ids`/`env` kommen erst in RK-07/08, bis dahin `std = []` belassen und Δ11 dort schließen). Aliase als solche kommentieren. |
| **Differenztest** | Workspace-weit: `contextra-db`-Tests (`fusion_edge_cases_test.rs`, `integration_temporal_filter.rs`) vorher/nachher gleich. |
| **Abnahme** | `cargo check --workspace --all-targets --locked`; `feature_audit.py` meldet für `rank` kein verbotenes Default-Feature (Abzug F in M = 0). |
| **Gates** | GL, GH, GD |
| **Rollback** | B revertieren; A schadet nicht (zusätzliches Feature, aktiv im Default). |

### RK-05 — dev-Kanten und Brücken-Tests (entspricht S-12b)

| | |
| --- | --- |
| **A** | Tests nach R12 sortieren: `tests/hardening_tests.rs` und `tests/legacy_reference/temporal_filter.rs` nutzen `contextra_types`/`contextra_ports` [V]. Sie erhalten `[[test]] … required-features = ["workspace"]` (Manifest siehe §6.1). Den serde-Konformitätstest für die R4-Spiegel (§6.4) und den `From<RankError>`-Tabellentest (heute in `compat/error.rs`) dort ablegen. |
| **B** | `[dev-dependencies]` auf `types` und `ports` streichen. Die Brücken-Tests laufen über die optionalen normalen Abhängigkeiten (`workspace`). Falls `testkit` Bausteintests hostet, entsteht `testkit/tests/rank/` als Alternative (D-12) — **nicht** erforderlich, solange `required-features` genügt. |
| **Abnahme** | `cargo test -p contextra-rank --no-default-features` kompiliert ohne `types`/`ports` in der Closure (`cargo tree -e normal,dev`); **Testanzahl vorher = nachher** (Baseline aus RK-00); G-L1 prüft dev-Kanten. |
| **Gates** | GL, GH |
| **Rollback** | Revert B. |

### RK-06 — Fehlerabbildung (entspricht G-E1 für `rank`)

| | |
| --- | --- |
| **A** | `RankError` um `Conformal(#[from] ConformalError)` erweitern (D-R05; `#[non_exhaustive]` bleibt). Ausgabe `Display`: `"conformal error: {0}"`; bestehende Texte unverändert. |
| **B** | `compat/error.rs`: Abbildung der neuen Variante; `errors/rank.md` vollständig (alle 4 Varianten, Zielvariante, Display-String, Vermerk zu F-R09). **Tabellengetriebener Test** je Variante (Display bitgleich zum Altfehler). `grep -rn 'ContextraError::[A-Z]' src` bleibt leer (nur `compat/`). |
| **Abnahme** | `error_topology_mapper.py` im Prüfmodus ohne Befund; Test grün. |
| **Gates** | GE |
| **Rollback** | Revert. |

### RK-07 — `Clock` und `SystemClock` aus `env` (entspricht Anteil von S-33; nach S-11)

*Voraussetzung:* S-11 ist abgeschlossen; `env::Clock` ohne `'static`, Verbraucher geprüft.

| | |
| --- | --- |
| **A** | `contextra-env` als Kernabhängigkeit eintragen (`default-features = false`), `std` leitet an `env/std` weiter. `contextra-env` mit `test-utils` als **dev**-Kante. |
| **B** | `calibration/isotonic.rs`: Trait und `SystemClock` entfernen, `pub use contextra_env::{Clock, SystemClock}`; `contract.rs`: `pub use contextra_env::Clock`; `temporal.rs:178` zeigt auf `crate::Clock`; `compat/clock.rs` nach §6.5; `FixedTestClock` ergänzt `monotonic_nanos`. `ALLOW-DETERMINISM`-Markierung entfällt in `rank` (steht in `env`). |
| **Differenztest DT-1** | `IsotonicCalibrator` mit `ManualClock` (neu) und mit einer Test-Uhr, die das alte Verhalten nachbildet (`monotonic = now`): `last_calibration_at`, `cached_ece`, serde-Ausgabe, `PartialEq` bitgleich; zusätzlich Prüfung, dass **kein** Produktionspfad `monotonic_nanos` aufruft (F-R03). |
| **Abnahme** | `cargo tree -p contextra-rank -e normal --no-default-features` zeigt genau `contextra-env`; `symbol-uniqueness`: keine zweite `Clock`. |
| **Gates** | GL, GD, GE |
| **Rollback** | Revert B. Hat der `Clock`-Wechsel externe Implementierer gebrochen (Verbraucherliste aus RK-00), gilt SPEC §9.5 Punkt 2 (Gate rot und nicht in einem Tag behebbar → Revert, Befund). |

### RK-08 — `DocId` und `TxId` aus `ids` (entspricht S-10C-Anteil für `rank`; nach S-10C **und** ADR D-R01)

| | |
| --- | --- |
| **A** | `contextra-ids` als Kernabhängigkeit (`default-features = false`), `std = ["contextra-ids/std", …]`. Differenztest-Gerüst (Commit A, SPEC §6.4): alte und neue Implementierung gleichzeitig kompiliert; die Legacy-Referenz liegt bereits in `tests/legacy_reference/doc_id_reference.rs` [V]. |
| **B** | `local_types.rs`: `pub use contextra_ids::{DocId, TxId}`; je nach ADR `legacy.rs` mit `doc_id_from_key`. Aufrufstellen in `dibud/*`, `fusion/types.rs:78` (`synthesized_from(&[DocId])`) und `temporal.rs` prüfen: kein `as_u64()` in Logik (F-R12). `compat/types.rs`: `From`-Paare für `DocId`/`TxId` entfernen (E0119). `MAX_TOPK_CAPACITY` nach D-R03. |
| **Differenztest DT-2** | 10.000 Zufallsschlüssel plus Randfälle (leer, Unicode, 4 KiB, reine Zahl, `doc_7`, `doc-7`, Überlauf `u64::MAX + 1`): Ergebnis je ADR-Option; serde (JSON) bitgleich für numerische Werte. In **beiden** Breiten (`--features docid-128`). |
| **Golden** | `id_serde.json` bleibt gültig (F-R12). |
| **Abnahme** | `cargo tree`: Kern-Closure genau `{ids, env}` (A-01 erfüllt); `capabilities.fragment.toml` auf Ziel; M-Abzug A = 0. |
| **Gates** | GL, GF, GD, GE |
| **Rollback** | Revert B; Commit A bleibt. Zeigt DT-2 eine unerwartete Abweichung, bricht der Schritt ab und die Kopie bleibt (SPEC §9.5 Punkt 3): **die Toleranz wird nicht angepasst**. |

### RK-09 — Temporalfilter-Ablösung (S-33, korrigiert nach F-R05)

*Voraussetzung:* RK-07 und RK-08 (damit `TxId` und `Clock` typgleich sind, siehe `TEMPORAL_PARITY.md`).

| | |
| --- | --- |
| **A** | Differenztest DT-3: `db::temporal_filter` (alt) ↔ `rank::temporal` ↔ eingefrorene Referenz `rank/tests/legacy_reference/temporal_filter.rs`, ≥ 10.000 Proptest-Fälle (der Test aus `engine/tests/proof_temporal_filter_vs_rank.rs` zieht nach `rank/tests/` bzw. `db/tests/` um; der Engine-Test mit totem Importpfad wird entfernt, Löschung als leere Datei mit Inhalt `DELETED` gemäß `CLAUDE.md`). |
| **B** | `contextra-db/src/temporal_filter.rs` wird `pub use contextra_rank::temporal::*` (Pfade `contextra_db::temporal_filter::…` bleiben, INV-2). Die beiden `#[allow]`-Abweichungen (`indexing_slicing`, `cast_sign_loss`) liegen schon in `rank`. `FusionResult` bleibt `rank::SearchResult` (bereits identisch). |
| **Abnahme** | `db`-Tests `integration_temporal_filter.rs` unverändert grün; Filterausgaben bitgleich (§6.4). |
| **Gates** | GD, GL |
| **Rollback** | Revert B. Weicht eine Ausgabe ab → Schritt abbrechen, Kopie bleibt, Befund (SPEC §9.5 Punkt 3). |

### RK-10 — Abnahme und Entnehmbarkeitsnachweis (SPEC §8.4)

| | |
| --- | --- |
| **A** | `README.md`-Beispiel auf ≤ 10 Zeilen kürzen und als Doctest/`examples/standalone.rs` mit `--no-default-features` belegen (existiert, 12 Zeilen). `Cargo.toml`: `include` nach §6.1 (F-R10), `publish = false` nach D-R06 entfernen. |
| **B** | `docs/decoupling/REPORT.md` neu erzeugen: HEAD-Hash, Datum, Gate-Reihenfolge G-L2 → G-L1 → G-L3 → G-L4 → G-H1 mit Exit-Codes, M = 100, `cargo package -p contextra-rank --locked` (SOLL, Ergebnis ehrlich berichten), Breitentest, Korpus. Danach — und nur durch diesen Schritt — `extractable = true`, `verified_at = <HEAD>` in `capabilities.fragment.toml`. |
| **Abnahme** | Alle Punkte aus §11. |
| **Gates** | alle |
| **Rollback** | `extractable` zurücksetzen (CI entzieht es ohnehin bei rotem Gate). |

### RK-11 — Aufräumen (Anteil von S-70)

Entfernen: `#[deprecated]`-Re-Exports und -Brückenfunktionen (`PortsClockAdapter`, `IsotonicPortsClockExt`, `*_types_compat`), Aliase `dibud`, `experimental-rank`, `types-compat`, `ports-compat`, `MAX_SEARCH_K`, `ConfigFingerprint::new` (D-R04), `legacy::doc_id_from_key` (D-R01). Nur bei grünem G-E1, G-A1 und nach Prüfung der Verbraucherliste. Voraussetzung: S-70 insgesamt.

---

## 8 Teststrategie

| ID | Test | Eingabe | Toleranz | Schritt | Gate |
| --- | --- | --- | --- | --- | --- |
| DT-1 | Clock-Differenz | `ManualClock` gegen Alt-Uhr, `IsotonicCalibrator` komplett | bitgleich (Feld- und serde-Ebene) | RK-07 | GD |
| DT-2 | `DocId`-Differenz | 10.000 Zufallsschlüssel + Randfälle, 64 und 128 Bit | je ADR; Zahl-/Präfixfälle bitgleich | RK-08 | GD |
| DT-3 | Temporalfilter | ≥ 10.000 Proptest-Fälle, `extract_validity_window`, `is_valid_bitemporal`, `sort_candidates_deterministic` | bitgleich | RK-09 | GD |
| DT-4 | Fusion-Golden | `rrf_fusion.json`, mRRF, Score-Normalisierung | bitgleich (f32 über `total_cmp` und Bitvergleich) | RK-03, RK-04 | GF, GD |
| DT-5 | Format-Golden | `Calibrator` (Platt, Isotonic, Conformal), Provenance | Hash gleich | RK-02 | GF |
| DT-6 | Feature-Additivität | gleiche Eingabe in jeder Kombination aus §6.2 (ohne Resonanzkonfiguration) | bitgleich | RK-03 | GD |
| DT-7 | Breitentest | Korpus in 64 und 128 Bit | Bytes gleich | RK-02 | GF |
| DT-8 | Fehler-Display | je `RankError`-Variante nach `ContextraError` | String bitgleich | RK-06 | GE |
| DT-9 | R4-Konformität | serde-Namen von `FusionStrategy`, `SignalFailurePolicy` in `rank` und `types` | gleich | RK-05 | GE |
| NT-1 | Negativtest G-L2 | absichtlich `use contextra_types::…` im Kern | Exit 1 | RK-01 | GL |
| NT-2 | Negativtest dev-Kante | absichtlich `dev-dependencies.contextra-types` | Exit 1 | RK-05 | GL |

Bestehende Tests bleiben unverändert (INV-3 „Testanzahl vorher/nachher gleich“, Baseline in RK-00). Die Clippy-Befunde in `rank_invariants.rs` (`FINDINGS.md` Nr. 2) werden nicht angefasst (SPEC: kein stilles Umformatieren).

---

## 9 Migrationsmatrix der Verbraucher

| Crate | Änderung | Schritt | Prüfung |
| --- | --- | --- | --- |
| `contextra-db` | `features = ["workspace", "exp-resonance"]`; `temporal_filter.rs` → `pub use` | RK-04, RK-09 | `db`-Tests, `fusion_edge_cases_test.rs` |
| `contextra-engine` | Beweistest `proof_temporal_filter_vs_rank.rs` umziehen oder entfernen | RK-09 | `cargo check -p contextra-engine --all-targets` |
| `contextra-mcp`, `cognition`, `bench` | keine Änderung erwartet; Kompilierprüfung nach RK-04 | RK-04 | `cargo check --workspace --all-targets` |
| `contextra-infer-candle` | keine Änderung am Aufruf; dev-Kante auf `rank` mit Default bleibt (eigener Schritt S-12b/S-39) | – | `cargo tree -p contextra-infer-candle --no-default-features` |
| Wurzel-`Cargo.toml` | `contextra-rank = { path = … }` bleibt; **kein** `default-features = false` nötig | – | – |

---

## 10 Risiken und Rückfall

| ID | Risiko | Wahrscheinlichkeit [E] | Wirkung | Gegenmaßnahme / Rückfall |
| --- | --- | --- | --- | --- |
| R-1 | Workspace-Datei bleibt weg (F-R01) | hoch, solange nicht entschieden | nichts prüfbar | RK-00 zuerst; ohne sie startet kein Schritt |
| R-2 | `env::Clock` ist nach S-11 nicht identisch mit `ports::Clock` | mittel | Adapter muss bleiben | `compat/clock.rs` unverändert lassen, nur Kern umstellen |
| R-3 | Externe Implementierer von `rank::Clock` brechen (kein Default für `monotonic_nanos`) | niedrig im Workspace, unbekannt extern | Kompilierfehler | Verbraucherliste aus RK-00; Hinweis im `CHANGELOG.md`; SPEC §9.5 Punkt 2 |
| R-4 | ADR D-R01 verzögert RK-08 | mittel | `ids`-Kern bleibt offen | RK-07 und RK-09 (Clock-Teil) laufen unabhängig weiter; A-01 gilt erst nach RK-08 |
| R-5 | F-R07 bestätigt sich nicht (Hasher stabil) | mittel | D-R01-Empfehlung ändert sich | Option (b) vollständig umsetzen |
| R-6 | `ResonanceConfig`-Änderung stört Code, der `enabled` nutzt | niedrig | Kompilierfehler | `grep` zeigt keine Nutzung im Workspace; ADR führt es auf |
| R-7 | Differenztest DT-2/DT-3 rot | niedrig | Schritt bricht ab | Toleranz nie anpassen; Befund (SPEC §9.5 Punkt 3) |
| R-8 | `cargo package` scheitert (Pfad-Deps unveröffentlicht) | hoch | SOLL-Punkt offen | als Befund führen, kein Blocker (D-R06) |

---

## 11 Abnahme (Definition of Done für `contextra-rank`)

Der Crate gilt als entnehmbar, wenn der Bericht `crates/contextra-rank/docs/decoupling/REPORT.md` mit HEAD-Hash und Datum alles Folgende belegt (SPEC §8.4):

- [ ] G-L2 → G-L1 → G-L3 → G-L4 → G-H1 grün (Exit 0, in dieser Reihenfolge, in CI blockierend, D-23)
- [ ] Kern-Closure genau `{contextra-ids, contextra-env}` (≤ 4)
- [ ] `default = ["std"]`; `feature_audit.py` ohne Befund; Aliase gekennzeichnet
- [ ] `cargo hack check --each-feature --no-dev-deps` und `--features docid-128` grün
- [ ] 0 Brückenreferenzen im Kern (strenge Zählung inkl. Kommentare)
- [ ] keine dev-Kante auf `types`, `ports`, `testkit`
- [ ] `RankError` (+ `ConformalError`) mit `errors/rank.md`, G-E1 grün
- [ ] keine eigene `DocId`, `TxId`, `Clock`, `SystemClock`; `symbol-uniqueness` grün
- [ ] Golden-Korpus und Breitentest grün, kein Hash geändert
- [ ] DT-1 bis DT-9 grün; `temporal`-Ablösung in `db` per `pub use`
- [ ] M = 100; keine `unwrap`/`expect`/`panic!` in `src/`; `forbid(unsafe_code)`
- [ ] README-Beispiel ≤ 10 Zeilen kompiliert ohne weitere `contextra-*`; `examples/` lauffähig
- [ ] `capabilities.fragment.toml`: `level`, `sublevel`, `core_deps`, Tier je Experiment, `extractable` mit `verified_at`
- [ ] SOLL: `cargo package -p contextra-rank --locked` (Ergebnis dokumentiert)

---

## 12 Nicht-Ziele und offene Fragen

**Nicht-Ziele.** Keine Formatänderung (INV-1); keine In-Place-Migration der ID-Breite (D-16); kein Zusammenlegen mit `adapt::drift` (B16); kein Austausch des Hashers für `ConfigFingerprint::new` (D-R04); keine Änderung von Algorithmen (RRF, PAVA, Platt, Conformal, DiBud); kein `no_std` (D-19); keine Zeitreise-, Regel- oder Löschlogik aus SPEC Kapitel 11 bis 15 — `rank` hat laut README kein Löschketten- oder Compliance-Thema und trägt nach INV-13 keine personenbezogenen Typen. Die Kapitel 11 bis 15 betreffen `rank` damit nur indirekt über G-PII1 (keine personenbezogenen Typen im Kern); dafür ist in RK-00 ein `grep` auf `PersonalRef`/Personenbezug als Nullprüfung vorgesehen [E].

**Offene Fragen an die Eigentümer.**

1. Wer stellt die Workspace-`Cargo.toml` wieder her, und war das Löschen in `24487fa16` beabsichtigt (F-R01)?
2. D-R01: Soll `rank::DocId::from_key` die Legacy-Semantik für Zahlen und `doc_`-Präfix behalten (Empfehlung) oder vollständig auf `ids` wechseln?
3. D-R02: Ist die Aufweichung von X1 (zwei-Feld-Datentyp im Host-Kern) akzeptabel?
4. D-R07: Wird `docid-128` als Weiterleitung in `rank` in §4.1 nachgetragen?
5. SPEC-Erratum F-R05: S-33 und §6.4 sollen `db::temporal_filter` statt `engine::temporal_filter` nennen.
6. Nach SPEC §7.3 laufen die meisten Gates in CI nicht. Bis S-01 abgeschlossen ist, bleiben alle „Gate grün“-Aussagen dieses Plans lokale Nachweise.

---

## Anhang A — Prüfbefehle (nach Wiederherstellung der Workspace-Datei)

```bash
# Kern und Features
cargo check -p contextra-rank --no-default-features --locked
cargo hack check -p contextra-rank --each-feature --no-dev-deps --locked
cargo check -p contextra-rank --features docid-128 --locked
cargo test  -p contextra-rank --locked
cargo test  -p contextra-rank --all-features --locked

# Closure und Isolation
cargo tree -p contextra-rank -e normal --no-default-features
cargo tree -p contextra-rank -e normal,dev --no-default-features
python3 tools/isolation_proof.py --crate contextra-rank --modes no-default,default,each-feature

# Gates
python3 tools/gate_bridge_null.py            # G-L2 (strenge Zählung inkl. Kommentare)
python3 tools/gate_layering.py --crate contextra-rank   # G-L1
python3 tools/feature_audit.py                # G-H1
python3 tools/determinism_guard.py --crate contextra-rank
python3 tools/error_topology_mapper.py        # G-E1
python3 tools/modularity_score.py -c contextra-rank
(cd crates/contextra-rank/tests/golden && sha256sum -c MANIFEST.sha256)

# Verbraucher und Altlasten
grep -rn 'contextra_rank::' crates benchmarks tests
grep -rnE 'contextra[_-](types|ports|testkit)' crates/contextra-rank/src | grep -v '/compat/'
cargo public-api -p contextra-rank > docs/decoupling/public-api.txt
```

## Anhang B — Zuordnung zu SPEC-Schritten

| RK | SPEC-Schritt | Bemerkung |
| --- | --- | --- |
| RK-00 | S-00 | plus F-R01, F-R07 |
| RK-01 | S-03 (Anteil), G-DOC1 | |
| RK-02 | S-02 | Korpus, Breitentest |
| RK-03 | S-12a (Vorarbeit), S-33 | additive Naht, D-R02 |
| RK-04 | S-12a | Default-Flip |
| RK-05 | S-12b | dev-Kanten, F-09 |
| RK-06 | G-E1 | §6.3 |
| RK-07 | S-33 (`Clock`), S-11 | |
| RK-08 | S-33 (`DocId`), S-10C, D-20 | |
| RK-09 | S-33 (Temporalfilter) | korrigiert auf `db` |
| RK-10 | §8.4 | Entnehmbarkeit |
| RK-11 | S-70 | |

## Anhang C — Belegübersicht

| Aussage | Beleg |
| --- | --- |
| Workspace-`Cargo.toml` gelöscht | `git show --stat HEAD -- Cargo.toml` (236 Löschungen) |
| Default-Features | `crates/contextra-rank/Cargo.toml`, Abschnitt `[features]` |
| Zwei Clock-Traits | `src/contract.rs:7`, `src/calibration/isotonic.rs:14` |
| Lokale `DocId`, `TxId`, `from_key` | `src/local_types.rs` |
| `ids::TxId` vorhanden | `crates/contextra-ids/src/lib.rs:407` |
| `env::Clock` ohne Default-Impl, mit `'static` | `crates/contextra-env/src/clock.rs` |
| `ResonanceConfig` zweigestaltig | `src/fusion/ext.rs:14-29`, `src/fusion/resonance.rs` |
| `cfg(feature = "dibud")` | `src/lib.rs:71,78` |
| Temporalfilter in `db` | `crates/contextra-db/src/temporal_filter.rs` (383 Zeilen), `diff` gegen `rank/src/temporal.rs` |
| Toter Engine-Importpfad | `crates/contextra-engine/tests/proof_temporal_filter_vs_rank.rs` |
| Golden-Hashes gültig | `sha256sum -c MANIFEST.sha256`: 4 × `OK` |
| `ahash` mit `getrandom` | `Cargo.lock` Zeilen 57–68 |
| Keine Panics in `src/` | `grep -rnE '\.unwrap\(\)|\.expect\(|panic!|todo!' src` |
| Verbraucher | `grep -rn 'contextra_rank' crates benchmarks` |
