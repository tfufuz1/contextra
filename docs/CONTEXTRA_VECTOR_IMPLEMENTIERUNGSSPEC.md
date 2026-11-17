# contextra-vector — Implementierungsspezifikation (SOLL-Zustand nach `docs/SPEC.md` v6)

Stand: 17. Nov 2026 · Autor: Principal Rust Engineer (Review) · Gegenstand: `crates/contextra-vector`
Geprüfter Stand: Repository `tfufuz1/contextra`, HEAD `24487fa16` (16.11.2026). Die SPEC v6 beschreibt `fcd387021` (11.11.2026).
Toolchain der Messungen: `rustc 1.89.0` / `cargo 1.89.0` (Ubuntu-Paket `rustc-1.89`), `clippy 1.89.0`.

---

## 0 Lesehilfe

### 0.1 Belegmarken

| Marke | Bedeutung |
| --- | --- |
| **[V]** | In dieser Analyse ausgeführt und gemessen (Build, Test, Clippy, Skript, Probe-Programm). |
| **[L]** | Am Quelltext gelesen oder per `grep` belegt, nicht ausgeführt. |
| **[E]** | Einschätzung, nicht belegt. |
| **[D]** | Festlegung dieser Spezifikation (Vorschlag, bis ein ADR sie bestätigt). |
| **[S]** | Aus `docs/SPEC.md` v6 übernommen. |

### 0.2 Normative Sprache und Rangfolge

MUSS, DARF NICHT, SOLL nach RFC 2119. Diese Spezifikation konkretisiert `docs/SPEC.md` v6 für ein Crate. Sie ändert keine Invariante, keine Regel und keine Entscheidung der SPEC. Wo der Code der SPEC widerspricht, steht das in Kapitel 4. Wo die SPEC selbst eine Lücke hat, steht das in Kapitel 9 als offene Entscheidung. Rangfolge nach SPEC §1.2: SPEC, dann diese Spezifikation, dann crate-lokale Dokumente (`SPEC.md`, `PLAN.md`, `README.md`).

### 0.3 Was nicht geprüft wurde

Damit niemand mehr aus diesem Dokument liest, als darin steht:

- Nicht gelesen oder nicht bewertet: `hnsw/core_rebuild/*`, `hnsw/core_insert/*`, `hnsw/arena.rs`, `acorn/*`, `quantize/*` (außer Lint-Zählung), `distance/*` (außer Dispatch-Kopf), `compute_pool.rs`, `compat/purge.rs`, `compat/scanner.rs`, `compat/candidate_stream.rs`, `fuzz/`, `proptest-regressions/`, Inhalt der Benchmarks.
- Nicht ausgeführt: die 7 Test-Targets mit `required-features` auf `experimental-diskann` oder `simd`, alle Benchmarks, Miri, Loom, TSan, Mutation-Tests, `cargo public-api`, `cargo hack`.
- Nicht verfügbar im Klon: `governance/`, `.github/`, `justfile`, `.githooks/`, `docs/decisions/`, `docs/decoupling/` im Wurzelverzeichnis [V: `ls`, `git ls-files`]. Die Gates G-L1, G-H1, G-D1 und G-E1 konnten daher nicht in der kanonischen Form laufen (siehe 7.3).

---

## 1 Ziel, Geltung, Nicht-Ziele

### 1.1 Ziel

`contextra-vector` ist ein **entnehmbarer E1b-Baustein** (SPEC §2.2): ein Vektorindex (HNSW, SQ8, ACORN-Filter) mit Kern-Closure `ids`, `env`, `mvcc`. Er MUSS

1. ohne `contextra-types`, `contextra-ports` und `contextra-testkit` vollständig benutzbar sein (Einfügen, Suchen, Löschen, Commit, Rollback, Persistieren, Öffnen),
2. alle Gates G-L2, G-L1, G-L3, G-L4, G-H1 in dieser Reihenfolge bestehen (SPEC §2.1, §8.4),
3. ein Persistenzformat mit Breitenfeld `id_width_bits` führen (SPEC §6.2),
4. die Invarianten INV-1 bis INV-12 und INV-VEC-* einhalten,
5. den Bausteinvertrag aus SPEC §5.4 vollständig erfüllen.

### 1.2 Nicht-Ziele

- Keine neue Funktionalität im Sinne von `AGENTS.md` Regel 5 (SPEC D-09): Herauslösen, Umbenennen, Zusammenführen und Reparieren sind erlaubt.
- Kein `VisibilityLog` im Crate (SPEC §9.4: `mvcc` bleibt Kernabhängigkeit).
- Keine In-Place-Migration zwischen 64 und 128 Bit (D-16).
- Keine Aktivierung von `partial-index-rebuild` im Default (VETO-F02).
- Kein `no_std` (D-19).

---

## 2 SOLL-Zustand (aus SPEC v6) [S]

| Merkmal | SOLL | Quelle |
| --- | --- | --- |
| Ebene | E1b | §2.2, §4.1 |
| Kern direkt → Closure | `ids`, `env`, `mvcc` → 3 | §4.1 |
| Features | `mmap`→`sys`; `simd`→`simd`; `exp-*`; `workspace`; `docid-128` (nur weiterleiten) | §4.1, R10 |
| Default | `std` | §4.1 |
| Fehlertyp | `VectorError` (`thiserror`, `#[non_exhaustive]`), `From<VectorError> for ContextraError` nur in `compat/error.rs` | R3, §5.3c |
| Brücke | Alles mit `contextra_types`/`contextra_ports`, die Port-Implementierung `VectorIndex for HnswIndex`, Purger, Scanner nur in `compat/` hinter `workspace` | R2, §5.3d, §9.4 |
| Dev-Kanten | keine auf `types`, `ports`, `testkit` | R12 |
| Persistenz | HNSW `HNSW_VERSION` 2 → 3 mit `id_width_bits: u16`; Rejects `IdWidthMismatch`, `IdWidthUnknown`; Golden-Korpus | §6.1, §6.2, S-36 |
| `contract.rs` | `now_unix_ms`, `Clock`, `Rng` vereinen (auf `env`) | S-36 |
| `simd` | `compute_distance` ersetzt durch `contextra_simd::dispatch`, Skalarpfad bleibt Orakel; Orakel-Differenztest | §9.4, S-36 |
| V-VETO | `PartialRebuildConfig` bleibt aus; Wachtest existiert und ist grün | §9.4 |
| DiskANN | Herauslösen nach `contextra-vector-diskann` | D-09, S-80 |
| Bausteinvertrag | README (Beispiel ≤ 10 Zeilen, ohne weitere `contextra-*`), `examples/`, `capabilities.fragment.toml`, crate-`SPEC.md`, `REPORT.md`, Golden-Korpus | §5.4, §8.4 |
| Invarianten | INV-12 (kein `unwrap`/`expect`/`panic!`/`todo!`), INV-11 (`forbid(unsafe_code)`), INV-5 (Determinismus) | §3 |
| Crate-Invarianten | INV-VEC-ZEROIZE-1, INV-VEC-CONNECTIVITY-1, INV-HNSW-SLOT-1, INV-TENANT-IDX-1, INV-INDEX-APPLY-1, INV-INDEX-SNAPSHOT-1 | crate-`SPEC.md` §7 |

Beitrag des Crates zu den Kapiteln 11 bis 14 (Klasse D, Ableitungen): siehe Kapitel 8.

---

## 3 IST-Zustand (gemessen)

### 3.1 Größe und Aufbau [V/L]

- `src/` ≈ 17.700 Zeilen (Kern HNSW-Verzeichnis ≈ 6.000, `diskann/` 4.158 einschließlich Tests, `compat/` 1.913, `persistence/` 972, `quantize/` 1.101, `distance/` 1.009, `acorn/` 209). Die Zahl in `AGENTS.md` (15.814 LOC) ist veraltet.
- `#![forbid(unsafe_code)]` gilt crateweit (`lib.rs`); `UNSAFE_TRANSITION.md` meldet alle Mmap-Stellen als migriert nach `contextra-sys`.
- Verbraucher: `contextra-db` und `contextra-engine` (Feature `workspace`), `contextra-agent` (nur Tests), `contextra-bench`. Genutzt werden `HnswIndex`, `HnswConfig(Builder)`, `DistanceMetric`, `ScalarQuantizer`, `MmapIndex`, `HnswHeader`, `VectorError`, `DiskAnnConfig`/`DiskAnnIndex`, `acorn::*`, `distance::*`, `ComputePool`, `MonotonicCounterClock`, `GhostFreeVectorIndex`, `RebuildStatus` [L].

### 3.2 Abhängigkeiten [V]

| Messung | Ergebnis |
| --- | --- |
| `cargo tree -p contextra-vector -e normal --no-default-features` (intern) | `ids`, `env`, `mvcc` (= SOLL, Closure 3) |
| mit Default-Features | identisch (Default ist `std`) |
| mit `--features workspace` | zusätzlich `ports`, `types`, `sys` |
| `tools/gate_bridge_null.py` (G-L2) | bestanden, 0 ungegatete Referenzen |
| strenge `grep`-Zählung außerhalb `compat/` | 0 |
| interne dev-Abhängigkeiten | keine (R12 erfüllt) |

Die in der SPEC genannten F-07 (Closure 5 statt 3) und F-15 (14 ungegatete Referenzen, Dubletten neben `compat/`) sind für `vector` am HEAD erledigt.

### 3.3 Build-Matrix [V]

`cargo check -p contextra-vector --locked --lib`:

| Konfiguration | Ergebnis |
| --- | --- |
| `--no-default-features` | grün (11 Warnungen) |
| default | grün (11) |
| `workspace` | grün (17) |
| `ports-compat` | grün (17) |
| `simd` | grün (11) |
| `mmap` | grün (10) |
| `docid-128` | grün (11) |
| `experimental-rabitq` | grün (11) |
| `experimental-predicate-augmented-search` | grün (11) |
| `partial-index-rebuild` | grün (11) |
| **`experimental-diskann`** | **rot** (Syntaxfehler `diskann/persistence/loader.rs:134–139`) |
| `workspace,simd,mmap,experimental-all,partial-index-rebuild,docid-128` | **rot** (gleiche Ursache) |

Nach einem lokalen Testpatch der fehlenden Klammer bleiben 21 Folgefehler im DiskANN-Modul (Anhang B). Der Patch wurde zurückgenommen; der Klon ist bis auf die drei wiederhergestellten Wurzeldateien unverändert.

### 3.4 Tests und Benchmarks [V]

