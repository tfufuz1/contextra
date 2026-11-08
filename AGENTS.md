# AGENTS.md — Contextra Baukasten (Root)

> Gilt für das gesamte Repository `tfufuz1/contextra`. Jede `crates/*/AGENTS.md` ergänzt diese Datei um crate-spezifische Verträge und **wiederholt sie nicht** (DRY). Bei Widerspruch gilt: Crate-Datei vor Root-Datei, `docs/SPEC.md` §0.6 vor beiden.
>
> **Stand:** 2026-11-07, Basis `2127d457e`. Alle Ist-Angaben stammen aus statischer Lektüre (`Cargo.toml`, `capabilities.toml`, Quellbäume, `grep`). **Nichts davon wurde kompiliert.** Zahlen sind Heuristiken (Evidenzstufe E1, SPEC §1.4) und vor Verwendung als Beleg mit dem genannten Befehl neu zu messen.

## 1. Rolle & Auftrag

Du agierst als **Principal Rust Engineer**: Systemprogrammierung, Durability, Concurrency, RAG-Speicherkerne. Du verantwortest architektonische Integrität, Speichersicherheit und deterministische Performance eines Multi-Crate-Ökosystems.

**Auftrag:** Das historisch gewachsene, verdrahtete System wird zu einem **orthogonalen Baukasten** umgebaut (`PLAN.md`, Phasen 0–7). Jeder Crate ist

- **isoliert** nutzbar, ohne weiche Abhängigkeit oder Querverdrahtung,
- **portabel**: ohne Modifikation und ohne Setup-Ritual in ein Fremdprojekt importierbar,
- **entschuldet**: historische Domänenlogik (vor allem die kaskadierende Löschkette) liegt in einem dafür vorgesehenen Crate, nicht im Kern,
- **messbar**: einzeln testbar, benchmarkbar und profilierbar.

**Haltung:** Kein fehlerhafter Zustand wird akzeptiert. Liefere keine bloße Fehlerbeschreibung, sondern einen Lösungsvorschlag mit belegbarer technischer Begründung. Löse das Problem, suche keine Schuldigen.

## 2. Quellen der Wahrheit & Vorrang

| Frage | Maßgebliche Quelle |
|---|---|
| Was ist **jetzt** im Code? | der Code selbst (`cargo metadata`, `cargo tree`, Quellen). Berichte und Skriptausgaben sind Hypothesen (INV-EVIDENCE-1). |
| Ring, Klasse, Reifegrad, `may_depend_on` | `capabilities.toml` (einzige Quelle, INV-DOC-SINGLE-SOURCE-1). **Lücke:** `contextra-durable-fs` fehlt dort (D6). |
| Wohin soll der Umbau? | `PLAN.md` (Phasen, Gates G0–G7, Baukasten-Vertrag M1–M11) und `docs/SPEC.md` v3.0 §12 sowie §2.6 |
| Normative Regeln, Invarianten, Vetos | `docs/SPEC.md` §3 (INV-*), `docs/VETOES.md`, `docs/CONSTITUTION.md` |
| Technische Schulden | `DEPRECATED_DEBT.md` (DEBT-01 bis DEBT-05) |
| Gates | `governance/gates.toml`, `.github/workflows/` |

**Regeln:**

1. **Verifiziere, bevor du handelst.** API-Namen, Pfade und Signaturen am Code prüfen, nie aus dem Gedächtnis oder aus alten Dokumenten übernehmen.
2. **Nicht raten.** Fehlt eine Angabe oder widersprechen sich Quellen (siehe §10): Fail-Closed wählen, Annahme in einer Zeile benennen, als offenen Punkt melden.
3. **Code ist Ist, PLAN/SPEC sind Soll.** Wo beides auseinanderläuft, steht in der Crate-Datei jeweils „Ist“ und „Soll“ getrennt. Gib Soll nie als Ist aus.
4. Normativ sind Sätze mit MUSS, DARF NICHT, SOLL, KANN (SPEC §0.6). Zahlen mit „Startwert“ sind kalibrierbar.

## 3. Baukasten-Vertrag (PLAN §3, M1–M11)

Jeder Crate erfüllt diese Regeln. Die Spalte „Prüfung“ nennt einen Befehl, der **heute** funktioniert.

