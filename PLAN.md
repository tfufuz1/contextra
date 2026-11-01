# Contextra v1 – Umsetzungsplan (PLAN)

| Feld | Wert |
|---|---|
| Begleitdokument zu | `SPEC.md` (Version 1.0.0-spec.1, Stand 2026-11-01) |
| Basis | `github.com/tfufuz1/contextra`, `main` @ `76ea331c6` |
| Prüfmodus | statisch und online. Kein Build und kein Test durch diesen Plan. Alle Messungen sind Arbeitspakete (WP-0.1). |
| Verbindlichkeit | Reihenfolge, Gates und Abnahmen sind verbindlich. Aufwand ist Schätzung [E]. |

---

## 1. Grundsätze der Umsetzung

1. **Gate-getrieben.** Kein Phasenwechsel ohne bestandenes Gate (Abschnitt 4).
2. **Ein PR = ein WP = höchstens ein Crate-Schnitt** (AGENTS §2 Nr. 12).
3. **Rot vor Grün.** Jeder WP beginnt mit einem fehlschlagenden Test, der den Befund belegt. Ohne roten Test kein Fix.
4. **Erst entfernen, dann bauen.** Phase 1 reduziert den Workspace, bevor neuer Code entsteht. Sonst wächst der Umfang, den die Gates prüfen müssen.
5. **Evidenz statt Behauptung.** Jeder WP legt Logs und Zahlen unter `evidence/<WP-ID>/` ab (Befehl, Commit, Ergebnis). Ein Status ohne Evidenz gilt als offen.
6. **Keine Löschaussage außerhalb der Zulässigen** (SPEC 13.2), bis G-LEGAL bestanden ist. Das gilt auch für README, Changelog und Website.
7. **Zeitboxen** für Unbekanntes (WP-2.4, WP-4.1, WP-4.2). Überschreitung löst eine Entscheidung nach Gate G0 aus (SPEC 3.2), kein stilles Weiterziehen.

---

## 2. Ausgangslage

### 2.1 Stand des Clones

| Punkt | Stand | Konsequenz |
|---|---|---|
| HEAD | `76ea331c6` („Shell-Commit“) | Quelldokumente beziehen sich auf ältere Stände; Befunde sind je Stand zu prüfen (SPEC 0.3). |
| Rückbau | nicht umgesetzt: 37 Workspace-Mitglieder, 27 Default | Phase 1 ist Voraussetzung für fast alles. |
| Build (lib) | erfolgreich [U] | Basis vorhanden. |
| Build (all-targets) | 26 Fehler [U] | WP-0.1 kategorisiert, WP-1.x beseitigt die meisten durch Entfernung. |
| Clippy | hunderte Meldungen pro Crate [U] | Teils Test-Code (fixbar per `clippy.toml`), teils Produktionscode (P6). |
| Tests | nicht vollständig gelaufen; Testzahlen aus Audit [D] | Neu messen (WP-0.1). |
| CI | 13 Jobs mit `continue-on-error` [V] | Undeklarierte Features bleiben unentdeckt (WP-0.2). |

### 2.2 Kritische Befunde (Reihenfolge nach Wirkung)

| Rang | Befund | SPEC-Ref. | Behebung | Gate |
|---|---|---|---|---|
| 1 | Snapshot-Sentinel D-C02: Commits scheitern nach erstem Flush ohne Snapshot | 4.2 | WP-2.1 | G-C |
| 2 | WAL-Integrität abgeschaltet (`allow(unexpected_cfgs)`, nicht deklarierte Feature-Gates) | 4.2 | WP-0.2, WP-2.3 | G-A, G-C |
| 3 | Verschlüsselung Default aus, HKDF-Passphrase, Argon2-Untergrenze 19 MiB | 4.2, D-05, D-06 | WP-2.5 | G-C |
| 4 | Löschpfad ignoriert Ziel, verschluckt Fehler, Canary im produktiven WAL | 4.2 | WP-0.3 (Freeze), WP-1.3 (Entfernen), WP-3.x (Neu) | G-B, G-D |
| 5 | Lock-Bug beim Delete (Race) | 4.2 | WP-3.4 | G-D |
| 6 | Follower-Commit nicht cancel-safe | 4.2 | WP-2.7 | G-C |
| 7 | Zwei `Vfs`-Traits; Store ohne `Vfs` | 4.2, D-04 | WP-2.6 | G-C |
| 8 | u8-VNNI-Kernel: falsche Distanzen (Muster belegt) | 4.2, D-28 | WP-1.4 | G-B |
| 9 | `AutoExtraction` und `TenantPolicy` Default falsch | 4.2, D-07, D-27 | WP-2.5, WP-1.x | G-C |

### 2.3 Abhängigkeitsfolge (kritischer Pfad)

```
WP-0.1 ─▶ WP-0.2 ─▶ WP-1.1 ─▶ WP-1.2 ─▶ WP-1.3 ─▶ G-B
                                          │
WP-0.3 (Freeze, sofort) ◀─────────────────┘ (parallel möglich)
G-B ─▶ WP-2.1 ─▶ WP-2.2 ─▶ WP-2.3 ─▶ WP-2.4(Zeitbox) ─▶ WP-2.10 ─▶ G-C ─▶ G0-Teil-1
G-C ─▶ WP-3.1 ─▶ WP-3.2 ─▶ WP-3.3 ─▶ WP-3.4 ─▶ WP-3.5 ─▶ WP-3.6 ─▶ WP-3.7 ─▶ WP-3.8 ─▶ WP-3.9 ─▶ WP-3.10 ─▶ WP-3.11 ─▶ WP-3.12 ─▶ G-D
G-D ─▶ WP-4.1, WP-4.2, WP-4.3 (parallel) ─▶ G-E
G-D ─▶ WP-5.1 (Lab, startet ab WP-2.6) ─▶ WP-5.2 ─▶ WP-5.3 ─▶ WP-5.4 ─▶ G-F ─▶ G0-Entscheidung
G-E + G-F ─▶ WP-6.x ─▶ G-G ─▶ WP-7.x ─▶ G-H
```

**Parallelisierbar:** WP-0.3, WP-0.4, WP-0.5, WP-5.5, WP-6.5 sind unabhängig und können jederzeit laufen. WP-2.4 läuft parallel zu WP-2.5 bis WP-2.9. WP-5.1 beginnt nach WP-2.6.

---

## 3. Arbeitspakete

Jeder WP nennt: Ziel, Änderungen, Abnahme (prüfbar), Abhängigkeit, Aufwand (E), Risiko-Verweis.

### Phase 0 – Basislinie und Guardrails (1–2 Wochen)

**WP-0.1 Basislinie messen**
- Ziel: Ist-Zustand als Zahlen, nicht als Behauptung.
- Änderungen: keine Code-Änderung. Ablage `evidence/WP-0.1/`.
- Schritte:
  1. `rustup toolchain install 1.89.0 --component rustfmt,clippy`
  2. `cargo metadata --format-version 1 --locked > /dev/null`
  3. `cargo check --workspace --all-targets --locked 2>&1 | tee check.log`, Fehler nach Kategorie zählen (Feature, Import, Typ, Test-Code).
  4. `cargo test --workspace --locked --no-fail-fast 2>&1 | tee test.log` (ohne Crash-Tests parallel; Crash-Tests sequenziell).
  5. `cargo clippy --workspace --all-targets --locked -- -D warnings` Meldungen je Crate und Lint.
  6. `tokei` über `crates/` und `benchmarks/`, Baseline in `evidence/WP-0.1/loc.txt`.
  7. Policy-Zählung (Panics, `let _`, `allow`, `std::fs`, `HashMap` im Produktionscode) mit Trennung von `#[cfg(test)]`.
- Abnahme: Alle Zahlen des Audit-Logs [U] und der Quelldokumente sind neu gemessen oder als „nicht gemessen“ markiert. `evidence/WP-0.1/README.md` listet Befehl, Commit und Ergebnis.
- Aufwand: S (2–3 Tage).