| Messung | Ergebnis |
| --- | --- |
| `cargo test --lib --features workspace` | 56 bestanden, 0 fehlgeschlagen |
| Integrationstests mit `workspace`, die kompilieren | **58 von 58 bestanden** |
| `cargo check --all-targets --keep-going`, **default** | **64 Targets scheitern, 672 Fehler** (davon 447 × E0599 „no method `insert`/`commit`/`search`“, 112 × E0432 unaufgelöster Import) |
| `--all-targets --keep-going`, `workspace` | **22 Targets scheitern (21 Tests, 1 Bench), 87 Fehler** |
| `--all-targets --keep-going`, `workspace,simd,partial-index-rebuild,docid-128` | 28 Targets scheitern, 144 Fehler, **einschließlich `lib test`** |
| Golden-Korpus `tests/golden/MANIFEST.sha256` | 9 von 9 Hashes stimmen |
| Beispiel `examples/standalone_vector.rs` (default) | kompiliert nicht (7 Fehler) |

Die 21 Test-Targets, die auch mit `workspace` nicht kompilieren: `acorn_hnsw_differential`, `acorn_selectivity_edge_cases_test`, `adaptive_ef_recall`, `campaign_j24_test`, `deleted_nodes_lock_contention`, `deletion_zeroize_test`, `filtered_selectivity_recall`, `hnsw_rebuild_connectivity_test`, `hnsw_scanner_purge_test`, `nan_validation_policy`, `proof_determinism_seed`, `proof_recall_oracle`, `ram_reduction`, `recalibration`, `recall`, `recall_vs_brute_force`, `reconstruction_attack_test`, `sq8_bias_calibration`, `sq8_rerank_recall`, `wp05_insert_search_contract`, `wp10_quality_gaps`; dazu der Bench `overfetch_selectivity`.

Ursachen mit `workspace` (87 Fehler): 34 × E0308 „mismatched types“, 17 × E0599 (Methode fehlt), 7 × E0061 (Methode nimmt 3 statt 2 Argumente, z. B. `wp05_insert_search_contract`), 2 × `include_str!` auf nicht mehr vorhandene Dateien (`src/distance.rs`, `src/hnsw/core_search.rs` sind seit dem Verzeichnis-Split keine Dateien mehr), außerdem fehlende Trait-Mitglieder und eine nicht erschöpfende `match`. Das sind Test-Drift gegen die API, nicht Fehler der Bibliothek.

Der V-VETO-Wachtest `partial_rebuild_veto_f02_guard` scheitert mit `partial-index-rebuild` (3 Fehler) und mit den Default-Features (8 Fehler). Die SPEC verlangt, ihn „vor jeder Änderung grün zu bestätigen“. Das ist derzeit nicht möglich.

### 3.5 Lints (INV-12) [V]

Das Crate setzt `#![deny(clippy::indexing_slicing, unwrap_used, expect_used, panic, cast_possible_truncation, cast_sign_loss)]`. `cargo clippy -p contextra-vector --lib` liefert:

| Konfiguration | Fehler | Warnungen |
| --- | --- | --- |
| `--no-default-features` | 204 | 18 |
| `--features workspace` | 221 | 24 |

Aufschlüsselung `workspace`: 104 × „indexing may panic“, 19 × „slicing may panic“, 44 + 21 Casts mit möglicher Kürzung auf 32/64-Bit-Zeigern, 21 weitere Kürzungen, 11 × Vorzeichenverlust. Schwerpunkte: `hnsw/arena.rs` (34), `hnsw/types/persist.rs` (22), `quantize/train.rs` (21), `compat/hnsw_vector_index.rs` (21), `quantize/distance.rs` (14), `hnsw/deletion.rs` (14), `distance/scalar.rs` (13), `distance/inject.rs` (13). Die Aussage „Zero-Panic erfüllt“ in `README.md`, `SPEC.md` und `PLAN.md` ist damit nicht belegt: Die `deny`-Lints wirken nur im Clippy-Lauf, `cargo check` ist grün, `cargo clippy` ist rot.

### 3.6 Isolation (G-L4) [V]

`tools/isolation_proof.py --crate contextra-vector`: Modi `no-default` und `default` laufen durch, Modus `each-feature` (19 Features) bricht am Feature `experimental` ab (Syntaxfehler aus 3.3). Entnehmbar im Sinne SPEC §2.1 ist das Crate damit nicht.

---

## 4 Befundregister

IDs `VEC-F-xx` sind neu und stabil. „Schritt“ verweist auf Kapitel 6.

| ID | Befund | Beleg | Schweregrad | Schritt |
| --- | --- | --- | --- | --- |
| VEC-F-01 | Der Kern ist ohne Brücke kein benutzbarer Index: `insert`, `search`, `commit`, `delete`, `rollback`, `rollback_to_tx`, `search_at`, `all_doc_ids`, `len`, `stats` liegen ausschließlich in `compat/hnsw_vector_index.rs` (828 Zeilen) hinter `workspace` und liefern `ContextraError`. | [V] Build der Tests (447 × E0599); [L] Datei | hoch | S-36f |
| VEC-F-02 | Die Suchlogik existiert dreifach (`VectorIndex::search`, `Index::search` in `compat/`, `search_filtered_internal` im Kern) und weicht ab: Quantizer-Pfad, Fallback der Metrik (`_ => Err` bzw. `_ => 1/(1+d)`), Behandlung von `k = 0`, Nebenwirkung `set_entry_point` im Lesepfad. | [L] | hoch | S-36f |
| VEC-F-03 | **`search(q, 0)` panict** („attempt to subtract with overflow“ bei `select_nth_unstable_by(k - 1)`) sobald das Ergebnis nicht leer ist. Die Variante `Index::search` fängt `limit == 0` ab, die Port-Variante nicht. | [V] Probe-Programm, `compat/hnsw_vector_index.rs:205` (Debug-Build) | hoch (INV-12) | S-36f |
| VEC-F-04 | **Operationsreihenfolge in einer Transaktion geht verloren:** `insert(1)`, `insert(2)`, `delete(1)` in derselben Tx, dann `commit`: Die Suche liefert danach `[1, 2]`. Löschungen werden vor Einfügungen angewendet. | [V] Probe-Programm | hoch (Korrektheit, relevant für Z-E1) | S-36f |
| VEC-F-05 | `commit()` ist bei einem Fehler nicht atomar: Nach `tx_buffer.drain(tx)` gehen die Operationen verloren; Quantizer-Training und Umwandlung der Knoten in `U8` sind bereits angewendet. | [L] | mittel | S-36f |
| VEC-F-06 | `Index::insert`/`remove` (`compat`) umgehen die Tx-Staging-Logik, vergeben `last_tx_id + 1` selbst und kürzen `ExternalId(u128)` per `as u64` still auf 64 Bit. | [L] | mittel | S-36g |
| VEC-F-07 | Trait-Wildwuchs: `contract::Index` und `contract::Maintain` haben **keine** Implementierung; in `compat/` stehen Schattenkopien `Index`/`Maintain`; `types::Maintain` ist ein dritter, der als einziger implementiert ist. `contract::Clock` (`now_unix_ms`) und `contract::Rng` (`&mut self`) duplizieren `env` und werden intern nicht benutzt. | [L] `grep` | mittel | S-36g |
| VEC-F-08 | `From<contextra_types::DistanceMetric> for DistanceMetric` fällt bei unbekannten Varianten still auf `Cosine` zurück (fail-open). | [L] `compat/types.rs` | mittel | S-36e |
| VEC-F-09 | `VectorError` leitet `Clone` und `PartialEq` ab und trägt I/O-Fehler nur als `String`; die Fehlerkette (`#[source]`) geht verloren (SPEC §6.3 Pkt. 3, Ausnahme „I/O mit `#[source]`“). | [L] `error.rs` | niedrig | S-36e |
| VEC-F-10 | Persistenzformat ohne Breitenfeld: Header 84 Byte, `HNSW_VERSION = 2`, `NodeRecord` 25 Byte (64 Bit) oder 33 Byte (128 Bit). Eine Breitenabweichung wird nur heuristisch erkannt (Vergleich `vector_offset` von Record 0, nur bei Version 2 und `node_count > 0`); Version 1 umgeht die Prüfung. | [L] `persistence/header.rs`, `mmap.rs`, `node.rs` | hoch (INV-1, §6.2) | S-36i, S-36j |
| VEC-F-11 | `HnswHeader::verify_checksum` wird in `src/` nur von Tests aufgerufen; beim Öffnen wird keine Header-Prüfsumme gelesen oder geprüft. Die Doku („BLAKE3-Prüfsumme“) suggeriert Schutz, den es nicht gibt. | [V] `grep` | mittel | S-36i |
| VEC-F-12 | `load_mmap_from_instance` und weitere Pfade nutzen direkte Indizierung und Slicing (z. B. `cal_bytes[i * 4..(i + 1) * 4]`, `maxes[i]`), `as usize`-Casts auf Dateiinhalt. | [L]; Clippy 104 + 19 | mittel | S-36d |
| VEC-F-13 | Die gesamte INV-12-Lintgruppe schlägt unter Clippy fehl (204/221 Fehler). | [V] | hoch | S-36d |
| VEC-F-14 | DiskANN-Modul unter `experimental-diskann` kompiliert nicht (Syntaxfehler plus 21 Folgefehler); `experimental-diskann` impliziert `mmap` nicht, obwohl der Code `contextra_sys` nutzt. G-L4 `each-feature` ist rot. | [V] | hoch (Blocker für S-80) | S-36c |
| VEC-F-15 | Test- und Bench-Landschaft ist defekt (3.4): 64/22 Targets kompilieren nicht in Default/`workspace`; mehrere Tests sind gegen Dateien geschrieben, die nicht mehr existieren; Beispiel und README-Quickstart kompilieren nicht. | [V] | hoch | S-36b |
| VEC-F-16 | `[[test]]`-Blöcke mit `required-features` fehlen für alle Tests, die `ports`/`types` brauchen (nur `hnsw_tests` hat einen). | [L] `Cargo.toml` | hoch | S-36b |
| VEC-F-17 | V-VETO-Wachtest `partial_rebuild_veto_f02_guard` kompiliert in keiner geprüften Konfiguration. | [V] | hoch | S-36b |
| VEC-F-18 | Feature-Hygiene (R10): ohne `cfg`-Nutzung und ohne Weiterleitung sind `std`, `serde`, `tracing`, `inspect`, `graph`; reine Weiterleitungen `experimental`, `experimental-all`, `diskann`, `quantize-rabitq`, `ports-compat`. Die geforderten Namen `exp-*` fehlen. `memmap2` ist nicht optional, obwohl `MmapIndex::open` hinter `mmap` steht; `rand` steht als normale Abhängigkeit, wird laut `grep` aber nur in einem `#[cfg(test)]`-Test von `quantize_rabitq.rs` genutzt. | [V] `grep -o 'feature = "…"'`; [L] | niedrig | S-36h |
| VEC-F-19 | Crate-lokale Dokumente sind veraltet: `SPEC.md`/`PLAN.md` auf v3 („Default `std, ports-compat`“, „Ist: Ring 0“), `README.md` behauptet „keine internen Pflichtabhängigkeiten“ (tatsächlich `ids`, `env`, `mvcc`) und beschreibt eine API, die ohne Brücke nicht existiert; `REPORT.md` fehlt; `capabilities.fragment.toml` führt `transitional = ["ports-compat"]`. | [L] | niedrig | S-36l |
| VEC-F-20 | `compat/hnsw_vector_index.rs` trägt `#[cfg(any(feature = "ports-compat", feature = "workspace"))]` redundant zu `compat/mod.rs`; 10 bis 17 Dead-Code- und Unused-Import-Warnungen (u. a. `search_filtered_internal` ist ohne Brücke ungenutzt). | [V] | niedrig | S-36d |
| VEC-F-21 | `async fn` im Kern ohne Wartepunkt: `search_filtered_internal` hat 0 `.await`, die `VectorIndex`-Methoden in `compat/` rufen nur den Kern. Die Asynchronität ist dekorativ und zwingt Kernbenutzer in eine Laufzeit (README-Beispiel nutzt `#[tokio::main]`). | [L] | niedrig | S-36f |
| VEC-F-22 | Das Wurzel-Manifest `Cargo.toml`, `capabilities.toml` und `deny.toml` wurden im HEAD-Commit `24487fa16` gelöscht; der Workspace baut so nicht. | [V] `git show --stat HEAD` | hoch (Repo, nicht Crate) | S-36a |
| VEC-F-23 | `governance/`, `.github/`, `justfile`, `.githooks/` fehlen im Klon, obwohl SPEC §7.3 sie als vorhanden führt. Die Gates sind damit in dieser Umgebung nicht in kanonischer Form ausführbar. | [V] `ls`, `git ls-files` | mittel (Repo) | S-36a |

