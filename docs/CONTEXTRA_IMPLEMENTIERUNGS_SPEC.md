# Contextra – Einheitliche Implementierungs-Spezifikation

> **Status:** verbindlich · **Stand:** 9. November 2026 · **Basis:** `tfufuz1/contextra`, HEAD `b39fc2b`
> **Ersetzt:** `02_SPEC_FUER_CODING_AGENTEN.md` und `02_AGENTEN_SPEZIFIKATION.md` (beide werden hiermit vereinigt; Widersprüche sind in §2 entschieden)
> **Methode:** rein statische Analyse. Es wurde nichts kompiliert oder ausgeführt. Jede Aussage ist mit **[V]** (am Code verifiziert, per `grep`/Manifest-Lesen) oder **[H]** (Hypothese, vor dem Handeln zu prüfen) markiert.

---

## 0. Entscheidungen in Kürze

| # | Entscheidung |
|---|---|
| E1 | **Fokus:** Contextra ist ein **eingebetteter, durabler Hybrid-Retrieval-Kern** für RAG: LSM-Store, HNSW, BM25, Graph (PPR), Rangfusion, Orchestrierung, Python-Bindings, MCP-Server. Alles andere ist Baustein, eingefroren oder Archiv. |
| E2 | **Build zuerst.** Der Kern muss kompilieren, bevor irgendetwas umstrukturiert oder gehärtet wird. (Spec 1 stellte das zurück – falsch.) |
| E3 | **Der erste Blocker ist nicht `store::kv`**, sondern ein **doppelter `[features]`-Abschnitt in `contextra-infer-ollama/Cargo.toml`** [V]. Er lässt `cargo metadata` für den *gesamten* Workspace scheitern. Keine der beiden Specs erwähnt ihn. |
| E4 | **Mechanische Löschung bleibt, Nachweis-Maschinerie geht.** Dokument-Löschung über Store, Text, Vektor, Graph und Tombstone-Purge bleiben (`Purger`, `LayerScanner`). Zeugen, Signaturen, Revocation-Log, Shredding, Compliance-Export gehen ins Archiv. (Spec 2 hatte hier recht, Spec 1 nicht: `Purger` hat 4 Implementierungen [V].) |
| E5 | **`contextra-crypto` bleibt als schlanker Baustein** und `store → crypto` bleibt bestehen. Der WAL-HMAC (`wal_crypto`, `KeyManager`) ist Durability-Kern [V]. (Spec 1 wollte die Kante entfernen – nicht umsetzbar.) |
| E6 | **Eine Produkt-Fassade: `contextra`.** `contextra-db` wird in Phase 4 hineingefaltet, nicht vorher. (PyPI-Name, `py` und `mcp` hängen an `contextra` [V].) |
| E7 | **Tokio bleibt im Store.** 381 `async fn` [V]; kein Entkopplungsversuch in dieser Phase, stattdessen dokumentierte Grenze. |
| E8 | **`StoreIo` ist die eine I/O-Abstraktion des Stores.** `durable-fs` bleibt als eigenständiger Baustein, wird nicht verdrahtet. |
| E9 | **SIMD:** `contextra-simd` bleibt. AVX-512 hinter Feature `simd-avx512`. Standard ist Laufzeit-Dispatch (AVX2/NEON) mit Skalar-Fallback. Ein ADR mit Benchmark gegen `simsimd` ist Pflicht; Kriterium in §6.5. |
| E10 | **Engine-Chunker bleibt** (wird vom MCP benutzt); **kein Merge mit `ingest`** in dieser Phase. |
| E11 | **RRF:** `weighted_reciprocal_rank_fusion` ist die produktive Implementierung, `rrf_fuse` das Referenz-Orakel. Es sind **keine** echten Duplikate (§2). |
| E12 | **Der Evaluations-Harness ist Pflicht-Lieferung**, nicht Kür. Ohne gemessene Qualität ist der Kern nicht belegt. |
| E13 | **CI wird auf einen kleinen, funktionierenden Satz reduziert.** Der aktuelle `rust-ci.yml` ruft `cargo run -p xtask` auf, ein Paket, das es nicht gibt [V]. |

---

## 1. Befund: Was der Code wirklich sagt

### 1.1 Gemessene Größen (Zeilen `.rs`, [V])

| Crate | src | tests/ | `unsafe` | Normale interne Abhängigkeiten |
|---|---:|---:|---:|---|
| types | 8,2k | 1,7k | 0 | – |
| ports | 6,7k | 0,2k | 0 | types |
| sys | 2,2k | 0,2k | 22 | – |
| wire | 3,2k | 2,1k | 45 | – |
| durable-fs | 1,9k | 1,3k | 0 | – |
| mvcc | 4,3k | 2,3k | 0 | types, ports |
| simd | 3,3k | 2,0k | 269 | types (opt) |
| rank | 5,1k | 3,3k | 0 | types, ports (opt) |
| text | 9,4k | 7,7k | 0 | types, ports (opt) |
| graph | 22,5k | 9,1k | 0 | types, ports (opt) |
| vector | 16,4k | 10,2k | 0 | types, ports, mvcc, simd (alle opt), sys |
| store | 29,1k | 23,0k | 0 | types, ports, sys, mvcc, **crypto** |
| crypto | 8,5k | 9,1k | 0 | types, ports |
| checkpoint | 5,0k | 4,3k | 0 | types, ports |
| engine | 22,2k | 8,0k | 0 | types, ports, mvcc, **crypto**, store, vector, text, checkpoint, rank, graph; opt: sandbox, infer-onnx, adapt |
| db | 2,1k | 12,3k | 0 | engine, types, ports |
| contextra (Fassade) | 2,1k | 2,7k | 0 | types, ports, db; opt: license, infer-candle |
| mcp | 6,4k | 3,2k | 0 | contextra, types, ports, wire, rank, crypto, privacy, router; opt: infer-* |
| py | 3,5k | 0 | 0 | types, contextra, wire |
| ingest | 20,9k | 8,1k | 0 | – |
| kvcache | 6,8k | 2,9k | 0 | types, ports, crypto (alle opt) |
| erasure / audit-export / avv-generator / license | 16,3k / 1,0k / 1,1k / 1,1k | – | 0 | – |
| agent / cognition | 5,6k / 7,3k | 5,2k / 3,3k | 0 | agent→db,graph,checkpoint,router; cognition→engine,graph |

Weitere Fakten [V]: 36 Crates im Workspace, 35 Workflows, 20 `#[ignore]`-Tests, 46 `pub trait` in `ports`, 47 `ring =`/`klasse =`-Einträge in Manifesten, `AGENTS.md` erwähnt „ring" 26-mal, `tests/layering.rs` und `tests/unsafe_islands.rs` existieren.

### 1.2 Bewertung der Behauptungen beider Specs

