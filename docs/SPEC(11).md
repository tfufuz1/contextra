# Contextra als Baukasten — Spezifikation v5 (normativ)

Nov 11, 2026 · @kiki · Revision „Restarbeit“

## 1 Status, Geltung, Änderungsprotokoll

Diese Spezifikation v5 ersetzt v4 und ist die einzige normative Quelle für Crate-Grenzen, Abhängigkeitsrichtung, Brücken, Features, Persistenzformate, Gates und die Reihenfolge der Entkopplung. Sie ändert die Zielarchitektur von v4 nicht. Sie gleicht den Plan mit dem Repository ab, führt in §9 nur noch offene Arbeit, schließt die in v4 fehlenden Anhänge A und B und korrigiert Aussagen, die der Code widerlegt hat.

### 1.1 Grundlage und Grenzen

| Punkt | Inhalt |
| --- | --- |
| Geprüfter Stand | Repository `tfufuz1/contextra`, HEAD `fcd387021` (11.11.2026, PR #6032). v4 beschrieb `ec6bd67` (PR #6018), also 14 PRs früher. |
| Methode | Frischer Klon, Auswertung aller `Cargo.toml` per Skript (Anhang B), `grep`, Dateilisten, Lesen einzelner Dateien. **`cargo` stand nicht zur Verfügung: Es wurde nichts gebaut, kein Test und, bis auf `tools/gate_bridge_null.py`, kein Gate ausgeführt.** Jede Zahl ist eine Näherung. |
| Belegmarken | **\[V\]** am HEAD per Datei, `grep` oder `Cargo.toml`-Auswertung geprüft (nicht gebaut) · **\[V\*\]** aus einem eigenen Feature-Resolver berechnet, nicht mit `cargo tree` bestätigt · **\[E\]** Einschätzung, nicht belegt · **\[D\]** Festlegung dieser Spezifikation · **\[Ü\]** aus v3/v4 übernommen, nicht neu geprüft. |
| Ist-Stand | Messwerte stehen nicht in dieser Spezifikation (D-15), sondern in generierten Berichten (§8.3). **Anhang C** ist eine einmalige Momentaufnahme mit HEAD-Hash und Datum und wird durch den Bericht aus S-00 ersetzt. |
| Crate-lokale Dokumente | Sie regeln die Binnenstruktur und sind verbindlich, soweit sie dieser Spezifikation nicht widersprechen. Ihre Zahl (v4: 39) wurde nicht neu gezählt \[Ü\]. |

### 1.2 Rangfolge

1. Diese Spezifikation und das Entscheidungsregister (§10). ADRs in `docs/decisions/` gelten nur, soweit sie ihr nicht widersprechen.
2. Crate-lokale `SPEC.md` und `PLAN.md`. Ein Widerspruch zu Punkt 1 ist ein Befund im Register (Anhang A).
3. `AGENTS.md` und `docs/**` sind nachrangig. `AGENTS.md` verweist auf eine nicht existierende Datei (`docs/CONTEXTRA_IMPLEMENTIERUNGS_SPEC.md`, Zeilen 4 und 84 \[V\]) und muss auf `docs/SPEC.md` zeigen (Schritt S-03).

### 1.3 Normative Sprache und Prüfbarkeit

MUSS, DARF NICHT und SOLL gelten nach RFC 2119. **Jede MUSS-Aussage hat ein Prüfverfahren aus §7 oder einen benannten Test.** Eine Aussage, deren Prüfverfahren nicht existiert, in `governance/ratchet.toml › not_run_baseline` steht oder in CI nicht blockiert (D-23), gilt nur als SOLL, bis das Verfahren läuft (D-04).

### 1.4 Änderungsprotokoll v4 → v5

| ID | Befund in v4 | Änderung in v5 |
| --- | --- | --- |
| W1 | Anhang A und B fehlen in der Datei, obwohl v4 (V1) sie als geschrieben meldet; fünf Verweise laufen ins Leere \[V\]. | Anhang A (Befundregister), B (Werkzeugliste) und C (Momentaufnahme) neu geschrieben. |
| W2 | §9 führt erledigte Arbeit mit. | §9.2 nennt je Schritt **Erledigt** und **Rest** am HEAD `fcd387021`. Kein Schritt ist vollständig erledigt; viele sind teilweise erledigt. |
| W3 | CI blockiert INV-4 und G-L1 nicht (`\|\| true`). | §7.1 und D-23: Gate-Schritte ohne `\|\| true` und ohne `continue-on-error`. S-01 erweitert (F-01, F-02). |
| W4 | §2.1 lässt offen, ob Default-Features der Abhängigkeiten in die Closure zählen. Sie tun es, und `vector` und `store` erben dadurch `types` und `ports` \[V\*\]. | §2.1 klargestellt; S-12a bekommt die Reihenfolge „`mvcc` und `crypto` zuerst“ (F-07). |
| W5 | INV-11 behauptet, `unsafe` außerhalb der Inseln stehe nur in Kommentaren. | Zwei Test-Binaries widerlegen das; Scope und Ausnahme in INV-11, Entscheidung D-21 (F-08). |
| W6 | §5.1: sechs `DocId`-Kopien, Differenztest gegen alle. | Fünf Kopien (`vector` ist erledigt). Der Differenztest deckt zwei Referenzen ab. Drei Kopien rechnen anders als `ids`; D-20 (F-04, F-05). |
| W7 | S-12b nennt fünf Crates mit verbotener dev-Kante. | Drei weitere: `rank`, `infer-candle`, `infer-ollama` (F-09). |
| W8 | S-12a nennt `prompt-guard` und lässt Features außerhalb der geschlossenen Liste unerwähnt. | `prompt-guard` ist erledigt; `graph`-Defaults (D-22) und `exp-*`-Umbenennungen konkret benannt. |
| W9 | §4.1: `kvcache` mit Kern `env → 1`. | `ids, env → 2` (HEAD, \[V\]). |
| W10 | R14 nennt nur `vault`. | Zwölf Crates mit `publish = false`, kein ADR (F-11). |
| W11 | §5.4 verlangt `version` an jeder internen Pfad-Abhängigkeit. | Am HEAD fehlt sie in 12 von 54 direkten Pfad-Abhängigkeiten und in allen 40 internen `[workspace.dependencies]`-Einträgen (F-12); S-03. |
| W12 | S-38 nennt `kv_shredding` und `revocation_log` als Teile von `crypto`. | Beide liegen am HEAD in `erasure`; `crypto` und `kvcache` führen parallele KV-Implementierungen (F-14). |
| W13 | D-13 „Verbraucher zu ermitteln“. | Ermittelt: kein Crate im Workspace hängt von `erasure` ab \[V\]. |
| W14 | Entscheidungsregister endet bei D-19. | D-20 bis D-23 ergänzt. |
| W15 | Verweise auf „S-12“ (§6.3, §8.1) und „S-30“ (§6.1) zeigen auf keinen Schritt \[V\]. | Auf S-12a, S-12b und S-30B korrigiert. |

### 1.5 Kurzbefund

- **Erledigt im Kern:** Crates `ids` und `env`; `compat/`-Brücken mit null ungegateten Brückenreferenzen in `simd`, `adapt`, `ingest`, `infer-onnx`, `license`, `audit-export`, `mvcc`, `crypto`, `text`, `rank`, `checkpoint`, `kvcache`, `vault`, `infer-ollama`; eigene Fehlertypen in allen E0/E1-Crates; `vault` als Crate samt Quelle; `SnapshotSource`, `PostingStore`; V-VETO-Wachtest-Datei; `forbid(unsafe_code)` in allen 37 Nicht-Insel-Crates von `crates/`.
- **Offen im Kern:** ungegatete Brückenreferenzen in `graph` (140), `store` (28), `vector` (14), `infer-candle` (7) \[V\]; verbotene Defaults in 13 Crates; verbotene dev-Kanten in 8 Crates; `rank` ohne `ids`/`env`; `ports` und `types` haben nichts abgegeben (S-50, S-51); `env` unverändert (S-11); `cognition` hängt noch an `engine` und `graph`.
- **Offen in der Durchsetzung:** 38 Gates in `not_run_baseline`, `golden`-Wurzelkorpus fehlt, `commit-msg`-Hook fehlt, CI blockiert INV-4 und G-L1 nicht (§7.3).

## 2 Begriffe, Ebenen, Entnehmbarkeit

Ein Baustein ist nur dann einsetzbar, wenn seine Abhängigkeiten berechenbar sind. Deshalb definiert dieser Abschnitt jede Größe so, dass ein Werkzeug sie ohne Auslegung ausgeben kann.

### 2.1 Begriffe

| Begriff | Definition |
| --- | --- |
| **Kern** | Der Code eines Crates mit `--no-default-features`, ohne das Feature `workspace`. |
| **Brücke** | Das Modul `src/compat/` hinter dem Feature `workspace`. Nur dort stehen `contextra_types`, `contextra_ports`, Implementierungen von `ports`-Traits und `From`-Konvertierungen nach `ContextraError`. |
| **Kern-Closure** | Alle internen `contextra-*`-Crates in der Ausgabe von `cargo tree -p <crate> -e normal --no-default-features`, **transitiv**, ohne das Crate selbst. Dev-Abhängigkeiten zählen nicht, werden aber durch R12 geregelt. **Die Default-Features der Abhängigkeiten gelten mit:** `--no-default-features` wirkt nur auf das Wurzel-Crate. Eine interne Kante ohne `default-features = false` zieht die Default-Features des Ziels und damit dessen Brücke in die Closure (F-07). |
| **Feature-Closure** | Wie die Kern-Closure, mit genau einem zusätzlich aktivierten Feature. |
| **Mikro-Kerne** | `contextra-ids` und `contextra-env`. Beide sind E0 und für jeden Baustein als Kernabhängigkeit erlaubt. E0-Crates behalten lokale Mini-Traits (R7). |
| **Anwendungs-Crate** | `contextra`, `contextra-mcp`, `contextra-py`, `contextra-bench`. Nur sie dürfen das nicht-additive Feature `docid-128` aktivieren (R10, D-02). |
| **Entnehmbar** | Ein Crate ist entnehmbar, wenn G-L2, G-L1, G-L3, G-L4 und G-H1 (§7) in dieser Reihenfolge grün sind. |

### 2.2 Ebenen

Die Ebene eines Crates folgt aus seiner Kern-Closure. Die Spalte „Kern“ in §4.1 gibt deshalb immer die **berechnete transitive Closure** an, nicht nur die direkten Einträge.

| Ebene | Definition | Crates (Ziel) |
| --- | --- | --- |
| **E0** | Kern-Closure leer. | `ids`, `env`, `sys`, `durable-fs`, `simd`, `wire`, `adapt`, `ingest`, `privacy`, `sandbox`, `prompt-guard`, `infer-onnx`, `erasure`, `license`, `audit-export`, `avv-generator` |
| **E1a** | Kern-Closure ⊆ E0, höchstens 4 Mitglieder. | `mvcc`, `crypto`, `text`, `graph`, `rank`, `checkpoint`, `kvcache`, `vault`, `infer-ollama`, `infer-candle` |
| **E1b** | Kern-Closure ⊆ E0 ∪ E1a, mindestens ein E1a-Mitglied, höchstens 4 Mitglieder. Ausnahme `store` mit 5 (D-03). | `store`, `vector` |
| **E2** | Integration. Darf E0, E1 und E2 nutzen, nie E3. | `types`, `ports`, `router`, `agent`, `cognition`, `engine`, `db`, `contextra` |
| **E3** | Anwendungen und Testwerkzeuge. | `mcp`, `py`, `testkit`, `bench` |

Die Ebene beschränkt nur die Kern-Closure. Kanten über Features und dev-Kanten regeln R12 und R13.

## 3 Invarianten

Die zwölf Invarianten gelten für jeden Umbauschritt. Ein Schritt, der eine verletzt, wird nicht gemerged. Die Gates stehen in §7; ihren heutigen Durchsetzungsstand zeigt §7.3.

| ID | Invariante | Prüfverfahren |
| --- | --- | --- |
| INV-1 | **Formatgleichheit.** Kein persistiertes Byte ändert sich. Einzige Ausnahme: die Versionsanhebungen mit Feld `id_width_bits` nach §6.2, nur in den dort benannten Schritten. Jede Ausnahme erzeugt Golden-Dateien für v(alt) und v(neu); der Lesepfad für v(alt) bleibt. | G-F1 |
| INV-2 | **Pfadgleichheit.** Öffentliche Importpfade bleiben mit `#[deprecated(since, note)]`-Re-Export gültig. Entfernung nur in S-70. | G-A1 |
| INV-3 | **Ein Schritt, ein Commit-Paar.** Commit A verschiebt oder benennt um, Commit B baut um. Beide sind einzeln revertierbar. Format: `S-xx A: …` und `S-xx B: …` (D-08). | Commit-Lint in `.githooks/commit-msg` |
| INV-4 | **Der Build bleibt grün.** `cargo check --workspace --all-targets --locked` nach jedem Commit, zusätzlich `cargo check -p contextra-vault --locked`, solange `vault` nicht Workspace-Mitglied ist (R14). | G-L3 |
| INV-5 | **Determinismus.** Gleiche Eingabe ergibt gleiche Rangfolge, Tie-Break nach ID aufsteigend. Gilt bei `SeededRng` und `ManualClock`. Bei Nebenläufigkeit gilt Gleichheit der Rangfolge, nicht der Reihenfolge von Seiteneffekten (D-10). | G-D1, `tools/determinism_guard.py` |
| INV-6 | **Durability-Pfade bleiben unberührt** (WAL, Manifest, Flush, Recovery), sofern der Schritt sie nicht ausdrücklich nennt. | Crash- und Recovery-Suite, G-F1 |
| INV-7 | **Keine Compliance-Aussagen** in Code und Doku („DSGVO-konform“, „revisionssicher“, „garantiert gelöscht“). | G-DOC1 |
| INV-8 | **Kein Rückwärtspfeil.** Normale und dev-Kanten zeigen nur nach unten: E3 → E2 → E1b → E1a → E0. Feature-Kanten folgen R13. | G-L1 |
| INV-9 | **Reproduzierbarkeit.** Toolchain `1.89.0` (`rust-toolchain.toml`) \[V\], `--locked`, Werkzeugliste aus Anhang B. | CI |
| INV-10 | **Keine stille Gate-Abschwächung.** Änderungen an `governance/gates.toml`, `ratchet.toml` und `protected-paths.toml` nur mit ADR und Label `protected-path`. Ratchet-Werte dürfen nur sinken; `not_run_baseline` darf nur schrumpfen (D-04). | `gate-weakening`, `ratchet` |
| INV-11 | **Unsafe-Inseln.** `unsafe` nur in `simd`, `sys`, `wire`, jeweils mit lückenlosem `// SAFETY:`-Nachweis. Alle anderen Crates führen `#![forbid(unsafe_code)]`. Am HEAD stehen außerhalb der Inseln `unsafe`-Blöcke nur in zwei Test-Binaries (`text/tests/alloc_profiler.rs`, `graph/tests/ppr_alloc_test.rs`, je ein `GlobalAlloc`-Zähler) \[V\]; sie sind bis zur Entscheidung D-21 die einzigen benannten Ausnahmen. | `tests/unsafe_islands.rs`, `unsafe-inventory` |
| INV-12 | **Keine Panics in neuem Nicht-Test-Code, in allen Crates.** Kein `unwrap()`, `expect()`, `panic!()`, `todo!()`. Das erweitert R8 aus v3, das nur `ids` und `env` erfasste und damit hinter `AGENTS.md` zurückblieb. | `unwrap-gate` (`.github/workflows/unwrap-gate.yml` \[V\]) |

## 4 Zielarchitektur

Jeder Baustein hat genau eine Ebene, einen eigenen Fehlertyp, einen geschlossenen Satz von Features und einen Default, der nur das Wesensnotwendige enthält. Die Tabelle in 4.1 ersetzt die Tabelle aus v3; die Regeln in 4.2 sind für alle Crates verbindlich.

### 4.1 Crate-Tabelle (normativ)

„Kern direkt → Closure“ nennt die direkten Kernabhängigkeiten und die berechnete transitive Menge (§2.1). Für E2 und E3 wird die Closure nicht budgetiert und von G-L4 berechnet. Ein Feature `workspace` zieht **genau** die Crates, die in der Spalte „Features“ hinter dem Pfeil stehen; steht dort kein Pfeil, zieht es `types` und `ports`. Tier-Werte stehen in `capabilities.toml` (einzige Quelle).

| Crate | Ebene | Kern direkt → Closure | Features (Ziel) | Default (Ziel) | Fehler |
| --- | --- | --- | --- | --- | --- |
| `ids` | E0 | — | `docid-128`, `std` | `std` | `IdError` |
| `env` | E0 | — | `test-utils`, `std` | `std` | — |
| `sys` | E0 | — | — | — | `SysError` |
| `durable-fs` | E0 | — | `crc`, `std` | `std`, `crc` | besteht |
| `simd` | E0 | — | `workspace`→types | `std` | `SimdError` |
| `wire` | E0 | — | — | `std` | besteht |
| `adapt` | E0 | — | `bandit-routing`, `exp-flow-thompson`, `exp-rie-greedy` | `std` | besteht |
| `ingest` | E0 | — | `workspace`→ids, types; `fuzz`; `fmt-*` | `fmt-text` | besteht |
| `privacy` | E0 | — | — | `std` | besteht |
| `sandbox` | E0 | — | — | — | besteht |
| `prompt-guard` | E0 | — | `file-audit` | `std` | `GuardError` |
| `infer-onnx` | E0 | — | `onnx`, `download`, `test-utils`, `workspace`→ports, types | — | besteht |
| `erasure` | E0 | — | `signing`, `aead`, `async`, `conformance` | `signing`, `aead` | besteht |
| `license` | E0 | — | `workspace`→types, ports | `std` | besteht |
| `audit-export` | E0 | — | `workspace`→types; `test-utils` | `std` | besteht |
| `avv-generator` | E0 | — | — | `std` | besteht |
| `mvcc` | E1a | ids, env → 2 | `workspace`; `docid-128` | — | `MvccError` |
| `crypto` | E1a | ids, env → 2 | `workspace`; `legacy-erasure`; `cloud-egress-guard`; `kv-encryption` (bis S-38) | — | `CryptoError` |
| `text` | E1a | ids, env → 2 | `workspace` | `std` | `TextError` |
| `graph` | E1a | ids, env → 2 | `workspace`; `exp-hyperedge` | — | `GraphError` |
| `rank` | E1a | ids, env → 2 | `workspace`; `exp-dibud`, `exp-resonance` | `std` | `RankError` |
| `checkpoint` | E1a | ids, env → 2 | `workspace` | — | `CheckpointError` |
| `kvcache` | E1a | ids, env → 2 | `kv-encryption`→crypto; `kv-shredding`; `exp-kivi`; `workspace` | `std` | `KvCacheError` |
| `vault` | E1a | sys → 1 | `workspace` | — | `VaultError` |
| `infer-ollama` | E1a | env → 1 | `workspace` | — | `OllamaError` |
| `infer-candle` | E1a | env → 1 | `candle`; `kv-bridge`→kvcache; `workspace` | — | `CandleError` |
| `store` | E1b | ids, env, sys, mvcc, crypto → 5 | `workspace` | — | `StoreError` |
| `vector` | E1b | ids, env, mvcc → 3 | `mmap`→sys; `simd`→simd; `exp-*`; `workspace` | `std` | `VectorError` |
| `types` | E2 | ids | — | — | `ContextraError` |
| `ports` | E2 | types, ids, env | — | — | — |
| `router` | E2 | types, ports, adapt | `subprocess-dispatch`→wire | — | besteht |
| `agent` | E2 | types, ports | `routing`→router | — | `AgentError` |
| `cognition` | E2 | types, ports | `exp-hyperedge`→graph | — | `CognitionError` |
| `engine` | E2 | ids, env, types, ports, mvcc, store, vector, text, graph, rank, checkpoint (plus `sys`, `crypto` transitiv) | `adaptive-*`→adapt; `onnx`→infer-onnx; `sandbox`→sandbox; `license`→license; `exp-extraction` | — | `ContextraError` |
| `db` | E2 | engine, types, ports, vault | `cognition`→cognition; `exp-diskann`→vector-diskann (nach S-80) | — | — |
| `contextra` | E2 | db, types, ports | `license`→license; `infer-candle` | `embedded` | — |
| `mcp` | E3 | contextra, types, ports, wire, rank, crypto, privacy, router, prompt-guard | fordert `prompt-guard/file-audit` | — | `McpError` |
| `py` | E3 | types, contextra, wire | `docid-128`→ids | — | — |
| `testkit` | E3 | ports, types, env, plus die Bausteine, deren Tests es hostet (R12) | `testkit` | `testkit` | — |
| `bench` | E3 | store, vector, text, db | — | — | — |

**Anmerkungen**

- `store` ist wegen `mvcc` und `crypto` die einzige Ausnahme vom Budget von 4 Mitgliedern (D-03).
- `engine` hat 9 direkte Abhängigkeiten ohne `ids`/`env` und erfüllt damit die Grenze „≤ 9“ aus `AGENTS.md`. Mit `sys` und `crypto` (über `store`) hat es 13 Closure-Mitglieder \[E, aus dieser Tabelle abgeleitet\].
- `db` braucht `vault` im Kern, weil es den Typ re-exportiert.
- `ids` behält das Feature `std` samt Default (Abwärtskompatibilität). Das Ziel ist kein `no_std` (D-19).
- `erasure`: das Feature `vault` heißt künftig `aead` (D-11). `async` ist nicht mehr Default (D-13).
- `kvcache` bleibt E1a, obwohl `kv-encryption` einen E1a-Baustein (`crypto`) zieht. Solche Kanten regelt R13.

### 4.2 Regeln

| Regel | Inhalt | Prüfverfahren |
| --- | --- | --- |
| **R1 Richtung** | Abhängigkeiten zeigen nach unten (INV-8). | G-L1 |
| **R2 Brücke** | `contextra_types`, `contextra_ports` und `contextra_testkit` kommen im Kern nicht vor, auch nicht in Doc-Kommentaren und `#[cfg(test)]`-Modulen. Tests des Kerns nutzen `env` (`test-utils`) und lokale Fakes. Ausnahme: `tests/` und `compat/` hinter `required-features = ["workspace"]`. | G-L2 |
| **R3 Eigene Fehler** | `thiserror`, `#[non_exhaustive]`. `impl From<LocalError> for ContextraError` nur in `compat/`. | G-E1 |
| **R4 Ein Besitzer je Konzept** | IDs → `ids`. `Clock`, `Rng`, `IdGen`, `MetricsSink` → `env`. `Tombstone*` → `mvcc`. `PprConfig`, `PprAlgorithm`, `TlHfdParams` → `graph`. KV-Verschlüsselung → `kvcache`. Lizenz-Traits → `license`. Erasure-Familie → `erasure`. Ausnahme: kleine geschlossene Wert-Enums (höchstens 8 Unit-Varianten, z. B. `DistanceMetric`) dürfen gespiegelt werden, wenn Namen und Reihenfolge identisch sind und ein Konformitätstest die serde-Namen gleichsetzt. | `symbol-uniqueness` |
| **R5 Experimente** | X1 bis X8 (§4.3). | `feature_audit.py`, G-L1 |
| **R6 Nachweis statt Behauptung** | „Entnehmbar“ gilt nur nach §2.1 und §8.4. | Gates |
| **R7 E0-Seams lokal** | E0-Crates definieren Umwelt-Traits lokal und minimal. Die Verbindung zu `env` oder `ports` liegt nur in `compat/`. | G-L2 |
| **R8** | Aufgehoben, ersetzt durch INV-12. | — |
| **R9 Re-Export-Übergang** | Verlässt ein Typ sein Crate, bleibt ein `#[deprecated]`-Re-Export, außer er würde einen Rückwärtspfeil erzeugen; dann entfällt er ersatzlos, wenn nur Besitzer und Ursprung ihn nutzen. Quellcode darf entfernt werden, wenn sein Inhalt in einem Besitzer-Crate liegt oder per Git-Tag archiviert ist (D-09). | G-A1 |
| **R10 Features** | `default` enthält nie `workspace`, `test-utils`, `*-compat`, `exp-*`, `experimental-*`, `legacy-*`. Erlaubt sind `std` und die geschlossene Liste `crc`, `fmt-text`, `embedded`, `testkit`, `signing`, `aead`. `async`, `file-audit`, `kv-encryption`, `candle`, `onnx` sind nie Default. Experimente heißen `exp-<name>`, das Sammelfeature `experimental-all`. `*-compat` sind Aliase für `workspace` und entfallen in S-70. Ein Feature ohne `cfg(feature = "x")` und ohne Weiterleitung wird entfernt. Nicht-additive Features (`docid-128`) aktivieren nur Anwendungs-Crates; Bausteine leiten sie weiter und setzen sie nie in `default` (D-02). | G-H1, `tools/feature_audit.py` |
| **R11 Fan-in-Ratchet** | Kein neuer Verwender von `types` oder `ports` aus E0 oder E1. Fan-in = Zahl der Crates, die `contextra-types` bzw. `contextra-ports` als normale, auch optionale Abhängigkeit führen; dev-Kanten zählen nicht (R12). `ratchet.toml` führt `types_fan_in` und `ports_fan_in`; die Startwerte werden in S-00 neu gemessen. | `ratchet` |
| **R12 Dev-Abhängigkeiten** | dev-Kanten von E0 und E1 zeigen nur auf E0 und externe Crates. Tests, die `types`, `ports` oder `testkit` brauchen, liegen in `testkit/tests/<crate>/` (normale Kante E3 → E1) oder hinter `required-features = ["workspace"]` (dort sind `types`/`ports` optionale normale Abhängigkeiten). Begründung: Eine dev-Kante auf `testkit` zieht `types` und `ports` in jedes `cargo test --no-default-features` und hebt die Isolation auf (D-12). | G-L1 (normal und dev), G-H1 |
| **R13 Kanten innerhalb einer Ebene** | Eine Kante zwischen zwei Crates derselben Ebene existiert nur als benanntes Feature, nie im Kern, und der Kantengraph ist azyklisch, Feature-Closures eingeschlossen (z. B. `kvcache` → `crypto`, `infer-candle` → `kvcache`). | G-L1 (je Feature einzeln) |
| **R14 Workspace und Publizierbarkeit** | Jedes Crate in `crates/` ist Workspace-Mitglied. `exclude` und `publish = false` sind je Crate nur mit ADR zulässig. Ausnahmen am HEAD `fcd387021` \[V\]: `contextra-vault` steht in `exclude` (`Cargo.toml`, Zeilen 44 bis 46); `publish = false` tragen zwölf Crates (`adapt`, `checkpoint`, `crypto`, `durable-fs`, `kvcache`, `mvcc`, `privacy`, `prompt-guard`, `rank`, `sys`, `text`, `vault`). Kein ADR in `docs/decisions/` nennt `exclude` oder `publish` (F-11). Für `vault` ist die Begründung vor der Änderung zu klären, danach wird es Mitglied (D-14, S-22). Für die übrigen elf gilt: Ein Baustein mit Entnehmbarkeitsanspruch (§8.4) setzt `publish` nicht auf `false`, oder ein ADR begründet es. | G-H1 |

### 4.3 Experimente (X1 bis X8)

- **X1** Experiment-Code hängt nur von Stabilem ab; der Host-Kern kennt keine Experimente.
- **X2** Feature `exp-<name>`, nie in `default`, Sammelfeature `experimental-all`, Teil der CI-Matrix.
- **X3** Die Naht (Trait) liegt im Host-Kern in `ext.rs`, ohne `types`/`ports`.
- **X4** Eigenes Crate ab 2.000 Zeilen, oder mit eigenen externen Abhängigkeiten, oder bei mindestens 2 Hosts.
- **X5** Eine Verhaltens-Baseline liegt im selben Crate (z. B. Recall gegen Brute-Force).
- **X6** Persistierte Experimentdaten tragen eine eigene Magic- und Versionskennung; das Kernformat ändert sich nie.
- **X7** Tier `experimental` in `capabilities.toml`.
- **X8** Ein schreibendes Experiment behält den Lesepfad oder dokumentiert eine Migrationssperre.

## 5 Kerne, Brücken und Bausteinvertrag

Die beiden Mikro-Kerne `ids` und `env` tragen alles, was die Bausteine gemeinsam brauchen; alles andere erreicht einen Baustein nur über seine Brücke. Dieser Abschnitt legt die Verträge fest und gibt jedem neuen Baustein eine Checkliste.

### 5.1 `contextra-ids`

**Zweck:** Identitäts- und Mandantentypen ohne interne Abhängigkeit. **Am HEAD \[V\]:** Abhängigkeiten `blake3`, `serde`, `thiserror`; Feature `docid-128`; `types/{ids,refs,tenant_scope}.rs` sind Re-Exports.

- `DocId` ist `u64` (`repr(transparent)`) oder mit `docid-128` `u128` (`repr(C, align(16))`).
- **Die Breite ist eine Eigenschaft des gesamten Builds und nicht additiv.** Cargo vereinheitlicht Features über den ganzen Abhängigkeitsgraphen: Aktiviert irgendein Crate `docid-128`, gilt es für alle. Deshalb dürfen nur Anwendungs-Crates (§2.1) das Feature aktivieren. Bausteine leiten es nur weiter (`docid-128 = ["contextra-ids/docid-128"]`) und führen es nie im Default (D-02). G-H1 prüft das.
- `ID_WIDTH_BITS: u32` ist 64 oder 128, mit Compile-Time-Prüfung von `size_of` und `align_of` (8/8 bzw. 16/16). Persistierte Köpfe tragen `id_width_bits: u16`. Die Umrechnung ist eine `const`-Prüfung (`u16::try_from`), kein `as`.
- `DocId::from_key(&str) -> Result<Self, IdError>`: BLAKE3-Präfix, die ersten 8 bzw. 16 Bytes, Little Endian. Display-Text und Variante von `IdError` sind bitgleich zu den früheren `InvalidInput`-Texten.
- `types` enthält `impl From<IdError> for ContextraError` (Variante `InvalidInput(String)`).
- **Offen:** Am HEAD `fcd387021` gibt es 88 Aufrufstellen von `::from_key(` außerhalb von `/tests/` und `/benches/` \[V, grobe Zählung; die Baseline vom 10.11. nannte 87\]. Jede wird als *mechanisch* (`?` über `From`) oder *nicht-mechanisch* (Musterabgleich auf `ContextraError::InvalidInput`) klassifiziert. Mehr als 20 nicht-mechanische Stellen lösen das Rückfallverfahren aus §9.5 aus.

**Abnahme:** `ids_tests` grün. Differenztest gegen die fünf verbliebenen `DocId`-Kopien (`erasure`, `graph`, `ingest`, `rank`, `text` \[V\]; `vector` nutzt `ids::DocId` bereits) über 10.000 zufällige Schlüssel plus Randfälle (leer, Unicode, 4 KiB): gleiche `from_key`-Bytes, gleiche serde-Ausgabe (JSON und `bincode`). Golden-Vektoren in 64 **und** 128 Bit. Am HEAD sind die Golden-Vektoren vorhanden (`ids/tests/golden/`); `differential_ids.rs` vergleicht nur gegen `types_legacy` und `vector_legacy` \[V\]. Kopien, die anders rechnen als `ids`, regelt D-20.

### 5.2 `contextra-env`

**Am HEAD \[V\]:** `default = ["std"]`, Feature `test-utils`; `ports/clock.rs` re-exportiert `contextra_env::clock::*`.

```rust
pub trait Clock: Send + Sync {
    fn now_unix_nanos(&self) -> u64;   // 0 vor der Epoche, nie Panic
    fn monotonic_nanos(&self) -> u64;
}
pub fn unix_ms<C: Clock + ?Sized>(c: &C) -> u64 { c.now_unix_nanos() / 1_000_000 }  // abrunden

pub trait Rng: Send + Sync {
    fn next_u64(&self) -> u64;
    fn fill_bytes(&self, dest: &mut [u8]);
    fn next_unit_f64(&self) -> f64;
}
```

- **Kein `'static`-Bound (D-10).** v3 verlangte `'static`; das verbietet geliehene Uhren und Fakes mit Referenzen. `Arc<dyn Clock>` bleibt unverändert nutzbar. Verbraucher, die `'static` brauchen, schreiben `T: Clock + 'static` selbst. Auswirkung auf bestehende Verbraucher \[E\]: wird in S-11 mit `cargo check --workspace` geprüft.
- **Determinismus bei Nebenläufigkeit (D-10).** `Rng` arbeitet mit `&self` und braucht innere Veränderbarkeit. Bei `SeededRng` (SplitMix64) ist die *Menge* der erzeugten Werte reproduzierbar, die *Zuordnung* zu Aufrufern bei parallelem Zugriff nicht. Komponenten, die reproduzierbar sein müssen, nutzen deshalb einen eigenen Generator je Aufgabe: `SeededRng::fork(stream: u64) -> SeededRng` leitet aus Seed und Stream-ID einen unabhängigen Seed ab.
- `SeededRng`: Folge bitgleich zu den Golden-Folgen (Seeds 0, 1, 42, `u64::MAX`, je 1.024 Werte).
- `IdGen`, `SequentialIdGen`, `MetricsSink`, `NoopMetricsSink`, `InMemoryMetricsSink`, `MetricKey` wie in der früheren `ports`-Fassung.
- `test-utils` enthält `ManualClock` und die Konformitätssuite für Clock, Rng und Metrics.

**Abnahme:** `cargo tree -p contextra-env -e features` zeigt `test-utils` nicht im Default. `unix_ms`-Differenztest gegen die `now_unix_ms`-Kopien in `text`, `graph` und `vector/contract.rs` mit `ManualClock` (`legacy_reference_unix_ms` existiert in `env` \[V\]).

### 5.3 Brückenmuster (verbindlich)

**a) `Clock` E0-lokal ⇐ `env`:**

