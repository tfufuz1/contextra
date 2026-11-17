# contextra-store — Implementierungsspezifikation

| | |
|---|---|
| **Dokument** | `crates/contextra-store/docs/IMPLEMENTIERUNGSSPEZIFIKATION.md` |
| **Stand** | Entwurf 1, 17. November 2026 |
| **Autor** | Principal Rust Engineer (Claude), im Auftrag von @tfufuz1 |
| **Bezug** | `docs/SPEC.md` (v6, 981 Zeilen) · Klon-HEAD `24487fa` (flach, 1 Commit) |
| **Geltung** | `crates/contextra-store` (Ebene E1b, Ring 1) |
| **Status** | Zur Freigabe. Entscheidungen D-ST-01 … D-ST-11 (Abschnitt 5) sind offen. |

---

## 0. Lesehinweise, Belegmarken, Grenzen der Analyse

### 0.1 Belegmarken

| Marke | Bedeutung |
|---|---|
| **[V]** | Am Klon gelesen, gegrept oder ausgeführt. Nicht kompiliert. |
| **[V\*]** | Aus dem Quelltext abgeleitet. Kompilier- oder Laufverhalten ist **nicht** ausgeführt worden. |
| **[R]** | Aus `reports/` (Läufe vom 16.11.2026) übernommen. Dort ausgeführt, hier **nicht** wiederholt. |
| **[E]** | Einschätzung des Autors (Aufwand, Risiko, Erwartung). |
| **[D]** | Entscheidungsvorschlag. Bindet erst nach Freigabe. |

### 0.2 Grenzen (bitte zuerst lesen)

1. **Nichts wurde gebaut oder getestet.** In dieser Sitzung wurden nur `git`, `grep`, `sha256sum` und zwei kleine Python-Skripte ausgeführt (Modulbaum-Auswertung, Testklassifikation). Alle Aussagen zu Kompilierbarkeit sind [V\*] oder [R].
2. **Der Klon ist ein Teil-Snapshot.** Es fehlen: Workspace-`Cargo.toml`, `governance/`, `.github/`, `.githooks/`, `justfile`, `docs/decoupling/`, `docs/decisions/` und `capabilities.toml` auf Wurzelebene. Im Wurzelverzeichnis sind nur 5 Dateien getrackt [V]. Folgen: Workspace-geerbte Felder (`workspace = true`) sind nicht auflösbar, `default-features` der Workspace-Abhängigkeiten sind nicht prüfbar, und die Gates aus SPEC §7 sind hier nicht ausführbar.
3. **Keine Historie.** Der Klon hat einen Commit. Der in der SPEC genannte HEAD `fcd387021` ist nicht enthalten. Der Klon ist gegenüber der SPEC-Bestandsaufnahme teils **weiter** (siehe 3.2).
4. **Die crate-lokalen Dokumente sind unzuverlässig.** `SPEC.md`, `PLAN.md`, `README.md`, `docs/decoupling/REPORT.md` und der generierte Block in `AGENTS.md` widersprechen dem Code (Belege in ST-F-13). Diese Spezifikation stützt sich auf den Code, nicht auf diese Dokumente.
5. **Die Berichte vom 16.11. [R]** nennen einen Branch `fix/test-targets`, der nicht im Klon liegt. Ob ihre Zahlen für `24487fa` gelten, ist offen. WP-ST-00 klärt das.
6. **Kapitel 11 der SPEC (Löschkette)** weist dem Crate `store` nur die Rolle „Ablageort von K- und D-Inhalten" zu. Die Pflichten in 6.7 sind daraus **abgeleitet** [D], nicht wörtlich vorgegeben.
7. **SPEC-Querverweise** (R2, R3, INV-x, G-x, D-xx, S-xx) beruhen auf meiner Lesung der SPEC. Vor Freigabe sollten sie stichprobenartig gegen `docs/SPEC.md` gegengeprüft werden.

---

## 1. Kurzbefund

### 1.1 Was bereits steht (Ist ist weiter als die SPEC annimmt)

- **Brückenreferenzen im Kern: 0.** Kein `contextra_types`, `contextra_ports` oder `contextra_testkit` in `src/` außerhalb von `src/compat/` [V]. Die SPEC nennt 28 Referenzen in 7 Dateien.
- **`StoreError`** existiert (`src/error.rs`), `#[non_exhaustive]`, mit `From<StoreError> for ContextraError` in `compat/error.rs` und einem tabellengetriebenen Test [V]. Die Display-Strings stimmen mit `ContextraError` überein [V].
- **`compat/`** enthält `engine`, `scanner`, `license`, `error`. Die `StorageEngine`-Brücke delegiert (`.map_err(Into::into)`) [V].
- **Cargo-Kante:** `contextra-types` und `contextra-ports` sind optional hinter `workspace`; `docid-128` leitet an `contextra-ids` weiter [V].
- **Kern-Closure** aus `Cargo.toml` abgeleitet: `ids, env, sys, mvcc, crypto` = 5 [V\*]. `mvcc` hat `default = []`, `crypto` hat `default = ["kv-encryption"]` ohne Abhängigkeitsaktivierung. `cargo tree` ist nicht ausgeführt.
- **`unsafe`:** 0 Vorkommen, `forbid` aktiv [V].
- **`kv/`** ist nach `archive/contextra-store-kv` verschoben [V].
- `tenant_codec` nutzt `ids::{TenantId, CollectionId, DocId, TxId}` [V].

### 1.2 Was dem SOLL widerspricht (Top-Lücken)

| # | Lücke | Befund |
|---|---|---|
| 1 | Test- und Check-Build rot (Blocker) | ST-F-01 |
| 2 | 92 von 150 Integrationstests werden ohne `workspace`-Feature **stumm zu leeren Binaries**; 118 importieren `types/ports/testkit` | ST-F-02, ST-F-03 |
| 3 | Golden-Korpus nicht G-F1-konform: `sha256sum -c` schlägt 5 von 5 fehl, eine gelistete Datei fehlt | ST-F-04 |
| 4 | Manifest-Version bereits auf 2 gesetzt, ohne Inhalt, ohne Golden; ein Test erwartet noch 1 | ST-F-05 |
| 5 | `decode_chunk_key` unter `docid-128` voraussichtlich nicht typkorrekt und verlustbehaftet | ST-F-06 |
| 6 | Tote Dateien (365 Zeilen) und zwei parallele Datei-Nähte; keine `ports::Vfs`-Brücke | ST-F-07 |
| 7 | `StaticKeyMaterialSource` liefert bei Erschöpfung Ersatz-Schlüsselbytes | ST-F-09 |
| 8 | Löschnachweis T-DEL-1 nicht erbracht [R]; Löschfehler nur als Warnung | ST-F-16 |

### 1.3 Reihenfolge in einem Satz

Erst Nachweisbarkeit herstellen und den roten Build triagieren (Welle 0), dann Formate und Golden-Korpus absichern (Welle 1), dann Tests, Features und Fehlerabbildung entkoppeln (Welle 2), danach Nähte, Löschkette und Crash-Nachweise (Welle 3) und zuletzt Dokumentation und Entnehmbarkeit (Welle 4).

---

## 2. SOLL-Zustand (aus SPEC v6 für `store` abgeleitet)

