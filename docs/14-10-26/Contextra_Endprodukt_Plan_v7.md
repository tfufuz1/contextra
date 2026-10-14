# Contextra — Endprodukt-Zerlegung, Verdrahtungsplan und Agenten-Betrieb (v7)

**Stand:** Dienstag, 13. Oktober 2026
**Repository:** `tfufuz1/contextra`, geprüfter HEAD `59f853cb`
**Bezug:** Umsetzungsplan v6 (Commit `e2b02b9`), Kernschnitt-Spezifikation (Commit `4b7709e6`), Spezifikation v4 (Zielbeschreibung)
**Status:** Entscheidungsvorlage. Ersetzt v6 für die Umsetzung. Entscheidungen mit „Annahme“ sind Vorschläge, die bestätigt werden müssen (Abschnitt 10).

**Kennzeichnung der Aussagen:**

- **[G]** gemessen: direkt aus Dateien oder Git gelesen.
- **[E]** Emulation: Logik des Gates in Python nachgebildet, weil kein `cargo` verfügbar ist. Nicht als Lauf des Gates zu verstehen.
- **[H]** Heuristik: Aufrufmuster per Textsuche. Kein Compilerbeweis.
- **[U]** unverifiziert: Behauptung aus einem der Vorgängerdokumente, am Code nicht geprüft.

---

## 0. Zusammenfassung

1. **Der Kern ist das Problem, nicht die Breite.** Der Plan v6 hat recht, dass die Arbeit nicht verloren ist. Er überschätzt aber, wie weit der Schnitt ist, und unterschätzt, wie viel im Kern noch nicht verdrahtet ist.
2. **Am HEAD sind 101 der 111 als „verdrahten“ markierten Kernfunktionen [H] ohne erkennbaren Produktionsaufrufer.** Das ist der eigentliche Umfang von EP1.
3. **Die Löschkette ist teilweise angeschlossen.** `Collection::delete` ruft jetzt `erase_document` auf, das Footprint, Ledger, Layer-Zeugen und KV-Entfernung ausführt [G]. Der MCP-Handler verwirft das Ergebnis und liefert `proof: null` [G]. Die Nachweis-Ausstellung (`DpiDeletionRuntime`) und der physische Purge haben keinen Produktionsaufrufer [G].
4. **Die Gates sind nicht so belastbar, wie die Vorgänger annehmen.** `check-dag` kann strukturell nicht fehlschlagen, auch nicht bei Zyklen [G]. Ein echter Ring-Gate (`check-ring-layering --strict`) existiert, und meine Emulation zeigt sechs nicht erlaubte Verletzungen [E]. Ob die CI tatsächlich rot ist, ist [U].
5. **Der Kern hat 10 reguläre und 16 optionale Kanten in den Parkplatz [G].** Die größte Einzelmaßnahme ist das Umschalten der Default-Features von `engine` und `contextra`, nicht das Schreiben neuer Ports.
6. **`contextra-core` ist eine reine Re-Export-Fassade** [G]. Sie zu entfernen ist mechanisch, aber betrifft 110 Dateien und rund 300 Zeilen.
7. **Die Hüllenzahl 32 bzw. 30 gilt nur mit allen Features aktiv** [G]. Der Default-Build ist deutlich kleiner. Jedes Gate muss die Feature-Konfiguration angeben.
8. **Das Ziel „Ökosystem aus abschaltbaren Modulen“ braucht einen Produkt-Manifest-Mechanismus.** Ein Produkt wird durch eine Liste von Crates und Features definiert, und ein Gate prüft seine Hülle (Abschnitt 4.3).

---

## 1. Ausgangslage

| Messgröße | Wert | Quelle |
|---|---|---|
| Crates unter `crates/` | 36 | [G] `ls crates/` |
| Workspace-Mitglieder inkl. `benchmarks/`, `xtask`, `xtask-heavy` | 37 | [G] `Cargo.toml` |
| In `capabilities.toml` geführt | 34 (ohne `contextra-bench`, `xtask`, `xtask-heavy`) | [G] |
| Ring-Angaben: `Cargo.toml` vs. `capabilities.toml` | 34 von 34 gleich | [G] |
| Commits seit Plan v6 (`e2b02b9`) | 11, davon 55 Dateien geändert | [G] `git log` |
| Öffentliche Funktionen aus Anhang A | 333, alle noch definiert | [G] |
| davon mit Produktionsaufruf-Treffer (Heuristik) | 24 | [H] |
| Kern-Crates / Parkplatz-Crates (Kernschnitt-Definition) | 18 / 15 | [G] |

---

## 2. Korrekturen an den Vorgängerdokumenten

| Aussage | Stand am HEAD | Bewertung |
|---|---|---|
| Plan B4: Löschkette nicht angeschlossen | `delete()` → `erase_document` ist verdrahtet; `proof` bleibt `null` | **teilweise überholt** [G] |
| Kernschnitt N1: `contextra` hängt regulär an `infer-*`, `router`, `audit-export`, `avv-generator` | Diese Kanten sind `optional`. Default-aktiv ist nur `infer-candle` | **falsch** [G] |
| Hüllenzahlen 32 (`mcp`) und 30 (`contextra`) als Startwert | Gelten nur mit allen Features. Default-Build ca. 26 bzw. 23 | **Metrik unklar** [H] |
| Kernschnitt GV-1: `check-dag` kann nicht rot werden | Bestätigt. Zusätzlich: Zyklen werden verschluckt | **bestätigt, verschärft** [G] |
| Kernschnitt: Ring-Verletzungen „nicht messbar“ | Es gibt einen Ring-Gate mit eigener Matrix, in der CI mit `--strict` | **falsch** [G] |
| Kernschnitt K2a Schritt 5: „Kante auf `core` entfernen“ in einem kleinen PR | 110 Dateien betroffen, rund 300 Zeilen | **unterschätzt** [G] |
| Kernschnitt Abschnitt 3, Regel 1: Ring kommt aus `capabilities.toml` | Ring steht an drei Stellen: `capabilities.toml`, `Cargo.toml`-Metadaten und hartkodiert im Gate (Abschnitt 6.2) | **unvollständig** [G] |
| Plan D5 / Kernschnitt D3: `rank` gehört zu EP3 bzw. Kern | `rank` bleibt im Kern. Ungenutzte Teile sind Parkplatz | **übernommen** |
| `with_kv_hooks` fehlt in Produktion (Plan 4.4) | Hooks werden in `lifecycle.rs` direkt im Struct gesetzt. Nur der Builder ist ungenutzt | **präzisiert** [G] |

