# contextra-crypto — Spezifikation

> **Geltung:** Crate-lokale Spezifikation. Normativ bleibt `docs/SPEC.md` (v2, 10.11.2026). Dieses Dokument überträgt die Anforderungen für dieses Crate, ergänzt sie um verifizierte Ist-Fakten und markiert Abweichungen.
> **Faktenbasis:** statische Analyse mit `tools/` am geklonten Repo-Stand (`main`, HEAD `f0eda1cb9`). `cargo` ist in der Analyseumgebung nicht verfügbar. **Es wurde nichts gebaut, getestet oder ausgeführt.**
> **Beleg-Marken:** *[V]* am Quelltext/Manifest verifiziert, *[T]* aus einem tools/-Bericht, *[S]* aus docs/SPEC.md übernommen, *[E]* Festlegung dieser Spezifikation.

**Pfad:** `crates/contextra-crypto/` · **Umsetzungsplan:** `PLAN.md` (im selben Verzeichnis)

## 1. Zweck

Kryptografische Bausteine: AEAD, DEK, KDF, Secrets, WAL-HMAC. Der KV-Teil wandert nach `kvcache`. *[S]*

## 2. Zielzustand (docs/SPEC.md §3.1) [S]

| Merkmal | Ziel |
|---|---|
| Gruppe | foundation |
| Ebene (Kern-Closure) | 1 |
| Kern-Closure | ids, env |
| Default-Features | **leer** |
| Fehlertyp (R3) | `CryptoError` (besteht) |
| Brücke (hinter Feature `workspace`, §1.1) | `From<CryptoError>` |

Ebenen (SPEC §1.1): E0 = Kern-Closure leer · E1 = Kern-Closure ⊆ E0, höchstens 4 Mitglieder · E2 = Integration · E3 = Anwendungen und Testwerkzeuge. Der Kern ist der Code mit `--no-default-features` ohne Feature `workspace`.

## 3. Ist-Zustand (tools/)

| Messgröße | Wert |
|---|---|
| Lines of Code (Produktion, `loc.py`) | 6821 Zeilen in 32 Dateien |
| Version / publish | workspace-vererbt / <unset> |
| Interne Normal-Abhängigkeiten (Ist, Cargo.toml) | types, ports |
| Interne Dev-Abhängigkeiten (Ist) | testkit |
| Features (Ist, Anzahl) | 5 |
| Default-Features (Ist) | legacy-erasure, kv-encryption, test-utils |
| `cfg(feature = …)`-Stellen in src/ | 48 |
| `#[test]`-Vorkommen (grep, src + tests) | 334 |
| zero-panic: Produktionsbefunde, bereinigt (Tool-Gesamt; davon in Testdateien) | 0 (Tool 1; 1 in Testdateien) |
| unsafe: Vorkommen / ohne SAFETY / Insel | 0 / 0 / False |
| Determinismus: Produktionsbefunde, bereinigt (Tool-Gesamt) | 0 (Tool 0) |
| Port-M3-Leaks (port_contract_audit) | 0 |
| Error-Enums (Ist) | CryptoError, DeletionProofError, VerificationErrorReason |

**Tool-Caveat:** `zero_panic_analyzer.py` klassifiziert `#[cfg(test)]`-Module (`tests.rs`) als Produktion. Im Repo betrifft das 449 von 478 Treffern. Die Zahlen oben sind deshalb bereinigt; die Rohzahl steht in Klammern. Die Korrektur am Tool ist eigener Schritt (S-00).

**Quellen:** `tools/loc.py`, `tools/zero_panic_analyzer.py`, `tools/unsafe_boundary_auditor.py`, `tools/determinism_guard.py`, `tools/port_contract_auditor.py`, `tools/error_topology_mapper.py`, `tools/cargo_dependency_graph.py`, Cargo.toml-Auswertung.

## 4. Ist-vs-Ziel-Abgleich [V]

- **Ist-Kanten, die das Ziel nicht vorsieht:** `types`, `ports`
- **Ziel-Kanten, die im Ist fehlen:** `ids`, `env`
- **Default-Features weichen ab:** Ist `legacy-erasure, kv-encryption, test-utils` → Ziel `leer`.

## 5. Verifizierte Befunde und Abweichungen zu docs/SPEC.md [V]

- Verifiziert: `CURRENT_KV_FORMAT_VERSION = 2` in `src/kv_cipher.rs:46`. `CURRENT_KV_KEY_DERIVATION_VERSION = 1` in `src/kv_segment/segment.rs:21` (B10 bestätigt, Duplikat siehe `kvcache`).

## 6. Anforderungen

### 6.1 Crate-Spezifikation (docs/SPEC.md §6) [S]