```rust
// crates/contextra-<x>/src/compat/env.rs  (feature = "workspace")
impl<T: contextra_env::Clock + ?Sized> crate::Clock for T {
    fn now_unix_nanos(&self) -> u64 { contextra_env::Clock::now_unix_nanos(self) }
    fn monotonic_nanos(&self) -> u64 { contextra_env::Clock::monotonic_nanos(self) }
}
```

Voraussetzung: Das lokale Trait ist lokal (Orphan-Regel erfüllt), und es gibt keine konkrete `impl crate::Clock for T`, die mit dem Blanket überlappt (E0119). Konkrete lokale Typen stehen hinter `cfg(not(feature = "workspace"))` oder als Newtype.

**b) `Rng` mit `&mut self` ⇐ `env::Rng` (`&self`):** Blanket-Impl lokal-von-`env`. Die Gegenrichtung ist verboten; E1-Kerne nutzen ausschließlich `env::Rng`.

**c) Fehler:** `impl From<VectorError> for contextra_types::ContextraError` nur in `compat/error.rs`; Variantenabbildung nach §6.3.

**d) Port-Implementierung:** `impl contextra_ports::VectorIndex for HnswIndex` nur in `compat/`. Kern-Trait-Methoden enthalten kein `ContextraError`.

**e) Konfliktregel:** Zwei Brücken im selben Crate dürfen nicht dasselbe lokale Trait für denselben Blanket implementieren. Gibt es zwei Quellen (`env` und `ports`), implementiert nur `env` den Blanket; `ports`-Typen erreichen ihn, weil `ports` die `env`-Traits re-exportiert.

