# Contextra Baukasten – Refaktorisierungsplan

Begleitdokument zur „Contextra Baukasten – Spezifikation v5“ · Stand 06.11.2026 · Basis: `tfufuz1/contextra`

Dieser Plan legt Reihenfolge, Arbeitspakete, Gates und Risiken fest. Aufwände sind als relative Größen (S/M/L/XL) angegeben, nicht als Zeitschätzung, weil weder Build noch Testlaufzeiten gemessen werden konnten.

## 1. Leitprinzipien

1. **Immer baubar.** Jeder Merge hinterlässt einen grünen `cargo build --workspace` und grüne Tests. Große Schnitte werden in Teil-PRs zerlegt.
2. **Von außen nach innen löschen.** Zuerst entfernen, was andere benutzen (Fassade, MCP, Engine), zuletzt, was nur noch intern hängt (Store-Compaction, Krypto). So bricht nie ein Verbraucher, dessen Lieferant schon weg ist.
3. **Layout und DAG-Prüfung zuerst.** Die Ring-Struktur gilt vom ersten Tag an (Phase 1), alle späteren PRs entstehen schon im Zielzustand.
4. **Messen vor Ändern.** Ohne Baseline (Phase 0) ist keine Performance-Aussage der Spezifikation prüfbar.
5. **Feature-Flags nur als Brücke.** Flag `legacy-erasure` ist befristet (Phase 3) und muss am Ende der Phase vollständig entfernt sein. Hinweis: Cargo-Features sind additiv; ein dauerhaftes Gating destruktiver Pfade verdoppelt die `cfg`-Fläche in 16 Crates und ist ausdrücklich nicht gewollt.

## 2. Überblick

| Phase | Inhalt | Größe | Gate |
| --- | --- | --- | --- |
| 0 | Baseline: Build, Tests, Benchmarks, Tag | M | G0 |
| 1 | Ring-Layout und DAG-Erzwingung | L | G1 |
| 2 | Ports v2 entwerfen, erproben, einfrieren | L | G2 |
| 3 | Löschung entfernen und Extensions abtrennen | XL | G3 |
| 4 | Store, Graph, Vektor, Text auf append-only umbauen | XL | G4 |
| 5 | Entkopplung und Zero-Copy-Härtung | L | G5 |
| 6 | Produktschnitte und Beispiele | M | G6 |
| 7 | Dokumentation und CI-Bereinigung | S | G7 |

Phase 3 und 4 sind die Hauptarbeit. Phase 4 darf erst starten, wenn G2 und G3 bestanden sind.

## 3. Phase 0 – Baseline

**Ziel:** Ein belegter Ausgangszustand. Alles Weitere wird dagegen gemessen.

| WP | Arbeit | Ergebnis |
| --- | --- | --- |
| 0.1 | Rust-Toolchain installieren (`rust-toolchain.toml`, Min-Version 1.89), `cargo metadata` ausführen | nachweisen, ob D1 (`xtask`) noch gilt, beheben falls ja |
| 0.2 | `cargo build --workspace --locked`, `cargo test --workspace --locked` | Liste roter Tests; die 20 `#[ignore]`-Stellen (D5) erfassen |
| 0.3 | Befunde D2 bis D4 und D6 prüfen | jeder Befund: Ja/Nein und Behebung |
| 0.4 | Benchmarks aus `benchmarks/contextra-bench` laufen lassen, p50/p99/p99,9 für Vektor, Text, Graph, Store | `baseline_20261106.json` im Repo |
| 0.5 | Kompilierzeit und Binary-Größe je Crate messen (`cargo build --timings`, `cargo bloat`) | Baseline-Tabelle |
| 0.6 | `git tag pre-baukasten`, Branch `baukasten/main` | Rückfalloption |
| 0.7 | Lösch-Inventar erzeugen: alle Treffer für `tombstone`, `erasure`, `shred`, `purge`, `remove_with_graph_repair`, `delete_prefix`, `retention`, `forget`, `gdpr`, `dsgvo` nach Crate und Datei | `docs/refactor/erase-inventory.md` |

**Gate G0:** Build und Tests grün (oder rote Tests als bekannte, dokumentierte Liste), Baseline-Dateien eingecheckt.

**Stand der statischen Voranalyse (zur Orientierung, noch nicht gegen Build geprüft):** Tombstone-Begriffe in 16 Crates (Spitze: `store` 41 Dateien, `graph` 31, `vector` 24, `text` 17); `remove_with_graph_repair` in 3 Dateien von `contextra-vector`; Erasure-Code in `engine/contextra_impl/erasure.rs`, `mcp` (3 Dateien), Löschnachweise in `crypto` (`deletion_proof/`, `ed25519_proof.rs`, `kv_shredding.rs`, `revocation_log.rs`).

## 4. Phase 1 – Ring-Layout und DAG-Erzwingung

**Ziel:** Das Repository ist physisch nach Ringen geordnet und die Abhängigkeitsrichtung ist maschinell geprüft. Verstöße, die es heute gibt, stehen auf einer Ausnahmeliste, die nur kleiner werden darf.

