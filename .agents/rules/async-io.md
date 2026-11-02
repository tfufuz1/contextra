# Asynchrone I/O und Laufzeit

Abgeleitet aus SPEC 5.4, 14.2, D-03, D-54, I-14, T-LINT-2.

- **Runtime.** Tokio multi-thread. Die Bibliothek erzeugt keine eigene Runtime. `EngineBuilder::runtime(Handle)` ist Pflicht.
- **Features.** Nur `rt-multi-thread`, `sync`, `time`, `macros` im Workspace (D-54). Keine `net`, `fs`, `process`, `signal`, `io-std`.
- **Cancel-Safety (I-14).** Jede mutierende Operation läuft in einem vom Aufrufer unabhängigen Task. Der Aufrufer wartet auf ein `oneshot`. Abbruch des Aufruf-Futures ändert den Ausgang nicht (T-CANCEL-1).
- **Kein `block_on`** im Bibliothekscode (D-03).
- **Keine Sperre über `.await` fremder Tasks.** `await_holding_lock = deny`. Die Sperrreihenfolge lautet: Dokumentlock, Commit-Barriere, Purge-Barriere, Journal.
- **Kein Blockieren in `Drop`.** `Drop` setzt Stop-Flags. Der geordnete Shutdown läuft über `close().await` (14.3).
- **I/O nur über `Vfs`** (D-04). Keine direkten Dateizugriffe, kein `tokio::fs`.
- **Hintergrundtasks** besitzen Stop-Flag und `JoinHandle`.
