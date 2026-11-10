# contextra-erasure — Spezifikation

> **Geltung:** Crate-lokale Spezifikation. Normativ bleibt `docs/SPEC.md` (v2, 10.11.2026). Dieses Dokument überträgt die Anforderungen für dieses Crate, ergänzt sie um verifizierte Ist-Fakten und markiert Abweichungen.
> **Faktenbasis:** statische Analyse mit `tools/` am geklonten Repo-Stand (`main`, HEAD `f0eda1cb9`). `cargo` ist in der Analyseumgebung nicht verfügbar. **Es wurde nichts gebaut, getestet oder ausgeführt.**
> **Beleg-Marken:** *[V]* am Quelltext/Manifest verifiziert, *[T]* aus einem tools/-Bericht, *[S]* aus docs/SPEC.md übernommen, *[E]* Festlegung dieser Spezifikation.

**Pfad:** `crates/contextra-erasure/` · **Umsetzungsplan:** `PLAN.md` (im selben Verzeichnis)

## 1. Zweck

Löschkette: Löschbeweise, Ledger, Signaturen, Scanner-Traits. *[S]*

## 2. Zielzustand (docs/SPEC.md §3.1) [S]

| Merkmal | Ziel |
|---|---|
| Gruppe | compliance |
| Ebene (Kern-Closure) | 0 |
| Kern-Closure | — |
| Default-Features | `signing`, `vault`, `async` (Verhalten wie heute) |
| Fehlertyp (R3) | besteht |
| Brücke (hinter Feature `workspace`, §1.1) | Scanner-Abbildung |

Ebenen (SPEC §1.1): E0 = Kern-Closure leer · E1 = Kern-Closure ⊆ E0, höchstens 4 Mitglieder · E2 = Integration · E3 = Anwendungen und Testwerkzeuge. Der Kern ist der Code mit `--no-default-features` ohne Feature `workspace`.

## 3. Ist-Zustand (tools/)

| Messgröße | Wert |
|---|---|
| Lines of Code (Produktion, `loc.py`) | 12392 Zeilen in 66 Dateien |
| Version / publish | workspace-vererbt / <unset> |
| Interne Normal-Abhängigkeiten (Ist, Cargo.toml) | keine |
| Interne Dev-Abhängigkeiten (Ist) | keine |
| Features (Ist, Anzahl) | 0 |
| Default-Features (Ist) | keine |
| `cfg(feature = …)`-Stellen in src/ | 0 |
| `#[test]`-Vorkommen (grep, src + tests) | 139 |
| zero-panic: Produktionsbefunde, bereinigt (Tool-Gesamt; davon in Testdateien) | 0 (Tool 0; 0 in Testdateien) |
| unsafe: Vorkommen / ohne SAFETY / Insel | 0 / 0 / False |
| Determinismus: Produktionsbefunde, bereinigt (Tool-Gesamt) | 3 (Tool 3) |
| Port-M3-Leaks (port_contract_audit) | 0 |
| Error-Enums (Ist) | SignerError, ObserverError, VerificationErrorReason, ErasureError, DeletionProofError, HkdfError |

**Tool-Caveat:** `zero_panic_analyzer.py` klassifiziert `#[cfg(test)]`-Module (`tests.rs`) als Produktion. Im Repo betrifft das 449 von 478 Treffern. Die Zahlen oben sind deshalb bereinigt; die Rohzahl steht in Klammern. Die Korrektur am Tool ist eigener Schritt (S-00).

**Quellen:** `tools/loc.py`, `tools/zero_panic_analyzer.py`, `tools/unsafe_boundary_auditor.py`, `tools/determinism_guard.py`, `tools/port_contract_auditor.py`, `tools/error_topology_mapper.py`, `tools/cargo_dependency_graph.py`, Cargo.toml-Auswertung.

## 4. Ist-vs-Ziel-Abgleich [V]

- **Ist-Kanten, die das Ziel nicht vorsieht:** keine
- **Ziel-Kanten, die im Ist fehlen:** keine
- **Default-Features weichen ab:** Ist `leer` → Ziel `signing, vault, async`.

## 5. Verifizierte Befunde und Abweichungen zu docs/SPEC.md [V]

- Verifiziert: `Cargo.toml` enthält **keine** Features. Die in §6.1 vorgesehenen `signing`, `vault`, `async` existieren noch nicht. Es gibt keine interne Abhängigkeit (Ziel-Ebene E0 ist erfüllt).

## 6. Anforderungen

### 6.1 Crate-Spezifikation (docs/SPEC.md §6) [S]