**WP-0.2 Undeklarierte Features blockierend erkennen**
- Ziel: Befund „WAL-Integrität abgeschaltet“ darf nicht wieder unbemerkt entstehen.
- Änderungen:
  - Jobs `feature-matrix`, `cargo-hack-feature-check` aus `continue-on-error` nehmen.
  - Neuer Job `undeclared-features`: `cargo check` mit `RUSTFLAGS="--cap-lints=allow"` ist **nicht** erlaubt. Stattdessen Lint `unexpected_cfgs = "deny"` ohne Crate-`allow`, zusätzlich Skript, das jedes `feature = "…"` in `src/` gegen `[features]` aller `Cargo.toml` abgleicht.
  - Jobs `supply-chain`, `coverage`, `test-cross-platform`: Entscheidung je Job (blockieren oder bewusst nicht blockierend mit Begründung in `governance/ci-policy.md`).
- Abnahme: Testdatei mit `#[cfg(feature = "unbekannt")]` lässt CI rot werden. Entfernen des Tests lässt grün werden. Evidenz in `evidence/WP-0.2/`.
- Abhängigkeit: WP-0.1.
- Aufwand: S.

**WP-0.3 Erasure-Freeze (Sofortmaßnahme)**
- Ziel: Der fehlerhafte Löschpfad löscht nicht mehr falsch, bis er ersetzt ist (SPEC 8.4 Sofortmaßnahmen).
- Änderungen:
  - MCP-Werkzeuge `contextra_delete`, `drop_collection` (`crates/contextra-mcp/src/tools_crud.rs:210–232`): Antwort `Unsupported`, Tool-Schema entsprechend.
  - `Contextra::erase_*` und alle Aufrufe von `LsmPurger::purge` in der Engine: `Unsupported` (kein stiller Lauf).
  - Dokumentation des logischen Löschens als „nur logisch, ohne Wirkung auf Indexe, Caches, Backups“ (Kennzeichnung im Rustdoc).
- Abnahme: Test ruft jede Löschoberfläche auf und erwartet `Unsupported` oder logische Löschung mit Kennzeichnung. Keine Löschaussage in README (Lint, WP-0.4).
- Abhängigkeit: keine. Sofort.
- Aufwand: S.

**WP-0.4 Doku-Freeze und Wortlaut-Lint**
- Ziel: Keine Löschaussagen, die der Code nicht hält (SPEC 13.2).
- Änderungen: Lint-Skript `tools/wording_lint.sh` (Verbotsliste aus SPEC 13.2) über Quelle, `README.md`, `SECURITY.md`, `CONTRIBUTING.md`, `docs/`. Bereinigung der Treffer auf „in Entwicklung“.
- Abnahme: Lint grün. Treffer-Liste vorher/nachher in Evidenz.
- Abhängigkeit: keine.
- Aufwand: S.

**WP-0.5 ADR-001 und Entscheidungsregister**
- Ziel: Normative Quelle konsistent (SPEC 3.3).
- Änderungen: `docs/decisions/` anlegen. `ADR-001-erasure-v1.md` mit Entscheidungen D-01 bis D-30, Verweis auf SPEC. `AGENTS.md`-Änderungen gemäß SPEC 3.3 als eigener Commit.
- Abnahme: `docs/decisions/` existiert, ADR-Verweise in Code und Doku zeigen auf existierende Dateien (Lint: keine toten ADR-Verweise, Befund I-76 [K]).
- Abhängigkeit: keine.
- Aufwand: S.

**Gate G-A** (Ende Phase 0): WP-0.1 bis WP-0.5 abgeschlossen, Evidenz vollständig, Blocking-CI aktiv.

---

### Phase 1 – Rückbau auf den Kern (3–4 Wochen)

**WP-1.1 Konsumenten entkoppeln**
- Ziel: Entfernbare Crates haben keinen Verwender mehr im Kern.
- Änderungen:
  - `engine` ↛ `graph`, `checkpoint`, `privacy`, `cognition`, `license`. Relations-API (`relate`, `__rel:`) entfernen (D-17).
  - `contextra-rank` in `engine` auf Fusion reduzieren (`fuse_*` bleibt, `IsotonicCalibrator` entfernt) (D-26).
  - `store` Dev-Abhängigkeit `contextra-graph` entfernen.
  - `contextra-db` prüfen: Inhalt in Engine oder Fassade überführen (Default), oder als Kern begründen.
  - Fassade `contextra` ohne `privacy`, `license`, `checkpoint`.
- Abnahme: `cargo tree -p contextra -e normal --prefix none | sort -u` enthält keines der Crates aus SPEC 5.1 als Entfernungskandidat. `cargo check -p contextra --all-targets --locked` grün (Kernpfad).
- Abhängigkeit: WP-0.1, WP-0.3.
- Aufwand: M (1–2 Wochen). Risiko R-09.

**WP-1.2 Workspace reduzieren**
- Ziel: Workspace enthält nur Kern-Crates (SPEC 5.1).
- Änderungen:
  - Branch `archive/v0-full` anlegen, dort Stand sichern.
  - Entfernen aus `members` und `default-members`: `graph`, `checkpoint`, `privacy`, `ingest`, `cognition`, `adapt`, `agent`, `router`, `kvcache`, `infer-candle`, `infer-ollama`, `infer-onnx`, `sandbox`, `audit-export`, `avv-generator`, `license`, `mcp`, `py`, `wire`, `core`, `benchmarks/contextra-bench`.
  - Crate-Verzeichnisse aus `main` entfernen (Historie bleibt).
  - `Cargo.lock` mit `--locked` neu bestätigen (Lockfile-Änderung als eigener Commit).
- Abnahme: Workspace-Mitglieder = Kernliste (13 Produktions-Crates, `testkit`, `tests/arch`). `cargo metadata --locked` erfolgreich. Befund I-11 (Mitglieder) geschlossen.
- Abhängigkeit: WP-1.1.
- Aufwand: S–M.

**WP-1.3 Alter Löschpfad entfernen**
- Ziel: Keine Verwendung der fehlerhaften Löschtypen mehr (SPEC Anhang A).
- Änderungen: alle Dateien aus Anhang A der SPEC, die dem alten Löschpfad zugehören. Testdateien, die nur entfernte Typen nutzen, werden gelöscht. Isolations- und Mandantentests bleiben (Rückbau-Dokument 7 Nr. 4).
- Abnahme: `grep -rE "DeletionProof|KeyRegistry|kv_shredding|ErasureReport|LsmPurger|ScanTarget|LayerWitness" crates/` liefert nur Treffer in `archive/`-Verweisen oder Doku-Historie. `cargo check --workspace --all-targets --locked` ohne Fehler aus entfernten Typen.
- Abhängigkeit: WP-1.2.
- Aufwand: M.

**WP-1.4 SIMD-Reduktion**
- Ziel: D-28 umsetzen.
- Änderungen: u8-VNNI-Pfad entfernen (`simd/src/kernels/avx512.rs` u8-Teil, Dispatch). Skalarpfad als Referenz. Test T-SIMD-1 (Cross-Check je CPU-Dispatch-Pfad) für f32-Kernel.
- Abnahme: T-SIMD-1 grün für alle verbleibenden Pfade. Kein `_mm512_dpbusd_epi32` im Code.
- Abhängigkeit: WP-1.2.
- Aufwand: S.

**WP-1.5 Tests bereinigen**
- Ziel: Keine Tests auf entfernten Typen, keine Tests mit falschen Erwartungen aus Quelldokumenten.
- Änderungen: Tests mit Erwartungen gemäß SPEC klären:
  - `retention::…unpinned_floor_max` (Erwartung korrigieren, Logik entspricht Spec laut Audit [D]).
  - `commit_tests::…monotonic_across_concurrent_commits` und `scan_tests::…get_at_seq…` (Nummernsemantik, Abschnitt WP-2.2).
  - `compaction::tests::basic::…` (Testdaten mit gleicher Sequenznummer [D]).
  - `kv_crypto_shred_instant_unrecoverable.rs:32` (Test entfernen mit Entfernung von `kv_*`).
  - `io_tests::rotate_and_seal_readonly_guarantee`: als Nicht-root ausführen oder Rechteprüfung mit `geteuid` überspringen als **explizite** Skip-Begründung, nicht stilles `ignore`.
