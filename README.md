# MemFuse Cognitive OS — Finale Konsolidierte Gesamtspezifikation (Zielarchitektur-v2-Edition)

> **Status:** Normativ · Einzige maßgebliche Quelle für Produkt, Architektur, Algorithmen,
> Implementierungsvorgaben, Sicherheitsmodell, Schnittstellenspezifikation, Stabilisierungsphasen und
> Optimierungs-Roadmap des MemFuse Cognitive OS.
>
> **Charakter:** Dieses Dokument führt Produktvision, Zielarchitektur, normative Signaturen,
> algorithmische Spezifikationen, mikrofeingranulare Schnittstellendefinitionen, ein verbindliches
> Stabilisierungs- und Gate-Modell sowie die priorisierte Optimierungs-Roadmap in einem einzigen,
> in sich geschlossenen Dokument zusammen. Es ersetzt vollständig alle vorherigen
> Spezifikationsfassungen, -deltas und separat geführten Stabilisierungspläne. **Es referenziert keine
> externen Dokumente** — jede Aussage, die für die Arbeit an MemFuse nötig ist, steht hier.
>
> **Leitentscheidung dieser Fassung (2):** Gegenüber der Vorfassung ist dieses Dokument um **Teil A —
> Stabilisierungsauftrag** ergänzt und in seiner Roadmap (§17, §18) komplett neu geordnet. Grund:
> eine unabhängige Prüfung des Repository-Zustands hat gezeigt, dass Diagnose-Artefakte (Testergebnisse,
> Lint-Reports, Audit-Dokumente) systematisch vom tatsächlichen Code-Zustand abweichen können, sobald sie
> nicht mechanisch an einen Commit gebunden und automatisch neu erzeugt werden — mit der Folge, dass an
> bereits gelöster Stelle weitergearbeitet und an tatsächlich offener Stelle vorbeigearbeitet wird. **Ab
> sofort gilt: kein Feature-Ausbau, bevor Ground Truth hergestellt und das Fundament (Ring 0–1 des
> Crate-Graphen, vormals „Layer 0–2") nachweisbar stabil ist.** Teil A ist ranghöher als alle übrigen Teile
> dieses Dokuments; im Konfliktfall gilt Teil A.
>
> **Leitentscheidung dieser Fassung (3) — Zielarchitektur v2, ab sofort verbindlich:** Zusätzlich zu Teil A
> ergänzt dieses Dokument **Teil A2 — Zielarchitektur v2 (Ring-Modell)** direkt im Anschluss an Teil A. Die
> Prüfung `MEMFUSE_ZIELARCHITEKTUR.md` (v1, Basis-Commit `3f37ab30`) und ihre Korrektur
> `MEMFUSE_ZIELARCHITEKTUR_v2.md` (Basis-Commit `806a40c1`) haben strukturelle Verstöße gegen P5 (Crate
> `memfuse-db` hängt aufwärts von `memfuse-candle`/`memfuse-ollama`/`memfuse-embed` ab), eine unvollständige
> Unsafe-Inventur, eine nicht funktionsfähige Lint-Mechanik (`forbid` plus Insel-`allow` verletzt Rust-Semantik,
> E0453) sowie eine als 🟢 markierte, tatsächlich als Stub implementierte KV-Cache-Bridge belegt. **Ab sofort
> gilt: Der bisherige Layer-0–5-Crate-DAG aus §4 (10 Crates) wird durch das in Teil A2/§4 beschriebene
> Ring-0–4-Modell (27 Fach-Crates + 3 Tooling-Crates) ersetzt; neuer Code wird ausschließlich gegen das
> Ring-Modell geschrieben.** Die alte Schichtdarstellung bleibt in §4 als „Vorher"-Referenz erhalten, damit
> bestehender Code und bestehende Diskussionen zuordenbar bleiben — sie ist ab dieser Fassung **nicht mehr
> normativ**. Teil A2 ist ranghöher als §3–§9 dieses Dokuments, soweit sie Crate-Zuschnitt, Abhängigkeitsrichtung,
> Unsafe-Politik, Panic-Politik oder KV-Cache-Architektur betreffen; im Konfliktfall gilt Teil A2 vor Teil A
> nachrangig, d. h. Ground-Truth-Herstellung (Teil A) hat Vorrang vor Struktur-Migration (Teil A2), aber beide
> stehen über allen übrigen Abschnitten.
>
> **Leitentscheidung dieser Fassung (4) — Fassung 2.1 (Korrekturfassung):** Fassung 2.1 behebt Fehler der
> Fassung 2 (fehlender Teil A, Discount-Richtung des Bandits, Beweis der Stern-Expansion, instabiler
> Shard-Hasher in H2, nicht kompilierbare Skizzen, unsafe-pflichtige Entwürfe im Safe-Crate `memfuse-store`),
> ergänzt die Systeminvarianten §4(1)–§4(7) als Anker und übernimmt sechs Punkte aus der Prüfung der
> „Mikrofeingranularen Schnittstellenspezifikation" (Payload-Sharing, capacity-basiertes Speicherbudget,
> normierte Stern-Expansion, persistente idempotente Cascade-Queue, gemessene SQ8-Bias-Korrektur, begrenzte
> WAL-Queue). Der bisher zwischen §19 und §20 eingebettete Block ist als **Anhang B** (nachrangig) ans
> Dokumentende verschoben. Änderungsprotokoll und Prüfnachweise: **Anhang C**.
>
> **Sprache:** Rust 2021, Workspace-Layout, `#![forbid(unsafe_code)]` als Default in jedem Nicht-Insel-Crate
> (drei Unsafe-Inseln, §0.4 — vormals sechs Ausnahme-Crates).
>
> **Lesart:** Jeder Abschnitt ist eigenständig implementierbar. Codeblöcke sind **normativ**, nicht
> illustrativ — Feldnamen, Typnamen und Funktionssignaturen sind exakt zu übernehmen, sofern nicht
> als „Beispiel" markiert. Wo `unimplemented!()` steht, ist die Signatur und das umgebende
> Vertrags-/Fehlerverhalten normativ, der Funktionskörper ist gemäß der in Prosa/Formel gegebenen
> Algorithmusbeschreibung des jeweiligen Abschnitts zu füllen. **Jeder Abschnitt außerhalb von Teil A
> trägt zusätzlich eine Phasen-Kennzeichnung** (`[Phase 1]` … `[Phase 5]`, siehe §A.3) — sie sagt, ab
> welchem Stabilisierungs-Gate an diesem Abschnitt gearbeitet werden darf. Ein Abschnitt ohne
> ausdrückliche Phasen-Kennzeichnung gilt als `[Phase 1]` (Fundament).
>
> **Reifegrad-Kennzeichnung, durchgängig verwendet — mit verschärfter Bedeutung, siehe §A.2:**
> - 🟢 **Produktiv (Zielaussage)** — im Code vorhanden, korrekt und als Produktions-Default aktiv,
>   **sofern durch einen frischen, commit-gebundenen CI-Lauf bestätigt** (§A.2). Unbestätigt ist die
>   Markierung eine Behauptung aus einer Vorversion dieses Dokuments, keine verifizierte Tatsache.
> - 🟡 **Hinter Feature-Flag** — im Code vollständig und korrekt vorhanden, aber nicht der
>   Produktions-Default; Aktivierung erfordert ein explizites Cargo-Feature.
> - 🔴 **Spezifiziert, zu bauen** — normativer Zielzustand dieses Dokuments, im Code noch nicht
>   vorhanden.
> - ⚖️ **Produktentscheidung ausstehend** — technisch möglich oder vorhanden, Default-Wechsel an
>   messbares Kriterium gebunden.
> - ⚠️ **Opus-Optimierung** — aus Architektur-Review identifiziert, priorisiert umzusetzen, mit
>   Stufe (0–3) und Aufwandseinschätzung versehen.
> - 🔍 **Nachverifikation ausstehend** (neu) — Reifegrad aus einer früheren Dokumentfassung
>   übernommen, aber noch nicht gegen einen frischen CI-Lauf am aktuellen HEAD bestätigt. Jeder
>   Contributor, der auf einen 🟢/🔴-Marker reagiert, MUSS ihn faktisch als 🔍 behandeln, bis Gate 0
>   (§A.3) für den betroffenen Crate durchlaufen ist.
>
> **Korrektur durch Teil A2, verbindlich:** Der bisherige Marker 🟢 für die KV-Cache-Bridge (§9.2) war
> **falsch** — die verifizierte Implementierung speichert nur einen Platzhalter-String und zählt einen
> Zähler hoch, ohne echten Prefill einzusparen (§9.2, „Vorher/Jetzt"). Ab dieser Fassung gilt jeder 🟢-Marker
> zusätzlich als widerrufen, sobald Teil A2 für den betroffenen Bereich einen belegten Gegenbefund („D#"
> in Teil A2 §2) nennt; maßgeblich ist die D#-Tabelle in §A2.1.

---

## Inhaltsverzeichnis

**A.** [Stabilisierungsauftrag: Ground Truth, Reifegrade, Gates](#teil-a)
**A2.** [Zielarchitektur v2 — Ring-Modell, verbindlich ab sofort](#a2-zielarchitektur-v2)
0. [Meta: Workspace-Layout und Build-Konfiguration](#0-meta)
1. [Kernthese und Leitprinzip](#1-kernthese)
2. [Produktvision, Alleinstellungsmerkmale und Nicht-Ziele](#2-vision)
3. [Architekturprinzipien P1–P30](#3-prinzipien)
4. [Systemarchitektur: der Crate-Graph (Ring-Modell, vormals Crate-DAG)](#4-architektur)
5. [Speicherschicht: LSM-Tree, WAL und Block-Cache](#5-speicher)
6. [Wissensgraph-Datenmodell: binäre Kanten und n-äre Hyperkanten](#6-graph)
7. [Retrieval-Pipeline: 4-Signal-Fusion und ihre Algorithmen](#7-retrieval)
8. [Contextual-Bandit-Routing](#8-bandit)
9. [Inferenz, KV-Cache v2 und Zero-Copy-IPC](#9-inferenz)
10. [Sicherheits- und Datenschutzmodell](#10-sicherheit)
11. [Betriebsmodi](#11-betrieb)
12. [FlatBuffers-Schema (vollständig)](#12-schema)
13. [Fehlertaxonomie (crateübergreifend)](#13-fehler)
14. [Feature-Flag-Politik: Produktions-Default vs. Opt-in](#14-features)
15. [Test- und CI-Spezifikation](#15-tests)
16. [Vollständige Abnahmekriterien](#16-abnahme)
17. [Priorisierte Optimierungs-Roadmap (Opus-Analyse)](#17-optimierungen)
18. [Gesamtroadmap](#18-roadmap)
19. [Rückverfolgbarkeitsmatrix](#19-matrix)
20. [Migrationsplan v2 und ADR-Übersicht (neu)](#20-migration-v2)

**B.** [Anhang B — Begründungen, Ist-Zustand, Literatur (nachrangig)](#anhang-b)
**C.** [Anhang C — Änderungsprotokoll Fassung 2.1 und Prüfnachweise](#anhang-c)

---

<a id="teil-a"></a>
## Teil A — Stabilisierungsauftrag: Ground Truth, Reifegrade, Gates

> **Rekonstruktionsvermerk (Fassung 2.1):** Teil A ist im Kopf dieses Dokuments und in Teil A2 als ranghöchster
> Teil (§A.1–§A.5) referenziert, stand in Fassung 2 aber nicht im Dokument. Dieser Teil ist aus den Aussagen
> rekonstruiert, die das Dokument selbst über Teil A macht (Kopf, §A2, §16, §17, §18, §20). Wo diese Aussagen
> nichts festlegen, ist die Festlegung mit **[F2.1]** gekennzeichnet und vom Product Owner zu bestätigen. Liegt
> die ursprüngliche Fassung vor, ersetzt sie diesen Teil vollständig.

### A.1 Rang und Ground-Truth-Regel

Teil A ist ranghöher als alle übrigen Teile (Ausnahme-Reihenfolge laut Kopf: Ground-Truth-Herstellung vor
Struktur-Migration vor allen übrigen Abschnitten). **Ground Truth** ist der Zustand, in dem für einen Crate
gilt: ein frischer, an genau einen Commit gebundener CI-Lauf liegt vor, und alle Reifegrad-Marker des Crates
wurden aus diesem Lauf erzeugt. **Kein Feature-Ausbau, bevor Ground Truth hergestellt und das Fundament
(Ring 0–1) nachweisbar stabil ist.**

### A.2 Reifegrad-Marker (verschärfte Bedeutung)

- 🟢 gilt nur, wenn ein frischer, commit-gebundener CI-Lauf am aktuellen HEAD den Sachverhalt bestätigt.
  Andernfalls ist die Markierung eine Behauptung aus einer Vorfassung und wird wie 🔍 behandelt.
- 🔍 (Nachverifikation ausstehend) ist der Zustand jedes aus einer Vorfassung übernommenen Markers, bis Gate 0
  für den betroffenen Crate durchlaufen ist. Contributor MÜSSEN 🟢/🔴 bis dahin faktisch als 🔍 behandeln.
- Ein 🟢 wird widerrufen, sobald Teil A2 einen belegten Gegenbefund (D#, §A2.1) nennt oder ein CI-Lauf ihn
  widerlegt. **[F2.1]** Marker werden maschinell aus `capabilities.toml` und den CI-Ergebnissen erzeugt und
  nicht von Hand gepflegt (P12, §20).

### A.3 Phasen und Gates **[F2.1]**

Jeder Abschnitt außerhalb von Teil A trägt implizit `[Phase 1]`, sofern er keine andere Kennzeichnung führt.
Die Phasen entsprechen den Stufen der Roadmap (§17, §18): Phase *n* ↔ Stufe *n − 1*.

| Phase | Inhalt | Eintrittsbedingung |
|---|---|---|
| 1 | Fundament (Ring 0–1): Korrektheit und Betriebssicherheit (Stufe 0) | Gate 0 für den betroffenen Crate |
| 2 | Hot-Path-Performance (Stufe 1) | Gate 1 |
| 3 | Speicher und Struktur (Stufe 2) | Gate 2 |
| 4 | Governance und Produktentscheidungen (Stufe 3) | Gate 3 |
| 5 | Fernziele (§18, Stufe 4) | Gate 4 |

- **Gate 0 (je Crate):** frischer, commit-gebundener Lauf von `cargo fmt --check`,
  `cargo clippy --workspace --all-targets --locked -- -D warnings` und `cargo test -p <crate> --locked`, grün am
  aktuellen HEAD; Marker des Crates neu erzeugt (§A.2).
- **Gate 1 (Fundament stabil):** Gate 0 für alle Ring-0/1-Crates; Kernkriterien K-* aus §16.1, soweit Stufe 0;
  Loom-Tests aus §15.3 grün; Layering-Test (§4.3) scharf.
- **Gate 2 bis 4:** alle in der jeweils vorangehenden Phase genannten Abnahmekriterien (§16) grün, am selben
  Commit wie der Gate-Nachweis.

### A.4 Diagnose-Artefakte und Commit-Bindung

Testergebnisse, Lint-Reports, Audit-Dokumente und Marker-Tabellen tragen im Kopf `commit: <SHA>`,
`generated_by: <Befehl>`, `generated_at: <UTC>` und `toolchain: <rustc-Version>`. Ein Artefakt mit
`commit ≠ HEAD` ist **veraltet** und darf nicht als Begründung für Arbeit oder Nicht-Arbeit dienen. Artefakte
werden mechanisch (xtask/CI, 🔴 zu bauen) erzeugt, nie von Hand editiert.

### A.5 Änderungsdisziplin

Arbeit an einem Abschnitt der Phase *n* beginnt erst nach Gate *n − 1*. Ausgenommen sind Korrekturen mit
Stufe-0-Charakter (Panic, Datenverlust, Sicherheitslücke) und reine Dokumentationsänderungen. Beschlossene
ADRs (§20.3) und Teil-A2-Migrationsschritte (§20.2) gelten als Phase-1-Arbeit, soweit sie Ring 0–1 betreffen.

---

<a id="a2-zielarchitektur-v2"></a>
## Teil A2 — Zielarchitektur v2 (Ring-Modell), verbindlich ab sofort

> Rang: ranghöher als §3–§9, nachrangig zu Teil A (§A.1–§A.5). Quelle: `MEMFUSE_ZIELARCHITEKTUR.md` (v1,
> Basis `3f37ab30`) und ihre Korrekturfassung `MEMFUSE_ZIELARCHITEKTUR_v2.md` (v2, Basis `806a40c1`, ersetzt v1
> vollständig). Kennzeichnung wie in der Quelle übernommen: **[V]** am Repo maschinell geprüft, **[D]** aus
> Quelltext/Cargo-/rustc-Semantik abgeleitet, nicht ausgeführt, **[U]** Urteil.

### A2.0 Warum diese Fassung existiert — vorher/jetzt in einem Satz

**Vorher (Stand dieser Spec bis zur Vorfassung):** ein Layer-0–5-Crate-DAG mit 10 benannten Crates (§4 alt),
`memfuse-db` als monolithische Fassade mit über 40 öffentlichen Methoden, KV-Cache-Bridge als 🟢 markiert,
`#![forbid(unsafe_code)]` als Default mit sechs pauschalen Ausnahme-Crates, Reifegrad-Marker handgepflegt.
**Jetzt (diese Fassung, ab sofort im Code umzusetzen):** ein Ring-0–4-Crate-Graph mit 27 Fach- plus 3
Tooling-Crates, `memfuse-db` wird schrittweise in `engine`/`cognition`/`rank`/`adapt`/`router`/`privacy`
zerlegt, KV-Cache auf 🔴 zurückgestuft und in drei Ausbaustufen A/B/C neu spezifiziert, genau drei benannte
Unsafe-Inseln mit `deny`+`forbid`-Mechanik statt pauschalem `forbid`+Insel-`allow` (letzteres ist wegen E0453
gar nicht baubar), Reifegrad-Marker sollen aus `capabilities.toml` generiert werden. Der Übergang erfolgt
strangler-artig (§20), nicht per Big-Bang-Rewrite.

### A2.1 Warum v1 nicht unverändert übernommen wird

v1 (`MEMFUSE_ZIELARCHITEKTUR.md`) war architektonisch richtig ausgerichtet (Ports/Adapter, Sync-Kern,
gestufter KV-Ausbau, generierte Spec), enthielt aber neun am Code belegte Defekte, die den direkten Weg in
diese Spec verboten hätten. Maßgeblich ist ausschließlich v2; v1 wird hier nur referenziert, wo sie den
historischen Ausgangspunkt einer Entscheidung erklärt.

| # | Defekt in v1 | Beleg (v2) | Konsequenz in dieser Spec |
|---|---|---|---|
| D1 | Unsafe-Inventar unvollständig (v1: nur `simd` + mmap-Wrapper) | **[V]** Win32-ACL in `wal/io.rs` (13 unsafe), mmap in `wal/replay.rs`/`index/persistence.rs`, `mlock` in `db/volatile_vault.rs` (4), SIMD in `distance.rs` (73), Generat in `core-ipc-gen` (44) | Dritte Insel `memfuse-sys` (§0.4 neu) |
| D2 | Lint-Mechanik `forbid` + Insel-`allow` nicht baubar | **[D]** `forbid` ist per Rust-Semantik nicht lokal überschreibbar (E0453); Cargo verbietet `[lints] workspace = true` neben eigenen `[lints.*]`-Tabellen | Workspace-`deny` + `#![forbid]` pro Nicht-Insel-Crate (§0.4 neu) |
| D3 | `memfuse-checkpoint` fälschlich in `memfuse-store` verschmolzen gedacht | **[V]** `checkpoint` hängt nur an `core`/`StorageEngine`, nicht an `store`; eigener globaler Zustand (`ORPHAN_REGISTRY`) muss entfernt werden | `checkpoint` bleibt eigenes Ring-1-Crate, ohne globalen Zustand |
| D4 | `memfuse-core`-Zerlegung im Migrationsplan unvollständig | **[V]** `core` = 10.353 LOC; `tx_buffer`, `seq_log`, `snapshot` u. a. waren v1 nicht zugeordnet | Neues Ring-0-Crate `memfuse-mvcc` |
| D5 | Kennzahlen (async fn, Panic-Stellen) enthielten Testcode, waren 3–20× zu hoch | **[V]** echte Prod-Zahlen deutlich kleiner (§A2 unten, Migrationsplan) | Aufwandsschätzung Phase 2/Panic-Politik nach unten korrigiert |
| D6 | `indexing_slicing = deny` workspace-weit geplant | **[V]** ≈ 640 Index-/Slice-Ausdrücke in Hot-Paths | `deny` nur an Parsing-Grenzen, sonst `warn` + `debug_assert!` |
| D7 | Plan sah `deny.toml` „neu anlegen" vor | **[V]** `deny.toml` existiert bereits seit `81edd9af` (Lizenzen, Quellen, Bans) | Erweitern statt neu anlegen |
| D8 | Prüfbefehl `cargo check --workspace` sollte Netzfreiheit belegen | **[D]** `--workspace` ignoriert `default-members`; Netzfreiheit kommt von der `onnx-bench`-Feature-Trennung | Exit-Kriterien in §20 korrigiert |
| D9 | KV-Optionen unvollständig (Toleranz, Speicherebene, dtype ungeklärt) | **[D]** `ModelWeights` ist `Clone`; naives Spillen großer KV-Blöcke in eine LSM-Engine ist ein Fehlgriff (Write-Amplifikation) | KV in drei Stufen A/B/C, siehe §9 |

### A2.2 Entfallende, neue und unveränderte Crates gegenüber der Vorfassung

**Entfallen als eigenständige Crates** (gehen in die Ring-Struktur auf, Re-Export mit `#[deprecated]` während
der Strangler-Phase, §20): `memfuse-core`, `memfuse-core-ipc-gen`, `memfuse-checkpoint` *(bleibt de facto
erhalten, siehe D3 — nur die v1-Fusionsidee entfällt)*, `memfuse-calibration`, `memfuse-router` *(bleibt
erhalten, wird nur schlanker)*, `memfuse-embed`, `memfuse-candle`, `memfuse-ollama`, `memfuse-db`.

**Neu:** `memfuse-types`, `memfuse-ports`, `memfuse-mvcc`, `memfuse-wire` (vormals `core-ipc-gen`),
`memfuse-sys`, `memfuse-simd` (vormals Teil von `memfuse-index`), `memfuse-vector` (vormals `memfuse-index`),
`memfuse-rank` (vormals Teil von `memfuse-db` + `memfuse-calibration`), `memfuse-adapt` (vormals Teil von
`memfuse-router` + `memfuse-db`), `memfuse-kvcache` (vormals Teil von `memfuse-crypto` + `memfuse-candle`),
`memfuse-infer-candle`/`-ollama`/`-onnx` (vormals `memfuse-candle`/`-ollama`/`-embed`), `memfuse-engine`,
`memfuse-cognition`, `memfuse-privacy` (vormals Teile von `memfuse-crypto` + `memfuse-mcp`), `memfuse`
(neue, einzige Fassade/Composition Root).

**Unverändert im Zuschnitt, nur ggf. intern angepasst:** `memfuse-store`, `memfuse-crypto` (schlanker, ohne
Egress-Vault/KV-Segment, die nach `privacy`/`kvcache` wandern), `memfuse-text`, `memfuse-graph`,
`memfuse-sandbox`, `memfuse-agent`, `memfuse-mcp`, `memfuse-py`, `memfuse-bench`, `xtask`.

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

<a id="0-meta"></a>
## 0. Meta: Workspace-Layout und Build-Konfiguration

### 0.1 Verzeichnisstruktur

**Tatsächlicher Workspace-Bestand (IST-Stand laut `ls crates/`, 27 Fach-/Modul-Crates):**

> **Hinweis zur Migration:** IST-Stand weicht stellenweise vom normativen SOLL-Zielzustand (§A2/§4.2) ab. Wo Namen oder Ring-Zuweisungen im Code-Bestand noch dem Übergangsstadium entsprechen (z. B. `memfuse-db` als Monolith statt getrennter `engine`/`cognition`/`privacy`, `memfuse-index` statt `memfuse-vector`, `memfuse-candle` statt `memfuse-infer-candle`), ist dies explizit als „In Migration" bzw. „Strangler / Legacy" gekennzeichnet. Für Details zur Ring-Zuordnung und Migrationsreihenfolge siehe `ARCHITECTURE.md` (folgt in Kürze) und §20.

```
memfuse/
├── Cargo.toml                      # [workspace], resolver = "2"
├── xtask/                          # CI-Tooling (Drift-Gates, Layering, Benchmarks)
├── schemas/
│   └── memfuse.fbs                 # FlatBuffers IPC-Schema (§12)
├── crates/
│   ├── memfuse/                    # Ring 4 — Shell / Facade & Composition Root (In Migration)
│   ├── memfuse-adapt/              # Ring 0 — Adaptive Control & Bandit (Fertig)
│   ├── memfuse-agent/              # Ring 3 — Workflow Engine, Audit & DLQ (Fertig)
│   ├── memfuse-calibration/        # Ring 0 — Legacy Score Calibration (Strangler / In Migration nach adapt/rank)
│   ├── memfuse-candle/             # Ring 2 — Inferenz-Candle & GGUF (In Migration nach infer-candle)
│   ├── memfuse-checkpoint/         # Ring 1 — Snapshot & Time-Travel Checkpoint Registry (Fertig)
│   ├── memfuse-core/               # Ring 0 — Legacy Facade & Re-exports (Strangler / In Migration)
│   ├── memfuse-crypto/             # Ring 0 — Crypto Kernel, AEAD & Deletion Proofs (Fertig)
│   ├── memfuse-db/                 # Ring 3 — Database Orchestrator & Collection Monolith (In Migration)
│   ├── memfuse-embed/              # Ring 2 — Inferenz-ONNX Embeddings & Reranker (In Migration nach infer-onnx)
│   ├── memfuse-graph/              # Ring 0 — CSR Graph, Forward-Push PPR & Hyperedges (Fertig)
│   ├── memfuse-index/              # Ring 0 — HNSW, DiskANN & Quantisierung (In Migration / Vektor-Index)
│   ├── memfuse-kvcache/            # Ring 1 — Tenant Isolated KV-Cache & Segment Storage (Fertig)
│   ├── memfuse-mcp/                # Ring 4 — MCP Server, Stdio Protocol & Egress Guard (Fertig)
│   ├── memfuse-mvcc/               # Ring 0 — SeqLog, SnapshotRegistry & TxBuffer (Fertig)
│   ├── memfuse-ollama/             # Ring 2 — Ollama Inferenz Client & Context Prefixer (Fertig)
│   ├── memfuse-ports/              # Ring 0 — Core Abstract Traits & Interfaces (Fertig)
│   ├── memfuse-py/                 # Ring 4 — PyO3 Python Bindings (Fertig)
│   ├── memfuse-router/             # Ring 3 — SLM Profile Router & Dispatcher (Fertig)
│   ├── memfuse-sandbox/            # Ring 2 — WASM Execution Boundary (Fertig)
│   ├── memfuse-simd/               # Ring 0 — Unsafe Island: SIMD Distance Kernels (Fertig)
│   ├── memfuse-store/              # Ring 1 — LSM Storage Engine, WAL & KvKeyLocks (Fertig)
│   ├── memfuse-sys/                # Ring 0 — Unsafe Island: OS System Call Wrappers (Fertig)
│   ├── memfuse-testkit/            # Tooling — FaultVfs, ManualClock & In-Memory Store (Fertig)
│   ├── memfuse-text/               # Ring 0 — BM25/BM25F Search & German Morphology (Fertig)
│   ├── memfuse-types/              # Ring 0 — Canonical Domain Types & Error Enums (Fertig)
│   └── memfuse-wire/               # Ring 0 — Unsafe Island: FlatBuffers IPC Generat & Adapters (Fertig)
├── benchmarks/
│   └── memfuse-bench/              # Benchmark Harness
├── .github/workflows/
│   └── merge-gate.yml              # CI Merge Gate
└── docs/decisions/                 # ADR-0NN-*.md
```

**Vorher (Fassung bis zur Vorgängerversion dieser Spec, Layer-0–5-Modell — nicht mehr normativ, siehe §4 „Vorher"):**

```
memfuse/
├── crates/
│   ├── memfuse-core-ipc-gen/       # Layer 0 — FlatBuffers-generierter Code
│   ├── memfuse-core/               # Layer 0 — Kerntypen, Traits, Fehlerbehandlung
│   ├── memfuse-store/              # Layer 1 — LSM-Tree, WAL, Block-Cache
│   ├── memfuse-crypto/             # Layer 1 — AES-256-GCM-SIV, DeletionProof, KV-Segment-Security
│   ├── memfuse-text/               # Layer 1 — BM25/BM25F-Volltextindex, deutsche Morphologie
│   ├── memfuse-index/              # Layer 1 — HNSW/DiskANN-Vektorindex, SIMD-Distanz
│   ├── memfuse-graph/              # Layer 1 — CSR-Graph, PPR, Leiden, Hyperkanten
│   ├── memfuse-checkpoint/         # Layer 1 — Snapshotting
│   ├── memfuse-calibration/        # Layer 1 — Score-Kalibrierung, Drift-Erkennung
│   ├── memfuse-db/                 # Layer 2 — Collection-API, 4-Signal-Fusion, Provenance
│   ├── memfuse-router/             # Layer 3 — Contextual-Bandit-Routing
│   ├── memfuse-candle/             # Layer 3 — Natives GGUF-Inferenz-Backend, KV-Cache-Bridge
│   ├── memfuse-ollama/             # Layer 3 — Ollama-Client, Contextual-Chunk-Prefixing
│   ├── memfuse-embed/              # Layer 3 — ONNX-Embeddings, Cross-Encoder (optional)
│   ├── memfuse-agent/              # Layer 3 — Persistente Agent-Workflow-Engine
│   ├── memfuse-py/                 # Layer 3 — Python-FFI via PyO3 (eigener Workspace, war real nur Member)
│   ├── memfuse-sandbox/            # Layer 6.5 — WASM Execution Boundary
│   ├── memfuse-mcp/                # Layer 4 — MCP-Server, Egress-Gateway
│   └── memfuse-bench/              # Layer 5 — Benchmark-Harness
```

Grund der Ablösung, Zuordnung alt→neu je Crate und Migrationsreihenfolge: §A2.2, §4, §20.

### 0.2 Root-`Cargo.toml` (normativ)

**Jetzt:** Werte werden **aus dem realen Manifest generiert** (P12/§A2.3), nicht mehr handgepflegt — Grund:
die Vorfassung enthielt vier unbelegte Werte (`rust-version`, `license`, `flatbuffers`, `thiserror`), die am
Code widerlegt sind (§A2.1, D8-Nachbarbefund). Bis der Generator (§20, Phase 5) steht, gelten die verifizierten
Ist-Werte als normativ, ergänzt um die neu beschlossenen Workspace-Lints und das `release-abort`-Profil:

```toml
[workspace]
resolver = "2"
members = ["crates/*"]
default-members = [ "crates/*" ]     # ohne memfuse-infer-onnx / --features onnx-bench (§A2.1 D8)
exclude = ["xtask"]

[workspace.package]
edition = "2021"
rust-version = "1.89"                # vorher fälschlich als 1.79 spezifiziert
license = "MIT OR Apache-2.0"        # vorher fälschlich als "Apache-2.0" spezifiziert

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
flatbuffers = "24.3"                 # vorher fälschlich als "23" spezifiziert
crossbeam-epoch = "0.9"
arc-swap = "1"
ahash = "0.8"
scc = "2"
quick_cache = "0.5"
zerocopy = "0.7"
thiserror = "2"                      # vorher fälschlich als "1" spezifiziert
tokio = { version = "1", features = ["rt-multi-thread", "sync", "time", "macros"] }
aes-gcm-siv = "0.11"
blake3 = "1"
wasmtime = "25"

# Neu ab dieser Fassung (§A2, §4.2 der Zielarchitektur-Quelle):
[workspace.lints.rust]
unsafe_code = "deny"                 # bewusst "deny", nicht "forbid" — s. Begründung unten
unsafe_op_in_unsafe_fn = "deny"

[workspace.lints.clippy]
undocumented_unsafe_blocks = "deny"
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
todo = "deny"
unimplemented = "deny"

[profile.release]
panic = "unwind"                     # Root bleibt unwind — vorher "abort" (widersprach memfuse-py, §A2.1)

[profile.release-abort]
inherits = "release"
panic = "abort"                      # nur für Binaries ohne FFI, per Paket ausgewählt
```

**Warum `deny` statt `forbid` für `unsafe_code` (verbindliche Korrektur, §A2.1 D2):** `forbid` ist in Rust
nicht lokal überschreibbar (Compiler-Fehler E0453); ein Versuch, `#![forbid(unsafe_code)]` workspace-weit zu
setzen und in den drei Unsafe-Inseln (§0.4) per `#[allow(unsafe_code)]` zu durchbrechen, baut nicht. Stattdessen
gilt: Workspace-Lint ist `deny`; jeder Nicht-Insel-Crate setzt zusätzlich in seiner `lib.rs` explizit
`#![forbid(unsafe_code)]` (das ist zulässig, weil die Workspace-Stufe nur `deny` ist); jede Insel setzt
`#![allow(unsafe_code)]`. Ein Inventar-Test erzwingt, dass kein weiterer Crate `allow(unsafe_code)` trägt
(§0.4, `tests/unsafe_islands.rs`).

**Vorher (nicht mehr normativ):**

```toml
[workspace]
resolver = "2"
members = ["crates/*", "xtask"]

[workspace.package]
edition = "2021"
rust-version = "1.79"
license = "Apache-2.0"

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
flatbuffers = "23"
thiserror = "1"
wasmtime = "23"
# ... (gekürzt, siehe Git-Historie dieses Dokuments)
```

`xtask` ist ab dieser Fassung kein Workspace-Member mehr (`exclude`), sondern ein eigenständiges Cargo-Projekt,
damit CI-Tooling-Abhängigkeiten nicht in `cargo tree --workspace` erscheinen (Grundlage für den
`cargo tree`-basierten Netzfreiheits- und Bans-Test, §20 Phase 0R).

### 0.3 Cargo-Feature-Katalog (crateübergreifend normativ)

**Politik ab dieser Fassung (P5/§A2.3):** Additive Features nur noch für **schwere optionale Abhängigkeiten**
(onnx, cuda, metal, wasmtime). **Keine typverändernden Features mehr** — `docid-128` entfällt ersatzlos
(ADR-N05, §20.3); Verhalten wird über Laufzeitkonfiguration statt Compile-Time-Flags gesteuert, wo immer das
ohne Typenwechsel möglich ist. Der Katalog wird perspektivisch aus den Crate-Manifesten generiert (P12); bis
dahin gilt die folgende, an den Ring-Zuschnitt angepasste Tabelle als normativ:

| Feature | Definierender Crate (jetzt) | Definierender Crate (vorher) | Default | Wirkung |
|---|---|---|---|---|
| ~~`docid-128`~~ | — (entfällt) | `memfuse-core` | — | **Entfällt** (ADR-N05): typverändernde Features sind verboten; externes 128-Bit-`DocId` mit internem dichten `DocIdx(u32)` ist eine offene Entscheidung (§A2.4 Nr. 3), keine Compile-Time-Option |
| `block-cache-v2` | `memfuse-store` | `memfuse-store` | aus | `QuickCacheBlockCacheBackend` statt `LruBlockCacheBackend` (§5.4) |
| `egress-sherman-morrison` | `memfuse-adapt` | `memfuse-router` | aus | `ShermanMorrisonBandit` statt `DiagonalApproximation` (§8.2) |
| `experimental-diskann` | `memfuse-vector` | `memfuse-index` | aus | `DiskAnnIndex` über `VectorIndexTier::DiskAnn` wählbar (§7.5) |
| `bandit-routing` | `memfuse-adapt` | `memfuse-router` | an | Aktiviert den Bandit-Router überhaupt |
| `cloud-egress-guard` | `memfuse-privacy` | `memfuse-mcp` | an | Aktiviert `egress_gateway`-Modul (im Manifest der Vorfassung real nicht vorhanden, wird mit dem Umzug nach `privacy` nachgezogen) |
| `wasm-sandbox` | `memfuse-sandbox` | `memfuse-mcp` | an | Aktiviert die Sandbox; `memfuse-sandbox` ist bis zur Anbindung an einen Konsumenten aus `default-members` ausgeschlossen (§A2.2, Waisen-Crate) |
| `kv-bridge` | `memfuse-kvcache` | `memfuse-candle` | **aus** | Aktiviert Stufe B/C (§9.2); Stufe A ist kein Feature, sondern Default-Verhalten sobald `memfuse-infer-candle` aktiv ist. War in der Vorfassung fälschlich als „an" mit 🟢-Status dokumentiert, obwohl der Code ein Stub war (§A2.1) |
| `edge-reinforcement-learning` | `memfuse-graph` | `memfuse-graph` | aus | Aktiviert `SignalKind::EdgeReinforcement`-Pfad |
| `fault-injection` | `memfuse-testkit` | `memfuse-store` | nur `dev-dependencies` | Deterministische I/O-Fehlerinjektion; wandert in den neuen Tooling-Crate `memfuse-testkit` |
| `loom` | `memfuse-mvcc`, `memfuse-graph` | `memfuse-store`, `memfuse-graph` | nur `dev-dependencies` | `loom::sync::*` statt `std::sync::*` hinter `#[cfg(loom)]` |
| `bm25f` | `memfuse-text` | `memfuse-text` | an | Feldgewichtete BM25-Bewertung |
| `adaptive-decay` / `-control` | `memfuse-cognition` | `memfuse-db` | an | Kalibrierungs-Feintuning |
| `partial-index-rebuild` | `memfuse-vector` | `memfuse-index` | an | Inkrementeller Indexaufbau |
| `onnx-bench` | `memfuse-bench` | — (v1 erzwang `onnx` hart) | aus | Einzige Stelle, die `memfuse-infer-onnx`/`ort` in einen Benchmark-Build zieht; bereits umgesetzt (§A2.1 D8) |

### 0.4 Globale Compile-Time-Regeln

**Jetzt (verbindlich, §A2.1 D1/D2 — löst den Mechanismus der Vorfassung vollständig ab):** Es gibt **genau
drei** Unsafe-Inseln, nicht sechs. Jeder Nicht-Insel-Crate erhält in `lib.rs`:

```rust
#![forbid(unsafe_code)]
```

Das ist zulässig, weil die Workspace-Lint-Stufe (§0.2) nur `deny` ist, nicht `forbid` — ein pauschaler
Workspace-`forbid` mit lokalem `allow` in den Inseln, wie er in der Vorfassung beschrieben war, kompiliert
wegen E0453 nicht und wurde deshalb verworfen.

| Insel (jetzt) | Begründung | Herkunft / Vorher |
|---|---|---|
| `memfuse-simd` | Distanzkernel mit Laufzeit-Dispatch (AVX2, AVX-512, NEON), Längenprüfung im safe Wrapper, `OnceLock<fn>`-Dispatch, Proptest gegen skalares Orakel je Stufe, Miri nur für den skalaren Pfad | vorher: `memfuse-index` (SIMD **und** mmap in einem Topf) |
| `memfuse-sys` (**neu**, §A2.1 D1) | `ReadOnlyMap` (mmap), `LockedBuf` (`mlock`/`munlock`/`VirtualLock`), Owner-only-ACL (Win32); Verträge safe gekapselt (`Deref<Target=[u8]>` für Mmap; Zeroize-vor-`munlock`-Drop-Guard für `LockedBuf`) | vorher verteilt und **unvollständig erfasst** in `memfuse-store` (Win32-ACL), `memfuse-index` (mmap), `memfuse-db` (`mlock`, `volatile_vault`) — die Vorfassung nannte hierfür fälschlich `memfuse-store`, `memfuse-db` und `memfuse-index` als *drei separate* Ausnahmen statt einer gemeinsamen Insel |
| `memfuse-wire` | FlatBuffers-Generat: `#![allow(unsafe_code, clippy::unwrap_used)]` auf Crate-Ebene, weil flatc-generierter Code `unwrap` für Felder mit Default erzeugt; bestehendes Drift-Gate `check_flatbuffers_drift` bleibt | vorher: `memfuse-core-ipc-gen` |

**Entfallen als Unsafe-Ausnahme (0 `unsafe` verifiziert, §A2.1 D1):** `memfuse-router`/`memfuse-adapt`
(Sherman-Morrison-Arithmetik ist in Safe Rust implementiert, keine Intrinsics) und `memfuse-embed`/
`memfuse-infer-onnx` (die C-FFI-Grenze zu `ort` liegt außerhalb des Crates in der `ort`-Bibliothek selbst).
`memfuse-db` entfällt als Ausnahme-Crate, weil `mlock` in die neue Insel `memfuse-sys` wandert.

**Erzwingung:** `tests/unsafe_islands.rs` (Workspace-Root) prüft (a) das Schlüsselwort `unsafe` kommt nur in
`memfuse-sys`, `memfuse-simd`, `memfuse-wire` vor, (b) jede Nicht-Insel-`lib.rs` enthält `forbid(unsafe_code)`,
(c) `allow(unsafe_code)` steht nur in den drei Inseln. Bis Migrationsphase 1c (§20) gilt eine explizit
benannte Übergangsliste mit Datei:Zeile-Angaben für `memfuse-vector`, `memfuse-store`, `memfuse-db` (Feature
`volatile-vault`) und `memfuse-wire` — diese Ausnahme ist befristet, nicht dauerhaft.

Jeder Crate erhält zusätzlich workspace-weit (§0.2, `[workspace.lints.clippy]`):

```rust
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::todo, clippy::unimplemented)]
```

**Vorher:** eine versionierte Ausnahmeliste `.unwrap-baseline.json` im Crate-Root plus CI-Ratchet
(`check-unwrap-baseline`), der nur schrumpfen durfte. **Jetzt:** Der Ratchet entfällt ersatzlos (§20 Phase
0R: `xtask/{unwrap_ratchet, check_unwrap_ratchet, check_unwrap_baseline_trend}.rs` werden gelöscht); die
verifizierte Zahl produktiver `unwrap`/`expect`/`panic`-Stellen liegt bei ca. 11 (Store 1, DB/Engine 1,
Crypto 3, Candle/Infer 1, Core/Types 1, Text 1, Py 3), diese werden im selben Umbau-PR behoben, in dem der
Lint scharf geschaltet wird — kein Ratchet, kein Übergangszustand mit offener Ausnahmeliste. `xtask` selbst
erhält befristet ein `#![allow]`, bis es in Phase 5 unter 3.000 Zeilen reduziert ist.

**`indexing_slicing` (neu, §A2.1 D6):** entgegen einem ursprünglich erwogenen workspace-weiten `deny` (das an
≈ 640 Hot-Path-Ausdrücken gescheitert wäre) gilt `deny` nur an klar benannten Parsing-/Decode-Grenzen
(`store/wal/{replay,encode}`, `store/sstable`-Decode, `memfuse-wire`, `memfuse-mcp/protocol`,
`memfuse-text/tokenizer`); in den rechenintensiven Kernen gilt `warn` plus `debug_assert!` am
Schleifeneintritt, `get_unchecked` ist ausschließlich in `memfuse-simd` zulässig.

### 0.5 Non-Obvious Decisions (systemweit bindend)

- **TxId-Generation:** IMMER `collection.allocate_tx()` — NIEMALS `SystemTime::as_nanos()`
- **fsync-Fehler:** IMMER mit `?` propagieren — NIEMALS `let _ = dir.sync_all()`
- **Dokument-Chunking:** IMMER `MarkdownChunker` — NIEMALS gesamten Text als 1 Vektor embedden
- **MCP-Transport:** stdio JSON-RPC 2.0 ONLY — axum wurde entfernt (ADR-010)
- **WAL-HMAC-Key:** IMMER via `load_or_create_integrity_key()` — NIEMALS hardcoded
- **TOMBSTONE_BIT-Disziplin (ADR-041):** Bit 63 strikt maskieren (`seq & !TOMBSTONE_BIT`) vor `max_seq`-Vergleichen
- **SSTable-Flush-Sichtbarkeit (ADR-043):** `last_committed_tx` vor `sstables.push()` aktualisieren

---

<a id="1-kernthese"></a>
## 1. Kernthese und Leitprinzip

**MemFuse ist eine souveräne, vollständig lokal betriebene Gedächtnisschicht für KI-Agenten** — eine
eingebettete, kryptographisch isolierte AI-Memory-Bibliothek in Rust mit Python- und MCP-Bindings, die
ohne Cloud-Abhängigkeit, ohne Telemetrie und ohne API-Key betrieben werden kann.

Ihr Alleinstellungsmerkmal ist die Kombination aus:

- einer **4-Signal-Retrieval-Fusion** (Vektor, Volltext, Graph, Metadaten) statt reiner Vektorsuche,
- einer **kryptographisch integritätsgesicherten Storage-Engine** (LSM-Tree, WAL mit HMAC-Kette, AES-256-GCM-SIV at rest),
- **WASM-/Sandbox-Ausführungsisolation** für Agent-Tool-Aufrufe,
- echter **Air-Gap-Inferenz** (lokales GGUF-Backend, kein Netzwerkzwang) mit verschlüsseltem, LSM-rückfallfähigem KV-Cache,
- einem **Contextual-Bandit-Router**, der Anfragen adaptiv auf Retrieval-Strategien verteilt,
- und — als jüngste, noch zu bauende Erweiterung des Datenmodells — **n-ären Hyperkanten** für Fakten, die sich
  nicht auf ein Subjekt-Prädikat-Objekt-Tripel reduzieren lassen (§6).

### Leitprinzip

**Korrektheit schlägt Performance schlägt Feature.**

Jede Optimierung, die eine Korrektheitsgarantie (Datenintegrität, Nebenläufigkeitssicherheit,
Wiederherstellbarkeit, Deadlockfreiheit) aufweicht, ist unzulässig — unabhängig vom Performancegewinn.
Jede Performance-Optimierung, die eine noch nicht spezifizierte Fähigkeit vorwegnimmt, ist nachrangig
gegenüber der Fertigstellung bereits spezifizierter Fähigkeiten. Jede Erweiterung eines bestehenden
Subsystems muss geprüft werden gegen die Invarianten, die dieses Subsystem bereits trägt — nicht nur
dagegen, *dass* eine Erweiterung grundsätzlich möglich ist, sondern *welches bestehende Invariant dadurch
unter Druck gerät* und wie es gewahrt bleibt.

---

<a id="2-vision"></a>
## 2. Produktvision, Alleinstellungsmerkmale und Nicht-Ziele

### 2.1 Was MemFuse ist

Eine eingebettete (embedded) Gedächtnisschicht, kein Cloud-Service. MemFuse läuft im Prozess des
aufrufenden Agenten oder als lokaler MCP-Server — es gibt keine serverseitige Multi-Tenant-Instanz
und keine Datenübertragung an Dritte, sofern nicht explizit über das Cloud-Egress-Gateway (§10.4)
angefordert.

### 2.2 Distributionswege

| Kanal | Paket | Zielgruppe |
|---|---|---|
| MCP-Server (primär) | `uvx memfuse-mcp --db-path ... --allow-write` | Claude Desktop, Cursor, beliebige MCP-Clients |
| Python-Bibliothek | `pip install memfuse` | In-Process-Einbettung in Python-Agenten |
| Rust-Crate | `cargo add memfuse-db` | Native Rust-Anwendungen |

Eine Desktop-Shell (`memfuse-tauri`) existierte als Prototyp, ist aber zugunsten der PyPI-Bibliothek
und des MCP-Servers als primäre Vertriebswege eingestellt (deprecated, ADR-077).

### 2.3 Alleinstellungsmerkmale und ihr Reifegrad

1. **4-Signal-Hybridsuche** (🟢) — Vektorsuche (HNSW), Volltextsuche (BM25/BM25F), Wissensgraph-Traversierung
   (CSR + Forward-Push-Personalized-PageRank), Metadaten-Filter; fusioniert über Reciprocal Rank Fusion
   (RRF, Default 🟢) oder score-normalisierte Fusion (Opt-in 🟡, mit hartem RRF-Fallback bei Signaldegradation).

2. **Kalibriertes Retrieval mit Lyapunov-Drift-Erkennung** (🟢) — Score-Schwellenwerte werden nicht statisch,
   sondern über ein laufend kalibriertes Modell mit gedeckelter Drift-Eskalation bestimmt.

3. **MCP-native Zero-Trust-Sandbox** (🟢) — Tool-Ausführung mit getrennt konfigurierbarem Fuel- (Rechenschritt-)
   und Wall-Clock-Budget (Default 5 s), orthogonal zueinander konfigurierbar.

4. **Kryptographische DSGVO-Art.-17-Löschung** (🟢) — Löschvorgänge erzeugen einen verifizierbaren `DeletionProof`
   über eine race-freie HMAC-Kette.

5. **Session-DAG** (🟢) — Konversationsverzweigung als persistenter, azyklischer Graph.

6. **Air-Gap-KV-Cache-Bridge mit LSM-Fallback-Spill** (🟢) — der KV-Cache liegt primär verschlüsselt im RAM; bei
   Speicherdruck greift kontrolliertes Auslagern auf die SSD statt verlustbehafteten Verwerfens.

7. **Cloud-Egress Privacy Gateway** (🟢, weitgehend auditiert) — mehrschichtiger DLP-Pfad mit Surrogat-Tokenisierung,
   Bulk-Exfiltration-Erkennung und Rehydration der Cloud-Antwort (§10.4).

8. **Contextual-Bandit-Routing** (🟢 Grundfunktion / 🟡 mathematisch korrekte Variante) — LinUCB-basiertes Routing;
   siehe §8 für die Unterscheidung zwischen Produktions-Default und Ridge-korrekter Opt-in-Variante.

9. **Gestufte Vektorindex-Architektur** (🟢 HNSW / 🟢 DiskANN als Tier) — HNSW als Standard, DiskANN für
   RAM-sprengende Korpora.

10. **Deutsche Morphologie inkl. BM25F** (🟢) — Kompositazerlegung im Volltextindex plus feldgewichtete Bewertung.

11. **Zero-Copy-Storage-Pfad** (🟢) — seit der Grundarchitektur produktiv.

12. **Key-granulare Schreibnebenläufigkeit** (🟢) — `kv_locks` statt collection-weitem Mutex.

13. **N-äre Hyperkanten** (🔴, vollständig spezifiziert, siehe §6) — Fakten mit mehr als zwei Beteiligten als
    erstklassige, atomar invalidierbare Struktur statt Zerlegung in mehrere, im Zusammenhang verlorene Binärkanten.

### 2.4 Nicht-Ziele

MemFuse ist explizit **kein** Cloud-SaaS-Produkt, **kein** Multi-Tenant-Enterprise-System, **kein** Framework für
LLM-Training, **keine** primär GUI-getriebene Desktop-Anwendung und **kein** Cluster-/Replikations-System. Ein
`memfuse-cluster`-Veto besteht bewusst: verteilter Konsensbetrieb ist kein Ziel der aktuellen Produktphase.
Passives WAL-Shipping für Backup-Zwecke ist als Fernziel vorgesehen (Roadmap-Stufe 4, §18), aber nicht Bestandteil
des Kernprodukts.

---

<a id="3-prinzipien"></a>
## 3. Architekturprinzipien P1–P30

Diese Prinzipien sind normativ für jede gegenwärtige und künftige Erweiterung des Systems. **P1–P25 sind aus
der Vorfassung unverändert übernommen** und gelten fort. **P26–P30 sind ab dieser Fassung neu** (Quelle: Teil
A2, `MEMFUSE_ZIELARCHITEKTUR_v2.md` §2). Zusätzlich ändert Teil A2 die **Anwendung** von P5 und P8–P22 auf
Crate-Ebene (siehe die Δ-Vermerke unten), ohne ihren Wortlaut zu ändern.

**P1 — Korrektheit schlägt Performance schlägt Feature.** Siehe §1.

**P2 — WAL-First-Persistenz.** Keine Zustandsänderung wird im Speicher sichtbar gemacht, bevor sie physisch
in das Write-Ahead-Log geschrieben und mit dem Datenträger synchronisiert wurde.

**P3 — Deterministische Recovery.** Der Systemzustand muss sich allein aus dem WAL rekonstruieren lassen.

**P4 — Keine stillschweigende I/O-Fehlerunterdrückung.** `fsync`-Fehler IMMER mit `?` propagieren.

**P5 — Strikte DAG-Modularität.** Abhängigkeiten im Crate-Graphen verlaufen strikt abwärts; ein Verstoß
gilt als Architekturdefekt. **Δ (verbindliche Korrektur, §A2.1):** In der Vorfassung war dieses Prinzip
dokumentiert, aber am realen Crate `memfuse-db` (Layer 2) nachweislich verletzt — es hing hart von
`memfuse-candle`/`memfuse-ollama` (Layer 3) und optional von `memfuse-embed` (Layer 3) ab, weil
`EmbeddingBackend` samt Konstruktion in `memfuse-db` lag. Ab dieser Fassung wird P5 **maschinell** erzwungen
(`tests/layering.rs` gegen `cargo_metadata`, plus `deny.toml`-Bans mit `wrappers`-Liste, §4.2 unten), nicht
mehr nur dokumentiert. Reihenfolge und Richtung sind jetzt die Ring-Matrix in §4.2 (Ring-Modell), nicht mehr
die alte Layer-Tabelle.

**P6 — Feingranulare Fehlerbehandlung.** Domänenspezifische `Result<T, E>`-Enums statt generischer `panic!`-Pfade.

**P7 — Verbot von `unwrap()`/`expect()` auf toxischen Daten.** CI-gated über `.unwrap-baseline.json`.

**P8–P22 — Sovereign-Core-Grundsätze.** Umfassen u. a.: Verschlüsselung at rest als Default, HMAC-Kettenintegrität,
Zero-Trust-Sandbox, Air-Gap-fähige Inferenz, deterministische Transaktions-ID-Vergabe, Tombstone-Bit-Disziplin,
SSTable-Flush-Sichtbarkeit, Kaskaden-Invalidierung, kryptographische Löschnachweise, feingranulare Feature-Gates.

**P23 — Zeitbudgets sind orthogonal konfigurierbar.** Rechenschritt-Budget (Fuel) und Wall-Clock-Budget für
Sandbox-Ausführungen sind zwei unabhängige Achsen. Ein Tool kann rechnerisch günstig, aber durch blockierendes I/O
langsam sein, oder umgekehrt — beide Fälle müssen unabhängig begrenzbar sein. 🟢 Produktiv erfüllt (§10.2).

**P24 — Lokalität vor globaler Neuberechnung.** Jeder Algorithmus, dessen Eingabe eine anfragebestimmte Teilmenge
des Gesamtzustands ist (PPR mit wenigen Seed-Knoten, Cascade-Invalidierung ausgehend von einem Dokument), MUSS
eine zur Anfragegröße proportionale Laufzeit haben — niemals zur Größe des Gesamtzustands ($O(V+E)$ ist für
solche Anfragen unzulässig). Dieses Prinzip ist der normative Grund für den Forward-Push-PPR-Algorithmus (§7.2)
und für das harte Fan-out-Limit der Hyperkanten-Cascade-Invalidierung (§6.5, H5).

**P25 — Cache-Treffer sind lock-frei bzw. lock-günstig zu gestalten.** Ein Lesetreffer im Block-Cache soll nach
Möglichkeit keinen exklusiv sperrenden, mutierenden Zugriff erfordern, da Cache-Treffer der mit Abstand häufigste
Zugriffspfad sind und jede darin verborgene Schreibsperre unter Last zur Kontention wird (§5.4).

### Neu ab dieser Fassung: P26–P30 (Teil A2)

**P26 — Sync-Kern, async-Schale.** Ring 0 (`memfuse-types` … `memfuse-adapt`) enthält kein `tokio`. Kerne sind
Zustandsautomaten ohne I/O; die Engine (Ring 3) treibt Persistenz und bindet Kerne per begrenztem `ComputePool`
an. **Vorher/Jetzt:** Die Vorfassung empfahl Sync-Kerne als wünschenswert, ohne Ursache zu benennen; verifiziert
ist, dass die tatsächliche Ursache heutiger `async fn` in den Kernen zweierlei ist — native AFIT-Ports (nicht
`dyn`-kompatibel) und I/O-Aufrufe direkt in den Kernmodulen (`graph/csr.rs`, `text/inverted.rs`,
`index/diskann.rs`), nicht Rechenlogik. Die Migration trennt daher zuerst die Ports (P27), dann das I/O (§4.1
der Quelle), bevor Kerne als synchron gelten dürfen. Erzwingung: `cargo tree -e normal -p memfuse-{vector,text,graph,rank,adapt}` ohne `tokio`.

**P27 — Ports sind `dyn`-kompatibel per Konstruktion.** Traits in `memfuse-ports` sind synchron, wo das
Blockierverhalten ohnehin durch `mmap`/`pread` gegeben ist (`StorageRead`, `VectorIndex`, `TextIndex`,
`GraphIndex`); wo echtes asynchrones Warten nötig ist (`StorageWrite::commit`, `Embedder::embed`), liefern sie
`BoxFuture` statt natives `async fn`, weil natives AFIT nicht objektsicher ist. Grund: die Engine muss Backends
zur Laufzeit als `Arc<dyn Trait>` austauschen können (Composition Root, Ring 4 `memfuse`, §4.2).

**P28 — Injizierter Nichtdeterminismus.** `Clock`, `Rng`, `IdGen` sind Ports, niemals direkte Aufrufe von
`SystemTime::now()`/`rand::thread_rng()` in Kern- oder Persistenzcode. `memfuse-testkit` liefert
`ManualClock` und eine In-Memory-`StorageEngine` für deterministische Simulationstests (WAL/LSM-Crash-Injektion,
§20 Phase 3a) und `loom`-Lock-Protokoll-Tests. **Δ gegenüber Vorfassung:** `memfuse-testkit` entsteht bereits
in Migrationsphase 0R, nicht erst nachträglich — Determinismus-Infrastruktur geht der Sync-Migration voraus,
nicht hinterher.

**P29 — Kein globaler veränderlicher Zustand.** `static OnceLock`/`Lazy` sind ausschließlich für unveränderliche
Konstanten zulässig (Regex, Stopwortlisten). Veränderlicher Zustand gehört immer einer Instanz. **Grund:**
verifiziert wurde ein globaler `static ORPHAN_REGISTRY: OnceLock` in `checkpoint/orphan.rs` sowie ein als
`unimplemented!()` spezifizierter globaler `static CIPHER_INSTANCE` (§9.3) — beide widersprechen der
Schlüssel- bzw. Registry-Hierarchie und werden im Zuge der Migration entfernt (§20 Phase 1c bzw. §9).

**P30 — Jeder Crate-Zuschnitt braucht ein Kriterium.** Ein Crate existiert nur, wenn er mindestens eines
erfüllt: **I** Isolation einer flüchtigen Abhängigkeit (candle, ort, wasmtime, pyo3, reqwest) · **U**
Unsafe-Insel · **C** eigener Änderungsrhythmus/Bounded Context · **S** Größe > 8.000 LOC (Compile-Parallelität)
· **D** Richtungserzwingung (z. B. Composition Root). Crates ohne erfülltes Kriterium werden zusammengelegt.
Die vollständige Kriterienzuordnung für alle 27+3 Crates steht in §4.2.

### Ergänzende Grundsätze

- **Nebenläufigkeitssicherheit vor Nebenläufigkeitsperformance:** Sperrenhierarchien werden explizit dokumentiert
  und dürfen nicht durch bloßen Analogieschluss auf neue Mutationspfade übertragen werden, ohne die
  Deadlockfreiheit für den neuen Fall erneut zu beweisen (konkretes Beispiel: §6.5, H2).

- **Geschlossene Enums bleiben geschlossen:** Wo ein Enum bewusst **nicht** `#[non_exhaustive]` deklariert ist
  (z. B. `SignalKind`), ist das eine architektonische Entscheidung. Eine neue Kategorie von Information wird
  in ein bestehendes offenes Signal integriert, statt das Enum breaking zu erweitern (§6.5, H3).

- **Kein Sicherungsnetz, keine Schema-Änderung:** Persistenzformat-Änderungen werden nur vorgenommen, wenn ein
  automatisiertes CI-Drift-Gate zwischen Schema und generiertem Code aktiv läuft (§6.5, H4).

- **Explizite Unvollständigkeit statt stiller Lücken:** Wo ein Subsystem eine neue Datenklasse strukturell nicht
  berücksichtigt, wird dies über ein sichtbares Konfigurations-/Report-Flag markiert (§6.5, H6).

---

<a id="4-architektur"></a>
## 4. Systemarchitektur: der Crate-Graph (Ring-Modell, vormals Crate-DAG)

> **Status dieses Abschnitts:** Der Layer-0–5-Crate-DAG (unten unter „4.0 Vorher" archiviert) ist **nicht mehr
> normativ**. Ab dieser Fassung gilt das in §4.2 beschriebene Ring-0–4-Modell aus Teil A2 als verbindlich. Der
> Umbau erfolgt strangler-artig gemäß §20, nicht per Big-Bang — bis ein Migrationsschritt abgeschlossen ist,
> existiert der jeweilige alte Crate als `#[deprecated]`-Re-Export.

### 4.i Systeminvarianten §4(1)–§4(7) (normativ, Fassung 2.1)

Die sieben Invarianten gelten für jeden Crate und jeden Abschnitt dieses Dokuments. Verweise der Form „§4(n)"
und „Invariante n" (auch in Anhang B) bezeichnen die hier nummerierte Invariante. Ein Entwurf, der eine
Invariante nur „im Prinzip" wahrt, gilt als verletzt.

1. **Zero-Panic-Doktrin.** In Produktionspfaden (`src/`, ohne Tests und Benchmarks) gibt es kein `unwrap`,
   `expect`, `panic!`, `unreachable!`, `unimplemented!` (außer in normativen Signatur-Stubs dieses Dokuments),
   `todo!`, keine ungeprüfte Indizierung oder Slice-Bildung mit berechneten oder externen Indizes (stattdessen
   `get(..)` plus Fehler) und keine überlaufende Größenarithmetik auf Eingabewerten (`checked_*`/`saturating_*`).
   Erzwingung: Lint-Konfiguration aus §0.4. `catch_unwind` nur an FFI-Grenzen und nur unter `panic = "unwind"`.
2. **Unsafe-Isolation.** `#![forbid(unsafe_code)]` in jedem Nicht-Insel-Crate; `unsafe` nur in den drei
   Unsafe-Inseln (§0.4), jeder Block mit `// SAFETY:`. Folge: Entwürfe, die `unsafe` brauchen (intrusive
   lock-freie Listen, `MaybeUninit`-Ringe, `crossbeam_epoch::Shared::deref`), sind außerhalb der Inseln
   unzulässig.
3. **Determinismus.** Gleiches Binary, gleiche SIMD-Dispatch-Stufe, gleiche injizierte Zufalls- und Zeitquelle
   (P28) und gleicher logischer Zustand liefern gleiche Ergebnisse (Replay, Recovery, Tests). Die
   Iterationsreihenfolge von Hash-Containern geht nie in Ergebnisse, IDs, Serialisierung oder Solver-Eingaben
   ein (sortierte Reihenfolge oder totale Ordnung mit Tiebreaker). Ein Hasher, der Lock-Zuordnung oder Ergebnisse
   bestimmt, wird **einmal** pro Instanz mit festen Seeds angelegt, nie pro Aufruf (§6.6 H2). Über SIMD-Stufen
   hinweg gilt die Toleranz des Proptest-Orakels (§15.1), keine Bitgleichheit. Es besteht kein Cluster- oder
   Replikationsanspruch (§2.4).
4. **Deadlockfreiheit.** Sperrenhierarchie (§5.1) plus kanonische Reihenfolge innerhalb einer Stufe
   (aufsteigender Shard-Index, §5.2a, H2). Kein `.await` unter einem `std`-Lock. Wer auf einer begrenzten Queue
   blockieren kann, hält keinen Lock, den die Gegenseite braucht. Nachweis: Loom (§15.3).
5. **Speicherbudget-Transparenz.** Jede Operation, die neben einem bestehenden Snapshot, Index oder Puffer einen
   zweiten aufbaut (Compaction, Rebuild, Merge), berechnet vorab Residenz **und** Spitze (capacity-basiert,
   inklusive Tabellen-Overhead; §6.3) und bricht bei Überschreitung ohne Seiteneffekt mit typisiertem Fehler
   ab. Queues und Puffer in Produktionspfaden sind begrenzt.
6. **Lokalität und beschränkte Arbeit (P24).** Die Kosten einer latenzkritischen Operation hängen von der
   Größe der anfragebestimmten Teilmenge oder einer konfigurierten Obergrenze ab, nie von der Gesamtgröße des
   Bestands. Darüber hinausgehende Arbeit wird in persistente, wiederaufnehmbare Hintergrundarbeit überführt
   (§6.6 H5).
7. **Zero-Copy.** Payloads (Bytes, Vektoren, Teilnehmerlisten) werden zwischen Ringen geteilt (`Bytes`,
   `Arc<[T]>`, mmap-Slices), nicht kopiert; eine „Sicht" darf keine Kopie erzeugen (Test per `Arc::ptr_eq` oder
   Allokationszähler). **Snapshot-Regel S1:** Ein per `ArcSwap` veröffentlichter Snapshot ist unveränderlich;
   seine Felder enthalten keine Container mit innerer Mutabilität (`scc::HashMap`, `Mutex`, Atomics mit Einfluss
   auf die Lesesemantik), sonst ist die Atomaritätsargumentation des RCU-Swaps hinfällig.

### 4.0 Vorher: der Layer-0–5-Crate-DAG (archiviert, nicht mehr normativ)

MemFuse gliederte sich bis zur Vorfassung in einen mehrschichtigen Rust-Workspace mit zehn benannten Crates.
Abhängigkeiten sollten strikt abwärts verlaufen; **verifiziert wurde jedoch, dass diese Regel selbst verletzt
war** (§A2.1, P5-Δ) — der wichtigste Grund für die Ablösung durch das Ring-Modell.

| Layer | Crates | Verantwortung |
|---|---|---|
| **0** | `memfuse-core-ipc-gen`, `memfuse-core` | FlatBuffers-generierter IPC-Code; Kerntypen (`DocId`, `EntityId`, `TxId`, `ConfigFingerprint`), Traits, Fehlerbehandlung. |
| **1** | `memfuse-store`, `memfuse-index`, `memfuse-text`, `memfuse-crypto`, `memfuse-graph`, `memfuse-checkpoint`, `memfuse-calibration` | Persistenz- und Indexierungs-Primitive. Sollten einander laut Spec nicht kennen — real hingen `store` und `index` beide von `crypto` ab, `candle` von `store` (§A2.1, „§4 Tabelle" widerlegt). |
| **2** | `memfuse-db` | Öffentliche `Collection`-API, 4-Signal-Fusion, Multi-Step-Query-Engine, Kontext-Kompaktierung, Provenance-Tracking. Real: 26,7k LOC, > 40 öffentliche Methoden, bündelte mehrere Bounded Contexts (Datenebene, Retrieval, Kognition, Ingestion, `volatile_vault`) und hing **aufwärts** von `candle`/`ollama`/`embed` ab. |
| **3** | `memfuse-ollama`, `memfuse-candle`, `memfuse-embed`, `memfuse-agent`, `memfuse-router`, `memfuse-py` | Inferenz-Backends und Anwendungslogik. `memfuse-router` war laut Spec reine Bandit-Mathematik; real enthielt er zusätzlich SLM-Profil-Routing, MCP-Dispatch und Type-State-Egress-Schutz. |
| **4** | `memfuse-mcp`, `memfuse-sandbox` | Externe Schnittstelle für Agenten. `memfuse-sandbox` war real ein Waisen-Crate ohne Konsumenten. |
| **5** | `memfuse-bench` | Benchmark-Harness. Erzwang real ein hartes `onnx`-Feature und zog dadurch Netzwerk-Downloads in jeden `--workspace`-Build. |

Diese Tabelle wird ausschließlich zur Einordnung bestehenden Codes und bestehender Diskussionen (Issues, PRs,
ADRs vor dieser Fassung) aufbewahrt. Für neue Arbeit gilt ab sofort §4.2.

### 4.1 Safety-First-Doktrin

Safe Rust ist der Standard; `#![forbid(unsafe_code)]` gilt per Default in jedem Nicht-Insel-Crate. **Jetzt: drei**
namentlich benannte Unsafe-Inseln (`memfuse-sys`, `memfuse-simd`, `memfuse-wire`) mit `#![allow(unsafe_code)]`
auf Crate-Ebene und `// SAFETY:`-Beweiskommentar an jeder Stelle (§0.4). **Vorher** waren es sechs pauschal
benannte Ausnahme-Crates mit einer Mechanik (`forbid` + lokales `allow`), die wegen E0453 nicht kompilierbar
gewesen wäre (§A2.1, D2) — dieser Fehler wird mit dieser Fassung korrigiert, nicht fortgeschrieben.

### 4.2 Ring-Modell (Zielarchitektur v2, verbindlich)

**Leitprinzipien:** siehe §3, P26–P30, sowie P1–P25 unverändert. **Warum diese Schnitte:** (a) reine Trennung
gegen I/O, (b) Unsafe-Isolation, (c) flüchtige Abhängigkeiten (candle, ort, wasmtime, pyo3, reqwest) als
Blätter, (d) Bounded Contexts, geprüft gegen das Kriterium I/U/C/S/D (P30). Die Anzahl der Crates ist kein
Ziel für sich — sie ist eine Konsequenz der Kriterien, mit einer offenen Konsolidierungsoption (§A2.4 Nr. 4).

| Ring | Crate | Inhalt | Herkunft (vorher) | Kriterium (P30) |
|---|---|---|---|---|
| **0** (sync, kein `tokio`) | `memfuse-types` | IDs (`DocId`, `DocIdx`, `TxId`, `TenantId`), `ModelFingerprint`, Filter-AST, Budgets, Importance, `ErrorClass`, Schema-Versionen, Tombstone-Semantik | `memfuse-core` (Teilmenge `types`) | D |
| | `memfuse-ports` | Traits (P27): `VectorIndex`/`TextIndex`/`GraphIndex`/`StorageRead` (sync), `StorageWrite`/`Embedder`/`TextGenerator`/`KvPrefixStore`/`ToolSandbox` (async, `BoxFuture`), `Clock`/`Rng`/`IdGen`/`MetricsSink`/`DriftStatusProvider` | `memfuse-core` (Teilmenge `traits`), `memfuse-db::DriftStatusProvider` | D |
| | `memfuse-mvcc` **(neu)** | `SeqLog`, `SnapshotRegistry`, `TxBuffer`; loom-getestet | `memfuse-core` (`seq_log`, `snapshot`, `tx_buffer` — in der Vorfassung des Migrationsplans nicht zugeordnet, §A2.1 D4) | C, D |
| | `memfuse-wire` | FlatBuffers-Generat und Adapter, Unsafe-Insel | `memfuse-core-ipc-gen` | U |
| | `memfuse-sys` **(neu)** | `ReadOnlyMap` (mmap), `LockedBuf` (mlock/VirtualLock), Owner-only-ACL (Win32), Unsafe-Insel | verteilt über `memfuse-store`, `memfuse-index`, `memfuse-db` (§0.4) | U |
| | `memfuse-simd` | Distanzkernel, Laufzeit-Dispatch, Unsafe-Insel | `memfuse-index` (`distance.rs`) | U |
| | `memfuse-crypto` | Schlüsselhierarchie, AEAD, WAL-HMAC-Kette, Deletion-Proof, Zeroize, Anti-Tamper | `memfuse-crypto` (verschlankt: Egress-Vault und KV-Segment wandern nach `privacy`/`kvcache`) | C |
| | `memfuse-vector` | HNSW, DiskANN, Quantisierung | `memfuse-index` | S, C |
| | `memfuse-text` | BM25/BM25F, Morphologie | `memfuse-text` | C |
| | `memfuse-graph` | CSR, PPR, Leiden, Hyperkanten | `memfuse-graph` | S, C |
| | `memfuse-rank` | 4-Signal-Fusion, Isotonic/Platt-Kalibrierung, Drift | `memfuse-db::fusion`, `memfuse-calibration` | C |
| | `memfuse-adapt` | Bandit, Lyapunov, PID, Homeostat, Decay; `Clock`/`Rng` injiziert (P28) | `memfuse-router` (Bandit/Lyapunov-Teil), `memfuse-calibration::pid`, `memfuse-db` (Decay/Homeostat/PID) | C |
| **1** (Persistenz, async an I/O-Grenzen erlaubt) | `memfuse-store` | WAL (Group-Commit, HMAC-Kette), LSM, MVCC-Pin | `memfuse-store` | S, C |
| | `memfuse-kvcache` **(neu)** | Prefix-Radix-Baum, KV-Blöcke, Tiering, AEAD, Segmentdateien (§9) | `memfuse-crypto::kv_segment`, `memfuse-candle::kv_bridge` | C |
| | `memfuse-checkpoint` | Time-Travel-Registry gegen Port `StorageEngine`, ohne globalen Zustand | `memfuse-checkpoint` (Global-State `ORPHAN_REGISTRY` entfernt, P29) | C, D |
| **2** (Blätter, flüchtige Abhängigkeiten) | `memfuse-infer-candle` | GGUF, eigenes Llama-Modell mit `KvState` (§9, Stufe B) | `memfuse-candle` | I |
| | `memfuse-infer-ollama` | HTTP-Backend, Contextual-Chunk-Prefixing | `memfuse-ollama` | I |
| | `memfuse-infer-onnx` | `ort`, Cross-Encoder; aus `default-members` ausgeschlossen | `memfuse-embed` | I |
| | `memfuse-sandbox` | WASM-Isolation, Fuel- und Wall-Clock-Budget; implementiert `ToolSandbox` | `memfuse-sandbox` (bislang Waisen-Crate, jetzt an `memfuse-agent`/`memfuse-mcp` angebunden) | I |
| **3** (Anwendungskern) | `memfuse-engine` | Collection, Transaktionen, `RetrievalPlanner`, Ingestion, Export/Import, `ComputePool` | `memfuse-db` (Datenebene) | S, C |
| | `memfuse-cognition` | Consolidation, Synthese, Kompaktierung, Scheduler | `memfuse-db` (Kontrollebene) | C |
| | `memfuse-privacy` | Egress-Gateway, PII-Vault, DLP, `GuardedPayload`, Prompt-Injection-Filter | `memfuse-crypto::egress_vault`, `memfuse-mcp::egress_gateway`, `memfuse-router::guarded_payload` | C |
| | `memfuse-router` | SLM-Profil-Routing, MCP-Dispatch (schlank; **keine** Numerik mehr — die wandert nach `adapt`) | `memfuse-router` (Rest nach Abzug von `adapt`) | C |
| | `memfuse-agent` | Workflow-Engine, Audit, DLQ | `memfuse-agent` | C |
| **4** (Ränder) | `memfuse` **(neu)** | Fassade, Builder, **einzige Composition Root** | `memfuse-db` (Fassaden-Anteil) | D |
| | `memfuse-mcp` | stdio-JSON-RPC, Protokoll, Tool-Wiring | `memfuse-mcp` | C |
| | `memfuse-py` | PyO3, eigene Runtime, `catch_unwind` | `memfuse-py` | I |
| **Tooling** | `memfuse-testkit` **(neu)**, `memfuse-bench`, `xtask` | Fault-VFS, `ManualClock`, In-Memory-`StorageEngine` (P28); Benchmarks; CI-Tooling (Ziel < 3.000 LOC) | `memfuse-bench`, `xtask` | — |

**Entfallen als eigenständige Crates:** `memfuse-core`, `memfuse-core-ipc-gen`, `memfuse-calibration`,
`memfuse-embed`, `memfuse-candle`, `memfuse-ollama`, `memfuse-db` (nach Abschluss der Strangler-Phase, §20).
**Löschkandidat, vor Löschung zu prüfen:** `memfuse-core/types/saos.rs` (707 LOC, nur intern referenziert).

### 4.3 Abhängigkeitsmatrix (löst die alte Layer-Reihenfolge ab)

```
Ring 0  → Ring 0 in Reihenfolge  types → {ports, mvcc, wire, sys, simd, crypto} → {vector, text, graph, rank, adapt}
          Kerne (vector, text, graph, rank, adapt) kennen einander nicht; Kommunikation nur über types/ports/mvcc.
Ring 1  → Ring 0. Kein Ring-1-Crate hängt von einem anderen Ring-1-Crate ab.
Ring 2  → types, ports (+ crypto für Fingerprints). Niemals Ring 1 oder 3.
Ring 3  → Ring 0, Ring 1, Ports von Ring 2 (nie deren konkrete Crates).
          Interne Ordnung: privacy < engine < {cognition, router, agent}.
Ring 4  → alles.
dev-Kanten → nur memfuse-testkit und Crates desselben oder tieferen Rings.
```

**Erzwingung (verbindlich ab Migrationsphase 1a, Warnmodus in Phase 0R, §20):**
1. `tests/layering.rs` (Workspace-Root, `cargo_metadata`) prüft alle Kantentypen (normal, build, dev,
   target-spezifisch) gegen die Matrix.
2. `deny.toml` (bestehende Datei **erweitern**, nicht neu anlegen — §A2.1 D7) erhält `[[bans.deny]]`-Einträge
   mit `wrappers`-Listen für `tokio` (verboten außerhalb Ring 1+), `candle-core`, `ort`, `wasmtime`, `pyo3`,
   `reqwest` (je genau ein erlaubtes Blatt-Crate). `wrappers` begrenzt nur direkte Abhängige; transitives
   Einschleppen prüft eine `cargo tree`-Assertion in CI zusätzlich.
3. `tests/unsafe_islands.rs` (§0.4).

### 4.4 Trait-Eindeutigkeit und Modul-Governance (⚠️ Opus-Optimierung 2.6, Stufe 2, mittel) — historischer Befund, Crate-Zuordnung aktualisiert

**Hinweis:** Der folgende Befund wurde am vormaligen `memfuse-core/src/traits/` erhoben. Nach der Migration
(§20, Phase 1b) liegt dieses Verzeichnis in `memfuse-ports`; die Maßnahmen gelten unverändert für den neuen
Ort. Der Befund selbst bleibt hier unverändert dokumentiert, damit er nicht verloren geht.

**Problem:** `memfuse-core/src/traits/` enthält zehn Dateien; `mod.rs` deklariert nur fünf. Vier Dateien
(`graph_index.rs`, `lifecycle.rs`, `text_index.rs`, `vector_index.rs`) sind dadurch **nie kompiliert** und
enthalten Zweitdefinitionen von neun Verträgen (`VectorIndex`, `TextEmbeddingEngine`, `SegmentSynthesizer`,
`TextIndex`, `GraphIndex`, `DistanceCalculator`, `MemoryLifecycleManager`, `GroundingValidator`,
`ResponseGroundingValidator`), die parallel in `index.rs`/`observability.rs` leben. `GraphIndex` ist zwischen
beiden Fassungen bereits inhaltlich auseinandergelaufen (abweichende Doc-Kontrakte und Default-Implementierungen)
— der Compiler kann das nicht erkennen, weil die unverdrahtete Fassung nie gebaut wird. Das bestehende
Duplikat-Gate (`xtask check_duplicate_symbols`) prüft laut eigener Spezifikation nur *innerhalb derselben Datei*
und ist für dateiübergreifende Duplikate im selben Modulverzeichnis strukturell blind.

**Einordnung:** Die vier unverdrahteten Dateien sind die **korrekte** Zerlegung (ein Vertrag pro Datei); der
tatsächlich kompilierte Zustand ist der Monolith `index.rs` (drei unabhängige Verträge in einer Datei). Eine
Lösung, die schlicht die vier Dateien löscht, würde die bessere Zerlegung entfernen und die falsche behalten.

**Maßnahme (verbindlich):**
1. Divergenz in `GraphIndex` auflösen — die aktuell kompilierte Fassung in `index.rs` ist die Referenz für die
   Zusammenführung, nicht automatisch die inhaltlich richtige.
2. `graph_index`, `vector_index`, `text_index`, `lifecycle` in `traits/mod.rs` deklarieren.
3. `index.rs` und die duplizierten Teile von `observability.rs` löschen, sobald (1)/(2) grün sind.
4. **Governance-Gate `GOV-D` (neu, CI-Pflicht):** `xtask check-module-reachability` verifiziert, dass jede
   `.rs`-Datei unter `src/` von genau einer `mod`-Deklaration aus erreichbar ist. Eine unerreichbare, aber
   vorhandene Datei ist ein CI-Fehler, kein stiller Zustand — dies schließt exakt die Lücke, die `CORE-D`
   ermöglicht hat.

**Testpflicht:** `crates/memfuse-core/tests/no_orphan_modules.rs` — schlägt fehl, sobald eine Datei unter `src/`
existiert, die von keinem `mod`-Pfad aus erreichbar ist.

---

<a id="5-speicher"></a>
## 5. Speicherschicht: LSM-Tree, WAL und Block-Cache

### 5.1 Grundprinzip und Lock-Hierarchie

Keine Zustandsänderung wird im Speicher sichtbar gemacht, bevor sie physisch in das Write-Ahead-Log geschrieben
und mit dem Datenträger synchronisiert wurde (WAL-First, P2). Der Systemzustand muss sich allein aus dem Log
rekonstruieren lassen (deterministische Recovery, P3). Schreibzugriffe sperren nicht die gesamte Collection, sondern
nur die betroffenen Schlüssel über eine key-granulare Lock-Hierarchie:

```
collections (RwLock) → kv_locks (schlüssel-granular, KvKeyLocks) → embedder (RwLock)
```

Diese Hierarchie ist für **Einzelschlüssel**-Mutationen ausgelegt und deadlockfrei bewiesen. Jede künftige
Mutation, die mehrere Schlüssel gleichzeitig unter `kv_locks` hält, muss diesen Beweis für den Mehrschlüsselfall
gesondert führen — sie darf sich nicht per Analogieschluss auf den Einzelschlüsselfall berufen (konkret
angewendet in §6.5, H2).

### 5.2 Modulstruktur `memfuse-store`

```
crates/memfuse-store/src/
├── lib.rs
├── lsm.rs               # LSM-Tree, Compaction
├── wal.rs                # Write-Ahead-Log + HMAC-Kette
├── wal_flusher.rs        # begrenzter MPSC-Group-Commit-Actor (§5.3)
├── block_cache/
│   ├── mod.rs            # BlockCacheBackend-Trait
│   ├── lru.rs            # LruBlockCacheBackend (Default)
│   └── sieve.rs          # SieveCacheBackend (Opt-in, `block-cache-v2`)
├── kv_locks.rs           # KvKeyLocks (§5.2a)
└── error.rs
```

### 5.2a Key-granulares Locking: `kv_locks.rs` (normativ)

**Änderung Fassung 2.1:** (a) Die Zuordnung Schlüssel → Shard liegt ausschließlich in `KvKeyLocks` (`key_hash`,
eine Hasher-Instanz mit festen Seeds). Der Aufrufer hasht nie selbst. Fassung 2 hashte in H2 mit
`ahash::RandomState::new().hash_one(e)`; jede `RandomState`-Instanz ist zufällig geseedet (mit ahash 0.8 geprüft:
zwei Aufrufe liefern verschiedene Werte, auch prozessübergreifend), also konnten zwei Threads dieselbe Entität
auf verschiedene Shards abbilden — dann fehlt der gegenseitige Ausschluss. (b) `acquire` liefert `Result`
statt `.unwrap()` (§4(1)). (c) `acquire_multi_sorted` sortiert und dedupliziert selbst; die Eingabe muss nicht
vorsortiert sein.

```rust
use std::hash::{BuildHasher, Hash};
use std::sync::{RwLock, RwLockWriteGuard};

/// Sperrenhierarchie (verbindlich, systemweit einzuhalten):
///   collections (RwLock) → kv_locks (schlüssel-granular) → embedder (RwLock)
pub struct KvKeyLocks {
    shards: Vec<RwLock<()>>,
    shard_mask: u64,
    hasher: ahash::RandomState, // EINE Instanz, feste Seeds (§4(3))
}

pub struct KeyGuard<'a> { _guard: RwLockWriteGuard<'a, ()> }
pub struct MultiKeyGuard<'a> { _guards: Vec<RwLockWriteGuard<'a, ()>> }

#[derive(Debug, thiserror::Error)]
pub enum LockError {
    #[error("lock poisoned")] Poisoned,
    #[error("lock acquisition timed out")] Timeout,
    #[error("shard index out of range")] ShardOutOfRange,
}

impl KvKeyLocks {
    pub fn new(shard_count_pow2: u32) -> Self {
        let n = 1u64 << shard_count_pow2;
        Self {
            shards: (0..n).map(|_| RwLock::new(())).collect(),
            shard_mask: n - 1,
            hasher: ahash::RandomState::with_seeds(
                0x9E37_79B9_7F4A_7C15, 0xBF58_476D_1CE4_E5B9,
                0x94D0_49BB_1331_11EB, 0x2545_F491_4F6C_DD1D,
            ),
        }
    }

    /// Einziger zulässiger Weg, aus einem Schlüssel einen Lock-Hash zu erzeugen.
    pub fn key_hash<T: Hash + ?Sized>(&self, key: &T) -> u64 { self.hasher.hash_one(key) }

    fn shard_for(&self, key_hash: u64) -> usize { (key_hash & self.shard_mask) as usize }

    pub fn acquire(&self, key_hash: u64) -> Result<KeyGuard<'_>, LockError> {
        let shard = self.shards.get(self.shard_for(key_hash)).ok_or(LockError::ShardOutOfRange)?;
        Ok(KeyGuard { _guard: shard.write().map_err(|_| LockError::Poisoned)? })
    }

    /// H2-Pflichtmethode: Erwirbt N Shards STRIKT in aufsteigender Shard-Index-Reihenfolge.
    pub fn acquire_multi_sorted(&self, key_hashes: &[u64]) -> Result<MultiKeyGuard<'_>, LockError> {
        let mut idx: Vec<usize> = key_hashes.iter().map(|h| self.shard_for(*h)).collect();
        idx.sort_unstable();
        idx.dedup();
        let mut guards = Vec::with_capacity(idx.len());
        for i in idx {
            let shard = self.shards.get(i).ok_or(LockError::ShardOutOfRange)?;
            guards.push(shard.write().map_err(|_| LockError::Poisoned)?);
        }
        Ok(MultiKeyGuard { _guards: guards })
    }
}
```

**Testpflicht (AK-14):** `crates/memfuse-store/tests/kv_locks_stable_shard.rs` — `key_hash(&x)` ist für dieselbe
Instanz stabil, und zwei nebenläufige `acquire` auf dieselbe Entität schließen sich aus (Regression zu
`RandomState::new()` pro Aufruf).

**Loom-Testpflicht:** `crates/memfuse-store/tests/loom_multi_key_lock.rs` MUSS unter `#[cfg(loom)]` zwei
nebenläufige `acquire_multi_sorted`-Aufrufe mit überlappenden, unterschiedlich sortierten Schlüsselmengen
modellieren und deren Terminierung ohne Deadlock nachweisen.

### 5.3 WAL-Pipe: begrenzter MPSC-Group-Commit-Actor, `wal_flusher.rs` (normativ)

Die klassische Implementierung leidet unter geteilter Eigentümerschaft am File-Handle, was zu HMAC-Ketten-Forks
und stillen Datenverlusten führen kann. Zielarchitektur: **genau ein Eigentümer** des File-Handles, von `fsync`
und der Fortschreibung der HMAC-Kette (der Flusher-Task), viele Produzenten, eine **begrenzte** Queue.

**Änderung Fassung 2.1 (ersetzt den SPSC-Ring-Puffer der Fassung 2):** Die WAL hat mehrere Produzenten (jeder
schreibende Thread), ein SPSC-Ring passt dafür nicht. `Box<[MaybeUninit<WalEntry>]>` ist außerdem ohne `unsafe`
nicht lesbar, `memfuse-store` ist aber keine Unsafe-Insel (§4(2)). Und `WalEntry.hmac_prev` vom Produzenten zu
setzen, brächte den HMAC-Ketten-Fork zurück, den der Entwurf verhindern soll. Der Ist-Zustand im Repo ist bereits
ein MPSC-Actor mit `oneshot`-Ack nach `sync_all` und Group Commit, aber mit `unbounded_channel` (keine
Backpressure). Zielzustand: gleiche Struktur, begrenzte Queue.

**Vertrag (P2, Durability):** `append` kehrt erst **nach** dem `fsync` zurück, der den Eintrag enthält. „Entkoppelt"
ist nur die Wartezeit auf die Queue-Kapazität; die Latenz eines Commits umfasst immer mindestens einen `fsync`
(Group Commit amortisiert ihn über alle wartenden Einträge).

```rust
use bytes::Bytes;
use tokio::sync::{mpsc, oneshot};

pub const DEFAULT_WAL_QUEUE_CAPACITY: usize = 1_024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WalSeq(pub u64);

pub struct WalAppend {
    pub payload: Bytes, // OHNE hmac_prev: Sequenz und Kettenglied vergibt ausschließlich der Flusher
    pub ack: oneshot::Sender<Result<WalSeq, WalError>>,
}

pub enum WalCommand {
    Append(WalAppend),
    Seal { ack: oneshot::Sender<Result<(), WalError>> },
}

#[derive(Clone)]
pub struct WalHandle { tx: mpsc::Sender<WalCommand> }

pub struct WalFlusherConfig {
    pub queue_capacity: usize, // > 0, Default DEFAULT_WAL_QUEUE_CAPACITY
    pub max_batch_entries: usize,
    pub batch_window: std::time::Duration,
}

impl WalHandle {
    pub fn channel(queue_capacity: usize) -> Result<(WalHandle, mpsc::Receiver<WalCommand>), WalError> {
        if queue_capacity == 0 { return Err(WalError::InvalidCapacity); }
        let (tx, rx) = mpsc::channel(queue_capacity);
        Ok((WalHandle { tx }, rx))
    }

    /// Wartet bei voller Queue (Backpressure) und kehrt erst NACH fsync zurück.
    pub async fn append(&self, payload: Bytes) -> Result<WalSeq, WalError> {
        let (ack, rx) = oneshot::channel();
        self.tx.send(WalCommand::Append(WalAppend { payload, ack }))
            .await.map_err(|_| WalError::FlusherClosed)?;
        rx.await.map_err(|_| WalError::FlusherClosed)?
    }

    /// Nicht wartend: volle Queue ⇒ `Backpressure`.
    pub fn try_append(&self, payload: Bytes)
        -> Result<oneshot::Receiver<Result<WalSeq, WalError>>, WalError>
    {
        let (ack, rx) = oneshot::channel();
        match self.tx.try_send(WalCommand::Append(WalAppend { payload, ack })) {
            Ok(()) => Ok(rx),
            Err(mpsc::error::TrySendError::Full(_)) => Err(WalError::Backpressure),
            Err(mpsc::error::TrySendError::Closed(_)) => Err(WalError::FlusherClosed),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WalError {
    #[error("wal queue full (backpressure)")] Backpressure,
    #[error("wal flusher closed")] FlusherClosed,
    #[error("wal queue capacity must be > 0")] InvalidCapacity,
    #[error("hmac chain fork detected at sequence {0}")] HmacChainFork(u64),
}
```

**Flusher-Schleife (Körper gemäß Prosa):** Auf den ersten `recv().await` folgt ein Drain per `try_recv` bis
`max_batch_entries` oder Ablauf von `batch_window`; alle Einträge werden geschrieben, die Kette im Flusher
fortgeschrieben, **ein** `sync_data`/`sync_all` ausgeführt, danach werden alle `ack`s beantwortet. Ein Fehler
beantwortet alle Acks des Batches mit `Err`. Der Flusher ist der einzige Aufrufer von `fsync`.

**Deadlock-Regel (§4(4)):** Produzenten halten beim `send().await` keinen Lock, den der Flusher benötigt. Die
Kettenzustands-Mutex des Ist-Zustands (`last_hmac`, produzentenseitig) entfällt im Zielzustand, weil die Kette im
Flusher fortgeschrieben wird; bis dahin gilt: der Flusher greift nie auf diese Mutex zu.

**Testpflicht (AK-13):** `crates/memfuse-store/tests/wal_backpressure.rs` — bei voller Queue liefert `try_append`
`Backpressure`, `append` wartet; ein bestätigter `append` ist nach simuliertem Absturz (Fault-VFS) wiederherstellbar;
ein unbestätigter darf fehlen. `loom_group_commit.rs` (§15.3) bleibt bestehen.

**⚠️ Opus-Optimierung 0.1 — WAL-Replay-Panic entschärfen (Stufe 0, gering):**
Die Replay-Routine liest die Dateigröße einmalig vor dem `mmap`, prüft Zugriffsgrenzen aber gegen diesen separat
gehaltenen Wert statt gegen die tatsächliche Länge der gemappten Region. Maßnahme: Dateigröße ausschließlich aus
`mmap.len()` ableiten, alle Slice-Zugriffe auf `mmap.get(a..b)` mit `.ok_or(WalCorruption)` umstellen. Die zweite
parallele Scan-Implementierung auf denselben Hilfsfunktions-Pfad reduzieren.

**⚠️ Opus-Optimierung 0.5 — Recovery-Pfad differenzieren (Stufe 0, mittel):**
Den Intent-Datensatz um einen expliziten Ergebnisstatus (committed/aborted) erweitern und bei Repair-on-Open
auswerten, statt pauschal vorwärts zu committen.

### 5.4 Block-Cache: `BlockCacheBackend`-Trait (normativ)

```rust
pub trait BlockCacheBackend<K, V>: Send + Sync {
    fn get(&self, key: &K) -> Option<V>;
    fn insert(&self, key: K, value: V);
    fn capacity(&self) -> usize;
    fn len(&self) -> usize;
}
```

**`block_cache/lru.rs` (🟢 Produktions-Default):**

```rust
pub struct LruBlockCacheBackend<K, V> {
    inner: RwLock<lru::LruCache<K, V>>,
}

impl<K: Hash + Eq + Clone, V: Clone + Send + Sync> BlockCacheBackend<K, V>
    for LruBlockCacheBackend<K, V>
{
    fn get(&self, key: &K) -> Option<V> {
        // Cache-Hit erfordert Write-Lock, da LRU-Reordering mutiert (P25-Verstoß, dokumentiert als
        // bewusster Trade-off des Default-Pfads — siehe SieveCacheBackend für den lock-armen Pfad).
        // Poison-tolerant (§4(1)): ein Cache enthält nur rekonstruierbare Daten.
        self.inner.write().unwrap_or_else(std::sync::PoisonError::into_inner).get(key).cloned()
    }
    fn insert(&self, key: K, value: V) {
        self.inner.write().unwrap_or_else(std::sync::PoisonError::into_inner).put(key, value);
    }
    fn capacity(&self) -> usize {
        self.inner.read().unwrap_or_else(std::sync::PoisonError::into_inner).cap().get()
    }
    fn len(&self) -> usize {
        self.inner.read().unwrap_or_else(std::sync::PoisonError::into_inner).len()
    }
}
```

**`block_cache/quick_cache.rs` (🟡 Opt-in `block-cache-v2`, produktiv im Code vorhanden):**

Der tatsächlich implementierte lock-günstige Pfad wrappt den `quick_cache`-Crate (S3-FIFO-artige Eviction über
drei FIFO-Warteschlangen: Small ≈ 10 % Kapazität mit „Quick Demotion" für One-Hit-Wonders, Main, Ghost) als
`QuickCacheBlockCacheBackend`. Dies ist der Stand, der durch `FINAL_12` §0.3 am Code verifiziert ist — `LRU`
bleibt der Produktions-Default, `block-cache-v2` schaltet auf `QuickCacheBlockCacheBackend` um.

```rust
use quick_cache::sync::Cache as QuickCache;

pub struct QuickCacheBlockCacheBackend<K, V> {
    inner: QuickCache<K, V>,
}

impl<K, V> BlockCacheBackend<K, V> for QuickCacheBlockCacheBackend<K, V>
where
    K: std::hash::Hash + Eq + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    fn get(&self, key: &K) -> Option<V> { self.inner.get(key) }
    fn insert(&self, key: K, value: V) { self.inner.insert(key, value); }
    fn capacity(&self) -> usize { self.inner.capacity() as usize }
    fn len(&self) -> usize { self.inner.len() }
}
```

**`block_cache/sieve.rs` (🔴 Zielarchitektur, löst `QuickCacheBlockCacheBackend` perspektivisch ab):**

SIEVE verzichtet auf Listen-Neuordnung bei Lesetreffern: Ein Treffer setzt nur ein `visited`-Bit. Eviction läuft
über einen umlaufenden Zeiger („Hand"): gesetztes Bit ⇒ begnadigt (Bit gelöscht, bleibt im Cache), gelöschtes
Bit ⇒ verdrängt.

**Verbindliche Einordnung:** `quick_cache` bleibt die deklarierte Abhängigkeit und `QuickCacheBlockCacheBackend`
der Inhalt von `block-cache-v2`, bis `SieveCacheBackend` denselben Trait implementiert, denselben Loom- und
Benchmark-Nachweis wie `QuickCacheBlockCacheBackend` erbringt und per ADR als Ablösung beschlossen wird. Bis
dahin ist `SieveCacheBackend` ein Zielentwurf unter `#[cfg(feature = "block-cache-sieve-experimental")]`.

**Änderung Fassung 2.1 (ersetzt die `crossbeam-epoch`-Skizze):** Eine Liste aus `Atomic<SieveNode>` braucht
`unsafe` (`Shared::deref`), `memfuse-store` ist aber keine Unsafe-Insel (§4(2)); `Shared<'static, _>` ist zudem
`!Send` und `!Sync` (mit `crossbeam-epoch` 0.9 geprüft), der Trait verlangt `Send + Sync`. Sicherer Entwurf: Der
Index ist `scc::HashMap<K, Arc<SieveNode<V>>>`, `visited` liegt im Node, die Reihenfolge-Struktur (Slab mit
Index-Verkettung und `hand`) liegt unter einem `Mutex` und wird **nur im Miss-/Evict-Pfad** berührt. Ein Treffer
nimmt keinen Mutex. Er ist damit lock-arm, nicht wait-free (ein Bucket-Lesezugriff der `scc::HashMap`).
Byte-Kapazität: `size_bytes` im Node, `used_bytes` als `AtomicUsize`, Eviction bis `used_bytes ≤ capacity_bytes`.
Der Trait bleibt unverändert; `QuickCacheBlockCacheBackend` gewichtet im Repo bereits per `Weighter`.

```rust
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

pub struct SieveNode<V> {
    pub value: V,
    pub size_bytes: usize,
    visited: AtomicBool,
}

/// Index-verkettete Liste in einem Vec-Slab (safe Rust) plus `hand`; nur unter dem Mutex berührt.
struct SieveOrder<K> { slots: Vec<Option<K>>, hand: usize }

pub struct SieveCacheBackend<K, V> {
    index: scc::HashMap<K, Arc<SieveNode<V>>>,
    order: Mutex<SieveOrder<K>>,
    capacity_bytes: usize,
    used_bytes: AtomicUsize,
}

impl<K, V> BlockCacheBackend<K, V> for SieveCacheBackend<K, V>
where
    K: std::hash::Hash + Eq + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    fn get(&self, key: &K) -> Option<V> {
        // Hit-Pfad: kein Mutex, keine Listenmutation, ein Relaxed-Store.
        self.index.read(key, |_, node| {
            node.visited.store(true, Ordering::Relaxed);
            node.value.clone()
        })
    }
    fn insert(&self, key: K, value: V) { unimplemented!() } // Miss-Pfad: Mutex, Eviction bis used_bytes ≤ capacity_bytes
    fn capacity(&self) -> usize { self.capacity_bytes }
    fn len(&self) -> usize { self.index.len() }
}
```

**Nachweis vor Aktivierung:** Konformitätstest gegen einen Referenz-Simulator des SIEVE-Algorithmus (gleiche
Trefferfolge bei gleicher Zugriffsfolge), Loom-Test des Miss-Pfads, Benchmark gegen `QuickCacheBlockCacheBackend`.

**Sharding:** Alle Backends werden über ein `ShardedBlockCache<K, V, B: BlockCacheBackend<K,V>>` mit
konfigurierbarer Shard-Zahl (Default 16, `ahash`-basiertes Routing) gekapselt.

**⚠️ Opus-Optimierung 1.7 — Byte-basierte Cache-Kapazität (Stufe 1, mittel):**
Kapazität byte-basiert statt eintragsbasiert führen (Eviction anhand der tatsächlichen Bytegröße) — gilt für
`QuickCacheBlockCacheBackend` und die perspektivische `SieveCacheBackend` gleichermaßen.

### 5.5 Öffentliche Storage-API

```rust
pub struct LsmStore {
    wal: WalHandle,
    block_cache: Box<dyn BlockCacheBackend<BlockId, Bytes>>,
    kv_locks: KvKeyLocks,
}

impl LsmStore {
    pub fn get(&self, prefix: &str, key: &[u8]) -> Result<Option<Bytes>, StoreError>;
    pub fn put(&self, prefix: &str, key: &[u8], value: Bytes) -> Result<(), StoreError>;
    pub fn delete(&self, prefix: &str, key: &[u8]) -> Result<DeletionProof, StoreError>;
    pub fn scan_prefix(&self, prefix: &str) -> Result<impl Iterator<Item = (Vec<u8>, Bytes)>, StoreError>;
    pub fn compact(&self) -> Result<(), StoreError>;
    pub fn compact_async(&self, max_compaction_peak_memory_mb: usize) -> tokio::task::JoinHandle<Result<(), StoreError>>;
}

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error(transparent)] Wal(#[from] WalError),
    #[error(transparent)] Lock(#[from] LockError),
    #[error("compaction memory budget {budget_mb}MB exceeded (estimated {estimated_mb}MB)")]
    CompactionBudgetExceeded { budget_mb: usize, estimated_mb: usize },
    #[error(transparent)] Io(#[from] std::io::Error),
}
```

**⚠️ Opus-Optimierung 1.4 — SSTable Zero-Copy-Slice (Stufe 1, trivial):**
Beim Lesen eines Datenblocks nach CRC-Prüfung: referenzzählendes Slicing statt vollständiger Kopie.

**⚠️ Opus-Optimierung 2.4 — Manifest-Batch-Fsync (Stufe 2, mittel):**
Zusammengehörige Manifest-Änderungen in einem Batch, ein `fsync` pro Zustandsübergang statt pro Einzeleintrag.

**⚠️ Opus-Optimierung 1.6 — MemTable Range-Sharding (Stufe 1, hoch):**
Sharding-Grenzen aus dem Namensraum-Präfix ableiten, sodass Flush sortierfrei und Präfix-Scan auf eine Partition
beschränkt wird.

### 5.6 Compaction/MANIFEST-Atomarität (⚠️ Opus-Optimierung 0.6, Stufe 0, gering — STO-A)

**Einordnung:** Dies ist einer der fünf systemweit schwerwiegendsten Befunde des Architektur-Reviews, auf
derselben Prioritätsstufe wie der WAL-Replay-Panic-Fix (§5.3, Opus 0.1), und gehört ebenso in Stufe 0.

**Problem:** `maybe_compact()` schreibt Manifest-Änderungen nicht als einen atomaren Übergang, sondern als
Sequenz: (1) `manifest.append(Add { output_path })`, (2) In-Memory-Swap, (3) `manifest.append(Remove { old })` +
Löschen der alten Dateien — Fehler in (3) werden nur geloggt, nicht propagiert. Ein Absturz oder ein regulärer
Abbruch zwischen (1) und (3) hinterlässt ein MANIFEST, das die gemergte **und** alle Input-SSTables als gültig
führt. Beim Recovery werden beide geladen; bei einer vollständigen Kompaktierung verwirft der Merge Tombstones —
die alten SSTables enthalten die gelöschten Versionen jedoch noch. **Gelöschte Daten können nach einem Absturz
zurückkehren.** Für ein System mit kryptographischer `DeletionProof`-Zusicherung (Art. 17 DSGVO / Machine
Unlearning) ist das keine reine Storage-Performance-Frage, sondern ein Bruch des Sicherheitsversprechens aus §10.

Zusätzlich: Ein beschädigtes oder nicht ladbares MANIFEST degradiert aktuell still auf „lade jede `.sst`-Datei im
Verzeichnis" — ein Verstoß gegen die projektweite „No Silent Failures"-Doktrin (P4/P6) — und die Shadowing-
Reihenfolge zwischen Merge-Ausgabe und ihren Inputs wird nicht persistiert, sondern nach Recovery lexikografisch
neu geraten.

**Lösung (verbindlich):**

```rust
/// Ein Zustandsübergang = ein Record, ein fsync. Ersetzt die bisherige Add/Remove-Paar-Sequenz.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestEntry {
    pub kind: ManifestEntryKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ManifestEntryKind {
    /// Atomarer Kompaktierungs-Übergang: entfernte Dateien, neue Datei, Position (löst die
    /// Shadowing-Reihenfolge-Frage — die Position gehört in den Record, nicht in die Recovery-Heuristik).
    Replace { removed: Vec<PathBuf>, added: PathBuf, position: u32 },
    Add { path: PathBuf },
}

impl Manifest {
    /// Geschrieben und ge-fsynct NACH erfolgreichem Merge, VOR dem In-Memory-Swap.
    /// Ein halb geschriebener Record fällt über die CRC-Prüfung heraus — der Vorzustand gilt dann als aktuell.
    pub fn commit_replace(&self, removed: Vec<PathBuf>, added: PathBuf, position: u32)
        -> Result<(), ManifestError>;

    /// MUSS `Err` propagieren — kein Fallback auf Verzeichnis-Scan bei Ladefehler.
    pub fn load(path: &Path) -> Result<Vec<ManifestEntry>, ManifestError>;

    /// Rollover per `MANIFEST.new` + atomarem `rename` statt unbegrenztem Wachstum der Historie —
    /// Startzeit wird proportional zu den aktiven SSTables statt zur vollständigen Historie.
    pub fn rollover(&self) -> Result<(), ManifestError>;
}

#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
    #[error("manifest record failed CRC check at offset {0}, previous state retained")]
    CorruptRecord(u64),
    #[error(transparent)] Io(#[from] std::io::Error),
}
```

Kandidatenauswahl für Compaction wird zusätzlich in **einem** Lock-Fenster gelesen und verwendet (nicht über
drei getrennte Fenster hinweg), um den zugehörigen Out-of-Bounds-Panic-Pfad bei nebenläufiger Compaction
auszuschließen.

**Testpflicht:** `crates/memfuse-store/tests/manifest_crash_no_resurrection.rs` — simuliert einen Absturz nach
Schritt (1) im alten Modell (bzw. nach dem `commit_replace`-`fsync` im neuen Modell) und belegt, dass nach
Recovery keine Tombstone-Version aus den alten SSTables sichtbar wird.

---

<a id="6-graph"></a>
## 6. Wissensgraph-Datenmodell: binäre Kanten und n-äre Hyperkanten

### 6.1 Binäre Kanten als Grundmodell (🟢)

Der Wissensgraph wird primär als gerichteter, gewichteter Graph in einer CSR-Struktur gehalten:

```rust
#[derive(Debug, Clone)]
pub struct Edge {
    pub target: EntityId,
    pub weight: f32,
    pub edge_type: EdgeType,
    pub tx_valid_from: TxId,
    pub tx_valid_to: Option<TxId>,
    pub business_valid_from: Option<i64>,
    pub business_valid_to: Option<i64>,
    pub source_doc_id: Option<DocId>,
}
```

`EdgeType` ist als `#[non_exhaustive] enum { Default }` deklariert. Kanten tragen sowohl transaktionale (MVCC)
als auch fachliche (Business-Zeit) Gültigkeit — bi-temporal.

`DocId` (Default `u64`, feature-gated `u128` via BLAKE3-Truncation) und `EntityId` (`u64`) bilden die gemeinsame
Identitätsgrundlage:

```rust
#[cfg(not(feature = "docid-128"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DocId(pub u64);

#[cfg(feature = "docid-128")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(C, align(16))]
pub struct DocId(pub u128);

impl DocId {
    pub fn derive(collection_key: &[u8], seq: u64) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(collection_key);
        hasher.update(&seq.to_le_bytes());
        let digest = hasher.finalize();
        #[cfg(not(feature = "docid-128"))]
        { Self(u64::from_le_bytes(digest.as_bytes()[0..8].try_into().unwrap())) }
        #[cfg(feature = "docid-128")]
        { Self(u128::from_le_bytes(digest.as_bytes()[0..16].try_into().unwrap())) }
    }
}
```

### 6.2 Modulstruktur `memfuse-graph`

```
crates/memfuse-graph/src/
├── lib.rs
├── csr.rs             # CsrGraph, GraphInner, ArcSwap-RCU
├── edge.rs            # Edge, EdgeType-Nutzung, PersistedEdgePayload
├── hyperedge.rs       # HyperEdge, RoleBinding, RoleId, HyperEdgeId — 🔴
├── ppr.rs             # Forward-Push (Andersen-Chung-Lang)
├── path_rag.rs        # PathGraph, bidirektionale Suche, Hyperkanten-Expansion
├── community.rs       # Leiden, Stern-Expansion-Projektion
├── cascade.rs         # Cascade-Invalidierung binär + Hyperkanten
└── error.rs
```

### 6.3 RCU-Snapshot-Architektur: `csr.rs` (normativ)

**Änderungen Fassung 2.1:** (1) Hyperkanten-Payloads sind `Arc`-geteilt (§6.4): Der Klon in
`InnerWriteGuard::drop` (Ist-Zustand: tiefer Klon samt `Vec<RoleBinding>` jeder Hyperkante) kopiert dann nur
Referenzzähler. (2) Die Speicherschätzung rechnet **capacity-basiert** und liefert zusätzlich die **Spitze** des
Rebuilds. Der Ist-Zustand zählt nur `len()·size_of` und übersieht Tabellen-Overhead und Wachstumsspitze.
(3) Snapshot-Regel S1 (§4(7)): Der Snapshot ist unveränderlich, `scc::HashMap` gehört nicht in `GraphInner`.

**Restkosten (bewusst offen):** Ein Klon von `GraphInner` bleibt O(Anzahl Hyperkanten + Entitäten) an
Referenzzähler-Inkrementen und Tabellen-Allokation pro Veröffentlichung. Abhilfe ist Veröffentlichung pro
Schreib-Batch statt pro Einzelmutation oder eine persistente Map (strukturelles Sharing); beides ist nicht Teil
dieser Fassung.

```rust
use arc_swap::ArcSwap;
use ahash::AHashMap;
use std::mem::size_of;
use std::sync::Arc;

#[derive(Clone)] // sicher: jeder Klon erhöht nur Referenzzähler bzw. kopiert Tabellenstruktur, keine Payloads
pub struct GraphInner {
    pub adjacency: Vec<Vec<Edge>>,
    pub node_index: AHashMap<EntityId, usize>,
    // H1: Teil von GraphInner, NICHT separat — automatisch vom ArcSwap miterfasst.
    pub hyperedges: AHashMap<HyperEdgeId, Arc<HyperEdge>>,       // Payload geteilt (§6.4)
    pub hyperedge_index: AHashMap<EntityId, Arc<[HyperEdgeId]>>, // unveränderlicher Slice je Entität
    pub hyperedge_order: Arc<[HyperEdgeId]>,                     // aufsteigend sortiert (§4(3), §7.5)
}

pub struct MemoryEstimate { pub shared_payload_bytes: usize, pub private_bytes: usize }
impl MemoryEstimate {
    pub fn total(&self) -> usize { self.shared_payload_bytes.saturating_add(self.private_bytes) }
}

/// Obere Schranke für eine hashbrown-Tabelle: Buckets = nextpow2(⌊8·cap/7⌋ + 1),
/// je Bucket size_of::<(K, V)>() + 1 Kontrollbyte, plus 16 Byte Gruppenpuffer.
pub fn table_bytes<K, V>(capacity: usize) -> usize {
    if capacity == 0 { return 0; }
    let buckets = (capacity.saturating_mul(8) / 7 + 1).next_power_of_two().max(4);
    buckets.saturating_mul(size_of::<(K, V)>() + 1).saturating_add(16)
}

impl GraphInner {
    /// `presized = true`: Kapazität = len() (so MUSS ein Rebuild seine Maps und Vektoren anlegen).
    fn estimate(&self, presized: bool) -> MemoryEstimate {
        let cap = |len: usize, capacity: usize| if presized { len } else { capacity };
        let shared_payload_bytes: usize = self.hyperedges.values()
            .map(|h| size_of::<HyperEdge>() + h.participants.len() * size_of::<RoleBinding>()).sum::<usize>()
            + self.hyperedge_index.values().map(|v| v.len() * size_of::<HyperEdgeId>()).sum::<usize>();
        let adjacency_bytes = cap(self.adjacency.len(), self.adjacency.capacity()) * size_of::<Vec<Edge>>()
            + self.adjacency.iter().map(|v| cap(v.len(), v.capacity()) * size_of::<Edge>()).sum::<usize>();
        let private_bytes = adjacency_bytes
            + table_bytes::<EntityId, usize>(cap(self.node_index.len(), self.node_index.capacity()))
            + table_bytes::<HyperEdgeId, Arc<HyperEdge>>(cap(self.hyperedges.len(), self.hyperedges.capacity()))
            + table_bytes::<EntityId, Arc<[HyperEdgeId]>>(cap(self.hyperedge_index.len(), self.hyperedge_index.capacity()))
            + self.hyperedge_order.len() * size_of::<HyperEdgeId>();
        MemoryEstimate { shared_payload_bytes, private_bytes }
    }

    /// Residenz dieses Snapshots (H1: inkl. Hyperkanten, capacity-basiert).
    pub fn estimate_memory_bytes(&self) -> usize { self.estimate(false).total() }

    /// Spitze bei `compact()`: alter Snapshot bleibt bis zum Swap (und für laufende Leser) erreichbar,
    /// der neue wird daneben aufgebaut. Geteilte Payloads zählen einmal.
    pub fn estimate_compaction_peak_bytes(&self) -> usize {
        self.estimate(false).total().saturating_add(self.estimate(true).private_bytes)
    }
}

pub struct CsrGraph {
    inner: ArcSwap<GraphInner>,
    kv_locks: Arc<memfuse_store::KvKeyLocks>,
}

impl CsrGraph {
    /// Atomarer Snapshot-Austausch. Leser sehen NIE einen gemischten Alt-/Neu-Zustand.
    /// `rebuild` MUSS Maps mit `with_capacity(len)` vorbelegen (sonst gilt die Spitzenschätzung nicht).
    pub fn compact(&self) -> Result<(), GraphError> {
        let old = self.inner.load();
        let new_inner = Self::rebuild(&old)?;
        self.inner.store(Arc::new(new_inner));
        Ok(())
    }

    pub fn compact_async(&self, max_compaction_peak_memory_mb: usize)
        -> tokio::task::JoinHandle<Result<(), GraphError>>
    {
        // Budgetprüfung gegen die SPITZE, nicht gegen die Residenz (§4(5)).
        let estimated = self.inner.load().estimate_compaction_peak_bytes() / (1024 * 1024);
        if estimated > max_compaction_peak_memory_mb {
            return tokio::spawn(async move {
                Err(GraphError::CompactionBudgetExceeded {
                    budget_mb: max_compaction_peak_memory_mb,
                    estimated_mb: estimated,
                })
            });
        }
        unimplemented!()
    }

    pub fn neighbors_with_weights(&self, id: EntityId) -> Vec<(EntityId, f32)> { unimplemented!() }

    /// Sekundärindex-Zugriff, additiv — keine Kopie: liefert den geteilten Slice.
    pub fn hyperedges_for_entity(&self, id: EntityId) -> Option<Arc<[HyperEdgeId]>> {
        self.inner.load().hyperedge_index.get(&id).cloned()
    }
}
```

**Kosten des Indexes:** Der Slice je Entität ist unveränderlich; ein Einfügen ersetzt ihn durch einen neuen Slice
der Länge n+1 (O(Grad der Entität)). Für Hub-Entitäten mit sehr vielen Hyperkanten ist das der teuerste Teil des
Schreibpfads und in `binary_edge_regression.rs`/`hyperedge_memory_budget.rs` (AK-2, AK-8) mitzumessen.

**⚠️ Opus-Optimierung 2.1 — Inkrementelle Graph-Kompaktierung (Stufe 2, hoch):**
PPR-Pfad löst bei jeder Anfrage vollständigen CSR-Rebuild aus. Ziel: append-only Delta-Segmente plus
periodischer Merge im Hintergrund.

**⚠️ Opus-Optimierung 2.2 — CSR-Sentinel statt `Option` (Stufe 2, mittel):**
Mehrere Kantenspalten als `Vec<Option<T>>` — ca. Halbierung des Speicherbedarfs pro Kante durch Sentinel-Werte.

### 6.4 Hyperkanten: Datenstruktur (🔴 vollständig spezifiziert)

**Designentscheidung (verbindlich):** Es wird **kein** generisches RDF-Reifikations-Pattern verwendet.
Stattdessen wird eine kohärente, erstklassige Rust-Struktur mit Zero-Copy-Deserialisierung via FlatBuffers/Mmap
spezifiziert. **Zwei Repräsentationen, ein Persistenzformat:** Der Schreibpfad (`relate_n_ary`) konstruiert eine
neue Hyperkante ohnehin aus frisch übergebenen Daten — dort ist eine besitzende Struktur korrekt und einfach.
Der Lesepfad (`hyperedges_for_entity`/Traversal) dereferenziert dagegen bei **jeder** Anfrage potenziell
tausende bereits persistierter Hyperkanten aus dem RCU-Snapshot; hier erzwingt eine besitzende `Vec<RoleBinding>`
pro gelesener Hyperkante eine Heap-Kopie, obwohl die Daten bereits deserialisiert im Snapshot-Speicher liegen.
Die Lesesicht referenziert diesen Speicher stattdessen zero-copy:

```rust
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HyperEdgeId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoleId(pub u32);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoleBinding {
    pub role: RoleId,
    pub entity: EntityId,
}

/// Referenzzählender Slice auf einen zusammenhängenden `RoleBinding`-Bereich innerhalb des
/// Mmap-/RCU-Snapshot-Speichers. Hält nur Pointer + Länge + geteilten Referenzzähler auf den
/// zugrundeliegenden `Arc`-Puffer — kein `Vec`-Allocation-Overhead beim Lesen.
#[derive(Clone)]
pub struct ArcSlice<T> {
    backing: Arc<[T]>,
    start: u32,
    len: u32,
}

impl<T> ArcSlice<T> {
    /// Ganze Sicht auf einen geteilten Puffer — nur Referenzzähler-Inkrement, keine Kopie.
    pub fn whole(backing: Arc<[T]>) -> Self {
        let len = u32::try_from(backing.len()).unwrap_or(u32::MAX);
        Self { backing, start: 0, len }
    }
    /// Validierter Teilbereich; `None` bei ungültigem Bereich (kein Panic, §4(1)).
    pub fn new(backing: Arc<[T]>, start: u32, len: u32) -> Option<Self> {
        let end = (start as usize).checked_add(len as usize)?;
        backing.get(start as usize..end)?;
        Some(Self { backing, start, len })
    }
}

impl<T> std::ops::Deref for ArcSlice<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        let start = self.start as usize;
        let end = start.saturating_add(self.len as usize);
        self.backing.get(start..end).unwrap_or(&[]) // Konstruktoren validieren; hier defensiv statt Indexierung
    }
}

/// Snapshot-Variante. `participants` ist ein geteilter Puffer (`Arc<[RoleBinding]>`): Klone des Snapshots und
/// Lesesichten teilen ihn, statt ihn zu kopieren (§4(7)). serde benötigt das Feature `rc`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperEdge {
    pub id: HyperEdgeId,
    pub predicate: EdgeType,
    pub participants: Arc<[RoleBinding]>,       // min. 2, validiert in `relate_n_ary`; size_of::<RoleBinding>() = 16
    pub weight: f32,
    pub tx_valid_from: Option<TxId>,
    pub tx_valid_to: Option<TxId>,
    pub business_valid_from: Option<i64>,
    pub business_valid_to: Option<i64>,
    pub source_doc_id: Option<DocId>,
}

/// Zero-Copy-Lesesicht — Traversal-Hotpath (`hyperedges_for_entity`, PPR-Expansion, §7.2).
/// `participants` referenziert den RCU-Snapshot-Speicher direkt statt ihn zu kopieren.
#[derive(Clone)]
pub struct HyperEdgeView {
    pub id: HyperEdgeId,
    pub predicate: EdgeType,
    pub participants: ArcSlice<RoleBinding>,
    pub weight: f32,
    pub source_doc_id: Option<DocId>,
}

impl HyperEdge {
    /// Erzeugt die Zero-Copy-Sicht ohne die `participants` zu kopieren — teilt sich den `Arc`
    /// mit dem im `GraphInner` gehaltenen Original (siehe §6.3, H1).
    pub fn as_view(&self) -> HyperEdgeView {
        HyperEdgeView {
            id: self.id,
            predicate: self.predicate,
            // Fassung 2 kopierte hier (`Arc::from(self.participants.as_slice())`) — jetzt Referenzzähler-Inkrement.
            participants: ArcSlice::whole(Arc::clone(&self.participants)),
            weight: self.weight,
            source_doc_id: self.source_doc_id,
        }
    }
}

/// Rollen-Interner — dieselbe Interning-Strategie wie EdgeType/Prädikate.
pub struct RoleInterner {
    forward: scc::HashMap<String, RoleId>,
    backward: scc::HashMap<RoleId, String>,
    next_id: std::sync::atomic::AtomicU32,
}

impl RoleInterner {
    pub fn intern(&self, name: &str) -> RoleId;
    pub fn resolve(&self, id: RoleId) -> Option<String>;
}
```

**Konsequenz für `GraphInner` (§6.3, H1):** `hyperedges: AHashMap<HyperEdgeId, Arc<HyperEdge>>` und
`hyperedge_index: AHashMap<EntityId, Arc<[HyperEdgeId]>>` — das `Arc` auf `HyperEdge` und das `Arc<[RoleBinding]>`
darin machen `as_view()` und den Snapshot-Klon zero-copy-fähig (Test AK-9), da mehrere `HyperEdgeView`s denselben
Teilnehmer-Speicher teilen können, ohne dass ihre Lebensdauer an eine geliehene Referenz auf `GraphInner` gebunden ist
(wichtig, weil `GraphInner` selbst per `ArcSwap` ausgetauscht wird, siehe H1-Lösung unten).

**Persistenz:** Neues LSM-Präfix `__graph:hyperedge:`, Value = FlatBuffers-serialisiertes `HyperEdge`.
Sekundärindex `__graph:hyperedge_by_entity:{EntityId} -> Vec<HyperEdgeId>`. Additiv — kein Breaking Change.

**API-Oberfläche:**
```rust
impl Collection {
    /// Binärer Pfad — bleibt Hotpath, KEINE interne Umleitung auf `relate_n_ary`.
    pub fn relate(&self, from: EntityId, to: EntityId, predicate: EdgeType, doc_id: DocId) -> Result<(), DbError>;

    /// NEU — Hyperkanten-Schreibpfad.
    pub fn relate_n_ary(
        &self,
        predicate: EdgeType,
        participants: &[(RoleId, EntityId)],
        doc_id: DocId,
    ) -> Result<HyperEdgeId, DbError>;
}
```

### 6.5 Traversal-Semantik

PathRAG wird um einen optionalen Hyperkanten-Expansionsschritt ergänzt: Beim Erreichen eines Knotens während der
Forward-Push-Traversierung werden zusätzlich alle `RoleBinding`-Partner als „virtuelle" Nachbarn mit
rollenspezifischem Gewichtsabschlag (Startwert `0.85`) eingespeist.

### 6.6 Integrationshindernisse H1–H6 und ihre verbindliche Lösung

Diese sechs Hindernisse benennen nicht nur, *dass* eine Integration möglich ist, sondern *welches bestehende
Invariant unter Druck gerät* und wie es gewahrt bleibt.

#### H1 — RCU-Snapshot-Inkonsistenz zwischen CSR und Hyperkanten-Sekundärindex

**Problem:** Separater Hyperkanten-Index außerhalb von `GraphInner` → Zeitfenster für inkonsistenten Zustand.

**Lösung (verbindlich):** Der Hyperkanten-Index wird **Teil von `GraphInner`** selbst — automatisch vom
`ArcSwap`-Swap miterfasst. `GraphInner::estimate_memory_bytes()` MUSS Hyperkanten einschließen und capacity-basiert rechnen; die Budgetprüfung
von `compact_async` verwendet `estimate_compaction_peak_bytes()` (§6.3, Snapshot-Regel S1: kein `scc::HashMap` im Snapshot).

#### H2 — Kanonisches Multi-Key-Locking zur Deadlock-Prävention

**Problem:** `relate_n_ary()` mit N Teilnehmern muss N Entitäten gleichzeitig unter `kv_locks` halten.
Naives Lock-Ordering → Deadlock bei überlappenden, unterschiedlich sortierten Mengen.

**Lösung (verbindlich):**

```rust
impl CsrGraph {
    pub fn relate_n_ary(
        &self,
        predicate: EdgeType,
        participants: &[RoleBinding],
        doc_id: DocId,
    ) -> Result<HyperEdgeId, GraphMutationError> {
        if participants.len() < 2 {
            return Err(GraphMutationError::InsufficientParticipants(participants.len()));
        }

        // 1. Kanonische Sortierung (H2) — Grundlage der Deadlockfreiheit.
        let mut entities: Vec<EntityId> = participants.iter().map(|p| p.entity).collect();
        entities.sort_unstable_by_key(|e| e.0);
        entities.dedup();
        // Hash ausschließlich über KvKeyLocks (feste Seeds, eine Instanz; §5.2a). NIE `RandomState::new()` hier.
        let key_hashes: Vec<u64> = entities.iter().map(|e| self.kv_locks.key_hash(e)).collect();

        // 2. Multi-Key-Lock in sortierter Shard-Reihenfolge.
        let _guards = self.kv_locks.acquire_multi_sorted(&key_hashes)
            .map_err(|_| GraphMutationError::LockAcquisitionTimeout)?;

        // 3. Atomare LSM-Schreibung: Primär + Sekundärindex für JEDEN Teilnehmer.
        let id = HyperEdgeId(self.next_hyperedge_id());
        let hyperedge = HyperEdge {
            id, predicate, participants: Arc::from(participants), weight: 1.0,
            tx_valid_from: None, tx_valid_to: None,
            business_valid_from: None, business_valid_to: None,
            source_doc_id: Some(doc_id),
        };
        self.persist_hyperedge_atomic(&hyperedge)?;

        // 4. RCU-Registrierung (H1).
        self.register_in_rcu_snapshot(&hyperedge)?;

        Ok(id)
    }
}
```

**Loom-Testpflicht (AK-3):** `crates/memfuse-graph/tests/loom_relate_n_ary.rs` mit überlappenden,
unterschiedlich geordneten Mengen. **Shard-Stabilität (AK-14, Fassung 2.1):** siehe §5.2a — dieselbe Entität muss in
jedem Aufruf denselben Shard treffen, sonst ist der Ausschluss wirkungslos.

#### H3 — `SignalKind` ist ein geschlossenes Enum

**Lösung (verbindlich):** Hyperkanten-Treffer fließen als zusätzliche Kandidaten **in `SignalKind::Graph`** ein —
PathRAG liefert bereits ein Graph-Signal; Hyperkanten-Expansion ist ein interner Erweiterungsschritt der Pfadsuche.
`SignalKind` bleibt strukturell unverändert. Diff-Test-Pflicht: `signal_kind_no_new_variant.rs`.

#### H4 — FlatBuffers-Schemaerweiterung erfordert aktives CI-Drift-Gate

**Lösung (verbindlich, harte Vorbedingung):** Das FlatBuffers-CI-Drift-Gate MUSS produktiv und grün sein,
**bevor** das `HyperEdge`-FlatBuffers-Schema gemerged wird — per CI-Job-Abhängigkeit erzwungen (`needs: [flatbuffers-drift-gate]`).

#### H5 — Cascade-Invalidierung: hartes Fan-out-Limit, persistente idempotente Queue

**Änderung Fassung 2.1:** Fassung 2 gab im Überlauffall `Err(PartialCascadeQueued(proof))` zurück und ließ die
Hintergrundqueue offen. Ein Teilerfolg ist kein Fehler, und ein „Beweis" vor Abschluss der Löschung ist irreführend.
Ist-Zustand im Repo: Die zurückgestellten Hyperkanten liegen nur in einer In-Memory-`VecDeque`; außerhalb von Tests
habe ich keinen `enqueue`-Aufruf gefunden, sie gehen bei einem Absturz verloren oder werden nie befüllt.

**Lösung (verbindlich, Pflichtbestandteil):**

1. **Ein atomarer WAL-Commit** enthält die synchronen Tombstones (die ersten `θ` Hyperkanten in aufsteigender
   `HyperEdgeId`-Reihenfolge, §4(3)) **und** die Queue-Einträge für den Rest. Es gibt keinen Zustand „Rest weder
   tombstoniert noch in der Queue".
2. **Persistente Queue:** LSM-Präfix `__graph:cascade_queue:{doc_id_be}:{hyperedge_id_be}` (Big-Endian, damit der
   Präfix-Scan je Dokument geordnet ist). Ein `put` auf denselben Schlüssel ist idempotent.
3. **Worker** (Start beim Öffnen und bei Benachrichtigung): Präfix-Scan, Batches von `CASCADE_BATCH = 128`. Je
   Hyperkante: `tombstone_hyperedge_atomic(id)` **und** Löschen des Queue-Schlüssels in **einer** Transaktion; Locks
   nach H2 nur für die Dauer eines Batches, nie über Batches hinweg (RCU-Lesepfad bleibt frei).
4. **Idempotenz:** `tombstone_hyperedge_atomic` setzt `tx_valid_to` nur, wenn es `None` ist, und schreibt sonst
   nichts. Ein wiederholter Lauf auf dasselbe `doc_id` erzeugt keine doppelten Tombstones.
5. **`DeletionProof`** wird erst ausgestellt, wenn für das `doc_id` kein Queue-Eintrag mehr existiert: im Report
   (`queued_for_background == 0`) oder beim Abschluss durch den Worker (persistiert unter
   `__graph:cascade_proof:{doc_id_be}`, abrufbar über `cascade_status`). Nie für eine unvollständige Löschung.
6. **Fehlersemantik:** `Ok(CascadeReport)` auch bei Teilverarbeitung; `Err` nur für echte Fehler (Lock, I/O,
   Budget). Die Variante `GraphMutationError::PartialCascadeQueued` entfällt (§6.7).

```rust
pub const DEFAULT_HYPEREDGE_CASCADE_FANOUT_LIMIT: usize = 1_000;
pub const CASCADE_BATCH: usize = 128;

pub struct CascadeTicket(pub u64);

pub struct CascadeReport {
    pub tombstoned_synchronously: usize,
    pub queued_for_background: usize,
    pub ticket: Option<CascadeTicket>,                       // Some, solange queued_for_background > 0
    pub deletion_proof: Option<memfuse_crypto::DeletionProof>, // nur bei vollständiger Löschung
}

pub struct CascadeStatus {
    pub pending: usize,
    pub deletion_proof: Option<memfuse_crypto::DeletionProof>,
}

pub fn cascade_invalidate_hyperedges_for_superseded_doc(
    graph: &CsrGraph,
    doc_id: DocId,
    fanout_limit: usize,
) -> Result<CascadeReport, GraphMutationError> {
    let mut affected = graph.hyperedges_for_doc(doc_id);
    affected.sort_unstable();                          // deterministisch (§4(3))
    let split = affected.len().min(fanout_limit);      // split ≤ len: `split_at` kann nicht paniken
    let (sync_part, async_part) = affected.split_at(split);

    let mut tx = graph.begin_cascade_tx(doc_id)?;      // EIN WAL-Commit für beide Teile
    for id in sync_part { tx.tombstone(*id)?; }        // idempotent
    for id in async_part { tx.enqueue(doc_id, *id)?; } // idempotent (put auf festen Schlüssel)
    tx.commit()?;

    let queued = async_part.len();
    Ok(CascadeReport {
        tombstoned_synchronously: sync_part.len(),
        queued_for_background: queued,
        ticket: (queued > 0).then(|| graph.cascade_ticket(doc_id)),
        deletion_proof: if queued == 0 { Some(graph.issue_deletion_proof(doc_id)?) } else { None },
    })
}

pub fn cascade_status(graph: &CsrGraph, doc_id: DocId) -> Result<CascadeStatus, GraphMutationError> {
    unimplemented!() // pending = Anzahl Queue-Einträge; Proof aus __graph:cascade_proof:{doc_id_be}, falls vorhanden
}
```

**Testpflicht (AK-6, AK-11):** `hyperedge_cascade_fanout.rs` (Umschalten bei > θ) und
`hyperedge_cascade_crash_recovery.rs`: Absturz nach dem Commit und vor dem ersten Worker-Batch, Neustart, danach
sind alle Hyperkanten des Dokuments genau einmal tombstoniert und die Queue ist leer; zweiter Lauf ist ein No-op.

#### H6 — Community-Detection/Leiden sieht Hyperkanten nicht

**Lösung (verbindlich für Sichtbarkeit):**

```rust
pub struct CommunityDetectionConfig {
    pub resolution_gamma: f32,
    /// H6: sichtbares Unvollständigkeits-Flag, Default false.
    pub hyperedges_included: bool,
}

pub struct CommunityAssignment {
    pub node_to_community: AHashMap<EntityId, u32>,
    pub hyperedges_included: bool, // 1:1 aus Config, im Report sichtbar
}
```

**Stern-Expansion als Zielarchitektur:** Der Hypergraph wird in einen bipartiten Graphen überführt. Jede
Hyperkante wird als künstlicher Knoten `v_e` repräsentiert; es entstehen nur binäre Kanten mit $O(|e|)$
Skalierung (statt $O(|e|^2)$ bei Cliquen-Expansion). `StarExpansionIterator` (§7.5) erzeugt die virtuellen Kanten
on-the-fly, ohne die Hyperkanten zu klonen.

**Konvention K (verbindlich ab Fassung 2.1).** Sei $N = |e| \ge 2$ und $w = w(e)$.

- *Referenz-Clique* $K(e)$: jedes ungeordnete Teilnehmerpaar erhält $p(e) = w / \binom{N}{2}$. Die Gesamtmasse
  der Hyperkante ist damit $w$, unabhängig von $N$ (große Hyperkanten dominieren nicht). Für $N = 2$ ergibt sich
  die binäre Kante mit Gewicht $w$.
- *Sternkante* je Teilnehmer $u \leftrightarrow v_e$: $a(e) = N \cdot p(e) = \dfrac{2\,w(e)}{|e| - 1}$
  (`star_weight`, siehe unten).

**Satz (Schur-Komplement).** Eliminiert man $v_e$ aus der Laplace-Matrix des Sterns, entsteht exakt die
Laplace-Matrix von $K(e)$.
*Beweis.* Teilnehmerblock $L_{pp} = aI$, Kopplung $L_{pv} = -a\mathbf 1$, $L_{vv} = Na$. Schur-Komplement:
$L_{pp} - L_{pv}L_{vv}^{-1}L_{vp} = aI - \tfrac{a}{N}\mathbf 1\mathbf 1^{\top}$. Die Clique mit Paargewicht $p$ hat
$p(NI - \mathbf 1\mathbf 1^{\top})$. Beide sind gleich genau dann, wenn $a = Np$. $\square$
Numerisch geprüft für $N \in \{2,3,5,8,20\}$ (maximale Abweichung $\approx 10^{-16}$).

**Was der Satz nicht besagt:** Er gilt für Laplace-basierte Größen (effektive Leitfähigkeit, Diffusion, PPR-artige
Prozesse). Die **Modularität** $Q$ auf dem Sterngraphen ist nicht identisch mit $Q$ auf der Clique, weil `v_e`
Grad trägt (Grad $= N\cdot a$). Der Resolution-Parameter $\gamma$ bleibt tuning-pflichtig, `hyperedges_included`
bleibt `false`, bis die Validierung gegen Benchmark-Netzwerke abgeschlossen ist.

**Korrektur gegenüber Fassung 2:** Dort stand $a = w/(|e|-1)$ mit dem „Beweis", die Teilnehmergradsumme des Sterns
$|e|\,w/(|e|-1)$ entspreche der Clique. Bei Paargewicht $w$ hat die Clique aber die Gradsumme $|e|(|e|-1)\,w$
(N=3: $1{,}5\,w$ gegen $6\,w$); gleich sind beide nur bei $|e|=2$. Die alte Formel entspricht Konvention K mit
halbem Hyperkantengewicht. Der Ist-Zustand im Repo verwendet $a = w$ ohne Normierung. **Umstellung:** Konvention K
ist eine Modellierungsentscheidung (§A2.4 Nr. 6). Wer die Größenabhängigkeit der Masse will, wählt die
Zhou-Konvention $p = w/(|e|-1)$, dann ist $a = N\,w/(|e|-1)$; in beiden Fällen gilt $a = N p$, und der Test
prüft gegen die konfigurierte Konvention.

```rust
/// Konvention K: a(e) = 2·w(e)/(|e|−1). `None` bei |e| < 2 oder nicht endlichem Gewicht.
pub fn star_weight(w: f32, n: usize) -> Option<f32> {
    if n < 2 || !w.is_finite() { None } else { Some(2.0 * w / (n as f32 - 1.0)) }
}
```

**Testpflicht (AK-10):** `crates/memfuse-graph/tests/star_expansion_equals_clique.rs` — (a) für zufällige
Hyperkanten ($N \in [2, 64]$) stimmt das Schur-Komplement des Sterns mit der Clique-Laplace-Matrix überein
(relative Toleranz $10^{-9}$, f64-Referenz), (b) $N = 2$ ergibt die binäre Kante, (c) die Iterationsreihenfolge ist
deterministisch, (d) der Iterator klont keine Hyperkante (Allokationszähler).

### 6.7 Hyperkanten-Fehler-Enum

```rust
#[derive(Debug, thiserror::Error)]
pub enum GraphMutationError {
    #[error("lock acquisition timed out")]
    LockAcquisitionTimeout,
    // Fassung 2.1: `PartialCascadeQueued` entfällt. Teilverarbeitung ist `Ok(CascadeReport)` (§6.6 H5).
    #[error("role binding invalid: {0}")]
    RoleBindingInvalid(String),
    #[error("rcu snapshot reclamation pending, retry")]
    EpochReclamationPending,
    #[error("hyperedge requires >= 2 participants, got {0}")]
    InsufficientParticipants(usize),
}
```

---

<a id="7-retrieval"></a>
## 7. Retrieval-Pipeline: 4-Signal-Fusion und ihre Algorithmen

### 7.1 4-Signal-Fusion

Jede Hybridsuche kombiniert bis zu vier unabhängige Signale — Vektor (HNSW-k-NN), Text (BM25/BM25F), Graph
(PPR-Traversierung inkl. Hyperkanten-Erweiterung), optional Kanten-Reinforcement (feature-gated) — über das
geschlossene `SignalKind`-Enum:

```rust
/// BEWUSST NICHT `#[non_exhaustive]` — Erweiterung erfolgt NIEMALS durch neue Varianten (H3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignalKind {
    Vector,
    Text,
    Graph,
    EdgeReinforcement, // feature-gated
}

impl SignalKind {
    /// Allokationsfrei, `eq_ignore_ascii_case` statt `to_lowercase()`-Allokation.
    pub fn from_name(name: &str) -> Option<Self> {
        if name.eq_ignore_ascii_case("vector") { Some(Self::Vector) }
        else if name.eq_ignore_ascii_case("text") { Some(Self::Text) }
        else if name.eq_ignore_ascii_case("graph") { Some(Self::Graph) }
        else if name.eq_ignore_ascii_case("edgereinforcement") { Some(Self::EdgeReinforcement) }
        else { None }
    }
}
```

Die Fusion erfolgt standardmäßig über **Reciprocal Rank Fusion (RRF)**, score-blind und robust bei
Signalausfall. Score-normalisierte Fusion als Opt-in mit hartem RRF-Fallback.

**⚠️ Opus-Optimierung 1.10 — Top-k-Selektion (Stufe 1, gering):**
Volle Sortierung durch begrenzte Selektion in linearer Zeit ersetzen.

### 7.2 Graph-Signal: Forward-Push-PPR (🟢)

**Andersen-Chung-Lang Forward-Push-Algorithmus.** Exploriert nur Knoten, die signifikant zur PageRank-Masse beitragen.

Initialisierung für Seed-Knoten $s$: $r(s) = 1$, $p(s) = 0$. Für jeden Knoten $u$ mit
$\frac{r(u)}{d(u)} > \epsilon$:

1. $p(u) \leftarrow p(u) + \alpha \cdot r(u)$
2. $r(u) \leftarrow (1 - \alpha) \frac{r(u)}{2}$
3. $r(v) \leftarrow r(v) + (1 - \alpha) \frac{r(u)}{2\, d(u)}$ für alle Nachbarn $v$

Laufzeit: $O\!\left(\frac{1}{\alpha \epsilon}\right)$ — **unabhängig von der Gesamtgröße des Graphen** (P24).

```rust
pub struct PprParams {
    pub alpha: f32,           // Teleport-Wahrscheinlichkeit
    pub epsilon: f32,         // Fehlertoleranz-Schwellenwert
    pub hyperedge_decay: f32, // Default 0.85
}

pub fn forward_push_ppr(
    graph: &CsrGraph,
    seeds: &[EntityId],
    params: &PprParams,
) -> AHashMap<EntityId, f32> {
    let mut p: AHashMap<EntityId, f32> = AHashMap::new();
    let mut r: AHashMap<EntityId, f32> = seeds.iter()
        .map(|&s| (s, 1.0 / seeds.len() as f32)).collect();
    let mut queue: std::collections::VecDeque<EntityId> = seeds.iter().copied().collect();

    while let Some(u) = queue.pop_front() {
        let degree = graph.degree(u).max(1) as f32;
        let r_u = *r.get(&u).unwrap_or(&0.0);
        if r_u / degree <= params.epsilon { continue; }

        *p.entry(u).or_insert(0.0) += params.alpha * r_u;
        let residual_kept = (1.0 - params.alpha) * r_u / 2.0;
        r.insert(u, residual_kept);

        let push_share = (1.0 - params.alpha) * r_u / (2.0 * degree);

        // Binäre Nachbarn (unveränderter Hotpath).
        for (v, _w) in graph.neighbors_with_weights(u) {
            *r.entry(v).or_insert(0.0) += push_share;
            queue.push_back(v);
        }

        // 🔴 NEU (H3): Hyperkanten-Partner als virtuelle Nachbarn, Gewichtsabschlag.
        for hedge_id in graph.hyperedges_for_entity(u) {
            for role_binding in graph.hyperedge_participants(hedge_id) {
                if role_binding.entity == u { continue; }
                *r.entry(role_binding.entity).or_insert(0.0) += push_share * params.hyperedge_decay;
                queue.push_back(role_binding.entity);
            }
        }
    }
    p
}
```

### 7.3 Volltextsuche: BM25 mit Block-Max WAND und BM25F (🟢)

```rust
pub struct ResidentPostingIndex {
    postings: AHashMap<TermId, PostingList>,
    doc_lengths: Vec<u32>,
    field_lengths: AHashMap<(DocId, FieldId), u32>, // BM25F-Voraussetzung
}

pub struct Bm25fParams {
    pub k1: f32,
    pub b: f32,
    pub field_weights: AHashMap<FieldId, f32>,
}

impl ResidentPostingIndex {
    /// Block-Max WAND: Top-k ohne vollständige Postinglisten-Traversierung.
    pub fn search_topk(&self, query_terms: &[TermId], k: usize, params: &Bm25fParams) -> Vec<(DocId, f32)>;

    /// BM25F-Score für ein einzelnes Dokument, feldgewichtet.
    fn bm25f_score(&self, doc: DocId, terms: &[TermId], params: &Bm25fParams) -> f32;
}

/// Deutsche Kompositazerlegung.
pub fn decompose_german_compound(word: &str, dictionary: &CompoundDictionary) -> Vec<String>;
```

Persistenz: Residenter Index wird beim Start aus LSM-Präfix `__text:posting:` materialisiert.

**⚠️ Opus-Optimierung 1.9 — Text-Posting-Format (Stufe 1, hoch):**
Umstellung von Einzelschlüssel- auf Listenspeicherung. Delta-kodierte Dokument-IDs. Ein Lesezugriff
pro Suchbegriff statt unbegrenztem Präfix-Scan.

### 7.4 Vektorindex: HNSW + DiskANN (🟢)

```rust
pub struct HnswIndex<const D: usize> {
    layers: Vec<HnswLayer<D>>,
    entry_point: AtomicUsize,
    sq8_codebook: Sq8Codebook,
}

pub struct Sq8Codebook {
    pub min: [f32; D_MAX],
    pub max: [f32; D_MAX],
    pub clip_percentile: f32, // Default 0.999
}

impl<const D: usize> HnswIndex<D> {
    pub fn search_knn(&self, query: &[f32; D], k: usize, ef_search: usize) -> Vec<(DocId, f32)>;
    pub fn insert(&mut self, id: DocId, vector: [f32; D]) -> Result<(), IndexError>;
    pub fn delete(&mut self, id: DocId) -> Result<(), IndexError>; // native Tombstone
}
```

**NaN-sichere Distanz-Pipeline:**
```rust
/// Bitweise SIMD-Maskierung statt Branch: NaN → f32::INFINITY.
#[inline]
fn masked_l2_distance_avx512(a: &[f32], b: &[f32]) -> f32 {
    // SAFETY: `a`/`b` sind 64-Byte-aligned und exakt D Elemente lang.
    unsafe { unimplemented!() }
}
```

**DiskANN (🟢 offizieller Tier):**
```rust
pub struct DiskAnnIndex<const D: usize> {
    mmap: memmap2::Mmap,
    tombstones: scc::HashSet<DocId>, // native, kein HNSW-Fallback nötig
    tombstone_wal: TombstoneWal,
}

pub enum VectorIndexTier {
    Hnsw,
    #[cfg(feature = "experimental-diskann")]
    DiskAnn,
}
```

**Zielarchitektur „HNSW v2" (🔴 Arena-Allocator):**
```rust
pub struct HnswArena<const D: usize> {
    storage: std::sync::Arc<MmapArena>,
    head: crossbeam_epoch::Atomic<NodeRecord<D>>,
    capacity: usize,
}

pub struct NodeRecord<const D: usize> {
    pub vector: [f32; D],
    pub neighbor_offsets: [u32; MAX_M],   // Offsets statt Pointer
    pub neighbor_count: u16,
}

impl<const D: usize> HnswArena<D> {
    /// Relinking über CAS statt Mutex.
    pub fn relink(&self, node_offset: u32, new_neighbors: &[u32]) -> Result<(), IndexError>;
}
```

**SQ8-Bias-Kalibrierung (ab Fassung 2.1).** SQ8 quantisiert je Dimension mit Schrittweite
$\Delta_d = (\max_d - \min_d)/255$ und rundet auf die nächste Stufe; Werte außerhalb des Perzentil-Bereichs werden
geclippt. Das verschiebt die Distanzen **systematisch**. Es betrifft nur **absolute Schwellen** (kalibrierte
Score-Schwellen in `memfuse-rank`), nicht das Ranking, weil der Offset im Mittel für alle Kandidaten gleich ist.

- Der Rundungsanteil der Fehlerenergie ist $c_q = \sum_d \Delta_d^2/12$ (`rounding_energy`, Diagnosewert;
  gemessen 4,88e-5 gegen berechnet 4,89e-5 auf synthetischen Daten mit $D=384$).
- Der **Netto-Offset** auf Normen und Distanzen ist **nicht** einfach $+c_q$. Randeffekte an den Bereichsgrenzen
  und vor allem das Perzentil-Clipping können ihn dominieren und umkehren (synthetisch, Gauß, 99,9-%-Clipping:
  etwa $-75\,c_q$, negatives Vorzeichen). Deshalb wird er **gemessen**, nicht analytisch abgeleitet.
- Kalibrierung beim Codebook-Training und bei jedem Rebuild: zwei disjunkte Stichproben (Vektoren und Queries,
  je ≥ 2 000, Seed über den `Rng`-Port, P28); Mittelwert und Standardabweichung von
  $d^2_{\text{quant}} - d^2_{\text{exakt}}$ für den asymmetrischen Pfad (Query exakt) und den symmetrischen Pfad.
- Ergebnis im Index-Header, gebunden an die Codebook-Version. Ein neues Codebook macht Schwellen der alten Version
  ungültig (Header-Prüfung).
- Anwendung: `memfuse-rank` subtrahiert `l2sq_asym_mean` von quantisierten Distanzen, bevor sie mit absoluten
  Schwellen verglichen werden. Der Offset ist nur im Mittel konstant; die Streuung `l2sq_asym_std` (synthetisch
  0,37 % der Distanzskala) bleibt als Rauschterm. Kandidaten innerhalb von $\pm 2\,\sigma$ der Schwelle werden, wenn
  der Original-Float-Vektor verfügbar ist, damit neu bewertet.

```rust
pub struct Sq8Bias {
    pub codebook_version: u32,
    pub sample_pairs: u32,
    pub l2sq_asym_mean: f32, // E[d²_quant(q exakt, x quantisiert) − d²_exakt]
    pub l2sq_asym_std: f32,
    pub l2sq_sym_mean: f32,  // beide quantisiert
    pub rounding_energy: f32, // Σ_d Δ_d²/12 — nur der Rundungsanteil
}

impl Sq8Codebook {
    pub fn rounding_energy(&self, dim: usize) -> f32 {
        self.min.iter().zip(self.max.iter()).take(dim)
            .map(|(lo, hi)| { let d = (hi - lo) / 255.0; d * d / 12.0 }).sum()
    }
}
```

Die Zahlen stammen aus synthetischen Daten ($n = 30\,000$, $D = 384$), nicht aus echten Embeddings; die
Nachmessung auf Produktivdaten ist Teil von AK-12 (`sq8_bias_calibration.rs`: Bias auf Holdout-Paaren innerhalb
10 % des kalibrierten Werts, Rundungsenergie innerhalb 2 % von $\sum \Delta^2/12$, Spearman-Korrelation der
Rangfolge ≥ 0,99).

**⚠️ Opus-Optimierungen für den HNSW-Hot-Path:**

| ID | Maßnahme | Aufwand |
|---|---|---|
| 1.1 | Nachbarlisten-Auflösung ohne Allokation — Referenz statt Kopie; mittelfristig fester Stride | Gering → Hoch |
| 1.2 | Backlink-Auflösung von O(P×B) auf O(1) — HashMap pro Suche | Gering |
| 1.3 | Lock auf Quantisierer einmalig pro Suchaufruf, Distanz direkt auf Mmap-Slice | Mittel |

### 7.5 Community-Detection: Leiden (🟢 binärer Pfad)

Stern-Expansion für Hyperkanten-Projektion (🔴), Gewichte nach Konvention K (§6.6 H6). Der
`StarExpansionIterator` hält einen **Snapshot** (`Arc<GraphInner>`), keine Kopie der Hyperkanten. Ist-Zustand im
Repo: `StarExpansionIterator::new` klont alle aktiven Hyperkanten (`.cloned().collect()`).

```rust
pub struct StarEdge {
    pub participant: EntityId,
    pub virtual_node: HyperEdgeId,
    pub weight: f32, // star_weight(w(e), |e|)
}

pub struct StarExpansionIterator {
    snapshot: Arc<GraphInner>, // gehaltener Snapshot, keine Kopie
    edge_pos: usize,           // Index in snapshot.hyperedge_order (aufsteigende HyperEdgeId)
    participant_pos: usize,
}

impl Iterator for StarExpansionIterator {
    type Item = StarEdge;
    fn next(&mut self) -> Option<StarEdge> { unimplemented!() }
}
```

**Vertrag:** nur aktive (nicht tombstonierte) Hyperkanten; Hyperkanten mit $|e| < 2$ werden übersprungen
(`star_weight` liefert `None`); Reihenfolge deterministisch (aufsteigende `HyperEdgeId`, innerhalb einer
Hyperkante in Teilnehmer-Reihenfolge; §4(3)); keine Allokation pro `next()`.

### 7.6 Provenance-Tracking

**⚠️ Opus-Optimierung 1.8 — `build_provenance` Struct (Stufe 1, gering-mittel):**
Von 14 positionellen Parametern auf benannte Struct:

```rust
#[derive(Default)]
pub struct ProvenanceBuilder {
    source_doc_id: Option<DocId>,
    signal_contributions: Vec<(SignalKind, f32)>,
    fusion_mode: Option<FusionMode>,
    calibrated_threshold: Option<f32>,
}

impl ProvenanceBuilder {
    pub fn source_doc_id(mut self, id: DocId) -> Self { self.source_doc_id = Some(id); self }
    pub fn add_signal(mut self, kind: SignalKind, score: f32) -> Self {
        self.signal_contributions.push((kind, score)); self
    }
    pub fn build(self) -> Result<ProvenanceRecord, DbError> { unimplemented!() }
}
```

---

<a id="8-bandit"></a>
## 8. Contextual-Bandit-Routing

MemFuse integriert einen Multi-Armed-Bandit-Router (LinUCB, Li et al. 2010) zur adaptiven Aussteuerung der
Retrieval-Strategien.

### 8.1 Gemeinsame Schnittstelle

```rust
pub trait BanditPolicy: Send + Sync {
    // Fassung 2.1: `Result` statt `debug_assert` (Opus 0.2, harte Dimensionsprüfung).
    fn select_arm(&self, context: &[f32]) -> Result<RetrievalStrategy, BanditError>;
    fn update(&mut self, context: &[f32], arm: RetrievalStrategy, reward: f32) -> Result<(), BanditError>;
}

pub enum RetrievalStrategy { Vector, Text, Graph, Hybrid }
```

### 8.2 Zwei Implementierungsvarianten

**`DiagonalApproximation` (🟢 Produktions-Default):**

```rust
pub struct DiagonalApproximationBandit {
    theta: Vec<f32>,
    sigma_sq: Vec<f32>,
    drift: LyapunovDriftWatcher,
}

impl BanditPolicy for DiagonalApproximationBandit {
    fn update(&mut self, context: &[f32], _arm: RetrievalStrategy, reward: f32) -> Result<(), BanditError> {
        if context.len() != self.theta.len() || self.sigma_sq.len() != self.theta.len() {
            return Err(BanditError::DimensionMismatch { expected: self.theta.len(), actual: context.len() });
        }
        for ((th, sg), &xi) in self.theta.iter_mut().zip(self.sigma_sq.iter_mut()).zip(context) {
            *th += reward * xi / sg.max(1e-8);
            *sg += xi * xi;
        }
        Ok(())
    }
    fn select_arm(&self, context: &[f32]) -> Result<RetrievalStrategy, BanditError> { unimplemented!() }
}
```

Dies ist strukturell ein SGD-artiges Verfahren — **keine** exakte Ridge-Regression im Sinne von $\theta = A^{-1}b$.

**`ShermanMorrisonBandit` (🟡 Opt-in, `egress-sherman-morrison`):**

Mathematisch korrekte inkrementelle Matrixinversion:

$$(A + xx^\top)^{-1} = A^{-1} - \frac{A^{-1}xx^\top A^{-1}}{1 + x^\top A^{-1} x}$$

wobei $A = \sum x_t x_t^\top + \lambda I$ und $b = \sum r_t x_t$, $\theta = A^{-1}b$.

**Änderung Fassung 2.1:** `AlignedVector<{ D * D }>` kompiliert auf stable nicht („generic parameters may not be
used in const operations", mit rustc 1.75 geprüft; die Sprachregel gilt unverändert in späteren stable-Versionen).
Außerdem wäre `x.data.len() != D` bei einem Array immer falsch. Die Dimension ist deshalb ein **Laufzeitwert**
mit harter Prüfung (Opus 0.2). `memfuse-router` bleibt `forbid(unsafe_code)` (§0.4): Der Kern ist Safe Rust;
SIMD-Kerne für `matvec`/`axpy` kommen, falls die Latenzmessung sie verlangt (§8.4), als sichere API aus
`memfuse-simd` (Unsafe-Insel), nicht als `unsafe` im Router.

**Discounting (Vergessen):** $A_t = \gamma A_{t-1} + x x^\top$, $b_t = \gamma b_{t-1} + r x$, $\gamma \in (0, 1]$. Für
$A^{-1}$ heißt das $A^{-1} \leftarrow \gamma^{-1} A^{-1}$ **vor** dem Rang-1-Update, die Unsicherheit wächst. Fassung 2
schrieb $A^{-1} \leftarrow \gamma A^{-1}$; das verkleinert die Unsicherheit und macht die Politik nach dem
Vergessen überzuversichtlich. Der Drift-Einmal-Discount (`discount_once`) skaliert $A^{-1}$ mit $\gamma^{-1}$ und
$b$ mit $\gamma$; $\theta = A^{-1}b$ bleibt dabei unverändert.

```rust
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum BanditError {
    #[error("rank-1 update denominator near zero or non-finite")]
    SingularUpdate,
    #[error("dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },
    #[error("invalid parameter: {0}")]
    InvalidParameter(&'static str),
}

pub struct ShermanMorrisonBandit {
    dim: usize,
    inv_a: Vec<f32>, // dim*dim, row-major, A⁻¹
    b: Vec<f32>,
    theta: Vec<f32>,
    gamma: f32,      // in (0, 1]
    v: Vec<f32>,     // Arbeitspuffer
}

impl ShermanMorrisonBandit {
    pub fn new(dim: usize, lambda: f32, gamma: f32) -> Result<Self, BanditError> {
        if dim == 0 || !(lambda.is_finite() && lambda > 0.0) { return Err(BanditError::InvalidParameter("dim/lambda")); }
        if !(gamma.is_finite() && gamma > 0.0 && gamma <= 1.0) { return Err(BanditError::InvalidParameter("gamma")); }
        let mut inv_a = vec![0.0f32; dim * dim];
        for (i, row) in inv_a.chunks_exact_mut(dim).enumerate() {
            if let Some(d) = row.get_mut(i) { *d = 1.0 / lambda; }
        }
        Ok(Self { dim, inv_a, b: vec![0.0; dim], theta: vec![0.0; dim], gamma, v: vec![0.0; dim] })
    }

    /// O(d²), ohne `unsafe`.
    pub fn update_rank_1(&mut self, x: &[f32], reward: f32) -> Result<(), BanditError> {
        if x.len() != self.dim {
            return Err(BanditError::DimensionMismatch { expected: self.dim, actual: x.len() });
        }
        let g_inv = 1.0 / self.gamma;
        for (vi, row) in self.v.iter_mut().zip(self.inv_a.chunks_exact(self.dim)) {
            *vi = g_inv * row.iter().zip(x).map(|(a, xj)| a * xj).sum::<f32>(); // v = γ⁻¹ A⁻¹ x
        }
        let s = 1.0 + x.iter().zip(&self.v).map(|(a, b)| a * b).sum::<f32>();
        if !s.is_finite() || s < 1e-8 { return Err(BanditError::SingularUpdate); }
        for (row, vi) in self.inv_a.chunks_exact_mut(self.dim).zip(&self.v) {
            for (a, vj) in row.iter_mut().zip(&self.v) { *a = g_inv * *a - vi * vj / s; } // γ⁻¹A⁻¹ − v vᵀ/s
        }
        for (bi, xi) in self.b.iter_mut().zip(x) { *bi = self.gamma * *bi + reward * xi; }
        for (ti, row) in self.theta.iter_mut().zip(self.inv_a.chunks_exact(self.dim)) {
            *ti = row.iter().zip(&self.b).map(|(a, bj)| a * bj).sum();
        }
        Ok(())
    }

    /// Einmaliger Drift-Discount: A ← γA, b ← γb  ⇒  A⁻¹ ← γ⁻¹A⁻¹, θ unverändert.
    pub fn discount_once(&mut self, gamma: f32) -> Result<(), BanditError> {
        if !(gamma.is_finite() && gamma > 0.0 && gamma <= 1.0) { return Err(BanditError::InvalidParameter("gamma")); }
        let g_inv = 1.0 / gamma;
        self.inv_a.iter_mut().for_each(|a| *a *= g_inv);
        self.b.iter_mut().for_each(|b| *b *= gamma);
        Ok(())
    }

    pub fn theta(&self) -> &[f32] { &self.theta }
}
```

**⚠️ Opus-Optimierung 0.2 — Dimensionsprüfung (Stufe 0, gering):**
`score()`/`update()` von `debug_assert` auf harte `Result`-Fehlerbehandlung mit `DimensionMismatch { expected, actual }`.
Dimensions-Versionierung in `BanditProfileState`.

**⚠️ Opus-Optimierung 0.3 — Drift-Bandit-Kopplung (Stufe 0, gering):**
Drift-Reaktionsmethode bei `DriftDetected` tatsächlich aufrufen statt nur loggen. Mit konfigurierbarem `k_drift`.

### 8.3 Gedeckelter Lyapunov-Drift-Regelkreis (🟢)

```rust
pub struct LyapunovDriftWatcher {
    pub drift_decay_window: u32,   // Default 50
    pub drift_gamma: f32,          // Default 0.95
    steps_remaining: std::sync::atomic::AtomicU32,
    integrator_state: std::sync::atomic::AtomicU32, // f32-Bits
}

impl LyapunovDriftWatcher {
    /// Anti-Windup: bei PID-Sättigung stoppt der Integrator sofort.
    pub fn update(&self, error: f32, dt_seconds: f32, saturated: bool) -> f32 {
        if saturated { return self.current_alpha(); }
        // Zeitfensterbasierte, gedeckelte Eskalation.
        unimplemented!()
    }
}
```

### 8.4 Default-Umstellung

Der Wechsel des Produktions-Defaults zu `ShermanMorrison` ist an ein CI-Latenzbudget-Gate gebunden:
Kriterium < 5 % der medianen LLM/SLM-Inferenzlatenz. `DiagonalApproximation` bleibt als Low-Memory-Opt-out.

### 8.5 Off-Policy-Evaluation (IPS): Voraussetzung Randomisierung (ab Fassung 2.1)

LinUCB wählt deterministisch (Argmax). Die Propensity des gewählten Arms ist damit 1, die aller anderen 0; IPS
gegen eine andere Policy ist dann nicht definiert (kein gemeinsamer Träger). Off-Policy-Evaluation braucht eine
**randomisierte Logging-Policy**:

- ε-greedy über $K = 4$ Arme: $\mu(a\mid x) = (1-\varepsilon)\,\mathbb 1[a = a^*] + \varepsilon/K$. Mit
  $\varepsilon \ge 0{,}04$ ist $\mu \ge 0{,}01$ für jeden Arm, der Clamp $\max(p, 0{,}01)$ greift nie und
  verzerrt die Schätzung nicht.
- Die Zufallszahl kommt aus dem `Rng`-Port (P28); Seed und **Propensity zum Entscheidungszeitpunkt** werden im
  WAL-/Provenance-Datensatz gespeichert (Feld `propensity: f32`), nie nachträglich rekonstruiert (die Policy
  ändert sich).
- Kosten: etwa $\varepsilon$ suboptimale Auswahlen; deshalb Randomisierung optional auf einen Anteil des Verkehrs
  begrenzen.
- Varianz: Bei Bedarf self-normalized IPS (SNIPS) statt IPS.

Testpflicht (AK-15): `ips_requires_propensity.rs` — jeder geloggte Datensatz trägt `propensity ≥ 0,01`.

---

<a id="9-inferenz"></a>
## 9. Inferenz, KV-Cache v2 und Zero-Copy-IPC

> **Statuskorrektur, verbindlich (§A2.1):** Die KV-Cache-Bridge war in jeder Vorfassung dieser Spec als 🟢
> markiert. Eine Verifikation am Code (`candle/inference.rs:485`) hat ergeben, dass `generate_with_context`
> nur `format!("kv_cache_tensor:…")` als Platzhalter-„Tensor" ablegt, ein Treffer lediglich einen Zähler
> (`prefill_skip_count`) erhöht, ohne dass tatsächlich Prefill-Rechenzeit eingespart wird, und `generate()`
> in jedem Fall den vollständigen Text neu verarbeitet. **Der Status wird hiermit korrigiert auf 🔴** — die
> KV-Cache-Bridge ist spezifiziert, im Code nicht vorhanden, und bleibt 🔴, bis der in §9.2 genannte
> Golden-Test grün ist.

### 9.1 Zero-Copy-Eviction-Bridge (🟢, unverändert)

Durchgehende Zero-Copy-Datenpipeline auf Basis von `Bytes` und Mmap. FlatBuffers erlaubt direktes Auslesen
aus `&[u8]` ohne Heap-Allokation. Diese Eigenschaft ist von der KV-Cache-Neuspezifikation unberührt.

### 9.2 KV-Cache v2 (🔴, ersetzt §9.2 der Vorfassung vollständig)

**Vorher (nicht mehr normativ — Grund für die Ablösung siehe Statuskorrektur oben):** eine `KvCacheBridge`
mit `scc::HashMap<SessionId, EncryptedKvSegment>` als RAM-Tier und `memfuse_store::LsmStore` als
Fallback-Spill; `EncryptedKvSegment` mit `rope_offset` pro Segment. Diese Skizze existierte nur in der Spec,
nicht im Code, und ist außerdem architektonisch problematisch (siehe „Warum kein LSM-Spill" unten).

**Jetzt: drei Ausbaustufen, deklariert über `KvReusePolicy`, Crate `memfuse-kvcache` (Ring 1).**

**Fakten aus der Verifikation (§A2, `MEMFUSE_ZIELARCHITEKTUR_v2.md` §5.1):**
- Upstream (`candle-transformers 0.11.0`, `quantized_llama.rs`): KV ist pro Layer privat
  (`Option<(Tensor, Tensor)>`); jeder Schritt kopiert per `Tensor::cat` (O(n) je Layer und Token);
  `index_pos == 0` verwirft den Cache; Keys werden nach RoPE gecacht; Werte sind F32, weil `QMatMul` F32
  liefert. `LayerWeights.kv_cache` ist **privat**; öffentlich sind nur `forward` und `clear_kv_cache`.
- Größenordnung (Beispiel Llama-3.2-3B, 28 Layer, 8 KV-Köpfe, head_dim 128): 224 KiB/Token, ein 16-Token-Block
  ≈ 3,5 MiB, 2048 Token ≈ 448 MiB. Ein 2-GiB-RAM-Budget hält damit ≈ 4 gleichzeitige Prompts bei F32.
- **Warum kein LSM-Spill (Korrektur gegenüber der „Vorher"-Skizze):** KV-Blöcke sind Megabyte-groß. Eine
  Leveled-Compaction-LSM-Engine schreibt Werte dieser Größe mehrfach um (Größenordnung 10×,
  konfigurationsabhängig) — das ist Write-Amplifikation ohne Gegenwert. Persistenz für KV-Blöcke erfolgt
  stattdessen über **append-only Segmentdateien** in `memfuse-kvcache` (Header, Prüfsumme, AEAD; Index wird
  beim Start aus den Segment-Headern aufgebaut). Die LSM-Engine (`memfuse-store`) trägt höchstens Metadaten.

| Stufe | Inhalt | Modell | Persistenz | Gate vor Produktions-Default |
|---|---|---|---|---|
| **A** | In-RAM-Prefix-Reuse per `ModelWeights::clone()` nach Prefill an Segmentgrenzen (System-Prompt, Kontextsegmente); Gewichte/KV-Tensoren teilen sich per `Arc`, `cat` erzeugt neue Tensoren | Upstream unverändert, kein Fork | keine | Logit-Vergleich Clone vs. Neuberechnung (bitgleich erwartet bei gleicher Batch-Form) |
| **B** | Eigenes Llama-Modell (Fork von `quantized_llama.rs`, Lizenz vorab prüfen) mit erstklassigem `KvState`: `truncate`, `export_block`, `import_block`; Radix-Baum, Copy-on-Write, Referenzzählung; RAII-Guards geben Blöcke bei `Drop` zurück (cancel-sicher) | eigenes Modell, an Upstream-Logits verifiziert | RAM | Golden-, Property- (Export→Import identisch), Cancellation-Test |
| **C** | Spill in T2: verschlüsselte Segmentdateien, AEAD mit AAD aus Tenant/Fingerprint/Token-Hash/Positionsbereich; Löschen per Crypto-Shredding des Tenant-Schlüssels | eigenes Modell (B) | Platte | Isolations- und Crash-Test der Segmente |

**Entscheidungsstatus (offen, §A2.4 Nr. 2):** Stufe A liefert bereits Nutzen ohne Fork-Risiko und ist der
nächste konkrete Schritt. Ob B/C gebaut werden, entscheidet sich erst nach Messung von A — das ist keine
Verzögerung, sondern eine bewusste Sequenzierung, damit die Fork-Wartungslast (Risiko Nr. 2, §A2.4) nicht
ungeprüft eingegangen wird.

```rust
// memfuse-ports — Vertrag, gilt ab Stufe B
pub struct PrefixKey {
    pub model: ModelFingerprint,
    pub tokenizer_hash: [u8; 32],
    pub layout: KvLayout,   // n_layer, n_kv_head, head_dim, dtype
    pub rope: RopeConfig,  // base, scaling
}

pub trait KvPrefixStore: Send + Sync {
    /// Längster exakt passender Prefix in Blöcken; kein Treffer ist kein Fehler.
    fn lookup(&self, tenant: TenantId, key: &PrefixKey, tokens: &[u32]) -> Option<KvPrefixHit>;
    fn insert(&self, tenant: TenantId, key: &PrefixKey, tokens: &[u32], blocks: Vec<KvBlock>) -> Result<(), KvError>;
}

// memfuse-infer-candle — KV wird erstklassig, nicht privat (Stufe B)
pub struct KvState { layers: Vec<LayerKv>, pos: usize }
pub struct LayerKv { k: Tensor, v: Tensor }   // Keys nach RoPE, wie im Upstream-Modell

impl KvState {
    pub fn truncate(&mut self, pos: usize);
    pub fn export_block(&self, range: Range<usize>) -> Result<KvBlock, InferError>;
    pub fn import_block(&mut self, block: &KvBlock) -> Result<(), InferError>;
}
```

**Golden-Test-Kriterium (verschärft gegenüber der Vorfassung):** Die feste Toleranz 1e-4 aus der ursprünglichen
Zielarchitektur-Prüfung entfällt — quantisierte Kernel liefern bei unterschiedlicher Batch-Form (Prefill n
gegen m + Rest) nicht zwingend bitgleiche Summen. Stattdessen gilt: (i) identische Greedy-Tokenfolge über ein
festes Prompt-Set (≥ 200 Prompts × 64 Token); (ii) `max |Δlogit| ≤ 2 ×` gemessene Run-to-Run-Varianz des
Upstream-Modells. Zusätzlich: Tenant-Isolationstest (kein Teilen von Blöcken über Tenants oder Modelle hinweg
— sonst verrät Cache-Hit-Timing fremde Prompts), Cancellation-Test (Abbruch mitten im Prefill gibt alle
Blöcke zurück).

**Metrik-Korrektur (Sofortmaßnahme, unabhängig vom Stufenausbau):** `prefill_skip_count` zählt derzeit jeden
Cache-„Treffer" auf den Platzhalter hoch, obwohl kein Prefill übersprungen wird. Dieser Zähler wird korrigiert,
bevor er in Dashboards oder Abnahmekriterien verwendet wird (§20, Phase 0R).

### 9.3 AES-Schlüsselplan-Wiederverwendung — globaler Zustand entfernt (P29)

**Vorher (nicht mehr normativ):**

```rust
use aes_gcm_siv::Aes256GcmSiv;
use std::sync::OnceLock;

static CIPHER_INSTANCE: OnceLock<Aes256GcmSiv> = OnceLock::new();

pub fn cipher() -> &'static Aes256GcmSiv { unimplemented!() }
```

Dieser Entwurf verstößt gegen P29 (kein globaler veränderlicher Zustand, §3): ein einzelner globaler Cipher
kann nicht pro Tenant/Schlüssel rotiert werden und widerspricht der in `memfuse-crypto` beschriebenen
Schlüsselhierarchie.

**Jetzt (verbindlich):** Die Cipher-Instanz wird **pro Schlüssel** im `KeyManager` (Instanzzustand, nicht
`static`) gecacht und thread-safe über `OnceCell`-Felder wiederverwendet:

```rust
pub struct KeyManager {
    ciphers: scc::HashMap<KeyId, OnceCell<Aes256GcmSiv>>,
    // Nonce-Strategie: ⚖️ Entscheidung ausstehend (§A2.4 Nr. 5). KEIN In-Memory-Zähler ohne Hochwasserstand.
}

impl KeyManager {
    pub fn cipher_for(&self, key_id: KeyId) -> Result<&Aes256GcmSiv, CryptoError>;
}
```

**Nonce-Strategie (⚖️ ausstehend, Fassung 2.1):** Fassung 2 sah einen deterministischen `AtomicU64`-Zähler statt
zufälliger Nonces vor. So ist das nicht übernehmbar: Ein Zähler im RAM beginnt nach einem Neustart wieder bei 0
und wiederholt Nonces unter demselben Schlüssel. Ist-Zustand im Repo: 4-Byte-Präfix je Schlüssel plus 8 Byte
`OsRng`, bewusst ohne persistierten Zähler. Optionen: (a) Status quo mit festgelegtem Nachrichtenbudget je Schlüssel
(bei 64 Zufallsbits liegt die Kollisionswahrscheinlichkeit bei etwa $n^2/2^{65}$, also $pprox 2^{-32}$ bei
$n pprox 9\cdot 10^4$ Nachrichten); (b) persistierter Epochenzähler (4 Byte, bei jedem Öffnen erhöht und vor der
ersten Nutzung `fsync`'d) ‖ `AtomicU64`-Zähler (8 Byte). AES-256-GCM-SIV ist nonce-misuse-resistent: eine
Wiederholung offenbart nur die Gleichheit identischer Klartexte, kompromittiert aber den Schlüssel nicht.
Schlüsselrotation erfolgt über gezielten Austausch des `OnceCell`-Eintrags je `KeyId`,
nicht über einen globalen Austausch.

### 9.4 Ring-2/3-Crates (vormals „Layer-3-Crates")

- **`memfuse-infer-ollama`** (vormals `memfuse-ollama`): `OllamaClient::generate(prompt, contextual_prefix) -> Result<String, OllamaError>`.
  Contextual-Chunk-Prefixing fügt Retrieval-Kontext als System-Präfix ein.
- **`memfuse-infer-onnx`** (vormals `memfuse-embed`): `EmbeddingModel::embed(texts) -> Result<Vec<Vec<f32>>, EmbedError>`
  (ONNX, aus `default-members` ausgeschlossen, §0.2/§0.3), `CrossEncoderReranker::rerank(query, candidates) -> Vec<SearchResult>`.
- **`memfuse-agent`:** `AgentWorkflow`-Engine mit persistentem Zustand über `Checkpointable`. Unverändert im
  Zuschnitt, jetzt Ring 3.
- **`memfuse-py`:** PyO3-Bindings. **Δ gegenüber Vorfassung:** die Panic-Strategie-Isolation war spezifiziert,
  aber am Root-Profil (`panic = "abort"`) real wirkungslos (`catch_unwind` ist im Release-Build mit
  `panic = "abort"` funktionslos). Ab dieser Fassung: Root-Profil `panic = "unwind"` (§0.2), `memfuse-py` als
  regulärer Workspace-Member, `release-abort` nur für Binaries ohne FFI. Testpflicht: ein Panic muss im
  `maturin build --release`-Wheel als `PyErr` ankommen, nicht nur im Debug-Build.

`memfuse-py` ist damit — anders als in der Vorfassung dokumentiert — reguläres Workspace-Mitglied, nicht
„eigener Workspace"; die davon abweichende Doku-Behauptung in `memfuse-py-ci.yml` und den Python-Tests wird
im selben Zug korrigiert (§20, Phase 0R, Track T3).

---

<a id="10-sicherheit"></a>
## 10. Sicherheits- und Datenschutzmodell

### 10.1 Kryptographische Grundlagen

AES-256-GCM-SIV für Daten at rest, WAL mit race-freier HMAC-Kette, `DeletionProof` für DSGVO-Art.-17-Nachweise.

```rust
pub struct DeletionProof {
    pub key_hash: [u8; 32],
    pub hmac_chain_entry: [u8; 32],
    pub prev_hmac: [u8; 32],
    pub timestamp: i64,
}

pub struct HmacChain {
    last: std::sync::Mutex<[u8; 32]>,
}

impl HmacChain {
    /// MUSS atomar gegenüber gleichzeitigen `append`-Aufrufen sein.
    pub fn append(&self, payload: &[u8]) -> Result<[u8; 32], CryptoError>;
    pub fn verify_chain(entries: &[[u8; 32]]) -> Result<(), CryptoError>;
}

#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("hmac chain fork at index {0}")]
    ChainFork(usize),
    #[error("aead operation failed")]
    AeadFailure,
}
```

### 10.2 WASM-Sandbox (🟢)

```rust
pub struct WasmCapabilities {
    pub max_fuel: Option<u64>,
    pub max_wall_clock_ms: u64, // Default 5000; 0 = unbegrenzt
    pub allow_cloud_egress: bool, // Default false
}

pub struct SandboxExecutor {
    engine: wasmtime::Engine,
}

impl SandboxExecutor {
    pub fn execute(&self, module: &[u8], caps: &WasmCapabilities) -> Result<Vec<u8>, SandboxError> {
        let mut store = wasmtime::Store::new(&self.engine, ());
        if let Some(fuel) = caps.max_fuel { store.set_fuel(fuel).map_err(SandboxError::from)?; }
        // Wall-Clock unabhängig von Fuel via separatem Timeout-Task (P23).
        unimplemented!()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SandboxError {
    #[error("fuel budget exhausted")]
    FuelExhausted,
    #[error("wall clock budget of {0}ms exceeded")]
    WallClockExceeded(u64),
}
```

`fd_write`-WASI-Stub MUSS `iovs` korrekt parsen.

### 10.3 Prompt-Injection-Schutz

Eingaben aus dem Agenten-Kontext werden nicht ungeprüft als Steuerbefehle interpretiert; der Lesepfad ist
durchgehend Zero-Copy, um unnötige Pufferkopien sensibler Daten zu vermeiden.

### 10.4 Cloud-Egress Privacy Gateway (🟢, 5-Schichten-Architektur)

1. **Token-Vaulting/Pattern-Matching (`EgressVault`):** `RegexSet`-Klassifikation, Payload-Deckel.
2. **Vorabstraktion.**
3. **Graph-Generalisierung.**
4. **Bulk-Exfiltration-Detektor:** Großvolumige Anfragemuster erkennen.
5. **Re-Hydration:** `CloudResponseRehydrator::rehydrate` — Round-Trip-sicher, Multibyte-UTF-8-panic-sicher.

```rust
pub struct EgressVault {
    pattern_matcher: regex::RegexSet,
    surrogate_map: scc::HashMap<SurrogateToken, OriginalEntity>,
}

impl EgressVault {
    pub fn generate_surrogate(&self, entity: &OriginalEntity, session: SessionId) -> SurrogateToken;
    pub fn get_entity(&self, token: &SurrogateToken) -> Option<OriginalEntity>;
}

pub struct BulkExfiltrationDetector {
    pub max_bytes_per_window: usize,
    pub window: std::time::Duration,
}

pub struct CloudResponseRehydrator;
impl CloudResponseRehydrator {
    pub fn rehydrate(&self, response: &str, vault: &EgressVault) -> String { unimplemented!() }
}
```

**⚠️ Opus-Optimierung 0.4 — Egress-Klassifizierung (Stufe 0, gering):**
Eigene, restriktivere Policy-Kategorie für Cloud-Egress-Methoden statt gleiche wie lokale Lesezugriffe.

---

<a id="11-betrieb"></a>
## 11. Betriebsmodi

MemFuse wird ausschließlich eingebettet betrieben: im Prozess des aufrufenden Agenten (Rust- oder Python-Bindung)
oder als lokaler MCP-Server über stdio-JSON-RPC. Es gibt keinen Server-Modus mit Netzwerk-Listener für
Multi-Tenant-Zugriff. Der Cloud-Egress-Pfad (§10.4) ist der einzige Punkt, an dem Daten das lokale System
verlassen — ausschließlich auf explizite Anforderung, nie als Hintergrundtelemetrie.

---

<a id="12-schema"></a>
## 12. FlatBuffers-Schema (vollständig, `schemas/memfuse.fbs`)

```fbs
namespace memfuse.ipc;

table RoleBindingFb {
  role: uint32;
  entity: uint64;
}

table HyperEdgeFb {
  id: uint64;
  predicate_tag: uint32;          // EdgeType-Diskriminante
  participants: [RoleBindingFb];  // min. 2, validiert applikationsseitig
  weight: float32;
  tx_valid_from: uint64;
  tx_valid_to: uint64;            // 0 = None (Sentinel, dokumentiert)
  business_valid_from: int64;
  business_valid_to: int64;       // i64::MIN = None (Sentinel)
  source_doc_id: uint64;          // oder uint128-Encoding bei docid-128
}

table EdgeFb {
  target: uint64;
  weight: float32;
  edge_type_tag: uint32;
  tx_valid_from: uint64;
  tx_valid_to: uint64;
  business_valid_from: int64;
  business_valid_to: int64;
  source_doc_id: uint64;
}

root_type HyperEdgeFb;
```

**CI-Drift-Gate (`xtask check-flatbuffers-drift`):** Vergleicht Hash des generierten Codes gegen committeten
Referenz-Hash. Jede Schema-Änderung ohne begleitende Regenerierung schlägt den Merge-Gate-Job fehl. **Dieses
Gate MUSS grün sein, bevor `HyperEdgeFb` gemerged wird (H4).**

---

<a id="13-fehler"></a>
## 13. Fehlertaxonomie (crateübergreifend)

| Crate | Fehler-Enum | Einbettet |
|---|---|---|
| `memfuse-core` | `CoreError` | — |
| `memfuse-store` | `StoreError` | `WalError`, `LockError`, `CoreError` |
| `memfuse-crypto` | `CryptoError` | — |
| `memfuse-index` | `IndexError` | `CoreError` |
| `memfuse-graph` | `GraphMutationError`, `GraphError` | `LockError` |
| `memfuse-router` | `BanditError` | — |
| `memfuse-candle` | `KvBridgeError` | `CryptoError` |
| `memfuse-mcp` | `SandboxError`, `EgressError` | `wasmtime::Error` |
| `memfuse-db` | `DbError` | alle Layer-1-Fehler per `#[from]` |

**Regel (verbindlich):** Kein öffentlicher Funktionsrückgabetyp ist `Box<dyn std::error::Error>`. Jeder Crate
exportiert genau einen (oder wenige, klar abgegrenzte) `thiserror`-Fehlertyp(en); `memfuse-db` als oberste
Konsumentenschicht bündelt alle Unterfehler verlustfrei per `#[from]`/`#[error(transparent)]`.

```rust
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error(transparent)] Store(#[from] memfuse_store::StoreError),
    #[error(transparent)] Graph(#[from] memfuse_graph::GraphMutationError),
    #[error(transparent)] Index(#[from] memfuse_index::IndexError),
    #[error("provenance builder missing required field: {0}")]
    ProvenanceIncomplete(&'static str),
}
```

---

<a id="14-features"></a>
## 14. Feature-Flag-Politik: Produktions-Default vs. Opt-in

Ein Breaking-Change- oder Performance-Trade-off-Feature wird hinter einem Cargo-Feature isoliert, bis eine
explizite Produktentscheidung den Wechsel des Defaults auslöst. Dies ist **kein Mangel**, sondern verbindliche
Politik.

| Feature-Flag | Reifegrad | Beschreibung |
|---|---|---|
| `cloud-egress-guard` | 🟢 | DLP/Egress-Kontrolle, Surrogat-Tokenisierung, Bulk-Exfiltration-Detektor |
| `bandit-routing` | 🟢 | LinUCB-Grundfunktion, Lyapunov-Kopplung, gedeckelte Drift-Eskalation |
| `egress-sherman-morrison` | 🟡 | Mathematisch korrekte Ridge-Regression; einziger Pfad mit LinUCB-Regret-Garantie |
| `kv-bridge` | 🟢 | KV-Cache-Bridge inkl. LSM-Fallback, AES-256-GCM-SIV |
| `wasm-sandbox` | 🟢 | Fuel- und Wall-Clock-Budget orthogonal |
| `experimental-diskann` | 🟢 (Tier) | Native Tombstones, SQ8-Perzentil-Clipping |
| `docid-128` | 🟡 | 128-Bit-BLAKE3-DocId, Rollout vollzogen, Default bleibt `u64` |
| `block-cache-v2` | 🟡 | S3-FIFO-Backend (`quick_cache`), Default bleibt LRU |
| `bm25f` | 🟢 | Feldgewichtete BM25-Bewertung |
| `flatbuffers-drift-gate` (xtask) | 🟢 | CI-Gate gegen Schema-Drift |
| `fault-injection` | 🟢 | Test-only |
| `loom` | Dev | Nebenläufigkeits-Modelltests |
| `adaptive-decay` / `-control` | 🟢 | Kalibrierungs-Feintuning |
| `partial-index-rebuild` | 🟢 | Inkrementeller Indexaufbau |
| `edge-reinforcement-learning` | 🟢 (Gate) | Kantenverstärkung als optionales Fusionsverhalten |
| **Hyperkanten (`relate_n_ary`, `HyperEdge`)** | **🔴** | Nicht implementiert — kein Flag, siehe §6 |

---

<a id="15-tests"></a>
## 15. Test- und CI-Spezifikation

### 15.1 Unit-Tests

Jede öffentliche Funktion mit nicht-trivialer Logik erhält mindestens:
- einen Normalfall-Test,
- einen Grenzfall-Test (leere Eingabe, einzelnes Element),
- einen Fehlerfall-Test, der das korrekte `Result::Err`-Enum-Mitglied prüft (kein pauschales `is_err()`).

### 15.2 Integrationstests

| Testpfad | Zweck / AK |
|---|---|
| `crates/memfuse-graph/tests/hyperedge_persistence_survives_restart.rs` | AK-1 |
| `crates/memfuse-graph/tests/hyperedge_compact_race.rs` | AK-1 (nebenläufig zu `compact()`) |
| `crates/memfuse-graph/tests/hyperedge_memory_budget.rs` | AK-2 |
| `crates/memfuse-db/tests/signal_kind_no_new_variant.rs` | AK-4 |
| `crates/memfuse-graph/tests/hyperedge_cascade_fanout.rs` | AK-6 (High-Fan-out) |
| `crates/memfuse-graph/tests/community_hyperedges_included_flag.rs` | AK-7 |
| `crates/memfuse-graph/benches/binary_edge_regression.rs` | AK-8 (Baseline-Vergleich) |
| `crates/memfuse-graph/tests/hyperedge_payload_sharing.rs` | AK-9 |
| `crates/memfuse-graph/tests/star_expansion_equals_clique.rs` | AK-10 |
| `crates/memfuse-graph/tests/hyperedge_cascade_crash_recovery.rs` | AK-11 |
| `crates/memfuse-index/tests/sq8_bias_calibration.rs` | AK-12 |
| `crates/memfuse-store/tests/wal_backpressure.rs` | AK-13 |
| `crates/memfuse-store/tests/kv_locks_stable_shard.rs` | AK-14 |
| `crates/memfuse-router/tests/ips_requires_propensity.rs` | AK-15 |

### 15.3 Loom-Tests (`#[cfg(loom)]`, `loom`-Feature)

| Testpfad | Zweck |
|---|---|
| `crates/memfuse-store/tests/loom_group_commit.rs` | Group-Commit-Atomizität |
| `crates/memfuse-store/tests/loom_multi_key_lock.rs` | Multi-Key-Deadlockfreiheit |
| `crates/memfuse-graph/tests/loom_relate_n_ary.rs` | AK-3, Hyperkanten-Deadlockfreiheit |

Alle drei MÜSSEN als eigener CI-Job sichtbar grün laufen.

### 15.4 `.github/workflows/merge-gate.yml` — Pflicht-Jobs

```yaml
jobs:
  unit-tests:
    run: cargo test --workspace
  flatbuffers-drift-gate:
    run: cargo run -p xtask -- check-flatbuffers-drift
  hyperedge-schema-merge:
    needs: [flatbuffers-drift-gate]
    run: cargo test -p memfuse-graph --features hyperedges -- hyperedge
  check-bandit-latency-budget:
    run: cargo run -p xtask --features memfuse-router/egress-sherman-morrison -- check-bandit-latency-budget
  loom-tests:
    run: RUSTFLAGS="--cfg loom" cargo test --workspace --features loom -- --test-threads=1
  clippy-panic-lints:   # ersetzt `check-unwrap-baseline`: der Ratchet entfällt ersatzlos (§0.4)
    run: cargo clippy --workspace --lib --bins --locked -- -D warnings   # Lint-Konfiguration aus §0.4
```

**⚠️ Opus-Optimierung 3.1 — Feature-Kombinationen in CI (Stufe 3, gering):**
Powerset-Build der relevanten Feature-Flags ergänzen.

**⚠️ Opus-Optimierung 3.2 — Panic-Inventar (Stufe 3, gering):**
Gate nur für `src/` (ohne Tests/Benchmarks), harte sinkende Obergrenze.

---

<a id="16-abnahme"></a>
## 16. Vollständige Abnahmekriterien

### 16.1 Kernsystem-Abnahmekriterien

| Nr. | Kriterium | Reifegrad |
|---|---|---|
| K-1 | `insert_lock` vollständig durch key-granulare `kv_locks` ersetzt | 🟢 |
| K-2 | HNSW-Hotpath nutzt unaligned-SIMD-Distanzkernel statt Heap-Allokation | 🟢 |
| K-3 | BM25 nutzt residenten Postinglisten-Index mit Block-Max WAND | 🟢 |
| K-4 | Block-Cache erzwingt bei Lesetreffer im Opt-in-Backend keinen Write-Lock | 🟡 |
| K-5 | DiskANN löscht ohne HNSW-Fallback (native Tombstones) | 🟢 |
| K-6 | Bandit-Latenzbudget CI-gated | 🟢 |
| K-7 | `compact()` blockiert keine nebenläufigen Leser (RCU-Swap) | 🟢 |
| K-8 | PPR proportional zur Seed-Menge, nicht zur Graphgröße (P24) | 🟢 |
| K-9 | `build_provenance` nutzt `ProvenanceBuilder`-Struct | 🟢 |
| K-10 | FlatBuffers-Drift-Gate aktiv | 🟢 |
| K-11 | `SignalKind::from_name` allokationsfrei (`eq_ignore_ascii_case`) | 🟢 |
| K-12 | BM25F produktiv nutzbar | 🟢 |
| K-13 | KV-Cache-Bridge LSM-Fallback mit Testabdeckung für alle Kombinationen | 🟢 |
| K-14 | Drift-Alpha gedeckelt | 🟢 |
| K-15 | Cloud-Egress 5-Schichten produktiv, Rehydration inkl. UTF-8-Sicherheit | 🟢 |
| K-16 | ⚖️ Bandit-Default → ShermanMorrison (sobald Gate besteht) | ⚖️ |
| K-17 | ⚖️ Block-Cache-Default → SIEVE (sobald entschieden) | ⚖️ |
| K-18 | ⚖️ DocId-128 als Produktions-Default (Major-Release) | ⚖️ |
| K-19 | Group-Commit-Loom-Test sichtbar grün in CI | 🔴 |

### 16.2 Abnahmekriterien AK-1 bis AK-15 (normativ und abschließend; AK-9 bis AK-15 ab Fassung 2.1)

| AK | Kriterium | Nachweis (Testpfad) |
|---|---|---|
| AK-1 | `HyperEdge` mit ≥3 `RoleBinding`s persistiert, restart-fest, per `hyperedges_for_entity` auffindbar, konsistent unter gleichzeitigem `compact()` | `hyperedge_persistence_survives_restart.rs`, `hyperedge_compact_race.rs` |
| AK-2 | `estimate_memory_bytes()` inkl. Hyperkanten und capacity-basiert; `estimate_compaction_peak_bytes()` unterschätzt den mit Zählallokator gemessenen Spitzenwert nie und überschätzt ihn höchstens um Faktor 1,5; `compact_async`-Budget-Check greift gegen die Spitze | `hyperedge_memory_budget.rs` |
| AK-3 | Zwei gleichzeitige `relate_n_ary` mit überlappenden, unterschiedlich geordneten Mengen deadlockfrei | `loom_relate_n_ary.rs` |
| AK-4 | `SignalKind` strukturell unverändert (kein `Hyperedge`-Signal) | `signal_kind_no_new_variant.rs` |
| AK-5 | FlatBuffers-Drift-Gate grün **vor** `HyperEdgeFb`-Merge | CI-Job-Abhängigkeit `hyperedge-schema-merge: needs: [flatbuffers-drift-gate]` |
| AK-6 | Cascade bricht bei >1.000 Hyperkanten kontrolliert auf Hintergrundverarbeitung um | `hyperedge_cascade_fanout.rs` |
| AK-7 | `hyperedges_included` im Report sichtbar `false` ohne Projektion | `community_hyperedges_included_flag.rs` |
| AK-8 | Keine Regression auf binäre `relate()`/`Edge`-Benchmarks | `binary_edge_regression.rs` |
| AK-9 | `as_view()` und Snapshot-Klon kopieren keine Teilnehmer: `Arc::ptr_eq` auf `participants`, Allokationsvolumen des Klons unabhängig von `Σ\|e\|` | `hyperedge_payload_sharing.rs` |
| AK-10 | Stern-Expansion entspricht der konfigurierten Clique-Konvention (Schur-Komplement), N=2 ≙ binäre Kante, deterministische Reihenfolge, kein Klon der Hyperkanten | `star_expansion_equals_clique.rs` |
| AK-11 | Cascade überlebt Absturz nach Commit: alle Hyperkanten genau einmal tombstoniert, Queue leer, Wiederholung ist No-op; `DeletionProof` nur bei vollständiger Löschung | `hyperedge_cascade_crash_recovery.rs` |
| AK-12 | SQ8-Bias wird gemessen und im Header gebunden an die Codebook-Version; Genauigkeitsgrenzen laut §7.4 | `sq8_bias_calibration.rs` |
| AK-13 | WAL-Queue ist begrenzt: `Backpressure`/Warten bei voller Queue, `append` kehrt erst nach `fsync` zurück | `wal_backpressure.rs` |
| AK-14 | Shard-Zuordnung ist je `KvKeyLocks`-Instanz stabil; gleiche Entität ⇒ gleicher Shard | `kv_locks_stable_shard.rs` |
| AK-15 | Jeder Routing-Datensatz trägt `propensity ≥ 0,01` (randomisierte Logging-Policy) | `ips_requires_propensity.rs` |

---

<a id="17-optimierungen"></a>
## 17. Priorisierte Optimierungs-Roadmap (Opus-Analyse)

Die Reihenfolge der Stufen ist **verbindlich**: Stufe 0 blockiert bzw. gefährdet den Betrieb, Stufe 1 ist der
größte Hebel für Latenz/Durchsatz, Stufe 2 betrifft Speicherverbrauch und Struktur, Stufe 3 ist Governance.

### Stufe 0 — Korrektheit und Betriebssicherheit (ZUERST)

| ID | Maßnahme | Problem | Aufwand |
|---|---|---|---|
| **0.1** | **WAL-Replay-Panic entschärfen** | Dateigröße separat von `mmap.len()`, Direktindizierung → Panic bei veränderter Datei, Crash-Loop möglich | Gering |
| **0.2** | **Bandit-Dimensionsprüfung** | `debug_assert` statt `Result` bei Dimensionsmismatch → stilles Teil-Skalarprodukt nach Modellwechsel | Gering |
| **0.3** | **Drift-Bandit-Kopplung verdrahten** | Drift-Wächter erkennt Verteilungsverschiebung, ruft aber Bandit-Reaktionsmethode nie auf | Gering |
| **0.4** | **Cloud-Egress-Klassifizierung** | Egress-Methode in gleicher Policy-Kategorie wie lokale Lesezugriffe | Gering |
| **0.5** | **Recovery-Pfad differenzieren** | Offene Intents werden pauschal vorwärts committet, egal ob Erfolg oder Abbruch | Mittel |

### Stufe 1 — Hot-Path-Performance (größter Hebel)

| ID | Maßnahme | Problem | Aufwand |
|---|---|---|---|
| **1.1** | **HNSW: Nachbarlisten ohne Allokation** | Pro Knoten eine Heap-Allokation für Adjazenzliste, größter Einzelfaktor im Suchpfad | Gering → Hoch |
| **1.2** | **HNSW: Backlink O(1)** | Lineare Suche über Einfüge-Operationen bei Batch-Insert, quadratisch bei großen Batches | Gering |
| **1.3** | **HNSW: Distanzpfad Lock/Allokation** | Lock auf Quantisierer pro Kandidat, Mmap-Vektor Element-für-Element dekodiert | Mittel |
| **1.4** | **SSTable: Zero-Copy-Slice** | Vollständige Blockkopie nach CRC-Prüfung, obwohl Puffertyp Slicing unterstützt | Trivial |
| **1.5** | **AES-Schlüsselplan wiederverwenden** | Key-Schedule-Neuaufbau pro Verschlüsselung/Entschlüsselung | Mittel |
| **1.6** | **MemTable: Range-Sharding** | Hash-Sharding zerstört Flush-Sortierung und Präfix-Scans | Hoch |
| **1.7** | **Block-Cache: Byte-basierte Kapazität** | Eintragsbasierte Kapazität bei variabler Blockgröße → unvorhersehbarer Speicher | Mittel |
| **1.8** | **RRF: `build_provenance`-Struct** | 14 positionelle Parameter desselben Typs → stille Vertauschung möglich | Gering-Mittel |
| **1.9** | **Text: Posting-Format umstellen** | Jedes Posting als Einzelschlüssel → massive Schreib-/Leseverstärkung | Hoch |
| **1.10** | **RRF/Text: Top-k-Selektion** | Volle Sortierung statt linearer k-Selektion | Gering |

### Stufe 2 — Speicher und Struktur

| ID | Maßnahme | Problem | Aufwand |
|---|---|---|---|
| **2.1** | **Graph: Inkrementelle Kompaktierung** | PPR löst vollständigen CSR-Rebuild pro Anfrage aus | Hoch |
| **2.2** | **CSR: Sentinel statt `Option`** | `Vec<Option<T>>` ohne Nischenoptimierung → doppelter Speicher pro Kante | Mittel |
| **2.3** | **Checkpoint: Indizes zusammenführen** | Zwei separate Locks für Sequenz- und Namens-Index → Inkonsistenzfenster | Gering-Mittel |
| **2.4** | **Manifest: Batch-Fsync** | Ein `fsync` pro Einzeleintrag statt pro Zustandsübergang | Mittel |
| **2.5** | **`memfuse-py` in Workspace** | FFI-Grenze nicht von `cargo test --workspace` erfasst | Gering |

### Stufe 3 — Governance und Prozess

| ID | Maßnahme | Problem | Aufwand |
|---|---|---|---|
| **3.1** | **Feature-Kombinationen in CI** | Opt-in-Features werden nie in Kombination gebaut | Gering (Einrichtung) |
| **3.2** | **Panic-Inventar kontinuierlich** | Gate unterscheidet nicht zwischen Test- und Produktivcode | Gering |

### Kurzübersicht nach Aufwand/Nutzen

| Sofort umsetzbar (gering, hoher Nutzen) | Mittelfristig (mittel) | Struktureller Umbau (hoch) |
|---|---|---|
| 0.1 WAL-Replay-Bounds | 1.3 Distanzpfad-Lock/Allokation | 1.1 HNSW-Nachbarformat |
| 0.2 Bandit-Dimensionsprüfung | 1.5 AES-Schlüsselplan | 1.6 MemTable Range-Sharding |
| 0.3 Drift-Bandit-Kopplung | 1.7 Byte-basierte Cache-Kapazität | 1.9 Text-Posting-Format |
| 0.4 Egress-Kategorisierung | 1.8 build_provenance-Struct | 2.1 Inkrementelle Graph-Kompaktierung |
| 1.2 Backlink-Lookup | 2.2 CSR-Sentinel statt Option | |
| 1.4 SSTable-Zero-Copy-Slice | 2.3 Checkpoint-Index-Merge | |
| 2.5 memfuse-py in Workspace | 2.4 Manifest-Batch-Fsync | |
| 3.1 / 3.2 Governance-Gates | 0.5 Transaktions-Intent-Status | |

---

<a id="18-roadmap"></a>
## 18. Gesamtroadmap

### Stufe 0 — Unmittelbar

1. **Opus-Optimierungen Stufe 0** (§17): WAL-Replay-Panic, Bandit-Dimensionsprüfung, Drift-Kopplung, Egress-Klassifizierung, Intent-Recovery.
2. Loom-Test für Group-Commit sichtbar grün in CI (reine Verifikationslücke).
3. Benchmark-Ausführung des Bandit-Latency-Gates mit produktivem $d$.

### Stufe 1 — Strukturell

4. **Opus-Optimierungen Stufe 1** (§17): HNSW-Hot-Path, SSTable, AES, MemTable, Block-Cache, Provenance, Text-Index, Fusion.
5. **N-äre Hyperkanten** (§6), vollständig spezifiziert, Reihenfolge: H4-Nachweis → Datenmodell (§6.4) → H2 → H1 → H3 → H5 → H6 → Stern-Expansion.
6. HNSW-Dateiformat v2 (Arena + CSR + allokationsfreie Traversierung).

### Stufe 2 — Speicher, Struktur und Produktions-Default-Entscheidungen

7. **Opus-Optimierungen Stufe 2** (§17): Graph-Kompaktierung, CSR-Sentinel, Checkpoint, Manifest, memfuse-py.
8. Bandit-Default `DiagonalApproximation` → `ShermanMorrison` (⚖️ sobald Gate besteht).
9. Block-Cache-Default LRU → SIEVE (⚖️ sobald entschieden).
10. RaBitQ-/PQ-Evaluierung (nach HNSW v2).

### Stufe 3 — Governance und Produktentscheidungen

11. **Opus-Optimierungen Stufe 3** (§17): Feature-Powerset CI, Panic-Inventar.
12. Formale ADR-Revision der Leiden-Umstellung.
13. Major-Release-Planung: DocId-128-Cutover mit DiskANN-Tier-Vollfreigabe bündeln.

### Stufe 4 — Fernziele

14. Memory Consolidation (`consolidate_via_llm()`)
15. CausalEdge
16. Passives WAL-Shipping
17. Vollständige `ProvenanceRecord`-API-Exposition
18. `edge-reinforcement-learning`-Vollspezifikation
19. Automatische NLP-Extraktion n-ärer Fakten aus Freitext

---

<a id="19-matrix"></a>
## 19. Rückverfolgbarkeitsmatrix

| Bereich | Reifegrad | Verweis |
|---|---|---|
| Key-granulare `kv_locks` statt collection-weitem Mutex | 🟢 | §5.1, §5.2a |
| HNSW-SIMD-Hotpath (unaligned Distanzkernel, `AHashSet`-Vorallokation, `try_write()`-Pruning) | 🟢 | §7.4 |
| SQ8-Perzentil-Clipping | 🟢 | §7.4 |
| BM25 residenter Index + Block-Max WAND | 🟢 | §7.3 |
| BM25F feldgewichtete Bewertung | 🟢 | §7.3 |
| Block-Cache: klassisches LRU | 🟢 (Default) | §5.4 |
| Block-Cache: SIEVE/S3-FIFO | 🟡 | §5.4 |
| Sherman-Morrison-Bandit + CI-Latency-Gate | 🟡 (Opt-in) / Gate 🟢 | §8 |
| Leiden statt Label-Propagation (binärer Pfad) | 🟢 | §7.5 |
| RCU-Snapshot-Swap für `CsrGraph::compact()` | 🟢 | §6.3 |
| Native DiskANN-Tombstones | 🟢 | §7.4 |
| DocId-128-Bit-Migration (Rollout) | 🟡 | §6.1 |
| Score-normalisierte Fusion mit RRF-Fallback | 🟡 (Opt-in) | §7.1 |
| Forward-Push-PPR | 🟢 | §7.2 |
| `ProvenanceBuilder`-Struktur | 🟢 | §7.6 |
| FlatBuffers-CI-Drift-Gate | 🟢 | §12 |
| Cloud-Egress Fünf-Schichten (Surrogat, Bulk, Rehydration) | 🟢 | §10.4 |
| KV-Cache-Bridge LSM-Fallback-Spill | 🟢 | §9.2 |
| Bandit-Drift-Alpha-Eskalation gedeckelt | 🟢 | §8.3 |
| Loom-Test sichtbar grün in CI | 🔴 | §15.3 |
| HNSW-Dateiformat v2 (Arena) | 🔴 | §7.4 |
| RaBitQ/PQ-Quantisierung jenseits SQ8 | 🔴 | §7.4 |
| ADR-Formalrevision (Leiden statt LPA) | 🔴 (Dokumentation) | §18 |
| **N-äre Hyperkanten (gesamt: H1–H6, `relate_n_ary`)** | **🔴** | §6 |
| WAL-Replay-Panic-Fix | ⚠️ Opus 0.1 | §5.3, §17 |
| Bandit-Dimensionsprüfung | ⚠️ Opus 0.2 | §8.2, §17 |
| Drift-Bandit-Kopplung verdrahten | ⚠️ Opus 0.3 | §8.2, §17 |
| Egress-Klassifizierung korrigieren | ⚠️ Opus 0.4 | §10.4, §17 |
| Intent-Recovery differenzieren | ⚠️ Opus 0.5 | §5.3, §17 |
| HNSW-Nachbarlisten-Allokation | ⚠️ Opus 1.1 | §7.4, §17 |
| HNSW-Backlink O(1) | ⚠️ Opus 1.2 | §7.4, §17 |
| Distanzpfad Lock/Allokation | ⚠️ Opus 1.3 | §7.4, §17 |
| SSTable Zero-Copy-Slice | ⚠️ Opus 1.4 | §5.5, §17 |
| AES-Schlüsselplan wiederverwenden | ⚠️ Opus 1.5 | §9.3, §17 |
| MemTable Range-Sharding | ⚠️ Opus 1.6 | §5, §17 |
| Block-Cache byte-basiert | ⚠️ Opus 1.7 | §5.4, §17 |
| `build_provenance` Struct | ⚠️ Opus 1.8 | §7.6, §17 |
| Text-Posting-Format | ⚠️ Opus 1.9 | §7.3, §17 |
| Top-k-Selektion | ⚠️ Opus 1.10 | §7.1, §17 |
| Graph inkrementelle Kompaktierung | ⚠️ Opus 2.1 | §6.3, §17 |
| CSR-Sentinel statt Option | ⚠️ Opus 2.2 | §6.3, §17 |
| Checkpoint-Index-Merge | ⚠️ Opus 2.3 | §17 |
| Manifest-Batch-Fsync | ⚠️ Opus 2.4 | §5, §17 |
| `memfuse-py` in Root-Workspace | ⚠️ Opus 2.5 | §9.4, §17 |
| Feature-Powerset CI | ⚠️ Opus 3.1 | §15.4, §17 |
| Panic-Inventar-Gate | ⚠️ Opus 3.2 | §15.4, §17 |

---

<a id="20-migration-v2"></a>
## 20. Migrationsplan v2 und ADR-Übersicht (neu)

Dieser Abschnitt operationalisiert Teil A2 (§A2) und §4.2–§4.3. Er beschreibt, **wie** vom archivierten
Layer-0–5-Zustand (§4.0) in das Ring-Modell (§4.2) überführt wird — als Ergänzung, nicht als Ersatz der in §17/§18
beschriebenen Stabilisierungs- und Optimierungs-Roadmap. Beide Roadmaps laufen nebeneinander: §17/§18 adressiert
Korrektheit/Performance am bestehenden Code, §20 adressiert den Strukturumbau. Wo beide denselben Code betreffen,
gilt Teil A (Ground Truth vor Feature-Ausbau) vor §20 vor §17/§18.

### 20.1 Strangler-Regel (verbindlich)

Neue Crates entstehen **neben** den alten. Alte Crates (`memfuse-core`, `memfuse-db`, `memfuse-candle`,
`memfuse-ollama`, `memfuse-embed`, `memfuse-calibration`) re-exportieren die neuen Symbole mit `#[deprecated]`
für einen vollen Release-Zyklus, bis alle internen und die dokumentierte externe API (`cargo add memfuse-db`,
§2.2) umgestellt sind. Der neue, öffentlich beworbene Fassaden-Crate heißt `memfuse` (`cargo add memfuse`);
`memfuse-db` bleibt als Kompatibilitäts-Re-Export bestehen, bis der Deprecation-Zyklus abgeschlossen ist.

### 20.2 Phasenplan

Umfang: **S** < 1 Woche, **M** 1–3 Wochen, **L** > 3 Wochen (Schätzung nach betroffenen LOC, nicht kalibriert).
Jede Phase ist einzeln auslieferbar; das Exit-Kriterium ist zugleich das Merge-Gate.

| Phase | Umfang | Inhalt | Exit-Kriterium |
|---|---|---|---|
| **0R** Rest der Leitplanken | S | `memfuse-py`-Panic-Test gegen das `maturin`-Release-Wheel; `memfuse-bench` ohne `onnx`-Zwang (bereits umgesetzt); Root-Lints zuerst setzen, danach Vererbung in allen 20+ Crates; `unwrap`/`expect`/`panic`-Prod-Stellen (≈ 11) im selben PR beheben; `.unwrap-baseline.json`-Ratchet löschen; `deny.toml` erweitern (nicht neu anlegen); `tests/layering.rs` im Warnmodus; `audit.toml` bereinigen; `prefill_skip_count`-Fix (§9.2); `memfuse-testkit`-Skelett (`ManualClock`, In-Memory-`StorageEngine`) | `cargo clippy --workspace --all-targets --locked -- -D warnings` grün mit den neuen Lints; 20/20 Crates erben `[workspace.lints]`; `cargo fetch --locked && cargo check --workspace --locked --offline` grün (Netzfreiheit über `default-members`, nicht über `--workspace`, §A2.1 D8); `cargo tree --workspace -e normal -i ort-sys` ohne Treffer; Layering-Test läuft (Warnmodus) |
| **1a** Aufwärtskanten auflösen | M | Fassade `memfuse` + Builder entstehen; `memfuse-db`/`memfuse-engine` erhält `Arc<dyn Embedder>` statt eigener Backend-Konstruktion; `Weak`-Setter → `MetricsSink`/Ports; Kante `router → db` über Ports lösen | `cargo tree -p memfuse-db -e normal` ohne `candle`, `ollama`, `embed`; Layering-Test scharf für Ring 3 |
| **1b** `core`-Zerlegung | M (≈ 10,3k LOC verschoben) | `memfuse-core` → `memfuse-types`/`memfuse-ports`/`memfuse-mvcc`; `memfuse-core` wird `#[deprecated]`-Re-Export; Entscheidung über `saos.rs` (Löschkandidat) | Kein Crate importiert `memfuse_core::` außer dem Re-Export-Test |
| **1c** Unsafe-Inseln zuerst | M | `memfuse-sys`, `memfuse-simd` extrahieren; `memfuse-core-ipc-gen` → `memfuse-wire`; `memfuse-checkpoint` ohne globalen Zustand (P29) | `tests/unsafe_islands.rs` scharf; `memfuse-vector`, `memfuse-store`, `memfuse-db`/`memfuse-engine` tragen `#![forbid(unsafe_code)]` |
| **2** Sync-Kerne | L | `StorageRead` (sync) / `StorageWrite` (async) trennen; Persistenzaufrufe aus `csr.rs`, `inverted.rs`, `diskann.rs` in die Engine verschieben; begrenzter `ComputePool` statt unbegrenztem `spawn_blocking` | `cargo tree -e normal -p memfuse-{vector,text,graph}` ohne `tokio`; Benchmark-p99 ≤ +3 % oder ≤ 2σ der vorher eingefrorenen Baseline |
| **3a** Konsistenz-Spike | S | ADR-N06-Prototyp: Crash-Injektion über `memfuse-testkit`-Fault-VFS (≥ 10⁴ Läufe, deterministischer Seed) | Kriterien: rekonstruierte Indizes = Orakel; `visible_lsn` monoton; keine sichtbare Teilmenge eines Commits |
| **3b** `db` zerlegen | L | Zerlegung in `engine`/`cognition`/`rank`/`adapt`/`router`/`privacy`; acht (real sechs) `hybrid_search_*`-Varianten → `search(SearchRequest)`; `docid-128` entfernt (ADR-N05) | `memfuse-db` nur noch Re-Export; Fassade `memfuse` ≤ 20 `pub fn` (heute 50 in `memfuse-db`) |
| **4** KV echt | L | Stufe A messen → optional B (eigenes Llama-Modell mit `KvState`) → optional C (Segment-Spill) | Gates gemäß §9.2-Tabelle; Spec-Status wechselt erst nach grünem Golden-Test von 🔴 auf 🟡/🟢 |
| **5** Hygiene | M | God-Files zerlegen (`csr`, `hnsw`, `fusion`, `sstable`, `compaction`, `diskann`, `ollama`-Client); Governance-Tag-Kommentare aus dem Quellcode; `xtask` auf < 3.000 LOC; `results/`-Verzeichnis (≈ 25 MB) aus dem Repo; Reifegrad-Marker und Feature-Katalog aus `capabilities.toml` generieren (P12) | Keine Datei > 1.000 Zeilen außerhalb Generat/Tests; alle Spec-Marker generiert, nicht handgepflegt |

### 20.3 ADR-Übersicht (Kontext · Entscheidung · Konsequenzen · Alternativen · Exit-Kriterium je ADR-Dokument unter `docs/decisions/`)

| ADR | Gegenstand | Status |
|---|---|---|
| N01 | Layer-Regeln maschinell (`cargo_metadata`-Test + `cargo-deny`-`wrappers`) | beschlossen, Warnmodus in Phase 0R, scharf ab 1a |
| N02 | Sync-Kern: `StorageRead` sync / `StorageWrite` async, begrenzter `ComputePool` | beschlossen; Benchmark-Gate vor Phase 2 |
| N03 | Drei Unsafe-Inseln (`sys`, `simd`, `wire`), Mechanik `deny`+`forbid` pro Crate | beschlossen; ersetzt die Sechs-Ausnahmen-Regel der Vorfassung vollständig |
| N04 | Panic-Profile: Root `unwind`, `release-abort` nur für Binaries ohne FFI | Profil bereits umgesetzt; Release-Wheel-Test (memfuse-py) offen |
| N05 | Identifikatoren: externes `DocId` (128 Bit) + internes dichtes `DocIdx(u32)` (Option C) | Empfehlung, gekoppelt an N06; **offen** (§A2.4 Nr. 3) |
| N06 | Konsistenzmodell: WAL als einzige Wahrheit, abgeleiteter Zustand statt 2PC-Härtung (Option B) | Empfehlung, Spike-Gate; **offen** (§A2.4 Nr. 1) |
| N07 | KV in Stufen A/B/C, Segmentdateien statt LSM-Spill | Stufe A beschlossen; B/C nach Messung |
| N08 | Generierte Spec und Capability-Manifest (`capabilities.toml`, Testverweis pro Zeile) | beschlossen |
| N09 | Crate-Schnittkriterien I/U/C/S/D (P30) | neu, beschlossen |
| N10 | Kein globaler veränderlicher Zustand (P29) | neu, beschlossen |

### 20.4 Was aus der Vorfassung unverändert beibehalten wird

Um Missverständnisse auszuschließen: Diese Überarbeitung ist ein **Struktur**-Umbau, kein Widerruf der
bewährten Kern-Invarianten. Unverändert bleiben: WAL-First mit HMAC-Kette (P2), MVCC, RCU-Snapshots im
CSR-Graph, Tenant-Isolation, Sandbox mit getrennten Fuel-/Wall-Clock-Budgets (P23), deterministische Recovery
(P3), und die DAG-Doktrin selbst (P5) — sie war immer richtig, sie wurde nur bislang nicht maschinell
erzwungen, was diese Fassung nachholt.

---


*Diese finale konsolidierte Gesamtspezifikation vereinigt Produktvision, Zielarchitektur (jetzt: Ring-Modell,
Teil A2/§4.2), normative Implementierungsvorgaben, algorithmische Spezifikationen, mikrofeingranulare
Schnittstellendefinitionen, die priorisierte Optimierungs-Roadmap und den Migrationsplan v2 (§20) des MemFuse
Cognitive OS. Sie ist in sich geschlossen und ersetzt alle vorherigen Einzeldokumente — einschließlich
`MEMFUSE_ZIELARCHITEKTUR.md` und `MEMFUSE_ZIELARCHITEKTUR_v2.md`, deren Inhalt hiermit in Teil A2, §0, §3, §4
und §9 aufgegangen ist — als maßgebliche Quelle. Künftige Änderungen erfolgen als direkte Überarbeitung dieses
Dokuments, nicht als weiteres Delta-Dokument.*

---

<a id="anhang-b"></a>
# Anhang B — Begründungen, Ist-Zustand, Literatur und Restrisiken je Maßnahme (nachrangig)

> **Rang:** nachrangig zu §0–§20. Dieser Block war in Fassung 2 als eigener Abschnitt „Mikrofeingranulare
> Schnittstellenspezifikation & Systemoptimierung für Memfuse Cognitive OS" zwischen §19 und §20 eingebettet und
> trug Nummern (5.1, 6.3.1, …), die mit dem Hauptteil kollidierten. Ab Fassung 2.1 tragen sie das Präfix `B.`.
> Der Anhang liefert Ist-Zustand, Literatur, Migrationspfad und Restrisiken. **Rust-Skizzen in diesem Anhang sind
> nicht normativ**, wo sie von §5–§10 abweichen; dort gelten §5–§10. Korrigierte Stellen sind mit „[v2.1]"
> markiert. Verweise „§4(n)" und „Invariante n" bezeichnen die Invarianten aus §4.i.

Die vorliegende Spezifikation definiert die mikrofeingranulare Architektur für das Memfuse Cognitive OS. Die Analyse adressiert die Beseitigung struktureller Flaschenhälse in den Bereichen Wissensgraph-Modellierung, Contextual-Bandit-Routing, Cache-Kontention, Vektorindex-Traversierung, LSM-Storage-Engine, Inferenz-Brücken und kryptographischer Sicherheit. Die Lösungsarchitekturen sind so konzipiert, dass sie direkt in deterministischen, threadsicheren Rust-Code überführt werden können, ohne die systemweiten Invarianten zu verletzen.

## B.5.1 N-äre Hyperkanten im Wissensgraphen (`crates/memfuse-graph`)

Die Repräsentation n-ärer Relationen in herkömmlichen Graphdatenbanken führt häufig zu einem semantischen Informationsverlust, wenn komplexe Ereignisse in binäre Subjekt-Prädikat-Objekt-Tripel zerschnitten werden. Die nachfolgenden Spezifikationen definieren die Integration von Hyperkanten in die bestehende Compressed Sparse Row (CSR) Struktur.

### B.5.1.1 — H1: RCU-Snapshot-Integration

**Ist-Zustand im Repo:** `crates/memfuse-graph/src/csr.rs` berechnet die Speicherschätzung des asynchronen `compact()`-Prozesses ausschließlich auf Basis der binären Adjazenzliste, während Hyperkanten als getrennte Datenstruktur außerhalb der atomaren Swap-Grenze modelliert werden.

**Referenzierte Literatur:**

- Yan et al., 2023, "Hypergraph Database Storage", arXiv:2302.06119 — Spezifiziert die Repräsentation von n-ären Relationen in speichereffizienten Bipartit-Graphen zur Optimierung von Subhypergraph-Matching-Verfahren.
    
- Guo et al., 2024, "HyperGraphRAG", arXiv:2503.21322 — Belegt, dass die Isolation von Entitäten und Hyperkanten in parallelen Speicherstrukturen die Retrieval-Genauigkeit in RAG-Systemen signifikant erhöht.
    

**Mathematische/algorithmische Spezifikation [v2.1 korrigiert]:** Die Speicherkosten des RCU-Snapshots müssen
streng deterministisch berechenbar sein, um Allokationsausfälle zu verhindern. Mit $s_R = \text{size\_of}::<\text{RoleBinding}>() = 16$
(nicht 8), $s_I = \text{size\_of}::<\text{HyperEdgeId}>() = 8$, $s_H = \text{size\_of}::<\text{HyperEdge}>()$ (mehr als 100 Byte,
nicht 32) und der Tabellenschranke $T(c) = \text{buckets}(c)\cdot(\text{size\_of}::<(K,V)>() + 1) + 16$ mit
$\text{buckets}(c) = \text{nextpow2}(\lfloor 8c/7 \rfloor + 1)$ gilt

$$S_{\text{total}} = S_{\text{adj}} + \sum_{e \in E_H}\bigl(s_H + \vert e\vert \cdot s_R\bigr) + \sum_{v}\deg_H(v)\cdot s_I + \vert E_H\vert\cdot s_I + \sum_{m} T_m(\text{cap}_m)$$

wobei $\text{cap}_m$ die **Kapazität** (nicht die Länge) jeder Tabelle $m \in \{\text{node},\, H,\, \text{idx}\}$ ist. Die Spitze
beim Rebuild ist $S_{\text{peak}} = S_{\text{total}}^{\text{alt}} + S_{\text{privat}}^{\text{neu}}$ mit vorbelegten Tabellen
($\text{cap} = \text{len}$); geteilte Payloads ($\text{Arc}$) zählen einmal. Normativ ist §6.3.

**Rust-Schnittstelle:** [v2.1] Die frühere Skizze (`scc::HashMap` im Snapshot, `&'a [RoleBinding]`,
`capacity()*32`) ist entfallen. Normativ sind §6.3 (`GraphInner`, `estimate_*`) und §6.4 (`HyperEdge`,
`ArcSlice`, `HyperEdgeView`). Der Snapshot enthält keine Container mit innerer Mutabilität (Snapshot-Regel S1).

**Lock-/Nebenläufigkeitsmodell:** Der Lesezugriff ist durch die Verwendung von `arc_swap::ArcSwap<GraphInner>` vollständig lock-frei ($O(1)$). Modifikationen berechnen einen neuen `GraphInner`-Zustand im Hintergrund und publizieren diesen atomar.

**Invarianten-Nachweis:** Die Definition erfüllt Invariante 5 (Speicherbudget-Transparenz), da die exakte Byte-Berechnung des Hyperkanten-Indizes vor der Kompaktierung evaluiert wird. Invariante 7 (Zero-Copy) wird gewahrt, indem `HyperEdgeView` einen `ArcSlice<RoleBinding>` hält, der sich den `Arc<[RoleBinding]>` des Snapshots teilt (Referenzzähler-Inkrement statt Kopie, §6.4). [v2.1]

**Migrationspfad:** Ein Migrations-Job muss bestehende `GraphInner`-Strukturen deserialisieren und die leeren `hyperedges`-Maps initialisieren, bevor die RCU-Pointer ausgetauscht werden.

**Restrisiken/offene Fragen:** [v2.1] Jede Veröffentlichung klont die Map-Strukturen (O(Hyperkanten + Entitäten) Referenzzähler-Inkremente, keine Payload-Kopie). Wachstum einer nicht vorbelegten Tabelle kann die Schätzung um die Verdopplungsdifferenz überschreiten; deshalb MUSS der Rebuild vorbelegen (§6.3).

### B.5.1.2 — H2: Deadlock-freies Multi-Key-Locking

**Ist-Zustand im Repo:** Naives Locking über `kv_locks` für eine Hyperkante mit $N$ Teilnehmern führt zu zyklischen Wartebedingungen, wenn zwei überlappende Transaktionen die Locks in unterschiedlicher Reihenfolge anfordern.

**Referenzierte Literatur:**

- Sarkar et al., 2020, "LSM-tree compaction", arXiv:2202.04522 — Analysiert Nebenläufigkeitskontrollen in skalierbaren Speicherarchitekturen und totale Ordnungen in Lock-Hierarchien.
    

**Mathematische/algorithmische Spezifikation:** Die Deadlock-Freiheit bei der Belegung von $N$ unabhängigen Schlüsseln erfordert die Einhaltung einer totalen Ordnung $\leq_{L}$ über die Sperrenmenge $L$. Sei $h: \text{EntityId} \to \mathbb{N}$ eine eindeutige Hash-Abbildung auf den Shard-Index. Für eine Menge von Entitäten $E = \{e_1, \dots, e_N\}$ wird die Sperrsequenz $S = \text{sort}(\{h(e_i) \mid e_i \in E\})$ generiert. Doppelte Shard-Indizes werden entfernt. Die Komplexität für die Akquise beträgt im Worst-Case $O(N \log N)$ für die Sortierung und $O(K)$ für das Sperren, wobei $K \leq N$ die Anzahl der betroffenen Shards ist.

**Rust-Schnittstelle (normativ):**

Rust

```
use std::sync::RwLockWriteGuard;

#[derive(Debug, thiserror::Error)]
pub enum LockError {
    #[error("lock poisoned")]
    Poisoned,
    #[error("lock acquisition timed out")]
    Timeout,
}

pub struct MultiKeyGuard<'a> {
    _guards: Vec<RwLockWriteGuard<'a, ()>>,
}

impl KvKeyLocks {
    pub fn acquire_multi_sorted(&self, sorted_key_hashes: &[u64]) -> Result<MultiKeyGuard<'_>, LockError> {
        let mut shard_indices: Vec<usize> = sorted_key_hashes.iter()
            .map(|&h| (h & self.shard_mask) as usize)
            .collect();
        shard_indices.sort_unstable();
        shard_indices.dedup();
        
        let mut guards = Vec::with_capacity(shard_indices.len());
        for idx in shard_indices {
            guards.push(self.shards[idx].write().map_err(|_| LockError::Poisoned)?);
        }
        Ok(MultiKeyGuard { _guards: guards })
    }
}
```

**Lock-/Nebenläufigkeitsmodell:** Durch die Erzwingung einer monoton steigenden Erwerbsreihenfolge über die physischen Shard-Indizes wird die Entstehung von Zyklen im Betriebsmittel-Zuweisungsgraphen mathematisch ausgeschlossen.

**Invarianten-Nachweis:** Erfüllt strikt Invariante 4 (Deadlockfreiheit bei Multi-Key-Locking) durch den Beweis der totalen Ordnung vor der Lock-Akquise.

**Migrationspfad:** Sämtliche Mutations-APIs (`relate_n_ary`), die mehr als eine Entität berühren, müssen verbindlich auf `acquire_multi_sorted` umgestellt werden.

**Restrisiken/offene Fragen:** Eine hohe Kollisionsrate (viele Entitäten hashen auf denselben Shard) reduziert die Parallelität, was ein Re-Tuning der `shard_mask` bei wachsender Graphgröße erfordert.

### B.5.1.3 — H3: Atomare Multi-Entity-Registrierung (`relate_n_ary`)

**Ist-Zustand im Repo:** Es existiert keine Transaktionsklammer, die einen Write-Ahead-Log (WAL) Eintrag für $N$ Graph-Knoten atomar in das LSM-System flusht und gleichzeitig den RCU-Graphen aktualisiert.

**Referenzierte Literatur:**

- Yan et al., 2023, "Hypergraph Database Storage", arXiv:2302.06119 — Spezifiziert atomare Schreiboperationen in n-ären Relationen über strukturierte Delta-Logs.
    

**Mathematische/algorithmische Spezifikation:** Die Funktion `relate_n_ary` operiert als logische Transaktion. Die Atomarität wird durch die Vorab-Allokation einer deterministischen `TxId` und das sequentielle Schreiben der Tupel $(e_i, \text{HyperEdgeId})$ in das WAL unter einem einzelnen Group-Commit gewährleistet. Die Komplexität ist $O(\vert{}P\vert{} \cdot \log(\text{MemTable}))$, wobei $\vert{}P\vert{}$ die Anzahl der Teilnehmer ist.

**Rust-Schnittstelle (normativ):**

Rust

```
use memfuse_core::{DocId, TxId};

pub trait GraphCollectionMutation {
    fn relate_n_ary(
        &self,
        predicate_tag: u32,
        participants: &[RoleBinding],
        doc_id: DocId,
    ) -> Result<HyperEdgeId, GraphMutationError>;
}
```

**Lock-/Nebenläufigkeitsmodell:** Exklusive Write-Sperren werden über `acquire_multi_sorted` (H2) auf Entitätsebene gehalten, bis der `fsync` in das WAL erfolgreich beendet wurde. Rollback erfolgt durch Löschung der unvollständigen In-Memory-Einträge bei I/O-Fehlern.

**Invarianten-Nachweis:** §4(3) Determinismus wird eingehalten, da die Transaktionsgenerierung unabhängig von der Thread-Ausführung sequentiell geordnet ist.

**Migrationspfad:** Das offene Enum `SignalKind` wird nicht modifiziert. Hyperkanten-Treffer fließen additiv als `SignalKind::Graph` in die Ranking-Fusion ein.

**Restrisiken/offene Fragen:** Lange Transaktionen durch I/O-Latenz beim WAL-Flush blockieren konkurrierende Leseoperationen auf den betroffenen Entitäts-Shards.

### B.5.1.4 — H4: Nachweispflicht vor Implementierung

**Ist-Zustand im Repo:** Es fehlt ein analytischer Nachweis, ob die Cliquen-Expansion oder eine native Bipartit-Darstellung für die Nachbarschaftstraversierung optimal ist.

**Referenzierte Literatur:**

- Guo et al., 2024, "HyperGraphRAG", arXiv:2503.21322 — Bipartite Transformation von Hypergraphen für effizientes RAG.
    

**Mathematische/algorithmische Spezifikation:** Bei der Cliquen-Expansion einer Hyperkante $e$ mit Fan-out $N$ entstehen $\frac{N(N-1)}{2}$ binäre Kanten. Die Traversierung eines Knotens $v \in e$ kostet $O(N)$. In der bipartiten Stern-Expansion (ein künstlicher Knoten $v_e$ pro Hyperkante, verbunden mit allen $v \in e$) entstehen exakt $N$ Kanten. Die Traversierung von $v$ zu allen Nachbarn in $e$ erfolgt über $v_e$ in zwei Hops und kostet ebenfalls $O(N)$. Da die Speicherkomplexität der Stern-Expansion jedoch $O(N)$ gegenüber $O(N^2)$ beträgt, ist die bipartite Repräsentation (Stern-Expansion) für Speicherung und Traversierung zwingend vorzuziehen.

|**Metrik**|**Cliquen-Expansion**|**Bipartite Repräsentation (Stern)**|
|---|---|---|
|Kantenanzahl|$O(N^2)$|$O(N)$|
|Speicherplatz|Hoch|Minimal|
|Pfadlänge|1 Hop|2 Hops|
|Traversierung|$O(N)$|$O(N)$|

**Rust-Schnittstelle (normativ):** Keine direkte API-Schnittstelle; dies ist eine architekturelle Entscheidungsvorgabe.

**Lock-/Nebenläufigkeitsmodell:** Lese-Pfad über RCU (`ArcSwap`) erfordert keine Anpassung der Locks für Zwei-Hop-Traversierungen.

**Invarianten-Nachweis:** Erfüllt Invariante 6 ($O(\text{Seed})$ statt $O(\text{Graph})$), da die Traversierung durch den Nachbarschaftsgrad $N$ limitiert bleibt und nicht quadratisch explodiert.

**Migrationspfad:** Das Schema für Hyperkanten muss die `flatbuffers-drift-gate` in CI erfolgreich passieren, bevor diese Struktur eingeführt wird.

**Restrisiken/offene Fragen:** Die Zwei-Hop-Semantik verlängert die effektive Pfadtiefe in GraphRAG-Algorithmen, was Anpassungen in der Decay-Funktion beim Forward-Push Personalized PageRank erfordert.

### B.5.1.5 — H5: Kaskadierende Invalidierung ohne Kostenexplosion

**Ist-Zustand im Repo:** Die Löschung eines Dokuments löst eine ungebundene Kaskade von Invalidierungen aus. Bei Hyperkanten mit tausenden Teilnehmern führt dies zur Blockade des Main-Threads.

**Referenzierte Literatur:**

- Sarkar et al., 2020, "LSM-tree compaction", arXiv:2202.04522 — Analysiert Tombstone-Propagierung in Speichersystemen.
    

**Mathematische/algorithmische Spezifikation:** Um eine $O(N^2)$-Kostenexplosion bei der Kaskadenlöschung zu verhindern, wird die Löschmenge $D$ evaluiert. Ist $\vert{}D\vert{} \leq \theta$ (mit $\theta = 1000$), erfolgt die Löschung (Tombstone-Schreibung) synchron, $O(\vert{}D\vert{})$. Ist $\vert{}D\vert{} > \theta$, wird die Menge $D$ an der Grenze $\theta$ geteilt. Die ersten $\theta$ Elemente werden synchron verarbeitet. Der Rest $D \setminus D_{\theta}$ wird als asynchroner Task in die Background-Queue delegiert, wodurch die synchrone Latenz konstant $O(\theta)$ wird.

**Rust-Schnittstelle (normativ):**

Rust

```
use memfuse_core::DocId;

pub const DEFAULT_HYPEREDGE_CASCADE_FANOUT_LIMIT: usize = 1_000;

pub struct CascadeReport { // [v2.1] vollständige Fassung in §6.6 H5 (ticket, deletion_proof)
    pub tombstoned_synchronously: usize,
    pub queued_for_background: usize,
}

pub fn cascade_invalidate_hyperedges_for_superseded_doc(
    graph: &CsrGraph,
    doc_id: DocId,
    fanout_limit: usize,
) -> Result<CascadeReport, GraphMutationError>;
```

**Lock-/Nebenläufigkeitsmodell:** Der asynchrone Worker akquiriert Locks in kleinen Batches, um den RCU-Lese-Pfad nicht zu blockieren.

**Invarianten-Nachweis:** §4(6) Die Ausführungszeit der synchronen Funktion ist strikt durch das Fan-out-Limit $\theta$ nach oben beschränkt, was System-Latenz-Spikes verhindert.

**Migrationspfad:** Default-Aktivierung des Background-Workers beim Hochfahren der Memfuse-Engine.

**Restrisiken/offene Fragen:** Abstürze während der asynchronen Verarbeitung können verwaiste Hyperkanten hinterlassen. [v2.1] Die Delete-Queue ist persistent und idempotent, normativ in §6.6 H5. Zur DLQ-Replay-Logik siehe §B.6.3.1.

### B.5.1.6 — H6: Projektion auf binäre Kantengewichte für Community Detection (Leiden)

**Ist-Zustand im Repo:** Der Leiden-Algorithmus iteriert über die binären Kanten. Hyperkanten werden durch `hyperedges_included: false` ignoriert.

**Referenzierte Literatur:**

- Traag et al., 2019, "From Louvain to Leiden: guaranteeing well-connected communities", arXiv:1810.08473 — Referenz zur Maximierung der Graph-Modularität in komplexen Netzwerken.
    

**Mathematische/algorithmische Spezifikation:** Für den Leiden-Algorithmus, der auf die Maximierung der Modularität $Q$ ausgelegt ist, werden Hyperkanten über einen Iterator als bipartiter Graph projiziert. Um zu verhindern, dass große Hyperkanten das Modularity-Clustering dominieren, wird das Gewicht $w(u, v_e)$ zwischen Teilnehmer $u$ und Hyperkanten-Knoten $v_e$ skaliert:

$$w(u, v_e) = \frac{2\,w(e)}{\vert{}e\vert{} - 1}$$ [v2.1: Konvention K, siehe §6.6 H6; Fassung 2 hatte $w(e)/(\vert{}e\vert{}-1)$]

Die Komplexität der Iteration bleibt $O(\vert{}E_B\vert{}) = O(\sum_{e \in E_H} \vert{}e\vert{})$.

**Rust-Schnittstelle:** [v2.1] Normativ ist §7.5 (`StarExpansionIterator` über `Arc<GraphInner>`, Item `StarEdge`).

**Lock-/Nebenläufigkeitsmodell:** Der Iterator arbeitet lock-frei auf einer unveränderlichen RCU-Snapshot-Referenz (`Arc`).

**Invarianten-Nachweis:** §4(7) Zero-Copy. Der Iterator generiert die virtuellen Kanten "on-the-fly" ohne Allokation einer neuen Adjazenzmatrix im Speicher.

**Migrationspfad:** Der `CommunityDetectionConfig` Struct wird um `hyperedges_included: bool` ergänzt, was standardmäßig auf `false` verbleibt, bis die Validierung gegen Benchmark-Netzwerke abgeschlossen ist.

**Restrisiken/offene Fragen:** Virtuelle Knoten im Leiden-Algorithmus verändern die Modularity-Resolution. Ein Hyperparameter-Tuning des Resolution-Parameters $\gamma$ ist für Netzwerke mit hoher Hyperkanten-Dichte zwingend.

### B.5.1.7 — GC: Speicherverlust durch verwaiste Knoten und defekte Kaskaden

**Ist-Zustand im Repo:** Unvollständige Löschungen hinterlassen Knoten ohne aktive Kanten, was den Speicherbedarf über Zeit aufbläht.

**Referenzierte Literatur:**

- Epoch-based reclamation Techniken analog zu Keir Fraser's EBR-Konzepten (implizit in `crossbeam-epoch`).
    

**Mathematische/algorithmische Spezifikation:** Die Garbage Collection identifiziert Knoten $v$, für die gilt: $\text{deg}_{\text{in}}(v) + \text{deg}_{\text{out}}(v) == 0$. Die Reklamation erfolgt Epochen-basiert. In der `compact()`-Phase wird der Graph gescannt ($O(\vert{}V\vert{})$). Verwaiste Knoten werden nicht in den neuen `ArcSwap`-Snapshot übernommen.

**Rust-Schnittstelle (normativ):**

Rust

```
pub trait GraphGarbageCollection {
    fn sweep_orphans(&self) -> Result<usize, GraphMutationError>;
}
```

**Lock-/Nebenläufigkeitsmodell:** `sweep_orphans` akquiriert den globalen Schreib-Lock für den neuen Snapshot, beeinträchtigt aber nicht die Leseprozesse auf dem aktiven Snapshot.

**Invarianten-Nachweis:** §4(5) Transparenz des Speicherbudgets wird durch die Freigabe des Speichers beim Austausch der Epochen sichergestellt.

**Migrationspfad:** Hintergrund-Cronjob implementieren, der `sweep_orphans` bei geringer Systemlast aufruft.

**Restrisiken/offene Fragen:** Bei sehr großen Graphen kann der $O(\vert{}V\vert{})$-Scan zu CPU-Spikes führen.

## B.5.2 Contextual-Bandit-Routing (LinUCB) (`crates/memfuse-router`)

Das Contextual-Bandit-Modell entscheidet adaptiv über die Retrieval-Strategien. Die aktuelle Implementierung untergräbt jedoch die mathematischen Garantien des LinUCB-Algorithmus.

### B.5.2.1 — Falsche Mathematik im Produktions-Default (Diagonal-Approximation)

**Ist-Zustand im Repo:** Die `DiagonalApproximation` aktualisiert die Kovarianzmatrix-Diagonale iterativ als $\sigma^2_i \mathrel{+}= x_i^2$ und akkumuliert Parameter als $\theta_i \mathrel{+}= r \cdot x_i / \max(\sigma^2_i, 10^{-8})$. Dies ist eine Form der stochastischen Gradientenabstieg-Optimierung (SGD), aber keine echte Ridge-Regression.

**Referenzierte Literatur:**

- Li et al., 2010, "A Contextual-Bandit Approach to Personalized News Article Recommendation" — Etabliert den LinUCB-Standard.
    
- Zang et al., 2022, arXiv:2201.09910 — "diagonal approximation lacks theoretical justification" und bricht die Regret-Bounds.
    

**Mathematische/algorithmische Spezifikation:** Die echte LinUCB-Schranke erfordert $\theta = A^{-1}b$. Die aktuelle Implementierung verfehlt dies, da die Updates von $A$ (oder dessen Diagonale) nicht retroaktiv auf die bisher akkumulierten Werte in $b$ angewendet werden. Die Regret-Garantie von $O(d \sqrt{T \log T})$ zerfällt unter der Diagonal-Approximation für korrelierte Features zu einem linearen Regret $O(T)$ im Worst-Case. Um minimale Korrektheit zu wahren, muss $b$ separat akkumuliert und $\theta$ bei jeder Anfrage als $\theta_i = b_i / \sigma^2_i$ berechnet werden.

**Rust-Schnittstelle (normativ):**

Rust

```
pub struct CorrectedDiagonalBandit {
    pub precision_diag: Vec<f32>, // A_diag
    pub b: Vec<f32>,
    pub theta: Vec<f32>,
    pub alpha: f32,
}

impl CorrectedDiagonalBandit {
    pub fn update(&mut self, context: &[f32], reward: f32) {
        // ... update precision_diag and b, then compute theta = b / precision_diag
        unimplemented!()
    }
}
```

**Lock-/Nebenläufigkeitsmodell:** Mutationen sind sequenziell.

**Invarianten-Nachweis:** §4(3) Determinismus bleibt gewahrt.

**Migrationspfad:** Unmittelbares Update der bestehenden Struktur.

**Restrisiken/offene Fragen:** Die Diagonale ignoriert Feature-Korrelationen bei dichten LLM-Embeddings.

### B.5.2.2 — Sherman-Morrison-Update als Performance-Blocker (SIMD)

**Ist-Zustand im Repo:** Die korrekte Matrixinversion via Sherman-Morrison ist hinter dem Feature-Flag `egress-sherman-morrison` versteckt, da die $O(d^2)$-Operation ohne SIMD zu langsam ist.

**Referenzierte Literatur:**

- arXiv:2501.13139, "Efficient LinearUCB for Embedded Learning Systems" — Optimierung durch Sherman-Morrison und SIMD-Vektorisierung.
    

**Mathematische/algorithmische Spezifikation:** Die Sherman-Morrison-Formel für ein Rang-1-Update lautet:

$$(A + xx^T)^{-1} = A^{-1} - \frac{A^{-1}xx^T A^{-1}}{1 + x^T A^{-1} x}$$

Um die Latenz zu drücken, erfordert dies Vektorisierung. Die $d \times d$ Matrix muss cache-aligned ($64$ Byte) im Row-Major-Format im Speicher liegen, um False Sharing zu vermeiden. Die Berechnung von $v = A^{-1}x$ und das Update werden durch `fma` (Fused Multiply-Add) SIMD-Instruktionen beschleunigt. Komplexität: $O(d^2 / W)$, wobei $W=8$ für 256-Bit AVX.

**Rust-Schnittstelle:** [v2.1] Die frühere Skizze (`[f32; D * D]`, `x.data.len() != D`, `unsafe` im Router) ist entfallen: Sie kompiliert auf stable nicht und die Dimensionsprüfung war wirkungslos. Normativ ist §8.2 (Laufzeit-Dimension, Safe Rust).

**Lock-/Nebenläufigkeitsmodell:** Thread-lokale Ausführung ohne I/O.

**Invarianten-Nachweis:** §4(2) [v2.1] `memfuse-router` bleibt `forbid(unsafe_code)`; SIMD-Kerne kämen als sichere API aus `memfuse-simd` (Unsafe-Insel). §4(1) Harte `Result`-Dimensionsprüfung, kein `.unwrap()`.

**Migrationspfad:** CI-Gate für Latenz validieren, dann Feature-Flag `egress-sherman-morrison` zum Standard erheben.

**Restrisiken/offene Fragen:** Floating-Point-Präzisionsverlust über Millionen von Updates. Ein periodischer Cholesky-Rebuild von Grund auf ist empfehlenswert.

### B.5.2.3 — Fehlende Drift-Bandit-Kopplung

**Ist-Zustand im Repo:** Ein `LyapunovDriftWatcher` erkennt Konzeptdrift in der Feature-Verteilung, löst aber keine Parameteranpassung im Router aus.

**Referenzierte Literatur:**

- Wu et al., 2020, "Non-stationary contextual bandit" — Methoden zur Anpassung von Exploration unter Drift.
    

**Mathematische/algorithmische Spezifikation:** Wird Drift detektiert, muss die Exploration kurzzeitig eskalieren und das Vertrauen in alte Daten verringert werden. Eskalationsformel: $\alpha_t = \min(\alpha_{t-1} \cdot k_{\text{drift}}, \alpha_{\text{max}})$, mit Decay in Folgerunden. Discounting [v2.1 korrigiert]: $A \leftarrow \gamma A$, $b \leftarrow \gamma b$ mit $\gamma \in (0, 1)$, äquivalent $A^{-1} \leftarrow \gamma^{-1} A^{-1}$ (die Unsicherheit wächst, $\theta = A^{-1}b$ bleibt unverändert). Fassung 2 schrieb $A^{-1} \leftarrow \gamma A^{-1}$; das verkleinert die Unsicherheit.

**Rust-Schnittstelle (normativ):**

Rust

```
pub trait BanditPolicy: Send + Sync {
    fn apply_drift_penalty(&mut self, k_drift: f32, alpha_max: f32, gamma: f32);
}
```

**Lock-/Nebenläufigkeitsmodell:** Der Drift-Monitor benachrichtigt den Banditen asynchron über einen MPSC-Channel, um Latenz-Spikes im Inferenz-Pfad zu vermeiden.

**Invarianten-Nachweis:** §4(3) Determinismus der Updates bleibt durch Kanal-Synchronisation erhalten.

**Migrationspfad:** Schnittstelle in `BanditPolicy` implementieren und Channel-Listener im Main-Event-Loop aktivieren.

**Restrisiken/offene Fragen:** Aggressives $\gamma$ kann zu kurzzeitig extrem instabilen Routing-Entscheidungen führen.

### B.5.2.4 — Über-Fitting/Regret-Fehler generell (Off-Policy-Schätzung)

**Ist-Zustand im Repo:** Fehlendes Online-Monitoring der Banditen-Performance.

**Referenzierte Literatur:**

- Joachims et al., 2015, "Counterfactual Risk Minimization", arXiv:1502.02362 — IPS-Methodik.
    

**Mathematische/algorithmische Spezifikation:** Die kontrafaktische Evaluation einer neuen Policy $\pi_{\text{new}}$ aus geloggten Daten der Policy $\pi_{\text{old}}$ erfolgt über Inverse Propensity Scoring (IPS):

$$V_{\text{IPS}}(\pi_{\text{new}}) = \frac{1}{t} \sum_{i=1}^t r_i \frac{\mathbb{I}(\pi_{\text{new}}(x_i) == a_i)}{P_{\pi_{\text{old}}}(a_i \mid x_i)}$$

Der Nenner wird auf $\max(p, 0.01)$ geklemmt, um Varianz-Explosionen zu dämpfen. Komplexität: $O(t)$. **[v2.1] Voraussetzung:** IPS braucht eine randomisierte Logging-Policy mit Propensity-Untergrenze; ein deterministisches Argmax-LinUCB hat Propensity 1 und macht IPS wertlos (§8.5).

**Rust-Schnittstelle (normativ):**

Rust

```
pub struct OffPolicyEvaluator {
    cumulative_ips: f64,
    samples: u64,
}

impl OffPolicyEvaluator {
    pub fn observe(&mut self, target_action: u32, logged_action: u32, propensity: f32, reward: f32) {
        if target_action == logged_action {
            let p = propensity.max(0.01);
            self.cumulative_ips += (reward / p) as f64;
        }
        self.samples += 1;
    }
}
```

**Lock-/Nebenläufigkeitsmodell:** Lock-freier Akkumulator (Atomic oder thread-lokal).

**Invarianten-Nachweis:** §4(5) Fester Speicherverbrauch (zwei Skalare), keine unsichtbaren Allokationen.

**Migrationspfad:** Die WAL-Struktur muss Propensity-Werte bei jedem Logging mitschreiben.

**Restrisiken/offene Fragen:** IPS ist bias-anfällig, wenn Propensities stark von der Gleichverteilung abweichen.

## B.5.3 Block-Cache Lock-Kontention (`crates/memfuse-store`)

Der Cache-Layer ist entscheidend für das LSM-Tree-Leseverhalten.

### B.5.3.1 — Ineffizienter Default (LRU Lock-Kontention)

**Ist-Zustand im Repo:** `LruBlockCacheBackend` nutzt `RwLock`. Jeder Read-Hit mutiert die Double-Linked-List zur Aktualisierung der Recency und erzwingt einen exklusiven Write-Lock.

**Referenzierte Literatur:**

- Zhang et al., 2024, "SIEVE is Simpler than LRU", NSDI 2024 / arXiv:2312.13123.
    

**Mathematische/algorithmische Spezifikation:** Unter Last verhält sich der RWLock nach dem Gesetz von Amdahl als starker Flaschenhals. Die zu erwartende Wartezeit steigt quadratisch mit der Thread-Anzahl $T$, proportional zu $p_{\text{hit}}^2$.

### B.5.3.2 — SIEVE-Alternative als lock-freier Standard

**Ist-Zustand im Repo:** `quick_cache` (S3-FIFO) ist hinter `block-cache-v2` verfügbar.

**Mathematische/algorithmische Spezifikation:** SIEVE eliminiert List-Reordering beim Read-Hit vollständig (Erfüllung von P25). Jeder Knoten trägt ein atomares `visited`-Bit. Bei einem Hit wird das Bit mit `Ordering::Relaxed` gesetzt ($O(1)$ lock-frei). Die Verdrängung (Eviction) nutzt einen umlaufenden Zeiger (`hand`). Ist das Cache-Limit erreicht, wird `hand` bewegt. Ist `visited == 1`, wird es auf $0$ gesetzt und der Knoten bleibt. Ist `visited == 0`, wird der Knoten entfernt. Worst-Case-Eviction-Komplexität: $O(C)$ wobei $C$ die Cache-Größe ist, Average-Case $O(1)$.

**Rust-Schnittstelle:** [v2.1] Die frühere `crossbeam-epoch`-Skizze ist entfallen (braucht `unsafe` im Safe-Crate `memfuse-store`; `Shared<'static,_>` ist `!Send`/`!Sync`). Normativ ist §5.4: `scc::HashMap<K, Arc<SieveNode<V>>>` mit `visited` im Node und Mutex nur im Miss-/Evict-Pfad. `get` ist lock-arm, nicht wait-free.

**Invarianten-Nachweis:** §4(2) kein `unsafe`; §4(4) der Mutex wird nie während `get` gehalten.

**Migrationspfad:** Benchmark in CI gegen `quick_cache`. Bei Erfolg Flag `block-cache-v2` zur SIEVE-Implementation umleiten.

**Restrisiken/offene Fragen:** SIEVE bietet keinen dedizierten Schutz gegen sequenzielle Scans (Scan-Resistance), was bei großen Bereichsabfragen den Cache flushen kann.

### B.5.3.3 — Byte-basierte statt eintragsbasierte Kapazität

**Ist-Zustand im Repo:** Kapazität basiert auf der Element-Anzahl, was bei variablen Werten (Texte, Arrays) unberechenbaren Speicherverbrauch erzeugt.

**Mathematische/algorithmische Spezifikation:** Die Cache-Kapazität wird als $C_{\text{bytes}}$ definiert. Beim Einfügen eines Elements mit Größe $s$ wird atomar $S_{\text{current}} \mathrel{+}= s$ gerechnet. Wenn $S_{\text{current}} > C_{\text{bytes}}$, ruft SIEVE so lange Eviction auf, bis die Bedingung wieder erfüllt ist.

**Rust-Schnittstelle (normativ):**

Rust

```
impl<K, V> SieveCacheBackend<K, V> {
    pub fn capacity(&self) -> usize; // Return max bytes
    pub fn current_size(&self) -> usize; // Return current bytes via Relaxed AtomicUsize
}
```

**Lock-/Nebenläufigkeitsmodell:** Lock-frei mittels `fetch_add` / `fetch_sub`.

**Invarianten-Nachweis:** §4(5) Transparenz des Speicherbudgets.

**Migrationspfad:** Alle Insertion-Pfade müssen die Funktion zur Byte-Größen-Schätzung des jeweiligen Typs implementieren.

**Restrisiken/offene Fragen:** Ungenauigkeiten bei der Schätzung des Struct-Overheads im RAM.

## B.5.4 GraphRAG & Community Detection — vertieft

### B.5.4.1 — Vollständiger mathematischer Übergang auf binäre Gewichte

**Ist-Zustand im Repo:** Hyperkanten werden vom Leiden-Algorithmus ignoriert.

**Referenzierte Literatur:**

- Traag et al., 2019, "From Louvain to Leiden".
    

**Mathematische/algorithmische Spezifikation:** Um Hyperkanten $e \in E_H$ in den binären Leiden-Solver zu integrieren, ohne die Modularitätsberechnung $Q$ zu verzerren, nutzen wir eine Stern-Expansion. Sei $v_e$ ein synthetischer Knoten für $e$. Für jedes $u \in e$ entsteht eine Kante $(u, v_e)$ mit dem normalisierten Gewicht:

$$w(u, v_e) = \frac{2\,w(e)}{\vert{}e\vert{} - 1}$$ [v2.1: Konvention K, siehe §6.6 H6; Fassung 2 hatte $w(e)/(\vert{}e\vert{}-1)$]

[v2.1: der frühere „Beweis der Informationserhaltung" war falsch — die Teilnehmergradsumme des Sterns $\vert{}e\vert{}\,w/(\vert{}e\vert{}-1)$ entspricht nur für $\vert{}e\vert{}=2$ der Clique mit Paargewicht $w$.] Korrekte Aussage: Mit Referenz-Clique $p = w/\binom{\vert{}e\vert{}}{2}$ und Sternkante $a = \vert{}e\vert{}\,p = 2w/(\vert{}e\vert{}-1)$ ist das Schur-Komplement des Sterns exakt die Clique-Laplace-Matrix (Beweis und Grenzen in §6.6 H6; die Modularität bleibt nur näherungsweise erhalten). Rechenkosten: $O(\vert{}e\vert{})$ für den Stern gegenüber $O(\vert{}e\vert{}^2)$ für die Clique.

**Rust-Schnittstelle (normativ):**

Rust

```
// Iterator implementiert in 5.1.6.
```

**Lock-/Nebenläufigkeitsmodell:** Lock-freier Iterator über Snapshot.

**Invarianten-Nachweis:** §4(6) Komplexität korreliert mit Kantenanzahl, keine $N^2$ Explosion.

**Migrationspfad:** Konfiguration über `CommunityDetectionConfig`.

**Restrisiken/offene Fragen:** Das Einbringen virtueller Knoten reduziert künstlich die Dichte des Netzwerks, was die intrinsische Resolution $\gamma$ des Leiden-Algorithmus verschiebt.

## B.5.5 Vektorindex-Traversierung: HNSW-Dateiformat v2 (`crates/memfuse-index`)

Der Suchpfad in HNSW leidet unter Speicher-Ineffizienzen.

### B.5.5.1 — Allokations-Overhead pro Knoten (Arena-Modell)

**Ist-Zustand im Repo:** Traversierung allokiert `Vec<u32>` pro Knoten (`Cow::Owned`), was den GC und Allocator massiv belastet.

**Referenzierte Literatur:**

- Malkov & Yashunin, 2020, HNSW Originalkonzepte in flat memory layouts.
    

**Mathematische/algorithmische Spezifikation:** Um Allokationen zu eliminieren, wird eine Mmap-gestützte Arena-Struktur implementiert. Adjazenzlisten werden pro HNSW-Layer $l$ als Compressed Sparse Row (CSR) `offsets_l` und `targets_l` abgespeichert. Ein Zugriff auf Nachbarn von Knoten $i$ im Layer $l$ benötigt zwei Array-Lookups: `start = offsets_l[i]`, `end = offsets_l[i+1]`. Zeit: $O(1)$, Alloc: 0.

**Rust-Schnittstelle (normativ):**

Rust

```
pub struct HnswArenaView<'m> {
    pub arena_vectors: &'m [f32],
    pub layer_offsets: Box<[&'m [u32]]>,
    pub layer_targets: Box<[&'m [u32]]>,
    pub stride: usize,
}

impl<'m> HnswArenaView<'m> {
    #[inline(always)]
    pub fn get_neighbors(&self, node: u32, layer: u8) -> &'m [u32] {
        let offsets = self.layer_offsets[layer as usize];
        let targets = self.layer_targets[layer as usize];
        &targets[offsets[node as usize] as usize .. offsets[node as usize + 1] as usize]
    }
}
```

**Lock-/Nebenläufigkeitsmodell:** Lesezugriffe sind vollständig parallelisierbar, da die Mmap unveränderlich ist.

**Invarianten-Nachweis:** §4(7) Zero-Copy erfüllt. Die Slice-Referenz referenziert den Speicher der Mmap direkt.

**Migrationspfad:** Binär inkompatibles Format. Ein `HNSW_VERSION=2` Header wird eingeführt; ein Hintergrund-Prozess re-indiziert alte Vektoren.

**Restrisiken/offene Fragen:** Inserts erfordern einen RAM-Overlay (Chunked Allocation), der beim Kompaktieren periodisch in die Datei zurückgeschrieben wird.

### B.5.5.2 — Backlink-Lookup nicht O(1)

**Ist-Zustand im Repo:** Lineare Iteration bei der Backlink-Auflösung in Batch-Inserts $O(P \times B)$.

**Mathematische/algorithmische Spezifikation:** Die Auflösung erfordert einen $O(1)$ Hash-Lookup. Eine temporäre HashMap wird am Start der Batch-Verarbeitung generiert. Der Schlüssel ist ein bit-gepackter `u64` bestehend aus `ram_idx` (32 Bit) und `layer` (8 Bit). Zeitkomplexität fällt auf $O(1)$ je Schritt.

**Rust-Schnittstelle (normativ):**

Rust

```
#[derive(Default)]
pub struct SearchScratch {
    pub overlay_backlinks: ahash::AHashMap<u64, &'static [u32]>,
}
```

**Lock-/Nebenläufigkeitsmodell:** Thread-lokaler Scratch-Puffer, lock-frei.

**Invarianten-Nachweis:** §4(1) Keine impliziten Panics durch Boundary Checks, saubere Hash-Ergebnisse.

**Migrationspfad:** Sofort ersetzbar im Insert-Pipeline-Code.

**Restrisiken/offene Fragen:** Hashmap Allokation für extrem kleine Batches eventuell überproportional teuer.

### B.5.5.3 — Distanzpfad Lock/Allokation

**Ist-Zustand im Repo:** Mmap Vektoren werden elementweise gelesen und dekodiert; Quantisierer sperren den Lesevorgang.

**Mathematische/algorithmische Spezifikation:** Der Vektorzugriff muss als konstanter Slice direkt an die SIMD-Engine gereicht werden. Die Quantisierungs-Skalare (`scale`, `min`) des Codebooks werden am Start der Query einmalig per Read-Lock kopiert und in der Engine lokal gekapselt, statt pro Kandidat gesperrt zu werden.

**Lock-/Nebenläufigkeitsmodell:** Einmaliger RWLock-Acquire pro Query.

**Invarianten-Nachweis:** §4(7) Zero-Copy (Übergabe eines Slices statt eines iterativ allozierenden `Vec`).

### B.5.5.4 — NaN-Sicherheit bei SIMD-Distanzberechnung

**Ist-Zustand im Repo:** Skalarer NaN-Check läuft bei jedem Vektorvergleich vor der SIMD-Schleife, was Performance massiv degradiert.

**Mathematische/algorithmische Spezifikation:** NaN-Werte kontaminieren L2-Normen. Um den Check im O(N) Hot-Path zu umgehen, wird die Validierung erzwungen an:

1. Den Query-Vektor $q$ am Start der Funktion ($O(D)$ Skalar).
    
2. Beim Insert jedes Vektors. Dies wird durch ein Flag `VALIDATED_NO_NAN` im Dateikopf manifestiert. Innerhalb von `dot4_avx2` wird nicht mehr geprüft.
    

**Rust-Schnittstelle (normativ):**

Rust

```
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx2", enable = "fma")]
#[allow(unsafe_code)]
pub unsafe fn dot4_avx2(q: &[f32], arena: &[f32], bases: [usize; 4], dim: usize, out: &mut [f32; 4]) {
    // Implementierung via _mm256_fmadd_ps ohne NaN Check
    unimplemented!()
}
```

**Lock-/Nebenläufigkeitsmodell:** Thread-lokale Register-Operationen.

**Invarianten-Nachweis:** §4(2) `unsafe_code` Begründung: Hardware-Acceleration für die zentrale mathematische Operation; Isolierung durch Garantien aus der Validierungs-Phase.

**Migrationspfad:** Altdaten (Header ohne Flag) triggern den langsamen Pfad, bis der Index kompaktiert wird.

**Restrisiken/offene Fragen:** Keine, da CPU FMA mathematisch deterministisch ist.

### B.5.5.5 — Top-k-Selektion

**Ist-Zustand im Repo:** `sort_unstable_by` sortiert volle Arrays in $O(M \log M)$.

**Mathematische/algorithmische Spezifikation:** Für materialisierte Listen wird Introselect (`select_nth_unstable_by`) genutzt: $O(M)$ im Average/Worst-Case. Für das Streaming im Traversierungspfad wird ein Bounded Min-Heap der Größe $k$ eingesetzt: $O(M \log k)$.

**Rust-Schnittstelle (normativ):**

Rust

```
pub fn select_top_k_materialized(scores: &[f32], id_table: &[u64], k: usize) -> Vec<u32> {
    let mut idx: Vec<u32> = (0..scores.len() as u32).collect();
    let cmp = |&a: &u32, &b: &u32| {
        scores[b as usize].total_cmp(&scores[a as usize])
            .then_with(|| id_table[a as usize].cmp(&id_table[b as usize]))
    };
    if k < idx.len() {
        idx.select_nth_unstable_by(k, cmp);
        idx.truncate(k);
    }
    idx.sort_unstable_by(cmp);
    idx
}
```

**Lock-/Nebenläufigkeitsmodell:** Keine Synchronisation notwendig.

**Invarianten-Nachweis:** §4(3) Determinismus durch total order via `total_cmp` und Tie-Breaker.

**Migrationspfad:** Austausch im `hybrid_search` Code.

**Restrisiken/offene Fragen:** Partielle Sortierung verliert die absolute Ranking-Ordnung über den Index $k$ hinaus.

### B.5.5.6 — CSR-Sentinel statt Option

**Ist-Zustand im Repo:** `Vec<Option<u32>>` in Graph- und Index-Strukturen verbraucht unnötigen Speicher für Diskriminanten.

**Mathematische/algorithmische Spezifikation:** Ersatz von `Option<u32>` durch `u32` mit dem Sentinel `u32::MAX`. Dies halbiert den RAM-Footprint für spärliche Vektoren von 8 auf 4 Byte pro Eintrag.

**Rust-Schnittstelle (normativ):**

Rust

```
pub const SENTINEL_NULL_ID: u32 = u32::MAX;
// Arrays nutzen direkt u32
```

**Lock-/Nebenläufigkeitsmodell:** N/A.

**Invarianten-Nachweis:** §4(5) Reduzierter Speicherverbrauch erhöht Budget-Transparenz.

**Migrationspfad:** Binär inkompatibles Format; bedarf Adapter.

**Restrisiken/offene Fragen:** Keine, solange Systemlimit bei $< 4.2 \times 10^9$ Knoten bleibt.

### B.5.5.7 — Partielle Rebuilds mit Recall-Erhaltungsgarantie

**Ist-Zustand im Repo:** Codebooks in der Skalar-Quantisierung driften, was zu Recall-Verlust bei Updates führt.

**Mathematische/algorithmische Spezifikation:** Das Codebook $C$ wird periodisch rekalibriert, wenn der Kullback-Leibler-Divergenzschätzer (oder die Min/Max Verschiebung) eine Schwelle überschreitet. Partielle Rebuilds der Layer erfolgen im Hintergrund.

**Lock-/Nebenläufigkeitsmodell:** RCU-Mechanik für das Codebook.

**Invarianten-Nachweis:** §4(3) Determinismus der Suchergebnisse bezogen auf die Epoche des Snapshots.

## B.6. Weitere Systembereiche

### B.6.1 LSM-Storage-Engine (`crates/memfuse-store`)

#### B.6.1.1 — SSTable-Lock-Handoff

**Spezifikation:** Zur Vermeidung von Stalls beim Flush der MemTable auf Disk wird das Mutex nicht über die I/O-Operation gehalten. Eine `AtomicU64`-Sequenznummer regelt den Handoff der Zuständigkeit für den Gruppen-Commit.

#### B.6.1.2 — Lock-freie WAL-Pipeline

**Spezifikation:** [v2.1] Der WAL nutzt einen MPSC-Actor mit begrenzter Queue und `oneshot`-Ack nach `fsync` (Group Commit), normativ in §5.3. SPSC war falsch (mehrere Produzenten); der Commit kehrt nie vor dem `fsync` zurück (P2).

#### B.6.1.3 — Inkomplette Tombstone-Propagierung

**Spezifikation:** Tombstones dürfen erst verworfen werden, wenn kein aktiver Snapshot (Lese-Transaktion) mehr existiert, der eine Sequenznummer kleiner der des Tombstones referenziert. Beweis: Behalte Versionen mit `seq > min_snapshot_seq` PLUS die neueste Version $\leq \text{min\_snapshot\_seq}$.

#### B.6.1.4 — Manifest-Fehlerbehandlung bei Crash-Recovery

**Spezifikation:** Der Zustand der Kompaktierung (lösche Inputs, füge Output hinzu) muss zwingend ein atomarer `ManifestEntry::Replace` Record sein. Ein teilgeschriebener Add/Remove hinterlässt bei einem Crash doppelte oder verwaiste Daten (Resurrection von gelöschten Schlüsseln).

#### B.6.1.5 — Zero-Copy-Slice

**Spezifikation:** Das Lesen von SSTable Blöcken nach dem Entschlüsseln und der CRC-Prüfung nutzt `bytes::Bytes::slice(4..)`, anstatt den Nutzlastpuffer neu zu kopieren. Erfüllt §4(7).

#### B.6.1.6 — MemTable Range-Sharding

**Spezifikation:** Hash-Sharding der In-Memory-Daten zerstört Präfix-Scans (erfordert 16 B-Tree Traversals). Range-Sharding teilt die Schlüssel alphanumerisch auf Shards auf, sodass ein Präfix-Scan zumeist in einem Lock-Fenster eines Shards bedient werden kann.

#### B.6.1.7 — Checkpoint-Index-Merge

**Spezifikation:** Statt separater Locks für `name_index` und `seq_index` fasst ein `RwLock` einen Struct zusammen, der beide Maps enthält. Atomarität ist gegeben.

### B.6.2 Inference, KV-Bridge & Routing (`crates/memfuse-candle`)

#### B.6.2.1 — Asynchrone Zero-Copy-KV-Cache-Eviction-Bridge

**Spezifikation:** Wenn Token-Mengen RAM übersteigen, werden paged KV-Blöcke (verschlüsselt via AES-GCM-SIV) per `Bytes`-Slice direkt in die LSM-Engine gespült. Async-I/O verhindert, dass die GPU-/CPU-Inferenz ins Stocken gerät.

#### B.6.2.2 — Instabilität des PID-Controllers (Lyapunov)

**Spezifikation:** Zur Latenzbegrenzung des Routings misst ein PID-Regler Abweichungen. Formel für Anti-Windup unter Berücksichtigung von $\Delta t$: $I_{new} = \text{clamp}(I_{old} + e \cdot \Delta t, -I_{max}, I_{max})$. Ohne Sättigungsgrenze läuft der Integrator ins Unendliche. Erfüllt deterministische Stabilität.

#### B.6.2.3 — Zero-Copy-Deserialisierung im IPC-Generator

**Spezifikation:** FlatBuffers wird genutzt, um IPC-Nachrichten vom Memfuse-Prozess zum MCP-Client (Python/Node) als direkte Referenz in Memory-Mapped Slices bereitzustellen, ohne Deserialisierungs-Kopien (zero-copy).

### B.6.3 Agenten-State, Crypto & MCP

#### B.6.3.1 — Atomare DLQ-Replay-Logik

**Spezifikation:** Ein Event, das fehlschlägt, wird als `(Session, Node, Step)`-Schlüssel persistiert. Idempotenz: Bei Replay prüft die Engine die WAL-Transaktions-ID, um Doppelbuchungen zu verhindern.

#### B.6.3.2 — Zeroize-on-Panic im Egress-Vault

**Spezifikation:** Um PII-Daten nach einem Panic (z.B. Timeout beim Regex-Matching) zu vernichten, werden sensible Strings in `zeroize::Zeroizing<Vec<u8>>` gewrappt. Der Drop-Guard sorgt deterministisch für die Überschreibung im RAM. Verteidigung in der Tiefe (§4(1)). [v2.1] Wirksam nur unter `panic = "unwind"` (Root-Profil, §0.2); im `release-abort`-Profil laufen keine Destruktoren, dort schützt nur das Prozessende, und Core-Dumps sind betrieblich zu deaktivieren.

#### B.6.3.3 — Race Conditions bei Budget-Berechnungen

**Spezifikation:** Das Agent-Budget wird über eine RAII-Struktur verwaltet: `budget.reserve(n) -> Reservation`. Bei Erfolg `reservation.settle()`, bei Drop erfolgt eine garantierte Rückerstattung. Double-Spend ist ausgeschlossen.

#### B.6.3.4 — Lückenhafte WASM-Sandbox-Egress-Isolierung

**Spezifikation:** Wasmtime erfordert strikte Speicherbegrenzungen (`Store::set_fuel`) und Memory-Limits. Cloud-Aufrufe innerhalb des WASM müssen von Datei-Reads logisch getrennt als `CloudEgress` in den Capability-Flags geführt werden.

#### B.6.3.5 — Kryptographisch verifizierbare Deletion Proofs

**Spezifikation:** Wenn ein Record aus dem LSM entfernt wird, erzeugt die HMAC-Kette der WAL einen Nachweis. Quittung: $H(\text{hmac}_{\text{prev}} \parallel \text{delete\_event})$. Verifikation in $O(1)$ Zeit ohne Klartext-Zugang (DSGVO Art. 17 konform).

#### B.6.3.6 — AES-Schlüsselplan-Wiederverwendung

**Spezifikation:** Die Expansionsrunde `new_from_slice` für AES-256-GCM-SIV kostet massive CPU-Zyklen. Die Struktur wird im `KeyManager` pro Schlüssel gecacht und thread-safe (`OnceCell`) wiederverwendet. [v2.1] Nonce-Strategie ⚖️ offen (§9.3, §A2.4 Nr. 5): Ein reiner In-Memory-Zähler beginnt nach Neustart bei 0 und ist ohne persistierten Hochwasserstand unzulässig; das Repo nutzt bewusst `OsRng`-Nonces.

## B.7. Priorisierung und Abhängigkeitsanalyse

Nach dem Schema: Aufwand (Trivial/Gering/Mittel/Hoch) × Nutzen (Latenz-/Speicher-/Korrektheitsgewinn).

### Stufe 0 — Unmittelbar (Korrektheit/Sicherheit)

|**Problem-ID**|**Maßnahme**|**Aufwand**|**Nutzen**|**Abhängigkeit**|
|---|---|---|---|---|
|**6.1.4**|LSM Manifest-Batch-Fsync (`Replace` Record)|Trivial|Verhindert Auferstehung gelöschter Daten|Keine|
|**5.2.1**|Bandit-Dimensionsprüfung / Ridge Math|Gering|Verhindert NaN/Dimensions-Crash|Keine|
|**6.3.4**|Egress-Klassifizierung & Wasmtime Limits|Gering|Sicherheitsisolation|Keine|
|**6.1.3**|Intent-Recovery & Tombstone-Propagierung|Mittel|Deterministisches Recovery|6.1.4|

### Stufe 1 — Hot-Path-Performance

|**Problem-ID**|**Maßnahme**|**Aufwand**|**Nutzen**|**Abhängigkeit**|
|---|---|---|---|---|
|**5.5.1**|HNSW v2 Arena Allocation|Hoch|Beseitigt 90% der Allokationen (3-5x Speedup)|Storage `Bytes` API|
|**6.1.5**|SSTable Zero-Copy-Slice & `StorageEngine::get`|Mittel|Verhindert Vollkopie auf Ebene 0|Keine|
|**6.3.6**|AES-Schlüsselplan Wiederverwendung|Gering|Reduziert Crypto-Overhead bei KV-Cache|Keine|
|**5.3.2**|Block-Cache Byte-Cap & SIEVE Lock-free|Mittel|Beseitigt LRU Kontention|Keine|
|**5.5.5**|Top-k Selektion (Introselect/Heap)|Trivial|$O(M \log M) \to O(M)$|Keine|

### Stufe 2 — Speicher und Struktur

|**Problem-ID**|**Maßnahme**|**Aufwand**|**Nutzen**|**Abhängigkeit**|
|---|---|---|---|---|
|**5.1.1**|N-äre Hyperkanten (RCU Integration)|Hoch|Modell-Exaktheit für LLM Inferenz|H2 (Locks)|
|**5.1.7**|Inkrementelle Graph-Kompaktierung (GC)|Mittel|$O(E)$ Lese-Spikes verhindern|H1|
|**5.5.6**|CSR Sentinel statt Option|Gering|Halbiert RAM-Footprint für CSR|HNSW v2|
|**6.1.7**|Checkpoint-Index-Merge|Gering|Beseitigt Race-Condition|Keine|

### Stufe 3 — Governance/Prozess

|**Problem-ID**|**Maßnahme**|**Aufwand**|**Nutzen**|**Abhängigkeit**|
|---|---|---|---|---|
|**CI**|Feature-Powerset-CI in GitHub Actions|Gering|Sichert Kompilierbarkeit aller Feature-Pfade|Keine|
|**CI**|Kontinuierliches Panic-Inventar-Gate|Mittel|Sichert §4(1) Zero-Panic Doctrine|Keine|

_(Harte Abhängigkeit verzeichnet: H3 `relate_n_ary` setzt das in Stufe 2 implementierte H2 Multi-Key-Locking voraus; der Bandit-Default-Wechsel in 5.2.2 setzt den Erfolg im Sherman-Morrison-Latenzgate voraus)._


---

---

<a id="anhang-c"></a>
# Anhang C — Änderungsprotokoll Fassung 2.1 und Prüfnachweise

## C.1 Geändert oder ergänzt

| Bereich | Änderung | Grund |
|---|---|---|
| Kopf, ToC | Teil A ergänzt (Rekonstruktion, §A.1–§A.5); Verweis auf die nicht vorhandene Tabelle A2.9 → `§A2.1`; Verweis `§4.5` → `§4.2`; Leitentscheidung (4) | Teil A fehlte, A2.9 und §4.5 existierten nicht |
| §4.i | Systeminvarianten §4(1)–§4(7) definiert, mit Lokalität P24 als (6) | „§4(n)"/„Invariante n" waren nirgends definiert |
| §5.2a | `key_hash` in `KvKeyLocks` (feste Seeds), `acquire` mit `Result` | `RandomState::new()` pro Aufruf ⇒ instabile Shards |
| §5.3 | SPSC-`MaybeUninit`-Ring → begrenzter MPSC-Group-Commit-Actor | mehrere Produzenten, `unsafe` im Safe-Crate, `hmac_prev` vom Produzenten |
| §5.4 | `crossbeam-epoch`-SIEVE → sicherer Entwurf; LRU poison-tolerant | `unsafe`, `!Send`/`!Sync`, `.unwrap()` |
| §6.3, §6.4 | `Arc`-Payload-Sharing, capacity-basierte Schätzung, Spitzenschätzung, Regel S1 | tiefer Klon je Schreibzugriff, fehlender Rehash-Peak |
| §6.6 H2 | Hash über `kv_locks.key_hash`; `Arc::from(participants)` | s. o. |
| §6.6 H5, §6.7 | persistente idempotente Cascade-Queue, `Ok(CascadeReport)`, `PartialCascadeQueued` entfällt | In-Memory-Queue geht bei Absturz verloren |
| §6.6 H6, §7.5 | Konvention K, korrekter Beweis, Iterator über Snapshot | Beweis falsch, Iterator klont alle Hyperkanten |
| §7.4 | SQ8-Bias-Kalibrierung (gemessen) | absolute Schwellen verschieben sich |
| §8.1, §8.2, §8.5 | `Result`-Trait, Laufzeit-Dimension, Discount `γ⁻¹`, IPS-Randomisierung | Compile-Fehler, Richtungsfehler, Propensity 1 |
| §9.3, §A2.4 | Nonce-Zähler als offene Entscheidung | Zähler beginnt nach Neustart bei 0 |
| §15, §16.2 | AK-9 bis AK-15, AK-2 präzisiert, `check-unwrap-baseline` → `clippy-panic-lints` | Ratchet entfällt laut §0.4 |
| Anhang B | Block ans Ende verschoben, `B.`-Nummerierung, Skizzen korrigiert | Nummernkollision, widersprüchliche normative Skizzen |

## C.2 Bewusst nicht aus dem neueren Dokument übernommen

SIEVE-Skizze (`unsafe`, `!Send`), Bandit-Code (`[f32; D*D]`, `unsafe` im Router), WAL als SPSC-Ring, AVX-512-SQ8-Kern
(Repo hat `dot_product_u8_avx512vnni`), `scc::HashMap` im Snapshot, deterministischer Nonce-Zähler, Zeroize-on-Panic
als Panic-Schutz im `release-abort`-Profil, Ring-Puffer für die WAL-Queue.

## C.3 Prüfnachweise (Sandbox, rustc 1.75, Repo-Stand `729b19b1`)

- `ahash::RandomState::new().hash_one(42)` liefert je Aufruf und je Prozesslauf verschiedene Werte; `with_seeds` ist stabil.
- `AlignedVector<{ D * D }>` scheitert mit „generic parameters may not be used in const operations"; `Shared<'static, T>` ist weder `Send` noch `Sync`.
- Kompiliert und ausgeführt: `KvKeyLocks`, `ShermanMorrisonBandit` (inkl. `discount_once`, Dimensionsfehler), `SieveCacheBackend` (`Send + Sync`), `WalHandle` (Backpressure, geschlossener Flusher), `GraphInner`-Schätzung (`size_of::<RoleBinding>() = 16`), `star_weight`.
- Star-Expansion: Schur-Komplement gleich Clique-Laplace-Matrix für N ∈ {2,3,5,8,20} (Abweichung ≈ 1e-16); alte Formel bei N=3: Gradsumme 1,5w gegen 6w.
- SQ8 (synthetisch, D=384): Rundungsenergie 4,88e-5 gegen Σ Δ²/12 = 4,89e-5; Netto-Offset bei Gauß-Daten mit 99,9-%-Clipping ≈ −75·c_q, bei Gleichverteilung ≈ 2·c_q — daher gemessener Bias statt analytischer Konstante.
- Repo-Befunde: tiefer Klon in `InnerWriteGuard::drop`; `len()·size_of`-Schätzung; `StarExpansionIterator::new` mit `.cloned().collect()` und Gewicht `w`; Cascade-Queue nur als In-Memory-`VecDeque`, `enqueue` nur in Tests; WAL-Flusher mit `unbounded_channel` und `oneshot`-Ack nach `sync_all`; Bandit mit `γ⁻¹` und Laufzeit-Dimension; keine Propensity-Protokollierung; `OsRng`-Nonces.

## C.4 Offen und nicht geprüft

- Teil A ist eine Rekonstruktion (Kennzeichnung **[F2.1]**); die ursprüngliche Fassung ersetzt sie.
- Die Konvention K (Faktor 2, Gesamtmasse w) ist ein Arbeitsstand (§A2.4 Nr. 6).
- SQ8-Zahlen beruhen auf synthetischen Daten; die Nachmessung auf echten Embeddings steht aus.
- Der O(H)-Klon je Veröffentlichung bleibt (§6.3, Restkosten).
- Nicht geändert, aber aufgefallen: `quick_cache = "0.5"` in §0.2 gegen `0.6` im Repo; `unreachable!()` in `quantize.rs` des Repos widerspricht §4(1).
