# Phase D: Geschäftslogik und forensische Sicherheitsüberprüfung

## D1. Mandantentrennung & Globale Datenlecks (INV-TENANT-1, INV-TENANT-IDX-1)
- **BM25 / IDF Globale Aggregate**:
  - In `crates/contextra-text/src/inverted/index_struct.rs` wird `avg_doc_len_x1000: Arc<AtomicU64>` als globales Aggregat gepflegt.
  - *Befund (S0)*: Wenn Dokumente mehrerer Mandanten im selben Volltextindex verarbeitet werden, fließen die Dokumentenlängen und TF/IDF-Korpuskennzahlen aller Mandanten in eine gemeinsame globale Statistik ein. Ein Angreifer kann über geringfügige Score-Veränderungen bei BM25-Anfragen das Vorhandensein, die Anzahl und die durchschnittliche Länge von Dokumenten anderer Mandanten rekonstruieren (Seitenkanal/Datenleck).

---

## D2. ACL-Gefilterte Suche vor Distanzberechnung (§9.4, M3)
- In `crates/contextra-vector/src/hnsw/core_search.rs` führt die HNSW-Suche bei großer Treffermenge ACORN-artige Graph-Traversierungen aus.
- *Befund (S2)*: Bei exakten SIMD-Scans wird der ACL-Filter vorab geprüft, jedoch existieren im Graph-Traversierungspfad Fallbacks, bei denen Distanzberechnungen für Nachbarknoten ausgeführt werden, bevor deren ACL-Berechtigung verifiziert ist (Timing-Seitenkanal bezüglich Existenz nicht-berechtigter Vektoren).

---

## D3. Schreibpfad, Atomizität & Error Handling (INV-ATOMIC-IDX-1, INV-WAL-SSOT-1)
- **Fehlerbehandlung in Ring 0 & 1**:
  - In Produktion (ohne Tests/Benches) wurden **2.776 `.unwrap()`**, **1.426 `.expect()`** und **121 `panic!()`** Aufrufe identifiziert.
  - *Befund (S1)*: Alleine `contextra-store` enthält **471 `unwrap()`** und **749 `expect()`** Aufrufe. Ein I/O-Fehler, Plattenplatzmangel oder korrumpiertes Bit auf dem Schreibpfad führt zu einem unkontrollierten Thread-/Prozess-Crash (`panic!`) anstelle eines geregelten Fail-Closed `Result::Err`.

---

## D4. Start- & Recovery-Sequenz B0–B9 (§22.1)
- In `crates/contextra-engine/src/contextra_impl/lifecycle.rs` wird beim Datenbankstart die Recovery über WAL und LSM-Store initiiert.
- *Befund (S1)*: Die streng geforderte 10-stufige B0–B9 Validierungssequenz (mit expliziter Prüfung der Sidecar-Ketten-Integrität und Vorab-Fence-Validierung vor dem ersten Lesezugriff) wird im Initialisierungspfad nicht als atomare B0–B9 Pipeline durchgehalten, sondern teilweise in Hintergrund-Reaper verschoben.

---

## D5. Löschpfad, Proof Pipeline & Stubs (INV-SHRED-1, INV-PROOF-COVERAGE-1)
- **Stub-Identifikation**:
  - In `crates/contextra-crypto/src/deletion_proof.rs:1537`: `LayerCleanupProof::verify_and_create(DeletionLayer::HnswIndex, || Ok(true));`
  - In `crates/contextra-crypto/src/kv_shredding.rs:526` & `565`: Funktionen geben bedingungslos `Ok(())` zurück.
  - *Befund (S0)*: Der Löschnachweis-Builder verlässt sich für HNSW-Vektorindex-Bereinigungen auf einen Stub, der unabhängig vom tatsächlichen Bereinigungszustand des Index `Ok(true)` zurückgibt. Ein `DeletionProof` behauptet kryptografisch die vollständige Bereinigung aller 8 Schichten, obwohl die Vektorindex-Schicht nicht physisch bereinigt, sondern per Stub als erfolgreich quittiert wurde.

---

## D6. Prompts, Forget-Guard & Tool-Egress (§21.2–21.6)
- In `crates/contextra-mcp` ist die Guardrail-Pipeline integriert.
- *Befund (S2)*: Der Bestätigungstoken für `memory_forget` wird im `dry_run`-Modus generiert, jedoch fehlt die kryptografische Bindung des Tokens an die Session-ID des Aufrufers, wodurch Cross-Session Token-Replay theoretisch möglich ist.

---

## D7. Nichtdeterminismus (INV-NONDET-1, P28)
- In `crates/contextra-ports/src/rng.rs:8` existiert `static ENTROPY_COUNTER: AtomicU64 = AtomicU64::new(1);`.
- In `crates/contextra-vector/src/hnsw/core_rebuild.rs` werden direkte System-Clock-Aufrufe (`Instant::now()`) getätigt.
- *Befund (S1)*: Der Einsatz eines globalen veränderlichen Atomic-Counters zur Entropieerzeugung sowie direkter Systemuhraufrufe verletzt `INV-NONDET-1`, wonach sämtlicher Zufall und Zeitstempel aus deterministischen Ports stammen müssen.

---

## D8. Transaktionen, SSI & Nebenläufigkeit (INV-SSI-ACTIVE-1, INV-LOCK-ORDER-1)
- Es wurden **172 Mutex/RwLock-Sperrpunkte** in `store` und `engine` analysiert.
- *Befund (S1)*: An mehreren Stellen im Transaktions-Commit-Pfad werden `tokio::sync::Mutex` Güter erworben und über Async-Await-Punkte hinweg gehalten (`commit_mutex`, `flush_mutex`), was unter hoher Nebenläufigkeit zu Thread-Starvation und Deadlocks führen kann.

---

## D9–D10. Serialisierung & Zero-Panic Politik (INV-FORMAT-1, §3.1)
- Persistent gespeicherte Datenstrukturen verwenden teilweise `bincode` ohne explizite Versionspräfixe am Dateianfang.
- *Befund (S1)*: Der Verstoß gegen die Zero-Panic-Policy (über 4.000 `unwrap`/`expect` im Produktionscode) widerspricht direkt REQ-GOV / §3.1.

---

## D11. Unsafe Code & Unsafe-Inseln (§6.7, P2)
- Im Gesamten Workspace wurden **388 `unsafe` Schlüsselwörter** gefunden.
- Distribution:
  - `contextra-simd`: 310
  - `contextra-sys`: 33
  - `contextra-wire`: 45
- *Befund (Eingehalten / Positive Validierung)*: Der `unsafe` Code verteilt sich **exakt auf die drei genehmigten Unsafe-Inseln** (`simd`, `sys`, `wire`). Außerhalb dieser drei Inseln existiert **kein einziger `unsafe`-Block** im gesamten Workspace.

---

## D12. Gates & Governance-Wirksamkeit (§27.5)
- In `xtask` sind diverse Gates implementiert.
- *Befund (S2)*: Gates wie `cargo xtask check-claims` basieren teilweise auf Regex-Matching gegen Whitelists. Das Fehlen von CI-Selbsttests gegen synthetische Fehler erzeugt das Risiko von "False Greens" (Gates signalisieren Erfolg, obwohl Invarianten verletzt sind).
