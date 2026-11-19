# contextra-erasure — Crate-Spezifikation (Löschkette, v3.0)

**Ebene:** E0 · **Normativ unter:** `docs/SPEC.md` v3.0 (§6, §7, §9, §10, §12, §13, §14, §16)
**Status:** Soll-Zustand. Ersetzt die Fassung vom 18.11.2026 vollständig.

---

## 1 Zweck und Grenzen

`contextra-erasure` ist der **Koordinator** der Löschkette. Er besitzt:

- die Zustandsmaschine (`state`), das Journal-Protokoll und die Recovery,
- die Ports (`ErasureLayer`, `ReadGate`, `KeyVault`, `KeyAnchor`, `ErasureJournal`, `ErasedSetMirror`, `ReportSink`, `PositiveControl`, `ProofSigner`, `RetentionAttestation`),
- Messung und Bericht (`ErasureReport` V5, Siegel 1–4, Klassifikation),
- die Schlüssel-Abstraktion `ScopeDescriptor` und die Scope-Auflösung.

Sie besitzt **nicht**:

- Schlüsselkryptografie (→ `contextra-crypto`),
- Dateisystem-I/O (→ `contextra-durable-fs`, Trait `Vfs`),
- Speicherstrukturen (→ `store`, `text`, `vector`, `graph`, `checkpoint`, `mvcc`).

Die Implementierungen der Ports liegen in E1. Diese Crate kennt sie nur über Trait-Objekte. Eine Abhängigkeit nach oben besteht nicht (R1).

## 2 Module

| Modul | Inhalt | Status |
|---|---|---|
| `types` | `ErasureScope`, `EraseTarget`, `ClassSet`, `ResidualRisk`, `AssumptionCode`, `Custody`, `Class` (reine Datentypen) | Typen liegen in `contextra-types` (§2.4 SPEC). Hier nur Re-Exporte |
| `state` | Zustände, `step`, `classify`, `TransitionError` | neu, total kodiert |
| `saga` | Takt 1 (`erase`), Takt 2 (Epoche), Hygiene | neu |
| `journal` | Datei-Format, Group-Commit, Torn-Tail, Replay | neu |
| `ports` | alle Traits aus §9 SPEC | neu, ersetzt `ReportSink` ohne Implementierung |
| `gate` | `ReadGate`-Port und `LiveMask`/`ErasedSet`-Datenstruktur | Implementierung in `db` bzw. `store` |
| `scope` | Auflösung `Document`, `Subject`, `Collection`, `Tenant` | neu |
| `witness` | `LayerWitness`, `Scanner<Armed>`, Positivkontrolle | neu, `Scanner<Armed>` nur über `arm()` |
| `report` | `ErasureReport` V5, kanonische Kodierung, Siegel | neu |
| `recover` | Startpfad (§6.6 SPEC) | neu |

**Entfernt** gegenüber der Fassung vom 18.11.2026: `vault` (SagaState-Vault, Tresor-Tombstones), `crypto_local`, `types_local` (Vfs-Duplikat, R4), `io` (Vfs-Duplikat), `gate` in der alten Form, `ReportSink` ohne Implementierung. Die Modulkette `saga` → `vault` → `gate` → `io` → `types_local` wird nicht übernommen.

## 3 Zustände und Übergänge

Die Zustände und ihr Verhalten stehen in SPEC §6.2. Diese Crate implementiert:

```rust
pub enum State {
    Requested, Frozen, LogicallyDeleted, PlaintextPurged, PlaintextMeasured,
    KeyDestroyed, CryptoMeasured, EpochSealed, Hygiene, HygieneMeasured,
    Reported, Completed,
    Restricted { basis: RestrictBasis, until: Option<Timestamp> },
    Refused, Lifted,
    Pending { reason: PendingReason, attempt: u32, next_retry_at: Timestamp },
}

pub fn step(state: &State, ev: Event) -> Result<State, TransitionError>;
pub fn classify(state: &State) -> Class;     // L | P | CStar | C1 | C2 | C3 | M
```

**Eigenschaften (testpflichtig, T-DEL-24):**

- `step` ist total über `(State, Event)`; ungültige Kombinationen liefern `TransitionError`, nie Panik.
- `classify` ist **monoton**: Ein Zustand, der eine Klasse erreicht, verliert sie nicht durch spätere Ereignisse außer `Lifted`/`Refused` (die vor Takt 2 stehen müssen).
- `E` impliziert `L`, `P`, `C` (Klassenordnung L < P < C* < C1–C3 < M).
- `Completed`, `Refused`, `Lifted` sind terminal (INV-9).
- `Restricted` darf nicht nach `KeyDestroyed` oder später entstehen (Art. 18 schützt nicht gegen bereits erfolgte Vernichtung; Gate-Test).

## 4 Takt 1 (synchron, Klasse L)

Funktion: `Eraser::erase(scope, on_retention) -> Result<ErasureTicket, EraseError>`, asynchron.

Ablauf (SPEC §6.3):

