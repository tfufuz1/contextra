# Contextra als Baukasten — Implementierungsspezifikation v2 (normativ)

Stand: 10. November 2026 · Ersetzt: „Entkopplungs- und Entnahmeplan“ (Teil 1) und „Implementierungsspezifikation der übrigen Crates“ (Teil 2) · Repo: `tfufuz1/contextra`

---

## 0. Status, Geltung, Methode

### 0.1 Grundlage und Grenzen

| Punkt | Inhalt |
|---|---|
| Analysierter Stand | Flacher Klon, HEAD `ef3364f` („Shell-Commit“, 10.11.2026 01:00). Die Entwürfe nennen `80f1c33`. Ob und was sich zwischen beiden Ständen geändert hat, ist **nicht** geprüft; S-00 erhebt die Baseline neu. |
| Methode | Ausschließlich statische Analyse: Cargo-Manifeste (Python/`tomllib`), `rg`/`grep`-Zählungen, Dateivergleiche, Lesen von Quelltext. |
| Nicht möglich | In der Analyseumgebung fehlt `cargo`. **Nichts wurde gebaut, getestet oder ausgeführt.** Jede Zeile „Nachweis“ in diesem Dokument ist ein Abnahmekriterium für die Umsetzung, kein Ergebnis. |
| Beleg-Marken | **[V]** von mir am Quelltext/Manifest verifiziert. **[E]** aus den Entwürfen übernommen, nicht neu geprüft. **[A]** aus [V] abgeleitet. Eine Aussage ohne Marke ist eine Festlegung (Entscheidung) dieser Spezifikation. |
| Plan-Datei im Repo | `docs/decoupling/Contextra als Baukasten Entkopplungs- und Entnahmeplan.md` ist byte-identisch zur Anlage (`cmp`) [V]. Daneben liegen dort `Implementierungsspezifikation_{store,vektor,graph}.md` (Schwester-Specs). |
| Nicht auflösbare Verweise | Die Entwürfe verweisen auf `RESTRUCTURE_PLAN.md` (WP-xx, D1, E1–E7, Gate G3). Die Datei existiert weder im Repo noch in der Anlage [V]. Diese Spezifikation definiert die benötigten Regeln selbst (§3.3, X1–X8) und **verweist nirgends** auf diese Datei. |

### 0.2 Rangfolge

1. Dieses Dokument gilt für: Crate-Grenzen, Abhängigkeitsrichtung, `ids`/`env`, Brücken, Fehlerabbildung, Features, Reihenfolge, Gates.
2. Die Schwester-Specs (`store`, `vector`, `graph`) gelten nur für die **Binnenzerlegung** der drei Crates und erst nach Abschluss von Phase 5. Wo sie widersprechen, gilt §2 B01.
3. `AGENTS.md` und `docs/**` sind nachrangig und werden in S-03 an dieses Dokument angepasst.

### 0.3 Normative Sprache

MUSS / DARF NICHT / SOLL gemäß RFC 2119. „Commit“ = ein Schritt-Commit (siehe INV-3).

---

## 1. Begriffe und Grundinvarianten

### 1.1 Begriffe

- **Kern** eines Crates: sein Code mit `--no-default-features`, ohne Feature `workspace`.
- **Brücke:** Verzeichnis `src/compat/` (Modul `compat`) hinter dem Feature `workspace`. Nur dort stehen `contextra_types`, `contextra_ports` sowie alle Implementierungen von `ports`-Traits und alle `From`-Konvertierungen nach `ContextraError`.
- **Kern-Closure:** Menge der internen `contextra-*`-Crates in `cargo tree -p <crate> -e normal --no-default-features`.
- **Feature-Closure:** wie oben, je einzeln aktiviertem Feature.
- **Ebene** wird an der **Kern-Closure** gemessen:
  - **E0:** Kern-Closure leer.
  - **E1:** Kern-Closure ⊆ {E0-Crates}, höchstens 4 Mitglieder (Ausnahme `store`, §3.1).
  - **E2:** Integration; darf E0/E1/E2 nutzen, nie E3.
  - **E3:** Anwendungen und Testwerkzeuge.
- **Mikro-Kerne:** `contextra-ids`, `contextra-env`. Sie sind E0 und für **jeden** E1-/E2-/E3-Baustein als Kern-Abhängigkeit erlaubt (Regel R7 ersetzt hierfür K8 des Entwurfs nicht, sondern ergänzt sie).

### 1.2 Invarianten (gelten für jeden Schritt)

| ID | Invariante |
|---|---|
| INV-1 | **Formatgleichheit:** Kein persistiertes Byte ändert sich. Nachweis: Golden-Korpus (§5.1) vor und nach dem Schritt identisch. |
| INV-2 | **Pfadgleichheit:** Öffentliche Importpfade bleiben für eine Übergangsfrist gültig (Re-Export mit `#[deprecated(since, note)]`). Entfernung nur in S-70. |
| INV-3 | **Ein Schritt, ein Commit-Paar:** Commit A verschiebt oder benennt um (kein Umbau), Commit B baut um. Beide einzeln revertierbar. |
| INV-4 | **Build-Pfad bleibt grün:** `cargo check --workspace --all-targets --locked` nach jedem Commit. |
| INV-5 | **Determinismus:** Gleiche Eingabe ⇒ gleiche Rangfolge; Tie-Break nach ID aufsteigend [V: `AGENTS.md` §1.1]. |
| INV-6 | **Durability-Pfade bleiben unberührt**, sofern der Schritt sie nicht ausdrücklich nennt (WAL, Manifest, Flush, Recovery). |
| INV-7 | **Keine Compliance-Aussagen** in Code/Docs (`AGENTS.md` §1.2). Gate G-DOC1 (§8). |
| INV-8 | **Kein Rückwärtspfeil:** Abhängigkeiten zeigen nur nach unten (E3→E2→E1→E0). Experimente zeigen auf Stabiles, nie umgekehrt. |

---

## 2. Korrekturen und Blindstellen (Befund → bindende Entscheidung)

### B01 — Drei Dokumente, drei Architekturen

**Befund [V]:** Der Plan sieht `crates/<gruppe>/`, **ein** Crate je Baustein mit `compat/`-Modul und die Mikro-Kerne `ids`/`env` vor. Die Schwester-Specs sehen `modules/<x>/<baustein>/`, viele Mikro-Crates (`store-io`, `store-wal`, `vector-hnsw`, `graph-csr` …), ein eigenes `*-contextra-compat`-**Crate** und crate-lokale Seam-Traits `Clock`/`Rng`/`MetricsSink`/`FileIo` vor („Kein Baustein importiert `contextra_*`“). Die Vektor-Spec legt `docid-128` an `vector-types` (M-3), der Plan an `ids`. Keines der Dokumente erwähnt das andere.

**Entscheidung:**
- Es gibt kein `modules/`-Verzeichnis. Die Brücke ist ein **Modul** `compat/` im Crate (kein Extra-Crate).
- `ids`/`env` sind für alle Bausteine erlaubte E0-Abhängigkeiten. Die Seam-Definitionen `Clock`, `Rng`, `MetricsSink`, `IdGen` der Schwester-Specs entfallen zugunsten `env`; `DocKey`/`ExternalId`-Typen zugunsten `ids`. `docid-128` gehört **ausschließlich** `ids` (B03).
- Die Mikro-Crate-Zerlegung der Schwester-Specs ist **Folgeprojekt F-1** und nicht Teil dieser Spezifikation (§9.3).

### B02 — Die `Clock`-/`Rng`-Kopien sind nicht kompatibel (Teil 2, G1 ist unvollständig)

**Befund [V]** (Signaturen direkt gelesen):

| Ort | `Clock` | `Rng` |
|---|---|---|
| `ports/clock.rs:8`, `rng.rs:12` | `now_unix_nanos`, `monotonic_nanos`; `Send+Sync+'static` | `next_u64(&self)`, `fill_bytes(&self,…)`, `next_unit_f64(&self)`; `'static` |
| `vector/types.rs:169/203` | wie ports, ohne `'static` | `next_u64(&self)`, `next_f64(&self)` |
| `vector/contract.rs:39/45` | **`now_unix_ms`** (Millisekunden) | **`&mut self`**, `Send` |
| `text/contract.rs:16/19`, `graph/contract.rs:15/17` | **`now_unix_ms`** | **`&mut self`**, `Send` |
| `adapt/{clock,contract,rng}.rs` | nanos + monotonic; (`clock.rs` mit `'static`) | **`&mut self`**, `next_f64`/`next_f32` |
| `license/types.rs:122`, `rank/.../isotonic.rs:14` | `monotonic_nanos` mit Default-Impl | — |
| `erasure/types_local.rs:2059`, `privacy/types.rs:11` | nanos + monotonic | — |

Der Entwurf behauptet, `Rng` sei nur in `adapt` abweichend; tatsächlich weichen auch `text`, `graph` und `vector/contract.rs` ab, und `Clock` unterscheidet sich in der **Einheit** (ms vs. ns).

**Entscheidung:**
- `env::Clock` = `ports::Clock` (Nanosekunden, `monotonic_nanos`, `Send+Sync+'static`). `env::Rng` = `ports::Rng` mit `&self` (siehe §4.2).
- Millisekunden-Verbraucher (`text`, `graph`, `vector/contract.rs`) benutzen `env::Clock` und `env::unix_ms(&dyn Clock) -> u64` = `now_unix_nanos() / 1_000_000` (Abrunden). Persistierte ms-Zeitstempel MÜSSEN im Differenztest (§5.4) bitgleich zu `now_unix_ms` der alten Implementierung bei identischer Quelle sein.
- `&mut self`-Verbraucher `text`/`graph`/`vector` wechseln auf `&self`-`env::Rng`. Das ist für alle heutigen Aufrufstellen ein Superset (Mutex/Atomic innerhalb `SeededRng`). Der Umstieg ist ein **Umbau-Commit** (INV-3 B).
- E0-Crates (`adapt`, `privacy`, `license`, `erasure`, `ingest`, `audit-export`) behalten ihre lokalen Traits (R7/K8). Ihre Brücke bindet per Blanket-Impl **von `env` auf lokal** (Richtung `&self → &mut self` und `'static → ohne`), nie umgekehrt (§4.3).

### B03 — `docid-128` ändert die Typbreite und ist nicht additiv

**Befund [V]:** Das Feature existiert in 6 Manifesten (`types`, `store`→`types`, `text`, `vector`, `graph` als leeres Feature, `py`→`types`). `types::DocId` ist `u64` (`repr(transparent)`) oder `u128` (`repr(C, align(16))`); `vector/types.rs:25` benutzt für `u128` stattdessen `repr(transparent)`. `DocId::from_key` nimmt die ersten 8 (bzw. 16) Bytes von BLAKE3 [V: `ids.rs:373–385`]. In `store/manifest/` und `store/wal/*.rs` fand ich **keine** Kennung der ID-Breite (nur `MFMN` + Version 1); `vector/persistence/header.rs` hat `HNSW_VERSION = 2` — ob dort ein Breitenfeld existiert, habe ich nicht gelesen. Cargo-Feature-Unification kann `docid-128` in `types` einschalten, während `vector`/`text`/`graph` ihre leeren/lokalen Kopien auf 64 Bit lassen (Typfehler oder, schlimmer, stilles Abschneiden).

**Entscheidung:**
1. `ids` besitzt `docid-128` allein. Jeder Crate mit Abhängigkeit auf `ids` leitet `docid-128 = ["contextra-ids/docid-128"]` weiter; leere/lokale Features entfallen. E0-Crates ohne `ids` (`ingest`, `erasure`) bekommen **kein** `docid-128`; ihre Konvertierung von `ids::DocId` ist `TryFrom` (fehlschlagend bei Überlauf), nie `as`.
2. `ids` führt `pub const ID_WIDTH_BITS: u32` (64 oder 128) und die Compile-Time-Prüfung `size_of`/`align_of` (8/8 bzw. 16/16).
3. Jedes persistierte Format, das IDs trägt, MUSS die Breite beim Öffnen prüfen (§5.2). Fehlt das Feld heute, wird die Formatversion erhöht und v(alt) als „Breite unbekannt“ nur mit expliziter Konfiguration geöffnet.
4. CI-Matrix: `cargo hack check --workspace --features docid-128` zusätzlich zum Default.

