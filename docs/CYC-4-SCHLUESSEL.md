# CYC-4 — Schlüsselzyklus (Hierarchie, Provider, Epochen, Tomb-/Partitions-/Entry-Schlüssel)

Ersetzt R14 §8.4. Abhängigkeiten: SYS-1 §3 (Ports `KeyProvider`, `Entropy`, `Kdf`), SYS-2 §3 (Siegel, Schlüsselquelle), §5.1 (`KEYS`), SYS-4 §2.5 (Zahlen), SYS-5 §4/§8 (Profile, Restexposition), CYC-3 §3.5 (ER2). Gate: **G-E** (Konformanzsuite `KeyProviderConformance` ist Gate-Pflicht für jeden Provider, SYS-6 §6).

## 1. Zweck und Grenzen

Der Zyklus legt fest, welche Schlüssel es gibt, wo jeder liegt, wie er entsteht, rotiert, unbenutzbar wird und vernichtet wird, und was der Kern aus dem Zustand eines Schlüssels schließen darf. Die **Schlüsselverwaltung selbst liegt außerhalb des Kerns** (NZ-15): der Kern kennt nur den Port `KeyProvider`. Der Kern erzeugt nie Schlüsselmaterial außer durch `Entropy` im Auftrag des Providers (`create_*` erhält `&dyn Entropy`).

**Kernsätze.**
1. **Nichterreichbarkeit ist nie Vernichtung.** Ein Provider, der nicht antwortet, `Disabled` meldet oder einen Schlüssel nicht liefert, erfüllt `Cryptographic` nicht. Nur `probe == Absent` für **jede** Epoche `e ≤ key_epoch` tut es (`INV-KEY-4`).
2. **Ein fehlender Schlüssel ohne Vernichtungsabsicht ist Verlust**, nie Löschung (`KeyLost`, Quarantäne, Ticket `Incomplete`).
3. **Schlüssel stehen nie unverhüllt in Dateien** (`INV-KEY-1`). Clusterschlüssel, Tomb-, Partitions- und Entry-Schlüssel stehen **nie** in `KEYS`, Checkpoints oder Kerndateien (`INV-KEY-5`).
4. Rückrufe in den Provider laufen nie unter Kernsperren und nie unter Rang 1 (`INV-PIPE-1`, SYS-1 §5).

## 2. Hierarchie und Schlüsselarten

```text
KEK            (KeyProvider, extern; kek_id in KEYS und Catalog)
 └─ RootKey    (32 B Zufall, in KEYS KEK-umhüllt)
     ├─ Space-DEK je Space           (RootKey-umhüllt, in KEYS)
     ├─ redo_key, catalog_key, file_key, format_key, idem_key, ledger_key, LedgerScopeKey      (KDF-Ableitung, nirgends gespeichert)
Provider-only (nie in KEYS):
     Clusterschlüssel (cluster, key_epoch)     Tomb-Schlüssel (tomb_epoch)
     Partitionsschlüssel (partition_tag, generation)     Entry-Schlüssel (entry_tag)
     keyed_mac-Schlüssel (MacLabel)
```

| Schlüssel | Quelle | Ableitung / Label (`Kdf`, Domain-Separation) | Lebensdauer | Vernichtung |
|---|---|---|---|---|
| KEK | Provider | — | bis Operator ihn zurückzieht (§9.2) | Provider |
| RootKey | `create` (Entropy) | — | Instanzleben | nie einzeln (Instanzende) |
| Space-DEK | `Spaces::create` (Entropy), RootKey-umhüllt in `KEYS` | `auth_key = HKDF(DEK, "ctxs space auth v1")` | Spaceleben | `space_drop` löscht die `KEYS`-Zeile (kein Nachweis, `Plain`/Systemdaten) |
| `redo_key`, `catalog_key`, `file_key`, `format_key`, `idem_key`, `ledger_key` | `KDF(RootKey, Label)` | Labels exakt wie SYS-2 §3.3 und CYC-1 §6 (`"ctxs redo-auth v1"`, `"ctxs catalog-auth v1"`, `"ctxs file-auth v1"`, `"ctxs format-auth v1"`, `"ctxs idem-mac v1"`, `"ctxs ledger key v1" ‖ instance_id`) | Instanzleben | — |
| `LedgerScopeKey` | `KDF(RootKey, "ctxs ledger scope v1" ‖ instance_id)` (FIX-09) | | Instanzleben | — |
| Clusterschlüssel | Provider (`create_cluster_keys`) | je `(cluster, key_epoch)` ein 32-B-Schlüssel | Clusterleben | `destroy_many` (alle Epochen) |
| Tomb-Schlüssel | Provider (`create_tomb_epoch`) | leitet nur `tomb_id` ab (SYS-2 §5.7) | bis Retire (CYC-6 §7) | `retire_tomb_epoch` |
| Partitionsschlüssel | Provider (`create_partition_key`) | je `(partition_tag, generation)` | Generationsleben (CYC-7 §5) | `destroy_partition_key` |
| Entry-Schlüssel | Provider (`create_entry_key`) | je `entry_tag` (zufällig, CYC-7 §9) | Personenleben im Verzeichnis | `destroy_entry_key` |
| `keyed_mac`-Schlüssel | Provider-intern je `MacLabel` | verlässt den Provider nie | je Label-Epoche | Rotation (§8.3) |