| WP | Arbeit |
| --- | --- |
| 1.1 | `git mv` der Crates in `ring0_foundation/`, `ring1_indices/`, `ring2_inference/`, `ring3_composition/`, `ring4_surface/`, `extensions/`, `tooling/` gemäß Spezifikation 3.1. `git mv` erhält die Historie. |
| 1.2 | Alle Pfade nachziehen: `Cargo.toml` (`members`, `default-members`, `[workspace.dependencies]`), `capabilities.toml` (`path`, `ring`), `CODEOWNERS`, `justfile`, `flake.nix`, `scripts/`, `.github`-Workflows, `mcp.json`. |
| 1.3 | `capabilities.toml`: `ring` und `may_depend_on` neu setzen; `contextra-durable-fs` eintragen (D6). |
| 1.4 | `tooling/xtask` anlegen mit `check-dag`: liest `cargo metadata`, vergleicht Verzeichnisring, `capabilities.toml`-Ring und tatsächliche Kanten. Bricht bei jeder aufwärts gerichteten Kante und bei jeder Kante in `extensions/` aus dem Kern. |
| 1.5 | `deny.toml` um Regel `deny` für Kante Kern → `extensions/` erweitern. |
| 1.6 | `xtask/dag-exceptions.toml` mit den heutigen Verstößen (u. a. `graph` → `adapt` je nach Zuordnung, `router` → `privacy`, `mcp` → `license`). Einträge haben ein Ablaufdatum (Phase) und werden nur gelöscht, nie geändert. |
| 1.7 | CI: `check-dag` als Pflicht-Check. |
| 1.8 | `contextra-core` (deprecated, 256 Zeilen) entfernen, Abhängige umhängen. |

**Hinweis zur Verzeichnisstruktur:** Verzeichnisse allein erzwingen nichts, Cargo kennt keine Ringe. Die sichtbare Ordnung hilft dem Auge; die Erzwingung kommt aus 1.4 und 1.5.

**Gate G1:** `check-dag` grün mit Ausnahmeliste; Workspace baut und testet wie in G0; keine funktionale Änderung (Diff besteht nur aus Verschiebungen und Pfaden).

## 5. Phase 2 – Ports v2

**Ziel:** Die Traits, an denen alle Ring-1-Crates hängen, sind final. Ohne diese Phase erzeugt der Index-Umbau Nacharbeit in jedem Index.

| WP | Arbeit |
| --- | --- |
| 2.1 | Entwurf `Storage` v2 in `contextra-ports`: `append`, `get`, `get_at_seq`, `scan`, `with_value` (Callback), `PinnedBlock`-Guard (Spec P-4), Batch-Append. **Ohne** `delete*`. Native async-Methoden, kein `BoxFuture` (P-2). |
| 2.2 | Entwurf `VectorIndex`, `TextIndex`, `GraphIndex` v2: `insert`, `search`, `supersede(old, new)`, ohne `delete`/`remove_*`. |
| 2.3 | Alte Traits unverändert unter `ports::legacy` und `feature = "legacy-erasure"` belassen, damit alle Crates weiter bauen. |
| 2.4 | **Spike:** Prototypische Implementierung der v2-Traits für `contextra-store` (nur Lesepfad) und je ein Mini-Konsument in `vector`, `text`, `graph`. Prüfen: Passen Lifetimes? Entsteht irgendwo eine Heap-Allokation? Messung mit zählendem Allokator. |
| 2.5 | Contract-Tests (`ports/contract_tests.rs`, bereits vorhanden) auf v2 portieren. |
| 2.6 | `dyn`-Adapter für Ring 3/4 (`DynStorage` mit `BoxFuture`), ausschließlich dort. |
| 2.7 | Freeze: Version `contextra-ports 1.0.0`, Doku-Kommentar „Änderungen nur durch ADR“, `cargo-semver-checks` in CI. |

**Gate G2:** Spike beweist: Lesepfad Store → Index ohne Kopie und ohne Allokation; Contract-Tests grün; `ports 1.0.0` getaggt. Fehlt der Nachweis, wird nicht eingefroren, sondern 2.1 bis 2.4 wiederholt.

## 6. Phase 3 – Löschung entfernen und Extensions abtrennen

**Ziel:** Kein Löschpfad im Kern (Spec B4). Von außen nach innen, jeweils ein PR pro Zeile.

