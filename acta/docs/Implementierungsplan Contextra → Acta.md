# Implementierungsplan Contextra → Acta

*Principal-Rust-Engineering · Stand 2026-11-15 · Normativ: `acta/docs/SPEC.md` Fassung 3.0 · Quelle: `tfufuz1/contextra`, HEAD `50cdc3d` (gepinnt)*

*Methode: ausschließlich statische Analyse (Manifest-Parsing, Quelltext-Lektüre, Grep). Es wurde nichts kompiliert oder ausgeführt; in der Analyseumgebung war keine Rust-Toolchain vorhanden. Zahlen sind Messwerte am Klon, Aufwandsklassen sind relative Schätzungen.*

Beweismarken: **\[V\]** am Quelltext oder Manifest des Klons verifiziert · **\[S\]** aus der SPEC übernommen · **\[A\]** abgeleitet · **\[E\]** Empfehlung bzw. Entscheidungsvorschlag.

## 1. Kurzfassung

1. **Acta ist kein Rename von Contextra.** Die bisherige Extraktion (`docs/extract_acta.py`: ids, env, crypto, types, ports; ca. 30 k LOC inkl. Tests) ersetzt Texte, prüft aber nichts. Sie hat den SPEC-Befund V-05 mitgenommen und zusätzliche Abweichungen von der SPEC erzeugt (Abschnitt 4: zwölf neue Befunde A-01 bis A-12). Der Plan ersetzt „kopieren und umbenennen“ durch **Port-by-Redesign**: Contextra dient als Wissensquelle und Testorakel, der Code entsteht aus der SPEC, und Gates verhindern den Wiedereintritt bekannter Mängel (Abschnitt 5).
2. **Hubs.** In Contextra sind `types`, `ports`, `engine` und `db` die Hubs (`docs/HUB_REMOVAL.md`). Die Entkopplung läuft: `engine` ist bereits aufgelöst (15 LOC Fassade), die Module liegen aber jetzt in `db` (23 313 LOC); ADR-0121 verlagert alle Fehlerkonvertierungen nach `types`, wodurch `types` jeden Baustein kennt. Acta übernimmt den **Zielzustand** der Entkopplung, nicht ihre Zwischenstände (Abschnitt 6).
3. **Acta-Hub-Modell.** Zwei schmale Hubs: `acta-types` (R2a: Werte, Taxonomien) und `acta-ports` (R2b: nur ringübergreifende Verträge), beide mit Größenbudget und „Rule of Two“. Alle übrigen Traits gehören dem Verbraucher. Es gibt keinen `engine`- oder `db`-Hub; `acta-engine` ist die Use-Case-Schicht, `acta-node` die einzige Stelle, die alle konkreten Crates verdrahtet.
4. **Die SPEC braucht sieben Entscheidungen vor R1** (Abschnitt 12), darunter Ring-Unterteilung R2, AEAD und Nonce, Commitment-Funktion (SPEC §4.1 und §6.6 widersprechen sich) und Laufzeitmodell (sync oder async).
5. **Reihenfolge nach SPEC §12:** R0 Stop-the-Line, R1 Krypto-Kern, R2 Walking Skeleton, R3 Datenmodell, R4/R5 Store und Indizes, R6 Chronik und Node, R7 Audit. Mission-Module (Privacy, Ingest, Routing, lokale Inferenz) laufen ab R2 in parallelen Spuren (Abschnitte 7 bis 9).
6. **Größte Risiken:** `contextra-store` (32 015 LOC, 160 Testdateien) und `contextra-erasure` (22 910 LOC, davon 2 439 LOC Duplikattypen). Beide werden nicht portiert, sondern modulweise neu aufgebaut.

## 2. Analyse der SPEC 3.0

### 2.1 Was die Mission technisch bedeutet \[S\]

Personenbezogene Daten (Ebene **P**) liegen verschlüsselt im Tresor und werden durch Schlüsselvernichtung unlesbar. Sach- und Entscheidungszusammenhänge (Ebene **N**, Chronik) sind nachweislich frei von Personenbezug und bitemporal abfragbar. Indizes und Caches (Ebene **D**) tragen zwingend Herkunft (Lineage) und werden bei Löschung maskiert und neu aufgebaut. Das Kernparadoxon „Zeitreise gegen Löschung“ löst die SPEC durch strikte Trennung von N und P und die Regel „Löschung gewinnt immer“: `as_of(t)` auf Gelöschtes liefert `Erased`, nie Klartext.

### 2.2 Normative Zusagen und ihr Besitzer in der Codebasis \[S, E\]

| Zusage | Kern der Anforderung | Besitzer (Acta-Crate) | Typ- bzw. Gate-Mechanik |
| --- | --- | --- | --- |
| S1 | Sofortige Lesesperre, auch bei `as_of` | acta-erasure (Gate), acta-mvcc und acta-store (Leser) | `ErasedView`, `ReadGate`; G-READ-1 |
| S2 | AEAD vor dem WAL | acta-crypto, acta-store | `Sealed<P>`; T-WAL-ENC |
| S3 | Mehrbetroffene: Modus DeleteAll | acta-erasure | Scope-Auflösung in E0 |
| S4 | Crash-Invarianz C1 bis C12 | acta-erasure, acta-durable-fs | `FaultVfs`, Journal |
| S5 | Chronik frei von Personenbezug | acta-chronicle | N-Validator, `K_MIN` ≥ 5 |
| S6 | Zeitreise deterministisch | acta-chronicle, acta-rank | Hash-Kette, T-RANK-1 |
| S7 | Fail-Closed-Krypto | acta-crypto, acta-env | kein `Default` für Schlüssel; G-CRY-1 |
| S8 | Löschnachweis nur mit Positivkontrolle | acta-erasure | `Scanner<Unarmed/Armed>` |
| S9 | Restore-Schutz (72 h) | acta-erasure | `RestoreGuard` |
| S10 | Lineage-Typzwang | acta-types und Index-Crates | `Derived<T>`, Compile-Fail-Test |
| S11 | Kein gemeinsamer Schlüssel im Container | acta-crypto, acta-store | ein Umschlag je Record |
| S12 | Keine Roh-IDs in Protokollen | acta-erasure (Ledger), acta-chronicle | keyed Commitments |

### 2.3 Strukturvorgaben, die den Plan bestimmen \[S\]

- **Ringmodell R0 bis R7** (§3.1): Abhängigkeiten zeigen nur nach unten, Querkanten im Ring sind verboten; Gate G1 prüft per `cargo metadata`.
- **Port-Tabelle** (§3.3): `Clock/Rng/IdGen` in `acta-env`, `Vfs` in `acta-durable-fs`, `KeyAnchor` und `ReportSigner` in `acta-crypto`, `ErasureLayer` im Löschkern mit Implementierungen in `acta-erasure-adapters`, `SnapshotRevoker` in `acta-mvcc`. Jeder Port hat genau eine Produktionsimplementierung.
- **Hygiene** (§3.2): Features rein additiv, `unsafe` nur in sys, simd und wire, Zero-Panic-Lints, verbotene Std-Aufrufe (`fs::remove_file`, `hard_link`, `copy`, `SystemTime::now`, `thread_rng`).
- **Berichtssprache** (§1.3): keine Aussagen wie „DSGVO-konform“ oder „100 % gelöscht“; nur die Klassen G, K, P, D, C mit den Annahmen A-FLASH, A-RAM, A-ANCHOR, A-CRYPTO, A-EXT.

Die SPEC deckt die Mission-Säulen Datenschutz, Nachvollziehbarkeit und Datenverwaltung konstruktiv ab. Souveränität (lokale Inferenz), Effizienz, Präzision und Kontrolle sowie Entscheidungshilfe liegen **außerhalb** des SPEC-Ringmodells und stecken in Contextra-Crates (adapt, router, kvcache, infer-\*, rank, agent, cognition, privacy, prompt-guard). Ihre Einordnung leistet dieser Plan (Abschnitte 7 und 8).