| ID | Regel | Prüfung |
|---|---|---|
| M1 | Abhängigkeiten zeigen nur auf gleiche oder niedrigere Ringe und nur auf Kanten aus `may_depend_on`. | `cargo tree -p <crate> -e normal --depth 1`, Abgleich mit `capabilities.toml` |
| M2 | Jeder Baustein baut und testet isoliert. | `cargo build -p <crate> --no-default-features --locked` und `cargo test -p <crate> --locked` |
| M3 | Öffentliche Signaturen nutzen nur Typen aus `types`, `ports` oder dem Crate selbst. Kein `pub use` fremder Implementierungstypen. | `grep -n "pub use contextra_" crates/<crate>/src/lib.rs` |
| M4 | Umwelt nur über Ports: I/O `Vfs`, Zeit `Clock`, Zufall `Rng`, IDs `IdGen`, Metriken `MetricsSink`. | `cargo clippy -p <crate> -- -D warnings` (`clippy.toml`: `disallowed-methods`) |
| M5 | `default` ist minimal. `onnx`, `candle`, `wasmtime`, `tokio`, `pyo3` sind nie im Minimalbau erzwungen. | `cargo build -p <crate> --no-default-features` |
| M6 | Keine Dev-Abhängigkeit auf höhere Schichten. | `[dev-dependencies]` in `Cargo.toml` lesen |
| M7 | Datenhalter implementieren nur `Purger` und `LayerScanner`. Saga, Nachweis, Rechtsbezug und Reporting liegen ausschließlich in `contextra-erasure`. | siehe §5 |
| M8 | Ring 0 und Speicherkerne halten ihre Logik synchron. Async nur an der Port-Grenze. | `grep -rn "async fn\|tokio::" crates/<crate>/src` (Ratchet: darf nicht steigen) |
| M9 | `unsafe` nur in `simd`, `sys`, `wire`, jeweils mit `// SAFETY:`. Sonst `#![forbid(unsafe_code)]`. | `grep -L "forbid(unsafe_code)" crates/*/src/lib.rs` (Ist: genau diese drei Treffer) |
| M10 | Eigenständige SemVer-Linie je Baustein. | `Cargo.toml` (heute setzen **alle** Crates `version.workspace = true`, WP 6.3) |
| M11 | Jede `README.md` enthält Entkopplungsprofil, Schnittstellen und ein Minimalbeispiel für Fremdprojekte. | Review |

## 4. Ring-Modell & Crate-Atlas

Richtung: **Ring 4 → 3 → 2 → 1 → 0**. Aufwärtskanten und Zyklen sind verboten (P1, P5). Kern `[K]` hängt nie von Rand `[R]`, Experimentell `[E]` oder Archiv `[A]` ab (P30).

```
Ring 4  Oberflächen     contextra (Fassade)   mcp   py
Ring 3  Orchestrierung  engine   erasure   db   cognition   router   privacy   agent   ingest
Ring 2  Inferenz/Sandbox infer-onnx   infer-candle   infer-ollama   sandbox
Ring 1  Persistenz       store   vector   text   graph   checkpoint   kvcache
Ring 0  Fundament        types   ports   mvcc   crypto   rank   adapt   durable-fs   | Inseln: sys  simd  wire
Werkzeug                 testkit   contextra-bench
Extension                license   audit-export   avv-generator
Auslaufend               core  (Strangler-Fassade, DEBT-03)
```

Spalten: **Soll** = Ring laut `PLAN.md` §4 / SPEC §4. **cap.** = Ring in `capabilities.toml`. `≠` markiert Abweichung (Arbeit für WP 1.1).

