# AGENTS – contextra-types

Abgeleitet aus [`docs/SPEC.md`](../../docs/SPEC.md) (8.1, 9.1 bis 9.3, Anhang C, F.10). Bei Widerspruch gilt die SPEC (D-98). Gemeinsame Regeln: [`AGENTS.md`](../../AGENTS.md).

**Zweck:** Reine Datentypen: IDs, Tags, Schlüssel, Fehler, Löschtypen. Kein I/O.

**Ring und Abhängigkeiten:** Ring 0. Erlaubt: keine anderen Contextra-Crates (AR-5, T-ARCH-1). Kein `async`, kein I/O, keine Zeit, kein Zufall.

## Fachprofil (Arbeitsrolle)

Principal Engineer für Datenmodellierung und Binärformate. Hintergrund: Schlüsselkodierung wie im FoundationDB-Tuple-Layer, Typzustandsmodelle in sicherheitskritischem Rust. Dieses Profil beschreibt Arbeitsweise und Fachwissen und begründet keine eigene Anforderung.

- Ordnungserhaltende, eindeutig dekodierbare Schlüsselkodierung: feste Längen, Präfixfreiheit, Bereichsabfragen, absteigendes Sequenzfeld `!commit_seq`. Typische Fehlerklassen: Endianness, Längenfelder, Nullbytes.
- Rust-Typsystem als Beweiswerkzeug: Newtypes, Typestate, private Konstruktoren, `#[non_exhaustive]`. Ungültige Zustände sind nicht konstruierbar.
- `serde`-Grenzen (Arrays über 32 Bytes brauchen manuelle Implementierungen mit Längenprüfung), `Zeroize`/`ZeroizeOnDrop`, Debug-Ausgabe ohne Geheimnisse.
- Kanonische Serialisierung (stabile Bytes für Signaturen und Hashes), Versionierung von Wire-Formaten.

## Normative Anker

- **Schlüsselraum:** `region(1) ‖ tenant_tag(16) ‖ coll_tag(16) ‖ cluster_tag(16) ‖ doc_tag(16) ‖ family(1) ‖ sub(u32) ‖ !commit_seq(u64)` = 78 Byte, big-endian (9.1, D-31). Region `LOC` und Familie `Locator` entfallen (D-70).
- **Matcher:** nur `Range{lo,hi}` fester Tag-Länge oder `Exact`; keine String-Präfixe, keine variablen Präfixe (D-10, T-ISO-2).
- **Tags:** `BLAKE3-keyed(K_idx, label ‖ eingaben)[0..16]`, Eingaben je Feld mit `u32`-Längenpräfix (9.3, D-21, Anhang C). Typen sind `[u8;16]`-Newtypes.
- **Identität:** `DocRef { subject: Option<SubjectRef>, key: DocKey }`; Suchtreffer als `DocHandle { cluster_tag, doc_tag }`; `DocId(u64)` entfällt (D-08, D-70, D-40).
- **Löschtypen:** `ErasureId`, `EraseTarget`, `LayerWitness`, `ResidualRisk`, `KeyCoverage` (8.1). `LayerWitness` ist nur aus einem `Scanner<Armed>` konstruierbar (I-19, D-72): Konstruktoren privat, Typzustandsmuster, `#[must_use]`.
- **Grenzen:** Kapazitätskonstanten nach D-87 (Dimension ≤ 4096, Query-Terme ≤ 256, `k` ≤ 1000, `max_doc_bytes`).
- **Format:** Jede Änderung eines Schlüssel- oder Berichtsformats ist ein Formatversions-Wechsel mit Golden-Test (D-42, T-FMT-1, Anhang F).
- **Namen:** Kein Modul `types::types`; erlaubt sind `domain`, `error`, `erasure`, `tags` (D-90). Keine `misc`/`util`-Module.

## Verboten

I/O, `async`, Glob-Re-Exporte, Feature-Schalter, `DocId(u64)` und `docid-128` (D-08, D-84), `saos` und `AutoExtractionMode` (D-27, F-31), Fehlertexte mit Nutzerinhalt (I-11).

## Abnahme

Roundtrip- und Ordnungs-Proptests über alle Schlüsselfamilien (T-KEY-1), Golden-Bytes (T-FMT-1), `T-DOCREF-1`, `T-ISO-2`, `T-ISO-3`, `T-ARCH-1`, `T-ARCH-5`, `T-LIMIT-1`. Miri läuft auf diesem Crate (15.1, T-UNSAFE-1). Die Lib kompiliert ohne Warnungen.

## Arbeitspakete

WP-3.1 (Löschtypen, `DocRef`, `DocHandle`, Tags mit Längenpräfix, `KeyCoverage`, `DocId` entfernt), WP-1.1 (`AutoExtractionMode`), WP-1.5 (Namensregeln D-90). Statusquelle: `.agents/jules/tasks/BACKLOG.md`.

## Prüfbefehle

```bash
cargo test -p contextra-types --locked
cargo +nightly miri test -p contextra-types
cargo test -p contextra-arch-tests --locked
```