## 3. Ausgangslage Contextra (gepinnt auf `50cdc3d`)

### 3.1 Größenordnung \[V\]

39 Crates, rund 280 k LOC in `src/`, 921 Integrationstest-Dateien. Die relevanten Brocken:

| Crate | LOC | Test-Dateien | unsafe | Harte interne Abhängigkeiten |
| --- | --- | --- | --- | --- |
| store | 32 015 | 160 | 0 | ids, env, sys, mvcc, crypto |
| graph | 23 593 | 61 | 1 | ids, env |
| db | 23 313 | 51 | 1 | zehn Crates (vault, types, ports, mvcc, store, vector, text, checkpoint, rank, graph) |
| erasure | 22 910 | 43 | 0 | keine (alles selbst dupliziert) |
| ingest | 21 140 | 33 | 2 | keine |
| vector | 17 744 | 92 | 10 | ids, env, mvcc |
| text | 9 535 | 31 | 0 | ids, env |
| types / ports | 7 510 / 6 755 | 13 / 3 | 0 | ids / env, types |
| crypto | 6 290 | 23 | 1 | ids, env |
| checkpoint | 5 593 | 20 | 0 | ids, env |
| mvcc / rank | 4 858 / 5 852 | 23 / 24 | 0 | ids, env / keine |

### 3.2 Stand der Entkopplung \[V\]

- Zielmodell „Baukasten“: E0 (Leaf-Bausteine), E1 (Algorithmus- und State-Kerne), E2 (Integration: types, ports, router, agent, cognition, db), E3 (Anwendungen). Gate-Werkzeuge liegen in `tools/` (`gate_g1_layering.py`, `isolation_proof.py`).
- Baseline vom 2026-11-08: 61 Kantenverstöße; 18 Bausteine hängen noch an `types` oder `ports`; 48 von 68 Warnungen sind `deprecated`-Re-Exports.
- Hub-Abbau in fünf Schritten (`HUB_REMOVAL.md` §3): Brücken umkehren, `ports` auflösen, `types` aufteilen, `engine` auflösen, `db` auflösen. **Erledigt:** Schritt 4 (ADR-0122, `contextra-engine` ist `#[deprecated]`-Fassade). **Offen:** 1, 2, 3 und 5.
- Zwischenlösungen, die Acta nicht übernehmen darf: `compat/`-Brücken hinter dem Feature `workspace`, deprecated Re-Exports, Shims wie `engine_local.rs` (`pub use contextra_engine as engine`), `.inc`-Includes in `db` (`vault_sys.inc`, `multistep_pid.inc`, `vault_compat.inc`) sowie ADR-0121 (siehe 6.1).

### 3.3 SPEC-Befunde, die im Contextra-Code reproduzierbar sind \[V\]

| SPEC-ID | Nachweis im Klon |
| --- | --- |
| V-01 | `erasure/src/vault/sealer.rs:101` Konstante `[0x55u8; 32]` als `k_unit`; `:279` `keyed_hash(&self.k_unit, …)` als Fallback |
| V-02 | `sealer.rs:313-341` XOR mit BLAKE3-Strom (`b ^ mask_byte`), keine Nonce, kein Tag |
| V-04 | `vault/slot.rs:208` `mask_epoch_key` ohne Aufrufer außer Re-Export |
| V-05 | `saga/core.rs:72` `k_idx: [0u8; 32]` im `Default`; `vault/keyring.rs:78` `[0x42u8; 32]` |
| V-06 | `store/src/tenant_codec.rs:18-62` ASCII-Schlüssel `t:{tenant}:{coll}:chunk:{doc}`; `util.rs:47-135` Dummy-`RecordSealer` |
| V-08 | `store/src/lsm/fence.rs:59` `enter_purge` ohne Aufrufer im Workspace |
| V-09 | Grep nach `doc_ord`, `source_set`, `RoaringBitmap` in `crates/`: 0 Treffer |
| V-10 | `checkpoint/src/hardlink_cloner.rs:243` `fs::hard_link` ohne Registry |
| V-15 | `erasure/src/revocation_log.rs:52-56` Kennungen als `String` |

Nicht reproduziert (kein Build möglich): V-13, V-14 (Testkompilierung, Golden-Drift) und V-17 (Compaction-Hänger). Sie bleiben Pflichtfälle der Abnahme.

**Weitere Muster \[V\]:** 57 direkte `std::fs`-Aufrufe (`remove_file`, `copy`, `hard_link`) außerhalb von Tests (grob gefiltert); `unwrap_or_default()` 131-mal und `let _ =` 648-mal in `src/` (nicht jede Stelle ist ein Fehler, im Krypto-, Store- und Erasure-Pfad aber jede prüfpflichtig); `erasure` hat `default = [signing, vault, async]` (tokio im Default, V-03); `store` und `checkpoint` sind async-first.

## 4. Ist-Zustand `acta/`: neue Befunde A-01 bis A-12

Der Ordner enthält ids, env, crypto, types und ports (ca. 30 k LOC inkl. Tests) mit strengem Lint-Set. Er entstand mit `extract_acta.py` (Ersetzungsregeln `Contextra` → `Acta`).

| ID | Befund | Beleg \[V\] | Maßnahme |
| --- | --- | --- | --- |
| A-01 | AEAD weicht von der SPEC ab: AES-256-GCM-SIV, 12-Byte-Nonce, Umschlag v2 statt XChaCha20-Poly1305, 24-Byte-Nonce, Umschlag v3 mit AAD-Struktur | `acta-crypto/src/aead.rs:10-31,80-90` | Neuaufbau in R1 |
| A-02 | `seal_record` verschluckt Fehler per `unwrap_or_default()` und liefert einen leeren Umschlag (V-05 liegt in Acta vor) | `aead.rs:71` | sofort in R0 |
| A-03 | `derive_dek` ersetzt einen fehlenden Subject-Salt durch Nullen (verletzt H3) und verwirft HKDF-Fehler per `let _ =` | `dek.rs:79-87` | sofort in R0 |
| A-04 | Nonce ist öffentlicher Parameter (`seal_record_with_nonce`) statt aus dem `Rng`-Port (verletzt I-15); kein `CryptoRng`-Port, direkter `OsRng`-Zugriff | `aead.rs:56-66`; Grep `CryptoRng`: 0 Treffer | R1 |
| A-05 | `Rng::fill_bytes(&self, &mut [u8])` ist unfehlbar; ein Entropiefehler kann nicht propagiert werden (S7, T-RNG-1) | `acta-env/src/rng.rs:50` | R0/R1 |
| A-06 | R0-Crate `acta-crypto` hängt optional an R2 (`acta-types`, `acta-ports`); Default-Feature `workspace` schaltet das ein (Ringverstoß, V-16-Muster) | `acta-crypto/Cargo.toml` | Kante entfernen |
| A-07 | `acta-ports` → `acta-types` ist eine Querkante im Ring R2; Ports re-exportiert Types wholesale; `#![allow(deprecated)]` | `acta-ports/src/lib.rs:5-28` | Hub-Neuschnitt (6.3) |
| A-08 | `erasure_legacy.rs` verweist auf ein nicht existierendes `acta-erasure-spi` (Entscheidung D-E1); die SPEC kennt `acta-erasure` und `acta-erasure-adapters` | `acta-ports/src/erasure_legacy.rs:1-8` | Namenskonflikt auflösen |
| A-09 | `acta-types/src/error.rs`: 1 177 Zeilen, 46 Fehlervarianten fremder Domänen (`WalCorruption`, `HnswConnectivityDegraded`, `Sandbox`); Algorithmus-Konfiguration im Hub (`PprConfig`, `TlHfdParams`, `DistanceMetric`, `MAX_SEARCH_K`) | `error.rs`; `types/domain/misc.rs:98,389,484` | verteilen (6.4) |
| A-10 | `acta-ids`: `TenantId::SYSTEM`, `DEFAULT` und `INVALID` sind alle `0`; öffentliche Tuple-Felder (`TenantId(0)` frei konstruierbar); 64-Bit-IDs statt 16-Byte-Tags und `doc_ord: u32` der SPEC | `acta-ids/src/lib.rs:79-119` | Neuentwurf R0/R3 |
| A-11 | Rename-Artefakte: Re-Export-Listen falsch sortiert (`ActaError`); `clippy.toml` verweist auf `acta_ports::clock::Clock` (SPEC: `acta-env`) und eine nicht vorhandene `governance/clippy.toml`; `.cargo/config.toml` ohne `xtask`-Alias (SPEC §8.5 verlangt `cargo xtask gates`) | `acta/clippy.toml`, `.cargo/config.toml` | R0 |
| A-12 | Ausnahme „`thread_rng` für Schlüssel, Salt und Nonce“ (ADR-098) widerspricht P6 und S7 (alle Zufallsquellen über Port) | `acta/clippy.toml` | Ausnahme streichen |

