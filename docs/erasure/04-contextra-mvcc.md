# Implementierungsspezifikation `contextra-mvcc` (Löschkette – Snapshot-Entzug und Pin-Alterung)

*Ring 0 · Konzept-WP 3 (§3) und 9 (§4) · Abhängig von: 01 · Blockiert: 03 (`revoke_covering` im Gate), 09, 10 · Index: `00-INDEX-QUERSCHNITT.md`, `00a-INDEX-ERGAENZUNG.md`*

> **Verifikationsgrenze.** Statische Analyse am HEAD `815fc78eb`, keine Toolchain. **✔** = gelesen (Datei:Zeile), **[?]** = offen.

## 1. Zweck und Abgrenzung

`contextra-mvcc` hält die `SnapshotRegistry`. Sie bestimmt, welche Versionen die Compaction behalten muss (`min_active_seqno`). Für die Löschkette braucht der Kernel (03) drei Dinge: einen **korrekten** Entzug mandantenbezogener Snapshots, eine **Pin-Alterung** mit Bericht, und eine **Auskunft**, welche Snapshots ein Gate-Ende (I-9) oder `OverlappingSnapshotPinned` verursachen.

**Nicht hier:** Das Gate selbst (03), die Politik für Checkpoints (09), die Zeitplanung (10). Der Entzug ist **kein Sicherheitsmittel**: G1 stützt sich nie darauf (K-21, 03 §6.2).

Abhängigkeiten: `contextra-types`, `contextra-ports` (`Cargo.toml:16-17`) ✔. **Keine neue Kante.**

## 2. Ist-Zustand (✔ gelesen, `src/snapshot.rs`)

| Aspekt | Befund |
| --- | --- |
| Struktur | `SnapshotRegistry { active: Mutex<BTreeMap<u64, Vec<SnapshotMeta>>>, min_active_seqno: AtomicU64, clock, config }` (`:76-81`). `SnapshotMeta { tenant: Option<TenantId>, created_at: u64 }` (`:63-66`), **ohne Identität** |
| Guard | `SnapshotGuard { registry, seq_no, created_at: Option<u64>, tenant }`; `Drop` ruft `release_for_tenant_at(tenant, seq_no, created_at)` (`:526-531`) |
| Freigabe | `release_for_tenant_at`: sucht per `created_at` und Mandant; **Fallbacks:** nur per `created_at` (ohne Mandant), sonst **`metas.pop()`** (`:380-396`) |
| Entzug | `revoke_covering(tenant, range, gated_seq)` entfernt nur Metas mit `meta.tenant == Some(tenant)` im Bereich `seq ∈ range ∧ seq <= gated_seq` (`:298-327`). Mandantenlose Metas (`register`, `pin`) bleiben |
| Pins | `pin`, `pin_at`, `pin_for_tenant`, `pin_for_tenant_at` fügen Metas ein; `unpin(seq)` ruft `release(seq)` ohne Mandant und ohne Zeit (`:331-368`) |
| Alterung | `longest_active_pin[_at]` (`:425-446`), `reap_expired_pins[_at]` (`:458-489`) entfernen **alle** Metas einer Sequenz älter als `max_pin_duration` und liefern nur `Vec<u64>` |
| Aufrufer außerhalb | `reap_expired_pins`: nur `seq_log.rs` (eigene Struktur) und Tests; `revoke_covering`: **kein** Produktivaufrufer; `pin`/`unpin`: `store/src/lsm/ops/maintenance.rs:21,26` ✔ (Rollback-Pfad) |
| `min_active_seqno` | `u64::MAX`, wenn nichts aktiv ist (`update_min`, Release-Store) |
| Konfiguration | `SnapshotRegistryConfig { max_active_snapshots: 100_000, max_snapshot_age: 300 s }`. **[?]** Wird `max_snapshot_age` irgendwo erzwungen? `rg "max_snapshot_age" crates` |