| ID | Anforderung | Quelle (SPEC) | Prüfverfahren |
|---|---|---|---|
| A-01 | Ebene E1b. Kern-Abhängigkeiten direkt: `ids, env, sys, mvcc, crypto`. Kern-Closure = 5 interne Crates. | §4.1, D-03 | G-L1, `cargo tree --no-default-features` |
| A-02 | `default` ist leer. `workspace` aktiviert genau `types` + `ports`. Kein Feature ohne `cfg` oder Weiterleitung. | R10, §4.1 | `r10_features`-Test (neu), `capabilities-consistency` |
| A-03 | `types`/`ports`/`testkit` nur unter `src/compat/` hinter `workspace`. Gilt auch für Kommentare, `#[cfg(test)]` und Doctests. | R2, G-L2 | G-L2, `r2_bridge_null`-Test (neu) |
| A-04 | `StoreError` (`thiserror`, `#[non_exhaustive]`). `From<…> for ContextraError` nur in `compat`. Display bitgleich. Keine Sammelvariante, die Kontext verwirft. Tabelle unter `docs/decoupling/errors/store.md`. | R3, §6.3, G-E1 | Tabellentest, G-E1 |
| A-05 | Ein Besitzer je Konzept: IDs → `ids`; `Clock`/`Rng`/`IdGen`/`MetricsSink` → `env`; Lizenz-Typen → `license` (nach S-50). | R4, §5.1 | Duplikat-Scan |
| A-06 | Dev-Kanten nur auf E0 und externe Crates. Tests mit `types/ports/testkit` laufen unter `required-features = ["workspace"]` oder liegen in `testkit/tests/store/`. | R12, D-12, S-12b | G-L1 (dev), Testzahl vorher = nachher |
| A-07 | Persistierte Formate bleiben byte-identisch (Manifest `MFMN`, SSTable `MFSX`/Legacy `MFST`, WAL mit HMAC-Kette, Rollback-Intent). Golden-Korpus mit `MANIFEST.sha256`, prüfbar per `sha256sum -c`. | INV-1, §6.1, G-F1 | G-F1 |
| A-08 | ID-Breite: Ein Test entscheidet je Format, ob ein Breitenfeld nötig ist. Bei Bedarf Manifest 1→2 mit `id_width_bits`, Fehler `IdWidthMismatch`, Datei nach Ablehnung unverändert. | §6.2 | Breitentest (neu), CI-Matrix |
| A-09 | Durability-Pfade (WAL, Manifest, Flush, Recovery) unverändert bis auf Typimporte. Crash-/Recovery-Suite unverändert grün. | INV-6 | Crash-Suite, Loom |
| A-10 | `forbid(unsafe_code)`; keine neuen `unwrap`/`expect`/`panic` in Produktionspfaden; Determinismus (keine Wanduhr/Zufall außerhalb `env`). | INV-5, INV-11, INV-12 | clippy, `determinism_guard` |
| A-11 | Bausteinvertrag: `README` mit ≤ 10-Zeilen-Beispiel ohne `types/ports`; mindestens ein Kern-`example`; `SPEC.md`; `capabilities.fragment.toml` mit `sublevel`, `core_deps`, `extractable`. | §5.4, D-17 | Capability-Gate |
| A-12 | Entnehmbarkeit: Reihenfolge G-L2 → G-L1 → G-L3 → G-L4 → G-H1, dokumentiert in `docs/decoupling/REPORT.md`. | §8.4 | Nachweislauf |
| A-13 | Löschkette (abgeleitet [D]): physische Bereinigung belegbar; Schlüsselpfad fail-closed; keine Roh-Kennungen in Logs/Fehlern; Crash-Matrix. | §11 | siehe 6.7 |
| A-14 | Re-Exports verschobener Pfade bleiben gültig (`#[deprecated]`), Entfernung erst in S-70. | INV-2, R9 | G-A1 |

---

## 3. IST-Zustand (verifiziert am Klon)

### 3.1 Kennzahlen

| Messgröße | Wert | Marke |
|---|---|---|
| Quelldateien `src/` | 161 (158 im Modulbaum erreichbar, **3 tot**) | [V] |
| Zeilen `src/` | 25 508 (laut generiertem `AGENTS.md`-Block) | [V] |
| Integrationstests `tests/*.rs` | 150 Dateien | [V] |
| Testfunktionen (Grep `#[test]`/`#[tokio::test]`) | 265 in `src/`, 386 in `tests/` | [V] |
| `proptest!`-Blöcke / `#[ignore]` | 15 / 12 | [V] |
| Features | 10: `default, workspace, test-hooks, inspect, memory-only-storage, fault-injection, block-cache-v2, sieve-cache, docid-128, legacy-wal-key` | [V] |
| Features ohne jede `cfg`-Verwendung | `inspect`, `sieve-cache` | [V] |
| Undeklarierte `cfg(feature)`-Namen | 0 | [V] |
| Interne Reverse-Abhängigkeiten | `agent`, `db`, `engine`, `bench` | [V] |
| Verbraucher nutzen `inspect`/`sieve-cache`/`store::traits`/`store::io_seam` | nein (im Repo) | [V] |
| Lib-Tests laut Bericht | 284 bestanden, 16 fehlgeschlagen | [R] |
| Nicht kompilierende Test-Targets | 19 (≈ 60 Fehler); `cargo check --all-targets`: 69 Fehler im Store | [R] |

### 3.2 Abgleich SPEC v6 ↔ Klon

| SPEC v6 sagt (§9.2 S-37, §4.1, §6.2) | Klon | Wirkung |
|---|---|---|
| 28 ungegatete Brückenreferenzen in 7 Dateien (`scanner/*`, `engine/memory_only.rs`) | 0 außerhalb `compat/`; `scanner/mod.rs` hat 3 Zeilen | S-37-Kern ist umgesetzt. Neuerhebung statt Übernahme (WP-ST-00). |
| `types` und `ports` nicht optional, Closure 7 | Beide optional, Closure 5 [V\*] | F-07 für `store` aufgelöst, sofern Workspace-Defaults passen (nicht prüfbar). |
| `Features (Ziel)`: nur `workspace` | 10 Features | Spec/Code-Konflikt → D-ST-03. |
| Manifest 1→2 mit `id_width_bits` „vorgesehen" | `CURRENT_MANIFEST_VERSION = 2`, **kein** Breitenfeld, kein Golden v2 | Version angehoben ohne Inhalt → ST-F-05. |
| `FeatureRing` entfällt nach S-50 | Weiter in `compat/license.rs` | Wartet auf S-50 (WP-ST-10). |
| Golden-Korpus mit `MANIFEST.sha256` + `sha256sum -c` | Digests sind BLAKE3 | ST-F-04. |

### 3.3 Testlandschaft nach tatsächlichem Abhängigkeitsbedarf [V]

Klassifikation aller 150 Dateien in `tests/` durch Skript (Importe von `contextra_types`, `contextra_ports`, `contextra_testkit`):

| Klasse | Dateien | Bedeutung | Maßnahme |
|---|---|---|---|
| **K0** | 32 | Kein `types/ports/testkit` | Bleibt, läuft im Kern-Build |
| **K1** | 18 | Nur `TxId`/`DocId`/`TenantId`/`CollectionId` aus `types` (Re-Exports von `ids`) | Import auf `contextra_ids` umstellen, Gate entfällt |
| **K2** | 49 | Zusätzlich nur Trait `StorageEngine` (für Methodensyntax) | Trait-Import entfernen, inhärente Methoden nutzen (Vorbehalt unten) |
| **K3** | 30 | Nutzen `types::ContextraError`/`Result` | Auf `StoreError` umstellen |
| **K4** | 12 | `ports`-Sondertypen (`ResourceBudget`, `ResourceTracker`, `StorageStats`, `MetricsSink`, `Clock`, `RequestContext`) | Einzelfall (lokale Typen / `env` / `workspace`-Gate) |
| **K5** | 9 | `contextra_testkit` | `testkit/tests/store/` oder `workspace`-Gate (D-ST-04) |

Weitere Messwerte: 92 Dateien tragen `#![cfg(feature = "workspace")]` file-weit [V]; `Cargo.toml` hat 22 `[[test]]`-Einträge: 15 verlangen `test-hooks` (davon 4 zusätzlich `fault-injection`), 6 `legacy-wal-key`, 1 ohne `required-features` [V]; 11 Dateien tragen `#![cfg(feature = "fault-injection")]` file-weit. Von den 58 nicht-`workspace`-gegateten Dateien sind 32 K0; die übrigen 26 importieren `types/ports/testkit` und brauchen `workspace` implizit (siehe ST-F-02).

