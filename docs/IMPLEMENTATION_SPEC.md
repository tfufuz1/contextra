# IMPLEMENTATION_SPEC.md — Umsetzung von `contextra-storage` (Bezug: SPEC R8)

| | |
|---|---|
| Status | **Umsetzungsplan, nicht normativ.** Normativ ist `docs/SPEC.md` (R8). Bei Widerspruch gilt die Spec (Spec §1.3). Dieses Dokument legt fest, **wie** die Spec gebaut wird: Modulplan, Typen, Abläufe, Schnittstellenverträge zu den anderen Crates, Arbeitspakete je Gate, Nachweise. |
| Stand | 22.11.2026, Klon `940ff86`, `rustc 1.97.0` (MSRV 1.89, Spec D-17) |
| Sprache | Prosa deutsch, Bezeichner englisch (CLAUDE.md §0) |
| Verifikation | Alle Zahlen in §2 sind an diesem Stand gemessen (Befehle in Anhang B). Alles, was nicht gemessen ist, steht als „zu prüfen“ oder „Vorschlag“. Leistungswerte sind Hypothesen (Spec §21). |
| Änderungen an der Spec | Dieses Dokument ändert `docs/SPEC.md` nicht. Wo die Spec lückenhaft oder widersprüchlich ist, steht ein **Änderungsantrag SA-n** (§11). Nach Spec §1.5 gilt: erst die Spec ändern (Freigabe durch den Maintainer bei §4, §5, §9–§11, §14–§16, §18; Goldens und Freeze-Liste bleiben Pflicht, SA-15), dann den Code. |

Kennungen dieses Dokuments: `WP-g.n` Arbeitspaket des Gates g · `IF-n` Schnittstellenvertrag · `UE-n` Umsetzungsentscheidung innerhalb des Spielraums der Spec · `SA-n` Änderungsantrag an die Spec · `OP-n` offener Parameter (Spec-Entwurfswert, in G-8 zu messen) · `R-n` Risiko.

---

## 1. Geltung und Leseanleitung

1. **Auftrag.** `contextra-storage` ersetzt `contextra-store` und wird das Fundament eines für Sprachmodelle gebauten Datenbanksystems. Bis Gate G-6 wird nur der Kern entwickelt (Spec §22, CLAUDE.md). Die aufsetzenden Crates werden danach auf den Kern umgestellt.
2. **Was dieses Dokument leistet.** (a) Ist-Aufnahme von Kern, Port-Katalog und Nachbarcrates (§2), (b) Zielbild der Crate-Landschaft (§3), (c) Modulplan des Kerns (§4), (d) Abläufe und Zustandsautomaten (§5), (e) Schnittstellenverträge zu **allen** Nachbarcrates, die die Spec nicht beschreibt (§6), (f) Migration der Konsumenten (§7), (g) Arbeitspakete je Gate (§8), (h) Teststrategie (§9), (i) Risiken (§10), (j) Änderungsanträge (§11).
3. **Rangfolge bei Konflikt:** `docs/SPEC.md` > dieses Dokument > `AGENTS.md` > `CLAUDE.md` > Code-Kommentare. Nummerierung und Namen der Spec bleiben unverändert (Spec §25, letzter Satz).
4. **Für Agenten:** Jedes Arbeitspaket (§8) ist so geschnitten, dass es allein aus Spec-Abschnitt, Bestandsdateien und Nachweis-Tests bearbeitbar ist. Reihenfolge der Gates ist bindend (Spec §22: keine Phase vor dem Gate der vorherigen). Innerhalb eines Gates sind Pakete ohne Abhängigkeitspfeil parallel bearbeitbar.

---

## 2. Ist-Aufnahme (gemessen 22.11.2026)

### 2.1 Bestand `crates/contextra-storage`

| Modul   | Zeilen (`src`) | Dateien | Befund |
| ------- | -------------- | ------- | ------ |
| `codec` | 1 013          | 9       | Varint, Tag, `InternalKey`, `KeyRange`, Bloom, CRC-32C, XXH64. Goldens A.1 bestehen (`golden_anhang_a`: 7 Tests grün). **Behalten**, gegen R8 prüfen. |
| `env`   | 1 055          | 8       | `RealFs`, `RealClock`, `RealEntropy`, `Runtime`, `Governor`, `Env`, Metrik-Adapter. Gegen §4 abzugleichen. |
| `keys`  | 683            | 6       | `KeyEpoch`, `KeysFile`, Hierarchie, Nonce-Protokoll. Arbeitet mit **Domänenschlüsseln** (`KeyDomain`: 16 Treffer in `src`, 7 in `tests`) und 12-Byte-Nonces (11 Treffer `src`, 3 `tests`). **Umbau** auf Clusterschlüssel (D-19) und 24-Byte-Nonces (SD-14). |
| `redo`  | 2 230          | 9       | Schreib- und Replay-Zustandsautomaten, Frame, Header. Der Segmentkopf ist 48 Byte fest mit `chain_seed: u32` und `prealloc_len` (`redo/header.rs`), die Spec verlangt `root_gen` und `chain_seed` von 4 oder 16 Byte (§10.4). **Layout neu ableiten** (G-3). |

**Build- und Teststand (gemessen 22.11.2026, `rustc 1.97.0`):** `cargo check -p contextra-storage --all-targets --features testing` endet mit Exit 0 (25 Zeilen `warning`, u. a. `unused_mut` in `tests/redo`). `cargo test -p contextra-storage --features testing --no-fail-fast`: 200 bestanden, 1 fehlgeschlagen, 2 ignoriert. Der eine Fehlschlag ist `invariants::test_all_invariants_cited`: der Parser in `tests/invariants.rs` sucht Zeilen mit dem Präfix `|INV-` (altes Tabellenformat), Spec R8 §18 schreibt `| INV-… |` und enthält 59 Invarianten; Ergebnis „gefunden: 0“. Das ist ein Formatfehler im Test, kein Verstoß gegen eine Invariante; Behebung in WP-1.9.

Gesamtumfang `crates/contextra-storage/src`: 17 525 Zeilen (`wc -l` über alle `*.rs`). Die Tabelle nennt nur die Module, die der Umbau am stärksten trifft; `run`, `blob`, `catalog`, `engine` und `testing` sind in §4.1 (Verschiebung) und §8 (Pakete) erfasst. `tests/` enthält u. a. `layering.rs`, `invariants.rs`, `golden_anhang_a.rs`, `fs_mem_test.rs`, `fs_real_test.rs` und die Verzeichnisse `loom/`, `redo/`, `run/`, `keys/`, `catalog/`, `engine/`, `format/`.

### 2.2 Port-Katalog `contextra-ports::persistence`

Das Modul ist **eine Datei**: `crates/contextra-ports/src/persistence.rs`, 352 Zeilen, eingebunden mit `pub mod persistence;` (`lib.rs:65`). Es importiert nur `std::io` und `std::path::Path` (kein `contextra-types`, `blake3`, `serde`, `futures-util`). Die Eigenständigkeit **im Quelltext** ist damit schon erreicht; offen ist sie **auf Crate-Ebene** (§2.2.2).

#### 2.2.1 Vorhandene Items gegen Spec §4

| Port / Typ | Bestand | Spec §4 | Lücke |
|---|---|---|---|
| `Fs` | `create_dir_all`, `open`, `rename`, `remove_file`, `remove_dir`, `list_dir`, `sync_dir`, `lock`, `caps`, **`hard_link`** (Pflichtmethode, Zeile 110) | wie Bestand, ohne `hard_link`; Checkpoints kennen `Copy`/`Reflink`/`Hardlink` (§8.6) | `hard_link`, `reflink` als optionale Methoden (SA-2) |
| `File` | `read_at`, `read_batch`, **`write_all_at`**, `preallocate`, `len`, **`set_len`**, `sync_data`, `sync_all` | `write_at`, `truncate`, zusätzlich `punch_hole` | Namen (SA-3); `punch_hole` fehlt |
| `Clock` | `now_ms`, `monotonic_ns` (`persistence.rs:161`) | wie Bestand | zweite, gleichnamige `Clock` in `contextra-ports/src/clock.rs:8` (D-02: Doppeldefinition entfällt) |
| `Entropy`, `Metrics`, `Hooks`, `PressureSource` | vorhanden | vorhanden | `HookPoint` hat 4 Werte (`AfterRedoSync`, `AfterRedoWrite`, `BeforeCatalogEdit`, `BeforeVisible`); Absturzmatrix (§19.1) braucht ggf. mehr (zu prüfen, WP-3.1) |
| `Cipher` | `seal(&[u8;32], &[u8;12], …) -> Vec<u8>`; `open -> io::Result<Vec<u8>>` | 24-Byte-Nonce, `Result<_, CryptoError>` | **12 → 24 Byte** (Breaking Change), `CryptoError`, `Key32` |
| `Mac`, `Kdf` | vorhanden (`Kdf::derive` liefert `[u8;32]`) | wie Bestand, mit `Key32` | nur Typalias `Key32` |
| `KeyProvider` | `kek`, `domain_key`, `domain_key_create`, `make_durable`, alles `io::Result` | `kek`, `cluster_key(c, epoch)`, `create_cluster_key(c, e) -> epoch`, `destroy`, `probe`, `make_durable`, `Result<_, KeyError>` | **umbauen** (D-19); `KeyDomain(pub u32)` → `ClusterId` (16 Byte) |
| `KeyError` | `NotFound`, `Unavailable` | Verwendung in allen `KeyProvider`-Methoden | Varianten für `destroy`/`probe` ergänzen (SA-12) |
| `Secret32`, `SecretCell` | vorhanden; `Secret32::new`, `expose`; `Drop` nullt | `zeroize`-basiert, `SecretAlloc` als Fabrik | `SecretAlloc` fehlt; `Drop` auf `zeroize` umstellen |
| `FileKind`, `FileRef` | `FileKind { Run, Blob, Segment }`, `FileRef { kind, id: u64 }` | `FileKind::Segment(u16)` | Form von `Segment` angleichen |
| `Decision`, `CompactionFilter`, `Stamp` | vorhanden | vorhanden | — |
| `Checksum`, `Hasher`, `Observer`, `Anchor`, `ColdTier`, `SecretAlloc`, `Weigher`, `Filter`, `ExtensionBuilder`, `Runtime` | **fehlen** (`rg 'trait (Anchor\|Checksum\|Hasher)'` in `contextra-ports/src`: 0 Treffer) | vorhanden bzw. in „Verträge“ beschrieben | neu anlegen; `Runtime` und `Governor` liegen heute im Kern (`contextra-storage/src/env/runtime.rs:67`, `env/governor.rs:96`), nicht im Port (SA-9) |
| `ClusterId`, `ObserverEvent`, `Entry`, `ClusterSet`, `ExtError`, `AnchorValue`, `AnchorError`, `KeyState`, `CryptoError`, `Key32` | **fehlen** (`ClusterId`: 0 Treffer im gesamten Repo) | in §4 und §16 benannt | neu anlegen |