| Crate | Soll | cap. | Unsafe | Wichtigste Restschuld (Ist, belegt) |
|---|---|---|---|---|
| `types` | 0 | 0 | nein | `erasure.rs` (516 Zeilen Löschdomäne) und `witness.rs` im Fundament; doppelte Typnamen mit `ports` (K3) |
| `ports` | 0 | 0 | nein | zwei Typfamilien `LayerWitness` (`erasure.rs`, `scanner/witness.rs`); `Vfs` synchron (PLAN-Snippet zeigt async) |
| `sys` | 0 | 0 | **ja** | Insel (`#![allow(unsafe_code)]`, 20 `unsafe`-Treffer), `#![deny(unsafe_op_in_unsafe_fn)]`; Win32-Modul `acl_win32.rs` |
| `simd` | 0 | 0 | **ja** | Insel (`#![allow(unsafe_code)]`, ~230 `unsafe`-Treffer in `kernels/`); Miri- und Referenztest-Abdeckung je Kernel belegen |
| `wire` | 0 | 0 | **ja** | Insel (`#![deny(unsafe_code)]` mit lokalen Ausnahmen); generierter Code (`generated.rs`, `contextra_generated.rs`); 16 `unwrap/expect`-Treffer laut Heuristik trotz `deny(clippy::unwrap_used)` prüfen |
| `durable-fs` | 0 | **fehlt** | nein | nicht in `capabilities.toml` (D6); hängt nur von `sys`, **nicht** von `ports` (Vfs-Implementierung offen, WP 2.1) |
| `mvcc` | 0 | 0 | nein | autark; Dev-Kante auf `testkit` |
| `crypto` | 0 | 0 | nein | **88 `DeletionProof`- und 31 `kv_shredding`/`revocation_log`-Treffer** (Phase 3.1 offen) |
| `rank` | 0 | 0 | nein | enthält `calibration/`, `dibud/`, `drift.rs` neben `fusion/` |
| `adapt` | 0 | 0 | nein | autark; Features `bandit-routing` u. a. |
| `store` | 1 | 1 | nein | ~176 `tokio::`-Treffer, 77 direkte `fs`-Treffer, `kv_shredding`-Bezüge (28); Dev-Kanten auf `graph`, `sandbox`, `vector`; kein Dep auf `durable-fs` |
| `vector` | 1 | **0 ≠** | nein | Dep auf `crypto` (`wal_crypto`), 28 direkte `fs`-Treffer; implementiert `Purger`/`LayerScanner` |
| `text` | 1 | **0 ≠** | nein | autark; implementiert `Purger`/`LayerScanner`; 43 `async fn` |
| `graph` | 1 | **0 ≠** | nein | Dep auf `adapt` laut `capabilities.toml`, nicht mehr in `Cargo.toml`; Dev-Kante auf `store` |
| `checkpoint` | 1 | 1 | nein | `capabilities.toml` nennt noch `contextra-core`; Dev-Kanten auf `store`, `mvcc` |
| `kvcache` | 1 | 1 | nein | 6 `kv_shredding`-Treffer |
| `infer-onnx` / `-candle` / `-ollama` | 2 | 2 | nein | `candle` hängt an `crypto` (Ziel: nur `ports`, `types`, `rank`) |
| `sandbox` | 2 | 2 | nein | keine Contextra-Abhängigkeit; `wasmtime` nur hinter Feature halten |
| `engine` | 3 | 3 | nein | 14 interne Deps; kein `Backends`-Trait; `contextra_impl/erasure.rs` (618 Zeilen); `pub use` von `crypto::deletion_proof` und `store::lsm::DurabilityMode` |
| `erasure` | 3 | 3 | nein | **kein Konsument im Workspace**; Dep nur `crypto`, `ports`, `types` (Soll) |
| `db` | 3 | 3 | nein | zieht `sys` (Ziel: nur `engine`, `ports`, `types`); 10 Dev-Kanten |
| `cognition` | 3 | 3 | nein | zusätzliche Kante auf `graph` |
| `router` | 3 | 3 | nein | Dev-Kanten auf `db`, `graph`, `text`, `vector` |
| `privacy` | 3 | 3 | nein | 6 `kv_shredding`-Treffer; der Port `EgressPolicy` (WP 2.4) existiert in `ports` **noch nicht** (in `privacy` nur als `BlockReason::EgressPolicyDenied`) |
| `agent` | 3 | 3 | nein | harte Kanten auf `checkpoint`, `db`, `graph`, `router`, `store` |
| `ingest` | 3 | **4 ≠** | nein | Dep auf `engine` (Ziel: `chunker` extrahieren, WP 5.4; `contextra-chunker` existiert nicht) |
| `contextra` | 4 | 4 | nein | Fassade hängt hart an `infer-candle`, `license`; Ziel: nur optional |
| `mcp` | 4 | 4 | nein | 8 Kanten über das Soll (`crypto`, `privacy`, `rank`, `router`, `wire` hart; `infer-*` ×3 optional); Lösch-Werkzeuge in `server_tools.rs` |
| `py` | 4 | 4 | nein | Dep auf `db` statt auf die Fassade (dokumentierte Ausnahme `ANCHOR:ARCH:DAG-003` in `Cargo.toml`); Workspace-Profil `release` nutzt `panic = "unwind"` für FFI, Binaries bauen mit Profil `release-abort` |
| `core` | – | 0 (`deprecated`) | nein | nur noch 256 Zeilen Fassade; in der Fassade `contextra` und `checkpoint` per `capabilities.toml` noch erlaubt |
| `license`, `audit-export`, `avv-generator` | Ext. | 4 / 0 / 0 | nein | Ring von `audit-export` und `avv-generator` (0) widerspricht ihrer Rolle |
| `testkit` | Tool | Tooling | nein | nur als `dev-dependency` einbinden |

## 5. Löschkette: Grenze und Restschuld

**Regel (M7, PLAN Phase 3):** Datenhaltende Kerne (`store`, `vector`, `text`, `graph`, `kvcache`, `checkpoint`) kennen weder DSGVO, Sagas, Deletion-Proofs noch Berichtsformate. Sie bieten **mechanische** Bereinigung (`Purger`) und **Messung** (`LayerScanner`). Alles Übrige gehört nach `contextra-erasure`.

**Ist-Zustand (gemessen mit `grep -rIiE`, nur `src/`):**

| Ort | Befund | Ziel-WP |
|---|---|---|
| `crypto` | `deletion_proof/`, `ed25519_proof.rs`, `kv_shredding.rs`, `revocation_log.rs` liegen noch hier (PLAN 3.1). `kv_segment/` und `kv_cipher.rs` nennt der PLAN nicht; sie dienen `kvcache` und brauchen eine Zuordnungsentscheidung (ADR) | 3.1 |
| `store` | `kv/segment.rs`, `kv/delete_mode.rs`, `compaction/engine.rs`, `compaction/adaptive.rs` mit Shredding-Bezug | 3.2 |
| `engine` | `contextra_impl/erasure.rs` (Orchestrator, `ProofKeyProvider`), `pub use crypto::deletion_proof::{DeletionLayer, DeletionProof, …}` | 3.5 |
| `contextra` (Fassade) | `deletion_chain_e2e.rs`, `KvDeleteMode`, 5 `forget`-Treffer | 3.8 |
| `mcp` | 18 `forget`-Treffer in Werkzeugen, `proof_key.rs` | 3.7 |
| `kvcache`, `privacy` | je 6 Shredding-Treffer | 3.3/3.6 (Zuordnung klären) |
| `text`, `vector`, `graph` | **bereits sauber** (0 `DeletionProof`/`DSGVO`); implementieren `Purger`/`LayerScanner` | erledigt, Contract-Tests ergänzen (WP 2.5) |
| `erasure` | vollständig vorhanden (Saga, Vault, Gate, Journal, Ledger, Measure, Report, Restore), **ohne Aufrufer** | 3.9 |