**Vorbehalt K2:** Die inhärente API (`put`, `get`, `delete`, `commit`, `rollback`, `scan`, `scan_prefix`, `flush`, `close`) liegt in `lsm/engine/facade.rs` [V]. Die getrackten Lesepfade (`get_tracked`, `get_at_seq_tracked`, `scan_prefix_tracked`) ruft die Brücke über `crate::lsm::ops::read::*` auf; ob sie inhärent öffentlich sind, ist nicht geklärt [E]. Tests, die sie nutzen (z. B. `ssi_tracked_reads_via_trait.rs`), bleiben bis zur Klärung im `workspace`-Gate.

---

## 4. Befundregister

Schwere: **Blocker** (verhindert Nachweis), **Hoch**, **Mittel**, **Niedrig**.

| ID | Befund | Beleg | Schwere | SOLL | WP |
|---|---|---|---|---|---|
| **ST-F-01** | Test-/Check-Build rot. 69 Fehler (`--all-targets`), 19 Test-Targets nicht kompilierbar (u. a. `chaos_bitflip_sstable` 13, `compaction_merge_operator` 11), 16 Lib-Tests rot (u. a. `test_retain_versions_unpinned_floor_max`, `test_delete_memory_capacity_exceeded`, `test_compaction_backpressure_timeout_exceeded`), Stresstest `compaction_stress_and_gc` läuft in Timeout. Ursachen laut Bericht nicht untersucht. | [R] `reports/AUDIT_*`, `FIX_REPORT_*`, `summary.tsv` | Blocker | INV-4, A-06 | 00, 01 |
| **ST-F-02** | Stilles Leeren: 92 Tests file-weit `#![cfg(feature = "workspace")]`. Ohne Feature laufen im Standardlauf nur die 32 K0-Dateien (die übrigen 26 nicht-`workspace`-gegateten Dateien hängen an `test-hooks`/`legacy-wal-key`/`fault-injection`). `golden_corpus.rs` (G-F1) und `format_version_compat.rs` gehören zu den stumm geleerten 92. Weitere **26 Dateien** importieren `types/ports/testkit`, tragen aber nur `required-features = ["test-hooks"|"legacy-wal-key"|…]` bzw. `#![cfg(feature = "fault-injection")]` — **ohne `workspace`**. Sie kompilieren daher nur, wenn `workspace` zusätzlich gesetzt ist (11 davon rein per `#![cfg(feature = "fault-injection")]`, z. B. `group_commit_test.rs`, `disk_full_during_write.rs`) [V\*]. Das ist ein R2/R12-Verstoß, aber kein stilles Leeren. | [V], [V\*] | Hoch | R2, R12, G-L4 | 05 |
| **ST-F-03** | Dev-Kante `contextra-testkit`; 9 Testdateien importieren sie (K5). | [V] `Cargo.toml`, Klassifikation | Hoch | R12, D-12 | 05 |
| **ST-F-04** | G-F1 nicht erfüllbar: `MANIFEST.sha256` enthält BLAKE3-Digests; `sha256sum -c` meldet **5 von 5 FAILED** und eine gelistete Datei (`wal_with_hmac.log`) fehlt. `tenant_codec_vectors.json` ist nicht gelistet. WAL-Golden fehlen im Korpus. Test ist `workspace`-gated. | [V] ausgeführt: `sha256sum -c` | Hoch | INV-1, A-07 | 02 |
| **ST-F-05** | Manifest-Header-Version 2 wird bereits geschrieben (Konstante `CURRENT_MANIFEST_VERSION = 2`), Leser akzeptiert ≤ 2, Layout sonst identisch, **kein** `id_width_bits`, kein Golden v2. `tests/format_version_compat.rs:73` assertiert `CURRENT_MANIFEST_VERSION == 1` (Datei `workspace`-gated, daher stumm). `proof_format_constants.rs` akzeptiert 1 oder 2, `mz_c_manifest_byte_identity.rs` vergleicht gegen die Konstante. Eine Versionsanhebung ohne Inhalt ändert persistierte Bytes ohne Nutzen und erschwert Downgrades. | [V] | Hoch | INV-1, A-07/08 | 04 |
| **ST-F-06** | `decode_chunk_key` (`tenant_codec.rs`) parst die Dokument-ID als `u64` und baut `DocId(doc_raw)`. Unter `docid-128` ist `DocId(pub u128)`. Erwartung: Typfehler E0308 [V\*]; selbst mit Cast gingen IDs > `u64::MAX` beim Dekodieren verloren (Injektivität verletzt). `tests/docid_128_kv_engine.rs` prüft nur BLAKE3/Größen, nicht den Store. | [V\*] | Hoch | A-08, INV-4 | 03 |
| **ST-F-07** | `src/io_seam/{real_fs,store_io,types}.rs` (365 Zeilen) sind im Modulbaum nicht erreichbar; `io_seam.rs` enthält die Typen inline (u. a. `impl Vfs for RealFs` doppelt). `traits.rs` definiert `StoreFs`/`StoreFile`/`StoreFileOpenOptions` ohne Implementierer und Verwender. Damit existieren zwei Datei-Nähte (`io_seam::Vfs` und `traits::StoreFs`). In `compat/` gibt es keinen Adapter `ports::Vfs` ⇄ `io_seam::Vfs`; `FaultVfs` aus `testkit` kann den Store so nicht treiben. | [V] Modulbaum-Skript, Grep | Mittel | R4, R9, §9.4 | 08 |
| **ST-F-08** | `lsm/validate.rs::derive_doc_id(&[u8])` dupliziert die BLAKE3-Präfix-Ableitung von `ids::DocId::from_key(&str)` (sechste Kopie; die SPEC nennt fünf). Nicht austauschbar: Eingabe sind Bytes, `from_key` weist leere Schlüssel ab und nimmt `&str`. | [V] | Mittel | R4, D-20-Linie | 07 |
| **ST-F-09** | `StaticKeyMaterialSource` ist `pub` (re-exportiert), nicht test-gated. Ist der Vorrat erschöpft, füllt sie `dest` stillschweigend mit `(i as u8) + 0x42`. Ein fehlerfreier, vorhersagbarer Ersatzschlüssel widerspricht „kein Fallback-Schlüssel / fail-closed" (§11.4/§11.5). `OsKeyMaterialSource` ist korrekt fail-closed (`try_fill_bytes`). | [V] `key_source.rs` | Hoch | A-13 | 11 |
| **ST-F-10** | Fehlerabbildung: `From<CryptoError>` bildet `WalTruncationDetected` auf Nullen-HMACs ab (Display zeigt zwei 32-Byte-Nullarrays). Auffangzweige `other => …` kollabieren Fremdvarianten. `docs/decoupling/ERRORS.md` nennt für `WalTruncationDetected` und `CrossDeviceLink` verkürzte Display-Strings, die nicht zum Code passen. Zielort `docs/decoupling/errors/store.md` fehlt. | [V] `error.rs`, `ERRORS.md` | Mittel | R3, A-04 | 09 |
| **ST-F-11** | Irreführender Alias `StoreError as ContextraError` in `sstable/bloom.rs` (und in Tests). Verschleiert Typherkunft für Leser und das G-E1-Werkzeug. | [V] | Niedrig | R3 | 09 |
| **ST-F-12** | Feature-Hygiene: `inspect` und `sieve-cache` ohne `cfg` und ohne Weiterleitung. `#![allow(unexpected_cfgs)]` in `lib.rs` würde Tippfehler verdecken (aktuell keine undeklarierten Namen). `§4.1` führt nur `workspace`, der Code 10 Features. Inline-Versionen statt Workspace-Pins (`quick_cache`, `lru`, `uuid`, `tokio-util`, dev: `libc`, `criterion`, `proptest`, `loom`, `futures`, `redb`) — Wurzelmanifest nicht einsehbar. | [V] | Mittel | R10 | 06 |
| **ST-F-13** | Dokumentation widerspricht dem Code: `README` nennt Feature `wal-integrity` (das Test `no_encryption_feature.rs` verbietet), Beispiel nutzt `contextra_ports::StorageEngine`/`contextra_types::TxId`, listet `types`/`ports` als harte Abhängigkeiten. Crate-`SPEC.md` (v2, HEAD `f0eda1cb9`), `PLAN.md` (alle Schritte offen), `docs/decoupling/REPORT.md` (238 Brückenreferenzen; Ist 0), `AGENTS.md`-Inventar (7 normale Deps). `capabilities.fragment.toml` ohne `sublevel`, `core_deps`, `extractable`. | [V] | Mittel | A-11 | 15 |
| **ST-F-14** | Verschobene Pfade (`engine.rs`, `scanner/mod.rs`, `tenant_codec.rs` `pub use compat::…::*`) sind nur per Kommentar `MIGRATION(S-70)` markiert, nicht per `#[deprecated]`. Glob-Re-Exports können nicht deprecated werden. | [V] | Niedrig | INV-2, R9 | 17 |
| **ST-F-15** | `FeatureRing`/`validate_against_features` bleibt in `compat/license.rs` (R2-konform), bis S-50 greift. | [V] | Niedrig | §9.4 | 10 |
| **ST-F-16** | Löschnachweis: T-DEL-1 schlägt fehl, alte SSTable mit Canary bleibt nach Löschen+Flush liegen; `compact_all`/`compact_range` sind über die Fassade nicht erreichbar; ob das Manifest die alte Datei noch referenziert, ist ungeprüft [R]. Im Code melden Fehler beim Entfernen toter SSTables/WAL-Segmente nur `tracing::warn!`/`debug!` (`compaction/engine.rs:419`, `lsm/ops/compaction.rs:221`, `lsm/recovery/startup.rs:348,541`) — Restbestände werden nicht gemeldet [V]. | [R], [V] | Hoch | A-13 | 12 |
| **ST-F-17** | Lock-Lebenszyklus: `LsmStorage::drop` wartet nicht auf die Compaction-Task, die den Manifest-Lock hält → „Datenverzeichnis bereits in Benutzung" beim Wiederöffnen (Agent-Test). Laut Bericht plausibel, nicht nachgeprüft. | [R] | Mittel | INV-6 | 01 |
| **ST-F-18** | Determinismus-Reste: `tokio::time::Instant::now` (`wal/flusher.rs`), `HashMap`/`HashSet` (`lsm/engine.rs`, `lsm/recovery/startup.rs`), `AHashMap` (`sieve.rs`); `uuid`/`rand` für Dateinamen und Salt. Ob Iterationsreihenfolge sichtbar wird, ist ungeprüft. | [V] Grep | Niedrig | INV-5 | 06 |
| **ST-F-19** | ADR-N12 (Manifest-HWM) und ADR-N13 (WAL-Frame V4) sind „Pending Implementation", Spezifikationstests `#[ignore]`. Beide ändern persistierte Formate und kollidieren mit INV-1 und der Manifest-Versionsfrage. | [V] `docs/decisions/` | Hoch (Planung) | INV-1 | 04, 17 |