Bestehende Nähte außerhalb von `persistence`: `StorageEngine` (`storage.rs:35`), `KvBridgeStorage` (`kv_bridge_port.rs:15`), `KvPrefixStore` (`kv.rs:139`). Sie gehören zur alten Welt und sind **Adapterziele** für G-5 (§7), nicht Teil des Kerns.

#### 2.2.2 Crate-Abhängigkeiten von `contextra-ports`

`[dependencies]`: `contextra-types`, `blake3`, `futures-util = "0.3"` (nicht Workspace), `bytes`, `serde`, `serde_json`, `ahash`, `thiserror`. Features: `default = []`, `contract-tests`, `testkit`. Der Kern bindet das Crate heute **nicht** ein: `contextra-storage` nutzt `contextra_ports::persistence` in 59 Dateien (67 Zeilen), das Crate hängt aber mit allen genannten Abhängigkeiten im Baum. Damit ist die Allow-Liste aus Spec §3.5 **transitiv** verletzt (SA-1, IF-1).

`contextra-ports` selbst hat 26 Abhängige (27 `Cargo.toml` mit `contextra-ports`, darunter das Crate selbst). Spec §26.1 („etwa 26“) stimmt damit.

#### 2.2.3 Konformitätssuiten

`fs_suite` existiert als `run_fs_conformance_suite<F: Fs>(fs, root)` in **`contextra-storage/src/testing/conformance/fs_suite.rs`** (231 Zeilen, 9 Einzeltests). Aufrufer: `tests/fs_mem_test.rs` (MemFs), `tests/fs_real_test.rs` (RealFs), `tests/env/main.rs`, `tests/testing/mem_fs_tests.rs`. `contextra-testkit/src/conformance/` kennt nur Suiten der alten Ports (Clock, Rng, Metrics, Storage, Vector, Graph, Text), **keine** für `Fs`, `Cipher`, `Mac`, `Kdf`, `KeyProvider`. `contextra-ports/src/contract_tests.rs` (346 Zeilen) deckt keinen `persistence`-Port ab.

### 2.3 Nachbarcrates des Port-Katalogs

| Crate | Im Workspace | `src` Zeilen | Befund gegen den Kern |
|---|---|---|---|
| `contextra-crypto` | ja (`members`, `default-members`) | 7 668 | Eigene Primitive: AES-256-GCM-SIV mit **12-Byte-Nonce** (`wal_crypto.rs:197`; 4 Byte Präfix aus `OsRng` plus 8 Byte Zähler, `crypto.rs`), HKDF, BLAKE3, Ed25519, Argon2. **Keine Implementierung eines `persistence`-Ports** (`Cipher`, `Mac`, `Kdf`, `KeyProvider`: je 0). Kein `chacha20poly1305`. `KeyManager` implementiert `KeyProvider` nicht. 6 Crates hängen davon ab (u. a. `contextra-store`, `-engine`, `-db`, `-vector`, `-mcp`, `-kvcache`). |
| `contextra-durable-fs` | **nein** (weder `members` noch `default-members`) | 1 910 | Eigenes Trait `Vfs` (`vfs.rs:12`); `RealFs` ist nur ein Alias `pub type RealFs = StdVfs` (`vfs.rs:60`), **kein** Port-`Fs`. **Kein** `FileAnchor`, kein `FsCaps`, kein `Perm::Private`. `DirLock` legt nur eine Datei `.dirlock` an und nimmt **keinen** OS-Lock. `#![forbid(unsafe_code)]`, keine `libc`. Keine Abhängigkeit von `contextra-ports`. Kein anderes Crate hängt davon ab. |
| `contextra-testkit` | ja | 2 816 | Hängt an `contextra-ports` (Feature `testkit`). `ManualClock`, In-Memory-Stores, `FaultVfs` (eigene API, **kein** Port-`Fs`). **Kein Test-`KeyProvider`**, keine `Cipher`-/`Mac`-Referenz. Darf nur als `dev-dependency` genutzt werden (INV-EMBED-1). |
| Port-Implementierungen heute | — | — | `RealFs`, `MemFs`, `FaultFs`, `AesGcmSivCipher`, `Blake3Mac`, `Blake3Kdf` und `TestKeyProvider` liegen **im Kern** (`env/real_fs.rs`, `testing/mem_fs.rs`, `testing/fault_fs.rs`, `testing/refimpl.rs`, `tests/engine/flush_tests.rs`). `refimpl.rs` zieht `blake3` und `aes-gcm-siv` über das Feature `testing` (D-05: zu entfernen). |

### 2.4 Konsumentenmatrix `contextra-store`

Zählung: Zeilen mit `contextra_store` in `*.rs` (ohne `contextra-store` selbst), gemessen am Basis-Commit `cf870dcd1`. Die Zahlen aus §26.1 und dieser Abschnitt reproduzieren sich damit. Befehle und Rohausgaben stehen in `NACHWEIS.md`, Abschnitte 1 bis 4. „Prod“ = nicht in `tests/`, `benches/`, `*tests.rs` und nicht in einem `#[cfg(test)]`-Block; die Zuordnung je Zeile macht `split.py` aus `NACHWEIS.md` (Klammertiefe, ohne Stringparser). Die 26 Prod-Zeilen von `contextra-engine` sind einzeln geprüft.

| Konsument | `Cargo.toml` | Zeilen | Prod | Test | Genutzte Items in Prod |
|---|---|---|---|---|---|
| `contextra-engine` | `[dependencies]` | 144 | 26 (1 davon Doc-Kommentar) | 118 | `LsmStorage`, `LsmConfig`, `tenant_codec::TenantScopedStorage`, `SystemPressure`, `PressureLevel`, `MergeOperator` (Adapter `WasmMergeOperator`), `lsm::DurabilityMode` (Re-Export `lib.rs:18`), `scanner::{lsm_purgers, lsm_scanners}`; Feld `snapshot_registry` (ohne Namensnennung) |
| `contextra-db` | `[dependencies]` | 25 | 0 | 25 | keine. Nur Tests: `LsmConfig`, `LsmStorage`, `PressureLevel`, `SystemPressure`, `CompactionConfig` |
| `contextra-graph` | `[dev-dependencies]` | 16 | 0 | 16 | keine. Nur Tests: `LsmConfig`, `LsmStorage` |
| `contextra-cognition` | `[dependencies]` | 15 | 0 | 15 | keine. Nur Tests: `LsmConfig`, `LsmStorage` |
| `contextra` (Fassade) | `[dependencies]` | 5 | 2 | 3 | `lsm::config::DurabilityConfigError`, `lsm::DurabilityMode`, `kv::delete_mode::KvDeleteMode` |
| `contextra-agent` | `[dependencies]` | 3 | 3 | 0 | `LsmStorage` (nur im `use`). Die 35 Aufrufe von `inner_storage()` (davon 11 in `src`) nennen `contextra_store` nicht; `last_tx_id()` ist eine Port-Methode, `CheckpointGuard::for_agent_step` gehört zu `contextra-checkpoint` |
| `contextra-checkpoint` | `[dev-dependencies]` | 2 | 0 | 2 | keine. Eine Testdatei (`tests/checkpoint_systematic_crash.rs`), davon eine Zeile Kommentar |
| `benchmarks/contextra-bench` | `[dependencies]` | 1 | 0 | 1 (`benches/`) | `LsmConfig`, `LsmStorage` (Bench) |
| `contextra-py` | `[dependencies]` | 0 | 0 | 0 | keine (nur `Cargo.toml`) |
| Summe | | 211 | 31 | 180 | |

Zwei weitere Treffer in `xtask/src` (`symbol_exists.rs:1`, `gen_arch_docs.rs:292`) sind Zeichenketten und Doc-Kommentare; sie zählen nicht als Abhängigkeit und stehen nicht in der Summe. Keiner der Konsumenten macht die Abhängigkeit optional; ein Feature-Gate gibt es noch nicht.

**Befunde gegenüber Spec §26** (daraus folgen SA-6, SA-7, SA-10, SA-11; Textänderungen in SPEC §26, Freigabe durch den Maintainer offen):

1. Produktiv nutzen `contextra-store` nur `contextra-engine` (26), `contextra-agent` (3) und die Fassade `contextra` (2). `contextra-db` und `contextra-cognition` führen es unter `[dependencies]`, nutzen es aber nur in Tests (Kandidaten für `[dev-dependencies]`). `contextra-graph` ist schon `[dev-dependencies]`.
2. **Korrektur zu Befund 2 der Vorfassung und zu §26.2 (SA-6):** `contextra-checkpoint` importiert im Produktivcode kein `contextra_store`. Es hängt am Port `contextra_ports::StorageEngine` (Trait-Methoden `rollback_to_tx` und `last_tx_id`, Generik-Bound in `store.rs`), nicht an einem eigenen Trait. Der Snapshot-Typ kommt aus `contextra_mvcc::SnapshotRegistry`. `contextra_store` steht nur als Dev-Abhängigkeit.
3. §26.3 bildet nicht ab (SA-7), Fundstellen: `scanner::lsm_purgers`/`lsm_scanners` (4 Prod-Stellen in `contextra_impl/erasure.rs`); `snapshot_registry.min_active_seqno()` (Prod, `erasure.rs:146, 354`) und `register_at()` (nur Test, `erasure_orchestrator_tests.rs:114`); `last_tx_id()` (Port-Methode); `inner_storage()` (`contextra-engine/src/lib.rs:369`, Adapter-Handle); `PressureLevel` (Prod, `collection/crud/internal.rs`); `DurabilityConfigError` (Prod, `contextra/src/performance_profile.rs`). Die Zielnamen `Storage::last_stamp()` und `oldest_view_stamp()` aus der Vorfassung stehen nicht in der Spec. Maßgeblich sind `View::stamp()` (§8.3), `EraseReq.views` (§8.5) und `Error::Config` (§8.7).
4. `MergeOperator` ist produktiv **nirgends** verdrahtet (SA-10 bestätigt): nur der Adapter `WasmMergeOperator` (`contextra-engine/src/lib.rs:56`, öffentlich exportiert) und zwei Aufrufe von `with_merge_operator` in `tests/wasm_merge_operator.rs` (Zeilen 278, 354). Das Register C-34 („`contextra-engine` nutzt `MergeOperator`“) ist im Code nicht belegt.
5. Die Zählung erfasst nur Pfadnennungen. Methodenaufrufe auf Werten aus anderen Crates (z. B. `inner_storage().last_tx_id()`, `snapshot_registry`) tauchen im Quelltext nicht als `contextra_store` auf und fehlen in der Zählung (SA-11). Der Umstellungsaufwand ist damit größer als die Zeilenzahl.
6. Nicht in der Matrix: `crates/contextra-store/fuzz/Cargo.toml` hängt von `contextra-store` ab; es gehört zum Altcrate und entfällt mit ihm.

### 2.5 Weitere Befunde