- Abnahme: `cargo test --workspace --locked` ohne Fehler außer dokumentiert-skippierten Tests (Liste in `governance/ignored-tests.md`, Ziel < 5).
- Abhängigkeit: WP-1.3.
- Aufwand: M.

**WP-1.6 Governance-Dateien**
- Änderungen: `capabilities.toml` an Ringe (SPEC 5.2), `governance/*` (Ratchet-Zähler), `clippy.toml` (`allow-*-in-tests`), `deny.toml` (Ignorierungen mit Ablaufdatum, Abschnitt WP-6.5).
- Abnahme: Architekturtest T-ARCH-3 grün.
- Aufwand: S.

**Gate G-B** (Ende Phase 1): `cargo check --workspace --all-targets --locked` grün. Workspace = Kern. Architekturtests grün. Alter Löschpfad entfernt. Archiv-Branch vorhanden.

---

### Phase 2 – Store-Härtung (6–9 Wochen)

**WP-2.1 Snapshot-Sentinel (D-C02)**
- Ziel: Commits nach Flush ohne Snapshot gelingen.
- Änderungen:
  - `compaction.rs:225–231`: `prune_bound` auf höchste committete Sequenz begrenzen. `u64::MAX` wird nicht als Sequenz verwendet (explizite Behandlung).
  - `ssi.rs:646–652`: `prune_through` erzwingt `bound_seq & TOMBSTONE_BIT == 0` durch Rückgabe eines Fehlers. Kein stilles Maskieren.
- Abnahme: Regressionstest „Flush ohne Snapshot, danach Commit mit frischem Snapshot gelingt“ ist **zuerst rot**, dann grün. Zusätzlich: Test „`prune_through` mit gesetztem Tombstone-Bit liefert Fehler“.
- Abhängigkeit: G-B.
- Aufwand: S (1–2 Tage).

**WP-2.2 Sequenzsemantik festschreiben**
- Ziel: Eindeutige Regel, Tests passen zur Regel (Befund F2 [D]).
- Änderungen:
  - Regel: Sichtbarkeit einer Transaktion ist die Sequenz ihres `TxEnd`. Einzeloperationen erhalten interne Sequenzen. Festschreiben in `docs/decisions/ADR-002-sequence-semantics.md`.
  - Tests #2 und #7 (`commit_tests::…monotonic…`, `scan_tests::…get_at_seq_mvcc…`) an die Regel anpassen.
  - Property-Test T-SEQ-1 gegen Referenzmodell.
- Abnahme: T-SEQ-1 grün. ADR-002 im Repo.
- Abhängigkeit: WP-2.1.
- Aufwand: M.

**WP-2.3 WAL-Integrität wiederherstellen und Replay vereinheitlichen**
- Ziel: R-SEC-6. Integrität immer aktiv, keine Legacy-Formate.
- Änderungen:
  - `#![allow(unexpected_cfgs)]` aus `store/src/lib.rs:45` entfernen.
  - Alle 7 `#[cfg(feature = "wal-integrity")]`-Gates in `wal/replay.rs` entfernen; der `not`-Zweig (Zeile 682) ebenfalls.
  - `encryption-at-rest` (11 Test-Dateien): Gate entfernen, Tests laufen unbedingt.
  - `WalVersion::V1` in `wal/io.rs` (Zeilen 43, 267, 284, 294, 415): Fehler `Integrity::LegacyFormat`.
  - Doppelte Replay-Logik (`wal/io.rs`, `wal/replay.rs`) auf eine Implementierung reduzieren (SPEC 11.5). Paritätstest T-WAL-2 bis zur Entfernung der zweiten Logik.
  - `legacy-wal-key` Feature und `tests/wal_legacy_*` entfernen (F-LEG, Abschnitt 15).
- Abnahme: Keine `cfg(feature = "wal-integrity")`, `encryption-at-rest`, `legacy-wal-key` im Code. T-WAL-1 (Abschneiden an jedem Byte, plain und verschlüsselt) grün. Der Test wird zuerst gegen den Ist-Stand rot.
- Abhängigkeit: WP-2.1.
- Aufwand: L (3–4 Wochen). Risiko R-01.

**WP-2.4 CRC-Fehler bei Offset 4 (verschlüsselt) lokalisieren** — Zeitbox 2 Wochen
- Ziel: Ursache des Replay-Fehlers im verschlüsselten Fall (Befund D-C03/D-C04 [K], F1 [D]).
- Vorgehen:
  1. Reproduktion als Test (rot), minimal: ein Batch, verschlüsselt, Neustart.
  2. Hypothesen in `evidence/WP-2.4/hypothesen.md`: Schlüsselableitung Schreiber vs. Leser (Salt, Nonce-Präfix), Frame-Layout (Offset 4), CRC über Klartext statt Chiffrat, Reihenfolge HMAC/AEAD.
  3. Jede Hypothese mit Test bestätigen oder verwerfen.
- Abnahme: Ursache dokumentiert mit Zeile, Test grün. **Oder** Zeitbox erreicht → Eintrag in Risikoregister, Entscheidung nach G0 (SPEC 3.2).
- Abhängigkeit: WP-2.3 (parallel möglich, sobald Gates entfernt sind).
- Aufwand: M–L, begrenzt auf 2 Wochen.

**WP-2.5 Sicherheitsdefaults (D-05, D-06, D-07)**
- Ziel: Fail-closed Standard.
- Änderungen:
  - `crypto/src/crypto.rs:88` `try_new` (HKDF-Passphrase) entfernen. Passphrase nur über `try_new_with_kdf` mit Argon2id und Header.
  - `crypto/src/kdf.rs:36` `MIN_M_COST_KIB = 65536`, `MIN_T_COST = 3`. Unterschreitung = `Config`-Fehler.
  - `contextra/src/builder.rs:147` `encryption_mode` entfällt als Option. Verschlüsselung ist Pflicht (`passphrase` oder `KeyProvider`).
  - `store/src/lsm/config.rs:89` `encryption_passphrase: Option<String>` → `Option<SecretBytes>`, kein `Serialize`, kein `Debug`-Klartext.
  - `TenantPolicy`: Varianten auf `Required` reduzieren (`engine/src/lib.rs:207–209`). Keine `Optional`-Variante.
  - `AutoExtractionMode` entfernen (D-27), Aufrufer-Schnittstelle prüfen.
  - `Zeroizing` für Passphrase-Puffer im Builder (B-S03).
- Abnahme: T-DEF-1, T-KDF-1, T-SEC-2 grün (zuerst rot). Der Standard-Builder ohne Passphrase schlägt fehl.
- Abhängigkeit: WP-2.1.
- Aufwand: M.

**WP-2.6 Vfs konsolidieren und Store migrieren (D-04)**
- Ziel: Ein `Vfs`, durchgehend genutzt. Voraussetzung für FaultVfs und Crash-Tests.
- Änderungen:
  - `ports/src/vfs.rs` erweitern: `rename`, `sync_file`, `sync_dir`, `open_append`, `read_at`, `remove_file`, `list_dir`, `len`, `exists`, `write_atomic`.
  - `durable-fs/src/vfs.rs` implementiert den Port. Zweiten Trait entfernen oder als Re-Export.
  - Store: 83 Treffer `tokio::fs::`/`std::fs::` (18 Dateien, grep, Obergrenze) auf `Vfs` migrieren. Reihenfolge: WAL, Manifest, SSTable, Blockcache-Datei.
  - Engine und Vector auf `Vfs` umstellen (gemessene Trefferzahlen in WP-0.1).
  - Lint T-LINT-2: `std::fs` und `tokio::fs` in `store`, `engine`, `vector` (Produktion) verboten.
