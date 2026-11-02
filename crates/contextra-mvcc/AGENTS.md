# AGENTS – contextra-mvcc

Abgeleitet aus [`docs/SPEC.md`](../../docs/SPEC.md). Bei Widerspruch gilt die SPEC.

**Zweck:** Sequenzen, Snapshots und SSI-Validierung für Mehrversions-Nebenläufigkeit.

**Ring und Abhängigkeiten:** Ring 2. Erlaubt: `types`, `ports`.

**Verboten:** Sentinel `u64::MAX` als "alles sichtbar" (I-12), Abhängigkeiten zu `sys`, `store`, `vector`, `text` oder `engine`

**Entscheidungen:** D-11, D-61

**Tests:** T-SNAP-1, T-VER-1, I-12 (Prüfung über `min_read_snapshot`)

**Befunde zu prüfen:** F-07, F-29
