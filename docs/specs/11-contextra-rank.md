# `contextra-rank` – Implementierungsspezifikation

|Feld|Wert|
|---|---|
|Zweck-Satz (D-97)|Reciprocal Rank Fusion für die Hybridsuche.|
|Ring|2|
|Erlaubte Workspace-Abhängigkeiten|`types`, `ports` (SPEC 5.3); Verbraucher: `engine`|
|Stufe / Gates|S3; G-G (Hybridsuche), G-C (Panic-Freiheit)|
|Arbeitspakete|WP-8.2 (RRF, Hybridsuche), WP-1.1/1.2 (Rückbau)|
|SPEC-Anker|8.11, 5.2, D-26, D-62, D-70, 7.2 (`search_hybrid`), T-RANK-1, T-HYB-1, F-31, F-36|

## 1. Ist-Zustand (HEAD `0ede89a26`) [V]

|Punkt|Befund|
|---|---|
|Stand gegenüber Index|Der Index (`00-INDEX.md`) nennt 3.440 Zeilen (HEAD `1454bbec0`). Seit Commit `34c2f640f` („reduce contextra-rank to deterministic RRF fusion“) sind es **2.370 Zeilen** `src`; `dibud`, `drift` und `fusion.rs` (2.252) sind weg. Die Abweichung ist in Index Abschnitt 4 nachzuziehen.|
|Dateien|`fusion/rrf.rs` (690), `explain.rs` (469), `fusion/provenance.rs` (308), `fusion/types.rs` (268), `fusion/normalized.rs` (200), `fusion/signal.rs` (164), `fusion/topk.rs` (100), `fusion/resonance.rs` (98), `lib.rs` (38), `fusion/mod.rs` (35)|
|Kanonischer Einstieg|`rrf_fuse(lists: &[Vec<Hit>], k: u32, top: usize) -> Vec<Hit>` (`rrf.rs:30`), `DEFAULT_RRF_K = 60`; Tie-Break: Score absteigend, dann `doc_tag` aufsteigend (falls vorhanden), dann `id`|
|Rechenweise|Summe von `1.0_f32 / (k + rank)` in **Eingabereihenfolge** (`f32`-Addition); Sortierung über `total_cmp`. Die Doku behauptet Permutationsinvarianz der Listenreihenfolge; bei drei oder mehr Listen ist `f32`-Addition **nicht assoziativ**, der Score kann von der Listenreihenfolge abhängen. `k == 0` wird **still** durch 60 ersetzt.|
|Identität|`Hit { id: String, score: f32, doc_tag: Option<[u8; 16]>, metadata: Option<serde_json::Value> }`; Zusammenführung über den **String-`id`** (`AHashMap<String, usize>`). Soll: `DocHandle` (D-70).|
|Außerhalb Soll (D-26)|Wrapper `reciprocal_rank_fusion`, `weighted_reciprocal_rank_fusion*` (mehrere Varianten, `mrrf`), `fuse_search_results_with_strategy`, `fuse_search_results_with_signal_strategies`, `score_normalized_fusion_with_options`, `apply_resonance_bonus`/`ResonanceConfig`, `modulate_and_renormalize_weights`, `weights_to_signal_factors`, `SignalKind`, `SignalFailurePolicy`, `ProvenanceBuilder`/`ProvenanceRecord`, `SearchReport`, `explain`/`RetrievalExplanation`, `g(x)`; sie bleiben mit `#[allow(deprecated)]` „für Abwärtskompatibilität zu `contextra-engine`“ exportiert. `engine/src/fusion.rs` (59 Z.) reicht sie durch.|
|Manifest|Abhängigkeiten `types`, `ports`, `serde`, `serde_json`, `ahash`, `tracing`, `thiserror`; Metadatum `ring = "0"` (Soll 2); `lib.rs` nennt „Ring 2“, „Klasse Rand“|
|Tests|4 Testdateien: `rank_tests.rs`, `rrf_property_tests.rs` (3 `proptest`-Stellen), `rrf_reference.rs`, `nan_inf_safety_tests.rs`|

## 2. Verantwortung und Abgrenzung

`rank` enthält **nur** die Reciprocal Rank Fusion (D-26): aus zwei Kandidatenlisten (Text, Vektor) entsteht eine deterministische, fusionierte Rangliste. Es kennt weder Index noch Gate, weder Schlüssel noch Dokumentinhalt. Parameter: `k_rrf = 60`; Tie-Break nach `doc_tag` (8.11).

