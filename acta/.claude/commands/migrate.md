---
description: "Einen Crate aus archive/ nach SPEC ueberfuehren (Argument: Crate-Name, z. B. acta-fs)"
argument-hint: "<crate>"
---
Ueberfuehre `$1` nach dem Skill `acta-archive-migration`. Optional (nur Haiku, Arbeitsregel 1 in CLAUDE.md) fuer Zuarbeit den Subagenten `$1-expert` ein (Datei `.claude/agents/$1-expert.md`; fuer `xtask` heisst er `acta-xtask-expert`).
Pruefe zuerst, dass alle Crates niedrigerer Level gruen sind. Entferne `archive/$1/` erst nach gruener Pruefung und liefere entfernte Dateien als leere Huellen.

Liefere nach Abschluss mit `/deliver`.