| Schritt | Crate | Entfernen / Verschieben | Größe |
| --- | --- | --- | --- |
| 3.1 | `contextra-license`, `-privacy`, `-avv-generator`, `-audit-export` | nach `extensions/` verschieben; Kanten aus `contextra`, `mcp`, `router` entfernen; Port `EgressPolicy` in `ports` (Router bleibt benutzbar ohne `privacy`) | M |
| 3.2 | `contextra` (Fassade) | Features `compliance`, `audit-export`, `avv-generator` entfallen; Lösch-API entfernen | S |
| 3.3 | `contextra-mcp` | 3 Dateien mit Erasure/`forget` bereinigen (12 `forget`-Treffer); Tool-Definitionen für Löschen entfernen; Lizenz-Gate herauslösen | M |
| 3.4 | `contextra-py` | Lösch-Bindings entfernen (nur falls vorhanden; per Inventar 0.7 prüfen) | S |
| 3.5 | `contextra-engine` | `contextra_impl/erasure.rs` nach `extensions/ext-erasure` verschieben oder streichen; `background_workers` auf Lösch-/Retention-Jobs prüfen; `decay_controller.rs` prüfen, ob Decay logisch löscht (dann ändern auf Gewichtung ohne Entfernung) | L |
| 3.6 | `contextra-db`, `-agent`, `-cognition` | Löschbezogene Aufrufe entfernen (`db` 10, `cognition` 11 Dateien mit Tombstone-Bezug) | M |
| 3.7 | `contextra-graph` | `cascade.rs`, `csr/graph_write/tombstone.rs`, `graph_persist::delete_edge_persistence`, `GraphIndex::{remove_edge, remove_entity}`, `provenance::remove_doc`, Kaskaden-Konstanten; „DeletedView“ in `ppr`-Pfaden ersetzen | L |
| 3.8 | `contextra-vector` | `hnsw/deletion.rs`, `deleted_ratio`, `DiskAnn::delete`, `VectorIndex::delete`; 24 Dateien mit Tombstone-Bezug aufräumen | L |
| 3.9 | `contextra-text` | Löschpfade, `purge`, 17 Dateien Tombstone | M |
| 3.10 | `contextra-store` | `compaction/engine.rs` und `adaptive.rs` (tombstone-getriebene Zweige), `memtable.rs`, `kv/delete_mode.rs`, `lsm/scan.rs`, `lsm/ops/read.rs`, `kv/segment.rs`; `crypto_shred`-Pfade; `StorageEngine::delete*` | XL |
| 3.11 | `contextra-mvcc` | Tombstone-Bit im `seq_no` **reservieren**, nicht entfernen (Spec 5.2); Reader lehnt gesetztes Bit ab | S |
| 3.12 | `contextra-crypto` | Aufspaltung: `crypto-core` (WAL-HMAC, Cipher, KDF, Audit-Kette, Anti-Tamper) bleibt Ring 0; `kv_shredding`, `deletion_proof/`, `ed25519_proof.rs`, `revocation_log.rs` nach `extensions/ext-erasure` | L |
| 3.13 | `contextra-ports` | `Purger`, `delete`-Methoden aus der v2-Linie sind bereits weg (Phase 2); `ports::legacy` löschen | S |
| 3.14 | Alle | Flag `legacy-erasure` und alle `cfg`-Zweige löschen; `xtask check-no-delete` aktivieren | M |

**Verfahren je Schritt:**

1. Aufrufer finden (`cargo check`, `grep`), Aufrufer zuerst bereinigen.
2. Funktion hinter `legacy-erasure` stellen, Build grün.
3. Tests, die Löschung prüfen, löschen oder in `ext-erasure` verschieben; Tests, die Löschung nur nebenbei nutzen (z. B. als Cleanup in Fixtures), umschreiben.
4. Code entfernen, sobald nichts mehr referenziert.

**Gate G3:** `check-no-delete` grün (mit Allowlist nur für das reservierte seq-Bit); kein Kern-Crate hängt von `extensions/` ab; Ausnahmeliste aus 1.6 enthält keinen Lösch-Eintrag mehr; Tests grün.

**Achtung bei Phase 3.5 und 3.7:** `decay_controller.rs` und das Graph-Modul `cascade.rs` behandeln neben Löschung auch **Aktualisierung** („Quelldokument gelöscht oder aktualisiert“). Aktualisierung bleibt als Supersession erhalten (Spec 5.1). Beim Entfernen darf diese Semantik nicht verloren gehen; dafür gibt es Regressionstests vorab (WP 3.0, siehe unten).

**WP 3.0 (vor 3.5):** Charakterisierungstests für Update-Verhalten schreiben (Dokument wird ersetzt, abgeleitete Kanten und Chunks verhalten sich wie erwartet). Sie sind die Messlatte, dass der Umbau nur die Löschung trifft.

## 7. Phase 4 – Append-only-Umbau

**Ziel:** Die in Spezifikation 5 beschriebenen Strukturen, jetzt ohne Altlasten.

| WP | Crate | Arbeit |
| --- | --- | --- |
| 4.1 | `store` | `KeepAllVersions` als Default; `DropSuperseded` hinter `gc-superseded`; Compaction nur noch Level-Merge; Memtable ohne Tombstone-Einträge; Dateiformat-Version erhöhen, falls Eintrags-Kodierung sich ändert (Migrationspfad: Reader beherrscht v1 und v2, Writer nur v2) |
| 4.2 | `store` | Block-Cache: `PinnedBlock` implementieren (Phase 2); Messung, dass `with_value` allokationsfrei ist |
| 4.3 | `sys` | `mmap`-Eigentümertyp `SealedMap` (`MAP_POPULATE`/`mlock`-Policy, keine `&mut`-Sicht); Fehlerpfad für abgeschnittene Dateien (`SIGBUS`-Vermeidung durch versiegelte, unveränderliche Dateien und Eigentümer-Lock) |
| 4.4 | `graph` | CSR-Segmente: Delta-Puffer + Merge-Job + Epochenwechsel (`ArcSwap`-ähnlich, Workspace-Abhängigkeit prüfen); Lesepfad ohne `RwLock` (63 Treffer auditieren: welche bleiben, welche weg); Supersession als `valid_from_seq` |
| 4.5 | `vector` | HNSW additiv; Bitmap `superseded: RoaringBitmap`; Nachverarbeitung im Suchpfad; Messung des Recall-Verlusts je Anteil abgelöster Vektoren (Tabelle in Dokumentation, damit Nutzer wissen, wann sich ein Neuaufbau lohnt) |
| 4.6 | `vector` | Dateisystem-I/O (28 Treffer) über `Vfs`-Port (B5, D4) |
| 4.7 | `text` | Supersession-Bitmap analog 4.5 |
| 4.8 | `rank` | Anpassung, falls Rangfusion Tombstone-Informationen erwartete (per Inventar prüfen) |
| 4.9 | `checkpoint` | Checkpoint-Inhalt ohne Löschmarken; Wiederanlauf-Tests |

