# IMPL-09 — `contextra-sim`, Invarianten-Prädikate, CI-Gates, Roadmap-Abnahme

**Phase:** ab P0 (wächst mit IMPL-03) · **SOLL:** SPEC §17, §18, §19, INV-1…12, INV-D1…D4 · **Crates:** `contextra-testkit`, neu `contextra-sim`, `contextra-bench`, `xtask`

## 1. Ziel

Deterministische Simulation des Kernels: gleiche Seeds ⇒ identischer Ablauf, Fehlerinjektion reproduzierbar. Jede Invariante hat ein ausführbares Prädikat; jedes Paket (IMPL-02…08) gilt erst als fertig, wenn seine Prädikate in CI laufen.

## 2. Bausteine

| Baustein | Beschreibung | Quelle/IST |
|---|---|---|
| `Clock` (Port) | `SystemClock`, `ManualClock` (nur explizit vorstellbar) | neu in `ports` |
| `Rng` (Port) | `SeededRng` (ChaCha8), jeder Zufall im Kernel über Port | neu |
| `FaultVfs` | **Eine** Implementierung: aus `durable-fs::FaultInjectingVfs` und `testkit::FaultVfs` konsolidieren (Duplikat laut IMPL-00) – behalten wird die in `durable-fs`, `testkit` re-exportiert | IST: 2 Varianten |
| Fehlerarten | Short-Write, torn write (512 B/4 KiB), fsync-Fehler, `ENOSPC`, `EIO` beim Read, Bit-Flip, Rename-Verlust (Verzeichnis-fsync fehlt), Crash (Verlust ungesyncter Daten) | |
| `Scheduler` | kooperativer Einzel-Thread-Scheduler für Writer/Worker-Aufgaben; Interleavings aus Seed | neu |
| Workload-Generator | Operationsfolgen (`put/retire/branch/merge/recall`) aus Seed, mit Modellorakel | neu |
| Orakel | einfaches In-Memory-Referenzmodell (Map + Commit-DAG) | neu |

Kernel-Code darf **keine** direkte Zeit-/Zufall-/Dateisystem-API verwenden (CI-Grep: `SystemTime::now`, `Instant::now`, `rand::thread_rng`, `std::fs::` außerhalb `durable-fs` und `sys`).

## 3. Invarianten-Prädikate

Jedes Prädikat: `fn check(&World) -> Result<(), Violation>`; wird nach jedem Simulationsschritt und nach jeder Recovery ausgeführt. Die Zuordnung ist hier **Arbeitsvorlage**; Wortlaut je INV laut SPEC §17 (`OPEN-09-1`).

| INV | Prädikat (Kurzform) |
|---|---|
| INV-1 | Truth ist ohne Indizes vollständig lesbar; Index-Löschung + Rebuild ändert Lese-Ergebnisse nicht |
| INV-2 | Recovery ist idempotent (2× Recovery ⇒ gleicher Zustand) |
| INV-3 | Kein partieller Commit sichtbar; sichtbar = Präfix der committeten Txns |
| INV-4 | Jedes Index-Segment: `source_truth_hash` passt zum referenzierten Truth oder Segment wird als stale erkannt |
| INV-5 | `hash(content) == content_hash` für alle gelesenen Zellen |
| INV-6 | Korruption wird gemeldet, nie still repariert |
| INV-7 | Fold deterministisch (gleicher `root_set_hash`) |
| INV-8 | Kein Merge-Publish ohne bestandene Gates |
| INV-9 … INV-12 | gemäß SPEC §17 (u. a. Trust-Monotonie, Format-Kompatibilität INV-11, Fail-Stop bei fsync INV-12) |
| INV-D1…D4 | Durability-Level-Garantien: `Durable`/`Strict`-bestätigte Commits überleben jeden Crash; `Relaxed`/`Volatile` dürfen verloren gehen, aber nie inkonsistent |

## 4. Simulationsläufe

- **`sim run --seed N --steps M --faults profile`**: Profile `none`, `crash-only`, `io-errors`, `torn`, `chaos`.
- CI: je PR 200 Seeds × 2 000 Schritte (Zeitlimit 10 Minuten auf der VM → ggf. 50 Seeds lokal); nightly 10 000 Seeds. Fehlschlag druckt `seed`, Schritt, Violation und **Replay-Befehl**; `sim shrink` reduziert die Operationsfolge (Delta-Debugging).
- Differenzialmodus: gleiche Workloads gegen Alt-Pfad und `kernel-v2` (Lese-Äquivalenz, siehe IMPL-04/05).
- Linearisierbarkeit der Commit-Reihenfolge: Prüfer auf der Historie (`commit_seq` dicht und monoton je Branch).

