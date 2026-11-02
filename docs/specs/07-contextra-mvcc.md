# `contextra-mvcc` – Implementierungsspezifikation

|Feld|Wert|
|---|---|
|Zweck-Satz (D-97)|Sequenzen, Snapshots und SSI-Validierung für Mehrversions-Nebenläufigkeit.|
|Ring|2|
|Erlaubte Workspace-Abhängigkeiten|`types`, `ports` (SPEC 5.3); Verbraucher: `store`, `engine` (**nicht** `vector`, D-88)|
|Stufe / Gates|S0; G-C (Sentinel-Fix F-07)|
|Arbeitspakete|WP-3.3 (Sentinel, Mandanten-Registry), WP-5.1 (Snapshot-API)|
|SPEC-Anker|8.7, I-12, D-11, D-61, D-87, T-SNAP-1, T-VER-1, T-CONC-1, F-07, F-29|

## 1. Ist-Zustand (HEAD `1454bbec0`) [V]

|Punkt|Befund|
|---|---|
|Umfang|4.195 Zeilen: `tx_buffer.rs` (1.425), `ssi.rs` (1.191), `seq_log.rs` (805), `snapshot.rs` (680), `error.rs`, `lib.rs`|
|Sentinel|`snapshot.rs:314`: `let min = active.keys().next().copied().unwrap_or(u64::MAX);` mit Kommentar „`u64::MAX` ist der korrekte Standardwert … die Compaction darf ALLE Tombstones aufräumen“. Weitere Stellen: `snapshot.rs:86` (`AtomicU64::new(u64::MAX)`), `:131` (Doku), `:558`, `:618`, `:658` (Tests). Das widerspricht 8.7 und I-12 (F-29).|
|`error.rs`|Fehlertext nennt „Sentinel-Wert wie `u64::MAX`“ als ungültige Sequenz|
|Registry|Snapshot-Registry ist nicht nach Mandanten geführt (D-61 verlangt `tenant_tag`)|
|Abhängigkeiten|`types`, `ports`, `parking_lot`, `ahash`, `serde`, `bytes`, `thiserror`, `tracing`|
|Tests|47/47 Lib-Tests bestanden **[M]** (3.0.0)|

## 2. Verantwortung und Abgrenzung

`mvcc` verwaltet **Sequenznummern, Snapshots und Konfliktprüfung** (SSI). Es kennt keine Schlüsselinhalte, kein Gate, keinen Löschkern. Die Entscheidung, **welche Snapshots widerrufen werden**, trifft `erasure`/`engine` (D-61: nur Snapshots des Mandanten des Ziels mit `seq ≤ gated_seq`); `mvcc` liefert dafür die Registry-Operation `revoke_covering` und markiert Handles als widerrufen. `vector` kennt `mvcc` nicht; Sichtbarkeit bleibt in `engine`/`erasure` (D-88).

## 3. Zielstruktur (Module) [E]

|Modul|Inhalt|
|---|---|
|`seq`|`SeqAllocator` (monoton, `next_seq = max(current_seq, ledger_floor) + 1`, I-20)|
|`snapshot`|`SnapshotRegistry` (nach `tenant_tag`), `SnapshotHandle` (RAII), `min_read_snapshot() -> Option<Seq>`|
|`ssi`|SSI-Validierung (Read-/Write-Set, Konfliktprüfung)|
|`tx`|`TxBuffer`, Transaktions-Lebenszyklus|
|`error`|`MvccError`|

Die vorhandenen Dateien `seq_log.rs`, `snapshot.rs`, `ssi.rs`, `tx_buffer.rs` bleiben als Modulbasis; Umbenennungen nur, soweit D-90 sie verlangt.

## 4. Schnittstellen [E]

```rust
/// Kein Sentinel: ohne offene Snapshots ist das Ergebnis None; die Kompaktion nutzt dann current_seq, nie "alles sichtbar"
pub fn min_read_snapshot(&self) -> Option<Seq>;

pub struct SnapshotRegistry { /* Map: TenantTag -> BTreeMap<Seq, RefCount>, Gesamtzähler */ }
impl SnapshotRegistry {
    pub fn open(&self, tenant: TenantTag, at: Seq) -> Result<SnapshotHandle, MvccError>;   // Limit: ≤ 1024 je Mandant (D-87)
    /// widerruft nur Snapshots des Mandanten mit seq <= gated_seq, die den Bereich berühren (D-61)
    pub fn revoke_covering(&self, tenant: TenantTag, range: &Matcher, gated_seq: Seq) -> RevokedCount;
    pub fn min_for_compaction(&self, current: Seq) -> Seq;       // min(min_read_snapshot, current); nie ein Sentinel
}
impl SnapshotHandle { pub fn seq(&self) -> Result<Seq, MvccError>; /* Err(Revoked) nach Widerruf */ }
```

`Seq` ist ein Typ ohne Sentinel (siehe `types`, TY-08). Ein widerrufener Snapshot liefert `SnapshotRevoked` bei jedem weiteren Zugriff.

## 5. Implementierungsvorgaben