---

## 5. Offene Entscheidungen

| ID | Frage | Optionen | Empfehlung [D] | Blockiert |
|---|---|---|---|---|
| **D-ST-01** | Manifest-Version (ST-F-05) | (A) Schreiber auf Header-Version 1 zurück, Leser akzeptiert weiter ≤ 2. (B) Version 2 behalten und inhaltlich definieren (`id_width_bits` und/oder HWM aus ADR-N12), mit Golden v1 + v2. | **A**, solange der Breitentest (WP-ST-03) breitenunabhängig ausfällt. **B** nur als **eine** gebündelte Anhebung zusammen mit ADR-N12. Begründung: INV-1; eine Anhebung ohne Inhalt ist nicht rückbaubar für ältere Leser. | WP-ST-04 |
| **D-ST-02** | Braucht ein Store-Format ein Breitenfeld? | Test entscheidet (§6.2). Erwartung [E]: `tenant_codec` schreibt IDs als Dezimal-ASCII, `TxId`/Sequenz sind unbedingt `u64` → Bytes für IDs ≤ `u64::MAX` breitenunabhängig. | Kein `id_width_bits` im Store, wenn der Test das bestätigt. Dekodierung außerhalb des 64-Bit-Bereichs liefert definiert `None`/Fehler. Die SPEC-Vorgabe „Manifest 1→2 vorgesehen" wird dann per ADR korrigiert. | WP-ST-03/04 |
| **D-ST-03** | Feature-Liste des Stores in §4.1 | (a) §4.1 auf die tatsächlichen Features erweitern. (b) Alles außer `workspace`/`docid-128` entfernen. | **a** mit Bereinigung: `inspect` und `sieve-cache` entfernen; `test-hooks`, `fault-injection`, `legacy-wal-key`, `block-cache-v2`, `memory-only-storage`, `docid-128` dokumentieren (je Zweck, nie in `default`). | WP-ST-06 |
| **D-ST-04** | Wohin mit K4/K5-Tests? | (a) `required-features = ["workspace"]`. (b) Nach `testkit/tests/store/`. (c) Auf lokale Fakes/`env` umschreiben. | K4: (c) wo `store` lokale Typen hat (`budget.rs`, `stats.rs`) bzw. `env`; sonst (a). K5: `ManualClock` → `env`; Rest (b) gemäß D-12. | WP-ST-05 |
| **D-ST-05** | Welche Datei-Naht bleibt? | (a) `io_seam::Vfs`, `traits.rs` und Orphans entfernen. (b) `traits::StoreFs` als Soll. | **a** — kleinster Eingriff, `io_seam` ist in Benutzung, fsync-Pfade bleiben unberührt (INV-6). Dazu Adapter in `compat/vfs.rs`. | WP-ST-08 |
| **D-ST-06** | `StaticKeyMaterialSource` | (a) Bei Erschöpfung `Err` zurückgeben und hinter `test-hooks` stellen. (b) Nur hinter `test-hooks`. | **a + b**. | WP-ST-11 |
| **D-ST-07** | Umfang der Bereinigungs-API (T-DEL-1) | (a) Store bietet `purge`-Primitive, Fassaden-Entscheidung liegt bei `engine`/`db`. (b) Store bleibt unverändert, Aussage in README anpassen. | **a**. Die Frage „Fassade ja/nein" bleibt Eigentümer-Entscheidung (Bericht §8.2). | WP-ST-12 |
| **D-ST-08** | Einordnung ADR-N12/N13 | Vor, während oder nach der Entkopplung. | **Nach** Welle 2. Sie sind Formatänderungen und gehören nicht in Umbauschritte (INV-1). | WP-ST-17 |
| **D-ST-09** | Umgang mit roten Tests | Fixen, ignorieren, löschen. | Nur **fixen oder mit Befund und Frist dokumentieren**. Kein `#[ignore]`/Löschen als Abkürzung. | WP-ST-01 |
| **D-ST-10** | `derive_doc_id(&[u8])` | (a) `DocId::from_key_bytes(&[u8])` in `ids` ergänzen, Store-Kopie entfernen. (b) Lokal lassen, Differenztest gegen `from_key` für gültige UTF-8-Schlüssel. | **b** zuerst (kein Eingriff in `ids`), (a) später mit S-10C. | WP-ST-07 |
| **D-ST-11** | Hash-Algorithmus des Golden-Manifests | SHA-256 (SPEC) oder BLAKE3 (Ist). | **SHA-256** (G-F1 nennt `sha256sum -c`); `sha2` als dev-Abhängigkeit. | WP-ST-02 |

---

## 6. Architekturvorgaben (Soll-Design)

### 6.1 `Cargo.toml` (Soll-Fassung, normativ für Features und Kanten)

```toml
[features]
default = []                                   # leer (R10)
workspace = ["dep:contextra-types", "dep:contextra-ports"]
docid-128 = ["contextra-ids/docid-128"]        # D-02
block-cache-v2 = ["dep:quick_cache"]
memory-only-storage = []
test-hooks = []                                # nie in default
fault-injection = []                           # nie in default
legacy-wal-key = []                            # nie in default
# entfernt: inspect, sieve-cache (D-ST-03)

[dependencies]
# Kern (5): ids, env, sys, mvcc, crypto — jeweils mit default-features = false
#           im [workspace.dependencies]-Eintrag der Wurzel (dort zu prüfen)
contextra-types = { workspace = true, optional = true }
contextra-ports = { workspace = true, optional = true }

[dev-dependencies]
# contextra-testkit entfällt (R12)
contextra-env = { workspace = true, features = ["test-utils"] }
sha2 = "0.10"                                  # Golden-Manifest (D-ST-11); Workspace-Pin bevorzugen
```

