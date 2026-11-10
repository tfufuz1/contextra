# contextra-env — Spezifikation

> **Geltung:** Crate-lokale Spezifikation. Normativ bleibt `docs/SPEC.md` (v2, 10.11.2026). Dieses Dokument überträgt die Anforderungen für dieses Crate, ergänzt sie um verifizierte Ist-Fakten und markiert Abweichungen.
> **Faktenbasis:** statische Analyse mit `tools/` am geklonten Repo-Stand (`main`, HEAD `f0eda1cb9`). `cargo` ist in der Analyseumgebung nicht verfügbar. **Es wurde nichts gebaut, getestet oder ausgeführt.**
> **Beleg-Marken:** *[V]* am Quelltext/Manifest verifiziert, *[T]* aus einem tools/-Bericht, *[S]* aus docs/SPEC.md übernommen, *[E]* Festlegung dieser Spezifikation.

**Pfad:** `crates/contextra-env/` · **Umsetzungsplan:** `PLAN.md` (im selben Verzeichnis)

## 1. Zweck

Injizierbare Umwelt: Clock, Rng, IdGen, MetricsSink und deren Standardimplementierungen. *[S]*

## 2. Zielzustand (docs/SPEC.md §3.1) [S]

| Merkmal | Ziel |
|---|---|
| Gruppe | foundation |
| Ebene (Kern-Closure) | 0 |
| Kern-Closure | — |
| Default-Features | `std` |
| Fehlertyp (R3) | — |
| Brücke (hinter Feature `workspace`, §1.1) | — |

Ebenen (SPEC §1.1): E0 = Kern-Closure leer · E1 = Kern-Closure ⊆ E0, höchstens 4 Mitglieder · E2 = Integration · E3 = Anwendungen und Testwerkzeuge. Der Kern ist der Code mit `--no-default-features` ohne Feature `workspace`.

## 3. Ist-Zustand (tools/)

*Crate existiert im Repo noch nicht. Es wird in S-10/S-11 bzw. im jeweiligen Schritt angelegt.*

## 4. Ist-vs-Ziel-Abgleich [V]

Crate existiert noch nicht. Ist-Vergleich entfällt; Ziel siehe §2.

## 5. Verifizierte Befunde und Abweichungen zu docs/SPEC.md [V]

- Neu: Quellbasis `crates/contextra-ports/src/{clock,rng,id_gen,metrics,env}.rs` (alle fünf Dateien vorhanden). Die Clock-/Rng-Varianten sind in B02 belegt.

## 6. Anforderungen

### 6.1 Vertrag (docs/SPEC.md §4.1 bzw. §4.2) [S]

#### 4.2 `contextra-env`

**Zweck:** injizierbare Umwelt (Zeit, Zufall, IDs, Metriken). Quelle: `ports/{clock,rng,id_gen,metrics,env}.rs` (109+208+101+335+16 Z. [V]; keine `ContextraError`-Nutzung [V]).

**Dependencies:** nur `std`. `#![forbid(unsafe_code)]`.

**Vertrag:**

```rust
pub trait Clock: Send + Sync + 'static {
    fn now_unix_nanos(&self) -> u64;     // 0 vor Epoche, nie Panic
    fn monotonic_nanos(&self) -> u64;
}
pub struct SystemClock;
pub fn unix_ms(c: &dyn Clock) -> u64 { c.now_unix_nanos() / 1_000_000 }   // Abrunden

pub trait Rng: Send + Sync + 'static {
    fn next_u64(&self) -> u64;
    fn fill_bytes(&self, dest: &mut [u8]);
    fn next_unit_f64(&self) -> f64;
    // + alle weiteren Methoden von ports::Rng 1:1 (Datei vollständig übernehmen)
}
pub struct SeededRng;                    // SplitMix64, Folge MUSS bitgleich bleiben
pub trait IdGen: Send + Sync + 'static { /* wie ports::IdGen */ }
pub struct SequentialIdGen;
pub trait MetricsSink: Send + Sync + 'static { /* wie ports::MetricsSink */ }
pub struct NoopMetricsSink; pub struct InMemoryMetricsSink; pub struct MetricKey; // + Snapshots
// Feature `test-utils`: pub struct ManualClock;  (aus testkit/manual_clock.rs, 172 Z.)
```

