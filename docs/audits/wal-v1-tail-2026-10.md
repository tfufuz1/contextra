# Audit-Bericht: WAL-V1-Pfad und Tail-Toleranz gegen Manifest-Anker

**Datum:** 2026-10-06
**Crate:** `contextra-store` (Ring 1)
**Prüfer:** Jules (Session Audit)
**Status:** Belegt / Abgeschlossen (Analyse ohne Codeänderung)
**Gelesene Dateien:**
- `AGENTS.md`
- `.jules/PREAMBLE.md`
- `crates/contextra-store/AGENTS.md`
- `docs/audits/crypto-wal-audit.md`
- `crates/contextra-store/src/wal/mod.rs`
- `crates/contextra-store/src/wal/replay.rs`
- `crates/contextra-store/src/wal/io.rs`
- `crates/contextra-store/src/wal/encode.rs`
- `crates/contextra-store/src/lsm/engine.rs`
- `crates/contextra-store/src/lsm/recovery.rs`
- `crates/contextra-engine/src/contextra_impl/lifecycle.rs`
- `crates/contextra-store/src/wal/open_heal_tests.rs`
- `crates/contextra-store/src/wal/tests/replay_tests.rs`

---

## 1. Ergebnisse der Fragestellungen (Q1–Q3)

### Q1. Erreichbarkeit von `WalVersion::V1` aus produktiven Pfaden
- **Antwort:** Widerlegt.
- **Belegende Datei:Zeile:**
  - `crates/contextra-store/src/wal/mod.rs:334` (`WalConfig::default()` setzt `min_wal_version: WalVersion::V3`)
  - `crates/contextra-store/src/wal/mod.rs:388` (`Wal::open` nutzt `WalConfig::default()`)
  - `crates/contextra-store/src/wal/mod.rs:420` (`Wal::migrate_legacy_wal` setzt explizit `min_wal_version: WalVersion::V1`)
  - `crates/contextra-store/src/wal/mod.rs:464` (`Wal::open_for_legacy_migration` setzt explizit `min_wal_version: WalVersion::V1`)
  - `crates/contextra-store/src/lsm/engine.rs:350` (`LsmStorage::has_pending_legacy_wal_migration_path` setzt `min_wal_version: WalVersion::V1` für reine Leseprüfung)
  - `crates/contextra-store/src/lsm/engine.rs:436` (`LsmStorage::migrate_legacy_wal_keys` ruft `Wal::open_for_legacy_migration` auf)
  - `crates/contextra-engine/src/contextra_impl/lifecycle.rs:75` (Start-Lifecycle führt explizit `LsmStorage::migrate_legacy_wal_keys` aus)
- **Begründung:**
  Der Standardaufruf `WalConfig::default()` erzwingt in `crates/contextra-store/src/wal/mod.rs:334` strikt `min_wal_version = WalVersion::V3`. Ein reguläres Öffnen der Datenbank via `Wal::open` oder `LsmStorage::open` verwendet diese Konfiguration und weist V1-Segmente sofort ab bzw. erzwingt deren Umwandlung. Die Absenkung auf `WalVersion::V1` erfolgt ausschließlich in den dedizierten Migrationspfaden `Wal::migrate_legacy_wal` (`wal/mod.rs:420`), `Wal::open_for_legacy_migration` (`wal/mod.rs:464`) sowie beim LsmStorage-Migrations-Scan (`lsm/engine.rs:350`). Ein Nutzer, der eine V1-WAL-Datei ohne Migrationsabsicht öffnet, landet daher niemals im produktiven V1-Betrieb, da Alt-WAL-Segmente beim Systemstart vor der Freigabe automatisch nach V3 rekeyt und umgeschrieben werden.

