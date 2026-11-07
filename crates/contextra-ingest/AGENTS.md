# AGENTS.md — `contextra-ingest`

> **Ring 3 (`capabilities.toml`: Ring 4 ≠)** · Klasse `[R]` (Rand) · Reifegrad `experimental` · Unsafe: nein · Rolle: Dokument-Parsing und Text-Extraktion (PDF, DOCX, ODF, E-Mail, Tabellen, Markup) mit Chunking.
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`; diese Datei nennt nur Crate-Spezifisches. Stand 2026-11-07, statisch gelesen, **nicht kompiliert**; Zahlen sind grep-Heuristiken.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-ingest --no-default-features --locked` |
| Contextra-Abhängigkeiten **Ist** (Cargo) | `engine`, `types` |
| Contextra-Abhängigkeiten **Soll** (PLAN §4) | `types` (über `contextra-chunker`) |
| Features | `all-formats`, `fmt-*`, `xml`, `zip-container` |
| Lints | `#![forbid(unsafe_code)]` |

## 2. Abhängigkeiten & Schulden

- **Kante `engine`** (Soll: durch Extraktion von `contextra-chunker` aufheben; das Crate `contextra-chunker` existiert noch nicht, WP 5.4).
- ~20.400 Zeilen, größter Randcrate. **Kein `[K]`-Crate darf `ingest` importieren** (P30, E34).

## 3. Modul-Karte (am Code verifiziert)

| Pfad | Verantwortung |
|---|---|
| `formats/` (`pdf`, `docx`, `pptx`, `odf`, `ole`, `email`, `sheets`, `structured`, `text`, `web`) | Extraktoren |
| `chunking/` (`prose`, `tabular`, `paged`, `email`, `structured`, `fallback`) | Chunking-Strategien |
| `safe/` (`archive`, `opc`, `xml`), `limits.rs` | Zip-Bomb-/Größenschutz, sicheres XML/OPC |
| `pipeline.rs`, `registry.rs`, `detect/`, `ir.rs`, `canonical.rs`, `fuzz_support.rs` | Pipeline, Formaterkennung, Zwischenform |

## 4. Verträge & Invarianten

1. **Alle Eingaben sind untrusted:** Größen-, Tiefen-, Entpack-Limits (`limits.rs`); kein Netz, keine externen Entitäten in XML.
2. **Jeder Extraktor ist Fuzz-Ziel**; Fehler sind typisiert, nie Panic.
3. Jeder Chunk trägt Quellenangabe (Provenienz).
4. Formatparser in Ring 0–3 sind verboten; Ingestion im Kern bleibt auf Text/Markdown beschränkt (SPEC §10.3).

## 5. Ports

Keine Ports (reine Bibliothek).

## 6. Nebenläufigkeit & Performance

- Parser sind synchron und zustandslos; Parallelität auf Dokumentebene durch den Aufrufer.

## 7. Anti-Patterns

- Entpacken ohne Limit.
- `unwrap` auf Parserfehlern.
- Abhängigkeit des Kerns auf `ingest`.

## 8. Tests & Messung

```bash
cargo test -p contextra-ingest --locked
cargo clippy -p contextra-ingest --all-targets -- -D warnings
```

- 26 Testdateien/-module; Fuzz je Format.

## 9. Offene Arbeit

| WP | Inhalt |
|---|---|
| 5.4 | `contextra-chunker` extrahieren, Kante `engine` entfernen |
| 1.1 | Ring angleichen |