**Konsequenz:** Die fünf vorhandenen Crates gelten als Entwurf, nicht als Basis. Sie werden in R0/R1 gegen die SPEC neu abgenommen (Status HARDEN oder REWRITE, Abschnitt 7).

## 5. Leitprinzip: Port-by-Redesign mit Defect Firewall

### 5.1 Regeln

1. **Die SPEC schlägt den Code.** Contextra-Quellen sind Wissensquelle und Testorakel, nie Vorlage (SPEC §0.3).
2. **Keine mechanischen Umbenennungen.** `extract_acta.py` bleibt als Inventurwerkzeug, erzeugt aber keinen Produktivcode mehr.
3. **Klassifikation je Modul** (Abschnitt 7): **ADOPT** (autarker, fachlich reiner Baustein; Kennungen, Fehlertyp und Defaults prüfen), **HARDEN** (Struktur trägt; Defekte, Tests und Gates ergänzen), **REWRITE** (aus der SPEC neu), **NEW** (kein Contextra-Pendant), **DEFER** (nach R7 oder hinter Feature), **DROP** (entfällt).
4. **Krypto-, Tresor-, Umschlag- und Schlüsselraum-Code ist Clean-Room:** Der Autor schreibt aus SPEC-Abschnitt und Testvektoren; das Review prüft gegen die Defektliste, nicht gegen den Altcode.
5. **Greenfield nutzen:** keine `#[deprecated]`-Aliase, keine Kompatibilitätsbrücken, keine `.inc`-Includes. Contextra-Namen werden bei Bedarf in einer Migrationstabelle dokumentiert.
6. **Jede übernommene Verhaltensannahme braucht einen Test** (Differential- oder Golden-Test). Das Muster existiert bereits in `acta-ids` (`tests/differential_ids.rs`, `tests/legacy_reference/`).
7. **Toolchain pinnen** (1.89.0 laut `rust-toolchain.toml`) und die Baseline vor dem ersten Eingriff messen (WP-0.1).

### 5.2 Pipeline je Crate

1. **Spec-Slice:** relevante SPEC-Paragraphen und Zusagen auf eine Seite verdichten; offene Fragen als ADR-Entwurf.
2. **Befundkatalog:** bekannte V- und A-Befunde plus Mustersuche (5.3) über die Quell-Crate.
3. **Charakterisierungstests am Orakel:** nur Verhalten, das die SPEC zulässt (z. B. BM25-Ranking, SSI-Konflikte); keine Tests, die Defekte festschreiben.
4. **Redesign:** Typen zuerst (Typestate, Newtypes), dann Algorithmen; ein geschlossener, klassifizierter Fehlertyp je Crate (6.5).
5. **Differential- und Golden-Tests** gegen Contextra für den erhaltenswerten Teil; Property-Tests (`proptest`) und Loom für Nebenläufigkeit.
6. **Gates** (Abschnitt 10) lokal und in der CI grün.
7. **Review:** vier Augen bei Krypto, Erasure und Store-Format, sonst zwei.
8. **Upstream-Ledger:** `UPSTREAM.md` je Crate (Quellpfade, Commit, Entscheidung, Grund).

### 5.3 Defect Firewall (Gate G-NOREN)

`extract_acta.py` enthält bereits `AUDIT_PATTERNS` (V-01, V-02, V-05, V-07). Sie werden zu einem harten Gate ausgebaut, das den **Neueintritt** von Contextra-Mustern in Produktionscode verhindert:

- `unwrap_or_default()` und `let _ =` auf `Result` in acta-crypto, acta-erasure, acta-store, acta-checkpoint
- Schlüssel-Literale (`[0x..u8; 32]`) außerhalb von `#[cfg(test)]`; `impl Default` für Typen mit Schlüsselmaterial oder `k_idx`
- `Default` für Identitätstypen (`TenantId` usw.)
- direkte `std::fs::{remove_file, copy, hard_link, rename}` außerhalb von `acta-durable-fs`
- `pub use <crate> as <alias>`, `include!("*.inc")`, `#[deprecated]`, Verzeichnisse `compat/`
- verbotene Aussagen aus SPEC §1.3 (G-WORD). Treffer in Contextra \[V\]: `erasure/src/journal/scope_purge.rs:25` („unwiederbringlich“) und `types_local.rs:571` („Revisionssicherer“)

Umsetzung als Rust-Werkzeug `cargo xtask gates` (SPEC §8.5), damit die CI hermetisch läuft. Die Python-Werkzeuge in `tools/` (`gate_g1_layering.py`, `isolation_proof.py`, `count_bridge_refs.py`) dienen als Prototypen und Testorakel für den Rust-Port.

### 5.4 Umgang mit dem beweglichen Ziel

Contextra wird weiter entkoppelt. Acta pinnt `50cdc3d`, führt je Crate den Upstream-Ledger und prüft pro Iteration per `git diff`, ob relevante Änderungen vorliegen (Sicherheitskorrekturen, neue Tests). Es gibt kein automatisches Mergen.

## 6. Hub-Architektur in Acta

### 6.1 Lehren aus den Contextra-Hubs \[V\]

| # | Befund in Contextra | Beleg | Konsequenz für Acta |
| --- | --- | --- | --- |
| L1 | Fehler-Monolith `ContextraError`: 1 177 Zeilen, domänenspezifische Varianten, 505 Aufrufstellen. ADR-0121 hält E0/E1-Crates fehlerfrei, verlagert aber alle `From<LocalError>` nach `types`; damit kennt `types` jeden Baustein. | `types/src/error.rs`, ADR-0121 | Kein Sammelfehler im Hub. Fehler je Crate, Aggregation erst im Verbraucher (6.5). |
| L2 | Algorithmus-Konfiguration im Hub (`PprConfig`, `TlHfdParams`, `DistanceMetric`, `MAX_SEARCH_K`; `TOMBSTONE_BIT` dreifach definiert) | Baukasten-Entscheidung F5, ADR-0120 | Konfiguration gehört dem Algorithmus-Crate, Formatkonstanten dem Identitäts-Crate `acta-ids`. |
| L3 | `ports` bündelt 43 Traits in 42 Dateien, 8 harte und 14 optionale Verbraucher, re-exportiert `types` | `HUB_REMOVAL.md` §2.2 | Ports nur für ringübergreifende Verträge, sonst Verbraucher-Besitz (Muster S-63). |
| L4 | `compat/`-Brücken und Feature `workspace`, das per Default wieder einschaltet | Baukasten-Entscheidung F2, F3 | Keine Brücken; `default = []`; R0 und R1 kennen R2 nicht. |
| L5 | God-Crates: `engine` (13 k LOC) wandert nach `db`, das jetzt 23 313 LOC hat (davon `collection/` 12 550) | ADR-0122, Inventar | Nach Use-Cases zerlegen, nicht in einen größeren Hub verschieben. |
| L6 | Deprecated-Re-Exports (48 Warnungen), `#![allow(deprecated)]` | `baukasten-status.md` | Im Greenfield keine Deprecation. |
| L7 | Seam-Duplikate: `Clock` in 10 Crates, `DocId` in 7; `erasure/types_local.rs` 2 439 LOC | Baukasten-Entscheidung F4 | Ein Eigentümer je Seam (`acta-env`, `acta-ids`); Gate G-DUP. |
| L8 | Verdeckte Kopplung durch Shims (`pub use contextra_engine as engine`) | `cognition/src/engine_local.rs` | Shim-Verbot (G-SHIM). |