**Gate G4:** Benchmarks nicht schlechter als Baseline (Spec 8); Recovery- und Crash-Tests grün (`fault-injection`-Feature in `store`); Graph-Lesepfad lock-frei nachgewiesen; keine Datei in Ring 0/1 ruft `std::fs` direkt auf (außer `sys`, `durable-fs`).

## 8. Phase 5 – Entkopplung und Zero-Copy-Härtung

| WP | Arbeit |
| --- | --- |
| 5.1 | `dyn`-Inventar für `engine` (60), `ports` (35), `store` (23), `mvcc` (16), `vector` (14), `graph` (11), `text` (9): je Stelle heiß oder kalt entscheiden, heiße auf Generics umstellen. |
| 5.2 | Quer-Kanten aus Spec 7.2 schneiden: `graph` → `adapt`, `vector` → `mvcc`/`crypto`, `mcp` → `infer-*` per Feature, `engine` zerlegen, wo sinnvoll (Chunker, Fusion, Decay). |
| 5.3 | `contextra-ingest` ohne `engine` baubar machen (heute hängt es vollständig daran). Voraussetzung für Produkt „Ingest“. |
| 5.4 | `contextra-db` (Legacy-Fassade, 13 Abhängigkeiten) auflösen (Strangler): Aufrufer auf `contextra` oder direkte Crates umstellen. |
| 5.5 | `unsafe`-Audit (B7): Zahlen nach `unsafe {`, `unsafe fn`, `unsafe impl` trennen; Inseln `infer-onnx` (5), `crypto` (5), `ingest` (10), `text` (8) deklarieren oder bereinigen. |
| 5.6 | Kompilierzeit-/Binary-Vergleich gegen Phase-0-Baseline; bei Aufblähung durch Monomorphisierung: betroffene Seam wieder auf `dyn` oder einen `#[inline(never)]`-Rand setzen. |

**Gate G5:** Keine Ring-Ausnahme außer solchen mit ADR; `contextra-store` allein ohne ONNX/Candle/CUDA (B6); Binary-Größe und Kompilierzeit ohne Verschlechterung.

## 9. Phase 6 – Produktschnitte

| WP | Arbeit |
| --- | --- |
| 6.1 | Für jedes Produkt der Tabelle in Spec 9 ein Verzeichnis `examples/product-<name>/` mit minimaler `Cargo.toml` und einem `main.rs`, der den Kern-Anwendungsfall zeigt. |
| 6.2 | CI-Job `product-matrix`: baut jedes Beispiel isoliert (`cargo build -p ...`) in einem sauberen Verzeichnis, das nur die erlaubten Crates kopiert. Das beweist Z1 maschinell. |
| 6.3 | Pro Produkt eine README mit Abhängigkeitsmenge und Garantien. |
| 6.4 | Versionierung: Crate-Versionen entkoppeln (heute alle `0.1.0` über `workspace.package`); Ring 0 mit eigener SemVer-Linie, damit ein Produkt Ring 0 pinnen kann, ohne dass Ring 3 mitzieht. |

**Gate G6:** Alle zehn Produkte bauen isoliert und laufen ihr Beispiel.

## 10. Phase 7 – Dokumentation und CI

- Alte Spezifikationen (`SPEC.md`, drei `CONTEXTRA_INTERFACE_SPECIFICATION*`) nach `docs/spec/archiv/` verschieben, Spezifikation v5 als einzige normative Quelle in `SOURCE_OF_TRUTH.md` eintragen.
- `ARCHITECTURE.md` aus `capabilities.toml` generieren (heute von Hand gepflegt).
- `CHANGELOG.md`: Breaking-Change-Eintrag „Löschung entfernt“.
- `legal-matrix.toml`, `SECURITY.md`, `VETOES.md` auf Konsistenz prüfen (enthalten vermutlich Löschzusagen).
- `reports.zip` und `crate-coupling-report_20261105.md` aktualisieren oder entfernen.

**Gate G7:** `docs/` enthält keine Zusage über Löschung oder DSGVO-Konformität des Kerns mehr.

## 11. Risikoregister

| ID | Risiko | Wirkung | Gegenmaßnahme |
| --- | --- | --- | --- |
| R1 | Baseline rot: Der Ist-Stand baut nicht (ARCHITECTURE nennt D1 bis D3) | Phase 0 dauert länger; alle Folgeaussagen schieben sich | G0 ist hartes Gate; kein Phase-1-Merge ohne Grün |
| R2 | Update-Semantik geht mit der Löschung verloren (Cascade, Decay) | stille Fehler im Retrieval | WP 3.0 Charakterisierungstests vor 3.5 |
| R3 | Falsch eingefrorene Ports | Nacharbeit in allen Indizes, Brechen der Entkopplung | Spike in 2.4 ist Gate-Bedingung |
| R4 | Speicherwachstum ohne Löschung | Edge-Geräte laufen voll | `gc-superseded` als Option; Wachstumsmessung in 4.1; Kapazitäts-API in `store` |
| R5 | Recall-Verlust im additiven HNSW durch viele abgelöste Vektoren | schlechtere Treffer | Messung 4.5; Neuaufbau-Empfehlung dokumentieren |
| R6 | Monomorphisierung bläht Build und Binary auf | Gegenteil des Ziels | 5.6; `dyn`-Fallback an kalten Rändern |
| R7 | Dateiformatänderung bricht vorhandene Daten | Datenverlust | Reader v1 + v2, Migrationstest mit Fixture-Dateien |
| R8 | Pfad-Umzug (Phase 1) stört laufende Branches | Merge-Konflikte | Phase 1 in einem Tag mergen, Freeze-Fenster, `git mv` ohne Inhaltsänderung |
| R9 | Rechtliche Folgen, wenn ein späteres Produkt personenbezogene Daten im EU-Raum speichert | Pflicht zur Löschung nicht erfüllbar | `ext-erasure` und DEK je Collection bleiben als Option (Spec 6); Entscheidung pro Produkt |
| R10 | `ingest` lässt sich nicht vom `engine` lösen | Produkt „Ingest“ nicht eigenständig | 5.3; zur Not Zerlegung von `engine` |