Zusätzlich: Alle `[[test]]`-Einträge, die `workspace` brauchen, erhalten `required-features = ["workspace"]` **statt** file-weitem `#![cfg(feature = "workspace")]`. Der Zustand „Datei kompiliert zu leerem Binary" ist verboten.

### 6.2 Modul- und Brückenlayout (Ziel)

```
src/
  lib.rs            # kein allow(unexpected_cfgs); Doku: Abhängigkeiten = Kern-Closure
  error.rs          # StoreError (R3)
  io_seam.rs        # einzige Datei-Naht (Vfs, VfsFile, StoreIo, RealFs inline)
  tenant_codec.rs   # ids::* ; decode breitenkorrekt
  key_source.rs     # fail-closed (D-ST-06)
  compat/           # EINZIGER Ort für types/ports/testkit (R2), hinter `workspace`
    engine/ …       # StorageEngine-Brücke (Delegation)
    scanner/ …
    error.rs        # From<StoreError> for ContextraError
    license.rs      # bis S-50
    vfs.rs          # NEU: ports::Vfs -> io_seam::Vfs Adapter
```

Entfallen: `traits.rs` (StoreFs-Naht), `io_seam/{real_fs,store_io,types}.rs` (tot). Vor dem Löschen Archiv in `archive/contextra-store-orphans/` bzw. Git-Tag (R9).

### 6.3 Datei-Naht und Brücke (D-ST-05)

- `io_seam::Vfs` bleibt die einzige Naht. `compat/vfs.rs` implementiert `io_seam::Vfs` für `PortsVfsAdapter<V: contextra_ports::Vfs>`.
- Fehlerabbildung im Adapter explizit (`ContextraError` → `StoreError`), da keine `From`-Richtung existiert [E]. Mapping in `docs/decoupling/errors/store.md` ergänzen.
- Zweck: `testkit::FaultVfs` kann Store-Dateizugriffe injizieren (Crash-Matrix, SPEC Kap. 14.2). Der Adapter ändert **keine** fsync-/Rename-Reihenfolge.

### 6.4 Fehlerabbildung (A-04)

- Soll-Tabelle `docs/decoupling/errors/store.md`: Spalten `StoreError`-Variante | `ContextraError`-Variante | Display | Test. Quelle ist `compat/error.rs`; Display-Strings aus dem Code übernehmen (nicht aus der alten `ERRORS.md`).
- `compat/error/tests.rs` auf **alle** Varianten von `StoreError` erweitern, plus `From<CryptoError>` und `From<MvccError>` für jede bekannte Fremdvariante.
- `WalTruncationDetected` aus `CryptoError` trägt keine HMACs. Das Nullen-Mapping wird per Test festgeschrieben und in der Tabelle als „diagnostisch leer" markiert. Echte HMACs entstehen nur an den Store-eigenen Wurfstellen.
- Alias `StoreError as ContextraError` entfernen (rein mechanisch, eigenes Commit-Paar).

### 6.5 Formate, Golden-Korpus, ID-Breite

**Golden-Korpus** (`tests/golden/`, crate-lokal, im Wurzelmanifest referenziert):

| Artefakt | Zustand | Soll |
|---|---|---|
| `manifest_v1.manifest` | vorhanden | bleibt |
| `manifest_v2.manifest` | fehlt | nur bei D-ST-01 = B |
| WAL (HMAC-Kette V3, ohne HMAC, abgeschnittener Schwanz) | fehlt (Eintrag `wal_with_hmac.log` verweist ins Leere) | erzeugen, deterministisch (`ManualClock`, fester Salt, `StaticKeyMaterialSource` unter `test-hooks`) |
| SSTable `MFSX` (V4) und Legacy `MFST` (V3), mehrblockig | prüfen, ggf. ergänzen | vollständig |
| Rollback-Intent (`MFRLBK`) | prüfen | ergänzen falls fehlend |
| `tenant_codec_vectors.json` | vorhanden, nicht gelistet | aufnehmen |
| `MANIFEST.sha256` | BLAKE3, 5/5 FAILED | SHA-256, `sha256sum -c` Exit 0 |

Regeln: Erzeuger-Tests sind `#[ignore]` (Aufruf über `regen.sh`), Prüf-Tests laufen immer und im Kern-Build. Ein Golden wird nie durch Neuerzeugen „repariert", sondern nur per ADR geändert (INV-1).

**ID-Breite (§6.2):** Der Breitentest (`tests/width_corpus.rs`) erzeugt Korpora in 64- und 128-Bit-Build aus identischen Eingaben (IDs ≤ `u64::MAX`) für Schlüsselkodierung, WAL, SSTable und Manifest und vergleicht SHA-256. Ergebnis entscheidet D-ST-02.

**`decode_chunk_key`-Korrektur (ST-F-06):** Rohtyp per `cfg` (`u64`/`u128`), Konstruktion über `DocId::new(raw)`, kein Tupelkonstruktor. Unter 64 Bit liefert ein Schlüssel mit ID > `u64::MAX` definiert `None`. Property-Test (Round-Trip) inklusive `u128::MAX` im 128-Bit-Build.

### 6.6 Test-Architektur

- **T1:** Kein Test unter `crates/contextra-store/tests/` importiert `contextra_types|ports|testkit`, außer sein `[[test]]`-Eintrag trägt `required-features = ["workspace"]`.
- **T2:** Umstellung nach Klassen K0–K5 (3.3), jeweils ein Commit-Paar je Klasse. Reihenfolge K1, K2, K3, K4, K5.
- **T3:** Wächtertests im Crate (Muster wie `no_encryption_feature.rs`):
  - `tests/r2_bridge_null.rs`: liest `src/**/*.rs` außer `compat/`, schlägt bei Treffer auf `contextra_(types|ports|testkit)` an.
  - `tests/r10_features.rs`: parst `Cargo.toml`, prüft `default = []`, keine Features ohne Nutzung, keine verbotenen Features.
- **T4:** Testzahl vorher = nachher. Vergleich über `cargo test -p contextra-store -- --list` (alle Feature-Kombinationen aus WP-ST-00), Diff als Gate. Grep-Basis: 265 + 386 [V]; die exakte Basis liefert WP-ST-00.
- **T5:** `cargo test -p contextra-store --no-default-features --no-run` muss grün sein (G-L4).

### 6.7 Löschketten-Berührungspunkte (abgeleitet [D], SPEC Kap. 11)

Der Store bleibt **klassen-agnostisch** (Byte-KV). Klassifikation (P/K/D) und Siegelung gehören in `engine`/`erasure`. Der Store schuldet:

1. **Physische Bereinigung belegbar machen (WP-ST-12).** Nach Löschen + Flush + Kompaktion enthält kein Datei im Datenverzeichnis die Wertbytes. Nicht mehr vom Manifest referenzierte SSTables und vollständig überholte WAL-Segmente werden entfernt; Fehler dabei werden **gemeldet** (Rückgabewert/`PurgeReport`), nicht nur geloggt. Der Report enthält Zähler, keine Schlüssel.
2. **Fail-closed-Schlüsselpfad (WP-ST-11).** Kein Fehler im Schlüssel-, Zufalls- oder Siegelpfad erzeugt Ersatzwerte. Insbesondere `KeyMaterialSource`, Salt, `load_or_create_integrity_key`.
3. **Keine Roh-Kennungen in Logs/Fehlern (WP-ST-14).** Stichprobe: Logzeilen enthalten nur Dateipfade; kein Schlüssel-/Wertinhalt gefunden [V]. Soll: Canary-Test über alle Fehlerpfade.
4. **Crash-Nachweise (WP-ST-13).** Vorhandene Suiten (`wal_lifecycle_crash_point_matrix`, `chaos_power_cut`, `crash_prefix_enumeration`, `wal_systematic_crash`) werden den Invarianten der Löschkette zugeordnet; Lücken (Monotonie von `TxId`/Sequenz über Rollback und Crash) werden ergänzt.
5. **Scan-Ansicht für die Messung (L5).** `compat/scanner` bleibt die Brücke; Inhalt und Vertrag bleiben unverändert (S-70-Aufräumung getrennt).

