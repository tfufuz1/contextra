# Audit-Bericht: Krypto-Hotpath O(N)-Verifikation und Append (`contextra-crypto`)

**Datum:** 2026-10-08
**Crate:** `contextra-crypto` (Ring 0)
**Task-Karte:** `.jules/tasks/T-2026-0226.toml`
**Prüfer:** Jules (Security & Cryptography Audit)
**Status:** Reiner Analysebericht (keine Codeänderung)

---

## 1. Gelesene und auditierte Dateien (Evidence)

Die folgenden Quellcode- und Dokumentationsdateien wurden für diesen Analysebericht vollständig auditiert und belegt:
- `crates/contextra-crypto/AGENTS.md`
- `crates/contextra-crypto/src/deletion_proof/proof_v2.rs`
- `crates/contextra-crypto/src/deletion_proof/proof_v3.rs`
- `crates/contextra-crypto/src/deletion_proof/verify.rs`
- `crates/contextra-crypto/src/deletion_proof/types.rs`
- `crates/contextra-crypto/src/revocation_log.rs`
- `crates/contextra-crypto/src/wal_crypto.rs`
- `crates/contextra-crypto/src/wal_completeness.rs`
- `docs/audits/crypto-wal-audit.md`
- `docs/audits/hnsw-slots-entrypoint-2026-10.md`
- `docs/BENCHMARKS_DELETION_PROOF.md`
- `README.md`

---

## 2. Identifikation der Hotpath-Funktionen & Komplexitätsklassen

### 2.1 `DeletionProof::create` / `create_v3` (Beweiserzeugung)

- **Belegende Dateien & Zeilen:**
  - `crates/contextra-crypto/src/deletion_proof/proof_v2.rs:20-65` (`DeletionProof::create` / `create_with_wal_receipt`)
  - `crates/contextra-crypto/src/deletion_proof/proof_v3.rs:105-150` (`DeletionProof::create_full_v3`)
  - `crates/contextra-crypto/src/deletion_proof/types.rs:20-27` (`hash_deleted_keys_length_prefixed`)
- **Ablauf & Schleifen:**
  1. *Deteministische Sortierung:* `deleted_keys.sort()` in `proof_v2.rs:43` und `proof_v3.rs:118` führt eine In-Place-Sortierung der übergebenen Schlüssel-Vektoren aus ($O(K \log K)$ Vergleiche, wobei $K = |\text{deleted\_keys}|$).
  2. *Blake3 Hashing:* `hash_deleted_keys_length_prefixed(&deleted_keys)` in `types.rs:20-27` iteriert über alle $K$ Schlüssel, liest deren Länge und speist die Längen-Präfixe sowie die Schlüsselbytes in den `blake3::Hasher` ein ($O(M)$ Byte-Hashing, wobei $M = \sum |key_i|$ die Gesamtzahl der Schlüsselbytes ist).
  3. *Signaturerzeugung:* HMAC-SHA256 (v2) bzw. Ed25519-Signierung (v3) erfolgt über das strukturierte Payload, welches ausschließlich den 32-Byte-Hash `deleted_keys_hash` enthält ($O(1)$).
- **Komplexitätsklasse:** **$O(K \log K + M)$** bezogen auf die Anzahl $K$ der gelöschten Schlüssel und die Gesamtlänge $M$ der Schlüsselbytes. Bezüglich der Historie $N$ des RevocationLogs ist die Erzeugung **$O(1)$**, da kein Zugriff auf das RevocationLog erfolgt.
- **Lock-Typ & Haltezeit:** **Kein Lock (0 ns)**. Die Methode ist eine reintreffende, CPU-bound Funktion ohne Mutex oder RwLock.

---

### 2.2 `DeletionProof::verify` / `verify_external` (Beweisverifikation)

