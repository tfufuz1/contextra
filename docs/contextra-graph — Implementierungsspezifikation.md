# contextra-graph — Implementierungsspezifikation

17. Nov 2026 · Revision 1 (Entwurf zur Freigabe) · Grundlage: `docs/SPEC.md` v6 (16.11.2026) · Geprüfter Klon: HEAD `24487fa` · Bezug: SPEC-Schritte S-34, S-51, S-81

## 1 Zusammenfassung

`contextra-graph` soll nach SPEC v6 ein entnehmbarer E1a-Baustein sein: Kern-Closure `ids`, `env`, leerer Default, null Brückenreferenzen im Kern, eigener Fehlertyp, Persistenz mit Golden-Korpus und Breitenfeld, Experimente hinter `exp-*`. Der Crate ist davon weit entfernt. Die Prüfung hat außerdem Befunde ergeben, die in der SPEC nicht stehen. Die wichtigsten:

1. **Persistenzdefekt (kritisch).** `Entity.attributes` ist `AHashMap<String, serde_json::Value>`, die Persistenz nutzt bincode. Ein Entity mit mindestens einem Attribut lässt sich schreiben, aber nicht mehr lesen (bincode unterstützt `deserialize_any` nicht). `load_from_storage` bricht beim ersten solchen Eintrag den ganzen Ladevorgang ab. Die Serialisierung ist außerdem nicht deterministisch: Eine Sonde erzeugte aus 20 Instanzen eines identischen Entity 20 verschiedene Byte-Folgen. \[V, Sonde\]
2. **Der Crate baut nicht in allen Konfigurationen.** Fünf Features und `--all-features` scheitern, der Lib-Test-Build ist mit `workspace` rot. `--no-default-features` baut nur dank Feature-Vereinheitlichung durch `types` (`ahash/serde`). \[V\*\]
3. **Die Typidentität hängt am Feature `workspace`.** `types.rs` schaltet per `cfg` zwischen lokalen Kopien und `contextra_types`-Typen um. Features sind damit nicht additiv.
4. **S-51 ist wie geschrieben nicht umsetzbar.** `PprConfig`, `PprAlgorithm` und `TlHfdParams` nach `graph` zu verlagern, erzeugt einen Abhängigkeitszyklus (`types → graph → types`). Siehe D-G4.
5. **Die SPEC-Zahlen sind überholt.** Gemessen sind 46 Brücken-Treffer außerhalb `compat/` statt 140. `contextra-env` ist deklariert, aber ungenutzt.
6. **Der Klon ist nicht baubar.** Wurzel-`Cargo.toml`, `justfile`, `governance/` und `.github/` sind nicht im Repository. Die Gates aus §7 lassen sich damit nicht reproduzieren.

**Plan:** 15 Schritte `S-34.0` bis `S-34.14` in fünf Phasen. Erst bauen und messen, dann Golden-Korpus, dann Entkopplung, dann die Formatänderung getrennt von jedem Umbauschritt (SPEC §6.1).

## 2 Methode, Evidenz und Grenzen

- **Gelesen:** die ganze `docs/SPEC.md` (981 Zeilen), `CLAUDE.md`, die drei Reports in `reports/`, crate-lokale `SPEC.md`, `PLAN.md`, `docs/decoupling/*`, `Cargo.toml`, `lib.rs`, `types.rs`, `error.rs`, `storage.rs`, `graph_persist.rs`, `compat/`, dazu `ids` und die `Cargo.toml` aller Konsumenten.
- **Ausgeführt:** `cargo check` mit Toolchain 1.89.0 (apt) in einem Scratch-Workspace aus `ids`, `env`, `types`, `ports`, `graph`. Das Wurzel-Manifest ist rekonstruiert, die Versionen stammen aus `Cargo.lock`. Dazu eine Sonde für bincode. **Es wurde kein Test ausgeführt.**
- **Marken:** \[V\] gelesen oder ausgeführt · \[V\*\] mit rekonstruiertem Manifest gemessen, kann sich mit dem echten ändern · \[E\] Einschätzung · \[D\*\] Vorschlag dieser Spezifikation, braucht Freigabe oder ADR.
- **Nicht-Ziele:** keine neuen Funktionen (D-09), kein `no_std` (D-19), keine Leistungsänderung, keine Auslagerung von Experimenten vor S-81.

## 3 SOLL-Zustand (aus SPEC v6)

