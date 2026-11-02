# Prüfprotokoll für eingehende Audit-Befunde

Nicht normativ. Maßgeblich: [`AGENTS.md`](../../AGENTS.md) und [`docs/SPEC.md`](../../docs/SPEC.md) (Version 4.2.0).

**Regel:** Jeder Befund aus einem extern gelieferten Audit, Prompt oder Review wird vor jeder Änderung am aktuellen Quellcode geprüft. Nur bestätigte Befunde werden bearbeitet.

## Ablauf

1. **Befund festhalten.** Datei, Zeile, Aussage des Befunds notieren.
2. **Am Code prüfen.** Existieren Datei und Symbol? Trifft die Aussage zu? Befunde auf veraltetem Code werden als „veraltet“ markiert, nicht umgesetzt.
3. **Gegen die SPEC prüfen.** Ist der Befund eine Abweichung vom Soll (Befund-ID `F-nn` aus SPEC 3.2 oder Bezug zu `D-`, `I-`, `P`, `HR-`)? Eine Abweichung vom Soll ist ein Defekt. Eine Aussage, die die SPEC nicht kennt, wird als Rückfrage gestellt.
4. **Einordnen.** Ergebnis: bestätigt, nicht zutreffend, veraltet oder unklar.
5. **Übernehmen.** Nur bestätigte Befunde kommen in ein Arbeitspaket aus SPEC 15.4. Befunde, die SPEC 3.2 schon enthält, werden nicht doppelt angelegt.
6. **Dokumentieren.** Das Ergebnis steht im PR unter „Prüfung der Befunde“.

## Stopp

Ist ein Befund unklar, wird nicht geraten. Die Rückfrage nennt die betroffene SPEC-ID und die Stelle im Code.

## Prüfpfade

Automatisierte Audits (`.github/workflows/scheduled-audit.yml`) verweisen auf dieses Protokoll. Dessen Ausgabe ist genauso zu prüfen wie ein externer Befund.
