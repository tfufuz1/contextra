# Source of Truth Diff & Spezifikationsabgleich

**Dokument:** `docs/spec/SOURCE_OF_TRUTH_DIFF.md`
**Rolle:** Dokumentations-Architekt
**Datum:** Oktober 2026
**Gegenstand:** Gegenüberstellung der normativen Hauptspezifikation des Repositories (`docs/spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md`, Fassung/Version v15) mit der vorausgegangenen Vollspezifikation `spec_vollstaendig.md` (Fassung 3 / v3, repräsentiert durch `docs/archive/spec/GESAMTSPEZIFIKATION.md` & `docs/spec/92-anhang-d-aenderungsprotokoll-v4.md`).

---

## 1. Governance & Normative Single Source of Truth

Gemäß `docs/spec/INDEX.md` und Constitution gilt für das Contextra-Workspace-Ökosystem:

1. **Normative Master-Quelle:**
   `docs/spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md` (identisch kopiert nach `docs/spec/CONTEXTRA_SPEZIFIKATION_v15.md`) ist die **einzige maßgebliche Source of Truth** für den IST- und SOLL-Zustand aller 33 Workspace-Crates in `crates/` sowie Tooling in `xtask/`.
2. **Prioritätsregel bei Widersprüchen:**
   - Bei Widersprüchen zwischen Quellcode und Dokumentation gilt der Quellcode.
   - Bei Widersprüchen zwischen verschiedenen Spezifikationsdokumenten gilt ausschließlich `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md` (Systemspezifikation v15).
   - Innerhalb der v15-Spezifikation schlägt **Teil 0** alle nachfolgenden Teile.

---

## 2. Struktur-Gegenüberstellung (v15 vs. v3 / Fassung 3)

| Abschnitt / Teil | v15 (`CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md`) | v3 (`spec_vollstaendig.md` / Fassung 3) | Status / Anmerkung |
|---|---|---|---|
| **Teil 0** — Vision, Positionierung, Scope | Vorhanden (aktualisiert auf 33 Crates) | Vorhanden (Fassung 3: MemFuse / Contextra Rebranding) | Erweitert um Ring-0–4 Governance |
| **TEIL A** — Architektur (IST) | Vorhanden (auf HEAD `e4d84c9` / `b564e830` verifiziert) | Vorhanden (historischer Stand Fassung 3) | v15 enthält exakte Crate-Pfade & Ringe |
| **Teil A4** — Architekten-Review: Machbarkeit & SOTA | Vorhanden (in v15 & `00e-teil-a4-...`) | **Fehlt in v3** (neu seit Fassung 4) | In v15 enthalten |
| **TEIL B** — Algorithmen, Funktionen, Parameter (IST) | Vorhanden (SOTA: TL-HFD, DiBud, FC-TS, LeanRAG) | Vorhanden (Vorstufe) | v15 enthält verifizierte Signaturen |
| **TEIL C** — Frühere SOLL-Punkte, die IST sind | Vorhanden (Abgleich gegen HEAD `3f9d2df`) | **Fehlt in v3** (neu in v13) | In v15 enthalten |
| **TEIL D** — SOLL-Zustand (Roadmap & Lücken) | Vorhanden (aktualisiert) | Vorhanden (Fassung 3 Stand) | Superseded in v15 durch Teil J/K |
| **TEIL E** — Produkt, Deployment, Lizenz, Markt | Vorhanden | Vorhanden | In v15 erhalten |
| **TEIL F** — Invarianten (verbindlich, IST + SOLL) | Vorhanden (P1–P30, Invarianten I-1..I-5) | Vorhanden (Grund-Invarianten) | In v15 präzisiert |
| **TEIL G** — Erweiterungen & Korrekturen gegen HEAD | Vorhanden (neu in v13, HEAD `e4d84c9`) | **Fehlt in v3** | In v15 enthalten |
| **TEIL H** — Stand & Status-Board | Vorhanden (Stand 30.09.2026) | **Fehlt in v3** (v3 enthielt veraltete Statusboards) | In v15 enthalten |
| **TEIL I** — Nachspezifikation der Lückenliste | Vorhanden (neu in v14, HEAD `81ab98a`) | **Fehlt in v3** | In v15 enthalten |
| **TEIL J** — SOLL-Zustand, geprüft gegen IST-Code | Vorhanden (neu in v14) | **Fehlt in v3** | In v15 enthalten |
| **TEIL K** — Vollständiges API- & Feature-Inventar | Vorhanden (neu in v15, HEAD `b564e830`, 33 Crates) | **Fehlt in v3** | In v15 enthalten |
| **TEIL L** — Verhaltensspezifikation Subsysteme | Vorhanden (neu in v15, bisher unbeschriebene Module) | **Fehlt in v3** | In v15 enthalten |
| **TEIL M** — Abnahme von v15 | Vorhanden (neu in v15) | **Fehlt in v3** | In v15 enthalten |
| **§21** — Normative SOTA-Algorithmen-Erweiterung | Vorhanden (§21.1–§21.4) | **Fehlt in v3** (neu seit Fassung 4) | In v15 enthalten |
| **Anhang D** — Änderungsprotokoll Fassung 4 | Vorhanden (in `92-anhang-d-aenderungsprotokoll-v4.md`) | **Fehlt in v3** | In v15 enthalten |

