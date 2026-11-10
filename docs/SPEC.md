# Contextra als Baukasten — Implementierungsspezifikation v3 (normativ)

*Stand: 10. November 2026 · Repository `tfufuz1/contextra` · Geprüfter HEAD `5408826f8` · Ersetzt `docs/SPEC.md` v2 (683 Zeilen; der Upload ist byte-identisch dazu)*

---

## 0. Status, Geltung, Änderungsprotokoll

### 0.1 Grundlage und Grenzen

| Punkt | Inhalt |
| --- | --- |
| Analysierter Stand | HEAD `5408826f8` (Merge PR #5959, 10.11.2026). v2 beschreibt `ef3364f`. Was seitdem umgesetzt wurde, steht in §2. |
| Methode | Statische Analyse: `tomllib` über alle Manifeste, `grep`-Zählungen, Dateisystemprüfung und die Skripte in `tools/` (`decouple_inspector.py`, `count_bridge_refs.py`, `feature_audit.py`, `loc.py`). |
| Nicht möglich | In der Analyseumgebung fehlen `cargo`, `cargo-hack`, `rg` und `jq`. **Nichts wurde gebaut, getestet oder ausgeführt.** Jede Zeile „Nachweis“ ist ein Abnahmekriterium, kein Ergebnis. |
| Beleg-Marken | **\[V\]** am HEAD gemessen · **\[T\]** aus einem `tools/`-Lauf · **\[A\]** aus \[V\] abgeleitet · **\[Ü\]** aus v2 übernommen, nicht neu geprüft · ohne Marke: Festlegung (Entscheidung) dieser Spezifikation. |
| Crate-lokale Dokumente | 39 Crates tragen eine eigene `SPEC.md` (plus `PLAN.md` bei einem Teil). Sie enthalten Binnendetails und sind für ihr Crate verbindlich, soweit sie diesem Dokument nicht widersprechen. |

### 0.2 Rangfolge

1. Dieses Dokument: Crate-Grenzen, Abhängigkeitsrichtung, `ids`/`env`, Brücken, Fehlerabbildung, Features, Reihenfolge, Gates, Score.
2. Crate-lokale `SPEC.md`/`PLAN.md`: Binnenzerlegung und Dateizuordnung je Crate. Bei Widerspruch gilt dieses Dokument, der Widerspruch ist als Befund in §0.4 oder im Crate-Dokument zu vermerken.
3. `AGENTS.md` und `docs/**` sind nachrangig und werden in S-03 angeglichen.

### 0.3 Normative Sprache und Prüfbarkeit

MUSS / DARF NICHT / SOLL nach RFC 2119. **Jede MUSS-Aussage hat ein benanntes Prüfverfahren** (Gate nach §8 oder Test). Eine Aussage ohne Prüfverfahren gilt nur als SOLL, bis das Verfahren existiert.

### 0.4 Änderungsprotokoll v2 → v3

| ID | Befund in v2 | Änderung in v3 | Beleg |
| --- | --- | --- | --- |
| K1 | Stand `ef3364f`, alle Schritte offen | Ist-Stand je Schritt (§2.2), Reihenfolge auf den Rest verkürzt (§6) | \[V\] `ids`, `env`, `prompt-guard`, `vault` existieren; `types` und `ports` re-exportieren |
| K2 | E1 = „Kern-Closure ⊆ E0“, aber `store` und `vector` hängen am E1-Crate `mvcc` | E1 wird in **E1a/E1b** geteilt (§1.1) | Widerspruch v2 §1.1 ↔ §3.1 |
| K3 | INV-1 „kein Byte ändert sich“ ↔ §5.2 hebt Formatversionen an ↔ §9.3 | INV-1 mit enger, benannter Ausnahme (§1.2) | Widerspruch v2 |
| K4 | `workspace` „nie Default“ (§3.1), aber Defaults enthalten es | Regel R10, Verstöße einzeln benannt (§2.4 B19) | \[V\] `license`, `audit-export` |
| K5 | D-06: Beweisdateien in `crypto` bleiben bis Nachweis | Überholt: bereits nach `archive/contextra-crypto-proofs` verschoben. Nachprüfung als Schritt S-70a (§6) | \[V\] `crypto/src` ohne `deletion_proof`, `ed25519_proof` |
| K6 | B11: Loom-Modell für `mvcc` fehlt möglicherweise | Vorbedingung von S-30 wird zur Verifikation | \[V\] `loom_sequence_log.rs`, `loom_snapshot_registry.rs`, `loom_tx_buffer.rs` |
| K7 | §3.1 lässt `db`, `engine`, `mcp` ohne explizite Liste | Vollständige `allowed_internal`-Listen inkl. optionaler Closures (§3.1) | \[V\] Manifeste |
| K8 | V-VETO verweist auf Test `partial_rebuild_veto_f02_guard` | Test existiert nicht \[V\]; Anlegen wird Voraussetzung von S-36 | `crates/contextra-vector/SPEC.md:58` |
| K9 | Gates nur als Befehle beschrieben | Gate → Werkzeug → Exit-Code → Status (§8); fehlende Werkzeuge als Arbeitspakete | \[V\] kein Golden-Korpus, kein Doc-Lint, kein G-F2-Job |
| K10 | Score nicht definiert | Modularitäts-Score M normativ (§8.6) | Anforderung „Score je Crate“ |
| K11 | ID-Breite: „Feld je Format“ ohne Klärung, dass die Breite Compile-Zeit-Eigenschaft ist | §5.2 präzisiert (Matrix, Fehlerfälle, Reihenfolge) | \[V\] `checkpoint` führt `id_width_bits` bereits |
| K12 | „Nachweis“ je Schritt uneinheitlich | Einheitliche Definition of Done (§6.1) | — |
| K13 | `env` mit `test-utils` im Default | Default nur `std` (R10) | \[V\] `env` Default `std, test-utils` |
| K14 | Tools verlangen `rg`, `jq`, `cargo-hack` ohne Voraussetzungsliste | Werkzeugliste in S-00 (§6) | \[V\] in der Analyseumgebung nicht vorhanden |

---

## 1. Begriffe und Invarianten

### 1.1 Begriffe

- **Kern** eines Crates: sein Code mit `--no-default-features`, ohne Feature `workspace`.
- **Brücke:** Modul `src/compat/` hinter dem Feature `workspace`. Nur dort stehen `contextra_types`, `contextra_ports`, alle Implementierungen von `ports`-Traits und alle `From`-Konvertierungen nach `ContextraError`.
- **Kern-Closure:** transitive Menge der internen `contextra-*`-Crates in `cargo tree -p <crate> -e normal --no-default-features`. **Feature-Closure:** wie oben je einzeln aktiviertem Feature.
- **Mikro-Kerne:** `contextra-ids`, `contextra-env`. Beide sind E0 und für jeden E1-, E2- und E3-Baustein als Kern-Abhängigkeit erlaubt. E0-Crates behalten lokale Mini-Traits (R7).
- **Ebenen** (gemessen an der Kern-Closure):

| Ebene | Definition | Crates (Ziel) |
| --- | --- | --- |
| **E0** | Kern-Closure leer | `ids`, `env`, `sys`, `durable-fs`, `simd`, `wire`, `adapt`, `ingest`, `privacy`, `sandbox`, `prompt-guard`, `infer-onnx`, `erasure`, `license`, `audit-export`, `avv-generator` |
| **E1a** | Kern-Closure ⊆ E0, höchstens 4 Mitglieder | `mvcc`, `crypto`, `text`, `graph`, `rank`, `checkpoint`, `kvcache`, `vault`, `infer-ollama`, `infer-candle` |
| **E1b** | Kern-Closure ⊆ E0 ∪ E1a, mindestens ein E1a-Mitglied, höchstens 4 Mitglieder (Ausnahme `store`: 5) | `store`, `vector` |
| **E2** | Integration; darf E0, E1, E2 nutzen, nie E3 | `types`, `ports`, `router`, `agent`, `cognition`, `engine`, `db`, `contextra` |
| **E3** | Anwendungen und Testwerkzeuge | `mcp`, `py`, `testkit`, `bench` |

*Begründung K2:* v2 definierte E1 als „Kern-Closure ⊆ E0“, ordnete aber `store` (→ `mvcc`, `crypto`) und `vector` (→ `mvcc`) selbst E1 zu. Die Teilung hebt den Widerspruch auf, ohne die Abhängigkeitsrichtung zu ändern.

- **Entnehmbar:** ein Crate gilt nur als entnehmbar, wenn die Gates G-L2 → G-L1 → G-L3 → G-L4 in dieser Reihenfolge grün sind (§8.5).

### 1.2 Invarianten (für jeden Schritt)

| ID | Invariante | Prüfverfahren |
| --- | --- | --- |
| INV-1 | **Formatgleichheit.** Kein persistiertes Byte ändert sich. **Einzige Ausnahme:** die Versionsanhebungen mit Feld `id_width_bits` nach §5.2, und zwar nur in den dort benannten Schritten. Jede Ausnahme erzeugt Golden-Dateien für v(alt) und v(neu); der Lesepfad für v(alt) bleibt. | G-F1 |
| INV-2 | **Pfadgleichheit.** Öffentliche Importpfade bleiben mit `#[deprecated(since, note)]`-Re-Export gültig. Entfernung nur in S-70. | G-A1 |
| INV-3 | **Ein Schritt, ein Commit-Paar.** Commit A verschiebt oder benennt um, Commit B baut um. Beide einzeln revertierbar. | Review, Commit-Nachrichten `S-xx A/B` |
| INV-4 | **Build-Pfad bleibt grün:** `cargo check --workspace --all-targets --locked` nach jedem Commit. | G-L3 |
| INV-5 | **Determinismus.** Gleiche Eingabe ⇒ gleiche Rangfolge, Tie-Break nach ID aufsteigend. | `determinism_guard.py`, G-D1 |
| INV-6 | **Durability-Pfade unberührt** (WAL, Manifest, Flush, Recovery), sofern der Schritt sie nicht ausdrücklich nennt. | Crash-/Recovery-Suite, G-F1 |
| INV-7 | **Keine Compliance-Aussagen** in Code und Doku (`AGENTS.md` §1.2). | G-DOC1 |
| INV-8 | **Kein Rückwärtspfeil:** E3→E2→E1b→E1a→E0, nie umgekehrt. | G-L1 |
| INV-9 | **Reproduzierbarkeit.** Toolchain `1.89.0` (`rust-toolchain.toml` \[V\]), `--locked`, Werkzeugliste aus S-00. | CI |
| INV-10 | **Keine stille Gate-Abschwächung.** Änderungen an `governance/gates.toml`, `ratchet.toml`, `protected-paths.toml` nur mit ADR und Label `protected-path`; Ratchet-Werte dürfen nur sinken. | `gate-weakening`, `ratchet` |

---

## 2. Ist-Stand am HEAD `5408826f8`

### 2.1 Kennzahlen \[V\]/\[T\]

| Größe | Wert |
| --- | --- |
| Crates in `crates/` | 39 (plus `benchmarks/contextra-bench`) |
| Interne Kanten | 117: 96 normal (davon 35 optional), 21 dev |
| Fan-in `types` / `ports` | 26 / 21 |
| Crates mit Inspector-Score 100 | 9 (`adapt`, `durable-fs`, `env`, `ids`, `privacy`, `prompt-guard`, `sandbox`, `sys`, `wire`) |
| Crates ohne ungegatete Brückenreferenz | 20 (Details §8.6, Anhang A) |
| Unsafe-Inseln | nur `simd` (269 Stellen), `wire` (45), `sys` (22) |
| `publish = false` | 1 von 39 Manifesten |

### 2.2 Schrittstatus

Status: **erledigt** · **teilweise** · **offen**. „Rest“ nennt die noch fehlende Arbeit.

| Schritt | Status | Beleg \[V\] | Rest |
| --- | --- | --- | --- |
| S-00 Baseline | teilweise | `docs/decoupling/baseline/` mit Unterordnern je Crate | Neu erheben mit `cargo` (nicht verifiziert); Werkzeugliste fehlt |
| S-01 Gate-Reparatur | teilweise | `xtask`-Alias entfernt; `tests/arch` entfernt; `gate_layering.py` und Negativtest vorhanden | `governance/protected-paths.toml` enthält noch zwei `xtask/**`-Globs; `justfile` kommentiert `xtask`; `ratchet.toml › not_run_baseline` listet 30+ Gates (u. a. `ring-dag`, `error-domain`, `capabilities-consistency`, `unsafe-inventory`, `doc-truth`) |
| S-02 Golden-Korpus | teilweise | `mvcc/tests/golden_corpus.rs`, `mvcc/tests/golden/` | Kein `tests/golden/` im Wurzelverzeichnis, kein `tools/golden_regen.sh`, keine `MANIFEST.sha256` |
| S-03 Doku/Schema | teilweise | `capabilities.toml` Schema v3, `AGENTS.md` nennt Ringe dekommissioniert | Verwenderliste in `types/lib.rs`, falsche Docstrings (`privacy`, `cognition`, `kvcache`, `avv-generator`, `sys/lib.rs:19`) einzeln prüfen |
| S-10 `ids` | weitgehend erledigt | `ids` mit `IdError`, `ID_WIDTH_BITS`, `forbid(unsafe_code)`; `types/{ids,refs,tenant_scope}.rs` re-exportieren; `docid-128` in `types` weitergeleitet | Commit C: 86 Aufrufstellen von `::from_key(` außerhalb von Tests, davon exakt klassifizieren (mechanisch / nicht-mechanisch) |
| S-11 `env` | erledigt | `ports/clock.rs` re-exportiert `contextra_env::clock::*`; `env` mit Feature `test-utils` | `env` hat `test-utils` im Default (K13) |
| S-12 Feature-Hygiene | teilweise | `adapt` Default `std`; `crypto` ohne `legacy-erasure` im Default | Verstöße gegen R10: siehe B19 |
| S-20 `license`, `audit-export` | teilweise | beide mit `compat/` | Defaults enthalten `workspace` (B19) |
| S-21 `prompt-guard` | erledigt | `mcp` hängt an `contextra-prompt-guard` mit `file-audit`; `mcp/src/prompt_injection.rs` ist Wrapper | — |
| S-22 `vault` | teilweise | Crate `vault` existiert, `db` hängt daran | `db/src` enthält weiter `volatile_vault.rs`, `vault_compat.inc`, `vault_sys.inc`: Quelle entfernen, Re-Export prüfen |
| S-23 `adapt` | teilweise | je 1 `Clock`/`Rng`-Trait; `sketched-bandit` entfernt | Features `flow-corrected-thompson` und `rie-greedy-personalization` auf `exp-flow-thompson`/`exp-rie-greedy` umbenennen; `offpolicy.rs` gegen v2-Datei `off_policy.rs` prüfen |
| S-24 `erasure` | offen | `erasure/Cargo.toml` ohne `[features]` | Features `signing`, `vault`, `async` einführen |
| S-30 `mvcc` | teilweise | `mvcc/src/tombstone.rs`, `tests/tombstone_diff.rs`, `mvcc_error.rs`, `compat/error.rs`, drei Loom-Tests | Commit B: `types`/`ports` weiter nicht optional; `types/src/tombstone.rs` besteht noch; ungegatete Referenzen: 12 in 6 Dateien |
| S-31 `crypto` | offen | `kv-encryption` im Default | `types`/`ports` nicht optional (34 Referenzen in 7 Dateien); `default = []`; `KeyManager`-Signaturen auf `[u8; 32]` |
| S-32 `text` | teilweise | `compat/`, Feature `workspace` | 39 Referenzen in 6 Dateien (u. a. `wand.rs` 20); Default enthält `types-compat` |
| S-33 `rank` | teilweise | `rank/src/temporal.rs` vorhanden | `engine/src/temporal_filter.rs` noch als eigene Kopie; Differenztest dokumentieren |
| S-34 `graph` | teilweise | `compat/` | 109 Referenzen in 52 Dateien; Default `types-compat`, `ports-compat`, `experimental-graph`; `PprConfig` weiter in `types` |
| S-35 `checkpoint` | weitgehend | `id_width_bits` in `meta.rs`, `manifest.rs`, `store.rs` | 4 Referenzen in 2 Dateien |
| S-36 `vector` | teilweise | `compat/`, Default `std, ports-compat` | 150 Referenzen in 35 Dateien; V-VETO-Test fehlt (K8) |
| S-37 `store` | teilweise | `kv/` nach `archive/contextra-store-kv`; `compat/` | 233 Referenzen in 65 Dateien; `CURRENT_MANIFEST_VERSION = 1`; `FeatureRing` noch vorhanden |
| S-38 KV-Umzug | offen | `kv_cipher`, `kv_segment/*` weiter in `crypto` | Paar-Umzug nach `kvcache`; Konstante `CURRENT_KV_KEY_DERIVATION_VERSION` noch in `crypto` |
| S-39 `infer-*` | teilweise | `infer-ollama` ohne ungegatete Referenz | `infer-candle`: 46 Referenzen in 14 Dateien, Kern hängt an `types`, `ports`, `rank` |
| S-50 `ports` abgeben | offen | `ports/src` enthält `erasure*`, `gate`, `ledger`, `license`, `policy`, `keys`, `barrier`, `conformance`, `contract_tests` | Verschieben; `FeatureRing` in 6 Crates (`contextra`, `engine`, `kvcache`, `license`, `ports`, `store`) |
| S-51 `types` abgeben | offen | `types/src/erasure.rs`, `PprConfig` in `types/types/domain/misc.rs` | Verschieben nach `erasure` und `graph` |
| S-60 Regler | teilweise | `adapt` hat `decay_controller`, `homeostat`, `pid_latency_controller` | Kopien in `engine` und `db` ersetzen nach Differenztest |
| S-61 Chunker | offen | `engine/src/chunker/` | Paritätstest, Adapter über `ingest::chunking` |
| S-62 `router` | teilweise | `wire` optional hinter `subprocess-dispatch` | `ports_local.rs` entfernen nach `mcp`-Umstellung |
| S-63 bis S-81 | offen | `contextra-vector-diskann` und `contextra-graph-research` existieren nicht | wie v2, verkürzt in §6 |

### 2.3 Messwerte Modularität (Auszug)

Vollständige Tabelle in Anhang A. Spalte „M“ ist der Score nach §8.6.

| Crate | Ebene | M | Ungegatete Refs (Dateien) | Kern hängt an E2 |
| --- | --- | --- | --- | --- |
| `infer-candle` | E1a | 15 | 46 (14) | ja (`types`, `ports`, `rank`) |
| `store` | E1b | 15 | 233 (65) | ja (`types`, `ports`) |
| `crypto` | E1a | 28 | 34 (7) | ja |
| `mvcc` | E1a | 34 | 12 (6) | ja |
| `vector` | E1b | 52 | 150 (35) | nein (optional) |
| `graph` | E1a | 54 | 109 (52) | nein (optional) |
| `text` | E1a | 61 | 39 (6) | nein (optional) |
| `kvcache` | E1a | 69 | 11 (5) | nein (optional) |
| `checkpoint` | E1a | 75 | 4 (2) | nein (optional) |

### 2.4 Neue Befunde (ergänzen B01–B16 aus v2)

**B17 — Aufwärtskanten E1→E2 (INV-8) \[V\].** `mvcc`, `crypto`, `store` und `infer-candle` führen `contextra-types` und `contextra-ports` als nicht-optionale Abhängigkeit. Weil `store` auf `mvcc` und `crypto` aufbaut und `vector` auf `mvcc`, ist die Kette `store → mvcc → types/ports` die Haupt-Blockade der Entnahme.

**B18 — Optional im Manifest, ungegatet im Quelltext \[V\]/\[A\].** Bei `vector`, `graph`, `text`, `kvcache`, `checkpoint` sind `types`/`ports` optional, der Kern referenziert sie aber in vielen Dateien ohne `cfg`-Gate. Hypothese: `cargo check -p <crate> --no-default-features` scheitert dort. Bei `graph`, `kvcache`, `rank`, `text`, `vector` ist das Kompatibilitäts-Feature sogar Default, sodass der isolierte Kern nie gebaut wird.

**B19 — Verstöße gegen die Default-Regel \[V\].** `license` und `audit-export`: `workspace` im Default; `audit-export` zusätzlich `test-utils`; `env`: `test-utils`; `testkit`: `erasure-conformance`; `graph`: `experimental-graph`; `rank`: `exp-dibud`, `exp-resonance`, `experimental-rank`; `kvcache`: `ports-compat`, `crypto-compat`; `crypto`: `kv-encryption` (Ziel: leer).

**B20 — Seam-Duplikate bestehen \[V\].** `trait Clock` in 10 Crates, `trait Rng` in 5, `IdGen` in 2, `MetricsSink` in 2; `struct DocId` in 7 Crates; `DistanceMetric` in 3. Die Kopien sind nach B02 nicht kompatibel (ms/ns, `&mut self`/`&self`).

**B21 — Gate-Lücken \[V\].** Kein Golden-Korpus im Wurzelverzeichnis, kein Doc-Lint für G-DOC1, kein CI-Job für `contextra.fbs.sha256` (die Datei existiert in `wire/schemas/`), kein G-L2-Skript (nur Zählung per `count_bridge_refs.py`), `gate_layering.py` bricht ohne `cargo` ab. 30+ Gates stehen in `not_run_baseline`.

**B22 — V-VETO-Test fehlt \[V\].** `partial_rebuild_veto_f02_guard` kommt nur in `crates/contextra-vector/SPEC.md` vor.

**B23 — D-06 überholt \[V\].** Die Beweisdateien (`deletion_proof`, `ed25519_proof`, `audit_chain` teilweise) liegen nicht mehr in `crypto/src` bzw. `archive/contextra-crypto-proofs` existiert. Die in v2 geforderten Nachweise (Item-Diff und Mutation-Score) sind nicht belegt und werden nachgeholt (S-70a).

**B24 — `erasure` ohne Features \[V\].** `erasure/Cargo.toml` hat keine `[features]`-Tabelle; die Trennung `signing`/`vault`/`async` aus v2 ist offen.

---

## 3. Zielzustand

### 3.1 Crate-Tabelle (normativ, ersetzt v2 §3.1)

„Kern“ = Kern-Closure (direkte und transitive interne Abhängigkeiten bei `--no-default-features`). „Feature-Closures“ nennt nur Features mit zusätzlichen internen Crates; `workspace` zieht immer `types` und `ports` (bei E2/E3 entfällt das Feature). Die Spalte „Default“ gilt nach R10. Tier-Werte stehen in `capabilities.toml` (Single Source of Truth); v3 ändert nur: `durable-fs` = experimental, `erasure`, `license`, `audit-export`, `avv-generator`, `sandbox`, `infer-onnx`, `infer-candle` = frozen (AGENTS.md §2.3, D-08).

| Crate | Ebene | Kern (Ziel) | Feature-Closures | Default (Ziel) | Eigener Fehler |
| --- | --- | --- | --- | --- | --- |
| `ids` | E0 | — | `docid-128` | — | `IdError` |
| `env` | E0 | — | `test-utils` | `std` | — |
| `sys` | E0 | — | — | — | `SysError` |
| `durable-fs` | E0 | — | — | `std`, `crc` | besteht |
| `simd` | E0 | — | `workspace`→types | `std` | `SimdError` |
| `wire` | E0 | — | — | `std` | besteht |
| `adapt` | E0 | — | `bandit-routing`, `exp-flow-thompson`, `exp-rie-greedy` | `std` | besteht |
| `ingest` | E0 | — | `workspace`→ids, types | `fmt-text` | besteht |
| `privacy` | E0 | — | — | `std` | besteht |
| `sandbox` | E0 | — | — | — | besteht |
| `prompt-guard` | E0 | — | `file-audit` | `std` | `GuardError` |
| `infer-onnx` | E0 | — | `onnx`, `download`, `test-utils`, `workspace`→ports, types | — | besteht |
| `erasure` | E0 | — | `signing`, `vault`, `async`, `conformance` | `signing`, `vault`, `async` | besteht |
| `license` | E0 | — | `workspace`→types, ports | `std` | besteht |
| `audit-export` | E0 | — | `workspace`→types, `test-utils` | `std` | besteht |
| `avv-generator` | E0 | — | — | `std` | besteht |
| `mvcc` | E1a | ids, env | `workspace`→types, ports | — | `MvccError` |
| `crypto` | E1a | ids, env | `workspace`; `legacy-erasure` (nicht Default) | leer | `CryptoError` |
| `text` | E1a | ids, env | `workspace` | `std` | `TextError` |
| `graph` | E1a | ids, env | `workspace`; `exp-hyperedge` | leer | `GraphError` |
| `rank` | E1a | ids, env | `workspace`; `exp-dibud`, `exp-resonance` | `std` | `RankError` |
| `checkpoint` | E1a | ids, env | `workspace` | — | `CheckpointError` |
| `kvcache` | E1a | env | `kv-encryption`→crypto; `kv-shredding`; `exp-kivi`; `workspace` | `std` | `KvCacheError` |
| `vault` | E1a | sys | `workspace` | — | `VaultError` |
| `infer-ollama` | E1a | env | `workspace` | — | `OllamaError` |
| `infer-candle` | E1a | env | `candle`; `kv-bridge`→kvcache; `workspace` | — | `CandleError` |
| `store` | E1b | ids, env, sys, mvcc, crypto | `workspace` | — | `StoreError` |
| `vector` | E1b | ids, env, mvcc | `mmap`→sys; `simd`→simd; `exp-*`; `workspace` | `std` | `VectorError` |
| `types` | E2 | ids | — | — | `ContextraError` |
| `ports` | E2 | types, ids, env | — | — | — |
| `router` | E2 | types, ports, adapt | `subprocess-dispatch`→wire | — | besteht |
| `agent` | E2 | types, ports | `routing`→router | — | `AgentError` |
| `cognition` | E2 | types, ports | `exp-hyperedge`→graph | — | `CognitionError` |
| `engine` | E2 | ids, env, types, ports, mvcc, store, vector, text, graph, rank, checkpoint | `adaptive-*`→adapt; `onnx`→infer-onnx; `sandbox`→sandbox; `license`→license; `exp-extraction` | — | `ContextraError` |
| `db` | E2 | engine, types, ports, vault | `cognition`→cognition; `experimental-diskann`→vector-diskann (nach S-80) | — | — |
| `contextra` | E2 | db, types, ports | `license`→license; `infer-candle` | `embedded` | — |
| `mcp` | E3 | contextra, types, ports, wire, rank, crypto, privacy, router, prompt-guard | — | — | `McpError` |
| `py` | E3 | types, contextra, wire | `docid-128` | — | — |
| `testkit` | E3 | ports, types, env | — | `testkit` | — |
| `bench` | E3 | store, vector, text, db | — | — | — |

**Anmerkungen**

- `store` ist die einzige Ausnahme vom Budget „höchstens 4 Mitglieder“ (5: ids, env, sys, mvcc, crypto). Das Budget zählt nur die Kern-Closure.
- `kvcache` ist E1a, weil `crypto` nur hinter `kv-encryption` in der Closure steht. Aktiviert `kvcache` dieses Feature, bleibt es E1a, da `crypto` selbst E1a ist.
- `engine`: v2 sagte nur „alle benötigten“. v3 legt die Liste fest (11 Kern-Mitglieder inklusive Mikro-Kerne; `AGENTS.md` nennt „≤ 9“ ohne `ids`/`env`, bleibt damit gültig). `sandbox`, `infer-onnx` und `adapt` sind im Manifest bereits optional \[V\] und bleiben Features.
- `db` braucht `vault` als Kern-Abhängigkeit, weil `db` den Typ re-exportiert (v2 §6.3 forderte das, §3.1 führte es nicht auf).
- `ids` hat laut `capabilities.toml` nur das Feature `docid-128`; ein `std`-Feature ist nicht nötig (D-19: no\_std ist kein Ziel).

### 3.2 Regeln

| Regel | Inhalt | Prüfverfahren |
| --- | --- | --- |
| **R1 Richtung** | Abhängigkeiten zeigen nach unten (INV-8). | G-L1 |
| **R2 Brücke** | `contextra_types`, `contextra_ports`, `contextra_testkit` DEFINIEREN NICHT im Kern vor, auch nicht in Doc-Kommentaren und `#[cfg(test)]`-Modulen. Tests des Kerns nutzen `env` (`test-utils`) und lokale Fakes. Ausnahme: `tests/` hinter `required-features = ["workspace"]`. | G-L2 |
| **R3 Eigene Fehler** | `thiserror`, `#[non_exhaustive]`. `impl From<LocalError> for ContextraError` nur in `compat/`. | G-E1 |
| **R4 Ein Besitzer je Konzept** | IDs → `ids`; `Clock`/`Rng`/`IdGen`/`MetricsSink` → `env`; `Tombstone*` → `mvcc`; `PprConfig`/`PprAlgorithm`/`TlHfdParams` → `graph`; KV-Verschlüsselung → `kvcache`; Lizenz-Traits → `license`; Erasure-Familie → `erasure`. **Ausnahme:** gespiegelte, kleine, geschlossene Wert-Enums (≤ 8 Unit-Varianten) wie `DistanceMetric`, wenn Namen und Reihenfolge identisch sind und ein Konformitätstest die serde-Namen gleichsetzt. | Symbol-Eindeutigkeit (`symbol-uniqueness`) |
| **R5 Experimente** | X1–X8 (§3.3). | `feature_audit.py`, G-L1 |
| **R6 Nachweis statt Behauptung** | „Entnehmbar“ gilt nur nach §8.5. | Gates |
| **R7 E0-Seams lokal** | E0-Crates definieren Umwelt-Traits lokal und minimal; die Verbindung zu `env`/`ports` liegt nur in `compat/`. | G-L2 |
| **R8 Keine Panics** in neuen Kern-Pfaden von `ids`/`env` (außer Tests). | `unwrap-gate` |  |
| **R9 `pub use`-Übergang** | Verlässt ein Typ sein Crate, bleibt ein `#[deprecated]`-Re-Export, außer er würde einen Rückwärtspfeil erzeugen (dann ersatzlose Entfernung, wenn nur Besitzer und Ursprung ihn nutzen). | G-A1 |
| **R10 Feature-Regel (neu)** | `default` enthält NIE `workspace`, `test-utils`, `*-compat`, `exp-*`, `experimental-*`. `default` darf nur `std` und Eigenschaften enthalten, die für den Zweck des Crates wesensnotwendig sind (`crc`, `fmt-text`, `embedded`, `testkit`). Namensschema für Experimente: `exp-<name>`, Sammelfeature `experimental-all`. Ein Feature ohne `cfg(feature = "x")` im Quelltext und ohne Weiterleitung in einem anderen Manifest wird entfernt. | `feature_audit.py` (entrauscht, §8.4), G-L1 |
| **R11 Fan-in-Ratchet (neu)** | Kein neuer Verwender von `types`/`ports` aus E0 oder E1. `ratchet.toml` führt `types_fan_in` und `ports_fan_in` (Start 26 und 21) und lässt nur sinkende Werte zu. | `ratchet` |

### 3.3 Experimente (X1–X8)

- **X1** Experiment-Code hängt nur von Stabilem ab; der Host-Kern kennt keine Experimente.
- **X2** Feature `exp-<n>`, nie in `default`, Sammelfeature `experimental-all`, in der CI-Matrix.
- **X3** Die Naht (Trait) liegt im Host-Kern in `ext.rs`, ohne `types`/`ports`.
- **X4** Eigenes Crate bei ≥ 2.000 Zeilen **oder** eigenen externen Abhängigkeiten **oder** ≥ 2 Hosts.
- **X5** Verhaltens-Baseline im selben Crate (z. B. Recall gegen Brute-Force).
- **X6** Persistierte Experimentdaten haben eigene Magic-/Versionskennung; das Kernformat ändert sich nie.
- **X7** Tier `experimental` in `capabilities.toml`.
- **X8** Ein schreibendes Experiment behält den Lesepfad oder dokumentiert eine Migrationssperre.

---

## 4. Spezifikation der Kerne und Brücken

### 4.1 `contextra-ids` (Ist: umgesetzt)

**Zweck:** Identitäts- und Mandantentypen ohne interne Abhängigkeit. **Ist \[V\]:** Abhängigkeiten `blake3`, `serde`, `thiserror`; `forbid(unsafe_code)`; `IdError`; `ID_WIDTH_BITS`; Feature `docid-128`; `types/{ids,refs,tenant_scope}.rs` sind Re-Exports.

**Vertrag (bindend):**

- `DocId` ist `u64` (`repr(transparent)`) oder mit `docid-128` `u128` (`repr(C, align(16))`). Das Feature gilt für das **gesamte** Build und ist nicht additiv (B03).
- `ID_WIDTH_BITS: u32 = 64 oder 128`; Compile-Time-Prüfung von `size_of`/`align_of` (8/8 bzw. 16/16).
- `DocId::from_key(&str) -> Result<Self, IdError>`: BLAKE3-Präfix, erste 8 bzw. 16 Bytes, Little Endian. Display-Text und Variante von `IdError` sind bitgleich zu den früheren `InvalidInput`-Texten.
- `types` enthält `impl From<IdError> for ContextraError` (Variante `InvalidInput(String)`).

**Offen:** 86 Aufrufstellen von `::from_key(` außerhalb von Tests, davon 0 in `ids`. Jede ist als *mechanisch* (`?` über `From`) oder *nicht-mechanisch* (Musterabgleich auf `ContextraError::InvalidInput`) zu klassifizieren; mehr als 20 nicht-mechanische Stellen lösen das Rückfall-Verfahren aus §9.2 aus.

**Abnahme:** `ids_tests` grün in `ids`; Differenztest gegen die verbliebenen `DocId`-Kopien (`erasure`, `graph`, `ingest`, `rank`, `text`, `vector`) über 10.000 zufällige Schlüssel plus Randfälle (leer, Unicode, 4 KiB): gleiche `from_key`-Bytes, gleiche serde-Ausgabe (JSON und `bincode`); Golden-Vektoren in 64 **und** 128 Bit.

### 4.2 `contextra-env` (Ist: umgesetzt)

**Ist \[V\]:** `ports/clock.rs` re-exportiert `contextra_env::clock::*`; `env` hat Feature `test-utils`.

**Vertrag (bindend):**

```rust
pub trait Clock: Send + Sync + 'static {
    fn now_unix_nanos(&self) -> u64;   // 0 vor der Epoche, nie Panic
    fn monotonic_nanos(&self) -> u64;
}
pub fn unix_ms(c: &dyn Clock) -> u64 { c.now_unix_nanos() / 1_000_000 }   // Abrunden

pub trait Rng: Send + Sync + 'static {
    fn next_u64(&self) -> u64;
    fn fill_bytes(&self, dest: &mut [u8]);
    fn next_unit_f64(&self) -> f64;
}
```

- `SeededRng` (SplitMix64): Folge bitgleich zu den Golden-Folgen (Seeds 0, 1, 42, `u64::MAX`; je 1.024 Werte).
- `IdGen`, `SequentialIdGen`, `MetricsSink`, `NoopMetricsSink`, `InMemoryMetricsSink`, `MetricKey` wie in der früheren `ports`-Fassung.
- Feature `test-utils` enthält `ManualClock` und die Konformitätssuite für Clock, Rng und Metrics.

**Abweichung zu beheben (K13):** `default = ["std"]`; `test-utils` wird nur von Verbrauchern aktiviert, die es brauchen. Abnahme: `cargo tree -p contextra-env -e features` zeigt `test-utils` nicht im Default; `unix_ms`-Differenztest gegen die `now_unix_ms`-Kopien in `text`, `graph`, `vector/contract.rs` mit `ManualClock`.

### 4.3 Brückenmuster (verbindlich)

**a) `Clock` E0-lokal ⇐ `env`:**