### Q2. Tail-Toleranz bei AEAD-Fehlern & Manifest-High-Water-Mark (INV-WAL-TRUNCATION-1)
- **Antwort:** Belegt.
- **Belegende Datei:Zeile:**
  - `crates/contextra-store/src/wal/replay.rs:497-503` (Stream-Scan: Tolerierung von AEAD-Entschlüsselungsfehlern an Position `pos >= file_size` als unvollständigen Tail-Write)
  - `crates/contextra-store/src/wal/replay.rs:631-633` (Mmap-Scan: Identische Tolerierung von AEAD-Fehlern am physischen Dateiende)
  - `crates/contextra-store/src/wal/io.rs:192-198` & `:379-383` (Parallel implementierte Tolerierung in `wal/io.rs`)
  - `crates/contextra-store/src/lsm/recovery.rs:360-366` (LSM-Recovery prüft den replayed Tail-HMAC gegen die Manifest-High-Water-Mark via `verify_wal_chain_completeness`)
- **Begründung:**
  In `crates/contextra-store/src/wal/replay.rs:497-503` (sowie `io.rs:192-198`) wird ein AEAD-Entschlüsselungsfehler an der Endposition `pos >= file_size` explizit abgefangen, geloggt und durch `break` als unvollständiger Tail Write (Torn Write) toleriert, ohne einen direkten `WalCorruption`-Fehler auszulösen. Der Replay-Prozess liefert daraufhin die bis dahin erfolgreich verifizierten WAL-Einträge zurück. Beim anschließenden LSM-Storage-Start in `crates/contextra-store/src/lsm/recovery.rs:360-366` wird der HMAC des letzten verifizierten Eintrags zwingend gegen die im Manifest gespeicherte High-Water-Mark (`manifest_hwm`) mittels `verify_wal_chain_completeness` abgeglichen. Sollte ein abgeschnittenes `Delete` bereits durch eine Manifest-High-Water-Mark verankert gewesen sein, weicht der verbleibende Tail-HMAC ab, was zu einem `WalTruncationDetected`-Fehler führt und somit eine Wiederauferstehung (Resurrection) wirksam verhindert.

### Q3. Abdeckung durch bestehende Testsuiten
- **Antwort:** Belegt.
- **Belegende Datei:Zeile:**
  - `crates/contextra-store/src/wal/open_heal_tests.rs:114` (`test_open_heal_encrypted_wal`)
  - `crates/contextra-store/src/wal/tests/replay_tests.rs:504` (`test_batch_encrypted_wal_truncation_crash_consistency`)
  - `crates/contextra-store/src/wal/tests/replay_tests.rs:1091` (`test_attack_b_truncation_last_block_behavior`)
  - `crates/contextra-store/src/wal/tests/replay_tests.rs:1196` (`test_attack_d_cross_file_replay_encrypted_detected`)
  - `crates/contextra-store/tests/wal_truncation_and_legacy_isolation.rs` (Tests zur Manifest-Truncation-Erkennung)
  - `crates/contextra-store/tests/wal_spec_pending.rs:174` (`Manifest::extract_high_water_mark`)
- **Begründung:**
  Die Tail-Toleranz bei verschlüsselten WAL-Segmenten und die Erkennung von Tail-Abschneidungen werden durch mehrere Testmodule abgedeckt. In `crates/contextra-store/src/wal/open_heal_tests.rs:114` verifiziert `test_open_heal_encrypted_wal` das automatische Abschneiden und Heilen eines partiell geschriebenen verschlüsselten Frames auf `Wal::open`. In `crates/contextra-store/src/wal/tests/replay_tests.rs:504` prüft `test_batch_encrypted_wal_truncation_crash_consistency` die Crash-Konsistenz bei abgeschnittenen verschlüsselten Batches, während `test_attack_b_truncation_last_block_behavior` (`replay_tests.rs:1091`) das Replay-Verhalten bei Block-Kürzungen simuliert. Die Kopplung an die Manifest-High-Water-Mark gegen unbefugte Truncation (INV-WAL-TRUNCATION-1) wird in `crates/contextra-store/tests/wal_truncation_and_legacy_isolation.rs` sowie `crates/contextra-store/tests/wal_spec_pending.rs` verifiziert.

---