---

## 3. Abschnitte, die nur in v15 (`CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md`) stehen

1. **Teil A4 (Architekten-Review SOTA):** Re-Evaluierung und Zweitverifikation von SOTA-Algorithmen (DiBud-Kanalmodell, LeanRAG Cascade, Lovász Subgradient, BoundedTopK Iteratoren).
2. **Teil G (Erweiterungen und Korrekturen gegen HEAD `e4d84c9`):** Exakter Abgleich des Codes bezüglich SSI-Waterfall, Compaction-Peak-Bytes, LSM-Commit-Protokoll, WAL-Legacy-HMAC-Keys und DiskANN-Invarianten.
3. **Teil H (Status-Board & Termine Stand 30.09.2026):** Aktuelles Statusboard der Umsetzungsgrade über alle Ringe.
4. **Teil I (Nachspezifikation der Lückenliste):** Detaillierte Spezifikation fehlender API-Typen (`ErrorDto`, `Tombstone`, `MaintenanceScheduler`, Export/Import-Schnittstellen).
5. **Teil J (SOLL-Zustand vs. IST-Code):** Matrix aller verbleibenden SOLL-Anforderungen mit explizitem IST-Status gegen Codebase.
6. **Teil K (Vollständiges API- und Feature-Inventar aller 33 Crates):** Mikrofeingranulares Inventar der öffentlichen Structs, Enums, Traits und Funktionen aller Workspace-Crates in Ring 0–4 inklusive Modul-Tellen, `xtask`-Tools und Environment-Spezifikationen.
7. **Teil L (Verhaltensspezifikation bisher unbeschriebener Subsysteme):** Formale Verhaltensregeln für Checkpointing, Background-Worker, Dynamic-Drift, Reranker, Calibration und Prompt-Injection-Guards.
8. **Teil M (Abnahme der Fassung v15):** Formale Abnahmekriterien und Prüfprotokoll für V15.
9. **§21 (SOTA-Algorithmen-Erweiterung):** Detaillierte Algorithmen-Spezifikationen für TL-HFD (§21.1), DiBud (§21.2), FC-TS (§21.3) und LeanRAG (§21.4).
10. **Anhang D (Änderungsprotokoll Fassung 4 & v15):** Vollständiges Audit-Log der Dokumentenhistorie.

---

## 4. Abschnitte / Inhalte, die nur in v3 (`spec_vollstaendig.md` / Fassung 3) standen

1. **Historische Status-Boards der Fassung 3:** Veraltete Umsetzungsstände (Stand August/September 2026), die durch spätere Commits und das Status-Board in Teil H/J/K von v15 überholt wurden.
2. **Obsolete Roadmap-Klassifizierungen:** Frühere Roadmap-Stufen (z.B. LeanRAG als Stufe 4 Fernziel), die in Fassung 4 / v15 als Stufe 1½ re-priorisiert und umgesetzt wurden.
3. **Legacy-Crate-Bezeichnungen:** In Fassung 3 vereinzelt verbliebene alte `memfuse-*`-Referenzen, die in v15 mechanisch und vollständig auf `contextra-*` bereinigt wurden.

---

## 5. Fazit & Dokumenten-Hierarchie

`docs/spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md` ist die **vollständige, konsolidierte und normative Gesamtspezifikation (v15)**. Alle in der historischen Fassung 3 (`spec_vollstaendig.md`) enthaltenen normativen Vorgaben wurden in v15 übernommen, korrigiert und um die Teile A4, G, H, I, J, K, L, M sowie §21 erweitert.

Es gibt **keine verwaisten Normativinhalte** aus v3, die in v15 fehlen.
