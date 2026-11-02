# AGENTS – contextra-store

Abgeleitet aus [`docs/SPEC.md`](../../docs/SPEC.md). Bei Widerspruch gilt die SPEC.

**Zweck:** LSM-Speicher: WAL, Memtable, SSTables, Manifest, Kompaktion, Purge-Primitive.

**Ring und Abhängigkeiten:** Ring 2. Erlaubt: `types`, `ports`, `sys`, `crypto`, `mvcc`.

**Verboten:** `text`, `vector`, `erasure` (AR-2); `Option<KeyManager>` und Klartextpfade (D-05); direkter Zugriff auf `std::fs` (D-04); WAL-Rewrite (D-73); Sentinel `unwrap_or(u64::MAX)`; `mmap` (D-94)

**Entscheidungen:** D-05, D-09, D-31, D-57, D-73, D-78, D-85, D-94

**Tests:** T-ISO-1, T-ISO-2, T-WAL-1, T-WAL-2, T-ENC-1 bis T-ENC-3, T-CRASH-1 bis T-CRASH-6, T-GATEGROUP-1, T-COMPACT-RESTRICT-1

**Befunde zu prüfen:** F-06, F-09, F-13, F-21, F-32, F-29