### 6.2 Zielbild

```
R7  acta (Fassade, dünn) · acta-cli · acta-mcp · acta-verify (standalone)
R6  acta-node       ← einzige Stelle, die alle konkreten Crates verdrahtet
R5  acta-engine (Use-Cases) · acta-chronicle · Mission-Module (privacy, router …)
R4  acta-erasure-adapters   (implementiert ErasureLayer für Store, Vektor, Text, Graph, Cache, Checkpoint)
R3  acta-erasure      R1  acta-store · mvcc · text · vector · graph · rank · checkpoint
R2b acta-ports   (schmale Taille: ErasedView, Lesesicht des Gate, SnapshotRevoker, Barrier-Permits)
R2a acta-types   (Werte, Referenzen, ErrorClass, Lineage und Derived<T>, Filter)
R0  acta-ids (Tags, DocOrd, DataClass, Seq) · acta-env · acta-crypto · acta-sys · acta-durable-fs · acta-simd · acta-wire
```

**\[E\] ADR-A1:** SPEC §3.1 führt `types` und `ports` im selben Ring R2 und verbietet Querkanten, während `ports` auf `types` aufbaut (heute in Contextra und Acta so). Vorschlag: Unterteilung in R2a und R2b, `types` strikt unterhalb von `ports`. Zusatz: Tag-Typen (`TenantTag`, `CollectionTag`, `SubjectTag`, `DocTag`), `DocOrd(u32)` und `DataClass` wandern nach `acta-ids` (R0), weil `acta-crypto` sie für die AAD braucht und R0 nicht nach R2 sehen darf (A-06).

### 6.3 Hub-Regeln HR-1 bis HR-10 \[E\]

| Regel | Inhalt | Gate |
| --- | --- | --- |
| HR-1 | Hubs sind ausschließlich `acta-types` und `acta-ports`. Es gibt keinen `engine`- oder `db`-Hub. | G1 |
| HR-2 | `acta-types` enthält nur Wertetypen, Referenzen, Taxonomien (`ErrorClass`), Lineage und Filter-AST; keine Algorithmus-Konfiguration, keine Fehler fremder Domänen. | G-HUB-3 |
| HR-3 | `acta-ports` enthält nur Traits, deren Erbringer und Nutzer in Ringen liegen, die sich nicht sehen (R1 gegen R3). | G-HUB-2 |
| HR-4 | **Verbraucher-Besitz:** Jeder andere Trait wird im Crate des Verbrauchers definiert; Erbringer oder R4-Adapter implementiert ihn (Dependency Inversion). | G-PORT-OWNER |
| HR-5 | **Rule of Two:** Ein Element kommt erst in einen Hub, wenn mindestens zwei Crates es nachweislich nutzen. | G-HUB-2 |
| HR-6 | Ein Fehlertyp je Crate (`thiserror`, `#[non_exhaustive]`, kein `Other(String)`); Aggregation nur in R4, R5, R6 per `#[from]`. | G-ERR |
| HR-7 | Keine `compat/`-Brücken, keine Features mit Rückkante nach oben; Bibliotheken haben `default = []`, Features sind additiv. | G-FEAT |
| HR-8 | Keine Shims, Alias-Re-Exports, `include!` und `#[deprecated]`. | G-SHIM |
| HR-9 | Die Fassade `acta` (R7) enthält keine Logik; `acta-node` (R6) ist der Composition Root. | G1 |
| HR-10 | Typ-Invarianten der SPEC liegen im besitzenden Crate (z. B. `Sealed<P>` in `acta-crypto`), nicht im Hub. | Compile-Fail-Tests |

**Hub-Budget (Startwerte, nach R2 anhand von Messungen kalibrieren) \[E\]:** `acta-types` ≤ 3 000 LOC; `acta-ports` ≤ 1 500 LOC und ≤ 12 öffentliche Traits; Fan-in je Hub-Element dokumentiert. Zum Vergleich der Ist-Stand im Acta-Klon: types 7 510 LOC, ports 6 199 LOC mit 43 Traits.

### 6.4 Verteilung der heutigen Hub-Inhalte \[E\]

| Heute (types bzw. ports) | Ziel | Begründung |
| --- | --- | --- |
| `error.rs` (46 Varianten) | Fehler je Crate; `acta-types` behält `ErrorClass` und `ClassifiedError` | L1, SPEC §10.2 |
| `PprConfig`, `TlHfdParams` | `acta-graph` | L2 |
| `DistanceMetric` | `acta-simd` (Eigentümer); Vector und Engine nutzen es | L2 |
| `TOMBSTONE_BIT`, `MAX_REF_LEN` | `acta-ids` | ADR-0120 |
| `MAX_SEARCH_K`, `saos.rs`, `RetrievalStrategy` | `acta-engine` | nur Retrieval-Orchestrierung |
| `TokenBudget`, `ResourceBudget` | Verbraucher (engine, router); nach Rule of Two ggf. `acta-types` | L7 |
| `SagaRecord`, `ScanMethod`, `ErasureId`, `GateId`, `WorkflowState`, `witness.rs`, `erasure.rs` | `acta-erasure` | SPEC §3.3: Löschtypen kanonisch im Löschkern |
| `tombstone.rs` | entfällt (Bitformat in `acta-ids`) | ADR-0120 |
| `ModelFingerprint`, `schema.rs` | bleiben in `acta-types`, falls ≥ 2 Verbraucher (Store-Manifest, Inferenz), sonst zum Verbraucher | HR-5 |
| ports: `attention`, `embedding`, `reranker`, `query_rewriter`, `inference`, `lifecycle`, `plugin`, `license`, `graph*`, `vector_index`, `text_index`, `kv*`, `storage` | Verbraucher-Besitz (engine, router, agent, license) | HR-4 |
| ports: `barrier`, Lesesicht von `gate`, `ErasedView`, `SnapshotRevoker` | bleiben in `acta-ports` | Taille zwischen R1 und R3 |
| ports: `clock`, `rng`, `id_gen`, `metrics`, `vfs`, `fs` | `acta-env` bzw. `acta-durable-fs` | SPEC §3.3 |
| ports: `erasure`, `erasure_legacy`, `scanner`, `ledger`, `key_material`, `keys`, `report_signer`, `conformance` | `acta-erasure` bzw. `acta-crypto` | SPEC §3.3 |

### 6.5 Fehlerarchitektur (Skizze) \[E\]

```rust
// acta-types (R2a): einzige gemeinsame Fehlerabstraktion
#[non_exhaustive]
pub enum ErrorClass { Transient, Integrity, Crypto, Erased, Policy, Invalid }
pub trait ClassifiedError { fn class(&self) -> ErrorClass; fn code(&self) -> &'static str; }

// acta-store (R1): eigener, geschlossener Fehler
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum StoreError { /* … */ }
impl ClassifiedError for StoreError { /* Integrity löst Poisoning aus (SPEC §10.2) */ }

// acta-node (R6): Aggregation nur hier
#[derive(Debug, thiserror::Error)]
pub enum NodeError { #[error(transparent)] Store(#[from] StoreError), /* … */ }
```