### 4.1 Korrekturen an `docs/SPEC.md` (Rückmeldung)

| Stelle | SPEC sagt | HEAD `24487fa16` |
| --- | --- | --- |
| §1.5, Anhang C, S-36 | `vector`: 14 ungegatete Brückenreferenzen, Closure 5 statt 3, Default `ports-compat` | 0 Referenzen [V]; Closure 3 [V]; Default `std` [L] |
| F-15 | Altdateien `hnsw/vector_index_impl.rs`, `purge.rs`, `scanner.rs`, `candidate_stream.rs`, `diskann/vector_index_impl.rs` neben `compat/` | Dateien existieren nicht mehr |
| F-07 | Default `workspace` in `mvcc` zieht `types`/`ports` in die Closure | `mvcc` hat `default = []` [L]; Closure von `vector` ist 3 [V] |
| §4.1 Tabelle | `vector`-Features `mmap`, `simd`, `exp-*`, `workspace` | Zusätzlich vorhanden: `partial-index-rebuild`, `docid-128`, `diskann`, `quantize-rabitq`, `ports-compat`, fünf leere Features |
| S-36 | „V-VETO-Wachtest existiert (Grünstatus nicht geprüft)“ | Existiert, kompiliert nicht [V] |
| S-80 | `vector/diskann/` herauslösen | Modul kompiliert nicht; die Ebene des neuen Crates ist in §4.1 nicht festgelegt (offene Entscheidung D-V7) |
| §7.3 | Gates in CI, `justfile` | Dateien im Klon nicht vorhanden [V] |

---

## 5 Zielarchitektur

### 5.1 Leitentscheidungen

| ID | Entscheidung [D] | Begründung |
| --- | --- | --- |
| **D-V1** | Der **Kern ist synchron.** Alle `async`-Signaturen entstehen nur in `compat/` (Port-Trait). | VEC-F-21: kein Wartepunkt im Kern; Kernbenutzer brauchen keine Laufzeit. |
| **D-V2** | Die Kernmethoden heißen **`stage_insert`, `stage_delete`, `commit_tx`, `rollback_tx`, `rollback_to_tx`, `search`, `search_at`, `search_filtered`**. Sie heißen nicht `insert`/`commit`, damit sie die `async`-Port-Methoden nicht per Methodenauflösung verdecken (inhärente Methoden gewinnen vor Trait-Methoden). | Bestehende Verbraucher (`db`, `engine`, Tests) rufen `.insert(...).await` über das Port-Trait. |
| **D-V3** | **Eine** Suchpipeline: `plan → descent → layer0 → rerank → score → order`. Port-Variante, Vertragsvariante und gefilterte Suche sind dünne Adapter. Tie-Break: Score absteigend, `DocId` aufsteigend (INV-5). | VEC-F-02. |
| **D-V4** | Transaktionssemantik: Operationen einer Tx werden **in Einreihungsreihenfolge** angewendet; `commit_tx` ist ein Zwei-Phasen-Ablauf (Phase 1 berechnen ohne Zustandsänderung, Phase 2 anwenden, die nicht fehlschlagen darf). Ein Fehler in Phase 1 lässt den Index und den Tx-Puffer unverändert. | VEC-F-04, VEC-F-05. |
| **D-V5** | `HnswHeader` Version 3: Feld `id_width_bits: u16` bei Offset 84, reserviert `u16` bei 86 (= 0), danach 32 Byte BLAKE3-Header-Prüfsumme über die Bytes `[0, 88)` bei Offset 88; `SIZE_V3 = 120`. Der Leser prüft die Prüfsumme beim Öffnen fail-closed. Version 1 und 2 bleiben lesbar (INV-1). | VEC-F-10, VEC-F-11, SPEC §6.2. |
| **D-V6** | `OpenOptions::assume_legacy_id_width(bits)` ist der einzige Weg, eine Datei ohne Breitenfeld zu öffnen. `MmapIndex::open(path)` bleibt als `#[deprecated]`-Hülle erhalten (INV-2, R9) und ruft intern `assume_legacy_id_width(ID_WIDTH_BITS)` auf, **nur wenn** die heutige Layout-Heuristik besteht; sonst `IdWidthUnknown`. | Erhält Verbraucher (`db`), erfüllt die Absicht von §6.2. Wörtlich verlangt §6.2 „sonst `IdWidthUnknown`“ ohne Heuristik; Abweichung als ADR bestätigen. |
| **D-V7** | `contextra-vector-diskann` wird **E2** (Integration) und hängt per normaler Kante von `vector` (mit `default-features = false`) ab. | §4.1 legt die Ebene nicht fest. E1b → E1b im Kern wäre ein R13-Verstoß; E2 darf E1 nutzen. Alternative: R13-Ausnahme per ADR. |
| **D-V8** | `VectorError` behält Variantennamen und `Display`-Texte bitgleich (SPEC §6.3 Pkt. 2). I/O-Fehler werden zu `Io(Arc<std::io::Error>)` mit manuell implementiertem `PartialEq` (Art und Text) und `#[source]`. Vorab: `grep` über Verbraucher auf Muster `VectorError::Io(`; bei Treffern Variante `IoSource` ergänzen, `Io(String)` bleibt `#[deprecated]`. | VEC-F-09; Variantenform ändern wäre ein Bruch. |
| **D-V9** | `partial-index-rebuild` behält seinen Namen (VETO-F02, durch ADR zu dokumentierende Ausnahme von X2/`exp-*`); alle anderen Experimente heißen `exp-*`. | Der Name ist in `VETOES.md`, Tests und Benches verankert. |

### 5.2 Modul-Zielbild

```
crates/contextra-vector/src/
├── lib.rs                 Fassade, Re-Exports (R9), #![forbid(unsafe_code)], Lint-Block
├── error.rs               VectorError (D-V8), Result
├── types.rs               DocId/TxId (ids), Clock/Rng (env), DistanceMetric, ScoredDocument, Maintain
├── contract.rs            nur noch: DistanceFn, BlockTransform, ExternalId (TryFrom<DocId>), Metric, ScoredPoint; Index/Maintain/Clock/Rng #[deprecated] bis S-70
├── casting.rs             geprüfte Konvertierungen (usize/u32/u64/i64) – Grundlage der Lint-Sanierung
├── hnsw/
│   ├── api.rs             NEU: inhärente Kern-API (stage_*, commit_tx, rollback_*, search*, len, stats, all_doc_ids)
│   ├── commit.rs          NEU: Zwei-Phasen-Commit (D-V4)
│   ├── search.rs          NEU: eine Suchpipeline (D-V3); core_search/* wird Bestandteil
│   └── …                  bestehende Module (arena, deletion, core_rebuild, staging, …)
├── persistence/
│   ├── header.rs          v1/v2/v3-Leser, v3-Schreiber (D-V5)
│   ├── open_options.rs    NEU: OpenOptions, IdWidthMismatch/IdWidthUnknown
│   └── …
├── quantize/, distance/, acorn/, compute_pool.rs, fixed_hasher.rs   unverändert in der Rolle
├── partial_rebuild.rs     unverändert (VETO-F02)
└── compat/                (nur hinter `workspace`)
    ├── hnsw_vector_index.rs  nur noch Delegation: async-Hüllen + Fehlerabbildung
    ├── error.rs, types.rs, purge.rs, scanner.rs, candidate_stream.rs
    └── diskann_vector_index.rs  → wandert mit S-80
```

### 5.3 Kern-API (Skizze, normativ in Form und Fehlerverhalten)