**Verbote für neuen Code:**

- Kein neuer Deletion-Proof-, Saga-, Vault-, Ledger-, Audit- oder Rechtstext-Code außerhalb von `contextra-erasure` (Ausnahme: die reinen Port-Typen in `contextra-ports`).
- Keine neue Abhängigkeit auf `contextra-crypto::deletion_proof`, `kv_shredding` oder `revocation_log`. Wer sie noch braucht, trägt die Stelle in `DEPRECATED_DEBT.md` (DEBT-01) ein.
- `contextra-erasure` importiert **nie** `store`, `vector`, `text`, `graph`. Es spricht nur über Ports.

**Wording (SPEC §2.4a, §2.6):** In Code, Doku und Fehlertexten kein „garantiert“, „100 % gelöscht“, „DSGVO-konform“, „unwiederbringlich“. Formuliere technisch und messbar („Schlüssel vernichtet“, „Scan fand 0 Treffer in Schicht X unter Annahmen A“). Ausschlüsse (Flash-Wear-Leveling, Swap, RAM, externe Backups) stehen im Bericht, nicht in Marketingtext.

## 6. Vetos & harte Invarianten

Diese Nummer (§6) ist stabil, weil `docs/VETOES.md` darauf verweist.

**Vetos (unverhandelbar, Änderung nur per ADR):**

- **VETO-F10 – Keine mandantenübergreifenden Datenflüsse.** Kein Cache, Index, Embedding, keine Statistik und kein Audit-Eintrag überschreitet Tenant-Grenzen. Der Tenant kommt aus `RequestContext` bzw. der Transaktion, nie aus Literalen, Defaults oder Nutzerparametern (INV-TENANT-1).
- **VETO-F02 – HNSW-Löschung nur per Tombstone + Slot-Nullung + vollständigem Rebuild.** Kein Rewiring der Nachbarn, kein partielles Teilgraph-Rebuild als Löschstrategie. Slots werden erst nach Rebuild-Abschluss wiederverwendet (INV-HNSW-SLOT-1).
- **VETO-OP03 – Kein Echtzeit-Audio/STT** im Kern.
- **Single-Node-Doktrin** – kein Sharding, kein P2P, keine verteilte Replikation, kein Netz-Listener im Kern.

**Invarianten, die jeder Agent kennen muss:**

| Thema | Regel |
|---|---|
| Zero-Panic (P7) | Kein `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!`, `unreachable!`, `assert!` und kein ungeprüftes Indexieren in Produktionspfaden. Fehler sind typisiert (`ContextraError`). Workspace-Lints `unwrap_used`/`expect_used` stehen auf `deny`. |
| Determinismus (P28) | Zeit, Zufall, IDs nur über `Clock`, `Rng`, `IdGen`. Verboten in Kernlogik: `Instant::now`, `SystemTime::now`, `thread_rng`, `std::env::var`, `HashMap`/`HashSet` mit `RandomState` (`clippy.toml`). Ausnahme: Schlüssel-/Salt-Material (`governance/determinism-allow.toml`). |
| WAL zuerst (P24) | Aller persistente Anwendungszustand geht durch das WAL. Weitere Persistenz nur über die Sidecar-Logs (INV-SIDECAR-1). |
| I/O (INV-FSYNC-1) | Datei-I/O nur über den `Vfs`-Port. `fsync`-Fehler werden propagiert, nie mit `let _ =` verworfen; nach einem `fsync`-Fehler wird der Handle vergiftet. |
| Fail-Closed (INV-FAILCLOSED-1) | Unbekannter Zustand, unbekannte Formatversion, gebrochene Kette: Ablehnung mit benanntem Grund. |
| Sicherheit ≠ Feature (A1) | Kein `cfg(not(feature = …))`-Ersatz für Sicherheitscode. |
| Formate (INV-FORMAT-1) | Jede persistente Datei trägt Magic und Version; neuere Version ⇒ Öffnen verweigert. Formate explizit spezifiziert, nicht aus `serde`-Derives abgeleitet. |
| Logs (INV-LOG-1) | Keine Nutzinhalte in Logs, Metriken, Fehlertexten, Panic-Meldungen. |
| Sperren (INV-LOCK-ORDER-1) | Feste Sperrenordnung; kein Guard über `.await` (`clippy::await_holding_lock = deny`). |
| Stdout (INV-STDOUT-1) | `stdout` gehört im MCP-Binär ausschließlich dem Protokoll. |

## 7. Engineering-Prinzipien (verbindlich)

**Orthogonalität & DRY.** Zwei unabhängige Komponenten dürfen sich nicht gegenseitig beeinflussen. Dupliziere keine Typfamilie: eine Typfamilie je Konzept (A2). Vor jedem neuen `struct`/`enum`/`trait`: `grep -rn "struct <Name>" crates/` und `docs/TYPE_REGISTRY.md`.

**Design by Contract.** Jede öffentliche Funktion prüft Vorbedingungen und liefert typisierte Fehler. Dokumentiere `# Fehler`, `# Panics` (= keine) und bei persistenten Operationen `# Crash` (welcher Zustand nach Abbruch gilt). Ungültige Zustände sollen per Typ (Newtype, Typestate, `pub(crate)`-Konstruktor) unkonstruierbar sein, nicht per Laufzeit-Check.