- Abnahme: Lint grün. Alle bestehenden Store-Tests grün mit `RealVfs`. Ein FaultVfs-Smoke-Test (ohne Vollumfang) läuft.
- Abhängigkeit: G-B. Großer Umbau, in Teil-PRs je Modul.
- Aufwand: L (3–5 Wochen). Risiko R-02.

**WP-2.7 Follower-Commit cancel-safe (D-C01)**
- Ziel: Abbruch des Aufrufers verändert den Commit nicht halb.
- Änderungen: `store/src/lsm/ops/write.rs`: Commit läuft in owned Task (`tokio::spawn`), Ergebnis über `oneshot`. Der zweite `rx.await` ohne Timeout (ca. Zeile 564) entfällt. Cancel-Guard für Follower und Leader vereinheitlicht.
- Abnahme: T-CANCEL-1 (Abbruch an jedem `await`) grün, zuerst rot.
- Abhängigkeit: WP-2.1.
- Aufwand: M.

**WP-2.8 Lebenszyklus und Drop (D-C08)**
- Ziel: Kein Blockieren, keine verwaisten Locks.
- Änderungen: `close(self)` in Engine und Store. `Drop for LsmStorage` (`store/src/lsm/engine.rs:75–79`): nur Flags und Cancel, keine Blockierung, Locks freigeben.
- Abnahme: T-DROP-1 grün (zuerst rot: Neuöffnen im selben Prozess schlägt heute fehl [K]).
- Abhängigkeit: WP-2.6.
- Aufwand: S–M.

**WP-2.9 Panic-Freiheit und Determinismus im Store**
- Ziel: P6, P28.
- Änderungen:
  - `scanner/sstable.rs:35,47` `expect` entfällt mit Entfernung des alten Scanners (WP-1.3). Verbleibende `expect`/`unwrap` im Store-Produktionscode (Zählung WP-0.1) auf null bringen.
  - `manifest/core.rs:284,338–342`: `HashMap`/`HashSet` → `BTreeMap`/`BTreeSet`. AGENTS §2 Nr. 3 nennt `AHashMap` als Umstellung. `BTreeMap` gewählt, weil die Iterationsreihenfolge damit ohne Seed-Kontrolle garantiert ist (Ergebnisse waren bisher nur zufällig sortiert [U-Quelle Audit 2]).
  - `let _ =` im Store-Produktionscode entfernen (Fehler propagieren oder persistieren).
- Abnahme: Store-Zähler für `expect`, `unwrap`, `let _ =`, `HashMap` im Produktionscode = 0 (Ratchet-Gate in CI).
- Abhängigkeit: WP-2.6.
- Aufwand: M.

**WP-2.10 Crash-Matrix (Store-Ebene)**
- Ziel: Nachweis der Haltbarkeit an jedem fsync-Punkt (Gate G1 nach Entscheidungsgrundlage).
- Änderungen: Test-Harness mit FaultVfs, Kill nach Operation *n* für WAL, Flush, Compaction, Manifest. Ergebnisprotokoll je Punkt.
- Abnahme: 0 verlorene bestätigte Transaktionen, 0 wiederaufgetauchte abgebrochene Transaktionen. Protokoll in `evidence/WP-2.10/`.
- Abhängigkeit: WP-2.3, WP-2.6, WP-2.7.
- Aufwand: L.

**Gate G-C** (Ende Phase 2): WP-2.1 bis WP-2.10 abgeschlossen. Crash-Matrix grün. Clippy `-D warnings` für `store`, `engine`, `crypto`, `mvcc`. Ratchet-Zähler im Store = 0. **Entscheidung G0 Teil 1** (SPEC 3.2: Korrektheit und Crash-Haltbarkeit des LSM).

---

### Phase 3 – Löschkette (6–8 Wochen)

**WP-3.1 Typen (`types/src/erasure`)**
- Ziel: Zustände, Ziele, Klassen, Layer-IDs, Report-Struktur als reine Typen (T-ARCH-1).
- Änderungen: `EraseTarget`, `RetentionAttestation`, `RetentionCode`, `ErasureState`, `ErasureStatus`, `Class`, `LayerId` (exhaustiv), `RefusalReason`, `ResidualRisk` (SPEC 9.4), `ReportV1`-Struktur. `step`-Funktion `ErasureState::step` rein, total, ohne I/O.
- Abnahme: Property-Test: alle Übergänge total, `classify` monoton (T-SAGA-1 aus Quelle, übernommen). Kein `async`, kein I/O in `types` (T-ARCH-1).
- Abhängigkeit: G-C.
- Aufwand: M.

**WP-3.2 Schlüsselcodec und `doc_id` (D-08, D-09, D-21)**
- Ziel: Blinde Schlüssel, 128-Bit-`doc_id`, Kollisionsablehnung.
- Änderungen: `KeyFamily`-Enum (exhaustiv), Codec für `family ‖ tenant_tag ‖ coll_tag ‖ payload`, `K_idx`-Ableitung über Labels. `DocId` 128 Bit, Feature `docid-128` **entfernt**, Typ fest 128 Bit (`types/src/types/domain/ids.rs`). `IdCollision`-Fehler.
- Abnahme: T-ISO-3 grün (erzwungene Kollision). Codec-Property-Test (Roundtrip, keine Mehrdeutigkeit). T-ISO-2 (Präfix-IDs) grün, da keine Präfixe mehr.
- Abhängigkeit: WP-3.1.
- Aufwand: M.

**WP-3.3 Manifest pro Dokument (SPEC 7.3)**
- Ziel: Vollständige Liste der Schlüssel jedes Dokuments, atomar mit dem Dokument.
- Änderungen: Insert, Update, Delete schreiben `Manifest` in derselben Transaktion. Update entfernt Schlüssel nicht mehr vorkommender Terme.
- Abnahme: T-COV-1 (RecordingStorage) zeigt für alle Schreibpfade: jeder geschriebene Schlüssel steht im Manifest. Fehlendes Manifest → `Refused(NoManifest)`.
- Abhängigkeit: WP-3.2.
- Aufwand: M.

**WP-3.4 Insert, Update, Delete und Lock (N7)**
- Ziel: Lock für die Dauer der Operation (behebt `crud/delete.rs` Lock-Freigabe vor der Operation).
- Änderungen: `Collection::delete` hält das Dokument-Lock über `delete_op` und `commit`. Alle Schreibpfade nutzen denselben Lock-Pfad.
- Abnahme: Parallel-Test (Put und Delete auf demselben Dokument) zeigt keine Race (Stress + loom auf dem Lock-Pfad). Zuerst rot.
- Abhängigkeit: WP-3.3.
- Aufwand: S–M.

**WP-3.5 Journal und ErasedSet (SPEC 8.6)**
- Ziel: Durable Zustände, Hash-Kette, verschlüsselt.
- Änderungen: `journal.log` (Append, Länge, CRC, Hash-Kette, `fsync`), `scope/{id}` (verschlüsselt), `erased.set` (atomar ersetzt), Epochen-Commitments (`K_commit`). Schlüsselverzeichnis mit Rechteprüfung.
- Abnahme: T-CRASH-2 (abgerissener Write an jedem Byte von Journal und `erased.set`) grün. Defekte Hash-Kette → Start verweigert (T-KEY-1 analog).
- Abhängigkeit: WP-3.1, WP-2.6.
- Aufwand: M–L.

**WP-3.6 ReadGate in allen Lesepfaden (D-12, SPEC 8.3)**
- Ziel: Gesperrte Ziele sind für jeden Lesepfad gesperrt, Zeitreisen eingeschlossen.
- Änderungen: `TenantScopedStorage` prüft `ErasedSet` in `get`, `scan`, `get_at_seq`, `scan_at_seq`, Suche (Text, Vektor). Roher Store ist `pub(crate)` (T-ARCH-2).
- Abnahme: T-VER-1 grün über alle Sequenzen, zuerst rot.
- Abhängigkeit: WP-3.5.
- Aufwand: S–M.