### 5.4 Bausteinvertrag

Jedes Crate liefert diese Artefakte. Sie machen aus einem Crate einen Baustein, den ein anderes Projekt ohne Lektüre des Quelltexts einsetzen kann. Am HEAD tragen die geprüften Crates bereits `README.md`, `SPEC.md`, `capabilities.fragment.toml` und `examples/` \[V, Stichprobe `mvcc`, `infer-onnx`\].

| Artefakt | Anforderung |
| --- | --- |
| `Cargo.toml` | `[features]` nach R10; `rust-version`; `publish` nach R14; jede interne Pfad-Abhängigkeit mit `version`. |
| `src/lib.rs` | `#![forbid(unsafe_code)]` außer auf den Inseln (INV-11); Modul-Doku nennt Zweck und Ebene. |
| `src/error.rs` | Eigener Fehlertyp nach R3. |
| `src/compat/` | Nur mit Feature `workspace` (R2). |
| `README.md` | Zweck, Feature-Tabelle und ein Einsatzbeispiel in höchstens 10 Zeilen, das **ohne** weitere `contextra-*`-Crates auskommt. |
| `examples/` | Mindestens ein lauffähiges Beispiel nur mit dem Kern. |
| `SPEC.md` | Binnenstruktur und Status; widerspricht dieser Spezifikation nicht (§1.2). |
| `capabilities.fragment.toml` | Tier, Ebene, Unterebene (`E1a`/`E1b`). |
| Tests | Kern-Tests ohne Brücke; Brücken-Tests hinter `required-features = ["workspace"]`; bei Persistenz ein Golden-Korpus (§6.1). |
| Isolation | `tools/isolation_proof.py` ist grün, bevor das Crate als entnehmbar gilt (§8.4). |

## 6 Persistenz, Fehler und Verhalten

Der Umbau darf keine gespeicherten Daten unlesbar machen und kein beobachtbares Verhalten still ändern. Dafür gibt es drei Verträge: einen Golden-Korpus je Format, eine feste Regel für die ID-Breite und Differenztests für jede abgelöste Kopie.

### 6.1 Golden-Format-Korpus

**Ort und Form:** `tests/golden/<format>/` im Wurzelverzeichnis mit `MANIFEST.sha256`. Crate-lokale Korpora (z. B. `mvcc/tests/golden/` \[V\]) bleiben gültig und werden im Wurzel-Manifest referenziert. Je Eintrag gibt es einen Erzeugertest (`#[ignore]`, aufgerufen über `tools/golden_regen.sh`, das heute fehlt \[V\]) und einen Prüftest, der immer läuft. Bei Abweichung bricht der Schritt ab. Eine absichtliche Formatänderung ist nie Teil eines Umbauschritts.

| Format | Kennung am HEAD | Besitzer | Schritt |
| --- | --- | --- | --- |
| Manifest | `MFMN`, `CURRENT_MANIFEST_VERSION: u8 = 1` \[V\] | `store` | S-37 |
| SSTable | `MFSX`, Legacy `MFST` \[V\] | `store` | S-37 |
| WAL mit HMAC-Kette | `store/wal/*`, `wal_crypto` \[Ü\] | `store`, `crypto` | S-31, S-37 |
| Rollback-Intent | `MFRLBK\0\0` \[V\] | `store` | S-37 |
| HNSW | `0x484E5357`, `HNSW_VERSION: u16 = 2` \[V\] | `vector` | S-36 |
| DiskANN | `DANN`, `DANF`, `PWAL`, `TWAL` \[V\] | `vector`, später `vector-diskann` | S-36, S-80 |
| KDF-Header | `MFKD` \[V\] | `crypto` | S-31 |
| KV-Segment | Formatversion 2 \[Ü\]; `CURRENT_KV_KEY_DERIVATION_VERSION: u8 = 1` \[V\] | `crypto`, dann `kvcache` | S-38 |
| Erasure-Formate (**neu in v4**) | `CTXFILE`, `CTRALEDG`, `CTXSCOM`, `CTXSLOK` \[V\] | `erasure` | S-24 |
| Durable-Journal (**neu in v4**) | `JOURNAL_MAGIC` = `0x4354584A` \[V\] | `durable-fs` | S-02 |
| Text-Postings, Graph (bincode), Checkpoint Meta und Manifest | je Crate zu erheben | `text`, `graph`, `checkpoint` | S-32, S-34, S-35 |
| MVCC | `mvcc/tests/golden_corpus.rs` \[V\] | `mvcc` | S-30B |

v3 führte die Formate von `erasure` und `durable-fs` nicht auf; beide tragen Magic-Kennungen \[V\] und gehören in den Korpus. \[Ü\] = aus v3 übernommen, am HEAD nicht neu geprüft.

### 6.2 ID-Breite

Die Breite ist eine Compile-Zeit-Eigenschaft des gesamten Builds (§5.1). **Welche Formate ein Breitenfeld brauchen, entscheidet ein Test, nicht eine Liste:** S-00 erzeugt den Korpus eines Formats einmal mit 64 und einmal mit 128 Bit. Sind die Bytes gleich, ist das Format breitenunabhängig und braucht kein Feld. Unterscheiden sie sich, trägt das Format `id_width_bits: u16` im Kopf und prüft es beim Öffnen. `checkpoint` führt das Feld bereits (`u16`, serde-Default für Legacy-Dateien) \[V\].

| Build-Breite | Datei | Verhalten |
| --- | --- | --- |
| 64 | 64 | Öffnen |
| 128 | 128 | Öffnen |
| 64 | 128 | Ablehnen: `IdWidthMismatch { stored: 128, expected: 64 }` |
| 128 | 64 | Ablehnen: `IdWidthMismatch { stored: 64, expected: 128 }` |
| beliebig | Formatversion ohne Feld | Öffnen nur mit `OpenOptions::assume_legacy_id_width(bits)`, sonst `IdWidthUnknown` |

Die Feldnamen `stored` und `expected` folgen `checkpoint` \[V\]. **Prüfregel:** Nach einem abgelehnten Öffnen ist jede betroffene Datei byte-identisch zum Zustand davor (Hash-Vergleich im Test).

**Versionsanhebungen** sind die einzige INV-1-Ausnahme. Vorgesehen: `store` Manifest 1 → 2 (S-37), `vector` `HNSW_VERSION` 2 → 3 (S-36), `text` und `graph` nach der Erhebung ihrer Köpfe (S-32, S-34). Alle anderen Formate nur, wenn der Breitentest es verlangt. Reihenfolge je Format: erst der Leser (v alt und v neu), dann der Schreiber, mit Rückwärts-Lesetest über Golden-Dateien v alt. Die Versionsfelder haben verschiedene Typen (`u8` im Manifest, `u16` bei HNSW \[V\]); sie werden nicht vereinheitlicht, weil das ein Formatbruch wäre.

**Nicht-Ziel (D-16):** Eine In-Place-Umstellung eines Bestands zwischen 64 und 128 Bit gibt es nicht. Wer wechseln muss, baut den Bestand aus den Quelldokumenten neu auf oder schreibt dafür ein ADR.

**CI:** `cargo hack check --workspace --features docid-128` zusätzlich zum Default. E0-Crates ohne Abhängigkeit auf `ids` (`ingest`, `erasure`) erhalten kein `docid-128`; die Umwandlung von `ids::DocId` ist dort `TryFrom` und schlägt bei Überlauf fehl, nie `as`.

### 6.3 Fehlerabbildung

Jeder Baustein hat eine Tabelle `docs/decoupling/errors/<crate>.md` mit lokaler Variante, `ContextraError`-Variante und Display-String. Am HEAD liegen 15 Dateien vor \[V\]: `agent`, `checkpoint`, `cognition`, `crypto`, `graph`, `infer`, `kvcache`, `mvcc`, `prompt-guard`, `rank`, `router`, `store`, `text`, `vault`, `vector`. Es fehlen `ids` und die E0-Crates mit „besteht“, soweit sie eine `compat/`-Brücke erhalten. Anforderungen:

1. Jede heute genutzte `ContextraError`-Variante des Kerns hat genau eine lokale Entsprechung (Erhebung: `grep -rn 'ContextraError::[A-Z]' src`).
2. `Display` des konvertierten Fehlers ist bitgleich zum alten Fehler, geprüft mit einem tabellengetriebenen Test je Variante.
3. Keine Sammelvariante `Other(String)`, die Kontext verwirft. Ausnahme: I/O mit `#[source]`.
4. Das Gate `error-domain` (heute in `not_run_baseline` \[V\]) wird in S-01 definiert und in S-12a aktiviert. Ausgangsbasis ist `tools/error_topology_mapper.py`.

### 6.4 Differenztests