### B04 — Entkopplungsstand von `text` ist falsch eingeschätzt; Zählungen weichen ab

**Befund [V]:** Meine Zählung (Zeilen mit `contextra_(types|ports|mvcc|sys|crypto)` in `src/`, ohne Test-Pfade und Kommentarzeilen; Datei gilt als „gegatet“, wenn sie irgendeine `cfg(…compat|workspace…)`-Zeile enthält):

| Crate | Referenzen gesamt | davon in Dateien **ohne** jedes compat-`cfg` (Dateien) | Entwurf (ungegatet) |
|---|---|---|---|
| vector | 186 | 146 (36) | 114 (32) |
| graph | 135 | 133 (54) | 134 (50) |
| kvcache | 65 | 57 (10) | 55 (9) |
| checkpoint | 65 | 65 (6) | 72 (7) |
| mvcc | 8 | 8 (5) | 11 (5) |
| **text** | 64 | **32 (6)** | **8 (4)** |
| store | 160 | 160 (50) | — |
| crypto | 67 | 67 (15) | — |
| infer-candle | 43 | 43 (12) | — |
| rank / simd / infer-ollama / license / audit-export | 40 / 13 / 50 / 30 / 4 | 0 | 0 |

Bei `text` fehlen im Entwurf `wand.rs` (20 Referenzen, Zeilen 18–19: `StorageEngine`, `ContextraError`, `DocId`, `MAX_SEARCH_K`; kein `cfg`), `inverted/scanner.rs` und `inverted/index_struct.rs`. `text` ist **nicht** „nahe am Ziel“. Dass `infer-ollama` 0 ungegatete Dateien hat, bedeutet nur „jede Datei enthält irgendein `cfg`“; die Entwurfs-Heuristik (siehe dort, F3) überschätzt dort die Gatung.

**Entscheidung:** Zahlen sind Orientierung. Abnahme erfolgt ausschließlich über die Null-Prüfung G-L2 (§8.2) und den Build ohne Features, nie über Zählwerte. Aufwandsklasse `text`: mittel.

### B05 — Die Gate-Infrastruktur, auf die das Konzept sich stützt, ist nicht durchsetzend

**Befund [V]:**
- `.cargo/config.toml` definiert `xtask = "run --package xtask --"`; ein Package `xtask` existiert nicht (kein Verzeichnis, nicht in `members`). `governance/gates.toml` und `protected-paths.toml` verweisen auf `xtask`; `ring-dag`, `capabilities-consistency`, `unsafe-inventory` sind als `builtin` (xtask) deklariert, `ring-dag` steht zudem in `ratchet.toml › not_run_baseline`.
- `tests/layering.rs::test_gate_g1_layering_and_isolation` prüft nur, dass der Bericht **nicht leer** ist; ein Verstoß lässt den Test nicht scheitern. `tools/gate_g1_layering.py` ruft `cargo` auf und bricht ohne `cargo` mit Traceback ab (in meiner Umgebung).
- `tests/arch` (`contextra-arch-tests`, „Ring-Schichtung“) ist **kein** Workspace-Mitglied.
- Die in `gates.toml` verlangte Prüfung `public-api` läuft auf `nightly-2024-11-01`, der Workspace verlangt `rust-version = "1.89"` [V] ⇒ vermutlich nicht lauffähig [A].

**Entscheidung:** S-01 stellt her: ein einziges, mit Exit-Code ≠ 0 scheiterndes Layering-Gate (§8.1), Anschluss an CI, Entfernen der toten `xtask`-Verweise, Aufnahme oder Entfernung von `tests/arch`, funktionierender `public-api`-Lauf. Ohne S-01 DARF kein weiterer Schritt gemergt werden.

### B06 — `capabilities.toml` und `AGENTS.md` widersprechen sich

**Befund [V]:** `AGENTS.md` erklärt Ringe/Klassen/Wellen für dekommissioniert. `capabilities.toml` führt weiter `klasse`, `ring`, `may_depend_on`; z. B. `contextra-adapt` mit `may_depend_on = ['contextra-ports','contextra-types']`, obwohl `adapt` keine interne Abhängigkeit hat. `types/src/lib.rs` Zeile 20–28 nennt 30 Verwender inklusive eines nicht existierenden Crates `contextra-agreement-generator` [V]; tatsächlich 24 Verwender [V].

**Entscheidung:** Schema v3 (§8.3) ersetzt `klasse/ring`. `may_depend_on` wird aus dem Ziel (§3.1) erzeugt, nicht aus dem Ist.

### B07 — Eine Verzeichnis-Regruppierung bricht messbar viel

**Befund [V]:** Pfadabhängigkeiten außerhalb der Quelltexte: mindestens 9 Workflow-Dateien mit 30+ Pfadzeilen (`build-wheels`, `contextra-py-ci`, `fuzz-extended`, `loom-gate`, `publish-pypi`, `rust-ci`, `simd-cross-validation`, `tsan`, `unwrap-gate`; `mutation-testing` nennt Crate-Namen; meine Ausgabe war auf 30 Zeilen begrenzt), `capabilities.toml` (`path = …`), `CODEOWNERS` (je Crate), `justfile`, vier `fuzz/Cargo.toml` mit `path = "../../contextra-…"` (graph, crypto, vector, store), `include_str!("../../contextra-ports/src/storage.rs")` in `store/tests/toctou_put_if_absent.rs`, `db/tests/search_result_bound.rs` (Pfad zu `engine`), `.agents/rules`, 100+ Doku-Verweise.

**Entscheidung (D-01):** Das Verzeichnislayout bleibt **flach** (`crates/contextra-*`). Die Gruppierung steht als Metadatum `group` in `capabilities.toml`. Die physische Regruppierung ist Nicht-Ziel (§9.3); sie liefert keinen Entnahmegewinn und kostet die oben genannten Brüche.

### B08 — `Vfs` kann nicht verschoben werden, ohne den Fehlertyp zu ändern

**Befund [V]:** `ports/vfs.rs` (404 Z.) nutzt `crate::Result` (= `ContextraError`) in den Datei-Methoden (58 Treffer); nur `VfsFile` nutzt `std::io::Result`. Die Plan-Anweisung „`vfs` nach `durable-fs`, `ports` re-exportiert“ würde (a) `ports` von `durable-fs` und transitiv `sys` abhängig machen und (b) den Rückgabetyp aller Implementierer (`store::io_seam::RealFs`, `testkit::FaultVfs`) und Nutzer (`store`, `checkpoint`, `router`) ändern. `clock.rs`, `rng.rs`, `id_gen.rs`, `metrics.rs` (zusammen 753 Z.) enthalten dagegen **keine** `ContextraError`-Nutzung [V] und lassen sich sauber verschieben.

**Entscheidung:** `Vfs`, `VfsFile`, `VfsOpenOptions`, `VerifiedVia` bleiben in `ports`. `durable-fs` bleibt eigenständig (E0, Tier experimental, kein Verwender). Die Plan-Schritte „`io_seam`/`fsync.rs` nach `durable-fs`“ und „`sys-flush`“ entfallen (Nicht-Ziel). Verbraucher mit Dateisystembedarf (`store`, `checkpoint`) halten eine **crate-lokale** schmale Naht (siehe §6), die Brücke bindet an `ports::Vfs`.

### B09 — `ports` und `types` ohne Zielbild für `DistanceMetric`; mehrere Distanz-Implementierungen

**Befund [V]:** Drei gleichnamige Enums mit identischen Varianten und Reihenfolge (`Cosine`, `Euclidean`, `DotProduct`): `types/.../document.rs:20` (`#[non_exhaustive]`, nur `PartialEq`, **mit** `compute()`), `vector/types.rs:130` (`Eq+Hash`), `simd/metric.rs:5` (`Eq+Hash`, `serde` optional). Der Plan macht `simd` zum Besitzer und `vector → simd` optional — das ist nicht umsetzbar, weil `vector/distance.rs` die Metrik immer braucht. Zusätzlich existieren mindestens drei Distanzberechnungen (`types::DistanceMetric::compute`, `vector/distance.rs`, `simd`-Kernel); weitere Skalarpfade (z. B. in `quantize/`) habe ich nicht geprüft.

**Entscheidung (R4-Ausnahme):** Die drei Enums bleiben je im Besitz ihres Crates. Erlaubt sind kleine geschlossene Wert-Enums (≤ 8 Unit-Varianten) als **gespiegelte Kopie**, wenn (a) Variantennamen und -reihenfolge identisch sind und (b) ein Konformitätstest die serde-Namen aller Kopien gleichsetzt. Verbindung per `From` in `compat/`. `vector` bekommt das optionale Feature `simd`, das `compute_distance` durch `contextra_simd::dispatch` ersetzt; die Skalarpfade bleiben als Orakel. Differenztest §5.4 (Toleranz relativ 1e-5, f32).

### B10 — Das Verschieben der Krypto-Dateien bricht Garantie-Nachweise

**Befund [V]:** `mutants-guarantees.toml` bindet G2 (Löschung/Krypto) an `crypto/src/{kv_shredding,kv_cipher,deletion_proof,ed25519_proof,kdf}.rs` und G3 (Audit) an `audit_chain.rs`, `wal_crypto.rs`. `loom-gate.yml` hat `crypto/tests/loom_kv_deferred_zeroize.rs`. `CURRENT_KV_KEY_DERIVATION_VERSION` existiert doppelt: `crypto/kv_segment/segment.rs:21` und `kvcache/segment.rs:60`; `CURRENT_KV_FORMAT_VERSION = 2` in `kv_cipher.rs:46`. Die gleichnamigen Dateien in `erasure` sind **keine** Kopien (z. B. `deletion_proof.rs` 2.744 Z. in `erasure` gegenüber rund 1.450 Z. in `crypto` [E]).

**Entscheidung:**
- `legacy-erasure` verlässt die Defaults von `crypto` (S-12). Die Dateien `audit_chain`, `deletion_proof/`, `ed25519_proof` werden **nicht** archiviert, solange nicht (a) ein Item-Diff gegen `erasure` Gleichwertigkeit zeigt **und** (b) der Mutation-Score der Zieldateien in `erasure` ≥ Score der Quelldateien ist (Gate G-C1). Bis dahin bleiben sie, hinter dem Feature, im Crate (Tier frozen).
- Wandern `kv_cipher`, `kv_segment/*`, `kv_shredding`, `revocation_log` nach `kvcache`, MUSS derselbe Commit `mutants-guarantees.toml`, `loom-gate.yml` und `CODEOWNERS` auf die neuen Pfade ziehen und beide Versionskonstanten bis zur Vereinigung per `const_assert_eq` koppeln. Nach dem Umzug ist `kvcache` alleiniger Besitzer; `crypto` definiert keine KV-Versionskonstante mehr.
- Schnitt und Umzug (S-38) bilden ein untrennbares Commit-Paar.

### B11 — Nebenläufigkeits-Abdeckung fehlt genau dort, wo umgebaut wird

**Befund [V]:** `loom-gate.yml` löst nur für `store`, `crypto`, `vector` aus; `tsan.yml` nur für `store`, `vector`, `graph`; `mutation-testing.yml` und `rust-ci.yml` (Zeile 385) kennen `mvcc` nicht. `mvcc` enthält SSI, `SequenceLog` und `TxBuffer` — den Nebenläufigkeitskern, den `store` und `vector` benutzen — sowie `kvcache` (`eviction_worker`, `radix`).