---

## 7. Umsetzungsplan

### 7.1 Arbeitsregeln

- **Commit-Paare (INV-3):** je Schritt zwei Commits `S-37.<n> A: …` (Test/Gate zuerst, muss rot-grün-fähig sein) und `S-37.<n> B: …` (Umsetzung). Das Teilnummern-Schema `S-37.<n>` ist in der SPEC nicht definiert → Commit-Lint-Regex anpassen oder ADR ergänzen.
- **Ein Schritt, ein Thema:** keine Format-, Feature- und Testumstellung im selben Commit.
- **Durability-Pfade (INV-6):** Änderungen an `wal/`, `manifest/`, `lsm/recovery/`, `fsync.rs` nur mit Crash-Suite und Loom-Lauf; kein Mischen mit Strukturumbau.
- **Rückfall:** jedes WP ist einzeln revertierbar; Rückbau = Revert des B-Commits, A-Commit (Test) bleibt.
- **Nachweis:** jedes WP schließt mit einem Eintrag in `docs/decoupling/REPORT.md` (HEAD-Hash, Gate-Ergebnisse).

### 7.2 Wellen

| Welle | Inhalt | Ziel |
|---|---|---|
| **0** | WP-ST-00, 01 | Nachweisbar und baubar |
| **1** | WP-ST-02, 03, 04, 11 | Formate gesichert, Schlüsselpfad fail-closed |
| **2** | WP-ST-05, 06, 07, 09 | R2/R10/R12 erfüllt, Fehlerabbildung vollständig |
| **3** | WP-ST-08, 12, 13, 14 | Nähte, Löschkette, Crash-/Log-Nachweise |
| **4** | WP-ST-10, 15, 16, 17 | Bausteinvertrag, Entnehmbarkeit, Aufräumen (S-70) |

### 7.3 Arbeitspakete

Aufwand [E]: S ≤ 1 PT, M 2–4 PT, L 5–10 PT (PT = Personentag, grobe Schätzung).

#### WP-ST-00 — Basislinie und Rebaufähigkeit (S-00, Store-Anteil) · M
- **Vorbedingung P-0:** Arbeitskopie mit vollständigem Workspace (Wurzel-`Cargo.toml`, `governance/`, `.github/`, `justfile`, `.githooks/`). Toolchain gemäß `rust-toolchain.toml`.
- **Aufgaben:** (1) `cargo check -p contextra-store --locked` in den Modi default, `--no-default-features`, `--features workspace`, `--features docid-128`, `--features workspace,docid-128`, `--all-features`, `--all-targets`. (2) `cargo tree -p contextra-store -e normal --no-default-features` → Closure. (3) `cargo test -p contextra-store --lib` und `--tests` (je mit/ohne `workspace`, `test-hooks`, `fault-injection`, `legacy-wal-key`). (4) `-- --list` als Testzahl-Basis je Modus. (5) Strenge Brückenzählung (außerhalb `compat/`). (6) Prüfung der Verbraucher auf `inspect`, `sieve-cache`, `store::traits`, `store::io_seam` (Repo-intern bereits leer [V]). (7) `BASELINE.md` und `AGENTS.md`-Block regenerieren.
- **Abnahme:** Bericht mit HEAD-Hash; jede Zahl aus Abschnitt 3 als bestätigt/abweichend markiert (Anhang A).
- **Risiko:** keine Codeänderung.

#### WP-ST-01 — Triage des roten Builds (Blocker) · L
- **Ziel:** `cargo check -p contextra-store --all-targets --locked` ohne Fehler; Lib-Tests grün oder mit Befund dokumentiert (INV-4).
- **Vorgehen:** (a) Fehler nach Ursache gruppieren: API-Drift der Tests vs. Produktcode. (b) Je roter Lib-Test entscheiden: Testfehler oder Produktfehler (`test_retain_versions_unpinned_floor_max` und `compaction`-Backpressure betreffen Kompaktions-Semantik, vermutlich mit MVCC-Untergrenze). Produktfehler: eigenes Commit-Paar mit Regressionstest. (c) `compaction_stress_and_gc`: Deadlock vs. Langsamkeit klären (Loom/TSan, Timeout-Instrumentierung). (d) ST-F-17: Reproduktionstest „öffnen → schließen → sofort wieder öffnen"; Shutdown wartet auf die Compaction-Task, bevor der Manifest-Lock fällt.
- **Abnahme:** keine Kompilierfehler; Liste roter Tests leer oder je Test mit Befund und Frist. **Verboten:** `#[ignore]`/Löschen als Abkürzung (D-ST-09).
- **Risiko:** Hoch bei Berührung von `lsm/ops/compaction.rs`, `compaction/`, `lsm/engine.rs` (Durability-Nähe).

#### WP-ST-02 — Golden-Korpus G-F1 (S-02, Store-Anteil) · M
- Abschnitt 6.5 umsetzen: SHA-256-Manifest, fehlende WAL/SSTable-/Rollback-Golden erzeugen, `tenant_codec_vectors.json` aufnehmen, tote Verweise entfernen.
- `golden_corpus.rs` aus dem `workspace`-Gate lösen (K1-Umstellung auf `ids`), damit G-F1 im Kern-Build läuft.
- **Abnahme:** `(cd crates/contextra-store/tests/golden && sha256sum -c MANIFEST.sha256)` Exit 0; Test grün mit `--no-default-features`.
- **Risiko:** Niedrig; Korpus wird nie aus aktuellem Code „nachgezogen", sondern aus dokumentierten Eingaben deterministisch erzeugt.

#### WP-ST-03 — ID-Breite und `docid-128` (S-10C / §6.2, Store-Anteil) · M
- Breitentest und `decode_chunk_key`-Korrektur gemäß 6.5. `docid_128_kv_engine.rs` durch einen Test ersetzen, der den Store mit maximalen `DocId`-Werten beider Breiten ausübt.
- CI: `cargo hack`-Matrix für `--features docid-128` (Check + Test).
- **Abnahme:** beide Builds kompilieren und bestehen; Ergebnis des Breitenvergleichs dokumentiert und D-ST-02 entschieden.
- **Risiko:** Mittel (Korrektur ändert Dekodierung für Randwerte; Golden schützt Bestandsformat).

#### WP-ST-04 — Manifest-Version (D-ST-01) · S–M
- **Variante A:** `CURRENT_MANIFEST_VERSION = 1` für neue Manifeste; Leser akzeptiert weiter ≤ 2 (bereits geschriebene v2-Dateien bleiben lesbar). Vorab prüfen, ob `Manifest::open` den Header einer bestehenden v2-Datei beim Anhängen unverändert lässt [E]. `format_version_compat.rs:73` wird wieder wahr; `mz_c_manifest_byte_identity` bleibt konsistent.
- **Variante B:** Version 2 inhaltlich definieren (Feldlayout, Golden v1 + v2, Leser zuerst, Schreiber danach, `IdWidthMismatch` nur bei Bedarf). Nur gebündelt mit ADR-N12.
- **Abnahme:** Golden v1 (und ggf. v2) lesbar; Header-Byte-Test grün; Test mit realistischer v2-Altdatei.
- **Risiko:** Mittel (Downgrade-Verhalten älterer Binaries bei vorhandenen v2-Dateien).

#### WP-ST-05 — Test-Neuordnung (S-12b) · L
- Regeln T1–T5 (6.6). Skriptgestützt: Import-Umschreibung K1/K2, Gate-Umstellung auf `required-features`, Diff der `--list`-Ausgabe.
- K3: `StoreError` statt `ContextraError`; vorhandene Aliase entfernen.
- K4/K5 gemäß D-ST-04. Danach `contextra-testkit` aus den dev-Abhängigkeiten entfernen.
- **Abnahme:** Testzahl vorher = nachher in allen Modi; `--no-default-features --no-run` grün; `r2_bridge_null` und `r10_features` grün.
- **Risiko:** Mittel. 150 Dateien, hohe Streuung. Gegenmaßnahme: ein Commit-Paar je Klasse, mechanische Umschreibung per Skript, Review nur der Abweichungen.