- `tools/reference/generator.py`, die Pflichtquelle der Goldens (Spec §22 G-2, §19.4), **existiert nicht** (`ls tools/`: `audit_governance_paths.py`, `baseline/`, `decouple_vector_core.py`, `generate_migration_edges.py`, `loc.py`). WP-2.1 muss ihn anlegen (R-3).
- `contextra-store` hat 27 898 Zeilen `src` (`find src -name '*.rs' | xargs cat | wc -l`), davon 10 240 in Dateien mit „test“ im Pfad; inklusive `tests/` und `benches/` sind es 51 727 Zeilen. Es ist die Vorlage für den Verhaltensvergleich ab G-5, nicht für den Entwurf.
- Das Verzeichnis `crates/` enthält 35 `AGENTS.md`. CLAUDE.md §0 verbietet **neue** Selbstdeklarationen unter `crates/`; bestehende Dateien werden gepflegt (§7.4).

---

## 3. Zielbild der Crate-Landschaft

### 3.1 Abhängigkeitsrichtung (Spec §3.1, D-02, D-18)

```
                 contextra-ports  (Modul persistence: Port-Katalog, keine Logik)
                  ▲      ▲      ▲        ▲
                  │      │      │        │
   contextra-storage  contextra-crypto  contextra-durable-fs  contextra-testkit
   (Kern; std, zeroize,  (Cipher, Mac,    (FileAnchor, optionale  (Test-Primitive,
    crossbeam-skiplist,   Kdf, Hasher,     gehärtete Fs-Variante)  Test-KeyProvider,
    bytes optional)       FileKeyProvider)                         Konformität)
         ▲   dev-dep ◄───────┴───────────────────────┴──────────────────┘
         │
   Adapter (Feature je Konsument, ab G-5)
         ▲
   contextra-engine · -db · -cognition · -agent · contextra (Fassade)
```

Regeln:

1. Der Kern hängt nur am Port-Modul, nie an `contextra-crypto`, `contextra-durable-fs` oder `contextra-testkit` (außer `testkit` als Dev-Abhängigkeit). Implementierer hängen am Port, nie am Kern.
2. Es entstehen keine neuen Crates (D-18). `contextra-storage-ports` und `contextra-crypto-ref` entfallen (D-02, D-05).
3. `contextra-mvcc`, `-checkpoint`, `-vector`, `-text`, `-graph`, `-privacy`, `-audit-export`, `-license` bleiben obere Schichten (Spec §26.2) und sehen den Kern nur über die stabilen Flächen (§8.9 der Spec).

### 3.2 Ablösung in vier Phasen

| Phase | Gate | Zustand von `contextra-store` | Zustand der Konsumenten |
|---|---|---|---|
| A | G-1 bis G-4 | unverändert | unverändert |
| B | G-5 | unverändert; läuft parallel | Adapter hinter Cargo-Feature je Konsument, beide Backends testbar |
| C | G-6 | wird Referenz | Konsumentenmatrix zeigt auf den Kern |
| D | nach G-6 | entfernt, Feature-Weichen entfernt | nur noch Kern |

Vor G-5 ändert sich kein Konsument (Spec §22). Die in dieser Aufgabe vorgenommenen Änderungen an den `AGENTS.md` der Nachbarcrates sind reine Dokumentation (§7.4) und ändern weder Code noch Abhängigkeiten.

---

## 4. Modulplan `contextra-storage`

### 4.1 Zielbaum (Spec §3.2, P-6: Datei ≤ 400 Zeilen, Modultiefe ≤ 3)

```
src/
  lib.rs                 #![forbid(unsafe_code)], Lints (UE-1), Modulgerüst
  sync.rs                Ranked<T, RANK>, Generationszähler; einziger Ort mit cfg(loom)
  api/                   storage · config · space · change · view · cursor · page · hold · restriction
                         consumer · eraser · inspector · pending · error · listener
  engine/                committer/{mod, intake, group, gate, idem} · memtable · version · views
                         registry · claims · retention · volatile
  maint/                 scheduler (Lanes, Governor) · flush · compaction/ · blob_gc · purge · pack_drop
  erase/                 gate · cluster_table · links · protocol · ledger · oracle
  recovery/              mod (R1–R10) · anchor_check · redo_replay
  catalog/               snapshot · edits · log · rollover · current · reservations
  store/                 run/ · blob/ · segment/ · redo/
  keys/                  hierarchy · keys_file · cluster_keys · key_cache · nonce · rotation
  codec/                 varint · tag · checksum · comparator · limits
  env/                   fs · clock · entropy · governor · metrics · anchor · mem_fs · real_fs · fault_fs · sim_clock
  testing/               (Feature `testing`) Hooks, Konformitätssuiten, Absturzmatrix-Treiber
```

**Bestand → Ziel** (mechanische Verschiebung per `git mv`, WP-1.5; Inhalt wird erst im jeweiligen Gate umgebaut):

| Bestand | Ziel | Bemerkung |
|---|---|---|
| `redo/`, `run/`, `blob/` (Wurzelebene) | `store/redo`, `store/run`, `store/blob` | Spec §3.2: `store` bündelt sie; `segment/` entsteht neu |
| `engine/flush.rs`, `engine/compaction/` | `maint/flush.rs`, `maint/compaction/` | Spec D-22: Wartung liegt in `maint` |
| `engine/recovery.rs` | `recovery/` | eigenes Modul laut Spec §3.2 |
| `engine/committer.rs` (1 103 Zeilen) | `engine/committer/{mod,intake,group,gate,idem}` | P-6 |
| `testing/{mem_fs,fault_fs}` | bleibt unter `testing` (Feature); Spec §3.2 führt `MemFs`/`FaultFs` unter `env`: siehe SA-5 | |
| `tests/layering.rs` | wird auf das Zielbaum-Schichtmodell (§4.2) angepasst | liest `use crate::…` |

### 4.2 Schichtregeln (Spec §3.3, erzwungen durch `tests/layering.rs`)

```
api ──► engine ──► { store, catalog, erase } ──► { keys, codec, env }
erase ──► { store, catalog, keys, codec, env }          (nie engine; der Löschdurchlauf liegt in maint)
maint ──► { engine, erase, store, catalog, keys }
recovery ──► { engine (nur Memtable-Füllung über Schnittstelle), store, catalog, erase, keys, env }
keys ──► { codec, env }       store/catalog nutzen nur keys::*-Schnittstellen
```

`recovery` ist in der Spec nur im Modulschnitt erwähnt, nicht in §3.3. Die Kante `recovery → engine` ist eine **Annahme** (UE-2): Recovery füllt die Memtables über die Funktion `engine::memtable::Replayer::apply` und baut die `Version`. Wird sie abgelehnt, wandert die Memtable-Befüllung nach `engine::recovery_sink` und `recovery` hängt nur an `store`/`catalog`/`erase`.

**UE-1 (Lints).** `lib.rs` setzt `#![forbid(unsafe_code)]` und `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::todo, clippy::unimplemented, clippy::unreachable, clippy::indexing_slicing, clippy::as_conversions)]` für Produktionscode (P-13, P-14); Testcode und `src/testing/` dürfen per `#![cfg_attr(test, allow(…))]` bzw. Modulattribut lockern. `let _ =` auf `sync_*`, `rename`, `remove_file`, `truncate` wird durch einen Test über den Quelltext erzwungen (`tests/invariants.rs`), weil Clippy dafür keine eingebaute Regel hat.

### 4.3 Grundtypen (Modul `api` und `codec`)

```rust
pub struct SpaceId(pub u16);
pub struct ClusterId([u8; 16]);            // opak; ClusterId::NONE nur in nicht geclusterten Spaces (§6.1)
pub struct Stamp(u64);                     // 1 ≤ s < 2^56; 0 = leer; nur der Committer erzeugt Stamps (INV-STAMP-1)
pub struct Tag(u64);                       // (stamp << 8) | kind
pub enum  Kind { Put = 1, Delete = 2, RangeDelete = 3, EraseRange = 4, PutRef = 5 /* Merge = 6 reserviert */ }
pub struct IdemToken([u8; 16]);
pub struct Receipt { pub stamp: Stamp, pub durable: bool }
pub enum  Durability { Durable, Relaxed }
pub enum  Health { Ready, Degraded, Poisoned }
```

Festlegungen (UE-3, Vorschlag, innerhalb des Spielraums der Spec):

1. `Stamp` und `Tag` sind Newtypes mit `try_from(u64)`. Kein öffentliches `From<u64>`. Der Wert `≥ 2^56` ist `Err(Corrupt)` beim Dekodieren und `Exhausted` beim Vergeben (INV-STAMP-2).
2. `ClusterId` hat keine `Display`-Implementierung mit Inhalt; `Debug` zeigt nur die ersten vier Byte als Hex (P-12: keine Inhalte in Logs).
3. Der Vergleich `InternalKey::cmp` ist eine eigene Funktion (`codec::comparator`), kein `Ord` auf einer Konkatenation (Spec §6.2). Property-Test gegen ein Modell mit `Vec<(ClusterId, Vec<u8>, Reverse<Tag>)>`.
4. Fehler: ein Enum je Schicht (`CodecError`, `StoreError`, `CatalogError`, `ApplyError`, `OpenError`), alle ohne Inhaltsfelder (INV-KEY-3); `api::error::Error` fasst sie nach §8.7 der Spec zusammen. `From`-Umwandlungen nur nach oben.

---

## 5. Abläufe und Zustandsautomaten

Jeder Ablauf nennt die **Reihenfolge der E/A-Aufrufe**, weil daraus die Absturzpunkte (§5.9) und die Nachweise folgen. „S“ = `sync_data`/`sync_all`, „D“ = `sync_dir`.

### 5.1 Commit-Pfad (Spec §11.1–§11.5)

```
submit(Change)
 1 Intake (ohne Sperre):   Limits, MixedDurability, Forbidden          → ChangeTooLarge | Limit | MixedDurability | Forbidden
 2 Intake (INTAKE(2)):     Idempotenz-Index                            → altes Receipt | IdemConflict | IdemExpired
 3 Intake (CLUSTER_TABLE(7) lesen): Zustand, Restrictions              → ClusterGated | Restricted{ids, reasons}
 4 Intake (ohne Sperre):   create_cluster_key + make_durable beim ersten Schreiben (UE-5, §5.6)
 5 Intake (ohne Sperre):   Versiegelung (SealBudget, Nonce24 aus Entropy, AAD (space, cluster, op_uid, op_index))
 6 Intake (INTAKE(2)):     Frame ohne Stamp einreihen, Fertigmeldung (`Pending`) anlegen
 ─────────────────────────────────────────────────────────────────────────────────────────────────
 7 Committer (COMMITTER(1)): Gruppe bilden (Volatile getrennt), Gate-Prüfung je Frame (C-17), Guards prüfen
 8 Committer:              Stamps aus dem reservierten Block; reicht er nicht → Reserve-Edit (CATALOG_EDIT(3): Edit + S) VOR dem Frame
 9 Committer:              Gruppe kodieren → write_all_at (eine Grenze redo_align) → S (sync_data)
10 Committer:              Memtable füllen (lock-frei), dann durable_stamp, dann visible_stamp (Release)
11 Committer:              Receipt{stamp, durable} an alle Wartenden; Listener-Ereignisse einreihen (NOTIFY(9))
```