| Merkmal | SOLL | Quelle |
| --- | --- | --- |
| Ebene, Budget | E1a, Kern-Closure ⊆ E0, höchstens 4 Mitglieder | §2.2 |
| Kern direkt → Closure | `ids`, `env` → 2 | §4.1 |
| Features | `workspace`, Experimente als `exp-*`, Sammelfeature `experimental-all` | §4.1, R10 |
| Default | leer; `workspace`, `exp-*`, `*-compat` nie Default | R10, D-22 |
| Brücke | `src/compat/` hinter `workspace`; im Kern kein `contextra_types`, `contextra_ports`, `contextra_testkit`, auch nicht in Kommentaren und `cfg(test)` | R2, G-L2 |
| Fehler | `GraphError`, `GraphMutationError`, `#[non_exhaustive]`; `From<…> for ContextraError` nur in `compat/`; Display bitgleich | R3, §6.3 |
| IDs, Umwelt | `DocId`, `TxId` aus `ids`; Zeit und Zufall nur aus `env`; Differenztest | §5.1, §9.4, S-10C |
| Persistenz | Golden-Korpus; Breitentest 64/128 Bit; `id_width_bits`, wo sich Bytes unterscheiden; Formate eingefroren | §6.1, §6.2, INV-1 |
| Typbesitz | `PprConfig`, `PprAlgorithm`, `TlHfdParams` genau einmal, in `graph` | R4, S-51 |
| Experimente | Klasse A/B hinter `exp-*`; Klasse C (`hyperedge`, `path_rag`, `consistency_enforcement`) im Kern; ab 2.000 Zeilen `graph-research` | §9.4, X1–X8, S-81 |
| Dev-Kanten | keine auf `types`, `ports`, `testkit` | R12, D-12 |
| Hygiene | `forbid(unsafe_code)`; keine Panics in neuem Code; Tie-Break nach ID aufsteigend | INV-11, INV-12, INV-5 |
| Bausteinvertrag | README (höchstens 10 Zeilen, ohne weitere `contextra-*`), `examples/`, `capabilities.fragment.toml` mit `level`, `sublevel`, `core_deps` | §5.4, D-17 |
| Nachweis | M = 100; G-L2, G-L1, G-L3, G-L4, G-H1 in dieser Reihenfolge grün | §8.1, §8.4 |

**Pflichten aus Kapitel 11 bis 15 \[E, abgeleitet\].** Der Graph ist Datenklasse D (Ableitung mit Lineage, §11.2). Daraus folgt: (a) `source_doc_id` bleibt erhalten und über `purger` und `cascade` adressierbar (Löschschritt L2); (b) kein Lesepfad (Traversal, PPR, Path-RAG) liefert Daten eines gesperrten Ziels, und Rangfolgen ändern sich nicht durch unzugängliche Dokumente (Z-E1, Z-E8); (c) Logs und Fehlertexte tragen keine Roh-Kennungen (Z-E7); (d) Persistenz läuft über einen Port, die Verschlüsselung ist Aufgabe der Speicherschicht (Z-E2). Die SPEC weist `graph` dafür keinen Schritt zu. Das ist `S-34.14` \[D\*\].

## 4 IST-Zustand (verifiziert)

### 4.1 Repository und Umfeld

- Der Klon steht auf `24487fa` (3.100 Dateien), die SPEC beschreibt `fcd387021`. Nicht enthalten: Wurzel-`Cargo.toml` (Mitglieder, `[workspace.dependencies]`, Lints), `justfile`, `governance/`, `.github/`. \[V: `git ls-files`\]
- `contextra-graph`: 90 Dateien mit 23.595 Zeilen in `src/` (22 Dateien mit eingebetteten Testmodulen), 60 Integrationstests (10.228 Zeilen), 2 Benches, 1 Beispiel, `fuzz/`. `compat/` hat 13 Dateien mit 1.668 Zeilen. \[V\]
- Konsumenten: `engine` und `agent` (nur `CsrGraph`, über Defaults), `cognition` (`workspace`, leitet `physio-percolation` und `edge-reinforcement-learning` weiter), `db` (`workspace`, leitet `graph-connectivity-health` weiter, nutzt `percolation::*`, `detect_communities`, `path_rag`, Hyperedge-Typen). Kein Konsument nutzt `session_dag`; `mcp`, `py`, `bench` hängen nicht an `graph`. \[V: grep in `src/` und `tests/`\]

### 4.2 Build-Matrix (`cargo check`, Lib, 1.89.0) \[V\*\]

| Konfiguration | Ergebnis |
| --- | --- |
| `--no-default-features` | **rot**: 3× E0277 (`types.rs:118`, `124`): `AHashMap` ohne Feature `serde`. Mit `ahash/serde` grün (7 Warnungen) |
| Default | grün |
| `--features workspace` | grün, 109 Warnungen |
| `--all-features` | **rot**: 2× `GraphError: From<ApprhError>` nicht erfüllt |
| einzeln grün | `session-dag`, `entity-extraction`, `hyperedge-suggest`, `k-path-diffusion`, `edge-reinforcement-learning`, `docid-128`, `experimental-graph`, `ppr-forward-push` |
| einzeln **rot** | `apprh-diffusion` (wie `--all-features`); `graph-connectivity-health` und `physio-percolation` (E0432 `find_rebonding_candidates`); `test-support` (`deny(dead_code)`: `data`, `last_tx`); `types-compat` (E0432 `compat::provider` und vier Symbole); `ports-compat` (6 Fehler, u. a. `GraphIndexExt`, `compat::graph_index`) |
| Test-Targets mit `workspace` | **rot**: Lib-Test (3 Fehler: `basic_tests.rs:157`, `graph_index_tests.rs:221`, `227`), `ppr_candidate_stream.rs:273`, Bench `csr_traversal_bench.rs:59` (E0271). Benches melden zusätzlich `to_async` (criterion ohne `async_tokio`, hängt am Wurzel-Manifest). Die übrigen Test-Targets kompilieren |

### 4.3 Befunde