`ErrorClass::Integrity` und `ErrorClass::Crypto` lösen nach SPEC §10.2 die Vergiftung aus. So wirkt die Fehlertaxonomie, ohne dass ein Sammelenum im Hub entsteht.

## 7. Crate-Mapping: Contextra → Acta

Klassen: ADOPT, HARDEN, REWRITE, NEW, DEFER, DROP (Definition in 5.1). Aufwand: S, M, L, XL (relativ, nicht in Personenzeit kalibriert). LOC = `src/` am gepinnten Commit.

### 7.1 Fundament (R0) und Verträge (R2)

| Acta-Crate | Ring | Quelle (LOC) | Klasse | Nicht übernehmen bzw. Neuentwurf | Aufwand |
| --- | --- | --- | --- | --- | --- |
| acta-ids | R0 | ids (720) | HARDEN, in Teilen REWRITE | 16-Byte-Tags, `DocOrd(u32)`, `DataClass`, `TOMBSTONE_BIT`; keine öffentlichen Felder; kein `SYSTEM == DEFAULT == INVALID`. Golden- und Differential-Tests behalten. | S |
| acta-env | R0 | env (1 207) | HARDEN | Fallibler `CryptoRng` (`try_fill_bytes`); einziger Eigentümer von Clock, Rng, IdGen, MetricsSink; Metrics-Umfang (408 LOC) prüfen. | S |
| acta-sys | R0 | sys (2 402; 23 unsafe) | HARDEN | SAFETY-Audit; `mlock`, `MADV_DONTDUMP`, `RLIMIT_CORE = 0` (A-RAM). Speichersicherer Puffer aus `vault` (529 LOC, aus dem Workspace-Ausschluss holen) integrieren. | M |
| acta-durable-fs | R0 | durable-fs (1 910) | HARDEN | Vfs-Port erweitern: atomares Rename mit `fsync(dir)`, sicheres Scrub und Unlink, `FaultVfs`; einziger Dateizugriff im Workspace. | M |
| acta-crypto | R0 | crypto (6 290), erasure/crypto\_local, erasure/vault/sealer | REWRITE (Kern), ADOPT (`audit_chain`, nach Prüfung) | XChaCha20-Poly1305 v3, HKDF-Hierarchie H1 bis H5, `KeyAnchor`, `ReportSigner`, `Sealed<P>`, Commitments; eine Hash-Ketten-Implementierung statt drei (crypto, agent, privacy). | L |
| acta-simd | R0 (unsafe-Insel) | simd (3 627; 287 unsafe) | ADOPT nach SAFETY-Audit | `DistanceMetric` gehört hierher; Zeitpunkt R5. | M |
| acta-wire | R0 (unsafe-Insel) | wire (3 234; 45 unsafe) | DEFER | erst für CLI- und MCP-IPC (R7). | S |
| acta-types | R2a | types (7 510) | REWRITE nach 6.4 | Ziel ≤ 3 000 LOC. | M |
| acta-ports | R2b | ports (6 755) | REWRITE nach 6.4 | Ziel ≤ 1 500 LOC, ≤ 12 Traits. | S bis M |
| acta-testkit | dev | testkit (5 446) | HARDEN | `FaultVfs`, `ManualClock`, `SeededRng`, Konformitäts-Suites; früh bereitstellen, weil V-14 an fehlenden Symbolen scheiterte. | M |

### 7.2 Persistenz, Indizes, Löschkern (R1, R3, R4)

| Acta-Crate | Ring | Quelle (LOC) | Klasse | Nicht übernehmen bzw. Neuentwurf | Aufwand |
| --- | --- | --- | --- | --- | --- |
| acta-store | R1 | store (32 015; 160 Testdateien) | REWRITE der Struktur; Ideen und Tests als Orakel | KeyV2 (78 Byte, binär), Partitionen N/P/D, Manifest v3, SSTable 5 mit `source_set`, WAL-Frame v2, Siegelung vor dem WAL, `FlushPurgeFence` verdrahten, V-17 beheben, 57 direkte fs-Aufrufe durch Vfs ersetzen, Laufzeitmodell nach ADR-A3. | XL |
| acta-mvcc | R1 | mvcc (4 858) | HARDEN | `MvccRevoker` (SnapshotRevoker), `as_of` nur über `ErasedView`, drei Zeitachsen (valid, tx, erase), `TOMBSTONE_BIT` aus `acta-ids`. | M |
| acta-text | R1 | text (9 535) | HARDEN (Walking-Skeleton-Index) | Lineage (`source_set`), `put_derived`, Segment-Purge, Residue-Scan; BM25, WAND und die deutsche Morphologie (0 Contextra-Referenzen) übernehmen. | M bis L |
| acta-checkpoint | R1 | checkpoint (5 593) | HARDEN | `CopyRegistry` (V-10), Hardlinks nur über Vfs, `CheckpointLayer`. | M |
| acta-vector | R1 | vector (17 744; 10 unsafe) | HARDEN (Zeitpunkt R5) | Lineage, physisches Nullen, Rebuild-Modus, Sealed nur ≤ 50 000 Vektoren, Embeddings als Ebene P; DiskANN (experimentell) DEFER. | L bis XL |
| acta-graph | R1 | graph (23 593) | HARDEN (Zeitpunkt R5) | 167 `types`-Verwendungen im Kern entfernen; Lineage auf Kanten; `experimental-graph` aus den Defaults. | L bis XL |
| acta-rank | R1 | rank (5 852) | ADOPT | autark; `exp-*` aus den Defaults; Determinismus-Test T-RANK-1. | S |
| acta-erasure | R3 | erasure (22 910) | REWRITE modulweise (7.3) | siehe 7.3 | XL |
| acta-erasure-adapters | R4 | NEW (Wissen aus `docs/erasure/loeschkette_zuordnung.csv`, 4 546 Einträge) | NEW | `ErasureLayer` je Store, Vektor, Text, Graph, Cache, Checkpoint; keine Logik des Löschkerns duplizieren. | L |

### 7.3 Modulplan `contextra-erasure` → `acta-erasure`

| Contextra-Modul (LOC) | Acta-Ziel | Klasse | Anmerkung |
| --- | --- | --- | --- |
| `vault/` + `vault.rs` (3 577) | `::vault` | REWRITE | Slots U, T, C, S, D, F, E; Epochenmaskierung und Rotation; `destroy_many`; Grabstein nullt `tag` und `payload` (V-11, ist in Contextra bereits so) |
| `gate/` (1 367) | `::gate` | REWRITE | `ErasedOrds` als Roaring-Bitmap, `gate_epoch`, GateLog |
| `journal/` (1 435) | `::journal` | HARDEN | Crash-Matrix C1 bis C12 |
| `saga/` (1 437), `orchestrator.rs` | `::saga` | REWRITE | Reihenfolge E0, E1, E2, E2a, E2b, E3 … E9; `Pending`-Gründe und Backoff; kein Zustand `Failed` |
| `measure/` (1 466) | `::measure` | REWRITE | `Scanner<Unarmed/Armed>`, Messklassen MK1 bis MK3 |
| `purge/` (637) | `::purge` | REWRITE | Phasen a bis h, `PurgePriority` |
| `report/` (1 555), `deletion_proof.rs` (2 743), `ed25519_proof.rs` | `::report` | REWRITE und Konsolidierung | JCS (RFC 8785), Klassen G, K, P, D, C, Annahmen im Bericht |
| `ledger/` (1 538), `revocation_log.rs` (466) | `::ledger` | REWRITE | keyed Commitments, Epoche, Vernichtung nach 90 Tagen (S12, V-15) |
| `restore/` (144) | `::restore` | REWRITE | `RestoreGuard`, `T_ROT_BACKUP` = 72 h |
| `scope/` (905) | `::scope` | HARDEN | verschlüsselte Scope-Datei, Single-Flight |
| `supersede`, `replace`, `restricted` (≈ 520) | `::modes` | HARDEN | Art. 16 und 18, Replace |
| `kv_shredding.rs` (982; AES-GCM-SIV) | später bei `acta-kvcache` | DEFER | Algorithmuswechsel nötig (ADR-A2) |
| `types_local.rs` (2 439), `crypto_local.rs` (274), `io.rs`, `test_mocks.rs` | — | DROP | Duplikate von ids, types, crypto, env, durable-fs |

