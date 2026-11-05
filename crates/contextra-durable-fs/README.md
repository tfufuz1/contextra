Dateisystem-Implementierung `RealVfs` mit dauerhaften Schreib- und Austauschoperationen.

## Status

Ring 1. Ist-Zustand: Produktionsreif gemäß SPEC 5.1.0 (§8.5, DF-01 bis DF-12); das zweite `Vfs`-Trait ist entfernt (D-04); Startprobe (D-56, D-114) und bedingter `scrub` (D-83) sind vollständig nachgewiesen und getestet. Soll-Zustand nach [`docs/SPEC.md`](../../docs/SPEC.md), Abschnitt 5.1 und 8.

## Abhängigkeiten

Erlaubt laut SPEC 5.3: `types`, `ports`, `sys`.

Das Crate ist Teil der Architekturprüfung (`contextra-arch-tests`). Abhängigkeiten werden nicht von Hand gepflegt, sondern gegen die Matrix geprüft (AR-1, AR-11).
