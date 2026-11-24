# SP-COMMIT — Committer, Guards, Volatile, Idempotenz

| | |
|---|---|
| Modul | `engine/{committer,guard_window,volatile,idem}.rs` |
| Besitzt | einziger Sequencer, Stamp-Vergabe, Change-Validierung, Redo-Commit, Gruppensync, Guard-Ringe |
| Konsumiert | PORTS, REDO, CATALOG, MEMVER, ENV, KEYS |
| Spec-Bezug | R8 §6, §11; INV-STAMP-1/2/3, INV-ATOM-1, INV-COMMIT-2, INV-GUARD-1, INV-VOL-1, INV-LIM-1 |
| Phase | P1–P3 (P1 vor Crash-Sweep) |

## 1 Auftrag
Ein Committer pro Instanz linearisiert alle stamp-relevanten Änderungen einschließlich Gate/Control/Branch. Er nimmt keine Änderung an, bevor Limits, Guards, Restrictions, Clusterzustand und Durabilitätsanforderungen validiert sind. Keine I/O im schnellen Guard-Fenster; kalter Pfad prüft außerhalb kritischer Sperren und validiert anschließend erneut.

## 2 Verträge
`apply(Change) -> Receipt`; `durable_barrier() -> Receipt` synchronisiert Relaxed-Arbeit (E-40). Idem-Ergebnis wird als vorversiegelter SYS_IDEM-Record im selben Redo-Commit publiziert; aktive Records überleben `replay_floor` und werden erst nach TTL plus durablem Trim entfernt (E-48). SYS_IDEM-KeyUnavailable oder Kapazitätsgrenze lässt tokenierte Changes vor Mutation fail-closed scheitern. Token-Reuse mit anderem Request ergibt `IdemConflict`. Group commit: ein Redo-Append und ein Sync pro Gruppe; sichtbare Publikation erst nach Sync für Durable. Fsync-Fehler nie wiederholen (SD-16), Store wird Poisoned.

## 3 Regeln
Limits vor jeder Sperre prüfen. `max_body_bytes ≤ max_change_bytes/2`; `max_change_bytes ≤ redo_segment_bytes/8`; ein Einzelframe darf Gruppenziel übersteigen, nie Segmentgrenze (E-35/E-45). Branch-Operationen versiegeln ihren SYS_BRANCH-Payload vor Rang 1 und validieren Reservierung/Head/Generation im Committer; kein Provider-/Cipher-Aufruf unter Rang 1 (E-46). Guards decken Punkt- und Intervallmutationen einschließlich RangeDelete/EraseRange ab (E-47). `Relaxed` darf kein Redo für volatile/mixed write versprechen; Volatile-Schreiben emittiert kein Redo. Keine Rückrufe unter Sperren. Monotone Stamp-Reservierung; Exhausted statt Wrap.

## 4 Invarianten / Umsetzung
INV-STAMP-1/2/3, INV-ATOM-1, INV-COMMIT-2, INV-GUARD-1, INV-VOL-1, INV-LIM-1, INV-LOCK-1/2. FaultFs-Crash-Sweep ab erstem Write; Loom für Warteschlange, Cancel/Waiter, Poison, Guard-Race; Guard-Range-Bucket-Fehlalarm geht in exakten langsamen Pfad (E-34). Recovery-Properties decken Flush vor Token-Retry, Erase vor Retry und Token-Content-Mismatch ab.

## 5 Arbeitspakete
WP-10.1 Validierung/Limit-Gate + Guard API (P1) · WP-10.2 Sequencer/Stamp-Reserve (P1, CATALOG) · WP-10.3 Redo-Gruppencommit und Publish (P1, REDO/MEMVER) · WP-10.4 GuardWindow + Intervallguards (P3) · WP-10.5 Idempotenz-Space/Trim/Recovery (P1–P3, CATALOG) · WP-10.6 Volatile-Regeln + durable barrier (P3). Abnahme: Crash-Sweep und atomare Mehr-Space-Referenzmodellgleichheit.

## R9.1-Synchronisierung

Committer bezeichnet Sequencer plus Syncer. Intake wird unter Rang 2 angehängt; der Sequencer übernimmt per Leader-Protokoll unter Rang 1 nur vorbereitete RAM-Arbeit, vergibt Stamps und prüft `policy_epoch` sowie Guard-Deltas. Stufe A prüft Guards exakt auf dem Aufruferthread, Stufe B ausschließlich gegen das Konfliktfenster und liefert nur `GuardRetry`. Idempotenz ist ein impliziter `Absent`-Guard auf SYS_IDEM; Ergebnisse bleiben bis TTL plus durablem `idem_trim` unabhängig vom Replay-Floor erhalten.
