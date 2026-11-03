# AGENTS – contextra-engine

Abgeleitet aus [`docs/SPEC.md`](../../docs/SPEC.md) (8.13, 5.4, 5.5, 5.7, 14.1 bis 14.6). Bei Widerspruch gilt die SPEC (D-98). Gemeinsame Regeln: [`AGENTS.md`](../../AGENTS.md).

**Zweck:** Orchestrierung: Mandanten, Collections, Transaktionen, Suche, Hintergrundtasks.

**Ring und Abhängigkeiten:** Ring 3. Erlaubt: `types`, `ports`, `sys` (nur Prozesshärtung, D-88), `durable-fs`, `crypto`, `mvcc`, `store`, `text`, `vector`, `rank`, `erasure`.

## Fachprofil (Arbeitsrolle)

Datenbank-Kernel-Ingenieur für eingebettete Bibliotheken (Orchestrierung, Hintergrund-Worker, Lebenszyklen). Dieses Profil beschreibt Arbeitsweise und Fachwissen und begründet keine eigene Anforderung.

- Collection-Lebenszyklus, Transaktionskoordination über Store, Text und Vektor, Hybridsuche, Wartungsfenster.
- Embedded-Concurrency: Der Host besitzt die Runtime; keine Thread- oder Runtime-Überraschungen, Abbruch und Shutdown ohne Datenverlust, kein Lock über `.await`.
- Konsistenz über mehrere Indizes: atomares Sichtbarwerden, Purge-Reihenfolge je Schicht.

## Normative Anker

- **Engine fügt Schichten zusammen** und enthält **keine** Speicher-, Krypto- oder Löschlogik (8.13): `EngineBuilder`, Instanz (`instance`), `TenantScopedStorage`, Collections, Transaktionen, Suche, Hintergrundtasks, Startsequenz.
- **Laufzeit (5.4):** Tokio multi-thread; die Bibliothek erzeugt keine eigene Runtime, `EngineBuilder::runtime(Handle)` ist Pflicht. Mutierende Operationen laufen in einer vom Aufrufer unabhängigen Task (`spawn` + `oneshot`; I-14, T-CANCEL-1). Hintergrundtasks haben Stop-Flag und `JoinHandle`; `Drop` blockiert nie, geordneter Shutdown nur über `close().await` (14.3).
- **Sperrordnung:** Dokumentlock → Commit-Barriere → Purge-Barriere → Journal (5.4). Dokumentlocks gelten nur für `EraseTarget::Document`; für Subject, Collection, Tenant gibt es kein Lock-Set (D-57). Der Delete-Race (`let _guard = …; drop(_guard);` in `delete.rs`, F-08) ist ein bekannter Fehler.
- **Startsequenz (14.1):** Lock → Startprobe → HEADER und KDF-Grenzen → Prozesshärtung → `tmp/` bereinigen → Gate-Log laden → Journal lesen → Manifest und WAL-Recovery → Tresor öffnen und Waisen bereinigen → Ledger prüfen → Lesepfade freigeben, Hintergrundtasks starten, offene Löschungen fortsetzen. Fehler in den Schritten 1 bis 10 verhindern den Start.
- **Schreib- und Lesepfad (5.5):** `TenantContext` prüfen → `ReadGate` → Dokumentlock → Tags ableiten → Tresor-Slots → Envelope → eine Store-Transaktion mit Gruppen-Commit. Suche mit `ReadGate`-Filter und Nachladen (D-62); `SearchPage { hits, exhausted }`.
- **Tresor:** `engine` erzeugt `FileVault` aus `contextra-erasure` (D-95); die Fassade liefert nur `RealVfs`, Uhr, CSPRNG.
- **Konfiguration (5.7):** Pflichtparameter `data_dir`, `passphrase` (≥ 12 Bytes), `erase_sla`, `backup_retention`, `runtime`, `vfs`, `clock`, `rng`; ungültige oder fehlende Werte sind `ConfigError` beim `open`, keine stillen Korrekturen; `batch_window ≤ erase_sla / 2` (D-39, T-CFG-3).
- **Kapazität und Gesundheit:** `Limit::*` statt stillem Kürzen (D-87); `health()` und `Metrics` (D-64); Plattenreserve (D-65); Prozesshärtung (D-63).
- **Rückbau:** `chunker`, `extraction`, `export`/`import`, `temporal_filter`, `merge_operator_adapter` samt `block_on`, `decay_controller` und halbentfernte Graph-Reste liegen außerhalb des Soll (D-17, D-24, D-84, F-31, F-36) und werden entfernt, nicht repariert.

## Verboten

Löschlogik außerhalb `erasure`; `block_on` (D-03); `TenantPolicy` und mandantenlose Zugriffe (D-07); `AutoExtractionMode` (D-27); Feature-Schalter (`adaptive-candidate-pool-sizing`, `onnx`, `sandbox`, `reranking`, … D-84); Module `misc`, `util`, `*_impl`, `mod.rs` (D-90); Tresor-Erzeugung in der Fassade; Sperren über `.await`.

## Abnahme

`T-API-1`, `T-API-2`, `T-HYB-1`, `T-SEARCH-2`, `T-TENANT-1`, `T-SLA-1`, `T-CANCEL-1`, `T-CONC-1`, `T-LIMIT-1`, `T-OPS-1`, `T-OPS-2`, `T-ARCH-7`; vertikaler Pfad (Schreiben unter SSI-Last), Shutdown- und Restart-Tests, Löschen unter Last ohne Lesen von Gate-Daten.

## Arbeitspakete

WP-1.1, WP-1.5 (`contextra_impl` → `instance`), WP-2.1 (Kompilierbarkeit), WP-5.1 (Collections, `DocRef`-API, Suchstrom, Startsequenz), WP-5.2 (Probe, `health()`). Statusquelle: `.agents/jules/tasks/BACKLOG.md`. Die Fehlerliste F-01 ist eine Momentaufnahme; vor der Arbeit mit `cargo check` neu erheben.

## Prüfbefehle

```bash
cargo check -p contextra-engine --all-targets --locked
cargo test -p contextra-engine --locked
cargo clippy -p contextra-engine --all-targets --locked -- -D warnings
cargo test -p contextra-arch-tests --locked
```