#### WP-ST-06 — Feature-/Manifest-Hygiene (S-12a, Store-Anteil) · M
- `inspect`, `sieve-cache` entfernen (repo-intern ungenutzt [V]; externe Nutzer unbekannt → Hinweis im Changelog).
- `#![allow(unexpected_cfgs)]` entfernen; `cfg(loom)` per `[lints.rust] unexpected_cfgs … check-cfg` deklarieren (Lint-Konfiguration liegt im Wurzelmanifest, dort anzupassen).
- Inline-Versionen auf Workspace-Pins umstellen (Wurzelmanifest-Änderung nötig).
- `default-features = false` für `mvcc`/`crypto` im Workspace-Eintrag prüfen; `crypto` hat `default = ["kv-encryption"]` (R10-Thema in `crypto`, Abhängigkeit zu S-12a/S-31).
- `capabilities.fragment.toml`: `sublevel = "E1b"`, `core_deps`, `extractable = false`, Feature-Liste mit Zweck.
- Determinismus-Reste (ST-F-18): je Fundstelle entscheiden (`env::Clock` statt `Instant`, `BTreeMap` oder dokumentiert reihenfolgeunabhängig).
- **Abnahme:** `r10_features` grün; Capability-Konsistenz-Gate grün.

#### WP-ST-07 — `ids`/`env`-Reste (S-10C / S-11, Store-Anteil) · S
- D-ST-10: Differenztest `derive_doc_id` ↔ `DocId::from_key` für gültige UTF-8-Schlüssel (64 und 128 Bit); Randfall leerer Schlüssel dokumentieren.
- Prüfen, dass keine lokalen Kopien von `Clock`/`Rng`/`MetricsSink`/`IdGen` existieren (aktuell keine [V]).
- Veralteten Kommentar in `traits.rs`/`lib.rs` entfernen.

#### WP-ST-08 — Datei-Naht (D-ST-05) · M
- (1) Orphans archivieren/löschen, (2) `traits.rs` entfernen, (3) `compat/vfs.rs`-Adapter, (4) Tests: `vfs_sstable_recovery_lifecycle`, Fault-Matrix über den Adapter.
- **Abnahme:** `src/` enthält genau eine Datei-Naht; Modulbaum-Skript meldet 0 unerreichbare Dateien; Crash-Suite unverändert grün.
- **Risiko:** Niedrig bis Mittel; fsync-/Rename-Reihenfolge bleibt unberührt.

#### WP-ST-09 — Fehlerabbildung (A-04) · S–M
- 6.4 umsetzen: Tabelle, vollständiger Test, Alias entfernen, `ERRORS.md` auf `docs/decoupling/errors/store.md` überführen.

#### WP-ST-10 — Lizenz-Naht (S-50-Anteil) · S · *blockiert durch S-50/S-24*
- Nach S-50: `compat/license.rs` und `FeatureRing`-Abhängigkeit entfernen; `validate_durability` bleibt; Kommentar `lsm/config.rs:153` aufräumen.

#### WP-ST-11 — Schlüsselpfad fail-closed (Z-E9) · S–M
- `StaticKeyMaterialSource`: Erschöpfung → `Err`; hinter `test-hooks` (D-ST-06). Alle Aufrufer von `fill_key_bytes`, Salt-Erzeugung und `load_or_create_integrity_key` auditieren.
- Fehlerinjektionstest: schlägt die Schlüsselquelle fehl, entsteht keine Schlüssel-/Salt-Datei und kein offener Store.
- **Abnahme:** Negativtests grün; kein öffentlicher Typ liefert Ersatzbytes.

#### WP-ST-12 — Physische Bereinigung (T-DEL-1) · L · *Design vorab freigeben*
- Gemäß D-ST-07 (a): Primitive für „obsolete Dateien entfernen" mit `PurgeReport` (Zähler: entfernt, fehlgeschlagen, verbleibend); `compact_all`/`compact_range` als dokumentierte, getestete Pfade.
- Verhalten: Entfernen unreferenzierter SSTables und überholter WAL-Segmente mit fsync des Verzeichnisses; Fehler im Report, nicht nur `tracing`.
- Zusammenspiel mit MVCC: gepinnte Snapshots halten alte Versionen; der Report weist „zurückgehalten durch Snapshot" gesondert aus.
- Store-Test `tests/deletion_physical.rs`: Canary schreiben, löschen, flushen, kompaktieren, Datenverzeichnis byte-weise durchsuchen (0 Treffer erwartet); zusätzlich Fall mit gepinntem Snapshot.
- **Risiko:** Hoch (Durability-Nähe, Nebenläufigkeit mit `FlushPurgeFence`). Eigene Loom-Szenarien, keine Vermischung mit Strukturumbau.

#### WP-ST-13 — Crash-Nachweise · M
- Matrix „Invariante → Test" in `docs/TRACEABILITY.md` ergänzen; Lücken (Monotonie von `TxId`/Sequenz über Rollback und Crash) mit Tests schließen.

#### WP-ST-14 — Log-/Fehlerhygiene (Z-E7) · S
- Canary-Test: Schlüssel/Werte mit Canary schreiben, Fehlerpfade provozieren, `tracing`-Ausgabe und alle `Display`-Strings durchsuchen.

#### WP-ST-15 — Dokumentation und Bausteinvertrag (A-11) · M
- `README` neu: ≤ 10-Zeilen-Beispiel mit inhärenter API und `contextra_ids::TxId`; Abhängigkeits- und Feature-Abschnitt korrigieren (kein `wal-integrity`); Entnahme-Rezept nur mit der Kern-Closure.
- Crate-`SPEC.md`, `PLAN.md`, `AGENTS.md`-Block, `docs/decoupling/{BASELINE,REPORT}.md` auf Ist bringen. `lib.rs`-Moduldoku „Ausgehende Abhängigkeiten" korrigieren.
- `examples/store_standalone.rs` und `chaos_writer.rs` (nutzen `types/ports`) hinter `required-features = ["workspace"]`; `examples/standalone.rs` (Kern) bleibt [V].

#### WP-ST-16 — Entnehmbarkeitsnachweis (A-12) · S–M
- Reihenfolge G-L2 → G-L1 → G-L3 → G-L4 → G-H1; `cargo package`-Probe; `capabilities.fragment.toml`: `extractable = true`, `verified_at = <HEAD>`.
- Nur mit vollständigem Workspace und Gate-Werkzeug ausführbar (P-0).

#### WP-ST-17 — Aufräumen (S-70-Anteil) und ADR-Einordnung · S–M
- Verschobene Pfade: benannte Re-Exports mit `#[deprecated]` statt Glob (wo Verbraucher existieren), Entfernung erst in S-70.
- ADR-N12/N13 (D-ST-08) als eigene Planung nach Welle 2 terminieren; `wal_spec_pending.rs` bleibt `#[ignore]` bis dahin.

### 7.4 Abhängigkeiten

```
WP-00 ──► WP-01 ──► WP-02 ──► WP-03 ──► WP-04
            │                              │
            ├──► WP-11 (unabhängig)        ▼
            └──► WP-05 ──► WP-06 ──► WP-07  WP-09
                   │
                   └──► WP-08 ──► WP-12 ──► WP-13 ──► WP-14
WP-10 (nach S-50)    WP-15 ──► WP-16 ──► WP-17
```

### 7.5 Vorbedingungen aus anderen Schritten

| Vorbedingung | Quelle | betrifft |
|---|---|---|
| Vollständiger Workspace und Gate-Werkzeug | P-0 / S-00, S-01 | alle Gates, WP-ST-16 |
| `crypto`: `kv-encryption` aus `default` | S-12a / S-31 | WP-ST-06 |
| `ids`: ggf. `from_key_bytes` | S-10C | WP-ST-07 (optional) |
| `license`-Crate | S-50 / S-24 | WP-ST-10 |
| `testkit/tests/store/` als Ziel | S-12b / D-12 | WP-ST-05 (K5) |