```rust
// crates/contextra-vector/src/hnsw/api.rs   (kein contextra_types / contextra_ports)
impl<K: DistanceKernel> GenericHnswIndex<K> {
    pub fn stage_insert(&self, tx: TxId, id: DocId, embedding: &[f32]) -> Result<()>;
    pub fn stage_delete(&self, tx: TxId, id: DocId) -> Result<()>;
    pub fn commit_tx(&self, tx: TxId) -> Result<CommitStats>;       // D-V4, atomar
    pub fn rollback_tx(&self, tx: TxId) -> Result<()>;              // verwirft Puffer
    pub fn rollback_to_tx(&self, tx: TxId) -> Result<RollbackStats>;// physisch, INV-VEC-*

    pub fn search(&self, query: &[f32], k: usize) -> Result<Vec<ScoredDocument>>;
    pub fn search_at(&self, query: &[f32], k: usize, seq_no: u64) -> Result<Vec<ScoredDocument>>;
    pub fn search_filtered(
        &self, query: &[f32], k: usize,
        filter: Option<&(dyn Fn(DocId) -> bool + Send + Sync)>,
    ) -> Result<Vec<ScoredDocument>>;

    pub fn len(&self) -> usize;
    pub fn all_doc_ids(&self) -> Result<Vec<DocId>>;
    pub fn stats(&self) -> IndexStats;          // Kern-Typ, kein ports::VectorIndexStats
    pub fn last_tx_id(&self) -> TxId;
}
```

Verträge:

1. `search(q, 0)` liefert `Ok(vec![])`, nie einen Panic (VEC-F-03). `k > MAX_SEARCH_K` liefert `VectorError::InvalidInput`.
2. Dimensions- und NaN/Inf-Prüfung geschieht in **einer** Funktion, die Einfügen und Suche gemeinsam nutzen. Die Fehlertexte entsprechen denen der heutigen Port-Variante (bitgleich in `compat`, SPEC §6.3).
3. Die Suche verändert keinen Indexzustand (kein `set_entry_point` im Lesepfad); ein Fallback-Einstiegspunkt wird lokal berechnet.
4. `commit_tx`: Reihenfolge der Operationen wie eingereiht. Ein `stage_delete(1)` nach `stage_insert(1)` in derselben Tx entfernt das Dokument (Regressionstest zu VEC-F-04). Ein Fehler in Phase 1 lässt Index, Quantizer und Tx-Puffer unverändert; nach Phase 2 gilt `last_tx_id = max(last_tx_id, tx)`.
5. Kein `async`, keine Abhängigkeit von `tokio` im Kern (D-V1).
6. Zero-Panic: kein `unwrap`, `expect`, `panic!`, keine direkte Indizierung (INV-12).

Port-Brücke (nur `compat/`):

```rust
// compat/hnsw_vector_index.rs — nur Delegation
impl contextra_ports::VectorIndex for HnswIndex {
    async fn insert(&self, tx: TxId, id: DocId, e: &[f32]) -> contextra_types::Result<()> {
        self.stage_insert(tx, id, e).map_err(Into::into)
    }
    async fn search(&self, q: &[f32], k: usize) -> contextra_types::Result<Vec<contextra_types::ScoredDocument>> {
        self.search(q, k).map(|v| v.into_iter().map(Into::into).collect()).map_err(Into::into)
    }
    // … commit, delete, rollback, search_at, search_filtered, stats analog
}
```

### 5.4 Fehlermodell

Pflichtinhalt `docs/decoupling/errors/vector.md` (SPEC §6.3): je Variante lokale Variante, `ContextraError`-Abbildung, Display-String. Erhebung: `grep -rn 'ContextraError::[A-Z]' src`. Tabellengetriebener Display-Test je Variante in `tests/error_display_test.rs` (existiert; Abdeckung prüfen). Konvertierung der Metrik: `TryFrom<contextra_types::DistanceMetric>`; die `From`-Variante wird `#[deprecated]` (VEC-F-08).

### 5.5 Feature-Modell (Ziel, R10)

| Feature | Zieldefinition | Default | Bemerkung |
| --- | --- | --- | --- |
| `std` | leer, bleibt | ja | D-19, ADR-Hinweis „ohne `cfg`“, bis ein Nutzen entsteht |
| `mmap` | `["dep:contextra-sys", "dep:memmap2"]` | nein | `MmapIndex::open`, `load_mmap*` |
| `simd` | `["dep:contextra-simd", "contextra-simd?/serde"]` | nein | Orakeltest (S-36k) |
| `workspace` | `["dep:contextra-ports", "dep:contextra-types", "mmap"]` | nein | genau die Brücke aus §4.1 |
| `docid-128` | `["contextra-ids/docid-128"]` | nein | nur weiterleiten (D-02) |
| `partial-index-rebuild` | wie heute | nein | D-V9 |
| `exp-rabitq` | wie `experimental-rabitq` | nein | Alias `experimental-rabitq` bis S-70 |
| `exp-predicate-search` | wie `experimental-predicate-augmented-search` | nein | Alias bis S-70 |
| `exp-diskann` | entfällt mit S-80 | nein | bis dahin: `["mmap"]` plus Alias `experimental-diskann` |
| `experimental-all` | Summe aller `exp-*` | nein | |
| `ports-compat` | `["workspace"]` | nein | `#[deprecated]`-Alias bis S-70 |
| entfernt | `serde`, `tracing`, `inspect`, `graph`, `diskann`, `quantize-rabitq`, `experimental` | | nach `cargo hack`-Auditlauf; Features ohne `cfg` und ohne Weiterleitung entfallen (R10) |

Abhängigkeiten: `memmap2` optional hinter `mmap`; `rand` nach `dev-dependencies` (nach Bestätigung per `grep`, VEC-F-18).

### 5.6 Persistenz (Ziel)

`HNSW_VERSION = 3`, Magic unverändert `0x484E5357`.

| Offset | Feld | Typ | Version |
| --- | --- | --- | --- |
| 0–83 | v2-Header unverändert | | 1–3 |
| 84–85 | `id_width_bits` (64 oder 128) | `u16` LE | 3 |
| 86–87 | reserviert, 0 | `u16` | 3 |
| 88–119 | BLAKE3 über `[0, 88)` | `[u8; 32]` | 3 |

Öffnen (Matrix nach SPEC §6.2):

| Build | Datei | Ergebnis |
| --- | --- | --- |
| 64 | v3, 64 | öffnen |
| 128 | v3, 128 | öffnen |
| 64 | v3, 128 | `IdWidthMismatch { stored: 128, expected: 64 }` |
| 128 | v3, 64 | `IdWidthMismatch { stored: 64, expected: 128 }` |
| beliebig | v1/v2 ohne Option | `IdWidthUnknown` (D-V6: `open()` mit bestandener Heuristik) |
| beliebig | v1/v2 mit `assume_legacy_id_width(b)` | öffnen, Layout-Prüfung gegen `b` |
| beliebig | falsche Header-Prüfsumme | `VectorError::Storage`, fail-closed |

Prüfregel: Nach jedem abgelehnten Öffnen ist die Datei byte-identisch zum Zustand davor (Hash-Vergleich im Test). Reihenfolge: erst Leser (v1/v2/v3), dann Schreiber (v3), mit Rückwärts-Lesetest über die vorhandenen v2-Golden-Dateien (`hnsw_raw.bin`, `hnsw_sq8.bin`, `hnsw_tombstones.bin`). Der Schreibpfad (`write_atomic`, Temp-Datei, Rename, Verzeichnis-`fsync`) bleibt unberührt (INV-6). `NodeRecord::SIZE` und die Vektor-/Verbindungs-Offsets ändern sich nicht; nur `nodes_offset` verschiebt sich um 36 Byte.

### 5.7 Determinismus und Umwelt

- `Clock` und `Rng` kommen aus `env` (§5.2 der SPEC). `contract::Clock`/`Rng` werden `#[deprecated]` und intern nicht mehr verwendet; die lokalen Kopien von `now_unix_ms` verschwinden über `env::unix_ms` (Differenztest nach §6.4, `ManualClock`).
- Ebenenzuweisung hängt von `hash(doc_id, seed)` ab (INV-INDEX-APPLY-1); `level_rng_golden.json` bleibt maßgeblich.
- Parallelität: `ComputePool` ohne Tokio-Bindung bleibt; Ergebnisgleichheit der Rangfolge statt Gleichheit der Seiteneffektreihenfolge (D-10).

---

## 6 Umsetzungsplan

### 6.1 Rahmenregeln

- Jeder Schritt hat ein Commit-Paar `S-36x A: …` (verschieben, umbenennen, Tests) und `S-36x B: …` (umbauen), einzeln revertierbar (INV-3, D-08).
- Definition of Done je Schritt nach SPEC §9.1 (Punkte 1 bis 8). Vor jedem Schritt: Wachtest und Test-Baseline aus S-36b laufen lassen.
- Rückfall (§9.5): Gate nach B rot und nicht in einem Arbeitstag behebbar → B zurücknehmen; Differenztest rot → Schritt abbrechen, Toleranz nie anpassen; Format-Hash weicht ab → Schritt abbrechen.
- Die Schritt-IDs `S-36a` bis `S-36l` sind Unterschritte der SPEC-Schritte S-36, S-12a/b, S-11 und S-80; sie ersetzen keinen SPEC-Schritt.

### 6.2 Reihenfolge und Wellen

| Welle | Schritte | Ziel |
| --- | --- | --- |
| A Ausgangslage reparieren | S-36a, S-36b, S-36c, S-36d | Alles baut, alles ist messbar, Lints grün. Kein Verhaltensunterschied. |
| B Kern-API | S-36e, S-36f, S-36g | Der Kern wird benutzbar; Brücke delegiert nur. |
| C Format und Features | S-36h, S-36i, S-36j, S-36k | R10, Format v3, SIMD-Orakel. |
| D Herauslösen und Abnahme | S-80a bis S-80c, S-36l | DiskANN eigenes Crate, §8.4-Nachweis. |

Zwang: S-36a vor allem; S-36b vor S-36f (ohne grüne Test-Baseline ist keine Regression erkennbar); S-36c vor S-80; S-36f vor S-36g; S-36i (Leser) vor S-36j (Schreiber); S-11 (`env` ohne `'static`, `SeededRng::fork`) vor S-36g; S-12a (`mvcc` Default leer, bereits erfüllt [L]) ist keine Vorbedingung mehr.

### 6.3 Schritte

---

#### S-36a Ausgangslage und Baseline

**Ziel:** Ein reproduzierbarer, dokumentierter Startzustand.

**Arbeit**
1. Klären, warum `24487fa16` `Cargo.toml`, `capabilities.toml` und `deny.toml` gelöscht hat. Absicht (Übergabeformat nach `CLAUDE.md`) oder Versehen. Bei Versehen die drei Dateien aus `HEAD~1` wiederherstellen (VEC-F-22).
2. Prüfen, ob `governance/`, `.github/`, `justfile`, `.githooks/` fehlen oder nur im Klon fehlen (VEC-F-23). Ohne sie sind die Gates in §7.3 nur über die Skripte in `tools/` ausführbar.
3. Baseline-Bericht für `vector` erzeugen (SPEC §8.3, D-15): HEAD-Hash, Datum, Build-Matrix (3.3), Test-Zählung (3.4), Clippy-Zählung (3.5), Closure (3.2), `gate_bridge_null`, `isolation_proof`. Ablage `crates/contextra-vector/docs/decoupling/BASELINE.md` (ersetzt die Fassung vom 10.11.).
4. Die Reproduktionsbefehle aus Anhang A in `crates/contextra-vector/docs/decoupling/REPRODUCE.md` festhalten.