| ID | Befund | Beleg | Schritt |
| --- | --- | --- | --- |
| F-G01 | **Entity-Persistenz defekt:** nicht lesbar bei Attributen, nicht deterministisch. Kein Test deckt `attributes` ab. Derselbe Typ liegt in `types`. | `graph_persist.rs:43`, `106` (`?` bricht ab); `types.rs:124`; `types/domain/misc.rs:129`; Sonde \[V\] | S-34.2, S-34.10 |
| F-G02 | `--no-default-features` hängt an Feature-Vereinheitlichung (`ahash/serde` kommt von `types`). Isolationsbeweis G-L4 würde scheitern. | Matrix \[V\*\] | S-34.1 |
| F-G03 | Zwei Typuniversen: `types::local` ohne `workspace`, `contextra_types` mit. Betroffen: `EntityId`, `DocId`, `TxId`, `Edge`, `Entity`, `HyperEdgeId`, `RoleId`, `RoleBinding`, `PprConfig`, `PprAlgorithm`, `TlHfdParams`, `ResourceTracker`. | `types.rs` (Ende) \[V\] | S-34.3, S-34.12 |
| F-G04 | `docid-128 = []` leitet nicht an `ids` weiter; lokale `DocId(pub u64)` ist fest 64 Bit. `docid_128_path_rag.rs` prüft nur `cfg`. | `Cargo.toml:31`; `types.rs` \[V\] | S-34.3, S-34.7 |
| F-G05 | Aliasfeatures `types-compat`, `ports-compat` aktivieren Code, der an `workspace` hängt, nicht. | Matrix \[V\*\] | S-34.1 |
| F-G06 | `From<ApprhError> for GraphError` fehlt. | Matrix \[V\*\] | S-34.1, S-34.5 |
| F-G07 | `find_rebonding_candidates` ist in `lib.rs` ungegatet re-exportiert, die Funktion verlangt `contextra_ports::GraphIndex`. `db` und `cognition` leiten das Feature weiter. | `percolation.rs:89`; `lib.rs` \[V\] | S-34.1, S-34.6 |
| F-G08 | Self-Dev-Dependency mit `test-support` aktiviert den Feature-Stand in jedem Test-Build; `InMemoryStorageEngine` hat ohne `workspace` ungelesene Felder, `deny(dead_code)` bricht. | `Cargo.toml` dev-deps; FIX\_REPORT §4 \[V\] | S-34.1, S-34.8 |
| F-G09 | Tests erwarten `GraphError`, das Trait `GraphIndex` liefert `ContextraError`. Der FIX\_REPORT nennt die Korrektur, am Klon ist sie nicht wirksam. | Matrix \[V\*\] | S-34.1 |
| F-G10 | 46 Treffer für `contextra_(types\|ports\|testkit)` außerhalb `compat/` in 21 Dateien (SPEC: 140). Der Gating-Anteil ist nicht klassifiziert. `graph_persist.rs` ist komplett `workspace`-gated und nutzt `StorageEngine` direkt, obwohl die neutrale Naht `GraphStorage` (`storage.rs`) existiert. | grep \[V\] | S-34.6 |
| F-G11 | `contextra-env` ist Kernabhängigkeit ohne eine Verwendung; `contextra-ids` wird nur in `error.rs` genutzt. Quellcode ohne `Instant` und `SystemTime` außerhalb von Tests. | grep \[V\] | S-34.3, S-34.4 |
| F-G12 | Determinismus-Risiko: `std::HashMap` in `community.rs` (≥ 9 Stellen), `path_rag/*`, `ppr/snapshot.rs`, `hyperedge.rs`, `cascade.rs:200`; `AHashMap` in `provenance.rs`, `ppr/shadow_hook.rs`. Ob Ausgaben davon abhängen, ist ungeprüft. | grep \[V\], Wirkung \[E\] | S-34.9 |
| F-G13 | Formate ohne Kopf: Präfixe `__graph:entity:`, `__graph:edge:`, `__graph:community:`, `__graph:hyperedge:`, `__graph:hyperedge_by_entity:`, aber `graph:entity:deleted:` ohne `__`. bincode-Werte ohne Magic und Version, Legacy-Leser V1/V2 für Edge und HyperEdge. Golden-Korpus nur für IDs, PPR und Traversal, nicht für persistierte Records. | `csr/types.rs:19-25`; `hyperedge.rs:27`; `tests/golden/` \[V\] | S-34.2, S-34.10 |
| F-G14 | Dokumente widersprechen sich: crate-`SPEC.md` (verweist auf SPEC v2, nennt `contract.rs`, das nicht existiert, Default `workspace`, Reife `stable`) · `capabilities.fragment.toml` (`tier = "experimental"`, `dev_allowed.testkit`) · `PLAN.md` (Comment-Anchors statt S-34) · `docs/decoupling/CONTRACT.md` mit leeren Platzhaltern. | Dateien \[V\] | S-34.13 |
| F-G15 | Panics und unsafe: `unwrap()` 141×, `expect(` 16× in `src/` (Testmodule nicht getrennt gezählt); `panic!` und `todo!` 0; `tests/ppr_alloc_test.rs` enthält `unsafe` (D-21). | grep \[V\] | S-34.8 |

