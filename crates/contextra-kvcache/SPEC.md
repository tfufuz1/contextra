# contextra-kvcache — Spezifikation

> **Geltung:** Crate-lokale Spezifikation. Normativ bleibt `docs/SPEC.md` (v2, 10.11.2026). Dieses Dokument überträgt die Anforderungen für dieses Crate, ergänzt sie um verifizierte Ist-Fakten und markiert Abweichungen.
> **Faktenbasis:** statische Analyse mit `tools/` am geklonten Repo-Stand (`main`, HEAD `f0eda1cb9`). `cargo` ist in der Analyseumgebung nicht verfügbar. **Es wurde nichts gebaut, getestet oder ausgeführt.**
> **Beleg-Marken:** *[V]* am Quelltext/Manifest verifiziert, *[T]* aus einem tools/-Bericht, *[S]* aus docs/SPEC.md übernommen, *[E]* Festlegung dieser Spezifikation.

**Pfad:** `crates/contextra-kvcache/` · **Umsetzungsplan:** `PLAN.md` (im selben Verzeichnis)

## 1. Zweck

KV-Cache für LLM-Inferenz (Radix, Eviction, KIVI-Quantisierung) inkl. KV-Verschlüsselung. *[S]*

## 2. Zielzustand (docs/SPEC.md §3.1) [S]

| Merkmal | Ziel |
|---|---|
| Gruppe | llm |
| Ebene (Kern-Closure) | 1 |
| Kern-Closure | env, crypto (nur `kv-encryption`) |
| Default-Features | `std` |
| Fehlertyp (R3) | `KvCacheError` (besteht) |
| Brücke (hinter Feature `workspace`, §1.1) | `prefix_store`, `adapters`, Purger |

Ebenen (SPEC §1.1): E0 = Kern-Closure leer · E1 = Kern-Closure ⊆ E0, höchstens 4 Mitglieder · E2 = Integration · E3 = Anwendungen und Testwerkzeuge. Der Kern ist der Code mit `--no-default-features` ohne Feature `workspace`.

## 3. Ist-Zustand (tools/)

| Messgröße | Wert |
|---|---|
| Lines of Code (Produktion, `loc.py`) | 4994 Zeilen in 16 Dateien |
| Version / publish | workspace-vererbt / <unset> |
| Interne Normal-Abhängigkeiten (Ist, Cargo.toml) | types, ports, crypto |
| Interne Dev-Abhängigkeiten (Ist) | keine |
| Features (Ist, Anzahl) | 9 |
| Default-Features (Ist) | std, ports-compat, crypto-compat, kv-encryption |
| `cfg(feature = …)`-Stellen in src/ | 55 |
| `#[test]`-Vorkommen (grep, src + tests) | 104 |
| zero-panic: Produktionsbefunde, bereinigt (Tool-Gesamt; davon in Testdateien) | 0 (Tool 0; 0 in Testdateien) |
| unsafe: Vorkommen / ohne SAFETY / Insel | 0 / 0 / False |
| Determinismus: Produktionsbefunde, bereinigt (Tool-Gesamt) | 6 (Tool 6) |
| Port-M3-Leaks (port_contract_audit) | 0 |
| Error-Enums (Ist) | KvCacheError |

**Tool-Caveat:** `zero_panic_analyzer.py` klassifiziert `#[cfg(test)]`-Module (`tests.rs`) als Produktion. Im Repo betrifft das 449 von 478 Treffern. Die Zahlen oben sind deshalb bereinigt; die Rohzahl steht in Klammern. Die Korrektur am Tool ist eigener Schritt (S-00).

**Quellen:** `tools/loc.py`, `tools/zero_panic_analyzer.py`, `tools/unsafe_boundary_auditor.py`, `tools/determinism_guard.py`, `tools/port_contract_auditor.py`, `tools/error_topology_mapper.py`, `tools/cargo_dependency_graph.py`, Cargo.toml-Auswertung.

## 4. Ist-vs-Ziel-Abgleich [V]

- **Ist-Kanten, die das Ziel nicht vorsieht:** `types` (optional), `ports` (optional)
- **Ziel-Kanten, die im Ist fehlen:** `env`
- **Default-Features weichen ab:** Ist `std, ports-compat, crypto-compat, kv-encryption` → Ziel `std`.

## 5. Verifizierte Befunde und Abweichungen zu docs/SPEC.md [V]

- Verifiziert: Duplikat `CURRENT_KV_KEY_DERIVATION_VERSION = 1` in `src/segment.rs:60` (B10). Zur Vereinigung siehe S-38.

## 6. Anforderungen

### 6.1 Crate-Spezifikation (docs/SPEC.md §6) [S]

Brücke: `prefix_store.rs`, `adapters.rs` (inkl. Entfernen von `FeatureRing` aus dem Kern), `store.rs:790–862` (`KvCachePruner`/`KvCacheScanner`), `scratchpad_invalidation.rs`, Port-Teil von `service.rs`, `compat.rs`. Kern: `radix.rs`, `quantize_kivi.rs`, `attention_score.rs`, `eviction_worker.rs`, `segment.rs`, Rest von `store.rs`/`service.rs` auf `KvCacheError`, lokalen `TenantId`, `env::Clock`. `CacheDirective`/`StepId` einmal lokal (heute doppelt). Features: `default = ["std"]`; `kv-encryption` ohne `legacy-erasure`; `kivi-quantization`+`kvcache-kivi-quant` → `exp-kivi`; `content-addressed-kv-cache` → `exp-content-addressed`; `kvcache-attention-eviction` streichen. Aufnahme des KV-Teils aus `crypto` (S-38) als Feature `kv-shredding` (experimentell).

