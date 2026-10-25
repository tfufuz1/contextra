# Contextra — Abweichungsbericht Lane L2 (`docs/`)

**Stand:** 2026-10-25
**Bezug:** CONTEXTRA_SPEZIFIKATION.md (Fassung 1.2) / CONTEXTRA_IMPLEMENTIERUNGSPLAN.md (Fassung 1.2)
**Lane:** L2 (`docs/`)

---

## 1. Übersicht & Ziel
Dieser Bericht erfasst im Rahmen der Lane L2 alle Identifizierungen veralteter Dokumente, veralteter Spezifikationsfassungen, offener rechtlicher Prüfpunkte (P-01 bis P-14) und unklarer Stellen in der Dokumentation unter `docs/`.

---

## 2. Zur Entfernung empfohlene veraltete Spezifikations-Fassungen (Maintainer-Schritt)

Gemäß Lane-L2-Vorgabe (Punkt 2) wird empfohlen, die folgenden veralteten Kapitel- und Gesamtspezifikationsdateien physisch aus dem Repository zu entfernen, um Doppelführung und Verwirrung zu vermeiden. Sie wurden in dieser Überarbeitung durch Verweise auf die kanonische **Spezifikation 1.2** (`docs/CONTEXTRA_SPEZIFIKATION_1.2.md`) ersetzt.

1. `docs/GESAMTSPEZIFIKATION.md`
2. `docs/spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md`
3. `docs/spec/CONTEXTRA_SPEZIFIKATION_v15.md`
4. `docs/spec/00-meta-workspace-layout.md` bis `docs/spec/92-anhang-d-aenderungsprotokoll-v4.md` (alle 34 Kapiteldateien in `docs/spec/`)
5. `docs/archive/spec/*` (historische Spec-Entwürfe v2)
6. `docs/FEATURES_SPECIFICATION.md`

---

## 3. Übersicht behobener Abweichungen / Verweis-Aktualisierungen je Spezifikationsabschnitt

| Spezifikationsabschnitt | Art der Anpassung | Anzahl angepasster Dateien / Stellen |
|---|---|---|
| **§0.0 / §28.5** (Quellen der Soll-Architektur & Single Source of Truth) | Ersatz veralteter Spec-Texte durch Zeiger auf `docs/CONTEXTRA_SPEZIFIKATION_1.2.md` | 38 Dateien in `docs/spec/` & Root `docs/` |
| **§3.0 / §5.0** (Phasenbeschreibungen & Invariantenzuordnung) | Aufnahme des Implementierungsplans 1.2 als kanonische Arbeitsunterlage `docs/CONTEXTRA_IMPLEMENTIERUNGSPLAN_1.2.md` | 1 Datei (`docs/CONTEXTRA_IMPLEMENTIERUNGSPLAN_1.2.md`) |
| **§31.0 / §20.12** (Terminologie & Claim-Taxonomie) | Auditierung verbotener Begriffe im Ist-Zustand (`WalNoHmac`, `DeletionProof v3`) & Abgleiche | Auditierung über alle Unterverzeichnisse in `docs/` |

---

## 4. Offene Fragen & Rechtliche Prüfpunkte (P-01 bis P-14)

Gemäß §20.13 der Spezifikation 1.2 sind rechtliche Wertungen als Prüfpunkte geführt und bleiben bis zur Abnahme durch eine fachkundige Stelle offen:

- **P-01**: Rechtsnatur des Löschanspruchs (Art. 17 DSGVO vs. Löschung in KI-Speichersystemen).
- **P-02**: Anwendbarkeit des Standes der Technik (Art. 32 DSGVO) auf kryptografisches Löschen (Crypto-Shredding).
- **P-03**: Anforderungen an die Nachweisbarkeit gegenüber Aufsichtsbehörden.
- **P-04 bis P-14**: Spezifische Prüfpunkte zu Mandantentrennung, Aufbewahrungsfristen, Backup-Remanenz und Drittlandübermittlung.

---

## 5. Summary & Status

- **Status Lane L2**: Alle Dateien unter `docs/` (außer `docs/generated/**` und `docs/status/**`) wurden gegen Spezifikation 1.2 geprüft.
- **Single Source of Truth**: `docs/CONTEXTRA_SPEZIFIKATION_1.2.md` ist die einzige verbindliche Quelle der Soll-Architektur.
- **Implementierungsplan**: `docs/CONTEXTRA_IMPLEMENTIERUNGSPLAN_1.2.md` führt die Phasen PH0–PH9 mit Abnahmekriterien und Invarianten.