1. Validierung (Mandant, Berechtigung, Scope-Typ).
2. `RetentionAttestation::check` (Reject → `Refused`, Restrict → `Restricted`).
3. Duplikaterkennung über `scope_commitment` (BLAKE3-keyed). Laufende Saga ⇒ bestehendes Ticket.
4. `ScopeDescriptor` als `AEAD(K_saga[erasure_id])`, nur das Chiffrat und der Referenz-Hash gehen ins Journal.
5. Group-Commit (`journal::GroupCommit`, Wartefenster 1–2 ms, dedizierter Thread). Flush-Fehler ⇒ `Err`, keine Sperre.
6. Nach `fsync`: `LiveMask`/`SubjectMask`/Epochenmarke setzen, Publikation mit `SeqCst`. `CommitBarrier::exclusive()` mit Haltezeit ≤ 1 ms.
7. Ticket `Frozen`, Siegel 1.

Ein Lesezugriff nach Schritt 6 erhält `Erased` (ReadGate).

**Fehlersemantik:** Jede Stufe vor Schritt 6 ist folgenlos. Schritt 6 ist erst nach durablem Eintrag erlaubt (INV-JRN-1).

## 5 Takt 2 (Epochen, Klassen P, C)

Ein Worker (`saga::epoch::Worker`) schließt Epochen nach Zeitfenster (Standard 5 s, Obergrenze 60 s) oder Mengenschwelle (1.024).

Reihenfolge je Epoche (SPEC §6.4):

1. Scope-Chiffrate entschlüsseln, Slot-Liste erzeugen.
2. `ErasureLayer::purge` für Ebene A (Layer `store`, `text`, `vector`, `graph`, `mvcc`, `checkpoint`, `db` als Cache).
3. `ErasureLayer::witness` mit `Scanner<Armed>` → `LayerWitness` (Siegel 2).
4. Pre-Destruction-Inventar: Digests aller Chiffratblöcke je `key_id`.
5. `KeyDestroyed`-Intent mit Wrap-Nonce, durabel (Regel J1).
6. `KeyVault::destroy_batch` (idempotent).
7. Messung C: `verify_absent`, Entschlüsselungsprobe (`KeyAbsent`), Lesepfad-Probe (`Erased`), Nadelscan nach Wrap-Nonce.
8. Epoch Seal (`KeyVault::rotate_epoch`) ⇒ C1–C3 (Siegel 3).

**Idempotenz:** Ist der Schlüssel nach einem protokollierten Intent bereits abwesend, gilt Schritt 6 als vollzogen.

**Abbruch:** Jeder Fehler lässt den Zustand unverändert; die Sperre bleibt. Der Worker setzt `Pending` mit Backoff.

## 6 Hygiene (Klasse M)

Terminiert durch `purge_rebuild_max` (14 Tage). Ablauf in SPEC §6.5: Purge-Compaction (S3, S4, S6), WAL-Entfernung mit Dateiprüfung, Index-Neuaufbau mit `ArtKey`-Wechsel, Checkpoint-Sweep, Lineage-Kaskade, Messung M (Siegel 4).

Die Compaction-Primitive gehören zu `store` (§10.4 SPEC). Diese Crate ruft sie über `ErasureLayer::purge` auf und prüft das Ergebnis über `witness`.

## 7 Journal

- Eigene Datei über `Vfs` (`contextra-durable-fs`), **nicht** im LSM-WAL (ER-E15).
- Eintrag: `erasure_id (16 B) ‖ state (u8) ‖ attempt (u32) ‖ clock (u64) ‖ payload_len (u32) ‖ payload ‖ hash_chain (BLAKE3, 32 B)`.
- **Torn-Tail:** Ein unvollständiger letzter Eintrag wird beim Start verworfen. Vollständige Einträge nie (T-JRN-1).
- **Replay** ist idempotent. `recover()` läuft vor Freigabe des Lesepfads (INV-RECOVER-1).
- `K_saga[erasure_id]` wird bei `Completed` vernichtet. Danach enthält das Journal keinen Scope-Klartext und keinen Personenbezug mehr.
- Größe: Gate `journal-growth`. Kompaktion des Journals nur nach `Completed` oder `Refused`.

## 8 Ports (Kurzübersicht, Signaturen in SPEC §9)

| Port | Implementiert in | Hinweis |
|---|---|---|
| `ErasureLayer` | `store`, `text`, `vector`, `graph`, `checkpoint`, `mvcc`, `crypto` | je Layer genau eine Implementierung; Schichtzuordnung §9.1 SPEC |
| `ReadGate` | `db` (`TenantScopedStorage`) | O(1), `check` vor jeder Entschlüsselung |
| `KeyVault` | `crypto` | Custody K0/K1 ausgeliefert; K2/K3 `CapabilityUnsupported` |
| `KeyAnchor` | `crypto` (`FileOnly`, `OsKeyring`) | `profile()` bestimmt maximale Klasse |
| `ErasureJournal` | `erasure::journal` | Eigene Implementierung, nicht austauschbar im Regulated-Tier |
| `ErasedSetMirror` | `erasure` (Dateiziel), Fremdsystem optional | Pflicht im Regulated-Tier |
| `ReportSink` | `erasure::report` (Ledger außerhalb `data_dir`) | Ohne Ledger-Pfad: Start verweigert |
| `ProofSigner` | `crypto` (Ed25519) | |
| `PositiveControl` | `testkit` (Lab) und `erasure` (Scanner-Selbsttest) | Live-Canary nur in Lab/CI (ER-E17) |
| `RetentionAttestation` | Betreiber | Default: `Reject` bei aktiver Pflicht |