**Grundsatz zur Löschung (P4, P12):** `rank` erhält ausschließlich Kandidaten, die das `ReadGate` und den nachgelagerten Filter bereits passiert haben (D-62). Ein gegateter Treffer darf **weder** in der Fusion erscheinen **noch** die Ränge der übrigen Treffer verändern; sonst beeinflusst ein gelöschtes Dokument das Ergebnis über seine Position (Seitenkanal). Das Nachladen („bis `k` gültige Treffer oder Erschöpfung“) steuert `engine`, nicht `rank`.

## 3. Zielstruktur (Module) [E]

|Modul|Inhalt|
|---|---|
|`lib.rs`|Crate-Kopf (Q-01), Re-Exporte von `rrf_fuse`, `Candidate`, `Fused`, `K_RRF`|
|`rrf`|`rrf_fuse`, Festkomma-Beitrag, deterministische Sortierung|
|`bound`|`unseen_bound` (obere Schranke für noch nicht gesehene Dokumente) für die Abbruchregel des Nachladens (Abschnitt 4)|
|`error`|`RankError` (fester Text), nur für Eingabeverletzungen|

Genau ein Funktionsgebiet; keine Unterverzeichnisse nötig. Entfallen: `explain.rs`, `fusion/provenance.rs`, `fusion/types.rs` (soweit nicht `Candidate`), `fusion/normalized.rs`, `fusion/signal.rs`, `fusion/topk.rs` (ersetzt durch `select_nth`/Sortierung), `fusion/resonance.rs`, alle gewichteten Varianten und Wrapper.

## 4. Schnittstellen [E]

```rust
pub const K_RRF: u32 = 60;                         // 8.11, Anhang C

/// Ein Kandidat einer bereits gefilterten Liste; Rang = Position (1-basiert), nicht der Index-Score
pub struct Candidate { pub handle: DocHandle }      // DocHandle { cluster_tag, doc_tag } aus types (D-70)

pub struct Fused { pub handle: DocHandle, pub score: RrfScore }   // RrfScore: Festkomma-Ganzzahl, Ord

/// Fusion beliebig vieler Listen (v1: zwei). Reihenfolge der Listen ist ohne Einfluss (bitgleich).
pub fn rrf_fuse(lists: &[&[Candidate]], top: usize) -> Vec<Fused>;

/// Obere Schranke des Scores eines Dokuments, das in keiner Liste bis zu den Tiefen `depths[i]` auftrat
pub fn unseen_bound(depths: &[usize]) -> RrfScore;       // Σ_i 1 / (K_RRF + depths[i] + 1)
```

`RrfScore` ist eine **Festkomma-Ganzzahl** (`u64`, Beitrag je Liste `floor(2⁴⁰ / (K_RRF + rank))`); Addition von Ganzzahlen ist assoziativ und kommutativ, daher ist das Ergebnis bit-genau unabhängig von der Listenreihenfolge. Es gibt keinen Gleitkommawert in der Fusion und keinen Parameter `k` (fest 60). Sortierung: Score absteigend, dann `doc_tag` aufsteigend, dann `cluster_tag` aufsteigend (vollständige Ordnung, kein `Equal`).

**Abbruchregel des Nachladens [E] (Entwurf für `engine`, D-62):** `engine` liest aus beiden Strömen die ersten `n_t` und `n_v` gültigen Kandidaten und bildet `rrf_fuse`. Das fusionierte Top-`k` ist **endgültig**, sobald der `k`-te Score ≥ `unseen_bound([n_t, n_v])` ist (ein ungesehenes Dokument kann diese Schranke nicht überschreiten). Sonst werden `n_t` und `n_v` verdoppelt (Vektor: `ef` verdoppeln bis 4096). Sind beide Ströme erschöpft, ist das Ergebnis endgültig und `exhausted = true`, falls weniger als `k` Treffer vorliegen.

## 5. Implementierungsvorgaben