Der Kern leitet RootKey-Ableitungen nur über den Port `Kdf` ab. Der RootKey liegt im Prozessspeicher als `Zeroizing`; es gibt keinen zweiten Speicherort. Ein Restore benötigt KEK + `KEYS` aus der Kopie (CYC-6 §6); der Ledger liegt nie im Checkpoint.

## 3. Schnittstelle `KeyProvider` (vollständig, Port `ports::kernel`)

R14 nannte im Text `create_entry_key`, `destroy_entry_key` und `keyed_mac`, führte sie aber nicht im Trait (GAP-43). Dies ist die vollständige Fassung. Alle Methoden sind synchron (`INV-PORT-1`). `[key_batch_max]` begrenzt jeden Batch.

```rust
pub trait KeyProvider: Send + Sync {
    fn info(&self) -> KeyProviderInfo;
    // KEK
    fn kek_current(&self) -> Result<KekId, KeyError>;                                   // KEK, mit dem neue KEYS umhüllt werden
    fn kek(&self, id: KekId) -> Result<Key32, KeyError>;                                // Unavailable ⇒ LockedKeys beim Open
    // Clusterschlüssel
    fn create_cluster_keys(&self, request: BatchId, cs: &[ClusterId], e: &dyn Entropy) -> Result<Vec<KeyCreateResult>, KeyError>;  // ≤ [key_batch_max], idempotent je BatchId, durabel vor Rückgabe, Epoche 0
    fn rotate_cluster_key(&self, request: BatchId, c: ClusterId, to_epoch: u32, e: &dyn Entropy) -> Result<(), KeyError>;          // idempotent, durabel; to_epoch = bisher höchste + 1
    fn cluster_keys(&self, req: &[(ClusterId, u32)]) -> Result<Vec<KeyLookup>, KeyError>;           // ≤ [key_batch_max]
    fn destroy_many(&self, cs: &[ClusterId]) -> Result<DestroyReceipt, KeyError>;                   // ≤ [key_batch_max], ALLE Epochen, idempotent
    fn probe(&self, c: ClusterId, epoch: u32) -> Result<KeyProbe, KeyError>;                        // Present | Disabled | Absent
    fn list(&self) -> Result<Vec<KeyListEntry>, KeyError>;                                          // { cluster, created_utc, max_epoch }
    // Tomb
    fn create_tomb_epoch(&self, epoch: u32) -> Result<(), KeyError>;                                // idempotent, durabel
    fn tomb_key(&self, epoch: u32) -> Result<Option<Key32>, KeyError>;                              // None ⇒ retired
    fn retire_tomb_epoch(&self, epoch: u32) -> Result<KeyProbe, KeyError>;
    // Partitionen
    fn create_partition_key(&self, p: PartitionTag, generation: u32) -> Result<(), KeyError>;       // idempotent, durabel
    fn partition_key(&self, p: PartitionTag, generation: u32) -> Result<Option<Key32>, KeyError>;   // None ⇒ vernichtet
    fn destroy_partition_key(&self, p: PartitionTag, generation: u32) -> Result<KeyProbe, KeyError>;
    // Verzeichnis (CYC-7 §9)
    fn create_entry_key(&self, tag: EntryTag) -> Result<(), KeyError>;                              // idempotent, durabel
    fn entry_key(&self, tag: EntryTag) -> Result<Option<Key32>, KeyError>;                          // None ⇒ vernichtet
    fn destroy_entry_key(&self, tag: EntryTag) -> Result<KeyProbe, KeyError>;
    fn keyed_mac(&self, label: MacLabel, msg: &[u8]) -> Result<[u8; 32], KeyError>;                 // HMAC-SHA256 unter providerinternem Schlüssel
}
pub struct KeyProviderInfo { pub destroy: DestroyMode, pub keys_in_backups: KeysInBackups, pub assurance: KeyAssurance, pub max_batch: u16 }
pub enum DestroyMode { Immediate, Scheduled { max_delay: Duration } }
pub enum KeysInBackups { No, Bounded { until: Duration }, Yes }
pub enum KeyAssurance { Hardware, Kms, FilePassphrase }
pub enum KeyLookup { Present(Key32), Disabled, Missing }        // Missing ≠ Shredded
pub enum KeyProbe { Present, Disabled, Absent }
pub enum MacLabel { Directory { epoch: u32 }, Reverse { epoch: u32 } }
pub enum KeyError { Unavailable { retry_after: Option<Duration> }, Denied, Unsupported, Conflict, Invalid }
pub struct DestroyReceipt { pub attestation: Bytes, pub per_cluster: Vec<DestroyState> }           // DestroyState: Destroyed | Scheduled{eta} | NotFound
```