Jede Ablösung einer Kopie durch ihren Besitzer wird zuerst durch einen Differenztest abgesichert. Er kompiliert alte und neue Implementierung gleichzeitig (Commit A) und vergleicht gleiche Eingaben. Nach grünem Test folgt Commit B; der Test bleibt als Regressionstest, solange beide Seiten existieren.

| Bereich | Eingabe | Toleranz |
| --- | --- | --- |
| IDs | §5.1 | bitgleich |
| Clock, Rng, Millisekunden | `ManualClock`, `SeededRng` | bitgleich |
| Distanz (`types::DistanceMetric::compute`, `vector::distance`, `simd::dispatch`) | 10.000 Vektorpaare, Dimension 4, 17, 128, 768; normale, Null- und Denormal-Werte | relativ ≤ 1e-5 (f32), bitgleich bei gleicher Skalarfunktion; NaN- und Inf-Verhalten gleich |
| Regler (`decay`, `homeostat`, `pid`) | gleiche Eingabefolge mit `ManualClock` | bitgleich (f64) |
| Chunker (`engine::chunker`, `ingest::chunking::prose`) | Korpus | identische Chunk-Folge, Reihenfolge und Breadcrumbs |
| Tombstone (`types`, `mvcc`) | `mvcc/tests/tombstone_diff.rs` \[V\] | bitgleich |
| Temporalfilter (`engine::temporal_filter`, `rank::temporal`) | Filterausgaben | bitgleich |

## 7 Gates

Ein Gate ist ein Werkzeug mit festem Aufruf und festem Exit-Code. Nur ein Gate, das in CI läuft, gilt als Prüfverfahren im Sinne von §1.3. Am HEAD laufen nur wenige der benötigten Gates in CI; §7.3 listet die Lücken.

### 7.1 Konventionen

- **Exit-Codes:** 0 = grün, 1 = Verstoß, 2 = Werkzeug- oder Umgebungsfehler. Exit 2 gilt nie als grün. Fehlt `cargo`, endet das Gate mit 2, nicht mit 0.
- **Ein Aufruf, zwei Orte:** Jedes Gate läuft lokal über das `justfile` und in CI mit demselben Befehl. Der kanonische Aufruf je Gate wird in S-01 im `justfile` festgehalten; die Parameter der Skripte stehen dort, nicht in dieser Spezifikation.
- **Negativtest:** Jedes Gate hat einen Test, der einen absichtlichen Verstoß einbaut und Exit 1 erwartet, nach dem Vorbild von `tests/test_gate_layering_negative.py` \[V\].
- **Kein stilles Auslassen:** Ein Gate in `governance/ratchet.toml › not_run_baseline` zählt als nicht vorhanden (D-04). Die Liste darf nur schrumpfen; für jedes Gate gilt eine Frist (S-01).
- **Kein nicht-blockierender Lauf (D-23):** Ein Gate-Schritt in CI enthält weder `|| true` noch `continue-on-error`. Ein Gate, das nicht blockiert, gilt als nicht vorhanden.

### 7.2 Gate-Definitionen

Die Spalte „Verstoß, wenn“ ist das Sollverhalten. Ob ein vorhandenes Skript es heute vollständig leistet, prüft S-00.

| Gate | Prüft | Werkzeug | Verstoß, wenn |
| --- | --- | --- | --- |
| **G-L1** Richtung und Zyklen | INV-8, R1, R12, R13 | `tools/gate_layering.py` auf Basis von `cargo metadata` | eine normale, dev- oder Feature-Kante aufwärts zeigt, oder der Kantengraph einen Zyklus enthält (je Feature einzeln ausgewertet) |
| **G-L2** Brücke null | R2, R7 | `tools/gate_bridge_null.py` | im Kern (ohne `compat/` und ohne `cfg(feature = "workspace")`) `contextra_types`, `contextra_ports` oder `contextra_testkit` vorkommt, auch in Kommentaren und `#[cfg(test)]`. Anders als `tools/count_bridge_refs.py` zählt es gegatete Treffer nicht. |
| **G-L3** Build und Features | INV-4, R10 | `cargo check` über Workspace und je Crate mit `--no-default-features`; `cargo hack check --each-feature --no-dev-deps`; Lauf mit `--features docid-128`; `cargo check -p contextra-vault` | eine Kombination nicht baut |
| **G-L4** Isolation | Entnehmbarkeit | `tools/isolation_proof.py` | ein Crate mit seiner Kern-Closure in einem leeren Workspace nicht baut oder seine Kern-Tests (`--no-default-features`) nicht kompilieren |
| **G-H1** Hygiene | R10, R11, R12, R14, D-02 | `tools/feature_audit.py` (entrauscht: ohne `default` und Weiterleitungs-Features), `ratchet` | ein verbotenes Feature im Default steht, eine dev-Kante von E0/E1 auf `testkit` zeigt, `exclude` oder `publish = false` ohne ADR gesetzt ist, der Fan-in den Ratchet übersteigt, oder ein Baustein `docid-128` selbst aktiviert |
| **G-F1** Formate | INV-1, INV-6 | Prüftests des Golden-Korpus, `sha256sum -c` über `tests/golden/**/MANIFEST.sha256`; in `gates.toml` als `format-corpus` geführt \[V Name\] | ein Hash abweicht oder ein Format keinen Eintrag hat |
| **G-F2** Wire-Schema | Schema-Stabilität | `sha256sum -c` gegen `crates/contextra-wire/schemas/contextra.fbs.sha256` | sich `contextra.fbs` ändert, ohne dass die Hash-Datei im selben Commit mitwandert |
| **G-D1** Determinismus und Differenz | INV-5, §6.4 | `tools/determinism_guard.py`; Differenztests mit Namensmuster `diff_*` | eine Rangfolge oder ein Differenztest abweicht |
| **G-E1** Fehler | R3, §6.3 | `tools/error_topology_mapper.py` (Prüfmodus in S-01) und tabellengetriebene Display-Tests | eine Variante keine Entsprechung hat oder der Display-Text abweicht |
| **G-C1** Nebenläufigkeit und Mutation | INV-6 | Loom (`loom-gate.yml`), TSan (`tsan.yml`), Miri, Mutation (`mutation-testing.yml`, Baseline `mutants-guarantees.toml`) | ein Lauf fehlschlägt oder der Mutation-Score unter die Baseline fällt |
| **G-A1** Öffentliche API | INV-2, R9 | `cargo public-api` (`gates.toml › public-api`) | ein öffentlicher Pfad ohne `#[deprecated]`-Re-Export verschwindet oder sich ändert. Der Nightly muss mindestens `rust-version` 1.89 entsprechen; das Pinning regelt ein ADR. |
| **G-DOC1** Doku | INV-7, §1.3 | `tools/doc_lint.py` | eine Compliance-Aussage steht, ein Verweis auf eine nicht existierende Datei oder §-Überschrift läuft ins Leere, oder ein Docstring `capabilities.toml` widerspricht |

**Querschnittsgates:** `unwrap-gate` (INV-12), `unsafe-inventory` und `tests/unsafe_islands.rs` (INV-11), `symbol-uniqueness` (R4), `ratchet`, `gate-weakening` und `protected-paths` (INV-10), `error-domain` (= G-E1).

Zu G-C1: Der Pfadfilter von `loom-gate.yml` deckt `store`, `crypto`, `vector`, `mvcc`, `kvcache`, `checkpoint`, `ids` und `env` ab \[V\]. Die Mutation-Stufen „G2/G3“ aus v3 bleiben nur erhalten, wenn `mutants-guarantees.toml` sie definiert \[E, Datei nicht gelesen\].

### 7.3 Durchsetzungsstand (HEAD `fcd387021`, 11.11.2026)

Diese Tabelle altert schnell. Sie wird ab S-00 aus `governance/gates.toml` und den Workflows generiert (§8.3); der handgepflegte Stand dient nur als Ausgangspunkt. Alle Angaben \[V\] (Dateien gelesen, nichts ausgeführt).

| Gate | Werkzeug am HEAD | In CI | Lücke |
| --- | --- | --- | --- |
| G-L1 | `gate_layering.py` | Job `layering-gate` in `rust-ci.yml`, Aufruf `--all \|\| true` (Zeile 43): **blockiert nicht** | `\|\| true` entfernen. Das Skript wertet `-e normal` aus und prüft dev-Kanten nur gegen `testkit`; Feature-Kanten, Zyklen und Default-Features der Abhängigkeiten fehlen. `tests/layering.rs` delegiert an das Skript. `gate_g1_layering.py` (14 Zeilen) ist nur ein Alias auf `gate_dependencies.py`, das diese Spezifikation nicht kennt: beides klären, den Alias entfernen. |
| G-L2 | `gate_bridge_null.py` | nirgends | Das Skript ignoriert Kommentare; §7.2 verlangt, sie zu zählen. Es meldet 185 Treffer (`graph` 136), die strenge Zählung ergibt 189 (`graph` 140). CI-Job anlegen. |
| G-L3 | `cargo-hack` (`cargo-hack-feature-check`, `--each-feature --workspace --no-dev-deps`); `compile-check` | beides in `rust-ci.yml`; `compile-check` ruft `cargo check --workspace --all-targets --locked \|\| true` (Zeile 28): **INV-4 wird nicht durchgesetzt** | `\|\| true` entfernen; `cargo check -p contextra-vault` fehlt in CI. |
| G-L4 | `isolation_proof.py` | nur `justfile` | CI-Job anlegen. |
| G-H1 | `feature_audit.py` | nur `justfile`; `ratchet` steht in `not_run_baseline`; `types_fan_in` und `ports_fan_in` fehlen in `ratchet.toml` | entrauschen, CI-Job, Ratchet-Werte messen. |
| G-F1 | Gate `format-corpus`; `tests/golden/` und `tools/golden_regen.sh` fehlen. Crate-lokal vorhanden: `ids/tests/golden/` (mit `regen.sh`), `mvcc/tests/golden/` | nirgends | S-02. |
| G-F2 | `contextra.fbs.sha256` vorhanden | nirgends | CI-Job anlegen. |
| G-D1 | `determinism_guard.py`; `determinism` steht in `not_run_baseline`; kein Test mit Namensmuster `diff_*` im Repository | nirgends | CI-Job; Differenztests nach dem Namensmuster benennen oder das Muster in §7.2 an die Praxis anpassen (`differential_*`). |
| G-E1 | `error_topology_mapper.py`; `error-domain` steht in `not_run_baseline` | nirgends | S-01, S-12a. |
| G-C1 | `loom-gate.yml`, `tsan.yml`, `mutation-testing.yml` | ja; `loom-gate.yml` und `mutation-testing.yml` enthalten je ein `\|\| true`, `mutation-testing.yml` zusätzlich `continue-on-error` | Wirkung auf den Exit-Code prüfen \[E\]. |
| G-A1 | Eintrag `public-api` in `gates.toml` | in keinem Workflow | Job und Toolchain-ADR. |
| G-DOC1 | `doc_lint.py`; `doc-truth` steht in `not_run_baseline` | nirgends | CI-Job. |
| INV-3 Commit-Lint | `.githooks/` enthält nur `pre-commit` und `pre-push` | — | `commit-msg` anlegen. |
| INV-4 `docid-128` | Job `docid-128-check` (workspace-weit mit `contextra-types/docid-128`, Tests in beiden Breiten) ohne `\|\| true`; Job `check-docid-128` mit `\|\| true` | ja | Der Befehl `cargo check -p contextra --features docid-128` im Job `docid-128-check` verweist auf ein Feature, das `contextra` nicht hat (Features: `embedded`, `candle`); der Schritt kann nicht grün laufen \[E, nicht ausgeführt\] (F-02). |

Quelltext verweist teils auf „SPEC v4.0 §8.1“ (z. B. `tests/layering.rs`, Kopfzeilen in `ingest`, `license`, `adapt`). In dieser Spezifikation stehen die Gates in §7.2; diese Verweise werden in S-03 umgestellt.

## 8 Score, Berichte und Entnehmbarkeit

Der Modularitäts-Score M zeigt je Crate, wie weit es vom Zielzustand entfernt ist. Er ersetzt kein Gate: Ob ein Crate entnehmbar ist, entscheiden allein die Gates (R6).

### 8.1 Definition des Scores M

`tools/modularity_score.py` existiert \[V\] und rechnet nach der Formel unten, die dieser Abschnitt normativ übernimmt. M startet bei 100 und wird auf 0 bis 100 begrenzt.

| Abzug | Bedingung | Punkte |
| --- | --- | --- |
| A Kernabhängigkeiten | je überzählige direkte Kernabhängigkeit gegenüber §4.1 | 12, höchstens 35 |
| B Brückenreferenzen | n ungegatete Referenzen auf `contextra_types`, `contextra_ports`, `contextra_testkit` im Kern | `round(15 · log10(1 + n))`, höchstens 40 |
| C G-L2 verfehlt | E0 oder E1 mit n > 0 (kommt zu B hinzu; ein einziger Treffer kostet mindestens 15 + B) | 15 |
| D Brücke fehlt | E0 oder E1 mit `types`/`ports`-Bezug ohne `compat/` oder ohne Feature `workspace`, je Fehlstelle | 10 |
| E Fehlertyp fehlt | E0 oder E1 ohne eigenen Fehlertyp (R3) | 5 |
| F Default (neu ab S-12a) | je Feature im `default`, das R10 verbietet | 5, höchstens 15 |
| G dev-Kante (neu ab S-12b) | je dev-Kante von E0/E1 auf `types`, `ports` oder `testkit` (R12) | 5, höchstens 15 |

**Zielwert:** M = 100 für jedes E0- und E1-Crate. M < 100 listet die einzelnen Abzüge im Bericht. Der Ratchet verlangt, dass kein Crate nach einem Schritt einen niedrigeren Score hat als vorher.

**Schwächen der heutigen Implementierung \[V\]:**

- Die Zielwerte (`TARGET_CORE_DEPS`) sind im Skript hartkodiert und doppeln die Tabelle §4.1. Ändert sich eine der beiden, widersprechen sie sich unbemerkt. Festlegung D-17: Ebene, Unterebene und Ziel-Kernabhängigkeiten stehen je Crate in `capabilities.toml` (Felder `level`, `sublevel`, `core_deps`). `modularity_score.py`, `gate_layering.py` und `doc_lint.py` lesen sie von dort; das Gate `capabilities-consistency` prüft §4.1 dagegen.
- Der Abzug A vergleicht direkte Abhängigkeiten, die Ebenen in §2.2 sind aber über die Closure definiert. Die Closure-Größe wird in S-00 zusätzlich ausgegeben und gegen das Budget (4, bei `store` 5) geprüft.
- Die Skript-Kopfzeile verweist auf „SPEC §3.1“ und „§8.6“. Die Tabelle steht jetzt in §4.1, der Score in §8.1; die Verweise werden in S-03 angepasst.

### 8.2 Messung

