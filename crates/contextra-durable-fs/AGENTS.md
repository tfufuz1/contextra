# AGENTS – contextra-durable-fs

Abgeleitet aus [`docs/SPEC.md`](../../docs/SPEC.md). Bei Widerspruch gilt die SPEC.

**Zweck:** Dateisystem-Implementierung `RealVfs` mit dauerhaften Schreib- und Austauschoperationen.

**Ring und Abhängigkeiten:** Ring 1. Erlaubt: `types`, `ports`, `sys`.

**Verboten:** eigener `Vfs`-Trait, `unsafe` außerhalb von `sys`, `mmap`, `std::net`, Netzwerk-Bibliotheken, `scrub` auf Flash/COW ohne Prüfung (D-83)

**Entscheidungen:** D-04, D-50, D-55, D-56, D-83

**Tests:** T-PLAT-1, T-LOCK-1, T-CRASH-1 bis T-CRASH-6, T-FSYNC-1

**Befunde zu prüfen:** F-13, F-29, F-34
