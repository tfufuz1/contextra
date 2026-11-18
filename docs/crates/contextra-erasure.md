# contextra-erasure — Crate-Spezifikation

**Ebene:** E0 (Kern-Closure leer)  
**Bindung:** `docs/SPEC.md` §4, §5, §6, §7, §10, §11, §12

---

## 1 Zweck

Löschkern des Systems. Orchestriert die 6-Schritte-Lösch-Saga, verwaltet das Journal, den Vault (Schlüsselhierarchie), den ReadGate, die Messung und den Bericht. Dieses Crate enthält KEINE konkreten Speicher-Implementierungen — es arbeitet über Traits (Seams), die von den Layer-Crates implementiert werden.

## 2 Abhängigkeiten

**Kern:** Keine internen Abhängigkeiten (E0).  
**Features:**
- `signing`: Ed25519-Signaturen für Berichte (`ed25519-dalek`)
- `async`: Tokio-basierte Saga-Orchestrierung  
- `vault`: Persistenter Schlüsseltresor

## 3 Verantwortlichkeit

### 3.1 Schlüsselhierarchie (SPEC §4)

Besitzer der Schlüsselverwaltung:

- `MasterKey` → `TenantKEK` → `CollectionKEK` → `DocumentDEK` / `SubjectKey` → `SubjectDocDEK`
- MasterKey-Ableitung aus Passphrase + Salt via Argon2id
- Salt-Persistenz in `<data_dir>/master.salt`
- Alle Schlüssel: `ZeroizeOnDrop`, nicht `Clone`, nicht `Debug`
- **KEIN Fallback-Schlüssel.** Fehlt ein Slot → `Err(ErasureError::SlotMissing)`, nie blake3-Ableitung.
- **KEIN Null-Schlüssel als Default.** `FileVault::new_in_memory` nur für Tests, MUSS zufälligen Schlüssel verwenden.
- **AEAD korrekt:** AES-256-GCM-SIV mit zufälliger 96-Bit-Nonce, nicht XOR-Maske.

### 3.2 ErasureCore (SPEC §5.3)

Orchestriert die 6-Schritte-Saga über Seams:

```rust
pub struct ErasureCore {
    journal: Arc<dyn SagaJournal>,
    gate: Arc<dyn SagaGate>,
    vault: Arc<dyn SagaVault>,
    measure: Arc<dyn SagaMeasure>,
    report: Arc<dyn SagaReport>,
    ledger: Arc<dyn SagaLedger>,
    layers: Vec<Arc<dyn ErasureLayer>>,
    commit_barrier: Arc<dyn CommitBarrier>,
    purge_barrier: Arc<dyn PurgeBarrier>,
    snapshot_revoker: Arc<dyn SnapshotRevoker>,
}
```

### 3.3 Seams (Nähte)

| Seam | Verantwortung |
|------|---------------|
| `SagaJournal` | Append-only Log, fsync bei jedem Eintrag, Recovery offener Sagas |
| `SagaGate` | Lesesperren setzen/entfernen, `gated_seq` liefern |
| `SagaVault` | Schlüssel vernichten und Abwesenheit verifizieren |
| `SagaMeasure` | Positivkontrolle + Residuen-Scan je Schicht |
| `SagaReport` | Ed25519-signierter JSON-Bericht |
| `SagaLedger` | Hash-verketteter Abschlusseintrag außerhalb des Datenverzeichnisses |
| `SnapshotRevoker` | Snapshots bis `gated_seq` invalidieren |

### 3.4 ReadGate

Besitzer des `ReadGate`-Traits und der `GateDecision`-Enum. Die Gate-Implementierung MUSS O(1)-Lookups liefern — kein lineares Scannen über sortierte Vektoren.

**Festlegung:** Implementierung über `HashMap<KeyPrefix, GateDecision>` oder Bloom-Filter mit Fallback auf exakte Prüfung. Der aktuelle `ErasedSet` (sortierter Vec mit O(log n)) ist NICHT konform und MUSS ersetzt werden.

### 3.5 Hintergrundläufer

```rust
impl ErasureCore {
    pub async fn run_purge_window(&self) -> Result<WindowOutcome, ErasureError>;
}
```

Verarbeitet alle Sagas im Zustand `KeyRevoked` oder `Purging`. Wird extern periodisch aufgerufen (nicht interner Loop).

## 4 Zu behebende Befunde

| ID | Befund | Maßnahme |
|----|--------|----------|
| E-F01 | Fallback-Schlüssel nach "erased" | Entfernen. `Err(SlotMissing)` statt blake3-Ableitung. |
| E-F02 | `seal` ist keine AEAD (XOR-Maske) | Ersetzen durch AES-256-GCM-SIV aus `contextra-crypto`. |
| E-F03 | Deterministische Nonce | Zufällige 96-Bit-Nonce aus `OsRng::try_fill_bytes`. |
| E-F04 | Stille Fallbacks in Ableitungen | Entfernen. Jeder Fehler → `Err`. |
| E-F05 | `OsRng.fill_bytes` (panikt) | `try_fill_bytes` → `Result`. |
| E-F06 | Null-Schlüssel als Default | Zufällige Test-Schlüssel. |
| E-F07 | Schlüsseltypen sind `Clone` | `Clone` entfernen, `ZeroizeOnDrop` erzwingen. |
| E-F08 | `KeyRegistry` nicht persistent | Persistenz über Salt + Passphrase + Argon2id. |
| E-F11 | Typ-Dubletten | Eine kanonische Definition pro Typ. |
| E-F15 | Gate O(log n) statt O(1) | HashMap oder Bloom-Filter. |
| E-F16 | Positivkontrolle fälschbar | `Scanner<Armed>` privater Konstruktor. |

## 5 Test-Pflichten

Jede Seam-Implementierung MUSS liefern:

| Test-ID | Beschreibung |
|---------|-------------|
| T-J1 | Journal-Append + Recovery liefert korrekte offene Sagas |
| T-J2 | Nach Completed: `recover_open_sagas` enthält Saga nicht mehr |
| T-G1 | Gate unter gleichzeitigen Commits: `gated_seq` monoton |
| T-G2 | `decision_for_target` liefert `Erased` nach `add_gate` |
| T-V1 | `destroy_highest_key` → `verify_key_absent` = true |
| T-V2 | Zeroize: Schlüssel-Drop löscht Bytes |
| T-P1 | Purge idempotent: zweiter Aufruf = no-op |
| T-W1 | Witness nach Purge: `residue_matches == 0` |
| T-W2 | Witness ohne Purge: `residue_matches > 0` |
| T-R1 | Vollständige Saga Requested→Completed |
| T-R2 | Crash in Purging → Recovery → Completed |