## 5 Entscheidungen (Vorschläge, je ein ADR)

| ID | Frage | Empfehlung \[D\*\] |
| --- | --- | --- |
| D-G1 | Wie wird `Entity.attributes` persistiert, ohne die öffentliche Struktur zu ändern? | Im Speicher bleibt der Typ. Persistiert wird ein kanonisches JSON (Schlüssel sortiert) als Bytes-Feld im Format v2. Später optional ein geschlossenes `AttributeValue`-Enum in `types`. |
| D-G2 | Format v2 und Umschaltung des Schreibers | Umschlag mit Magic, Version, `id_width_bits`. Zwei Leser (v2 und Legacy). Der Schreiber bleibt `Legacy`, bis Golden und Breitentest grün sind. Eigener ADR, nie Teil eines Umbauschritts (§6.1). |
| D-G3 | `contextra-env`: entfernen oder nutzen? | Entfernen, solange es keine Zeit- oder Zufallsquelle gibt (F-G11). §4.1 und `core_deps` per ADR anpassen. Aufnahme, sobald S-34.9 oder S-34.14 eine Uhr braucht. |
| D-G4 | Eigentümer von `PprConfig`, `PprAlgorithm`, `TlHfdParams` | S-51 führt zum Zyklus `types → graph → types` \[E, begründet: Cargo erkennt Zyklen auch über optionale Kanten\]. Empfohlen: lokale Kopie im Graph-Kern bleibt, Konformitätstest (Serde-Namen, Default-Werte, `type_parity.rs`, `types_diff.rs`), R4-Ausnahme per ADR erweitern. Alternative: neues E0-Blattcrate für Wert-Typen. |
| D-G5 | Klassifikation der Module | Klasse C im Kern, Klasse A/B hinter `exp-*` nach §6.1. Aliasnamen bis S-70. |
| D-G6 | Wo liegen Tests gegen `ContextraError`? | In `testkit/tests/graph/` oder hinter `required-features = ["workspace"]` (R12). Kerntests prüfen `GraphError`. |
| D-G7 | Umgang mit bereits geschriebenen Entities mit Attributen | Salvage-Leser: Kopf `id`, `name`, `entity_type` lesen, Attribute verwerfen, Zähler im Ladebericht. Ein Abbruch des Ladens ist keine Option. Voraussetzung: Test bestätigt, dass bincode den Kopf trotz Resten liest \[E\]. |

## 6 Zieldesign

### 6.1 Modulzuordnung \[D\*, D-G5\]

| Gruppe | Module | Zeilen (ohne Tests) |
| --- | --- | --- |
| Kern (Klasse C und Infrastruktur) | `csr/*`, `ppr/` (`forward_push`, `dense`, `cost`, `snapshot`), `community`, `path_rag` (ohne `k_path`), `hyperedge`, `consistency_enforcement`, `cascade`, `provenance`, `purger`, `entity_extraction`, `hyperedge_suggest`, `arc_slice`, `storage`, `graph_view`, `text_gen`, `types`, `error` | Rest |
| Experiment, Kandidat für `graph-research` | `apprh/` (1.165), `tl_hfd/` (922), `edge_reinforcement*` (404), `percolation` (343), `path_rag/k_path` (336), `ppr/shadow_hook` (122) | ≈ 3.300, also ≥ 2.000 nach X4 |
| Experiment, bleibt im Crate | `session_dag` (1.016) als `exp-session-dag` | 1.016 |

`PprAlgorithm` trägt Varianten `TlHfd` und `ShadowModeTlHfd` im Kern-Enum. Das verletzt X1 und X3 und wird in S-34.11 durch eine Naht `ext.rs` entkoppelt. Ohne das Feature fällt der Algorithmus dokumentiert auf `ForwardPush` zurück.

### 6.2 Ziel-`Cargo.toml` (Auszug)

```toml
[features]
default = []
workspace = ["dep:contextra-types", "dep:contextra-ports"]
docid-128 = ["contextra-ids/docid-128"]
test-support = []                      # nur Fakes ohne ports
exp-apprh = []
exp-k-path = []
exp-percolation = []
exp-edge-reinforcement = []
exp-session-dag = []
experimental-all = ["exp-apprh", "exp-k-path", "exp-percolation", "exp-edge-reinforcement", "exp-session-dag"]
# Aliase bis S-70 (R10)
types-compat = ["workspace"]
ports-compat = ["workspace"]
apprh-diffusion = ["exp-apprh"]
k-path-diffusion = ["exp-k-path"]
graph-connectivity-health = ["exp-percolation"]
physio-percolation = ["exp-percolation"]
edge-reinforcement-learning = ["exp-edge-reinforcement"]
session-dag = ["exp-session-dag"]
experimental-graph = ["exp-session-dag"]
entity-extraction = []                 # ohne cfg, bleibt als No-op-Alias bis S-70
hyperedge-suggest = []
ppr-forward-push = []

[dependencies]
ahash = { workspace = true, features = ["serde"] }   # F-G02
contextra-ids = { workspace = true }
# contextra-env: nach D-G3 vorerst entfernt
```

Die Self-Dev-Dependency auf `test-support` entfällt. Tests, die Fakes brauchen, deklarieren `[[test]] required-features`.