## 12. Definition of Done (gesamt)

1. G0 bis G7 bestanden und dokumentiert (`docs/refactor/gates.md`).
2. Zehn Produkte bauen isoliert (6.2 grün).
3. `xtask check-dag`, `check-no-delete`, `cargo deny`, `cargo semver-checks` sind Pflicht-Checks.
4. Baseline-Vergleich: keine Verschlechterung bei p99/p99,9, Kompilierzeit und Binary-Größe.
5. `ext-erasure` baut als eigenständige Extension (auch wenn nicht benutzt), damit die Option erhalten bleibt.

## 13. Empfohlener nächster Schritt

Phase 0 beginnen: Toolchain installieren, `cargo build --workspace` und `cargo test --workspace` laufen lassen und die rote/grüne Liste liefern. Erst damit lässt sich Phase 1 verantwortungsvoll starten.

---

# Contextra Baukasten – Implementierungsspezifikation

Stand 06.11.2026 · Basis `tfufuz1/contextra` Commit `f55c504` (06.11.2026) · ersetzt den Refaktorisierungsplan der Löschungs-Linie

## 0. Entscheidung und Verifikationsstatus

**Entscheidung.** Die geplante Löschung entfällt vollständig. Lösch-, Tombstone- und Erasure-Pfade bleiben funktional und werden nicht umgebaut. Sie werden nur über Ports (`Purger`, `LayerScanner`) so geschnitten, dass sie mit ihrem Modul wandern. Gestrichen sind: alte Phase 3 (Löschung entfernen), Phase 4 (Append-only-Umbau), Ports v2 ohne `delete*`, Flag `legacy-erasure`, `check-no-delete`, Risiken R4, R5, R7, R9 und DoD-Punkt 5. Neu gefasst oder übernommen sind: Baseline, DAG-Erzwingung, Entkopplung, Ports-Freeze, Produktschnitte, Dokumentation.

**Verifikationsstatus.** Alle Zahlen stammen aus statischer Analyse des Klons: Manifeste per `tomllib`, Quelltext per `grep`, Git-Historie. Nichts wurde gebaut oder getestet, weil die Toolchain 1.89 in meiner Umgebung nicht erreichbar ist und `cargo 1.84` an einer Abhängigkeit mit `edition2024` scheitert. Trefferzahlen sind Wortvorkommen, keine Aufrufgraphen. Jede Zahl, auf der eine Entscheidung ruht, hat in Phase 0 oder im jeweiligen Spike einen Prüfschritt. Aufwände sind relative Größen (S/M/L/XL).

**Nicht vorhanden.** Die „Spezifikation v5“ des alten Plans liegt weder im Repo noch im Anhang vor. Ihre Verweise (Spec 3.1, 5, 7.2, 9, B4–B7, Z1, P-2, P-4) sind durch die Regeln in §3 ersetzt. Der Produktkatalog in Phase 6 ist neu definiert und muss gegen Spec 9 abgeglichen werden, sobald sie vorliegt.

## 1. Korrekturen am bisherigen Plan

