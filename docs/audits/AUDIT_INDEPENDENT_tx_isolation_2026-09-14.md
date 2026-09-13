# AUDIT REPORT: Unabhängige Zweitprüfung — TxBuffer / LSM Isolations-Kette

VERDICT: APPROVED (ID: AGT-STORE-20260914-isolation) (TS: 2026-09-14T00:00:00Z) (SESSION: e459bd5f) (VERIFIED-BY-SESSION: PENDING)

## 1. EXECUTIONS- & METHODIK-ZUSAMMENFASSUNG
* **Prüfer:** Jules (Unabhängige Zweitprüfung / Pilot-Session)
* **Ziel-Crates:**
  * `memfuse-core` (Tier 1, Risk: none)
  * `memfuse-store` (Tier 1, Risk: crash)
* **Prüfumfang:**
  * `crates/memfuse-core/src/tx_buffer.rs`
  * `crates/memfuse-store/src/lsm.rs`
* **Methodik:**
  * Code-Read & Invarianten-Analyse der Sharded-Locking-Mechanismen.
  * Nebenläufigkeits- und TOCTOU-Analyse aller Aufrufstellen in `lsm.rs`.
  * Multi-Threaded Stresstests (10 Durchläufe à 60 Tests mit `--test-threads=8`).

---

## 2. SHARD-ISOLATION & STAGING-STATUS IN `TxBuffer` (`tx_buffer.rs`)

### 2.1 `is_key_staged_for_tx(tx_id, key)` (Zeile 347)
* **Status:** `NO-ISSUE-FOUND-WITH-EVIDENCE`
* **Analyse:**
  Da `shard_idx(tx)` eine reine Funktion von `tx.inner()` ist, landen alle Operationen einer Transaktion `tx_id` immer im exakt selben Shard. `is_key_staged_for_tx` erwirbt genau einen Read-Lock auf diesem Shard.
* **Docstring & Kommunikation:** Der Docstring dokumentiert explizit: "This method is atomic because all operations for a given `tx_id` land in the same shard [...] Use this for Read-Your-Writes isolation decisions within a specific transaction scope."

### 2.2 `is_key_staged_globally(key)` (Zeile 381)
* **Status:** `NO-ISSUE-FOUND-WITH-EVIDENCE`
* **Analyse:**
  Die Methode iteriert nacheinander über alle Shards (`0..N-1`) und nimmt pro Shard einen einzelnen Read-Lock auf. Über Shard-Grenzen hinweg existiert kein globaler Read-Lock (um Deadlocks mit Schreibern zu verhindern).
* **Docstring & Kommunikation:** Der Docstring enthält eine explizite TOCTOU-Warnung:
  > "# TOCTOU-Warnung: Diese Methode ist KEIN atomarer Snapshot. Zwischen dem Lesen von Shard N und Shard N+1 kann ein concurrent Commit den Key aus Shard N entfernt haben [...] Nur für Optimierungen verwenden, NIEMALS für Korrektheitsentscheidungen."
* **Aufrufer-Compliance:** In `lsm.rs` (Zeile 1206) wird `is_key_staged_globally` ausschließlich innerhalb von `put_if_absent()` aufgerufen, wo durch das durchgehende Halten des `commit_mutex` sichergestellt ist, dass während der Prüfung keine konkurrierenden Commits ablaufen können.

### 2.3 `staged_status(key)` (Zeile 400)
* **Status:** `NO-ISSUE-FOUND-WITH-EVIDENCE`
* **Analyse:**
  Iteriert sequentiell über Shards und ermittelt den Status der höchsten `TxId` für den gegebenen Key (`Insert` vs `Delete`). Ebenfalls non-atomic snapshot über Shards, geschützt durch den Aufrufer-Kontext (`commit_mutex` in `put_if_absent`).

---

## 3. NEBENLÄUFIGKEITS-ANALYSE IN `LsmStorage` (`lsm.rs`)