### 2.1 Befund B-M1: Doppelfreigabe nach Entzug (Sicherheits- und Korrektheitsfehler)

`revoke_covering` entfernt die Meta eines Guards, **der Guard lebt weiter**. Beim `Drop` sucht `release_for_tenant_at` seine Meta nicht mehr. Statt nichts zu tun, greifen die Fallbacks:

1. Passt keine Meta mit `created_at == ts ∧ Mandant`, wird die **erste Meta mit gleichem `created_at`** entfernt, auch eines **anderen Mandanten** (`:388`). Mit einer Uhr, die gleiche Nanosekunden liefert (`ManualClock`, grobe Uhren), ist das reproduzierbar.
2. Findet auch das nichts, entfernt `metas.pop()` die **letzte beliebige Meta** der Sequenz (`:391`).

Folge: Der Entzug von Mandant A lässt beim Drop des entzogenen Guards die Registrierung von Mandant B an derselben Sequenz verschwinden. `min_active_seqno` steigt zu früh, die Compaction darf Versionen verwerfen, die Snapshot B noch lesen will. Das ist eine **Verletzung der Snapshot-Isolation anderer Mandanten**, ausgelöst durch genau die Funktion, die die Löschkette erstmals produktiv aufruft. Der bestehende Test (`:571-583`) prüft nur die Zählung des Entzugs, nicht das spätere `Drop`. **Muss vor dem ersten Produktivaufruf (03 WP-3.4) behoben sein.**

## 3. Snapshot-Entzug (WP-3)

### 3.1 Identität statt Heuristik

```rust
/// Eindeutige, nicht wiederverwendete Kennung einer Registrierung (Guard oder Pin).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct RegistrationId(u64);          // intern; kein serde, kein Konstruktor außerhalb des Crates

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RegistrationKind { Guard, Pin }

struct SnapshotMeta {
    id: RegistrationId,
    kind: RegistrationKind,
    tenant: Option<TenantId>,
    created_at: u64,
    revoked: Arc<AtomicBool>,            // geteilt mit dem Guard
}
```

- `SnapshotRegistry` erhält `next_id: AtomicU64` (Zähler ab 1, **deterministisch**, kein Zufall, kein `thread_rng`).
- Jede Registrierung (`try_register_for_tenant_at`, `pin_*`) vergibt eine ID. `SnapshotGuard` speichert `id` und `revoked: Arc<AtomicBool>`.
- **Freigabe nur über die ID:** `release_by_id(seq_no, id)` sucht die Meta mit dieser ID und entfernt sie (`swap_remove`). Wird sie nicht gefunden (bereits entzogen oder gereapt), ist das **ein No-op**. **Alle Fallbacks entfallen** (`created_at`-Suche ohne Mandant, `metas.pop()`).
- `SnapshotGuard::drop` ruft `release_by_id`. Die Methoden `release`, `release_at`, `release_for_tenant_at` werden entfernt. **[?]** Aufrufer: `rg "release_at|release_for_tenant_at|\.release\(" crates` (Erwartung: nur `snapshot.rs`; sonst Anpassung dort).
- **`unpin(seq_no)`** (Signatur unverändert) entfernt die **älteste Meta der Art `Pin`** dieser Sequenz und rührt Guards nie an. Gibt es keinen Pin, ist es ein No-op (vorher: beliebiger `pop`). Bestehende Paare `pin(seq)`/`unpin(seq)` (`maintenance.rs:21,26` ✔) verhalten sich gleich.
- Neu: `SnapshotGuard::is_revoked(&self) -> bool` (`Acquire`-Load auf `revoked`). Die Engine (10 §9) bricht damit lange Scans eines entzogenen Snapshots ab. **Kein** Fehler beim Lesen selbst, denn das Gate maskiert bereits (03 §6).

### 3.2 `revoke_covering` (Signatur unverändert, Semantik präzisiert)

```rust
pub fn revoke_covering(&self, tenant: TenantId, range: Range<u64>, gated_seq: u64) -> usize;
```