**Entscheidung:** S-01 nimmt `mvcc`, `kvcache`, `checkpoint`, `ids`, `env` in die Pfadfilter von `tsan.yml`/`loom-gate.yml`, `mvcc`+`kvcache` in die Crate-Liste der Mutation-Tests und `rust-ci.yml` auf. Enthält `crates/contextra-mvcc/tests` kein Loom-Modell (`rg -l loom crates/contextra-mvcc` leer), ist vor S-30 ein Modell für `SequenceLog` (Publish/Visibility, zwei Schreiber, ein Leser) zu schreiben; sonst ist S-30 nicht abnehmbar.

### B12 — Die Fehlerabbildung ist unterspezifiziert

**Befund [V]:** `ContextraError` ist `#[non_exhaustive]` mit 43 Varianten. `ids` wirft heute `ContextraError::InvalidInput("Key cannot be empty")` (`ids.rs:226–229`); weitere Validierungsstellen bei Zeilen 14, 64, 98 [E]. Mit lokalem `IdError` ändert sich der **Rückgabetyp** inhärenter Methoden (`DocId::from_key`, `EntityId::from_key`, Ref-Konstruktoren), denn `ids`-Typen sind in `types` nur noch Re-Exports; inhärente Methoden fremder Typen lassen sich nicht wrappen. Aufrufer von `::from_key(` außerhalb von `types`: Treffer-Zählung 85, **doppelt gezählt** (überlappende Suchpfade) ⇒ real etwa 40–85; S-00 misst exakt.

**Entscheidung:** §5.3 (Abbildungsvertrag) und S-10 mit separatem Aufrufstellen-Commit. Ausgangspunkt der Tabelle: **Display-Text und Variante bleiben bitgleich** (Test je Variante).

### B13 — Dev-Kanten auf `testkit` und Entnahmenachweis

**Befund [V]:** `testkit` (E3) ist Dev-Abhängigkeit von `checkpoint`, `crypto`, `mvcc`, `store` (E1) sowie `agent`, `cognition`, `db`, `engine`. Ein „Export in ein leeres Verzeichnis“ (R6 d) scheitert entweder oder nimmt `testkit` und damit `ports`/`types` mit.

**Entscheidung:** Dev-Kanten zu `testkit` sind erlaubt, stehen aber im Manifest `dev_allowed` (§8.3). Der Isolationsnachweis (G-L4) prüft `--edges normal` und baut zusätzlich `--no-dev-deps`; Tests laufen im Export nur mit `--with-dev` (nimmt dann `testkit`-Closure mit). `env` (Feature `test-utils`) übernimmt `ManualClock` und die Clock/Rng-Konformitätssuite, damit E1-Crates dafür nicht auf `testkit` angewiesen sind.

### B14 — Defaults und leere Features (Teil 2, G5/G6) — bestätigt, plus Ergänzungen

**Befund [V]:** Defaults: `adapt` 3 Experimente; `crypto` `legacy-erasure, kv-encryption, test-utils`; `graph` `types-compat, ports-compat, experimental-graph`; `kvcache` `ports-compat, crypto-compat, kv-encryption`; `license` `ports-compat`; `audit-export` `types-compat`; `rank`/`text` `types-compat`; `vector` `ports-compat`; `testkit` `erasure-conformance`. Nicht in `default-members` stehen u. a. `ingest`, `privacy`, `sandbox`, `infer-candle`, `py`, `bench`, `erasure`, `license`, `audit-export`, `avv-generator`.

**Entscheidung:** Zieldefaults §3.1. Da `default-members` diese Crates auslässt, genügt „wöchentlich `cargo check`“ nicht als Kompilierbarkeitsgarantie; CI führt bei **jedem** PR `cargo check --workspace --all-targets --locked` aus (S-01). Ein Feature ohne `cfg(feature = "x")` im Quelltext **und** ohne Weiterleitung in einem anderen Manifest wird entfernt; geprüft vom neuen `tools/feature_audit.py` (S-12).

### B15 — Mehr nicht verifizierbare Plan-Annahmen

- „Kein Verwender“ [V, nur Top-Level-`[dependencies]`/`[dev-dependencies]`/`[build-dependencies]` aller Manifeste inkl. `benchmarks/`; **nicht** ausgewertet: `[target.*.dependencies]`, `[patch]`, `examples/`, `tests/` im Wurzelverzeichnis]. S-00 ersetzt das durch `cargo metadata`.
- `wire/build.rs` verweist auf `../../schemas/contextra.fbs`, relativ zum Crate-Verzeichnis ein nicht existierender Pfad [V: nur `crates/contextra-wire/schemas/` existiert]. Die FlatBuffers-Regeneration läuft damit nie; der eingecheckte Code wird benutzt. **Entscheidung:** nicht ändern (Regeneration wäre eine Verhaltensänderung); stattdessen G-F2: CI prüft `contextra.fbs.sha256`.
- Crates tragen kein `publish = false`; interne Abhängigkeiten sind reine `path`-Abhängigkeiten ohne `version`. Eine crates.io-Veröffentlichung ist so nicht möglich. **Entscheidung:** „Entnahme“ = Git-Abhängigkeit (`git`+`rev`) oder Pfadkopie; crates.io ist Nicht-Ziel; neue Crates erhalten `publish = false`.
- `ids`/`env` MÜSSEN `#![forbid(unsafe_code)]` tragen; `governance/ratchet.toml › forbid_unsafe_crates = 30` steigt auf 32 (ADR-pflichtig, geschützter Pfad).

### B16 — Einzelentscheidungen, die der Entwurf offen ließ

| Thema | Befund [V] | Entscheidung |
|---|---|---|
| `ModelFingerprint` (Plan: → `kvcache`; Teil 2 K9: → `ports/kv`) | `ports/kv.rs` importiert ihn aus `types`. Alle Verbraucher der Typ-Variante sind Brücken oder Re-Exports (`crypto` wird auf Roh-Bytes umgestellt). | **Bleibt in `types`.** Kein Umzug. `kvcache` und `infer-candle` führen im Kern einen eigenen Newtype `ModelFingerprint(pub [u8; 32])`; die Konvertierung steht in `kvcache/compat.rs` (existiert) und neu in `infer-candle/compat/`. Schritt P5 des Entwurfs entfällt. |
| `RetrievalStrategy` | `types` und `adapt` führen verschiedene Gestalten | Bleibt in `types` (K10); `router/arm_registry.rs` bildet ab. |
| `contextra-vault` vs. `sys/vault.rs` | Öffentliche Namen gemeinsam: 3; `sys::vault` = gesperrter Speicher (`allocate`, `as_slice`, `is_locked`), `db::volatile_vault` = Chunk-Tresor (`ingest`, `commit`, `SignalModality`) [V] | **Aufbauend, nicht ersetzend:** `contextra-vault` (E1, Kern-Closure `sys`) nutzt `sys::vault`. |
| `mcp/protocol.rs` vs. `wire/jsonrpc.rs` | 5 gemeinsame, 7 nur-mcp (`McpError`, Tiefen-/Batch-Validierung), 10 nur-wire [V] | **Kein Merge**; beide bleiben. Dublette nur im Typnamen, nicht in der API. |
| `adapt/drift` vs. `rank/drift` | 2 gemeinsame Namen (`adapt`: 8 eigene, `rank`: 7 eigene) [V] | **Keine Dubletten**, keine Konsolidierung. |
| `contextra/query_rewriter` vs. `infer-ollama/query_rewriter` | 1 gemeinsamer Name [V] | **Keine Dubletten**, keine Konsolidierung. |
| `decay_controller` adapt/engine | 9 gemeinsam; `engine` hat 5 zusätzliche (`with_clock`, `set_clock`, `effective_half_life_ms`, `effective_score_wall_clock`, `should_evict_wall_clock`), `adapt` 1 (`config`) [V] | `adapt` bleibt Besitzer, **muss zuerst die 5 Methoden aufnehmen**; Zusammenlegung erst nach Differenztest (S-60). Der Entwurf übersah die Richtung. |
| `homeostat` adapt/db | 4 gemeinsam; `adapt` 11, `db` 3 zusätzliche (`is_expired`, `pool_size`, `remaining`) [V] | Wie oben: `adapt` nimmt die 3 Methoden auf, dann `pub use` in `db`. |
| `pid_latency_controller` adapt/db | 10 gemeinsam; 6 vs. 5 zusätzliche [V] | Wie oben, 5 Methoden aufnehmen. |
| `sketched-bandit` (leeres Feature) | kein `cfg(feature="sketched-bandit")` [E] | **Feature streichen**; Code bleibt unter `bandit-routing`. Verhalten unverändert. |
| `AvvError`/`AvvGeneratorError` | beide exportiert [E] | Item-Diff in S-03; sind sie identisch: `type`-Alias, sonst unverändert beide. |
| Optionale Plan-Schritte `store→durable-fs`, `checkpoint` in `mvcc` | — | Entfallen (B08) bzw. abgelehnt (Datei-Snapshot-Mechanik ≠ Transaktionsisolation). |

---

## 3. Zielzustand

### 3.1 Crate-Tabelle (final)

Ebene = Kern-Ebene. „Fehler“ = eigener Fehlertyp (R3). „Brücke“ = Inhalt von `compat/`.
`std` steht nur dort als Default, wo das Feature heute existiert (nicht bei `graph`, `checkpoint`, `erasure`); `workspace` ist **nie** Default.

| Crate | Gruppe (Meta) | Ebene | Kern-Closure (Ziel) | Default-Features (Ziel) | Fehler | Brücke (hinter `workspace`) |
|---|---|---|---|---|---|---|
| `ids` (neu) | foundation | 0 | — | `std` | `IdError` | — |
| `env` (neu) | foundation | 0 | — | `std` | — | — |
| `sys` | foundation | 0 | — | — | `SysError` (besteht) | — |
| `durable-fs` | foundation | 0 | — | `std`, `crc` | besteht | — |
| `simd` | foundation | 0 | — | `std` | `SimdError` (besteht) | `From<DistanceMetric>`↔`types` |
| `wire` | foundation | 0 | — | `std` | besteht | — |
| `mvcc` | foundation | 1 | ids, env | — | `MvccError` (neu) | `From<MvccError>`, Port-Impls |
| `crypto` | foundation | 1 | ids, env | **leer** | `CryptoError` (besteht) | `From<CryptoError>` |
| `types` | contracts | 2 | ids | — | `ContextraError` | — |
| `ports` | contracts | 2 | types, ids, env | — | — | — |
| `store` | storage | 1* | ids, env, sys, mvcc, crypto | — | `StoreError` (neu) | `engine/`, `scanner/`, `From` |
| `vector` | retrieval | 1 | ids, env, mvcc (`sys` nur `mmap`, `simd` nur `simd`) | `std` | `VectorError` | `VectorIndex`-Impls, Purger, `From` |
| `text` | retrieval | 1 | ids, env | `std` | `TextError` (neu) | `TextIndex`-Impl, Purge, `From` |
| `graph` | retrieval | 1 | ids, env | **leer** (kein `std`-Feature vorhanden; ohne `experimental-graph`) | `GraphError` (neu) | `GraphIndex`, Purger, `From` |
| `rank` | retrieval | 1 | ids, env | `std` | `RankError` | `From` |
| `checkpoint` | retrieval | 1 | ids, env | — | `CheckpointError` (neu) | `Checkpoint`-Impls, `From` |
| `kvcache` | llm | 1 | env, crypto (nur `kv-encryption`) | `std` | `KvCacheError` (besteht) | `prefix_store`, `adapters`, Purger |
| `router` | llm | 2 | types, ports, adapt (`wire` nur `subprocess-dispatch`) | — | besteht | — |
| `adapt` | llm | 0 | — | `std` | besteht | Blanket-Impl `env→lokal` |
| `infer-ollama` | llm | 1 | env | — | `OllamaError` | `trait_impls`, `query_rewriter`, `From` |
| `infer-candle` | llm | 1 | env (`kvcache` nur `kv-bridge`) | — (`candle` Feature steuert ML) | `CandleError` | Provider, Exporter, Gasp |
| `infer-onnx` | llm | 0 | — | — | besteht | `ports::EmbeddingProvider`-Anbindung |
| `ingest` | data | 0 | — | `fmt-text` | besteht | `ids`/`types::ContextChunk`-Mapping |
| `privacy` | safety | 0 | — | `std` | besteht | — |
| `sandbox` | safety | 0 | — | — | besteht | — |
| `prompt-guard` (neu) | safety | 0 | — | `std` | `GuardError` | — |
| `vault` (neu) | safety | 1 | sys | — | `VaultError` | `From` |
| `agent` | agentic | 2 | types, ports | — | `AgentError` (neu) | Speicherbindung an `db`/`graph`/`checkpoint`/`router` |
| `cognition` | agentic | 2 | types, ports, graph (nur `exp-hyperedge`) | — | `CognitionError` (neu) | Bindung an `engine` liegt in **`db`** (Feature `cognition`) |
| `erasure` | compliance | 0 | — | `signing`, `vault`, `async` (Verhalten wie heute) | besteht | Scanner-Abbildung |
| `license` | compliance | 0 | — | `std` | besteht | `ports`/`types` (aus Inline-Impls) |
| `audit-export` | compliance | 0 | — | `std` | besteht | `types`-`From` |
| `avv-generator` | compliance | 0 | — | `std` | besteht | — |
| `engine` | integration | 2 | alle benötigten | — | `ContextraError` | — |
| `db` | integration | 2 | engine, types, ports | — | — | — |
| `contextra` | integration | 2 | db, types, ports (`license`, `infer-candle` optional) | `embedded` | — | — |
| `mcp` | apps | 3 | contextra, types, ports, wire, rank, crypto, privacy, router, prompt-guard | — | `McpError` | — |
| `py` | apps | 3 | types, contextra, wire | — | — | — |
| `testkit` | testing | 3 | ports, types, env | `testkit` (ohne `erasure-conformance`) | — | — |
| `bench` | testing | 3 | store, vector, text, db | — | — | — |