---

## 3. Befunde am HEAD

### 3.1 Sicherheitsbefunde

| ID | Befund | Stand am HEAD | Quelle |
|---|---|---|---|
| B1 | Cloud-Egress ohne Guard | **bestätigt.** `contextra_cloud_query` ist in der Werkzeugliste registriert (`server_dispatch.rs`). `with_egress_guard` wird außerhalb von Tests nie aufgerufen. | [G] |
| B2 | Selbsttest ohne Aufrufer | **bestätigt.** `verify_wiring(bool, bool, bool)`, nur Test-Aufrufer. | [G] |
| B3 | `security_ports.toml` fehlt | **bestätigt.** `governance/` enthält die Datei nicht. | [G] |
| B6 | MCP-Werkzeuge aus Parkplatz | **bestätigt** für `contextra_cloud_query`, `contextra_consolidate` und `contextra_relate*`. | [G] |
| B9 | zwei Wege für `remember`/`recall` | **bestätigt.** `remember` hat drei Produktionstreffer, der MCP hat eine eigene Verarbeitung. | [G] |

### 3.2 Die Löschkette im Einzelnen (B4, korrigiert)

Der Pfad vom MCP bis zur Löschung ist heute:

```
contextra_forget (MCP, server_tools.rs)
  └─ id gesetzt:  col.delete(id)                          [G]
       └─ Collection::delete → erase_document(id, default) [G]
            ├─ Footprint berechnen                         [G]
            ├─ Ledger-Eintrag (ledger_append)              [G]
            ├─ Tombstones                                  [G]
            ├─ KV-Segmente entfernen (kv_hooks)            [G]
            ├─ Layer-Zeugen (Store, HNSW, Text, Relationen, Ledger) [G]
            └─ liefert DocErasure zurück                   [G]
       └─ Ergebnis wird verworfen, Antwort: proof = null, proof_scope = "collection_only" [G]
  └─ id nicht gesetzt: drop_collection(proof_key)          [G]
```

Die folgenden Teile sind **ohne Produktionsaufrufer** [G]:

- `DpiDeletionRuntime::open` (Nachweis-Laufzeit, nur Tests)
- `DpiPurgeDriver::new` und `run_physical` (physischer Purge)
- `generate_kv_deletion_proof`
- `read_kv_segment`, `write_kv_segment`, `delete_kv_segment`
- `KeyRegistry::open_durable`
- `scrub_and_remove` (nur intern in `durable-fs`)

In `erase_document` wird der Parameter `_opts` nicht gelesen. Assurance- und Purge-Optionen aus `forget_options` haben damit keine Wirkung auf den Einzeldokument-Pfad [G].

**Konsequenz für K1/K3:** Der Nachweis ist im Engine-Code schon zum großen Teil vorhanden. Die erste Maßnahme ist daher keine neue Logik, sondern das Durchreichen des `DocErasure` in die MCP-Antwort mit Status `belegt` oder `nicht_belegt` (Akzeptanztest in Abschnitt 8). Erst danach folgt die Nachweis-Ausstellung.

### 3.3 Verdrahtungsstand der 333 Funktionen (Anhang A)

Alle 333 Definitionen existieren noch [G]. Die Aufrufprüfung ist eine Heuristik [H]: Es wird nach `name(`-Mustern in Nicht-Testcode gesucht, `tests.rs`, `tests/`, `benches/`, `examples/` und `fuzz/` ausgeschlossen.

| Endprodukt (neue Zuordnung) | V | A | P | T | L | G | U | Summe | davon mit Treffer [H] |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| EP1 Contextra Memory (Kern) | 111 | 10 | 9 | 21 | 8 | 6 | 0 | 165 | 13 |
| EP2 Contextra Proof | 0 | 0 | 13 | 0 | 0 | 0 | 3 | 16 | 0 |
| EP3 Contextra Retrieval+ | 0 | 0 | 40 | 0 | 2 | 0 | 0 | 42 | 4 |
| EP4 Contextra Inference | 0 | 0 | 21 | 0 | 0 | 0 | 0 | 21 | 2 |
| EP5 Contextra Cognition | 0 | 0 | 59 | 0 | 0 | 0 | 0 | 59 | 3 |
| EP6 Contextra Guard | 0 | 0 | 15 | 0 | 0 | 0 | 0 | 15 | 1 |
| LAB Interne Werkzeuge | 0 | 0 | 0 | 15 | 0 | 0 | 0 | 15 | 1 |
| **Summe** | **111** | **10** | **157** | **36** | **10** | **6** | **3** | **333** | **24** |

