# AGENTS – contextra-ports

Abgeleitet aus [`docs/SPEC.md`](../../docs/SPEC.md). Bei Widerspruch gilt die SPEC.

**Zweck:** Schnittstellen (Traits) zwischen den Schichten: Speicher, Index, Dateisystem, Zeit, Zufall, Löschschicht.

**Ring und Abhängigkeiten:** Ring 0. Erlaubt: `types`.

**Verboten:** Implementierungen, Glob-Re-Export `pub use contextra_types::*` (AR-10), Ports `embedding`, `reranker`, `query_rewriter` (D-24), ein zweites `Vfs`

**Entscheidungen:** D-03, D-04, D-24, D-53, D-85, D-88, D-96

**Tests:** T-ARCH-3, T-ARCH-6, T-LINT-2, Conformance-Suite `RecordSealer`/`FileKeyring`

**Befunde zu prüfen:** F-05, F-13, F-29, F-32