Regeln:

- **Guards** (Spec §6.5) werden in Schritt 7 unter `COMMITTER(1)` gegen den neuesten Stand **einschließlich früherer Frames derselben Gruppe** geprüft. Dazu hält der Committer während der Gruppe eine kleine Hilfs-Map `(space, cluster, key) → letzter Stamp in dieser Gruppe`. `RangeUnchangedSince` nutzt `max_stamp` je Block (O(überlappende Blöcke)). Ein fehlgeschlagener Guard scheidet **nur diesen Frame** aus der Gruppe aus (`GuardFailed{guard_index}`), die Gruppe bleibt gültig.
- **Reihenfolge der Sichtbarkeit:** erst `sync_data`, dann Memtable (INV-COMMIT-1). `Relaxed`-Changes (Opt-in je Aufruf) werden nach Schritt 10 quittiert, **ohne** auf `S` zu warten (`Receipt.durable = false`); die Gruppe wird trotzdem geschrieben. Besteht eine Gruppe nur aus `Relaxed`-Frames, darf `S` bis zum Intervall `OP-3` aufgeschoben werden (Uhr nur für Fristen, F-11); `Storage::sync()` erzwingt es.
- **Volatile:** nur Memtable, kein Redo, nie `durable = true` (INV-COMMIT-3). Nach Absturz sind sie weg; die Memtable des Spaces ist leer.
- **Fehler** nach Spec §11.6: Fehler vor `S` mit gelungenem Abschneiden → `Degraded`; Fehler von `S` → **`Poisoned`, kein Retry, kein Abschneiden** (INV-SYNC-1), Wartende erhalten `Indeterminate`; Fehler nach erfolgreichem `S` → `Indeterminate` und `Poisoned`. `let _ =` auf `sync_*` ist verboten (P-13).
- **Zuweisung der Stamps:** streng aufsteigend je angenommenem Frame in Gruppenreihenfolge, Lücken zulässig (INV-STAMP-1, INV-STAMP-3). Überlauf → `Exhausted` (INV-STAMP-2).
- **Sperrfolge:** Schritt 3 liest `CLUSTER_TABLE(7)` aus `INTAKE(2)` heraus (aufsteigend, zulässig); Schritt 8 nimmt `CATALOG_EDIT(3)` unter `COMMITTER(1)` (aufsteigend, zulässig). Die Gate-Prüfung in Schritt 7 liest `CLUSTER_TABLE(7)` unter `COMMITTER(1)`; der Intake-Befund aus Schritt 3 ist nur eine frühe Abweisung, maßgeblich ist Schritt 7 (C-17).

### 5.2 Instanzzustand (Spec §11.6, D-25)

```
Ready ──(Fehler vor S, Abschneiden ok; ENOSPC)──► Degraded ──resume_writes()──► Ready
Ready ──(Abschneiden scheitert | S scheitert | Fehler nach S)──► Poisoned
Degraded ──(Abschneiden scheitert | S scheitert)──► Poisoned
Poisoned ──(nur Neustart; Recovery R1–R10)──► Ready
```

Umsetzung (UE-4):

1. Der Zustand ist ein `AtomicU8` plus Grund im Rang-4-Zustand `STATE(4)`. Übergänge nur über `fn transition(&self, from: Health, to: Health) -> Result<(), StateError>` mit Vergleich-und-Tausch; ein Übergang aus `Poisoned` heraus ist im Code nicht ausdrückbar (kein Pfad außer `Storage::open`).
2. `Degraded` hat `enospc_cooldown` (Spec §20.2): `resume_writes()` vor Ablauf liefert `Busy{StorageFull}`, nicht `Ok`.
3. Lesen bleibt in allen Zuständen erlaubt; Wartung nur, wenn sie Platz freigibt (`Degraded`) bzw. gar nicht (`Poisoned`).
4. Jeder Fehlerpfad hat einen Test auf `FaultFs` (Spec §19.1, P-8): Schreibfehler vor `S`, ENOSPC, Abschneiden schlägt fehl, `S` schlägt fehl, Fehler zwischen `S` und Receipt.

### 5.3 Öffnen und Recovery (Spec §14, R1–R10)

Der Ablauf ist die Spec-Tabelle R1–R10 in genau dieser Reihenfolge. Umsetzungsvorgaben:

| Schritt | Modul | E/A-Folge | Nachweis |
|---|---|---|---|
| R1 | `recovery/mod` | `Fs::lock(LOCK, mode)` | zwei Instanzen: zweite erhält `Locked` |
| R2 | `store`/`codec` | `open(FORMAT, Read)`, lesen, prüfen | Fuzz auf `FORMAT`; INV-FMT-4 |
| R3 | `keys/keys_file` | `KEYS` lesen, `kek()` vom Provider, entschlüsseln | `KeyUnavailable`, `Rollback` bei falscher `keys_generation` |
| R4 | `catalog/log` | `CURRENT` lesen, Snapshot laden, Edits replayen; **nur der letzte Record darf halb sein** (INV-CAT-3) | Absturzmatrix |
| R5 | `recovery/anchor_check` | `Anchor::read()`; Vergleich gegen R4-Stand | Anker über Stand ⇒ `Rollback`; leerer Anker bei vorhandenen Epochen ⇒ `Rollback` |
| R6 | `recovery/mod` | `.building.*` entfernen (`remove_file`, danach `sync_dir`); Verwaiste nach Schonfrist markieren | `.building.*` nie im Catalog |
| R7 | `recovery/redo_replay` | Segmente ab `redo_floor` lesen, Torn-Tail nach INV-LOG-8, Gate-Frames sofort wirksam | idempotent (INV-RECOV-2) |
| R8 | `recovery/mod` | Clusterzustände abgleichen | `Gated` bleibt `Gated` (INV-ERA-2) |
| R9 | `erase/ledger` | Ledger-Kette prüfen, `ledger_head` gegen Anker, offene `erase_pending` einplanen | `Rollback`/`Corrupt` |
| R10 | `recovery/mod` | Zustand `Ready` setzen | nur nach R1–R9 |

Festlegungen:

- **UE-6 (Torn-Tail).** Die Entscheidung „halb geschriebener letzter Frame“ trifft allein `redo_replay` anhand von Länge, Kettenprüfwert und der Bedingung, dass **nach** dem Frame nur Nullen oder Dateiende folgen (INV-LOG-8). Alles andere ist `Corrupt{RedoMidLog}`. Der Verwerfungs-Schnitt wird mit `truncate` + S dauerhaft gemacht, bevor Schreiben freigegeben wird (R10).
- **UE-7 (Replay-Grenze).** Replay überspringt Changes mit Stamp ≤ `space_flushed` je Space (INV-RECOV-1) und prüft `stamp_hw` (INV-STAMP-3). Der Replay-Zähler wird nur in Metriken geführt, nie in Fehlertexten.
- **Fail-closed:** Es gibt keinen Weg, R-Schritte zu überspringen (SD-07). `ReadOnly` führt R1–R9 aus, setzt aber kein `Ready` zum Schreiben.

### 5.4 Flush (Spec §13.2)

```
maint/flush:  Memtable einfrieren (REGISTRY(6) kurz) ──► Run schreiben (.building.<id>) ──► S ──► rename ──► D
             ──► Catalog-Edit {files_add(run), space_flushed(stamp)} (CATALOG_EDIT(3): Edit + S)
             ──► neue Version veröffentlichen (STATE/REGISTRY, kurz) ──► Memtable verwerfen ──► redo_floor prüfen (§13.4)
```

- **P-3** gilt wörtlich: Datei dauerhaft (Datei **und** Verzeichnis), dann Edit dauerhaft, erst dann sichtbar, erst dann alte Dateien löschen.
- Bei `Seal::Cluster` werden Blöcke vor dem Schreiben versiegelt (Nonce24 aus `Entropy`, Schranke 2^32 je Schlüssel über `seal_hw`-Blockreservierung, D-28).
- Wartung hält Rang 8 nicht über Edit oder E/A (D-22): Auswahl unter `MAINT(8)`, dann freigeben, dann E/A.

### 5.5 Catalog-Edit und Rollover (Spec §10.8)

1. Ein Edit ist ein Record in der aktuellen Catalog-Datei: Länge ‖ Nutzlast ‖ Prüfwert; `write_all_at` → S. Erst nach S gilt der Edit (P-2: eine Zustandsänderung = ein Edit).
2. **Rollover:** neue Catalog-Datei (`.building.*`) mit vollständigem Snapshot schreiben → S → `rename` → D → `CURRENT` atomar auf die neue Datei umstellen (`write_atomic`: tmp → `sync_all` → `rename` → D) → alte Datei in den Graveyard. Danach Anker auf neue `catalog_epoch` (§16.7: **erst** Catalog dauerhaft, dann Anker).
3. `reserve`-Edits (Stamp-Block, `seal_hw`, Segment-IDs) laufen über denselben Pfad; der Committer ruft ihn unter `COMMITTER(1)` → `CATALOG_EDIT(3)` auf (aufsteigend).
4. **UE-8 (Edit-Gruppen).** Mehrere Edits derselben Wartungsaktion (`files_add` + `files_remove`) sind **ein** Record (P-2). Der Catalog kennt keine Mehr-Record-Transaktion.

### 5.6 Clusterschlüssel (Spec §16.2, D-19, INV-KEYP-1)

1. **Anlage (UE-5).** Beim ersten Schreiben in einen Cluster mit `Seal::Cluster` ruft der Intake (ohne Sperre, P-10) `KeyProvider::create_cluster_key(c, entropy)` → liefert `epoch`; der Cluster-Eintrag im Catalog bekommt `key_epoch`. Vor dem Dauerhaftwerden des ersten Frames mit so versiegelten Körpern ruft der Kern `make_durable()` (Barriere). Der Schlüssel selbst steht **nie** im Catalog oder in `KEYS` (INV-KEY-1).
2. **Cache.** `keys/key_cache` hält `Secret32` (bzw. `SecretCell` über `SecretAlloc`) mit Obergrenze (`key_cache_entries`, Spec §20.2); Eintrag beim Löschen mit `zeroize` überschrieben.
3. **Vernichtung (ER6).** `destroy(c)` → `probe(c) == Absent` → Cache-Eintrag überschreiben → Edit `key_destroyed`. `None` von `cluster_key` ist nur nach geprüfter, dauerhafter Vernichtung zulässig (Vertrag (3) in Spec §4).
4. **Nonce.** 24 Byte aus `Entropy`; Zählung nur zur Schranke (2^32 je Schlüssel) über blockweise `seal_hw`-Reservierung im Catalog, **kein** Zählerzustand in der Nonce (SD-14, D-28). Erreichen der Schranke ⇒ Rotation (Epoche +1) vor dem nächsten Versiegeln.
5. **Provider-Aufrufe** laufen ohne gehaltene Sperre und dürfen blockieren (P-10).

### 5.7 Löschdurchlauf (Spec §15.5, ER1–ER7)