Der Score wird nach jedem Schritt neu erzeugt (Definition of Done, §9.1) und im Bericht nach 8.3 abgelegt. Die Ausgabe nennt je Crate Ebene, Score, Brückenzahl und die Einzelabzüge.

### 8.3 Generierte Berichte statt Ist-Stand in der Spezifikation

Messwerte gehören nicht in dieses Dokument (D-15). `tools/baseline/run_baseline.py` \[V\] erzeugt sie unter `docs/decoupling/baseline/` \[V: `DIFF_BASELINE.md`, `layering_violations.md` und ein Ordner je Crate\]. Der Bericht enthält mindestens:

- HEAD-Hash und Datum in der Kopfzeile;
- interne Kanten (normal, dev, optional) und Fan-in von `types` und `ports` nach der Definition in R11;
- ungegatete Referenzen je Crate nach G-L2;
- Score M je Crate mit Einzelabzügen;
- Gate-Status nach §7.3 und Schrittstatus nach §9.2;
- Ergebnis von `cargo check --no-default-features` je Crate (bisher nirgends belegt).

Diese Spezifikation verweist auf den Bericht und nennt Zahlen nur mit Hash und Datum.

### 8.4 Entnehmbarkeitsnachweis

Ein Crate gilt als entnehmbar, wenn der Bericht `crates/<crate>/docs/decoupling/REPORT.md` \[V: vorhanden z. B. bei `mvcc`, `infer-onnx`\] diese Punkte mit HEAD-Hash und Datum belegt:

1. G-L2, G-L1, G-L3, G-L4 und G-H1 sind in dieser Reihenfolge grün (Exit 0).
2. M = 100 (E0/E1); jede Abweichung ist begründet.
3. Das Einsatzbeispiel aus `README.md` kompiliert ohne weitere `contextra-*`-Crates (Doctest oder `examples/`).
4. Bei Persistenz: Golden-Korpus und Breitentest (§6.1, §6.2) sind grün.
5. Der Bausteinvertrag (§5.4) ist vollständig.
6. **SOLL:** `cargo package -p <crate> --locked` gelingt als Probe, dass das Crate keine versteckte Pfad-Abhängigkeit trägt \[E, nicht geprüft\].

Der Status steht als Feld `extractable` mit `verified_at` (HEAD-Hash) in `capabilities.fragment.toml`. Nur der Abnahmeschritt setzt ihn; CI entzieht ihn, sobald eines der Gates rot wird.

## 9 Umsetzungsplan

Der Plan umfasst 33 Schritte in einer festen Reihenfolge. Am HEAD `fcd387021` ist **kein Schritt vollständig erledigt**; in §9.2 steht je Schritt, was bereits steht und was bleibt. Der laufende Schrittstatus gehört weiterhin in den generierten Bericht (§8.3, D-15); die Spalten „Erledigt“ und „Rest“ sind die Ausgangslage dafür und sind mit S-00 neu zu erheben.

### 9.1 Definition of Done

Ein Schritt ist abgeschlossen, wenn alle Punkte erfüllt sind:

1. Commit-Paar nach INV-3, Nachrichten `S-xx A: …` und `S-xx B: …`.
2. `cargo check --workspace --all-targets --locked` ist grün (INV-4).
3. Die in der Zeile genannten Gates sind grün. **GL** = G-L1 bis G-L4, **GH** = G-H1, **GF** = G-F1 und G-F2, **GD** = G-D1, **GE** = G-E1, **GC** = G-C1.
4. Score und Brückenzählung sind neu erzeugt (§8.2); kein Crate hat einen niedrigeren Score als vorher.
5. Die crate-lokale `SPEC.md`/`PLAN.md` trägt den neuen Status; der Bericht nach §8.3 ist regeneriert.
6. Keine neue Verletzung von R10 bis R14.
7. Der Revert-Commit baut (`git revert` auf einem Wegwerf-Zweig, danach `cargo check`).
8. Alle Verweise in geänderten Dokumenten lösen auf (G-DOC1).

### 9.2 Schritte (Restarbeit)

„Voraussetzung“ nennt nur harte Abhängigkeiten. Alle Angaben \[V\] am HEAD `fcd387021`, sofern nicht anders markiert; nichts wurde gebaut (§1.1).