- Entfernt Metas mit `tenant == Some(tenant)`, `seq ∈ range`, `seq <= gated_seq`, **beider Arten** (`Guard` und `Pin`). Setzt für jede entfernte Meta `revoked.store(true, Release)`. Aktualisiert `min_active_seqno`. Rückgabe: Anzahl entfernter Metas.
- **Mandantenlose Metas bleiben** (K-21). Das ist Absicht und wird dokumentiert: Der Kernel behandelt sie über das Gate (03 §6.2). Die Doku-Kommentare nennen es ausdrücklich.
- Neu: `revoke_covering_report(...) -> RevokeReport { revoked: usize, retained_tenantless: usize, retained_min_seq: Option<u64> }`. `retained_tenantless` zählt mandantenlose Metas im Bereich, die **nicht** entzogen wurden. Der Kernel schreibt `OverlappingSnapshotPinned{min_seq}` als Ausnahme, wenn `retained_min_seq.is_some()` und G3 deshalb verzögert ist (03 §7.8). Der Bericht enthält nur Zahlen und Sequenzen, keine Schlüssel.
- **Aufrufer:** nur der Kernel in `publish_gate` für `EraseTarget::Tenant` (03 §6.2 Schritt 12).

### 3.3 Auskunft für das Gate-Ende (I-9) und die Diagnose

```rust
pub struct Blocker { pub seq: u64, pub kind: RegistrationKind, pub tenant: Option<TenantId>, pub age: Duration }
impl SnapshotRegistry {
    /// Alle Registrierungen mit `seq <= up_to`, aufsteigend nach Sequenz. Höchstens `limit` Einträge.
    pub fn blockers_up_to(&self, up_to: u64, limit: usize) -> Vec<Blocker>;
}
```

Der Kernel nutzt `min_read_snapshot()` (`:282`) ✔ für die Bedingung 3 in 03 §6.5 und `blockers_up_to` für Metrik und Alarm. Beide halten den `active`-Mutex nur für die Dauer des Kopierens (`parking_lot`, nie über `.await`).

## 4. Pin-Alterung (WP-9)

Heute hat `reap_expired_pins` keinen Produktivaufrufer ✔ (B-11). Ein vergessener Pin hält den Compaction-Floor beliebig lange. Wegen `DropKeys` (03 §7.2) blockiert das **G3 im Store nicht mehr**, es verzögert aber die reguläre Garbage Collection und Checkpoint-/Reader-Aufräumen. Die Alterung wird deshalb als Betriebsmaßnahme eingeführt, nicht als Löschvoraussetzung.

```rust
pub struct ReapPolicy { pub max_pin_age: Duration, pub max_guard_age: Option<Duration> }   // Default: 7 d, None
pub struct Reaped { pub seq: u64, pub kind: RegistrationKind, pub tenant: Option<TenantId>, pub age: Duration }
impl SnapshotRegistry {
    pub fn reap_expired(&self, policy: ReapPolicy, now_nanos: u64) -> Vec<Reaped>;   // sortiert nach seq, dann age
}
```

- Entfernt `Pin`-Metas älter als `max_pin_age`. `Guard`-Metas nur, wenn `max_guard_age = Some(..)` (Default: **nie**, denn ein lebender Guard gehört einem Leser). Setzt `revoked = true` für jede entfernte Meta. Aktualisiert `min_active_seqno`.
- Die bestehenden Methoden `reap_expired_pins`/`reap_expired_pins_at` bleiben mit **unverändertem Verhalten** (entfernen beide Arten, liefern `Vec<u64>`), damit vorhandene Tests grün bleiben. Sie werden in der Doku als „ohne Bericht, nur Tests“ markiert; neue Aufrufer nutzen `reap_expired`.
- **Aufrufer:** der `ErasureWorker` (10 §6) bei Wartung, **nicht** periodisch im Leerlauf: Er ruft `reap_expired` nur, wenn mindestens ein Auftrag offen ist (kein Polling ohne Aufträge, Index 3.4) oder wenn der Kernel `complete` wegen `snapshot_blocked` verweigert hat. Alarm (Metrik, nur Zähler) bei jedem Treffer.
- Ein gereapter Pin aus dem Checkpoint-Pfad (`pin`/`unpin` über `lsm/ops/maintenance.rs`) ist ein Fehlerfall im Checkpoint-Layer (09 §5): `drop_checkpoint` meldet ihn, statt ihn zu verschlucken.

