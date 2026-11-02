# contextra

Haupt-Bibliotheksfassade und einzige Composition Root von Contextra.

- **Ring:** Ring 4 (Fassade, Composition Root)
- **Normativ:** [docs/SPEC.md §7, §8.14](../../docs/SPEC.md)
- **Arbeitsanweisungen:** [AGENTS.md](AGENTS.md)

## Rolle & Zweck
`contextra` ist die primäre Fassade und die einzige **Composition Root** des Gesamtsystems (Ring 4). Sie re-exportiert ausschließlich die autorisierte öffentliche API:
- `ContextraBuilder` / `builder`: Konfiguriert und öffnet die Datenbank (`build().await?`).
  - Erzwingt Passphrase für Speicherverschlüsselung (Pflicht gemäß D-05).
  - Stellt System-Defaults und den CSPRNG-Adapter `OsRngAdapter` bereit.
- `Collection`: Dokumentenverwaltung (`insert`, `get`, `scan`, `soft_delete` (D-51)).
- Schnittstellen zur physischen Datenlöschung und Nachweisverifikation über die Kern-Module.

## Invarianten
- `#![forbid(unsafe_code)]`
- Kein anderes Crate im Workspace darf von `contextra` abhängen (AR-1).
- Zero Panic: Alle öffentlichen Funktionen liefern `Result<T, ContextraError>`.
- Pflichtverschlüsselung ohne Klartext-Opt-Out (D-05).
- Sauberes Beenden über `close().await?`; kein Blockieren in `Drop`.
- Veraltete Features (`agent_memory`, `graph`, `license`) sind entfernt.
- Sachliche, überprüfbare Formulierungen ohne Rechtsbehauptungen.
