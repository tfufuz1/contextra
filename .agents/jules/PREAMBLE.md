# Grundregeln für Jules

Nicht normativ. Maßgeblich: [`docs/SPEC.md`](../../docs/SPEC.md) (Version 4.3.0), [`AGENTS.md`](../../AGENTS.md) und `docs/VETOES.md`.

## A. Arbeitsmodus: selbstständig entscheiden

1. **Nicht warten, nicht fragen.** Wenn sich eine Frage aus SPEC, AGENTS.md, VETOES.md oder diesem Verzeichnis beantworten lässt, beantwortest du sie selbst. Die Entscheidung steht im PR unter „Getroffene Entscheidungen“ mit Begründung und SPEC-Verweis.
2. **Rangfolge bei Widersprüchen:** `docs/SPEC.md` > `docs/VETOES.md` > `docs/CONSTITUTION.md` > `AGENTS.md` > `.agents/`. Widerspricht ein abgeleitetes Dokument der SPEC, gilt die SPEC, und der Widerspruch wird im PR gemeldet.
3. **Lücke in der Spezifikation:** Wähle die strengste Auslegung, also die mit der geringsten Datenmenge, dem geringsten Schreibumfang und dem stärksten Fehlerschutz (SPEC 6.6, P2). Dokumentiere die Wahl und fahre fort.
4. **Arbeitspakete:** Die Reihenfolge steht in `.agents/jules/tasks/BACKLOG.md`. Ein Paket ohne Karte erstellst du selbst aus `_TEMPLATE.toml` und der Beschreibung in SPEC 15.4. Ein Paket ohne SPEC-Beschreibung wird nicht begonnen; es bekommt den Status `blocked` mit Begründung.
5. **Gleicher Fehler zweimal:** Ändere den Ansatz. Scheitert auch der zweite Ansatz, setzt du das Paket auf `blocked`, schreibst die Ursache in die Arbeitsliste und wechselst zum nächsten freigegebenen Paket. Du hältst die gesamte Sitzung nicht an.

## B. Stopp-Katalog

Anhalten darfst du nur in diesen Fällen. Jeder Fall wird im PR oder in der Arbeitsliste begründet.

- **S1 – Sicherheit:** Ein Test beweist, dass Verschlüsselung, WAL-Integrität, Mandantenisolation oder Löschwirkung verletzt ist, und die Ursache lässt sich nicht innerhalb des Pakets beheben.
- **S2 – Datenverlust:** Ein Schritt würde vorhandene Nutzerdaten oder Dateien außerhalb des Pakets verändern.
- **S3 – Geschützter Pfad:** Das Paket ist nur mit einer Änderung an einem geschützten Pfad vollständig lösbar. Erledige den übrigen Teil, beschreibe die nötige menschliche Änderung im PR und fahre mit dem nächsten Paket fort. Dies ist kein Halt der Sitzung.
- **S4 – Keine Arbeit mehr:** Alle Pakete sind `done` oder `blocked`.

Nicht als Stopp gelten: fehlende Toolchain (installiere sie über `.agents/jules/setup/environment_script.sh`), Testfehler außerhalb des Pakets (melden, nicht beheben), Compiler-Warnungen aus Altbestand (Ratchet, nicht beheben), Zeitüberschreitung eines einzelnen langen Befehls (mit Teilmenge oder `--package` wiederholen).

## C. Fachliche Grundregeln

1. Lies `AGENTS.md`, den Abschnitt der SPEC zur Aufgabe und die `AGENTS.md` des betroffenen Crates.
2. Verifiziere jedes Symbol, bevor du es verwendest. Nimm keine APIs aus dem Gedächtnis an. Prüfe Signaturen mit `grep` oder dem Compiler.
3. Ändere nur Dateien, die zum Paket gehören. Alles andere wird im PR unter „Gefundene Probleme außerhalb des Umfangs“ gemeldet.
4. Ändere niemals Gates, Lints, Baselines, Toolchain-Pins, `governance/`-Dateien oder geschützte Pfade, um Rot zu beseitigen. Behebe die Ursache.
5. Schreibe zuerst einen Test, der fehlschlägt, und dann die Korrektur (D-44, P10).
6. Füge keine neuen Verstöße gegen die harten Regeln hinzu (`unwrap`, `expect`, `panic!`, `let _ =` auf `Result`, `std::fs`, `allow`-Ausnahmen auf Modulebene).
7. **Fertig** heißt, folgende Befehle sind grün, im Paket ohne Ausnahme, und ihre Ausgabe ist im PR genannt:
   - `cargo fmt --all -- --check`
   - `cargo check --workspace --all-targets --locked`
   - `cargo test --workspace --locked` (bei Zeitüberschreitung: betroffene Crates einzeln mit `-p`, Gesamtlauf in der nächsten Sitzung)
   - `cargo test -p contextra-arch-tests --locked`
   Ist ein Befehl wegen Altbestands rot, der nicht zum Paket gehört, notiere das und prüfe, ob das Paket selbst keine neuen Fehler erzeugt. Die Befehle stehen auch in `docs/COMPILATION.md`.
8. Ändert sich `Cargo.toml`, wird `Cargo.lock` im selben Commit erneuert (D-93). Erneuere gezielt (`cargo update -p <crate>`), nicht den gesamten Lockfile. Der Umfang der Änderung (Zeilen, betroffene Abhängigkeiten) gehört in den PR.

## D. Übergangsregel Testmakro

Das Makro `#[ctx_test(id = "T-…")]` ist im Repository noch nicht definiert (Stand SPEC 4.3.0, Prüfung am HEAD `909ff27`). Bis WP-0.1 es liefert:

- Jeder neue Test bekommt die SPEC-ID als Dokumentationszeile direkt über der Funktion: `/// SPEC: T-ISO-1`.
- Der Funktionsname beginnt mit der kleingeschriebenen ID ohne Bindestriche, z. B. `t_iso_1_nachbardaten_bitgleich`.
- Nutze `#[test]` bzw. `#[tokio::test]` wie bisher.
- Bestehende Tests werden in dieser Phase nicht umbenannt. Das ist Teil von WP-0.1.

WP-0.1 ersetzt diese Regel, sobald das Makro existiert. Dann wechselst du alle neuen Tests auf das Makro.

## E. Commit- und PR-Regeln

- Commit-Nachrichten im Format `WP-x.y: <Kurzbeschreibung>`. Nenne die SPEC-IDs im Nachrichtenkörper.
- Vor der Einreichung: `git fetch origin && git rebase origin/<Hauptzweig>`. Löse Konflikte und führe die Prüfbefehle erneut aus.
- PR-Beschreibung: Abschnitte aus `.github/pull_request_template.md` (Arbeitspaket, Änderung, Prüfung, Risiko), plus „SPEC-IDs“, „Getroffene Entscheidungen“, „Testliste“ und „Gefundene Probleme außerhalb des Umfangs“.
- Ein PR pro Arbeitspaket. Mehrere Pakete in einer Sitzung ergeben mehrere PRs, jeweils von einem frischen Stand des Hauptzweigs aus.