- **Belegende Dateien & Zeilen:**
  - `crates/contextra-crypto/src/deletion_proof/verify.rs:18-106` (`DeletionProof::verify`)
  - `crates/contextra-crypto/src/deletion_proof/verify.rs:152-171` (`DeletionProof::verify_external`)
  - `crates/contextra-crypto/src/deletion_proof/verify.rs:118-125` (`DeletionProof::verify_with_key_history`)
- **Ablauf & Schleifen:**
  1. *Struktur:* Das `DeletionProof`-Struct speichert selbst **nicht** die Liste aller gelöschten Schlüssel, sondern lediglich den vorgehaltenen 32-Byte-Hash `deleted_keys_hash` (`crates/contextra-crypto/src/deletion_proof/proof.rs`).
  2. *Verifikation:* `verify` (v2/v3) und `verify_external` (v3 Ed25519) serialisieren die Metadaten-Felder (Scope, Transaction ID, Covered Layers) und prüfen die HMAC-SHA256-Checksumme bzw. führen eine Ed25519-Kurvenpunktverifikation (`ed25519_dalek::Verifier::verify`) über das Payload durch.
  3. *Schlüsselhistorie:* `verify_with_key_history` iteriert sequentiell über eine Historie von $H$ Prüfschlüsseln ($O(H)$).
- **Komplexitätsklasse:** **$O(1)$** bezogen auf die Anzahl $K$ der gelöschten Schlüssel und **$O(1)$** bezüglich der RevocationLog-Größe $N$. Bezüglich der Schlüsselhistorie $H$ beträgt die Zeit **$O(H)$**.
- **Lock-Typ & Haltezeit:** **Kein Lock (0 ns)**. Die Verifikation arbeitet rein zustandslos auf lokalen Byte-Slices.

---

### 2.3 `RevocationLog::append` (Widerrufs-Append)

- **Belegende Dateien & Zeilen:**
  - `crates/contextra-crypto/src/revocation_log.rs:538-593` (`RevocationLog::append`)
  - `crates/contextra-crypto/src/revocation_log.rs:253-255` (`entries: RwLock<Vec<RevocationEntry>>`)
- **Ablauf & Schleifen:**
  1. *Exklusive Sperre:* `let mut entries_write_guard = self.entries.write();` in `revocation_log.rs:543` akquiriert die exklusive Schreibsperre auf dem `parking_lot::RwLock<Vec<RevocationEntry>>`.
  2. *Vollständiges Klonen in RAM:* `let mut prospective_entries = entries_write_guard.clone();` in `revocation_log.rs:563` kloniert den **gesamten Vector** aller bisherigen $N$ Einträge im Speicher. Dies ist eine $O(N)$ Speicherallokation und Speicher-Kopie.
  3. *Vollständige Serialisierung:* `let serialized = bincode::serialize(&prospective_entries)` in `revocation_log.rs:567` serialisiert die gesamte Kette von $N+1$ Einträgen via `bincode`. Dies ist ein $O(N)$ CPU-Arbeitsgang.
  4. *Blockierende Festplatten-E/A:* `write_atomic_file(path, &serialized_log)?;` in `revocation_log.rs:571` schreibt den gesamten $O(N)$ serialisierten Puffer auf die Festplatte (temporäre Datei + `atomic_replace` / `fsync`).
  5. *Marker-Schreiben & Lock-Release:* `write_atomic_file(&m_path, &serialized_marker)?;` schreibt die Marker-Datei, bevor `entries_write_guard` in `revocation_log.rs:589` freigegeben wird.