Die Reihenfolge ist Vertrag und wird nie geändert (INV-ERA-6, Vorwärtsrichtung). Der Durchlauf liegt in `maint` (D-22), nutzt `erase::{gate, protocol, ledger, oracle}`:

| Schritt | E/A-Folge | Catalog-Edit | Absturz danach |
|---|---|---|---|
| ER1 Gate | Gate-Frame (neues Segment) → S | `cluster_state(c, Gated, g)` | Recovery R7/R8 gated sofort wieder |
| ER2 Log | ganze Segmente vor `g` entfernen (`redo_floor` anheben, `remove_file`, D) — entfällt bei `Seal::Cluster` | `redo_floor` | setzt bei ER2 fort |
| ER3 Runs | Runs ohne den Cluster neu schreiben (`.building.*` → S → `rename` → D) | `files_add`/`files_remove` je Gruppe, **ein** Edit | setzt bei ER3 fort |
| ER4 Blobs | Records mit Nullen ersetzen, neue Inkarnation | `blob_replace` | setzt bei ER4 fort |
| ER5 Scrub | Scrub über Dateien, Kopien, Caches; Orakelbericht | — | Bericht wird wiederholt |
| ER6 Schlüssel | `destroy` → `probe` → `zeroize` | `key_destroyed` | idempotent (D-07) |
| ER7 Quittung | Ledger-Eintrag → S → D; **danach** Anker `advance` | `erase_done`, `erase_swept` | zu alter Anker möglich, nie zu neuer (§16.7) |

Profile (`EraseLevel`) wählen eine Schrittmenge in aufsteigender Nummer; ohne `Seal::Cluster` höchstens ER5 (D-20). Der Auftragszustand steht im Catalog, nie im Speicher allein (INV-ERA-6).

### 5.8 Compaction und Dateilebenszyklus (Spec §9, §13)

- Dateizustand: `Building → Durable (Datei + Verzeichnis) → Lebendig (durabler files_add) → Obsolet (Edit files_remove) → Graveyard → entfernt`. Lebendig ⇔ durabler `files_add` ohne `files_remove` (INV-CAT-1). Redo-Segmente: lebendig ⇔ `segment_id ≥ redo_floor` (C-19).
- Claims (Spec §17.2): Leser halten einen Claim auf Run/Blob; Compaction ersetzt Dateien per **einem** Edit, die alten gehen in den Graveyard und werden erst entfernt, wenn kein Claim mehr auf sie zeigt (INV-VIEW-1, INV-BLOB-1).
- `blob_live_delta` ist nur Auswahlhilfe; Lebendigkeit eines Blob-Records folgt aus den Runs (C-09).
- Verwaisten-Suche (§13.7) betrifft nur `spaces/` und `segments/` (D-24), nie Ledger, `KEYS`, `FORMAT`.

### 5.9 Absturzpunkte (Grundlage der Absturzmatrix, Spec §19.1)

Jede Zeile bezeichnet einen Punkt, an dem `FaultFs` den Prozess „abstürzen“ lässt (alle nicht synchronisierten Schreibvorgänge verfallen, ggf. zerrissene Schreibvorgänge im Torn-Modus). Erwartung: Recovery R1–R10 ergibt einen der angegebenen Zustände.

| ID | Punkt | Erwarteter Zustand nach Neustart |
|---|---|---|
| X-01 | nach `write_all_at` der Redo-Gruppe, vor S | Gruppe fehlt vollständig oder Torn-Tail verworfen; kein Receipt wurde gesendet |
| X-02 | nach S der Redo-Gruppe, vor Memtable | Gruppe per Replay vorhanden; Aufrufer prüft per `IdemToken` |
| X-03 | nach Memtable, vor Receipt | wie X-02 |
| X-04 | Catalog-Edit geschrieben, vor S | Edit fehlt (halber letzter Record verworfen, INV-CAT-3) |
| X-05 | Run-Datei geschrieben, vor `rename` | `.building.*` wird in R6 entfernt |
| X-06 | nach `rename`, vor D | Datei nicht im Catalog ⇒ Verwaiste-Regel greift nach Schonfrist |
| X-07 | nach durablem Run, vor Edit | Run nicht lebendig; Redo bleibt Quelle (`space_flushed` unverändert) |
| X-08 | nach Edit `files_add`, vor Entfernen alter Dateien | alte Dateien im Graveyard/Verwaiste, Zustand konsistent |
| X-09 | Rollover: neue Catalog-Datei durabel, `CURRENT` nicht umgestellt | alte Datei gilt; neue wird in R6 entfernt |
| X-10 | Rollover: `CURRENT` umgestellt, Anker noch alt | Anker zu alt ⇒ zulässig (§16.7) |
| X-11 | Gate-Frame durabel, Edit `Gated` fehlt | R7/R8: Cluster wird aus dem Redo wieder `Gated` |
| X-12 | zwischen ER-Schritten (je Schritt) | Auftrag setzt mit dem nächsten Schritt fort (INV-ERA-6) |
| X-13 | ER6: `destroy` durabel, Edit `key_destroyed` fehlt | `probe` = Absent ⇒ Recovery trägt `key_destroyed` nach |
| X-14 | ER7: Ledger durabel, Anker alt | R9: Anker ist Vorfahre der Kette ⇒ zulässig |
| X-15 | `create_cluster_key` durabel, kein Frame | Schlüssel ohne Verwendung; wird nicht wiederverwendet, Epoche verfällt |

Die Matrix wird als Tabelle in `tests/crash/` geführt (WP-3.5); jede Zeile ist ein Test, der die ID im Kommentar nennt.

---

## 6. Schnittstellenverträge zu den anderen Crates

Die Spec nennt Rollen (§26.2), aber keine Verträge. Dieser Abschnitt legt sie fest. Jeder Vertrag nennt: was der Nachbar **liefern** muss, was er **nicht** darf, und wie der Nachweis aussieht. Alles hier ist Vorschlag im Rahmen der Spec; Abweichungen von der Spec sind als SA-n gekennzeichnet.

### IF-1 `contextra-ports::persistence` (Port-Katalog)

**Zielzustand des Moduls.**

- **Eigenständigkeit (D-02):** Das Modul importiert nichts aus `contextra-types`, `blake3`, `serde`, `futures-util`. Es liegt heute in einem Crate, dessen `[dependencies]` alle diese Crates enthalten (§2.2). Damit der Kern die Allow-Liste aus Spec §3.5 auch **transitiv** einhält (`cargo tree -e normal`, Spec §3.3), gibt es zwei Wege: (a) die Persistenz-Ports hinter einem Feature `persistence` führen und die übrigen Abhängigkeiten optional machen; der Kern bindet `contextra-ports` mit `default-features = false, features = ["persistence"]`; (b) die Allow-Listen-Prüfung auf **direkte** Abhängigkeiten und die `use`-Prüfung in `tests/layering.rs` beschränken. Empfohlen ist (a), weil (b) die Spec-Aussage „nur `std`, `zeroize`, …“ aushöhlt. Das ist ein Eingriff in §3.5 und damit **SA-1**.
- **Typen, die fehlen** (Spec §4): `Key32` (Vorschlag: Typalias `[u8; 32]`; die Geheimnis-Eigenschaft liegt in `Secret32`/`SecretCell`), `CryptoError`, `KeyState { Present, Absent }`, `ClusterId`, `AnchorValue { erase_epoch: u64, ledger_head: [u8; 32], catalog_epoch: u64 }`, `AnchorError`, `FileRef { kind, id }` mit `FileKind::Segment(u16)`, `ObserverEvent`, `Entry`, `ClusterSet`, `ExtError`, `Decision`, `Pressure`.
- **Traits, die fehlen:** `Checksum`, `Hasher`, `Observer`, `Anchor`, `ColdTier`, `SecretAlloc`, `Weigher`, `Filter`, `ExtensionBuilder`, `Runtime` (Spec §4, Verträge, letzter Absatz).
- **Zu ändern:** `Cipher` (24-Byte-Nonce, `CryptoError`), `KeyProvider` (Clusterschlüssel statt Domänen), `Secret32` (`zeroize`), `FileKind`.
- **Namen:** Spec §4 nennt `File::write_at` und `File::truncate`, der Bestand `write_all_at` und `set_len`. `write_at` klingt wie das teilweise schreibende `pwrite` und lädt zu Fehlern ein; Vorschlag: Spec auf `write_all_at` ändern (**SA-3**). `truncate` statt `set_len`: der Bestand soll der Spec folgen.
- **Fähigkeiten für Checkpoints:** Spec §8.6 kennt `CheckpointMode::{Copy, Reflink, Hardlink}`, der Port `Fs` aber weder `reflink` noch `hard_link` (der Bestand hat `hard_link` als Pflichtmethode, ohne dass die Spec es nennt). Vorschlag: beide als Methoden mit Standard `Err(Unsupported)` (**SA-2**).
- **Allokationen im Heißpfad:** `Cipher::seal` liefert `Vec<u8>`: eine Allokation je Versiegelung, im Widerspruch zu P-11/H-13. Vorschlag: zusätzlich `fn seal_into(&self, …, out: &mut Vec<u8>)` und `open_into` mit Standardimplementierung über `seal`/`open` (**SA-4**).
- **Anker und Instanz:** `AnchorValue` enthält keine `instance_id`, Spec §14.3 prüft aber `instance_id` „gegen den Anker“. Der Anker muss je Instanz getrennt sein (Dateiname oder Schlüssel aus `instance_id`) oder das Feld aufnehmen (**SA-13**).
- **Nachweis:** `testing::conformance` je Port eine Suite (`fs_suite` besteht; `cipher_suite`, `mac_suite`, `kdf_suite`, `hasher_suite`, `checksum_suite`, `anchor_suite`, `key_provider_suite` neu). Jede Implementierung in `contextra-crypto`, `contextra-durable-fs`, `contextra-testkit` besteht die jeweilige Suite.

**Wo die Konformitätssuiten leben (UE-9).** Die Suiten sind generische Funktionen über dem Port-Trait und liegen im Kern unter `testing::conformance` (Feature `testing`), weil `fs_suite` dort schon liegt und `contextra-testkit` keine Abhängigkeit auf den Kern haben soll. `contextra-crypto`, `-durable-fs` und `-testkit` rufen sie aus ihren Tests über die Dev-Abhängigkeit auf den Kern auf. Das ist keine Laufzeitabhängigkeit und verletzt Spec §3.5 nicht; es verlangt aber, dass der Kern **vor** den Implementierern baubar ist (G-1).

### IF-2 `contextra-durable-fs` → `RealFs`, `FileAnchor`