| Behauptung | Quelle | Ergebnis |
|---|---|---|
| `store/src/lib.rs` deklariert `pub mod kv;`, Datei fehlt | Spec 2 B1 | **Bestätigt** [V]. `store` kompiliert nicht. Keine Verwendung von `crate::kv` im Store gefunden, Entfernen der Zeile genügt voraussichtlich. |
| `crypto/src/kv_segment/` ist leer | Spec 2 B2 | **Widerlegt** [V]: 6 Dateien (`store.rs` 33 KB, `segment.rs`, `manager.rs`, …). `kvcache` nutzt das Modul. |
| `rrf_fuse` und `reciprocal_rank_fusion` doppelt | Spec 2 B3 / Spec 1 | **Teilweise falsch** [V]. `rrf_fuse` arbeitet auf `ScoredItem` (String-IDs, `k` Parameter, deterministische Tie-Breaks). `reciprocal_rank_fusion` ist ein dünner Wrapper um `weighted_reciprocal_rank_fusion` auf `SearchResult`. Zwei Typdomänen, nicht zwei Kopien. |
| Chunker doppelt (`engine/chunker` 1,5k, `ingest/chunking` 7,4k) | Spec 2 B4 | **Bestätigt**, aber **keine Kopie**: Engine-Chunker ist nur Markdown und wird vom MCP über `contextra::chunker` benutzt [V]. Merge ohne Compiler ist riskant, Nutzen gering. |
| Engine hängt hart an `infer-onnx` und `sandbox` | Spec 2 B5 | **Veraltet** [V]. Beide sind bereits `optional`, ebenso `adapt`. Hart sind `crypto` und `mvcc`. |
| `store → crypto` entfernen | Spec 1 AP-2 | **Nicht umsetzbar** [V]: `wal/encode.rs`, `wal/io.rs`, `wal/replay.rs`, `lsm/recovery.rs` nutzen `crypto::wal_crypto`, `KeyManager`, `verify_wal_chain_completeness`. |
| `Purger`/`LayerScanner` entfernen | Spec 1 AP-4 | **Falsch** [V]: Definiert in `ports/src/scanner/`, implementiert in `store` (`LsmPurger`, `WalScanner`, `MemtableScanner`), `vector` (`HnswPurger`, `HnswScanner`), `text` (`TextPurger`, `TextScanner`), `kvcache`. Spec 1 verlangt selbst „Löschen verschwindet aus allen Indizes" (§5.2 Pfad 3). |
| `store` enthält eine `unsafe`-Stelle | Spec 1 AP-7 | **Nicht gefunden** [V]. Regex-Suche liefert 0, `forbid(unsafe_code)` ist gesetzt. |
| Workflows `unwrap-gate`, `rust-ci`, … sind funktionsfähige Gates | Spec 1 AP-10 | **Falsch** [V]. `rust-ci.yml` ruft `cargo run -p xtask` (existiert nicht, auch `justfile` bricht). `unwrap-gate.yml`: aktuell **1313** Treffer gegen Baseline **1197** → rot. |
| `crypto` hat `legacy-erasure` als Feature | beide | **Bestätigt, aber falsch voreingestellt** [V]: `default = ["legacy-erasure", "kv-encryption", "test-utils"]`; `kv-encryption` impliziert `legacy-erasure`. Der Standard-Build zieht also `ed25519-dalek` und alle Proof-Module. |
| Rings/Klassen sind Altlast | beide | **Bestätigt** [V] (Manifest-Metadaten, `tests/layering.rs`, `AGENTS.md`). |
| `ports` enthält Lösch-/Lizenz-/Ledger-Module | beide | **Bestätigt** [V]: `erasure`, `erasure_legacy`, `license`, `ledger`, `report_signer`, `observer` (`ErasureObserver`). `gate` (`RangeGate`/`ReadGate`) [H]: prüfen, ob Erasure-spezifisch. |
| `types/src/{erasure,witness}.rs` | beide | Existieren [V]. `tombstone.rs` bleibt (mechanischer Purge). |
| 20 ignorierte Tests | Spec 1 | **Bestätigt** [V]. |

---

## 2. Produktdefinition und Zielstruktur

### 2.1 Produkt

Ein Prozess öffnet ein Verzeichnis, legt Collections an, fügt Dokumente (Vektor + Text + optional Graph-Kanten) ein, sucht hybrid und bekommt nach Update, Löschen oder Absturz konsistente Ergebnisse. Eingang über Rust-API (`contextra`), Python (`py`) und MCP.

**Qualitätsversprechen, die belegt werden müssen** (nicht mehr, nicht weniger):

1. **Durability:** Nach Crash/Neustart liegt der Zustand bis zum letzten vollständig bestätigten Schreibvorgang vor. Kein Panic beim Öffnen beschädigter Dateien.
2. **Konsistenz:** Insert/Update/Delete wirken auf alle Signale gleichzeitig oder gar nicht sichtbar.
3. **Suchqualität:** Hybrid-Fusion ist messbar besser oder gleich dem besten Einzelsignal (Eval-Harness §7).
4. **Determinismus:** Gleiche Eingabe liefert gleiche Rangfolge, auch bei Gleichstand.

