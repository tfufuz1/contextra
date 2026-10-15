> **SUPERSEDED:** superseded by docs/spec/CONTEXTRA_ENDPRODUKT_SPEZIFIKATION_v1_4.md

# Source of Truth Diff — Spezifikationsabgleich

**Stand:** Oktober 2026
**Rolle:** Dokumentations-Architekt
**Zweck:** Gegenüberstellung der im Repository gepflegten Hauptspezifikation **`CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md`** (Systemspezifikation v15, Stand 30.09.2026) mit der konsolidierten Master-Spezifikation **`spec_vollstaendig.md`** (v3 / Gesamtspezifikation) zur Herstellung einer eindeutigen *Single Source of Truth* (SSOT) ohne verwaiste Verweise.

---

## 1. Dokumenten-Metadaten im Vergleich

| Eigenschaft | `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md` (v15) | `spec_vollstaendig.md` (v3 / Master) |
| :--- | :--- | :--- |
| **Geltungsbereich** | Repository-interne Systemspezifikation (Ring 0–4, 33 Crates) | Konsolidierte Gesamtspezifikation (Teil 0–8 / Kapitel 0–21) |
| **Fassung / Version** | v15 (Stand 30.09.2026) | v3 / v17 Master (Fassung 3/4) |
| **Gliederungsstruktur** | Teile (Teil 0, TEIL A bis TEIL M) | Teile (Teil 0 bis Teil 8) bzw. Kapitel (§1 bis §21) |
| **Fokus** | IST- vs. SOLL-Abgleich, Crate-API-Inventar, Verhaltensspezifikation | SOTA-Algorithmen, Zielarchitektur, Master-Roadmap |

---

## 2. Abschnitte, die NUR in `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md` (Repo v15) stehen

Die folgenden Abschnitte wurden im Zuge der Spezifikationsversionen v13–v15 direkt im Repository nachspezifiziert und sind in der älteren konsolidierten Fassung `spec_vollstaendig.md` (v3) nicht enthalten:

1. **`Statuskennzeichnung` & `0.5 Explizite Scope-Ausschlüsse (❌)`**
   - Detaillierte Einstufung von Produktkomponenten (🟢 Stabil, 🟡 Experimentell, 🔴 Nicht freigegeben) sowie explizite Nicht-Ziele.
2. **`TEIL C — Frühere SOLL-Punkte, die inzwischen IST sind`**
   - Abgleich gegen HEAD `3f9d2df` bezüglich bereits umgesetzter Architekturmerkmale.
3. **`TEIL G — Erweiterungen und Korrekturen gegen HEAD e4d84c9 (v13)`**
   - G.1 Verifizierte Korrekturen, G.2 Zusätzliche IST-Funktionen, G.3 Invarianten-Register, G.4 Env-Variablen-Ergänzungen, G.5 QS-Erweiterungen, G.6 Konsolidiertes Befundregister.
4. **`TEIL H — Stand (Status-Board, 30.09.2026)`**
   - H.1 Termine, H.2 Stand der P0/P1-Punkte, H.3 Abgleich mit Prompter-Plan, H.4 Roadmap-Stand, H.5 Belegtiefe.
5. **`TEIL I — Nachspezifikation der Lückenliste (v14)`**
   - I.0 bis I.12: Mikrosektionale Nachspezifikation von Basistypen, Kognition, Retrieval, Storage Engine, Graph, Adaptive Routing, KV-Cache, Egress/Sandbox und Python-Bindings.
6. **`TEIL J — SOLL-Zustand, geprüft gegen den IST-Code (v14)`**
   - J.0 bis J.20: Detaillierte Prüfung aller offenen SOLL-Anforderungen gegen den realen Quellcode inklusive Abnahmekriterien.
7. **`TEIL K — Vollständiges API- und Feature-Inventar (v15)`**
   - K.0.1 bis K.35: Mikrofeingranulares Crate-by-Crate API-Inventar über alle 33 Workspace-Crates, MCP-Tool-Register und Umgebungsvariablen.
8. **`TEIL L — Verhaltensspezifikation bisher unbeschriebener Subsysteme (v15)`**
   - L.1 Crypto-Shredding für KV-Segmente (`contextra-crypto/src/kv_shredding.rs`)
   - L.2 Manifest & Schutz vor SSTable-Wiederauferstehung (`contextra-store/src/manifest/core.rs`)
   - L.3 Mandantenisolierter KV-Prefix-Store (`contextra-kvcache/src/prefix_store.rs`)
   - L.4 Eviction im KV-Cache (`contextra-kvcache/src/eviction_worker.rs`)
   - L.5 Egress-Kette (`contextra-privacy`)
   - L.6 WASI-Host der Sandbox (`contextra-sandbox/src/wasi.rs`)
   - L.7 MCP-Werkzeugregister und Bestätigungspflicht (`contextra-mcp/src/sandbox.rs`)
   - L.8 Bi-temporaler Filter und Hard-Scope (`contextra-engine`)
   - L.9 Widerspruchsregister & Hyperkanten-Vorschläge (`contextra-graph`)
   - L.10 Kontextuelles Chunk-Präfix (`contextra-infer-ollama/src/context_prefixer.rs`)
   - L.11 Export-Format v1 (`contextra-engine/src/export.rs`)
   - L.12 Agent: Budget und Dead-Letter-Queue (`contextra-agent`)
   - L.13 System-Schnittstellen (`contextra-sys`, `contextra-simd`, `contextra-wire`)
   - L.14 Python-Bindings (`contextra-py`)
   - L.15 Hintergrunde-Arbeiter der Engine (`contextra-engine/src/background_workers/`)
