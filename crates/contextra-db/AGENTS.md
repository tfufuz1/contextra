 Die Module umfassen Collection-Verwaltung, Hybridsuche und den flüchtigen Vault (`volatile-vault`).

Harte Regeln:
- Keine neuen Features in diesem Crate. Neue Funktionalität geht nach `contextra` oder in die passende Schicht. Ausnahme: Korrekturen mit Testnachweis.
- Jede öffentliche Funktion, die verschoben wird, bekommt einen Kompatibilitätspfad mit `#[deprecated]` und einer Migrationsnotiz, bevor sie entfernt wird.
- Die Integrationstests (45 Dateien) bilden das Verhaltensnetz. Vor jeder Verschiebung müssen sie grün sein und nach der Verschiebung unverändert grün bleiben.
- Abhängigkeiten sind weit gefächert (`engine`, `graph`, `store`, `vector`, `text`, `checkpoint`, `crypto`, `rank`, `sys`). Jede Migration muss die Zyklenfreiheit prüfen.
- Das Manifest führt `beta`. Der Migrationsstatus wird im Falt-Plan nachgehalten.

Verifikation: `cargo test -p contextra-db --locked`, `cargo test --workspace` nach jedem Migrationsschritt, Abhängigkeitsgraph mit `cargo tree -p contextra`.

Optimierungsfokus: Entfernung von Duplikatlogik, Vermeidung zusätzlicher Indirektion im Suchpfad.