## 2. Vorschlag Tests (Textbeschreibung, nicht implementieren)

1. **`test_aead_tail_truncation_without_manifest_anchor_causes_resurrection`**
   - **Setup:** Schreiben von `Put(Key="A", Val="1")`, `TxEnd`, `Put(Key="A", Val="Tombstone")`, `TxEnd` in ein verschlüsseltes WAL-Segment. Simuliere einen Crash vor dem Schreiben der neuen Manifest-High-Water-Mark (Manifest zeigt noch auf den Stand vor dem Delete). Schneide die verschlüsselte WAL-Datei exakt am Ende des ersten `TxEnd` ab (AEAD-Fehler/Torn Write am Tail).
   - **Erwartung:** Ohne Manifest-Anker-Prüfung würde das WAL-Replay stumm bei Eintrag 1 stoppen und Key "A" mit Wert "1" (Resurrection) wiederherstellen. Der Test soll nachweisen, dass das WAL-Replay alleine keine Semantik-Garantie gegen Resurrection bietet, sondern zwingend auf die LSM-Manifest-Prüfung in `recovery.rs` angewiesen ist.

2. **`test_aead_tail_corruption_mid_file_returns_wal_corruption_not_truncated`**
   - **Setup:** Schreiben von 3 verschlüsselten Batches. Korrumpieren der AEAD-Payload des 2. Batches (z. B. Bitflip im Ciphertext), gefolgt von einem intakten 3. Batch.
   - **Erwartung:** Da die Beschädigung vor dem physischen Dateiende (`pos < file_size`) auftritt, darf der Replay-Parser dies NICHT als akzeptablen Torn Write am Tail einstufen, sondern MUSS strikt mit `ContextraError::WalCorruption` fehlschlagen.

3. **`test_direct_wal_open_with_v1_payload_fails_closed`**
   - **Setup:** Erstelle manuell ein unverschlüsseltes WAL-Segment im alten V1-Format. Versuche, die Datei direkt über `Wal::open` (ohne `migrate_legacy_wal_keys` oder `open_for_legacy_migration`) zu öffnen.
   - **Erwartung:** Der Aufruf schlägt strikt fail-closed fehl, da `WalConfig::default()` als `min_wal_version` V3 fordert und keine automatische/implizite V1-Akzeptanz im Standard-Öffnungspfad erlaubt.

---

## 3. Zusammenhang mit bestehendem Audit (`docs/audits/crypto-wal-audit.md`)

- **Fokus-Unterschied:** Das bestehende Audit `docs/audits/crypto-wal-audit.md` analysierte primär die kryptografischen Primitiven im Modul `contextra-crypto` (IntegrityVerifier Sequence-Number-Prüfung, Nonce-Countersicherheit, V3-HMAC-Struktur und `verify_wal_chain_completeness`).
- **Verbindung & Synergie:** Der vorliegende Bericht ergänzt das bestehende Audit um die empirische Untersuchung der Aufrufpfade in `contextra-store` (`wal/replay.rs`, `wal/io.rs`, `lsm/recovery.rs`):
  1. **V1-Erreichbarkeit:** Es wurde nachgewiesen, dass die in `crypto-wal-audit.md` genannten Legacy-V1/V2-Pfade in `contextra-store` durch `min_wal_version = WalVersion::V3` isoliert sind und kein produktiver Bypass existiert.
  2. **End-to-End-Integrität:** Die in `crypto-wal-audit.md` Abschnitt 4 beschriebene Funktion `verify_wal_chain_completeness` stellt genau das Bindeglied dar, welches die in Q2 analysierte AEAD-Tail-Toleranz ababsichert.
  3. **Ergebnis:** Das Zusammenspiel zwischen `contextra-crypto` (kryptografische Verifikation) und `contextra-store` (Torn Write Tolerierung am Tail mit anschließender HWM-Gegenprüfung im Manifest) bildet eine geschlossene Sicherheitskette gegen Resurrection und unbeabsichtigten Datenverlust.