### 6.3 Ports im Kern

`GraphStorage` (Persistenz), `GraphView` (Lesen: Nachbarn, PPR) und `TextGenerator` (LLM-Extraktion) existieren bereits und sind neutral. Die Zielarchitektur macht sie zur einzigen Naht des Kerns:

- Persistenz (`persist_*`, `load_from_storage`) wird generisch über `GraphStorage` und verlässt `graph_persist.rs`; `compat/storage_bridge.rs` implementiert `GraphStorage` für jedes `contextra_ports::StorageEngine`.
- `ppr_stream`, `percolation` und `hyperedge_suggest` arbeiten auf `GraphView` und `TextGenerator`. Die Konstante `MAX_SEARCH_K` kommt aus `ids`.
- `session_dag` nutzt `GraphStorage` statt `StorageEngine`.

### 6.4 IDs und Typen

`types.rs` re-exportiert `DocId`, `EntityId`, `TxId` aus `contextra_ids` in **beiden** Konfigurationen. Die lokalen Kopien entfallen. Geprüfte Eigenschaften:

- Der Hash ist identisch: BLAKE3, erste 8 Byte, Little Endian (`types.rs` und `ids` gelesen). Die Golden-Dateien `tests/golden/doc_id.*` und `entity_id.*` müssen unverändert bleiben.
- `from_key("")` liefert lokal `ParseIntError`, in `ids` `IdError::EmptyKey`. `GraphError` hat beide `From`-Impls. Steht der alte Text in einem Test, bildet `GraphError::from(IdError)` ihn bitgleich ab (Display-Test nach §6.3).
- Lokal leiten `EntityId`, `DocId`, `TxId` `Default` ab, `ids::DocId` und `ids::EntityId` nicht. Aufrufstellen mit `Default` werden per grep erhoben und angepasst.
- Mit `docid-128` ist `ids::DocId` ein `u128`. Code, der `.0` oder `inner()` als `u64` nutzt, wird auf einen opaken Umgang umgestellt. `EntityId::from_doc_id` schneidet in diesem Modus ab: Kollisionsrisiko dokumentieren.
- `TxId::INVALID` und `INTERNAL_BASE` haben in `ids` dieselben Werte wie lokal.
- `Entity`, `Edge` bleiben in `types` Eigentümer-Typen für den Port `GraphIndex`; der Graph-Kern hält eine lokale Kopie mit `From` in `compat/` und Paritätstest (D-G4).

### 6.5 Persistenzformat v2 \[D-G1, D-G2\]

- **Schlüssel unverändert** (N9): alle Präfixe aus F-G13 bleiben, auch `graph:entity:deleted:`.
- **Umschlag:** `magic` 4 Byte (`CGR1`), `kind` u8 (Entity, Edge, HyperEdge, Community), `version` u16, `id_width_bits` u16, danach die Nutzlast. Entity-Attribute als kanonisches JSON (Schlüssel sortiert) in einem Bytes-Feld.
- **Breitenfeld:** Edge- und HyperEdge-Nutzlast tragen `DocId` und sind deshalb voraussichtlich breitenabhängig \[E\]; Entity und Community nicht. Entscheidend ist der Test aus §6.2: Korpus je einmal mit 64 und 128 Bit erzeugen, Bytes vergleichen. Abgelehnte Dateien bleiben byte-identisch.
- **Leser, in dieser Reihenfolge:** v2-Umschlag → Legacy V3 (aktuell) → V2 → V1 (nur Edge und HyperEdge) → Entity-Salvage (D-G7) mit `LoadReport { salvaged_entities, dropped_attributes }`.
- **Schreiber:** `PersistFormat::{Legacy, V2}`, Default `Legacy`. Umstellung nur per ADR nach grünem Rückwärts-Lesetest.

## 7 Umsetzungsplan

Die Schritte folgen §9.1 (Definition of Done) und INV-3: je Schritt ein Commit-Paar `S-34.x A: …` (verschieben, umbenennen) und `S-34.x B: …` (umbauen). Konsumenten (`engine`, `agent`, `cognition`, `db`) werden im selben B-Commit angepasst. Aufwand: S bis 1 Tag, M bis 3 Tage, L bis 1 Woche \[E\].

| Phase | Schritte | Ziel |
| --- | --- | --- |
| P0 | S-34.0, S-34.1, S-34.2 | Messbar, baubar, Golden vor jeder Änderung |
| P1 | S-34.3, S-34.4, S-34.5 | Kerne: IDs, Umwelt, Fehler |
| P2 | S-34.6, S-34.7, S-34.8 | Brücke null, Features, Tests |
| P3 | S-34.9, S-34.10 | Determinismus, Format v2 |
| P4 | S-34.11 bis S-34.14 | Experimente, Typbesitz, Bausteinvertrag, Klasse D |

### S-34.0 Baseline und Reproduzierbarkeit (S)

- **Änderungen:** Wurzel-`Cargo.toml`, `justfile`, `governance/`, `.github/` vom Eigentümer beschaffen \[O-1\]. Danach `docs/decoupling/BASELINE.md` neu erzeugen: Build-Matrix (§4.2), Testanzahl je Target, Brückenzählung streng, Closure per `cargo tree`. criterion-Feature `async_tokio` im Wurzel-Manifest prüfen.
- **Abnahme:** Bericht mit HEAD-Hash und Datum; §4.2 mit dem echten Manifest wiederholt, Abweichungen als Befund.
- **Voraussetzung:** keine.

