# MemFuse — Audit Intake Verification Protocol (`AUDIT_INTAKE_PROTOCOL.md`)

> **Regel (AGENTS.md §4)**: Jeder Befund ("Finding") aus einem extern zugelieferten Audit-Dokument, Prompt oder Review-Bericht MUSS vor der Implementierung am AKTUELLEN Quellcode verifiziert werden.

---

## 📋 Verifikations-Ablauf (Schritt-für-Schritt)

1. **Datei & Zeile öffnen**:
   - Die im Finding genannte Datei an der angegebenen Zeilennummer im AKTUELLEN Quellcode öffnen.
   - Nicht auf historische Prompts oder alte Audit-Berichte verlassen — der Code könnte zwischenzeitlich refactored worden sein.

2. **Invariante & Zustand prüfen**:
   - Existiert das gemeldete Problem tatsächlich noch im aktuellen Stand?
   - Liegt die genannte Stelle vielleicht in Test-Code (`#[cfg(test)]` / `tests/`), der von den Produktionsregeln ausgenommen ist (z.B. `.unwrap()` in Unit-Tests)?
   - Wurde das Problem bereits durch einen früheren PR oder Refactoring behoben?

3. **Kategorisierung & Dokumentation im PR**:
   - **Falls AKTIV**: Problem beheben, mit passendem `AI-TAG` versehen und Test/Assertion hinzufügen.
   - **Falls ENTKRÄFTET / OBSOLET**:
     - Den Finding im PR-Kommentar oder Session-Log explizit als **`[ENTKRÄFTET]`** markieren.
     - Begründung beifügen (z.B. *"Zeile 142 liegt in #[cfg(test)] Modul"*, *"Refactored in Commit abc1234 — Typ existiert nicht mehr"*).
     - **NIEMALS** blind fiktiven Code schreiben oder unzutreffende Findings stillschweigend abarbeiten.

---

## 🚫 Anti-Patterns (Verboten)

- ❌ **Blind-Implementierung**: Codeänderungen vornehmen, ohne die betreffende Datei vorher geöffnet zu haben.
- ❌ **Stilles Ignorieren**: Einen unzutreffenden Finding einfach wegzulassen, ohne im PR zu erklären, warum er unzutreffend war.
- ❌ **Copy-Paste von Stale-Audit-Prompts**: Veraltete Audit-Texte unbereinigt in neue Aufgaben-Prompts übernehmen.

---

## 🔒 Verdict-Unabhängigkeitspflicht (Audit-Workflow v2)

Jedes Audit-Ergebnis und `VERDICT:`-Tag in Audit-Reports unter `docs/audits/*.md` unterliegt der strikten Zwei-Session-Unabhängigkeitspflicht.

### 1. Status & VERIFIED-BY-SESSION Invariante
- **DRAFT-Status (`VERIFIED-BY-SESSION: PENDING`)**:
  Erstellt ein Agent einen Audit-Report oder einen Befund, wird das Feld `VERIFIED-BY-SESSION` zwingend auf `PENDING` gesetzt. Solange `VERIFIED-BY-SESSION: PENDING` steht, gilt der Report als **DRAFT** und darf in keinem PR-Body, keiner Commit-Message und keiner anderen Dokumentation als abgeschlossene Verifikation zitiert werden.
- **FINAL / VERIFIED-Status (`VERIFIED-BY-SESSION: <hash>`)**:
  Ein `VERDICT: GO/APPROVED` in einem Audit-Report gilt erst dann als endgültig ("VERIFIED"), wenn das Feld `VERIFIED-BY-SESSION` den 8-stelligen Hex-Hash einer zweiten Session trägt (`VERIFIED-BY-SESSION: <zweitsession-hash>`).

### 2. Bedingungen an die verifizierende Zweitsession
Die zweite Session MUSS zwingend folgende Kriterien erfüllen:
1. **Sitzungs-Identität**: Die SESSION-ID (`VERIFIED-BY-SESSION: <hash2>`) darf NICHT identisch mit der SESSION-ID des ursprünglichen Audit-Autors (`SESSION: <hash1>`) sein (`hash2 != hash1`).
2. **Chronologische Reihenfolge**: Die zweite Session muss zeitlich NACH der ersten Session stattfinden.
3. **Unabhängigkeit**: Die zweite Session muss den Befund unabhängig (ohne Kenntnis des Drafts als Prämisse) am Quellcode nachvollzogen und in ihrem eigenen PR-, Issue- oder Log-Eintrag explizit bestätigt haben.

### 3. Audit-Workflow Schritte (Kanonische Definition)
1. Automatisches Audit-Issue wird erstellt oder manueller Audit-Auftrag gestartet.
2. Agent führt statische/dynamische Analyse durch und erstellt den Draft-Report mit `VERIFIED-BY-SESSION: PENDING`.
3. **Draft-Sperre**: Der Draft-Report darf vom Ersteller-Agenten NICHT selbst als VERIFIED markiert oder freigegeben werden.
4. **Stichproben- & Unabhängigkeitspflicht**: Jeder Audit-Zyklus erfordert eine unabhängige Bestätigung durch mindestens einen der folgenden Schritte:
   a) Unabhängige Review des Reports durch eine zweite Agenten-Session (mit `VERIFIED-BY-SESSION: <hash2>`), ODER
   b) Cross-Crate-Invarianten-Test durch einen zweiten Agenten mit separatem System-Prompt (anderer Kontext, kein Zugriff auf den Draft-Report des ersten Agenten), ODER
   c) Manuelle Code-Review und Bestätigung durch einen menschlichen Reviewer, ODER
   d) Nachgewiesenes Fuzzing-/Property-Test-Ergebnis als Gate.
5. Erst nach erfolgreichem Vollzug von Schritt 4 darf `VERIFIED-BY-SESSION` von `PENDING` auf den Hash der verifizierenden Zweitsession aktualisiert werden.
