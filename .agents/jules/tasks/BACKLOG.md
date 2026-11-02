# Arbeitsliste für Jules

Nicht normativ. Maßgeblich sind die Arbeitspakete in [`docs/SPEC.md`](../../../docs/SPEC.md), Abschnitt 15.4, und die Gates in 15.3 (Version 4.3.0).

## Regeln

- Abarbeitung von oben nach unten. Ein Paket beginnt erst, wenn alle Pakete, von denen es abhängt, `done` sind.
- **Status** (Spalte „Status“, exakt eines dieser Wörter): `todo` · `in_progress` · `done` · `blocked`.
- Statusänderungen gehören in denselben PR wie die Arbeit am Paket.
- **Neue Zeilen** entstehen, wenn ein Paket in Teilpakete zerfällt, oder wenn bei der Arbeit ein Befund außerhalb des Pakets auftaucht, der ein eigenes Paket verdient. Ein neues Paket erhält den Status `todo` und eine Nummer im Schema `WP-x.y` mit Suffix (`WP-3.3a`).
- **Karte**: „vorhanden“ heißt, es gibt `tasks/WP-x.y.toml`. „bei Aufnahme“ heißt, Jules erstellt sie aus `_TEMPLATE.toml`, bevor es Code schreibt.
- Die Abhängigkeitsspalte folgt der Gate-Reihenfolge aus SPEC 15.3 (A → B → G0 → C → D → F → E → G → H). Bei Widerspruch zur Spalte gilt die Gate-Reihenfolge.

## Liste

