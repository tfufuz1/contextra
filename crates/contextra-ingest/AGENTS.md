# AGENTS.md — `contextra-ingest`

> **Ring 4** · Klasse `[R]` (Rand/Extension) · Reifegrad `stable` · Unsafe: nein · Rolle: Dokument-Parsing und Text-Extraktion (PDF, DOCX, ODF, E-Mail, Tabellen, Markup) mit strukturbewusstem Chunking.
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`; diese Datei nennt nur Crate-Spezifisches. Stand 2026-11-07, verifiziert gegen Paket B24.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-ingest --no-default-features --locked` |
| Contextra-Abhängigkeiten **Ist** (Cargo) | `types` |
| Contextra-Abhängigkeiten **Soll** (PLAN §4) | `types` |
| Features | `all-formats`, `fmt-*`, `xml`, `zip-container` |
| Lints | `#![forbid(unsafe_code)]` |

## 2. Abhängigkeiten & Schulden

- **Kante `engine` vollständig entfernt (Paket B24):** Token-Schätzung (`estimate_tokens`), Window-Splitting (`chunk_text_with_overlap`) und Fallback-Chunking sind autark in `chunking/` implementiert. `contextra-ingest` hängt nur noch an `types`.
- **Kein `[K]`-Crate darf `ingest` importieren** (P30, E34).

## 3. Modul-Karte (am Code verifiziert)

| Pfad | Verantwortung |
|---|---|
| `formats/` (`pdf`, `docx`, `pptx`, `odf`, `ole`, `email`, `sheets`, `structured`, `text`, `web`) | Extraktoren |
| `chunking/` (`prose`, `tabular`, `paged`, `email`, `structured`, `fallback`) | Autarke Chunking-Strategien |
| `safe/` (`archive`, `opc`, `xml`), `limits.rs` | Zip-Bomb-/Größenschutz, sicheres XML/OPC |
| `pipeline.rs`, `registry.rs`, `detect/`, `ir.rs`, `canonical.rs`, `fuzz_support.rs` | Pipeline, Formaterkennung, Zwischenform |

## 4. Verträge & Invarianten

1. **Alle Eingaben sind untrusted:** Größen-, Tiefen-, Entpack-Limits (`limits.rs`); kein Netz, keine externen Entitäten in XML.
2. **Jeder Extraktor ist Fuzz-Ziel**; Fehler sind typisiert (`IngestError`), nie Panic.
3. Jeder Chunk trägt Quellenangabe (Provenienz, Locators, kanonischer Hash).
4. Formatparser in Ring 0–3 sind verboten; Ingestion im Kern bleibt auf Text/Markdown beschränkt (SPEC §10.3).

## 5. Ports

Keine Ports (reine zustandslose Bibliothek / Ingestion-Engine).

## 6. Nebenläufigkeit & Performance

- Parser sind synchron und zustandslos; Parallelität auf Dokumentebene durch den Aufrufer.

## 7. Anti-Patterns

- Entpacken ohne Limit.
- `unwrap` auf Parserfehlern.
- Abhängigkeit des Kerns auf `ingest`.

## 8. Tests & Messung

```bash
cargo test -p contextra-ingest --locked
cargo clippy -p contextra-ingest -- -D warnings
```

- 26 Testdateien/-module; Fuzz je Format unter `fuzz/`.

## 9. Offene Arbeit

| WP | Inhalt |
|---|---|
| B24 | Erledigt: Autarke Entkopplung von `engine`, Kante `engine` entfernt, Standalone-Beispiel hinzugefügt |