| ID | Alte Annahme | Ist-Stand | Folge |
| --- | --- | --- | --- |
| D1 | `xtask` ist Workspace-Mitglied, Verzeichnis fehlt | `Cargo.toml` ist bereinigt. Aber `rust-ci.yml` (Zeilen 28, 118, 120) und `justfile` (Zeilen 24–29) rufen weiter `cargo xtask …`, die Schritte laufen ins Leere. `xtask/` wurde in Commit `068261abd` gelöscht und ist aus der Historie wiederherstellbar (inkl. `registry.toml`, `ring0-async-exceptions.toml`). | WP 0.2 |
| D2, D3 | Features `onnx` und `adaptive-candidate-pool-sizing` ohne `dep:` | Beide Features haben jetzt `dep:`-Einträge. Statisch behoben, Build mit Features steht aus. | WP 0.3 |
| D4 | Dateisystem-I/O in `vector` (28 Treffer) | Bestätigt: 28 Produktiv-Treffer (`diskann/build.rs` 11, `diskann/persistence.rs` 10, `hnsw/types.rs` 3, `diskann/types.rs` 3, `persistence/mmap.rs` 1), `diskann` ist experimentell. Größer ist der Fall `store` (Produktivcode u. a. `wal/mod.rs` 28, `wal/hmac.rs` 20, `lsm/recovery.rs` 17, `scanner/purge.rs` 11, `wal/replay.rs` 10) und `checkpoint` (`hardlink_cloner.rs` 11). | Phase 5 |
| D5 | 20 `#[ignore]` | Bestätigt: `store` 11, `infer-candle` 4, `db` 2, `adapt` 1, `infer-ollama` 1, `text` 1. | WP 0.3 |
| D6 | `durable-fs` fehlt in `capabilities.toml` | Bestätigt: 34 Einträge für 35 Crates. Das Manifest führt noch `spec = v4`. | WP 1.1 |
| D7 neu | „I/O nur über `Vfs`“ | Es gibt **zwei verschiedene** `Vfs`-Traits. `contextra_ports::Vfs` arbeitet auf ganzen Dateien (`read`, `write_atomic`, `sync_dir`). `contextra_durable_fs::vfs::Vfs` liefert Datei-Handles mit `read_at`/`write_at`/`sync_all`. Einziger Konsument des Ports-Traits ist `crypto/revocation_log.rs`. `contextra-durable-fs` hat keinen Konsumenten. `store`, `vector` und `checkpoint` nutzen `std::fs` direkt. | Phase 3.1, Phase 5 |
| D8 neu | Ringe stehen in `capabilities.toml` | Die Zuordnung steht zusätzlich als eigene Tabelle in `tests/layering.rs`. Die `LAYER_ALLOWLIST` hat 4 Einträge. Drei davon (`infer-onnx`→`infer-candle`, `infer-candle`→`store`, `infer-ollama`→`infer-onnx`) haben nach den Normal-Abhängigkeiten der Manifeste keine Kante mehr. | WP 1.2 |
| D9 neu | – | Kandidaten für tote Abhängigkeiten (0 Treffer im `src/` der Crate): `contextra`→`audit-export`, `avv-generator`, `privacy`, `crypto` (alle optional); `mcp`→`license`; `db`→`crypto`; `engine`→`sys`; `py`→`adapt`, `store`; `bench`→`store`, `text`. Mit `cargo machete` und Build-Matrix zu bestätigen. | WP 2.1 |
| D10 neu | `unsafe`-Audit in `infer-onnx` (5), `crypto` (5), `ingest` (10), `text` (8) | Das waren Wortvorkommen (Kommentare, `forbid`). Echter `unsafe`-Code: `simd` 272, `wire` 45, `sys` 21. 32 von 35 `lib.rs` tragen `forbid(unsafe_code)`, die drei Ausnahmen sind genau `simd`, `sys`, `wire`. | Altes WP 5.5 entfällt |
| D11 neu | `contextra-core` entfernen und Abhängige umhängen | Außerhalb der Crate verweist nur `crates/contextra-db/fuzz/Cargo.toml` darauf. Entfernung ist klein. | WP 2.5 |

## 2. Kopplungsmessung

Trefferzahlen und Dateien im `src/` der abhängigen Crate. Kanten zu `types` und `ports` sind weggelassen.

| Kante | Treffer / Dateien | Benutzt wird | Bewertung |
| --- | --- | --- | --- |
| engine → store | 132 / 19 | `LsmStorage` 50, `LsmConfig` 45 | zentrale Verdrahtung, Phase 4 |
| engine → vector | 131 / 18 | `HnswIndex` 55, `HnswConfig` 53, `hnsw_purger`, `hnsw_scanner` | Phase 4 |
| engine → graph | 88 / 18 | `CsrGraph` 52, `hyperedge`, `percolation` | Phase 4, Trait-Abdeckung offen |
| engine → text | 71 / 13 | `Language` 66, `text_purger`, `text_scanner` | `Language` ist ein Datentyp, Phase 2 |
| cognition → engine | 27 / 11 | `collection`, `Language`, `ProvenanceRecord`, `decay_controller` | folgt Phase 4 |
| cognition → graph, vector, store | 18, 8, 5 | `CsrGraph`, `HnswIndex`, `LsmStorage` | folgt Phase 4 |
| ingest → engine | 3 / 2 | nur `chunker` | Extraktion, Phase 2 |
| router → privacy | 4 / 1 | `guarded_payload` | `EgressPolicy`-Port, Phase 3 |
| privacy → avv-generator | 6 / 1 | `render_avv_markdown` | Feature-Gate, Phase 2 |
| graph → adapt | 1 / 1 | eine Fundstelle | trivial, Phase 2 |
| vector → crypto | 8 / 3 | `wal_crypto::WalHmac`, Fundstellen u. a. in `diskann/` | optional machen, Phase 2 |
| store → crypto | 9 / 8 | `wal_crypto` 4, `kv_shredding` 1 | WAL-Kante akzeptiert, Shredding optional |
| store, vector, checkpoint → mvcc | 13, 7, 2 | `SsiValidator`, `SequenceLog`, `SnapshotRegistry` | akzeptiert (Fundament) |
| router → adapt, infer-\* → rank | 16, 1–5 | Bandit-Code, Kalibratoren | akzeptiert (Fundament) |
| infer-candle → crypto | 8 / 2 | `EncryptedKvLayer`, `KvSegment` | prüfen, ob `kv_segment` nach `kvcache` gehört |
| mcp → contextra | 29 / 14 | Fassade | akzeptiert, Produktoberfläche |
| db → 13 Crates | je 1–11 | Komposition | Referenzkomposition, Phase 6 |