**Vertragsregeln (gelten für jede Implementierung, geprüft von `KeyProviderConformance`):**
1. **Idempotenz.** Wiederholte `create_*`/`destroy_*`/`retire_*` mit gleichem Argument liefern denselben Erfolg ohne neue Schlüssel; `create_cluster_keys` mit gleicher `BatchId` und gleicher Menge liefert dieselben Ergebnisse, mit gleicher `BatchId` und abweichender Menge `Conflict`.
2. **Durabilität.** Jede erfolgreiche `create_*`, `rotate_*`, `destroy_*`, `retire_*` ist beim Rückkehren durabel (auch über Providerneustart und Maschinenausfall).
3. **`destroy_many` vernichtet alle Epochen** eines Clusters atomar je Cluster; ein Teilerfolg des Batches ist zulässig, jedes `DestroyState` ist pro Cluster verbindlich.
4. **`Disabled`** heißt: unbenutzbar, aber wiederherstellbar (KMS-Karenz). Es erfüllt `Cryptographic` nicht.
5. **`probe` ist ehrlich.** `Absent` nur, wenn der Provider den Schlüssel nachweislich nicht mehr besitzt (bei `Scheduled`: erst nach Ablauf der Karenz).
6. **Kein Schlüssel verlässt den Provider** über `keyed_mac`; `Key32` ist `Zeroizing` und nicht `Clone`/`Debug`.
7. **Fehlerabbildung im Kern:** `Unavailable` ⇒ `KeyUnavailable{retry_after}` (F-6, kein Poison, Retry); `Denied` ⇒ `KeyUnavailable` mit `hint()`, beim Open `LockedKeys`; `Unsupported` ⇒ `InvalidInput{port}`; `Conflict`/`Invalid` ⇒ `Corrupt{Provider}` mit Observer-Ereignis (Logikfehler des Providers, F-8).

## 4. Schlüsselzustände und Vernichtung (ER2)

| Zustand | Provider-Beobachtung | Kernverhalten |
|---|---|---|
| nutzbar | `cluster_keys` ⇒ `Present`, `probe` ⇒ `Present` | normaler Betrieb |
| **nicht erreichbar** | `Err(Unavailable)` | `KeyUnavailable{retry_after}`; Cache gilt bis `[key_grace]` (§5.1); Schreibpfad `Rejected`, Lesen `KeyUnavailable`, Flush/Compaction pausieren mit Backoff; nie „leer“ |
| **deaktiviert** | `Disabled` (`Scheduled`-Karenz) | Ticket `KeyDestroying{eta}` und `Blocked{provider}`; Lesen weiter gesperrt (Gate ist durabel); kein Purge (`INV-ERA-3`) |
| **vernichtet** | `probe` ⇒ `Absent` für jede Epoche, Intent im Ledger | `KeysDestroyed`, Cluster `Shredded` |
| **verloren** | `Missing`/`Absent`, **kein** `KeyDestroyIntent` im Ledger | `KeyLost`: Quarantäne des Clusters (Lesen/Schreiben `KeyLost`), Ledger `KeyLost`, Ticket `Incomplete`; Löschintent bleibt aufnehmbar (Gate ohne Schlüssel möglich, Purge nach Plan, Receipt `Incomplete`) |

