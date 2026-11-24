# SP-KEYS — `keys` + `contextra-crypto-ref` + Provider

| | |
|---|---|
| Crates | `contextra-storage/src/keys/` (KeyCache, Wrapper), `contextra-crypto-ref`, `contextra-keys-file`, `contextra-keys-cohort` |
| Besitzt | Referenzimplementierungen von `Cipher/Mac/Kdf/Checksum`; `MemKeyProvider`; `KeyProviderConformance`; Schlüsselcache mit Generationszähler; RootKey-/Space-DEK-Umhüllung; Epochenrotation |
| Konsumiert | SP-PORTS, SP-CODEC (nur Formate von `KEYS`), SP-ENV (`Entropy`, `Clock`) |
| Spec-Bezug | R9.1 §5.2, §9.4, §11.5, §13.1; INV-KEY-1…5, INV-ERA-7 (Schlüsselseite), AN-06/07 |
| Phase | P0 (Primitive, Mem-Provider, Konformanz), P1 (Cache), P6 (`keys-file`, `keys-cohort`) |

## 1 Auftrag und Abgrenzung
Stellt Schlüsselmaterial bereit und vernichtet es nachweisbar. Der Kern kennt Kryptografie nur über Ports (SD-03, NZ-2); dieser Bereich liefert die **geprüften Referenzimplementierungen** und die Konformanzsuite. **Darf nicht:** eigene Kryptoprimitive erfinden (nur Bibliotheken: `chacha20poly1305`, `hkdf`, `blake3`, `crc32c`), Clusterschlüssel in Dateien des Kerns ablegen (INV-KEY-1), einen Provider unter einer Kernsperre aufrufen (§11.2).

## 2 Schnittstelle
Port-Trait `KeyProvider` siehe SP-PORTS (inkl. E-02). Ergänzungen:
```rust
// E-08: Batch-Lesen (Recovery, Flush, Cache-Miss-Bündelung)
fn cluster_keys(&self, req: &[(ClusterId, u32)]) -> Result<Vec<KeyLookup>, KeyError>;
pub enum KeyLookup { Present(Key32), Shredded, Missing }   // Missing ≠ Shredded (F-6 KeyLost)

pub struct KeyCache { .. }
impl KeyCache {
    pub fn get(&self, c: ClusterId, epoch: u32) -> Result<Key32Handle, KeyError>; // prüft gen vor+nach Nutzung
    pub fn prefetch(&self, req: &[(ClusterId, u32)]);                              // außerhalb jeder Sperre
    pub fn invalidate(&self, c: ClusterId);                                        // Gate/ER2: gen++, Zeroizing-Drop
}
pub fn wrap_root(kek: &Key32, root: &Key32, ent: &dyn Entropy) -> Vec<u8>;         // für KEYS (Format: SP-CODEC)
```
`Key32Handle` ist nicht `Clone`, `Debug` redigiert, `Drop` überschreibt.

