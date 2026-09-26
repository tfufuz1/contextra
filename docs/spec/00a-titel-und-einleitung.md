---
source: CONTEXTRA_SPEC_v4_MASTER.md
chapter: "00a"
---
# Contextra — Finale Konsolidierte Gesamtspezifikation (Fassung 4 · SOTA-Algorithmen-Integration & Architekten-Review)

> **Namenshinweis:** Dieses Dokument bezeichnete das Projekt bis Fassung 2.1 als „MemFuse". Mit Fassung 3
> ist der Produktname verbindlich **Contextra**. Die Umbenennung wurde mechanisch (Suchen/Ersetzen über den
> gesamten Dokumenttext, keine Handarbeit) durchgeführt: `MemFuse` → `Contextra`, `memfuse-*`-Crate-Präfixe →
> `contextra-*`. Grund für den Namenswechsel und Prüfung der Registry-Verfügbarkeit (crates.io, PyPI, npm):
> siehe Gespräch/Beratungsprotokoll; der bisherige Name kollidierte mit einem fremd belegten PyPI-Paket
> (`memfuse`, Autor Calvin Ku) sowie mit mehreren gleichnamigen GitHub-Projekten. Frühere Bezeichnung „MemFuse
> Cognitive OS" wird nicht fortgeführt; siehe §2 zur aktualisierten Positionierung als eingebettete,
> air-gapped-fähige Gedächtnisschicht statt „Cognitive OS"/„LLM OS" (Empfehlung der strategischen
> Tiefenberatung, §3.3 dieses Beratungsdokuments — dort auch als Begründung archiviert).