```rust
// crates/contextra-<x>/src/compat/env.rs  (feature = "workspace")
impl<T: contextra_env::Clock + ?Sized> crate::Clock for T {
    fn now_unix_nanos(&self) -> u64 { contextra_env::Clock::now_unix_nanos(self) }
    fn monotonic_nanos(&self) -> u64 { contextra_env::Clock::monotonic_nanos(self) }
}
```

Voraussetzung: das lokale Trait ist lokal (Orphan-Regel erfüllt) und es gibt keine konkrete `impl crate::Clock for T`, die mit dem Blanket überlappt (E0119). Konkrete lokale Typen stehen hinter `cfg(not(feature = "workspace"))` oder als Newtype.

**b) `Rng` mit `&mut self` ⇐ `env::Rng` (`&self`):** Blanket-Impl lokal-von-`env`. Die Gegenrichtung ist verboten; E1-Kerne nutzen ausschließlich `env::Rng`.

**c) Fehler:** `impl From<VectorError> for contextra_types::ContextraError` nur in `compat/error.rs`; Variantenabbildung nach §5.3.

**d) Port-Implementierung:** `impl contextra_ports::VectorIndex for HnswIndex` nur in `compat/`; Kern-Trait-Methoden enthalten kein `ContextraError`.

**e) Konfliktregel (neu):** Zwei Brücken im selben Crate dürfen nicht dasselbe lokale Trait für denselben Blanket implementieren. Gibt es zwei Quellen (`env` und `ports`), implementiert nur `env` den Blanket; `ports`-Typen erreichen ihn, weil `ports` die `env`-Traits re-exportiert.

