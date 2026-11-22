# AGENTS.md — Arbeitskontext: `contextra-storage` (Kern zuerst)

Dieses Dokument fasst `docs/SPEC.md` (Revision R7) für Entwickler und Agenten zusammen. **Bei Widerspruch gilt die Spec.** Ändere zuerst `docs/SPEC.md`, dann den Code (Spec §1.5). Ergänzend gilt `CLAUDE.md` für Umgebung, Push-Ablauf und Arbeitsregeln der Sitzung.

## 1. Auftrag und Fokus

`contextra-storage` ersetzt `crates/contextra-store` und ist das Fundament des Ökosystems für Sprachmodell-Anwendungen. **Bis Gate G-6 (Spec §22, Löschung ER0–ER7 abgeschlossen) wird ausschließlich dieser Kern entwickelt.** Alles andere im Repository ist ausgeblendet: keine Änderungen an anderen Crates, keine Anbindung aufsetzender Komponenten. Ausnahme sind Arbeiten, die der Kern zwingend braucht (Workspace-Einbindung, CI, Lint-Konfiguration). Kein neues Crate ohne Eintrag in Spec §3.1.

Der Kern ist ein eingebetteter, synchroner, crash-sicherer Schlüssel-Wert-Kern mit atomaren Mehr-Space-Änderungen, MVCC-Sichten und **löschlokaler Datenanordnung** (Cluster).

## 2. Kernentscheidungen (Kurzfassung der Spec)

| Thema | Regel | Spec |
|---|---|---|
| Cluster | Löschdomäne, Schlüsseldomäne, Platzierungseinheit; Ordnung `(cluster, key, tag↓)`; Blöcke clusterrein | §6.2, §7 |
| Pure / Packed | Datei mit einem Cluster oder mehreren Clustern einer Packgruppe; nie beides | §7.1 (R-PL-1) |
| Schreiber | Ein Committer; Gruppen-Fsync; er vergibt alle Stamps und setzt Cluster-Gates | §11.1, §17.1 |
| Gate | Löschung beginnt mit einem `ClusterGate`-Frame im Committer | §11.3, §15.5 (ER1) |
| Catalog | einzige Wahrheit über Dateien; lebendig ⇔ durabler `files_add` ohne `files_remove` | §10.8, INV-CAT-1 |
| Löschreihenfolge | Gate → Redo-Kürzung → Runs/Blobs → Schlüssel entfernen → Ledger → Anker; nur vorwärts | §15.5 |
| Löschstufen | ER0–ER7; kumulativ; Plain erreicht höchstens ER5 | §15.2 |
| Nonces | 24-Byte-Zufallswerte; Rotationsgrenze 2^32 je Schlüssel | §10.1, §16.3 |
| Clusterschlüssel | zufällig, im Catalog umhüllt; nicht aus der Wurzel abgeleitet | §16.2 |
| Versiegelung | `Seal::Cluster`: Redo-Nutzlast, Blöcke, Blob-Records; keine Stamp-AAD | §10.4, D-15 |
| Anker | Rollback-Schutz über Port `Anchor` (`erase_epoch`, `ledger_head`, `catalog_epoch`) | §14.2, §16.7 |
| Volatile-Spaces | nur Memtable, kein Redo; nie in derselben Gruppe wie dauerhafte Changes | §11.2, D-10 |
| Sperren | Rangordnung 1–9, nur aufsteigend; Memtable lock-frei (Rang 5 entfällt) | §17.1 |
| Politik | Der Kern kennt keine Rechte, Mandanten, Fristen oder Rechtsgründe | §1, §2.3 |
| Fehler | kein `panic`; Fsync-Fehler nie wiederholen; Fehler führen zu `Degraded` oder `Poisoned` | §5.1, §11.6 |

## 3. Ist-Stand

- **Spec:** R7 ist die einzige normative Fassung (`docs/SPEC.md`). Die Vorfassungen liegen unter `docs/archive/` und sind keine Quelle. Gate G-0 gilt mit dieser Revision als erfüllt. **G-1 setzt O-1 (Toolchain) und O-12 (benannte Reviewer) voraus** (§22, §24).
- **Code:** Der Kern ist **noch nicht neu gebaut**. Es gibt kein Crate `crates/contextra-storage-ports`. `crates/contextra-storage` ist Bestand aus dem früheren Entwurf R2 (kein Cluster, kein Gate-Frame, kein Anker) und hängt über `contextra-ports` vom Altstand ab (D-02). Er ist **Vorlage**, nicht Zielzustand.
- **Tests des Bestands:** `tests/format/golden_test.rs` prüft ein FORMAT-Golden mit 51 Byte, das nicht mit R7 übereinstimmt (R7: 53 Byte bei `Plain`, §10.2). Die Tests werden in G-2 und G-4 neu erzeugt (§19.3). Bis dahin gelten sie nicht als Nachweis.
- **Goldens:** Die Primitive CRC-32C, XXH64 und die Bloom-Positionen sind geprüft (Anhang A.1). FORMAT-, Frame- und Seal-Goldens sind **nicht verifiziert** (Anhang A.2).
- **Altstand** `crates/contextra-store` bleibt unverändert, bis der Neubau ihn ablöst.
- Es wurde nichts gemessen. Alle Leistungsaussagen sind Hypothesen (§21).

