# contextra

Haupt-Bibliotheksfassade und einzige Composition Root von Contextra.

- **Ring:** Ring 4 (Fassade, Composition Root)
- **Normativ:** [docs/SPEC.md §7, §8.14](../../docs/SPEC.md)
- **Arbeitsanweisungen:** [AGENTS.md](AGENTS.md)

## Rolle & Zweck
`contextra` ist die primäre Fassade und die einzige **Composition Root** des Gesamtsystems (Ring 4). Sie re-exportiert ausschließlich die autorisierte öffentliche API:
- `EngineBuilder`: Sicherer Builder zur Konfiguration und Öffnung der Datenbank (`open().await?`).
  - Erzwingt Tokio-Runtime-Handle.
  - Erzwingt Passphrase für Argon2id KDF ($m \ge 64\,\text{MiB}, t \ge 3, p \ge 1$).
  - Erzwingt Mandantenisolation (`TenantPolicy::Required`).
  - Stellt System-Defaults bereit (`RealVfs`, `SystemClock`, `OsRng`).
- `Collection`: Dokumentenverwaltung (`insert`, `get`, `scan`, `delete`, `search_hybrid`).
- Lösch- und Prüfpfade:
  - `db.erase(target, retention)`: Startet Löschung und liefert `ErasureReceipt` mit Status `Gated`.
  - `db.erasure_status(erasure_id)`: Statusabfrage der Saga.
  - `contextra::verify(report, target)`: Unabhängige Berichts- und Residuenprüfung.

## Invarianten
- `#![forbid(unsafe_code)]`
- Kein anderes Crate im Workspace darf von `contextra` abhängen (AR-1).
- Zero Panic: Alle Funktionen liefern `Result<T, ContextraError>`.
- Pflichtverschlüsselung ohne Opt-Out (D-05).
- Sauberes Beenden über `close().await?`; kein Blockieren in `Drop`.
- Veraltete Features (`audit-export`, `avv-generator`, `candle`, `privacy`, `license`) sind entfernt.
