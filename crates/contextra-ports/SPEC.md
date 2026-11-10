# contextra-ports — Spezifikation

> **Geltung:** Crate-lokale Spezifikation. Normativ bleibt `docs/SPEC.md` (v2, 10.11.2026). Dieses Dokument überträgt die Anforderungen für dieses Crate, ergänzt sie um verifizierte Ist-Fakten und markiert Abweichungen.
> **Faktenbasis:** statische Analyse mit `tools/` am geklonten Repo-Stand (`main`, HEAD `f0eda1cb9`). `cargo` ist in der Analyseumgebung nicht verfügbar. **Es wurde nichts gebaut, getestet oder ausgeführt.**
> **Beleg-Marken:** *[V]* am Quelltext/Manifest verifiziert, *[T]* aus einem tools/-Bericht, *[S]* aus docs/SPEC.md übernommen, *[E]* Festlegung dieser Spezifikation.

**Pfad:** `crates/contextra-ports/` · **Umsetzungsplan:** `PLAN.md` (im selben Verzeichnis)

## 1. Zweck

Port-Traits (Storage, Vector-/Text-/Graph-Index, Vfs, Embedding u. a.) als Vertragsschicht. *[S]*

## 2. Zielzustand (docs/SPEC.md §3.1) [S]

| Merkmal | Ziel |
|---|---|
| Gruppe | contracts |
| Ebene (Kern-Closure) | 2 |
| Kern-Closure | types, ids, env |
| Default-Features | — |
| Fehlertyp (R3) | — |
| Brücke (hinter Feature `workspace`, §1.1) | — |

Ebenen (SPEC §1.1): E0 = Kern-Closure leer · E1 = Kern-Closure ⊆ E0, höchstens 4 Mitglieder · E2 = Integration · E3 = Anwendungen und Testwerkzeuge. Der Kern ist der Code mit `--no-default-features` ohne Feature `workspace`.

## 3. Ist-Zustand (tools/)

| Messgröße | Wert |
|---|---|
| Lines of Code (Produktion, `loc.py`) | 4418 Zeilen in 41 Dateien |
| Version / publish | workspace-vererbt / <unset> |
| Interne Normal-Abhängigkeiten (Ist, Cargo.toml) | types |
| Interne Dev-Abhängigkeiten (Ist) | keine |
| Features (Ist, Anzahl) | 4 |
| Default-Features (Ist) | keine |
| `cfg(feature = …)`-Stellen in src/ | 3 |
| `#[test]`-Vorkommen (grep, src + tests) | 43 |
| zero-panic: Produktionsbefunde, bereinigt (Tool-Gesamt; davon in Testdateien) | 0 (Tool 0; 0 in Testdateien) |
| unsafe: Vorkommen / ohne SAFETY / Insel | 0 / 0 / False |
| Determinismus: Produktionsbefunde, bereinigt (Tool-Gesamt) | 3 (Tool 3) |
| Port-M3-Leaks (port_contract_audit) | 0 |
| Error-Enums (Ist) | EmbeddingError, LicenseError, PluginError, ObserverError, GraphMutationError, SignerError |

**Tool-Caveat:** `zero_panic_analyzer.py` klassifiziert `#[cfg(test)]`-Module (`tests.rs`) als Produktion. Im Repo betrifft das 449 von 478 Treffern. Die Zahlen oben sind deshalb bereinigt; die Rohzahl steht in Klammern. Die Korrektur am Tool ist eigener Schritt (S-00).

**Quellen:** `tools/loc.py`, `tools/zero_panic_analyzer.py`, `tools/unsafe_boundary_auditor.py`, `tools/determinism_guard.py`, `tools/port_contract_auditor.py`, `tools/error_topology_mapper.py`, `tools/cargo_dependency_graph.py`, Cargo.toml-Auswertung.

## 4. Ist-vs-Ziel-Abgleich [V]

- **Ist-Kanten, die das Ziel nicht vorsieht:** keine
- **Ziel-Kanten, die im Ist fehlen:** `ids`, `env`
- **Default-Features:** Ist = Ziel (leer).

## 5. Verifizierte Befunde und Abweichungen zu docs/SPEC.md [V]

- Verifiziert: `pub use contextra_types::{…, model_fingerprint, …}` in `src/lib.rs:23–24`. B16 (ModelFingerprint bleibt in types) ist damit konsistent.
- B08: `crate::Result` bzw. `std::io::Result` in `src/vfs.rs` an 9 Zeilen gefunden. Die SPEC nennt 58 Treffer. Die Zählung ist nicht reproduzierbar und wird in S-00 neu erhoben.

