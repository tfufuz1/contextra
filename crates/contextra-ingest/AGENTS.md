# AGENTS.md — `contextra-ingest`

## 1. Rolle
`contextra-ingest` stellt Dokumentformat-Extraktoren und strukturbewusstes Chunking außerhalb des Datenbank-Kerns bereit (SPEC §C-10.3).

## 2. Ring, Klasse, Reifegrad & `may_use`
- **Ring:** Ring 4 (Adapter)
- **Klasse:** Adapter `[A]`
- **Reifegrad:** `experimental`
- **`may_use` (Obergrenze):** `contextra-types`, `contextra-engine` (SPEC §C-10.3, `capabilities.toml`)

## 3. Garantien & Anforderungen
- **Defensive Eingabeverarbeitung:** Zero-Panic-Garantie bei unvollständigen, beschädigten oder bösartigen Quelldateien.
- **Fail-Closed Panik-Isolierung:** `std::panic::catch_unwind` an der Registry-Grenze fängt Parser-Panics ab.
- **Strikte Budgetierung:** Zip-Bomb-Schutz und Speicherlimits über `ResourceBudget` und `IngestLimits`.

## 4. Arbeitsregeln
1 Kein Panic/unwrap/expect/todo!/assert!/Indexing in Produktionscode (get(), checked_*, saturating_*), typisierte Fehler (IngestError), kein unsafe. In Tests erlaubt: #![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing, clippy::panic)].
2 Determinismus (P28): keine Uhr, kein Zufall, keine Umgebung, keine std-HashMap/HashSet.
3 Nur Bytes rein, nichts nach außen: kein Dateisystem, Netz, keine externen Ressourcen, keine Makroausführung.
4 Fail-Closed: verschlüsselt, beschädigt, unbekannte Version, Limit überschritten => benannter Fehler.
5 Versteckter Inhalt nicht extrahieren (Policy), Text über sanitize_text führen.
6 Revier-Treue: nur das eigene Revier ändern; Cargo.toml, Cargo.lock, lib.rs, registry.rs, fuzz/Cargo.toml und geschützte Pfade gehören P00A/P00B; Bedarf darüber hinaus => PR-Abschnitt "Out-of-scope Findings".
7 Wartbarkeit: Dateien <= 800 Zeilen, #[non_exhaustive] auf öffentlichen Enums/Structs, FILE-CONTEXT-Kopf, kein Typname doppelt im Workspace (vorher grep), Doku deutsch, Bezeichner englisch.
8 Test-First; Fixtures klein und im Test per Code erzeugt; keine echten personenbezogenen Daten.
9 Gates vor PR: cargo fmt --all · cargo clippy -p contextra-ingest --all-targets --all-features --locked -- -D warnings · cargo test -p contextra-ingest --all-features --locked · cargo run -p xtask -- check; dann git fetch origin && git rebase origin/main und Gates erneut. Gates/Lints/Baselines nie anfassen.
10 Branch wp/ingest-<kürzel>, Commit "WP-ING-<kürzel>(contextra-ingest): …", ein PR.

## 5. Implementierte & konsumierte Ports
- Implementiert: `DocumentExtractor`, `IrChunker`.
- Konsumiert: `MarkdownChunker` aus `contextra-engine`.

## 6. Tests & Gates
- **Contract Tests:** Integrationstests in `tests/contract_*.rs`.
- **Fuzzing:** Fuzzing-Matrix in `fuzz/`.

## 7. Offene Punkte
- Anbindung der echten Extraktions-Parser in Nachfolge-Arbeitspaketen (P01–P15).