**Dateien:** `Cargo.toml`, `capabilities.toml`, `deny.toml` (nur bei Wiederherstellung), `docs/decoupling/BASELINE.md`.
**Abnahme:** `cargo check -p contextra-vector --locked` im Workspace grün; Bericht trägt HEAD-Hash.
**Gates:** G-L3 (Teil).
**Risiko:** Gering. Die Wiederherstellung ist eine Repo-Entscheidung außerhalb des Crates; ohne sie ist kein Folgeschritt prüfbar.

---

#### S-36b Test-Target-Sanierung (VEC-F-15, -16, -17)

**Ziel:** `cargo check -p contextra-vector --all-targets --locked` ist grün in drei Konfigurationen: default, `workspace`, `workspace,simd,partial-index-rebuild,docid-128`. Keine Bibliotheksänderung.

**Arbeit**
1. **Commit A (nur Metadaten):** In `Cargo.toml` für jedes Test-Target, das `contextra_ports`/`contextra_types` nutzt, `[[test]] required-features = ["workspace"]` setzen. Ermittlung: `grep -l 'contextra_ports\|contextra_types' tests/*.rs`. Dasselbe für Benches und `examples/` (`standalone_vector` bis S-36l mit `required-features = ["workspace"]`, danach auf Kern-API umgestellt). Das ist die vom Fix-Report (`reports/FIX_REPORT_2026-11-16.md`, Abschnitt 4) vorgeschlagene Option und ändert keine Bibliothek.
2. **Commit B (Test-Drift):** Die 21 Test-Targets und den Bench aus 3.4 gegen die aktuelle API reparieren. Vorgehen je Target: Fehlerklasse bestimmen (E0308, E0061, fehlende Methode, `include_str!`), minimal anpassen. `include_str!`-Wächter (z. B. auf `src/distance.rs`, `src/hnsw/core_search.rs`) auf die Verzeichnisstruktur umstellen oder durch `grep`-freie Prüfungen ersetzen. Keine Abschwächung von Assertions; jede inhaltliche Anpassung steht mit Begründung im Commit-Text.
3. **V-VETO-Wachtest** `partial_rebuild_veto_f02_guard` kompilierbar machen (Feature `partial-index-rebuild`, 3 Fehler) und grün bestätigen, bevor S-36f beginnt. Wenn er fachlich rot ist, ist das ein Befund und kein Grund, ihn abzuschwächen.
4. **Test-Baseline** festhalten: Anzahl Tests je Target und Konfiguration vor und nach dem Schritt (SPEC S-12b: „Testanzahl vorher und nachher gleich“). Ausgangswerte: 56 Unit-Tests, 58 Integrationstargets grün (3.4).
5. Die 7 Targets mit `required-features` auf `experimental-diskann`/`simd` bleiben bis S-36c/S-36k außen vor, sind aber im Bericht mit „nicht ausgeführt“ zu führen.

**Abnahme:** 0 Kompilierfehler in den drei Konfigurationen; alle bisher grünen Tests weiter grün; Wachtest grün; Testzahl dokumentiert.
**Gates:** G-L3 (Targets), G-C1 (unverändert grün).
**Risiko:** Mittel. Die Reparatur kann echte Produktfehler sichtbar machen (VEC-F-03/-04 sind solche). Neu sichtbare rote Tests werden als Befund erfasst, nicht stumm repariert.

---

#### S-36c DiskANN-Reparatur (VEC-F-14)

**Ziel:** `experimental-diskann` baut und besteht G-L4 `each-feature`; die Voraussetzung für S-80.

**Arbeit**
1. `diskann/persistence/loader.rs:133–139`: `return Err(VectorError::Index(format!(…)))` fehlt eine schließende Klammer (`)));`).
2. Die 21 Folgefehler beheben (Anhang B): fehlende `use`-Zeilen (`SearchCandidate`, `std::sync::atomic::Ordering`, `Arc`, `VectorData`, `DiskAnnHeader`, `DiskAnnIndexInner`), `CachedNode` ist privat (Sichtbarkeit `pub(super)`/`pub(crate)` je nach Nutzung), `contextra_sys` ist ohne `mmap` nicht aufgelöst.
3. Feature `experimental-diskann = ["mmap"]` (impliziert `dep:contextra-sys`).
4. Die 6 Test-Targets mit `required-features = ["experimental-diskann"]` (`diskann_fault_injection_test`, `diskann_corruption_fallback_test`, `diskann_recall_test`, `mmap_toctou_test`, `quantize_persistence_audit`, `recall_audit`) sowie die drei weiteren DiskANN-Tests ohne `required-features` (`diskann_search_filtered`, `diskann_docid128_tombstones`, `wp07_diskann_formats`) ausführen und das Ergebnis berichten. Rote Tests sind Befunde. Keines dieser Targets wurde in dieser Analyse ausgeführt.
5. Golden-Hashes für `diskann.dann`, `diskann.pending.wal`, `diskann.tombstone.wal` bleiben bitgleich (INV-1).

**Abnahme:** `cargo check -p contextra-vector --features experimental-diskann` grün; `tools/isolation_proof.py --crate contextra-vector` durchgehend grün; `sha256sum -c tests/golden/MANIFEST.sha256` unverändert.
**Gates:** G-L3, G-L4, G-F1.
**Risiko:** Mittel. Die Reparatur ist wahrscheinlich mechanisch (nach einem Datei-Split); ob die DiskANN-Logik danach korrekt ist, zeigen erst diese Tests.

---

#### S-36d Lint- und Warnungs-Sanierung (INV-12; VEC-F-12, -13, -20)

**Ziel:** `cargo clippy -p contextra-vector --all-targets --locked -- -D warnings` ist grün in `--no-default-features` und `workspace`. Kein Verhaltensunterschied.

**Arbeit**
1. **Commit A:** `casting.rs` ausbauen: geprüfte Konvertierungen `u64 → usize`, `usize → u32`, `i64 → usize`, `usize → u64` mit `Result<_, VectorError>`; plus `get_or_err`-Hilfen für Slices (`slice.get(a..b).ok_or_else(…)`).
2. **Commit B (je Datei ein Commit, Reihenfolge nach Risiko):** `hnsw/types/persist.rs` (Dateieingaben, VEC-F-12), `persistence/*`, `hnsw/arena.rs` (34), `quantize/train.rs` (21), `quantize/distance.rs` (14), `hnsw/deletion.rs` (14), `distance/scalar.rs` und `distance/inject.rs` (je 13), `core_rebuild/*`, `core_insert/*`, `compat/*`. Heißpfade (`distance/*`, `arena`, `quantize/distance`): Indexierung durch `chunks_exact`/Iteratoren statt `get` ersetzen, nicht durch zusätzliche Prüfungen in der inneren Schleife. Wo ein Index mathematisch garantiert ist, ist `#[allow(clippy::indexing_slicing)]` mit Begründungskommentar (Invariante, die ihn sichert) zulässig, höchstens je Funktion und nur nach Review; Anzahl der `allow`s steht im Bericht.
3. Dead-Code und Unused-Imports entfernen (10 bis 17 Warnungen, VEC-F-20); überflüssige `#[cfg(any(feature = "ports-compat", feature = "workspace"))]` auf Elementen in `compat/` streichen.
4. **Leistungsnachweis:** `distance_bench`, `sq8_bench`, `hnsw_bench` (letzterer nach S-36b kompilierbar) vor und nach dem Schritt; Regression > 3 % im Median ist ein Befund (INV-6 gilt sinngemäß, Messwerte im Bericht).
5. `rg 'unwrap\(\)|expect\(|panic!' src` in Nicht-Test-Code muss leer sein (INV-12; `unwrap-gate`).

**Abnahme:** Clippy grün; Test-Baseline unverändert; Golden-Hashes unverändert; Bench-Vergleich im Bericht.
**Gates:** G-L3, G-F1, G-D1 (Rangfolge unverändert), `unwrap-gate`.
**Risiko:** Mittel. Mechanisch, aber breit (≈ 220 Stellen). Deshalb je Datei ein Commit und vor jedem Commit die Test-Baseline.

---

#### S-36e Fehlerabbildung (VEC-F-08, -09)

**Ziel:** SPEC §6.3 erfüllt: Tabelle, bitgleiche Texte, keine verlorene Fehlerkette, keine stille Metrik-Rückfalllösung.

**Arbeit**
1. `docs/decoupling/errors/vector.md` anlegen (heute existiert `docs/decoupling/ERRORS.md` mit 58 Zeilen; Form prüfen und auf den Pfad der SPEC umziehen).
2. `grep -rn 'VectorError::Io(' crates benchmarks` über alle Verbraucher; danach D-V8 umsetzen.
3. `compat/types.rs`: `TryFrom<contextra_types::DistanceMetric> for DistanceMetric`; `From` bleibt `#[deprecated(since, note)]` (R9). Die Konvertierung `ScoredDocument` bleibt verlustfrei; die Rückrichtung rechnet `distance = 1.0 − score` nur für `Cosine` korrekt; Dokumentationskommentar oder Abbildung je Metrik.
4. Tabellengetriebener Test: je `VectorError`-Variante und je `From`-Abbildung `Display` bitgleich zum alten Text (Golden-Strings aus dem Stand vor dem Schritt).
5. Fehlerinjektion: Ein I/O-Fehler beim Öffnen liefert `source()` mit der ursprünglichen `io::ErrorKind`.

**Abnahme:** `tests/error_display_test.rs`, `s36_error_mapping.rs` grün; G-E1 (`tools/error_topology_mapper.py`) ohne neue Befunde.
**Gates:** G-E1, G-A1, GL.
**Risiko:** Gering bis mittel (Variantenform, D-V8).

---

#### S-36f Kern-API und eine Suchpipeline (VEC-F-01 bis -05, -21)

**Ziel:** Der Kern trägt die Funktion; `compat/` delegiert nur.

