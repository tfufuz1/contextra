# AGENTS – contextra-engine

Abgeleitet aus [`docs/SPEC.md`](../../docs/SPEC.md). Bei Widerspruch gilt die SPEC.

**Zweck:** Orchestrierung: Mandanten, Collections, Transaktionen, Suche, Hintergrundtasks.

**Ring und Abhängigkeiten:** Ring 3. Erlaubt: `types`, `ports`, `sys` (nur Prozesshärtung, D-88), `durable-fs`, `crypto`, `mvcc`, `store`, `text`, `vector`, `rank`, `erasure`.

**Verboten:** Löschlogik außerhalb `erasure` (SPEC 8.13); `block_on` (D-03); `TenantPolicy` (D-07); `AutoExtractionMode` (D-27); Feature-Schalter; Module mit Namen `misc`, `util`, `*_impl` (D-90); Tresor-Erzeugung in der Fassade (D-95)

**Entscheidungen:** D-03, D-07, D-27, D-40, D-41, D-51, D-57, D-62, D-63, D-88, D-90, D-95

**Tests:** T-API-1, T-API-2, T-HYB-1, T-SEARCH-2, T-TENANT-1, T-SLA-1, T-ARCH-7

**Befunde zu prüfen:** F-01 (Fassung 3.0.0, nicht bestätigt), F-08 (Delete-Race), F-10, F-30, F-31