**ER2-Ablauf (Details zu CYC-3 §3.5):**
1. Vorbedingung: G1…G5 durabel, Anker fortgeschrieben, Ledger `KeyDestroyIntent` durabel.
2. `destroy_many(H)` in Batches ≤ `[key_batch_max]`, sortiert nach `ClusterId` (deterministisch, wiederholbar). Aufruf außerhalb aller Kernsperren auf dem Aufruferthread oder Lane `urgent`.
3. `probe(c, e)` für jedes `c ∈ H` und jede Epoche `0 ≤ e ≤ key_epoch(c)` (`key_epoch` aus `ClusterIndex`, nie aus dem Provider). Ergebnis je Cluster: alle `Absent` ⇒ vernichtet; irgendein `Disabled` ⇒ `KeyDestroying{eta}` mit `eta = max(eta_i)` aus `DestroyState::Scheduled`; irgendein `Present` ⇒ Fehler `Corrupt{Provider}` (Provider hat nicht vernichtet; Ticket `Blocked{provider}`, Alarm, Wiederholung).
4. Wiederholung bei `Disabled` und `Unavailable`: nächster Versuch nach `min(eta, [key_probe_interval])` und danach alle `[key_probe_interval]`; Backoff ist begrenzt durch die Frist (`purge_deadline`); eine echte Fristüberschreitung setzt `Overdue` und läuft weiter (CYC-3 §6 Nr. 4).
5. Erst wenn **alle** `c ∈ H` vernichtet sind: Ledger `KeysDestroyed`, Schlüsselcache leeren, `gen++`, Edit `cluster_state → Shredded`. Teilmengen werden nicht einzeln geschlossen (ein Ticket, ein `KeysDestroyed`).
6. Der Kern bewahrt `DestroyReceipt.attestation` als Digest im Ledger (`provider_attestation_digest`, SYS-2 §5.7) und im Receipt (`key_attestations`). Der Digest ist `HMAC(ledger_key, attestation)`, nie der Rohwert.

## 5. Cache, Registrierung, Epochen

### 5.1 Schlüsselcache

Je Cluster ein Eintrag `{gen, key(e)[…], loaded_mono}`, `Zeroizing`, Budgetklasse `KeyCache`, Verdrängung LRU, **nie** verdrängt für Löschpfade (SYS-4 §3).
1. Ein Treffer ist gültig, solange `entry.gen == cluster.gen` (`INV-KEY-2`: `gen` wird vor und nach dem Entsiegeln gelesen, CYC-2 §3.4). Es gibt keine Zeit-TTL im Normalbetrieb; der Cache wird ausschließlich durch Verdrängung, Gate oder ER2 geleert.
2. **Providerausfall.** Gecachte Schlüssel dienen weiter bis `[key_grace]` ab dem ersten fehlgeschlagenen Providerzugriff (monotone Uhr); danach verweigert der Kern mit `KeyUnavailable`. Neue Schlüssel (Cache-Miss) sind sofort `KeyUnavailable`.
3. Gate und ER2 invalidieren den Eintrag sofort (`gen++`) und überschreiben den Puffer; `gen`-Erhöhung gilt vor jeder weiteren Entsiegelung.
4. Der Cache wird nie auf Platte gelegt und nie in Core-Dumps freiwillig geschrieben (Adapter: `mlock`, `DONTDUMP`, SYS-5 §8 Nr. 1).

### 5.2 Registrierung (Erzeugung)