\* `store` ist die einzige Ausnahme vom Budget „≤ 4 Mitglieder“ (5: ids, env, sys, mvcc, crypto): WAL-HMAC (`crypto`) und Sequenzen (`mvcc`) sind konstitutiv. Das Budget zählt die **Kern-Closure**; Feature-Closures dürfen größer sein.
`router` ist E2, obwohl es nur E0/E2-Mitglieder zieht, weil `types`/`ports` E2 sind.

### 3.2 Regeln

- **R1 Richtung:** Abhängigkeiten zeigen nach unten (INV-8).
- **R2 Brücke:** `contextra_types`/`contextra_ports`/`contextra_testkit` DÜRFEN im Kern **nirgends** vorkommen, auch nicht in Doc-Kommentaren und `#[cfg(test)]`-Modulen des Kerns (Tests des Kerns nutzen `env::test-utils` und lokale Fakes). Ausnahme: `tests/`-Verzeichnis (Integrationstests) hinter `required-features = ["workspace"]`.
- **R3 Eigene Fehler:** `thiserror`; `#[non_exhaustive]`. `impl From<LocalError> for ContextraError` nur in `compat/` (Orphan-Regel erfüllt, da `LocalError` lokal [A]).
- **R4 Ein Besitzer je Konzept:** IDs → `ids`; `Clock`/`Rng`/`IdGen`/`MetricsSink` → `env`; `Tombstone*` → `mvcc`; `PprConfig`/`PprAlgorithm`/`TlHfdParams` → `graph`; KV-Verschlüsselung → `kvcache`; Lizenz-Traits → `license`. **R4-Ausnahme** gespiegelter Wert-Enums (B09).
- **R5 Experimente:** siehe X1–X8 (§3.3).
- **R6 Nachweis statt Behauptung:** „Entnehmbar“ gilt nur bei G-L1…G-L4 (§8) in Folge grün.
- **R7 E0-Crates definieren ihre Umwelt-Traits lokal und minimal;** die Verbindung zu `env`/`ports` liegt ausschließlich in `compat/` (Muster §4.3).
- **R8 Kein `unwrap`/`expect`/Panic** in neuen Kern-Pfaden von `ids`/`env`; Tests ausgenommen (Gate `unwrap-gate`).
- **R9 `pub use`-Übergang:** Verlässt ein Typ ein Crate, bleibt am alten Ort ein `pub use` mit `#[deprecated(since = "0.1.0", note = "…neuer Pfad…")]`, außer wenn dadurch ein Rückwärtspfeil entstünde (z. B. `types → mvcc` für `Tombstone`; dort wird der Pfad ersatzlos entfernt, weil nur `mvcc` und `types` selbst ihn nutzen [V]).

### 3.3 Experimente (X1–X8)

- **X1** Experiment-Code hängt nur von Stabilem ab. Der Host-Kern kennt keine Experimente.
- **X2** Feature `exp-<name>`; nie in `default`; Sammelfeature `experimental-all`; in der CI-Matrix.
- **X3** Die Naht (Trait) liegt im Host-Kern in `ext.rs`, ohne `types`/`ports`.
- **X4** Eigenes Crate, wenn > 2.000 Zeilen **oder** eigene externe Abhängigkeiten **oder** ≥ 2 Hosts.
- **X5** Verhaltens-Baseline im selben Crate (z. B. Recall gegen Brute-Force).
- **X6** Persistierte Experimentdaten haben eigene Magic-/Versionskennung; das Kernformat ändert sich nie.
- **X7** Tier `experimental` in `capabilities.toml`.
- **X8** Ein Experiment, das Daten schreibt, behält den Lesepfad oder dokumentiert eine Migrationssperre.

---

## 4. Spezifikation der neuen Kerne

### 4.1 `contextra-ids`

**Zweck:** alle Identitäts- und Mandantentypen, ohne interne Abhängigkeit.

**Inhalt (Quelle → Ziel)** [E, Zeilenzahlen nicht neu gezählt]:
- `types/src/types/domain/ids.rs`: `DocId`, `EntityId`, `TxId` (inkl. `INTERNAL_BASE`), `TenantId` (inkl. `SYSTEM`), `CollectionId`; `from_key` per BLAKE3-Präfix.
- `types/src/ids.rs`: `FileId`, `StoreId`, `Seq`, `Timestamp`. `ErasureId` und `GateId` gehen **nicht** hierher, sondern nach `erasure` (S-51).
- `types/src/refs.rs`: `CollectionRef`, `DocKey`, `OperatorRef`, `SubjectRef`, `TenantRef`, `MAX_REF_LEN`.
- `types/src/tenant_scope.rs`: `TenantScoped`, `TenantScopeViolation`.
- Konstanten `TOMBSTONE_BIT`, `MAX_SEARCH_K` aus `types/.../misc.rs`.
- Tests `types/.../tests/ids_tests.rs` (473 Z.) ziehen mit.

**Dependencies:** `blake3`, `serde`, `thiserror`. Keine weiteren. `#![forbid(unsafe_code)]`.

**Öffentliche Schnittstelle (Vertrag):**

```rust
// Feature `docid-128` wählt die Breite für das GANZE Workspace-Build (nicht additiv; siehe B03).
#[cfg(not(feature = "docid-128"))] pub struct DocId(pub u64);        // repr(transparent)
#[cfg(feature = "docid-128")]     pub struct DocId(pub u128);       // repr(C, align(16))
pub const ID_WIDTH_BITS: u32 = if cfg!(feature = "docid-128") { 128 } else { 64 };

#[non_exhaustive]
#[derive(Debug, thiserror::Error, PartialEq, Eq, Clone)]
pub enum IdError {
    #[error("Key cannot be empty")] EmptyKey,
    // weitere Varianten 1:1 aus den heutigen InvalidInput-Stellen
    // (Zeilen 14, 64, 98 in ids.rs sowie refs.rs/tenant_scope.rs, per S-10 erhoben):
    // Text der Display-Ausgabe MUSS dem heutigen String entsprechen.
}

impl DocId {
    pub fn from_key(key: &str) -> Result<Self, IdError>;   // BLAKE3[..8] LE (64) bzw. [..16] (128)
    pub fn inner(&self) -> /* u64 | u128 */;
}
impl From<u64> for DocId { /* wie heute: id as _ */ }
```

**Re-Export:** `types` macht `pub use contextra_ids::*;` (Pfad `contextra_types::DocId` bleibt) und enthält `impl From<IdError> for ContextraError` (Variante `InvalidInput(String)`, Text = `IdError::to_string()`).
**Kompatibilitätsbruch (bewusst):** `DocId::from_key` & Co. liefern `Result<_, IdError>`; `?` in Funktionen mit `ContextraError`-Fehlertyp funktioniert über `From`. Direkte Musterabgleiche auf `ContextraError::InvalidInput` an Aufrufstellen sind in S-10 Commit C zu korrigieren.

**Abnahme (zusätzlich zu §8):**
1. `ids_tests.rs` grün in `ids`.
2. Differenztest gegen die **noch bestehenden** Kopien (`types`, `vector`, `text`, `graph`, `rank`) über 10.000 zufällige Schlüssel + Randfälle (leer, Unicode, 4 KiB): gleiche `from_key`-Bytes und gleiche serde-Ausgabe (JSON und `bincode`).
3. Golden-Vektoren für `from_key` in 64 **und** 128 Bit (S-02).
4. `static_assert` auf `size_of`/`align_of`.

### 4.2 `contextra-env`

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

### 4.3 Brückenmuster (verbindlich)

**a) `Clock` E0-lokal ⇐ `env`:**

```rust
// crates/contextra-<x>/src/compat/env.rs  (feature = "workspace")
impl<T: contextra_env::Clock + ?Sized> crate::Clock for T {
    fn now_unix_nanos(&self) -> u64 { contextra_env::Clock::now_unix_nanos(self) }
    fn monotonic_nanos(&self) -> u64 { contextra_env::Clock::monotonic_nanos(self) }
}
```
Voraussetzung: das lokale Trait ist lokal (Orphan erfüllt); der Crate hat **keine** konkrete `impl crate::Clock for T` mit eigenem `T`, die mit dem Blanket überlappt — sonst Überlappung (E0119). Konkrete lokale Typen (`SystemClock` des Crates) werden hinter `cfg(not(feature = "workspace"))` oder als Newtype geführt. Vorlage: `license/types.rs:282` [V].

**b) `Rng` (`&mut self`) ⇐ `env::Rng` (`&self`):** Blanket-Impl lokal-von-`env`, weil `&self` jederzeit als `&mut self` aufrufbar ist. Die umgekehrte Richtung (`&mut` → `env`) ist verboten; E1-Kerne nutzen ausschließlich `env::Rng`.

**c) Fehler:** `impl From<VectorError> for contextra_types::ContextraError` nur in `compat/error.rs`; Variantenabbildung nach §5.3.

**d) Port-Implementierung:** `impl contextra_ports::VectorIndex for HnswIndex` nur in `compat/`; Kern-Trait-Methoden ohne `ContextraError`.

---

## 5. Persistenz-, Fehler- und Verhaltensvertrag

### 5.1 Golden-Format-Korpus (S-02, vor jedem Umbau)

Das Verzeichnis `tests/golden/` (Dateien + `MANIFEST.sha256`) enthält Byte-Ausgaben, erzeugt vom **unveränderten** HEAD:

| Format | Quelle / Kennung [V] |
|---|---|
| Manifest | `store/manifest/core.rs`: `MFMN`, Version 1 |
| SSTable | `store/sstable/block_cache.rs`: `MFSX`, Legacy `MFST` (Footer, Blöcke, Bloom) |
| WAL | `store/wal/*` inkl. HMAC-Kette (`wal_crypto`) |
| Rollback-Intent | `store/lsm/recovery.rs`: `MFRLBK\0\0` |
| HNSW | `vector/persistence/header.rs`: `0x484E5357`, Version 2 |
| DiskANN | `vector/diskann/format.rs`: `DANN` v1, `DANF`, `PWAL` v1, `TWAL` v1 |
| KDF-Header | `crypto/kdf.rs`: `MFKD` |
| KV-Segment | `crypto/kv_cipher.rs`: Formatversion 2; Schlüsselableitung Version 1 |
| Text-Postings, Graph-Persistenz (bincode), Checkpoint-Meta/Manifest | aus `text`, `graph`, `checkpoint` zu erheben |
| ID-Serialisierung | `DocId`, `EntityId`, `TxId`, `TenantId`, Refs in JSON und `bincode`, je 64/128 Bit |
| `ContextraError` | `Display` aller 43 Varianten |
| `SeededRng` | Folgen (B02) |

Je Eintrag: Erzeugertest (`#[ignore]` + Script `tools/golden_regen.sh`), Prüftest (läuft immer). Bei Abweichung bricht der Schritt ab; eine absichtliche Formatänderung ist nie Teil dieser Spezifikation.

### 5.2 ID-Breite (B03)

- Jedes Format mit IDs bekommt ein Feld `id_width_bits: u16` (64|128). `store`: Manifest-Version 1 → **2** (Kopf: `MFMN`, Version, `id_width_bits`); v1-Dateien werden mit `OpenOptions::assume_legacy_id_width(Bits)` geöffnet, sonst mit `StoreError::IdWidthUnknown` abgelehnt. `vector`: `HNSW_VERSION` 2 → 3 analog. `text`, `graph`, `checkpoint`: nach Lesen des jeweiligen Headers dasselbe Muster.
- Öffnen mit abweichender Breite ⇒ `…::IdWidthMismatch { stored, build }`, **kein** Panic, **keine** Teilmutation.
- Schreibe-Reihenfolge der Versionsanhebung: erst Leser (liest v(alt)+v(neu)), danach Schreiber (S-37). Rückwärts-Lesetest mit Korpus-Dateien aus S-02.

### 5.3 Fehlerabbildung

Pro Baustein eine Tabelle `docs/decoupling/errors/<crate>.md` (aus S-xx erzeugt) mit Spalten: lokale Variante → `ContextraError`-Variante → Display-String. Anforderungen:
1. Jede **heute genutzte** `ContextraError`-Variante des Kerns hat genau eine lokale Entsprechung (Erhebung: `rg 'ContextraError::[A-Z]' src`).
2. `Display` des konvertierten Fehlers ist bitgleich zum alten Fehler (Table-driven-Test).
3. Es gibt keine Sammelvariante `Other(String)`, die Kontext verwirft, außer für I/O (`std::io::Error` als Quelle, `#[source]`).
4. Gate `error-domain` (steht in `ratchet.toml › not_run_baseline`) wird in S-01 so definiert und in S-12 aktiviert.

### 5.4 Differenztests (Verhaltensgleichheit)

Jede Ablösung einer Kopie durch ihren Besitzer wird zuerst durch einen Differenztest abgesichert, der **alte und neue** Implementierung gleichzeitig kompiliert (Commit A) und gleiche Eingaben vergleicht:
- IDs: §4.1. Clock/Rng/ms: §4.2.
- Distanz: `types::DistanceMetric::compute` ↔ `vector::distance` ↔ `simd::dispatch` über 10.000 Vektorpaare (Dim 4/17/128/768, normale, Null-, Denormal-Werte): relative Abweichung ≤ 1e-5 bzw. **bitgleich** für Pfade, die heute dieselbe Skalarfunktion nutzen; NaN/Inf-Verhalten gleich.
- Regler (`decay`, `homeostat`, `pid`): gleiche Eingabefolge mit `ManualClock` ⇒ gleiche Ausgaben (Toleranz 0, f64 bitgleich), vor jedem `pub use` (S-60).
- Chunker (`engine::chunker` ↔ `ingest::chunking::prose`): identische Chunk-Folge, Reihenfolge und Breadcrumbs auf einem Korpus (S-61).
Nach grünem Differenztest folgt Commit B (Ersetzung); der Test bleibt als Regressionstest bestehen, solange beide Seiten existieren.

---

## 6. Spezifikation je Crate (final)

Pfade relativ zu `crates/contextra-<name>/src/`. Zeilenzahlen aus dem Entwurf [E].

### 6.1 E0 (Kern-Closure leer)

- **`sys`:** unverändert; `lib.rs:19` korrigieren (behauptet `durable-fs` baue darauf auf; beleglos). `vault.rs` bleibt (B16). Tier beta.
- **`durable-fs`:** unverändert (B08); Tier experimental; Doku: „kein Verwender; unabhängig von `sys`“.
- **`simd`:** `types-compat` → `workspace`, Brücke nach `compat/` (13 Referenzen, laut Manifest/Zählung ungegatet 0 [V]). `DistanceMetric` bleibt lokal (B09). Sonst unverändert. Tier beta.
- **`wire`:** unverändert; generierten Code im LOC-Report getrennt führen; `router`-Referenzen: nur `dispatch.rs:23` [E] ⇒ `wire` bei `router` optional (S-62).
- **`adapt`:** (1) `clock.rs`+`contract.rs` und `rng.rs`+`contract.rs` zu je **einem** Trait zusammenführen; `Rng` behält `&mut self` (R7). (2) `off_policy.rs` (6 Z.) entfernen, wenn `rg 'off_policy::'` leer. (3) `default = ["std"]`; `router` aktiviert `bandit-routing` und die Experimente gezielt. (4) `sketched-bandit` streichen (B16). (5) Features umbenennen: `exp-flow-thompson`, `exp-rie-greedy`; Aliasnamen **nicht** behalten (nur `router` setzt sie).
- **`privacy`:** Docstring-Aussage „Ausgehend: types, ports“ streichen [V: falsch]. Features ohne `cfg`-Nutzung streichen (B14). Keine Codeänderung.
- **`sandbox`:** keine Änderung; Doku-Hinweis, dass `mcp/sandbox.rs` ein anderes Ding ist.
- **`ingest`:** `fuzz_support.rs` hinter `fuzz`; Brücke `compat/` (`ids::DocId` ↔ lokales `DocId(u64)` per `TryFrom` im 128-Bit-Modus; `types::ContextChunk`-Mapping); kein Split nach Format. Chunker-Parität: S-61.
- **`erasure`:** Features `signing` (`ed25519-dalek`), `vault` (`aes-gcm-siv`), `async` (`tokio`, `futures-util`); Zuordnung Datei→Abhängigkeit per `rg 'use (ed25519_dalek|aes_gcm_siv|tokio|futures_util|bincode)'` in S-24 erheben und im Manifest festschreiben. Typen aus `types/erasure.rs` (616), `ports/{erasure,erasure_legacy,gate,ledger,observer,report_signer,keys,barrier}.rs` übernehmen **nach** Abgleich mit `types_local.rs`: identischer Name+Gestalt ⇒ eine Definition; abweichend ⇒ beide bleiben, Brücke wandelt. Scanner-Traits `Purger`/`LayerScanner` bleiben lokal **und** in `ports::scanner` (R7). `default = ["signing", "vault", "async"]` (erhält das heutige Verhalten); Nutzer ohne `tokio` wählen `--no-default-features`.
- **`avv-generator`:** Doku „Ring 4“ entfernen; nur Item-Diff `AvvError`/`AvvGeneratorError` (B16); Textzusagen in `template.rs` mit den tatsächlichen Features abgleichen (G-DOC1).
- **`license`:** `default = ["std"]`; Inline-Brücken (`types.rs:~210–290`, `signed_gate.rs:802`) → `compat/`. Danach bekommt `license` aus `ports` (S-50): `license.rs` ohne `VectorDeleteMode`, `policy.rs` (`LicenseGate`); `VectorDeleteMode` → `vector/hnsw/config.rs`.
- **`audit-export`:** `default = ["std"]`; Brücke `lib.rs:106–118` → `compat/`; `testkit.rs` hinter `test-utils`. Die Brücke `ProcessingRegistryEntry → AuditRecord` (Feature `privacy`) wird **nicht** gebaut (Nicht-Ziel).
- **`infer-onnx`:** `fakes` hinter `test-utils`; leere Features (`experimental`, `inspect`) streichen; `onnx` und `download` bleiben getrennt prüfbar.
- **`prompt-guard` (neu):** Quelle `mcp/src/prompt_injection/{audit,guard,policy,tests}.rs` (1.162 Z.) + Wrapper `prompt_injection.rs` (57 Z.); Imports nur `std`, `serde` [E]; `SecurityAuditLogger` (schreibt per `OpenOptions`) hinter `file-audit`. `mcp` re-exportiert (`lib.rs:26,53`).

### 6.2 E1