---

## 5. Persistenz-, Fehler- und Verhaltensvertrag

### 5.1 Golden-Format-Korpus (S-02)

**Ort und Form (festgelegt):** Wurzel-Verzeichnis `tests/golden/<format>/` mit `MANIFEST.sha256`. Crate-lokale Korpora (wie `mvcc/tests/golden/`) bleiben gültig und werden im Wurzel-Manifest referenziert. Je Eintrag: Erzeugertest (`#[ignore]`) über `tools/golden_regen.sh` und Prüftest (läuft immer). Bei Abweichung bricht der Schritt ab; eine absichtliche Formatänderung ist nie Teil eines Umbauschritts.

| Format | Quelle / Kennung | Gate-Schritt |
| --- | --- | --- |
| Manifest | `store/manifest/core.rs`: `MFMN`, Version 1 (`CURRENT_MANIFEST_VERSION = 1`) \[V\] | S-37 |
| SSTable | `store/sstable`: `MFSX`, Legacy `MFST` (Footer, Blöcke, Bloom) | S-37 |
| WAL | `store/wal/*` inkl. HMAC-Kette (`wal_crypto`) | S-31, S-37 |
| Rollback-Intent | `store/lsm/recovery.rs`: `MFRLBK\0\0` | S-37 |
| HNSW | `vector/persistence/header.rs`: `0x484E5357`, `HNSW_VERSION = 2` \[V\] | S-36 |
| DiskANN | `vector/diskann/format.rs`: `DANN` v1, `DANF`, `PWAL` v1, `TWAL` v1 | S-36, S-80 |
| KDF-Header | `crypto/kdf.rs`: `MFKD` | S-31 |
| KV-Segment | `crypto/kv_cipher.rs`: Formatversion 2; Schlüsselableitung Version 1 \[V\] | S-38 |
| Text-Postings, Graph (bincode), Checkpoint Meta/Manifest | je Crate zu erheben | S-32, S-34, S-35 |
| MVCC | `mvcc/tests/golden_corpus.rs` \[V\] | S-30 |

