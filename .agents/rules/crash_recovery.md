# Crash-Konsistenz und Wiederanlauf

Abgeleitet aus SPEC 11.2, 13.1 (I-8, I-9, I-9b, I-10, I-15, I-17), 13.2 (FM-1 bis FM-17), 14.1, 15.1, D-55, D-65, D-73, D-83, D-85.

- **Crash-Tabelle.** Jede Mehrschritt-Operation hat eine Tabelle „Abbruch nach Schritt n → Zustand nach Recovery“. Der Test erzeugt jede Zeile mit `FaultVfs` (Abbruch nach jedem `write`, `fsync`, `rename`, `unlink`).
- **Ordnung.** Tresor-Slot dauerhaft vor dem WAL-Frame unter dem DEK (I-9); `F`-Slot vor dem ersten dauerhaften Block (I-9b); `F`-Slot nach dauerhaftem Manifest-Swap, vor `unlink` genullt (I-17); Journal `Requested` vor dem Gate (I-8).
- **`fsync`-Fehler.** Datei fatal, Store `Poisoned`, Recovery beim Neuöffnen; nie Wiederholung auf demselben Handle (D-55, I-10, T-FSYNC-1). Im Löschpfad: `Pending` mit Neuöffnen und Prüfung.
- **Idempotenz.** Jeder Löschschritt ist nach Neustart fortsetzbar (I-15). Fehlt das Gate-Log, wird es aus dem Journal rekonstruiert; fehlt der Journal-Eintrag zum Gate, gilt das Gate trotzdem (11.2, FM-9).
- **Waisen.** `D`- und `F`-Slots ohne Gegenstück werden bei der Recovery genullt; Dateien ohne Manifest-Eintrag entfernt; `tmp/` wird beim Start geleert (D-83, FM-13, FM-14).
- **WAL.** Torn Write am Ende wird toleriert, Korruption in der Mitte verweigert den Start (T-WAL-1); WAL wird nie umgeschrieben (D-73).
- **Platte voll.** Rewrite reserviert vorab; sonst `Pending(DiskReserve)`, Gate bleibt (D-65, FM-3).