## 3 Regeln
1. **Hierarchie** exakt R8 §9.4: KEK (extern) → RootKey (32 B) → Space-DEK; Redo-/Catalog-/FORMAT-/Ledger-Schlüssel = `KDF(RootKey, Label)`; Clusterschlüssel ausschließlich über Provider.
2. **AEAD-Key-Nutzung (E-07):** SealFrames verwenden den ausgewählten Quellschlüssel direkt mit XChaCha20-Poly1305 und 24-Byte-Nonce; das Primitive führt HChaCha intern aus. Keine zusätzliche KDF pro Siegel und kein über mehrere Blöcke geteilter Subkey. HKDF ist auf Root-/Space-/Integritätslabels begrenzt.
3. **KeyCache:** Schlüssel liegen in `Zeroizing`; je Cluster ein Generationszähler (gemeinsam mit `cluster_table`, SP-CLUSTER); Treffer nur gültig, wenn `gen` vor **und** nach Nutzung (Entsiegeln) übereinstimmt (INV-KEY-2). TTL und `key_grace` (10 min): bei Providerausfall dürfen gecachte Schlüssel bis zum Ablauf weiter genutzt werden; Gate/ER2 invalidieren sofort. Cache ist größenbegrenzt (LRU nach Cluster); Evict überschreibt.
4. **Batch (E-08):** Cache-Misses werden gebündelt (≤ 256 je Anfrage). Recovery (R6) und Flush rufen `prefetch` für alle Cluster der zu verarbeitenden Frames/Memtable **vor** dem Entsiegeln. Ohne Batch wäre die Recovery-Zeit ∝ (#Cluster im Redo) × Providerlatenz — bei Remote-KMS unbrauchbar.
5. **Epochen:** `key_epoch` im Siegelrahmen; Rotation = neue Epoche für neue Writes (`create_cluster_key` idempotent je Epoche); alte Epoche vernichtet erst, wenn nichts mehr mit ihr versiegelt ist (Zähler aus `files_*`/Compaction, INV-KEY-3); `destroy_many` vernichtet **alle** Epochen (INV-KEY-4).
6. **`Disabled` vs `Absent`:** `probe` unterscheidet; Cache behandelt `Disabled` wie `Shredded` (nie Daten), ER2 gilt erst bei `Absent` als beendet (SP-ERASE).
7. **`MemKeyProvider`:** hält Schlüssel im Speicher, gibt sie für Orakel preis (`testing`), simuliert `Immediate`/`Scheduled{max_delay}`, Ausfälle (`retry_after`), Crash zwischen `create` und Rückgabe.
8. **`keys-file` (Einzelhost):** Schlüsseldatei je Cluster, KEK-umhüllt, Löschen = Unlink+`sync_dir` (Überschreiben ist nur best-effort und kein Crypto-Erasure-Beweis); `keys_in_backups=false` nur, wenn Verzeichnis außerhalb von Backups liegt (Betreiberzusicherung, im Receipt vermerkt). `create_cluster_keys` ist max. 256 IDs, idempotent per `BatchId`, gibt pro Cluster Status zurück; Teil-Erfolge sind erlaubt, jeder gemeldete Erfolg aber vor Rückgabe durable. Retry gibt denselben Epoch zurück.
9. **`keys-cohort`:** Cohorts ≤ 1024 unter Cohort-KEK-Generation; `destroy_many` = Rewrap ohne Gelöschte, dann KMS-Destroy der alten Generation; Keyring in ≥ 2 Failure Domains repliziert, **nie** in Backups. Atomarität des Rewrap (alte Generation erst nach durabler neuer löschen) ist Konformanzpflicht.

## 4 Fehlerverhalten
F-6: `KeyUnavailable{retry_after}` (Provider offline), `Shredded` (Absent/Disabled mit Löschnachweis), `KeyLost` (fehlt ohne Nachweis; Alarm, **keine** automatische Löschung). Fehlertexte ohne Schlüssel-/Clusterbytes.

## 5 Invarianten / Tests
INV-KEY-1 (Canary-Suche über alle Dateien/Checkpoints), INV-KEY-2 (loom: Gate zwischen Lookup und Entsiegeln), INV-KEY-3, INV-KEY-4 (Konformanz: mehrere Epochen, danach `probe == Absent` für alle). `KeyProviderConformance`: Idempotenz von `create/destroy`, Crash an jedem Punkt (`FaultFs` für `keys-file`), `probe`-Semantik, Batch ≡ Einzelaufrufe, Provider-Ausfall. Known-Answer-Tests der Primitive (RFC-Vektoren XChaCha20-Poly1305, HKDF, HMAC, CRC32C). `miri`. Die vorhandene `derive_subkey_fast`-Strecke vor XChaCha liegt außerhalb des finalen R8-Vertrags und muss vor G-F entfernt oder auf stabile Schlüsselrollen begrenzt werden; Format-/Receipt-Freeze kann nicht auf abweichendem Verhalten basieren.

## 6 Lücken / Errata
- **E-02/E-08** Batch-Schreiben/-Lesen sind beide in SP-PORTS normiert. Write: bis 256 IDs mit idempotentem `BatchId`, explizite Teilresultate, jeder Erfolg durable; Read: bis 256 `(ClusterId, epoch)`-Paare. Es wird keine fiktive atomare KMS-Batch-Transaktion verlangt.
- **E-25 (Altstand-Krypto):** `contextra-crypto` nutzt AES-GCM-SIV und `kv_shredding` (Gruppen-Subkeys, `revoke_*`). Beschluss: **keine** Übernahme in den Kern (NZ-2, andere Siegelform); `deletion_proof`/`ed25519_proof` bleiben als **Signaturschicht oben** (NZ-14) und signieren die kanonischen Receipt-Bytes (→ SP-BRIDGE WP-18.6).
- **E-26 (Nonce-Budget):** 24-Byte-Zufallsnonce je Siegel; Pflicht: Zähler der Siegel je (Quelle, Epoche) als Metrik, Rotation bei 2³² (SP-CODEC Regel 6).
- **E-11 (`IntegrityMode`)** wird in SP-RECOVERY entschieden; Provider sind davon unabhängig.

## 7 Arbeitspakete
| ID | Titel | PW | Abhängig von | Phase | Abnahme |
|---|---|---|---|---|---|
| WP-04.1 | `crypto-ref`: XChaCha20-Poly1305 direkt, HKDF nur für stabile Schlüsselrollen, keyed MAC mit Domain-Separation, CRC32C + KATs | 2.0 | WP-01.2 | P0 | KATs, miri; kein per-frame Subkey |
| WP-04.2 | `MemKeyProvider` + `KeyProviderConformance` | 2.5 | WP-04.1, WP-03.5 | P0 | Suite grün inkl. Crash-/Ausfallfälle |
| WP-04.3 | `KeyCache` (gen vor/nach, TTL, grace, Batch, Zeroizing) | 2.0 | WP-04.2 | P1 | loom INV-KEY-2 |
| WP-04.4 | `keys-file` | 2.5 | WP-04.2, WP-03.3 | P6 | Konformanz + Crash-Sweep |
| WP-04.5 | `keys-cohort` | 3.0 | WP-04.2 | P6 | Konformanz, Rewrap-Atomarität |
| WP-04.6 | KMS-/HSM-Adapterschnittstelle, SimKms, Spike eines realen KMS | 1.5 | WP-04.5 | P6 | Spike-Bericht |

## 8 Abnahme
Kein Schlüsselmaterial außerhalb `Zeroizing`/Provider (Test mit Canary-Schlüsseln, O-Files); Konformanzsuite für jeden Provider Gate-Pflicht; Batch-API vorhanden und in Recovery/Flush genutzt.

## R8.2-Nachtrag

**Spec-Bezug ergänzt:** R8.2 §9.4, §10.4; `INV-KEY-5`, `INV-LED-2` (Schlüsselseite); E-74, E-77.

1. **`KeyAssurance` und `KeysInBackups` (E-77):** Jeder Provider deklariert beides in `info()`. `keys-file` liefert `FilePassphrase`; es kann Vernichtung auf Journal-/CoW-/Flash-Dateisystemen nicht beweisen, daher setzt ERASE den Statuscap `CompleteWithResidual{WeakKeyAssurance}`, und `open` in `OpenProfile::Production` weist `Personal`+`Seal::Cluster` mit `ProviderAssuranceInsufficient` ab (Prüfung in SP-FACADE). `Bounded{until}` verlangt, dass Backup-Schlüsselexporte unter einem Epochenschlüssel liegen, der nach `until` vernichtet wird; `keys-cohort` deklariert `No`.
2. **Tomb-Epochen (K-31, E-74):** `create_tomb_epoch`, `tomb_key`, `retire_tomb_epoch` sind idempotent und durabel. Tomb-Schlüssel stehen nie in `KEYS`, Checkpoints oder Backups des Kerns (`INV-KEY-5`, Canary-Suche auf Tomb-Schlüsselmaterial erweitern). `tomb_key == None` nur nach Retire; Provider-Neustart verliert keine Epoche; `retire` liefert erst bei `Absent` Erfolg.
3. **`probe` je Epoche:** ER2 verlangt `Absent` für jede Epoche `≤ key_epoch`; Konformanz deckt Teilzustände ab (eine Epoche `Absent`, eine `Disabled`).
4. **Konformanz:** `KeyProviderConformance` um Tomb-Epochen, `KeysInBackups`/`KeyAssurance`-Deklaration und Crashverhalten beim Retire erweitert; `SimKms` kann Karenz und `Bounded`-Exporte simulieren.

| ID | Titel | PW | Phase | Abnahme |
|---|---|---|---|---|
| WP-04.7 | Provider-Angleichung R8.2: `info()` mit `KeysInBackups`/`KeyAssurance`, Tomb-Epochen in `keys-file`, `keys-cohort`, `SimKms`, Konformanz | 2.0 | WP-04.2, WP-01.5 | P6 (Tomb/Assurance-Deklaration schon P0 im `MemKeyProvider`) | Konformanzsuite grün, Canary-Test ohne Tomb-Schlüssel in Dateien, Retire-Crash-Matrix |

PW = Personenwochen, grobe Schätzung ohne Zusage (wie SPEC §14).

## R9.1-Synchronisierung

`destroy_many` wird für Gate-Hüllen in bestätigte Batches von höchstens 256 Clustern geteilt. Nur `probe == Absent` erfüllt Crypto-Erasure: `Unavailable` blockiert, `Lost` quarantänisiert bzw. ergibt `Incomplete`, `Disabled` bleibt `KeyDestroying`/`Blocked` und erlaubt ausschließlich Planung. `list` enthält `created_utc`; Tomb-Schlüssel bleiben außerhalb von KEYS und Backups.
