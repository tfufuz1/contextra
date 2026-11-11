# Contextra als Baukasten — Spezifikation v4 (normativ)

Nov 11, 2026 · @kiki

## 1 Status, Geltung, Änderungsprotokoll

Diese Spezifikation v4 ersetzt v3 und ist die einzige normative Quelle für Crate-Grenzen, Abhängigkeitsrichtung, Brücken, Features, Persistenzformate, Gates und die Reihenfolge der Entkopplung. Sie behebt die Befunde des Reviews vom 11.11.2026 (Anhang A).

### 1.1 Grundlage und Grenzen

| Punkt | Inhalt |
| --- | --- |
| Geprüfter Stand | Repository `tfufuz1/contextra`, HEAD `ec6bd67` (11.11.2026, PR #6018). v3 beschrieb `5408826f8` (PR #5959), also rund 59 PRs früher. |
| Methode | Dateien lesen und `grep`. Es wurde nichts installiert, gebaut oder ausgeführt. Jede Zahl ist eine Näherung. |
| Belegmarken | **\[V\]** am HEAD per Datei oder `grep` geprüft · **\[E\]** Einschätzung, nicht belegt · **\[D\]** Festlegung dieser Spezifikation. |
| Ist-Stand | Messwerte (Kanten, Referenzzahlen, Score, Schrittstatus) stehen nicht mehr in dieser Spezifikation, sondern in generierten Berichten unter `docs/decoupling/baseline/` (§8.3). So veraltet die normative Datei nicht mit jedem PR. |
| Crate-lokale Dokumente | 39 Crates haben eine eigene `SPEC.md` \[V\]. Sie regeln die Binnenstruktur und sind verbindlich, soweit sie dieser Spezifikation nicht widersprechen. |

### 1.2 Rangfolge

1. Diese Spezifikation und das Entscheidungsregister (§10). ADRs in `docs/decisions/` gelten nur, soweit sie ihr nicht widersprechen.
2. Crate-lokale `SPEC.md` und `PLAN.md`. Ein Widerspruch zu Punkt 1 ist ein Befund im Register (Anhang A).
3. `AGENTS.md` und `docs/**` sind nachrangig. `AGENTS.md` verweist auf eine nicht existierende Datei (`docs/CONTEXTRA_IMPLEMENTIERUNGS_SPEC.md`) und muss auf `docs/SPEC.md` zeigen (Schritt S-03).

### 1.3 Normative Sprache und Prüfbarkeit

MUSS, DARF NICHT und SOLL gelten nach RFC 2119. **Jede MUSS-Aussage hat ein Prüfverfahren aus §7 oder einen benannten Test.** Eine Aussage, deren Prüfverfahren nicht existiert oder in `governance/ratchet.toml › not_run_baseline` steht, gilt nur als SOLL, bis das Verfahren läuft (D-04).

### 1.4 Änderungsprotokoll v3 → v4

| ID | Befund in v3 | Änderung in v4 |
| --- | --- | --- |
| V1 | §8, §9 und Anhang A fehlen, 14 Verweise laufen ins Leere. | Neu geschrieben: §7 Gates, §8 Score, §10 Entscheidungen, Anhang A und B. Alle Verweise sind aufgelöst. |
| V2 | D-06, D-08, D-19 sind nirgends definiert. | Entscheidungsregister §10 mit Status je Eintrag. |
| V3 | Ist-Stand veraltet (mvcc, crypto, kvcache, rank bereits ohne Referenzen außerhalb `compat/`; `env` Default bereits `std`; `erasure` hat Features). | Ist-Stand aus der Spezifikation entfernt, generiert (§8.3). |
| V4 | §5.2: Tabellenzeile `64 \| 128 oder 128 \| 64` unlesbar; `u16` gegen `u32`. | §6.2 neu mit vollständiger Matrix und einem Breitentyp. |
| V5 | INV-1-Ausnahme nennt vier von elf Formaten mit IDs. | §6.2 deckt alle persistierten Formate ab. |
| V6 | Spalte „Kern“ mischt direkte und transitive Abhängigkeiten (`engine` ohne `sys`, `crypto`). | §2.2 definiert die Spalte als berechnete transitive Closure; §4.1 nennt direkt und Closure getrennt. |
| V7 | R10 widerspricht der Tabelle (`erasure` mit `async` im Default); `mvcc`, `checkpoint`, `prompt-guard`, `infer-candle` fehlen in der Default-Bereinigung. | R10 mit geschlossener Ausnahmeliste; S-12 vollständig. |
| V8 | Feature `vault` in `erasure` (= `aes-gcm-siv`) kollidiert mit dem Crate `vault`. | Umbenennung in `aead` (D-11). |
| V9 | S-37 verlangt G-L2, behält aber `FeatureRing` aus `ports` bis S-50. | S-50 wird vor S-37 gezogen (§9.3). |
| V10 | Dev-Abhängigkeiten ungeregelt; fünf E1-Crates hängen dev-seitig an `testkit` (E3). | Regel R12 und Gate G-H1. |
| V11 | Kanten innerhalb einer Ebene, über Features, ungeregelt. | Regel R13. |
| V12 | `contextra-vault` liegt in `exclude` und hat `publish = false`; die Spezifikation schweigt. | Regel R14. |
| V13 | `docid-128` ist nicht additiv; ein beliebiges Crate im Graphen kippt die ID-Breite für alle. | Entscheidung D-02: nur Anwendungs-Crates aktivieren das Feature (R10). |
| V14 | Prüfverfahren für INV-10, R4, R11 liegen in `not_run_baseline`. | D-04, Schritt S-01 und Gate-Statusspalte in §7.3. |
| V15 | Widersprüche zu `AGENTS.md` (Archiv-Liste, Commit-Format, „keine neuen Features“). | Auflösung in D-07 bis D-09; Schritt S-03. |
| V16 | Determinismus bei Nebenläufigkeit und `'static`-Bounds ungeregelt. | §5.1 und D-10. |

## 2 Begriffe, Ebenen, Entnehmbarkeit

Ein Baustein ist nur dann einsetzbar, wenn seine Abhängigkeiten berechenbar sind. Deshalb definiert dieser Abschnitt jede Größe so, dass ein Werkzeug sie ohne Auslegung ausgeben kann.

### 2.1 Begriffe

| Begriff | Definition |
| --- | --- |
| **Kern** | Der Code eines Crates mit `--no-default-features`, ohne das Feature `workspace`. |
| **Brücke** | Das Modul `src/compat/` hinter dem Feature `workspace`. Nur dort stehen `contextra_types`, `contextra_ports`, Implementierungen von `ports`-Traits und `From`-Konvertierungen nach `ContextraError`. |
| **Kern-Closure** | Alle internen `contextra-*`-Crates in der Ausgabe von `cargo tree -p <crate> -e normal --no-default-features`, **transitiv**, ohne das Crate selbst. Dev-Abhängigkeiten zählen nicht, werden aber durch R12 geregelt. |
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
| INV-11 | **Unsafe-Inseln.** `unsafe` nur in `simd`, `sys`, `wire`, jeweils mit lückenlosem `// SAFETY:`-Nachweis. Alle anderen Crates führen `#![forbid(unsafe_code)]`. Treffer für „unsafe“ außerhalb der Inseln sind am HEAD nur Kommentare \[V\]. | `tests/unsafe_islands.rs`, `unsafe-inventory` |
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
| `kvcache` | E1a | env → 1 | `kv-encryption`→crypto; `kv-shredding`; `exp-kivi`; `workspace` | `std` | `KvCacheError` |
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
| **R14 Workspace und Publizierbarkeit** | Jedes Crate in `crates/` ist Workspace-Mitglied. `exclude` und `publish = false` sind je Crate nur mit ADR zulässig. Ausnahmen heute: `contextra-vault` steht in `exclude` und hat `publish = false` \[V\], obwohl `db` davon abhängt und diese Spezifikation es als Baustein führt. Die Begründung ist hier nicht dokumentiert; sie ist vor der Änderung zu klären, danach wird `vault` Mitglied (D-14, S-22). | G-H1 |

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
- **Offen:** Am HEAD gibt es 87 Aufrufstellen von `::from_key(` außerhalb von `tests/` \[V, grobe Zählung\]. Jede wird als *mechanisch* (`?` über `From`) oder *nicht-mechanisch* (Musterabgleich auf `ContextraError::InvalidInput`) klassifiziert. Mehr als 20 nicht-mechanische Stellen lösen das Rückfallverfahren aus §9.5 aus.

**Abnahme:** `ids_tests` grün. Differenztest gegen die sechs verbliebenen `DocId`-Kopien (`erasure`, `graph`, `ingest`, `rank`, `text`, `vector` \[V\]) über 10.000 zufällige Schlüssel plus Randfälle (leer, Unicode, 4 KiB): gleiche `from_key`-Bytes, gleiche serde-Ausgabe (JSON und `bincode`). Golden-Vektoren in 64 **und** 128 Bit.

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
| MVCC | `mvcc/tests/golden_corpus.rs` \[V\] | `mvcc` | S-30 |

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
4. Das Gate `error-domain` (heute in `not_run_baseline` \[V\]) wird in S-01 definiert und in S-12 aktiviert. Ausgangsbasis ist `tools/error_topology_mapper.py`.

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

### 7.3 Durchsetzungsstand (Stand 11.11.2026, HEAD `ec6bd67`)

Diese Tabelle altert schnell. Sie wird ab S-00 aus `governance/gates.toml` und den Workflows generiert (§8.3); der handgepflegte Stand dient nur als Ausgangspunkt.

| Gate | Werkzeug am HEAD | In CI | Lücke |
| --- | --- | --- | --- |
| G-L1 | `gate_layering.py` \[V\] | `rust-ci.yml`, `justfile` \[V\] | dev- und Feature-Kanten sowie Zyklen prüfen; `gate_g1_layering.py` ist nirgends verdrahtet und entfällt; Verhältnis zu `tests/layering.rs` klären |
| G-L2 | `gate_bridge_null.py` \[V\] | nirgends \[V\] | CI-Job anlegen |
| G-L3 | `cargo-hack` \[V\] | `rust-ci.yml` \[V\] | Lauf für `vault` und Matrix `docid-128` ergänzen |
| G-L4 | `isolation_proof.py` \[V\] | nur `justfile` \[V\] | CI-Job anlegen |
| G-H1 | `feature_audit.py` \[V\] | nur `justfile` \[V\] | entrauschen; `ratchet` steht in `not_run_baseline` \[V\] |
| G-F1 | Gate `format-corpus` \[V Name\]; `tools/golden_regen.sh` und `tests/golden/` fehlen \[V\] | nirgends | S-02 |
| G-F2 | `contextra.fbs.sha256` vorhanden \[V\] | nirgends \[V\] | CI-Job anlegen |
| G-D1 | `determinism_guard.py` \[V\] | nirgends \[V\]; `determinism` steht in `not_run_baseline` | CI-Job anlegen |
| G-E1 | `error_topology_mapper.py` \[V\] | nirgends \[V\]; `error-domain` steht in `not_run_baseline` | S-01, S-12 |
| G-C1 | drei Workflows \[V\] | ja \[V\] | Läufe verifizieren |
| G-A1 | Eintrag in `gates.toml` \[V\] | in keinem Workflow \[V\] | Job und Toolchain-ADR |
| G-DOC1 | `doc_lint.py` \[V\] | nirgends \[V\]; `doc-truth` steht in `not_run_baseline` | CI-Job anlegen |
| INV-3 Commit-Lint | `.githooks/` enthält nur `pre-commit` und `pre-push` \[V\] | — | `commit-msg` anlegen |

Quelltext verweist teils auf „SPEC v4.0 §8.1“ (z. B. `tests/layering.rs` \[V\]). In dieser Spezifikation stehen die Gates in §7.2; diese Verweise werden in S-03 umgestellt.

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
| F Default (neu ab S-12) | je Feature im `default`, das R10 verbietet | 5, höchstens 15 |
| G dev-Kante (neu ab S-12) | je dev-Kante von E0/E1 auf `types`, `ports` oder `testkit` (R12) | 5, höchstens 15 |

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

Der Plan umfasst 33 Schritte in einer festen Reihenfolge. Der Schrittstatus steht nicht hier, sondern im generierten Bericht (§8.3); Teile einzelner Schritte sind am HEAD bereits erledigt (z. B. `types`/`ports` optional in `mvcc` und `crypto` \[V\]).

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

### 9.2 Schritte

„Voraussetzung“ nennt nur harte Abhängigkeiten. Die Schritte S-12 und S-50 unterscheiden sich von v3 (V9, V10).

| Schritt | Inhalt | Voraussetzung | Nachweis |
| --- | --- | --- | --- |
| **S-00** | Werkzeugliste (Anhang B) bereitstellen. Baseline mit `cargo` neu erheben (§8.3), einschließlich `--no-default-features`-Build je Crate. Die 87 `from_key`-Aufrufstellen klassifizieren. Breitentest je Format (§6.2). Skripte gegen §7.2 prüfen: Exit-Code 2, dev- und Feature-Kanten. `modularity_score.py` auf §8.1 und D-17 bringen. | — | Bericht mit HEAD-Hash; „baut ohne Default-Features“ je Crate belegt oder als Befund erfasst |
| **S-01** | `xtask`-Globs aus `protected-paths.toml` (Zeilen 10 und 78 \[V\]) und `xtask`-Kommentare im `justfile` (Zeilen 24 bis 30 \[V\]) entfernen. Für jedes der 38 Gates in `not_run_baseline` \[V\] einen CI-Job oder ein ADR mit Frist (D-04); zuerst `ratchet`, `gate-weakening`, `protected-paths`, `symbol-uniqueness`, `error-domain`, `feature-matrix`, `unsafe-inventory`, `doc-truth`. CI-Jobs für G-L2, G-L4, G-H1, G-F2, G-D1, G-E1, G-A1, G-DOC1. Hook `commit-msg`. Matrix `docid-128`. Ratchet für `types_fan_in` und `ports_fan_in`. `cargo check -p contextra-vault`. `gate_g1_layering.py` entfernen; Verhältnis zu `tests/layering.rs` klären. | S-00 | G-L1 scheitert bei absichtlichem Verstoß (`test_gate_layering_negative.py` \[V\]) und ist am Baseline-Stand grün; für jedes Gate existiert ein Negativtest |
| **S-02** | Wurzel-Korpus `tests/golden/` mit `MANIFEST.sha256`, `tools/golden_regen.sh`, Differenz-Harness (§6.1, §6.4). Korpus von `mvcc` einhängen; Formate von `erasure` und `durable-fs` ergänzen. | S-00 | GF am unveränderten Stand grün |
| **S-03** | Dokumentation und Metadaten. `AGENTS.md`: Archiv-Liste (§2.4) berichtigen, Verweis auf `docs/SPEC.md`, Commit-Format, Klarstellung zu „keine neuen Features“ (D-07 bis D-09); `docs/handoff/OPEN_ISSUES.md` und `STATUS.md` anlegen oder die Verweise entfernen. Quelltext-Verweise auf „SPEC v4.0 §8.1“ umstellen. Verwenderliste in `types/lib.rs` durch einen generierten Verweis ersetzen. Falsche Docstrings (`privacy`, `cognition`, `kvcache`, `avv-generator`, `sys/lib.rs:19`, `vector/types.rs:3–6` \[Ü\]). Item-Diff `AvvError`/`AvvGeneratorError`. `capabilities.toml` um `level`, `sublevel`, `core_deps` erweitern (D-17). | S-01 | G-DOC1, `capabilities-consistency` grün |
| **S-10C** | `ids` Commit C: `from_key`-Aufrufstellen nach Klassifikation anpassen. Differenztest §5.1 gegen alle `DocId`-Kopien. | S-02 | GF, GD, GE, GL |
| **S-11** | `env`: `'static`-Bound entfernen und `SeededRng::fork` ergänzen (D-10); `unix_ms`-Differenztest; `test-utils` nur dort aktivieren, wo es benutzt wird. | S-02 | GD, GL |
| **S-12a** | Feature-Hygiene nach R10 für alle Crates mit Verstoß: `mvcc`, `checkpoint` (Default `workspace`), `crypto`, `graph`, `kvcache`, `rank`, `text`, `vector`, `testkit`, `license`, `audit-export`, `prompt-guard` (`file-audit`), `infer-candle` (`candle`). `experimental-*` zu `exp-*`; `*-compat` als Aliase kennzeichnen. Gate `error-domain` aktivieren. | S-01 | GH; `cargo hack check --each-feature -p <crate>` |
| **S-12b** | Dev-Kanten nach R12: Tests, die `ports`, `types` oder `testkit` brauchen, wandern in `testkit/tests/<crate>/`. Betroffen sind `mvcc`, `crypto`, `checkpoint`, `kvcache`, `store` \[V\]. | S-12a, S-11 | GH, G-L1 (dev-Kanten); Anzahl der Tests vorher und nachher gleich |
| **S-20 / S-22 / S-23** | `license`, `audit-export`: `workspace` aus dem Default. `vault`: Quelle `volatile_vault.rs`, `vault_compat.inc`, `vault_sys.inc` aus `db` entfernen \[Ü\]; Workspace-Mitglied werden (D-14, R14). `adapt`: Features auf `exp-flow-thompson`, `exp-rie-greedy`; `offpolicy.rs` gegen `off_policy.rs` abgleichen. | S-12a | GL; `grep -rn 'volatile_vault' crates --exclude-dir=contextra-vault` nur Re-Exports |
| **S-24** | `erasure`: Feature `vault` heißt `aead` (D-11); Default `signing`, `aead`, ohne `async` (D-13); Verbraucher von `async` ermitteln und explizit aktivieren; Golden-Korpus für die Erasure-Formate. | S-12a, S-02 | `cargo check --no-default-features` und je Feature |
| **S-30B** | `mvcc`: `default` leer; `types/src/tombstone.rs` ersatzlos entfernen (R9; nur `mvcc` und `types` nutzen es \[Ü\]). Der Loom-Pfadfilter deckt `mvcc` ab \[V\]. | S-10C, S-11 | GL, GF, GC, GD (Tombstone), GE |
| **S-31** | `crypto`: `TenantId` → `ids`; `KeyManager`-Signaturen auf `[u8; 32]` (eigener Commit); `default` leer (`kv-encryption` bleibt bis S-38). | S-10C, S-11, S-12a | GL, GF (WAL-HMAC, `MFKD`, KV v2), GC, GE |
| **S-32** | `text`: `PostingStore`, Brücke (`wand.rs`, `scanner.rs`, `index_struct.rs`, `purge.rs`, `morph_index.rs`), `TextError`, Clock → `unix_ms`, ID-Breite im Header. | S-10C, S-11 | GL, GF (Postings v alt/neu), GD (ms), GE |
| **S-33** | `rank`: `engine::temporal_filter` wird `pub use` auf `rank::temporal` nach Differenztest. | S-10C, S-11 | GL, GD |
| **S-34** | `graph`: Brücke, `GraphError`, `default` leer, ID-Breite im Header. | S-10C, S-11, S-12a | GL, GF (bincode v alt/neu), GC (TSan), GE |
| **S-35** | `checkpoint`: `SnapshotSource`, Dateinaht, `CheckpointError`, Restreferenzen. | S-10C, S-11 | GL, GF (Meta und Manifest) |
| **S-36** | `vector`: **zuerst den V-VETO-Test anlegen** (`partial_rebuild_veto_f02_guard` kommt laut v3 nur in `vector/SPEC.md` vor \[Ü\]). Dann Kern auf `ids`/`env`/`mvcc`/`VectorError`, `contract.rs` vereinen, `VectorDeleteMode`, Feature `simd`, `HNSW_VERSION` 2 → 3. Ohne DiskANN-Umzug. | S-30B, S-10C, S-11 | GL, GF (HNSW v2/v3), GD (Distanz, Rng, ms), GC, GE |
| **S-50** | `ports` gibt ab: Lizenz-Traits → `license`, Erasure-Familie → `erasure`, Conformance → `testkit`. Re-Exports bleiben (R9). **Vor S-37**, weil `store` sonst `FeatureRing` aus `ports` im Kern behalten müsste (V9). | S-20, S-24 | GL; `grep -rln 'FeatureRing\|LicenseGate' crates/contextra-{store,vector,text,graph,kvcache,rank,checkpoint,mvcc}` leer, außer in `store` bis S-37 |
| **S-37** | `store`: `StoreError`, `compat/` (`engine/`, `scanner/`), Imports → `ids`/`env`. `validate_against_features` und die `FeatureRing`-Nutzung im Kern entfernen; die Prüfung rufen `engine` und `db` über `license` auf \[E\]. Zuletzt Manifest 1 → 2. | S-30B, S-31, S-10C, S-11, S-50 | GL, GF, GC (Loom/TSan), Crash- und Recovery-Suite unverändert grün (INV-6), GE |
| **S-38** | **Paar:** KV-Teil aus `crypto` (`kv_cipher`, `kv_segment/*`, `kv_shredding`, `revocation_log`) → `kvcache`; Kern, Brücke und Features von `kvcache`; `infer-candle/kv_bridge`; `mutants-guarantees.toml`, `loom-gate.yml`, `CODEOWNERS` im selben Commit. `CURRENT_KV_KEY_DERIVATION_VERSION` \[V\] und `CURRENT_KV_FORMAT_VERSION` per `const_assert_eq` koppeln. | S-31, S-11 | GL, GF (KV-Segment), GC (`loom_kv_deferred_zeroize` am neuen Ort grün), Mutation-Score ≥ Baseline |
| **S-39** | `infer-ollama` (`OllamaError`, `Rng`/`Clock` aus `env`), danach `infer-candle` (Feature `candle`, `ConfidenceCalibrator`, Newtype `ModelFingerprint`). | S-11, S-38 | GL; `cargo tree -p contextra-infer-candle --no-default-features` ohne `candle-*` und ohne `rank` im Kern |
| **S-51** | `types` gibt ab: Erasure-Typen → `erasure`; `PprConfig`, `PprAlgorithm`, `TlHfdParams` → `graph`. | S-50, S-34 | GL, GE |
| **S-60** | Regler: `adapt` nimmt die fehlenden Methoden auf (decay 5, homeostat 3, pid 5 \[Ü\]); danach `pub use` in `engine` und `db`. | S-23 | GD (bitgleich) |
| **S-61** | Chunker: Paritätstest in `engine` (`ingest` als Dev-Abhängigkeit), dann Adapter über `ingest::chunking`. `contextra::chunker::MarkdownChunker` bleibt API-stabil. | S-01 | GD; `mcp`-Tests unverändert |
| **S-62** | `router`: `ports_local.rs` entfernen, nachdem die drei `mcp`-Importe auf `contextra_ports` umgestellt sind; `Vfs`-Stelle auf lokale `&Path`-Funktion. | S-01 | GL |
| **S-63** | `cognition` invertieren (Brücke in `db`, Feature `cognition`), danach `agent`. | S-37, S-50 | GL; `cargo tree -p contextra-cognition -e normal` ohne `contextra-engine` |
| **S-64** | `engine` (Lizenz → `license`, `extraction` → `exp-extraction`), `db`, `contextra` (Feature `license`; ohne Feature gilt `OpenFastGate`-Semantik, per Test belegt), `mcp`, `py`, `testkit`. | S-50, S-51, S-33 | GL; G-A1 ohne unbeabsichtigte Diffs |
| **S-70** | Aufräumen: `#[deprecated]`-Re-Exports (R9) und `*-compat`-Features entfernen; Fehlervarianten in `ContextraError` entfernen (nur bei grünem G-E1); `archive/`-Verschiebungen bestätigen. | alle | alle Gates |
| **S-70a** | Nachprüfung B23: Item-Diff `archive/contextra-crypto-proofs` ↔ `erasure` und Mutation-Score der Zieldateien in `erasure` ≥ Score der Quelldateien. Bei Nichterfüllung Wiederherstellung hinter dem Feature `legacy-erasure` (nicht Default). | S-00 | G-C1 |
| **S-80** | `contextra-vector-diskann` aus `vector/diskann/` herauslösen (Aufspaltung bestehenden Codes, D-09). | S-36 | GL, DiskANN-Tests (`required-features`) isoliert grün |
| **S-81** | `contextra-graph-research` aus den Klasse-A/B-Experimenten herauslösen. | S-34 | GL, GD |

### 9.3 Reihenfolge-Zwang (Zyklenfreiheit \[E\])

- `ids` und `env` stehen vor allem E1.
- `mvcc` steht vor `vector` und `store`; der `crypto`-Kern steht vor `store`.
- **S-50 steht vor S-37** (V9): Erst wenn Lizenz- und Erasure-Ports ihre Besitzer haben, verliert `store` die `FeatureRing`-Nutzung.
- Der `crypto`-KV-Umzug (S-38) folgt dem `crypto`-Kern und geschieht gemeinsam mit `kvcache`.
- S-12b (dev-Kanten) steht vor dem Entnehmbarkeitsnachweis (§8.4) jedes betroffenen Crates.
- S-22 (`vault` wird Workspace-Mitglied) steht vor jeder weiteren Änderung an `db`.
- `cognition` folgt erst der `engine`-Lizenzumstellung; neue Crates (`vector-diskann`, `graph-research`) folgen erst dem jeweiligen Kernschritt.

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
| D-13 | `async` ist in `erasure` nicht Default. Verbraucher aktivieren es explizit. | `async` zieht `tokio` und `futures-util`; das ist für einen autarken Baustein nicht wesensnotwendig (R10). | festgelegt, Verbraucher in S-24 zu ermitteln |
| D-14 | `contextra-vault` wird Workspace-Mitglied, `exclude` und `publish = false` entfallen. | `db` hängt davon ab; INV-4 muss es erfassen. | festgelegt, Begründung für `exclude` vorher klären (S-22) |
| D-15 | Diese Spezifikation enthält keinen Ist-Stand. Messwerte stehen in generierten Berichten (§8.3). | Die Spezifikation war beim Review 59 PRs hinter dem Repo. | festgelegt (v4) |
| D-16 | Es gibt keine In-Place-Migration zwischen 64 und 128 Bit. | Ein Migrationswerkzeug wäre ein eigenes Produkt; der Bestand lässt sich aus den Quellen neu aufbauen. | festgelegt (v4) |
| D-17 | `capabilities.toml` ist die einzige Quelle für Ebene, Unterebene und Ziel-Kernabhängigkeiten (`level`, `sublevel`, `core_deps`). | Der Score hat die Zielwerte hartkodiert \[V\] und doppelt damit §4.1. | festgelegt (v4), Umsetzung S-03 |
| D-18 | Tier `frozen`: `erasure`, `license`, `audit-export`, `avv-generator`, `sandbox`, `infer-onnx`, `infer-candle`. Tier `experimental`: `durable-fs`. | Das war in v3 und `AGENTS.md` D-08; `AGENTS.md` verweist künftig auf D-18. | festgelegt (aus v3) |
| D-19 | `no_std` ist kein Ziel. `ids` behält das Feature `std` samt Default. | Das Feature existiert mit Default \[V\]; ein Entfernen wäre ein Bruch für Verbraucher mit `default-features = false`. | festgelegt |
