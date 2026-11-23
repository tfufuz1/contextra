# G-1-Nachweis (SPEC §22)

**Paket:** P3 · Gate-G-1-Nachweis
**Status des Nachweises:** gemessen, Urteil je Punkt unten. Die Gate-Feststellung trifft der Maintainer.
**Gate-Stand laut Messung:** G-1 **nicht erfüllt** (Workspace-Build nicht grün; mehrere Port-Punkte offen). Das ist ein Befund, keine Entscheidung.

## 1. Basis und Prüfstand

| Größe | Wert |
|---|---|
| Basis-Commit (`main`) | `cf870dcd1` (Shell-Commit auf `1e79a72ad`) |
| Prüfstand | `main` + PR-Kette #6060, #6061, #6063, #6064, #6065, #6066, #6067 (Head `ccfecb561`), gemergt ohne Konflikte |
| Nicht im Prüfstand | #6062 (reine Dokumentation, eigener Zweig) |
| P1 / P2 | P1 = WP-1.x-Kette (#6060–#6067), **nicht in `main`**; daher deren PR-Heads verwendet. P2 ist nicht eindeutig benannt; Annahme: P2 ist in `main` enthalten (WP-1.9, #6059). Bitte bestätigen. |
| Toolchain | `rustc`/`cargo` 1.100.0 (conda-forge), Clippy 0.1.100. `rustup` war nicht erreichbar. MSRV laut D-17 = 1.89, erfüllt. IMPL §2.1 nennt 1.97.0 als Messstand; Abweichung vermerkt. |
| Plattform | Ubuntu 24.04.5 LTS, x86_64 |
| CI | keine CI im Repository. Keine Aussage „CI grün“. |

Reproduzierbar mit:

```bash
git clone https://github.com/tfufuz1/contextra && cd contextra
git checkout -b integ/p1-heads main
git fetch origin pull/6067/head:pr-6067
git merge --no-ff pr-6067        # nur Prüfstand, nicht für einen PR
```

## 2. Prüfpunkte aus §22 (G-1)

Voraussetzung **G-0** (§22): erfüllt. `main` enthält die R8-Spezifikation mit Register (§23–§25).

Urteile: `erfüllt` · `nicht erfüllt` · `nicht prüfbar`.

| # | Punkt | Befehl | Ausgabeauszug | Urteil |
|---|---|---|---|---|
| 1 | `contextra-durable-fs` in `members` und `default-members` | `cargo check -p contextra-durable-fs` | `Finished dev profile … in 5.76s`, Exit 0 | **erfüllt** |
| 2 | `contextra-storage` bindet nur `contextra-ports[persistence]` ein | `cargo tree -e normal -p contextra-storage` | `└── contextra-ports v0.1.0` | **erfüllt** |
| 3 | Normalbaum frei von `contextra-types`, `blake3`, `serde`, `futures-util` | `cargo tree -e normal -p contextra-ports --no-default-features --features persistence` | nur `contextra-ports v0.1.0` | **erfüllt** |
| 4 | Testfeature frei von verbotenen Crates | `cargo tree -e normal -p contextra-storage --features testing` | `blake3 v1.8.7`, `aes-gcm-siv v0.11.1` | **nicht erfüllt** (nur Feature `testing`; D-05 offen, IMPL §2.3) |
| 5 | Modul `persistence` ohne verbotene Imports (D-02) | `grep -n '^use' src/persistence.rs` | nur `std::io`, `std::path::Path` | **erfüllt** |
| 6 | Doppel-`Clock` entfällt (D-02) | `grep -rn 'pub trait Clock'` | `src/clock.rs:8` **und** `src/persistence.rs:171` | **nicht erfüllt** (Abweichung dokumentiert in SPEC §27) |
| 7 | `KeyDomain` → `ClusterId` (D-02, WP-1.3) | `grep -n KeyDomain src/persistence.rs` | `pub struct KeyDomain(pub u32)` in Zeile 254; `KeyProvider` nutzt weiter `KeyDomain` | **nicht erfüllt** |
| 8 | Fehlende Ports ergänzt (D-02) | `grep -rn 'pub trait …'` | vorhanden: Checksum, Hasher, Anchor, Observer, Weigher, Filter, ColdTier, Runtime, ExtensionBuilder; **fehlt: `SecretAlloc`** | **nicht erfüllt** |
| 9 | `Cipher` mit 24-Byte-Nonce (WP-1.3) | `grep -n 'fn seal' src/persistence.rs` | `nonce: &[u8; 12]` (Zeile 233) | **nicht erfüllt** (UE-10 offen, WP-2.4) |
| 10 | `persistence.rs` ≤ 400 Zeilen (P-6) | `wc -l src/persistence.rs` | `629` | **nicht erfüllt** (Codierregel; IMPL §2.2 nannte 352) |
| 11 | `MemFs` als Test-Fake | `cargo test -p contextra-storage --test fs_mem_test` | `4 passed; 0 failed` | **erfüllt** |
| 12 | `FaultFs` injiziert an jedem Port-Aufruf (§19.1) | Trait-Abgleich, siehe Abschnitt 3 | `Fs::reflink` und `File::punch_hole` nicht überschrieben; Trait-Default `Unsupported`; Torn-Modus über `FaultFs` ungetestet | **nicht erfüllt** |
| 13 | `fs_suite` vollständig (WP-1.4: Torn-Modus, FsCaps, punch_hole) | `grep -n torn src/testing/conformance/fs_suite.rs` | keine Treffer; Suite prüft `truncate`, `punch_hole`, `hard_link`, `reflink`, `caps` | **nicht erfüllt** (Torn fehlt) |
| 14 | Konsumentenmatrix (§26.1) verifiziert | `grep -rh contextra_store --include=*.rs crates/<c>` je Konsument | engine 144, db 25, graph 16, cognition 15, Fassade 5, agent 3, checkpoint 2, bench 1, py 0; Summe ohne eigene Zeilen **211** | **erfüllt** (Zählung). Abbildungskorrekturen SA-6/7/10 eingereicht, in §26 offen. |
| 15 | Workspace-Build grün | `cargo check --workspace --keep-going` | 10 × E0308, Abschnitt 4 | **nicht erfüllt** |

Konsumenten unverändert (§26.4): Änderungen liegen nur in `crates/contextra-ports`, `crates/contextra-storage` und `Cargo.toml`/`Cargo.lock`. Das ist erfüllt.

### Testläufe (gemessen)

| Befehl | Ergebnis | Log |
|---|---|---|
| `cargo test -p contextra-storage --test layering --test invariants --test fs_mem_test` | layering 13/13, invariants 1/1, fs_mem_test 4/4 | `logs/test_layering_invariants_fs_mem.txt` |
| `cargo test -p contextra-storage --features testing --test testing --test env --test run` | testing 20/20, run 8/8, env 9/9 (1 ignoriert je Ziel) | `logs/test_testing_run_env.txt` |

Weitere Ziele (`loom`, `fs_real_test`, Absturzmatrix der Ziele `catalog`, `redo`, `engine`) sind **nicht ausgeführt**.

## 3. FaultFs und MemFs gegen den Port

Abgleich der Trait-Methoden mit den Implementierungen (Python-Auswertung über `persistence.rs` und `testing/*.rs`):

| Trait | Methoden | `FaultFs` | `MemFs` | Nicht überschrieben (Default) |
|---|---|---|---|---|
| `Fs` | 11 | 10 | 10 | `reflink` → `Err(Unsupported)` |
| `File` | 9 | 8 | 8 | `punch_hole` → `Err(Unsupported)` |

Folgerung: Für die beiden optionalen Methoden injiziert `FaultFs` keine Fehler. Das ist laut Spec-Text („an jedem Port-Aufruf“) eine Lücke. Torn-Schreiben (`CrashMode::Torn`) wird nur direkt auf `MemFs::crash_torn` getestet (`tests/testing/fault_fs_tests.rs`), nicht über `FaultFs`.

Fehlerinjektion, die vorhanden ist: EIO, ENOSPC, kurzes Schreiben (`torn_write_limit`), Absturz nach N Operationen, Absturzmodi Lost/Torn/Reordered.

## 4. Workspace-Build (Punkt 15)

`cargo check --workspace` → Exit 101. Mit `--keep-going` (Log: `logs/check_workspace_keep_going_integ.txt.gz`):

- **10 × E0308**, davon 9 in `contextra-vector` und 1 in `contextra-store` (`src/kv/segment.rs:147:42`).
- **Identische 10 Fehlerorte auf `main`** (`cf870dcd1`, Log: `logs/check_workspace_keep_going_main.txt.gz`). Die Fehler sind vorbestehend und nicht durch die P1-Kette entstanden.
- Durch diese beiden Wurzeln blockiert (Rückwärtsabhängigkeiten im Graphen): **9 von 38** Workspace-Mitgliedern — `contextra`, `contextra-agent`, `contextra-bench`, `contextra-cognition`, `contextra-db`, `contextra-engine`, `contextra-ingest`, `contextra-mcp`, `contextra-py`.

Die Gate-Bedingung „Workspace-Build grün“ ist damit **nicht erfüllt**. Die Entscheidung über die Auslegung liegt beim Maintainer (siehe Abschnitt 6).

## 5. Clippy (Regel 6)

Befehl: `cargo clippy -p contextra-ports -p contextra-storage --all-targets` auf `main` und im Prüfstand.

| Stand | Errors | Warnungen |
|---|---|---|
| `main` | 4 × `unwrap` (`engine/committer.rs` 563–580) | 120 |
| Prüfstand | dieselben 4 × `unwrap` | 121 |

- Die vier `unwrap`-Errors sind vorbestehend und identisch.
- Die übrigen Unterschiede sind größtenteils Modulverschiebungen (`engine/` → `maint/`, `redo/` → `store/redo/`, `recovery.rs` → `mod.rs`). Gleiche Meldungen, neue Pfade.
- **Eine echte Neumeldung netto:** `disallowed type std::collections::HashMap` in `catalog/state.rs` (8 auf `main`, 9 im Prüfstand). Verstoß gegen die Regel „keine neuen Meldungen“. Quelle: P28 aus `clippy.toml`.
- `contextra-durable-fs` wurde im Clippy-Lauf nicht separat ausgewertet, weil der Lauf am Storage-Crate abbrach.

## 6. SA-Vorschlag zum Workspace-Build (zur Entscheidung durch den Maintainer)

Ich entscheide nicht, ob G-1 mit diesem Stand gilt.

- **Option A: Bedingung einschränken.** Die Bedingung „Workspace-Build grün“ gilt für `contextra-ports`, `contextra-storage` und `contextra-durable-fs`. Folge: Das Gate ist enger gefasst. Die Blockade von 9 Konsumenten bleibt bis zu ihrer Umstellung bestehen, und das passt zu §22 („Vor G-5 ändert sich kein Konsument“).
- **Option B: Behebung als eigenes Paket.** E0308 in `contextra-vector` und `contextra-store` wird separat behoben. Folge: Zusätzlicher Arbeitsschritt außerhalb von G-1, danach ist der Workspace vollständig grün. Die Fehler betreffen Crates, die Konsumenten sind, deshalb ist der Eingriff mit dem Verbot „Kein Konsument ändert sich“ (§26.4) abzustimmen.

## 7. Was für das Gate noch fehlt

1. Entscheidung zu Punkt 15 (Option A oder B, Abschnitt 6).
2. Abschluss von WP-1.2/1.3: `KeyDomain` → `ClusterId`, `SecretAlloc`, 24-Byte-Nonce (UE-10 / WP-2.4), Entfall der Doppel-`Clock` (laut §27 nach Umstellung der Konsumenten).
3. Abschluss von WP-1.4: `reflink`/`punch_hole` in `FaultFs` (und `MemFs`), Torn-Modus in `fs_suite` und über `FaultFs`.
4. P-6: `persistence.rs` auf ≤ 400 Zeilen aufteilen.
5. Clippy-Neumeldung `HashMap` in `catalog/state.rs` beseitigen oder mit Ausnahme versehen.
6. Entscheidung, ob P2 in diesem Gate enthalten ist (Annahme in Abschnitt 1).

## 8. Nicht geprüft

- `contextra-durable-fs`: Clippy nicht separat ausgewertet.
- Vollständige Testsuite von `contextra-storage` (inklusive `loom`, `fs_real_test`, Absturzmatrix).
- `cargo-mutants`, Allokationsbudget, Messungen (SPEC §21: Leistungsaussagen sind Hypothesen).
- Keine CI-Aussage. Kein Push, keine Freigabe. Die Freigabe erteilt der Maintainer.
- Kein Befehl gegen `rustup`-stable 1.97.0.

## 9. Abgeleitete Hinweise zur Spec (keine Änderung hier)

- IMPL §2.2.2 (Stand vor der PR-Kette) sagt, `contextra-ports` sei transitiv verletzt. Mit #6061 ist das für den Normalbaum behoben (Tabellenzeilen 2 und 3); die IMPL-Stelle ist insoweit veraltet.
- SA-1 steht in SPEC §27 als „umgesetzt (PR #6061)“. Der Nachweis (Punkt 2 und 3) bestätigt das.
- Die Zeile in SPEC §27 für SA-5 (`MemFs` und `FaultFs` unter Feature `testing`) bleibt gültig. Die Lücke bei `reflink`/`punch_hole` berührt SA-5 nicht.
- Vorschlag für eine §27-Zeile (Sammel-Session): `| SA-16 | §22 G-1: Gate-Punkt „Workspace-Build grün“ betrifft die Crates des Kerns; Konsumenten-Fehler (E0308 in contextra-store/-vector) siehe Nachweis. | offen (Maintainer) |`
