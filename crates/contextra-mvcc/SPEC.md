# `contextra-mvcc` — Normative Spezifikation (SPEC v4.0)

> **Revision R12** (Repo-Abgleich, HEAD `60ea034`): Änderungen gemäß `SPEC.md` K-58…K-63 und Anhang B.5. Abschnitte mit Kennzeichnung „R12“ sind normativ.

> **Normativitätsklasse:** Klasse B (Storage-Bundle)
> **Status:** Übergang (R12, K-40/K-58): `stable` bis G-CUTOVER, danach eingefroren. Kernpfade des neuen Speicherkerns verwenden diese Crate nicht.
> **Sicherheitsstufe:** `#![forbid(unsafe_code)]`, Zero-Panic

---

## 1. Zweck und Systemrolle

`contextra-mvcc` stellt die fundamentale Transaktions- und Isolationsschicht (Ring 0) des Contextra-Baukastens bereit. Sie umfasst:
1. **Multi-Version Concurrency Control (MVCC)**: Zeitreihen- und Versionsverfolgung von Datenversionen via [`SequenceLog`].
2. **Snapshot-Isolation**: Aktive Snapshot-Registrierung und lock-freie Ermittlung minimaler aktiver Sequenzen (`min_active_seqno`) via [`SnapshotRegistry`].
3. **Serializable Snapshot Isolation (SSI)**: In-Memory-Konflikterkennung zur Prävention von Write-Skew-Anomalien und Phantomen via [`ReadSet`] und [`SequenceLogSsiValidator`].
4. **Partitioniertes Transaktions-Staging**: 2-Phase-Commit Staging-Puffer ([`TxBuffer`]) mit strikten Speicherobergrenzen (DoS-Schutz).

---

## 2. Invarianten und Verträge

- **INV-LOCK-ORDER-1 (Sperrenordnung):**
  Die Sperrenhierarchie ist total geordnet:
  `Dokumentlock -> TxBuffer Shards (0..N-1 sequentiell) -> SnapshotRegistry -> SequenceLog / SSI (CommittedWrites)`.
  Es werden niemals Sperren über `.await` gehalten (Crate ist rein synchron und frei von Async-Runtimes).
- **INV-MVCC-SSI-1 (Deterministische SSI-Validierung):**
  Konflikterkennung basiert rein auf Lese-Snapshot-Sequenzen und Commit-Sequenzen. Gleiche Eingaben erzeugen identische Entscheidungen.
- **INV-INDEX-SNAPSHOT-1 (Sichtbarkeitsinvariante):**
  Ein Eintrag ist zu Snapshot-Sequenz `as_of` genau dann sichtbar, wenn:
  `insert_seq <= as_of && (delete_seq.is_none() || delete_seq > as_of)`.
- **INV-NO-SENTINEL (Kein Sentinel-Leck):**
  `min_read_snapshot()` liefert `Option<u64>` (niemals `u64::MAX`). Sentinel-Werte und gesetzte Tombstone-Bits in `prune_through` werden abgewiesen.
- **INV-TENANT-ISOLATION (Mandanten-Isolation):**
  Widerruf von Snapshots via `revoke_covering(tenant, range, seq)` widerruft ausschließlich Snapshots des Zielmandanten.

---

## 3. Storage-Bundle-Kanten & Schnittstellen

Als Klasse-B-Baustein teilt `contextra-mvcc` gemeinsame Speichertypen innerhalb des Storage-Bundles:
- **`contextra-types`**: `Epoch` (R12, K-59; ersetzt `TxId`), `DocId`, `TenantId`, `ContextraError`, `Result`, `TOMBSTONE_BIT`.
- **`contextra-ports`**: Übergangs-Anbindung für `Clock` und `MetricsSink`.
- **State-Passing & Closure-Abstraktion (WP-4)**:
  Alle Komponenten bieten zeitstempelbasierte Signaturen (`*_at(now_nanos)`) und Closure-Konstruktoren (`with_time_fn`), wodurch Port-Imports für Zeiterfassung und Metriken optional entkoppelt sind.

---

## 4. Bounded Memory & Kapazitätsgrenzen

| Parameter | Standardwert | Bedeutung |
|---|---|---|
| `DEFAULT_SHARD_COUNT` | 64 | Partitionen des Transaktionspuffers |
| `DEFAULT_MAX_OPS_PER_TX` | 10.000 | Obergrenze gepufferter Operationen pro Transaktion |
| `DEFAULT_MAX_TX_STAGED_BYTES` | 16 MiB | Byte-Budget pro Einzelschreibtransaktion |
| `DEFAULT_MAX_TOTAL_STAGED_BYTES` | 256 MiB | Globales Byte-Budget über alle Transaktionen |
| `DEFAULT_MAX_TRACKED_COMMIT_KEYS` | 1.000.000 | Maximale Anzahl getrackter Commit-Schlüssel (Coarsening ab 80 %) |
| `DEFAULT_MAX_PIN_DURATION` | 300 s | Schwellenwert für Warnungen über verwaiste Pins |

## 5. R12-Ergänzungen

- **Keine Kante aus Retrieval-Crates (K-40):** `contextra-vector` entfernt seine optionale Abhängigkeit auf `contextra-mvcc`. Snapshot- und SSI-Logik der Retrieval-Crates läuft über `Epoch`-Sichten des Speicherkerns.
- **Übernahme in den Speicherkern:** Sequenzlog, Snapshot-Registry und SSI-Validierung werden in `contextra-storage` (Stamp/Epoch-Modell) überführt. Diese Crate liefert dazu nur Referenzverhalten und Differenztests (`INV-MVCC-SSI-1` als Testorakel).
- **INV-LOCK-ORDER-1** gilt unverändert, bis die Übernahme abgeschlossen ist.