Die Zuordnung weicht an zwei Stellen vom Plan v6 ab. Die 9 `rank`-Funktionen wechseln zu EP1, weil das Crate im Kern bleibt (D3). Die Funktionen bleiben aber geparkt. Die Lizenzfunktionen bleiben bei EP2, weil sie zur Aktivierung gehören (D4).

**Die 111 EP1-Funktionen mit Aktion V nach Crate** (ohne Treffer, also 101 Stück):

| Crate | ohne Treffer | Bemerkung |
|---|---:|---|
| `contextra-engine` | 29 | größter Block; Löschkette, Collection, Lifecycle |
| `contextra-vector` | 14 | HNSW, Residue-Zeugen |
| `contextra-crypto` | 12 | Schlüsselaufbau, Nachweis-Typen |
| `contextra-mvcc` | 10 | SSI-Pfade |
| `contextra-store` | 10 | WAL, LSM, Compaction |
| `contextra-types` | 8 | Typen, meist Konstruktoren |
| `contextra-checkpoint` | 5 | |
| `contextra-text` | 4 | |
| `contextra` (Fassade) | 3 | `builder`-Funktionen |
| `contextra-wire` | 3 | Encoding-Hilfen |
| `contextra-mcp` | 2 | |
| `contextra-durable-fs` | 1 | `scrub_and_remove`-Umfeld |

**Die 24 Heuristik-Treffer** sind zum Teil echt und zum Teil Namenskollisionen. Die manuelle Einordnung steht in der Spalte `pruefnotiz` der CSV. Eine Kollision ist wahrscheinlich bei `digest` (Treffer im Signer, Funktion aus `sandbox`). Bei `set_permissions` ist es unklar und muss geprüft werden. `root_as_search_response_with_opts` hat nur einen Treffer im Fuzz-Target und zählt damit faktisch nicht.

Die vollständige Zuordnung steht in `contextra_funktionszuordnung_v7.csv` (Trennzeichen `;`).

---

## 4. Zielbild: Endprodukte und Module

### 4.1 Endprodukte

Die Kennungen EP1 bis EP6 und LAB bleiben aus Plan v6 erhalten, damit die Verweise stabil bleiben. Die Inhalte ändern sich an zwei Stellen: `rank` gehört zum Kern, und der Kern hat keine Rand-Crates mehr.

| ID | Endprodukt | Versprechen | Zeitpunkt | Crates |
|---|---|---|---|---|
| **EP1** | **Contextra Memory** | Lokaler Speicher für Agenten: Dokumente, Hybridsuche, Mandanten, Löschung mit Nachweis. Rust-Bibliothek und MCP-Server. | **Release 1.0 (einziges Ziel jetzt)** | `types`, `ports`, `sys`, `wire`, `simd`, `durable-fs`, `crypto` (ohne Verifikation), `mvcc`, `text`, `rank` (nur Fusion und Kalibrator), `store`, `vector`, `checkpoint`, `engine` (Default ohne Rand), `db`, `license` (Sicherheits-Port), `contextra`, `mcp` |
| **EP2** | **Contextra Proof** | Unabhängige Prüfung von Löschnachweisen und Audit-Ketten. Kostenpflichtiger Compliance-Export. Eigener Verifier, möglichst in einer zweiten Sprache. | nach K4, eigenes Repository (Annahme, Abschnitt 10) | `audit-export`, `avv-generator`, Verifikation aus `crypto`, Lizenzaktivierung |
| **EP3** | **Contextra Retrieval+** | Beziehungen zwischen Dokumenten (Graph), Domänenvokabulare. | Jahr 2 | `graph` |
| **EP4** | **Contextra Inference** | Lokale Embedder, Reranker, Generatoren, KV-Cache. | Jahr 3, nur mit Break-even-Gate | `infer-candle`, `infer-ollama`, `infer-onnx` (Experiment), `kvcache` |
| **EP5** | **Contextra Cognition** | Konsolidierung, Vergessensdynamik, adaptive Parameter, Modellrouting, Agentenlauf. | experimentell, keine Zusage | `cognition`, `adapt`, `router`, `agent` |
| **EP6** | **Contextra Guard** | Kontrollierter Datenaustritt an Cloud-Dienste, Surrogat-Vault, WASM-Plugins mit Fuel-Begrenzung. | Jahr 3 oder nach Bedarf | `privacy`, `sandbox` |
| **LAB** | Interne Werkzeuge | Testkit, Benchmarks, Gates, Fault-Injection. Wird nie ausgeliefert. | laufend | `testkit`, `benchmarks`, `xtask`, `xtask-heavy` |

**Python-Bindings (`py`)** gehören zu EP1, werden aber erst nach K4 gebaut, ohne eigene Logik.

**Ring-Zuordnung** bleibt wie in `capabilities.toml` (Abschnitt 6.2 zeigt die Gate-Regeln).

### 4.2 Module und Aktivierung

Jedes Endprodukt besteht aus Crates. Zwei Mechanismen steuern, was in einem Produkt aktiv ist:

1. **Cargo-Features mit `default = []` für alle Rand-Module.** Ein Rand-Crate wird nur über ein benanntes Feature aktiviert. Die Default-Features des Kerns enthalten keine Rand-Crates.
2. **Produkt-Manifest** (`products/<ep>.toml`), das festlegt, welche Crates und Features zu einem Produkt gehören. Ein Gate baut das Produkt mit genau diesen Features und prüft, dass die Hülle nur aus erlaubten Crates besteht.

Beispiel für EP1 (Entwurf, noch nicht angelegt):

