# Nebenläufigkeit und Sperren

Abgeleitet aus SPEC 5.4, 14.2, 14.3, D-57, D-78, I-14, I-16, T-CONC-1, T-GATEGROUP-1. Ergänzt `async-io.md`.

- **Sperrordnung.** Dokumentlock → Commit-Barriere → Purge-Barriere → Journal. Keine umgekehrte Reihenfolge, keine verschachtelten Shard-Locks.
- **Dokumentlock** gilt nur für `Insert`, `Update`, `soft_delete`, `erase(Document)`. Für Subject, Collection, Tenant gibt es kein Lock-Set (D-57). Ein Lock wird bis zum Ende der geschützten Operation gehalten, nicht vorzeitig fallen gelassen (Delete-Race, F-08).
- **Commit-Barriere** (`RwLock`): Gruppen-Commit shared, Gate-Gruppe exklusiv. Gleichzeitige `erase`-Anfragen teilen eine Barriere-Phase und einen Gate-Log-`fsync` (D-78).
- **Purge-Barriere** (`RwLock`): Kompaktion shared, Purge exklusiv. Die Kompaktion prüft pro Block `purge_waiting` und bricht ab; das Teilergebnis wird verworfen. Keine Hintergrundoperation hält eine wartende Löschung länger als 1 s auf (I-16).
- **Keine Sperre über `.await`** einer fremden Task (`await_holding_lock = deny`).
- **Cancel-Safety (I-14).** Mutierende Operationen laufen in einer vom Aufrufer unabhängigen Task (`spawn` + `oneshot`).
- **Drop** blockiert nie; Stop-Flags; geordneter Shutdown nur über `close().await`.
- **Prüfung.** `loom` für Commit-Barriere, Gate, Purge-Barriere (`RUSTFLAGS="--cfg loom" cargo test -p <crate> --locked`). Entscheidungen async gegenüber Threads werden mit Messung belegt (BM-WRITE-DURING-ERASE, BM-PURGE-WAIT).