**Arbeit**
1. **Commit A (Differenztest-Gerüst, §6.4):** Neuer Test `tests/diff_search_pipeline.rs`: alte Port-Implementierung (bleibt unverändert im Test über die Brücke erreichbar) und neue Kern-API werden mit gleichen Eingaben verglichen: 10.000 Zufallsvektoren (`SeededRng`, Dimension 4, 17, 128, 768), Metriken Cosine/Euclidean/DotProduct, quantisiert und unquantisiert, `k ∈ {1, 5, 100}`. Gleichheit der `DocId`-Folge und bitgleiche Scores. Noch bevor neue Logik existiert, kompiliert der Test gegen eine dünne neue API, die auf den alten Pfad verweist.
2. **Commit B (Umbau):** `hnsw/api.rs`, `hnsw/commit.rs`, `hnsw/search.rs` nach 5.3 und D-V3/D-V4. Die drei Suchvarianten gehen in die eine Pipeline. Der Quantizer-Pfad der Port-Variante (`resolve_dist`, `ef * 4`) wird für **alle** Aufrufer gleich angewendet; die abweichende, quantizer-lose Vertragsvariante ist ein dokumentiertes Verhaltensdetail: Differenztest dokumentiert den Unterschied, kein stilles Angleichen (INV-5, D-20-Muster).
3. Regressionstests (alle neu):
   - `k = 0` → `Ok(vec![])` (VEC-F-03);
   - `insert(1), insert(2), delete(1)` in einer Tx → Suche liefert nur `2` (VEC-F-04);
   - Fehler in Phase 1 von `commit_tx` (z. B. Dimension in einer von drei Operationen verfälscht per Fehlerinjektion) → Index, Quantizer und Tx-Puffer unverändert (VEC-F-05);
   - Suche verändert den Einstiegspunkt nicht (Zustandsvergleich vorher/nachher).
4. `compat/hnsw_vector_index.rs` schrumpft auf Delegation. Dublette `search_filtered_internal` fällt weg oder wird die Pipeline-Hülle.
5. `README.md`-Quickstart wird in S-36l auf die Kern-API umgestellt.

**Abnahme:** Differenztest grün; alle 58 Integrationstargets plus die in S-36b reparierten bleiben grün; `hnsw_scanner_purge_test`, `hnsw_snapshot_delete_test`, `k02_snapshot_rebuild_determinism`, `hnsw_rebuild_search_consistency` (Snapshot- und Rebuild-Invarianten) unverändert grün; G-C1 (Loom/TSan-Pfadfilter deckt `vector` ab [Ü]) grün.
**Gates:** GL, GD, GC, GE, G-A1 (Pfade bleiben).
**Risiko:** **Hoch.** Berührt Commit- und Suchpfad. Absicherung: Differenztest vor dem Umbau, INV-6 (Durability-Pfade, `write_atomic`, Rebuild-Swap) bleibt unberührt, Revert-Test (DoD Punkt 7).

---

#### S-36g Vertragsdatei und Umwelt (VEC-F-06, -07; SPEC S-36, S-11)

**Ziel:** Ein Besitzer je Konzept (R4): `Clock`/`Rng` aus `env`, ein `Maintain`, keine Schattentraits.

**Voraussetzung:** S-11 (`'static`-Bound an `env::Clock`/`env::Rng` entfernt, `SeededRng::fork` vorhanden).

**Arbeit**
1. `contract::Clock` und `contract::Rng` auf `#[deprecated]` setzen; Re-Exports in `lib.rs` (`ContractClock`, `ContractRng`) bleiben bis S-70.
2. Schattentraits `Index`/`Maintain` in `compat/hnsw_vector_index.rs` entfernen. `contract::Index`/`Maintain`: entscheiden und umsetzen: (a) mit korrekter Semantik für den Kern implementieren (Payload ist ein Vektor, kein Byte-Blob; `&mut self` ist für einen intern synchronisierten Index unpassend) oder (b) `#[deprecated]` setzen und mit S-70 entfernen. **Empfehlung (b)**, weil kein Verbraucher sie nutzt [L] und die Kern-API aus 5.3 sie ersetzt.
3. `types::Maintain` bleibt der einzige Wartungstrait; `compact`, `is_rebuild_required` über die Kern-API erreichbar.
4. `ExternalId → DocId`: `TryFrom<ExternalId> for DocId` mit Fehler bei Überlauf (64-Bit-Build) statt `as u64` (VEC-F-06); `From<DocId> for ExternalId` verlustfrei.
5. Die verbliebenen `now_unix_ms`-Kopien in `contract.rs`/`vector/contract.rs` über `env::unix_ms` ersetzen (Differenztest mit `ManualClock`, bitgleich, SPEC §6.4).

**Abnahme:** `tests/wp02_contract_object_safety.rs`, `wp02_error_contract.rs`, `wp02_primitives.rs` grün; G-A1 zeigt nur beabsichtigte Diffs (`#[deprecated]` statt Entfernung).
**Gates:** GL, GD (ms), GE, G-A1.
**Risiko:** Gering.

---

#### S-36h Feature-Hygiene (VEC-F-18; SPEC S-12a)

**Ziel:** Feature-Modell nach 5.5; G-H1 und G-L3 `--each-feature` grün.

**Arbeit**
1. **Commit A:** Alias-Einträge ergänzen (`exp-rabitq`, `exp-predicate-search`, `exp-diskann`), `cfg`-Gates im Quelltext auf die Zielnamen umstellen (`quantize_rabitq`, `acorn`-Gate, `lib.rs`); Altnamen bleiben als Weiterleitungen und tragen im Kommentar `MIGRATION(S-70)`.
2. **Commit B:** Leere Features entfernen (`std` bleibt mit Begründung, D-19); `memmap2` optional hinter `mmap`; `rand` nach `dev-dependencies`; `ports-compat` auf `["workspace"]` mit Hinweis `deprecated`; `[package.metadata.contextra]` (`ring = "0"`) auf aktuelle Ebene (E1b) bringen; `capabilities.fragment.toml`: `transitional` beibehalten bis S-70.
3. `tools/feature_audit.py` und `cargo hack check -p contextra-vector --each-feature --no-dev-deps --locked` ausführen und Ergebnis berichten. Nicht gegateter Code, der ohne `mmap` `memmap2` braucht (`MmapIndex`-Struct), wird hinter `mmap` gelegt oder die Struktur so geändert, dass der Kern ohne `memmap2` baut. Konsequenz für `pub use persistence::MmapIndex` in `lib.rs` (R9): Re-Export unter `cfg(feature = "mmap")`; Verbraucher (`db`) nutzen `workspace` und damit `mmap`.
4. Verbraucher-Check: `grep -rn 'contextra-vector' crates/*/Cargo.toml benchmarks/*/Cargo.toml` und `cargo check --workspace --all-targets --locked`.

**Abnahme:** `cargo hack` je Feature grün; `cargo tree -p contextra-vector -e normal --no-default-features` zeigt `ids`, `env`, `mvcc` und kein `memmap2`.
**Gates:** GH, GL, G-L3.
**Risiko:** Mittel. Feature-Umbenennungen sind API für Verbraucher (`db` leitet `experimental-diskann` weiter). Deshalb Aliase bis S-70.

---

#### S-36i Formatversion 3: Leser (VEC-F-10, -11)

**Ziel:** Der Leser versteht v1, v2 und v3 und erzwingt die Breitenmatrix. Kein Schreiber ändert sich.

**Arbeit**
1. **Commit A:** Golden-Korpus erweitern, ohne Produktivcode zu ändern: Generator `tests/golden/regen.sh` um einen Lauf mit `--features docid-128` und Dateinamen `hnsw_raw_128.bin`, `hnsw_sq8_128.bin`, `hnsw_tombstones_128.bin` erweitern; **Breitentest nach SPEC §6.2:** Korpus einmal mit 64 und einmal mit 128 Bit erzeugen und Bytes vergleichen (erwartet: verschieden, weil `NodeRecord::SIZE` 25 gegen 33 ist; damit ist `id_width_bits` Pflicht). Die Manifest-Zeilen kommen in `tests/golden/MANIFEST.sha256` und in das Wurzel-Manifest `tests/golden/<format>/` (SPEC §6.1, S-02).
2. **Commit B:** `HnswHeader` Version 3 (D-V5) lesen: Felder, `SIZE_V3`, Prüfsumme; `OpenOptions` und Fehler `IdWidthMismatch { stored, expected }`, `IdWidthUnknown` als `VectorError`-Varianten (`#[non_exhaustive]`, `compat/error.rs` bildet auf `ContextraError::Index` oder `InvalidInput` ab, Text im Fehlerregister). `MmapIndex::open_with(path, OpenOptions)`; `open(path)` nach D-V6. `try_from_bytes` akzeptiert 1, 2, 3; unbekannte Version → Fehler (bitgleicher Text nach Muster „Unsupported HNSW version: {}, expected 1, 2 or 3“ im Fehlerregister vermerken, weil sich der Text ändert).
3. Tests: (a) v2-Golden-Dateien öffnen wie bisher; (b) synthetische v3-Dateien (64/128) öffnen; (c) alle vier Kreuzfälle der Matrix lehnen ab und die Datei ist hash-gleich danach; (d) verfälschte Header-Prüfsumme lehnt ab; (e) Proptest `try_from_bytes` mit zufälligen Bytes: nie Panic (Bestand `proof_open_rejects_corrupt`, `persistence_mutation_matrix_test`, `wp03_persistence_corruption` bleiben grün).

**Abnahme:** G-F1 (neue und alte Hashes), Fuzz-Smoke (`proof_fuzz_smoke_driver`) grün.
**Gates:** GF, GL, GE, G-A1.
**Risiko:** Mittel. Leser-only, rückwärtskompatibel. Heuristik in `open()` bleibt wirksam, damit `db` unverändert öffnet.

---

#### S-36j Formatversion 3: Schreiber (VEC-F-10)

**Ziel:** Neue Dateien werden als v3 geschrieben, mit `id_width_bits` und Header-Prüfsumme.

