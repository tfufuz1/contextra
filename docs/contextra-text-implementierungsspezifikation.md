# contextra-text — Crate-Spezifikation (SOLL-Zustand)

17. Nov 2026 · Revision C1 · bindet an `docs/SPEC.md` v6 (16.11.2026) · ersetzt die crate-lokale `SPEC.md` aus dem v2-Stand (HEAD `f0eda1cb9`). Umsetzungsplan: `PLAN.md` im selben Verzeichnis.

## 0 Geltung, Faktenbasis, Grenzen

Dieses Dokument ist nach `docs/SPEC.md` §1.2 nachrangig. Ein Widerspruch zu `docs/SPEC.md` ist ein Befund, kein Gegenargument. Es überträgt die Crate-Grenzen (§4.1, §5, §6), die Löschkette (Kapitel 11) und die Gates (§7, §14.4) auf `contextra-text` und legt fest, was im Crate zu bauen ist. MUSS, DARF NICHT und SOLL gelten nach RFC 2119; jede MUSS-Aussage nennt ihr Prüfverfahren (§1.3 der Hauptspezifikation).

### 0.1 Faktenbasis

| Punkt | Inhalt |
| --- | --- |
| Geprüfter Stand | Repository `tfufuz1/contextra`, flacher Klon, HEAD `24487fa` („Shell-Commit“). Die Hauptspezifikation beschreibt `fcd387021`; die Zahlen dort sind nicht übernommen, sondern am Klon neu erhoben. |
| **Fehlt im Klon** | Wurzel-`Cargo.toml`, `justfile`, `governance/`, `.github/`, `.githooks/`, `capabilities.toml`, `docs/decisions/`, `docs/decoupling/`. Die Python-Gates in `tools/` sind vorhanden, aber ohne Wurzelmanifest nicht lauffähig (F-T21). |
| Bau-Methode | Rekonstruierter Scratch-Workspace mit den Crates `ids`, `env`, `types`, `ports`, `text`, dem `Cargo.lock` des Repos und einem nachgebauten Wurzelmanifest (`edition = 2021`, leere `workspace.lints`; die Originalwerte sind unbekannt). Es wurde **nicht** das echte Workspace gebaut. |
| Toolchain | stable `1.97.0` statt der vorgeschriebenen `1.89.0` (`static.rust-lang.org` nicht erreichbar). Das ist eine Abweichung von INV-9; Ergebnisse gelten als Näherung. |
| Nicht ausgeführt | `clippy`, Fuzzing, Benchmarks, Loom/TSan/Miri, Mutation, `cargo public-api`, alle Python-Gates, aarch64, Tests mit `docid-128` (kompilieren nicht, F-T05). |

### 0.2 Belegmarken

**\[G\]** im Scratch-Workspace gebaut oder ausgeführt (Befehl in Anhang A) · **\[V\]** an Quelltext, Manifest oder per `grep` geprüft, nicht ausgeführt · **\[S\]** aus `docs/SPEC.md` übernommen · **\[D\]** Festlegung dieses Dokuments · **\[E\]** Einschätzung, nicht belegt.

## 1 Zweck, Abgrenzung, Nicht-Ziele

`contextra-text` ist der invertierte Textindex des Baukastens: Tokenizer und deutsche Morphologie, Posting-Listen mit Block-Max-Metadaten, BM25 und BM25F, Block-Max-WAND, der persistente `InvertedIndex` über das lokale Trait `PostingStore` und die RAM-Referenz `MemoryTextIndex`. Das Crate ist E1a mit der Kern-Closure `ids`, `env` (§2.2, §4.1 \[S\]).

**Rolle in Kapitel 11.** Ein Textindex ist Klasse **D** (Ableitung, §11.2): Er enthält Terme, Dokumentlängen und Dokument-IDs, die aus Quelltexten abgeleitet sind. Das Crate verschlüsselt nichts selbst, besitzt keinen Tresor und entscheidet nicht über Löschungen. Es liefert die Fähigkeiten, die die Löschkette von einer Ableitung braucht: vollständige Bereinigung je Dokument (L2), Lesesperre (L1), Messung mit Positivkontrolle (L5) und das Verweigern des Betriebs auf einem nicht versiegelten Speicher (Z-E2).

**Nicht-Ziele.** Keine Phrasen- oder Fuzzy-Suche. Keine Embeddings. Keine neue Sprache. Keine Formatänderung außer der ID-Breite nach §6.2 \[S\] und dem Formatvermerk aus §5.3 dieses Dokuments (INV-1). Keine Veröffentlichung auf crates.io ohne ADR (R14). Keine Zusammenlegung mit anderen Crates (D-01). Keine eigene Kryptographie.

## 2 Zielzustand gegen Ist-Zustand

| Merkmal | SOLL | IST am Klon | Befund |
| --- | --- | --- | --- |