```toml
[product]
id = "EP1"
name = "Contextra Memory"
status = "release-1.0"

[crates]
allowed = [
  "contextra-types", "contextra-ports", "contextra-sys", "contextra-wire",
  "contextra-simd", "contextra-durable-fs", "contextra-crypto", "contextra-mvcc",
  "contextra-text", "contextra-rank", "contextra-store", "contextra-vector",
  "contextra-checkpoint", "contextra-engine", "contextra-db", "contextra-license",
  "contextra", "contextra-mcp",
]

[features]
# Features, mit denen das Produkt gebaut wird. Ohne sie kein Rand-Crate.
build = ["fast"]
```

Das Gate `check-product-hull --product EP1` führt `cargo tree -p contextra-mcp -e normal --no-default-features --features <build>` aus und schlägt fehl, wenn ein Crate außerhalb von `allowed` auftaucht. Das gleiche Gate läuft mit allen Features als Bericht, aber nicht als Fehler.

**Ein Modul ist abschaltbar**, wenn (a) es hinter einem Feature liegt, (b) das Produkt-Manifest es nicht listet, und (c) der Build ohne das Feature besteht. Punkt (c) wird in K2a je Modul geprüft.

### 4.3 Metriken für die Hülle

| Metrik | Definition | Startwert [H] | Ziel K2 | Ziel K4 |
|---|---|---:|---:|---:|
| Hülle EP1, Default-Features, `mcp` | Anzahl Workspace-Crates in `cargo tree -e normal` | ca. 26 | ≤ 18 | 18 |
| Hülle EP1, alle Features (Bericht) | wie oben mit `--all-features` | 32 | Bericht | Bericht |
| Kern→Parkplatz-Kanten | Kanten aus Kern-Crates auf Parkplatz-Crates, inkl. optional | 26 (10 regulär) [G] | 0 regulär | 0 |
| Ring-Verletzungen (Gate-Emulation) | siehe Abschnitt 6 | 6 [E] | 0 | 0 |
| Unverdrahtete `pub fn` in EP1 | Aktion V ohne Produktionsaufruf | 101 [H] | 0 | 0 |

Die Hülle wird standardmäßig mit Default-Features gemessen (D10, Annahme).

---

## 5. Meilensteine

Der Ablauf folgt dem Kernschnitt-Dokument, mit den Korrekturen aus Abschnitt 2. Die Zeitangaben sind Größenordnungen für eine Person mit Agentenunterstützung.

### K0 — Wahrheit (ca. 1 Woche)

Die Messinstrumente müssen verlässlich sein, bevor etwas umgebaut wird. Diese Aufgaben ändern geschützte Pfade (`xtask/**`, `governance/**`, `.github/**`) und sind deshalb **nicht an Agenten delegierbar**, sondern brauchen ADR und Maintainer-Review.

| Nr. | Aufgabe | Akzeptanz |
|---|---|---|
| K0.1 | `compute_crate_layers`: Fehler propagieren, Zyklen in `check-dag` als Verstoß melden | Negativtest mit absichtlichem Zyklus wird rot |
| K0.2 | `check-dag` um Ring-Prüfung nach `capabilities.toml` ergänzen, oder als veraltet markieren und durch `check-ring-layering` ersetzen | Negativtest mit verletzender Testcrate wird rot |
| K0.3 | `ring0_crate_order` um `contextra-durable-fs` ergänzen. Dev-Abhängigkeit `contextra-store` → `contextra-sandbox` entfernen (Befund aus Abschnitt 6.3) | Gate-Lauf zeigt diese zwei Verletzungen nicht mehr |
| K0.4 | Ring-Gate einmal mit `--strict` in CI laufen lassen und Ergebnis dokumentieren | Ergebnis steht im Dokument, Status [U] wird zu [G] |
| K0.5 | Eine einzige Unwrap-Baseline. Heute widersprechen sich `governance/ratchet.toml` (`unwrap_exact = 502`) und `.github/unwrap_baseline.txt` (`786`) | Eine Quelle, Ratsche nur nach unten |
| K0.6 | `governance/security_ports.toml` anlegen; Gate prüft, dass jeder Eintrag eine reale Implementierung hat | Gate in CI |
| K0.7 | `prune-branches` reparieren oder abschalten (Skript fehlt laut Plan 14.6, [U]) | Workflow läuft grün oder ist aus |
| K0.8 | Toolchain 1.89.0 in Agenten-Umgebung und CI fixieren | `cargo xtask harness-list` läuft |
| K0.9 | Hülle messen (Abschnitt 4.3) und als Ausgabe des Gates festschreiben | Zahl ist reproduzierbar |

### K1 — Fail-closed (ca. 1 bis 2 Wochen, parallel zu K0 ab Tag 3)

| Nr. | Aufgabe | Akzeptanz |
|---|---|---|
| K1.1 | `contextra_cloud_query` nur mit Feature `guard` registrieren (D1). Im EP1-Build nicht erreichbar | Test: Aufruf liefert „Werkzeug nicht verfügbar“ |
| K1.2 | `verify_wiring` mit echten Port-Instanzen, Pflichtliste aus `security_ports.toml` (include_str!), im Startpfad | Test: Start ohne Sicherheits-Port schlägt fehl |
| K1.3 | `contextra_forget` gibt den `DocErasure`-Befund zurück: `belegt` oder `nicht_belegt` mit Schichtenliste, nie `null` | Test über die MCP-Schnittstelle, Einzeldokument und Kollektion |
| K1.4 | `erase_document` liest `_opts` oder der Parameter wird entfernt. Keine stillen Optionen | Test: Assurance-Option ändert das Ergebnis |
| K1.5 | Krypto-Review (OP-R15) beginnt, Befund-Liste oder Schließung | Liste steht im Dokument |

### K2 — Entkoppeln, dann schneiden (ca. 3 bis 5 Wochen)