- **`mvcc`:** `tombstone.rs` aus `types` verbatim hierher (Commit A; nur `mvcc` und `types` nutzen ihn [V]); danach Umbau (Commit B): IDs aus `ids`, `Clock`/`MetricsSink` aus `env`, `MvccError`; `types` verliert die Re-Exports ersatzlos. Vorbedingung: Loom-Modell (B11). Port-Impls und `From` in `compat/`. Kern-Closure: ids, env.
- **`crypto`:** Kern bleibt: `aead` (190), `dek` (153), `kdf` (319), `secret` (146), `error` (146), `anti_tamper` (191), `crypto.rs`/`KeyManager` (948), `wal_crypto` (1.163), `wal_completeness` (75). `TenantId`/`TenantScoped` aus `ids`. `KeyManager::{derive_kv_key, derive_kv_key_scoped, cipher_for, cipher_for_scoped}` (`crypto.rs:238–316`) nehmen `[u8; 32]` statt `kv_cipher::ModelFingerprint`; vor dem Umzug des KV-Teils umstellen (eigener Commit, Aufrufer in `kvcache`/`infer-candle` mit). `default = []`. `ed25519-dalek` bleibt, solange `ed25519_proof` im Crate bleibt (B10). KV-Teil und Altlast: B10, S-38.
- **`text`:** Neues lokales Trait `PostingStore` = **exakt** die in `src/` aufgerufenen `StorageEngine`-Methoden (Erhebung: `rg 'storage\.' src` und alle `StorageEngine`-Aufrufe in `wand.rs`, `inverted*`, `stream.rs`, `memory_index.rs`); keine Methode mehr als genutzt. Brücke implementiert es für `StorageEngine`. In `compat/`: `inverted/purge.rs`, `inverted/scanner.rs`, `inverted/morph_index.rs` (`TextIndex`), Fehlerkonvertierung, `wand.rs`-Teile, die `ports` direkt brauchen (nach B04). `types.rs` (130) und `contract.rs` (39) → `pub use` auf `ids`/`env`; `docid-128` an `ids` (B03). Leere Features (`experimental`, `bm25f`, `serde`, `tracing`, `inspect`) per Audit streichen. `default = ["std"]`.
- **`rank`:** `local_types.rs:10` (`DocId`) → `ids`; `calibration/isotonic.rs:14` (`Clock`) → `env`; `types-compat` → `workspace`/`compat/`. `engine/src/temporal_filter.rs` (383) → `rank/temporal.rs`. `dibud` → Feature `exp-dibud`, `fusion/resonance.rs` → `exp-resonance` (Naht `fusion/ext.rs`); `drift` bleibt (B16: keine Dublette).
- **`graph`:** Brücke: `provider.rs`, `purger.rs`, `csr/graph_index/*` und alle Importe von `ports::{graph_index,graph}`; lokale `DocId`/`TxId` (`types.rs:58,104`) → `ids`; `default = []`. `PprConfig`, `PprAlgorithm`, `TlHfdParams` (aus `types/misc.rs` ab Z. 375) wandern hierher (S-51); `engine`/`cognition` importieren dann aus `graph`. Experimente: Klasse A/B laut Entwurf (Blätter `hyperedge_suggest`, `entity_extraction`, `percolation`, `edge_reinforcement_buffer`; mit Naht `apprh`, `tl_hfd`, `session_dag`, `edge_reinforcement`, `community`); Klasse C (`hyperedge` 19 Inbound-Dateien, `path_rag` 14, `consistency_enforcement` 6) bleibt im Kern, optional `exp-hyperedge`. Ab > 2.000 Z. (X4): Crate `contextra-graph-research` (S-81). `ports/graph.rs` gibt `HyperEdgeId`/`RoleBinding`/`RoleId` hinter `exp-hyperedge` frei.
- **`checkpoint`:** lokales Trait `SnapshotSource` = exakt die in `src/` aufgerufenen `StorageEngine`-Methoden (29 Referenzen [E]); lokale schmale Dateisystemnaht (`hardlink_cloner.rs`) statt `ports::Vfs` (B08); `CheckpointError`; `WorkflowState` lokal abbilden; Brücke: `ports::{Checkpoint, CheckpointCoordinator}`-Impls, `From`. Kern-Closure: ids, env. Nicht mit `mvcc` zusammenlegen.
- **`vector`:** Brücke: `hnsw/vector_index_impl.rs`, `diskann/vector_index_impl.rs`, `hnsw/purge.rs`, `hnsw/scanner.rs`, `error.rs:61–91`, `types.rs:279ff`. Kern: `hnsw/types.rs` (5, 128–143, 203–210, 241–242, 279), `quantize/mod.rs`, `hnsw/core_{rebuild,search,insert}.rs`, `persistence/*`, `distance.rs`, `candidate_stream.rs` auf `VectorError`/`ids`/`env`/`mvcc`. `contract.rs` (119 Z.) wird zu `env`/`ids`-basiert vereint: `Clock` (ms→`unix_ms`), `Rng` (`&self`), `ExternalId(u128)` → `ids::DocId`-Pfad nur, wenn serde/`from_key` unberührt; sonst Kopie entfällt **nicht** (Differenztest entscheidet, Abbruch bei Abweichung). `VectorDeleteMode` hierher (S-50). `default = ["std"]` (kein `ports-compat`). `mvcc` bleibt Kern-Abhängigkeit (`SequenceLog`/`TxBuffer`-Nutzung) — die Schwester-Spec M-5-Alternative (eigener `VisibilityLog`) ist Nicht-Ziel. Feature `simd` (B09). Feature-Schema: `exp-diskann`, `exp-rabitq`, `exp-acorn`, `experimental-all`; `diskann/` (12 Dateien, 4.168 Z.) → Crate `contextra-vector-diskann` (S-80), `quantize_rabitq.rs` → `experimental/rabitq.rs`; Regel V-VETO: `PartialRebuildConfig` bleibt aus (Test `partial_rebuild_veto_f02_guard` MUSS grün bleiben, Pfad per `rg` bestätigen [E]).
- **`store`:** (1) Eigener `StoreError`; Abbildung der heute genutzten Varianten (`Storage`, `Io`, `WalCorruption`, `ChecksumMismatch`, `Transaction*`, `Conflict`, `WalTruncationDetected`, `CrossDeviceLink`) nach §5.3. (2) `compat/`: `engine/` (370), `scanner/` (1.400), `From<StoreError>`; `StorageEngine`-Impl nur dort. (3) `lsm/config.rs:68–92` (`validate_against_features`, `FeatureRing`) entfernen — **erst nach** S-50 (Lizenz-Ports); `validate_durability` bleibt. (4) `kv/` (56 Z., **nicht** in `lib.rs` eingebunden [V]) nach `archive/`. (5) `tenant_codec` → `ids::TenantId`; `Clock`/`MetricsSink`/`IdGen` → `env`. (6) `io_seam`/`fsync` unverändert (B08), `Vfs`-Bindung per Brücke. (7) ID-Breite im Manifest (§5.2) **zuletzt**. Kern-Closure: ids, env, sys, mvcc, crypto. INV-6 gilt strikt: WAL-, Flush- und Recovery-Code wird nur an Typ-/Importstellen berührt.
- **`kvcache`:** Brücke: `prefix_store.rs`, `adapters.rs` (inkl. Entfernen von `FeatureRing` aus dem Kern), `store.rs:790–862` (`KvCachePruner`/`KvCacheScanner`), `scratchpad_invalidation.rs`, Port-Teil von `service.rs`, `compat.rs`. Kern: `radix.rs`, `quantize_kivi.rs`, `attention_score.rs`, `eviction_worker.rs`, `segment.rs`, Rest von `store.rs`/`service.rs` auf `KvCacheError`, lokalen `TenantId`, `env::Clock`. `CacheDirective`/`StepId` einmal lokal (heute doppelt). Features: `default = ["std"]`; `kv-encryption` ohne `legacy-erasure`; `kivi-quantization`+`kvcache-kivi-quant` → `exp-kivi`; `content-addressed-kv-cache` → `exp-content-addressed`; `kvcache-attention-eviction` streichen. Aufnahme des KV-Teils aus `crypto` (S-38) als Feature `kv-shredding` (experimentell).
- **`infer-ollama`:** `OllamaError` lokal; `ImportanceScore` lokal (Skala 0..1 im Doc-Kommentar festhalten, bevor sie lokal wird) mit `From` in `compat/`; `trait_impls.rs`, `query_rewriter.rs`, `ContextraError`-Konvertierung, die eine `embedding.rs`-Referenz → `compat/`; `Rng`/`Clock` aus `env`. Vorbedingung: Prüfen, dass `--no-default-features` überhaupt kompiliert (ungegatetes `pub use error::…` neben `cfg(client)`). `workspace` ersetzt `types-compat`/`ports-compat`.
- **`infer-candle`:** `candle-core`, `candle-nn`, `candle-transformers`, `tokenizers`, `half` werden optional hinter Feature `candle`; Module `inference`, `gguf_loader`, `embedding`, `model` bekommen `cfg(feature = "candle")`; `fake.rs` → `test-utils`. Brücke: `embedding_provider.rs`, `attention_exporter.rs`, `kv_bridge.rs` (Feature `kv-bridge`, benötigt dann `kvcache`), `gasp.rs` (`GroundingValidator`). `rank`-Kopplung (`gasp.rs:24`, `IsotonicCalibrator`): lokales Trait `ConfidenceCalibrator`, Brücke füllt es mit `rank`. Eigener Newtype `ModelFingerprint` (B16). Features: `exp-kv-stage-b`, `exp-gasp`, `kv-bridge`. Vorbedingung: `rg 'contextra_infer_candle::' crates/` über alle Verbraucher, damit kein Symbol durch das Gaten verschwindet.
- **`vault` (neu):** Quelle `db/src/volatile_vault.rs` (592 Z.); `VaultError`; `contextra_types`-Bezug (2×) nur in `compat/`; baut auf `sys::vault`; `db` re-exportiert (`db/lib.rs:108`).

### 6.3 E2

- **`types`:** Bleibt: `error.rs` (`ContextraError`, 43 Varianten), `error_dto.rs`, `types/{budget,filter,importance,saos}.rs`, `types/domain/document.rs`, `types/domain/misc.rs` (abzüglich Auszüge), `schema.rs`, `RequestContext` aus `witness.rs`, `model_fingerprint.rs`, `retrieval_strategy.rs`. Geht: IDs/Refs/Tenant → `ids`; `tombstone.rs` → `mvcc`; `erasure.rs`, `SagaRecord`, `ScanMethod`, `ErasureId`, `GateId` → `erasure`; `PprConfig`/`PprAlgorithm`/`TlHfdParams` → `graph`. `lib.rs` Zeilen 20–28 (Verwenderliste) ersetzen durch generierten Verweis. Varianten `Sandbox*`, `Cluster`, `KvQuantization`, `KvDeleteModeConfig`, `HnswConnectivityDegraded`, `GraphRepairFailed` bleiben, **bis** alle lokalen Fehler existieren **und** G-E1 grün ist; erst dann entfernen (S-70), weil `#[non_exhaustive]` das Entfernen zu einer Verhaltensänderung für `Display`-Konsumenten macht.
- **`ports`:** Bleibt: `storage`, `vector_index`, `text_index`, `graph_index`, `graph`, `embedding`, `reranker`, `query_rewriter`, `checkpoint`, `kv`, `kv_bridge_port`, `attention`, `lifecycle`, `observability`, `plugin`, `vfs`, `fs`, `scanner/*`. Geht: `clock`/`rng`/`id_gen`/`metrics`/`env` → `env` (Re-Export bleibt); `erasure*`, `gate`, `ledger`, `observer`, `report_signer`, `keys`, `barrier` → `erasure` (Re-Exports in `lib.rs:16–22,156–206` entfernen); `license.rs`, `policy.rs` → `license`; `conformance.rs` (278), `contract_tests.rs` (632) → `testkit` nach Überschneidungsprüfung mit `testkit/conformance/` (1.612 Z.); `EgressPolicy` aus `license.rs` nach `privacy` **nur wenn** `privacy` sie ohnehin braucht (Prüfung per `rg`), sonst in `license`.
- **`router`:** `ports_local.rs` und die deprecated Re-Exports (`lib.rs:32–45`) entfernen, **nachdem** die drei `mcp`-Importe von `contextra_router::ports_local::{HybridSearchProvider, ContextPreparer, CommunityResolver}` auf `contextra_ports` umgestellt sind. `wire` optional hinter `subprocess-dispatch` (`dispatch.rs`, `transport.rs`); `ports::vfs`-Nutzung (eine Stelle) auf lokale `&Path`-Funktion; `RetrievalStrategy`-Abbildung nur in `arm_registry.rs`. Features: `exp-fc-ts`, `exp-routing-strategy`, `cloud-egress-guard`.
- **`engine`:** Herauslösen: `chunker/` → dünner Adapter über `ingest::chunking` (S-61); `temporal_filter.rs` → `rank`; `decay_controller.rs` → `adapt` (S-60); `extraction/` → `exp-extraction` (Vergleich mit `graph/entity_extraction.rs`: Namens-/API-Diff erheben, **keine** Zusammenlegung); `merge_operator_adapter.rs` bleibt. Lizenz: `contextra_impl/lifecycle.rs:9–55`, `lib.rs:17,22,373,410` → `contextra-license`. `VectorDeleteMode` aus `vector`. Leere Features (`reranking`) per Audit.
- **`db`:** `volatile_vault.rs` → `vault`; `pid_latency_controller.rs`, `homeostat.rs` → `pub use` aus `adapt` nach S-60; `multistep.rs` (442) bleibt. `experimental-diskann` zeigt auf `contextra-vector-diskann` als **echte optionale** Abhängigkeit (heute Dev-Abhängigkeit [V]). Neues Feature `cognition` (Brücke zu `cognition`, siehe unten).
- **`contextra`:** Lizenz-Gating (`builder.rs:33–35,243–350`, `performance_profile.rs`, `collection_profile.rs:33,128`) hinter Feature `license`; ohne Feature baut die Fassade ohne Lizenzprüfung (Verhalten **ohne** Feature = `OpenFastGate`-Semantik, per Test belegt). `agent_memory.rs` bleibt. `query_rewriter.rs` bleibt (B16).
- **`agent`:** Lokale Ports `ContextStore`, `ScratchpadStore`; Speicherbindung (`context.rs`, `clm_scratchpad.rs`, `tools/retrieve.rs`, `graph.rs`, Teile von `engine.rs`) → `compat/`; `router`-Kopplung hinter Feature `routing`. Kern-Closure: types, ports. Aus `db` genutzt: `Contextra::open_with_config`, `ContextraConfig`, `Collection`, `DistanceMetric`, `VolatileContextVault`, `VaultConfig`, `VaultChunk`, `SignalModality` [E] — diese bilden das Trait `ContextStore`.
- **`cognition`:** Lokale Ports `DocumentSource`, `ConsolidationSink` (inkl. `CommitIntent`-Ersatztyp, falls der Port-Zuschnitt scheitert), `HyperedgeSink` (`graph_sink.rs` als Ausgangspunkt), `DecayPolicy`; `CognitionError` statt `ContextraError::{Internal, PolicyViolation, InvalidInput}` (~30 Stellen); engine-gebundene Dateien (`consolidation_executor.rs`, `context.rs`, `semantic_aggregation_facade.rs`) → Brücke **im Crate `db`** (Feature `cognition`), damit `cognition` kein `compat/`-Zielcrate `engine` braucht. Features `graph-connectivity-health`, `edge-reinforcement-learning` leiten danach nur an `graph` weiter. Löschen der `*_local.rs`-Umbenennungsdateien (G7).

