Dateisystem-Implementierung `RealVfs` mit dauerhaften Schreib- und Austauschoperationen.

## Status

Ring 1. Ist-Zustand: vorhanden; das zweite `Vfs`-Trait ist entfernt (F-29); Startprobe (D-56) und `scrub` (D-83) nicht nachgewiesen; das Crate hat im Ist-Zustand keinen Verbraucher. Soll-Zustand nach [`docs/SPEC.md`](../../docs/SPEC.md), Abschnitt 5.1 und 8.

## Abhängigkeiten

Erlaubt laut SPEC 5.3: `types`, `ports`, `sys`.

Das Crate ist Teil der Architekturprüfung (`contextra-arch-tests`). Abhängigkeiten werden nicht von Hand gepflegt, sondern gegen die Matrix geprüft (AR-1, AR-11).