### S-34.1 Sofortreparaturen ohne Formatwirkung (M)

- `ahash` mit `features = ["serde"]` (F-G02).
- `From<ApprhError> for GraphError` samt Display-Test (F-G06); gleiches für `TlHfdError`, falls es fehlt.
- `find_rebonding_candidates` aus dem Gate von `lib.rs` nehmen oder auf `GraphView` umstellen (F-G07).
- `types-compat` und `ports-compat` als Alias für `workspace` (F-G05).
- `test_doubles` nur mit `workspace` kompilieren; Self-Dev-Dependency entfernen; `deny(dead_code)` auf `warn` oder gezieltes `allow` mit Begründung (F-G08).
- Die vier Testfehler beheben (F-G09): Tests, die `GraphIndex` nutzen, erwarten `ContextraError`, Kerntests rufen die inhärenten Methoden und erwarten `GraphError` (D-G6).
- **Abnahme:** `cargo check -p contextra-graph --all-targets` grün in den drei Konfigurationen aus §4.2; `cargo hack check -p contextra-graph --each-feature --no-dev-deps` grün; Testanzahl dokumentiert (Baseline für alle Folgeschritte).
- **Voraussetzung:** S-34.0.

### S-34.2 Golden-Korpus für Persistenz und Charakterisierung (M)

- Ergänzt `tests/golden/persist/` (mit `MANIFEST.sha256`, Anbindung an den Wurzel-Korpus aus S-02) um: Entity ohne Attribute, Edge V3, V2, V1, HyperEdge aktuell und `LegacyHyperEdgeV1`, Community-Zuordnung, Gelöscht-Marker, Schlüsselaufbau. Je einmal in 64 und 128 Bit.
- Charakterisierungstests für F-G01: Entity mit Attribut (Laden schlägt fehl, `#[ignore]` mit Verweis auf S-34.10) und Determinismus-Test, der heute rot ist.
- **Abnahme:** G-F1 am unveränderten Stand grün; die zwei Defekttests sind dokumentiert und ignoriert.
- **Voraussetzung:** S-34.1, S-02.

### S-34.3 IDs aus `ids` (M)

- Commit A: Differenztest `tests/differential_ids_graph.rs` gegen die lokale Kopie, erweitert `ids/tests/differential_ids.rs` (SPEC F-05). 10.000 zufällige Schlüssel plus leer, Unicode, 4 KiB; Bytes, JSON und bincode gleich.
- Commit B: `types.rs` re-exportiert aus `ids` (§6.4); `docid-128` leitet weiter (F-G04). D-20 ist für `graph` erledigt, weil die Werte identisch sind und nur der Fehlertyp abweicht \[V\].
- **Abnahme:** GD, GF (Golden aus S-34.2 unverändert), GE; `cargo check -p contextra-graph --features docid-128` grün, auch mit `workspace`.
- **Voraussetzung:** S-34.2, S-10C.

### S-34.4 Umwelt (`env`) (S)

- D-G3: `contextra-env` entfernen, solange kein Verbraucher existiert (ADR, §4.1 und `core_deps` anpassen). Sonst Zeit und Zufall nur über `Clock` und `Rng` aus `env` einführen.
- **Abnahme:** G-H1, G-L4 (Closure nur `ids`).
- **Voraussetzung:** S-34.3, S-11.

### S-34.5 Fehler (S)

- `docs/decoupling/errors/graph.md` vervollständigen: jede lokale Variante ↔ `ContextraError`-Variante ↔ Display (§6.3). Tabellengetriebene Tests in `compat/error.rs`. Die neun `ContextraError`-Erwähnungen außerhalb `compat/` entfernen. Keine Sammelvariante mit verworfenem Kontext (`Internal(String)` bleibt zulässig, wenn die Quelle genannt ist).
- **Abnahme:** G-E1.
- **Voraussetzung:** S-34.1.

### S-34.6 Brücke null (L)

- Commit A verschiebt: `graph_persist.rs`, die `workspace`-Blocke in `types.rs`, `ppr_stream.rs`, `test_doubles.rs` und die Brückenteile von `csr/graph_write/tombstone.rs` nach `compat/`.
- Commit B baut um: Persistenz generisch über `GraphStorage` (§6.3); `percolation`, `ppr_stream`, `hyperedge_suggest`, `session_dag` auf lokale Ports; Testmodule in `csr/tests/*`, `ppr/tests/*`, `community/tests.rs`, `cascade.rs` auf lokale Fakes oder nach `tests/` hinter `required-features`. Doc-Kommentare mit `contextra_types` umformulieren.
- **Abnahme:** `tools/gate_bridge_null.py` streng gezählt 0 (Kommentare eingeschlossen); G-L2, G-L1, G-L4; Testanzahl gleich; Persistenz-Golden unverändert.
- **Voraussetzung:** S-34.2, S-34.3, S-34.5. Risiko: hoch (INV-6, Kaskade und Persistenz).