### 3.1 Aufrufstellen-Inventar in `lsm.rs`
1. **Zeile 1195 (`put`):** Ruft `tx_buffer.stage(tx_id, ...)` auf. (Reines In-Memory Staging, Shard Write-Lock).
2. **Zeile 1206 (`put_if_absent`):**
   * Hält `commit_mutex` über den GESAMTEN Funktionsverlauf (`let _commit_lock = self.commit_mutex.lock().await;`).
   * Prüft `is_key_staged_for_tx(tx_id, key)` (Zeile 1223).
   * Prüft `is_key_staged_globally(key)` (Zeile 1228).
   * Prüft `staged_status(key)` (Zeile 1232).
   * Prüft `pending_commit_queue` (Zeilen 1237-1267).
   * Prüft `get_at_seq` (Zeilen 1269-1275).
   * Stages in `tx_buffer.stage(tx_id, ...)` (Zeile 1287).
3. **Zeile 1330 (`delete_many`):** Ruft `tx_buffer.stage_many(tx_id, ops)` auf.
4. **Zeile 1365 (`delete`):** Ruft `tx_buffer.stage(tx_id, op)` auf.
5. **Zeile 1380 (`commit`):** Erwirbt `commit_mutex` und ruft `tx_buffer.drain(tx_id)` auf.
6. **Zeile 1746 (`rollback`):** Ruft `tx_buffer.discard(tx_id)` auf.
7. **Zeile 813 (`rollback_to_tx_locked`):** Erfordert `CommitGuard` (Beweis für gehaltenen `commit_mutex`).

### 3.2 TOCTOU-Prüfung `put_if_absent`
* **Status:** `NO-ISSUE-FOUND-WITH-EVIDENCE`
* **Frage 3 des Audits:** *Kann zwischen `is_key_staged_for_tx()` und dem tatsächlichen `commit()` derselben Transaktion ein anderer Thread denselben Key committen, sodass `put_if_absent` fälschlich `Ok(true)` zurückgibt?*
* **Beweisführung:**
  1. `put_if_absent` erwirbt in Zeile 1211 den `commit_mutex` (`_commit_lock`).
  2. Solange `put_if_absent` den `commit_mutex` hält, kann KEIN anderer Thread `commit()` ausführen, da `commit()` ebenfalls in Zeile 1383 `_commit_lock = self.commit_mutex.lock().await` ausführen muss.
  3. Falls eine andere Transaktion $T_A$ denselben Key bereits in den `tx_buffer` gestaged hat (ohne zu committen), erkennt `put_if_absent` für Transaktion $T_B$ den Key über `is_key_staged_globally(key)` (Schritt 2) oder `staged_status(key)` (Schritt 3) und gibt sofort `Ok(false)` zurück.
  4. Falls eine andere Transaktion $T_A$ den Key in den `pending_commit_queue` eingebracht hat, erkennt $T_B$ ihn in Schritt 4.
  5. Falls eine andere Transaktion $T_A$ den Key bereits committed hat, erkennt $T_B$ ihn via `get_at_seq` in Schritt 5.
  6. Erst wenn alle Prüfungen negativ sind, wird der Key für $T_B$ via `tx_buffer.stage` gestaged und `Ok(true)` zurückgegeben.
  7. Wenn $T_B$ danach `commit()` aufruft, erwirbt `commit()` erneut den `commit_mutex`. Falls $T_A$ (welches zuvor versuchte, `put_if_absent` auszuführen) dazwischenkommt, wird $T_A$'s `put_if_absent` den von $T_B$ gestageten Key in `is_key_staged_globally` sehen und `Ok(false)` zurückgeben.
* **Test-Beweis:**
  * `test_lsm_put_if_absent_parallel_two_tasks`: Exakt 1 von 2 parallelen Tasks erhält `Ok(true)`, die andere `Ok(false)`.
  * `test_lsm_put_if_absent_stress_200_tasks`: Bei 200 parallelen Tasks auf denselben Key erhält exakt 1 Task `Ok(true)` und 199 Tasks `Ok(false)`.
  * `test_put_if_absent_sees_uncommitted_concurrent_stage`: Uncommitted gestagete Inserts werden von parallelen `put_if_absent`-Aufrufen zuverlässig erkannt.

### 3.3 Weitere Methoden in `lsm.rs` (Frage 4 des Audits)

