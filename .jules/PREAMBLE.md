# Contextra — Jules Preamble

1. Führe `cargo xtask jules start --card <datei>` aus. Lies NUR das erzeugte CONTEXT_PACK.md und die Crate-AGENTS.md.
2. Verifiziere jedes Symbol mit `cargo xtask symbol-exists <path>` bevor du es benutzt.
3. Ändere ausschließlich Dateien im `scope` der Karte. Alles andere: unter „Out-of-scope Findings" im PR melden.
4. Ändere niemals Gates, Lints, Baselines, Toolchain oder geschützte Pfade, um Rot zu beseitigen. Behebe die Ursache.
5. Schreibe zuerst einen Test, der fehlschlägt (rot), dann den Fix (grün).
6. Gleicher Fehler zweimal → `cargo xtask jules stop --reason thrash`.
7. „Fertig" heißt: `cargo xtask jules submit` grün. Behaupte nichts, was `session-report` nicht belegt.