**Keine globalen Zustände.** Konfiguration und Daten leben in parametrisierbaren Strukturen. Kein `static mut`, kein `lazy_static`/`OnceLock` mit veränderlichem Zustand in Bibliothekscode. Nur die Composition Root (`contextra::ContextraBuilder`) instanziiert und verdrahtet.

**Komposition vor Vererbung.** Teile Funktionalität über kleine Traits (Mixin-Prinzip), nicht über tiefe Hierarchien. Delegiere an spezialisierte Dienste. Ein Trait mit Austauschrolle gehört nach `contextra-ports` (Kantenregel 9); crate-interne Hilfs-Traits bleiben `pub(crate)`.

**Law of Demeter, Tell-Don’t-Ask, State-Passing.** Keine Ketten wie `a.b().c().d()`. Gib dem Objekt den Auftrag, statt seinen Zustand abzufragen und außen zu entscheiden. Reiche Zustand als Parameter oder Nachricht durch, statt ihn in einem Monolithen zu stauen.

**Concurrency.**

1. Analysiere den Arbeitsablauf zuerst und maximiere Nebenläufigkeit dort, wo Daten unabhängig sind.
2. **Shared Mutable State vermeiden.** Bevorzuge Message-Passing (Actor/Kanal, z. B. der WAL-Flusher in `store`), unveränderliche Snapshots (MVCC, RCU) und atomare Sequenzen.
3. Wo Locks unvermeidbar sind: dokumentierte Reihenfolge, kurze Haltezeit, nie über `.await`, nie um LLM-/Netzaufrufe.
4. Hot Path generisch/monomorphisiert (Lesepfad von `StorageEngine`, Vektor-Scan), Cold Path (Sagas, Reporting, Worker) über `dyn` (PLAN 5.2).
5. Jede neue Nebenläufigkeit bekommt einen `loom`- oder Property-Test.

**Algorithmische Strenge.** Notiere die Big-O-Komplexität (Zeit und Speicher) im Doc-Kommentar jeder neuen Kernoperation und belege sie mit einem Benchmark (`criterion`). Ohne Messung keine Performance-Aussage (INV-CLAIMS-1).

**Developer Experience.** Benenne Typen, Module und Variablen einheitlich und unmissverständlich. Folge dem Namensraum der Ports (`IdGen`, nicht `IdGenerator`). Keine `mod.rs`-Neuanlage in `contextra-erasure` (D-90). Der Code soll die Absicht ohne Kommentar erkennen lassen.

**Refactoring.** Früh, klein, kontinuierlich. Ein Pull Request = ein Arbeitspaket (WP) = höchstens ein Crate-Schnitt (REQ-CHG-2). Nach jedem Schnitt: Build, Tests, Zeilenzählung. Agil planen, aber systemische Überraschungen antizipieren: lege vor jedem Schnitt **Charakterisierungstests** an (Risiko R2).

## 8. Qualitätssicherung & Verifikationsbefehle

**Prinzip:** Tests entstehen aus Spezifikation und Invarianten, nicht aus der Implementierung desselben Erzeugers. Jede `INV-*` hat einen Test, dessen Verletzung er erkennt (Mutation belegt die Wirksamkeit). Ein nicht ausgeführtes Prüfwerkzeug ist **rot**, nie „bedingt grün“ (EP13).

**Testpyramide je Crate:** Unit (feingranular) → Property-Based (`proptest`, Randfälle und falsche Annahmen) → Differential-/Referenzmodell (`contextra-testkit::reference_model`) → Integration/End-to-End über die Fassade → Chaos (`FaultVfs`) → Fuzz (`cargo fuzz`) → Loom (Nebenläufigkeit) → Miri (Unsafe-Inseln) → Mutation (`cargo mutants`) → Benchmark (`criterion`).

**Regression:** Jeder gefundene Fehler bekommt zuerst einen roten Test (Red→Green), danach den Fix. „Finde denselben Fehler nie zweimal.“

**Befehle, die heute ohne `xtask` funktionieren:**

```bash
cargo fmt --all -- --check
cargo clippy -p <crate> --all-targets -- -D warnings
cargo build  -p <crate> --no-default-features --locked      # M2/M5
cargo test   -p <crate> --locked
cargo tree   -p <crate> -e normal --depth 1                 # M1
cargo deny check                                            # deny.toml
just crate <crate>                                          # check + test eines Crates
just loom-store                                             # Loom, Group-Commit
RUSTFLAGS="--cfg loom" cargo test -p <crate> --locked
cargo +nightly miri test -p contextra-simd                  # Unsafe-Inseln
cargo mutants --config mutants-guarantees.toml -p <crate>
(cd crates/<crate> && cargo +nightly fuzz run <target> -- -max_total_time=60)
cargo bench -p <crate>
```

> **Achtung `xtask`:** `justfile`, `governance/gates.toml`, `.agents/rules/*` und `.cargo/config.toml` (Alias) verweisen auf `cargo xtask …` (`check-ring-layering`, `panic-inventory`, `security-scan`, `check-dag`). Ein Verzeichnis `xtask/` existiert im Repository **nicht** (Befund D1, PLAN WP 0.1/1.3). `just gates` schlägt deshalb fehl. Behelf: die Befehle oben und die statischen Tests `tests/layering.rs`, `tests/unsafe_islands.rs`, `tests/arch/` (sie sind **nicht** als Workspace-Member eingetragen; vor Nutzung prüfen).

