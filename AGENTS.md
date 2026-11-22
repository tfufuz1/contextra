# AGENTS.md — Arbeitskontext: `contextra-storage` (Kern zuerst)

Dieses Dokument fasst `docs/SPEC.md` (Revision R6) für Entwickler und Agenten zusammen. **Bei Widerspruch gilt die Spec.** Ändere zuerst `docs/SPEC.md`, dann den Code (Spec §1.5). Ergänzend gilt `CLAUDE.md` für Umgebung, Push-Ablauf und Arbeitsregeln der Sitzung.

## 1. Auftrag und Fokus

`contextra-storage` ersetzt `crates/contextra-store` und ist das Fundament des Ökosystems für Sprachmodell-Anwendungen. **Bis Gate G-P (Spec §24) wird ausschließlich dieser Kern entwickelt.** Alles andere im Repository ist ausgeblendet: keine Änderungen an anderen Crates, keine Anbindung aufsetzender Komponenten, keine neuen Funktionen oberhalb des Kerns. Ausnahme sind Arbeiten, die der Kern zwingend braucht (Workspace-Einbindung, CI, Lint-Konfiguration).

Der Kern ist ein eingebetteter, synchroner, crash-sicherer Schlüssel-Wert-Kern mit atomaren Mehr-Space-Änderungen, MVCC-Sichten, Kopf-ohne-Körper-Lesen, budgetierten Seiten und **löschlokaler Datenanordnung** (Cluster).

## 2. Kernentscheidungen (Kurzfassung der Spec)

| Thema | Regel | Spec |
|---|---|---|
| Cluster | Löschdomäne, Schlüsseldomäne und Platzierungseinheit; Ordnung `(cluster, key, tag↓)`; Blöcke clusterrein | §2 L1–L2, §7.2 |
| Pure / Packed | Datei mit einem Cluster oder mehreren Clustern einer Packgruppe; nie beides | §8 |
| Schreiber | Ein Committer; Gruppen-Fsync; er vergibt alle Stamps und setzt Cluster-Gates | §12.1, §19 |
| Gate | Löschung beginnt mit einem `ClusterGate`-Frame im Committer, nicht nur mit einem Catalog-Edit | §17.5 ER1 |
| Catalog | einzige Wahrheit über Dateien; lebendig ⇔ durabler `files_add` | §11.8 |
| Löschreihenfolge | Gate → Schlüssel vernichten → Dateiwechsel (ein Edit) → Unlink; nur Roll-forward | §17.5 |
| Versiegelung | In `Encrypted` mit `Seal::Cluster` ist alles eines Clusters mit dem Clusterschlüssel versiegelt | §17.3 |
| Redo-AAD | `(space, cluster, op_uid, op_index)`, **ohne Stamp** (Versiegelung geschieht vor der Stamp-Vergabe) | §17.3 |
| Anker | Rollback-Schutz über Port `Anchor`; Pflicht bei `Encrypted`/`Authenticated` | §17.10 |
| Volatile-Spaces | kein Redo, keine Dateien | §7.4 |
| Sperren | Rangordnung 1–9, nur aufsteigend; Memtable lock-frei (kein Rang 5) | §19.2 |
| Politik | Der Kern kennt keine Rechte, Mandanten, Fristen oder Rechtsgründe | §3.3 |
| Fehler | kein `panic`; `unwrap`/`expect`/`indexing_slicing` verboten; Fsync-Fehler nie wiederholen | §3.6, §6 |

## 3. Ist-Stand (gemessen am 22.11.2026, belegt in Spec Anhang B)

- **Spec:** R6 liegt vor. Normativ wird sie mit **G-0**: zwei Reviewer, offene Punkte aus Spec §26 entschieden, CI grün. G-0 ist **nicht erfüllt**.
- **Code:** `crates/contextra-storage` ist **nicht im Workspace** und **baut nicht**: `Cargo.toml` verweist auf `../contextra-storage-ports` und `../contextra-crypto-ref`, beide Verzeichnisse fehlen.
- Der vorhandene Code (24 961 Zeilen in `src/` und `tests/`) folgt dem früheren Entwurf R2 (kein Cluster, kein Gate-Frame, kein Anker). Er ist **Vorlage**, nicht Zielzustand. Formatbetroffen sind `redo`, `run`, `blob`, `catalog`, `engine/committer`.
- `recovery`, `flush`, `compaction` fehlen im Modulbaum; `erase` ist nur Dokumentation; es gibt keine Facade.
- Altstand `crates/contextra-store`: 27 898 Zeilen `src/`, 580 Tests, 1 260 `unwrap`/`expect`-Treffer. Er bleibt unverändert, bis der Neubau ihn ablöst.
- Es existieren weder `ARCHITECTURE.md`, `CONTRIBUTING.md` noch `tools/check_spec_sync.py`, obwohl Dokumente auf sie verweisen.
- Es wurde nichts kompiliert oder gemessen. Alle Leistungszahlen der Spec sind Hypothesen.

