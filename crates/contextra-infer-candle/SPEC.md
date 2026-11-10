# contextra-infer-candle — Spezifikation

> **Geltung:** Crate-lokale Spezifikation. Normativ bleibt `docs/SPEC.md` (v2, 10.11.2026). Dieses Dokument überträgt die Anforderungen für dieses Crate, ergänzt sie um verifizierte Ist-Fakten und markiert Abweichungen.
> **Faktenbasis:** statische Analyse mit `tools/` am geklonten Repo-Stand (`main`, HEAD `f0eda1cb9`). `cargo` ist in der Analyseumgebung nicht verfügbar. **Es wurde nichts gebaut, getestet oder ausgeführt.**
> **Beleg-Marken:** *[V]* am Quelltext/Manifest verifiziert, *[T]* aus einem tools/-Bericht, *[S]* aus docs/SPEC.md übernommen, *[E]* Festlegung dieser Spezifikation.

**Pfad:** `crates/contextra-infer-candle/` · **Umsetzungsplan:** `PLAN.md` (im selben Verzeichnis)

## 1. Zweck

Lokale Inferenz mit Candle: Embedding-Provider, Attention-Export, GASP-Validierung. *[S]*

## 2. Zielzustand (docs/SPEC.md §3.1) [S]

| Merkmal | Ziel |
|---|---|
| Gruppe | llm |
| Ebene (Kern-Closure) | 1 |
| Kern-Closure | env (`kvcache` nur `kv-bridge`) |
| Default-Features | — (`candle` Feature steuert ML) |
| Fehlertyp (R3) | `CandleError` |
| Brücke (hinter Feature `workspace`, §1.1) | Provider, Exporter, Gasp |

Ebenen (SPEC §1.1): E0 = Kern-Closure leer · E1 = Kern-Closure ⊆ E0, höchstens 4 Mitglieder · E2 = Integration · E3 = Anwendungen und Testwerkzeuge. Der Kern ist der Code mit `--no-default-features` ohne Feature `workspace`.

## 3. Ist-Zustand (tools/)

| Messgröße | Wert |
|---|---|
| Lines of Code (Produktion, `loc.py`) | 5150 Zeilen in 20 Dateien |
| Version / publish | workspace-vererbt / <unset> |
| Interne Normal-Abhängigkeiten (Ist, Cargo.toml) | types, ports, rank, crypto |
| Interne Dev-Abhängigkeiten (Ist) | crypto |
| Features (Ist, Anzahl) | 6 |
| Default-Features (Ist) | keine |
| `cfg(feature = …)`-Stellen in src/ | 50 |
| `#[test]`-Vorkommen (grep, src + tests) | 56 |
| zero-panic: Produktionsbefunde, bereinigt (Tool-Gesamt; davon in Testdateien) | 0 (Tool 44; 44 in Testdateien) |
| unsafe: Vorkommen / ohne SAFETY / Insel | 0 / 0 / False |
| Determinismus: Produktionsbefunde, bereinigt (Tool-Gesamt) | 3 (Tool 4) |
| Port-M3-Leaks (port_contract_audit) | 0 |
| Error-Enums (Ist) | Error |

**Tool-Caveat:** `zero_panic_analyzer.py` klassifiziert `#[cfg(test)]`-Module (`tests.rs`) als Produktion. Im Repo betrifft das 449 von 478 Treffern. Die Zahlen oben sind deshalb bereinigt; die Rohzahl steht in Klammern. Die Korrektur am Tool ist eigener Schritt (S-00).

**Quellen:** `tools/loc.py`, `tools/zero_panic_analyzer.py`, `tools/unsafe_boundary_auditor.py`, `tools/determinism_guard.py`, `tools/port_contract_auditor.py`, `tools/error_topology_mapper.py`, `tools/cargo_dependency_graph.py`, Cargo.toml-Auswertung.

## 4. Ist-vs-Ziel-Abgleich [V]

- **Ist-Kanten, die das Ziel nicht vorsieht:** `types`, `ports`, `rank`, `crypto` (optional)
- **Ziel-Kanten, die im Ist fehlen:** `env`
- **Default-Features:** Ist = Ziel (leer).

## 5. Verifizierte Befunde und Abweichungen zu docs/SPEC.md [V]

- Keine zusätzlichen Abweichungen in dieser Prüfung festgestellt. Die Aussagen aus docs/SPEC.md sind nur dort übernommen, wo sie oben nicht widersprochen werden.

## 6. Anforderungen

### 6.1 Crate-Spezifikation (docs/SPEC.md §6) [S]

`candle-core`, `candle-nn`, `candle-transformers`, `tokenizers`, `half` werden optional hinter Feature `candle`; Module `inference`, `gguf_loader`, `embedding`, `model` bekommen `cfg(feature = "candle")`; `fake.rs` → `test-utils`. Brücke: `embedding_provider.rs`, `attention_exporter.rs`, `kv_bridge.rs` (Feature `kv-bridge`, benötigt dann `kvcache`), `gasp.rs` (`GroundingValidator`). `rank`-Kopplung (`gasp.rs:24`, `IsotonicCalibrator`): lokales Trait `ConfidenceCalibrator`, Brücke füllt es mit `rank`. Eigener Newtype `ModelFingerprint` (B16). Features: `exp-kv-stage-b`, `exp-gasp`, `kv-bridge`. Vorbedingung: `rg 'contextra_infer_candle::' crates/` über alle Verbraucher, damit kein Symbol durch das Gaten verschwindet.