## 6. Anforderungen

### 6.1 Crate-Spezifikation (docs/SPEC.md §6) [S]

Bleibt: `storage`, `vector_index`, `text_index`, `graph_index`, `graph`, `embedding`, `reranker`, `query_rewriter`, `checkpoint`, `kv`, `kv_bridge_port`, `attention`, `lifecycle`, `observability`, `plugin`, `vfs`, `fs`, `scanner/*`. Geht: `clock`/`rng`/`id_gen`/`metrics`/`env` → `env` (Re-Export bleibt); `erasure*`, `gate`, `ledger`, `observer`, `report_signer`, `keys`, `barrier` → `erasure` (Re-Exports in `lib.rs:16–22,156–206` entfernen); `license.rs`, `policy.rs` → `license`; `conformance.rs` (278), `contract_tests.rs` (632) → `testkit` nach Überschneidungsprüfung mit `testkit/conformance/` (1.612 Z.); `EgressPolicy` aus `license.rs` nach `privacy` **nur wenn** `privacy` sie ohnehin braucht (Prüfung per `rg`), sonst in `license`.

### 6.3 Regeln für jeden Schritt [S]

INV-1 Formatgleichheit · INV-2 Pfadgleichheit (Re-Export mit `#[deprecated]`) · INV-3 Ein Schritt = Commit-Paar · INV-4 `cargo check --workspace --all-targets --locked` grün · INV-5 Determinismus (Tie-Break nach ID aufsteigend) · INV-6 Durability-Pfade unberührt · INV-7 keine Compliance-Aussagen · INV-8 keine Rückwärtspfeile. Vollständig: docs/SPEC.md §1.2, Regeln R1–R9: §3.2.

## 7. Schnittstellen und Persistenz

Keine Zeile aus dem Golden-Korpus (§5.1) bezieht sich auf dieses Crate.

### 7.2 Öffentliche Schnittstelle

Ist-Stand und Ziel-Schnittstelle sind in den Abschnitten 3 bis 6 festgehalten. Die exakte Liste der `pub`-Symbole erhebt S-00 über `cargo public-api` (Gate G-A1); erlaubte Differenzen sind nur Re-Export-Pfade mit `#[deprecated]`.

## 8. Fundstellen (tools/, exakt) [T]

**Determinismus (Produktion, bereinigt: 3; die ersten 15):** Tool-Gesamt 3.

- `crates/contextra-ports/src/clock.rs:40` — instant_now — `start_instant: Instant::now(),`
- `crates/contextra-ports/src/clock.rs:55` — system_time_now — `SystemTime::now()`
- `crates/contextra-ports/src/clock.rs:64` — instant_now — `let nanos = Instant::now()`

**Fehler-Enums (Ist):**

- `crates/contextra-ports/src/embedding.rs:65` — EmbeddingError
- `crates/contextra-ports/src/license.rs:55` — LicenseError
- `crates/contextra-ports/src/plugin.rs:98` — PluginError
- `crates/contextra-ports/src/observer.rs:62` — ObserverError
- `crates/contextra-ports/src/graph.rs:93` — GraphMutationError
- `crates/contextra-ports/src/report_signer.rs:30` — SignerError

## 9. Befunde aus docs/SPEC.md (B-Nummern) [S]