| Anforderung | Inhalt |
|---|---|
| Workspace | in `members` aufnehmen (WP-1.1); Abhängigkeit nur `contextra-ports` (Feature `persistence`) |
| Ist | eigenes Trait `Vfs`, `RealFs` nur Alias, kein `FsCaps`, kein `Perm::Private`, kein OS-Lock, kein `FileAnchor` (§2.3) |
| Liefern | `FileAnchor: Anchor` — Datei **außerhalb** des Speicherverzeichnisses, atomar ersetzt (`write_atomic`), monoton (`advance` lehnt kleinere Werte ab), durabel bei Rückkehr (Spec §16.7); Dateiname aus `instance_id` abgeleitet (SA-13) |
| Liefern (optional) | gehärtete Port-`Fs`-Implementierung (`DurableFs`): echter OS-Lock (`flock`-Äquivalent ohne `unsafe`, siehe R-5), `Perm::Private` unabhängig von `umask`, `caps()` ehrlich |
| Nicht | `std::fs` im Kern; Abhängigkeit auf den Kern; eigene Fehler- oder Zustandsmodelle neben dem Port |
| Entscheidung `RealFs` | Der Kern **behält** `env/real_fs.rs` (233 Zeilen, nur `std`), weil `Env::real(root)` ohne `contextra-durable-fs` auskommen muss (Allow-Liste, §3.5). Spec §3.1/§26.2 sehen `RealFs` in `contextra-durable-fs`; das widerspricht Spec §3.2 („`std::fs` nur in `env/real_fs.rs`“) — **SA-8** |
| Nachweis | `fs_suite` und `anchor_suite` bestehen; Test: zweiter `Exclusive`-Lock schlägt fehl; Test: `advance` rückwärts ⇒ `AnchorError::NotMonotonic` |

### IF-3 `contextra-crypto` → `Cipher`, `Mac`, `Kdf`, `Hasher`, `FileKeyProvider`

| Anforderung | Inhalt |
|---|---|
| Ist | AES-256-GCM-SIV mit 12-Byte-Nonce, HKDF, BLAKE3 u. a.; keine Port-Implementierung (§2.3) |
| Liefern | `Cipher` mit **24-Byte-Nonce**, 32-Byte-Schlüssel, 16-Byte-Tag. Vorschlag: XChaCha20-Poly1305 (24-Byte-Nonce nativ). AES-GCM-SIV hat 12 Byte; eine 24-Byte-Nonce ließe sich dort nur über Schlüsselableitung je Nonce abbilden (zusätzliche Konstruktion, nicht Teil der Spec). **UE-10:** XChaCha20-Poly1305 als Standard, AES-GCM-SIV bleibt für `contextra-crypto` intern unverändert |
| Liefern | `Mac::mac16`/`verify16` (konstante Zeit, `subtle`), `Kdf::derive`, `Hasher::hash256` (unkeyed, nur Ledger-Kette, D-12) |
| Liefern | `FileKeyProvider: KeyProvider` — Clusterschlüssel zufällig, je Cluster/Epoche in einer Schlüsseldatei, mit KEK umhüllt; `destroy` überschreibt und entfernt, durabel bei Rückkehr; `probe` liefert `Present`/`Absent`; **Schlüssel stehen nie im Datenverzeichnis des Kerns** (INV-KEY-1) |
| Nicht | I/O im Kern-Heißpfad ohne Port; Panik; Schlüsselmaterial in `Debug`/Logs; neue Abhängigkeit des Kerns |
| Prüfen | Ob das vorhandene `KeyManager` (`crypto.rs`) als Grundlage taugt, ist **zu prüfen** (WP-2.4); es implementiert `KeyProvider` heute nicht |
| Nachweis | `cipher_suite` (Roundtrip, Manipulation ⇒ `CryptoError`, falscher Schlüssel, AAD-Bindung), `mac_suite`, `kdf_suite`, `key_provider_suite` (Anlage, `make_durable`-Barriere, `destroy` idempotent, `None` nur nach Vernichtung) |

Bruch-Folge: Das Ändern von `Cipher` auf 24-Byte-Nonce ist ein **Breaking Change** am Port. Im Repo implementiert nur der Kern selbst (`testing/refimpl.rs`) den heutigen `Cipher`; kein anderes Crate ist betroffen (§2.3).

### IF-4 `contextra-testkit`

| Anforderung | Inhalt |
|---|---|
| Liefern | Test-`KeyProvider` (deterministisch, im Speicher; Zeroize), Test-`Cipher`/`Mac`/`Kdf` (ohne Produktions-Krypto-Crates im Kern, D-05), `ManualClock` als Port-`Clock`, Zufallsquelle mit Seed als `Entropy` |
| Entfernen aus dem Kern | `testing/refimpl.rs` (zieht `blake3` und `aes-gcm-siv`) wandert nach `contextra-testkit` (WP-2.5) |
| Nicht | Abhängigkeit auf `contextra-store`; Nutzung außerhalb von `dev-dependencies` (INV-EMBED-1) |
| Nachweis | `cargo tree -e normal -p contextra-storage` enthält weder `blake3` noch `aes-gcm-siv` |

### IF-5 `contextra-mvcc`, `contextra-checkpoint`

- **`contextra-mvcc`** bleibt obere Schicht (SSI, Transaktionspuffer, Snapshot-Register). Der Kern liefert `View::stamp`, `view_at(stamp)`, `Guard::{Version, Unchanged, RangeUnchangedSince}`; der Stamp ist die **einzige** Ordnung (Spec §26.2). Der Kern kennt weder `TxId` noch SSI-Validierung. Die Abbildung `TxId ↔ Stamp` gehört in den Adapter (§7), nicht in den Kern.
- **`contextra-checkpoint`** nutzt heute **nicht** `contextra_store` im Produktivcode (§2.4, Befund 2). Der Vertrag lautet: benannte Checkpoints bleiben dort (NZ-18); der Kern liefert `Storage::checkpoint(dir, CheckpointMode)` mit `Copy`, `Reflink`, `Hardlink` (Spec §8.6, §15.10). `hard_link`/`reflink` kommen über optionale `Fs`-Methoden (SA-2); fehlt die Fähigkeit, fällt der Kern auf `Copy` zurück oder meldet `Unsupported` (zu entscheiden in WP-6.4).

### IF-6 `contextra-vector`, `contextra-text`, `contextra-graph`

- Der Kern kennt keine Indexstrukturen (Spec §2.2). Sie hängen sich als `ExtensionBuilder` (Run-Erweiterungen) bzw. als **Segmente** (vom Catalog verwaltete Dateien, Spec §10.9, §15.9) an.
- **Pflicht für jeden `ExtensionBuilder`:** `rewrite_without(block, drop: &ClusterSet)` (Spec §4, §15.9). Ohne diese Methode ist Löschung (ER3) in Spaces mit Erweiterung nicht zulässig; der Kern verweigert `register_extension` dann mit `Unsupported`.
- Segmentdateien tragen Nutzerdaten, die der Kern nicht lesen kann; sie müssen den Datei-MAC bzw. die Cluster-Versiegelung des Kerns nutzen oder selbst äquivalent schützen (zu klären in G-6, Spec §15.9).
- Nachweis: Konformitätslauf `extension_suite` im Kern mit Testimplementierung; die Crates bringen je eine eigene Implementierung erst nach G-6.

### IF-7 `contextra-privacy`, `contextra-audit-export`, `contextra-license`

- Konsumenten von `Eraser`, `ErasureReceipt` und Ledger-Bytes (Spec §15.8, §26.2). **Politik** (Rechtsgründe, Fristen, Mandanten) liegt dort, nie im Kern (NZ-14, NZ-16).
- Der Kern liefert: `Eraser::erase(cluster, EraseLevel, purge_deadline)` (Pflichtparameter, SD-06), `ErasureReceipt` mit Stufe und Prüfschritt, Ledger-Bytes (Kette, unkeyed `Hasher`), `Inspector` für Zustand. Er liefert **keine** Aussage über Rechtskonformität (O-2).
- Nicht erlaubt: Restrictions automatisch ablaufen lassen (O-3), Schlüssel oder Nutzerschlüssel in Quittungen schreiben (INV-KEY-3).
- Nachweis: `erase_receipt_golden` (Format, Anhang A der Spec); `doc_lint` prüft die Sprache (REQ-DOC-2).

### IF-8 `contextra-engine`, `-db`, `-cognition`, `-agent`, `contextra` (Konsumenten)

Die Umstellung folgt §7. Vertragskern gegenüber dem Kern:

1. Der Kern kennt **keine Mandanten**: `TenantScopedStorage` wird im Adapter auf `ClusterId` bzw. `tenant_tag` der Packgruppe abgebildet (Spec §26.3, NZ-16).
2. `MergeOperator` entfällt (D-30). Da produktiv nichts verdrahtet ist (§2.4, Befund 4), betrifft das nur `WasmMergeOperator` in `contextra-engine`; Ersatz: Guard-gestütztes Lesen-Ändern-Schreiben mit `Guard::Version`.
3. `SystemPressure` → `PressureSource`, `PressureLevel` → `Pressure`. Der Adapter implementiert `PressureSource` über den bestehenden Monitor.
4. `DurabilityMode` → `Durability` je Change und `durability_default` je Space.
5. Löschpfad (`scanner::lsm_purgers`/`lsm_scanners`, `erasure.rs`): wird auf `Eraser` mit `EraseLevel` umgestellt; der Mapping-Eintrag fehlt in §26.3 (SA-7).
6. `snapshot_registry.min_active_seqno()` → `View`-Registry des Kerns (`oldest_view_stamp()`); `last_tx_id()` → `Storage::last_stamp()`; `inner_storage()` → Adapter-Handle (SA-7). Die Namen der Ziel-Methoden sind **Vorschlag**; maßgeblich ist Spec §8.

---

## 7. Migration der Konsumenten

### 7.1 Grundsatz (Spec §26.4)

1. **Ab G-1:** Port-Katalog bereinigen, `contextra-durable-fs` in den Workspace, Konsumentenmatrix gegen den Code verifizieren (erledigt für die Zählung in §2.4). Kein Konsument ändert sich.
2. **Ab G-5:** je Konsument ein Adapter hinter einem Cargo-Feature `storage-core` (Vorschlag); Standard bleibt `contextra-store`. Beide Backends laufen parallel in den Konsumententests.
3. **Nach G-6:** Standard umschalten, `contextra-store` und die Feature-Weichen entfernen.

### 7.2 Reihenfolge der Umstellung (Vorschlag)

| Reihenfolge | Konsument | Grund |
|---|---|---|
| 1 | `contextra-engine` | einziger großer Produktivnutzer (ca. 26 Zeilen Prod, 144 gesamt); bestimmt den Adapter-Schnitt |
| 2 | `contextra` (Fassade), `contextra-agent` | wenige Prod-Zeilen, hängen an `engine`/`db` |
| 3 | `contextra-db`, `-cognition`, `-graph`, `-checkpoint`, `contextra-bench` | nur Tests/Benches; Umstellung = Dev-Abhängigkeit tauschen |
| 4 | `contextra-py` | nur `Cargo.toml`; mit der Fassade |

`contextra-db` und `contextra-cognition` führen `contextra-store` unter `[dependencies]`, nutzen es aber nur in Tests; vor der Umstellung auf `[dev-dependencies]` zu verschieben (Änderung am Konsumenten, daher erst ab G-5).

### 7.3 Adapter-Schnitt