Keine Rechts-, Compliance- oder Sicherheitsversprechen („DSGVO-konform", „revisionssicher", „garantiert gelöscht") in Doku oder Code.

### 2.2 Kategorien

**Kern** (Default-Build, wird gehärtet):

| Crate | Zielabhängigkeiten (intern, normal) |
|---|---|
| `types` | – |
| `ports` | types |
| `sys` | – (unsafe-Ausnahme, mmap) |
| `wire` | – (unsafe-Ausnahme; hängt an `py`/`mcp`/`router`) |
| `mvcc` | types, ports |
| `crypto` (schlank) | types, ports |
| `store` | types, ports, sys, mvcc, crypto |
| `simd` | types (opt) (unsafe-Ausnahme) |
| `vector` | types, ports, mvcc, simd, sys |
| `text` | types, ports |
| `graph` | types, ports |
| `rank` | types, ports |
| `checkpoint` | types, ports |
| `engine` | types, ports, mvcc, store, vector, text, graph, rank, checkpoint (**≤ 9**); optional per Feature: `adapt`, `infer-onnx`, `sandbox` |
| `contextra` | types, ports, engine (+ `db` ab Phase 4); optional: `infer-candle` |
| `mcp` | contextra, types, ports, wire (+ `rank`); `privacy`, `router`, `crypto` bis Phase 4 geduldet, dann entfernen |
| `testkit` | types, ports (nur `[dev-dependencies]` der Nutzer) |
| `ingest` | – |

**Bausteine** (im Workspace, nicht im Default-Build, einzeln entnehmbar): `durable-fs`, `kvcache`, `adapt`, `router`, `privacy`, `infer-ollama`, `py` (Build via maturin), `benchmarks/contextra-bench`.

**Eingefroren** (nicht weiterentwickeln, nur hinter Feature oder außerhalb Default): `sandbox`, `infer-onnx`, `infer-candle`.

**Archiv** (`archive/`, außerhalb des Workspace, nicht gelöscht): `erasure`, `audit-export`, `avv-generator`, `license`, `agent`, `cognition`, dazu die Proof-Module aus `crypto` (§5.1) und die zugehörigen Tests.

Begründung `agent`/`cognition`: Beide hängen an `engine`/`db`/`graph`, tragen 12k+ Zeilen Tests, sind aber kein Teil des Retrieval-Kerns. Sie archiviert zu halten kostet nichts und entlastet jede Änderung an `engine`.

### 2.3 Default-Build

`default-members` = Kern ohne `py`. Inferenz-Crates nur per Feature. `experimental-*` nie im Default.

---

## 3. Verbindliche Regeln

1. **Verifizieren vor Handeln.** Alle [H]-Aussagen mit `grep`/`cargo metadata` prüfen. Maßgeblich ist der Code, nicht `AGENTS.md`, `PLAN.md`, `capabilities.toml`.
2. **Ein Arbeitspaket = ein Commit.** Format: `WP-xx: <Kurzbeschreibung>` plus Zeile `Prüfbefehl: … → grün|rot`.
3. **Archivieren vor Löschen.** Nichts wird gelöscht außer durch ein Arbeitspaket, das es ausdrücklich verlangt. Archivierte Dateien werden danach nicht verändert.
4. **Fremde Fehler nicht „mal eben" beheben.** Ursache außerhalb des Pakets → Zeile in `docs/handoff/OPEN_ISSUES.md`, stoppen.
5. **Keine neuen Features** außer Eval-Harness und Tests. Kein neuer Governance-Überbau (keine neuen Workflows, Gates, Ratchets).
6. **Keine Umbenennung** von Crates oder Paketen (`contextra-*` bleibt).
7. **Unsafe:** Erlaubt nur in `simd`, `sys`, `wire`; jeder Block mit `// SAFETY:`. Alle anderen Crates behalten `#![forbid(unsafe_code)]`. Die Anzahl der `unsafe`-Stellen darf nicht steigen.
8. **Fehlerbehandlung:** In neuem Nicht-Test-Code kein `unwrap`, `expect`, `panic!`, `todo!`. Fehler über `thiserror`, keine `Box<dyn Error>` in öffentlichen Signaturen. Die Workspace-Lints (`clippy::unwrap_used = deny`) sind maßgeblich [V].
9. **Persistenzformate** tragen eine Versionsnummer und haben einen Kompatibilitätstest (Fixture der Vorversion lesen).
10. **Neue externe Abhängigkeit** nur mit ADR in `docs/decisions/`.
11. **Features:** Default = stabiler Kern. Experimentelles heißt `experimental-*`. Keine Feature-Namen, die auf entfernte Funktionen verweisen (`erasure`, `shredding`, `audit`, `license`).
12. **Begriffe:** „Welle", „Klasse A", „Ring", „SPEC v4" kommen in aktivem Code, Kommentaren und Namen nicht mehr vor.
13. **Sprache:** Code und Fehlermeldungen Englisch; Dokumentation pro Datei einheitlich Deutsch oder Englisch.
14. **Tests nicht abschalten.** Schwellen werden nicht durch Entfernen von Tests erfüllt. `#[ignore]` nur mit Eintrag in `docs/IGNORED_TESTS.md`.
15. **Keine Historie umschreiben.** Kein `rebase`, `--force`, `--amend` auf gepushte Commits.
16. **Ehrlichkeit im Status.** Nicht erfüllte Kriterien bleiben im Statusbericht als nicht erfüllt stehen.

---

## 4. Arbeitspakete

Reihenfolge strikt. Phase n+1 beginnt erst, wenn die Gate-Bedingung von Phase n erfüllt oder in `OPEN_ISSUES.md` begründet ist.

### Phase 0 – Sicherung und Entsperren

**WP-00 Sicherung.** Tag `archive/pre-restructure-2026-11-09` auf HEAD setzen, pushen. Prüfung: `git tag | grep pre-restructure`.

**WP-01 Workspace-Manifest reparieren.** `crates/contextra-infer-ollama/Cargo.toml` hat zwei `[features]`-Blöcke [V]. Den zweiten löschen, den ersten behalten (`types-compat = [dep:ports, dep:types]`, `ports-compat = ["types-compat"]`, `http = […]`), es sei denn, `grep -rn "infer-ollama" crates/*/Cargo.toml` zeigt Konsumenten, die den anderen Zuschnitt erwarten. Auffällig [H]: Das Manifest listet `regex = "1.13"` und `quick-xml = "0.41"` als Abhängigkeiten eines Ollama-REST-Adapters; Herkunft prüfen, ungenutzte entfernen. Prüfung: `cargo metadata --no-deps >/dev/null`.

**WP-02 Store-Blocker.** In `crates/contextra-store/src/lib.rs` Zeile `pub mod kv;` entfernen [V: Modul fehlt]. Direkte Folgefehler im Store beheben (z. B. Verweise auf `KvDeleteMode`). Prüfung: `cargo check -p contextra-store --locked`.

**WP-03 Phantom-Tooling entfernen.** `justfile`-Ziele und CI-Schritte, die `cargo xtask`/`-p xtask` aufrufen, entfernen oder auskommentieren (Paket existiert nicht [V]). Nichts nachbauen. Prüfung: `grep -rn xtask justfile .github/workflows | wc -l` → 0 aktive Treffer.

**WP-04 Build-Leiter.** In dieser Reihenfolge je `cargo check -p <crate> --locked`:
`types → ports → sys → wire → mvcc → simd → crypto → store → rank → text → graph → vector → checkpoint → engine → contextra-db → contextra → mcp → py`.
Pro Fehlerklasse ein Commit; Fehler außerhalb der Restrukturierung in `OPEN_ISSUES.md`. Frozen-/Baustein-Crates (`adapt`, `router`, `privacy`, `kvcache`, `infer-*`, `sandbox`, `ingest`, `durable-fs`) werden danach versucht; was nicht in einer Sitzung baut, wird als `frozen` markiert und aus `default-members` genommen.
**Gate 0:** `cargo check` auf den Kern-Crates der Tabelle in §2.2 grün, oder jeder Fehler mit Ursache in `OPEN_ISSUES.md`.

### Phase 1 – Archiv-Schnitt

**WP-10 Archiv-Crates verschieben.** Vorab `grep -rn "contextra_erasure\|contextra-erasure"` (analog je Crate) und Treffer notieren. Dann `git mv crates/<name> archive/<name>` für `erasure`, `audit-export`, `avv-generator`, `license`, `agent`, `cognition`. Aus `[workspace].members`, `default-members`, `[workspace.dependencies]`, `capabilities.toml` entfernen. `archive/README.md`: Name, Originalzweck, Grund, Verweis auf Tag. Mechanisch, kein Inhalt wird geändert.
Abhängige Tests (`contextra/tests/deletion_chain_e2e.rs`, `contextra/tests/license_gate_always_enforced.rs`, Lösch-/Beweis-Tests in `db/tests`, `store/tests/campaign_j42_audit.rs`) nach `archive/tests-legacy/<crate>/` verschieben, nicht löschen.
Prüfung: `cargo metadata --no-deps >/dev/null && cargo check -p contextra-types`.

**WP-11 `contextra` von `license` lösen.** Feature `license` und `SignedLicenseGate` aus `builder.rs` entfernen. `boot.rs` nutzt `LicenseGate`/`FeatureRing` aus `ports::license` [V]: Gate-Aufrufe durch direkten Durchlauf ersetzen (die Funktion, die das Gate schützt, wird immer erlaubt). Keine Neuimplementierung. Prüfung: `cargo check -p contextra --no-default-features`.

**WP-12 `crypto` schlanken.**
1. Per `grep` je Modul die Nutzer bestätigen [V für die Tabelle unten, H für Feinheiten].
2. **Bleiben:** `aead`, `kdf`, `dek`, `secret`, `error`, `crypto` (`KeyManager`), `wal_crypto`, `wal_completeness`, `anti_tamper` (nach Prüfung), `audit_chain` bis `privacy` und `mcp` entkoppelt sind (Nutzer: `privacy`, `mcp`, `store`-Test).
3. **Nach `archive/contextra-crypto-proofs/`:** `deletion_proof/`, `ed25519_proof.rs`, `kv_shredding.rs`, `revocation_log.rs`.
4. **`kv_cipher.rs` und `kv_segment/`:** Einziger externer Nutzer ist `kvcache` [V] → in Phase 3 (WP-35) nach `kvcache` verschieben.
5. Default-Features auf `default = []` setzen; `legacy-erasure` ist kein Default und wird nach dem Verschieben ersatzlos entfernt; `kv-encryption` verweist nicht mehr auf `legacy-erasure`. `ed25519-dalek` entfernen.
Prüfung: `cargo check -p contextra-crypto -p contextra-store -p contextra-kvcache`; `cargo tree -p contextra-crypto -e normal | grep -c ed25519` → 0.

**WP-13 Löschdomäne aus Fundament entfernen.**
- `types`: `erasure.rs`, `witness.rs` nach Archiv. `tombstone.rs` bleibt.
- `ports`: `erasure`, `erasure_legacy`, `license`, `ledger`, `report_signer`, `ErasureObserver` aus `observer` nach Archiv. `scanner/` (mit `Purger`, `LayerScanner`) **bleibt**. `gate` prüfen.
- Doppelte `LayerWitness`-Familie entfällt mit dem Entfernen (`witness` wird in `store`, `text`, `testkit`, `kvcache` geprüft und abgebaut; neutral umbenennen, wo mechanisch nötig).
- `testkit`: Feature `erasure-conformance` entfernen.
- `store/src/lsm/config.rs`: Parameter `deletion_proof_active` neutral benennen oder entfernen (Verhalten unverändert, `// TODO(migration)` bei Unklarheit).
Prüfung: `grep -rniE "deletion_proof|kv_shredding|revocation_log|erasure_report|avv|audit_report" crates/*/src | wc -l` → 0; `cargo check -p contextra-types -p contextra-ports -p contextra-testkit`.

**WP-14 Engine entschlacken.**
- `contextra_impl/erasure.rs` (619 Z.) entfernen; Re-Export `crypto::deletion_proof` in `engine/src/lib.rs:20` entfernen.
- `contextra_impl/lifecycle.rs` (596 Z.): prüfen, ob Lösch-spezifisch [H]. Die **mechanische Löschung eines Dokuments** (Store, Text, Vektor, Graph, Kompensation) bleibt in `crud.rs`/`collection` und erhält Test WP-45.
- `contextra-crypto` aus den Engine-Abhängigkeiten entfernen (nur diese zwei Stellen nutzen es [V]).
- Engine-Features: `onnx`, `sandbox`, `adapt` bleiben optional; `default` minimal.
Prüfung: `cargo check -p contextra-engine --no-default-features`; `cargo tree -p contextra-engine -e normal --no-default-features --depth 1` zeigt ≤ 9 interne Crates, weder `crypto`, `sandbox`, `infer-onnx`; `grep -rn erasure crates/contextra-engine/src` → nur Migrationskommentare.

**WP-15 MCP entschlacken.** `proof_key.rs`, `proof_key_env.rs` und alle Beweis-/Zeugen-Pfade entfernen. Das vorhandene Tool `contextra_forget` wird auf „Dokument löschen ohne Nachweisobjekt" reduziert (Alias von `contextra_delete`). Vorhandene Tools bleiben: `contextra_search`, `contextra_get`, `contextra_collections`, `contextra_create_collection`, `contextra_drop_collection`, `contextra_delete`, `contextra_explain`. Prüfung: `cargo check -p contextra-mcp`; `grep -rniE "proof|witness|erasure_report" crates/contextra-mcp/src` → 0.

**WP-16 Ring-Rückbau.**
- `[package.metadata.contextra] ring/klasse` in allen Manifesten ersetzen durch `bundle = "<foundation|compute|storage|retrieval|ingest|orchestration|interfaces|edge|frozen>"` und `status = "<stable|beta|experimental|frozen>"`.
- `tests/layering.rs`, `tests/arch`, `tests/unsafe_islands.rs`, `tests/test_gate_g8_negative.py` nach `archive/tests-legacy/` verschieben.
- Ersatz: **ein** Skript `scripts/check_deps.sh`: liest `cargo metadata`, vergleicht normale interne Abhängigkeiten gegen `docs/CATALOG.md`, prüft `forbid(unsafe_code)` außer Allowlist `scripts/unsafe_allowlist.txt` (`simd`, `sys`, `wire`).
Prüfung: `scripts/check_deps.sh` → Exit 0; `grep -rnE "ring *=|klasse *=" crates/*/Cargo.toml | wc -l` → 0.

**Gate 1:** Workspace-Manifest gültig, Kern-Crates bauen, `grep` aus WP-13 liefert 0, `archive/` vollständig.

### Phase 2 – Abhängigkeiten und Features

**WP-20 Dev-Kanten säubern.** `store` hat Dev-Dependency auf sich selbst mit `test-hooks` (zulässig). Dev-Kanten auf höhere Crates (`graph`, `vector`, `db`, `engine`) in `store`, `graph`, `checkpoint`, `router` entfernen oder in ein neues Workspace-Member `tests/integration` verlegen. `checkpoint → testkit` nur in `[dev-dependencies]`. Prüfung je Crate: `cargo test -p <crate> --no-run`.

**WP-21 `ports` verschlanken.** 46 Traits [V]. Kriterium: Jeder Trait hat ≥ 2 Implementierungen oder eine Test-Implementierung in `testkit`; sonst Kandidat für Archiv oder `pub(crate)`. Mindestens bleiben: `Storage`/`StorageEngine`, `VectorIndex`, `TextIndex`, `GraphIndex`, `Purger`, `LayerScanner`, Embedding-Port. Jeder Trait hat Doku mit Beispiel. Plugin-/Policy-/Lifecycle-Traits: nach Nutzung per `grep` entscheiden und im Migrationslog begründen.

**WP-22 Features aufräumen.** In `engine`, `db`, `graph`, `rank`, `vector`: Experimentelles hinter `experimental-*` (siehe §5.3–5.5). Prüfung: `cargo check -p <crate> --no-default-features` und `--all-features` je Crate (Definition P1, §6).

**WP-23 Default-Members.** Gemäß §2.3 setzen. Prüfung: `cargo check` (Default) und `cargo metadata`.

**Gate 2:** `scripts/check_deps.sh` grün; `cargo check` Default grün.

### Phase 3 – Kern härten (Durability zuerst)

Reihenfolge: `store` → `mvcc` → `vector` → `text` → `rank` → `graph` → `engine`.

**Store**

- **WP-30 Crash-Konsistenz-Test.** WAL an **jeder Byte-Grenze** eines Beispiel-Logs abschneiden: Wiederöffnen lädt konsistent bis zum letzten vollständigen Eintrag **oder** schlägt sauber mit Fehler fehl, nie Panic. Dazu `proptest` mit zufälligen Operationsfolgen (put/get/delete/scan/snapshot, Neustart dazwischen) gegen `redb` (Differential-Test existiert: `tests/differential/redb_operation_sequence.rs` [V]); Abweichungen werden dokumentiert, nicht abgeschaltet. Mit `fault-injection` zusätzlich: Absturz zwischen WAL-Append, fsync, Memtable-Flush, Manifest-Update, Compaction-Commit.
- **WP-31 Format-Versionen.** WAL, SSTable (V4), Manifest: Versionsfelder dokumentieren, Fixtures unter `tests/fixtures/`, Test „liest Version n-1". Prüfung: `cargo test -p contextra-store --features test-hooks`.
- **WP-32 Shredding-Reste.** `grep -rniE "shredd|deletion_proof|kv_delete" crates/contextra-store/src` abbauen; neutral „tombstone purge" benennen.
- **WP-33 Scanner/Systemdruck prüfen.** `scanner/` ist notwendig (Purger/Scanner-Implementierungen [V]). `system_pressure.rs`, `tenant_codec.rs`: Nutzer per `grep` prüfen, ungenutzte archivieren. Dateien > 800 Zeilen aufteilen (Store: `wal/`, `lsm/`, `sstable/` prüfen).
- **WP-34 Tokio-Grenze dokumentieren.** README „Bekannte Grenzen": benötigt Tokio-Runtime. Kein Umbau.
- **WP-35 `kvcache`/`crypto`.** `kv_cipher.rs` und `kv_segment/` von `crypto` nach `kvcache` verschieben, wenn `kvcache` einziger Nutzer bleibt [V]. README von `kvcache` ehrlich: „Prefix-Cache-Verwaltung für KV-Segmente; keine Anbindung an eine konkrete Inferenz-Runtime im Repo."

**mvcc**

- **WP-36** Isolationstests: Snapshot sieht keine späteren Commits, Write-Konflikte werden erkannt; `loom`-Test für die zentrale Struktur (falls nicht vorhanden).

**vector**

- **WP-40 Recall-Referenz.** 10 000 Zufallsvektoren, 128 d, fester Seed, Brute Force als Orakel, HNSW Recall@10 ≥ 0,95 bei dokumentierten `M` und `ef`. Verfehlen: Konfiguration/Bug dokumentieren, Schwelle nicht senken.
- **WP-41 Lösch-Konsistenz.** n einfügen, Teilmenge über `purge`/`deletion` entfernen: Gelöschte IDs erscheinen nie; Recall der Restmenge bleibt über Schwelle; `partial_rebuild` stellt Konnektivität her.
- **WP-42 Persistenz.** `HnswHeader`: Version + Checksumme, Test; Fuzz-Target für den `MmapIndex`-Header-Parser (anlegen, nicht nur dokumentieren).
- **WP-43 Verschlanken.** `acorn/`, `diskann/`, `quantize_rabitq.rs`, `predicate_augmented`: Nur HNSW + Persistenz + Filter sind Kern. `diskann`, `rabitq` bleiben feature-gesperrt (`experimental-*`), `acorn` nur behalten, wenn `hnsw/acorn_filtered.rs` es für Filter braucht [H].

**text**

- **WP-50 BM25-Referenz.** Gegen `tantivy` (dev-dependency, ADR) auf kleinem Korpus; Top-10-Überlappung ≥ dokumentierter Schwelle; Abweichungen (Tokenisierung, Stoppwörter) im README erklären. **Block-Max-WAND-Ergebnis muss identisch zur exhaustiven Auswertung sein** (Property-Test).
- **WP-51 Purge-Test.** Gelöschtes Dokument erscheint in keiner Query (analog WP-41).
- **WP-52 Morphologie.** Auf das für BM25 nötige Maß reduzieren; `morphology/` Rest hinter `experimental`.

**rank** (höchster Hebel, kleinster Aufwand)

- **WP-60 Fusion klarstellen.** Doku-Entscheidung E11 umsetzen: `weighted_reciprocal_rank_fusion` produktiv; `reciprocal_rank_fusion` bleibt als dünner Wrapper (Doku „unit weights, k=60"); `rrf_fuse` bleibt als Referenz-Orakel. `engine/src/fusion.rs` bleibt Re-Export. Differential-Test: gleiche Rangfolge von `rrf_fuse` und `weighted_reciprocal_rank_fusion` bei Einheitsgewichten.
- **WP-61 Golden-Tests.** Handgerechnet, Rechnung im Test dokumentiert: 2 Listen, 3 Listen, Gleichstand, ID fehlt in einer Liste, Gewichte 0 und 1, leere Eingabe, doppelte IDs in einer Liste.
- **WP-62 Reduktion.** Default: RRF, gewichtetes RRF, normalisierte Linearkombination. `mrrf` (steht aktuell **in `rrf.rs`** [V]), `resonance`, `global`, `dibud`, `calibration/`, `drift`, `explain`-Erweiterungen hinter `experimental-rank`; README nennt sie „nicht evaluiert".
- **WP-63 Determinismus.** Property-Test: Permutation der Eingabelisten bei gleichen Gewichten ändert die Ausgabe nicht; Tie-Break nach ID.

**graph**

- **WP-70 Referenzwerte.** PPR gegen Power-Iteration auf Zufallsgraphen (dokumentierte Toleranz) und gegen handgerechnete Graphen ≤ 10 Knoten.
- **WP-71 Verschlanken.** Kern ist CSR, Persistenz, PPR, `purger.rs`. Hinter `experimental-graph` bzw. Archiv: `community/`, `hyperedge*.rs`, `session_dag.rs`, `cascade.rs`, `percolation.rs`, `approh`, `tl_hfd`, `path_rag`, `edge_reinforcement*`, Diffusions-Features. Per `grep` prüfen, was `engine` tatsächlich nutzt; Genutztes bleibt zunächst.
- **WP-72 Entity-Extraktion messen.** Goldstandard 30–50 Sätze (de/en), Precision/Recall als Bericht. Nur Messung. Ergebnis entscheidet, ob das Graph-Signal in der Standard-Suche aktiv ist.

**engine**

- **WP-80 End-to-End ohne Inferenz.** Collection anlegen, N Dokumente mit festen Vektoren und Text, Hybrid-Suche (Vektor + BM25 + Graph), Fusion prüfen, Dokument löschen, erneut suchen → in allen drei Signalen verschwunden. Läuft mit `--no-default-features`.
- **WP-81 Rollback.** Fehler im zweiten Index-Schritt (Kompensation) → keine Teil-Schreibung sichtbar.
- **WP-82 Hintergrund-Worker.** `expiry`, `orphan`, `hyperedge` einzeln abschaltbar; Standard minimal. `decay_controller`, `extraction`, `orchestration/`: Nutzung prüfen, Ungenutztes hinter Feature.
- **WP-83 `collection/` (12,5k Zeilen) strukturieren.** Dateien > 800 Zeilen aufteilen; keine Logik in `lib.rs`.

**simd**

- **WP-90 Kernel-Tests.** Jeder Kernel hat einen Skalar-Referenztest mit Zufallseingaben, inkl. Längen 0, 1, 7, 8, 15, 16, 31, 33, nicht ausgerichtet. `// SAFETY:` je `unsafe`-Block (`grep -c unsafe` vs. `grep -c SAFETY` je Datei). Miri nur wo lauffähig.
- **WP-91 SIMD-ADR** gemäß §6.5.

**Gate 3:** Tests der Kern-Crates ausgeführt, Ergebnis in `docs/handoff/STATUS.md` (grün oder Liste der fehlschlagenden Tests).

### Phase 4 – Fassade und Schnittstellen

**WP-100 `db` in `contextra` falten.** Voraussetzung: Gate 3 erfüllt, Baseline-Testergebnis für `db` und `contextra` festgehalten. Vorgehen: `db`-Quellen (`homeostat`, `multistep`, `pid_latency_controller`, `volatile_vault`, `prelude`) nach `contextra/src/` verschieben, die 12,3k Zeilen `db/tests` mit nach `contextra/tests/`. `bench` auf `contextra` umhängen (`agent` ist archiviert). Wenn das Umhängen nicht in einer Sitzung gelingt: `db` bleibt als internes, nicht veröffentlichtes Crate; Eintrag in `OPEN_ISSUES.md`. Prüfung: `cargo test -p contextra --no-run`.

**WP-101 Python-Bindings.** `py` baut gegen `contextra`; Smoke-Test: einfügen, suchen, löschen, Neustart (analog Pflichtpfade §5.2). `contextra-py-ci.yml` bleibt.

**WP-102 MCP entkoppeln.** `privacy`, `router`, `crypto` aus `mcp` entfernen, soweit der Code (egress guard, prompt injection, bulk-exfiltration detector) nicht Teil des Such-Kerns ist; Verbleibendes in ein Feature `guards`. Entscheidung per `grep`; nichts neu schreiben.

### Phase 5 – Evaluation

Siehe §7.

### Phase 6 – Dokumentation und CI

Siehe §8 und §9.

---

## 5. Testanforderungen

### 5.1 Pflicht-Pfade (End-to-End, auf `contextra`/`engine`)

Mindestens ein Test pro Pfad:

1. Einfügen → Suche liefert Treffer.
2. Update → alte Version wird nicht mehr gefunden.
3. Löschen → Treffer verschwinden aus Vektor-, Text- und Graph-Index.
4. Neustart → Daten und Indizes sind wieder da.
5. Crash während Schreiben (`fault-injection` / `testkit::FaultVfs`) → Zustand konsistent.
6. Hybrid-Suche mit einem, zwei, drei Signalen → erwartbare Rangfolge.

### 5.2 Teststrategie

| Ebene | Werkzeug | Pflicht für |
|---|---|---|
| Unit | `cargo test` | alle |
| Golden/Referenz | handgerechnet, Brute Force, `tantivy` | `rank`, `text`, `vector`, `graph`, `simd` |
| Differential | `redb` | `store` |
| Property | `proptest` | `store`, `rank`, `text`, `mvcc` |
| Nebenläufigkeit | `loom` | `mvcc`, `store` |
| Fuzz | `cargo fuzz` | WAL-Decoder, SSTable-Decoder, HNSW-Header, Text-Tokenisierung, Ingest-Parser, MCP-Eingabe |
| Integration | `tests/integration` (eigenes Crate) | Zusammenspiel `engine` |

**Regel:** Ein Test zählt nur, wenn seine Erwartung **nicht** aus der getesteten Funktion selbst abgeleitet ist.

### 5.3 Abdeckung (einmal gemessen, in `docs/COVERAGE.md`)

`rank`, `mvcc` ≥ 85 %; `store`, `vector`, `text`, `graph` ≥ 75 %; `engine` ≥ 65 %; Bausteine ≥ 60 %; Archiv nicht gemessen. Nicht erfüllte Ziele werden dokumentiert, nicht kaschiert. (Vereinheitlicht aus den abweichenden Zielen beider Specs.)

### 5.4 Ignorierte Tests

Alle 20 `#[ignore]` [V] in `docs/IGNORED_TESTS.md` mit Grund und Entscheidung (reparieren, archivieren, löschen nur nach Freigabe).

### 5.5 Unwrap-Schuld

Der grep-basierte `unwrap-gate` ist aktuell rot (1313 > 1197 [V]) und redundant zu `clippy::unwrap_used = deny` [V]. Entscheidung: Baseline einmalig auf den gemessenen Wert setzen (Commit mit Begründung), Bestandstreffer in `docs/UNWRAP_DEBT.md` listen (nach Crate gruppiert), danach darf der Wert nur sinken.

---

## 6. Produktionsreife und Durability-Spezifikation

### 6.1 Definition „produktionsreif" (jede Kern-Crate)

- **P1** `cargo build --no-default-features` und `--all-features` grün.
- **P2** `cargo clippy -D warnings` sauber im Nicht-Test-Code.
- **P3** `#![warn(missing_docs)]`; öffentliche API in `API.md`; nicht dokumentierte `pub`-Items sind `pub(crate)`.
- **P4** Jede öffentliche Hauptfunktion hat einen implementierungsunabhängigen Test.
- **P5** Persistenzformate versioniert, Kompatibilitätstest vorhanden.
- **P6** README mit: Zweck · Reifegrad · Interne Abhängigkeiten · Features · Minimalbeispiel (Doctest) · Entnahme-Rezept · Bekannte Grenzen.
- **P7** Fuzz- oder Property-Test an jeder Grenze, an der fremde Bytes gelesen werden.

### 6.2 Durability-Vertrag des Stores

| Ereignis | Erwartetes Verhalten | Beleg |
|---|---|---|
| Crash nach WAL-Append, vor fsync | Eintrag darf fehlen, Zustand bleibt konsistent | WP-30 (Fault-Injection) |
| Crash nach fsync, vor Memtable-Flush | Eintrag wird beim Öffnen aus dem WAL wiederhergestellt | WP-30 |
| WAL am Ende abgeschnitten | Laden bis zum letzten vollständigen Eintrag **oder** sauberer Fehler | WP-30 (jede Byte-Grenze) |
| Flipped Bit im WAL/SSTable | Erkennung (HMAC/CRC), kein Panic | WP-30, Fuzz |
| Crash während Compaction/Manifest-Wechsel | Altes oder neues Manifest vollständig gültig, nie gemischt | WP-30 |
| Unbekannte Formatversion | Sauberer Fehler, kein Datenverlust | WP-31 |

Bestätigte Kommit-Semantik (Wann gilt ein Put als dauerhaft?) wird in `store/README.md` genau einmal formuliert und durch einen Test belegt. Das Verhalten ist **konfigurationsabhängig** (`DurabilityMode`) und wird je Modus beschrieben, nicht pauschal versprochen.

### 6.3 Such-Semantik

- Löschen und Update sind nach Rückkehr des Aufrufs in allen aktiven Signalen wirksam (kein „eventually").
- Fusion ist deterministisch (Tie-Break nach ID aufsteigend [V in `rrf_fuse`]; für den produktiven Pfad per WP-63 zu belegen).
- Hybrid-Suche liefert bei fehlendem Signal (z. B. kein Graph) das Ergebnis der verbleibenden Signale, nie einen Fehler.

### 6.4 Persistenz-Kompatibilität

Format-Änderungen erhöhen die Versionsnummer; ältere Versionen werden gelesen oder lehnen sauber ab. Migration läuft nie implizit beim Öffnen im Lesemodus.

### 6.5 SIMD-ADR (Kriterium, Entscheidung steht fest)

Benchmark in `benchmarks/` auf 128, 384, 768, 1024 Dimensionen für L2, Cosine, Dot: Eigener Kernel, `simsimd`, Skalar mit Autovektorisierung.
- Eigener Code bleibt, wenn er ≥ 1,2× schneller als Autovektorisierung ist **und** per Referenztests (WP-90) abgesichert ist.
- AVX-512 bleibt nur hinter `simd-avx512`; ist der Gewinn gegenüber AVX2 < 1,1×, wird der AVX-512-Kern archiviert.
- Wenn `simsimd` den eigenen Code in allen Fällen ≥ 1,2× schlägt, wird `simsimd` hinter Feature `simd-external` eingebunden, die eigenen Kernel bleiben als Referenz/Fallback.
- `docs/decisions/0001-simd.md` enthält Zahlen, Hardware und Reproduktionsbefehl.

---

## 7. Evaluations-Harness (Pflicht-Lieferung)

Basis: `benchmarks/contextra-bench` (3,6k Zeilen src [V]; enthält `beir_eval.rs`, `path_rag_sweep.rs`, `bin/scale_100k.rs`).

- **WP-110 Reaktivieren.** Baut gegen `contextra`. Veraltete Läufe (50 Chunks, Reranker +0 %) als `historical` markieren.
- **WP-111 Datensätze.** (a) Öffentliche IR-Testmenge (BEIR-Teilset; Lizenz prüfen; Download-Skript, keine Daten im Repo). (b) Eigener deutscher Mini-Korpus: ≥ 200 Dokumente, ≥ 50 Queries, Relevanzurteile, lizenz-unkritisch.
- **WP-112 Metriken.** Recall@10, nDCG@10, MRR@10, Latenz p50/p95/p99, RAM-Spitze, Indexaufbauzeit.
- **WP-113 Vergleichsmatrix.** Vektor allein · BM25 allein · RRF(V+T) · RRF(V+T+G) · gewichtete RRF · normalisierte Linearkombination. Ausgabe `benchmarks/results/<datum>.md` und `.json`.
- **WP-114 Skalierung.** 100 000 Dokumente. Scheitert der Lauf („Memory budget exceeded"), wird die Ursache beschrieben (Budget-Konfiguration vs. echter Bedarf); das Budget wird nicht stillschweigend erhöht.
- **WP-115 Bericht.** `docs/EVALUATION.md`: ehrliche Tabelle, Umgebung (CPU, RAM, Rust-Version), Reproduktionsbefehl. Kein Marketing.
- **Vorbereitung** (nicht Teil dieser Phase): Schnittstelle für Vergleich gegen Tantivy und LanceDB.

Prüfung: `cargo run -p contextra-bench --release -- --help` und ein Lauf auf dem Mini-Korpus.

---

## 8. Dokumentation

Pflichtdateien (Phase 6, vor Abschluss):

| Datei | Inhalt |
|---|---|
| `README.md` | Was, für wen, Schnellstart, Crate-Übersicht (Kern/Baustein/Eingefroren/Archiv) |
| `ARCHITECTURE.md` | Abhängigkeitsgraph (Mermaid, aus `cargo metadata` abgeleitet), Ports-Konzept, Durability-Vertrag |
| `AGENTS.md` | Gekürzt auf §3 und §4 dieser Spec; keine Ring-Aussagen (aktuell 26 Treffer [V]) |
| `docs/CATALOG.md` | Pro Crate: Bundle, Zweck, Reifegrad, interne Abhängigkeiten (aus `cargo metadata`), Entnahmeaufwand, Link |
| `docs/EXTRACTION.md` | Entnahme-Rezept je Crate; `tools/export_crate.py` prüfen und nur referenzieren, wenn funktionsfähig |
| `docs/MIGRATION_LOG.md` | Chronologisch, je WP: Dateien, mögliche Compiler-Folgen |
| `docs/handoff/OPEN_ISSUES.md`, `docs/handoff/STATUS.md` | Offene Punkte; Abnahme-Checkliste mit Befehl und Ergebnis |
| `docs/decisions/` | ADRs: 0001 SIMD, 0002 Archivierung/Crate-Schnitt, 0003 Ring-Rückbau, 0004 `crypto` bleibt schlank, 0005 `contextra` als Fassade, 0006 Tokio-Grenze |
| `docs/CI.md`, `docs/COVERAGE.md`, `docs/IGNORED_TESTS.md`, `docs/UNWRAP_DEBT.md`, `docs/EVALUATION.md` | siehe jeweilige Abschnitte |
| `<crate>/README.md`, `<crate>/API.md` | Kern- und Baustein-Crates |
| `archive/README.md` | Archivierte Crates mit Grund und Tag-Verweis |

Veraltete Dokumente (`PLAN.md`, `DEPRECATED_DEBT.md`, `capabilities.toml`, `mutants-guarantees.toml`, `docs/archive/*`, `docs/reports/*`, `docs/CONSTITUTION.md`, `docs/VETOES.md`, `docs/SPEC.md`, `docs/SOURCE_OF_TRUTH.md` [H: Inhalt prüfen]) → `docs/_history/` bzw. `archive/docs/`, nicht löschen. `tools/*.py` und `governance/` nicht anfassen; `tools/README.md` markiert sie „Legacy, nicht gepflegt".

Keine Aussage in READMEs ohne Codebeleg. Minimalbeispiele müssen als Doctest laufen.

---

## 9. CI-Zielzustand

Ersatz für die 35 Workflows. Pro Workflow eine Zeile in `docs/CI.md`.

**Behalten (nach Prüfung, dass sie nur existierende Members referenzieren):**

| Workflow | Zweck | Änderung |
|---|---|---|
| `rust-ci.yml` | Check, fmt, clippy `-D warnings`, Tests der Default-Members | `xtask`-Schritte entfernen; Coverage-Gate (aktuell workspace-weit ≥ 50 %) auf die Kern-Crates beschränken oder nicht blockierend machen |
| `deny.yml` | Lizenzen/Advisories | — |
| `unwrap-gate.yml` | Zählt Produktions-`unwrap` | Baseline neu setzen (§5.5) |
| `loom-gate.yml` | Nebenläufigkeit `mvcc`/`store` | Zielcrates prüfen |
| `fuzz-extended.yml` | Fuzz-Targets | nur existierende Targets |
| `bench.yml`, `bench-regression.yml` | Benchmarks | manuell / nicht blockierend |
| `mutation-testing.yml` | Mutation | geplant, nicht blockierend |
| `contextra-py-ci.yml`, `build-wheels.yml`, `publish-pypi.yml`, `release-mcp-binary.yml` | Release-Pfad | Ziele nach Fassadenumbau prüfen |
| `tsan.yml`, `simd-cross-validation.yml` | Concurrency/SIMD-Validierung | behalten, wenn lauffähig |

**Nach `.github/_disabled/`:** `jules-dispatch`, `refresh-jules-context`, `update-prompter-data`, `verdict`, `veto-deadline-gate`, `shell-commit-gate`, `commit-integrity-guard`, `prune-branches`, `nucleation-recall-history`, `proof-correctness`, `audit-integrity`, `scheduled-audit`, `baukasten-gates`, `context-gates`, `deep-gates`, `merge-gate`, `post-merge-verification`, `flake-report`, `chaos`, `unwrap-baseline-check`.

Prüfung: `actionlint`, falls verfügbar; sonst YAML-Syntax-Check.

---

## 10. Verbotene Handlungen

- Neue Features, Löschlogik, Compliance- oder Nachweis-Mechanismen einbauen.
- Crypto-Code „verbessern" (er wird verschlankt oder archiviert, nicht erweitert).
- Archivierte Dateien ändern (außer Dokumentation hinzufügen).
- Crates löschen statt archivieren.
- Ring-Tabellen oder -Kommentare neu einführen; neue Gates, Ratchets, Governance-Skripte.
- Neue externe Abhängigkeiten ohne ADR.
- Tests entfernen oder `#[ignore]` setzen, um Schwellen zu erreichen.
- Die Anzahl der `unsafe`-Stellen erhöhen.
- Schwellwerte (Recall, Coverage, Memory-Budget) stillschweigend senken oder erhöhen.
- Historie umschreiben.

---

## 11. Abnahme

Jede Zeile mit Befehl und Ergebnis in `docs/handoff/STATUS.md`; Nicht-Erfülltes bleibt als nicht erfüllt stehen.

- [ ] **Gate 0:** `cargo metadata` gültig; Kern-Crates `cargo check` grün oder dokumentiert (WP-01 bis WP-04).
- [ ] `cargo tree -p <crate> -e normal --depth 1` entspricht §2.2 und `docs/CATALOG.md` (`scripts/check_deps.sh` grün).
- [ ] `grep -rniE "deletion_proof|kv_shredding|revocation_log|erasure_report|avv|audit_report|license" crates/*/src` → 0 Treffer (außer Migrationskommentare).
- [ ] `cargo tree -p contextra-crypto -e normal | grep -c ed25519` → 0.
- [ ] `engine` ohne Default-Features baut; ≤ 9 interne Abhängigkeiten; kein `crypto`.
- [ ] `cargo test -p contextra-rank -p contextra-mvcc -p contextra-simd -p contextra-text` grün.
- [ ] `cargo test -p contextra-store -p contextra-vector -p contextra-graph -p contextra-engine` ausgeführt; Ergebnis dokumentiert.
- [ ] Alle sechs Pflicht-Pfade (§5.1) mit Test.
- [ ] WAL-Truncation-Test (WP-30) und Recall-Test (WP-40) vorhanden und ausgewertet.
- [ ] `docs/EVALUATION.md` mit Vergleichsmatrix und Mini-Korpus-Lauf.
- [ ] `archive/` enthält die sechs Crates plus `contextra-crypto-proofs`, Workspace referenziert sie nicht.
- [ ] Alle Pflichtdokumente (§8) existieren; Kern-READMEs erfüllen P6, Doctests laufen.
- [ ] ADRs 0001 bis 0006 vorhanden; Entscheider hat SIMD und die Archivierung bestätigt.
- [ ] Git-Tag `handoff-2026-11-11` gesetzt.

---

## 12. Offene Punkte und Unsicherheiten

| # | Punkt | Auswirkung |
|---|---|---|
| U1 | **Nichts wurde kompiliert.** Weitere Build-Blocker neben WP-01/WP-02 sind wahrscheinlich, besonders nach den Löschungen in Phase 1. | Phase 0 ist bewusst als Schleife angelegt. |
| U2 | Zeilenzahlen weichen von beiden Specs ab (z. B. `store` src: 29,1k statt 22,8k). Gemessen mit `wc -l` auf `src/` und `tests/`; Inline-Tests in `src/` sind mitgezählt. | Nur für Priorisierung relevant. |
| U3 | `engine/contextra_impl/lifecycle.rs`, `ports/gate`, `anti_tamper`, `audit_chain`, `engine/decay_controller` wurden nicht im Detail gelesen. | Entscheidung je per `grep` vor dem Verschieben (WP-12/13/14). |
| U4 | `shell`-/Kosten-Annahmen aus Spec 2 („195 Sitzungen pro Tag") sind nicht Teil dieser Spec. Sie ist nach Abhängigkeit geordnet, nicht nach Zeit. | Die Phasen-Gates ersetzen den Zeitplan. |
| U5 | Ob `contextra-db` ohne Bruch in `contextra` aufgeht, hängt von 12,3k Zeilen Tests ab. | Fallback in WP-100 definiert. |
| U6 | MCP-Tools `create/drop collection` und `delete` sind Schreibzugriffe; ob ein Nur-Lese-Betrieb per vorhandener Konfiguration möglich ist, wurde nicht geprüft [H]. | Kein neues Feature bauen; nur dokumentieren. |