### 6.4 E3

- **`mcp`:** `prompt_injection/` → `prompt-guard` (Re-Export). `protocol.rs` bleibt (B16). `egress_*.rs` bleiben. `sandbox.rs` bleibt (`CryptoKey` im `crypto`-Kern).
- **`py`:** Feature `docid-128 = ["contextra-ids/docid-128"]`; sonst unverändert. `cargo check`/`maturin` mit und ohne `pyo3` getrennt.
- **`testkit`:** `conformance/{clock,rng,metrics}.rs` und `manual_clock.rs` → `env` (`test-utils`); `conformance/vfs.rs` und `fault_vfs.rs` **bleiben** (B08); `erasure-conformance` raus aus `default`, Suite → `erasure` (Feature `conformance`). `ports/conformance.rs`, `contract_tests.rs` hierher.

---

## 7. Umsetzungsreihenfolge

Jeder Schritt = Commit-Paar (INV-3) mit den genannten Nachweisen. Nachweis-Kürzel: **GL** = G-L1…G-L4, **GF** = Golden, **GD** = Differenztest, **GE** = Fehlerabbildung, **GC** = Concurrency (§8). „Voraussetzung“ nennt nur harte Abhängigkeiten.

| Schritt | Inhalt | Voraussetzung | Nachweis | Rollback |
|---|---|---|---|---|
| **S-00** | Toolchain `1.89.0`; Baseline: `cargo metadata`-Graph, `cargo tree` je Crate (`--no-default-features` und Default), `cargo check` je Crate, `rg`-Zählungen (§8.2) nach `docs/decoupling/baseline/`; exakte Zählung der `from_key`-Aufrufer | — | Baseline-Dateien eingecheckt; Differenz zu `80f1c33` dokumentiert | n. a. |
| **S-01** | Gate-Reparatur (B05): `tools/gate_layering.py` (§8.1), `tests/layering.rs` scheitert bei Verstoß, `xtask`-Verweise entfernen, `tests/arch` Mitglied oder gelöscht, `public-api` lauffähig, CI: `cargo check --workspace --all-targets --locked`, TSan/Loom/Mutation-Pfade (B11), Matrix `docid-128`. Geschützte Pfade ⇒ ADR + Label `protected-path` | S-00 | Gate scheitert absichtlich an einem Test-Verstoß (Negativtest), grün am Baseline-Stand | Revert |
| **S-02** | Golden-Korpus + Differenz-Harness (§5.1, §5.4) | S-00 | GF grün am unveränderten HEAD | Revert |
| **S-03** | Doku/Metadaten: `AGENTS.md` (Autonomie-/Drohtext entfernen, Compliance-Absatz an K6 anpassen), `capabilities.toml` Schema v3 (§8.3) mit Ziel-`allowed_internal`, falsche Docstrings (`privacy`, `cognition`, `kvcache`, `avv-generator`, `types/lib.rs`, `sys/lib.rs:19`, `vector/types.rs:3–6`), Item-Diff `AvvError` | S-01 | `capabilities-consistency` grün; G-DOC1 grün | Revert |
| **S-10** | `ids`: (A) Crate anlegen, Quellcode **kopieren** (mit `IdError`), `types` unverändert; Differenztest alt↔neu; (B) `types` → Re-Export + `From<IdError>`; (C) Aufrufstellen von `from_key` & Co. anpassen | S-02 | GF, GD, GE, GL | Revert C, B, A |
| **S-11** | `env`: (A) Crate anlegen, Dateien aus `ports` kopieren; (B) `ports` → Re-Export; Konformitätssuite in `env` | S-02 | GF (Rng-Folgen), GD, GL | wie S-10 |
| **S-12** | Feature-Hygiene (B14): `tools/feature_audit.py`, Defaults gemäß §3.1 für E0-/Test-Crates (`adapt`, `license`, `audit-export`, `testkit`, `crypto`, `graph`, `kvcache`, `rank`, `text`, `vector`), leere Features entfernen, `erasure` Features; Gate `error-domain` aktivieren | S-01 | `feature_audit` grün; `cargo hack check --each-feature -p <crate>` je Crate; `--workspace` grün | je Crate Revert |
| **S-20** | `license`, `audit-export`: Brücken → `compat/` | S-12 | GL | Revert |
| **S-21** | `prompt-guard`: (A) verschieben, (B) `file-audit`-Feature | S-01 | GL, Test `prompt_injection` unverändert grün | Revert |
| **S-22** | `vault` aus `db`, baut auf `sys::vault` | S-01 | GL; `rg 'volatile_vault' crates --glob '!crates/contextra-vault/**'` nur Re-Exports | Revert |
| **S-23** | `adapt`: Dubletten-Traits vereinigen (R7), `off_policy.rs`, Features, `sketched-bandit` | S-12 | GL; `rg 'trait (Clock\|Rng)' src` je 1 | Revert |
| **S-24** | `erasure` Features + Typabgleich (die Typen-Übernahme aus `types`/`ports` folgt in S-50/S-51) | S-12 | `cargo check --no-default-features` + je Feature | Revert |
| **S-30** | `mvcc`: Loom-Modell (B11) → (A) `tombstone.rs` verschieben → (B) auf `ids`/`env`/`MvccError`; Brücke | S-10, S-11 | GL, GF, GC, GD (Tombstone-Semantik), GE | Revert B, A |
| **S-31** | `crypto` Kern: `TenantId`→`ids`; KV-Signaturen auf `[u8;32]` (eigener Commit); `default = []`; `MvccError`/`CryptoError`-Abbildung | S-10, S-11, S-12 | GL, GF (WAL-HMAC, MFKD, KV v2), GC, GE | Revert |
| **S-32** | `text` (B04): `PostingStore`, Brücke (`wand.rs`, `scanner.rs`, `index_struct.rs`, `purge.rs`, `morph_index.rs`), `TextError`, Clock→`unix_ms` | S-10, S-11 | GL, GF (Postings), GD (ms), GE | Revert |
| **S-33** | `rank` (+ `temporal.rs` aus `engine` als eigener Commit; Quelle verbleibt als `pub use`) | S-10, S-11 | GL, GD (Filterausgaben) | Revert |
| **S-34** | `graph`: Brücke, `GraphError`, `default = ["std"]`, `PprConfig` & Co. aus `types` (zusammen mit S-51-Teil) | S-10, S-11, S-12 | GL, GF (bincode), GC (TSan), GE | Revert |
| **S-35** | `checkpoint`: `SnapshotSource`, Dateinaht, `CheckpointError` | S-10, S-11 | GL, GF (Meta/Manifest) | Revert |
| **S-36** | `vector`: Kern auf `ids`/`env`/`mvcc`/`VectorError`; `contract.rs` vereinen; `VectorDeleteMode`; Feature `simd`; **ohne** DiskANN-Umzug | S-30, S-10, S-11 | GL, GF (HNSW), GD (Distanz, Rng, ms), GC, GE | Revert |
| **S-37** | `store`: `StoreError`, `compat/` (`engine/`, `scanner/`), Imports → `ids`/`env`; `kv/` archivieren; `FeatureRing`-Entfernung **nach** S-50; zuletzt ID-Breite im Manifest (v2, §5.2) | S-30, S-31, S-10, S-11 | GL, GF, GC (Loom/TSan), Crash-/Recovery-Suite unverändert grün (INV-6), GE | Revert pro Teil |
| **S-38** | **Paar:** `crypto`-KV-Teil → `kvcache` (`kv_cipher`, `kv_segment/*`, `kv_shredding`, `revocation_log`; Konstanten-Kopplung bis Vereinigung); `kvcache` Kern/Brücke/Features; `infer-candle/kv_bridge` auf `kvcache`; `mutants-guarantees.toml`, `loom-gate.yml`, `CODEOWNERS` im selben Commit | S-31, S-11 | GL, GF (KV-Segment), GC (Loom-Test `loom_kv_deferred_zeroize` am neuen Ort grün), Mutation-Score G2 ≥ Baseline | Revert als Paar |
| **S-39** | `infer-ollama`, danach `infer-candle` (Feature `candle`, `ConfidenceCalibrator`, Newtype) | S-11, S-38 | GL; `cargo tree --no-default-features` ohne `candle-*` | Revert |
| **S-50** | `ports` → `license` (Lizenz-Traits), `erasure` (Erasure-Familie), `testkit` (Conformance); Re-Exports bleiben (R9) | S-20, S-24 | GL; `rg 'FeatureRing\|LicenseGate' crates/contextra-{store,vector,text,graph,kvcache,rank,checkpoint,mvcc}` leer | Revert |
| **S-51** | `types` → `erasure` (Erasure-Typen), `graph` (Ppr/TlHfd); `lib.rs`-Verwenderliste | S-50, S-34 | GL, GE | Revert |
| **S-60** | Regler: `adapt` nimmt die fehlenden Methoden auf (decay 5, homeostat 3, pid 5) → GD (bitgleich) → `engine/db` `pub use` | S-23 | GD | Revert |
| **S-61** | Chunker: Paritätstest in `engine` (`ingest` als Dev-Abhängigkeit) → `engine/chunker` als Adapter über `ingest::chunking` (`contextra::chunker::MarkdownChunker` API-stabil, `mcp/server_tools.rs:240`) | S-01 | GD (Chunk-Folge, Breadcrumbs); `mcp`-Tests unverändert | Revert |
| **S-62** | `router`: `ports_local` weg (nach `mcp`-Umstellung), `wire` optional, `Vfs`-Stelle | S-01 | GL | Revert |
| **S-63** | `cognition` invertieren (Brücke in `db`), danach `agent` invertieren | S-37, S-50 | GL (`rg 'contextra_engine' src --glob '!compat/**'` leer; `cargo tree -p contextra-cognition -e normal` ohne `contextra-engine`) | Revert |
| **S-64** | `engine` (Lizenz → `license`, `extraction` → `exp-extraction`), `db`, `contextra` (Feature `license`), `mcp`, `py`, `testkit` | S-50, S-51, S-33 | GL; `cargo public-api` ohne unbeabsichtigte Diffs | Revert |
| **S-70** | Aufräumen: deprecated Re-Exports (R9) entfernen, Fehlervarianten in `ContextraError` entfernen (nur wenn G-E1 grün), `archive/`-Verschiebungen laut B10 (nur bei erfülltem Nachweis) | alle | alle Gates | Revert |
| **S-80** | `contextra-vector-diskann` (aus `vector/diskann/`) | S-36 | GL, DiskANN-Tests (`required-features`) isoliert grün | Revert |
| **S-81** | `contextra-graph-research` (aus Klasse A/B-Experimenten) | S-34 | GL, GD | Revert |

Reihenfolge-Zwang (Zyklenfreiheit [A]): `ids`,`env` vor allem E1; `mvcc` vor `vector`/`store`; `crypto`-Kern vor `store`; `crypto`-KV-Umzug nach `crypto`-Kern und gemeinsam mit `kvcache`; Lizenz-/Erasure-Ports (S-50) vor dem Entfernen von `FeatureRing` aus `store`/`kvcache`; `cognition` nach `engine`-Lizenzumstellung nicht vorher.

---

## 8. Gates, CI, Governance

### 8.1 G-L1 Layering (Ersatz für `gate_g1_layering.py`)