**K2a: Kanten schneiden, Crates bleiben im Baum.** Jeder PR bleibt unter etwa 400 geänderten Zeilen. Die Reihenfolge folgt dem Kernschnitt: unten zuerst, `mcp` zuletzt.

| Schritt | Änderung | Hülle EP1 (Default) danach |
|---|---|---|
| 1 | `engine`: `default = []` und Default-Feature-Auflösung prüfen. Wo ein Rand-Crate ohne Feature nicht kompiliert: Port oder `cfg` | Ziel: `engine` ohne `graph`, `adapt`, `kvcache`, `candle` |
| 2 | `db`: `cognition`, `graph`, `adapt` entfernen (zusammen 6 Dateien betroffen) | |
| 3 | `mcp`: `privacy` und `adapt` regulär; hinter Feature oder Port bringen. Werkzeugliste aus der Registrierung statt Konstante | |
| 4 | `contextra` (Fassade): `candle` aus den Default-Features nehmen | |
| 5 | `contextra-core` entfernen: 110 Dateien, rund 300 Zeilen. Symbole aus `types`, `ports`, `mvcc`, `wire` importieren. **Als 3 bis 4 mechanische PRs, nach Modul getrennt** | |
| 6 | `default-members` auf EP1 und LAB setzen | |

**K2b: Mechanische Löschung.** Erst wenn die Hülle das Ziel erreicht und `check-ring-layering --strict` grün ist, entfernt ein einziger PR (Label `scope:core-cut`) die Parkplatz-Crates aus Workspace und Dateibaum. Der Diff ist groß, das Risiko klein, weil nichts mehr daran hängt. Vorher werden Tags `park/<ep>/v1` auf den Ausgangsstand gesetzt, damit nichts verloren geht.

**Austritt K2:** `cargo build` baut EP1 und LAB. Default-Hülle EP1 ≤ 18. Kern→Parkplatz-Kanten regulär = 0. `capabilities.toml` führt je Crate `product` und `status`. Das Produkt-Manifest existiert für EP1.

### K3 — Beweisbar vergessen (ca. 2 bis 3 Monate)

Das ist das Produktversprechen und der Hauptteil der Arbeit. Die Reihenfolge ist verbindlich:

1. **Krypto-Befunde schließen** (Schlüsselaufbau, Nonce-Länge, `unsafe` im Test). Vorrang vor allem anderen.
2. **Rohträger-Test schreiben, bevor die Nachweis-Kette verdrahtet wird.** Er stammt aus der Spezifikation, nicht vom implementierenden Agenten, und ist zunächst rot.
3. **Nachweis-Ausstellung verdrahten:** `DpiDeletionRuntime::open` im Startpfad, `finalize_logical` im Löschpfad, Signer-Schlüssel aus Konfiguration.
4. **Purge verdrahten:** `DpiPurgeDriver` mit `run_physical` nach `erase_document`, `scrub_and_remove` im Purge-Pfad.
5. **KV-Nachweis verdrahten:** `generate_kv_deletion_proof`, `read/write/delete_kv_segment` in den Lifecycle. `KeyRegistry::open_durable` im Startpfad.
6. **Lineage-Kaskade** (F07): Quelllöschung entfernt abgeleitete Daten. Ohne sie darf kein Aggregat-Feature aktiv werden.
7. **`proof` wird erst `belegt`, wenn der Rohträger-Test grün ist.**

**Austritt K3:** Die acht kritischen Verdrahtungen aus Plan 4.4 laufen über Integrationstests auf dem echten Pfad. Rohträger-Test grün. `forget` liefert für Einzeldokumente einen belegten Nachweis.

### K4 — Crash-sicher und vollständig verdrahtet (ca. 2 bis 3 Monate)

Compaction und MANIFEST (OP-R12), HMAC-Zustand (OP-R7), Group-Commit-Liveness (OP-R8), Checkpoint-Indizes (OP-R13), atomares `put_if_absent` (OP-R1). Jeder Prüfauftrag liefert einen Test, der den Befund belegt, oder wird als [V] geschlossen. Fault-Injection und Loom werden Pflicht-Gates.

**Austritt K4:** Zähler „öffentliche Funktionen ohne Aufrufer und ohne Aktion“ = 0 für EP1. Crash-Test-Suite grün. Release `v0.1.0`. Danach beginnt EP2 als eigenes Repository.

**Eingefroren bis K4 abgeschlossen:** Optimierungen (F01 bis F06), `py`, EP2 bis EP6, und alle Features außer F07, F08, F09, F54.

### K5 — Erweiterungen (nach K4, je Produkt eigenes Gate)

Jedes Endprodukt bekommt erst einen eigenen Startpunkt, wenn EP1 die Akzeptanz erfüllt. EP2 zuerst, weil es die Trennung von Erzeuger und Prüfer herstellt. EP3 bis EP6 erst nach Bedarf.

---

## 6. Gates und Ring-Regeln

### 6.1 Was heute geprüft wird

| Gate | Was es prüft | Kann es rot werden? |
|---|---|---|
| `check-dag` | Layer aus Abhängigkeiten abgeleitet, dann Vergleich | **Nein** [G]: Layer ist `max(Abhängigkeiten)+1`. Bei Zyklen liefert die Berechnung einen Fehler, der mit `let _ =` verworfen wird. Alle Layer bleiben 0. |
| `check-ring-layering --strict` | Kanten gegen Ring-Matrix mit Allowlist | **Ja**. Emulation zeigt sechs nicht erlaubte Verletzungen [E]. CI-Status [U]. |
| `check-ring-capabilities-consistency` | Konsistenz zwischen Manifest und Cargo | [U], nicht geprüft |

