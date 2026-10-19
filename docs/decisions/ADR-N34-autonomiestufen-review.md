# ADR-N34: Autonomiestufen, Risiko-Tiers und automatisierte Review-Schleife

* **Status:** Accepted
* **Datum:** 2026-10-19
* **Autoren:** Principal Architect (Contextra)
* **Betroffene Komponenten:** Governance, GitHub Integration, Agenten-Harness (`.jules/`), `.github/CODEOWNERS`
* **Spezifikationsreferenzen:** `.jules/prompter-tiers.toml`, `.github/CODEOWNERS`, ADR-N30

---

## 1. Ist-Zustand & Befund (mit Pfad- und Zeilennachweisen)

Eine Untersuchung der Steuerung der Autonomiegrade von Agenten sowie der Review-Mechanismen ergibt folgende Lücken:

1. **Unvollständige Zuordnung von Autonomie-Klassen:**
   - In `.jules/prompter-tiers.toml` existiert eine Einteilung von Aufgaben in Tiers und Risikoklassen (`risk: sec | crash | simd | utf8 | fusion | concurrency | calib | none`).
   - Es fehlte jedoch eine verbindliche Regelung, welche Kombinationen aus Tier, Risiko und Pfadschutzkategorie autonom ohne menschliche Zwischenprüfung (Plan-Freigabe, Auto-Merge) ausgeführt werden dürfen.

2. **Inkonsistente Review-Feedback-Schleife bei PRs:**
   - Erhält ein Agent in einem bestehenden Pull Request automatisierte Verdict-Ergebnisse oder Review-Kommentare, fehlte ein normierter Workflow, wie der Agent auf Basis des sticky Verdict-Kommentars selbstständig Korrekturen vornimmt.

3. **Inaktive Handle-Zuweisung in `.github/CODEOWNERS`:**
   - `.github/CODEOWNERS` ist unvollständig konfiguriert und verweist teilweise auf generische Platzhalter statt auf den tatsächlichen Repository-Owner (z. B. `@tfufuz1`).

---

## 2. Entscheidung & Soll-Zustand

1. **Risikobasierte Plan-Freigabe und Auto-Merge:**
   Die Autonomie von Jules-Sessions wird streng anhand der Parameter **Tier**, **Risk** und **Path-Protection** gestaffelt:
   - **Vollautomatisch (Auto-Plan & Auto-Merge):**
     Gilt ausschließlich für Aufgaben mit **Tier 1 UND `risk = "none"`**, die **keine geschützten Pfade** (`protected-paths.toml`) berühren. Nach grünem PR-Verdict erfolgt der Merge automatisch.
   - **Manuelle Plan-Freigabe & Manuelles Review erforderlich:**
     Alle Aufgaben mit **Tier 2 oder Tier 3**, oder mit **`risk != "none"`**, oder die **geschützte Pfade** (`protected-paths.toml`) verändern, erfordern:
     1. Vorab-Freigabe des Ausführungsplans durch den Maintainer.
     2. Manuelles Review und expliziten Merge-Approval durch den Maintainer.

2. **PR-Überarbeitung via Sticky Verdict-Kommentar:**
   - Schlägt ein CI-Lauf fehl oder fordert der Maintainer Änderungen, analysiert der Agent den **sticky Verdict-Kommentar** am PR.
   - Der Agent nimmt die geforderten Anpassungen im bestehenden Branch/PR vor und stößt den Re-Test an, ohne einen neuen PR zu eröffnen.

3. **CODEOWNERS im Solo-Betrieb auf echte Handles:**
   - `.github/CODEOWNERS` wird auf produktive, reale Maintainer-Handles (primär `@tfufuz1`) umgestellt, um klare Eigentümerschaften für geschützte Pfade und Manual-Reviews sicherzustellen.

---

## 3. Konsequenzen

* **Positiv:**
  - Hohe Entwicklungsgeschwindigkeit für kleine, risikofreie Dokumentations- und Governance-Tasks (Tier 1 + `risk=none`).
  - Maximale Sicherheit gegen unbeabsichtigte Regressionen in kritischen Komponenten (SIMD, Crypto, Concurrency) und geschützten Pfaden.
  - Eindeutige Zuweisung von Codeowners für Reviews.
* **Negativ:**
  - Manuelle Wartezeiten für Tasks mit erhöhtem Risiko oder Tier ≥ 2.

---

## 4. Verworfene Alternativen

* **Globale Vollautonomie für alle Tasks:**
  Verworfen, da Änderungen an Unsafe-Isolierung, Crypto-Shredding oder Core-Persistenz ohne menschliche Prüfung unkalkulierbare Risiken bergen.

---

## 5. Migrationsschritte

1. Hinterlegen der Autonomie-Matrix im Agenten-Harness (`xtask`).
2. Implementierung der Feedback-Loop-Logik zur Auswertung des sticky Verdict-Kommentars.
3. Aktualisierung von `.github/CODEOWNERS` mit echten Maintainer-Handles (`@tfufuz1`).
