# AGENTS – contextra-testkit

Abgeleitet aus [`docs/SPEC.md`](../../docs/SPEC.md) (8.15, 15.1, P7, AR-14, D-84, D-97). Bei Widerspruch gilt die SPEC (D-98). Gemeinsame Regeln: [`AGENTS.md`](../../AGENTS.md).

**Zweck:** Testhilfen: `FaultVfs`, Referenzmodell, deterministische Uhr, In-Memory-Ports.

**Ring und Abhängigkeiten:** Ring dev. Dev-Abhängigkeiten folgen der Matrix SPEC 5.3, `testkit` ist davon ausgenommen (AR-14). Produktionscrates dürfen nicht von `testkit` abhängen.

## Fachprofil (Arbeitsrolle)

Test- und Simulationsingenieur (Property-Based Testing, deterministische Simulation, Fehlerinjektion, Mutationstests). Dieses Profil beschreibt Arbeitsweise und Fachwissen und begründet keine eigene Anforderung.

- `FaultVfs`: Abbruch an jedem Syscall, Reordering, Teilpersistenz, `ENOSPC`, `EIO`, `fsync`-Fehler.
- `ManualClock`, deterministischer `Rng`, `ReferenceModel` (naives In-Memory-Orakel), In-Memory-Ports.
- Conformance-Suiten je Port; Benchmark-Harness mit Regressionsgrenzen (15.2).

## Normative Anker

- Crash-Matrix: Abbruch nach **jedem** `write`, `fsync`, `rename`, `unlink` der Saga und des Commit-Pfads plus Fehlerinjektion (15.1, T-CRASH-*, T-FSYNC-1, T-WAL-*).
- Determinismus über Ports (P7): jeder Fehler ist mit einem Seed reproduzierbar.
- Ein Test-Double ist erst gültig, wenn es dieselbe Conformance-Suite wie die echte Implementierung besteht (Arbeitshinweis; belegt durch `tests/tk_conformance_*.rs` und `tests/tk_fault_vfs_conformance.rs`).
- Jeder Test trägt `#[ctx_test(id = "T-…")]` (T-TRACE-1).
- Das Feature `fault-injection` in `store` entfällt zugunsten von `FaultVfs` (D-84).

## Verboten

Feature-Schalter zum Aktivieren (D-84), Selbst-Dev-Abhängigkeit zum Aktivieren von Features (D-97), Abhängigkeit von Produktionscrates auf `testkit`, Zeit- und Zufallsquellen außerhalb von `ManualClock` und seeded `Rng`.

## Abnahme

Selbsttest der Conformance-Suiten; Mutanten im Testkit werden erkannt; `T-CRASH-1` bis `T-CRASH-6`; Referenzmodell-Proptests (`tests/tk_model_mvcc.rs`).

## Prüfbefehle

```bash
cargo test -p contextra-testkit --locked
cargo test --locked --test tk_fault_vfs_conformance --test tk_conformance_storage
cargo test -p contextra-arch-tests --locked
```