### 7.4 Anwendung und Integration (R5 bis R7)

| Acta-Crate | Ring | Quelle (LOC) | Klasse | Anmerkung | Aufwand |
| --- | --- | --- | --- | --- | --- |
| acta-chronicle | R5 | NEW; Wissen aus `crypto/audit_chain`, `agent/audit`, `privacy/context_edit_audit` | NEW | N-Validator (Whitelist, Entropie, `K_MIN`), `ChronicleEntry`, Hash-Kette, Checkpoint alle 1 024 Einträge, Takedown im Vier-Augen-Prinzip, zufällige Handles | L |
| acta-engine | R5 | db (23 313) | REWRITE nach 7.5 | Use-Case-Schicht, kein Hub | XL |
| acta-node | R6 | NEW; Verdrahtung aus `db/contextra_impl`, `backends.rs` | NEW | Composition Root, Boot-Selbsttest (nicht abschaltbar), Konfiguration ohne Default für `k_idx` | M |
| acta | R7 | contextra (2 042) | NEW (dünn) | kuratierte API, keine Logik | S |
| acta-cli | R7 | NEW | NEW |  | S |
| acta-verify | R7 | NEW | NEW | eigenständig, ≤ 800 LOC, keine Acta-Abhängigkeit (SPEC §8.4) | M |
| acta-mcp | R7 | mcp (4 813; Closure 22) | DEFER, dann HARDEN | nur gegen die Fassade `acta`; `forget` ausschließlich über `ErasureCore::execute` (G-ERA-1) | M |

### 7.5 Zerlegung von `contextra-db` (statt Verschiebung in einen neuen Hub)

| Contextra (`db`, LOC) | Acta-Ziel | Hinweis |
| --- | --- | --- |
| `collection/` (12 550) | `acta-engine::{ingest, retrieval, crud}` | Datenklasse bei jedem Schreibvorgang (R-CLS), Gate vor jedem Lesepfad |
| `transaction/` (1 658) | `acta-engine::tx` | Compensating Actions, Intent-Journal mit dem Erasure-Journal abstimmen |
| `chunker/` (1 533), `extraction/` (642) | `acta-ingest` (Mission-Modul) bzw. `acta-engine::chunk` | PII-Hinweise am Chunk |
| `background_workers/` (1 298), `decay_controller.rs` | `acta-engine::maintenance` | Retention- und Expiry-Sweeper; Decay nur auf Ebene D und N, nie als Löschersatz |
| `contextra_impl/` (1 452), `backends.rs` | `acta-node` | Verdrahtung, Boot-Selbsttest |
| `export.rs`, `import.rs`, `filter.rs`, `temporal_filter.rs`, `fusion.rs` | `acta-engine`; Filter-AST ggf. `acta-types` | Export nur über gegateten Reader |
| `multistep.rs`, `pid_latency_controller.rs`, `homeostat.rs`, `cognition/` | Mission-Module `acta-adapt`, `acta-cognition` (DEFER) | nicht im Kern |

### 7.6 Mission-Module außerhalb des SPEC-Rings

Sie benötigen eine SPEC-Ergänzung (Ringplatzierung, ADR-A7); bis dahin gilt die vorgeschlagene Platzierung als Annahme.

| Modul | Quelle (LOC) | Säule | Klasse | Platzierung | Bedingung |
| --- | --- | --- | --- | --- | --- |
| acta-privacy | privacy (5 782) | Datenschutz, Souveränität | HARDEN | Detektoren (PII, DLP) als R1-Leaf `acta-pii`; Egress-Policy in R5 | Der N-Validator braucht den Detektor unterhalb von R5. Namenskollision „Vault“ (Surrogate-Vault gegen speichergesicherten Puffer) auflösen. |
| acta-prompt-guard | prompt-guard (1 389) | Präzision, Kontrolle | ADOPT | R1-Leaf | Signaturen pflegen; keine Aussage „schützt vollständig“ |
| acta-ingest | ingest (21 140; 2 unsafe) | Verwaltung, Organisation | HARDEN | R1-Leaf, nicht vertrauenswürdige Eingaben | Fuzzing, Dependency-Review (z. B. fest gepinntes `zip =0.6.6`, `lopdf`, `calamine`), Ressourcenlimits, Datenklasse Pflicht |
| acta-adapt, acta-router | adapt (5 428), router (8 337) | Effizienz | ADOPT bzw. HARDEN | R1-Leaf bzw. R5 | Kaskadenrouting vom kleinen zum größeren Modell; deterministisch, wenn Entscheidungen in die Chronik fließen |
| acta-kvcache | kvcache (7 811) | Effizienz | DEFER | R1 | KV-Blöcke sind Ableitungen aus Personendaten: `CacheLayer`, Lineage, Siegelung nach ADR-A2 |
| acta-infer-candle, -onnx, -ollama | 6 808, 1 016, 6 782 | Souveränität | HARDEN | Adapter (R4/R5) | `Embedder`- und `Generator`-Traits gehören `acta-engine` (HR-4); Remote nur explizit (Zone Z4, A-EXT) |
| acta-sandbox | sandbox (2 377) | Kontrolle | DEFER | R1-Leaf | WASM-Ausführungsgrenze |
| acta-agent, acta-cognition | agent (5 937), cognition (8 478) | Entscheidungs- und Verwaltungshilfe | DEFER bis nach R6 | R5 oder höher | Audit über `acta-chronicle` statt eigener Kette; kein konkreter `Collection`-Typ (Shim-Befund F6) |
| acta-audit-export, acta-avv-generator | audit-export (992), avv-generator (1 078) | Transparenz, EU-Datenschutz | HARDEN | R7 | Art.-30-Register und AVV-Vorlagen nur mit Bezug auf Messklassen und Annahmen (G-WORD) |
| acta-license, acta-py | license (1 143), py (3 529) | — | DEFER | — | `license` ist laut Contextra-Entscheidung D-08 Tier frozen |

## 8. Mission-Säulen → Komponenten → Abnahme