### 6.2 Die Ring-Regeln (Gate-Logik)

Die Regeln des Gates sind nicht die des Kernschnitts. Das Gate kennt:

- Ring 0 darf nur auf Ring 0 nach einer hartkodierten Ordnung (`ring0_crate_order`) zeigen. Crates ohne Eintrag bekommen Ordnung 99. Jede Abhängigkeit eines gelisteten Crates auf ein ungelistetes Crate ist deshalb eine Verletzung.
- Ring 1 darf auf Ring 0 zeigen, nicht auf Ring 1.
- Ring 2 darf auf Ring 0 nur `types`, `ports`, `crypto`, `core`, `simd`, `rank` zeigen.
- Ring 3 darf auf Ring 0 und Ring 1 zeigen, nicht auf konkrete Ring-2-Crates. Innerhalb Ring 3 gilt die Ordnung `ring3_crate_order`.
- Ring 4 darf auf alles zeigen.
- Dev-Abhängigkeiten auf Tooling sind immer erlaubt, Dev-Abhängigkeiten innerhalb Ring 0 und Ring 3 ebenfalls. Alle anderen Dev-Abhängigkeiten werden geprüft.

Die Gate-Regeln sind in Code festgeschrieben, nicht in `capabilities.toml`. Die Kernschnitt-Regel 1 („nur abwärts oder auf denselben Ring“) und die Gate-Regeln sind nicht deckungsgleich. Das Gate verbietet Ring 1 → Ring 1 und Ring 2 → Ring 2 (außer dev), die Kernschnitt-Regel würde gleichen Ring erlauben. Die Regel muss vor K2 einmal festgelegt werden.

**Hinweis zu den Quellen:** Die Ring-Angabe steht an drei Stellen: `capabilities.toml` (Feld `ring`), `Cargo.toml` (`[package.metadata.contextra].ring`) und hartkodiert im Gate (`ring0_crate_order`, `ring3_crate_order`, Allowlist). Die ersten beiden stimmen überein [G]. Der dritte Ort ist eine stille Kopie der Ordnung und muss in K0 zusammengeführt werden.

### 6.3 Emulation des Ring-Gates [E]

Die Emulation nutzt alle deklarierten Abhängigkeiten, inklusive optional und dev, weil `cargo metadata --no-deps` sie ebenfalls listet. Zielabhängigkeiten für `target.*`-Abschnitte habe ich nicht erfasst.

| Von | Nach | Art | Regel | Erlaubt? |
|---|---|---|---|---|
| `contextra-engine` | `contextra-sandbox` | normal, optional | Ring 3 → konkretes Ring-2-Crate | nein |
| `contextra-engine` | `contextra-infer-candle` | normal, optional | Ring 3 → konkretes Ring-2-Crate | nein |
| `contextra-cognition` | `contextra-infer-candle` | dev | Ring 3 → konkretes Ring-2-Crate | nein |
| `contextra-crypto` | `contextra-durable-fs` | normal | Ring-0-Ordnung (durable-fs nicht gelistet, Ordnung 99) | nein |
| `contextra-vector` | `contextra-durable-fs` | normal | Ring-0-Ordnung (dito) | nein |
| `contextra-store` | `contextra-sandbox` | dev | Ring 1 → Ring 2 | nein |

Drei weitere Verletzungen sind mit Allowlist-Einträgen (Ablauf 31.12.2026) zugelassen. Die Allowlist-Gründe sind: Dev-Abhängigkeit von `graph` auf `store`, `infer-onnx` auf `infer-candle`, `infer-ollama` auf `infer-onnx`.

**Bedeutung:** Wenn die CI so läuft, wie die Emulation sagt, wäre der Job `Ring Layering Check` im `merge-gate.yml` rot. Das ist unwahrscheinlich, weil dann Pull Requests nicht durchgehen würden. Entweder läuft der Job nicht als Pflicht, oder meine Emulation erfasst etwas nicht (z. B. Zielabhängigkeiten oder den Lauf mit `--all-features`). **Prüfen: Actions-Tab des letzten `main`-Laufs und Branch-Protection.** Die Antwort ändert die Priorität von K0.4.

### 6.4 Core→Parkplatz-Kanten [G]

Kern-Crates nach Kernschnitt-Definition: 18. Parkplatz-Crates: 15.

| Von (Kern) | Nach (Parkplatz) | Art |
|---|---|---|
| `contextra` | `contextra-core` | regulär |
| `contextra-checkpoint` | `contextra-core` | regulär |
| `contextra-db` | `contextra-cognition` | regulär |
| `contextra-db` | `contextra-graph` | regulär |
| `contextra-db` | `contextra-adapt` | regulär |
| `contextra-mcp` | `contextra-adapt` | regulär |
| `contextra-mcp` | `contextra-privacy` | regulär |
| `contextra-simd` | `contextra-core` | regulär |
| `contextra-store` | `contextra-core` | regulär |
| `contextra-vector` | `contextra-core` | regulär |
| — | — | 16 weitere Kanten, alle optional (Liste in der CSV `ring_gate_emulation_v7.csv` und im Gate-Lauf) |

Die fünf Kanten auf `contextra-core` verschwinden mit K2a Schritt 5. Die übrigen fünf regulären Kanten (`db` auf Parkplatz, `mcp` auf `adapt` und `privacy`) brauchen Ports oder Feature-Gates.

### 6.5 Anzahl der Nutzungsstellen [G]