**Arbeit**
1. `HNSW_VERSION = 3`; `save_internal` und `HnswHeader::new*` schreiben v3; Offsets verschieben sich um 36 Byte (`nodes_offset = SIZE_V3`).
2. Rückwärts-Lesetest: v2-Golden → öffnen → Index weiter benutzbar → erneut speichern → neue Datei ist v3 und enthält dieselben Dokumente.
3. Breitentest und Kreuzmatrix mit **geschriebenen** Dateien (nicht nur synthetischen); `rollback`-Pfad und Rebuild-Swap (`core_rebuild/snapshot.rs`) schreiben ebenfalls v3.
4. Verbraucher: `contextra-db` und `contextra-engine` laufen mit v3-Dateien (`cargo test -p contextra-db`, `-p contextra-engine` — Ergebnis berichten, auch falls diese Crates nicht vollständig bauen, siehe `reports/FIX_REPORT_2026-11-16.md`).
5. Crash- und Recovery-Suite (`tests/corruption_fault_test.rs`, `wp03_persistence_corruption.rs`, `mmap_malformed_data_test.rs`, `proof_open_rejects_corrupt.rs`) unverändert grün (INV-6).

**Abnahme:** G-F1 mit v2- und v3-Golden; Beschreibung der Versionsanhebung im ADR (INV-1-Ausnahme, Schritt S-36 in SPEC §6.2 vorgesehen).
**Gates:** GF, GC, GL.
**Risiko:** Mittel bis hoch (Formatänderung). Rücknahme: Schreiber-Commit revertieren; Leser bleibt.

---

#### S-36k SIMD-Verdrahtung und Orakel-Differenztest

**Ziel:** SPEC §9.4: `simd` ersetzt `compute_distance` durch `contextra_simd::dispatch`, Skalarpfad bleibt Orakel.

**Arbeit**
1. Ist-Stand prüfen: `distance/dispatch.rs` enthält bereits `#[cfg(feature = "simd")]`-Pfade mit `contextra_simd::Dispatcher::auto()` [L]; was fehlt, ist der Differenztest.
2. Test `tests/diff_simd_distance.rs` (Namensmuster `diff_*`, §7.2): 10.000 Vektorpaare, Dimension 4, 17, 128, 768; normale, Null- und Denormal-Werte; Toleranz relativ ≤ 1e-5 (f32), bitgleich bei gleicher Skalarfunktion; NaN-/Inf-Verhalten gleich (SPEC §6.4). Bestehende Tests `simd_differential_test`, `simd_numerical_audit`, `proof_simd_dispatch_legacy`, `golden_distance_oracle` einbinden.
3. `Dispatcher::auto()` pro Aufruf: Kosten messen (Bench) und bei messbarem Aufwand einmal je Index cachen (Bericht).
4. Feature `simd` in G-L3 und G-L4 (`each-feature`).

**Abnahme:** Differenztest grün; `ranking_golden.json` und `distance_oracle.json` bitgleich.
**Gates:** GD, GL.
**Risiko:** Gering.

---

#### S-80a bis S-80c DiskANN herauslösen (D-09)

**Voraussetzung:** S-36c.

- **S-80a Kopplungsinventur und Gerüst.** Messung: `diskann/` nutzt `crate::types` (28 Stellen), `crate::error` (23), `crate::hnsw` (7), `crate::quantize` (3), `crate::distance` (3); außerhalb von `diskann/` und `compat/` verweist nur `lib.rs` auf `diskann` [V]. Das neue Crate `contextra-vector-diskann` (Ebene E2 nach D-V7) erhält: `diskann/*` (4.158 Zeilen einschließlich Tests), `compat/diskann_vector_index.rs` (190), die DiskANN-Tests (9 Targets, siehe S-36c), `benches/filtered_search_selectivity_bench.rs`, die Golden-Dateien `diskann.*`. Aus `vector` werden die benötigten Elemente (`ScalarQuantizer`, `VectorData` oder ein schmaler Ersatz, `distance::*`, `VectorError`, `DocId`/`TxId`) als öffentliche Kern-API ausgewiesen; jede neue `pub`-Fläche ist eine Zeile im Befundregister.
- **S-80b Verschieben (Commit A) und Umbauen (Commit B).** A: Dateien per `git mv`, Pfade in `Cargo.toml` und Tests. B: Importe auf `contextra_vector::…`, Feature `workspace` im neuen Crate für die Port-Brücke, `VectorError` des neuen Crates `DiskAnnError` mit `From<VectorError>`, wenn die SPEC-Regel R3 es verlangt (E2-Crates haben nach §4.1 `ContextraError` oder eigenen Fehler; Entscheidung im ADR).
- **S-80c Verbraucher und Aufräumen.** `contextra-db`: Feature `exp-diskann = ["contextra-vector-diskann"]` statt `contextra-vector/experimental-diskann` (§4.1). Re-Export `contextra_vector::{DiskAnnConfig, DiskAnnFallbackPolicy, DiskAnnIndex}` entfällt ersatzlos (R9: ein Re-Export würde einen Rückwärtspfeil erzeugen; Nutzer sind `db`-Tests); Hinweis im Änderungsprotokoll. Aus `vector`: Features `exp-diskann`, `diskann`, `experimental-diskann`, Dev-Abhängigkeiten, `[[test]]`-Blöcke, `[[bench]]`-Eintrag und `compat/mod.rs`-Zeile entfernen. Workspace-`members`, `capabilities.toml`, `capabilities.fragment.toml` beider Crates.

**Abnahme:** `cargo check --workspace --all-targets --locked`; `cargo tree -p contextra-vector` ohne DiskANN; die DiskANN-Tests laufen im neuen Crate; Golden-Hashes unverändert; G-L1 (E2 → E1 erlaubt, kein Zyklus).
**Gates:** GL, GF, G-A1 (beabsichtigtes Verschwinden der drei Re-Exports, im ADR vermerkt).
**Risiko:** Mittel. Die Ebenenentscheidung (D-V7) muss vor S-80a feststehen.

---

#### S-36l Abnahme (SPEC §5.4, §8.4)

**Ziel:** Der Entnehmbarkeitsnachweis.

**Arbeit**
1. `README.md` neu: Zweck, Feature-Tabelle (5.5), **ein** Einsatzbeispiel in höchstens 10 Zeilen, das nur `contextra-vector` braucht (Kern-API, synchron, ohne `tokio`):
   ```rust
   use contextra_vector::hnsw::{HnswConfigBuilder, HnswIndex};
   use contextra_vector::types::{DocId, TxId};
   let index = HnswIndex::try_new(HnswConfigBuilder::new(4).m(16).ef_construction(32).ef_search(16).build()?)?;
   index.stage_insert(TxId::new(1), DocId::new(1), &[1.0, 0.0, 0.0, 0.0])?;
   index.commit_tx(TxId::new(1))?;
   let hits = index.search(&[0.9, 0.1, 0.0, 0.0], 1)?;
   ```
   Der Text „Ring 0, keine internen Pflichtabhängigkeiten“ wird korrigiert (`ids`, `env`, `mvcc`).
2. `examples/standalone_vector.rs` auf die Kern-API; Doctest für das README-Beispiel.
3. `SPEC.md` und `PLAN.md` des Crates auf v6 heben (Ebene E1b, Features, Persistenz v3, Status je Schritt); `API.md` und `AGENTS.md` (generierter Block per `python3 tools/crate_inventory.py agents`) aktualisieren; `UNSAFE_TRANSITION.md` gegen `unsafe-inventory` prüfen.
4. `crates/contextra-vector/docs/decoupling/REPORT.md` mit HEAD-Hash und Datum: G-L2, G-L1, G-L3, G-L4, G-H1 (Exit 0), Score M = 100 oder begründete Abzüge (`tools/modularity_score.py`), README-Beispiel kompiliert, Golden-Korpus und Breitentest grün, Bausteinvertrag vollständig, `cargo package -p contextra-vector --locked` als SOLL-Probe.
5. `capabilities.fragment.toml`: `extractable = true`, `verified_at = <HEAD>`; `dev_allowed = []`; `transitional` leer, sobald S-70 durch ist.
6. Kapitel-8-Anbindungen (siehe dort) als eigene Folgeschritte S-36m bis S-36o im Befundregister eröffnen, falls nicht in diesem Zug umgesetzt.

**Abnahme:** Alle Punkte aus SPEC §8.4 belegt.
**Gates:** G-L2, G-L1, G-L3, G-L4, G-H1, G-F1, G-D1, G-E1, G-DOC1.

---

## 7 Teststrategie und Gates

### 7.1 Teststufen

| Stufe | Inhalt | Zeitpunkt |
| --- | --- | --- |
| Baseline | Testzählung je Target und Konfiguration (3.4) | vor und nach jedem Schritt |
| Differenz | Alt gegen neu, gleiche Eingaben (§6.4): Suche, Distanz/SIMD, `Clock`/`Rng`, `unix_ms` | vor jedem Umbau (Commit A) |
| Regression | `k = 0`, Tx-Reihenfolge, atomarer Commit, unveränderter Einstiegspunkt, Breitenmatrix | in S-36f, S-36i |
| Golden | `tests/golden/*` plus 128-Bit-Dateien; `sha256sum -c` | jeder Schritt, der Persistenz berührt |
| Eigenschaft | `proptest` für Header-Parser, Distanz, Quantizer, Löschen | bestehend plus neu |
| Nebenläufigkeit | Loom (`loom_quantizer_*`), TSan, Miri | nach S-36f und S-36j |
| Mutation | `mutation-testing.yml`, Baseline `mutants-guarantees.toml` | nach S-36f, S-36j |
| Leistung | `distance_bench`, `sq8_bench`, `hnsw_bench`, `hnsw_delete_search_bench` | S-36d, S-36f, S-36k |

### 7.2 Gate-Zuordnung

| Gate | Prüfung am Crate | Schritte |
| --- | --- | --- |
| G-L1 | Kante `vector → ids, env, mvcc`; optionale Kanten `ports`, `types`, `sys`, `simd` nur hinter Feature; keine dev-Kanten | S-36h, S-80c |
| G-L2 | 0 Referenzen im Kern (heute erfüllt [V]) | alle |
| G-L3 | Build in allen Konfigurationen und je Feature | S-36b, S-36c, S-36h |
| G-L4 | `isolation_proof.py --crate contextra-vector` (alle Modi) | S-36c, S-36h, S-36l |
| G-H1 | Features, dev-Kanten, `docid-128` | S-36h |
| G-F1 | Golden-Hashes, v2 und v3, 64 und 128 | S-36c, S-36i, S-36j |
| G-D1 | `diff_*`-Tests, Rangfolge | S-36f, S-36g, S-36k |
| G-E1 | Fehlerregister, Display-Tests | S-36e, S-36i |
| G-C1 | Loom, TSan, Mutation | S-36f, S-36j |
| G-A1 | `cargo public-api`: nur beabsichtigte Diffs | S-36e, S-36g, S-36h, S-80c |
| G-DOC1 | Verweise und Aussagen (INV-7) | S-36l |

