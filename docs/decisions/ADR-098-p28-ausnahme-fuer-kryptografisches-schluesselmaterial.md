# ADR-098: P28-Ausnahme fuer kryptografisches Schluesselmaterial und reine Dateinamens-Eindeutigkeit

* **Status:** Accepted
* **Datum:** 2026-09-27
* **Kontext / Auslöser:** P28 wurde ursprünglich zur Sicherung reproduzierbarer Agentenläufe und deterministischer Tests eingeführt. Kryptografisches Schlüssel- und Saltmaterial hat ein entgegengesetztes Anforderungsprofil: Vorhersagbarkeit ist hier ein Sicherheitsfehler, keine wünschenswerte Eigenschaft. Ein Architektur-Audit (siehe Gesamtspezifikation Teil X.11) ergab, dass die pauschale Ersetzung von `rand::thread_rng()` durch den injizierten `Rng`-Port an drei Stellen (`wal/hmac.rs:112,117`, `wal/hmac.rs:215`, `lsm/recovery.rs:105`) zu vorhersagbarem Schlüsselmaterial führen und damit die Sicherheit schwächen würde.

## Entscheidung
P28 gilt ab sofort mit einer expliziten, eng gefassten Ausnahme für:
1. **Kryptografisches Schlüssel- und Saltmaterial**, das zwingend `rand::thread_rng()` bzw. einen echten CSPRNG verwenden MUSS statt des injizierten, deterministischen `Rng`-Ports (da dieser in Tests durch einen festen Seed ersetzbar ist und ein darüber erzeugter Schlüssel vorhersagbar wäre).
2. **Reine Dateinamens-Eindeutigkeit** ohne Sicherheits- oder Sichtbarkeitsbezug (z. B. temporäre SSTable-/WAL-Segment-Namen), die `SystemTime::now()` verwenden darf, solange der Wert in keine Sichtbarkeits-, Sequenz- oder Prüfsummenentscheidung einfließt.

Jede Verwendung von Wanduhrzeit, die in einem nutzersichtbaren, deterministisch erwarteten Artefakt landet (z. B. ein `StateCheckpoint`-Zeitstempel), bleibt weiterhin strikt unter P28 und MUSS über `Clock` laufen.

## Begründung
Die strikte Einhaltung von Determinismus ist für Testbarkeit und Reproduzierbarkeit essenziell, darf aber nicht die kryptografische Sicherheit beeinträchtigen. Die Differenzierung trennt deterministische Logikpfade von echten Entropieanforderungen der Kryptografie.

## Konsequenzen
- `AGENTS.md` (Regel O2 / Invarianten-Sektion) wird präzisiert.
- Die Gesamtspezifikation `docs/spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md` (P28-Zeile) wird entsprechend nachgeführt.
- Bestehende Call-Sites in `wal/hmac.rs`, `lsm/recovery.rs`, `wal/flusher.rs` und `compaction/engine.rs` bleiben unverändert und sind damit regelkonform.