**WP-3.7 Barrier und Drain (SPEC 8.4)**
- Ziel: Exklusivität gegen Compaction, WAL-Entfernung mit Prüfung.
- Änderungen: `RwLock` Compaction (shared) / Purge (exklusiv, mit Frist). Drain: `rotate_and_seal`, Flush aller Memtables, Entfernen versiegelter WAL-Dateien, `exists`-Prüfung (N9: heute nur Debug-Log, `compaction.rs:197–210` [V]).
- Abnahme: T-CRASH-3 (ENOSPC/EIO bei Drain) → `Pending`. Fehlende Prüfung ist als Fehler sichtbar.
- Abhängigkeit: WP-2.6, WP-3.6.
- Aufwand: M.

**WP-3.8 Purge-Rewrite (SPEC 8.4)**
- Ziel: Streamender, atomarer Austausch exakter Schlüssel über alle Versionen.
- Änderungen:
  - `sstable/builder.rs`: `create_like` (Schlüsselkontext, Formatversion, Originalwerte `seq`, `tx_id`, `flags`). Entfernt Fehlbau mit `tx_id = 1` (N1 [V]).
  - `sstable/reader.rs`: Streaming-Iterator mit Seq und Flags.
  - `store/src/erase/rewrite.rs`: Kandidaten, Stream, Matcher (exakt), Abbruch bei Fehler.
  - `manifest`: atomarer Eintrag „ersetze alt durch neu“.
  - Blockcache-Eviction mit `zeroize` (WP-4.3).
- Abnahme: T-RAW-1 auf Store-Ebene (Rohscan ohne Schlüssel-Needles) und T-ENC-1 (neue SSTable lesbar, gleiche Verschlüsselung). Speicherbedarf konstant (Messung RSS).
- Abhängigkeit: WP-3.7, WP-2.6.
- Aufwand: L (3–5 Wochen). Risiko R-05.

**WP-3.9 Snapshot-Widerruf (D-11)**
- Ziel: Frist für Löschung auch bei offenen Snapshots.
- Änderungen: `SnapshotRegistry` widerruft Snapshots, die eine betroffene Sequenz pinnen. Fehler `SnapshotRevoked`. Journal-Eintrag je Widerruf.
- Abnahme: T-CONC-1 Teil „mit offenem Snapshot“ grün. Löschung wartet nicht auf Snapshot-Ende (Zeitmessung mit gehaltenem Snapshot).
- Abhängigkeit: WP-3.8.
- Aufwand: M. Risiko R-08.

**WP-3.10 Saga und `recover()` (SPEC 8.2, 8.7)**
- Ziel: Vollständiger Lebenszyklus mit Wiederaufnahme nach Absturz.
- Änderungen: `contextra-engine/src/erasure/{saga,recover}.rs`. Zustandsübergänge durch `ErasureState::step`. `Pending` mit `attempt` und `next_retry_at`. `erase_sla`-Überschreitung → Alarm (Metrik, Audit-Eintrag), kein Abbruch. Recovery vor Freigabe des Lesepfads.
- Abnahme: T-CRASH-1 (Absturz nach jeder Vfs-Operation der Saga) grün: nach `recover()` terminal oder `Pending`, nie Lesezugriff auf nicht gesperrtes Ziel. T-SLA-1 grün.
- Abhängigkeit: WP-3.5, WP-3.6, WP-3.8, WP-3.9.
- Aufwand: L.

**WP-3.11 Erase-API und Batch (SPEC 6, 8.8)**
- Ziel: Öffentliche Oberfläche wie in SPEC 6.
- Änderungen: `erase`, `ErasureHandle`, `erasure_status`, Batch-Fenster (Standard 60 s, `Timing::Now`), Dedup nach Commitment, `RetentionAttestation`-Pflicht, `Refused`-Pfad.
- Abnahme: T-API-1 (trybuild: ohne TenantContext kein Collection), T-API-2 grün. Latenz `Gated` p50 ≤ 10 ms auf Testkorpus (vorläufig, Endmessung WP-6.1).
- Abhängigkeit: WP-3.10.
- Aufwand: M.

**WP-3.12 Betroffenen-Löschung (D-14, SPEC 7.6)**
- Ziel: `EraseTarget::Subject` und `Collection`.
- Änderungen: `Subject`-Index schreiben (WP-3.3), Auflösung, Batch-Erase.
- Abnahme: Test: Betroffener mit 1.000 Dokumenten, Erase, Nachbarn (anderer Betroffener) bitgleich (T-ISO-1 Variante).
- Abhängigkeit: WP-3.11.
- Aufwand: S–M.

**Gate G-D** (Ende Phase 3): T-VER-1, T-CRASH-1..4, T-CANCEL-1, T-CONC-1, T-ISO-1..3, T-API-1..2, T-SLA-1 grün. Löschpfad funktional auf Store- und Engine-Ebene. Messung der Schichten noch nicht Gate (kommt mit Phase 5).

---

### Phase 4 – Indexe (4–6 Wochen)

**WP-4.1 Textindex auf exakte Schlüssel (D-20)**
- Ziel: Postings `Text(term_tag, doc_id)`, `Df(term_tag)`. Keine Per-Term-Listen.
- Änderungen: `contextra-text/src/inverted/`: Schreibpfad, Suche (Präfix-Scan auf `term_tag`, fixe Länge), BM25-Berechnung aus `Df`. Entfernen von `block_on` (C-L19 [K]). Entfernen alter `purge.rs`, `scanner.rs`.
- Abnahme: T-ISO-1 (Text-Layer), T-RAW-3 (Textschicht) grün. Suchqualität: Recall@10 gegen Basislinie auf Testkorpus (Abweichung ≤ 2 %, sonst Entscheidung). Lasttest Insert/Update mit Df-Zählern.
- Abhängigkeit: G-D.
- Aufwand: L (3–4 Wochen). Risiko R-03.

**WP-4.2 HNSW: Zeroing und Batch-Rebuild (D-18, D-19)**
- Ziel: Vektorinhalt nicht mehr lesbar, Kanten bleiben bis Rebuild.
- Änderungen:
  - Löschung setzt Slot-Bytes auf 0 (`zeroize`), Tombstone-Liste, `DocMap`-Eintrag entfernen.
  - Rebuild bei Tombstone-Quote ≥ 0,2 im Hintergrund. Arena-Reuse erst nach Rebuild.
  - `hnsw/deletion.rs` (Kantenreparatur) entfernt (VETO-F02).
  - Snapshot-Dateien verschlüsselt und über `Vfs` (D-19).
- Abnahme: T-RAW-3 (HeapScan, Vektormuster) grün. Keine Vektorbytes in Snapshot-Dateien nach Löschung. Rebuild-Kosten gemessen (Basislinie).
- Abhängigkeit: G-D.
- Aufwand: L. Risiko R-04.

**WP-4.3 Blockcache-Eviction**
- Ziel: Entschlüsselte Blöcke ersetzter Dateien verschwinden.
- Änderungen: `store/src/sstable/block_cache.rs`: Eviction nach Dateiersetzung, `zeroize` vor Freigabe.
- Abnahme: Heap-Scan (T-RAW-3) nach Erase ohne Befund für Blockcache-Schicht.
- Abhängigkeit: WP-3.8.
- Aufwand: S.

**WP-4.4 Speicherhygiene**
- Ziel: Needles und Schlüssel nicht unbeabsichtigt kopiert.
- Änderungen: `Zeroizing` für alle Puffer mit Nutzerdaten im Löschpfad. Keine `String`-Zwischenspeicher für Inhalte. `Clone` nur, wo nötig.
- Abnahme: Lint „kein `String` in `erase/`“ (T-LINT-2 erweitert). Heap-Scan-Test für Insert/Erase-Zyklus.
- Abhängigkeit: WP-4.1, WP-4.2.
- Aufwand: S–M.