```
contextra-engine ──► trait-Naht (bestehend: StorageEngine / KvPrefixStore aus contextra-ports)
                          ├── Backend A: LsmStorage (contextra-store)       [bis G-6]
                          └── Backend B: CoreAdapter (contextra-storage)     [ab G-5]
```

- Der Adapter liegt **nicht** im Kern, sondern beim Konsumenten (bzw. in einem bestehenden Crate; keine neuen Crates, D-18). Ob die Naht `StorageEngine` aus `contextra-ports` taugt, ist in G-1 zu prüfen (Spec §26.2); die Lücken aus §2.4 Befund 3 sind vorher zu schließen.
- Abbildung: Spec §26.3 plus die Ergänzungen aus SA-7.

### 7.4 Pflege der `AGENTS.md` (dieser Auftrag)

Die `AGENTS.md` der Nachbarcrates enthalten eine einheitlich benannte Sektion **„Storage-Umbau (`contextra-storage`)“** mit: Rolle gegenüber dem Kern, Stand (gemessen), Vertrag (Verweis auf IF-n), Verbote vor G-5 und Nachweis. `crates/contextra-store/AGENTS.md` trägt einen Archiv-Hinweis. Rein dokumentarisch; kein Code, keine Abhängigkeit wird geändert. Die Sektion verweist auf dieses Dokument und auf `docs/SPEC.md`, enthält keine `ANCHOR[`-/`FILE-CONTEXT`-Selbstdeklaration (CLAUDE.md §0).

---

## 8. Arbeitspakete je Gate

Notation: `WP-g.n` — Eingang (Spec/Bestand) → Ausgang (Dateien) → Nachweis. Pfeil `⇐` = Abhängigkeit.

### G-1 Port-Katalog, Fs-Fakes, Workspace

| WP | Inhalt | Nachweis |
|---|---|---|
| WP-1.1 | `contextra-durable-fs` in `members` und `default-members` aufnehmen; Build grün | `cargo check -p contextra-durable-fs` |
| WP-1.2 | `persistence` eigenständig: Feature `persistence` in `contextra-ports`, übrige Abhängigkeiten optional (SA-1); Doppel-`Clock` entfernen | `cargo tree -e normal -p contextra-storage` ohne `contextra-types`, `blake3`, `serde`, `futures-util` |
| WP-1.3 | Fehlende Typen/Traits anlegen (IF-1); `Cipher` 24 Byte; `KeyProvider` auf `ClusterId`; `File::{write_all_at (SA-3), truncate}`; optionale `hard_link`/`reflink` (SA-2) ⇐ WP-1.2 | Kompilierung des Kerns gegen den neuen Port |
| WP-1.4 | `MemFs`, `FaultFs` gegen den neuen `Fs`-Vertrag; `fs_suite` vollständig (Torn-Modus, `FsCaps`, `punch_hole`) ⇐ WP-1.3 | `fs_mem_test`, Fault-Tests |
| WP-1.5 | Mechanische Verschiebung nach Zielbaum (§4.1) per `git mv`; `tests/layering.rs` auf §4.2 anpassen ⇐ WP-1.3 | `cargo test -p contextra-storage --test layering` |
| WP-1.6 | `Runtime` in den Port (SA-9), `Governor` bleibt im Kern | Test `Runtime::manual()` deterministisch |
| WP-1.7 | Konsumentenmatrix gegen Spec §26 abgleichen und SA-6/SA-7/SA-10 einreichen | §2.4 dieses Dokuments |
| WP-1.9 | `tests/invariants.rs`: Parser auf das Format `\| INV-… \|` aus Spec §18 umstellen (59 Invarianten); Test muss danach grün sein, ohne die Schwelle (≥ 35) zu senken. Umgesetzt: 59 Invarianten gefunden, 33 zitiert; die 26 noch nicht zitierten stehen in der Ausnahmeliste `NOCH_OFFEN`, die nur schrumpfen darf (der Test schlägt fehl, sobald ein Eintrag zitiert wird oder nicht in §18 steht) | `cargo test -p contextra-storage --test invariants` |
| WP-1.8 | entfällt: O-12 ist mit SA-15 entschieden (Maintainer-Freigabe statt zwei Reviewer) | Eintrag in Spec §24 (erledigt) |

### G-2 Formate, Checksummen, Seal-Rahmen, Schlüsselhierarchie

| WP | Inhalt | Nachweis |
|---|---|---|
| WP-2.1 | `tools/reference/generator.py` anlegen (existiert nicht, §2.5): unabhängiger Referenzcodierer für FORMAT, Frame, Run-Kopf, Seal-Rahmen | Goldens in `tests/format/` aus dem Generator, nicht aus dem Kern |
| WP-2.2 | `codec` gegen Spec §10 prüfen und ergänzen (Varint, Tag, Comparator, Limits, Checksum-Port) ⇐ WP-1.3 | `golden_anhang_a` bleibt grün; neue FORMAT-Goldens (53 Byte bei `Plain`, Spec §10.2) |
| WP-2.3 | `keys/`: `hierarchy`, `keys_file`, `cluster_keys`, `key_cache`, `nonce`, `rotation` auf Clusterschlüssel und 24-Byte-Nonces umbauen ⇐ WP-1.3 | `keys`-Tests; `KeyDomain`-Treffer = 0 |
| WP-2.4 | `contextra-crypto`: `Cipher` (24 Byte), `Mac`, `Kdf`, `Hasher`, `FileKeyProvider` (IF-3) ⇐ WP-1.3 | `cipher_suite`, `mac_suite`, `kdf_suite`, `key_provider_suite` |
| WP-2.5 | Test-Primitive nach `contextra-testkit`; `testing/refimpl.rs` aus dem Kern entfernen (IF-4, D-05) ⇐ WP-2.4 | `cargo tree` ohne `blake3`/`aes-gcm-siv` |
| WP-2.6 | Seal-Rahmen (Spec §10.10), AAD-Regeln, `SealBudget` | Golden `seal_frame`, Manipulationstest |

### G-3 Redo, Gruppen, Gate-Frames, Absturzmatrix

| WP | Inhalt | Nachweis |
|---|---|---|
| WP-3.1 | Redo-Segmentkopf neu ableiten (`root_gen`, `chain_seed` 4/16 Byte, §10.4); Hook-Punkte der Absturzmatrix festlegen | Golden Segmentkopf |
| WP-3.2 | Frame-Codec mit Siegelbit 7, Gruppen, Torn-Tail (INV-LOG-8), `redo_align` | Property-Test Frame-Roundtrip; Fuzz |
| WP-3.3 | `engine/committer/{intake,group,gate,idem}` nach §5.1; Gate-Frame (§11.3) | INV-COMMIT-1/2/3, INV-STAMP-*, C-17 |
| WP-3.4 | Zustände `Ready/Degraded/Poisoned` (§5.2) | je Fehlerklasse ein `FaultFs`-Test |
| WP-3.5 | Absturzmatrix X-01…X-15 (§5.9) als Treiber `testing/crash` | Matrix grün auf `FaultFs` |

### G-4 Catalog, Anker, Recovery

| WP | Inhalt | Nachweis |
|---|---|---|
| WP-4.1 | `catalog/{snapshot,edits,log,rollover,current,reservations}` (§5.5) | INV-CAT-1/3; Absturz X-04, X-09, X-10 |
| WP-4.2 | `env/anchor`, `FileAnchor` in `contextra-durable-fs` (IF-2) | `anchor_suite` |
| WP-4.3 | `recovery/` R1–R10 (§5.3) | Absturzmatrix X-05…X-08, X-11 |
| WP-4.4 | Restore (`Storage::restore`) und `instance_id`-Prüfung (SA-13) | Restore-Test: abweichender Anker ⇒ Ablehnung |

### G-5 Lesen und Schreiben

| WP | Inhalt | Nachweis |
|---|---|---|
| WP-5.1 | `engine/{memtable,version,views}` (lock-frei, `crossbeam-skiplist`) | Modelltest gegen `BTreeMap` |
| WP-5.2 | `store/{run,blob}` Lesepfad, Blockcache, Bloom | Goldens Run/Blob; Prüfwert-Verletzung ⇒ Quarantäne (§5.3) |
| WP-5.3 | `api/{view,cursor,page}`, Iteratoren, Präfixe, `estimate` | Zeitreise-Tests (`view_at`) |
| WP-5.4 | `maint/flush`, `maint/compaction` (§5.4, §5.8) | Absturz X-07, X-08 |
| WP-5.5 | Adapter hinter Feature je Konsument (§7) | Konsumententests laufen mit beiden Backends |

### G-6 Löschung

| WP | Inhalt | Nachweis |
|---|---|---|
| WP-6.1 | `erase/{gate,cluster_table,links,protocol}`, `maint/purge` (§5.7) | INV-ERA-1…9; Absturz X-12 |
| WP-6.2 | `erase/ledger`, Anker-Fortschritt, `ErasureReceipt` | Absturz X-14; Receipt-Golden |
| WP-6.3 | Löschorakel (`erase/oracle`, §15.11) und Scrub | Orakelbericht über alle Stufen |
| WP-6.4 | Checkpoint/Rekey-Klon (§15.10), `pack_drop` (§15.12) | Checkpoint-Tests je Modus |
| WP-6.5 | Konsumentenmatrix zeigt auf den Kern; `contextra-store` entfernen | Gate-Bedingung Spec §26.4 |

### G-7 / G-8 / G-F (Kurzfassung)

- **G-7:** Rotation, `rekey_clone`, Zeroize-Nachweis, Härtung des Standard-`KeyProvider`.
- **G-8:** `loom`/`shuttle` für Sperrränge (INV-LOCK-2), Messung H-1…H-18, Festlegung der Standardwerte (§20.2); `OP-1…OP-n` aus diesem Dokument (z. B. `OP-3`, Intervall des aufgeschobenen `sync_data` bei reinen `Relaxed`-Gruppen).
- **G-F:** Format-Freeze (Spec §10.12); alle Goldens verifiziert.

---

## 9. Teststrategie

| Ebene | Werkzeug | Gegenstand |
|---|---|---|
| Einheit | `cargo test`, `proptest` | Codecs total (P-4), Comparator, Tag, Limits |
| Golden | `tools/reference/generator.py` (WP-2.1) | FORMAT, Frame, Run-Kopf, Seal-Rahmen; Primitive aus Anhang A.1 |
| Absturz | `FaultFs` | Matrix X-01…X-15 (§5.9), Torn-Writes, ENOSPC, `S`-Fehler |
| Invarianten | `tests/invariants.rs` | jede ID aus Spec §18 mit Test, ID im Kommentar |
| Schichten | `tests/layering.rs` | `use crate::…` gegen §4.2; `cargo tree -e normal` gegen Allow-Liste |
| Nebenläufigkeit | `loom`, `shuttle` | Sperrränge, Committer/Intake, Claims (G-8) |
| Mutation | `cargo-mutants` | Schwellen Spec §20.4 (derzeit **nicht installiert**, CLAUDE.md §9) |
| Allokation | zählender Allokator | Heißpfadbudget (P-11, Spec §20.5) |
| Konformität | `testing::conformance` | je Port eine Suite (IF-1) |
| Sprache | `tools/doc_lint.py` (sofern vorhanden) | REQ-DOC-2 |