Kern bleibt: `aead` (190), `dek` (153), `kdf` (319), `secret` (146), `error` (146), `anti_tamper` (191), `crypto.rs`/`KeyManager` (948), `wal_crypto` (1.163), `wal_completeness` (75). `TenantId`/`TenantScoped` aus `ids`. `KeyManager::{derive_kv_key, derive_kv_key_scoped, cipher_for, cipher_for_scoped}` (`crypto.rs:238–316`) nehmen `[u8; 32]` statt `kv_cipher::ModelFingerprint`; vor dem Umzug des KV-Teils umstellen (eigener Commit, Aufrufer in `kvcache`/`infer-candle` mit). `default = []`. `ed25519-dalek` bleibt, solange `ed25519_proof` im Crate bleibt (B10). KV-Teil und Altlast: B10, S-38.

### 6.2 Brückenmuster [S]

Die Brücke liegt ausschließlich in `src/compat/` hinter dem Feature `workspace`. Verbindliche Muster: docs/SPEC.md §4.3 (a–d). R2 gilt: `contextra_types`, `contextra_ports`, `contextra_testkit` dürfen im Kern nirgends vorkommen, auch nicht in Doc-Kommentaren oder `#[cfg(test)]`.

### 6.3 Regeln für jeden Schritt [S]

INV-1 Formatgleichheit · INV-2 Pfadgleichheit (Re-Export mit `#[deprecated]`) · INV-3 Ein Schritt = Commit-Paar · INV-4 `cargo check --workspace --all-targets --locked` grün · INV-5 Determinismus (Tie-Break nach ID aufsteigend) · INV-6 Durability-Pfade unberührt · INV-7 keine Compliance-Aussagen · INV-8 keine Rückwärtspfeile. Vollständig: docs/SPEC.md §1.2, Regeln R1–R9: §3.2.

### 6.5 Fehlerabbildung [S]

Eigener Fehlertyp: `CryptoError` (besteht) (thiserror, `#[non_exhaustive]`). Jede heute genutzte `ContextraError`-Variante hat genau eine lokale Entsprechung; `Display` bleibt bitgleich; keine Sammelvariante `Other(String)` ohne Quelle. Tabelle je Baustein: `docs/decoupling/errors/<crate>.md` (aus S-xx erzeugt). Quelle: §5.3.

## 7. Schnittstellen und Persistenz

### 7.1 Persistierte Formate (§5.1 Golden-Korpus) [S]

| Format | Quelle / Kennung |
|---|---|
| KDF-Header | `crypto/kdf.rs`: `MFKD` |
| KV-Segment | `crypto/kv_cipher.rs`: Formatversion 2; Schlüsselableitung Version 1 |

Jeder Eintrag hat einen Erzeugertest (`#[ignore]`, Script `tools/golden_regen.sh`) und einen Prüftest, der immer läuft. Abweichung ⇒ Schritt abbrechen.

### 7.2 Öffentliche Schnittstelle

Ist-Stand und Ziel-Schnittstelle sind in den Abschnitten 3 bis 6 festgehalten. Die exakte Liste der `pub`-Symbole erhebt S-00 über `cargo public-api` (Gate G-A1); erlaubte Differenzen sind nur Re-Export-Pfade mit `#[deprecated]`.

## 8. Fundstellen (tools/, exakt) [T]

**zero-panic (Produktion, bereinigt: 0; die ersten 15):** Tool-Gesamt 1, davon 1 in Testdateien (`tests.rs` unter `#[cfg(test)]`, Tool-Fehlklassifikation).


**Fehler-Enums (Ist):**

- `crates/contextra-crypto/src/error.rs:21` — CryptoError
- `crates/contextra-crypto/src/ed25519_proof.rs:29` — DeletionProofError
- `crates/contextra-crypto/src/deletion_proof/verifier.rs:14` — VerificationErrorReason

## 9. Befunde aus docs/SPEC.md (B-Nummern) [S]

**B04 — Entkopplungsstand von `text` ist falsch eingeschätzt; Zählungen weichen ab**
> **Entscheidung:** Zahlen sind Orientierung. Abnahme erfolgt ausschließlich über die Null-Prüfung G-L2 (§8.2) und den Build ohne Features, nie über Zählwerte. Aufwandsklasse `text`: mittel.

**B07 — Eine Verzeichnis-Regruppierung bricht messbar viel**
> **Entscheidung (D-01):** Das Verzeichnislayout bleibt **flach** (`crates/contextra-*`). Die Gruppierung steht als Metadatum `group` in `capabilities.toml`. Die physische Regruppierung ist Nicht-Ziel (§9.3); sie liefert keinen Entnahmegewinn und kostet die oben genannten Brüche.

