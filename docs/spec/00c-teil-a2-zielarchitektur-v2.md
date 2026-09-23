---
source: CONTEXTRA_SPEC_v4_MASTER.md
chapter: "00c"
---
## Teil A2 — Zielarchitektur v2 (Ring-Modell), verbindlich ab sofort

> Rang: ranghöher als §3–§9, nachrangig zu Teil A (§A.1–§A.5). Quelle: `CONTEXTRA_ZIELARCHITEKTUR.md` (v1,
> Basis `3f37ab30`) und ihre Korrekturfassung `CONTEXTRA_ZIELARCHITEKTUR_v2.md` (v2, Basis `806a40c1`, ersetzt v1
> vollständig). Kennzeichnung wie in der Quelle übernommen: **[V]** am Repo maschinell geprüft, **[D]** aus
> Quelltext/Cargo-/rustc-Semantik abgeleitet, nicht ausgeführt, **[U]** Urteil.

### A2.0 Warum diese Fassung existiert — vorher/jetzt in einem Satz

**Vorher (Stand dieser Spec bis zur Vorfassung):** ein Layer-0–5-Crate-DAG mit 10 benannten Crates (§4 alt),
`contextra-db` als monolithische Fassade mit über 40 öffentlichen Methoden, KV-Cache-Bridge als 🟢 markiert,
`#![forbid(unsafe_code)]` als Default mit sechs pauschalen Ausnahme-Crates, Reifegrad-Marker handgepflegt.
**Jetzt (diese Fassung, ab sofort im Code umzusetzen):** ein Ring-0–4-Crate-Graph mit 27 Fach- plus 3
Tooling-Crates, `contextra-db` wird schrittweise in `engine`/`cognition`/`rank`/`adapt`/`router`/`privacy`
zerlegt, KV-Cache auf 🔴 zurückgestuft und in drei Ausbaustufen A/B/C neu spezifiziert, genau drei benannte
Unsafe-Inseln mit `deny`+`forbid`-Mechanik statt pauschalem `forbid`+Insel-`allow` (letzteres ist wegen E0453
gar nicht baubar), Reifegrad-Marker sollen aus `capabilities.toml` generiert werden. Der Übergang erfolgt
strangler-artig (§20), nicht per Big-Bang-Rewrite.

### A2.1 Warum v1 nicht unverändert übernommen wird

v1 (`CONTEXTRA_ZIELARCHITEKTUR.md`) war architektonisch richtig ausgerichtet (Ports/Adapter, Sync-Kern,
gestufter KV-Ausbau, generierte Spec), enthielt aber neun am Code belegte Defekte, die den direkten Weg in
diese Spec verboten hätten. Maßgeblich ist ausschließlich v2; v1 wird hier nur referenziert, wo sie den
historischen Ausgangspunkt einer Entscheidung erklärt.