| Säule | Komponenten | SPEC-Anker | Messbare Abnahme |
| --- | --- | --- | --- |
| Souveränes Unternehmenswissen | Tresor und Store on-premise, `KeyAnchor` (HSM oder KMS beim Kunden), lokale Inferenz, kein Egress im Default | §3.3, §9.4 | Test `no_egress_default`; Schlüssel nur im Kundenanker; Zone Z4 im Bericht als A-EXT ausgewiesen |
| Transparenz und Nachvollziehbarkeit | Chronik (Norm, Policy, Config, Model, Rule, Receipt), Ledger-Hash-Kette, signierter Erasure-Report, `acta-verify`, Art.-30-Export | §7.4, §8 | T-CHR-1 und T-CHR-2; `as_of` byte-identisch (S6); `acta-verify` besteht ohne Acta-Code |
| Präzision und Kontrolle | Rank (Kalibrierung, Drift), Gate vor jedem Lesepfad, Prompt-Guard, Datenklassen R-CLS, Budgets | §2.2, §6 | `ClassViolation` ohne Klassendeklaration; T-RANK-1; Recall- und Precision-Benchmarks als Messwerte |
| Effizienz und Ressourcenschonung | SIMD, Quantisierung (SQ8, RaBitQ), Kaskadenrouting, KV-Cache, Rebuild statt Sealed, Batch-Vernichtung `destroy_many`, Gruppen-Commit | §4.6, §5.5, §10.3 | Gate-Latenz unabhängig von der Datenmenge (R-O1); Benchmarks gegen die Contextra-Baseline |
| Schutz personenbezogener Daten | Tresor, Umschlag v3, Crypto-Shredding, Lineage, Gate, PII-Erkennung, Embeddings als Ebene P | S1 bis S4, S10 bis S12 | T-CRY-1 (Bytes-Grep), T-DEL-1, `hnsw_no_residue_after_erase` |
| Verwaltung und Organisation von Daten | Mandanten und Collections, Partitionen N/P/D, Retention-Klassen, Sweeper, Checkpoints, Ingest | §2.2, §6.3 | Retention-Tests je Datenklasse; T-REST |
| Entscheidungs- und Verwaltungshilfe | Chronik als Entscheidungsnachweis, Zeitreise, Handle H\_d, später Agent und Cognition | §7.4 | Entscheidung nach Löschung der Person rekonstruierbar; Eingabe-Referenz liefert `Erased` |
| Europäischer Datenschutz | Abläufe zu Art. 5, 12, 16 bis 19, 25 (erase, restrict, supersede), SLA 24 h und 72 h, Art.-19-Meldung, SDM-Baustein-60-Zuordnung, G-WORD | §9 | Fristentests (`legal_max` 30 Tage), Wortfilter, Annahmen im Bericht; Rechtsfreigabe G5 vor Release |

Hinweis: Dieser Plan ist keine Rechtsberatung. Aussagen zur Rechtslage stammen aus der SPEC §9.

## 9. Roadmap nach SPEC §12

Prinzip: dünner vertikaler Durchstich vor horizontaler Breite. Jede Stufe hat ein festes Abnahmekriterium; ohne grünes Kriterium beginnt die nächste Stufe nicht.

### R0 Stop-the-Line (V-01, V-03, V-05, V-13, V-14, V-16; A-02, A-03, A-05, A-06, A-11, A-12)

- **WP-0.1 Baseline:** Toolchain 1.89.0 bereitstellen; `cargo check` und `cargo test` je Crate in `acta/` und im Contextra-Klon; Messprotokoll (Warnungen, Testläufe, Laufzeiten).
- **WP-0.2 `acta-crypto` fail-closed:** `seal_record` liefert `Result`; `derive_dek` meldet Fehler bei fehlendem Salt und bei HKDF-Fehlern; Kante nach `acta-types` und `acta-ports` entfernen; `default = []`.
- **WP-0.3 Workspace-Hygiene:** `cargo xtask gates` (G1, G-DEF-1, G-NOREN); `clippy.toml` bereinigen (Clock-Pfad, `thread_rng`-Ausnahme); `tokio` nicht im Default der Kerne; Persistenzformate nicht auf `bincode` 1.x gründen (Pflegestatus per `cargo audit` prüfen).
- **WP-0.4 Hub-Inventur:** Werkzeug zur Messung von LOC, Traits und Fan-in je Hub-Element (Grundlage für HR-5).
- **WP-0.5 ADR-Paket A1 bis A7** entscheiden (Abschnitt 12).
- **Abnahme:** `cargo test --workspace` grün; Pflichttests V-05 grün; G1 und G-NOREN grün.

### R1 Krypto-Kern und Tresor (V-02, V-04, V-12, V-16; A-01, A-04, A-10)

- **WP-1.1** `acta-ids`: Tags, `DocOrd`, `DataClass`, keine öffentlichen Felder.
- **WP-1.2** `acta-env`: fallibler `CryptoRng`; Fault-Injection-Test T-RNG-1.
- **WP-1.3** Umschlag v3: XChaCha20-Poly1305, AAD nach §4.4, Nonce nur intern; Testvektoren aus `draft-irtf-cfrg-xchacha` und eine unabhängige Gegenimplementierung.
- **WP-1.4** Schlüsselhierarchie H1 bis H5; `resolve_dek` löst vier Tags getrennt auf; fehlender Slot liefert `Err(Erased(SlotType))`.
- **WP-1.5** Tresor: Layout §4.5, Maskierung, Epochenrotation (Crash C11), `destroy_many`, Grabstein.
- **WP-1.6** `KeyAnchor` (`FileAnchor` nur für Dev, KMS und HSM als Trait-Implementierungen) und `ReportSigner`.
- **Abnahme:** Testvektoren; `envelope_is_aead`; `slot_payload_masked_at_rest`; `vault_missing_slot_is_erased`; `no_default_sealer_in_release`; Bytes-Grep über Tresordateien.

### R2 Walking Skeleton (V-08, V-09 teilweise, V-15)

Pfad: Ingest (`Sealed<P>`) → WAL → `acta-text` mit Lineage → `ErasureCore` synchron (E0 bis E3) → Messung MK2 → Bericht.

- **WP-2.1** Hub-Neuschnitt, nur soweit der Durchstich ihn braucht.
- **WP-2.2** `acta-durable-fs` mit `FaultVfs`; `acta-store` Minimalpfad (WAL v2, Memtable, reduzierter KeyV2).
- **WP-2.3** `acta-text` mit `Derived<T>`; Maskierung über `ErasedOrds` im Suchpfad.
- **WP-2.4** `acta-erasure`: Gate, Journal, Saga E0 bis E3, MK2, kanonischer Bericht.
- **WP-2.5** Kanarien-Test Ende-zu-Ende; Crash-Tests C1 bis C6.
- **Abnahme:** T-DEL-1; C1 bis C6; ein Bericht ist mit einem unabhängigen Prüfer verifizierbar.

### R3 bis R7

| Stufe | Inhalt | Behobene Befunde | Abnahme |
| --- | --- | --- | --- |
| R3 Datenmodell | monotone `doc_ord`-Lineage, KeyV2 mit `cluster = subject_tag`, Partitionen N/P/D, Manifest v3, SSTable 5, Siegelung vor dem WAL | V-06, V-07 | Invarianten I-5, I-6, I-7; Tests `no_ascii_keys`, `no_v2_envelope_in_P` |
| R4 Store-Fähigkeiten | `FlushPurgeFence` verdrahtet, gefilterter Rewrite mit Datei-Drop, WAL-Frame v2 mit `chain_tag` | V-08, V-17 | Loom-Modelle Flush gegen Purge; Crash C7 und C8; Compaction-Stresstest auch auf einem Kern |
| R5 Indizes und Schichten | `acta-vector`, `acta-graph`, Cache, Checkpoint mit `CopyRegistry`, `acta-erasure-adapters` | V-09, V-10 | T-VEC, T-TEXT, T-GRAPH, T-REST; `hnsw_no_residue_after_erase` |
| R6 Chronik und Node | `acta-chronicle`, `acta-engine`, `acta-node`, Boot-Selbsttest | V-14 (Rest) | G1, G2, G-ERA-1, G-READ-1; T-CHR-1 und T-CHR-2 |
| R7 Audit und Härtung | `acta-verify`, Notary und HSM-Signatur, externe Krypto-Prüfung (G4), Rechtsfreigabe (G5), Fassade, CLI, MCP | D-12 | unabhängiges Audit ohne Acta-Code; Messwerte statt Zielwerte |

### Parallele Spuren ab R2

- **M1 Datenschutz und Eingang:** acta-pii, acta-prompt-guard, acta-ingest (Fuzzing von Anfang an)
- **M2 Effizienz:** acta-simd, acta-adapt, acta-router, später acta-kvcache
- **M3 Souveränität:** lokale Inferenz-Adapter hinter den Embedder- und Generator-Traits der Engine
- **M4 Nachweise:** acta-audit-export und acta-avv-generator ab R6
- **M5 Entscheidungshilfe:** acta-agent und acta-cognition erst nach R6

## 10. Qualität: Gates und Tests

