Sequenzen, Snapshots und SSI-Validierung für Mehrversions-Nebenläufigkeit.

## Status

Ring 2. Ist-Zustand: vorhanden, 47/47 Lib-Tests in 3.0.0; Sentinel `u64::MAX` in `snapshot.rs:314` (`update_min`) widerspricht 8.7. Soll-Zustand nach [`docs/SPEC.md`](../../docs/SPEC.md), Abschnitt 5.1 und 8.

## Abhängigkeiten

Erlaubt laut SPEC 5.3: `types`, `ports`.

Das Crate ist Teil der Architekturprüfung (`contextra-arch-tests`). Abhängigkeiten werden nicht von Hand gepflegt, sondern gegen die Matrix geprüft (AR-1, AR-11).