### S-34.7 Features und Defaults (M)

- Ziel-`Cargo.toml` aus §6.2 mit Aliasen. `default` leer (D-22): Kein Konsument braucht `session-dag`, `entity-extraction` oder `hyperedge-suggest` \[V\]; `engine` und `agent` brauchen `CsrGraph` mit `workspace`, `db` und `cognition` die `exp-percolation` und `exp-edge-reinforcement` (Weiterleitung auf die neuen Namen).
- Feature-Closure je Feature geprüft, Kantengraph azyklisch (R13).
- **Abnahme:** G-H1, G-L3 (`--each-feature`), `cargo check --workspace` mit den vier Konsumenten.
- **Voraussetzung:** S-34.6, S-12a. Risiko: `engine` und `agent` bauen heute nur durch Vereinheitlichung mit `db`.

### S-34.8 Tests und Hygiene (M)

- `capabilities.fragment.toml`: `dev_allowed.testkit` entfernen. Tests mit `types` oder `ports` bekommen `[[test]] required-features = ["workspace"]` oder wandern nach `testkit/tests/graph/` (R12, D-12). Das betrifft rund 40 der 60 Test-Dateien \[V: grep\].
- INV-12: `unwrap()` und `expect(` in Nicht-Test-Code zählen und abbauen; Gate `unwrap-gate` nur gegen Neucode.
- INV-11 und D-21: `ppr_alloc_test.rs` mit lückenlosem `// SAFETY:` oder Messung ohne `unsafe`.
- **Abnahme:** G-H1 (keine dev-Kante), Testanzahl vorher gleich nachher.
- **Voraussetzung:** S-34.6.

### S-34.9 Determinismus (M)

- Prüfen und beheben, wo Ausgaben von der Iterationsreihenfolge abhängen (F-G12): `BTreeMap`, sortierte `Vec` oder `ahash::RandomState::with_seeds`. Tie-Break nach `EntityId`, dann Ziel-ID aufsteigend (INV-5).
- Tests: Permutationsinvarianz (proptest) für `community`, `path_rag`, `cascade`; Lauf-zu-Lauf-Golden `ppr_results.json`, `traversal_results.json` unverändert.
- **Abnahme:** G-D1 (`tools/determinism_guard.py`), GF.
- **Voraussetzung:** S-34.2.

### S-34.10 Persistenzformat v2 und Defektbehebung (L)

- Eigener Zweig und eigener ADR (D-G1, D-G2, D-G7). Commit A: Leser v2, Legacy und Salvage mit Rückwärts-Lesetest über alle Golden-Dateien aus S-34.2. Commit B: Schreiber `V2` hinter `PersistFormat`, Default bleibt `Legacy`.
- `types::Entity` wird mit angefasst, deshalb Abstimmung mit dem Eigentümer von `types` (S-51).
- Die zwei ignorierten Tests aus S-34.2 werden grün.
- **Abnahme:** G-F1 (Alt-Dateien lesbar), Breitentest nach §6.2, Crash- und Recovery-Suite unverändert grün (INV-6), `LoadReport` im Test mit Salvage-Fall.
- **Voraussetzung:** S-34.3, S-34.6, S-34.9. Risiko: hoch (INV-1-Ausnahme, in Abstimmung mit dem Betrieb, ob Bestand mit Attributen existiert, O-2).

### S-34.11 Experimente vorbereiten (M)

- Umbenennung in `exp-*` (§6.2), Naht `ext.rs` für `PprAlgorithm`-Strategien, `capabilities.toml`-Tier `experimental` je Experiment (X2, X3, X7). Baseline im selben Crate je Experiment (X5, z. B. Recall gegen Forward-Push).
- **Abnahme:** G-H1, `cargo hack check --each-feature`, GD.
- **Voraussetzung:** S-34.7. S-81 (Auslagerung nach `contextra-graph-research`) folgt danach.

### S-34.12 Typbesitz (S-51) (M)

- Nach D-G4: Konformitätstest zwischen `graph::{PprConfig, PprAlgorithm, TlHfdParams}` und `types::{…}` (Serde-Namen, Defaults, Varianten), ADR zur R4-Ausnahme, `types` behält seine Fassung mit `#[deprecated]`-Hinweis auf die kanonische Fassung. Kein Zyklus.
- **Abnahme:** `symbol-uniqueness` mit eingetragener Ausnahme, G-L1 ohne Zyklus, GE.
- **Voraussetzung:** S-34.3, ADR D-G4.

### S-34.13 Bausteinvertrag und Entnehmbarkeit (M)

- `README.md` mit Beispiel von höchstens 10 Zeilen (Ausgangspunkt: `examples/isolated_graph_traversal.rs`), `capabilities.fragment.toml` (`level`, `sublevel = "E1a"`, `core_deps`, `tier` abgestimmt mit D-18), crate-`SPEC.md` auf SPEC v6 umschreiben (kein `contract.rs`, richtige Defaults, Reife passend zum Tier), `PLAN.md` durch `PLAN_S-34.md` ersetzen, `docs/decoupling/REPORT.md` mit HEAD-Hash, `version` an der Pfad-Abhängigkeit (F-12), `cargo package -p contextra-graph --locked` als Probe.
- **Abnahme:** §8.4 vollständig; G-DOC1; Feld `extractable` setzt nur dieser Schritt.
- **Voraussetzung:** S-34.6 bis S-34.10.