**Gate G-E** (Ende Phase 4): Löschtests je Index grün. Suchqualität im Rahmen der Basislinie. Keine Kanten-Reparatur pro Löschung im Code.

---

### Phase 5 – Verifikation und Nachweis (4–5 Wochen)

**WP-5.1 Erasure Lab (`contextra-testkit/src/erasure_lab/`)**
- Ziel: Unabhängige Messung (SPEC 9.2).
- Änderungen: `ForensicScanner`, `HeapScanner` (Linux, `/proc/self/mem`), `CanaryFactory`, `ReferenceModel`, `RecordingStorage`, `NeighborDigest`, `LeakyPurger`, Evidenz-JSON je Lauf.
- Abnahme: T-RAW-2 (`LeakyPurger` wird gefunden, sauberes System besteht). Positivkontrolle in jedem Lauf (Lauf ungültig ohne Canary-Fund). Testkit ohne Abhängigkeit auf Löschimplementierung (Scanner liest nur Bytes).
- Abhängigkeit: WP-2.6 (Start), G-D (Abschluss).
- Aufwand: M–L.

**WP-5.2 Bericht v1 und Verifier (D-13)**
- Ziel: Signierter Bericht, extern prüfbar.
- Änderungen: `crypto/src/proof_v1.rs`: kanonisches JSON, Ed25519-Signatur, `classify` (rein). `examples/verify_erasure_report/`. Testvektoren in `tests/vectors/`.
- Abnahme: T-PROOF-1 (jedes Byte) grün. T-REPORT-1 (Restrisiken vorhanden, kein Klartext) grün. Verifier ohne Contextra-Abhängigkeit (`cargo tree` des Beispiels zeigt nur `ed25519` und JSON-Parser).
- Abhängigkeit: WP-3.10, WP-5.1.
- Aufwand: M.

**WP-5.3 `verify` (SPEC 9.5)**
- Ziel: Rescan aus Identität im neuen Prozess.
- Änderungen: `verify(ctx, target)` leitet Blind-Schlüssel ab, prüft `ErasedSet`, rescannt. Grenze im Bericht ausgewiesen.
- Abnahme: Test: nach Neustart und Erase liefert `verify` `PhysicalMeasured` für Schlüsselreste; manipulierter Rest (absichtlich eingefügter Schlüssel) wird gemeldet.
- Abhängigkeit: WP-5.2.
- Aufwand: S–M.

**WP-5.4 Löschcheckliste (SPEC 13.3)**
- Ziel: Textexport, keine Rechtsaussage.
- Änderungen: `erasure_checklist()` mit fixem Inhalt, Versionierung. Freigabe G-LEGAL ausstehend (Kennzeichnung im Text).
- Abnahme: Wortlaut-Lint grün. Test: Inhalt stabil (Snapshot-Test).
- Abhängigkeit: WP-0.4.
- Aufwand: S.

**WP-5.5 Wortlaut-Lint und Berichtsprüfung**
- Ziel: Verbotene Begriffe nirgends (T-LINT-1).
- Änderungen: Lint aus WP-0.4 erweitert auf Berichtsvorlagen und Fehlermeldungen.
- Abnahme: Lint grün, in CI blockierend.
- Abhängigkeit: WP-0.4.
- Aufwand: S.

**Gate G-F** (Ende Phase 5): T-RAW-1..3, T-PROOF-1, T-REPORT-1, T-LINT-1/2, T-COV-1 grün. Erasure Lab validiert (LeakyPurger wird gefunden). **Entscheidung G0 Teil 2** (SPEC 3.2: Rohscan-Vergleich LSM vs. redb, Schreibverstärkung). Ergebnis als `docs/decisions/ADR-003-storage-g0.md`.

---

### Phase 6 – Härtung und Performance (3–4 Wochen)

**WP-6.1 Basislinie und Leistungsgates (SPEC 12.4)**
- Ziel: Messbare Gates auf dokumentierter Hardware.
- Änderungen: Hardware-Beschreibung in `bench/HARDWARE.md`. Benchmarks `lsm_concurrency_bench`, `wal_bench`, `sstable_bench`, `redb_comparative_bench` (vorhanden [V], nie gelaufen). Neue Erase-Benchmarks (10⁴, 10⁶, 10⁷ Dokumente; Batch 1.000).
- Abnahme: Basislinie als JSON in `bench/baseline/`. Gates aus 12.4 gemessen. Abweichungen schriftlich begründet.
- Abhängigkeit: G-E, G-F.
- Aufwand: M–L (Laufzeit, nicht Entwicklung).

**WP-6.2 Fuzzing und Nebenläufigkeitstests**
- Ziel: Parser- und Nebenläufigkeitsfehler finden.
- Änderungen: `cargo-fuzz`-Ziele: Journal, SSTable-Index, WAL-Frame, Schlüsselcodec, Report-Parser. `loom`-Modelle: Barrier, ReadGate, Gate-Set, Journal.
- Abnahme: 72 h Fuzz-Dauerlauf ohne Absturz pro Ziel (Korpus im Repo). loom-Modelle grün mit dokumentiertem Schedule-Bound.
- Abhängigkeit: G-E.
- Aufwand: M (plus Laufzeit).

**WP-6.3 Mutationstests**
- Ziel: Testqualität der Löschkette.
- Änderungen: `mutants-guarantees.toml` für `step`, `classify`, Matcher, Barrier, ReadGate.
- Abnahme: ≥ 95 % der Mutanten getötet, Überlebende begründet.
- Abhängigkeit: G-F.
- Aufwand: S–M.

**WP-6.4 Unsafe-Audit**
- Ziel: Nur `simd` und `sys` mit `unsafe`, jeder Block dokumentiert.
- Änderungen: `// SAFETY:` für jeden Block (Alignment, Bounds, Gültigkeit). Zwei bekannte Stellen mit `safe`-Funktion und unsafe-Inhalt (`mmap_readonly`, `reopen_and_dup2` [U]) umstrukturieren: `unsafe fn` mit Vertrag oder sicherer Wrapper mit Prüfung.
- Abnahme: `undocumented_unsafe_blocks = deny` grün. `befund_safe_fn_mit_unsafe_inhalt` [U] = 0. Miri-Lauf für `simd` (nicht blockierend, Ergebnis dokumentiert).
- Abhängigkeit: WP-1.4.
- Aufwand: M.

**WP-6.5 Lieferkette (SPEC 10.6)**
- Ziel: Advisory-Ignorierungen begründet oder beseitigt.
- Änderungen: Für jede der sechs Ignorierungen [V] (`deny.toml:9–14`): Prüfung, ob die Abhängigkeit nach Entfernung (WP-1.2) noch im Kern ist. Verbleibende Ignorierungen mit Ablaufdatum und ADR. `Cargo.lock`-Updates als eigener Commit.
- Abnahme: `cargo deny check` und `cargo audit` grün, ohne Ignorierungen ohne Ablaufdatum.
- Abhängigkeit: WP-1.2.
- Aufwand: S–M.

**WP-6.6 Gate G0 Messung (SPEC 3.2)**
- Ziel: Zahlen für die Speicherentscheidung.
- Änderungen: Vergleichsharness LSM vs. redb mit identischer Dokumenthülle. Rohscan-Protokoll.
- Abnahme: Tabelle `residual_by_layer` und Schreibverstärkung für beide Varianten in `evidence/WP-6.6/`. Entscheidung wird in WP-5.x-Gate G-F dokumentiert, nicht hier neu verhandelt.
- Abhängigkeit: WP-5.1.
- Aufwand: M.

**Gate G-G** (Ende Phase 6): Leistungsgates erfüllt oder begründet abgewichen. Fuzz-Dauerlauf bestanden. Mutation ≥ 95 %. Unsafe-Audit abgeschlossen. Lieferkette ohne offene Ignorierung ohne Ablaufdatum.

---

### Phase 7 – Freigabe (parallel ab Gate G-D, extern)

