# AGENTS – ctx-verify

Abgeleitet aus [`docs/SPEC.md`](../../docs/SPEC.md) (8.17, 12, D-13, D-72, E-L11, Anhang D, F.10). Bei Widerspruch gilt die SPEC (D-98). Gemeinsame Regeln: [`AGENTS.md`](../../AGENTS.md).

**Zweck:** Eigenständiger Verifizierer für Löschberichte und Feldscans.

**Ring und Abhängigkeiten:** Ring dev (Binärprogramm). Abhängig weder von `store`, `engine`, `erasure` noch von Produktions-Decodern in `crypto` (AR-4); nutzt das Scanner-Modul aus `contextra-lab`.

## Fachprofil (Arbeitsrolle)

Forensik- und Verifikationsingenieur (unabhängige Prüfinstanz, Supply-Chain-Denken). Dieses Profil beschreibt Arbeitsweise und Fachwissen und begründet keine eigene Anforderung.

- Feldscan (MK1 Struktur, MK2 Rohscan, MK3 Schlüsselabwesenheit), Positivkontrolle vor jedem Negativbefund.
- Ed25519-Prüfung, kanonische JSON-Prüfung, Ledger- und Restrisiko-Validierung, Exit-Codes für Automatisierung.

## Normative Anker

- **`ctx-verify report <bericht.json> --pub <report.pub> [--ledger <datei>]`:** prüft kanonische Serialisierung, Ed25519-Signatur, Hash-Kette der Journal-Reduktion, Zeugen-Konsistenz (jeder Zeuge hat eine Positivkontrolle), Vollständigkeit der Restrisikoliste gegen den Katalog (Anhang D), Zeitordnung. Ausgabe `VALID` oder `INVALID(code)`; Exit-Code 0 bzw. 2 (8.17).
- **`ctx-verify scan <datadir> --passphrase-fd N --target <ziel>`:** unabhängige Feldprüfung mit eigenen Parsern und Krypto-Aufrufen (D-72, AR-4); Tags werden aus `K_master` abgeleitet, nicht aus Berichtsdaten übernommen; MK1 über alle Strukturen, MK2 mit Klartext-Needles des Betreibers, Positivkontrolle.
- Der öffentliche Schlüssel `report.pub` ist ein expliziter Parameter; eine im Datenverzeichnis liegende Schlüsseldatei beweist nichts (D-13).
- Die Signatur beweist Integrität und Herkunft aus der Installation, nicht die Wahrheit des Inhalts (E-L11); dieser Satz steht im Bericht.
- Kein Netzwerk, keine Secrets in Ausgaben (HR-1, HR-4).

## Verboten

Import von Kernlogik, die den Befund verfälschen könnte; Negativbefund ohne bestandene Positivkontrolle; Passphrase über Argumente oder Umgebung statt Dateideskriptor; Secrets oder Inhalte in Ausgaben; `unsafe`.

## Abnahme

`T-PROOF-1` (gültige Berichte akzeptiert, jede Manipulation abgelehnt: Signatur, Kette, fehlende Positivkontrolle, fehlendes Restrisiko), `T-RAW-1` bis `T-RAW-6`, `T-MUT-1` (eingepflanzte Restdaten werden gefunden), Golden-Berichte, Fuzz der Parser (`fuzz/`).

## Arbeitspakete

WP-4.4. Statusquelle: `.agents/jules/tasks/BACKLOG.md`.

## Prüfbefehle

```bash
cargo test -p ctx-verify --locked
cargo test -p contextra-arch-tests --locked
```