---

## 8. Gate- und Nachweismatrix

| WP | Gates / Nachweise |
|---|---|
| 00 | Basisbericht, `cargo tree`, Testzahlen |
| 01 | `cargo check --all-targets`, Lib-Tests, Loom/TSan für Compaction |
| 02 | G-F1 (`sha256sum -c`), Kern-Build-Lauf |
| 03 | `docid-128`-Matrix (Check + Test), Breitenvergleich |
| 04 | Golden v1 (v2), Header-Byte-Test, v2-Altdatei-Test |
| 05 | Testzahl-Diff, `--no-run` Kern, `r2_bridge_null`, G-L1 (dev) |
| 06 | `r10_features`, Capability-Konsistenz |
| 08 | Modulbaum 0 unerreichbar, Crash-Suite, Fault-Matrix über Adapter |
| 09 | Tabellentest alle Varianten, G-E1 |
| 11 | Negativ-/Fehlerinjektionstests |
| 12 | `deletion_physical`, Loom-Szenarien, Crash-Suite |
| 14 | Canary-Suche über Logs und Fehlertexte |
| 16 | G-L2 → G-L1 → G-L3 → G-L4 → G-H1 |

**Negativproben (Pflicht):** Für jedes neue Gate/jeden Wächtertest wird einmal bewusst ein Verstoß eingebaut (z. B. `use contextra_types` im Kern, Digest-Änderung im Golden, Feature ohne `cfg`) und das Rotwerden belegt, bevor der Verstoß entfernt wird.

---

## 9. Risiken

| Risiko | Wahrscheinlichkeit / Wirkung [E] | Gegenmaßnahme |
|---|---|---|
| Roter Build verdeckt echte Produktfehler (Kompaktion, Lock-Lebenszyklus) | Hoch / Hoch | WP-ST-01 vor allen Umbauten; kein Vermischen mit Strukturumbau |
| Formatdrift durch Manifest-Versionsanhebung | Mittel / Hoch | D-ST-01, Golden-Korpus zuerst (WP-ST-02) |
| Massenumschreibung von 150 Tests verändert getestetes Verhalten (trait- vs. inhärente Methoden) | Mittel / Mittel | Klassenweise Commit-Paare, Testzahl-Diff, K2-Vorbehalt prüfen |
| `docid-128`-Korrektur ändert Randverhalten | Niedrig / Mittel | Property-Tests, Golden |
| Durability-Regression durch WP-ST-12 | Mittel / Hoch | Design-Freigabe, Loom, Crash-Suite, eigenes Commit-Paar |
| Gates nicht ausführbar im Teil-Snapshot | Hoch / Mittel | P-0 vor Welle 0; bis dahin alle Gate-Aussagen als „nicht geprüft" führen |
| ADR-N12/N13 kollidieren mit INV-1 | Mittel / Hoch | D-ST-08, getrennte Planung nach Welle 2 |
| Externe Nutzer von `inspect`/`sieve-cache`/`store::traits` | Niedrig / Niedrig | Changelog-Hinweis, repo-intern ungenutzt [V] |

---

## 10. Definition of Done (Gesamtziel `contextra-store`)

Der SOLL-Zustand gilt als erreicht, wenn **alle** Punkte belegt sind:

1. `cargo check -p contextra-store --all-targets --locked` und die Lib-Tests sind grün; offene Befunde sind einzeln mit Frist dokumentiert (INV-4).
2. Kern-Closure = `ids, env, sys, mvcc, crypto` laut `cargo tree --no-default-features`; `default = []`.
3. Außerhalb von `src/compat/` null Referenzen auf `types/ports/testkit`, auch in Tests, Beispielen und Doctests; keine stillen Leer-Testbinaries; Testzahl unverändert.
4. `sha256sum -c MANIFEST.sha256` Exit 0; Golden-Korpus vollständig; Breitentest dokumentiert; `docid-128`-Build und -Tests grün.
5. Manifest-Version gemäß D-ST-01 entschieden und durch Golden und Header-Test gesichert.
6. Eine Datei-Naht; Adapter für `ports::Vfs` vorhanden; Modulbaum ohne unerreichbare Dateien.
7. Fehlerabbildung vollständig, Tabelle unter `docs/decoupling/errors/store.md`, Display bitgleich.
8. Schlüsselpfad fail-closed; physische Bereinigung belegt (T-DEL-1 auf Store-Ebene); Canary-Test für Logs/Fehler grün.
9. Crash-/Recovery-Suite unverändert grün; Loom-Szenarien für geänderte Nebenläufigkeit grün.
10. Bausteinvertrag erfüllt (README, Beispiel, `SPEC.md`, `capabilities.fragment.toml`), Entnehmbarkeit in `docs/decoupling/REPORT.md` belegt.

---

## Anhang A — Zu bestätigende Messwerte (WP-ST-00)

| Aussage | Marke | Bestätigung durch |
|---|---|---|
| Brückenreferenzen außerhalb `compat/` = 0 | [V] | Strenge Zählung |
| Kern-Closure = 5 | [V\*] | `cargo tree` |
| 69 Check-Fehler, 19 Test-Targets, 16 rote Lib-Tests | [R] | Läufe aus WP-ST-00 |
| `decode_chunk_key` unter `docid-128` nicht kompilierbar | [V\*] | `cargo check --features docid-128` |
| `sha256sum -c` 5/5 FAILED | [V] ausgeführt | Wiederholung |
| `format_version_compat.rs:73` rot (mit `workspace`) | [V\*] | `cargo test --features workspace` |
| 3 unerreichbare Dateien in `io_seam/` | [V] Skript | Wiederholung |
| 150 Testdateien: K0 32 / K1 18 / K2 49 / K3 30 / K4 12 / K5 9 | [V] Skript | Wiederholung |
| `get_tracked` & Co. nicht inhärent öffentlich | [E] | Quelltextprüfung `lsm/engine/facade.rs`, `lsm/ops/read.rs` |
| Manifest-`open` lässt v2-Header beim Anhängen unverändert | [E] | Test mit v2-Altdatei |

## Anhang B — Zuordnung SPEC-Schritte ↔ Arbeitspakete

| SPEC-Schritt | Arbeitspakete |
|---|---|
| S-00 / S-01 | WP-ST-00 |
| S-02 | WP-ST-02 |
| S-10C | WP-ST-03, 07 |
| S-11 | WP-ST-07 |
| S-12a | WP-ST-06 |
| S-12b | WP-ST-05 |
| S-31 | WP-ST-06 (crypto-Abhängigkeit) |
| S-37 | WP-ST-04, 08, 09, 10 |
| S-50 | WP-ST-10 |
| S-70 | WP-ST-17 |
| Kap. 11 (abgeleitet) | WP-ST-11, 12, 13, 14 |
| §8.4 | WP-ST-15, 16 |

## Anhang C — Rückfragen an die SPEC (Korrekturbedarf)

1. **§4.1 Features:** Zeile `store` nennt nur `workspace`; der Code (zu Recht) mehr. Liste präzisieren (D-ST-03).
2. **§6.2:** „Manifest 1→2 vorgesehen" widerspricht der Regel „Ein Test entscheidet". Für den Store ist ein breitenunabhängiges Ergebnis zu erwarten (D-ST-02).
3. **§9.2 S-37:** Bestandsaufnahme (28 Referenzen, Closure 7) ist am Klon überholt; Neuerhebung statt Übernahme.
4. **§6.1/G-F1:** Verlangt SHA-256; der Store nutzt BLAKE3. Entweder Store anpassen (empfohlen) oder die SPEC öffnen.
5. **Commit-Schema (INV-3):** Teilschritte `S-37.<n>` sind nicht definiert.
6. **Kap. 11:** Pflichten des Stores bei der physischen Bereinigung sind nicht ausdrücklich zugewiesen (Abschnitt 6.7 ist abgeleitet).
7. **Repo-Stand:** Der in der SPEC genannte HEAD ist im Klon nicht vorhanden; der Branch `fix/test-targets` ebenso.