`Clusters::register` für `Sealed`: (1) `Entropy`-ClusterId; (2) `create_cluster_keys(BatchId, …)` **zuerst**, durabel; (3) `cluster_register`-Edit (Catalog, `key_epoch = 0`). Ein Absturz zwischen (2) und (3) lässt einen **verwaisten Schlüssel** zurück (kein Cluster, keine Daten). Das ist harmlos, wird aber nie stillschweigend akzeptiert:

**Orphan-Sweep (DEC-48).** Ein Wartungsjob (`idle`) beim Open und danach periodisch vergleicht `KeyProvider::list()` mit der Cluster-Registry. Jeder Providereintrag ohne Cluster und mit `created_utc` älter als `[orphan_key_grace]` wird mit `destroy_many` entfernt; kein Ledgerereignis und kein Receipt (keine Daten, nie ein Löschauftrag), Zähler `orphan_keys_destroyed_total`. Jüngere Einträge bleiben (laufende Registrierung). Der Sweep läuft nie bei `skew_state = Uncertain` (fail-closed) und nie, wenn `list()` unvollständig oder fehlerhaft antwortet. Der umgekehrte Fall (Cluster ohne Schlüssel) ist `KeyLost` (§4).

### 5.3 Epochen und Rotation (DEC-49)

`key_epoch` ist ein Feld des Clusters (`ClusterIndex`, Catalog-Op `key_epoch` 0x001C) und des `ops_head` jeder Op; neue Writes siegeln immer mit der **aktuellen** Epoche, ältere Daten bleiben unter ihrer Epoche lesbar (Entsiegeln wählt den Schlüssel aus dem im Siegelrahmen stehenden `key_epoch`, SYS-2 §3.1).

**`Clusters::rotate_key(c) -> Result<u32, Error>`** (neu in SYS-3 §3, B1):
1. Aufruferthread: Cluster `Live`, kein `BLOCK_WRITE`, `e' = key_epoch + 1` (Überlauf ⇒ `Exhausted`).
2. `rotate_cluster_key(BatchId, c, e', Entropy)` (durabel).
3. Catalog-Edit `key_epoch(c, e')`; ab Publikation siegeln neue Writes mit `e'`. Ein Frame, der noch mit `e` vorbereitet wurde und danach sequenziert wird, bleibt gültig: der `ops_head` bindet seine `key_epoch`, und alte Epochen werden nie einzeln vernichtet (unten). Der Sequencer prüft die Epoche daher nicht.
4. Absturz zwischen 2 und 3: Provider kennt `e'`, Catalog `e`; der Kern siegelt weiter mit `e` (sicher). Der nächste `rotate_key` ist idempotent (gleiche `BatchId`-Ableitung aus `(cluster, e')`).

**Auslöser.** Der Kern erzwingt keine Rotation (SYS-2 §3.1). Meldet der Verbrauchszähler `rotation_recommended` (≥ `[rotation_recommended_seals]` Siegel je `(Schlüssel, Epoche)`), ruft die Engine `rotate_key`. Der Kern ruft es nie selbst (Politik liegt oben).

**Alte Epochen werden in v1 nicht einzeln vernichtet.** `destroy_many` vernichtet **alle** Epochen; eine Einzelvernichtung würde Re-Siegeln aller Runs, Blobs, Redo-Segmente, Checkpoints und Extension-Kopien voraussetzen, und Checkpoints sind absichtlich unveränderlich. `INV-KEY-3` (alte Epoche erst vernichten, wenn nichts mehr mit ihr versiegelt ist) ist damit in v1 vakuum erfüllt und bleibt als Schutzregel für ein späteres Re-Seal-Feature stehen. Re-Seal ist nicht Teil von v1 (DEC-49). Die Rotation begrenzt den Nonce-/Verbrauchsraum, nicht den Wirkungsbereich eines Schlüsselverlusts.

## 6. Referenzprovider `FilePassphrase` und Testprovider

Der Kern liefert (offen, 1.5) einen Referenzprovider im Crate `contextra-crypto` und Testprovider in `storage/testing`.