### 5.2 ID-Breite (B03, präzisiert)

Die Breite ist eine **Compile-Zeit-Eigenschaft** des gesamten Builds. Jedes persistierte Format mit IDs trägt `id_width_bits: u16` (64 oder 128) im Kopf und prüft es beim Öffnen. `checkpoint` führt das Feld bereits \[V\].

| Build-Breite | Datei | Verhalten |
| --- | --- | --- |
| 64 | 64 | Öffnen |
| 128 | 128 | Öffnen |
| 64 | 128 oder 128 | 64 |
| beliebig | Formatversion ohne Feld | nur mit `OpenOptions::assume_legacy_id_width(bits)`, sonst `…::IdWidthUnknown` |

**Prüfregel:** Nach abgelehntem Öffnen ist jede betroffene Datei byte-identisch zum Zustand davor (Hash-Vergleich im Test).

**Versionsanhebungen (die einzige INV-1-Ausnahme):** `store` Manifest v1 → v2 (S-37); `vector` `HNSW_VERSION` 2 → 3 (S-36); `text`, `graph` nach Lesen des jeweiligen Headers dasselbe Muster (S-32, S-34); `checkpoint` nur Prüfung (S-35). Reihenfolge je Format: erst Leser (v alt und v neu), danach Schreiber; Rückwärts-Lesetest mit Golden-Dateien v alt.