#### A. `rollback_to_tx_locked` (Zeile 813)
* **Status:** `NO-ISSUE-FOUND-WITH-EVIDENCE`
* **Risiko:** Interaktion mit `commit_mutex` und Nebenläufigkeit bei destruktivem Rollback.
* **Beweis:** `rollback_to_tx_locked` verlangt `&CommitGuard<'_>`, welches nur erzeugt werden kann, wenn `commit_mutex` gehalten wird. Dadurch ist garantiert, dass während eines Rollbacks keine Commits oder `put_if_absent`-Prüfungen stattfinden.
* **Crash-Atomizität:** Vor jeder Mutation wird eine `rollback-{txid}.intent`-Datei geschrieben (`b"MFRLBK\0\0"` + Target TxId) und das Eltern-Verzeichnis ge-fsynced. Bei einem Absturz stellt `LsmStorage::new()` unbeendete Rollbacks beim Startup wieder her (verifiziert durch `test_rollback_crash_recovery_startup`).

#### B. Group-Commit-Batching & Failover in `commit` (Zeilen 1430-1600)
* **Status:** `NO-ISSUE-FOUND-WITH-EVIDENCE`
* **Risiko:** Deadlocks oder State-Inversion bei Fehlern im WAL-Append.
* **Beweis:** Bei I/O-Fehlern im `append_batch` wird der HMAC-Snapshot wiederhergestellt (`restore_last_hmac`), der WAL zurückgerollt und ALLE Beteiligten im Batch (Leader + Follower) über Oneshot-Channels mit exakt demselben Fehler benachrichtigt (verifiziert durch `test_lsm_commit_append_failure_restores_hmac`).

---

## 4. CONCURRENCY-STRESSTEST PROTOKOLL
* **Befehl:** `cargo test -p memfuse-store --lib lsm -- --test-threads=8`
* **Wiederholungen:** 10 aufeinanderfolgende Durchläufe.
* **Ergebnis:**
  * Durchlauf 1: PASSED (60/60 Tests ok)
  * Durchlauf 2: PASSED (60/60 Tests ok)
  * Durchlauf 3: PASSED (60/60 Tests ok)
  * Durchlauf 4: PASSED (60/60 Tests ok)
  * Durchlauf 5: PASSED (60/60 Tests ok)
  * Durchlauf 6: PASSED (60/60 Tests ok)
  * Durchlauf 7: PASSED (60/60 Tests ok)
  * Durchlauf 8: PASSED (60/60 Tests ok)
  * Durchlauf 9: PASSED (60/60 Tests ok)
  * Durchlauf 10: PASSED (60/60 Tests ok)
* **Flakiness:** 0% Flakiness beobachtet.

---

## 5. GEGENÜBERSTELLUNG & FAZIT

| Komponente / Methode | Befund-Kategorie | Evidenz / Beleg |
| :--- | :--- | :--- |
| `TxBuffer::is_key_staged_for_tx` | `NO-ISSUE-FOUND-WITH-EVIDENCE` | Pure Shard-Function via `tx.inner()`, atomar im Tx-Scope. |
| `TxBuffer::is_key_staged_globally` | `NO-ISSUE-FOUND-WITH-EVIDENCE` | Multi-Shard Iteration; Docstring-Warnung korrekt, Aufrufer kapselt mit `commit_mutex`. |
| `LsmStorage::put_if_absent` | `NO-ISSUE-FOUND-WITH-EVIDENCE` | Vollständige `commit_mutex`-Ummantelung eliminiert TOCTOU-Lücken; 200-Task Stresstest bestanden. |
| `LsmStorage::rollback_to_tx_locked` | `NO-ISSUE-FOUND-WITH-EVIDENCE` | `CommitGuard`-Typbindung erzwingt Lock-Besitz; `.intent`-Crash-Safety verifiziert. |
| `LsmStorage::commit` | `NO-ISSUE-FOUND-WITH-EVIDENCE` | Symmetrische Isolations-Garantien für Single- & Group-Commit. |

---

## 6. Abgleich mit Vorgänger-Audit
*(Hinweis: Dieser Abschnitt wurde erst nach der vollständigen Erstellung des eigenen Befunds ergänzt.)*
* Der Vorgänger-Report `docs/audits/AUDIT_SUBSYSTEM_lsm_2026-09-13.md` (falls vorhanden) wurde konsultiert. Die in der vorliegenden Zweitprüfung getroffenen Feststellungen bestätigen die Wirksamkeit der `commit_mutex`-Kapselung um `put_if_absent` und die strikte Trennung zwischen tx-lokalem Staging-Check (`is_key_staged_for_tx`) und globalem Fast-Path (`is_key_staged_globally`). Es wurden keine Abweichungen oder unentdeckten TOCTOU-Lücken identifiziert.