| WP | Titel | Gate | hängt ab von | Karte | Status | Notiz |
|---|---|---|---|---|---|---|
| WP-0.1 | Governance: Ratchet, Ausnahmeliste, PR-Vorlage, `tests/arch`-Grundgerüst, Wortlaut-Lint, Testmakro `ctx_test` (T-TRACE-1) | – | – | bei Aufnahme | todo | Ratchet-Datei `governance/ratchet.toml` fehlt (F-20). Erst das Makro anlegen, dann Übergangsregel `PREAMBLE.md` D ablösen. |
| WP-0.2 | Toolchain-Regel D-82, `rust-toolchain.toml`, MSRV-Matrix, `cargo-semver-checks`, SBOM-Job | A | WP-0.1 | bei Aufnahme | todo | Achtung: `rust-toolchain.toml` ist geschützt. Änderung im PR als „menschliche Freigabe nötig“ kennzeichnen, Rest umsetzen (PREAMBLE B, S3). |
| WP-0.3 | Build wieder grün: Lizenz-Rest, `mmap_readonly`-Aufrufer auf `Vfs`-Lesen (D-94), `Cargo.lock` erneuern (D-93) | A | WP-0.1 | vorhanden | todo | Karte `tasks/WP-0.3.toml`. |
| WP-1.1 | Rückbau Graph und Altlasten: Verweise (F-05), `AutoExtractionMode`, Ports `embedding`/`reranker`/`query_rewriter` | B | WP-0.3 | vorhanden | todo | Karte `tasks/WP-1.1.toml`. |
| WP-1.2 | Branch `archive/v0-full` (D-22); Rückbauliste D-84 (Features, `acl_win32.rs`, `WasmMergeOperator`) | B | WP-1.1 | bei Aufnahme | todo | Der Archiv-Branch wird im Repository angelegt. Hat Jules keinen Schreibzugriff auf Branches, notieren und mit dem Rückbau fortfahren. |
| WP-1.3 | Tokio-Features (D-54), `deny.toml` (Bans, Advisories mit Ablaufdatum, tote `clarify`/`wrappers`) | B | WP-1.2 | bei Aufnahme | todo | `deny.toml` ist geschützt (menschliche Freigabe). Ziel: `cargo tree -e normal` ohne `mio` und `socket2`. |
| WP-1.4 | Ein `Vfs`: `durable-fs` auf `ports::Vfs`; `store` und `vector` ohne `std::fs` (F-13, F-29) | B | WP-1.1 | bei Aufnahme | todo | Ausgangszahlen laut F-29: 169 Zugriffe in `store`, 51 in `vector`. Neu zählen, nicht übernehmen. |
| WP-1.5 | Struktur und Namen: Fassade nach D-89, `contextra_impl` → `instance`, D-90, Crate-Köpfe D-91, Matrixkorrektur D-88, `tests/arch` T-ARCH-5..8, Folgedokumente nach D-98, Sitzungsmarken aus dem Quellbaum, T-DOC-1 | B | WP-1.4 | bei Aufnahme | todo | Marken `ANCHOR[`, `FILE-CONTEXT`, `SESSION:`, `STAND:` entfernen. Die Anzahl ist in F-37 genannt, neu zählen. |
| WP-2.1 | `contextra-engine` und Tests wieder kompilierbar (F-01, F-02), `fmt` (F-03) | A | WP-1.5 | bei Aufnahme | todo | Die Fehlerliste aus F-01 ist vermutlich veraltet. Vor der Arbeit mit `cargo check` neu erheben. |
| WP-3.1 | `types`: `DocRef`, `DocHandle`, Tags mit Längenpräfix, Löschtypen, `KeyCoverage`, `DocId` entfernt; `ports`: Löschports; `erasure`-Skelett als Workspace-Mitglied (D-96) | G0 | WP-2.1 | bei Aufnahme | todo | Erstes Paket des Löschkerns. |
| WP-3.2 | `crypto`: D-06 Grenzen, D-52 Slot-AAD, D-60 Nonce-Schema, D-80 Algorithmenliste, `K_sign` abgeleitet | G-F | WP-3.1 | bei Aufnahme | todo | Kryptografie: Test vor Änderung, keine Änderung ohne Testnachweis. |
| WP-3.3 | `store`: Schlüsselraum 9.1, Envelope v2, WAL ohne Rewrite, `WalFloor`, Sentinel-Fix, Delete-Race-Fix, Pflichtverschlüsselung | G-C | WP-3.2 | bei Aufnahme | todo | Umfang groß. Bei Bedarf in Teilpakete zerlegen (siehe Regeln). |
| WP-3.4 | G0-Prototyp: Schlüsselraum mit D-70/D-71, SSTable-Rewrite, BM-ERASE-WA/-WA-TEXT auf 1/10 GiB | G0 | WP-3.3 | bei Aufnahme | todo | Ergebnis ist ein Messprotokoll und eine Entscheidung über die Fallbackstufe (SPEC 11.9). |
| WP-3.5 | Fortschreibung der SPEC nach G0 (Anhang H), falls Stufe > 0 | G0 | WP-3.4 | bei Aufnahme | todo | Nur ausführen, wenn WP-3.4 eine Stufe > 0 ergibt. Sonst `done` ohne Änderung. |
| WP-4.1 | `erasure`: Tresor (`FileVault`, F.3), `RecordSealer`/`FileKeyring`, Dateischlüssel-Lebenszyklus | G-D | WP-3.4 | bei Aufnahme | todo | |
| WP-4.2 | `erasure`: Gate-Log, Gate-Gruppen-Commit, Saga, Journal, Recovery, Kompaktions-Kooperation | G-D | WP-4.1 | bei Aufnahme | todo | |
| WP-4.3 | Messung: `Scanner<Armed>`, `LayerWitness`, Bericht, Ledger und Restore-Guard | G-D | WP-4.2 | bei Aufnahme | todo | |
| WP-4.4 | `lab` und `ctx-verify` (unabhängige Parser nach Anhang F), Mutationstests | G-D | WP-4.3 | bei Aufnahme | done | ctx-verify umgesetzt und verifiziert |
| WP-5.1 | `engine`: `TenantScopedStorage`, Collections, `DocRef`-API, Suchstrom mit Nachladen, Startsequenz 14.1 | G-D | WP-4.4 | bei Aufnahme | todo | |
| WP-5.2 | Plattformprobe, Prozesshärtung, `health()`/`Metrics`, Plattenreserve | G-F | WP-3.2 | bei Aufnahme | todo | |
| WP-6.1 | Benchmark-Harness `ctx-bench`, Referenzhardware, erste Basislinien aller BM | G-D | WP-4.4 | bei Aufnahme | todo | Ohne Basislinie kein Gate (P9). |
| WP-7.1 | `text`: Segmentindex, Analyzer `de/1` und `und/1`, BMW, Positionen, Entkopplung, Rewrite | G-E | WP-6.1 | bei Aufnahme | todo | |
| WP-8.1 | `vector`: Arena D-86, Surrogat, Kantenreparatur, Rebuild, verschlüsselter Snapshot über `Vfs` | G-G | WP-7.1 | bei Aufnahme | todo | Entfällt im ersten Release bei Cut-Linie (SPEC 1.3). |
| WP-8.2 | `rank` (RRF), Hybridsuche | G-G | WP-8.1 | bei Aufnahme | todo | Entfällt im ersten Release bei Cut-Linie. |
| WP-9.1 | Doku: README (HR-3, 6.4), Rustdoc, `docs/OPERATIONS.md`, `docs/DSFA-BAUSTEIN.md` (Anhang G), Runbooks, `SECURITY.md` | G-H | WP-8.2 | bei Aufnahme | todo | |
| WP-9.2 | Release: signierte Artefakte, SBOM, Evidenzbündel, Fuzz- und Mutationsläufe, G-LEGAL-Stand | G-H | WP-9.1 | bei Aufnahme | todo | Teile, die eine externe Freigabe brauchen (G-LEGAL), werden als `blocked` mit Begründung geführt. |