`ports` macht `pub use contextra_env::{Clock, SystemClock, Rng, SeededRng, IdGen, SequentialIdGen, MetricsSink, NoopMetricsSink, InMemoryMetricsSink, MetricKey /* … */};` und behält die Module `clock`, `rng`, `id_gen`, `metrics`, `env` als dünne Re-Export-Dateien, damit `contextra_ports::clock::Clock` gültig bleibt (R9).

**Abnahme:**
1. Golden-Folgen von `SeededRng` (Seeds 0, 1, 42, `u64::MAX`; je 1.024 Werte) vor dem Umzug aus `ports` erzeugt, danach bitgleich (S-02).
2. Konformitätssuite (aus `testkit/conformance/{clock,rng,metrics}.rs`) läuft im Crate `env` unter `test-utils`.
3. `unix_ms`-Differenztest gegen `now_unix_ms` der Kopien in `text`/`graph`/`vector/contract.rs` mit identischer Quelle (`ManualClock`).

### 6.3 Regeln für jeden Schritt [S]

INV-1 Formatgleichheit · INV-2 Pfadgleichheit (Re-Export mit `#[deprecated]`) · INV-3 Ein Schritt = Commit-Paar · INV-4 `cargo check --workspace --all-targets --locked` grün · INV-5 Determinismus (Tie-Break nach ID aufsteigend) · INV-6 Durability-Pfade unberührt · INV-7 keine Compliance-Aussagen · INV-8 keine Rückwärtspfeile. Vollständig: docs/SPEC.md §1.2, Regeln R1–R9: §3.2.

### 6.4 ID-Breite [S]

Persistierte Formate mit IDs tragen `id_width_bits: u16` (64|128). Öffnen mit abweichender Breite ⇒ `…::IdWidthMismatch { stored, build }`, kein Panic, keine Teilmutation (docs/SPEC.md §5.2, B03).

## 7. Schnittstellen und Persistenz

Keine Zeile aus dem Golden-Korpus (§5.1) bezieht sich auf dieses Crate.

### 7.2 Öffentliche Schnittstelle

Ist-Stand und Ziel-Schnittstelle sind in den Abschnitten 3 bis 6 festgehalten. Die exakte Liste der `pub`-Symbole erhebt S-00 über `cargo public-api` (Gate G-A1); erlaubte Differenzen sind nur Re-Export-Pfade mit `#[deprecated]`.

## 8. Fundstellen (tools/, exakt) [T]

Keine Befunde in den Audit-Berichten.

## 9. Befunde aus docs/SPEC.md (B-Nummern) [S]

**B01 — Drei Dokumente, drei Architekturen**
> **Entscheidung:** - Es gibt kein `modules/`-Verzeichnis. Die Brücke ist ein **Modul** `compat/` im Crate (kein Extra-Crate). - `ids`/`env` sind für alle Bausteine erlaubte E0-Abhängigkeiten. Die Seam-Definitionen `Clock`, `Rng`, `MetricsSink`, `IdGen` der Schwester-Specs entfallen zugunsten `env`; `DocKey`/`ExternalId`-Typen zugunsten `ids`. `docid-128` gehört **ausschließlich** `ids` (B03). - Die Mikro-Crate-Zerlegung der Schwester-Specs ist **Folgeprojekt F-1** und nicht Teil dieser Spezifikation (§9.3).