| # | Defekt in v1 | Beleg (v2) | Konsequenz in dieser Spec |
|---|---|---|---|
| D1 | Unsafe-Inventar unvollständig (v1: nur `simd` + mmap-Wrapper) | **[V]** Win32-ACL in `wal/io.rs` (13 unsafe), mmap in `wal/replay.rs`/`index/persistence.rs`, `mlock` in `db/volatile_vault.rs` (4), SIMD in `distance.rs` (73), Generat in `core-ipc-gen` (44) | Dritte Insel `contextra-sys` (§0.4 neu) |
| D2 | Lint-Mechanik `forbid` + Insel-`allow` nicht baubar | **[D]** `forbid` ist per Rust-Semantik nicht lokal überschreibbar (E0453); Cargo verbietet `[lints] workspace = true` neben eigenen `[lints.*]`-Tabellen | Workspace-`deny` + `#![forbid]` pro Nicht-Insel-Crate (§0.4 neu) |
| D3 | `contextra-checkpoint` fälschlich in `contextra-store` verschmolzen gedacht | **[V]** `checkpoint` hängt nur an `core`/`StorageEngine`, nicht an `store`; eigener globaler Zustand (`ORPHAN_REGISTRY`) muss entfernt werden | `checkpoint` bleibt eigenes Ring-1-Crate, ohne globalen Zustand |
| D4 | `contextra-core`-Zerlegung im Migrationsplan unvollständig | **[V]** `core` = 10.353 LOC; `tx_buffer`, `seq_log`, `snapshot` u. a. waren v1 nicht zugeordnet | Neues Ring-0-Crate `contextra-mvcc` |
| D5 | Kennzahlen (async fn, Panic-Stellen) enthielten Testcode, waren 3–20× zu hoch | **[V]** echte Prod-Zahlen deutlich kleiner (§A2 unten, Migrationsplan) | Aufwandsschätzung Phase 2/Panic-Politik nach unten korrigiert |
| D6 | `indexing_slicing = deny` workspace-weit geplant | **[V]** ≈ 640 Index-/Slice-Ausdrücke in Hot-Paths | `deny` nur an Parsing-Grenzen, sonst `warn` + `debug_assert!` |
| D7 | Plan sah `deny.toml` „neu anlegen" vor | **[V]** `deny.toml` existiert bereits seit `81edd9af` (Lizenzen, Quellen, Bans) | Erweitern statt neu anlegen |
| D8 | Prüfbefehl `cargo check --workspace` sollte Netzfreiheit belegen | **[D]** `--workspace` ignoriert `default-members`; Netzfreiheit kommt von der `onnx-bench`-Feature-Trennung | Exit-Kriterien in §20 korrigiert |
| D9 | KV-Optionen unvollständig (Toleranz, Speicherebene, dtype ungeklärt) | **[D]** `ModelWeights` ist `Clone`; naives Spillen großer KV-Blöcke in eine LSM-Engine ist ein Fehlgriff (Write-Amplifikation) | KV in drei Stufen A/B/C, siehe §9 |

### A2.2 Entfallende, neue und unveränderte Crates gegenüber der Vorfassung

**Entfallen als eigenständige Crates** (gehen in die Ring-Struktur auf, Re-Export mit `#[deprecated]` während
der Strangler-Phase, §20): `contextra-core`, `contextra-core-ipc-gen`, `contextra-checkpoint` *(bleibt de facto
erhalten, siehe D3 — nur die v1-Fusionsidee entfällt)*, `contextra-calibration`, `contextra-router` *(bleibt
erhalten, wird nur schlanker)*, `contextra-embed`, `contextra-candle`, `contextra-ollama`, `contextra-db`.

**Neu:** `contextra-types`, `contextra-ports`, `contextra-mvcc`, `contextra-wire` (vormals `core-ipc-gen`),
`contextra-sys`, `contextra-simd` (vormals Teil von `contextra-index`), `contextra-vector` (vormals `contextra-index`),
`contextra-rank` (vormals Teil von `contextra-db` + `contextra-calibration`), `contextra-adapt` (vormals Teil von
`contextra-router` + `contextra-db`), `contextra-kvcache` (vormals Teil von `contextra-crypto` + `contextra-candle`),
`contextra-infer-candle`/`-ollama`/`-onnx` (vormals `contextra-candle`/`-ollama`/`-embed`), `contextra-engine`,
`contextra-cognition`, `contextra-privacy` (vormals Teile von `contextra-crypto` + `contextra-mcp`), `contextra`
(neue, einzige Fassade/Composition Root).

**Unverändert im Zuschnitt, nur ggf. intern angepasst:** `contextra-store`, `contextra-crypto` (schlanker, ohne
Egress-Vault/KV-Segment, die nach `privacy`/`kvcache` wandern), `contextra-text`, `contextra-graph`,
`contextra-sandbox`, `contextra-agent`, `contextra-mcp`, `contextra-py`, `contextra-bench`, `xtask`.