## 5. Tests

Bestehende Tests in `snapshot.rs` und `ssi.rs` bleiben unverändert grün (Verhaltensänderung nur dort, wo ein Test den fehlerhaften Fallback voraussetzt: das wird dann als Fehler **im Test** behandelt und dokumentiert).

| ID | Inhalt |
| --- | --- |
| TST-MVCC-01 | **Regression B-M1:** Mandant A und B am gleichen `seq` und **gleichem** `created_at` (`ManualClock`). `revoke_covering(A, …)`; `drop(guard_A)`; Mandant B bleibt registriert, `min_active_seqno` unverändert. Das Test schlägt auf dem heutigen Code fehl |
| TST-MVCC-02 | Doppelfreigabe: Guard, Entzug, `drop`, erneut `reap_expired` ⇒ keine Auswirkung auf andere Registrierungen (proptest über beliebige Folgen aus `register/pin/revoke/reap/drop`, Referenzmodell: `BTreeMap<RegistrationId, Meta>`) |
| TST-MVCC-03 | `unpin` entfernt nur `Pin`-Metas, nie `Guard`; ohne Pin No-op |
| TST-MVCC-04 | `revoke_covering` entfernt `Pin` und `Guard` des Mandanten, lässt mandantenlose Metas stehen, `RevokeReport` zählt sie |
| TST-MVCC-05 | `SnapshotGuard::is_revoked` wird nach Entzug und nach `reap_expired` wahr, davor falsch |
| TST-MVCC-06 | `reap_expired`: `max_pin_age` (nur Pins), `max_guard_age = Some` (auch Guards), Default lässt Guards stehen; Bericht sortiert und ohne Duplikate |
| TST-MVCC-07 | `blockers_up_to` respektiert `limit`, Reihenfolge, keine Schlüssel im Ergebnis |
| TST-MVCC-08 | `RegistrationId` strikt monoton, nach 10⁶ Registrierungen eindeutig (Stresstest, mehrere Threads) |
| TST-MVCC-09 | Nebenläufigkeit: `register`/`drop`/`revoke` parallel (bestehende Lock-Disziplin: ein Mutex, kein `.await` darunter) |
| TST-MVCC-10 | Kein Zufall und keine Systemzeit in neuem Code: Zeit nur über `clock` (`clippy.toml`-Verbote ✔) |

## 6. Definition of Done

Build, Test, `clippy -D warnings`, `cargo deny check` grün; `tests/layering.rs` grün (keine neue Kante); TST-MVCC-01 war vor der Änderung rot und ist danach grün (Nachweis im Commit-Text); Dokumentation mit „# Fehler / # Crash“ je neuem `pub`-Element; kein `unwrap`/`expect` in Produktivcode (Crate-Lints ✔ `[lints] workspace`).

## 7. Offene Punkte

| Nr. | Frage | Vorschlag |
| --- | --- | --- |
| O-M1 | `max_snapshot_age` (300 s) wird möglicherweise nicht erzwungen | WP-0: `rg "max_snapshot_age" crates`. Nicht Teil der Löschkette, nur Befund melden |
| O-M2 | `seq_log.rs` hat eine eigene `reap_expired_pins`-Variante | Unverändert, keine Berührung |
| O-M3 | Soll `revoke_covering` auch mandantenlose Snapshots entziehen können? | Nein. Mandantenlose Registrierungen gehören dem Gesamtsystem (Backups, Checkpoints); sie sperrt das Gate |