Regeln: Absturztests laufen nur auf `FaultFs`, nie auf Stubs (AGENTS.md §5.10). Leistungswerte sind Hypothesen, bis ein Messbericht in `evidence/` vorliegt (Spec §21). Ein Gate gilt erst, wenn seine Tests in der CI grün sind (Spec §22).

---

## 10. Risiken

| ID | Risiko | Wirkung | Gegenmaßnahme |
|---|---|---|---|
| R-1 | Port-Breaking-Change `Cipher` (12 → 24 Byte) und `KeyProvider` | andere Port-Nutzer brechen | nur der Kern implementiert die alten Traits (§2.3); Änderung in **einem** Commit mit Kernanpassung (WP-1.3) |
| R-2 | Transitive Abhängigkeiten von `contextra-ports` verletzen §3.5 | Allow-Liste nicht prüfbar | Feature `persistence` (SA-1); CI-Prüfung `cargo tree` |
| R-3 | `tools/reference/generator.py` fehlt | keine unabhängigen Goldens, G-2/G-F blockiert | WP-2.1 zuerst in G-2 |
| R-4 | 24-Byte-Nonce passt nicht zu AES-GCM-SIV | Krypto-Wahl für `contextra-crypto` | UE-10: XChaCha20-Poly1305; Entscheidung vor WP-2.4, Freigabe nach §1.5 durch den Maintainer |
| R-5 | OS-Lock ohne `unsafe`/`libc` | `DirLock` in `durable-fs` ist kein echter Lock | `std::fs::File::try_lock` kompiliert und läuft auf `rustc 1.97.0` (gemessen); ob es schon unter der MSRV 1.89 stabil ist, ist vor WP-4.2 per CI-Job zu prüfen |
| R-6 | Konsumenten nutzen mehr API als Spec §26.3 abbildet | Adapter unvollständig | SA-7; Matrix in G-1 abschließen |
| R-7 | `contextra-store` (27 898 Zeilen) und Kern laufen parallel | doppelter Wartungsaufwand bis G-6 | Store einfrieren (Archiv-Hinweis, §7.4), nur Sicherheitsfixes |
| R-8 | Gleichzeitige Arbeit an der Spec und am Code | Drift | Spec zuerst (§1.5); SA-n-Liste (§11) als Eingangskorb |
| R-9 | Reviewerpflicht (O-12) | entfällt mit SA-15 | erledigt (Maintainer-Freigabe) |
| R-10 | Zeilenlimit P-6 (≤ 400) bei `committer` (1 103 Zeilen im Bestand) | Verstoß gegen P-6 | Aufspaltung in WP-1.5 |

---

## 11. Änderungsanträge an die Spec (SA-n)

Eingang für den Prozess nach Spec §1.5. Jeder Antrag nennt Stelle, Befund, Vorschlag. Bis zur Annahme gilt die Spec unverändert.

| ID | Stelle | Befund | Vorschlag |
|---|---|---|---|
| SA-1 | §3.5, §4 | `contextra-ports` zieht `contextra-types`, `blake3`, `serde`, `futures-util` als Crate-Abhängigkeit; der Kern verletzt die Allow-Liste transitiv | Feature `persistence`, übrige Abhängigkeiten optional; Kern mit `default-features = false` |
| SA-2 | §4, §8.6 | `Fs` hat `hard_link` (Bestand) ohne Spezifikation; `reflink` fehlt | `hard_link`, `reflink` als Methoden mit Standard `Err(Unsupported)` |
| SA-3 | §4 | `File::write_at`/`truncate` (Spec) vs. `write_all_at`/`set_len` (Bestand); `write_at` klingt wie `pwrite` mit Teilschreiben | Spec: `write_all_at`; Bestand: `set_len` → `truncate` |
| SA-4 | §4, P-11 | `Cipher::seal` liefert `Vec<u8>`: eine Allokation je Versiegelung | `seal_into`/`open_into` mit Standardimplementierung |
| SA-5 | §3.2 | Spec führt `MemFs`/`FaultFs` unter `env`; Bestand unter `testing` (Feature) | Unter `testing` belassen; §3.2 und `std::fs`-Regel anpassen |
| SA-6 | §26.1, §26.2 | `contextra-checkpoint` importiert im Produktivcode kein `contextra_store`; Rollback und letzte TxId laufen über den Port `contextra_ports::StorageEngine` (nicht über einen eigenen Trait, Korrektur zu §2.4); Prod-/Test-Nutzung nicht unterschieden | §26.1 um Prod/Test-Spalte ergänzen; §26.2-Zeile für `checkpoint` korrigieren. Textänderung formuliert (WP-1.7), Freigabe offen |
| SA-7 | §26.3 | Nicht abgebildet: `scanner::lsm_purgers/lsm_scanners` (4 Prod-Stellen), `snapshot_registry.min_active_seqno` (2 Prod-Stellen), `register_at` (nur Test), `last_tx_id` (Port-Methode), `inner_storage` (Adapter-Handle), `PressureLevel` (Prod), `DurabilityConfigError` (Prod) | Zeilen in §26.3 ergänzen (Ziele: `EraseLevel`/`Eraser`, `EraseReq.views`, `View::stamp()`, `Pressure`, `Error::Config`). Die Namen `last_stamp`/`oldest_view_stamp` aus IF-8 stehen nicht in der Spec. Textänderung formuliert (WP-1.7), Freigabe offen |
| SA-8 | §3.1, §3.2, §26.2 | `RealFs` laut §3.1/§26.2 in `contextra-durable-fs`, laut §3.2 in `env/real_fs.rs`; der Kern braucht `RealFs` für `Env::real` | `RealFs` im Kern; `contextra-durable-fs` liefert `FileAnchor` und optionale gehärtete `Fs` |
| SA-9 | §4 | `Runtime` steht nur in den „Verträgen“, nicht im Codeblock; Bestand hat `Runtime` im Kern | Trait `Runtime` mit Signatur in den Codeblock; `Governor` bleibt im Kern |
| SA-10 | §23.2 D-30, §26.3 | bestätigt: `MergeOperator` ist produktiv nirgends verdrahtet. Nur `WasmMergeOperator` (öffentlich exportiert, `contextra-engine/src/lib.rs:56`) und zwei Aufrufe von `with_merge_operator` in `tests/wasm_merge_operator.rs`. Register C-34 („`contextra-engine` nutzt `MergeOperator`“) ist im Code nicht belegt | Prüfauftrag in D-30 als erfüllt markieren; Ersatz nur für den Adapter. §26.3 ergänzt (WP-1.7), Freigabe offen; D-30 und C-34 in dieser Änderung nicht berührt |
| SA-11 | §26.1 | Zeilenzählung erfasst nur Pfadnennungen. Prod/Test: engine 26/118, db 0/25, graph 0/16, cognition 0/15, contextra 2/3, agent 3/0, checkpoint 0/2, bench 0/1. Methodenaufrufe auf Werten aus anderen Crates (agent: 35 Aufrufe von `inner_storage()`) fehlen | Prod-/Test-Tabelle in §26.1 (Textänderung formuliert, WP-1.7), Freigabe offen. Umstellungsaufwand nicht aus Zeilen ableiten; Zählung über die API-Liste (IF-8) |
| SA-12 | §4 | `KeyError` hat im Bestand nur `NotFound`, `Unavailable`; `destroy`/`probe` brauchen weitere Fehlerfälle | Varianten festlegen (z. B. `DestroyFailed`, `Unsupported`); keine Inhaltsfelder |
| SA-13 | §14.3, §16.7 | `AnchorValue` ohne `instance_id`, §14.3 prüft sie „gegen den Anker“ | Anker je Instanz trennen (Dateiname/Schlüssel aus `instance_id`) oder Feld aufnehmen |
| SA-14 | §22 G-2, §19.4 | `tools/reference/generator.py` existiert nicht | Als Lieferung von WP-2.1 festhalten; G-2-Bedingung unverändert |
| SA-15 | §1.5, §22 G-1, §24 O-12 | Die Pflicht zu zwei benannten Reviewern hält den Ablauf auf (ein Maintainer, O-12 blockiert G-1) | Entfällt; Freigabe durch den Maintainer, Goldens und Freeze-Liste-Prüfung bleiben Pflicht. Von der Spec übernommen (§27) |

---

## Anhang A: Offene Parameter

| ID | Parameter | Entwurfswert | Messung |
|---|---|---|---|
| OP-3 | Aufschubintervall von `sync_data` bei reinen `Relaxed`-Gruppen | offen | G-8 (Durabilität gegen Latenz) |

Weitere `OP-n` werden bei Bedarf in den Arbeitspaketen angelegt; Zahlen daraus sind Hypothesen (Spec §21).

## Anhang B: Messbefehle (Stand 22.11.2026)

```bash
# Zeilen und Dateien je Modul des Bestands
for m in codec env keys redo; do find crates/contextra-storage/src/$m -name '*.rs' | xargs cat | wc -l; done
find crates/contextra-storage/src -name '*.rs' | xargs cat | wc -l                 # 17 525
# Port-Modul
wc -l crates/contextra-ports/src/persistence.rs                                     # 352
rg -n 'nonce: &\[u8; ?(12|24)\]|fn (write_all_at|set_len|hard_link)|struct KeyDomain' crates/contextra-ports/src/persistence.rs
rg -l 'contextra_ports::persistence' --glob '*.rs' crates | wc -l
# Konsumenten
rg -c 'contextra_store' --glob '*.rs' crates benchmarks
rg -l 'contextra-ports' crates/*/Cargo.toml benchmarks/*/Cargo.toml | wc -l       # 27
# Workspace
rg -n 'durable-fs|contextra-storage' Cargo.toml
ls tools/reference/generator.py                                                     # nicht vorhanden
```

Die Zahl für `contextra-store` (27 898 Zeilen `src`) stammt aus `find crates/contextra-store/src -name '*.rs' | xargs cat | wc -l`. Eine Modulzählung der Konsumentenprüfung kam auf 18 203 Zeilen; die Differenz (u. a. Testdateien unter `src/`, 10 240 Zeilen) ist **nicht vollständig aufgeklärt**, ändert aber keine Entscheidung dieses Dokuments.

## Anhang C: Herkunft der Befunde

Die Zählungen in §2.2–§2.4 wurden mit `rg`, `wc` und `cat` am Klon `940ff86` erhoben. Stichproben gegen den Quelltext (Nonce-Länge, `hard_link`, `write_all_at`/`set_len`, Zeilenzahl von `persistence.rs`, Konsumentenzeilen, Workspace-Mitgliedschaft, Existenz des Referenzcodierers) wurden direkt wiederholt. Angaben, die nur auf Signaturen oder Grep-Treffern beruhen (z. B. Vollständigkeit von `MemFs`/`FaultFs` gegenüber dem `Fs`-Vertrag), sind als „zu prüfen“ gekennzeichnet.