**B02 — Die `Clock`-/`Rng`-Kopien sind nicht kompatibel (Teil 2, G1 ist unvollständig)**
> **Entscheidung:** - `env::Clock` = `ports::Clock` (Nanosekunden, `monotonic_nanos`, `Send+Sync+'static`). `env::Rng` = `ports::Rng` mit `&self` (siehe §4.2). - Millisekunden-Verbraucher (`text`, `graph`, `vector/contract.rs`) benutzen `env::Clock` und `env::unix_ms(&dyn Clock) -> u64` = `now_unix_nanos() / 1_000_000` (Abrunden). Persistierte ms-Zeitstempel MÜSSEN im Differenztest (§5.4) bitgleich zu `now_unix_ms` der alten Implementierung bei identischer Quelle sein. - `&mut self`-Verbraucher `text`/`graph`/`vector` wechseln auf `&self`-`env::Rng`. Das ist für alle heutigen Aufrufstellen ein Superset (Mutex/Atomic innerhalb `SeededRng`). Der Umstieg ist ein **Umbau-Commit** (INV-3 B). - E0-Crates (`adapt`, `privacy`, `license`, `erasure`, `ingest`, `audit-export`) behalten ihre lokalen Traits (R7/K8). Ihre Brücke bindet per Blanket-Impl **von `env` auf lokal** (Richtung `&self → &mut …

**B11 — Nebenläufigkeits-Abdeckung fehlt genau dort, wo umgebaut wird**
> **Entscheidung:** S-01 nimmt `mvcc`, `kvcache`, `checkpoint`, `ids`, `env` in die Pfadfilter von `tsan.yml`/`loom-gate.yml`, `mvcc`+`kvcache` in die Crate-Liste der Mutation-Tests und `rust-ci.yml` auf. Enthält `crates/contextra-mvcc/tests` kein Loom-Modell (`rg -l loom crates/contextra-mvcc` leer), ist vor S-30 ein Modell für `SequenceLog` (Publish/Visibility, zwei Schreiber, ein Leser) zu schreiben; sonst ist S-30 nicht abnehmbar.

**B13 — Dev-Kanten auf `testkit` und Entnahmenachweis**
> **Entscheidung:** Dev-Kanten zu `testkit` sind erlaubt, stehen aber im Manifest `dev_allowed` (§8.3). Der Isolationsnachweis (G-L4) prüft `--edges normal` und baut zusätzlich `--no-dev-deps`; Tests laufen im Export nur mit `--with-dev` (nimmt dann `testkit`-Closure mit). `env` (Feature `test-utils`) übernimmt `ManualClock` und die Clock/Rng-Konformitätssuite, damit E1-Crates dafür nicht auf `testkit` angewiesen sind.

**B15 — Mehr nicht verifizierbare Plan-Annahmen**

## 10. Entscheidungen und Risiken [S]

**Entscheidungen (§9.1):**

| D-11 | Ebene-0-Crates behalten lokale Mini-Traits (R7/K8); E1+ nutzen `ids`/`env` |

Nicht-Ziele (§9.3), gelten für alle Crates: physische Regruppierung, Mikro-Crate-Zerlegung (F-1), Verschieben von `Vfs`/`io_seam`, crates.io-Veröffentlichung, Formatänderungen außer ID-Breite, Änderung der FlatBuffers-Schemas.

## 11. Abnahme [S]

Ein Crate gilt als entnehmbar (R6), wenn in dieser Reihenfolge grün: **G-L2** (Null-Prüfungen, nur bei Brücke) → **G-L1** (Kern- und Feature-Closure) → **G-L3** (`--no-default-features`, `--each-feature`) → **G-L4** (isolierter Export). Danach **G-F1/G-D1/G-E1/G-C1** für den betroffenen Schritt (§8.4).

```bash
cargo check -p contextra-env --no-default-features --locked
cargo hack check -p contextra-env --each-feature --no-dev-deps --locked
python3 tools/isolation_proof.py --crate contextra-env --modes no-default,default,each-feature
```