|ID|Vorgabe|Quelle|Test|
|---|---|---|---|
|MV-01|`min_read_snapshot` liefert `Option<Seq>`. Der Sentinel `u64::MAX` entfällt in Berechnung, Atomics, Doku und Tests (`min_active_seqno` als `Option`-ähnlicher Zustand, z. B. getrennter Zähler `active == 0`).|8.7, I-12, F-07|T-SNAP-1, Property|
|MV-02|Ohne offene Snapshots gilt für die Kompaktion `current_seq`, nie „alles sichtbar“.|8.7|T-SNAP-1|
|MV-03|Die Registry ist nach `tenant_tag` geführt; `revoke_covering` berührt nur Snapshots des Mandanten des Ziels mit `seq ≤ gated_seq`. Snapshots anderer Mandanten bleiben unberührt (auch ihre Zählung).|D-61|T-SNAP-1|
|MV-04|Obergrenze Snapshots je Mandant ≤ 1024; Überschreitung ⇒ `Limit::Snapshots`, nie stilles Kürzen.|D-87|T-LIMIT-1|
|MV-05|Widerruf ist **sofort** wirksam: nach `revoke_covering` liefert `SnapshotHandle::seq()` `SnapshotRevoked`; eine laufende Lese-Iteration prüft den Widerruf vor jedem Block.|D-11, T-VER-1|T-VER-1, T-SNAP-1|
|MV-06|`next_seq = max(current_seq, ledger_floor) + 1`; die Sequenz bleibt über Restores monoton und läuft nie rückwärts.|I-20, D-74|T-RESTORE-3|
|MV-07|`pin`-Dauer-Konvertierungen (`u64::try_from(..).unwrap_or(u64::MAX)`) werden ohne Sentinel formuliert (Sättigung als `saturating`-Rechnung auf `Duration`).|I-12, P6|Clippy|
|MV-08|Keine Sperre über `.await` einer fremden Task; Lockreihenfolge: Dokumentlock → Commit-Barriere → Purge-Barrier → Journal (die Barrieren gehören `store`).|5.4, `await_holding_lock = deny`|T-CONC-1 (loom)|
|MV-09|SSI erkennt Schreib-Schreib- und Lese-Schreib-Antidependenzen; Tombstone-Bit-Semantik gibt es nicht mehr (Tombstone = `vlen = 0`, F.5).|F.5|Property gegen `ReferenceModel`|
|MV-10|Keine Panics, kein `unwrap`; Zähler mit `checked_*`/`saturating_*`.|P6, Q-03|Clippy|

## 6. Rückbau und Migration

|Position|Maßnahme|WP|Nachweis|
|---|---|---|---|
|`snapshot.rs:314` und `:86`, `:131`, `:558`, `:618`, `:658`|Sentinel entfernen (MV-01, MV-02); Kommentar „`u64::MAX` ist KORREKT“ löschen|WP-3.3|Grep `unwrap_or(u64::MAX)` in `mvcc` = 0|
|Fehlertext in `error.rs` („Sentinel-Wert wie `u64::MAX`“)|Text auf „ungültige Sequenz“ ohne Sentinelhinweis|WP-3.3|T-LOG-1|
|Registry ohne Mandantenbezug|nach `tenant_tag` indizieren (MV-03); Konsumenten in `store`/`engine` übergeben den Tag|WP-3.3, WP-5.1|T-SNAP-1|
|`seq_log.rs:265,314`, `tx_buffer.rs:671` (`unwrap_or(u64::MAX)` für Nanosekunden)|durch Sättigung ersetzen (MV-07)|WP-3.3|Clippy|
|Bezüge auf `SeqBitTombstone`/Tombstone-Bit|entfernen; Tombstone ist ein Wert mit `vlen = 0`|WP-3.3|Grep|

## 7. Arbeitspakete

1. WP-3.3 (rot): Property-Test „kein Sentinel“: für beliebige Folgen aus `open`/`drop` ist `min_read_snapshot` bei leerer Registry `None` und die Kompaktionsgrenze `current_seq`; Test mit zwei Mandanten für T-SNAP-1.
2. Grün: `Option`-Umbau in `snapshot.rs`, danach `seq_log.rs`, `tx_buffer.rs`.
3. Loom-Modell für Registry (öffnen, widerrufen, schließen gleichzeitig) und `SeqAllocator`.
4. `ReferenceModel` (Testkit) liefert das Orakel für MVCC-Sichtbarkeit.

## 8. Tests und Benchmarks

|ID|Prüft|
|---|---|
|T-SNAP-1|Widerruf nur für Snapshots des Mandanten (D-61); `min_read_snapshot` ohne Sentinel (I-12)|
|T-VER-1|Zeitreise/Snapshot liefert ab Gate `Erased`; widerrufene Handles liefern `SnapshotRevoked`|
|T-CONC-1|`loom`: Registry, Sequenzvergabe, Barrieren (zusammen mit `store`)|
|T-LIMIT-1|Snapshot-Grenze 1024|
|T-RESTORE-3|Sequenz monoton über Restores (I-20)|
|Property|`proptest` gegen `ReferenceModel`: Sichtbarkeit, SSI-Konflikte|
|Miri|`miri` auf `mvcc` (SPEC 15.1)|

## 9. Abnahme

Definition of Done (Index Abschnitt 8); Grep `u64::MAX` in `mvcc` nur in Konstanten ohne Sentinelrolle (Review-Liste im PR); T-SNAP-1 und Loom-Tests grün; Miri sauber.

## 10. Offene Punkte

- Wo die Tombstone-Auflösung für `get` endet (Version mit `vlen = 0` als „gelöscht“) legt `store` fest; `mvcc` kennt nur Sequenzen.
- `revoke_covering` erhält einen `Matcher` aus `types`; ob die Registry zusätzlich je Snapshot den berührten Bereich speichert (für gezielten Widerruf) oder alle Snapshots des Mandanten mit `seq ≤ gated_seq` widerruft, ist eine Entscheidung für WP-5.1. D-61 sagt „die ein Ziel berühren“, T-SNAP-1 prüft den Mandantenbezug; der strengere (alle des Mandanten mit `seq ≤ gated_seq`) ist einfacher und fail-closed.