**`FilePassphrase`** (nur `Development` und `Standard` mit `acknowledged_weak_keys`; `Strict` verbietet ihn, SYS-5 P-1):
1. Ort: `<keyring_dir>/keyring-<n>.kr`, **nie** unter `<root>` (SYS-2 §4) und nicht im Checkpoint. Eine Datei je Generation; die höchste gültige `n` gilt.
2. Format: Dateikopf (`file_kind = 11` nur für den Referenzprovider, außerhalb des eingefrorenen Kerninventars der Dateiarten; Magic `"CTXS"+"KRNG"`), `kdf_params` (Argon2id m,t,p, Salt 16 B), danach versiegelte Einträge `(kind u8, id[16 | 4 | 32], epoch u32, SealFrame{Key32})` unter `KDF(Argon2id(passphrase, salt), "ctxs keyring v1")`, MAC-gekettet über den Eintragsstrom. KEK = Argon2id-Ableitung mit anderem Label (`"ctxs kek v1"`).
3. Schreiben atomar: `.building` → `sync_all` → `rename` → `sync_dir`; Vernichtung = Neuschreiben der Generation ohne die Einträge; die Vorgängerdatei wird überschrieben (`Fs::write_at` mit Nullen, best-effort) und entlinkt.
4. `info()`: `destroy = Immediate`, `keys_in_backups` aus Betreiberkonfiguration (`No` Standard; `Yes` verbietet `Cryptographic`), `assurance = FilePassphrase`. Auf Journal-/CoW-/Flash-Medien ist Vernichtung **nicht beweisbar**: das Receipt trägt Statuscap `WeakKeyAssurance`, `Complete` ist ausgeschlossen. `probe == Absent` bezeugt nur, dass die aktuelle Generation den Schlüssel nicht mehr enthält.
5. Passphrase-Eingabe liegt in der Engine (`KeyPromptPort`); falsche Passphrase ⇒ `Denied` ⇒ beim Open `LockedKeys`.

**Testprovider** (`SimKms`, `MemKeyProvider`; nur Feature `testing`, DEC-43): konfigurierbar `Immediate`/`Scheduled{max_delay}`, Fehlerinjektion (`Unavailable`, `Disabled`-Karenz, Teilerfolg in `destroy_many`), legen ihre Schlüssel für das Löschorakel offen (SYS-6 §7).

## 7. Tomb-Epochen (Schlüsselseite; Ablauf Retire: CYC-6 §7)

1. `tomb_key(epoch)` leitet ausschließlich `tomb_id` ab (SYS-2 §5.7). Standard: Epoche `0`, erzeugt bei `Storage::create` (`create_tomb_epoch(0)`), keine Rotation (`[tomb_epoch_rotation]` aus), kein Retire.
2. **Rotation** (`Storage::rotate_tomb_epoch()`, nur wenn `[tomb_epoch_rotation]` aktiv oder auf Betreiberaufruf, neu in SYS-3 §2): `create_tomb_epoch(e+1)` (durabel) → Ledger `TombEpochOpened{e+1}` → `TombEpochClosed{e}` → Catalog `anchor_mark`. Höchstens `[max_active_tomb_epochs]` nicht retired Epochen; darüber ⇒ `Limit{TombEpochs}`.
3. Neue Intents verwenden immer die offene Epoche. `last_erase_epoch(e)` wird mit dem Schließen final (CYC-6 §7).
4. **Fehlender Tomb-Schlüssel einer nicht retired Epoche** ⇒ `KeyLost` der Tomb-Domäne (nicht als Retire deuten, `INV-KEY-5`): neue Intents sind `KeyUnavailable`, Restore bleibt quarantined, Alarm. `tomb_key == None` ist nur nach einem Ledger-`TombEpochRetired` erwartet.
5. `tomb_key` wird im Prozessspeicher gecacht (`Zeroizing`, wie §5.1) und nie in Checkpoints oder `KEYS` geschrieben.

## 8. Partitions-, Entry-Schlüssel und `keyed_mac`

### 8.1 Partitionsschlüssel (CYC-7 §5)

`create_partition_key(tag, g)` **vor** dem ersten Block der Generation `g`; `partition_key(tag, g)` für Lesen und Rebuild; `destroy_partition_key(tag, g)` erst nach Publish von `g+1` und `verify_absent` (`INV-PART-1`), durch den Index-Owner nach den Pins (CYC-7 §5). `None` bei einer nicht vernichteten Generation ⇒ `KeyLost` der Index-Domäne (Partition neu aufbauen, Ticket `Incomplete`). Partitionsschlüssel sind unabhängig von Clusterschlüsseln und werden von Cluster-`destroy_many` nicht berührt.