- **Komplexitätsklasse:** **$O(N)$** bezogen auf die Gesamtanzahl $N$ der Einträge im `RevocationLog`.
- **Lock-Typ & Haltezeit-Abschätzung:**
  - **Lock-Typ:** `parking_lot::RwLock` (exklusive Schreibsperre `RwLockWriteGuard<Vec<RevocationEntry>>`).
  - **Haltezeit:** Das `entries_write_guard` wird in Zeile 543 akquiriert und erst in Zeile 589 (nach dem Klonen, der `bincode`-Serialisierung und beiden blockierenden atomaren Dateisystem-Schreibvorgängen) freigegeben.
  - **Haltezeit-Abschätzung:** Bei wachsendem $N$ (z. B. $N = 10^5$ Einträge) skaliert die Sperrzeit linear $O(N)$. Das Halten der exklusiven Schreibsperre während synchroner Festplatten-E/A blockiert alle parallelen Leseabfragen (`is_revoked`, `len`, `is_empty`, `verify_integrity`) sowie alle parallelen `append`-Operationen vollständig für die gesamte Dauer der Festplatten-E/A.

---

### 2.4 `IntegrityVerifier` / `verify_wal_chain_completeness` (WAL-Integrität)

- **Belegende Dateien & Zeilen:**
  - `crates/contextra-crypto/src/wal_crypto.rs:191-285` (`IntegrityVerifier::verify_and_update_v3` / `v2`)
  - `crates/contextra-crypto/src/wal_completeness.rs:31-39` (`verify_wal_chain_completeness`)
- **Ablauf & Schleifen:**
  1. *`IntegrityVerifier`:* Verifiziert einen einzelnen `WalEntrySnapshot` bezüglich Sequenzmonotonie und berechnet den HMAC-SHA256 über den vorherigen HMAC und den aktuellen Eintrag ($O(1)$ pro WAL-Eintrag).
  2. *`verify_wal_chain_completeness`:* Führt einen 32-Byte Konstantzeitvergleich (`subtle::ConstantTimeEq`) zwischen `wal_tail_hmac` und `manifest_high_water_mark` durch ($O(1)$).
- **Komplexitätsklasse:** **$O(1)$** je Eintrag bzw. Vergleich. Unabhängig von der RevocationLog-Größe $N$.
- **Lock-Typ & Haltezeit:** **Kein Lock (0 ns)**. Die Operationen sind rein zustandslos bzw. mutieren lokale Struct-Felder.

---

## 3. Zusammenfassung der Komplexitätsklassen & Lock-Haltezeiten

| Funktion / Hotpath | Datei & Zeilen | Zeitkomplexität | Lock-Typ | Lock-Haltezeit (Skalierung) |
|---|---|---|---|---|
| **`DeletionProof::create` (v2/v3)** | `proof_v2.rs:20-65`, `proof_v3.rs:105-150` | $O(K \log K + M)$ ($K$ Keys, $M$ Bytes) | Kein Lock | 0 ns |
| **`DeletionProof::verify` (v2/v3)** | `verify.rs:18-106` | $O(1)$ (relativ zu $K$ und $N$) | Kein Lock | 0 ns |
| **`DeletionProof::verify_external`** | `verify.rs:152-171` | $O(1)$ (Ed25519 Curve Verification) | Kein Lock | 0 ns |
| **`RevocationLog::append`** | `revocation_log.rs:538-593` | **$O(N)$** ($N$ Log-Einträge) | `parking_lot::RwLock` (Write) | **$O(N)$** (gehalten über Cloning, Bincode & Disk I/O) |
| **`RevocationLog::is_revoked`** | `revocation_log.rs:596-599` | $O(1)$ expected (HashSet) | `parking_lot::RwLock` (Read) | $O(1)$ (blockiert während `append`) |
| **`IntegrityVerifier::verify_and_update`** | `wal_crypto.rs:191-285` | $O(1)$ je WAL-Eintrag | Kein Lock | 0 ns |
| **`verify_wal_chain_completeness`** | `wal_completeness.rs:31-39` | $O(1)$ (32-Byte ConstantTimeEq) | Kein Lock | 0 ns |

---

## 4. Abgleich mit Benchmark-Kennzahlen (2,28 µs / 2,49 µs)

### 4.1 Analyse der dokumentierten Messwerte
In `README.md:136` wird die Löschbeweis-Zeit wie folgt zitiert:
> `| **5. Löschbeweis-Zeit** | **2,28 µs (O(1) Verifikation) / 2,49 µs (1 Key Erzeugung)** |`