| Kante | Dateien | Zeilen | Mit `cfg(feature)` |
|---|---:|---:|---:|
| `engine` → `graph` | 19 | 90 | 9 |
| `engine` → `adapt` | 6 | 8 | 3 |
| `engine` → `kvcache` | 3 | 4 | 2 |
| `engine` → `infer-candle` | 3 | 4 | 2 |
| `engine` → `sandbox` | 1 | 2 | 0 |
| `db` → `cognition` | 1 | 12 | 1 |
| `db` → `graph` | 2 | 2 | 1 |
| `db` → `adapt` | 3 | 6 | 1 |
| `mcp` → `privacy` | 3 | 6 | 0 |
| `store`, `vector`, `simd`, `checkpoint` → `core` | 110 | rund 300 | 22 |

`engine` → `graph` ist mit 90 Zeilen der aufwendigste Teil von K2a Schritt 1. Die 10 Dateien ohne `cfg`-Bezug müssen einzeln geprüft werden. Die Spalte zählt Dateien mit irgendeinem `cfg(feature)`, nicht die betroffene Stelle.

---

## 7. Betriebsmodell für Agenten

### 7.1 Engpass

Mit 195 Jules-Sitzungen pro Tag ist die Sitzungszahl kein Engpass. Der Engpass ist (a) die Prüfkapazität des Maintainers, (b) Merge-Konflikte bei Änderungen in denselben Crates und (c) das Risiko, dass ein falsch grüner Test eine Änderung durchlässt. Die Kernschnitt-Begrenzung „höchstens zwei offene PRs“ bremst deshalb zu grob. Besser ist eine Begrenzung je Crate und je Prüfer.

**Vorschlag (Annahme D11):**

- Höchstens drei offene PRs, und jeder PR betrifft genau ein Crate oder eine Gate-Datei.
- Zwei Karten dürfen nicht dasselbe Crate ändern, solange eine offen ist.
- Protected-Path-Karten (`xtask/**`, `governance/**`, `.github/**`, `capabilities.toml`, `AGENTS.md`) gehen nur an einen Maintainer, nicht an Agenten.

### 7.2 Kartenformat

Jede Karte hat diese Felder. Ohne vollständige Karte startet kein Agent.

| Feld | Inhalt |
|---|---|
| `product` | `EP1` oder `LAB`. Andere Werte werden vom Gate abgelehnt. |
| `scope` | Liste der erlaubten Pfade. Außerhalb ist die Änderung ein Fehler. |
| `ziel` | Eine Sache, messbar. |
| `messgroesse` | Welche Zahl sich ändert, mit Startwert und Ziel (Abschnitt 4.3). |
| `akzeptanz` | Testname, der über den echten Pfad läuft (P29). |
| `acceptance_by` | Wer den Abnahmetest schreibt: Spezifikation, Referenzmodell oder zweite Karte. Der implementierende Agent schreibt ihn nicht. |
| `verbot` | Was nicht angefasst werden darf (z. B. Parkplatz-Code lesen oder ändern). |

### 7.3 Regeln

| Regel | Durchsetzung |
|---|---|
| Keine neue `pub fn` ohne Produktionsaufrufer. Neue Funktionen sind `pub(crate)`, bis ein Aufrufer existiert | Gate: Anzahl unverdrahteter `pub fn` in EP1 darf nicht steigen |
| Keine neuen Crates oder Abhängigkeiten ohne ADR | `deny.toml`, Abhängigkeits-Gate |
| Parkplatz-Code wird weder gelesen noch geändert. Wer ihn braucht, meldet das und stoppt | `AGENTS.md`, Gate `product-scope` (neu) |
| Jede Verdrahtung hat einen Integrationstest über den echten Pfad | Review-Checkliste in der Karte |
| Zahlen aus Forschungsberichten sind ein Anlass für einen Versuch, nie Evidenz | INV-EVIDENCE-1 |
| Änderungen an geschützten Pfaden nur mit ADR | Protected-Path-Gate |

### 7.4 Neues Gate: `product-scope`

Ein PR darf nur Pfade ändern, die zum Produkt aus der Karte gehören. Das Gate vergleicht die geänderten Dateien mit dem Produkt-Manifest. Es ist die technische Form der Regel „Parkplatz wird nicht angefasst“.

---

## 8. Akzeptanzkriterien für die ersten Karten

Diese Karten sind bereit, in Jules gestartet zu werden. Die ersten drei sind wegen des geschützten Pfads Maintainer-Aufgaben.

| Karte | product | Pfad (scope) | Akzeptanz | acceptance_by |
|---|---|---|---|---|
| K0.1 Zyklen in check-dag | LAB | `xtask/src/lib.rs` | Negativtest mit Zyklus wird rot | Maintainer |
| K0.3 Ring-0-Ordnung | LAB | `xtask/src/check_ring_layering.rs` | Gate-Lauf ohne die zwei genannten Verletzungen | Maintainer |
| K1.3 forget gibt Befund zurück | EP1 | `crates/contextra-mcp/src/server_tools.rs`, `crates/contextra-engine/src/collection/crud/delete.rs` | MCP-Test: Antwort ist `belegt` oder `nicht_belegt`, nie `null` | Spezifikation (Abschnitt 4 Kernschnitt) |
| K1.1 cloud_query nur mit `guard` | EP1 | `crates/contextra-mcp/src/server_dispatch.rs`, `server_tools.rs`, `Cargo.toml` | Test: Werkzeug fehlt im EP1-Build | zweite Karte |
| K1.2 verify_wiring mit echten Ports | EP1 | `crates/contextra/src/wiring_selftest.rs`, Startpfad der Fassade | Start ohne Sicherheits-Port scheitert | zweite Karte |
| K2a.5a Symbole aus `core` nach `types` (Modul `store`, Teil 1) | EP1 | `crates/contextra-store/src/**` (max. 25 Dateien) | `cargo check -p contextra-store` ohne `core`-Abhängigkeit | Compiler |
| K3 Rohträger-Test | EP1 | `crates/contextra-engine/tests/**` | Test ist zunächst rot und schlägt ohne Implementierung fehl | Spezifikation |