### 8.2 Entry-Schlüssel

`create_entry_key(entry_tag)` mit `entry_tag` zufällig 16 B (CYC-7 §9; die ID wird **nie** aus der ClusterId abgeleitet, FIX-23). `destroy_entry_key` ist Teil der Personenlöschung (CYC-7 §9). Fehlt der Schlüssel einer lebenden Verzeichniszeile ⇒ `KeyLost` der Verzeichnisdomäne (die Person ist nicht mehr auflösbar; Wurzelcluster bleibt über seine eigene Löschung erreichbar, wenn die ClusterId anderweitig bekannt ist).

### 8.3 `keyed_mac` und Label-Epochen

`MacLabel::Directory{epoch}` und `MacLabel::Reverse{epoch}` erzeugen `dir_key` und `rev_key` (SYS-2 §5.8). Die Label-Epoche steht **nicht** im Klartext der Zeile (sie würde Zeilen gruppieren); der Verzeichnis-Owner führt die aktive Epoche und probiert beim Lookup die aktive, danach die Vorgängerepoche (höchstens 2 Epochen gleichzeitig). **Rotation der Label-Epoche** ist ein oberer Workflow O(Tenant): Zeilen neu berechnen, in einen neuen Schlüsselbereich schreiben, alte Epoche beim Provider abschalten. Sie ist die einzige Möglichkeit, das `dir_key`-Backup-Restrisiko (SYS-5 §8 Nr. 11) zu beenden.

## 9. Zusicherungen, KEK, Cohorts

### 9.1 `KeysInBackups`, `KeyAssurance`, Profil

| `KeysInBackups` | Bedeutung | Wirkung |
|---|---|---|
| `No` | kein Export außerhalb des Providers | `Cryptographic` zulässig |
| `Bounded{until}` | Exporte werden spätestens `until` nach Vernichtung vernichtet | `Cryptographic` zulässig, Statuscap `BoundedBackupKeys{until}` (entfällt bei neu berechnetem Receipt nach Ablauf, solange `probe == Absent`) |
| `Yes` | Exporte unbegrenzt | `Cryptographic` ⇒ `Unsupported`; `Standard`/`Strict` verbieten für `Personal` (SYS-5 P-2) |

| `KeyAssurance` | Eigenschaften | Profil |
|---|---|---|
| `Hardware` | nicht exportierbar, Vernichtungsnachweis | `Standard`, `Strict` |
| `Kms` | Destroy-API, Karenz ⇒ `Disabled` | `Standard` |
| `FilePassphrase` | §6, Cap `WeakKeyAssurance` | nur mit `acknowledged_weak_keys`; nicht `Strict` |

Die Prüfung beim Open erzeugt `ProfileViolation{P-1|P-2}`; ein Auftrag mit unzureichender Zusicherung ohne Ack erhält `ProviderAssuranceInsufficient` (CYC-3 §3.1 Nr. 5). Die Frist-Konsistenz (`max_delay ≤ [key_destroy_fraction_max] × D`, `INV-DL-1`) ist in CYC-3 §6 und SYS-4 §6 festgelegt.

### 9.2 KEK-Rotation (DEC-54)

Rotation ersetzt nur die Hülle des RootKey, nie Daten.
1. `kek_current()` liefert `K'`; der Kern entwickelt `RootKey` im RAM, umhüllt ihn neu und schreibt `KEYS.new` (Dateikopf, `kek_id = K'`, neuer `wrapped_root`, alle Space-DEK-Zeilen unverändert, neues `keys_crc`).
2. `sync_all` auf `KEYS.new`, `rename` auf `KEYS`, `sync_dir`.
3. Catalog-Op `kek_id(K')`.
4. **Open-Regel:** `KEYS.kek_id` ist maßgeblich. Weicht der Catalog-Wert ab (Absturz nach 2, vor 3), repariert der Kern per Edit; ist der Catalog-Wert **neuer** als `KEYS` (unmöglich ohne Manipulation), ist das `Corrupt{Keys}` und das Open scheitert.
5. **Alte KEK nicht sofort zurückziehen.** Bereits erzeugte Checkpoints enthalten `KEYS` mit der alten Hülle; ein Restore braucht die alte KEK. Der Kern führt die Liste der KEK-IDs in vorhandenen Checkpoints (`checkpoints().list()`) und gibt sie über `Inspector` aus; der Operator darf eine KEK erst zurückziehen, wenn alle Checkpoints, die sie benötigen, nach `backup_horizon` verfallen sind. Das ist eine Betriebsregel, kein Kernzwang.