**Gate-Integrität (REQ-QA-010):** Gates, Lints, Ratchets und Baselines (`governance/ratchet.toml`, `.github/unwrap_baseline.txt`) werden nie abgeschwächt oder umgangen. Baselines dürfen nur sinken.

**Automatisierung:** Manuelle Schritte sind ein Defekt. Alles Wiederholbare gehört in `just`, ein Skript oder CI. Git-Hooks: `just bootstrap` setzt `core.hooksPath` auf `.githooks`.

## 9. Arbeitsweise für Agenten

1. **Aufgabe lesen**, Ziel und Quelle der Wahrheit bestimmen (§2). Bei Kontingentdruck: SPEC §0 und gezielt den betroffenen Abschnitt lesen, nicht das ganze Dokument.
2. **Ist erfassen**: `cargo tree -p <crate> -e normal`, `grep` nach den Symbolen, vorhandene Tests lesen.
3. **Roten Test schreiben** (oder Charakterisierungstest, wenn Verhalten erhalten bleiben soll).
4. **Kleinste mögliche Änderung** umsetzen. Kein Over-Engineering, keine spekulativen Abstraktionen (A10: keine Fähigkeit vor dem zugehörigen Test).
5. **Verifizieren**: §8-Befehle für den betroffenen Crate **und** für jeden Crate, der ihn konsumiert.
6. **Schuld buchen**: Löst du eine DEBT-Stelle, trage sie in `DEPRECATED_DEBT.md` aus; findest du eine neue, trage sie ein. Keine neuen Querkanten (`DEPRECATED_DEBT.md` §4).
7. **PR-Text** mit WP-Nummer aus `PLAN.md`, Vorher/Nachher-Messung und einer Zeile „Nicht geprüft“ für alles, was du nicht ausführen konntest.

**Nicht erlaubt:** Tests löschen oder abschwächen, um grün zu werden; Snapshots ungeprüft akzeptieren; `#[ignore]` ohne Begründung und Ticket (D5); Gate-Ausnahmen ohne Ablauf-Meilenstein (`dag-exceptions.toml`); Mocks im Produktionscode (INV-EMBED-1, INV-WIRING-2); Features mit Standard-`default` für schwere Laufzeiten.

## 10. Bekannte Konflikte & Doku-Schulden (zu entscheiden, nicht zu raten)

| # | Konflikt | Behandlung bis zur Entscheidung |
|---|---|---|
| K1 | **Vfs sync vs. async.** `PLAN.md` §5.2 zeigt `#[async_trait] trait Vfs`. Code (`ports/src/vfs.rs`) ist synchron, und PLAN M8 / SPEC P26 verlangen synchrone Ring-0-Kerne. | `Vfs` bleibt **synchron**. Async gehört in die Schale (`spawn_blocking`). Snippet im PLAN ist veraltet. |
| K2 | **Signaturen von `Purger`/`LayerScanner`.** PLAN §5.3 / SPEC §12.2: `purge_scope(&KeyScope)`, `scan_for_residual`. SPEC §5.3: `ErasureScope`, `DeletionLayer`. Code: `purge(ctx, &ScanTarget) -> PurgeStats`, `scan_units(&mut ScanSession, ctx)`, `WitnessLayer`. | **Code ist bindend.** Konsolidierung per ADR in WP 2.2. Keine vierte Variante erfinden. |
| K3 | **Doppelte Löschtypen.** `LayerWitness` ist in `ports` zweimal definiert (`erasure.rs:183`, `scanner/witness.rs:17`); `EraseTarget` und `LayerId` stehen sowohl in `types/src/erasure.rs` als auch in `ports/src/erasure.rs`. Verstößt gegen A2 und INV-WITNESS-MINT-1. `types/src/erasure.rs` (516 Zeilen: `EraseRequest`, `EraseReceipt`, `ErasureStatus`, `RetentionAttestation` …) liegt zudem im Fundament, obwohl die Domäne nach `erasure` gehört. | Neue Nutzung nur von `ports::scanner::*`; Vereinigung und Verlagerung per ADR in WP 2.2/3.9. |
| K4 | **SPEC-Altregeln vs. v3.0.** SPEC §0.2 Nr. 11, A2, Kantenregel 5 und §4.3 (`crypto/proof/`) verlangen Löschtypen in `crypto`; §2.6, §12 und `PLAN.md` Phase 3 verlangen sie in `contextra-erasure`. Auch `INV-DELETION-WIRE-1`, G1–G7 (§2.4) und `forget` in `engine` (§4.3) beschreiben noch den Monolithen. | **v3.0-Entscheidung gilt** (PLAN, §2.6, §12). SPEC-Bereinigung in Phase 7; bis dahin Neucode nach §5. |
| K5 | **Archiv-Liste vs. Workspace.** SPEC §0.5/§2.3/Anhang B archivieren `graph`, `cognition`, `router`, `agent`, `adapt`, `kvcache`, `infer-onnx`, `infer-ollama`. `Cargo.toml` führt sie als aktive Mitglieder, `PLAN.md` §4 als Bausteine. | **Workspace und PLAN gelten.** Behandle sie als aktive Bausteine (Klasse laut `capabilities.toml`). |
| K6 | **Versionslabel.** SPEC-Kopf sagt „3.0“, `capabilities.toml` sagt `spec = "v4"`, `.agents/claude/CLAUDE.md` sagt „4.0“. | „3.0“ ist die Referenz dieser Dateien. Label in WP 7 vereinheitlichen. |
| K7 | **Veraltete Verweise.** `.agents/claude/CLAUDE.md`, `.agents/jules/*`, `.agents/rules/*` verweisen auf frühere AGENTS-Abschnitte (§2a, §4) und auf `cargo xtask`. | **Diese Datei ist maßgeblich.** Die Verweise in `.agents/` sind in WP 7 anzupassen. |
| K8 | **Leaks in öffentlichen APIs (M3).** `engine` re-exportiert `crypto::deletion_proof::*` und `store::lsm::DurabilityMode`; `db` re-exportiert `checkpoint`. | Keine neuen Re-Exports. Abbau in WP 3.5/5.3. |
| K9 | **Werkzeug-Crates.** SPEC §4.2 nennt `testkit`, `xtask`, `xtask-heavy`, `bench`. Im Repository existieren `testkit` und `benchmarks/contextra-bench`; `xtask*` fehlen. | `testkit` und `contextra-bench` gelten als Werkzeug und sind nie Abhängigkeit des Kerns. `xtask` siehe §8. |
| K10 | **`governance/security_ports.toml` zeigt auf nicht existierende Pfade** (`contextra_ports::kms::*`, `::purger::Purger`, `::log::DurableLog`, `::journal::ErasureJournal`, `contextra_store::vfs::StdVfs`). `StorageRead`/`StorageWrite` (verlangt von `.agents/rules/async-io.md`) existieren nicht. | Pfade nie aus diesen Dateien übernehmen; am Code prüfen. Korrektur in WP 2.2/7. |

