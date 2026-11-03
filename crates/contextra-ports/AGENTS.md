# AGENTS – contextra-ports

Abgeleitet aus [`docs/SPEC.md`](../../docs/SPEC.md) (8.2, 5.3, D-03, D-04, D-53, D-85, D-88, D-96). Bei Widerspruch gilt die SPEC (D-98). Gemeinsame Regeln: [`AGENTS.md`](../../AGENTS.md).

**Zweck:** Schnittstellen (Traits) zwischen den Schichten: Speicher, Index, Dateisystem, Zeit, Zufall, Löschschicht.

**Ring und Abhängigkeiten:** Ring 0. Erlaubt: `types`.

## Fachprofil (Arbeitsrolle)

Staff Engineer für Hexagonal-Architektur und deterministische Simulationstests (Denkschule: FoundationDB-Simulation, TigerBeetle-VOPR). Dieses Profil beschreibt Arbeitsweise und Fachwissen und begründet keine eigene Anforderung.

- Trait-Design: Objektsicherheit, Fehlertypen ohne Informationsverlust, keine Leaks interner Typen, Stabilität über Semver.
- Vertrags-Semantik von Dateisystemoperationen (Atomarität von `rename`, Sichtbarkeit nach `fsync`, Fehler-Vergiftung), so spezifiziert, dass `FaultVfs` sie exakt nachbilden kann.
- Dependency Inversion unter den Ringregeln: Ports hängen nur an Ring 0.

## Normative Anker

- **Portliste (8.2):** `StorageEngine`, `TextIndex`, `VectorIndex`, `Vfs`, `Clock`, `Rng`, `IdGen`, `ErasureLayer`, `ReadGate`, `RecordSealer`, `FileKeyring`, `LedgerSink`, `Metrics`. Altports entfallen (D-24).
- **Dispatch (D-03):** Heißpfad-Ports (`StorageEngine`, `TextIndex`, `VectorIndex`) statisch dispatcht (`async fn` in Traits bzw. `-> impl Future + Send`). Kalte Ports und `Vfs` dyn-fähig (`BoxFuture`). Kein `block_on`.
- **Ein `Vfs`-Trait** im Workspace (D-04, T-LINT-2). Positionelles Lesen ersetzt `mmap` (D-94).
- **`ErasureLayer`:** `id`, `resolve`, `on_gate`, `purge`, `witness` (nur lesend, mit `&Scanner<Armed>`), `key_coverage` (8.2, D-69). **`FileKeyring`:** `create`, `open`, `destroy` (Slot nullen, `sync`, Fehlen prüfen; D-85).
- **Löschports vor Implementierungen (D-96):** `ErasureLayer`, `ReadGate`, `RecordSealer`, `FileKeyring`, `LedgerSink` und Löschtypen entstehen hier und in `types`, bevor `store`, `text`, `vector` sie implementieren.
- **Port für Arena-Sperren (D-88 (2)):** `mlock`/`madvise` der Vektorarena läuft über einen Port aus diesem Crate, implementiert in `engine`.
- **Re-Exporte:** `types`-Typen nur einzeln und nur, wenn sie in Trait-Signaturen vorkommen; kein `pub use …::*` (AR-10, T-ARCH-6).

## Entscheidungshilfen

- Beschreibt die Methode Verhalten oder Implementierung? Ports beschreiben nur Verhalten.
- Hat jede Methode dokumentierte Fehler- und Crash-Semantik? Ohne sie kann `FaultVfs` die Methode nicht exakt simulieren.
- Gehört ein Typ nur in eine Implementierung? Dann nicht hierher.
- Testhilfen wie `contract_tests` gehören in `contextra-testkit`, nicht in dieses Crate (F-36).

## Verboten

Implementierungen, Glob-Re-Export `pub use contextra_types::*`, die Ports `embedding`, `reranker`, `query_rewriter`, `license` (D-24, D-23), ein zweites `Vfs`, Feature-Schalter.

## Abnahme

Conformance-Suiten im `testkit` laufen identisch gegen Real-, In-Memory- und Fault-Implementierung (`tests/tk_conformance_*.rs`), `T-ARCH-3`, `T-ARCH-6`, `T-LINT-2`, Conformance von `RecordSealer`/`FileKeyring` (D-53).

## Arbeitspakete

WP-1.1 (Altports), WP-1.4 (ein `Vfs`), WP-3.1 (Löschports, D-96). Statusquelle: `.agents/jules/tasks/BACKLOG.md`.

## Prüfbefehle

```bash
cargo test -p contextra-ports --locked
cargo test -p contextra-arch-tests --locked
```