**Struktur der Engine.** `contextra-engine` hat 82 Dateien und 21.768 Zeilen: `collection/` 12.418, `contextra_impl/` 2.071, `transaction/` 1.652, `chunker/` 1.533, `background_workers/` 1.290, `extraction/` 642. `Collection<S: StorageEngine = LsmStorage, V: VectorIndex = HnswIndex>` ist nur halb generisch: der Graph ist fest als `Arc<CsrGraph>` verdrahtet, der Textindex als `InvertedIndex<S>`, und die Defaults binden die konkreten Typen. Die Engine enthält keine `Arc<dyn …>` auf die vier Index-Ports. Die Ports wirken dort nur als Generics-Schranke. Das ist der Hauptpunkt der Entkopplung.

## 3. Baukasten-Vertrag

Jedes Modul erfüllt diese Regeln. Sie ersetzen die Verweise auf die fehlende Spec v5 und werden von `xtask` geprüft (Phase 1).

1. **M1 Richtung.** Abhängigkeiten zeigen nur abwärts. Maßgeblich ist `may_depend_on` in `capabilities.toml`, geprüft gegen Normal-, Build- und Dev-Kanten.
2. **M2 Eigenständig bauen.** `cargo build -p <modul> --no-default-features` benötigt nur die eigenen `may_depend_on`-Crates. Nachweis: Produktmatrix in sauberem Verzeichnis (Phase 6).
3. **M3 Schnittstellen.** Öffentliche Signaturen enthalten nur Typen aus `types`, `ports` oder dem Modul selbst, keine konkreten Typen eines Geschwistermoduls.
4. **M4 Umwelt injizieren.** Dateizugriff über `Vfs`, Zeit über `Clock`, Zufall über `Rng`, IDs über `IdGen`, Metriken über `MetricsSink`. Alle fünf Ports existieren bereits.
5. **M5 Features additiv.** Defaults sind minimal. Schwergewichte (`onnx`, `candle`, `wasmtime`, `pyo3`, `reqwest`) stehen nie im Default.
6. **M6 Tests ohne höhere Ringe.** Ein Modul testet sich mit `testkit` und eigenen Dev-Abhängigkeiten. Heute verletzt: `store` (Dev: `graph`, `vector`, `sandbox`), `graph` (Dev: `store`), `agent` (Dev: `text`, `vector`), `router` (Dev: `db`, `graph`, `text`, `vector`).
7. **M7 Löschung gehört dem Datenhalter.** Wer Daten hält, implementiert `Purger` und `LayerScanner` selbst. Orchestrierung (Nachweis, Kaskade) liegt darüber und kennt nur die Ports.
8. **M8 Runtime-Neutralität unten.** Fundament und Indizes ohne `tokio` in `[dependencies]`. Heute erfüllt in `types`, `ports`, `mvcc`, `vector`, `text`, `graph`, `crypto`, `rank`, `adapt`, `kvcache`, `sys`, `simd`, `wire`, `durable-fs`. Ausnahmen mit `tokio`: `store`, `checkpoint`.
9. **M9 `unsafe` nur in Inseln.** Erlaubt in `simd`, `sys`, `wire`. Überall sonst `forbid(unsafe_code)`.
10. **M10 Versionierung.** Jedes Modul hat eine eigene SemVer-Version. Path-Abhängigkeiten tragen `version`. Bezug von außen per Git-Tag `<crate>-vX.Y.Z`.
11. **M11 Dokumentation.** Jedes Modul hat eine README mit Abhängigkeitsmenge, Features, Garantien und einem Einbettungsbeispiel.

## 4. Modulkatalog (Zielzustand)

Ringnamen folgen dem alten Plan: 0 Fundament, 1 Indizes und Speicher, 2 Inference, 3 Komposition, 4 Oberfläche, dazu Extensions.

| Modul | Ring | Ziel-Abhängigkeiten (intern) | Maßnahme |
| --- | --- | --- | --- |
| `types` | 0 | – | – |
| `ports` | 0 | `types` | Phase 3, Freeze nach Phase 4 |
| `wire`, `sys` | 0 | – | – |
| `simd` | 0 | `types` | – |
| `durable-fs` | 0 | `sys`, `ports` | Vfs konsolidieren (3.1), in `capabilities.toml` eintragen |
| `mvcc` | 0 | `ports`, `types` | – |
| `crypto` | 0 | `ports`, `types` | Aufspaltung in Core und Erasure (5.5) |
| `crypto-erasure` (neu) | 0 | `crypto`, `ports`, `types` | aus `crypto` ausgelagert, nichts entfällt |
| `rank`, `adapt` | 0 | `ports`, `types` | – |
| `chunker` (neu) | 1 | `types` | aus `engine/chunker/` (1.533 Zeilen) |
| `text` | 1 | `ports`, `types` | bereits eigenständig; `Language` nach `types` |
| `graph` | 1 | `ports`, `types` | `adapt`-Kante schneiden, Dev-Kante `store` entfernen |
| `vector` | 1 | `ports`, `types`, `mvcc`, `simd`, `sys`; `crypto` optional | Phase 2 und 5 |
| `store` | 1 | `ports`, `types`, `mvcc`, `crypto`, `sys`, `durable-fs`; `crypto-erasure` optional | Phase 5 |
| `checkpoint` | 1 | `ports`, `types`, `mvcc` | Phase 5 |
| `kvcache` | 1 | `ports`, `types`, `crypto` | `kv_segment`-Zuordnung prüfen |
| `infer-onnx`, `-candle`, `-ollama` | 2 | `ports`, `types`, `rank` (candle: `crypto`) | – |
| `sandbox` | 2 | – | – |
| `engine` | 3 | `ports`, `types`; Backends per Feature `default-backends` | Phase 4 |
| `cognition` | 3 | `engine`, `ports`, `types` | folgt Phase 4 |
| `router` | 3 | `ports`, `types`, `wire`, `adapt` | `privacy` entfällt über `EgressPolicy` |
| `privacy` | 3 | `ports`, `types`; `avv-generator` per Feature | Phase 2 |
| `ingest` | 3 | `types`, `chunker` | ohne `engine` |
| `agent`, `db` | 3 | wie heute | `db` ist Referenzkomposition, kein Baustein |
| `contextra`, `mcp`, `py` | 4 | Produktoberflächen | tote Abhängigkeiten entfernen |
| `license`, `audit-export`, `avv-generator` | Extension | `ports`, `types` | bleiben, nur optional eingebunden |

