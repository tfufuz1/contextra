> **Status 30.10.2026 (Reduktion):** Diese Regel gilt weiter, soweit sie nicht auf entfernte Crates oder entfernte Routinen verweist. Solche Stellen sind veraltet. Maßgeblich: `AGENTS.md` und `docs/SPEC.md` (Fassung 4.0).

1. Lies die Crate-AGENTS.md und die Task-Karte.
2. Verifiziere jedes Symbol bevor du es benutzt.
3. Ändere ausschließlich Dateien im `scope` der Karte. Alles andere: unter „Out-of-scope Findings" im PR melden.
4. Ändere niemals Gates, Lints, Baselines, Toolchain oder geschützte Pfade, um Rot zu beseitigen. Behebe die Ursache.
5. Schreibe zuerst einen Test, der fehlschlägt (rot), dann den Fix (grün).
6. Gleicher Fehler zweimal → Vorgehen reflektieren und stoppen.
7. „Fertig" heißt: Tests und Checks (`just check`, `just test`) grün.