**CI:** `cargo hack check --workspace --features docid-128` zusätzlich zum Default. E0-Crates ohne Abhängigkeit auf `ids` (`ingest`, `erasure`) erhalten kein `docid-128`; Konvertierung von `ids::DocId` ist dort `TryFrom` (schlägt bei Überlauf fehl), nie `as`.

### 5.3 Fehlerabbildung

Je Baustein eine Tabelle `docs/decoupling/errors/<crate>.md` (lokale Variante → `ContextraError`-Variante → Display-String). Für mindestens zehn Crates \[V\] (u. a. `agent`, `checkpoint`, `cognition`, `crypto`, `graph`, `kvcache`, `mvcc`, `prompt-guard`, `rank`) liegen sie vor; `store`, `vector`, `text`, `ids` sind zu prüfen. Anforderungen:

1. Jede heute genutzte `ContextraError`-Variante des Kerns hat genau eine lokale Entsprechung (Erhebung: `grep -rn 'ContextraError::[A-Z]' src`).
2. `Display` des konvertierten Fehlers ist bitgleich zum alten Fehler (tabellengetriebener Test je Variante).
3. Keine Sammelvariante `Other(String)`, die Kontext verwirft; Ausnahme: I/O mit `#[source]`.
4. Gate `error-domain` (heute in `not_run_baseline`) wird in S-01 definiert und in S-12 aktiviert; Ausgangsbasis ist `tools/error_topology_mapper.py`.