**B10 — Das Verschieben der Krypto-Dateien bricht Garantie-Nachweise**
> **Entscheidung:** - `legacy-erasure` verlässt die Defaults von `crypto` (S-12). Die Dateien `audit_chain`, `deletion_proof/`, `ed25519_proof` werden **nicht** archiviert, solange nicht (a) ein Item-Diff gegen `erasure` Gleichwertigkeit zeigt **und** (b) der Mutation-Score der Zieldateien in `erasure` ≥ Score der Quelldateien ist (Gate G-C1). Bis dahin bleiben sie, hinter dem Feature, im Crate (Tier frozen). - Wandern `kv_cipher`, `kv_segment/*`, `kv_shredding`, `revocation_log` nach `kvcache`, MUSS derselbe Commit `mutants-guarantees.toml`, `loom-gate.yml` und `CODEOWNERS` auf die neuen Pfade ziehen und beide Versionskonstanten bis zur Vereinigung per `const_assert_eq` koppeln. Nach dem Umzug ist `kvcache` alleiniger Besitzer; `crypto` definiert keine KV-Versionskonstante mehr. - Schnitt und Umzug (S-38) bilden ein untrennbares Commit-Paar.

**B11 — Nebenläufigkeits-Abdeckung fehlt genau dort, wo umgebaut wird**
> **Entscheidung:** S-01 nimmt `mvcc`, `kvcache`, `checkpoint`, `ids`, `env` in die Pfadfilter von `tsan.yml`/`loom-gate.yml`, `mvcc`+`kvcache` in die Crate-Liste der Mutation-Tests und `rust-ci.yml` auf. Enthält `crates/contextra-mvcc/tests` kein Loom-Modell (`rg -l loom crates/contextra-mvcc` leer), ist vor S-30 ein Modell für `SequenceLog` (Publish/Visibility, zwei Schreiber, ein Leser) zu schreiben; sonst ist S-30 nicht abnehmbar.

**B13 — Dev-Kanten auf `testkit` und Entnahmenachweis**
> **Entscheidung:** Dev-Kanten zu `testkit` sind erlaubt, stehen aber im Manifest `dev_allowed` (§8.3). Der Isolationsnachweis (G-L4) prüft `--edges normal` und baut zusätzlich `--no-dev-deps`; Tests laufen im Export nur mit `--with-dev` (nimmt dann `testkit`-Closure mit). `env` (Feature `test-utils`) übernimmt `ManualClock` und die Clock/Rng-Konformitätssuite, damit E1-Crates dafür nicht auf `testkit` angewiesen sind.

**B14 — Defaults und leere Features (Teil 2, G5/G6) — bestätigt, plus Ergänzungen**
> **Entscheidung:** Zieldefaults §3.1. Da `default-members` diese Crates auslässt, genügt „wöchentlich `cargo check`“ nicht als Kompilierbarkeitsgarantie; CI führt bei **jedem** PR `cargo check --workspace --all-targets --locked` aus (S-01). Ein Feature ohne `cfg(feature = "x")` im Quelltext **und** ohne Weiterleitung in einem anderen Manifest wird entfernt; geprüft vom neuen `tools/feature_audit.py` (S-12).

**B16 — Einzelentscheidungen, die der Entwurf offen ließ**
> `ModelFingerprint` (Plan: → `kvcache`; Teil 2 K9: → `ports/kv`) — `ports/kv.rs` importiert ihn aus `types`. Alle Verbraucher der Typ-Variante sind Brücken oder Re-Exports (`crypto` wird auf Roh-Bytes umgestellt). — **Bleibt in `types`.** Kein Umzug. `kvcache` und `infer-candle` führen im Kern einen …

## 10. Entscheidungen und Risiken [S]

**Entscheidungen (§9.1):**

| D-06 | `legacy-erasure`-Dateien in `crypto` bleiben bis Nachweis (B10) |

Nicht-Ziele (§9.3), gelten für alle Crates: physische Regruppierung, Mikro-Crate-Zerlegung (F-1), Verschieben von `Vfs`/`io_seam`, crates.io-Veröffentlichung, Formatänderungen außer ID-Breite, Änderung der FlatBuffers-Schemas.

## 11. Abnahme [S]

Ein Crate gilt als entnehmbar (R6), wenn in dieser Reihenfolge grün: **G-L2** (Null-Prüfungen, nur bei Brücke) → **G-L1** (Kern- und Feature-Closure) → **G-L3** (`--no-default-features`, `--each-feature`) → **G-L4** (isolierter Export). Danach **G-F1/G-D1/G-E1/G-C1** für den betroffenen Schritt (§8.4).

```bash
cargo check -p contextra-crypto --no-default-features --locked
cargo hack check -p contextra-crypto --each-feature --no-dev-deps --locked
python3 tools/isolation_proof.py --crate contextra-crypto --modes no-default,default,each-feature
```