Vollständige Ring-Landkarte, Abhängigkeitsmatrix und Herkunftszuordnung: §4. Migrationsreihenfolge: §20.

### A2.3 Leitprinzipien-Delta gegenüber §3 (P1–P25)

Details und vollständiger Wortlaut der fünf neuen Prinzipien P26–P30 sowie der Δ-Änderungen an P5/P6/P9/P10/P11
stehen in §3 selbst. Zusammengefasst: Sync-Kern wird erzwungen statt empfohlen (P26), Ports werden `dyn`-kompatibel
konstruiert statt nativem AFIT (P27), Nichtdeterminismus-Quellen werden injiziert statt implizit genutzt (P28),
globaler veränderlicher Zustand ist verboten (P29), jeder Crate-Zuschnitt braucht ein explizites Kriterium
I/U/C/S/D (P30).

### A2.4 Offene, noch nicht entschiedene Punkte (bewusst nicht in dieser Fassung präjudiziert)

Diese Punkte sind **keine** stillschweigenden Annahmen, sondern explizit offene Entscheidungen, die vor dem
jeweiligen Migrationsschritt (§20) einen Product-Owner-Beschluss oder einen Spike mit Exit-Kriterium brauchen:

1. **ADR-N06 (Konsistenzmodell):** 2PC mit Intent-Keys härten (Option A) oder auf WAL-als-einzige-Wahrheit mit
   abgeleitetem, per `applied_lsn` nachholendem Zustand umstellen (Option B, empfohlen, §4.3 der Quelle). Braucht
   ein vom Product Owner genanntes Recovery-Zeit-Ziel, bevor der Spike (Phase 3a) freigegeben wird.
2. **KV-Ambition:** nur Stufe A (RAM, Upstream-`ModelWeights::clone()`, kein Fork) oder A+B+C (eigenes
   Llama-Modell mit `KvState`, optional Platten-Spill)? Stufe A liefert bereits Nutzen ohne Fork-Risiko;
   B/C werden erst nach Messung an Stufe A entschieden (§9.2).
3. **`DocId`-Migration (ADR-N05):** Umstieg auf externes 128-Bit-`DocId` mit internem dichten `DocIdx(u32)`
   ändert das Persistenzformat (WAL v2, v1 bleibt lesbar). Akzeptanz dieses Formatbruchs ist offen, gekoppelt
   an die Entscheidung zu ADR-N06.
4. **Crate-Anzahl:** 27 Fach- + 3 Tooling-Crates werden vorläufig akzeptiert. Zeigt `cargo build --timings`
   nach Phase 3 keinen Parallelitätsgewinn, ist eine Konsolidierung (`adapt`→`rank`, `mvcc`→`types`) offen.
5. **Nonce-Strategie (§9.3, ab Fassung 2.1):** Status quo (4-Byte-Präfix je Schlüssel + 8 Byte `OsRng`,
   Nachrichtenbudget je Schlüssel) oder persistierter Epochenzähler ‖ Zähler. Ein reiner `AtomicU64`-Zähler im
   RAM ist nicht zulässig (beginnt nach Neustart bei 0).
6. **Clique-Konvention der Stern-Expansion (§6.6 H6, ab Fassung 2.1):** Konvention K (Gesamtmasse je Hyperkante
   = w(e), N=2 ≙ binäre Kante) ist als Arbeitsstand festgelegt; ob die Größenabhängigkeit der Masse
   (Alternative: Zhou-Konvention p = w/(|e|−1)) fachlich gewünscht ist, entscheidet der Product Owner nach
   Benchmark.

Diese sechs Punkte dürfen **nicht** durch Weiterarbeit am Code stillschweigend entschieden werden; jede
Umsetzung, die eine dieser Fragen präjudiziert, braucht vorab die zugehörige ADR (§20.3) im Status
„beschlossen".

---

<a id="a3-status"></a>