### 7.3 Gate-Läufe in dieser Analyse [V]

| Gate | Lauf | Ergebnis |
| --- | --- | --- |
| G-L2 | `python3 tools/gate_bridge_null.py` | bestanden (28 Crates, 0 Treffer) |
| G-L4 | `python3 tools/isolation_proof.py --crate contextra-vector` | `no-default`, `default` ok; `each-feature` rot bei `experimental` |
| G-F1 | `sha256sum -c tests/golden/MANIFEST.sha256` | 9 von 9 ok |
| G-L1 | `python3 tools/gate_layering.py --all` | lief ohne eindeutiges Ergebnis für `vector`; meldete Feature-Baum-Fehler anderer Crates (z. B. `contextra`: Feature `license` fehlt). Das deckt sich mit F-02 der SPEC. |
| G-H1, G-D1, G-E1, G-A1, G-C1 | nicht ausgeführt | `governance/`, `justfile`, CI-Dateien fehlen im Klon |

---

## 8 Beitrag zu den Kapiteln 11 bis 14 (Klasse D, Ableitungen)

Ein Vektorindex ist nach SPEC §11.2 Klasse **D**: Embeddings sind Ableitungen personenbezogener Daten und müssen über Lineage bereinigt und neu aufgebaut werden können. Das Crate trägt dazu bei; folgende Anforderungen sind **Folgeschritte** und nicht Teil der Schritte S-36a bis S-36l. Sie sind hier festgehalten, damit die Kern-API aus 5.3 sie nicht verbaut.

| Anforderung | Quelle | Stand im Crate | Folgeschritt |
| --- | --- | --- | --- |
| Nach Rückkehr von L1 liefert die Suche keinen Treffer des Ziels (auch nicht aus `Superseded`-Versionen). Die Sperre wirkt vor jeder Ausgabe, in O(1). | §11.4.2, Z-E1 | Filter-Pfad `search_filtered` vorhanden [L]; ein Gate-Aufruf vor der Ausgabe ist nicht Teil des Kerns | S-36m: `search_*` nehmen einen `ReadGuard`-Parameter oder der Filter wird vom Aufrufer injiziert (Vertrag dokumentieren und per Test belegen) |
| Rangfolge, Zählwerte und Fehlerbilder ändern sich nicht durch unzugängliche Dokumente (Vor-Filterung, nicht Nach-Filterung mit abgeschnittenem `ef`). | Z-E8 | `acorn`-Filter und `overfetch_selectivity` existieren; Gleichheit der Rangfolge bei unzugänglichen Dokumenten ist nicht belegt | S-36n: Rangtest mit unzugänglichem Dokument; `stats()`/`len()` dürfen es nicht preisgeben |
| Physische Bereinigung eines Dokuments inklusive Datei: Zeroize im Speicher (INV-VEC-ZEROIZE-1, `deletion.rs`), danach Rebuild und atomares Neuschreiben der mmap-Datei, damit alte Vektorbytes nicht in der Datei verbleiben. | L2 | In-Memory-Zeroize vorhanden [L]; Datei-Rest nach Löschung nicht belegt | S-36o: Messverfahren `scan_residue(doc_id)` über Arena, Knoten, mmap-Datei (Positivkontrolle nach Z-E10) |
| Persistierte Vektoren liegen vor dem Schreiben unter AEAD (Z-E2). | Z-E2 | `contract::BlockTransform` ist als Seam definiert; Nutzung im Schreibpfad **nicht geprüft** | S-36o: Prüfen, ob `BlockTransform` im Schreib- und Lesepfad verdrahtet ist; sonst verdrahten |
| Logs und Fehlertexte ohne Roh-Kennungen (Z-E7). | Z-E7 | Fehlertexte enthalten teils interne Details; `DocId` in Logs nicht auditiert | in S-36e mit prüfen |

`compat/purge.rs` (241 Zeilen), `compat/scanner.rs` (340) und `compat/candidate_stream.rs` (186) wurden nicht gelesen; ihre Rolle für L2/L5 ist zu klären, bevor S-36m bis S-36o beschrieben werden.

---

## 9 Risiken und offene Entscheidungen

### 9.1 Risiken

| ID | Risiko | Gegenmaßnahme |
| --- | --- | --- |
| R-V1 | Test-Reparatur (S-36b) deckt echte Produktfehler auf, die bisher unsichtbar waren (zwei sind schon durch Probe-Programme bestätigt). | Befunde erfassen, nicht abschwächen; Regressionstests zuerst. |
| R-V2 | Lint-Sanierung (S-36d) verschlechtert Heißpfade. | Iteratoren statt Prüfungen in inneren Schleifen; Bench vor/nach; Grenze 3 %. |
| R-V3 | Umbau des Commit-Pfads (S-36f) ändert Semantik für `db`/`engine` (Reihenfolge der Operationen). | Differenztest; Abstimmung mit `engine_commit_crash_points`, `commit_crash_consistency`; semantische Änderung (VEC-F-04) im Änderungsprotokoll nennen. |
| R-V4 | Format v3 bricht Verbraucher, die Dateien öffnen. | Leser vor Schreiber; `open()` bleibt mit Heuristik; Golden v2 bleibt lesbar. |
| R-V5 | Verbraucher (`db`, `engine`) bauen laut Audit bereits nicht (`reports/AUDIT_VERIFIKATION_2026-11-16.md`); Schritte ohne Verbraucher-Nachweis sind blind. | In S-36a/S-36j Verbraucherbau festhalten, Abhängigkeiten der Reihenfolge nach SPEC §9.3. |
| R-V6 | Das Fehlen von CI und `governance/` im Klon verhindert kanonische Gate-Läufe. | In S-36a klären; bis dahin Skripte aus `tools/` plus Cargo-Befehle (Anhang A). |

### 9.2 Offene Entscheidungen (ADR nötig)

| ID | Frage | Empfehlung |
| --- | --- | --- |
| D-V3/D-V4 | Einheitliche Suchpipeline: Unterschied zwischen Port- und Vertragsvariante (Quantizer-Pfad) | Pipeline mit dem Quantizer-Pfad; Vertragsvariante als dokumentierter Sonderfall oder entfernen (D-V3) |
| D-V6 | `open()` mit Heuristik gegen §6.2-Wortlaut | wie in 5.1 |
| D-V7 | Ebene von `contextra-vector-diskann` | E2 (alternativ R13-Ausnahme) |
| D-V8 | Variantenform `Io` | `Arc<io::Error>`, sonst `IoSource` |
| D-V9 | Name `partial-index-rebuild` gegen X2 | Ausnahme dokumentieren |
| VEC-F-22 | Wurzelmanifest gelöscht | Absicht klären, ggf. wiederherstellen |

---

## Anhang A Reproduktion der Messungen

```bash
# Toolchain (Ubuntu 24.04, ohne rustup)
apt-get update && apt-get install -y --no-install-recommends rustc-1.89 cargo-1.89 rust-1.89-clippy
export PATH=/usr/lib/rust-1.89/bin:$PATH CARGO_TARGET_DIR=$HOME/target

git clone https://github.com/tfufuz1/contextra && cd contextra
# Wurzeldateien fehlen am HEAD 24487fa16:
for f in Cargo.toml capabilities.toml deny.toml; do git show HEAD~1:$f > $f; done

# Build-Matrix (3.3)
for f in "--no-default-features" "" "--features workspace" "--features simd" "--features mmap" \
         "--features docid-128" "--features experimental-diskann" "--features experimental-rabitq" \
         "--features partial-index-rebuild"; do
  cargo check -p contextra-vector --locked $f; done

# Targets (3.4)
cargo check -p contextra-vector --locked --all-targets --keep-going
cargo check -p contextra-vector --locked --all-targets --keep-going --features workspace
cargo test  -p contextra-vector --locked --lib --features workspace
cargo test  -p contextra-vector --locked --features workspace --test <name>

# Lints (3.5)
cargo clippy -p contextra-vector --locked --lib --no-default-features
cargo clippy -p contextra-vector --locked --lib --features workspace

# Gates (3.2, 3.6, 7.3)
python3 tools/gate_bridge_null.py
python3 tools/isolation_proof.py --crate contextra-vector
(cd crates/contextra-vector/tests/golden && sha256sum -c MANIFEST.sha256)
cargo tree -p contextra-vector -e normal --no-default-features
cargo tree -p contextra-vector -e normal --no-default-features --features workspace

# Probe-Programm zu VEC-F-03 und VEC-F-04 (Beispiel, `--features workspace`):
#   idx.insert(tx, 1, v1); idx.insert(tx, 2, v2); idx.delete(tx, 1); idx.commit(tx);
#   search(v1, 5) liefert [1, 2]   (SOLL: [2])
#   search(v1, 0) panict: "attempt to subtract with overflow", compat/hnsw_vector_index.rs:205
```

## Anhang B DiskANN: Fehler nach Korrektur der fehlenden Klammer [V]

Syntaxfehler: `diskann/persistence/loader.rs:134`, `return Err(VectorError::Index(format!(…)));` schließt nur zwei von drei geöffneten Klammern. Nach lokalem Testpatch (`)));`) verbleiben 21 Fehler:

| Fehler | Anzahl | Beispielorte |
| --- | --- | --- |
| `SearchCandidate` nicht im Gültigkeitsbereich (E0412/E0422) | 6 | `build/flush.rs:15, 63, 187, 196`, `build/mod.rs:69, 78` |
| `VectorData` nicht deklariert (E0433) | 4 | `persistence/*`, `build/*` |
| `Ordering` nicht deklariert (E0433) | 4 | `build/pending_wal.rs:214`, `persistence/delta.rs:11` |
| `contextra_sys` unaufgelöst (E0433) | 2 | `persistence/loader.rs`, `persistence/writer.rs` (Feature `mmap` fehlt) |
| `CachedNode` privat (E0603) | 1 | `hnsw/types/mod.rs` |
| `DiskAnnHeader`, `DiskAnnIndexInner`, `Arc` | 4 | `persistence/delta.rs`, `persistence/node.rs`, `build/tombstone_wal.rs` |

## Anhang C Nicht ausgeführte Targets mit `required-features`

`diskann_fault_injection_test`, `diskann_corruption_fallback_test`, `diskann_recall_test`, `mmap_toctou_test`, `quantize_persistence_audit`, `recall_audit` (alle `experimental-diskann`), `simd_differential_test` (`simd`), `hnsw_tests` (`workspace`, kompiliert). Ihr Grünstatus ist offen und wird in S-36c und S-36k erhoben.