|ID|Vorgabe|Quelle|Test|
|---|---|---|---|
|RK-01|`rank` enthält nur RRF mit `k_rrf = 60`. Keine Gewichtung, keine Score-Normalisierung, keine Signalpolitik, keine Resonanz, keine Provenienz, keine Erklärungstexte, keine Wrapper „für Abwärtskompatibilität“.|D-26, P3, P8|T-RANK-1, Grep|
|RK-02|Der Beitrag je Liste ist eine **Festkomma-Ganzzahl** (`floor(2⁴⁰ / (60 + rank))`); keine `f32`/`f64`-Rechnung in der Fusion. Dadurch ist das Ergebnis bit-genau unabhängig von Listenreihenfolge und Summationsreihenfolge (Permutationsinvarianz, auch bei ≥ 3 Listen).|8.11, T-RANK-1|T-RANK-1 (Property: beliebige Permutation ⇒ identische Ausgabe)|
|RK-03|Deterministischer Tie-Break nach `doc_tag` aufsteigend, danach `cluster_tag` aufsteigend; die Ordnung ist total. Gleicher `(cluster_tag, doc_tag)` in mehreren Listen ⇒ ein Eintrag mit summiertem Score.|8.11, D-70|T-RANK-1|
|RK-04|Identität ist `DocHandle` aus `types`; kein `String`-Schlüssel, keine Metadaten, kein `serde_json::Value` in der Fusion. Metadaten lädt `engine` nach der Fusion (`load`, 7.2).|D-70, D-09, I-11|T-DOCREF-1, Grep|
|RK-05|Eingaben sind **bereits gefilterte** Kandidaten (`ReadGate`, nachgelagerter Metadatenfilter). `rank` kennt keinen Gate-Typ. Ränge werden nach dem Filtern vergeben.|D-62, P4|T-HYB-1, T-GATE-1 (Verbund)|
|RK-06|Kein stilles Korrigieren: `top = 0` ⇒ leere Ausgabe; mehr als 1.000 Kandidaten je Liste ⇒ `RankError::TooLarge` (Grenze aus `k ≤ 1000`, D-87); Duplikate **innerhalb** einer Liste ⇒ `RankError::Duplicate` (Rang wäre doppelt vergeben). Es gibt keinen Parameter `k`, der 0 still ersetzen müsste.|P2, D-87|T-RANK-1|
|RK-07|`unseen_bound` ist eine **gültige** obere Schranke: für jedes Dokument, das in Liste `i` erst nach Tiefe `depths[i]` auftritt, gilt `score ≤ unseen_bound(depths)`. Beweis als Property-Test gegen Vollfusion.|D-62|T-HYB-1|
|RK-08|Keine Panics, kein `unwrap`/`expect`, keine Indexierung per `[]`, kein `unsafe`; Ganzzahlrechnung mit `checked_*`/`saturating_*`. Keine Hintergrund-Threads.|P6, Q-02, Q-03|Clippy|
|RK-09|Ring 2; Abhängigkeiten nur `types` (und `ports` nur bei Verwendung eines Port-Typs); `serde`, `serde_json`, `ahash`, `thiserror`, `tracing` nur bei belegter Verwendung; Metadatum `ring = "2"`.|D-91, P8|T-ARCH-5, `cargo tree`|
|RK-10|Zielumfang: ≤ 400 Zeilen `src` ohne Tests (RRF ist klein; das Budget 5.8 verlangt Reduktion, Ist 2.370).|5.8, F-36|`tokei`|

## 6. Rückbau und Migration

|Position|Maßnahme|WP|Nachweis|
|---|---|---|---|
|`explain.rs`, `fusion/provenance.rs`, `fusion/normalized.rs`, `fusion/signal.rs`, `fusion/resonance.rs`, `fusion/topk.rs`|entfernen|WP-1.1|Grep, `tokei`|
|Wrapper (`reciprocal_rank_fusion`, `weighted_*`, `fuse_search_results_*`, `score_normalized_*`, `apply_resonance_bonus`, `g`) und `#[allow(deprecated)]`|entfernen; `engine` im selben Commit anpassen|WP-1.1, WP-2.1|`cargo check --locked`|
|`engine/src/fusion.rs` (59 Z.), `FusionStrategy`, `SignalFusionStrategies` in `types`|entfernen bzw. durch `rrf_fuse`-Aufruf in `engine::search::hybrid` ersetzen|WP-5.1, WP-3.1|Grep|
|`Hit { id: String, metadata }`|durch `Candidate`/`Fused` mit `DocHandle`|WP-8.2|T-DOCREF-1|
|`f32`-Rechnung, `k == 0`-Ersetzung|durch RK-02, RK-06|WP-8.2|T-RANK-1|
|Manifest: `ring = "0"`, Abhängigkeiten|RK-09|WP-1.5|T-ARCH-5|
|Tests (`rank_tests.rs`, `rrf_reference.rs`, `nan_inf_safety_tests.rs`)|`rrf_reference.rs` (naive Referenz) bleibt als Orakel; NaN/Inf-Test entfällt (keine Gleitkommazahlen)|WP-8.2|T-RANK-1|