| Gate | Prüfung | Herkunft |
| --- | --- | --- |
| G1 | Ringmodell und erlaubte Kanten per `cargo metadata` | SPEC §8.5 |
| G2, G3 | Jeder Port genau einmal verdrahtet; keine nicht injizierte Zeit oder Zufallsquelle | SPEC §8.5 |
| G-CRY-1, G-FS-1, G-ERA-1, G-READ-1, G-DEF-1, G-WORD, G-SUPPLY | Krypto-Fallbacks, direkter Dateizugriff, Löschpfad, gegateter Lesepfad, `default-members`, Wortfilter, `cargo deny` | SPEC §8.5 |
| **G-NOREN** | Muster aus 5.3 in Produktionscode | dieser Plan |
| **G-HUB-1 bis 3** | LOC-Budget der Hubs; Rule of Two; keine Algorithmus-Konfiguration oder Fremdfehler im Hub | dieser Plan |
| **G-PORT-OWNER** | Trait nur in `acta-ports`, wenn er ringübergreifend benötigt wird | dieser Plan |
| **G-ERR, G-FEAT, G-SHIM, G-DUP** | Fehler je Crate; additive Features, `default = []`; keine Shims und Aliase; keine doppelten Seams (Clock, Rng, DocId) | dieser Plan |
| **G-UPSTREAM** | `UPSTREAM.md` je portiertem Crate vorhanden und aktuell | dieser Plan |

**Testarten:** Golden-Tests (kanonisches JSON nach JCS, Testvektoren), Differential-Tests gegen Contextra für übernommenes Verhalten, `proptest`, Loom (Gate, Fence, Barrier), Crash-Matrix mit `FaultVfs`, Mutationstests (`acta/.cargo/mutants.toml`; Contextra führt zusätzlich `mutants-guarantees.toml`), Fuzzing (Ingest, WAL-Parser, Wire), Compile-Fail-Tests (S10, S11), `cargo deny` und `cargo audit`. Eine Zusage ohne automatisierten Test gilt laut SPEC als nicht erfüllt.

## 11. Risiken

| Risiko | Einschätzung | Gegenmaßnahme |
| --- | --- | --- |
| Store-Neubau dauert länger oder regressiert | hoch | Durchstich zuerst; 160 Contextra-Testdateien als Orakel; Store-Formate früh einfrieren (Golden) |
| Contextra bewegt sich weiter | mittel | Pin, Upstream-Ledger, Diff pro Iteration |
| SPEC-Unklarheiten blockieren R1 | hoch | ADR-Paket A1 bis A7 in R0 |
| Krypto-Eigenbau | hoch | nur geprüfte Bibliotheken (RustCrypto), keine eigenen Primitive; externes Audit (G4) |
| Laufzeitmodell spät entschieden | mittel | ADR-A3 vor R2, weil es Store-, Checkpoint- und Vfs-APIs prägt |
| Falsche Compliance-Aussagen | mittel | G-WORD, Rechtsreview (G5) |
| Angriffsfläche der Dokument-Parser | hoch | Fuzzing, Ressourcenlimits, Sandbox, Dependency-Review |
| Baubarkeit des Ist-Stands unbekannt | mittel | WP-0.1 vor jedem Eingriff |
| Umfang (ca. 280 k LOC Quelle) | hoch | Klassen ADOPT, DEFER und DROP konsequent nutzen; keine Zeitschätzung ohne Team- und Kapazitätsdaten |

## 12. Offene Entscheidungen (ADR-Paket vor R1)

| ADR | Frage | Empfehlung \[E\] |
| --- | --- | --- |
| A1 | Ring R2 in R2a (types) und R2b (ports) teilen; Tags, `DocOrd`, `DataClass` nach `acta-ids` | ja (6.2) |
| A2 | AEAD und Nonce: SPEC §4.1 verlangt XChaCha20-Poly1305, der Acta-Code nutzt AES-256-GCM-SIV | der SPEC folgen; Algorithmus-Byte `alg` für Krypto-Agilität (EDPB-Forderung, §9.2); AES-GCM-SIV nur, wenn ein HSM-Key-Wrap es erzwingt |
| A3 | Laufzeitmodell: Contextra-Kern ist async (710 async-Funktionen workspace-weit laut Baseline), die SPEC verbietet tokio im Default und `.await` unter Locks | R0 bis R3 und Store blockierend mit Threads; async nur an den Rändern (node, cli, mcp, Inferenz). Das vereinfacht Loom- und Crash-Tests. Alternative: async-Store behalten. |
| A4 | Commitment-Funktion: §4.1 nennt BLAKE3 keyed, §6.6 HMAC-SHA256 | eine festlegen; Empfehlung HMAC-SHA256 wie §6.6 |
| A5 | Domänenpräfixe: SPEC mischt `acta/kdf/v3` mit `ctx/report/v1` und `ctx/slotmask/v1` | einheitlich `acta/…`; Anhang A.1 der SPEC anpassen |
| A6 | Herkunft der Tags (`tenant_tag`, `collection_tag`, `subject_tag`, `doc_tag`, je 16 Byte): zufällig oder keyed aus externer ID | Annahme \[A\]: keyed Ableitung mit `k_idx` ohne Default; mit dem SPEC-Autor bestätigen |
| A7 | Ringplatzierung der Mission-Module (acta-pii unter acta-chronicle, Inferenz-Adapter, Router); Umgang mit dem Namen `acta-erasure-spi` (A-08) | wie in 7.6; `acta-erasure-spi` streichen |

Zusätzlich prüfen: Slot-MAC auf 64 Bit gekürzt (§4.5). Das ist bei geheimem MAC-Schlüssel vertretbar, sollte aber im Krypto-Review ausdrücklich bestätigt werden.

## 13. Nächste Schritte (erste zwei Wochen)

1. Toolchain bereitstellen und die Baseline für `acta/` und Contextra messen (WP-0.1).
2. A-02 und A-03 beheben, jeweils mit Pflichttest; kleinste und sicherste Änderungen mit sofortigem Nutzen.
3. ADR-Paket A1 bis A7 in einem Entscheidungstermin klären.
4. `xtask gates` mit G1, G-DEF-1 und G-NOREN aufsetzen.
5. Hub-Inventur-Werkzeug bauen und die Zuordnung aus 6.4 als Tabelle ins Repository legen.
6. SPEC-Slice für R1 (Krypto) schreiben und Testvektoren beschaffen.

## Anhang: Methodik und Grenzen

- **Gelesen:** `acta/docs/SPEC.md` vollständig; `docs/HUB_REMOVAL.md`, `docs/CRATE_DECOUPLING_MATRIX.md`, `docs/decoupling/BAUKASTEN_ARCHITEKTURENTSCHEIDUNG.md` (erste Hälfte), `docs/decoupling/README.md`, ADR-0120, ADR-0121, ADR-0122, ADR-078, `docs/generated/baukasten-status.md`, Kopf von `docs/extract_acta.py` und `docs/erasure/loeschkette_zuordnung.csv`, alle Crate-Manifeste, Stichproben im Quelltext der zitierten Dateien.
- **Nicht gelesen:** `docs/crate-coupling-report.md` (290 KB), Prompt-Katalog, Crate-Interna jenseits der zitierten Stellen. Zeilenverweise gelten für Commit `50cdc3d`.
- **Zahlen:** LOC sind Zeilen in `src/**/*.rs`; die unsafe-Zahl ist eine grobe Zählung des Schlüsselworts; Grep-Zählungen sind ungefiltert, sofern nicht anders angegeben.
- **Nicht gebaut, nicht getestet:** Die SPEC-Befunde V-13, V-14 und V-17 wurden nicht reproduziert. Aufwandsklassen sind relative Schätzungen.
- Dieser Plan ist keine Rechtsberatung; die Rechtsfreigabe (G5) bleibt Teil von R7.