### 9.3 Cohort-Strategie (Provider-intern, informativ)

Clusterschlüssel in Cohorts ≤ 1024 unter einer Cohort-KEK; `destroy_many` = Rewrap ohne die Gelöschten, dann KMS-Destroy der alten Generation. Voraussetzung: `keys_in_backups = No`, Replikation in mindestens zwei Failure Domains. Das ist Providerverantwortung; der Kern sieht nur den Port.

## 10. Fehler- und Absturzverhalten je Schritt

| Schritt | Unterbrechung | Soll |
|---|---|---|
| Registrierung (§5.2) | nach `create_cluster_keys`, vor `cluster_register` | Waisenschlüssel; Sweep nach `[orphan_key_grace]` |
| `rotate_key` (§5.3) | nach Provider, vor Edit | weiter mit alter Epoche; Wiederholung idempotent |
| ER2 (§4) | nach `destroy_many`, vor `KeysDestroyed` | `probe`; Intent vorhanden ⇒ nachholen; kein Intent ⇒ `KeyLost` |
| ER2 | Provider Teilerfolg | Ticket `KeyDestroying`/`Blocked{provider}`, Wiederholung nur der Fehlenden |
| KEK-Rotation (§9.2) | nach `KEYS`-Rename, vor Catalog | Open repariert Catalog |
| `create_tomb_epoch` | nach Provider, vor Ledger | Wiederholung idempotent; Epoche ohne Ledger-Ereignis wird beim Open nachgetragen |
| Provider beim Open nicht erreichbar | KEK `Unavailable`/`Denied` | `LockedKeys`; Löschintents aufnehmbar (CYC-3 §9.4) |
| Cache beim Providerausfall | `[key_grace]` überschritten | `KeyUnavailable` auch für gecachte Cluster |

## 11. Parameter

SYS-4 §2.5 und §2.7: `key_batch_max`, `key_probe_interval`, `key_grace`, `orphan_key_grace`, `rotation_recommended_seals`, `tomb_epoch_rotation`, `max_active_tomb_epochs`, `key_destroy_fraction_max`, `purge_deadline`.

## 12. Entscheidungen und geschlossene Lücken

DEC-05, DEC-27 (Entry-Schlüssel), DEC-48 (Orphan-Sweep), DEC-49 (Rotation = Schreibepoche, keine Einzelvernichtung), DEC-54 (KEK-Rotation: `KEYS` vor Catalog, alte KEK bis Horizont); GAP-43 (Trait unvollständig, Batch-Konstante fehlte im Register), GAP-44 (Waisenschlüssel und Schlüssel-ohne-Cluster nicht behandelt), GAP-45 (Epochen- und KEK-Rotation ohne Protokoll) (SYS-7 §3).

## 13. Abnahme

`KeyProviderConformance` (Idempotenz, alle Epochen, `probe` je Epoche, Crash, Tomb-Epochen, Neustart ohne Epochenverlust, Entry-Schlüssel, `keyed_mac`, Teilerfolg `destroy_many`), `er2_requires_absent_all_epochs`, `disabled_blocks_purge`, `key_lost_quarantine`, `unavailable_is_not_destroyed`, `orphan_key_sweep`, `rotate_key_crash`, `rotate_key_seals_new_epoch_only`, `kek_rotation_crash`, `key_cache_gen_race` (`loom`, RACE-4), `key_grace_boundary`, `tomb_key_missing_is_keylost`, `partition_key_lifecycle`, `entry_key_destroy`, `file_passphrase_weak_cap`. Modelle: M2 (ER2-Teil). Simulation: RACE-4, RACE-25. Messung: H-6, H-17. Gate G-E.

**Offene Punkte (extern):** HSM/KMS-Provider und deren SLA für `Scheduled`-Karenz; Betrieb der Replikation in zwei Failure Domains (Cohort-Strategie).