**B02 — Die `Clock`-/`Rng`-Kopien sind nicht kompatibel (Teil 2, G1 ist unvollständig)**
> **Entscheidung:** - `env::Clock` = `ports::Clock` (Nanosekunden, `monotonic_nanos`, `Send+Sync+'static`). `env::Rng` = `ports::Rng` mit `&self` (siehe §4.2). - Millisekunden-Verbraucher (`text`, `graph`, `vector/contract.rs`) benutzen `env::Clock` und `env::unix_ms(&dyn Clock) -> u64` = `now_unix_nanos() / 1_000_000` (Abrunden). Persistierte ms-Zeitstempel MÜSSEN im Differenztest (§5.4) bitgleich zu `now_unix_ms` der alten Implementierung bei identischer Quelle sein. - `&mut self`-Verbraucher `text`/`graph`/`vector` wechseln auf `&self`-`env::Rng`. Das ist für alle heutigen Aufrufstellen ein Superset (Mutex/Atomic innerhalb `SeededRng`). Der Umstieg ist ein **Umbau-Commit** (INV-3 B). - E0-Crates (`adapt`, `privacy`, `license`, `erasure`, `ingest`, `audit-export`) behalten ihre lokalen Traits (R7/K8). Ihre Brücke bindet per Blanket-Impl **von `env` auf lokal** (Richtung `&self → &mut …
> `ports/clock.rs:8`, `rng.rs:12` — `now_unix_nanos`, `monotonic_nanos`; `Send+Sync+'static` — `next_u64(&self)`, `fill_bytes(&self,…)`, `next_unit_f64(&self)`; `'static`

**B08 — `Vfs` kann nicht verschoben werden, ohne den Fehlertyp zu ändern**
> **Entscheidung:** `Vfs`, `VfsFile`, `VfsOpenOptions`, `VerifiedVia` bleiben in `ports`. `durable-fs` bleibt eigenständig (E0, Tier experimental, kein Verwender). Die Plan-Schritte „`io_seam`/`fsync.rs` nach `durable-fs`“ und „`sys-flush`“ entfallen (Nicht-Ziel). Verbraucher mit Dateisystembedarf (`store`, `checkpoint`) halten eine **crate-lokale** schmale Naht (siehe §6), die Brücke bindet an `ports::Vfs`.

**B09 — `ports` und `types` ohne Zielbild für `DistanceMetric`; mehrere Distanz-Implementierungen**
> **Entscheidung (R4-Ausnahme):** Die drei Enums bleiben je im Besitz ihres Crates. Erlaubt sind kleine geschlossene Wert-Enums (≤ 8 Unit-Varianten) als **gespiegelte Kopie**, wenn (a) Variantennamen und -reihenfolge identisch sind und (b) ein Konformitätstest die serde-Namen aller Kopien gleichsetzt. Verbindung per `From` in `compat/`. `vector` bekommt das optionale Feature `simd`, das `compute_distance` durch `contextra_simd::dispatch` ersetzt; die Skalarpfade bleiben als Orakel. Differenztest §5.4 (Toleranz relativ 1e-5, f32).

**B13 — Dev-Kanten auf `testkit` und Entnahmenachweis**
> **Entscheidung:** Dev-Kanten zu `testkit` sind erlaubt, stehen aber im Manifest `dev_allowed` (§8.3). Der Isolationsnachweis (G-L4) prüft `--edges normal` und baut zusätzlich `--no-dev-deps`; Tests laufen im Export nur mit `--with-dev` (nimmt dann `testkit`-Closure mit). `env` (Feature `test-utils`) übernimmt `ManualClock` und die Clock/Rng-Konformitätssuite, damit E1-Crates dafür nicht auf `testkit` angewiesen sind.

**B16 — Einzelentscheidungen, die der Entwurf offen ließ**
> `ModelFingerprint` (Plan: → `kvcache`; Teil 2 K9: → `ports/kv`) — `ports/kv.rs` importiert ihn aus `types`. Alle Verbraucher der Typ-Variante sind Brücken oder Re-Exports (`crypto` wird auf Roh-Bytes umgestellt). — **Bleibt in `types`.** Kein Umzug. `kvcache` und `infer-candle` führen im Kern einen …

## 10. Entscheidungen und Risiken [S]

**Entscheidungen (§9.1):**

| D-02 | `Vfs` bleibt in `ports`; `durable-fs` eigenständig (B08) |

Nicht-Ziele (§9.3), gelten für alle Crates: physische Regruppierung, Mikro-Crate-Zerlegung (F-1), Verschieben von `Vfs`/`io_seam`, crates.io-Veröffentlichung, Formatänderungen außer ID-Breite, Änderung der FlatBuffers-Schemas.

## 11. Abnahme [S]

Ein Crate gilt als entnehmbar (R6), wenn in dieser Reihenfolge grün: **G-L2** (Null-Prüfungen, nur bei Brücke) → **G-L1** (Kern- und Feature-Closure) → **G-L3** (`--no-default-features`, `--each-feature`) → **G-L4** (isolierter Export). Danach **G-F1/G-D1/G-E1/G-C1** für den betroffenen Schritt (§8.4).

```bash
cargo check -p contextra-ports --no-default-features --locked
cargo hack check -p contextra-ports --each-feature --no-dev-deps --locked
python3 tools/isolation_proof.py --crate contextra-ports --modes no-default,default,each-feature
```