Features `signing` (`ed25519-dalek`), `vault` (`aes-gcm-siv`), `async` (`tokio`, `futures-util`); Zuordnung Datei→Abhängigkeit per `rg 'use (ed25519_dalek|aes_gcm_siv|tokio|futures_util|bincode)'` in S-24 erheben und im Manifest festschreiben. Typen aus `types/erasure.rs` (616), `ports/{erasure,erasure_legacy,gate,ledger,observer,report_signer,keys,barrier}.rs` übernehmen **nach** Abgleich mit `types_local.rs`: identischer Name+Gestalt ⇒ eine Definition; abweichend ⇒ beide bleiben, Brücke wandelt. Scanner-Traits `Purger`/`LayerScanner` bleiben lokal **und** in `ports::scanner` (R7). `default = ["signing", "vault", "async"]` (erhält das heutige Verhalten); Nutzer ohne `tokio` wählen `--no-default-features`.

### 6.2 Brückenmuster [S]

Die Brücke liegt ausschließlich in `src/compat/` hinter dem Feature `workspace`. Verbindliche Muster: docs/SPEC.md §4.3 (a–d). R2 gilt: `contextra_types`, `contextra_ports`, `contextra_testkit` dürfen im Kern nirgends vorkommen, auch nicht in Doc-Kommentaren oder `#[cfg(test)]`.

### 6.3 Regeln für jeden Schritt [S]

INV-1 Formatgleichheit · INV-2 Pfadgleichheit (Re-Export mit `#[deprecated]`) · INV-3 Ein Schritt = Commit-Paar · INV-4 `cargo check --workspace --all-targets --locked` grün · INV-5 Determinismus (Tie-Break nach ID aufsteigend) · INV-6 Durability-Pfade unberührt · INV-7 keine Compliance-Aussagen · INV-8 keine Rückwärtspfeile. Vollständig: docs/SPEC.md §1.2, Regeln R1–R9: §3.2.

### 6.5 Fehlerabbildung [S]

Eigener Fehlertyp: besteht (thiserror, `#[non_exhaustive]`). Jede heute genutzte `ContextraError`-Variante hat genau eine lokale Entsprechung; `Display` bleibt bitgleich; keine Sammelvariante `Other(String)` ohne Quelle. Tabelle je Baustein: `docs/decoupling/errors/<crate>.md` (aus S-xx erzeugt). Quelle: §5.3.

## 7. Schnittstellen und Persistenz

Keine Zeile aus dem Golden-Korpus (§5.1) bezieht sich auf dieses Crate.

### 7.2 Öffentliche Schnittstelle

Ist-Stand und Ziel-Schnittstelle sind in den Abschnitten 3 bis 6 festgehalten. Die exakte Liste der `pub`-Symbole erhebt S-00 über `cargo public-api` (Gate G-A1); erlaubte Differenzen sind nur Re-Export-Pfade mit `#[deprecated]`.

## 8. Fundstellen (tools/, exakt) [T]

**Determinismus (Produktion, bereinigt: 3; die ersten 15):** Tool-Gesamt 3.

- `crates/contextra-erasure/src/types_local.rs:2078` — instant_now — `start_instant: std::time::Instant::now(),`
- `crates/contextra-erasure/src/types_local.rs:2092` — system_time_now — `std::time::SystemTime::now()`
- `crates/contextra-erasure/src/types_local.rs:2100` — instant_now — `let nanos = std::time::Instant::now()`

**Fehler-Enums (Ist):**

- `crates/contextra-erasure/src/types_local.rs:1073` — SignerError
- `crates/contextra-erasure/src/types_local.rs:1153` — ObserverError
- `crates/contextra-erasure/src/deletion_proof.rs:2629` — VerificationErrorReason
- `crates/contextra-erasure/src/error.rs:7` — ErasureError
- `crates/contextra-erasure/src/ed25519_proof.rs:39` — DeletionProofError
- `crates/contextra-erasure/src/crypto_local.rs:61` — HkdfError

## 9. Befunde aus docs/SPEC.md (B-Nummern) [S]