> **Status:** Normativ · Die Gesamtspezifikation wurde in `docs/spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md`
> ("Contextra — Finale Produktspezifikation (Synthese)") vollständig konsolidiert und synthetisiert. Sie ist die
> einzige maßgebliche Quelle für Produkt, Architektur, Algorithmen, Implementierungsvorgaben, Sicherheitsmodell,
> Schnittstellenspezifikation, Stabilisierungsphasen und Optimierungs-Roadmap.
>
> **Charakter:** Dieses Dokument führt Produktvision, Zielarchitektur, normative Signaturen,
> algorithmische Spezifikationen, mikrofeingranulare Schnittstellendefinitionen, ein verbindliches
> Stabilisierungs- und Gate-Modell sowie die priorisierte Optimierungs-Roadmap in einem einzigen,
> in sich geschlossenen Dokument zusammen. Es ersetzt vollständig alle vorherigen
> Spezifikationsfassungen, -deltas und separat geführten Stabilisierungspläne. **Es referenziert keine
> externen Dokumente** — jede Aussage, die für die Arbeit an Contextra nötig ist, steht hier.
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
> Prüfung `CONTEXTRA_ZIELARCHITEKTUR.md` (v1, Basis-Commit `3f37ab30`) und ihre Korrektur
> `CONTEXTRA_ZIELARCHITEKTUR_v2.md` (Basis-Commit `806a40c1`) haben strukturelle Verstöße gegen P5 (Crate
> `contextra-db` hängt aufwärts von `contextra-infer-candle`/`contextra-infer-ollama`/`contextra-infer-onnx` ab), eine unvollständige
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
> Shard-Hasher in H2, nicht kompilierbare Skizzen, unsafe-pflichtige Entwürfe im Safe-Crate `contextra-store`),
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
> **falsch** — die verifizierte Implementierung speicherte in einer früheren Prüfung nur einen
> Platzhalter-String und zählte einen Zähler hoch, ohne echten Prefill einzusparen (§9.2, „Vorher/Jetzt").
> Ab dieser Fassung gilt jeder 🟢-Marker zusätzlich als widerrufen, sobald Teil A2 für den betroffenen Bereich
> einen belegten Gegenbefund („D#" in Teil A2 §2) nennt; maßgeblich ist die D#-Tabelle in §A2.1. **Teil A3
> (neu, Fassung 3) korrigiert diesen Befund für den aktuellen HEAD erneut** — die KV-Cache-Bridge ist laut
> Commit-Historie (`a16fd650`, `d37a70b6`) inzwischen über den reinen Platzhalter hinaus ausgebaut (Prefix-Radix-Baum,
> RAII-Guards, Tier-2-AEAD-Verschlüsselung, `KvState` für Stufe B); Teil A3 §A3.1 führt die verifizierte
> Einzelbewertung.
>
> **Leitentscheidung dieser Fassung (5) — Fassung 3 (Rename + Status-Konsolidierung, ranghöchste Ergänzung):**
> Fassung 3 fügt **Teil A3 — Aktueller Umsetzungsstand & priorisierte Restarbeit** direkt im Anschluss an
> Teil A2 ein und macht dieses Gesamtdokument damit zur einzigen Quelle der Wahrheit, die (a) den
> Produktnamen auf Contextra aktualisiert, (b) den Status jedes in Teil A2/§17/§18/§20 als offen markierten
> Punktes gegen den tatsächlichen Repository-Zustand zum Zeitpunkt dieser Fassung nachführt, und (c) die
> Ergebnisse der externen strategischen Tiefenberatung (Scope-Einfrierung, Prozess- vor Feature-Risiko,
> Positionierung) als normative Priorisierung übernimmt. **Teil A3 ist ranghöher als Teil A2, soweit es um
> Priorisierung und Statusaussagen geht** (Teil A2 bleibt maßgeblich für die technische Zielarchitektur selbst
> — das Ring-Modell, Crate-Zuschnitt und Abhängigkeitsrichtung ändern sich durch Fassung 3 nicht). Im
> Konfliktfall zwischen einem Reifegrad-Marker in §5–§20 und einer Statusaussage in Teil A3 gilt Teil A3, da
> es der zuletzt gegen den Code geprüfte Stand ist.
>
> **Leitentscheidung dieser Fassung (6) — Fassung 4 (SOTA-Algorithmen-Integration, Architekten-Review):**
> Fassung 4 verarbeitet zwei extern zugelieferte Dokumente — einen Deep-Research-Bericht zu vier
> algorithmischen State-of-the-Art-Verfahren aus 2025/2026 (`Deep-Research-Bericht_Contextra_erweitert.md`,
> inkl. dessen eigenem Teil C, einem bereits vorab durchgeführten Quellcode-Abgleich gegen Repo-Stand
> `4b9387d6`) sowie diese Spezifikation selbst in der Fassung 3 — und führt sie zusammen. Die Rolle dieser
> Fassung ist die eines **Principal-Architect-Reviews**: Sie prüft Umsetzbarkeit und Optimierungspotenzial der
> vier Verfahren gegen den tatsächlichen, frisch geklonten Workspace-Zustand (identischer Commit `4b9387d6`,
> Repository `github.com/tfufuz1/contextra`, geprüft am 23.09.2026) und integriert nur das, was diese Prüfung
> bestätigt. Fassung 4 ergänzt **Teil A4 — Architekten-Review: Machbarkeit und Optimierungspotenzial der
> SOTA-Algorithmen** direkt im Anschluss an Teil A3 sowie **§21 — Normative SOTA-Algorithmen-Erweiterung**
> direkt im Anschluss an §20, korrigiert das FlatBuffers-Schema in §12 (`child_edge_ids`), ergänzt
> Abnahmekriterien AK-16 bis AK-19 in §16.2 und trägt die vier Verfahren in die Roadmap (§17, §18) mit
> revidierter Priorität ein. **Teil A4 ist wie Teil A3 ranghöher als §5–§20, soweit es um Priorisierung und
> Statusaussagen zu den vier SOTA-Verfahren geht; im Konfliktfall zwischen einer Aussage in §21 und einer
> Aussage in Teil A4 gilt Teil A4**, da dort die Beleglage und der Ist-Abgleich geführt werden — §21 ist die
> daraus abgeleitete normative Schnittstellen- und Algorithmusspezifikation. Diese Fassung ändert an Teil
> A/A2/A3 und §0–§20 inhaltlich nichts außer den in diesem Absatz genannten punktuellen Ergänzungen (§12,
> §16.2, §17, §18); alle übrigen Aussagen der Fassung 3 bleiben unverändert normativ. Entsprechend der eigenen
> Regel dieses Dokuments („Künftige Änderungen erfolgen als direkte Überarbeitung dieses Dokuments, nicht als
> weiteres Delta-Dokument", vormals Fassung-3-Schlussvermerk vor Anhang B) ist Fassung 4 **eine einzige,
> in sich geschlossene Datei** und kein separat geführtes Zusatzdokument. Änderungsprotokoll und
> Prüfnachweise dieser Fassung: **Anhang D**.

---

## Inhaltsverzeichnis

**A.** [Stabilisierungsauftrag: Ground Truth, Reifegrade, Gates](#teil-a)
**A2.** [Zielarchitektur v2 — Ring-Modell, verbindlich ab sofort](#a2-zielarchitektur-v2)
**A3.** [Aktueller Umsetzungsstand & priorisierte Restarbeit (Fassung 3, Quelle der Wahrheit)](#a3-status)
**A4.** [Architekten-Review: Machbarkeit & Optimierungspotenzial der SOTA-Algorithmen (Fassung 4)](#a4-review)
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
21. [Normative SOTA-Algorithmen-Erweiterung: TL-HFD, DiBud, FC-TS, LeanRAG](#21-sota)

**B.** [Anhang B — Begründungen, Ist-Zustand, Literatur (nachrangig)](#anhang-b)
**C.** [Anhang C — Änderungsprotokoll Fassung 2.1 und Prüfnachweise](#anhang-c)
**D.** [Anhang D — Änderungsprotokoll Fassung 4 und Prüfnachweise](#anhang-d)

---

<a id="teil-a"></a>