## 4. Reihenfolge der Arbeit

Die Gates G-0 bis G-F stehen in Spec §22. **Keine Phase beginnt vor dem Gate der vorherigen.** Nächste Schritte:

1. O-1 und O-12 entscheiden (Spec §24), dann Gate G-1: `contextra-storage-ports` und `MemFs`/`FaultFs` anlegen, Workspace-Build grün.
2. Gate G-2: Format-Codecs und Goldens aus `tools/reference/generator.py` (Pflicht vor G-F). Die Goldens in §10 vor der Übernahme gegen den Referenzcodierer abgleichen (Anhang A.2).
3. Gate G-3 bis G-6: Redo, Catalog und Recovery, Lesen und Schreiben, Löschung.
4. CI einrichten (`cargo-deny`, `cargo-mutants`, `loom`/`shuttle`, Absturzmatrix).

## 5. Verbindliche Regeln für Änderungen am Kern

1. **Spec zuerst.** Keine Abweichung ohne Änderung von `docs/SPEC.md`. Änderungen an §4, §5, §9–§11, §14–§16, §18 brauchen zwei Reviewer, aktualisierte Goldens (Anhang A) und eine Prüfung gegen die Freeze-Liste (§10.12).
2. **Jede Invariante hat einen Test.** Tests zitieren die ID (`// INV-ERA-3`). Neue Invariante ⇒ Zeile in der Tabelle §18 (`tests/invariants.rs` liest sie).
3. **Keine Panik im Produktionspfad.** `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!`, `unreachable!` und Indexzugriff sind verboten. `let _ =` auf `sync_*`, `rename`, `unlink` ist verboten.
4. **Kein `unsafe`** im Kern (`#![forbid(unsafe_code)]`). Plattformnahes liegt hinter Ports.
5. **Nur die Abhängigkeiten aus Spec §23.3** (`std`, `contextra-storage-ports`, `zeroize`, `crossbeam-skiplist`, optional `bytes`). Keine Krypto-Crates im Kern (D-05). Jede weitere Abhängigkeit braucht einen Eintrag in §23.3 und `cargo-deny`.
6. **Sperrordnung einhalten** (Spec §17.1). Keine E/A unter Rang 4 oder höher, die nicht im Sperrrang vorgesehen ist. Kein Rückruf an Nutzercode unter einer Sperre (INV-LOCK-1).
7. **Fsync-Fehler nie wiederholen.** Fehler führen zu `Degraded` (§11.6); ENOSPC sperrt Schreibe bis `resume_writes`.
8. **Reihenfolge der Löschung nie ändern** (§15.5). Ein begonnener Löschauftrag wird fortgesetzt, nie zurückgenommen (INV-ERA-6).
9. **Keine Inhalte in Fehlertexten, Logs oder Metriknamen** (Schlüssel, Nutzerschlüssel, Körper; INV-KEY-3).
10. **Absturztests laufen auf `FaultFs`**, nicht auf Stubs. Die Absturzmatrix (Spec §19.1) ist Gate-Bestandteil.
11. **Keine erfundenen Zahlen.** Messwerte mit `grep`, `wc` oder einem Messlauf belegen. Leistungszahlen bleiben Hypothesen (Spec §21), bis ein Messbericht vorliegt.
12. **Dokumentationssprache:** keine Begriffe wie „DSGVO-konform“ oder „unwiderruflich“ ohne Messwert; Löschaussagen nennen Stufe und Prüfschritt (Spec §15.1, §15.2, §15.11). `tools/doc_lint.py` prüft das, sofern vorhanden.
13. **Sprache:** Prosa und Rustdoc deutsch, Bezeichner englisch. Keine Produkt- oder Fremdsystemnamen im Crate.
14. **Keine Selbstdeklarationen:** keine `ANCHOR[`- oder `FILE-CONTEXT`-Kommentare, keine `contract.rs`, keine `AGENTS.md`/`SPEC.md`/`INVENTAR.md` unter `crates/`.

## 6. Was der Kern nicht tut (Auszug, Spec §2.2)

Verteilung, eigene Kryptografie-Protokolle außerhalb von Ports, Schemas und Sekundärindizes, Tokenizer, Embeddings, ANN, Volltext, Graphalgorithmen, Rechte- und Mandantenprüfung, Rechtsbewertung von Löschfristen, Schlüsselverwaltung, Commit-DAG, Inhaltsadressierung (O-5). Diese Dinge gehören in aufsetzende Schichten, die erst nach G-6 angebunden werden.

## 7. Dokumentenkarte

| Pfad | Rolle |
|---|---|
| `docs/SPEC.md` | **normativ** (R7) |
| `AGENTS.md` | dieser Arbeitskontext |
| `CLAUDE.md` | Sitzungsumgebung, Push-Ablauf, Arbeitsregeln |
| `docs/archive/` | Vorfassungen R2 und R6-Cluster-Entwurf. **Keine Anforderungsquelle.** |

## 8. Offene Entscheidungen, die den Kern blockieren

Siehe Spec §24. Vor G-1 müssen mindestens entschieden sein: O-1 (Toolchain-Pin und MSRV) und O-12 (benannte Reviewer). Vor G-F: O-7 (externer Anker) und O-13 (Bloom oder Ribbon). Rechtlich zu prüfen bleibt O-2; der Kern trifft dazu keine Aussage.
