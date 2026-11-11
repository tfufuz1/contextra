# Implementierungsspezifikation: Löschkette in `contextra-erasure`

**Repository:** https://github.com/tfufuz1/contextra
**Basis (HEAD):** `16387928` (Merge PR #5996)
**Ziel-Crate:** `crates/contextra-erasure` (autark, `#![forbid(unsafe_code)]`, 0 interne Abhängigkeiten)
**Begleitdatei:** `loeschkette_zuordnung.csv` (4.546 Zeilen, jede Zeile mit Arbeitspaket, Ziel-Modul und GitHub-Link)

---

## 0. Zweck und Geltungsbereich

Diese Spezifikation beschreibt, wie die Löschkette (Löschsaga, Tresor und Schlüsselvernichtung, Read-Gate, Journal und Ledger, Messung und Zeugen, signierte Löschberichte, Restore-Schutz) vollständig im Crate `contextra-erasure` lebt. Sie legt fest:

1. den Ist-Zustand auf HEAD mit Commit-Referenzen,
2. die Lücken gegenüber dem historischen Funktionsbestand,
3. die Arbeitspakete WP-0 bis WP-12 mit Ziel-Modul, Akzeptanzkriterium und Abhängigkeit,
4. die Regeln, wie historischer Code übernommen werden darf.

Nicht Teil dieser Spezifikation: Orchestrierung, die an einen Async-Runtime-Stack oder an einzelne Adapter gekoppelt ist, sowie Neubauten außerhalb der Löschkette.

---

## 1. Ist-Zustand auf HEAD `16387928`

### 1.1 Crate-Profil (verifiziert)

| Merkmal | Befund | Quelle |
|---|---|---|
| Interne Abhängigkeiten | 0 | [`crates/contextra-erasure/Cargo.toml`](https://github.com/tfufuz1/contextra/blob/16387928/crates/contextra-erasure/Cargo.toml) |
| Features | `signing`, `vault`, `async`; Default: alle drei | Cargo.toml, Abschnitt `[features]` |
| Dateien (inkl. Tests) | 110 Dateien unter `crates/contextra-erasure` | `git ls-tree -r HEAD` |
| Modul-Karte | `saga/`, `vault/`, `gate/`, `journal/`, `ledger/`, `purge/`, `measure/`, `report/`, `restore/`, `kv_shredding.rs`, `revocation_log.rs`, `types_local.rs` | [`src/lib.rs`](https://github.com/tfufuz1/contextra/blob/16387928/crates/contextra-erasure/src/lib.rs) |
| Letzte strukturelle Änderung | Modulare Entkopplung | Commit [`916c88c5`](https://github.com/tfufuz1/contextra/commit/916c88c5) |
| Repo-eigene Doku | `AGENTS.md`, `SPEC.md`, `PLAN.md` im Crate-Verzeichnis | [`AGENTS.md`](https://github.com/tfufuz1/contextra/blob/16387928/crates/contextra-erasure/AGENTS.md), [`SPEC.md`](https://github.com/tfufuz1/contextra/blob/16387928/crates/contextra-erasure/SPEC.md), [`PLAN.md`](https://github.com/tfufuz1/contextra/blob/16387928/crates/contextra-erasure/PLAN.md) |

### 1.2 Vorhandene Kerntypen auf HEAD

| Typ / Funktion | Fundstelle auf HEAD | Anmerkung |
|---|---|---|
| `InMemoryLedger` | [`src/ledger.rs:167`](https://github.com/tfufuz1/contextra/blob/16387928/crates/contextra-erasure/src/ledger.rs#L167) | Ledger-Typ existiert, aber nicht als `ErasureLedger`. Persistenz und Fail-Closed-Verhalten sind zu prüfen (WP-8). |
| `SagaState` | [`src/saga/machine.rs:27`](https://github.com/tfufuz1/contextra/blob/16387928/crates/contextra-erasure/src/saga/machine.rs#L27) | Zustandsmaschine vorhanden. |
| `EraseSaga` | [`src/saga/machine.rs:109`](https://github.com/tfufuz1/contextra/blob/16387928/crates/contextra-erasure/src/saga/machine.rs#L109) | Saga-Objekt vorhanden. |
| `SlotType` / `encode_slot` | [`src/vault/slot.rs:70`](https://github.com/tfufuz1/contextra/blob/16387928/crates/contextra-erasure/src/vault/slot.rs#L70) | Tresor-Slots vorhanden. |
| `ErasedSet`, `ReadGate` | [`src/gate/erased_set.rs`](https://github.com/tfufuz1/contextra/blob/16387928/crates/contextra-erasure/src/gate/erased_set.rs), [`src/gate/read_gate.rs`](https://github.com/tfufuz1/contextra/blob/16387928/crates/contextra-erasure/src/gate/read_gate.rs) | Lesesperre vorhanden. |
| `DeletionProof` | [`src/deletion_proof.rs`](https://github.com/tfufuz1/contextra/blob/16387928/crates/contextra-erasure/src/deletion_proof.rs) | Beweistyp vorhanden. |
| `kv_shredding`, `revocation_log` | [`src/kv_shredding.rs`](https://github.com/tfufuz1/contextra/blob/16387928/crates/contextra-erasure/src/kv_shredding.rs), [`src/revocation_log.rs`](https://github.com/tfufuz1/contextra/blob/16387928/crates/contextra-erasure/src/revocation_log.rs) | Schlüsselhierarchie und Widerrufslog vorhanden. |
| `restore/guard`, `restore/replay` | [`src/restore/guard.rs`](https://github.com/tfufuz1/contextra/blob/16387928/crates/contextra-erasure/src/restore/guard.rs) | Restore-Schutz vorhanden. |

### 1.3 Auf HEAD nicht vorhanden (Namensprüfung über alle `crates/`)

| Name | Treffer auf HEAD | Historische Fundstelle |
|---|---|---|
| `ErasureLedger` (Typ) | 0 | [`crates/contextra-store/src/erasure_ledger/mod.rs`](https://github.com/tfufuz1/contextra/blob/71213b97b/crates/contextra-store/src/erasure_ledger/mod.rs) @ `71213b97b` |
| `FenceProtocol` / `lsm/fence.rs` | 0 | [`crates/contextra-store/src/lsm/fence.rs`](https://github.com/tfufuz1/contextra/blob/c1d5e2e64/crates/contextra-store/src/lsm/fence.rs) @ `c1d5e2e64` |
| `ScopeKeyring` | 0 | [`crates/contextra-crypto/src/scope_keyring.rs`](https://github.com/tfufuz1/contextra/blob/ffeb79555/crates/contextra-crypto/src/scope_keyring.rs) @ `ffeb79555` |
| `KeyHierarchy` (KEK/DEK) | 0 | [`crates/contextra-crypto/src/hierarchy.rs`](https://github.com/tfufuz1/contextra/blob/1364a54d5/crates/contextra-crypto/src/hierarchy.rs) @ `1364a54d5` |
| `execute_forget_pipeline` | 0 | [`crates/contextra-engine/src/contextra_impl/forget.rs:167`](https://github.com/tfufuz1/contextra/blob/9386c2687/crates/contextra-engine/src/contextra_impl/forget.rs#L167) @ `9386c2687` |
| `purge_keys` | 0 | Siehe §2.1 |
| `load_or_create_integrity_key` (WAL) | 7 Dateien enthalten den Namen auf HEAD; Zuordnung zum Löschpfad ist zu prüfen | Historisch: [`crates/memfuse-store/src/wal.rs`](https://github.com/tfufuz1/contextra/blob/f292806d0/crates/memfuse-store/src/wal.rs) @ `f292806d0` |
| WAL-Flusher-Actor | 0 | [`crates/contextra-store/src/wal/flusher.rs`](https://github.com/tfufuz1/contextra/blob/dbe74c290/crates/contextra-store/src/wal/flusher.rs) @ `dbe74c290` |
| `SplladeSignal` / `learned_sparse` | 0 | nie implementiert (kein Löschbezug, nicht Teil dieser Spezifikation) |

**Hinweis:** `ErasureLedger` ist als Name nicht vorhanden, `InMemoryLedger` auf HEAD übernimmt aber eine Ledger-Rolle. Vor WP-8 muss klar sein, ob `InMemoryLedger` ersetzt oder ausgebaut wird.

---

## 2. Historische Löschfunktionen: verifizierte Referenzen

Die folgende Tabelle listet die Schlüsselfunktionen der historischen Löschkette mit Commit, Pfad und Zeile. Alle Referenzen wurden gegen das Repository geprüft.

| Funktion | Commit | Pfad:Zeile | Verifiziert |
|---|---|---|---|
| `Contextra::execute_forget_pipeline` | `9386c2687` | [`crates/contextra-engine/src/contextra_impl/forget.rs:167`](https://github.com/tfufuz1/contextra/blob/9386c2687/crates/contextra-engine/src/contextra_impl/forget.rs#L167) | ja |
| `ProofIssuer::issue` | `d835c682d` | [`crates/contextra-crypto/src/deletion_proof/issuer.rs:112`](https://github.com/tfufuz1/contextra/blob/d835c682d/crates/contextra-crypto/src/deletion_proof/issuer.rs#L112) | ja |
| `ErasureLedger::append_batch` | `e0a7395f1` | [`crates/contextra-store/src/erasure_ledger/mod.rs:540`](https://github.com/tfufuz1/contextra/blob/e0a7395f1/crates/contextra-store/src/erasure_ledger/mod.rs#L540) | ja |
| `ProofStore::open` | `ea157f1cb` | [`crates/contextra-crypto/src/deletion_proof/proof_store.rs:96`](https://github.com/tfufuz1/contextra/blob/ea157f1cb/crates/contextra-crypto/src/deletion_proof/proof_store.rs#L96) | ja |
| `DpiPurgeDriver::run_physical` | `3158b654e` | [`crates/contextra-engine/src/deletion/purge_driver.rs:129`](https://github.com/tfufuz1/contextra/blob/3158b654e/crates/contextra-engine/src/deletion/purge_driver.rs#L129) | ja |
| `SstableReader::open_with_keyring` | `28998bdf7` | [`crates/contextra-store/src/sstable/reader.rs:288`](https://github.com/tfufuz1/contextra/blob/28998bdf7/crates/contextra-store/src/sstable/reader.rs#L288) | ja |
| `ErasureLedger` (Modul) | `71213b97b` | [`crates/contextra-store/src/erasure_ledger/mod.rs`](https://github.com/tfufuz1/contextra/blob/71213b97b/crates/contextra-store/src/erasure_ledger/mod.rs) | ja (Pfad) |
| `ErasureLedger` Entry | `e67ea4e55` | [`crates/contextra-store/src/erasure_ledger/entry.rs`](https://github.com/tfufuz1/contextra/blob/e67ea4e55/crates/contextra-store/src/erasure_ledger/entry.rs) | ja (Pfad) |
| `ScopeKeyring` | `ffeb79555` | [`crates/contextra-crypto/src/scope_keyring.rs`](https://github.com/tfufuz1/contextra/blob/ffeb79555/crates/contextra-crypto/src/scope_keyring.rs) | ja (Pfad) |
| `KeyHierarchy` | `1364a54d5` | [`crates/contextra-crypto/src/hierarchy.rs`](https://github.com/tfufuz1/contextra/blob/1364a54d5/crates/contextra-crypto/src/hierarchy.rs) | ja (Pfad) |
| `FenceProtocol` | `c1d5e2e64` | [`crates/contextra-store/src/lsm/fence.rs`](https://github.com/tfufuz1/contextra/blob/c1d5e2e64/crates/contextra-store/src/lsm/fence.rs) | ja (Pfad) |
| WAL-Integritätsschlüssel | `f292806d0` | [`crates/memfuse-store/src/wal.rs`](https://github.com/tfufuz1/contextra/blob/f292806d0/crates/memfuse-store/src/wal.rs) | ja (Pfad) |
| WAL-Flusher | `dbe74c290` | [`crates/contextra-store/src/wal/flusher.rs`](https://github.com/tfufuz1/contextra/blob/dbe74c290/crates/contextra-store/src/wal/flusher.rs) | ja (Pfad) |
| Bandit-Numerik (`cholesky_invert_f64`, `update_arm`) | `496589066` | [`crates/contextra-adapt/src/bandit.rs`](https://github.com/tfufuz1/contextra/blob/496589066/crates/contextra-adapt/src/bandit.rs) | ja (Pfad), kein Löschbezug |
| Fusion-Typen (`FixedRankScore`) | `8d8fbbb22` | [`crates/contextra-rank/src/fusion/types.rs`](https://github.com/tfufuz1/contextra/blob/8d8fbbb22/crates/contextra-rank/src/fusion/types.rs) | ja (Pfad), kein Löschbezug |

**Regel:** Ein Funktionsname ohne Zeile in dieser Tabelle gilt als nicht verifiziert und darf nicht als Referenz in den Code übernommen werden, bevor sein Commit geprüft ist.

---

## 3. Ziel-Architektur

### 3.1 Grundsatz

Die gesamte Löschlogik lebt in `contextra-erasure`. Das Crate bleibt autark: keine Abhängigkeit zu `contextra-types`, `contextra-ports`, `contextra-crypto` oder `contextra-store`. Wo ein Baustein Infrastruktur braucht (WAL, Fence, Index-Zugriff, Schlüsselspeicher), definiert `contextra-erasure` einen **Port** in `src/types_local.rs`. Die Implementierung liegt außerhalb des Crates als **Adapter**.

### 3.2 Ports (Soll)

| Port | Zweck | Implementiert von (Adapter) | Status auf HEAD |
|---|---|---|---|
| `StorageBackend` | Blob-Lesen und -Schreiben | Außerhalb | vorhanden (`types_local.rs`) |
| `Purger` | Physische Bereinigung | Außerhalb | vorhanden |
| `LayerScanner` | Messung je Schicht | Außerhalb | vorhanden |
| `Clock`, `Rng` | Injizierbare Zeit und Zufall | Außerhalb | vorhanden |
| `FenceGate` | Nebenläufigkeitsschutz beim Purge | `contextra-store` | **fehlt** (WP-6) |
| `WalSink` | Journal- und WAL-Anbindung | `contextra-store` | **fehlt** (WP-7) |
| `KeyStore` | Schlüsselvernichtung und Lesen | `contextra-crypto` / Vault | **fehlt** (WP-K) |
| `ResidueScanner` | Restspuren im Index | Index-Crate | **fehlt** (WP-12) |

### 3.3 Modul-Zuordnung (Soll)

| Arbeitspaket | Ziel-Modul in `contextra-erasure` |
|---|---|
| WP-1 | `error` (Fehlertaxonomie) |
| WP-2 | `purge` (PurgePhase) |
| WP-3 | `scope` (dauerhafter Scope, Single-Flight) |
| WP-4 | `saga/driver` und `restore/replay` (Resume) |
| WP-5 | `vault` (Rewrite, Journal, Manifest) über Port |
| WP-6 | `gate` (Fence) über `FenceGate` |
| WP-7 | `journal` über `WalSink` |
| WP-8 | `ledger` |
| WP-9 | `measure`, `report`, `deletion_proof` |
| WP-10 | `saga` und `orchestrator` |
| WP-11 | `restore` |
| WP-12 | `measure` über `ResidueScanner` |
| WP-K | `kv_shredding` und `vault/keyring` über `KeyStore` |

---

## 4. Invarianten

Jede Invariante wird durch mindestens einen benannten Test erzwungen.

| ID | Aussage | Status |
|---|---|---|
| INV-CLASS-MEASURED-1 | Kein Löschbericht ohne verifizierte Scanner-Zeugen | vorhanden (Repo-Doku) |
| INV-NO-RESURRECTION-1 | Keine Reaktivierung gelöschter Schlüssel nach Replay oder Wiederanlauf | vorhanden (Repo-Doku) |
| INV-VAULT-FAILCLOSED-1 | Jeder E/A- oder Synchronisationsfehler beim Überschreiben vergiftet das Vault-Handle dauerhaft | vorhanden (Repo-Doku) |
| INV-CLAIM-BIND-1 | Berichte enthalten keine rechtsverbindlichen Garantiezusagen | vorhanden (Repo-Doku) |
| INV-ERA-1 | Genau ein Löschpfad: kein paralleler Koordinator | neu (WP-10) |
| INV-ERA-2 | Purge ist ohne Verlust wiederaufnehmbar (Resume führt den Purge tatsächlich aus) | neu (WP-4) |
| INV-ERA-3 | Pro Scope höchstens ein laufender Purge (Single-Flight) | neu (WP-3) |
| INV-ERA-4 | Jede Schlüsselvernichtung ist im Ledger vor dem Ende der Saga verankert | neu (WP-8) |
| INV-ERA-5 | Ein Fence blockiert konkurrierende Schreiber während eines physischen Purge | neu (WP-6) |
| INV-ERA-6 | WAL-Einträge zu einem gelöschten Scope werden entsorgt oder unlesbar gemacht | neu (WP-7) |
| INV-ERA-7 | Ein Löschbericht bindet die Zeugen, auf die er sich stützt (Hash-Kette) | neu (WP-9) |

---

## 5. Arbeitspakete

Jedes Arbeitspaket ist ein eigener PR. Er enthält Code, Tests und Doku im selben Commit. Reihenfolge und Abhängigkeiten stehen in §6.

### WP-0: Baubarkeit und Gate

- **Ziel:** Das Crate baut standalone und alle Tests laufen grün, bevor ein Restore beginnt.
- **Ist:** Standalone-Build dokumentiert (`AGENTS.md` §1). Keine Messung des aktuellen Zustands in dieser Spezifikation.
- **Akzeptanz:** `cargo check -p contextra-erasure --no-default-features` und `cargo test -p contextra-erasure` grün; `cargo clippy` ohne `unwrap`, `expect`, `panic` in Produktionscode.
- **Abhängigkeit:** keine.
- **Historische Testinfrastruktur:** Konformanz-Suiten und Fault-Injection existierten historisch im Testkit-Crate. Die Commit-Referenz dafür ist noch nicht verifiziert. Übernahme nur, wenn sie ohne Abhängigkeit aus `contextra-erasure` heraus läuft.

### WP-1: Fehlertaxonomie des Purge

- **Ziel:** Ein Fehlertyp pro Phase, keine Stringfehler.
- **Ist:** `error.rs` vorhanden, Purge-Fehler nicht taxonomisch getrennt (Prüfung in diesem WP).
- **Ziel-Modul:** `error`.
- **Akzeptanz:** Jeder Fehler ist einer Phase zugeordnet und hat einen Test, der den Zustand nach dem Fehler prüft.
- **Abhängigkeit:** WP-0.

### WP-2: Eine `PurgePhase`

- **Ziel:** Eine einzige Zustandsmaschine für den physischen Purge.
- **Ist:** `purge/window.rs`, `purge/order.rs`, `purge/reserve.rs`, `purge/cx.rs` existieren. Phasen sind implizit verteilt.
- **Ziel-Modul:** `purge`.
- **Akzeptanz:** Eine Enum-Definition für alle Phasen; jeder Übergang ist in `can_transition_to`-Form geprüft.
- **Abhängigkeit:** WP-1.

### WP-3: Dauerhafter Scope und Single-Flight (INV-ERA-3)

- **Ziel:** Der Scope eines Purge überlebt Abstürze, und pro Scope läuft höchstens ein Purge.
- **Ist:** `scope/codec.rs`, `scope/file.rs`, `scope/set.rs` vorhanden.
- **Ziel-Modul:** `scope`.
- **Akzeptanz:** Test mit simuliertem Absturz zwischen zwei Purge-Starts zeigt genau einen laufenden Purge.
- **Abhängigkeit:** WP-2.

### WP-4: Resume führt den Purge aus (INV-ERA-2)

- **Ziel:** Ein Wiederanlauf setzt den unterbrochenen Purge fort, statt nur seinen Zustand zu lesen.
- **Ist:** `restore/replay.rs` und `saga/driver.rs` vorhanden. Ob ein Resume tatsächlich Purge-Schritte ausführt, ist zu prüfen.
- **Ziel-Modul:** `saga/driver` und `restore/replay`.
- **Akzeptanz:** Crash-Matrix-Test (`tests/crash_matrix_recovery.rs`, auf HEAD vorhanden) erweitert um Purge-Fortsetzung.
- **Abhängigkeit:** WP-3.

### WP-5: `purge_keys` mit Rewrite, Journal, Manifest und RAM-Swap

- **Ziel:** Schlüsselvernichtung über Rewrite mit Journal und Manifest, ohne Klartext im Swap.
- **Historische Referenz:** Name `purge_keys` als Funktion auf HEAD nicht vorhanden; siehe Kandidatenliste (§7).
- **Ziel-Modul:** `vault` über Port.
- **Akzeptanz:** Test prüft, dass nach einem Absturz im Rewrite der Klartext nicht lesbar ist.
- **Abhängigkeit:** WP-4, WP-K.

### WP-5a: Sofortfix Swap-Fehler

- **Ziel:** Ein Fehler beim Speicher-Swap darf keinen Klartext hinterlassen.
- **Ziel-Modul:** `vault/sealer.rs`.
- **Akzeptanz:** Fault-Injection-Test am Swap.
- **Abhängigkeit:** WP-0 (kann vor WP-1 starten).

### WP-6: Fence-Protokoll (INV-ERA-5)

- **Ziel:** Während eines physischen Purge blockiert ein Fence konkurrierende Schreiber.
- **Historische Referenz:** [`crates/contextra-store/src/lsm/fence.rs`](https://github.com/tfufuz1/contextra/blob/c1d5e2e64/crates/contextra-store/src/lsm/fence.rs) @ `c1d5e2e64`, 51 Funktionen (nicht übernehmen, ohne vorher Schnittstelle und Testabdeckung zu prüfen).
- **Ziel-Modul:** `gate` über Port `FenceGate`; Adapter in `contextra-store`.
- **Akzeptanz:** Nebenläufigkeitstest mit `loom` (als Dev-Dependency vorhanden) zeigt, dass kein Schreiber nach dem Fence eine Änderung im gefencten Bereich sieht.
- **Abhängigkeit:** WP-4.

### WP-7: WAL-Entsorgung, Scope-Purge und Verankerung (INV-ERA-6)

- **Ziel:** WAL-Einträge zu einem gelöschten Scope sind entsorgt oder unlesbar; die Entsorgung ist verankert.
- **Historische Referenz:** WAL-Integritätsschlüssel [`crates/memfuse-store/src/wal.rs`](https://github.com/tfufuz1/contextra/blob/f292806d0/crates/memfuse-store/src/wal.rs) @ `f292806d0`.
- **Ausschluss:** Der historische Flusher-Actor mit eigenen Threads ([`wal/flusher.rs`](https://github.com/tfufuz1/contextra/blob/dbe74c290/crates/contextra-store/src/wal/flusher.rs) @ `dbe74c290`) wird **nicht** übernommen. Er widerspricht der synchronen Kernregel des Repos.
- **Ziel-Modul:** `journal` über Port `WalSink`.
- **Akzeptanz:** Test liest das WAL nach Scope-Purge und findet keinen Klartext zum Scope.
- **Abhängigkeit:** WP-6.

### WP-8: Ledger (INV-ERA-4)

- **Ziel:** Jede Schlüsselvernichtung ist vor Ende der Saga im Ledger verankert. Fail-Closed beim Öffnen.
- **Ist:** `InMemoryLedger` ([`src/ledger.rs:167`](https://github.com/tfufuz1/contextra/blob/16387928/crates/contextra-erasure/src/ledger.rs#L167)) auf HEAD. Keine Persistenz verifiziert.
- **Historische Referenz:** `ErasureLedger::append_batch` ([`erasure_ledger/mod.rs:540`](https://github.com/tfufuz1/contextra/blob/e0a7395f1/crates/contextra-store/src/erasure_ledger/mod.rs#L540) @ `e0a7395f1`), Entry ([`entry.rs`](https://github.com/tfufuz1/contextra/blob/e67ea4e55/crates/contextra-store/src/erasure_ledger/entry.rs) @ `e67ea4e55`). Empfehlung: neu konstruieren, Vertrag zuerst, alte Funktionen nur als Referenz.
- **Ziel-Modul:** `ledger`.
- **Akzeptanz:** Fail-Closed-Tests: fehlende oder beschädigte Ledger-Datei führt zum Abbruch, nicht zu stiller Fortsetzung.
- **Abhängigkeit:** WP-4.

### WP-9: Zeugen und Beweis (INV-ERA-7)

- **Ziel:** Ein Löschbericht bindet die Zeugen, auf die er sich stützt. Kanonische, deterministische Form.
- **Ist:** `report/build.rs`, `report/canonical.rs`, `report/signing.rs`, `measure/witness.rs` vorhanden. Historische Beweis-Kette in `contextra-crypto/src/deletion_proof/` verteilt.
- **Historische Referenz:** `ProofIssuer::issue` ([`issuer.rs:112`](https://github.com/tfufuz1/contextra/blob/d835c682d/crates/contextra-crypto/src/deletion_proof/issuer.rs#L112) @ `d835c682d`), `ProofStore::open` ([`proof_store.rs:96`](https://github.com/tfufuz1/contextra/blob/ea157f1cb/crates/contextra-crypto/src/deletion_proof/proof_store.rs#L96) @ `ea157f1cb`).
- **Ziel-Modul:** `measure`, `report`, `deletion_proof`.
- **Akzeptanz:** Externer Verifikationstest (`tests/deletion_proof_external_verify_rejects_tamper.rs`, auf HEAD vorhanden) bleibt grün; neue Test-Vektoren für kanonische Form.
- **Abhängigkeit:** WP-8.

### WP-10: Ein Koordinator, ein Pfad (INV-ERA-1)

- **Ziel:** Genau ein Löschpfad. Der alte Pfad über `Contextra::execute_forget_pipeline` ([`forget.rs:167`](https://github.com/tfufuz1/contextra/blob/9386c2687/crates/contextra-engine/src/contextra_impl/forget.rs#L167) @ `9386c2687`) und `DpiPurgeDriver::run_physical` ([`purge_driver.rs:129`](https://github.com/tfufuz1/contextra/blob/3158b654e/crates/contextra-engine/src/deletion/purge_driver.rs#L129) @ `3158b654e`) werden in `saga` und `orchestrator` zusammengeführt.
- **Ziel-Modul:** `saga`, `orchestrator`.
- **Akzeptanz:** Statische Prüfung: genau eine öffentliche Einstiegsfunktion für Löschung im Crate.
- **Abhängigkeit:** WP-9.

### WP-11: Restore und Anti-Resurrection (INV-NO-RESURRECTION-1)

- **Ziel:** Ein Restore aus Backup reaktiviert keine gelöschten Schlüssel.
- **Ist:** `restore/guard.rs` und `restore/replay.rs` vorhanden.
- **Ziel-Modul:** `restore`.
- **Akzeptanz:** Test: Backup vor Löschung wird eingespielt, gelöschter Scope bleibt unlesbar.
- **Abhängigkeit:** WP-10.

### WP-12: Restspuren im Index

- **Ziel:** Nach Löschung enthält der Vektorindex keine lesbaren Spuren des gelöschten Dokuments.
- **Historische Referenz:** `purger.rs`, `residue_scan`, `zero_overlay` im HNSW-Modul. Commit-Angaben für diese Dateien sind noch nicht am Repository verifiziert.
- **Ziel-Modul:** `measure` über Port `ResidueScanner`; Adapter im Index-Crate.
- **Akzeptanz:** Positivkontrolle: bekannter Restwert wird gefunden. Negativkontrolle: nach Löschung wird er nicht gefunden.
- **Abhängigkeit:** WP-9.

### WP-K: Schlüsselhierarchie (ergänzt diese Spezifikation)

- **Ziel:** KEK pro Gruppe, DEK pro Datensatz, gewrappt; Vernichtung des KEK macht alle DEK der Gruppe unlesbar.
- **Ist:** `kv_shredding.rs` und `vault/keyring.rs` auf HEAD vorhanden. Gruppen-Vernichtung und Epochen-Trennung sind zu prüfen.
- **Historische Referenz:** `ScopeKeyring` ([`scope_keyring.rs`](https://github.com/tfufuz1/contextra/blob/ffeb79555/crates/contextra-crypto/src/scope_keyring.rs) @ `ffeb79555`), `KeyHierarchy` ([`hierarchy.rs`](https://github.com/tfufuz1/contextra/blob/1364a54d5/crates/contextra-crypto/src/hierarchy.rs) @ `1364a54d5`), `SstableReader::open_with_keyring` ([`reader.rs:288`](https://github.com/tfufuz1/contextra/blob/28998bdf7/crates/contextra-store/src/sstable/reader.rs#L288) @ `28998bdf7`).
- **Ziel-Modul:** `kv_shredding`, `vault/keyring` über `KeyStore`.
- **Akzeptanz:** Test: nach Vernichtung eines KEK sind alle abhängigen DEK nicht mehr entschlüsselbar. Test: Ein DEK ist nie im Klartext in einem Speicherbereich, der nicht gelöscht wird.
- **Abhängigkeit:** WP-0. Kann parallel zu WP-1 bis WP-3 laufen.

---

## 6. Reihenfolge und Abhängigkeiten

```
WP-0 ──► WP-1 ──► WP-2 ──► WP-3 ──► WP-4 ──► WP-6 ──► WP-7
  │                                  │         
  ├──► WP-5a                         ├──► WP-5 ◄── WP-K
  │                                  │
  └──────────────► WP-K ─────────────┘
                                     └──► WP-8 ──► WP-9 ──► WP-10 ──► WP-11
                                                              └──► WP-12
```

**Parallel möglich:** WP-K und WP-1 bis WP-3. WP-5a nach WP-0, unabhängig von allen anderen. WP-11 und WP-12 nach WP-10 bzw. WP-9.

---

## 7. Zuordnung der historischen Kandidaten

Die Datei `loeschkette_zuordnung.csv` ordnet jede der 4.546 Kandidaten-Zeilen einem Arbeitspaket zu. Spalten: Kategorie, Quell-Crate, Quell-Datei, qualifizierter Name, Relevanz, Arbeitspaket, Ziel-Modul, Commit, GitHub-Link am Commit mit Zeile, HEAD-Status, HEAD-Fundstelle mit GitHub-Link, und ob die Funktion auf HEAD bereits in `contextra-erasure` liegt.

### 7.1 Ergebnis

| Arbeitspaket | Zeilen gesamt | Relevanz KERN | Im Zielcrate auf HEAD |
|---|---|---|---|
| WP-0 | 236 | 229 | 49 |
| WP-1 | 13 | 10 | 0 |
| WP-2 | 181 | 181 | 31 |
| WP-3 | 19 | 19 | 0 |
| WP-4 | 6 | 6 | 0 |
| WP-5 | 61 | 55 | 34 |
| WP-6 | 58 | 58 | 0 |
| WP-7 | 90 | 83 | 5 |
| WP-8 | 173 | 173 | 41 |
| WP-9 | 901 | 796 | 285 |
| WP-10 | 738 | 735 | 521 |
| WP-11 | 53 | 51 | 13 |
| WP-12 | 1.052 | 146 | 19 |
| WP-K | 201 | 199 | 68 |
| PRUEFEN | 707 | 0 | 19 |
| AUSSERHALB | 57 | 0 | 0 |

### 7.2 Einordnung der Relevanz

- **KERN:** Name oder Pfad enthält einen Löschbegriff (`erasure`, `purge`, `forget`, `proof`, `ledger`, `fence`, `wal`, `restore`, `key`, `zeroize` u. a.).
- **NACHBAR:** Kein Löschbegriff im Namen, aber im Löschkern-Crate oder im Speicher-Kern.
- **RAUSCHEN:** Keine Verbindung zur Löschkette (z. B. Distanzfunktionen, DOM-Helfer, Suchfunktionen). Diese Zeilen werden nicht übernommen.
- **PRUEFEN:** Keine eindeutige Zuordnung. Manuelle Prüfung nötig.

**Grenze der Methode:** Die Zuordnung ist regelbasiert über Namen und Pfade. Sie ist ein Vorschlag, kein Nachweis. Jede Zeile mit Relevanz KERN, die in ein Arbeitspaket übernommen wird, braucht eine Prüfung des Funktionskörpers am Commit.

### 7.3 Rauschen-Anteil

Von 4.546 Zeilen sind 1.334 RAUSCHEN. Der Großteil (883) gehört zu WP-12, weil die Vektor-Crates allgemeine Funktionen wie Distanzberechnung und Heuristiken enthalten. Diese Zeilen sind in der CSV markiert und dürfen nicht als Löschfunktionen gezählt werden.

---

## 8. Befunde in der Repo-Dokumentation

Diese Punkte sind vor der Umsetzung zu klären:

1. **Ring-Einordnung widersprüchlich.** [`crates/contextra-erasure/AGENTS.md`](https://github.com/tfufuz1/contextra/blob/16387928/crates/contextra-erasure/AGENTS.md) nennt „Ebene 0“. [`Cargo.toml`](https://github.com/tfufuz1/contextra/blob/16387928/crates/contextra-erasure/Cargo.toml) setzt `ring = "3"`. Eine der beiden Angaben ist falsch.
2. **Spezifikationsversion.** `SPEC.md` des Crates zitiert „docs/SPEC.md v4.0“, das Root-`AGENTS.md` nennt v4.1. Maßgeblich muss eine Version sein.
3. **ErasureLedger-Begriff.** Die Löschkette spricht von `ErasureLedger`, das Crate führt `InMemoryLedger`. Der Name muss vor WP-8 entschieden werden.
4. **Fence-Protokoll.** Historisch vorhanden, auf HEAD nicht. Eine Entscheidung, ob WP-6 den alten Fence übernimmt oder neu baut, fehlt.

---

## 9. Abnahme

Ein Arbeitspaket gilt als abgenommen, wenn:

1. `cargo check -p contextra-erasure --no-default-features` grün ist,
2. `cargo test -p contextra-erasure` grün ist,
3. die zugehörigen Invarianten (§4) durch benannte Tests erzwungen sind,
4. keine neue Abhängigkeit zu `contextra-*` hinzugekommen ist,
5. Clippy ohne `unwrap`, `expect`, `panic` in Produktionscode läuft,
6. jede übernommene historische Funktion mit Commit und Zeile dokumentiert ist.

Gesamtabnahme: Autarkie-Score ≥ 95/100 (Messung über das Repo-Werkzeug `tools/decouple_inspector.py`, falls vorhanden; sonst manuell prüfen).

---

## 10. Offene Entscheidungen

| # | Frage | Empfehlung |
|---|---|---|
| E1 | Ring: Ebene 0 oder Ring 3? | Zuerst klären, dann beide Dateien angleichen. |
| E2 | Maßgebliche Spezifikation | `contextra-erasure/SPEC.md` plus diese Spezifikation. |
| E3 | ErasureLedger: neu bauen oder `InMemoryLedger` ausbauen? | Ausbauen, historische Referenz nur für Vertrag und Tests. |
| E4 | Fence: alten übernehmen oder neu bauen? | Neu bauen, Vertrag aus historischer Schnittstelle ableiten. |
| E5 | Schlüsselgruppen-Wechsel (Epochen) | Vor WP-K festlegen. |
| E6 | Rechtliche Einordnung der Außenformulierung | Vor Veröffentlichung extern prüfen lassen. Die Spezifikation macht keine Rechtsaussage. |

---

## 11. Grenzen dieser Spezifikation

- Die Zuordnung der 4.546 Zeilen ist regelbasiert. Sie ist ein Vorschlag, der am Funktionskörper zu prüfen ist.
- Die Referenzen in §2 sind an Commit und Pfad geprüft, nicht an Funktionskörper oder Korrektheit.
- Keine Kompilierung der historischen Commits. Ob ein historischer Stand baubar ist, ist nicht geprüft.
- Die Schlüsselhierarchie (WP-K) und der Fence (WP-6) sind im Repo nur historisch belegt. Ihre Semantik ist nicht gegen Tests geprüft.
- Die Außenformulierung der Löschwirkung ist keine rechtliche Aussage.