### 6.2 Brückenmuster [S]

Die Brücke liegt ausschließlich in `src/compat/` hinter dem Feature `workspace`. Verbindliche Muster: docs/SPEC.md §4.3 (a–d). R2 gilt: `contextra_types`, `contextra_ports`, `contextra_testkit` dürfen im Kern nirgends vorkommen, auch nicht in Doc-Kommentaren oder `#[cfg(test)]`.

### 6.3 Regeln für jeden Schritt [S]

INV-1 Formatgleichheit · INV-2 Pfadgleichheit (Re-Export mit `#[deprecated]`) · INV-3 Ein Schritt = Commit-Paar · INV-4 `cargo check --workspace --all-targets --locked` grün · INV-5 Determinismus (Tie-Break nach ID aufsteigend) · INV-6 Durability-Pfade unberührt · INV-7 keine Compliance-Aussagen · INV-8 keine Rückwärtspfeile. Vollständig: docs/SPEC.md §1.2, Regeln R1–R9: §3.2.

### 6.5 Fehlerabbildung [S]

Eigener Fehlertyp: `CandleError` (thiserror, `#[non_exhaustive]`). Jede heute genutzte `ContextraError`-Variante hat genau eine lokale Entsprechung; `Display` bleibt bitgleich; keine Sammelvariante `Other(String)` ohne Quelle. Tabelle je Baustein: `docs/decoupling/errors/<crate>.md` (aus S-xx erzeugt). Quelle: §5.3.

## 7. Schnittstellen und Persistenz

Keine Zeile aus dem Golden-Korpus (§5.1) bezieht sich auf dieses Crate.

### 7.2 Öffentliche Schnittstelle

Ist-Stand und Ziel-Schnittstelle sind in den Abschnitten 3 bis 6 festgehalten. Die exakte Liste der `pub`-Symbole erhebt S-00 über `cargo public-api` (Gate G-A1); erlaubte Differenzen sind nur Re-Export-Pfade mit `#[deprecated]`.

## 8. Fundstellen (tools/, exakt) [T]

**zero-panic (Produktion, bereinigt: 0; die ersten 15):** Tool-Gesamt 44, davon 44 in Testdateien (`tests.rs` unter `#[cfg(test)]`, Tool-Fehlklassifikation).


**Determinismus (Produktion, bereinigt: 3; die ersten 15):** Tool-Gesamt 4.

- `crates/contextra-infer-candle/src/model/quantized_llama.rs:21` — random_hashmap — `use std::collections::HashMap;`
- `crates/contextra-infer-candle/src/model/quantized_llama.rs:304` — random_hashmap — `pub masks: HashMap<(usize, usize), Tensor>,`
- `crates/contextra-infer-candle/src/model/quantized_llama.rs:439` — random_hashmap — `masks: HashMap::new(),`

**Fehler-Enums (Ist):**

- `crates/contextra-infer-candle/src/error.rs:12` — Error

## 9. Befunde aus docs/SPEC.md (B-Nummern) [S]

**B14 — Defaults und leere Features (Teil 2, G5/G6) — bestätigt, plus Ergänzungen**
> **Entscheidung:** Zieldefaults §3.1. Da `default-members` diese Crates auslässt, genügt „wöchentlich `cargo check`“ nicht als Kompilierbarkeitsgarantie; CI führt bei **jedem** PR `cargo check --workspace --all-targets --locked` aus (S-01). Ein Feature ohne `cfg(feature = "x")` im Quelltext **und** ohne Weiterleitung in einem anderen Manifest wird entfernt; geprüft vom neuen `tools/feature_audit.py` (S-12).

**B16 — Einzelentscheidungen, die der Entwurf offen ließ**
> `ModelFingerprint` (Plan: → `kvcache`; Teil 2 K9: → `ports/kv`) — `ports/kv.rs` importiert ihn aus `types`. Alle Verbraucher der Typ-Variante sind Brücken oder Re-Exports (`crypto` wird auf Roh-Bytes umgestellt). — **Bleibt in `types`.** Kein Umzug. `kvcache` und `infer-candle` führen im Kern einen …

## 10. Entscheidungen und Risiken [S]

Keine Einträge in §9.1 oder §9.2.

Nicht-Ziele (§9.3), gelten für alle Crates: physische Regruppierung, Mikro-Crate-Zerlegung (F-1), Verschieben von `Vfs`/`io_seam`, crates.io-Veröffentlichung, Formatänderungen außer ID-Breite, Änderung der FlatBuffers-Schemas.

## 11. Abnahme [S]

Ein Crate gilt als entnehmbar (R6), wenn in dieser Reihenfolge grün: **G-L2** (Null-Prüfungen, nur bei Brücke) → **G-L1** (Kern- und Feature-Closure) → **G-L3** (`--no-default-features`, `--each-feature`) → **G-L4** (isolierter Export). Danach **G-F1/G-D1/G-E1/G-C1** für den betroffenen Schritt (§8.4).

```bash
cargo check -p contextra-infer-candle --no-default-features --locked
cargo hack check -p contextra-infer-candle --each-feature --no-dev-deps --locked
python3 tools/isolation_proof.py --crate contextra-infer-candle --modes no-default,default,each-feature
```