## 11. Diagnose- & Analysewerkzeuge (`tools/`) für Coding-Agenten

Für mikrofeingranulare Codebase-Analysen, Konformitätschecks und Architekturprüfungen stehen spezialisierte Python-Werkzeuge für Introspektion, Fehlerkartierung und automatisierte Dokumentation bereit. Alle Werkzeuge laufen ohne externe Abhängigkeiten (Standardbibliothek), speichern Berichte nachvollziehbar in `docs/diagnostics/` und unterstützen `--json`, `--crate` und `--workspace`:

| Werkzeug | Primäre Prüfung | Ausgabeberichte |
|---|---|---|
| `python3 tools/zero_panic_analyzer.py` | Zero-Panic (P7): Detektiert `unwrap`, `expect`, `panic!`, `todo!`, `assert!` in Prod vs. Test | `ZERO_PANIC_AUDIT.md`, `.json` |
| `python3 tools/unsafe_boundary_auditor.py` | Unsafe-Inseln (M9): Auditiert `unsafe`, erzwingt `// SAFETY:`-Dokumentation | `UNSAFE_BOUNDARY_AUDIT.md`, `.json` |
| `python3 tools/determinism_guard.py` | Determinismus (P28): Findet `Instant::now`, `SystemTime::now`, `thread_rng`, ungesicherte `HashMap`/`HashSet` | `DETERMINISM_GUARD_AUDIT.md`, `.json` |
| `python3 tools/type_registry_sync.py` | Typ-Register (K3/A2): Extrahiert alle Typen und erkennt Namenskollisionen/Shadowing über Crates | `TYPE_REGISTRY_COLLISIONS.md`, `.json` |
| `python3 tools/async_boundary_scanner.py` | Async-Grenzen (M8 / INV-LOCK-ORDER-1): Findet `async`/`tokio` in Ring 0 und Lock-Guards über `.await` | `ASYNC_BOUNDARY_REPORT.md`, `.json` |
| `python3 tools/fsync_durability_tracer.py` | Durability (INV-FSYNC-1 / M4): Findet rohes `std::fs`/`tokio::fs` und ignorierte `fsync` (`let _ =`) | `FSYNC_DURABILITY_AUDIT.md`, `.json` |
| `python3 tools/port_contract_auditor.py` | Verträge & Re-Exporte (M3 / M7): Validiert Port-Traits und unterbindet fremde Re-Exporte | `PORT_CONTRACT_AUDIT.md`, `.json` |
| `python3 tools/dead_code_pruner.py` | Code-Hygiene: Mikrofeingranulare Erkennung toter/unreferenzierter interner Symbole | `DEAD_CODE_ANALYSIS.md`, `.json` |
| `python3 tools/error_topology_mapper.py` | Fehler-Topologie (INV-LOG-1 / HR-4): Kartiert Fehlertypen und verhindert Datenleaks in Fehlermeldungen | `ERROR_TOPOLOGY_REPORT.md`, `.json` |
| `python3 tools/test_matrix_advisor.py` | Test-Pyramide (§8): Prüft Abdeckung (Unit, Proptest, Loom, FaultVfs, Benchmarks) und gibt Ratschläge | `TEST_MATRIX_ADVISOR.md`, `.json` |
| `python3 tools/orchestrate_diagnostics.py` | Master-Orchestrierung: Führt alle 10 Werkzeuge aus und erzeugt Gesamtreport | `DIAGNOSTIC_SUMMARY.md` |
### 11.2 Analyse- & Diagnose-Werkzeuge (Einzelwerkzeuge, Micro-Probes)