### 6.2 Brückenmuster [S]

Die Brücke liegt ausschließlich in `src/compat/` hinter dem Feature `workspace`. Verbindliche Muster: docs/SPEC.md §4.3 (a–d). R2 gilt: `contextra_types`, `contextra_ports`, `contextra_testkit` dürfen im Kern nirgends vorkommen, auch nicht in Doc-Kommentaren oder `#[cfg(test)]`.

### 6.3 Regeln für jeden Schritt [S]

INV-1 Formatgleichheit · INV-2 Pfadgleichheit (Re-Export mit `#[deprecated]`) · INV-3 Ein Schritt = Commit-Paar · INV-4 `cargo check --workspace --all-targets --locked` grün · INV-5 Determinismus (Tie-Break nach ID aufsteigend) · INV-6 Durability-Pfade unberührt · INV-7 keine Compliance-Aussagen · INV-8 keine Rückwärtspfeile. Vollständig: docs/SPEC.md §1.2, Regeln R1–R9: §3.2.

### 6.5 Fehlerabbildung [S]

Eigener Fehlertyp: `KvCacheError` (besteht) (thiserror, `#[non_exhaustive]`). Jede heute genutzte `ContextraError`-Variante hat genau eine lokale Entsprechung; `Display` bleibt bitgleich; keine Sammelvariante `Other(String)` ohne Quelle. Tabelle je Baustein: `docs/decoupling/errors/<crate>.md` (aus S-xx erzeugt). Quelle: §5.3.

## 7. Schnittstellen und Persistenz

Keine Zeile aus dem Golden-Korpus (§5.1) bezieht sich auf dieses Crate.

### 7.2 Öffentliche Schnittstelle

Ist-Stand und Ziel-Schnittstelle sind in den Abschnitten 3 bis 6 festgehalten. Die exakte Liste der `pub`-Symbole erhebt S-00 über `cargo public-api` (Gate G-A1); erlaubte Differenzen sind nur Re-Export-Pfade mit `#[deprecated]`.

## 8. Fundstellen (tools/, exakt) [T]

**Determinismus (Produktion, bereinigt: 6; die ersten 15):** Tool-Gesamt 6.

- `crates/contextra-kvcache/src/eviction_worker.rs:21` — random_hashmap — `use std::collections::HashMap;`
- `crates/contextra-kvcache/src/eviction_worker.rs:35` — random_hashmap — `segment_map: RwLock<HashMap<(TenantId, u64), RequestId>>,`
- `crates/contextra-kvcache/src/eviction_worker.rs:42` — random_hashmap — `segment_map: RwLock::new(HashMap::new()),`
- `crates/contextra-kvcache/src/store.rs:139` — instant_now — `pinned_at: std::time::Instant::now(),`
- `crates/contextra-kvcache/src/store.rs:207` — instant_now — `let now = std::time::Instant::now();`
- `crates/contextra-kvcache/src/store.rs:240` — instant_now — `let base_time = std::time::Instant::now();`

**Fehler-Enums (Ist):**

- `crates/contextra-kvcache/src/error.rs:8` — KvCacheError

## 9. Befunde aus docs/SPEC.md (B-Nummern) [S]

**B04 — Entkopplungsstand von `text` ist falsch eingeschätzt; Zählungen weichen ab**
> **Entscheidung:** Zahlen sind Orientierung. Abnahme erfolgt ausschließlich über die Null-Prüfung G-L2 (§8.2) und den Build ohne Features, nie über Zählwerte. Aufwandsklasse `text`: mittel.

**B10 — Das Verschieben der Krypto-Dateien bricht Garantie-Nachweise**
> **Entscheidung:** - `legacy-erasure` verlässt die Defaults von `crypto` (S-12). Die Dateien `audit_chain`, `deletion_proof/`, `ed25519_proof` werden **nicht** archiviert, solange nicht (a) ein Item-Diff gegen `erasure` Gleichwertigkeit zeigt **und** (b) der Mutation-Score der Zieldateien in `erasure` ≥ Score der Quelldateien ist (Gate G-C1). Bis dahin bleiben sie, hinter dem Feature, im Crate (Tier frozen). - Wandern `kv_cipher`, `kv_segment/*`, `kv_shredding`, `revocation_log` nach `kvcache`, MUSS derselbe Commit `mutants-guarantees.toml`, `loom-gate.yml` und `CODEOWNERS` auf die neuen Pfade ziehen und beide Versionskonstanten bis zur Vereinigung per `const_assert_eq` koppeln. Nach dem Umzug ist `kvcache` alleiniger Besitzer; `crypto` definiert keine KV-Versionskonstante mehr. - Schnitt und Umzug (S-38) bilden ein untrennbares Commit-Paar.

**B11 — Nebenläufigkeits-Abdeckung fehlt genau dort, wo umgebaut wird**
> **Entscheidung:** S-01 nimmt `mvcc`, `kvcache`, `checkpoint`, `ids`, `env` in die Pfadfilter von `tsan.yml`/`loom-gate.yml`, `mvcc`+`kvcache` in die Crate-Liste der Mutation-Tests und `rust-ci.yml` auf. Enthält `crates/contextra-mvcc/tests` kein Loom-Modell (`rg -l loom crates/contextra-mvcc` leer), ist vor S-30 ein Modell für `SequenceLog` (Publish/Visibility, zwei Schreiber, ein Leser) zu schreiben; sonst ist S-30 nicht abnehmbar.

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
cargo check -p contextra-kvcache --no-default-features --locked
cargo hack check -p contextra-kvcache --each-feature --no-dev-deps --locked
python3 tools/isolation_proof.py --crate contextra-kvcache --modes no-default,default,each-feature
```