### 5.4 Differenztests (Verhaltensgleichheit)

Jede Ablösung einer Kopie durch ihren Besitzer wird zuerst durch einen Differenztest abgesichert, der alte und neue Implementierung gleichzeitig kompiliert (Commit A) und gleiche Eingaben vergleicht. Nach grünem Test folgt Commit B; der Test bleibt als Regressionstest, solange beide Seiten existieren.

| Bereich | Eingabe | Toleranz |
| --- | --- | --- |
| IDs | §4.1 | bitgleich |
| Clock/Rng/ms | `ManualClock`, `SeededRng` | bitgleich |
| Distanz (`types::DistanceMetric::compute` ↔ `vector::distance` ↔ `simd::dispatch`) | 10.000 Vektorpaare, Dim 4/17/128/768, normale, Null-, Denormal-Werte | relativ ≤ 1e-5 (f32) bzw. bitgleich, wo beide Seiten dieselbe Skalarfunktion nutzen; NaN/Inf-Verhalten gleich |
| Regler (`decay`, `homeostat`, `pid`) | gleiche Eingabefolge mit `ManualClock` | bitgleich (f64) |
| Chunker (`engine::chunker` ↔ `ingest::chunking::prose`) | Korpus | identische Chunk-Folge, Reihenfolge, Breadcrumbs |
| Tombstone (`types` ↔ `mvcc`) | `mvcc/tests/tombstone_diff.rs` \[V\] | bitgleich |
| Temporalfilter (`engine::temporal_filter` ↔ `rank::temporal`) | Filterausgaben | bitgleich |

---

## 6. Umsetzungsreihenfolge (Rest ab HEAD)

### 6.1 Definition of Done (gilt für jeden Schritt)

Ein Schritt ist abgeschlossen, wenn alle Punkte erfüllt sind:

1. Commit-Paar nach INV-3, Nachrichten `S-xx A` und `S-xx B`.
2. `cargo check --workspace --all-targets --locked` grün (INV-4).
3. Die in der Zeile genannten Gates sind grün. **GL** = G-L1 bis G-L4, **GF** = G-F1, **GD** = G-D1, **GE** = G-E1, **GC** = G-C1 (§8.4).
4. Score und Brückenzählung neu erzeugt (§8.6); kein Crate hat einen niedrigeren Score als vorher (Ratchet).
5. Crate-lokale `SPEC.md`/`PLAN.md` tragen den neuen Status.
6. Keine neue Verletzung von R10 und R11.
7. Der Revert-Commit baut (`git revert` auf einem Wegwerf-Zweig, `cargo check`).

### 6.2 Schritte

Nummern und Reihenfolge entsprechen v2. „R“ hinter der Nummer bedeutet Restarbeit nach §2.2. Die Voraussetzungen nennen nur harte Abhängigkeiten.