9. **`TEIL M — Abnahme von v15`**
   - Formaler Abnahmevermerk der Systemspezifikation v15.

---

## 3. Abschnitte, die NUR in `spec_vollstaendig.md` (v3 / Master) stehen

Die folgenden Kapitel und Anhänge bilden den theoretischen Überbau und die historische Entstehungsgeschichte ab, die in `spec_vollstaendig.md` enthalten sind, in `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md` jedoch als eigenständige Anhänge / Dokumente ausgelagert wurden:

1. **`Teil A4 — Architekten-Review: Machbarkeit & Optimierungspotenzial der SOTA-Algorithmen (Fassung 4)`**
   - Detaillierte SOTA-Bewertung für TL-HFD, DiBud, FC-TS und LeanRAG (in Repo unter `docs/spec/00e-teil-a4-architekten-review-sota.md`).
2. **`Teil 7 — Ursachen-Taxonomie (U1–U7)` / `00d-teil-a3-aktueller-umsetzungsstand.md`**
   - Ursachenanalyse historischer Abweichungen (U1: Prototypen-Schulden, U2: Fehlende Ports, U3: Performance-Regressionen, U4: Concurrency-Races, U5: Unvollständige Tests, U6: Veto-Rückstände, U7: Dokumentations-Drift).
3. **`Teil 8 — 60-Tage-Roadmap & Migrationsplan`**
   - Detaillierter 60-Tage-Phasenplan zur schrittweisen Abarbeitung aller Befunde (im Repo unter `docs/spec/20-migrationsplan-und-adr.md` und `18-gesamtroadmap.md`).
4. **`Anhang B — Begründungen, Ist-Zustand, Literatur und Restrisiken`**
   - Ausführliche mathematische und theoretische Begründungen für Hyperkanten, LinUCB-Bandit, Block-Cache-Lock-Freeing, GraphRAG Community-Detection und HNSW v2 Format (im Repo unter `docs/spec/90-anhang-b-begruendungen-a.md` und `90-anhang-b-begruendungen-b.md`).
5. **`Anhang C & D — Änderungsprotokolle Fassung 2.1 und Fassung 4`**
   - Prüfnachweise, Sandbox-Runner-Protokolle und historische Diff-Aufstellungen (im Repo unter `docs/spec/91-anhang-c-aenderungsprotokoll-v2-1.md` und `docs/spec/92-anhang-d-aenderungsprotokoll-v4.md`).

---

## 4. Zuordnungsmatrix der Kerninhalte

| Themenschwerpunkt | `spec_vollstaendig.md` (v3 / Master) | `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md` (v15) |
| :--- | :--- | :--- |
| **Vision & Prinzipien** | Teil 0 / Kapitel 1–3 | Teil 0 / Architektur-Prinzipien P1–P30 |
| **Crate-Graph & Ringe** | Teil 2 / Kapitel 4 | TEIL A (A.1, A.2) & TEIL K (K.1–K.33) |
| **Storage & LSM-Tree** | Teil 3 / Kapitel 5 | TEIL B.2, TEIL I.4, TEIL K.25 (`contextra-store`), TEIL L.2 |
| **Wissensgraph** | Teil 3 / Kapitel 6 | TEIL B.3, TEIL I.5, TEIL K.17 (`contextra-graph`), TEIL L.9 |
| **Retrieval & Fusion** | Teil 4 / Kapitel 7 | TEIL B.3, TEIL I.3, TEIL K.13 (`contextra-rank`) |
| **Bandit Routing** | Teil 4 / Kapitel 8 | TEIL B.5, TEIL I.6, TEIL K.1 (`contextra-adapt`), TEIL K.22 (`contextra-router`) |
| **KV-Cache & Inferenz** | Teil 3/4 / Kapitel 9 | TEIL B.8, TEIL I.7, TEIL K.20 (`contextra-kvcache`), TEIL L.3–L.4 |
| **Sicherheit & Privacy** | Teil 5 / Kapitel 10 | TEIL B.6/B.7, TEIL I.8, TEIL K.9 (`contextra-crypto`), TEIL K.12 (`contextra-privacy`), TEIL L.1/L.5 |
| **API & Inventar** | Teil 6 / Crate-Katalog | TEIL K (K.1 bis K.35 lückenloses Inventar) |

---

## 5. Normative Quellenhierarchie (Single Source of Truth)

Gemäß der Governance in `docs/spec/INDEX.md` gilt für das gesamte Workspace-Repository:

1. **Hauptquelle der Wahrheit (SSOT):**
   **`docs/spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md`** (Systemspezifikation v15) ist die **einzige verbindliche normative Quelle der Wahrheit** für den aktuellen Quellcode-IST- und SOLL-Zustand aller Workspace-Crates in `crates/`.
2. **Begleitende Feature-Spezifikation:**
   `docs/FEATURES_SPECIFICATION.md` dient als ergänzender Crate- & Feature-Flag-Katalog.
3. **Historische & theoretische Referenzen:**
   Die nummerierten Einzeldateien in `docs/spec/` (`00a` bis `92`) sowie `docs/GESAMTSPEZIFIKATION.md` liefern mathematische, theoretische und historische Hintergründe. Bei Widersprüchen zwischen diesen Dokumenten und `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md` hat stets **`CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md`** Vorrang.