**Verboten:** Ports mit Stub-Implementierung, die Erfolg melden (INV-WIRING-2). Fehlt eine Implementierung, liefert der Aufruf `CapabilityUnsupported`.

## 9 Scanner und Messung

- `Scanner<S>` mit Typzustand `Disarmed` → `Armed`. `arm()` führt den Positivkontrolltest aus und schlägt fehl, wenn die Kontrolle nicht gefunden wird (INV-WITNESS-MINT-1, T-DEL-7, T-DEL-33).
- `LayerWitness` enthält: Ebene, Methode, Zählwerte, Digest, Positivkontrolle (Ergebnis), Ergebnis.
- Der Scanner sieht keine Store-Interna. Er arbeitet auf Bytes (`ForensicScanner`, §19.3 SPEC).
- Die Messung ist rein lesend. Der Purger kennt den Witness nicht (Trennung, §8 Invariante 2).

## 10 Bericht `ErasureReport` V5

- Kanonische Kodierung: festgelegte Feldreihenfolge, Längenpräfixe für Byte-Felder, keine Map-Typen. `bincode` für signierte Felder verboten (ER-E16).
- Ed25519-Signatur über die kanonischen Bytes. Unbekannte Felder werden abgelehnt.
- Pflichtfelder: siehe SPEC §13.1.
- Merkle-Pfade binden jede Einzelmessung an die Wurzel.
- V4 ist eingefroren. Ein V4-Bericht wird nicht mehr erzeugt; sein Verifier bleibt für Altbestände bis zum Ende der Aufbewahrungsfrist.
- **Verboten im Bericht:** Klartext-IDs, Dateinamen, DocIds, Inhalte, Chunk-Hashes (INV-12).

Verifikation: `doctor verify-report` (offline). Testvektoren für einen unabhängigen Verifier (T-DEL-28).

## 11 Fehler

`ErasureError` mit Kategorien aus SPEC §11.1. Keine Variante trägt Klartext oder Schlüsselmaterial. `Display` und `Debug` sind geprüft (INV-12, Test).

## 12 Konfiguration

Die Werte stehen in SPEC §18 (`ErasureConfig`). Diese Crate validiert sie in `ErasureConfig::validate()` vor dem Öffnen. Verletzungen liefern `ConfigError`, nie einen Standardwert.

## 13 Lint und Gates

- `#![forbid(unsafe_code)]`, `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]`.
- Keine Abhängigkeit auf `aes-gcm-siv`, `sha2`, `hkdf`, `argon2` (SPEC §4.8). Kryptografie nur über den Port `ProofSigner` und über `crypto`.
- Gates: `layering`, `ownership`, `surface-coverage`, `claim-generator`, `deletion-honesty`, `loc-budget` (≤ 2.500 Zeilen für die gesamte Löschkette, SPEC §21).

## 14 Tests

Pflichttests: T-DEL-7, T-DEL-14, T-DEL-15, T-DEL-20, T-DEL-21, T-DEL-22, T-DEL-23, T-DEL-24, T-DEL-25, T-DEL-26, T-DEL-28, T-DEL-30, T-DEL-31, T-DEL-33, T-DEL-35. Mutationstests (`cargo-mutants`, Gate ≥ 95 % für `step`, `classify`, Barriere, Anker). `loom` für `LiveMask`-Publikation und ReadGate.

## 15 Arbeitspakete (Reihenfolge)

| WP | Inhalt | Gate |
|---|---|---|
| L-1 | Typen und `state::step`/`classify` mit Tests T-DEL-24 | `layering`, mutants |
| L-2 | `journal` mit Torn-Tail und Replay (T-JRN-1, T-DEL-21) | Fuzz Journal-Replay |
| L-3 | `ports` ohne Stub-Implementierungen | `unwired` |
| L-4 | `scope`, `witness`, Positivkontrolle | T-DEL-7, T-DEL-33 |
| L-5 | Takt 1 und Takt 2 gegen `ReferenceModel` | T-DEL-26 |
| L-6 | Vfs-Dubletten auflösen (`ports`, `erasure::types_local`, `store::io_seam`, `store::io_seam::types`) → nur `durable-fs` | `ownership` |
| L-7 | Bericht V5 mit Ed25519 und Testvektoren | T-DEL-28 |
| L-8 | Recovery vor Lesepfad (T-DEL-20) | Crash-Matrix |

**Phase 1 (Erasure Lab, `testkit`) muss vor L-5 stehen** (ER-E20, Gate G0).

## 16 Offene Punkte

Siehe SPEC §23. Für diese Crate relevant: P-01, P-03, P-06, P-11, P-19 (NIST CE-Bedingungen), O-37 (K2-Spike), O-39 (Standardwerte).