`tools/gate_layering.py`, Exit-Code 0 = grün, 1 = Verstoß, 2 = Werkzeugfehler (kein „Bericht nicht leer“-Test).
1. `cargo metadata --format-version 1 --locked --no-deps` und `cargo tree -p <crate> -e normal --no-default-features -f '{p}'` je Crate.
2. Kern-Closure ⊆ `allowed_internal` (capabilities v3). Abweichung ⇒ Verstoß.
3. Je Feature `f`: `cargo tree … --no-default-features --features f` ⊆ `feature_closure_allowed[f]`.
4. Kein Zyklus; keine Kante E_n → E_m mit m > n (INV-8); keine Kante außerhalb `dev_allowed` auf `testkit`.
5. Negativtest `tests/test_gate_layering_negative.py`: temporär eine verbotene Kante einfügen ⇒ Exit 1.

### 8.2 G-L2 Null-Prüfungen (je Crate X mit Brücke)

```bash
# Kern frei von types/ports:
rg -n 'contextra_(types|ports)' crates/contextra-X/src --glob '!compat/**' --glob '!tests/**'   # MUSS leer sein
# Kern frei von testkit:
rg -n 'contextra_testkit' crates/contextra-X/src --glob '!compat/**'                           # MUSS leer sein
# Brücke nur hinter Feature:
rg -n 'mod compat' crates/contextra-X/src/lib.rs                                                # MUSS cfg(feature = "workspace") tragen
# Doc-Kommentare und #[cfg(test)] im Kern zählen ebenfalls (R2): kein Ausschluss von //-Zeilen.
```
Zusätzlich je Crate: `cargo check -p X --no-default-features` und `cargo hack check -p X --each-feature --no-dev-deps`.

### 8.3 `capabilities.toml` Schema v3

```toml
[version]
schema = "3"

[crates.contextra-vector]
group   = "retrieval"          # ersetzt klasse/ring
level   = 1
tier    = "beta"               # stable | beta | experimental | frozen
path    = "crates/contextra-vector"
unsafe_island = false
allowed_internal = ["contextra-ids", "contextra-env", "contextra-mvcc"]   # Kern-Closure
[crates.contextra-vector.feature_closure_allowed]
mmap = ["contextra-sys"]
simd = ["contextra-simd"]
workspace = ["contextra-types", "contextra-ports"]
dev_allowed = ["contextra-testkit"]
test = "cargo test -p contextra-vector --locked"
```
Tier-Zuordnung (ersetzt K4 des Entwurfs): `durable-fs`, `simd` (bis zum Verwender), `graph`, `vector`-Experimente → `experimental`/`beta`; `erasure`, `license`, `audit-export`, `avv-generator` → `frozen`; Rest nach bisherigem Stand. Das Gate `capabilities-consistency` vergleicht Manifest ↔ Datei.

### 8.4 Weitere Gates

| Gate | Inhalt |
|---|---|
| **G-L3** | `cargo check --workspace --all-targets --locked` (PR) und `--features docid-128` (Matrix); `cargo hack` je Crate |
| **G-L4** | `tools/isolation_proof.py`: Crate + vollständige interne Pfad-Closure in ein leeres temporäres Workspace kopieren, **keine** Abhängigkeit ausklammern, neues `Cargo.lock`, `cargo check` mit `--no-default-features`, Default und je Feature; Fehlschlag, wenn eine interne Abhängigkeit außerhalb von `allowed_internal` mitkopiert werden muss. Ersetzt die Aussagekraft von `export_crate.py` (klammert interne Abhängigkeiten aus [E]) |
| **G-F1** | Golden-Korpus (§5.1) |
| **G-F2** | `contextra.fbs.sha256` stimmt mit `contextra.fbs` (B15) |
| **G-E1** | Fehlerabbildung (§5.3) |
| **G-D1** | Differenztests (§5.4) |
| **G-C1** | Loom/TSan/Mutation: Pfadfilter und Crate-Listen um `mvcc`, `kvcache`, `checkpoint`, `ids`, `env` erweitert; Mutation-Score der Garantien G1/G2/G3 (`mutants-guarantees.toml`) ≥ Baseline aus S-00 |
| **G-A1** | `cargo public-api` auf der gepinnten, zu `rust-version` passenden Toolchain; erlaubte Differenzen = Re-Export-Pfade mit `#[deprecated]` |
| **G-U1** | `unsafe-inventory`: `ids`, `env`, `prompt-guard`, `vault` (außer über `sys`) `#![forbid(unsafe_code)]`; `ratchet.toml` angepasst (ADR) |
| **G-DOC1** | Doc-Lint: verbotene Zusicherungen („DSGVO-konform“, „revisionssicher“, „garantiert gelöscht“) in `*.rs`, `*.md`, Vorlagen von `avv-generator`/`audit-export`; Docs nennen keine Features, die es nicht mehr gibt |

### 8.5 Abnahmematrix je Crate (Kurzform)

Ein Crate gilt als **entnehmbar** (R6), wenn in dieser Reihenfolge grün: G-L2 (Null-Prüfungen), G-L1 (Kern- und Feature-Closure), G-L3 (`--no-default-features`, `--each-feature`), G-L4 (isolierter Export), danach G-F1/G-D1/G-E1/G-C1 für den betroffenen Schritt. „Entnehmbar“ ohne G-L4 ist nicht zulässig.

---

## 9. Entscheidungsregister, Risiken, Nicht-Ziele

### 9.1 Getroffene Entscheidungen (früher „du entscheidest“)

| ID | Entscheidung |
|---|---|
| D-01 | Flaches Layout; Gruppe nur als Metadatum (B07) |
| D-02 | `Vfs` bleibt in `ports`; `durable-fs` eigenständig (B08) |
| D-03 | `ModelFingerprint`/`RetrievalStrategy` bleiben in `types` (B16) |
| D-04 | `DistanceMetric`: gespiegelte Kopien statt Besitzerwechsel (B09) |
| D-05 | `sketched-bandit` wird gestrichen; Features ohne Nutzung werden gestrichen (B14, B16) |
| D-06 | `legacy-erasure`-Dateien in `crypto` bleiben bis Nachweis (B10) |
| D-07 | `vault` baut auf `sys::vault`; `mcp::protocol` und `wire::jsonrpc` bleiben getrennt; `adapt::drift`/`rank::drift` und die beiden `query_rewriter` bleiben getrennt (B16) |
| D-08 | Compliance-Crates bleiben kompilierbar in der Workspace-CI (nicht `archive/`), Tier `frozen` (K6 des Entwurfs bestätigt; `AGENTS.md` wird angepasst) |
| D-09 | Verteilung per Git/Pfad, nicht crates.io (B15) |
| D-10 | `wire/build.rs` unverändert; Prüfsumme in CI (B15) |
| D-11 | Ebene-0-Crates behalten lokale Mini-Traits (R7/K8); E1+ nutzen `ids`/`env` |
| D-12 | Mikro-Crate-Zerlegung (Schwester-Specs) ist Folgeprojekt F-1 |

### 9.2 Restrisiken mit Abbruchkriterium

| Risiko | Auslöser | Abbruchkriterium / Gegenmaßnahme |
|---|---|---|
| Ungebaut: Kompilierbarkeit mit `--no-default-features` ist für kein Crate belegt | S-00 | Schlägt der Build am Baseline-Stand fehl, wird zuerst dieser Zustand repariert (eigener Commit je Crate); erst danach beginnt der jeweilige Schritt |
| `ids`-Umzug ändert Rückgabetypen (B12) | S-10 C | Mehr als ~20 nicht-mechanische Aufrufstellen ⇒ Rückfall: `types` behält `DocId::from_key` als freie Funktion `types::doc_id_from_key` plus Methode via Extension-Trait (Trait-Import nötig) — nur mit ADR |
| Feature-Unification von `docid-128` | CI-Matrix | Jede Kombination, die nur mit dem Feature oder nur ohne baut, ist ein Fehler |
| Format-/Recovery-Drift im `store` | S-37 | Crash-/Recovery-Suite oder GF rot ⇒ Schritt verwerfen, nicht reparieren |
| Mutation-Score G2/G3 sinkt | S-38/S-70 | Schritt verwerfen oder Tests nachziehen, bis ≥ Baseline |
| `cognition`-Port-Zuschnitt (`CommitIntent` aus `engine::transaction`, 1.652 Z. [E]) | S-63 | Port scheitert ⇒ `CommitIntent` als eigener Typ in `cognition`, Abbildung in `db`-Brücke |
| `infer-ollama` baut nicht ohne Default | S-39 | Zuerst Gating reparieren (eigener Commit) |
| Deprecated-Re-Exports sammeln sich | R9 | S-70 ist Pflichtschritt; `deprecated`-Zähler fällt monoton (Ratchet) |

### 9.3 Nicht-Ziele (ausdrücklich nicht Teil dieser Spezifikation)

Physische Regruppierung der Verzeichnisse; Mikro-Crate-Zerlegung von `store`/`vector`/`graph` (F-1); Verschieben von `Vfs`, `io_seam`, `store/fsync.rs`; Veröffentlichung auf crates.io; Änderung von Persistenzformaten (außer dem ID-Breitenfeld, §5.2); Änderung von FlatBuffers-Schemas oder `wire/build.rs`; Zusammenlegung der in B16 als „keine Dublette“ bewerteten Module; Brücke `privacy ↔ audit-export`; Änderung von Rangfolge-, Fusions- oder Durability-Semantik.

---

## Anhang A — Messdaten (Quelle: Manifest-Scan, [V])

**Eingehende interne Normal-Abhängigkeiten.** `types`: 24 Crates (mvcc, mcp, license*, testkit, infer-ollama*, checkpoint, contextra, router, text*, ports, py, db, engine, audit-export*, graph*, cognition, rank*, crypto, infer-candle, kvcache*, agent, vector*, store, simd*; * = optional). `ports`: 20 Crates (6 optional). **Ohne Verwender** (nur Manifest-Scan): `durable-fs`, `simd`, `ingest`, `kvcache`, `erasure`, `license`, `audit-export`, `avv-generator`, `agent`, `cognition`, `py`, `mcp`, `bench`, `testkit` (nur Dev-Kanten).

**Hart abhängig von `types`+`ports`** (nach Ziel zu ersetzen): `mvcc`, `checkpoint`, `crypto`, `store`, `router`, `testkit`, `infer-candle`; nur `types`: `py`.

**`vector`** hat `mvcc` und `sys` optional, `types`/`ports` optional (Default `ports-compat`) — die ungegateten Importe im Kern (B04) machen die Optionalität wirkungslos.

**Default-Mitglieder (`cargo check` ohne `--workspace`)** enthalten nicht: `ingest`, `privacy`, `sandbox`, `infer-candle`, `py`, `bench`, `erasure`, `license`, `audit-export`, `avv-generator`.

## Anhang B — Reproduktion der Prüfungen

```bash
# S-00 Baseline
rustup toolchain install 1.89.0 && cargo metadata --format-version 1 --locked > baseline/metadata.json
for c in $(cargo metadata --no-deps --format-version 1 | jq -r '.packages[].name'); do
  cargo tree -p "$c" -e normal --no-default-features -f '{p}' > "baseline/tree-$c.txt" 2>&1
  cargo check -p "$c" --no-default-features > "baseline/check-$c.txt" 2>&1
done
# R2-Nullprüfung (Beispiel vector)
rg -n 'contextra_(types|ports|mvcc|sys|crypto)' crates/contextra-vector/src --glob '!compat/**' --glob '!tests/**'
# Zählung wie in B04 (Python-Skript tools/count_bridge_refs.py in S-00 einchecken)
# Aufrufer von from_key (exakt, ohne Doppelzählung)
rg -n --type rust '::from_key\(' crates | rg -v '/tests/'
# Feature-Audit
python3 tools/feature_audit.py --fail-on-unused
# Isolationsnachweis
python3 tools/isolation_proof.py --crate contextra-text --modes no-default,default,each-feature
```