**WP-7.1 Externes Review**
- Ziel: G-SEC (SPEC Quelle Final, Phase 5).
- Umfang: Saga, Sicherheitsbasis (SPEC 10), Berichtsformat und Verifier (SPEC 9). Kein Review des Gesamtsystems.
- Abnahme: schriftlicher Bericht; alle Befunde hoch und mittel behoben oder mit Restrisiko in SPEC aufgenommen.
- Beauftragung: sofort nach Gate G-D, damit Ergebnisse vor G-H vorliegen (Entscheidung E10 der Entscheidungsgrundlage).

**WP-7.2 G-LEGAL**
- Ziel: schriftliche Antworten auf SPEC 13.4.
- Umfang: Fachanwalt oder Datenschutzbeauftragte(r). Bereitstellung: SPEC, Bericht, Checkliste.
- Abnahme: schriftliche Antwort auf alle Fragen 13.4. Ohne Antwort keine Löschaussage über 13.2 hinaus.
- Beauftragung: sofort, Vorlauf 4–8 Wochen.

**WP-7.3 Dokumentation und Versionierung**
- Umfang: `README`, `SECURITY.md`, `CHANGELOG.md` (Breaking Changes: Verschlüsselung Pflicht, DocId 128 Bit, Entfernung Crates, neue API), Migrationsleitfaden, `docs/` konsistent mit SPEC (Doc-Drift-Lint).
- Abnahme: Lint „Doku ↔ SPEC“ grün. Keine toten Verweise.

**WP-7.4 Release-Candidate und v1.0.0**
- Umfang: Tag `v1.0.0-rc.1`, Freigabeprotokoll (alle Gates G-A bis G-H mit Evidenz-Links), Support-Policy, SemVer-Regeln.
- Abnahme: Gate G-H (SPEC-Freigabe, G-LEGAL, G-SEC, alle Tests, Doku).

**Gate G-H** (Freigabe): WP-7.1 bis WP-7.4 abgeschlossen. Erst dann darf die Produktaussage aus SPEC 13.2 verwendet werden.

---

## 4. Gates im Überblick

| Gate | Ende von | Kriterium (Kurz) | Entscheidung |
|---|---|---|---|
| G-A | Phase 0 | Baseline gemessen, Freeze aktiv, Blocking-CI | – |
| G-B | Phase 1 | Workspace = Kern, all-targets grün, alter Löschpfad entfernt | – |
| G-C | Phase 2 | Crash-Matrix, Ratchet Store = 0, Clippy sauber | G0 Teil 1 (LSM-Korrektheit) |
| G-D | Phase 3 | Löschkette funktional (T-VER, T-CRASH, T-CONC, T-ISO, T-API) | – |
| G-E | Phase 4 | Indexe löschen und suchen im Rahmen der Basislinie | – |
| G-F | Phase 5 | Lab validiert, Bericht signiert, Rohscan grün | G0 Teil 2 (Speicherwahl, ADR-003) |
| G-G | Phase 6 | Leistung, Fuzz, Mutation, Unsafe, Lieferkette | – |
| G-H | Phase 7 | Review, G-LEGAL, Doku, RC | Freigabe v1.0.0 |

**Ausstiegsregel G0** (SPEC 3.2): Verfehlt der LSM G-C (Crash-Matrix) zweimal nach vier Wochen Frist oder überschreitet WP-2.4 die Zeitbox, startet der redb-Spike. Die Entscheidung erfolgt schriftlich in ADR-003 mit Zahlen.

---

## 5. Reihenfolge und Parallelität

| Woche (ca.) | Hauptstrang | Parallelstrang |
|---|---|---|
| 1–2 | WP-0.1, WP-0.2 | WP-0.3, WP-0.4, WP-0.5 |
| 3–6 | WP-1.1 bis WP-1.6 | WP-6.5 (Abhängigkeiten nach Entfernung prüfen) |
| 7–9 | WP-2.1, WP-2.2, WP-2.3 | WP-2.4 (Zeitbox, parallel zu 2.3) |
| 10–13 | WP-2.5, WP-2.6, WP-2.7, WP-2.8 | WP-5.1 (Lab, ab WP-2.6), WP-5.5 |
| 14–16 | WP-2.9, WP-2.10 → **G-C** | WP-7.2 (G-LEGAL-Anfrage) |
| 17–24 | WP-3.1 bis WP-3.12 → **G-D** | WP-5.2 (Bericht) |
| 25–30 | WP-4.1, WP-4.2, WP-4.3, WP-4.4 → **G-E** | WP-7.1 (Review-Beauftragung nach G-D) |
| 31–35 | WP-5.3, WP-5.4, WP-6.6 → **G-F** (G0) | WP-6.2 Fuzz-Laufzeit |
| 36–40 | WP-6.1, WP-6.3, WP-6.4 → **G-G** | – |
| 41–44 | WP-7.3, WP-7.4 → **G-H** | Review-Umsetzung |

Die Wochen sind Kalenderwochen bei 1 FTE ohne Puffer. Siehe Abschnitt 6.

---

## 6. Aufwand

Schätzung [E], Einheit: Personenwochen bei vollem Fokus (1 FTE). Unsicherheit ist angegeben.

| Phase | Wochen | Unsicherheit | Haupttreiber |
|---|---|---|---|
| P0 Basislinie | 1–2 | gering | Messung |
| P1 Rückbau | 3–4 | mittel | Entkopplung Engine, Testbereinigung |
| P2 Store-Härtung | 9–12 | hoch | WP-2.3 (WAL), WP-2.4 (Ursache offen), WP-2.6 (Vfs, 83 Stellen) |
| P3 Löschkette | 8–10 | mittel | WP-3.8 (Rewrite), WP-3.10 (Saga) |
| P4 Indexe | 6–8 | hoch | WP-4.1 (Textindex-Umbau), WP-4.2 (HNSW-Rebuild) |
| P5 Nachweis | 5–6 | mittel | Erasure Lab |
| P6 Härtung | 4–6 | mittel | Laufzeiten (Fuzz, Benchmarks) |
| **Summe** | **36–48** | | |
| P7 Freigabe | parallel | extern | Review, G-LEGAL |

**Einordnung.** Die Quelldokumente nennen 4–7 Monate Teilzeit (Produktspezifikation) und 13–16 Wochen (Bericht 3). Beide unterschätzen nach dieser Analyse die Vfs-Migration (83 Stellen im Store, Vfs bisher ungenutzt), den Textindex-Umbau und die WAL-Ursache, die nicht lokalisiert ist. 36–48 Personenwochen bei vollem Fokus entsprechen bei 50 % Kapazität etwa 14–19 Monaten Kalenderzeit. Mit Assistenz durch Code-Erzeugung sinkt der Schreibaufwand, nicht der Prüfaufwand; die Gates bleiben der Engpass.

**Kürzungsoptionen** (jeweils mit Verlust an Garantie, nicht an Mission):
- Textindex als Erstversion ohne Postings-Umbau (Löschen über Full-Scan der Textschicht). Verlust: Leistung bei großen Korpora. Akzeptabel bis ca. 10⁵ Dokumente, dokumentieren.
- Vektor-Rebuild nur manuell statt Hintergrund. Verlust: Betriebskomfort.
- Python und MCP in v1.1 (bereits geplant).

---

## 7. Risikoregister

