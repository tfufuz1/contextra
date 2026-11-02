# AGENTS – contextra-testkit

Abgeleitet aus [`docs/SPEC.md`](../../docs/SPEC.md). Bei Widerspruch gilt die SPEC.

**Zweck:** Testhilfen: `FaultVfs`, Referenzmodell, deterministische Uhr, In-Memory-Ports.

**Ring und Abhängigkeiten:** Ring dev. Erlaubt: nach AR-14 frei für Dev-Abhängigkeiten; Produktionscrates dürfen nicht von `testkit` abhängen.

**Verboten:** Feature-Schalter zum Aktivieren (D-84); Selbst-Dev-Abhängigkeit zum Aktivieren von Features (D-97)

**Entscheidungen:** D-84, D-97; `FaultVfs` für Crash-Matrix (SPEC 15.1)

**Tests:** T-CRASH-1 bis T-CRASH-6, Conformance-Suites, `ReferenceModel`

**Befunde zu prüfen:** F-02 (Dev-Targets scheitern in 3.0.0)