| Schritt | Erledigt | Rest | Voraussetzung | Nachweis |
| --- | --- | --- | --- | --- |
| **S-00** | Baseline-Bericht von v4-Zeit vorhanden (`docs/decoupling/baseline/`, aber HEAD `9305c5d04`, 10.11.). `from_key`-Inventar (87) liegt dort. | Bericht neu erzeugen; `cargo check --no-default-features` je Crate nachholen (hier nicht möglich); 88 `from_key`-Stellen klassifizieren; Breitentest je Format (§6.2); `gate_bridge_null.py` auf Kommentare erweitern; `gate_layering.py` auf dev-Kanten, Feature-Kanten, Zyklen und Default-Features der Abhängigkeiten (F-07) erweitern und Exit-Code 2 erzwingen; `modularity_score.py` auf §8.1 und D-17; die Closure aus Anhang B durch `cargo tree` bestätigen. | — | Bericht mit HEAD-Hash; „baut ohne Default-Features“ je Crate belegt oder als Befund erfasst |
| **S-01** | `gate_layering.py`, `test_gate_layering_negative.py` (in CI), `cargo-hack`-Job, `docid-128`-Matrix mit Tests in beiden Breiten. | Zuerst: `\|\| true` aus `rust-ci.yml` Zeilen 28, 43, 57 und die Wirkung der `continue-on-error`-Stellen prüfen (D-23, F-01); `cargo check -p contextra --features docid-128` reparieren (F-02). `xtask`-Globs in `protected-paths.toml` (Zeilen 10, 78) und `xtask`-Kommentare im `justfile` (Zeilen 24 bis 31) entfernen. Für jedes der 38 Gates in `not_run_baseline` einen CI-Job oder ein ADR mit Frist (D-04); zuerst `ratchet`, `gate-weakening`, `protected-paths`, `symbol-uniqueness`, `error-domain`, `feature-matrix`, `unsafe-inventory`, `doc-truth`. CI-Jobs für G-L2, G-L4, G-H1, G-F2, G-D1, G-E1, G-A1, G-DOC1. Hook `commit-msg`. Ratchet für `types_fan_in` und `ports_fan_in`. `cargo check -p contextra-vault`. `gate_g1_layering.py` und `gate_dependencies.py` zusammenführen. | S-00 | G-L1 scheitert bei absichtlichem Verstoß und blockiert in CI; am Baseline-Stand grün; für jedes Gate ein Negativtest |
| **S-02** | Crate-lokale Korpora: `ids/tests/golden/` (Hash-Manifest, `regen.sh`), `mvcc/tests/golden/`. | Wurzel-Korpus `tests/golden/` mit `MANIFEST.sha256`, `tools/golden_regen.sh`, Differenz-Harness (§6.1, §6.4); `mvcc` und `ids` einhängen; Formate von `erasure` und `durable-fs` ergänzen. | S-00 | GF am unveränderten Stand grün |
| **S-03** | `capabilities.toml` Schema v3 trägt `level` (numerisch) für alle Crates. | `AGENTS.md`: Zeilen 4, 73 („v3“) und 84 (`WP-xx`) berichtigen, Archiv-Liste (§2.4) gegen den Code prüfen (D-07 bis D-09); `docs/handoff/OPEN_ISSUES.md` und `STATUS.md` fehlen: anlegen oder Verweise entfernen; Quelltext-Verweise auf „SPEC v4.0“ umstellen; Verwenderliste in `types/lib.rs` durch generierten Verweis ersetzen; Docstrings (`privacy`, `cognition`, `kvcache`, `avv-generator`, `sys/lib.rs:19`, `vector/types.rs:3–6` \[Ü\]); Item-Diff `AvvError`/`AvvGeneratorError`; `capabilities.toml` um `sublevel` und `core_deps` erweitern (D-17; beide Felder fehlen \[V\]); `version` an 12 direkten Pfad-Abhängigkeiten und an den 40 internen `[workspace.dependencies]`-Einträgen ergänzen (F-12). | S-01 | G-DOC1, `capabilities-consistency` grün; `cargo package`-Probe (§8.4 Punkt 6) für drei Stichproben-Crates |
| **S-10C** | Crate `ids` (u64/u128, `from_key`, `IdError`); `types` re-exportiert und hat `From<IdError> for ContextraError`; Golden-Vektoren 64/128 inkl. Display-Golden; `vector` nutzt `ids::DocId`; `mvcc`, `crypto`, `checkpoint`, `graph`, `text`, `kvcache` führen `ids` und `env` als Kernabhängigkeit. | 88 Aufrufstellen klassifizieren und anpassen (Rückfall §9.5 Punkt 1 bei mehr als 20 nicht-mechanischen). Differenztest um Referenzen für `text`, `graph`, `rank`, `ingest`, `erasure` erweitern. **Vorab D-20:** `rank` (`from_key` über `AHasher`, Zahl- und Präfix-Parsing statt BLAKE3), `graph` (Fehlertyp `ParseIntError`) und `erasure` (`ErasureError`) rechnen oder melden anders als `ids`; ein Austausch ändert dort gespeicherte oder verglichene IDs (INV-1). Danach die fünf Kopien ersetzen. | S-02 | GF, GD, GE, GL |
| **S-11** | `env` mit `Clock`, `Rng`, `IdGen`, `MetricsSink`, `ManualClock`; `legacy_reference_unix_ms.rs` und `conformance_env.rs` vorhanden. | `'static`-Bound an `Clock` (`clock.rs:18`) und `Rng` (`rng.rs:32`) entfernen; `SeededRng::fork` ergänzen (fehlt); `unix_ms` von `&dyn Clock` auf `C: Clock + ?Sized` (§5.2); `now_unix_ms`-Kopien in `text/contract.rs`, `kvcache/contract.rs`, `vector/contract.rs` über `unix_ms` ersetzen (Differenztest); `test-utils` nur dort aktivieren, wo es benutzt wird. | S-02 | GD, GL |
| **S-12a** | `prompt-guard` (`file-audit` nicht Default), `ingest`, `infer-onnx`, `store`, `sys`, `adapt` und alle E2/E3 ohne verbotenen Default. `types`/`ports` sind in `mvcc`, `crypto`, `checkpoint`, `graph`, `kvcache`, `rank`, `text`, `vector` als optional geführt. | **`mvcc` und `crypto` zuerst:** ihr Default `workspace` zieht `types` und `ports` in die Closure von `vector` und `store` (F-07). Danach Default bereinigen in: `audit-export` (`workspace`, `test-utils`), `checkpoint` (`workspace`), `crypto` (`kv-encryption`, `workspace`), `graph` (`workspace`, `experimental-graph`, dazu `session-dag`, `entity-extraction`, `hyperedge-suggest`: D-22), `infer-candle` (`candle`), `kvcache` (`workspace`, `ports-compat`, `crypto-compat`, `kv-encryption`), `license` (`workspace`), `mvcc` (`workspace`), `rank` (`workspace`, `exp-dibud`, `exp-resonance`, `experimental-rank`), `text` (`workspace`), `vector` (`ports-compat`), `testkit` (`erasure-conformance`); `erasure` in S-24. `experimental-*` zu `exp-*` in `graph`, `rank`, `text`, `vector`, `db`, `engine`; `*-compat` als Aliase kennzeichnen. Features ohne `cfg` und ohne Weiterleitung entfernen (Auditlauf; nicht geprüft \[E\]). Gate `error-domain` aktivieren. | S-01 | GH; `cargo hack check --each-feature -p <crate>` |
| **S-12b** | Crate-lokal: `testkit` hat nur `conformance_selftest.rs` und `in_memory_doubles.rs`; `testkit/tests/<crate>/` existiert nicht. | dev-Kanten von E0/E1 auf `types`, `ports`, `testkit` oder E1-Crates entfernen: `mvcc` (`ports`, `types`), `crypto` (`ports`, `types`), `checkpoint` (`testkit`), `kvcache` (`crypto`, `ports`, `types`), `store` (`testkit`) und, **neu gegenüber v4**, `rank` (`ports`, `types`), `infer-candle` (`crypto`, `ports`, `rank`, `types`), `infer-ollama` (`ports`, `types`). Tests nach `testkit/tests/<crate>/` oder hinter `required-features = ["workspace"]`. | S-12a, S-11 | GH, G-L1 (dev-Kanten); Testanzahl vorher und nachher gleich |
| **S-20** | — | `license`: `workspace` aus dem Default (heute `std`, `workspace`); `audit-export`: `workspace` und `test-utils` aus dem Default. | S-12a | GL |
| **S-22** | Quelle liegt in `contextra-vault` (`vault.rs`, 428 Zeilen); `db/volatile_vault.rs` ist ein Zwei-Zeilen-Re-Export; `VaultError`, `compat/`. | `vault` in `members` aufnehmen und aus `exclude` entfernen (`Cargo.toml` Zeilen 44 bis 46); Begründung des `exclude` klären und ein ADR schreiben (R14, F-11); `vault_sys.inc` (34 Zeilen) und `vault_compat.inc` (2 Zeilen) in `db/src/` auf Verwendung prüfen und entfernen. | S-12a | GL; `grep -rn 'volatile_vault' crates --exclude-dir=contextra-vault` nur Re-Exports; `cargo check -p contextra-vault` entfällt, sobald Mitglied |
| **S-23** | `adapt` hat `exp-flow-thompson`, `exp-rie-greedy`, Default `std`. `off_policy.rs` existiert nicht; der Abgleich mit `offpolicy.rs` ist gegenstandslos. | Altnamen `flow-corrected-thompson`, `rie-greedy-personalization`, `exp-bandit-routing` als Aliase kennzeichnen; die `cfg`-Gates in `lib.rs` (Zeilen 53 bis 98) hängen noch an den Altnamen und werden auf die Zielnamen umgestellt. Ziel-Features laut §4.1: `bandit-routing`, `exp-flow-thompson`, `exp-rie-greedy`. | S-12a | GL |
| **S-24** | `erasure` kernfrei (E0), `signing`-Feature, Beweisdateien in `archive/`. Kein Crate im Workspace hängt von `erasure` ab (D-13 gegenstandslos, F-10). | Feature `vault` → `aead` (D-11; neun `cfg(feature = "vault")`-Stellen); Default `signing`, `aead` ohne `async`; `#[non_exhaustive]` an `ErasureError` (R3, fehlt); Golden-Korpus für `CTXFILE`, `CTRALEDG`, `CTXSCOM`, `CTXSLOK`; `examples/` fehlt (§5.4). | S-12a, S-02 | `cargo check --no-default-features` und je Feature |
| **S-30B** | `mvcc` hat `compat/`, `MvccError` (`mvcc_error.rs`), `golden_corpus.rs`, `docid-128`-Weiterleitung. | `default` leer (S-12a deckt es ab); `types/src/tombstone.rs` ersatzlos entfernen (Re-Exports in `types/lib.rs` Zeilen 49 und 73; R9). | S-10C, S-11 | GL, GF, GC, GD (Tombstone), GE |
| **S-31** | `TenantId` kommt aus `ids`; `CryptoError`; `compat/`; Kern `ids`, `env`. | `default` leer (`kv-encryption` bleibt Feature bis S-38). `KeyManager`: additive `*_bytes`-Varianten existieren (`crypto.rs` Zeilen 261 bis 297); die `ModelFingerprint`-Signaturen (Zeilen 312 ff.) werden in einem eigenen Commit auf `[u8; 32]` umgestellt. | S-10C, S-11, S-12a | GL, GF (WAL-HMAC, `MFKD`, KV v2), GC, GE |
| **S-32** | `PostingStore` (`storage.rs`), `TextError`, Brücke aus `env` für `Clock` (`contract.rs` Zeile 38), Kern ohne Brückenreferenzen (0, streng gezählt), Kern `ids`, `env`. | Lokale `DocId`-Kopie (`types.rs`) und `now_unix_ms`-Kopie (`contract.rs`) ersetzen; `id_width_bits` im Header und Breitentest (kein `id_width_bits` im Crate \[V\]); Features `workspace`, `types-compat`, `ports-compat` auf einen Weiterleitungsweg reduzieren; Default `workspace` (S-12a). | S-10C, S-11 | GL, GF (Postings v alt/neu), GD (ms), GE |
| **S-33** | `RankError`, `compat/`, Kern ohne Brückenreferenzen (0). | `rank` hat **keine** internen Kernabhängigkeiten (kein `ids`, kein `env`): `DocId` (`local_types.rs`, siehe D-20) und `Clock` ablösen; `dibud` → `exp-dibud`, `fusion/resonance.rs` → `exp-resonance` mit Naht `fusion/ext.rs`; `engine::temporal_filter` (`temporal_filter.rs`, volles Modul) wird `pub use` auf `rank::temporal` nach Differenztest. | S-10C, S-11 | GL, GD |
| **S-34** | Kern `ids`, `env`; `GraphError`, `GraphMutationError`; `compat/`. | **140 ungegatete Brückenreferenzen in 60 Dateien** (streng gezählt; das Skript meldet 136), u. a. `cascade.rs`, `csr/graph_write/mod.rs`, `ppr/tests/*`, `apprh/mod.rs`, `test_doubles.rs`, `lib.rs`; lokale `DocId` und `TxId` (`types.rs`, D-20); `default` leer (D-22); `id_width_bits` im Header; Brücke nach §9.4. | S-10C, S-11, S-12a | GL, GF (bincode v alt/neu), GC (TSan), GE |
| **S-35** | `SnapshotSource` (`snapshot_source.rs`, 12 Methoden), `CheckpointError`, Kern ohne Brückenreferenzen (0), `id_width_bits` im Kopf (vor v4). | Nur noch: Default `workspace` und dev-Kante `testkit` (S-12a, S-12b); Dateinaht (`hardlink_cloner.rs`) gegen `ports::Vfs` prüfen \[E\]; Golden-Korpus für Meta und Manifest. | S-10C, S-11 | GL, GF (Meta und Manifest) |
| **S-36** | `DocId`/`TxId` aus `ids`; `VectorError`; `partial_rebuild_veto_f02_guard.rs` existiert unter `vector/tests/` (Grünstatus nicht geprüft). | **14 ungegatete Brückenreferenzen in 6 Dateien** (`hnsw/vector_index_impl.rs`, `hnsw/purge.rs`, `hnsw/scanner.rs`, `hnsw/tests.rs`, `candidate_stream.rs`, `diskann/vector_index_impl.rs`); die Dateien liegen parallel zu `compat/{hnsw_vector_index,diskann_vector_index,purge,scanner,candidate_stream}.rs`, ob als Dubletten, ist zu prüfen (F-15). `contract.rs` (`now_unix_ms`, `Clock`, `Rng`) vereinen; `HNSW_VERSION` 2 → 3 (heute 2); Feature `simd`: Verdrahtung in `distance.rs` begonnen, Orakel-Differenztest fehlt; Default `ports-compat` entfernen; Closure heute 5 statt 3 (F-07). Wachtest vor jeder Änderung grün bestätigen. | S-30B, S-10C, S-11 | GL, GF (HNSW v2/v3), GD (Distanz, Rng, ms), GC, GE |
| **S-50** | — | `ports/license.rs` (135 Zeilen: `FeatureRing`, `LicenseGate`, `OpenFastGate`, `VectorDeleteMode`, `LicenseError`) und `license/src/types.rs` definieren dieselben Konzepte doppelt (R4); `ports/erasure.rs` (272 Zeilen), `erasure_legacy.rs` und `conformance.rs` (278 Zeilen) liegen noch in `ports`. Abgabe nach `license`, `erasure` und `testkit`; Re-Exports bleiben (R9). `store/compat/license.rs` nutzt heute `ports::license::FeatureRing`. | S-20, S-24 | GL; `grep -rln 'FeatureRing\|LicenseGate' crates/contextra-{store,vector,text,graph,kvcache,rank,checkpoint,mvcc}` leer, außer in `store` bis S-37 |
| **S-37** | `StoreError`; `compat/{engine,scanner,license,error}` hinter `workspace`; `FeatureRing` nur noch in `compat/license.rs`. | `types` und `ports` stehen als **nicht optionale** Kernabhängigkeiten in `[dependencies]` (Closure 7 statt 5, F-07). **28 ungegatete Brückenreferenzen in 7 Dateien** (`src/scanner/{wal,tests,runtime,store_scan_view,memtable,sstable}.rs`, `src/engine/memory_only.rs`), parallel zu `compat/scanner` und `compat/engine` (F-15). `validate_against_features` liegt noch in `compat/license.rs`; die Prüfung rufen künftig `engine` und `db` über `license` auf \[E\]. `tenant_codec` lokal gegen `ids::TenantId` prüfen. Zuletzt Manifest 1 → 2 mit `id_width_bits`. | S-30B, S-31, S-10C, S-11, S-50 | GL, GF, GC (Loom/TSan), Crash- und Recovery-Suite unverändert grün (INV-6), GE |
| **S-38** | `kvcache`: Kern `env`, `ids`, `compat/`, `KvCacheError`, 0 Brückenreferenzen. `kv_shredding.rs` und `revocation_log.rs` liegen bereits in `erasure`, nicht mehr in `crypto`. | **Paar:** `crypto` (`kv_cipher.rs`, `kv_segment/{delete_mode,eviction_worker,manager,segment,store}.rs`) und `kvcache` (`cipher.rs`, `segment.rs`, `store.rs`, `eviction_worker.rs`) führen parallele KV-Implementierungen (R4, F-14). **Erst Item-Diff, dann eine Quelle festlegen**, dann umziehen. Kern, Brücke und Features von `kvcache`; `infer-candle/kv_bridge`; `mutants-guarantees.toml`, `loom-gate.yml`, `CODEOWNERS` im selben Commit. `CURRENT_KV_KEY_DERIVATION_VERSION` und `CURRENT_KV_FORMAT_VERSION` per `const_assert_eq` koppeln. | S-31, S-11 | GL, GF (KV-Segment), GC (`loom_kv_deferred_zeroize` am neuen Ort grün), Mutation-Score ≥ Baseline |
| **S-39** | `OllamaError`, `CandleError`, `compat/` in beiden; `candle-*`, `tokenizers`, `half` optional hinter `candle`; Kern von `infer-candle`: `env`. | `infer-ollama`: Kern ohne `env` (Rng/Clock aus `env` fehlen); Beispiele fehlen. `infer-candle`: Default `candle` entfernen; 7 ungegatete Referenzen in `inference/inference_tests.rs`; `capabilities.fragment.toml` und `examples/` fehlen; `ConfidenceCalibrator`, Newtype `ModelFingerprint`; `grep -rn 'contextra_infer_candle::' crates` über alle Verbraucher vor dem Gaten. | S-11, S-38 | GL; `cargo tree -p contextra-infer-candle --no-default-features` ohne `candle-*` und ohne `rank` im Kern |
| **S-51** | — | `types/src/erasure.rs` (617 Zeilen) → `erasure`; `PprConfig`, `PprAlgorithm`, `TlHfdParams` stehen in `types/src/types/domain/misc.rs` **und** doppelt in `graph/src/types.rs` (`TlHfdParams` zusätzlich in `graph/src/tl_hfd/params.rs`): auf einen Besitzer (`graph`) zusammenführen. | S-50, S-34 | GL, GE |
| **S-60** | `adapt` hat `decay_controller.rs`, `homeostat.rs`, `pid.rs`. | Kopien in `engine/src/decay_controller.rs`, `db/src/homeostat.rs`, `db/src/pid_latency_controller.rs` bestehen daneben; Methodenparität (decay 5, homeostat 3, pid 5 \[Ü\]) herstellen, Differenztest, dann `pub use`. | S-23 | GD (bitgleich) |
| **S-61** | Paritätstest `engine/tests/proof_chunker_vs_ingest.rs` (vergleicht u. a. die Token-Schätzung); `ingest` ist dev-Abhängigkeit von `engine`. | Test auf die Chunk-Folge samt Breadcrumbs erweitern \[E\]; dann `engine/src/chunker/` über `ingest::chunking` anbinden; `contextra::chunker::MarkdownChunker` bleibt API-stabil. | S-01 | GD; `mcp`-Tests unverändert |
| **S-62** | Außerhalb von `router` verweist nichts mehr auf `contextra_router::ports_local` (`grep`, `mcp` eingeschlossen). | `router/src/ports_local.rs` und `pub mod ports_local;` (`lib.rs:33`) entfernen; `Vfs`-Stelle auf lokale `&Path`-Funktion (nicht geprüft). `cognition/src/ports_local.rs` ist ein anderes Modul (S-63). | S-01 | GL |
| **S-63** | Lokale Ports `DocumentSource`, `ConsolidationSink` (`cognition/ports.rs`), `ContextStore`, `ScratchpadStore` (`agent/ports.rs`); `db` hat das optionale Feature `cognition`. | `cognition` hängt im Kern noch an `contextra-engine` und `contextra-graph`; `agent` im Kern an `db` (mit `volatile-vault`), `checkpoint`, `router`. Vier `*_local.rs` in `cognition` (`engine_local`, `graph_local`, `ports_local`, `types_local`) löschen. Inversion in `db` abschließen. | S-37, S-50 | GL; `cargo tree -p contextra-cognition -e normal` ohne `contextra-engine` |
| **S-64** | `engine`-Features `exp-extraction`, `sandbox`, `onnx`, `adaptive-candidate-pool-sizing`; `mcp` aktiviert `prompt-guard/file-audit` (fest verdrahtet). | `contextra`: Features `license` und `docid-128` fehlen (F-02); `engine`: Lizenz → `license` (Feature fehlt, `license_seam.rs` vorhanden), `exp-extraction` ohne Zusammenlegung; `db`; `testkit`-Default `erasure-conformance` entfernen, Suite nach `erasure` (Feature `conformance`); `py`: `docid-128` leitet an `contextra-types` statt `contextra-ids` weiter (gleichwertig; §4.1 oder Code angleichen); ohne Feature `license` gilt `OpenFastGate`-Semantik, per Test belegt. | S-50, S-51, S-33 | GL; G-A1 ohne unbeabsichtigte Diffs |
| **S-70** | — | `#[deprecated]`-Re-Exports (R9) entfernen; `*-compat`-Features entfernen (14 Crates: `adapt`, `audit-export`, `checkpoint`, `crypto`, `graph`, `infer-ollama`, `ingest`, `kvcache`, `license`, `mvcc`, `rank`, `simd`, `text`, `vector`), Alias `vault` in `erasure`, Altnamen `dibud`, `diskann`; Fehlervarianten in `ContextraError` entfernen (nur bei grünem G-E1; `Sandbox*` hat 12 Treffer in `types/error.rs`); `archive/`-Verschiebungen bestätigen. | alle | alle Gates |
| **S-70a** | `archive/contextra-crypto-proofs` existiert; `crypto` hat kein Feature `legacy-erasure` mehr. | Item-Diff `archive/contextra-crypto-proofs` ↔ `erasure` und Mutation-Score der Zieldateien in `erasure` ≥ Score der Quelldateien; bei Nichterfüllung Wiederherstellung hinter `legacy-erasure` (nicht Default). **Unabhängig von allem außer S-00; wegen des Risikos früh ausführen.** | S-00 | G-C1 |
| **S-80** | — (Crate `contextra-vector-diskann` existiert nicht) | `vector/diskann/` herauslösen (D-09). | S-36 | GL, DiskANN-Tests (`required-features`) isoliert grün |
| **S-81** | — (Crate `contextra-graph-research` existiert nicht) | Klasse-A/B-Experimente herauslösen. | S-34 | GL, GD |

### 9.3 Reihenfolge-Zwang und empfohlene Wellen

**Zwang (Zyklenfreiheit \[E\])**

- `ids` und `env` stehen vor allem E1.
- `mvcc` steht vor `vector` und `store`; der `crypto`-Kern steht vor `store`.
- **S-12a für `mvcc` und `crypto` steht vor jedem Closure-Nachweis von `vector` und `store`** (F-07), sonst bleibt G-L4 rot.
- **S-50 steht vor S-37** (V9): Erst wenn Lizenz- und Erasure-Ports ihre Besitzer haben, verliert `store` die `FeatureRing`-Nutzung.
- Der `crypto`-KV-Umzug (S-38) folgt dem `crypto`-Kern, beginnt mit einem Item-Diff und geschieht gemeinsam mit `kvcache`.
- S-12b (dev-Kanten) steht vor dem Entnehmbarkeitsnachweis (§8.4) jedes betroffenen Crates.
- S-22 (`vault` wird Workspace-Mitglied) steht vor jeder weiteren Änderung an `db`.
- `cognition` folgt erst der `engine`-Lizenzumstellung; neue Crates (`vector-diskann`, `graph-research`) folgen erst dem jeweiligen Kernschritt.

**Empfohlene Wellen (Parallelität innerhalb einer Welle)**

| Welle | Schritte | Ziel |
| --- | --- | --- |
| 0 | S-00, S-01 (zuerst `\|\| true`), S-02, S-03, S-70a | Gates ehrlich machen, Korpus anlegen, Risiko `crypto-proofs` früh klären |
| 1 | S-11, S-12a (`mvcc`, `crypto` zuerst), S-12b, S-20, S-22, S-23, S-24, S-10C (mit D-20) | Hygiene, saubere Closures, Kerne fertig |
| 2 | S-30B, S-31, S-32, S-33, S-34, S-35, S-36 (untereinander parallel), S-50 | E1-Bausteine, `ports` gibt ab |
| 3 | S-37, S-38, S-39, S-51 | `store`, KV-Umzug, Inferenz, `types` gibt ab |
| 4 | S-60, S-61, S-62, S-63, S-64 | E2 und E3 |
| 5 | S-70, S-80, S-81 | Aufräumen, neue Crates |