Die detaillierten Messergebnisse in `docs/BENCHMARKS_DELETION_PROOF.md` §0–1 weisen folgende Werte aus:
- `deletion_proof_verification_latency (1 key)`: p50 = 2,32 µs
- `deletion_proof_verification_latency (1,000,000 keys)`: p50 = 2,32 µs
- `deletion_proof_creation_latency (1 key)`: p50 = 2,50 µs
- `deletion_proof_creation_latency (1,000,000 keys)`: p50 = 1,06 s

### 4.2 Befund zur Abdeckung & Degradations-Risiko
1. **Messbereich des Benchmarks:** Der Benchmark `deletion_proof_latency_bench` in `docs/BENCHMARKS_DELETION_PROOF.md` misst **ausschließlich** die isolierten Funktionen `DeletionProof::create_with_wal_receipt` und `DeletionProof::verify`.
2. **Bestätigung für `DeletionProof::verify`:** Die Zahl 2,28 µs / 2,32 µs für die Verifikation ist tatsächlich **$O(1)$** unabhängig von der Anzahl der gelöschten Schlüssel $K$ (2,32 µs bei 1 Key und 2,32 µs bei 1.000.000 Keys), weil `DeletionProof` nur den 32-Byte Blake3-Hash `deleted_keys_hash` speichert.
3. **Erzeugungs-Skalierung bei großem $K$:** Die Erzeugung `DeletionProof::create` skaliert erwartungsgemäß mit $K$ und $M$: 2,50 µs bei 1 Key vs. 1,06 s bei 1.000.000 Keys.
4. **Fehlende `RevocationLog`-Einbindung in Benchmarks:** Der `RevocationLog` wird in keinem der bestehenden Benchmarks (`deletion_proof_latency_bench` oder `performance_baseline_2026-09-25.md`) gemessen. Sämtliche gemessenen Latenzen wurden bei isolierter Beweiserzeugung ohne aktive `RevocationLog::append`-Aufrufe erfasst ($N = 0$).
5. **Degradations-Prognose für `RevocationLog::append`:** Wenn Schlüssel widerrufen werden (z. B. via `KeyRegistry::revoke_group` / `revoke_record`), wird `RevocationLog::append` aufgerufen. Da `RevocationLog::append` bei jedem Aufruf die gesamte Bincode-Historie aller $N$ Einträge neu serialisiert und atomar auf die Festplatte schreibt, bleibt die Latenz **nicht** bei ~2,5 µs, sondern degradiert linear ($O(N)$). Bei großem $N$ führt dies zu spürbaren Latenz-Spikes und blockiert konkurrierende Leseanfragen (`is_revoked`) auf dem Schreib-Lock.

---

## 5. Vorschlag Tests (Textbeschreibung)

Um künftige Performance-Regressionen und Lock-Contention im Krypto-Hotpath systematisch zu überwachen, wird die Einführung folgender Tests empfohlen:

1. **Regressionstest `bench_revocation_log_append_scaling` (Criterion / Benchmark):**
   - *Beschreibung:* Systematische Messung von `RevocationLog::append` bei Log-Größen $N \in \{10, 100, 1.000, 10.000, 100.000\}$.
   - *Ziel:* Empirischer Nachweis der $O(N)$ Disk-I/O- und Serialisierungs-Degradation zur Absicherung gegen unbeabsichtigte Regressions-Verschlechterungen.

2. **Concurrency-Stress-Test `test_revocation_log_concurrent_read_write_contention`:**
   - *Beschreibung:* Ein Test-Setup, das $T_R$ Leser-Threads startet, welche kontinuierlich `is_revoked()` aufrufen, während ein Schreiber-Thread wiederholt `append()` ausführt.
   - *Ziel:* Quantifizierung der Lese-Blockade-Dauer (`entries.write()` Hold-Time) unter paralleler Last.

---

*Bericht Ende.*
