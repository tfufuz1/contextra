# Beitragen zu Contextra

<!-- doc-stamp: 2026-11-08 | Normativ: docs/SPEC.md Fassung 4.1, AGENTS.md, capabilities.toml -->

Vielen Dank für Ihr Interesse. Dieses Dokument beschreibt, wie Beiträge und Änderungen in das modulare **Contextra Baukastensystem** eingebracht werden.

---

## 1. Voraussetzungen

- Rust-Toolchain laut `rust-toolchain.toml` (MSRV 1.89).
- Kenntnis der Baukasten-Kriterien (B1–B12 in `docs/SPEC.md` §2).
- Kenntnis der Crate-Klassen (Klasse A, B, C, T in `docs/SPEC.md` §5).
- Einhaltung der Verifikations-Gates G1–G10 (`docs/SPEC.md` §8).

## 2. Ablauf

1. Aufgabe einem Arbeitspaket aus `PLAN.md` zuordnen oder ein begründetes Issue / RFC-Entwurf abstimmen.
2. Branch nach Schema `wp/<nummer>-<kurzbeschreibung>` anlegen, z. B. `wp/decouple-rank`.
3. Änderungen scharf entlang der Crate-Grenzen schneiden. Für Klasse-A-Bausteine gilt strikt: keine internen `contextra-*`-Abhängigkeiten (B1, B2).
4. Lokal prüfen:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --locked
```

5. Bei Änderungen an Klasse-A-Bausteinen den autonomen Standalone-Build prüfen:

```bash
tools/export-crate <crate-name> /tmp/test-export && cd /tmp/test-export && cargo test --locked
```

6. Vor dem PR auf den aktuellen Hauptzweig rebasen: `git fetch origin && git rebase origin/main`.

## 3. Verbindliche Regeln

- **Zero-Panic (B9, E-07):** Kein `unwrap()`, `expect()`, `panic!()`, `todo!()` oder ungeprüftes Indexieren in Produktionscode (`clippy::unwrap_used`, `clippy::indexing_slicing`).
- **Unsafe-Disziplin (B10):** `#![forbid(unsafe_code)]` im gesamten Workspace außer in den deklarierten Unsafe-Inseln (`contextra-sys`, `contextra-simd`, `contextra-wire`). Jeder Unsafe-Block muss einen stichhaltigen `// SAFETY:`-Kommentar tragen.
- **Umwelt-Determinismus (B11):** Zeit, Zufall und IDs ausschließlich über injizierte Schnittstellen (`Clock`, `Rng`, `IdGen`). Keine direkten Aufrufe von `SystemTime::now()` im Kern.
- **Fail-Closed Durability:** Fsync-Fehler müssen propagiert werden und führen zum Poisoning des betroffenen Datei-Handles (`INV-FSYNC-1`).
- **Keine Rechts- oder Löschzusagen im Kern (B7, §6):** Indizes und Speichermodule bleiben frei von DSGVO-, Saga-, Witness- und Löschzertifikat-Logik. Diese verbleibt exklusiv in `contextra-erasure`.

## 4. Commit-Nachrichten

Format:
```
<typ>(<scope>): Kurze Beschreibung im Imperativ (max. 50 Zeichen)

- Begründung der Änderung mit Verweis auf docs/SPEC.md §... und PLAN.md
- Nachweis der Verifikations-Gates (z. B. G1, G3, G9, G10)
```

## 5. Fragen zum Scope und Entscheidungen

Architekturentscheidungen und Änderungen an Schnittstellen oder Crate-Klassen werden im Normativen Entscheidungsregister (`docs/SPEC.md` §15) und in Architecture Decision Records (`docs/decisions/`) festgehalten.

## 6. Lizenz

Beiträge werden unter der Doppellizenz MIT oder Apache-2.0 des Projekts aufgenommen. Jeder Klasse-A-Baustein führt eigenständige Lizenzdateien (`LICENSE-MIT`, `LICENSE-APACHE`) zur Wahrung der Lizenzautarkie (B12).