## 7. Arbeitspakete

1. WP-8.2 (rot): `T-RANK-1` als Property-Test: (a) beliebige Permutation von 2–4 Listen ⇒ bitgleiche Ausgabe; (b) Vergleich gegen die naive Referenz (`rrf_reference.rs`, in Ganzzahlarithmetik); (c) totale Ordnung ohne `Equal`; (d) `unseen_bound` gegen Vollfusion (RK-07).
2. Grün: `rrf`, `bound`, `error`; danach Rückbau (Abschnitt 6) im selben PR, damit `engine` nicht auf entfernte Wrapper zeigt.
3. WP-5.1 (Verbund): `engine::search::hybrid` nutzt Abbruchregel aus Abschnitt 4; `T-HYB-1` (Nachladen bis `k` oder Erschöpfung, `exhausted` korrekt) läuft in `engine` mit Mocks (`testkit`).
4. Mikrobenchmark: Fusion zweier Listen à 1.000 Kandidaten (Anteil an BM-WRITE entfällt; Anteil an Hybridsuche protokollieren).

## 8. Tests und Benchmarks

|ID|Prüft|
|---|---|
|T-RANK-1|RRF deterministisch: Permutationsinvarianz bitgleich, totale Ordnung, Referenzgleichheit|
|T-HYB-1|Nachladen bis `k` gültige Treffer oder Erschöpfung; `exhausted`; Abbruchregel mit `unseen_bound` (Verbund mit `engine`)|
|T-SEARCH-2|nie stilles Verkürzen (Verbund)|
|T-LIMIT-1|Listen > 1.000 Kandidaten ⇒ `RankError::TooLarge` bzw. `Limit::*`|
|T-DOCREF-1|Identität über `DocHandle`; gleiche `doc_tag` mit anderem `cluster_tag` ⇒ verschiedene Einträge|
|Property|`proptest`: Stabilität bei Duplikaten in verschiedenen Listen; Monotonie (Verbesserung des Rangs senkt den Score nie)|
|Mikro (`criterion`)|Fusionskosten bei 2 × 1.000 Kandidaten (Basislinie, P9)|

## 9. Abnahme

Definition of Done (Index Abschnitt 8) und: Grep `weighted|resonance|provenance|explain|normalized|signal|f32|f64|serde_json` in `rank/src` = 0; `cargo tree -p contextra-rank` zeigt nur `types` (und ggf. `ports`); `tokei` ≤ 400 Zeilen `src` [E]; T-RANK-1 grün; Ratchet-Eintrag `rank` = 0.

## 10. Offene Punkte

- **L-19 (neu): Hybridsuche und Nachladen.** SPEC 7.2 (`search_hybrid`), D-62 und 8.11 legen weder die Fusionstiefe noch den Abbruch des Nachladens für **fusionierte** Listen fest (D-62 beschreibt Text und Vektor einzeln). Abschnitt 4 schlägt eine beweisbare Abbruchregel vor (`unseen_bound`); ob sie die Fusionsqualität (Recall der Hybridsuche) ausreichend trägt, ist mit BM-VEC/BM-TEXT zu messen. Ohne Entscheidung bleibt `T-HYB-1` für Hybridsuche unterbestimmt.
- **Festkommaformat:** `2⁴⁰` als Skalierung ist ein Entwurf **[E]**. Bei `K_RRF = 60` und Rängen ≤ 1.000 liegen Beiträge im Bereich 2⁴⁰/1060 ≈ 1,0 · 10⁹ bis 2⁴⁰/61 ≈ 1,8 · 10¹⁰; die Summe über zwei Listen passt in `u64` ohne Überlauf. Eine Änderung der Skalierung ändert die Reihenfolge nur bei Gleichstand nach Rundung (sichtbar in Golden-Tests); sie ist daher als Formatkonstante zu behandeln.
- **Gewichtung/Normalisierung:** Die Ist-Wrapper erlauben gewichtete Fusion. D-26 sagt „nur RRF“; wer gewichtete Hybridsuche braucht, stellt einen Antrag nach Anhang H. Der Ratchet für `engine` (Verbraucher) beendet die Nutzung im selben WP.
- **Treffer-Anreicherung:** `Hit.metadata` (Ist) wird von `engine` nach der Fusion über `load(DocHandle)` geholt; ob die Fusion Metadaten durchreichen darf, ist wegen I-11/HR-4 nein (Metadaten sind Nutzerinhalt).