---

## 9. Risiken

- **Kein Build in dieser Umgebung.** Alle Hüllen- und Gate-Zahlen mit [H] oder [E] sind Schätzungen. Die erste Aufgabe in K0 ist, sie mit der gepinnten Toolchain zu messen.
- **Defaults umschalten kann Kompilierfehler aufdecken.** Das größte Terminrisiko liegt in K2a Schritt 1 (`engine` ohne `graph`, 19 Dateien).
- **Die Zahl unverdrahteter Funktionen ist eine obere Schranke.** Die Heuristik findet nur direkte `name(`-Aufrufe. Funktionszeiger und Makro-Aufrufe werden übersehen, die Zahl 101 ist daher eher zu hoch.
- **Die Löschung bleibt Selbstaussage**, bis ein unabhängiger Prüfer (EP2) sie nachvollzogen hat. Das Wort „beweisbar“ erscheint in keinem Text, bevor Rohträger-Test und externe Prüfung vorliegen.
- **Geschützte Pfade** machen einen großen Teil von K0 nicht delegierbar. Das begrenzt den Durchsatz stärker als die Sitzungszahl.
- **EP1 bleibt groß.** Rund 137.000 Zeilen Rust vor dem Schnitt, davon 34.000 in `store`. Der Schnitt macht den Kern fokussiert, nicht klein.
- **Die CI-Frage in 6.3 ist offen.** Wenn der Ring-Gate tatsächlich rot ist, hat `main` ein Problem, das vor allen anderen Arbeiten steht.
- **Nicht geprüft:** Dev- und Build-Abhängigkeiten in `target.*`-Abschnitten, Kryptografie, Compaction, Gateway-Logik, die beiden CSV-Listen des Vorgängerplans (die Anhangs-CSVs liegen nicht im Repository), Jules-Betriebsangaben aus Plan 16.

---

## 10. Entscheidungen

Die folgenden Punkte brauchen eine Entscheidung. Die Spalte „Stand“ zeigt, ob der Plan bereits davon ausgeht.

| # | Frage | Vorschlag | Stand |
|---|---|---|---|
| D1 | `contextra_cloud_query` in 1.0? | Nein, nur mit Feature `guard` | Annahme |
| D2 | Legacy-WAL (`migrate_legacy_wal`, `with_legacy_fallback`)? | Entfernen. Formatstabilität gilt ab 1.0 | Annahme |
| D3 | `rank` im Kern? | Ja, nur Fusion und Kalibrator. Rest geparkt | Annahme |
| D4 | `license` im Kern? | Ja, solange `LicenseGate` Sicherheits-Port ist. Aktivierung gehört zu EP2 | Annahme |
| D5 | `graph` für Hybridsuche nötig? | Nein. Gate: `engine` besteht Tests ohne `graph` | Annahme, zu prüfen in K2a |
| D6 | `remember`/`recall` oder MCP-Verarbeitung als einziger Weg? | Ein Weg: MCP ruft die Fassade | Annahme |
| D7 | `cognition`, `adapt`, `router`, `agent` | Bleiben als Parkplatz im Repository, getaggt. Entscheidung über eigenes Repository oder Löschung vor K2b | offen |
| D8 | Python-Bindings | Nach K4, dünn, ohne eigene Logik | Annahme |
| D9 | **EP2 als eigenes Repository nach K4?** | Ja. Erzeuger und Prüfer dürfen keinen Code teilen. Das Produkt-Manifest bleibt trotzdem der Mechanismus für Aktivierung | **Bitte bestätigen** |
| D10 | Welche Feature-Konfiguration für die Hüllenzahl? | Default-Features als Gate, alle Features als Bericht | **Bitte bestätigen** |
| D11 | Begrenzung der Agenten | Drei offene PRs, ein Crate je PR, keine zwei Karten auf demselben Crate | Vorschlag |
| D12 | Ring-Gate: Pflicht oder beobachten? | Erst CI-Status klären (6.3), dann entscheiden | offen |

---

## 11. Was dieses Dokument nicht leistet

- Es ist kein Ersatz für einen Build. Keine der Gate-Zahlen ist mit `cargo` gemessen.
- Die Heuristik zur Verdrahtung ist eine Schätzung. Die Spalte „Prüfnotiz“ der CSV enthält die bekannten Kollisionen. Weitere sind wahrscheinlich.
- Die 867 undokumentierten Elemente und ihre Zuordnung aus Plan v6 sind in diesem Dokument nicht enthalten. Die CSV dazu lag im Vorgängerplan nicht im Repository.
- Die Spezifikation v4 wurde im Repository nicht geprüft.
- Die Angaben zum Jules-Betrieb aus Plan 16 wurden nicht verifiziert.

---

## Anhang: Dateien

| Datei | Inhalt |
|---|---|
| `Contextra_Endprodukt_Plan_v7.md` | Dieses Dokument |
| `contextra_funktionszuordnung_v7.csv` | 333 Zeilen. Spalten: `nr`, `crate`, `funktion`, `datei`, `plan_endprodukt`, `plan_aktion`, `endprodukt`, `aktion`, `status_head`, `treffer_prod`, `erster_treffer`, `pruefnotiz`, `begruendung_plan`. Trennzeichen `;` |
| `ring_gate_emulation_v7.csv` | Ergebnis der Emulation des Ring-Gates. Trennzeichen `;` |