| ID | Risiko | W | Wirkung | Gegenmaßnahme | Frühwarnung | Owner |
|---|---|---|---|---|---|---|
| R-01 | Ursache des verschlüsselten WAL-Fehlers (WP-2.4) nicht lokalisierbar | M | hoch | Zeitbox 2 Wochen, dann G0 | Hypothesen nach 1 Woche ohne Bestätigung | Entwickler |
| R-02 | Vfs-Migration größer als geschätzt (83 Stellen plus Tests) | M | hoch | Modulweise PRs, `RealVfs` zuerst, Lint erst nach Migration | Anzahl offener Stellen je Woche | Entwickler |
| R-03 | Textindex mit Einzelschlüsseln langsamer als Basislinie | M | mittel | Lasttest vor Umbau-Abschluss, Kürzungsoption (6) | Recall/Latenz-Abweichung in WP-4.1 | Entwickler |
| R-04 | HNSW-Rebuild bei großen Mengen zu teuer | M | mittel | Rebuild-Schwelle konfigurierbar, Messung in WP-4.2 | Rebuild-Dauer über 10⁶ Vektoren | Entwickler |
| R-05 | Schreibverstärkung der Purge bei verstreuten Schlüsseln | H | mittel | Batch-Fenster (SPEC 8.8), Messung G-E, G-G | Bytes je gelöschtem Byte | Entwickler |
| R-06 | G-LEGAL verzögert sich; Löschaussagen bleiben gesperrt | H | hoch (Marktwirkung) | Vorlauf ab Phase 2, Produkt ohne Aussage bleibt nutzbar | Antwortfrist überschritten | Auftraggeber |
| R-07 | Solo-Kapazität reicht nicht für Gates und Review | H | hoch | Scope-Disziplin (5.1), Kürzungsoptionen, externe Review-Termine früh | Wochen ohne Gate-Fortschritt | Auftraggeber |
| R-08 | Snapshot-Widerruf erzeugt Ausfälle in Lesepfaden | M | mittel | Dokumentierter Fehler `SnapshotRevoked`, Metrik, Retry-Hinweis | Widerrufsrate im Betrieb | Entwickler |
| R-09 | Entfernung von Crates bricht unbekannte externe Nutzer | M | mittel | Archiv-Branch, Changelog, Migrationsleitfaden | Issues nach Release | Auftraggeber |
| R-10 | Keine Basislinie für Leistungsgates (Hardware) | M | mittel | Hardwaredokument in WP-6.1, Gates relativ | fehlende Wiederholbarkeit | Entwickler |
| R-11 | Unentdeckter Datenpfad (Schlüssel ohne Manifest) | M | hoch | RecordingStorage (T-COV-1), Lint, Review | T-COV-1 rot | Entwickler |
| R-12 | Schlüsselremanenz im Dateisystem (Restrisiko, nicht behebbar in v1) | H | mittel | im Bericht ausgewiesen, Checkliste 13.3 | – | – (akzeptiert) |
| R-13 | Aggregierte `Df`-Zähler werden als personenbezogen bewertet | N | mittel | G-LEGAL-Frage 5, Architektur erlaubt Umbau | Rechtsantwort | Auftraggeber |
| R-14 | Neue `allow`-Attribute im Produktionscode umgehen Lints | M | mittel | Allow-Ledger mit Ablauf (`governance/allow-ledger.toml`), CI-Zähler | Ratchet steigt | Entwickler |

---

## 8. Definition of Done

Für jeden WP gelten alle Punkte. Ein Punkt fehlt → WP offen.

1. **Roter Test zuerst** (Commit-Historie zeigt den roten Stand), dann grüner Test.
2. `cargo fmt --all -- --check` grün.
3. `cargo clippy --workspace --all-targets --locked -- -D warnings` grün für betroffene Crates.
4. Kein neues `unwrap`, `expect`, `panic!`, `let _ =`, `#[allow(...)]` im Produktionscode. Ratchet-Zähler nicht gestiegen.
5. Keine Verwendung von `std::fs`, `tokio::fs`, `Instant::now`, `SystemTime::now`, `thread_rng` außerhalb der Port-Implementierung.
6. Keine neue Feature-Flag auf Sicherheitspfad (P5).
7. Architekturtests grün (T-ARCH-1..3).
8. Wortlaut-Lint grün (T-LINT-1).
9. Evidenz unter `evidence/<WP-ID>/`: Befehl, Commit-Hash, Ausgabe, Zeitpunkt.
10. Doku und ADR aktualisiert, wenn sich eine Entscheidung ändert. Keine Abweichung von SPEC ohne ADR.
11. Ein PR = ein WP = höchstens ein Crate-Schnitt.

Für Gates zusätzlich: alle Tests des Gates in CI grün, nicht nur lokal. Nicht-blockierende Jobs liefern Ergebnisse, aber kein Gate hängt an ihnen.

---

## 9. v1.1-Backlog

| ID | Erweiterung | Voraussetzung | SPEC-Ref. |
|---|---|---|---|
| V1.1-1 | Einschränkung der Verarbeitung (Art. 18) als Zustand `Restricted` | v1 stabil | D-15, X-2 |
| V1.1-2 | MCP-Server als Adapter auf stabiler API | API-Freeze v1 | D-25 |
| V1.1-3 | Python-Bindings (PyO3) als Adapter | API-Freeze v1 | D-25 |
| V1.1-4 | Schlüsselvernichtung als zusätzliche Schicht (Klasse C) | G-LEGAL-Antwort 1 positiv, KeyAnchor-Profil | D-01, X-1 |
| V1.1-5 | Restore-Guard mit externem `ErasureLedger` | Betriebsfeedback | X-3 |
| V1.1-6 | Empfänger-Hook `on_erased` (Art. 19) | v1 Hook-Schnittstelle | X-5 |
| V1.1-7 | Graph mit Ebene-A-Adapter | Nachweis der Kantenbereinigung, Entscheidung D-17 | D-17 |
| V1.1-8 | Ingest mit Prozess-Sandbox | eigene Spezifikation, Angriffsfläche-Analyse | D-24 |
| V1.1-9 | TTL je Collection (Stretch aus Final) | Einschränkung (V1.1-1) | L-03 |
| V1.1-10 | Schlüsselrotation (Store-Schlüssel, Argon2-Parameter) | Migrationswerkzeug | 10.2 |
| V1.1-11 | Mandantenausgerichtete SSTable-Partitionen | Messung auf Last | X-6 |

---

## 10. Erste Schritte (diese Woche)

```bash
# 1. Arbeitszweig und Evidenzverzeichnis
cd contextra
git checkout -b wp-0/baseline
mkdir -p evidence/WP-0.1 evidence/WP-0.2

# 2. Toolchain fixieren
rustup toolchain install 1.89.0 --component rustfmt,clippy
rustup override set 1.89.0

# 3. Basislinie (WP-0.1)
cargo metadata --format-version 1 --locked > /dev/null && echo "lock ok" | tee evidence/WP-0.1/lock.txt
cargo check --workspace --all-targets --locked 2>&1 | tee evidence/WP-0.1/check_all_targets.log
cargo clippy --workspace --all-targets --locked -- -D warnings 2>&1 | tee evidence/WP-0.1/clippy.log
cargo fmt --all -- --check 2>&1 | tee evidence/WP-0.1/fmt.log

# 4. Undeklarierte Features sichtbar machen (WP-0.2, vorbereitend)
grep -rhoE 'feature = "[^"]+"' crates/*/src | sort | uniq -c | sort -rn > evidence/WP-0.2/features_used.txt

# 5. Freeze (WP-0.3): zuerst Test schreiben, der die Löschwerkzeuge aufruft
```

Danach in dieser Reihenfolge: WP-0.3 (Freeze), WP-0.4 (Wortlaut-Lint), WP-0.5 (ADR-001). Phase 1 beginnt erst, wenn `evidence/WP-0.1/` vollständig ist.

---

## 11. Offene Punkte dieses Plans

- Die Zahlen für Leistungsgates sind nicht festgelegt (WP-6.1). Vorher gilt nur die Relativregel.
- Der Umfang von `contextra-db` (WP-1.1) ist nicht gelesen. Die Entscheidung „in Fassade überführen“ ist Default, nicht Befund.
- Die Zeitbox von WP-2.4 ist eine Schätzung. Ob sie reicht, zeigt sich erst bei der Reproduktion.
- Ob bereits ausgelieferte Daten existieren (F-LEG), ist ungeklärt. Davon hängt WP-2.3 ab, falls ein Migrationspfad nötig ist.
- Die Gesamtschätzung hat keinen Puffer. Ein realistischer Puffer liegt bei +30 % für Unbekanntes.