### 9.4 Bindende Detailvorgaben je Crate

Diese Vorgaben stammen aus v3 und gelten weiter. \[Ü\] = aus v3 übernommen, am HEAD nicht neu geprüft; sie sind in S-00 gegen den Code zu verifizieren. Änderungen gegenüber v3 sind fett.

| Crate | Vorgabe |
| --- | --- |
| `crypto` | Kern bleibt: `aead`, `dek`, `kdf`, `secret`, `error`, `anti_tamper`, `crypto.rs` (`KeyManager`), `wal_crypto`, `wal_completeness`. `audit_chain.rs` folgt der Nachprüfung S-70a \[Ü\]. `KeyManager::{derive_kv_key, derive_kv_key_scoped, cipher_for, cipher_for_scoped}` nehmen `[u8; 32]` statt `kv_cipher::ModelFingerprint`. `ed25519-dalek` nur, solange `ed25519_proof`-Code im Crate liegt. |
| `text` | Das lokale Trait `PostingStore` enthält genau die in `src/` genutzten `StorageEngine`-Methoden (Erhebung per `grep`), keine mehr. Die Brücke implementiert es für `StorageEngine`. `types.rs` und `contract.rs` werden `pub use` auf `ids`/`env`. Leere Features nach Audit streichen. |
| `rank` | `DocId` → `ids`, `Clock` → `env`. `dibud` → `exp-dibud`, `fusion/resonance.rs` → `exp-resonance` mit Naht `fusion/ext.rs`. `drift` bleibt (keine Dublette). |
| `graph` | Brücke: `provider.rs`, `purger.rs`, `csr/graph_index/*`, alle Importe von `ports::{graph_index, graph}`. Lokale `DocId` und `TxId` → `ids`. Experimente der Klassen A und B hinter `exp-*`, Klasse C (`hyperedge`, `path_rag`, `consistency_enforcement`) bleibt im Kern. Ab 2.000 Zeilen Experimentcode: `graph-research` (S-81). |
| `checkpoint` | `SnapshotSource` = genau die genutzten `StorageEngine`-Methoden. Lokale schmale Dateisystemnaht (`hardlink_cloner.rs`) statt `ports::Vfs`. `WorkflowState` lokal abbilden. Nicht mit `mvcc` zusammenlegen. |
| `vector` | Brücke: `hnsw/vector_index_impl.rs`, `diskann/vector_index_impl.rs`, `hnsw/purge.rs`, `hnsw/scanner.rs`, Teile von `error.rs` und `types.rs`. `mvcc` bleibt Kernabhängigkeit; ein eigener `VisibilityLog` ist Nicht-Ziel. Das Feature `simd` ersetzt `compute_distance` durch `contextra_simd::dispatch`, die Skalarpfade bleiben als Orakel. V-VETO: `PartialRebuildConfig` bleibt aus; der Wachtest muss existieren und grün sein. |
| `store` | Eigener `StoreError` mit Abbildung der genutzten Varianten (`Storage`, `Io`, `WalCorruption`, `ChecksumMismatch`, `Transaction*`, `Conflict`, `WalTruncationDetected`, `CrossDeviceLink`). **`validate_against_features` und `FeatureRing` entfallen in S-37, nach S-50**; `validate_durability` bleibt. `tenant_codec` → `ids::TenantId`. `io_seam` und `fsync` unverändert, `Vfs` per Brücke. INV-6 strikt. |
| `kvcache` | Brücke: `prefix_store.rs`, `adapters.rs` (inkl. `FeatureRing` bis S-50), `KvCachePruner`/`KvCacheScanner`, `scratchpad_invalidation.rs`, Port-Teil von `service.rs`. `CacheDirective` und `StepId` einmal lokal. Features: `kv-encryption` ohne `legacy-erasure`, `exp-kivi`, `exp-content-addressed`; `kvcache-attention-eviction` entfällt. |
| `infer-ollama`, `infer-candle` | `ImportanceScore` lokal mit Skala 0 bis 1 im Doc-Kommentar, `From` in `compat/`. Vorbedingung: `--no-default-features` baut (ungegatetes `pub use error::…` prüfen). `candle-core`, `candle-nn`, `candle-transformers`, `tokenizers` und `half` sind optional hinter `candle`. Vorbedingung: `grep -rn 'contextra_infer_candle::' crates` über alle Verbraucher, damit durch das Gaten kein Symbol verschwindet. |
| `types`, `ports` | `types` behält `error`, `error_dto`, `budget`, `filter`, `importance`, `saos`, `document`, `misc` (abzüglich Auszug), `schema`, `RequestContext`, `model_fingerprint`, `retrieval_strategy`. Die Varianten `Sandbox*`, `Cluster`, `KvQuantization`, `KvDeleteModeConfig`, `HnswConnectivityDegraded` und `GraphRepairFailed` bleiben, bis G-E1 grün ist. `ports` behält `storage`, `vector_index`, `text_index`, `graph_index`, `graph`, `embedding`, `reranker`, `query_rewriter`, `checkpoint`, `kv`, `kv_bridge_port`, `attention`, `lifecycle`, `observability`, `plugin`, `vfs`, `fs`, `scanner/*`; `Vfs`, `VfsFile`, `VfsOpenOptions` und `VerifiedVia` bleiben. |
| `router`, `engine`, `db`, `contextra`, `agent`, `cognition` | `router`: `RetrievalStrategy`-Abbildung nur in `arm_registry.rs`. `engine`: `extraction/` → `exp-extraction`, ohne Zusammenlegung mit `graph/entity_extraction.rs`. `db`: Regler als `pub use` aus `adapt`; `multistep.rs` bleibt. `agent`: lokale Ports `ContextStore`, `ScratchpadStore`. `cognition`: lokale Ports `DocumentSource`, `ConsolidationSink`, `HyperedgeSink`, `DecayPolicy` und `CognitionError` statt `ContextraError`-Varianten; die `*_local.rs`-Umbenennungsdateien werden gelöscht. |
| E0-Crates | `sys`: `lib.rs:19` korrigieren. `ingest`: `fuzz_support.rs` hinter `fuzz`; Brücke mit `TryFrom` im 128-Bit-Modus. `license`: Inline-Brücken → `compat/`, `VectorDeleteMode` → `vector`. `audit-export`: `testkit.rs` hinter `test-utils`; die Brücke `ProcessingRegistryEntry → AuditRecord` wird nicht gebaut. `infer-onnx`: `fakes` hinter `test-utils`. **`erasure`: Feature `aead`, Formate nach §6.1. `prompt-guard`: `file-audit` aktivieren nur Verbraucher (`mcp`).** |
| E3-Crates | `mcp`: `protocol.rs`, `egress_*.rs`, `sandbox.rs` bleiben; aktiviert `prompt-guard/file-audit`. `py`: `docid-128 = ["contextra-ids/docid-128"]`, `maturin` mit und ohne `pyo3` getrennt prüfen. `testkit`: `conformance/vfs.rs` und `fault_vfs.rs` bleiben; `erasure-conformance` verlässt den Default, die Suite zieht nach `erasure` (Feature `conformance`); **`testkit` hostet die Bausteintests nach R12.** |

### 9.5 Rückfallverfahren

Dieses Verfahren ersetzt den in v3 genannten, aber nie geschriebenen „§9.2“.

1. **`from_key` mit mehr als 20 nicht-mechanischen Stellen (§5.1):** Commit C wird dreigeteilt. C1 stellt die mechanischen Stellen um. C2 führt in `types` die Übergangsfunktion `doc_id_from_key(&str) -> Result<DocId, ContextraError>` ein, markiert `#[deprecated]`. C3 stellt jede nicht-mechanische Stelle einzeln um und entfernt die Übergangsfunktion in S-70.
2. **Gate nach Commit B rot:** Lässt es sich nicht innerhalb eines Arbeitstags beheben, wird B per `git revert` zurückgenommen (INV-3). Commit A bleibt, weil er nur verschiebt. Der Fund kommt ins Befundregister (Anhang A).
3. **Differenztest rot:** Der Schritt bricht ab, die Kopie bleibt bestehen. Die Toleranz wird nicht angepasst, um den Test grün zu machen; die Abweichung wird als Befund erfasst und entschieden (INV-10 sinngemäß).
4. **Format-Hash weicht ab (G-F1):** Der Schritt bricht ab. Eine absichtliche Formatänderung ist nie Teil eines Umbauschritts (§6.1).

## 10 Entscheidungsregister

Jede Festlegung, auf die diese Spezifikation oder `AGENTS.md` verweist, steht hier mit Begründung. Eine Entscheidung ändert sich nur durch ein ADR in `docs/decisions/`. Die Nummern D-06, D-08 und D-19 stammen aus v3, wo sie nirgends definiert waren; D-08 bezeichnet in v4 etwas anderes (siehe D-08 und D-18).

| ID | Festlegung | Begründung | Status |
| --- | --- | --- | --- |
| D-01 | Bausteine bleiben getrennte Crates und werden über das Feature `workspace` und das Modul `compat/` an `types` und `ports` angebunden. Crates werden nicht zusammengelegt. | Ein Baustein muss ohne `types` und `ports` einsetzbar sein; die Anbindung ist eine Option, kein Kern. | festgelegt (aus v2) |
| D-02 | Nur Anwendungs-Crates (`contextra`, `mcp`, `py`, `bench`) aktivieren `docid-128`. Bausteine leiten es nur weiter. | Cargo vereinheitlicht Features im ganzen Graphen; ein Baustein, der das Feature setzt, ändert die Breite für alle anderen. Ein Typparameter `DocId<N>` wäre sauberer, müsste aber in jede Signatur \[E\]; er wird neu bewertet, sobald ein Projekt beide Breiten mischen muss. | festgelegt (v4) |
| D-03 | `store` darf 5 Kern-Closure-Mitglieder haben, alle anderen E1-Crates höchstens 4. | `store` braucht `mvcc` und `crypto` fachlich; eine Teilung wäre künstlich. | festgelegt (v4) |
| D-04 | Ein Gate in `ratchet.toml › not_run_baseline` zählt als nicht vorhanden. Die Liste darf nur schrumpfen; jedes Gate bekommt in S-01 einen CI-Job oder ein ADR mit Frist. | Sonst sind INV-10, R4 und R11 nur Absichtserklärungen. | festgelegt (v4) |
| D-05 | G-L1 wertet normale, dev- und Feature-Kanten aus. | `-e normal` lässt dev-Kanten auf `testkit` und Feature-Zyklen unsichtbar. | festgelegt (v4) |
| D-06 | Die Beweisdateien von `crypto` liegen in `archive/contextra-crypto-proofs` \[V\]. Die Nachprüfung läuft in S-70a. | Löst die v3-Fassung „bleiben bis Nachweis“ ab. | überholt, Nachprüfung offen |
| D-07 | `AGENTS.md` folgt dieser Spezifikation und dem Code: `erasure`, `license`, `audit-export`, `avv-generator`, `agent`, `cognition` liegen in `crates/` und sind Workspace-Mitglieder \[V\]. | `AGENTS.md` §2.4 behauptet das Gegenteil. | festgelegt (v4), Umsetzung S-03 |
| D-08 | Commit-Format: `S-xx A: …` und `S-xx B: …`. `WP-xx` entfällt. | INV-3 verlangt zwei Commits je Schritt; `AGENTS.md` kennt nur einen. | festgelegt (v4), Umsetzung S-03 |
| D-09 | Quellcode darf entfernt werden, wenn sein Inhalt in einem Besitzer-Crate liegt oder per Git-Tag archiviert ist. „Keine neuen Features“ (`AGENTS.md` Regel 5) verbietet neue Funktionalität; das Herauslösen bestehenden Codes (S-80, S-81) und das Umbenennen von Features sind erlaubt. | Die Spezifikation verschiebt und entfernt Code; `AGENTS.md` verlangt „Archivieren vor Löschen“. | festgelegt (v4) |
| D-10 | `env::Clock` und `env::Rng` tragen kein `'static`. `SeededRng::fork(stream)` liefert unabhängige Generatoren. Determinismus nach INV-5 gilt je Generator; ein geteilter `Rng` ist unter Nebenläufigkeit nicht reproduzierbar. | `'static` verbietet geliehene Fakes. `&self`-Zugriff auf einen gemeinsamen Zustand ordnet Werte nicht-deterministisch zu. | festgelegt (v4) |
| D-11 | Das Feature `vault` in `erasure` heißt `aead` (Abhängigkeit `aes-gcm-siv` \[V\]). `vault` bleibt bis S-70 als Alias. | Namensgleichheit mit dem Crate `contextra-vault`. | festgelegt (v4) |
| D-12 | Tests, die `ports`, `types` oder `testkit` brauchen, liegen in `testkit/tests/<crate>/`. Bausteine haben keine dev-Kante auf `testkit`. | Eine dev-Kante zieht `types` und `ports` in jeden Test-Build und hebt die Isolation auf. | festgelegt (v4) |
| D-13 | `async` ist in `erasure` nicht Default. Verbraucher aktivieren es explizit. | `async` zieht `tokio` und `futures-util`; das ist für einen autarken Baustein nicht wesensnotwendig (R10). | festgelegt; Verbraucher am HEAD `fcd387021`: keiner im Workspace \[V\] (S-24) |
| D-14 | `contextra-vault` wird Workspace-Mitglied, `exclude` und `publish = false` entfallen. | `db` hängt davon ab; INV-4 muss es erfassen. | festgelegt, Begründung für `exclude` vorher klären (S-22) |
| D-15 | Diese Spezifikation enthält keinen Ist-Stand. Messwerte stehen in generierten Berichten (§8.3). | Die Spezifikation war beim Review 59 PRs hinter dem Repo. | festgelegt (v4) |
| D-16 | Es gibt keine In-Place-Migration zwischen 64 und 128 Bit. | Ein Migrationswerkzeug wäre ein eigenes Produkt; der Bestand lässt sich aus den Quellen neu aufbauen. | festgelegt (v4) |
| D-17 | `capabilities.toml` ist die einzige Quelle für Ebene, Unterebene und Ziel-Kernabhängigkeiten (`level`, `sublevel`, `core_deps`). | Der Score hat die Zielwerte hartkodiert \[V\] und doppelt damit §4.1. | festgelegt (v4), Umsetzung S-03 |
| D-18 | Tier `frozen`: `erasure`, `license`, `audit-export`, `avv-generator`, `sandbox`, `infer-onnx`, `infer-candle`. Tier `experimental`: `durable-fs`. | Das war in v3 und `AGENTS.md` D-08; `AGENTS.md` verweist künftig auf D-18. | festgelegt (aus v3) |
| D-19 | `no_std` ist kein Ziel. `ids` behält das Feature `std` samt Default. | Das Feature existiert mit Default \[V\]; ein Entfernen wäre ein Bruch für Verbraucher mit `default-features = false`. | festgelegt |
| D-20 | `DocId`-Kopien, die anders rechnen oder anders melden als `ids::DocId` (`rank`: `from_key` über `AHasher` und Zahl-/Präfix-Parsing; `graph`: `ParseIntError`; `erasure`: `ErasureError`), werden nicht stillschweigend gleichgeschaltet. Je Kopie entscheidet vor dem Austausch ein ADR: (a) der Verbraucher akzeptiert die `ids`-Werte (Datenneuaufbau nach D-16), oder (b) die alte Funktion lebt als `from_legacy_key` im Besitzer-Crate weiter. | §5.1 verlangt bitgleiche Ergebnisse; wo die Kopie anders rechnet, würde der Austausch gespeicherte oder verglichene IDs ändern (INV-1). | Vorschlag, ADR nötig |
| D-21 | INV-11 gilt für Bibliotheks-Targets (`src/`). Test-Binaries mit `GlobalAlloc`-Zähler (`text/tests/alloc_profiler.rs`, `graph/tests/ppr_alloc_test.rs`) bleiben mit lückenlosem `// SAFETY:`-Nachweis als einzige Ausnahme, oder sie werden durch eine Messung ohne `unsafe` ersetzt. | Die Aussage „nur Kommentare“ ist falsch (F-08); ohne Entscheidung ist `tests/unsafe_islands.rs` entweder rot oder zu locker. | Vorschlag, Entscheidung der Eigentümer |
| D-22 | `graph` führt `session-dag`, `entity-extraction` und `hyperedge-suggest` im Default; sie stehen nicht in der geschlossenen Liste von R10. Vorschlag: `default` leer (so verlangt es §9.4 und S-34), Verbraucher (`engine`, `cognition`) aktivieren sie explizit. Alternative: die Liste in R10 erweitern. | R10 ist geschlossen; §9.4 sagt „`default` leer“. Die Verbraucher der drei Features sind noch nicht ermittelt \[E\]. | Vorschlag, in S-34 zu ermitteln |
| D-23 | Gate-Schritte in CI enthalten weder `\|\| true` noch `continue-on-error`. Ein nicht-blockierender Lauf zählt als nicht vorhanden (D-04 sinngemäß). | §7.1 definiert Exit-Codes; mit `\|\| true` sind sie wirkungslos (F-01). | festgelegt (v5) |