**B02 — Die `Clock`-/`Rng`-Kopien sind nicht kompatibel (Teil 2, G1 ist unvollständig)**
> **Entscheidung:** - `env::Clock` = `ports::Clock` (Nanosekunden, `monotonic_nanos`, `Send+Sync+'static`). `env::Rng` = `ports::Rng` mit `&self` (siehe §4.2). - Millisekunden-Verbraucher (`text`, `graph`, `vector/contract.rs`) benutzen `env::Clock` und `env::unix_ms(&dyn Clock) -> u64` = `now_unix_nanos() / 1_000_000` (Abrunden). Persistierte ms-Zeitstempel MÜSSEN im Differenztest (§5.4) bitgleich zu `now_unix_ms` der alten Implementierung bei identischer Quelle sein. - `&mut self`-Verbraucher `text`/`graph`/`vector` wechseln auf `&self`-`env::Rng`. Das ist für alle heutigen Aufrufstellen ein Superset (Mutex/Atomic innerhalb `SeededRng`). Der Umstieg ist ein **Umbau-Commit** (INV-3 B). - E0-Crates (`adapt`, `privacy`, `license`, `erasure`, `ingest`, `audit-export`) behalten ihre lokalen Traits (R7/K8). Ihre Brücke bindet per Blanket-Impl **von `env` auf lokal** (Richtung `&self → &mut …
> `erasure/types_local.rs:2059`, `privacy/types.rs:11` — nanos + monotonic — —

**B03 — `docid-128` ändert die Typbreite und ist nicht additiv**
> **Entscheidung:** 1. `ids` besitzt `docid-128` allein. Jeder Crate mit Abhängigkeit auf `ids` leitet `docid-128 = ["contextra-ids/docid-128"]` weiter; leere/lokale Features entfallen. E0-Crates ohne `ids` (`ingest`, `erasure`) bekommen **kein** `docid-128`; ihre Konvertierung von `ids::DocId` ist `TryFrom` (fehlschlagend bei Überlauf), nie `as`. 2. `ids` führt `pub const ID_WIDTH_BITS: u32` (64 oder 128) und die Compile-Time-Prüfung `size_of`/`align_of` (8/8 bzw. 16/16). 3. Jedes persistierte Format, das IDs trägt, MUSS die Breite beim Öffnen prüfen (§5.2). Fehlt das Feld heute, wird die Formatversion erhöht und v(alt) als „Breite unbekannt“ nur mit expliziter Konfiguration geöffnet. 4. CI-Matrix: `cargo hack check --workspace --features docid-128` zusätzlich zum Default.

**B10 — Das Verschieben der Krypto-Dateien bricht Garantie-Nachweise**
> **Entscheidung:** - `legacy-erasure` verlässt die Defaults von `crypto` (S-12). Die Dateien `audit_chain`, `deletion_proof/`, `ed25519_proof` werden **nicht** archiviert, solange nicht (a) ein Item-Diff gegen `erasure` Gleichwertigkeit zeigt **und** (b) der Mutation-Score der Zieldateien in `erasure` ≥ Score der Quelldateien ist (Gate G-C1). Bis dahin bleiben sie, hinter dem Feature, im Crate (Tier frozen). - Wandern `kv_cipher`, `kv_segment/*`, `kv_shredding`, `revocation_log` nach `kvcache`, MUSS derselbe Commit `mutants-guarantees.toml`, `loom-gate.yml` und `CODEOWNERS` auf die neuen Pfade ziehen und beide Versionskonstanten bis zur Vereinigung per `const_assert_eq` koppeln. Nach dem Umzug ist `kvcache` alleiniger Besitzer; `crypto` definiert keine KV-Versionskonstante mehr. - Schnitt und Umzug (S-38) bilden ein untrennbares Commit-Paar.

**B14 — Defaults und leere Features (Teil 2, G5/G6) — bestätigt, plus Ergänzungen**
> **Entscheidung:** Zieldefaults §3.1. Da `default-members` diese Crates auslässt, genügt „wöchentlich `cargo check`“ nicht als Kompilierbarkeitsgarantie; CI führt bei **jedem** PR `cargo check --workspace --all-targets --locked` aus (S-01). Ein Feature ohne `cfg(feature = "x")` im Quelltext **und** ohne Weiterleitung in einem anderen Manifest wird entfernt; geprüft vom neuen `tools/feature_audit.py` (S-12).

## 10. Entscheidungen und Risiken [S]

Keine Einträge in §9.1 oder §9.2.

Nicht-Ziele (§9.3), gelten für alle Crates: physische Regruppierung, Mikro-Crate-Zerlegung (F-1), Verschieben von `Vfs`/`io_seam`, crates.io-Veröffentlichung, Formatänderungen außer ID-Breite, Änderung der FlatBuffers-Schemas.

## 11. Abnahme [S]

Ein Crate gilt als entnehmbar (R6), wenn in dieser Reihenfolge grün: **G-L2** (Null-Prüfungen, nur bei Brücke) → **G-L1** (Kern- und Feature-Closure) → **G-L3** (`--no-default-features`, `--each-feature`) → **G-L4** (isolierter Export). Danach **G-F1/G-D1/G-E1/G-C1** für den betroffenen Schritt (§8.4).

```bash
cargo check -p contextra-erasure --no-default-features --locked
cargo hack check -p contextra-erasure --each-feature --no-dev-deps --locked
python3 tools/isolation_proof.py --crate contextra-erasure --modes no-default,default,each-feature
```

