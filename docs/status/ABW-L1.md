# Statusbericht Abweichungen Lane L1 (Root-Dokumente)

Dieses Dokument erfasst alle im Rahmen von Lane L1 identifizierten Abweichungen, Vorbehalte und offenen Punkte gemäß den Vorgaben der Spezifikation 1.2 (§28.5).

---

## Übersicht der behobenen Abweichungen

| Datei | Spezifikationsabschnitt | Behobene Abweichung / Anpassung |
|---|---|---|
| `README.md` | §2.0, §4.3, §6.0, §13.1, §20.12 | Produktbeschreibung auf Missionen M1–M5 ausgerichtet; M1 (persistenter Prefix-Cache) explizit als Zielzustand/Opt-in gekennbezeichnet; Löschaussagen strikt nach INV-CLAIMS-1 (§4.3, §20.12) bereinigt; Crate-Tabelle an `capabilities.toml` und §6.0 angepasst. |
| `ARCHITECTURE.md` | §6.0, §6.7, §6.8, §6.10, §6.11 | 5-Ring-Modell (Ring 0–4), Kantenregeln (§6.10), Kern-Geschlossenheit (§6.8) ohne Async in Ring 0 (P26), Rand-Brücken (§6.11) über `contextra-ports` und 3 Unsafe-Inseln (§6.7) vollständig spezifikationskonform dokumentiert; Crate-Inventar mit `capabilities.toml` abgeglichen. |
| `docs/CHANGELOG.md` | §0.1, §34 | Eintrag „Spezifikation 1.2 übernommen“ mit Referenz ergänzt; historische Versionstabelle unverändert erhalten. |
| `CONTRIBUTING.md` | §27.0, §28.0, REQ-QA-010 | Quell- und Entscheidungs-Rangfolge (§28.1) und Verbot der Gate-Schwächung (REQ-QA-010) aufgenommen. |
| `SECURITY.md` | §10.0, §19.1, §30.0 | Bedrohungs- und Sicherheitsmodell explizit auf §30.0 und §10.0 referenziert; Ingestion- und Provenienz-Regeln (INV-PROVENANCE-1) verankert. |
| `AGENTS.md` | §28.1, REQ-QA-010 | Verbindliche Rangfolge (§28.1) als erste Sektion verankert; Anti-Gaming / REQ-QA-010 gehärtet; Systemregeln in prägnanter Kurzform mit Paragraphenverweisen (§0–§34) konsolidiert. |

---

## Verbleibende Vorbehalte & Ununsichere Stellen

1. **Persistenter KV-Prefix-Cache (M1 / B1–B3, §13.1):** Der In-Memory-Prefix-Store ist im Code vorhanden, die persistenten Segmentdateien und das B1–B3 Tiering (§13.1) verbleiben im Zielzustand und sind in der Dokumentation entsprechend als Opt-in/Zielzustand ausgewiesen.
2. **Qualitäts-Gate-Status im Repository:** Dokumentations-Linter/Gates im Repository weisen unabhängig von Lane L1 bestehende Vorerkrankungen auf (z. B. verwaiste Crate-Referenzen in Inline-Analyse-Dateien oder veraltete Datum-Header in Hilfs-Dokumenten). An Produktionscode-Logik oder Gate-Skripten wurden vereinbarungsgemäß (Regel 7) keine Änderungen vorgenommen.

---

*Stand: 2026-10-25 — Erstellt für Lane L1 (Root-Dokumente) unter Spezifikation 1.2.*