| Schritt | Inhalt | Voraussetzung | Nachweis |
| --- | --- | --- | --- |
| **S-00R** | Werkzeugliste (§7) bereitstellen; Baseline mit `cargo` neu erheben: `cargo metadata`, `cargo tree` je Crate (`--no-default-features` und Default), `cargo check` je Crate, `count_bridge_refs.py`, Score; exakte Klassifikation der 86 `from_key`-Aufrufstellen | — | Baseline in `docs/decoupling/baseline/`; Differenz zu `ef3364f` und zum Stand dieser Spezifikation dokumentiert; Aussage „`--no-default-features` baut“ je Crate belegt oder als Befund erfasst |
| **S-01R** | `xtask`-Globs aus `protected-paths.toml` und Kommentar im `justfile` entfernen; für jedes Gate in `not_run_baseline` einen CI-Job oder einen ADR mit Frist; CI: `cargo check --workspace --all-targets --locked`, Matrix `docid-128`; `public-api` auf passender Toolchain; Ratchet `types_fan_in`, `ports_fan_in` (R11); `gate_g1_layering.py` entfernen, sobald G-L1 in CI läuft | S-00R | G-L1 scheitert bei absichtlichem Test-Verstoß (`test_gate_layering_negative.py`) und ist am Baseline-Stand grün |
| **S-02R** | Wurzel-Korpus `tests/golden/` mit `MANIFEST.sha256`, `tools/golden_regen.sh`, Differenz-Harness (§5.1, §5.4); `mvcc`-Korpus einhängen | S-00R | GF am unveränderten Stand grün |
| **S-03R** | Doku/Metadaten: `types/lib.rs`-Verwenderliste ersetzen (generierter Verweis), falsche Docstrings (`privacy`, `cognition`, `kvcache`, `avv-generator`, `sys/lib.rs:19`, `vector/types.rs:3–6`), Item-Diff `AvvError`/`AvvGeneratorError`; `capabilities.toml` um `sublevel` (E1a/E1b) erweitern | S-01R | `capabilities-consistency` grün; G-DOC1 grün |
| **S-10C** | `ids` Commit C: `from_key`-Aufrufstellen nach Klassifikation anpassen; Differenztest §4.1 gegen alle `DocId`-Kopien | S-02R | GF, GD, GE, GL |
| **S-11R** | `env`: `default = ["std"]`; `test-utils` nur dort aktivieren, wo benutzt; `unix_ms`-Differenztest | S-02R | GD, GL |
| **S-12R** | Feature-Hygiene nach R10 für `crypto` (leer), `graph`, `kvcache`, `rank`, `text`, `vector`, `testkit`, `license`, `audit-export`, `env`; `erasure` Features (S-24); Gate `error-domain` aktivieren | S-01R | `feature_audit` (entrauscht) grün; `cargo hack check --each-feature -p <crate>` je Crate |
| **S-20R / S-22R / S-23R** | `license`, `audit-export`: `workspace` aus Default; `vault`: Quelle `volatile_vault.rs`, `vault_compat.inc`, `vault_sys.inc` aus `db` entfernen, Re-Export; `adapt`: Features auf `exp-flow-thompson`, `exp-rie-greedy`, `offpolicy.rs` gegen `off_policy.rs` abgleichen | S-12R | GL; `grep -rn 'volatile_vault' crates --exclude-dir=contextra-vault` nur Re-Exports; `grep -rn 'trait \(Clock\|Rng\)' crates/contextra-adapt/src` je 1 |
| **S-24** | `erasure`: Features `signing`, `vault`, `async`, `conformance`; Datei→Abhängigkeit-Zuordnung per Erhebung im Manifest festschreiben; Default erhält heutiges Verhalten | S-12R | `cargo check --no-default-features` und je Feature |
| **S-30B** | `mvcc` Commit B: `ids`/`env`/`MvccError`; `types`/`ports` hinter `workspace`; `types/src/tombstone.rs` ersatzlos entfernen (R9, nur `mvcc` und `types` nutzen ihn \[Ü\]). Vor Beginn: Loom-Tests (`loom_sequence_log`, `loom_snapshot_registry`, `loom_tx_buffer`) in `loom-gate.yml` prüfen (Pfadfilter) | S-10C, S-11R | GL, GF, GC, GD (Tombstone), GE |
| **S-31** | `crypto` Kern: `TenantId` → `ids`; `KeyManager`-Signaturen auf `[u8; 32]` (eigener Commit); `default = []`; `types`/`ports` hinter `workspace` | S-10C, S-11R, S-12R | GL, GF (WAL-HMAC, MFKD, KV v2), GC, GE |
| **S-32** | `text`: `PostingStore`, Brücke (`wand.rs`, `scanner.rs`, `index_struct.rs`, `purge.rs`, `morph_index.rs`), `TextError`, Clock → `unix_ms`, ID-Breite im Header | S-10C, S-11R | GL, GF (Postings v alt/neu), GD (ms), GE |
| **S-33** | `rank`: `engine::temporal_filter` → `pub use` auf `rank::temporal` nach Differenztest; `types-compat` → `workspace` | S-10C, S-11R | GL, GD |
| **S-34** | `graph`: Brücke, `GraphError`, `default = []`, ID-Breite im Header | S-10C, S-11R, S-12R | GL, GF (bincode v alt/neu), GC (TSan), GE |
| **S-35** | `checkpoint`: `SnapshotSource`, Dateinaht, `CheckpointError`, restliche 4 Referenzen | S-10C, S-11R | GL, GF (Meta/Manifest) |
| **S-36** | `vector`: **zuerst V-VETO-Test anlegen** (K8); Kern auf `ids`/`env`/`mvcc`/`VectorError`; `contract.rs` vereinen; `VectorDeleteMode`; Feature `simd`; ID-Breite `HNSW_VERSION` 2 → 3; ohne DiskANN-Umzug | S-30B, S-10C, S-11R | GL, GF (HNSW v 2/3), GD (Distanz, Rng, ms), GC, GE |
| **S-37** | `store`: `StoreError`, `compat/` (`engine/`, `scanner/`), Imports → `ids`/`env`; `FeatureRing`-Entfernung **nach** S-50; zuletzt Manifest v1 → v2 | S-30B, S-31, S-10C, S-11R | GL, GF, GC (Loom/TSan), Crash-/Recovery-Suite unverändert grün (INV-6), GE |
| **S-38** | **Paar:** `crypto`-KV-Teil (`kv_cipher`, `kv_segment/*`, `kv_shredding`, `revocation_log`) → `kvcache`; `kvcache` Kern/Brücke/Features; `infer-candle/kv_bridge`; `mutants-guarantees.toml`, `loom-gate.yml`, `CODEOWNERS` im selben Commit; Konstanten `CURRENT_KV_KEY_DERIVATION_VERSION` und `CURRENT_KV_FORMAT_VERSION` per `const_assert_eq` koppeln | S-31, S-11R | GL, GF (KV-Segment), GC (`loom_kv_deferred_zeroize` am neuen Ort grün), Mutation-Score G2 ≥ Baseline |
| **S-39** | `infer-ollama` (`OllamaError`, `Rng`/`Clock` aus `env`), danach `infer-candle` (`candle`-Feature, `ConfidenceCalibrator`, Newtype `ModelFingerprint`) | S-11R, S-38 | GL; `cargo tree -p contextra-infer-candle --no-default-features` ohne `candle-*` und ohne `rank` im Kern |
| **S-50** | `ports` → `license` (Lizenz-Traits), `erasure` (Erasure-Familie), `testkit` (Conformance); Re-Exports bleiben (R9) | S-20R, S-24 | GL; `grep -rln 'FeatureRing\|LicenseGate' crates/contextra-{store,vector,text,graph,kvcache,rank,checkpoint,mvcc}` leer |
| **S-51** | `types` → `erasure` (Erasure-Typen), `graph` (`PprConfig`, `PprAlgorithm`, `TlHfdParams`) | S-50, S-34 | GL, GE |
| **S-60** | Regler: `adapt` nimmt die fehlenden Methoden auf (decay 5, homeostat 3, pid 5), dann `pub use` in `engine`/`db` | S-23R | GD (bitgleich) |
| **S-61** | Chunker: Paritätstest in `engine` (`ingest` als Dev-Abhängigkeit), danach Adapter über `ingest::chunking` (`contextra::chunker::MarkdownChunker` API-stabil) | S-01R | GD; `mcp`-Tests unverändert |
| **S-62R** | `router`: `ports_local.rs` entfernen, nachdem die drei `mcp`-Importe auf `contextra_ports` umgestellt sind; `Vfs`-Stelle auf lokale `&Path`-Funktion | S-01R | GL |
| **S-63** | `cognition` invertieren (Brücke im Crate `db`, Feature `cognition`), danach `agent` invertieren | S-37, S-50 | GL; `cargo tree -p contextra-cognition -e normal` ohne `contextra-engine` |
| **S-64** | `engine` (Lizenz → `license`, `extraction` → `exp-extraction`), `db`, `contextra` (Feature `license`; ohne Feature `OpenFastGate`-Semantik per Test), `mcp`, `py`, `testkit` | S-50, S-51, S-33 | GL; G-A1 ohne unbeabsichtigte Diffs |
| **S-70** | Aufräumen: deprecated Re-Exports (R9) entfernen, Fehlervarianten in `ContextraError` entfernen (nur bei grünem G-E1), `archive/`-Verschiebungen bestätigen | alle | alle Gates |
| **S-70a** | **Nachprüfung B23:** Item-Diff `archive/contextra-crypto-proofs` ↔ `erasure` und Mutation-Score der Zieldateien in `erasure` ≥ Score der Quelldateien; bei Nichterfüllung Wiederherstellung hinter Feature `legacy-erasure` (nicht Default) | S-00R | G-C1 (Mutation G2/G3 ≥ Baseline) |
| **S-80** | `contextra-vector-diskann` aus `vector/diskann/` | S-36 | GL, DiskANN-Tests (`required-features`) isoliert grün |
| **S-81** | `contextra-graph-research` aus den Klasse-A/B-Experimenten | S-34 | GL, GD |

### 6.3 Reihenfolge-Zwang (Zyklenfreiheit \[A\])

`ids` und `env` vor allem E1 (erfüllt). `mvcc` vor `vector` und `store`. `crypto`-Kern vor `store`. Der `crypto`-KV-Umzug erfolgt nach dem `crypto`-Kern und gemeinsam mit `kvcache`. Lizenz- und Erasure-Ports (S-50) vor dem Entfernen von `FeatureRing` aus `store`/`kvcache`. `cognition` erst nach der `engine`-Lizenzumstellung. Neue Crates (`vector-diskann`, `graph-research`) erst nach dem jeweiligen Kernschritt.

### 6.4 Bindende Detailvorgaben je Crate (aus v2, weiterhin gültig)