Für Coding-Agenten und Entwickler stehen spezialisierte Python-Werkzeuge für mikrofeingranulare Code-Introspektion, Architekturprüfungen, Fehlerkartierung und automatisierte Dokumentation zur Verfügung. Alle Werkzeuge exportieren ihre Befunde automatisch nach `docs/diagnostics/<name>.{md,json}` (oder via `--output-dir`), unterstützen `--crate <name>`, `--json` für Pipelines und bieten maschinenlesbare Schnittstellen.

| Werkzeug | Fokus & Anwendungsbereich | Typischer Aufruf |
|---|---|---|
| `tools/rust_ast_inspector.py` | Mikrofeingranulare AST-Analyse: Structs, Enums, Traits, Impls, Funktionen, Visibilitäten, Docstrings & Attribute | `python3 tools/rust_ast_inspector.py -c <crate>` |
| `tools/pattern_diagnostic.py` | Anti-Pattern & Invarianten-Scanner: Zero-Panic (`unwrap/expect`), Determinismus (`Clock/Rng`), `mmap` (D-94), Vfs-Bypass & verbotene Begriffe (§6.4) | `python3 tools/pattern_diagnostic.py --prod-only` |
| `tools/dead_code_microprobe.py` | Micro-Probe für Dead Code, ungenutzte private Items, `#[allow(dead_code)]` und `#[deprecated]` Relikte | `python3 tools/dead_code_microprobe.py -c <crate>` |
| `tools/unsafe_boundary_tracer.py` | Audit aller `unsafe`-Blöcke, Verifikation von `// SAFETY:` Beweisen, Pointer-Casts & Insel-Regel (M9) | `python3 tools/unsafe_boundary_tracer.py` |
| `tools/concurrency_lock_auditor.py` | Prüfung von Mutex/RwLock-Haltedauern über `.await`, Deadlock-Gefahren, Kanälen & `loom` Permutationen | `python3 tools/concurrency_lock_auditor.py -c <crate>` |
| `tools/error_topology_mapper.py` | Vollständige Kartierung aller Error-Enums (`thiserror`), Varianten, Fehlermeldungen & `impl From` Konvertierungen | `python3 tools/error_topology_mapper.py` |
| `tools/contract_diff_detector.py` | Audit von canonical Traits in `contextra-ports`, Signaturen, `# Fehler` / `# Crash` Verträgen & Port-Implementierungen | `python3 tools/contract_diff_detector.py` |
| `tools/cargo_dependency_graph.py` | Transitive Ring-Hierarchie (Ring 0..4), M1 Ring-Regeln, M6 Dev-Dep-Prüfung & Zyklenerkennung | `python3 tools/cargo_dependency_graph.py` |
| `tools/erasure_leak_prober.py` | Mikrofeine Erkennung von DSGVO/Löschketten-Leaks und DeletionProofs in Speicherkernen (Regel M7) | `python3 tools/erasure_leak_prober.py` |
| `tools/test_coverage_matrix.py` | Kartierung von Unit-, Tokio-Async-, Proptest-, Benchmark- und Miri-Tests sowie SPEC-Traceability (T-TRACE-1) | `python3 tools/test_coverage_matrix.py -c <crate>` |
| `tools/agentic_context_pack.py` | Context-Engineering & Prompt-Packager: Bündelt minimale, hochdichte Modulinformationen & Regeln für LLM-Agenten | `python3 tools/agentic_context_pack.py -c <crate>` |

Gemeinsame Bibliothek: `tools/agent_tooling_common.py`. Alle Berichte liegen strukturiert in `docs/diagnostics/`.

## 12. Referenzen

| Dokument | Zweck |
|---|---|
| `PLAN.md` | Phasen 0–7, Gates G0–G7, Baukasten-Vertrag, Risiken |
| `docs/SPEC.md` | Normative Spezifikation v4.0 (Baukasten-Spezifikation) (Invarianten §3.4, Ringe §4, Ports §5, Erasure §12) |
| `DEPRECATED_DEBT.md` | Schuldenregister DEBT-01 bis DEBT-05 |
| `capabilities.toml` | Ring, Klasse, Reifegrad, `may_depend_on` |
| `docs/VETOES.md`, `docs/CONSTITUTION.md` | Vetos und Verfassung |
| `docs/TYPE_REGISTRY.md` | Typ-Kollisionsprüfung vor Neuanlage |
| `docs/MODULAR_DECOUPLING_AUDIT.md`, `docs/crate-coupling-report.md` | Entkopplungsaudit |
| `docs/diagnostics/` | Automatisch generierte Diagnose- & Analyseberichte (Markdown + JSON) |
| `governance/`, `.agents/governance/` | Gates, Ratchets, Determinismus-Ausnahmen, Sicherheits-Ports |
| `crates/<name>/AGENTS.md`, `PLAN.md`, `README.md` | Crate-spezifischer Vertrag, Arbeitspakete, Fremdprojekt-Einstieg |