Physisches Verschieben in Ringverzeichnisse ist **nicht** Teil des Kerns (siehe Phase 7). Das Repo hat hohe Merge-Frequenz durch automatisierte Branches (zuletzt PR #5504); ein großer `git mv` bringt für die Entkopplung nichts, die Erzwingung kommt aus `check-dag`.

## 5. Schnittstellen-Spezifikation

### 5.1 Backend-Bündel in der Engine

Entwurf, Details im Spike WP 3.3. Vier lose Typparameter würden die 12.400 Zeilen von `collection/` überfrachten, deshalb ein Bündel:

```rust
pub trait Backends: Send + Sync + 'static {
    type Storage: StorageEngine;
    type Vector: VectorIndex;
    type Text: TextIndex;
    type Graph: GraphIndex;
    fn purgers(&self) -> Vec<Arc<dyn Purger>>;
}

pub struct Collection<B: Backends = DefaultBackends> { /* ... */ }
```

`DefaultBackends` (`LsmStorage`, `HnswIndex`, `InvertedIndex`, `CsrGraph`) liegt hinter dem Feature `default-backends` der Engine, im Default aktiv. Da `InvertedIndex<S>` den Storage kennt, braucht das Bündel Konstruktorfunktionen. Für Altcode bleibt `Collection<S, V>` als Alias bestehen, bis `cognition`, `db`, `mcp`, `agent` und `bench` umgestellt sind. Fallback, falls `CsrGraph` zu viele Methoden außerhalb von `GraphIndex` benötigt: Graph bleibt vorerst konkret, entkoppelt werden nur Storage, Vektor und Text.

### 5.2 Ein einziger Vfs

`ports::Vfs` wird zur einzigen Definition. Es erhält die Handle-Operationen aus `durable-fs` (`open` → `VfsFile` mit `read_at`, `write_at`, `sync_data`, `sync_all`) zusätzlich zu den Ganzdatei-Methoden. `durable-fs` implementiert `RealFs` gegen diesen Trait, sein zweiter `Vfs` entfällt. Der Umbau ist billig: es gibt nur fünf Implementierungen (`RealFs`, `MemVfs` in `ports/contract_tests.rs`, drei Test-Vfs in `crypto`). `store`, `vector`, `checkpoint` bekommen den Vfs per Konstruktor, Default `RealFs`.

### 5.3 EgressPolicy

Neuer Port in `ports`, nach dem Muster von `LicenseGate`. `router` kennt nur den Port und die Entscheidung `Allow`/`Redact`/`Deny`. `privacy` implementiert ihn, `mcp` verdrahtet beides. Die vier Treffer von `router` auf `guarded_payload` zeigen den Umfang.

### 5.4 Purger-Registry

`engine/contextra_impl/erasure.rs` importiert heute `hnsw_purger`, `hnsw_scanner`, `text_purger`, `text_scanner` direkt. Künftig liefert `Backends::purgers()` die Implementierungen als `Arc<dyn Purger>`, die Engine kennt nur `ports::scanner`. Verhalten und Nachweisformat (`deletion_proof`) bleiben unverändert.

### 5.5 Aufspaltung von `crypto`

`contextra-crypto` (Core) behält `wal_crypto`, `crypto`, `kdf`, `kv_cipher`, `kv_segment/`, `audit_chain`, `anti_tamper`, `wal_completeness`, `error`. `contextra-crypto-erasure` erhält `deletion_proof/`, `ed25519_proof.rs`, `kv_shredding.rs`, `revocation_log.rs`. Es gibt keine Rückkante, `crypto` kann `crypto-erasure` also nicht re-exportieren. Drei Importstellen sind umzuhängen: `engine` (`deletion_proof`, 4 Treffer), `mcp` (1 Treffer), `store` (`kv_shredding::KeyRegistry` in `kv/segment.rs`). In `store` wird daraus das Feature `crypto-shred`, **im Default aktiv**, damit sich das Verhalten nicht ändert; Produkte ohne Shredding schalten es ab. Die Dateiliste ist vor dem Schnitt gegen den Quelltext zu prüfen.

### 5.6 Chunker und Language

`contextra-chunker` nimmt `engine/src/chunker/` auf. `engine` behält `pub use contextra_chunker as chunker`, bestehende Pfade `contextra_engine::chunker::…` funktionieren weiter. `ingest` hängt dann an `chunker` statt `engine` (3 Treffer in 2 Dateien: `chunking/fallback.rs`). Der Typ `Language` (66 Treffer in `engine`) wird nach `types` verschoben, falls er ein reiner Datentyp ist; `text` re-exportiert ihn für Kompatibilität.