## Anhang A Befundregister

Jeder Befund nennt Beleg und Schritt. IDs sind stabil; ein behobener Befund bleibt mit Status stehen. Alle Belege \[V\] am HEAD `fcd387021`, außer wo markiert.

| ID | Befund | Beleg | Schritt oder Entscheidung | Status |
| --- | --- | --- | --- | --- |
| F-01 | CI blockiert INV-4 und G-L1 nicht. | `rust-ci.yml`: Zeile 28 (`cargo check --workspace --all-targets --locked \|\| true`), Zeile 43 (`gate_layering.py --all \|\| true`), Zeile 57 (`docid-128`-Schleife); 14 `continue-on-error` in `rust-ci.yml`; je ein `\|\| true` in `loom-gate.yml` und `mutation-testing.yml` | S-01, D-23 | offen |
| F-02 | `cargo check -p contextra --features docid-128` verweist auf ein nicht vorhandenes Feature. | `rust-ci.yml`, Job `docid-128-check`; `crates/contextra/Cargo.toml` `[features]`: `embedded`, `candle` | S-01, S-64 | offen \[E, nicht ausgeführt\] |
| F-03 | Die G-L2-Prüfung ignoriert Kommentare, anders als §7.2 verlangt. | Skript: 185 Treffer; `grep` streng: 189 (`graph` 140 statt 136) | S-00, S-01 | offen |
| F-04 | Drei `DocId`-Kopien rechnen oder melden anders als `ids`. | `rank/src/local_types.rs` (`AHasher`, `Result<_, String>`); `graph/src/types.rs` (`ParseIntError`); `erasure/src/types_local.rs` (`ErasureError`) | D-20, S-10C | offen |
| F-05 | Der Differenztest für `ids` deckt zwei Referenzen ab, nicht die fünf Kopien. | `ids/tests/differential_ids.rs`, `ids/tests/legacy_reference/{types_legacy,vector_legacy}.rs` | S-10C | offen |
| F-06 | Der Baseline-Bericht ist veraltet. | `docs/decoupling/baseline/DIFF_BASELINE.md`: HEAD `9305c5d04`, 10.11.2026 | S-00 | offen |
| F-07 | Default-Features der Abhängigkeiten lecken `types` und `ports` in Closures: `vector` 5 statt 3, `store` 7 statt 5 (Default-Closure von `vector`: 6). Ursache: Defaults `workspace` in `mvcc` und `crypto` (`store` hängt zusätzlich nicht optional an `types` und `ports`). | Anhang B (Resolver) \[V\*\]; `mvcc`, `crypto`: `default = [...workspace...]` | S-12a, S-30B, S-31, S-37; §2.1 | offen |
| F-08 | INV-11 nannte `unsafe` außerhalb der Inseln „nur Kommentare“. | `text/tests/alloc_profiler.rs`, `graph/tests/ppr_alloc_test.rs` (`unsafe impl GlobalAlloc`) | D-21 | offen |
| F-09 | R12 wird über die v4-Liste hinaus verletzt; `testkit/tests/<crate>/` existiert nicht. | dev-Kanten: `rank` (`ports`, `types`), `infer-candle` (`crypto`, `ports`, `rank`, `types`), `infer-ollama` (`ports`, `types`) | S-12b | offen |
| F-10 | `erasure` hat keinen Verbraucher im Workspace; `ErasureError` ist nicht `#[non_exhaustive]`. | `grep` über alle `Cargo.toml`; `erasure/src/error.rs` | S-24 | offen |
| F-11 | `publish = false` bei zwölf Crates und `exclude` bei `vault` ohne ADR. | Cargo-Dateien; `docs/decisions/` ohne Treffer zu `publish` oder `exclude` | R14, S-22 | offen |
| F-12 | `version` fehlt an Pfad-Abhängigkeiten, entgegen §5.4. | 12 von 54 direkten Pfad-Abhängigkeiten (u. a. in `infer-onnx`, `mcp`, `vault`, `db`, `graph`, `crypto`); 0 von 40 internen `[workspace.dependencies]` | S-03 | offen |
| F-13 | Anhang A und B fehlten in v4. | v4 endet mit §10 | W1 | behoben in v5 |
| F-14 | `crypto/kv_segment` und `kvcache` führen parallele KV-Implementierungen. | `crypto/src/{kv_cipher.rs,kv_segment/*}`; `kvcache/src/{cipher,segment,store,eviction_worker}.rs` | S-38 | offen |
| F-15 | In `store` und `vector` liegen Altdateien mit Brückenreferenzen neben `compat/`-Fassungen. | `store/src/scanner/*`, `store/src/engine/memory_only.rs` neben `store/src/compat/{scanner,engine}`; `vector/src/hnsw/*` neben `vector/src/compat/*` | S-36, S-37 | offen \[Dubletten nicht geprüft\] |
| F-16 | `gate_dependencies.py` und `gate_g1_layering.py` (Alias) existieren neben `gate_layering.py`; die Spezifikation kennt nur zwei der drei. | `tools/` | S-01 | offen |

## Anhang B Werkzeugliste und Reproduktion

INV-9 verlangt die Toolchain `1.89.0` und `--locked`. Belegt sind die folgenden Werkzeuge \[V\] aus `rust-toolchain.toml`, `.github/workflows/*.yml` und `justfile`.

| Werkzeug | Version oder Quelle | Zweck |
| --- | --- | --- |
| Rust | `1.89.0` mit `rustfmt` und `clippy` (`rust-toolchain.toml`) | alle Builds |
| Rust nightly | `dtolnay/rust-toolchain@nightly` in 5 Jobs | Miri, TSan, `cargo public-api` (G-A1: Nightly ≥ `rust-version` 1.89 \[E\]) |
| `cargo-hack` | `taiki-e/install-action@cargo-hack` | G-L3 |
| `cargo-mutants` | `mutation-testing.yml` | G-C1 |
| `cargo-deny` | `deny.yml` | Lizenz- und Advisory-Prüfung |
| `cargo-llvm-cov`, `cargo-nextest` | `rust-ci.yml` | Abdeckung, Testläufer |
| `cargo-fuzz`, `just` | `cargo install` in Workflows | Fuzzing, kanonische Gate-Aufrufe (§7.1) |
| Python | 3.11 in CI; die Gate-Skripte nutzen `tomllib`, also mindestens 3.11 | `tools/*.py` |
| `sha256sum` | coreutils | G-F1, G-F2 |

**Reproduktion der Messungen in Anhang C ohne `cargo`:**

1. `closure_probe.py` (liegt dieser Revision bei): liest alle `Cargo.toml` unter `crates/` und `benchmarks/`, löst interne Kanten mit Feature-Unifikation auf und gibt je Crate die Closure ohne und mit Default-Features aus. Es ist ein Näherungswerkzeug \[V\*\] und ersetzt `cargo tree` nicht (S-00).
2. Strenge Brückenzählung je Crate (Kommentare eingeschlossen, ohne `compat/`):
   `grep -rEn 'contextra_(types|ports|testkit)\b' crates/contextra-<x>/src --include=*.rs | grep -v '/compat/' | wc -l`
3. `python3 tools/gate_bridge_null.py` liefert die Zählung ohne Kommentare.

## Anhang C Momentaufnahme (HEAD `fcd387021`, 11.11.2026)

Nicht normativ. Wird durch den generierten Bericht (§8.3) abgelöst. „Closure“ nennt ist/soll, berechnet ohne Default-Features des Wurzel-Crates \[V\*\]; in Klammern die Zahl mit Defaults, wenn sie abweicht. „Brücke“ ist die strenge Zählung (Anhang B, Punkt 2). „Fehlt“ bezieht sich auf §5.4.

| Crate | Ebene | Brücke | Closure | Verbotene Defaults (R10) | Verbotene dev-Kanten (R12) | Fehlt (§5.4) |
| --- | --- | --- | --- | --- | --- | --- |
| `ids` | E0 | 0 | 0/0 | — | — | README, Fragment, `examples/`, REPORT |
| `env` | E0 | 0 | 0/0 | — | — | README, Fragment, `examples/`, REPORT |
| `sys` | E0 | 0 | 0/0 | — | — | — |
| `durable-fs` | E0 | 0 | 0/0 | — | — | — (Korpus: S-02) |
| `simd` | E0 | 0 | 0/0 | — | — | — |
| `wire` | E0 | 0 | 0/0 | — | — | — |
| `adapt` | E0 | 0 | 0/0 | — (Altnamen: S-23) | — | — |
| `ingest` | E0 | 0 | 0/0 | — | — | — |
| `privacy` | E0 | 0 | 0/0 | — | — | — |
| `sandbox` | E0 | 0 | 0/0 | — | — | REPORT |
| `prompt-guard` | E0 | 0 | 0/0 | — | — | — |
| `infer-onnx` | E0 | 0 | 0/0 | — | — | — |
| `erasure` | E0 | 0 | 0/0 | `vault`, `async` | — | `examples/` |
| `license` | E0 | 0 | 0/0 (4) | `workspace` | — | REPORT |
| `audit-export` | E0 | 0 | 0/0 (2) | `workspace`, `test-utils` | — | REPORT |
| `avv-generator` | E0 | 0 | 0/0 | — | — | REPORT |
| `mvcc` | E1a | 0 | 2/2 (4) | `workspace` | `ports`, `types` | — |
| `crypto` | E1a | 0 | 2/2 (4) | `kv-encryption`, `workspace` | `ports`, `types` | — |
| `text` | E1a | 0 | 2/2 (4) | `workspace` | — | — |
| `graph` | E1a | **140** | 2/2 (4) | `workspace`, `experimental-graph`, `session-dag`, `entity-extraction`, `hyperedge-suggest` | — | — |
| `rank` | E1a | 0 | **0**/2 | `workspace`, `exp-dibud`, `exp-resonance`, `experimental-rank` | `ports`, `types` | — |
| `checkpoint` | E1a | 0 | 2/2 (4) | `workspace` | `testkit` | — |
| `kvcache` | E1a | 0 | 2/2 (5) | `workspace`, `ports-compat`, `crypto-compat`, `kv-encryption` | `crypto`, `ports`, `types` | — |
| `vault` | E1a | 0 | 1/1 | — | — | `examples/` |
| `infer-ollama` | E1a | 0 | **0**/1 | — | `ports`, `types` | `examples/` |
| `infer-candle` | E1a | **7** | 1/1 | `candle` | `crypto`, `ports`, `rank`, `types` | Fragment, `examples/` |
| `store` | E1b | **28** | **7**/5 | — | `testkit` | — |
| `vector` | E1b | **14** | **5**/3 (6) | `ports-compat` | — | REPORT |

**E2 und E3 (Auswahl):** `cognition` hat im Kern `engine`, `graph`, `ports`, `types` (S-63). `agent` hat im Kern `checkpoint`, `db`, `ports`, `router`, `types` (S-63). `engine` hat 9 direkte Abhängigkeiten ohne `ids`/`env` (`checkpoint`, `graph`, `mvcc`, `ports`, `rank`, `store`, `text`, `types`, `vector`) und Closure 13: wie in §4.1. `db` hat im Kern `engine`, `ports`, `types`, `vault`. `types` hat im Kern `ids`; `ports` hat `env`, `types`.

**Fan-in nach R11 (Startwerte für `ratchet.toml`, mit `cargo metadata` in S-00 gegenzuprüfen \[V\*\]):** `types_fan_in` = 29 Crates, davon 18 aus E0/E1 (`adapt`, `audit-export`, `checkpoint`, `crypto`, `graph`, `infer-candle`, `infer-ollama`, `infer-onnx`, `ingest`, `kvcache`, `license`, `mvcc`, `rank`, `simd`, `store`, `text`, `vault`, `vector`). `ports_fan_in` = 22 Crates, davon 14 aus E0/E1 (`adapt`, `checkpoint`, `crypto`, `graph`, `infer-candle`, `infer-ollama`, `infer-onnx`, `kvcache`, `license`, `mvcc`, `rank`, `store`, `text`, `vector`). Gezählt sind normale, auch optionale Kanten aus den `Cargo.toml` (dev-Kanten ausgeschlossen, R11). Die R11-Zielrichtung ist, dass kein E0/E1-Crate die beiden Kanten im Kern, also nicht optional, führt; am HEAD tut das nur `store`.