## 4. Reihenfolge der Arbeit

Die Phasen P0–P6 und ihre Gates stehen in Spec §24. **Keine Phase beginnt vor dem Gate der vorherigen.** Nächste Schritte (Spec §24, Sofortmaßnahmen):

1. Workspace-Einbindung, fehlende Crates `contextra-storage-ports` und `contextra-crypto-ref` anlegen, Build grün.
2. Bestand Modul für Modul auf R6 umstellen (zuerst `codec`, `env`, `sync`, dann Formate).
3. CI einrichten, Toolchain-Pin klären (Pin 1.89.0 ist in der Sitzungsumgebung nicht ladbar, siehe `CLAUDE.md` §1).
4. Referenzcodierer `tools/reference/generator.py` und goldene Vektoren (Pflicht vor G-F).

## 5. Verbindliche Regeln für Änderungen am Kern

1. **Spec zuerst.** Keine Abweichung ohne Änderung von `docs/SPEC.md`. Änderungen an §8, §11, §12, §15, §17, §18, §22 brauchen zwei Reviewer und aktualisierte Referenzvektoren.
2. **Jede Invariante hat einen Test.** Tests zitieren die ID (`// INV-ERA-3`). Neue Invariante ⇒ Eintrag in Spec §22.
3. **Keine Panik im Produktionspfad.** `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!`, `unreachable!`, Indexzugriff sind verboten. `let _ =` auf `sync_*`, `rename`, `unlink` ist verboten.
4. **Kein `unsafe`** im Kern (`#![forbid(unsafe_code)]`). Plattformnahes liegt hinter Ports.
5. **Nur `std`, `contextra-storage-ports` und die bewilligten Abhängigkeiten** (Spec §3.5). Jede weitere Abhängigkeit braucht einen Registereintrag in Spec §25 und `cargo-deny`.
6. **Sperrordnung einhalten** (Spec §19.2). Keine E/A unter Rang ≥ 4. Kein Rückruf (`Observer`, `SegmentOwner`) unter einer Sperre.
7. **Fsync-Fehler nie wiederholen.** Fehler ⇒ Poison, Lesen bleibt.
8. **Reihenfolge der Löschung nie ändern** (Gate < Schlüssel < Dateiwechsel < Unlink). Ein begonnener Erase wird fortgesetzt, nie zurückgenommen.
9. **Keine Inhalte in Fehlertexten, Logs oder Metriknamen** (Schlüssel, Nutzerschlüssel, Körper).
10. **Absturztests laufen auf `FaultFs`**, nicht auf Stubs. Die Absturzmatrix (Spec §15.2) ist Gate-Bestandteil.
11. **Keine erfundenen Zahlen.** Messwerte mit `grep`, `wc` oder einem Messlauf belegen. Leistungszahlen bleiben Hypothesen (Spec §23), bis ein Messbericht vorliegt.
12. **Dokumentationssprache:** keine Begriffe wie „DSGVO-konform“ oder „unwiderruflich“; Löschaussagen nennen Stufe und Restexposition (Spec §17.4, §17.11). `tools/doc_lint.py` prüft das, sofern vorhanden.
13. **Sprache:** Prosa und Rustdoc deutsch, Bezeichner englisch. Keine Produkt- oder Fremdsystemnamen im Crate.
14. **Keine Selbstdeklarationen:** keine `ANCHOR[`- oder `FILE-CONTEXT`-Kommentare, keine `contract.rs`, keine `AGENTS.md`/`SPEC.md`/`INVENTAR.md` unter `crates/`.

## 6. Was der Kern nicht tut (Auszug, Spec §4)

Verteilung, eigene Kryptografie, Schemas und Sekundärindizes, Tokenizer, Embeddings, ANN, Volltext, Graphalgorithmen, Rechte- und Mandantenprüfung, Rechtsbewertung von Löschfristen, Schlüsselverwaltung, Commit-DAG. Diese Dinge gehören in aufsetzende Schichten, die erst nach G-P angebunden werden.

## 7. Dokumentenkarte

| Pfad | Rolle |
|---|---|
| `docs/SPEC.md` | **normativ** (R6) |
| `AGENTS.md` | dieser Arbeitskontext |
| `CLAUDE.md` | Sitzungsumgebung, Push-Ablauf, Arbeitsregeln |
| `docs/archive/` | Herkunft: R2, R3, R5-Fassungen, Konzepte, Review. **Keine Anforderungsquelle.** |

## 8. Offene Entscheidungen, die den Kern blockieren

Siehe Spec §26. Vor G-0 müssen mindestens entschieden sein: O-7 (Toolchain und CI), O-10 (Anker-Implementierung), O-12 (zweiter Reviewer). Rechtlich zu prüfen bleiben O-1 bis O-3; der Kern trifft dazu keine Aussage.
