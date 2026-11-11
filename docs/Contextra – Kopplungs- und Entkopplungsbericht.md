# Contextra – Kopplungs- und Entkopplungsbericht

*Repository `tfufuz1/contextra` · HEAD `5408826f8` (10.11.2026, Merge PR #5959) · Bezugsdokument `docs/SPEC.md` v2 (normativ) · Erstellt am 10.11.2026*

---

## 1. Zusammenfassung

**Gesamtbild.** Das Repository besteht aus 39 Crates in `crates/` (rund 200 kLOC laut Inspector, 185 kLOC laut `tools/loc.py`). Die Entkopplung ist zweigeteilt:

- **19 Crates sind bereits Bausteine** mit Score 100 und null ungegateten Brückenreferenzen (u. a. `ids`, `sys`, `wire`, `simd`, `rank`, `adapt`, `privacy`, `sandbox`, `prompt-guard`, `vault`, `ingest`, `erasure`, `durable-fs`). Dazu kommt `env` mit 95.
- **Das Kernprodukt (`store`, `mvcc`, `crypto`, `vector`, `graph`, `text`, `engine`) ist noch fest an `contextra-types` und `contextra-ports` verdrahtet.** Diese beiden E2-Crates haben 26 bzw. 21 Verwender (Fan-in).

**Die fünf wichtigsten Befunde**

1. **Vier E1-Crates hängen im Kern an E2:** `mvcc`, `crypto`, `store` und `infer-candle` führen `types` und `ports` als nicht-optionale Abhängigkeit. Das ist ein Rückwärtspfeil gegen INV-8. Er blockiert die Entnahme von `store` und, über `mvcc`, auch von `vector`.
2. **Manifest und Quelltext widersprechen sich** bei `vector`, `graph`, `text`, `kvcache`, `checkpoint`: Die Abhängigkeiten sind `optional`, der Quelltext referenziert `contextra_types`/`contextra_ports` aber in vielen Dateien ohne `cfg`-Gate (z. B. `vector` 150 Referenzen in 35 Dateien, `graph` 109 in 52). Hypothese \[A\]: Ein Build mit `--no-default-features` scheitert dort. Nachprüfen konnte ich das nicht, weil `cargo` fehlt.
3. **Seam-Duplikate:** `trait Clock` existiert in 10 Crates, `trait Rng` in 5, `trait MetricsSink` in 2, `DocId` als Struktur in 7 Crates, `DistanceMetric` in 3. `env` und `ids` sind als Besitzer bereits angelegt, die Kopien sind aber nicht abgelöst.
4. **Löschrelikte außerhalb von `erasure`:** `store` 49, `audit-export` 33, `text` 25, `graph` 21, `ports` 21, `vector` 11, `engine` 8 Fundstellen (Inspector-Heuristik, nicht alle sind echte Verstöße).
5. **`engine` ist der Integrationsklumpen:** 12 interne Abhängigkeiten, 496 Brückenreferenzen, 194 Stellen mit direktem Zugriff auf konkrete Kerne statt auf Ports.

**Was ich nicht belegen kann.** `cargo` ist in der Analyseumgebung nicht vorhanden. Die SPEC-Gates G-L1 (`gate_layering.py` bricht mit „No such file: cargo“ ab), G-L3 und G-L4 (`isolation_proof.py`, `extract_probe.py`) wurden **nicht ausgeführt**. Kein Crate gilt daher nach SPEC R6 als „entnehmbar bestätigt“. Alle Zahlen stammen aus statischer Analyse (Manifeste, Textsuche, Inspector-Skripte).

---

## 2. Methodik und Grenzen

| Quelle | Verwendung |
| --- | --- |
| `tools/decouple_inspector.py --all --json` | Repo-eigener Score (0–100), Abhängigkeiten, Löschrelikte, Dead-Code-Kandidaten, fehlende Ports |
| `tools/count_bridge_refs.py` | Zählung `contextra_(types\|ports\|testkit)` je Crate |
| `tools/feature_audit.py --fail-on-unused` | Features ohne `cfg(feature=…)` im Quelltext |
| `tools/loc.py`, `docs/crate-coupling-report.md` | Größenvergleich, älterer Bericht (Stand 00:41, noch mit „Ring“-Begriffen) |
| Eigenes Skript (`tomllib` + Regex) | SPEC-Score, Kantenmatrix, Layering, Seam-Duplikate, Modul-Leaf-Analyse |
| `docs/SPEC.md` §3.1 | Zielzustand je Crate (Ebene, Kern-Closure) |

**Grenzen**

- Der Inspector bestraft jede interne Abhängigkeit gleich, auch solche hinter `workspace`/`*-compat`, und nutzt veraltete Begriffe (Klasse A/B/C, SPEC v4.0). Er unterschätzt bereits entkoppelte Crates nicht, überschätzt aber die Kopplung bei E2-Crates (`engine`: Inspector 10, SPEC-Score 60).
- Die Gating-Erkennung ist textuell: Eine Datei gilt als gegatet, wenn sie irgendeine `cfg(…compat|workspace…)`-Zeile enthält (gleiche Heuristik wie SPEC B04). Das überschätzt die Gatung eher, die Zahlen sind also Untergrenzen.
- Beleg-Marken wie in der SPEC: **\[V\]** am Manifest oder Quelltext gemessen, **\[A\]** daraus abgeleitet, nicht gebaut.
- Der HEAD weicht vom SPEC-Stand (`ef3364f`) ab. Beispiele: `ids`, `env`, `vault`, `prompt-guard` existieren bereits, `ports` hängt schon von `env` ab, der tote `xtask`-Alias ist entfernt.

---

## 3. Architektur-Ist

### 3.1 Ebenenmodell (SPEC §1.1)

E0 = Kern-Closure leer · E1 = höchstens 4 E0-Mitglieder · E2 = Integration · E3 = Anwendungen/Testwerkzeuge.

| Ebene | Crates |
| --- | --- |
| E0 | `ids`, `env`, `sys`, `durable-fs`, `simd`, `wire`, `adapt`, `infer-onnx`, `ingest`, `privacy`, `sandbox`, `prompt-guard`, `erasure`, `license`, `audit-export`, `avv-generator` |
| E1 | `mvcc`, `crypto`, `store`, `vector`, `text`, `graph`, `rank`, `checkpoint`, `kvcache`, `infer-ollama`, `infer-candle`, `vault` |
| E2 | `types`, `ports`, `router`, `agent`, `cognition`, `engine`, `db`, `contextra` |
| E3 | `mcp`, `py`, `testkit` (+ `benchmarks/contextra-bench`) |

### 3.2 Kantenstatistik \[V\]

| Größe | Wert |
| --- | --- |
| Interne Kanten gesamt | 117 |
| davon normal / dev / build | 96 / 21 / 0 |
| davon optional (normal) | 35 |
| Fan-in `types` / `ports` | 26 / 21 |
| Fan-in `crypto` | 4 (`infer-candle`, `kvcache`, `mcp`, `store`) |
| Fan-in `graph`, `mvcc`, `rank`, `wire`, `sys` | je 3 |
| Dev-Kanten auf `testkit` | 8 (`agent`, `checkpoint`, `cognition`, `crypto`, `db`, `engine`, `mvcc`, `store`) |

`types` und `ports` sind damit die zentralen Kopplungsknoten. Jede Änderung dort wirkt auf bis zu 26 Crates.

---

## 4. Kopplungsbefunde (repoweit)

### 4.1 Aufwärtskanten E1→E2 (Verstoß gegen INV-8) \[V\]

| Crate | Kante | Status |
| --- | --- | --- |
| `mvcc` | → `types`, `ports` | **nicht optional**, 12 Brückenreferenzen |
| `crypto` | → `types`, `ports` | **nicht optional**, 34 Referenzen |
| `store` | → `types`, `ports` | **nicht optional**, 237 Referenzen |
| `infer-candle` | → `types`, `ports`, `rank` | **nicht optional**, 46 Referenzen |
| `vector`, `graph`, `text`, `rank`, `checkpoint`, `kvcache`, `infer-ollama`, `vault` | → `types`/`ports` | optional (hinter Feature) |
| `simd`, `audit-export`, `license`, `infer-onnx` (E0) | → `types`/`ports` | optional |

Die vier nicht-optionalen Kanten liegen ausgerechnet im Fundament der Speicherschicht: `store` → `mvcc`/`crypto` → `types`/`ports`. Solange `mvcc` und `crypto` an E2 hängen, kann kein darüberliegender Baustein E1 sein.

### 4.2 Optional im Manifest, ungegatet im Quelltext \[V\]/\[A\]

Die Brücke (`compat/`) und das Feature existieren bei `vector`, `graph`, `text`, `kvcache`, `checkpoint`, `simd`, `rank`. Der Kern greift trotzdem in vielen Dateien direkt auf `contextra_types`/`contextra_ports` zu.

| Crate | Ungegatete Referenzen | Dateien |
| --- | --- | --- |
| `store` | 233 | 65 |
| `vector` | 150 | 35 |
| `graph` | 109 | 52 |
| `text` | 39 | 6 |
| `crypto` | 34 | 7 |
| `infer-candle` | 46 | 14 |
| `kvcache` | 11 | 5 |
| `mvcc` | 12 | 6 |
| `checkpoint` | 4 | 2 |

Folge \[A\]: `cargo check -p <crate> --no-default-features` ist für diese Crates sehr wahrscheinlich rot. Bei `graph`, `kvcache`, `rank`, `text` und `vector` sind die Kompatibilitäts-Features sogar **Default** (`types-compat`, `ports-compat`), die Brücke ist also im Standardbuild immer aktiv und der Kern nie isoliert getestet. SPEC §3.1 fordert für `graph` und `crypto` leere Defaults.

### 4.3 Doppelte Seams und Typen \[V\]

| Konzept | Vorkommen (Anzahl Definitionen) | Soll-Besitzer (SPEC R4) |
| --- | --- | --- |
| `trait Clock` | `adapt`, `env`, `erasure`, `graph`, `license`, `privacy`, `prompt-guard`, `rank`, `text`, `vector` (2) | `env` |
| `trait Rng` | `adapt`, `env`, `graph`, `text`, `vector` (2) | `env` |
| `trait IdGen` | `env`, `erasure` | `env` |
| `trait MetricsSink` | `env`, `vector` | `env` |
| `DocId` (Struktur) | `ids`, `erasure`, `graph`, `ingest`, `rank`, `text`, `vector` | `ids` (`docid-128` nur dort, SPEC B03) |
| `DistanceMetric` | `types`, `vector`, `simd` | je Crate, gespiegelt (SPEC B09, R4-Ausnahme) |

Wichtig: Die Kopien sind **nicht kompatibel** (SPEC B02): Einheit ms gegenüber ns, `&mut self` gegenüber `&self`, `'static` ja/nein. Eine Konsolidierung braucht die Brückenmuster aus SPEC §4.3 und den Differenztest, kein bloßes Löschen.

### 4.4 Löschrelikte außerhalb der Compliance-Crates \[V, Heuristik\]

`store` 49, `audit-export` 33 (legitim, Compliance-Crate), `text` 25, `graph` 21, `ports` 21, `vector` 11, `engine` 8, `kvcache` 3, `contextra` 2. Die Inspector-Begriffe (`purger`, `deletion_proof`, `avv` …) treffen auch legitime mechanische Löschpfade (`Purger`, `LayerScanner`), die AGENTS.md §1.2 ausdrücklich erhält. Je Fundstelle ist manuell zu prüfen, ob sie Mechanik oder eine Rechts-/Compliance-Aussage (INV-7) ist.

### 4.5 Fehlende Port-Abstraktionen

`engine` 194 Stellen, `store` 26, `kvcache` 8, `infer-candle` 6 mit direktem Import konkreter Speicherkerne (Inspector). Bei `engine` ist das teils gewollt (Orchestrierung), erschwert aber den Austausch einzelner Signale.

### 4.6 Features \[V\]

- `feature_audit.py` meldet 107 „ungenutzte“ Features in 37 Crates. Die Liste enthält jedoch `default` selbst und reine Weiterleitungs-Features, sie ist daher **laut, nicht abnahmefähig**. Belastbare Kandidaten sind die Kompatibilitäts-Features, die nirgends per `cfg` auftauchen (z. B. bei `checkpoint`: `ports-compat`, `types-compat`, `workspace`).
- Auffällige Defaults: `audit-export` (`workspace` und `test-utils` im Default), `graph` (inkl. `experimental-graph`), `rank` (inkl. `exp-dibud`, `exp-resonance`), `kvcache` (inkl. `kv-encryption`), `infer-candle` (`candle`). SPEC X2 verbietet Experimente im Default.
- Hohe Featurezahl: `vector` 18, `ingest` 14, `db` 13, `engine` 12, `text` 11, `graph` 11.

### 4.7 Unsafe-Inseln \[V\]

Nur `simd` (269 Stellen), `wire` (45) und `sys` (22) enthalten `unsafe`. Das deckt sich mit AGENTS.md („Unsafe-Insel“) und ist sauber isoliert.

### 4.8 Dead-Code-Kandidaten (Inspector)

`ingest` 29, `types` 20, `checkpoint` 10, je 5 bei `graph`, `infer-ollama`, `mcp`, `store`. Vor jeder Löschung gilt AGENTS-Regel „kein Algorithmus wird gelöscht; hinter Feature sichern oder verdrahten“.

---

## 5. Modularitäts-Score je Crate

**SPEC-Score (0–100).** Start bei 100, Abzüge für:

- Kern-Abhängigkeiten über dem Zielzustand aus SPEC §3.1 (12 je Abweichung, max. 35),
- ungegatete Brückenreferenzen (15·log10(1+n), max. 40),
- verfehlte Null-Prüfung G-L2 bei E0/E1 (−15),
- fehlende `compat/`-Brücke oder fehlendes Workspace-Feature bei E0/E1 mit `types`/`ports` (je −10),
- fehlender eigener Fehlertyp bei E0/E1 (−5).

Bei E2/E3 entfällt die G-L2-Strafe (dort sind `types`/`ports` erlaubt). Der Inspector-Score ist zum Vergleich angegeben.

| Crate | Ebene | SPEC-Score | Inspector | Src-Zeilen | Ungegatete Refs (Dateien) |
| --- | --- | --- | --- | --- | --- |
| `infer-candle` | E1 | **15** | 71 | 6.549 | 46 (14) |
| `store` | E1 | **15** | 33 | 30.673 | 233 (65) |
| `crypto` | E1 | **28** | 88 | 6.011 | 34 (7) |
| `mvcc` | E1 | **34** | 83 | 4.683 | 12 (6) |
| `agent` | E2 | 36 | 54 | 5.672 | 86 (16) |
| `vector` | E1 | 52 | 29 | 16.931 | 150 (35) |
| `graph` | E1 | 54 | 46 | 22.526 | 109 (52) |
| `engine` | E2 | 60 | 10 | 21.902 | 496 (70) |
| `text` | E1 | 61 | 52 | 9.588 | 39 (6) |
| `cognition` | E2 | 63 | 70 | 7.430 | 42 (12) |
| `kvcache` | E1 | 69 | 56 | 7.167 | 11 (5) |
| `db` | E2 | 70 | 65 | 1.566 | 14 (4) |
| `router` | E2 | 74 | 61 | 7.875 | 51 (16) |
| `testkit` | E3 | 74 | 83 | 4.063 | 50 (16) |
| `checkpoint` | E1 | 75 | 71 | 5.398 | 4 (2) |
| `mcp` | E3 | 77 | 48 | 4.690 | 32 (8) |
| `contextra` | E2 | 79 | 55 | 2.023 | 26 (7) |
| `py` | E3 | 86 | 77 | 3.529 | 7 (5) |
| `ports` | E2 | 88 | 53 | 6.598 | 5 (5) |
| `env` | E0 | 95 | 100 | 855 | 0 |
| 19 Crates | E0/E1/E2 | **100** | 64–100 | – | 0 |

Die 19 Crates mit Score 100: `adapt`, `audit-export`, `avv-generator`, `durable-fs`, `erasure`, `ids`, `infer-ollama`, `infer-onnx`, `ingest`, `license`, `privacy`, `prompt-guard`, `rank`, `sandbox`, `simd`, `sys`, `types`, `vault`, `wire`.

**Lesehinweise**

- `store` und `engine` liegen im Inspector weit unter dem SPEC-Score, weil der Inspector alle fünf bzw. zwölf internen Abhängigkeiten gleich gewichtet. Für `store` sind jedoch nur `types` und `ports` über dem Ziel.
- `infer-candle` bekommt im SPEC-Score den schlechtesten Wert, weil dort `rank` zusätzlich zu `types`/`ports` im Kern steht (Ziel: nur `env`, `kvcache` hinter `kv-bridge`).
- Ein Score von 100 bedeutet „statisch unauffällig“, nicht „entnehmbar bestätigt“. Dafür fehlen G-L3 und G-L4.

---

## 6. Funktionsebene: Entnehmbarkeit einzelner Module und Funktionen

Als **Leaf-Modul** gilt eine Quelldatei ohne ungegatete Brückenreferenz und mit höchstens zwei `crate::`-Abhängigkeiten (ohne `compat/` und Tests).

| Crate | Dateien | G-L2-saubere Dateien | Leaf-Dateien | Leaf-Anteil LOC | Leaf-Anteil `pub fn` |
| --- | --- | --- | --- | --- | --- |
| `rank` | 26 | 26 | 24 | 87 % | 88 % |
| `kvcache` | 11 | 6 | 6 | 44 % | 27 % |
| `text` | 24 | 21 | 19 | 43 % | 45 % |
| `crypto` | 18 | 11 | 8 | 34 % | 36 % |
| `checkpoint` | 12 | 10 | 8 | 25 % | 16 % |
| `graph` | 62 | 19 | 18 | 19 % | 27 % |
| `vector` | 48 | 17 | 16 | 13 % | 12 % |
| `infer-candle` | 20 | 7 | 7 | 11 % | 5 % |
| `engine` | 72 | 15 | 15 | 8 % | 3 % |
| `store` | 64 | 13 | 13 | 5 % | 6 % |
| `mvcc` | 7 | 1 | 1 | 1 % | 0 % |

### 6.1 Hotspots (Datei · Brückenreferenzen · Fan-out)

| Crate | Datei | Refs | Fan-out | Anmerkung |
| --- | --- | --- | --- | --- |
| `store` | `lsm/engine.rs` | 17 | 8 | 46 öffentliche Funktionen, 435 Zeilen |
| `store` | `compaction/engine.rs` | 14 | 5 | 874 Zeilen |
| `store` | `scanner/mod.rs` | 13 | 1 | gehört nach SPEC in die Brücke (`scanner/`) |
| `store` | `lsm/recovery.rs` | – | 9 | höchster Fan-out, Durability-Pfad (INV-6) |
| `vector` | `quantize/mod.rs` | 28 | 1 |  |
| `vector` | `hnsw/types.rs` | 26 | 5 | 56 öffentliche Funktionen, 922 Zeilen |
| `vector` | `hnsw/core_rebuild.rs` | 12 | 6 | 1.168 Zeilen |
| `graph` | `cascade.rs` | 15 | 3 | 951 Zeilen |
| `graph` | `ppr/mod.rs` | – | 5 | Fan-out-Hub |
| `text` | `wand.rs` | 20 | 4 | 816 Zeilen; in der Entwurfs-Zählung übersehen (SPEC B04) |
| `text` | `inverted/index_struct.rs` | 2 | 4 | 23 öffentliche Funktionen |
| `engine` | `collection/crud/insert.rs` | 26 | 3 |  |
| `engine` | `collection/search/hybrid/mod.rs` | 26 | 1 |  |
| `engine` | `lib.rs` | 25 | 0 |  |

**Aussage:** `rank` ist praktisch komplett entnehmbar, `text`, `kvcache` und `crypto` zu rund einem Drittel bis zur Hälfte. Bei `store`, `mvcc`, `vector` und `engine` sind einzelne Funktionen heute nicht sauber herauszulösen, weil die Typen aus `types`/`ports` in der Signatur stehen.

---

## 7. Gate- und Werkzeugstatus

| Punkt (SPEC) | Stand \[V\] |
| --- | --- |
| B05 toter `xtask`-Alias | Entfernt (Kommentar in `.cargo/config.toml`) |
| B05 Layering-Gate | `tools/gate_layering.py` und negative Python-Tests vorhanden; Lauf braucht `cargo`, hier nicht ausführbar |
| B05 `tests/arch` | Im Verzeichnis `tests/` nicht vorhanden (Dateien: `layering.rs`, `unsafe_islands.rs`, Negativtests) |
| B11 CI-Abdeckung | Workflows `loom-gate`, `tsan`, `mutation-testing`, `unwrap-gate` vorhanden; ob `mvcc`, `kvcache`, `checkpoint`, `ids`, `env` in den Pfadfiltern stehen, habe ich nicht geprüft |
| B11 Loom-Modell `mvcc` | 6 Dateien in `crates/contextra-mvcc` enthalten „loom“ (u. a. `Cargo.toml`, `SPEC.md`); ein Modell für `SequenceLog` ist damit **nicht bestätigt** |
| `feature_audit.py` | Läuft, Ergebnis laut (siehe 4.6) |
| `isolation_proof.py`, `extract_probe.py` (G-L4) | Nicht ausgeführt (kein `cargo`) |

---

## 8. Entkopplungsplan

Jeder Schritt folgt INV-3 (Commit A verschiebt/benennt um, Commit B baut um) und bleibt unter den Invarianten INV-1 (Formatgleichheit) und INV-6 (Durability-Pfade nur, wenn genannt).

| Prio | Crate | Maßnahme | Nachweis |
| --- | --- | --- | --- |
| **P1** | `mvcc` | `types`/`ports` aus `[dependencies]` entfernen, hinter `workspace` legen; `MvccError` lokal; `ids`/`env` als einzige Kern-Abhängigkeiten; Port-Impls und `From<MvccError>` nach `compat/`. Zuerst Loom-Modell `SequenceLog` sicherstellen (B11) | G-L2, G-L3 (`--no-default-features`), Golden (INV-1), Loom |
| **P1** | `crypto` | wie `mvcc`; Default-Features leeren (SPEC §3.1); `legacy-erasure` aus Default (B10); KV-Teile erst mit `kvcache` als Paar verschieben (S-38) | G-L2, G-L3, Golden KDF/KV, Mutation-Score G2/G3 ≥ Baseline |
| **P2** | `text` | `wand.rs` (20 Refs) und `inverted/index_struct.rs` ins Gate ziehen, `TextError` lokal, Default `types-compat` durch `workspace` ersetzen. Ergebnis: erster Retrieval-Baustein mit grünem G-L2 | G-L2, G-L3, Differenztest ms-Zeitstempel |
| **P2** | `store` | `lsm/engine.rs`, `compaction/engine.rs`, `lsm/config.rs`, `scanner/mod.rs` auf `StoreError`/`env::Clock` umstellen; `scanner/` in `compat/`; danach `types`/`ports` aus dem Kern. `lsm/recovery.rs` nur anfassen, wenn der Schritt es ausdrücklich nennt (INV-6) | G-L2, Golden (Manifest, SSTable, WAL, Rollback-Intent), Loom/TSan |
| **P3** | `vector` | `quantize/mod.rs`, `hnsw/types.rs`, `hnsw/core_rebuild.rs`, `hnsw/core_search.rs`; lokale `Clock`/`Rng`-Kopien auf `env` (Brücke `&self→&mut self`); Feature `simd` mit Skalar-Orakel (B09) | G-L2, Golden (HNSW, DiskANN), Differenztest Distanz 1e-5 |
| **P3** | `graph` | `cascade.rs`, `ppr/mod.rs`, `edge_reinforcement_buffer.rs`, `percolation.rs`; Default leeren (`experimental-graph` entfernen, X2) | G-L2, G-L3, Golden Graph-Persistenz |
| **P4** | `kvcache`, `checkpoint` | Rest-Referenzen (11 bzw. 4) schließen; Default-Features bereinigen | G-L2, G-L3 |
| **P4** | `infer-candle` | `rank` aus dem Kern entfernen, `types`/`ports` hinter `workspace`, `kvcache` nur hinter `kv-bridge` | G-L2, G-L3 |
| **P5** | `engine`, `agent`, `cognition`, `db` | Ports statt konkreter Kerne (194 Stellen in `engine`); `agent` ohne direkte Kanten zu `db`/`graph`/`router`/`checkpoint` im Kern (SPEC: nur `types`, `ports`) | G-L1, Architekturtest |
| laufend | Seams | `Clock`/`Rng`/`MetricsSink` je Crate auf `env` umstellen, E0-Crates per Blanket-Impl in `compat/` (SPEC §4.3); `DocId`-Kopien auf `ids`; `docid-128` nur über `ids` weiterleiten (B03) | Differenztest, Matrix `--features docid-128` |
| laufend | Hygiene | Löschrelikte je Fundstelle prüfen (Mechanik vs. Compliance-Aussage); Default-Experimente streichen; `feature_audit` entrauschen (`default` und Weiterleitungen ausnehmen) | G-DOC1, `feature_audit` grün |

**Begründung der Reihenfolge.** `mvcc` und `crypto` sind die Basis von `store` und `vector`. Solange sie an `types`/`ports` hängen, bleibt jede höhere Entnahme blockiert. `text` ist der günstigste Erfolg und beweist das Muster, bevor die großen Crates folgen.

---

## 9. Entnahmereife

**Kandidaten, die statisch bereits entnehmbar wirken** (Score 100, keine interne Abhängigkeit oder nur gegatete): `ids`, `env`, `sys`, `wire`, `durable-fs`, `adapt`, `privacy`, `sandbox`, `prompt-guard`, `ingest`, `erasure`, `simd`, `rank`, `license`, `vault`, `infer-onnx`, `infer-ollama`.

**Nicht entnehmbar, bis die P1/P2-Schritte erledigt sind:** `store`, `mvcc`, `crypto`, `vector`, `graph`, `text` (E1-Kern) sowie alle E2/E3-Crates, die darauf aufbauen.

Bestätigung erst durch G-L2 → G-L1 → G-L3 → G-L4 in dieser Reihenfolge (SPEC §8). Hierfür ist eine Umgebung mit `cargo` nötig.

---

## 10. Risiken und offene Punkte

- **Statische Analyse:** Textsuche kann Makro-Expansionen, Glob-Importe und `cfg_attr` nicht auflösen. Die Zahlen sind Orientierung, Abnahme erfolgt nur über G-L2 bis G-L4.
- **Stand-Abweichung:** HEAD weicht vom SPEC-Stand ab. S-00 (Baseline neu erheben) ist vor dem ersten Umbau zu wiederholen.
- **Durability:** `store/lsm/recovery.rs` (Fan-out 9) und WAL/Manifest sind INV-6-geschützt. Änderungen dort brauchen Golden-Korpus und Crash-Tests.
- **Nebenläufigkeit:** `mvcc` (SSI, `SequenceLog`, `TxBuffer`) ist der Nebenläufigkeitskern. Ohne bestätigtes Loom-Modell ist P1 nicht abnehmbar.
- **Fehlerabbildung:** Lokale Fehlertypen ändern Rückgabetypen (SPEC B12). Display-Text und Variante müssen bitgleich bleiben.
- **Score-Aussagekraft:** Der SPEC-Score ist ein eigenes Heuristikmaß, kein Repo-Gate. Er eignet sich zum Priorisieren und Verfolgen des Fortschritts, ersetzt aber keine Gates.

---

## Anhang A – Reproduktion

```bash
git clone https://github.com/tfufuz1/contextra && cd contextra
python3 tools/decouple_inspector.py --all --json        # Inspector-Score
python3 tools/count_bridge_refs.py                      # Brückenreferenzen je Crate
python3 tools/feature_audit.py --fail-on-unused         # Feature-Audit (laut)
python3 tools/loc.py                                    # Größen
# benötigen cargo (hier nicht lauffähig):
python3 tools/gate_layering.py --all                    # G-L1
python3 tools/isolation_proof.py --crate contextra-text --modes no-default,default,each-feature   # G-L4
python3 tools/extract_probe.py contextra-text --test    # Entnahme-Probe
```

## Anhang B – Quellen im Repository

`docs/SPEC.md` (normativ) · `AGENTS.md` · `docs/crate-coupling-report.md` (älterer Stand, „Ring“-Begriffe) · `docs/decoupling/` (`REPORT.md`, `F0_REPORT.md`, `baseline/`) · `capabilities.toml` · `governance/` · `tools/*.py`