| Crate | Vorgabe |
| --- | --- |
| `crypto` | Kern bleibt: `aead`, `dek`, `kdf`, `secret`, `error`, `anti_tamper`, `crypto.rs` (`KeyManager`), `wal_crypto`, `wal_completeness`; `audit_chain.rs` liegt noch im Crate \[V\] und folgt B23. `KeyManager::{derive_kv_key, derive_kv_key_scoped, cipher_for, cipher_for_scoped}` nehmen `[u8; 32]` statt `kv_cipher::ModelFingerprint`. `ed25519-dalek` nur, solange `ed25519_proof`-Code im Crate ist. |
| `text` | Lokales Trait `PostingStore` enthält **genau** die in `src/` genutzten `StorageEngine`-Methoden (Erhebung per `grep`), keine Methode mehr. Brücke implementiert es für `StorageEngine`. `types.rs` und `contract.rs` werden `pub use` auf `ids`/`env`. Leere Features nach Audit streichen. |
| `rank` | `DocId` → `ids`, `Clock` → `env`; `dibud` → `exp-dibud`, `fusion/resonance.rs` → `exp-resonance` mit Naht `fusion/ext.rs`; `drift` bleibt (keine Dublette, B16). |
| `graph` | Brücke: `provider.rs`, `purger.rs`, `csr/graph_index/*`, alle Importe von `ports::{graph_index, graph}`. Lokale `DocId`/`TxId` → `ids`. Experimente Klasse A/B hinter `exp-*`, Klasse C (`hyperedge`, `path_rag`, `consistency_enforcement`) bleibt im Kern. Ab > 2.000 Zeilen Experimentcode: `graph-research` (S-81). |
| `checkpoint` | `SnapshotSource` = genau die genutzten `StorageEngine`-Methoden; lokale schmale Dateisystemnaht (`hardlink_cloner.rs`) statt `ports::Vfs` (B08); `WorkflowState` lokal abbilden; nicht mit `mvcc` zusammenlegen. |
| `vector` | Brücke: `hnsw/vector_index_impl.rs`, `diskann/vector_index_impl.rs`, `hnsw/purge.rs`, `hnsw/scanner.rs`, Teile von `error.rs` und `types.rs`. `mvcc` bleibt Kern-Abhängigkeit; eigener `VisibilityLog` ist Nicht-Ziel. Feature `simd` ersetzt `compute_distance` durch `contextra_simd::dispatch`, Skalarpfade bleiben als Orakel (B09). V-VETO: `PartialRebuildConfig` bleibt aus; der Wachtest muss existieren und grün sein. |
| `store` | Eigener `StoreError` mit Abbildung der heute genutzten Varianten (`Storage`, `Io`, `WalCorruption`, `ChecksumMismatch`, `Transaction*`, `Conflict`, `WalTruncationDetected`, `CrossDeviceLink`). `lsm/config.rs`: `validate_against_features` und `FeatureRing` erst nach S-50 entfernen, `validate_durability` bleibt. `tenant_codec` → `ids::TenantId`. `io_seam`/`fsync` unverändert, `Vfs` per Brücke (B08). INV-6 strikt. |
| `kvcache` | Brücke: `prefix_store.rs`, `adapters.rs` (inkl. `FeatureRing`), `KvCachePruner`/`KvCacheScanner`, `scratchpad_invalidation.rs`, Port-Teil von `service.rs`. `CacheDirective`/`StepId` einmal lokal. Features: `kv-encryption` ohne `legacy-erasure`, `exp-kivi`, `exp-content-addressed`; `kvcache-attention-eviction` streichen. |
| `infer-ollama`, `infer-candle` | `ImportanceScore` lokal mit Skala 0..1 im Doc-Kommentar, `From` in `compat/`. Vorbedingung: `--no-default-features` baut (ungegatetes `pub use error::…` prüfen). `candle-core`, `candle-nn`, `candle-transformers`, `tokenizers`, `half` optional hinter `candle`. Vorbedingung: `grep -rn 'contextra_infer_candle::' crates` über alle Verbraucher, damit kein Symbol durch das Gaten verschwindet. |
| `types`, `ports` | `types` behält `error`, `error_dto`, `budget`, `filter`, `importance`, `saos`, `document`, `misc` (abzüglich Auszug), `schema`, `RequestContext`, `model_fingerprint`, `retrieval_strategy`. Varianten `Sandbox*`, `Cluster`, `KvQuantization`, `KvDeleteModeConfig`, `HnswConnectivityDegraded`, `GraphRepairFailed` bleiben bis G-E1 grün ist. `ports` behält `storage`, `vector_index`, `text_index`, `graph_index`, `graph`, `embedding`, `reranker`, `query_rewriter`, `checkpoint`, `kv`, `kv_bridge_port`, `attention`, `lifecycle`, `observability`, `plugin`, `vfs`, `fs`, `scanner/*`; `Vfs`, `VfsFile`, `VfsOpenOptions`, `VerifiedVia` bleiben (B08). |
| `router`, `engine`, `db`, `contextra`, `agent`, `cognition` | `router`: `RetrievalStrategy`-Abbildung nur in `arm_registry.rs`. `engine`: `extraction/` → `exp-extraction` ohne Zusammenlegung mit `graph/entity_extraction.rs`. `db`: Regler als `pub use` aus `adapt`; `multistep.rs` bleibt. `agent`: lokale Ports `ContextStore`, `ScratchpadStore`. `cognition`: lokale Ports `DocumentSource`, `ConsolidationSink`, `HyperedgeSink`, `DecayPolicy`, `CognitionError` statt `ContextraError`-Varianten; `*_local.rs`-Umbenennungsdateien löschen. |
| E0-Crates | `sys`: `lib.rs:19` korrigieren. `ingest`: `fuzz_support.rs` hinter `fuzz`; Brücke mit `TryFrom` im 128-Bit-Modus. `license`: Inline-Brücken → `compat/`, `VectorDeleteMode` → `vector`. `audit-export`: `testkit.rs` hinter `test-utils`; Brücke `ProcessingRegistryEntry → AuditRecord` wird nicht gebaut. `infer-onnx`: `fakes` hinter `test-utils`. |
| E3-Crates | `mcp`: `protocol.rs`, `egress_*.rs`, `sandbox.rs` bleiben. `py`: `docid-128 = ["contextra-ids/docid-128"]`, `maturin` mit und ohne `pyo3` getrennt prüfen. `testkit`: `conformance/vfs.rs` und `fault_vfs.rs` bleiben; `erasure-conformance` aus `default`, Suite nach `erasure` (Feature `conformance`). |

---

## 7. Werkzeuge und Arbeitsumgebung (S-00R)

| Werkzeug | Zweck | Stand \[V\] |
| --- | --- | --- |
| Rust `1.89.0`, `rustfmt`, `clippy` | Toolchain (`rust-toolchain.toml`) | festgelegt |
| `cargo-hack` | `--each-feature`, `--no-dev-deps` (G-L3) | in der Analyseumgebung nicht vorhanden |
| `ripgrep`, `jq` | Zählungen, Baseline-Skript | nicht vorhanden; alle Befehle in §8 sind auch mit `grep -rn` formuliert |
| `cargo-public-api` | G-A1 | Toolchain-Konflikt (`nightly-2024-11-01` gegen `rust-version 1.89`) \[Ü\], nicht geprüft |
| `tools/gate_layering.py` | G-L1 | vorhanden, braucht `cargo` |
| `tools/isolation_proof.py` | G-L4 | vorhanden, braucht `cargo` |
| `tools/feature_audit.py` | R10 | vorhanden, **laut**: 107 Meldungen in 37 Crates, enthält `default` und Weiterleitungs-Features |
| `tools/count_bridge_refs.py` | Brückenzählung | vorhanden, zählt auch gegatete Treffer |
| `tools/error_topology_mapper.py` | Basis für G-E1 | vorhanden |
| `tools/gate_bridge_null.py` | G-L2 als Skript | **fehlt**, anzulegen in S-00R |
| `tools/modularity_score.py` | Score nach §8.6 | **fehlt**, anzulegen in S-00R (Referenzimplementierung liegt dem Begleitbericht bei) |
| `tools/golden_regen.sh`, `tests/golden/` | G-F1 | **fehlt** |
| `tools/doc_lint.py` | G-DOC1 | **fehlt** |