### S-34.14 Vertrag für Datenklasse D \[D\*, abhängig von S-103\]

- Lineage: `source_doc_id` und `source_doc_ids` sind Pflichtfelder jeder Ableitung; `purger` und `cascade` bekommen eine Löschoperation je `DocId` mit Bericht (nur Zählwerte).
- Lesesperre: lokaler Port `ReadGate` (`is_blocked(DocId) -> bool`, O(1), ohne I/O). Traversal, PPR und Path-RAG laufen auf der sichtbaren Teilmenge, sodass sich Rang und Zählwerte durch gesperrte Dokumente nicht ändern (Z-E8). Gesperrtes liefert dasselbe Fehlerbild wie Nicht-Vorhandenes.
- Logs: Audit aller `tracing`-Felder und Fehlertexte auf Entity-Namen und Roh-IDs (Z-E7), Canary-Test.
- **Abnahme:** Tests je Lesepfad (Z-E1), Rangtest mit gesperrtem Dokument (Z-E8), Canary-Suche (Z-E7).
- **Voraussetzung:** S-34.6, S-34.10, S-103.

## 8 Gates und Abnahme

| Schritt | Pflicht-Gates |
| --- | --- |
| S-34.0, .1 | G-L3 |
| S-34.2 | G-F1 |
| S-34.3 | G-D1, G-F1, G-E1, G-L1 bis G-L4 |
| S-34.4, .5 | G-H1, G-L4, G-E1 |
| S-34.6 | G-L2, G-L1, G-L4, G-F1, G-C1 (TSan, Loom) |
| S-34.7, .8 | G-H1, G-L3 |
| S-34.9 | G-D1 |
| S-34.10 | G-F1, G-C1, Crash-Suite |
| S-34.11, .12 | G-H1, G-L1 |
| S-34.13 | alle, dazu G-DOC1 |

Jeder Schritt erfüllt zusätzlich §9.1: Commit-Paar, `cargo check --workspace --all-targets --locked` grün, Score M und Brückenzählung neu erzeugt, crate-lokaler Status aktualisiert, Revert-Commit baut. Ohne das fehlende Wurzel-Manifest lässt sich das nicht prüfen (O-1).

**Entnehmbar** ist `contextra-graph`, wenn M = 100, die Gates G-L2, G-L1, G-L3, G-L4 und G-H1 in dieser Reihenfolge grün sind, das README-Beispiel ohne weitere `contextra-*`-Crates kompiliert, Golden-Korpus und Breitentest grün sind und `cargo package` gelingt.

## 9 Risiken und Rückfall

| Risiko | Wirkung | Gegenmaßnahme |
| --- | --- | --- |
| Fehlendes Wurzel-Manifest, §4.2 ist \[V\*\] | Zahlen können abweichen | S-34.0 zuerst, Matrix wiederholen |
| Formatänderung bricht Bestand | Datenverlust beim Laden | Zwei Leser, Schreiber `Legacy` bis ADR, Hash-Prüfung abgelehnter Dateien |
| Bestand mit Attributen existiert bereits | Attribute nicht rekonstruierbar (bincode kodiert keine Typ-Tags) | Salvage mit Ladebericht (D-G7), Abstimmung mit Betrieb |
| Entfernen der Defaults | `engine` und `agent` bauen nicht mehr | Konsumenten im selben B-Commit, `cargo check --workspace` |
| `docid-128` | `DocId` ist `u128`, `EntityId` bleibt `u64` | Opake Nutzung, Differenztest, Kollisionshinweis |
| Zyklus bei S-51 | Workspace baut nicht | D-G4, kein `graph`-Eintrag in `types` |
| Determinismus-Änderung verändert Rangfolgen | Golden bricht | Nur Reihenfolge-abhängige Stellen ändern, Golden-Abweichung ist Befund (§9.5 Punkt 3) |

Rückfall nach §9.5: Gate rot nach Commit B und nicht binnen eines Arbeitstags behebbar → `git revert` von B, Commit A bleibt, Befund ins Register. Ein roter Differenz- oder Golden-Test bricht den Schritt ab, die Toleranz wird nicht angepasst.

## 10 Offene Punkte

- **O-1:** Wurzel-`Cargo.toml`, `justfile`, `governance/` und `.github/` bereitstellen. Ohne sie sind §4.2 und alle Gates nur näherungsweise prüfbar.
- **O-2:** Gibt es produktive Bestände mit `Entity.attributes`? Davon hängt D-G7 ab.
- **O-3:** ADRs D-G1 bis D-G7 freigeben, vorrangig D-G4 (blockiert S-51) und D-G2.
- **O-4:** Soll die Aussage „5 Features rot“ als SPEC-Befund (Anhang A, F-21 ff.) übernommen werden? Vorschlag: F-G01 bis F-G15 als F-21 bis F-35 ins Register.
- **O-5:** Test-Baseline: Nach S-34.1 einmal `cargo test -p contextra-graph --features workspace` mit Ergebnisliste festhalten. Bis dahin ist keine Aussage über den Grünstatus der Tests belegt.
