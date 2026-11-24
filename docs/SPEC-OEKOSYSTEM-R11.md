# SPEC-ÖKOSYSTEM R11 — Speicherarchitektur, Crate-Topologie und Migration des Contextra-Ökosystems

| | |
|---|---|
| Status | **Final spezifiziert, Freigabeentwurf.** Wirksam nach Gate E-0 (Review durch zwei Reviewer, Entscheidungen O-1…O-7 getroffen). |
| Gegenstand | Speicherarchitektur des gesamten Contextra-Systems: Crate-Topologie, Schichten, Verträge, Löschkette, Krypto-Grenzen und Migration vom Altstand (`contextra-store`) auf den neuen Kern (`contextra-storage`). |
| Geltung | Ersetzt **SPEC-ECOSYSTEM-R10 §3–§6 und §9** (Errata in Anhang B). Die Kernnormen bleiben in `crates/contextra-storage/SPEC.md` (R9.1) maßgeblich. Bei Modulschnitt, Crate-Grenzen und Abhängigkeiten gilt dieses Dokument. Bei Kerninvarianten (§15 R9.1) gilt R9.1. |
| Befundbasis | Flacher Klon `tfufuz1/contextra` unter `/home/claude/tfufuz1/contextra`. Zählungen per `grep`, `find`, `wc` und Auswertung der `Cargo.toml`. **Nichts gebaut, nichts getestet.** Alle Aussagen über Build- und Testverhalten sind Hypothesen, bis M0 sie belegt. |
| Rechtlicher Vorbehalt | Technische Architektur. Keine Rechtsaussage zu Fristen, Art. 17 DSGVO oder Ausnahmen. |

---

## 0. Kurzfassung

1. **Der Speicher-Stack wird von 17 Crates auf 7 reduziert.** Davon bilden fünf die Speicherplattform: `contextra-storage`, `contextra-base`, `contextra-storage-adapters`, `contextra-storage-testkit` und `contextra-erasure`. Die übrigen zwei (`contextra-crypto`, `contextra-sys`) bleiben als allgemeine Bibliotheken bestehen. 13 Crates entfallen, 3 kommen neu hinzu (Details §4, Anhang A).
2. **Jeder Begriff hat genau eine Definition.** Vokabular (IDs, `DataClass`, Schlüsseltypen, Umgebungs-Traits) liegt in `contextra-base`. Verhalten und Kernports liegen in `contextra-storage`. Kryptografische Primitive liegen in `contextra-crypto`. Heute existieren Dubletten (§1, F-03, F-07).
3. **Es gibt genau einen Löschorchestrator**, den Kern-`Eraser` (ER0–ER7). `contextra-erasure` sammelt Owner-Attestationen, signiert und erzeugt den Löschbeweis. Den parallelen Löschstapel im Erasure-Crate (15+ Module) ersetzt dies (F-05).
4. **Es gibt genau ein AEAD für alle neuen Daten:** XChaCha20-Poly1305 (E-10). AES-GCM-SIV verschwindet aus neuen Pfaden.
5. **Abhängigkeitsrichtung ist erzwungen:** Der Kern hängt nur von `contextra-base` ab. Adapter (Fs, Krypto, Schlüssel) werden ausschließlich in der Composition Root eingebunden, also im Umbrella-Crate `contextra` und in Benchmarks (E-13).
6. **Die Konsolidierung hängt nicht vom Kernfortschritt ab.** Die Phasen M0–M4 ändern Struktur und Imports, nicht Verhalten. Der Kern hat heute keinen Verbraucher (F-09), das Risiko dieser Schritte ist daher gering. Die Datenmigration (M5) braucht dagegen die Kern-Gates G-F bis G-P.

---

## 1. Ist-Befund

Die Befunde sind durch Befehle am Klon belegt. Die Beleg-Spalte nennt das Prüfverfahren.

| ID | Befund | Beleg |
|---|---|---|
| F-01 | `contextra-storage` ist **kein Mitglied des Root-Workspaces**. Der Kern liegt verschachtelt unter `crates/contextra-storage/crates/*`, ohne Wurzel-`Cargo.toml`. Die Pakete verwenden `version.workspace = true`, das ohne Workspace-Wurzel nicht aufgelöst wird. | `members` in `Cargo.toml` (36 Einträge, kein Storage-Eintrag). `crates/contextra-storage/` enthält nur `SPEC.md`, `crates/`, `docs/`, `scripts/`. |
| F-02 | Kern und Ports pinnen `thiserror = "1"`. SPEC R9.1 §3.7.1 und SD-03 verlangen `2` (Workspace-Mindeststand). | beide `Cargo.toml` unter `crates/contextra-storage/crates/` |
| F-03 | **Doppelte Definitionen.** `pub trait Clock` steht in 11 Dateien. `contextra-env` und `contextra-storage-ports` definieren `Clock`, Rng/Entropy und MetricsSink/Metrics je separat. `Key32` steht in `keys/cache.rs` und `contextra-storage-ports/src/keys.rs`. `DataClass` steht in `contextra-storage-ports/src/ids.rs` und `contextra-crypto/src/audit_chain.rs`. `StorageEngine` steht in `contextra-ports` und in `contextra-store` (compat). | `grep -rln 'pub trait Clock'`, Vergleich der `pub`-Listen |
| F-04 | **Vier Platzhalter-Crates:** `keys-file`, `keys-cohort`, `storage-bridge` (je zwei Zeilen, Verweis auf P6) und `storage-fs` (ein Modul). | Dateiinhalte |
| F-05 | **Zweiter Löschstapel** in `contextra-erasure` (32 361 LOC, 128 Dateien) mit Modulen, die Kernrollen duplizieren: `gate`, `journal`, `ledger`, `purge`, `restore`, `saga`, `scope`, `measure`, `report`, `restricted`, `revocation_log`, `supersede`, `replace`, `crypto_local`, `kv_shredding`, `vault`. | `ls crates/contextra-erasure/src` |
| F-06 | **Zwei Vault-Implementierungen:** `contextra-vault` (816 LOC, nicht im `members`-Block, aber als Pfadabhängigkeit von `contextra-db` gebaut) und `contextra-erasure/src/vault.rs`. | `Cargo.toml`, Dateiliste |
| F-07 | **Zwei AEAD-Stacks.** `contextra-crypto-ref` nutzt `chacha20poly1305`, `hkdf`, `hmac`, `sha2`, `blake3`. `contextra-crypto` nutzt `aes-gcm-siv`, `argon2`, `hkdf`, `hmac`, `sha2`, `blake3`. `contextra-erasure` nutzt zusätzlich `aes-gcm-siv` (optional), `blake3`, `bincode`, `rand 0.8`, `parking_lot`; 29 Dateien dort verwenden `aes_gcm`, `blake3` oder `bincode`. SPEC R9.1 verbietet BLAKE3 und Serialisierungs-Crates im Kern (SD-03). | `Cargo.toml` der drei Crates, `grep` über `src` |
| F-08 | **Starke Kopplung** (Dateien außerhalb des Speicherverzeichnisses): `contextra_types` 729, `contextra_ports` 509, `contextra_store` 214, `contextra_ids` 102, `contextra_mvcc` 52, `contextra_crypto` 51, `contextra_erasure` 44, `contextra_env` 37, `contextra_checkpoint` 31, `contextra_sys` 16, `contextra_durable_fs` 15, `contextra_vault` 3, `contextra_storage` **0**. | `grep -rl 'contextra_X::'` über `crates/` und `benchmarks/` |
| F-09 | **Der Kern hat keinen Verbraucher.** `PHASE-STATUS.md` führt P0 und P1 als „in Arbeit“, P3 bis P8 teils als nicht begonnen. Die Fassade `Storage` ist unvollständig (SPEC R10 §0). | `docs/PHASE-STATUS.md` |
| F-10 | **Dokumentationssprawl.** 15 Konzept- und Migrationsdateien in zwei Ordnern, darin drei parallele Spec-Generationen (R7, R8, R9, R10) und drei Konzeptentwürfe (Lethe, Mnemos, Octa). 78 Schnittstellen-Dateien (Markdown und JSON) in `docs/interface_specs`. `CLAUDE.md` im Root ist leer (0 Byte). | `find docs -type f`, `wc -c CLAUDE.md` |
| F-11 | `deny.toml` enthält Übergangsausnahmen für das Tokio-Verbot für acht Legacy-Crates, gültig bis 2027-04-30 (`TRANSITION E29`). | `deny.toml` |
| F-12 | Die Toolchain ist mit `1.89.0` gepinnt. `PHASE-STATUS.md` meldet Prüfungen mit `1.85.1`. Diese Ergebnisse sind nicht auf den Pin übertragbar. | `rust-toolchain.toml`, `PHASE-STATUS.md` |
| F-13 | **Errata R10 §4.2:** `kv_shredding` und `deletion_proof` liegen im Erasure-Crate, nicht in `contextra-crypto`. | `find` |

**Schlussfolgerung:** Der Stack ist nicht fehlerhaft ausgerichtet, aber über Umbauwellen gewachsen. Die Kerngedanken sind vorhanden (Cluster als Löschdomäne, Gate/Schlüssel/Purge, Receipts). Die Struktur verteilt sie auf zu viele Crates und dupliziert sie. Ein Kern ohne Verbraucher lässt sich dagegen ohne Migrationsrisiko umstrukturieren.

---

## 2. Architekturprinzipien

| ID | Prinzip | Konsequenz |
|---|---|---|
| A-1 | **Ein Begriff, eine Definition.** | Dubletten (F-03) werden durch Lint verhindert (§10). |
| A-2 | **Abhängigkeiten zeigen nach unten.** | Schichten nach §3.2, Prüfung im Layering-Test und in `deny.toml`. |
| A-3 | **Der Kern kennt keine Adapter.** | Fs, Krypto und Schlüsselanbieter sind Ports. Adapter werden erst in der Composition Root gebunden (E-13). |
| A-4 | **Abhängigkeitsfußabdruck bestimmt Crate-Grenzen.** | Adapter mit schweren Abhängigkeiten (Krypto, POSIX, Windows-FFI) sind per Feature abgeschaltet. Eigene Crates nur bei Schwergewichten (KMS-SDKs, siehe O-6). |
| A-5 | **Test-Doubles verlassen die Testcrate nicht.** | `contextra-storage-testkit` ist dev-only. Produktive Crates dürfen es nicht als normale Abhängigkeit führen (§10). |
| A-6 | **Unsafe nur in Inseln.** | `contextra-sys` ist die einzige Plattform-FFI-Insel. `contextra-wire` ist die zweite, außerhalb des Speicherpfads. Adapter rufen die sicheren Wrapper aus `contextra-sys` auf und enthalten selbst kein `unsafe`. |
| A-7 | **Domäne konsumiert Verträge, nicht Dateien.** | Domain-Crates öffnen keine Dateien mit personenbezogenen Daten. Sie nutzen den Kern über die Fassade und den Owner-Vertrag (§5). |
| A-8 | **Compliance ist getrennt vom Mechanismus.** | Der Kern setzt um und liefert Belege. Signaturen, Beweisformate und Berichte liegen in `contextra-erasure`. Rechtliche Bewertung liegt außerhalb des Systems. |

---

## 3. Zielarchitektur

### 3.1 Crate-Topologie

```
Ring 5  Oberfläche        contextra (Umbrella, Composition Root) · contextra-db · contextra-mcp · contextra-py
                          contextra-router · benchmarks/contextra-bench
Ring 4  Orchestrierung    contextra-engine  (Collections, Dokumente, Identity-Owner, Session, Checkpoint-Registry,
                                              Erasure-Aufruf)  ·  contextra-cognition · contextra-agent · contextra-infer-*
Ring 3  Compliance/Owner  contextra-erasure (Attest, Signatur, DeletionProof v15, Report)
                          contextra-vector · contextra-text · contextra-graph · contextra-kvcache  (SegmentOwner)
                          contextra-rank · contextra-ingest · contextra-adapt · contextra-prompt-guard
Ring 2  Verträge/Adapter  contextra-ports (Domain-Verträge) · contextra-crypto (Primitive)
                          contextra-storage-adapters (Fs, Krypto-Binding, KeyProvider-Datei, Anchor)
                          contextra-privacy (Entscheidungen)
Ring 1  Vokabular/Kern    contextra-types (Domain-Typen) · contextra-storage (Kern, Modul `ports`)
Ring 0  Grundlagen        contextra-base (IDs, Speicher-Vokabular, Umgebungs-Traits) · contextra-sys (FFI-Insel) · contextra-simd · contextra-wire
Test    contextra-storage-testkit · contextra-testkit   (nur dev-Abhängigkeit)
```

Die Zuordnung je Crate steht in Anhang A. `contextra-storage` gehört in den Root-`members`-Block, erst dann gilt die Reihenfolge der Ringe als geprüft (M0).

### 3.2 Abhängigkeitsregeln

| Gruppe | Darf abhängen von | Darf **nicht** abhängen von |
|---|---|---|
| `contextra-base` | externe, bewilligte Crates (`zeroize`) | jedem internen Crate |
| `contextra-storage` (Kern) | `contextra-base`; externe `bytes`, `zeroize`, `thiserror 2`, `crossbeam-skiplist`, `siphasher` | `types`, `ports`, `crypto`, `erasure`, `engine`, jeder Adapter, jede Domäne, `tokio`, `serde*`, `bincode`, `rand`, `blake3` |
| `contextra-storage-adapters` | `contextra-storage`, `contextra-base`, `contextra-crypto`, `contextra-sys`; externe `rustix`, optional `argon2` | Domänen-Crates, `tokio` |
| `contextra-storage-testkit` | `contextra-storage`, `contextra-storage-adapters` (dev) | darf von keinem produktiven Crate abhängen |
| `contextra-erasure` | `contextra-storage`, `contextra-base`, `contextra-types`, `contextra-crypto`; externe `ed25519-dalek` (`verify_strict`), `sha2`, `zeroize`, `serde_json` (nur Berichte) | `bincode`, `rand`, `parking_lot`, `tokio`, `aes-gcm-siv`, Domänen-Owner |
| Domain-Owner (`vector`, `text`, `graph`, `kvcache`) | `contextra-ports`, `contextra-types`, `contextra-storage` (Kern-Traits), `contextra-base` | `contextra-storage-adapters`, `contextra-erasure`, `contextra-engine` |
| `contextra-engine` | Ring 0–3 einschließlich `contextra-storage` und `contextra-erasure` | `contextra-storage-adapters` |
| Composition Root (`contextra`, `benchmarks/*`) | alles bis Ring 4 und `contextra-storage-adapters` | — |

Durchsetzung: `tests/layering.rs` wird auf die Ringe in §3.1 umgestellt (Ring-Matrix Ring0–4 aus dem Bestand entfällt, E-12). `deny.toml` erhält Positivlisten für Kern, Adapter und Erasure. Der Adapter-Wrapper erlaubt nur die Composition Root.

---

## 4. Crate-Disposition

### 4.1 Speicherbezogene Crates (17 → 7)

| Heute (Crates) | Ziel | Aktion | Phase |
|---|---|---|---|
| `contextra-storage` (Kern, verschachtelt) | `contextra-storage` | aus Unterverzeichnis in `crates/` heben, Root-Mitglied, `thiserror 2` | M0 |
| `contextra-storage-ports` | `contextra-storage::ports` (Modul) | Traits in Kern verschieben; `Clock`, `Entropy`, `Metrics`, `Observer` nach `contextra-base`; `Checksum` entfällt (E-11) | M1, M2 |
| `contextra-env` | `contextra-base` (Modul `env`) | Traits vereinheitlichen; `Rng` wird `Entropy` | M1 |
| `contextra-ids` | `contextra-base` (Modul `ids`) | vollständig übernommen | M1 |
| — | `contextra-base` (Modul `storage`) | neues Speicher-Vokabular: `ClusterId`, `SpaceId`, `BranchId`, `Stamp`, `DataClass`, `Key32` | M1 |
| `contextra-storage-fs` | `contextra-storage-adapters` (Feature `fs-posix`) | RealFs über `rustix` | M2 |
| `contextra-durable-fs` | `contextra-storage-adapters` (Module `fs`) und `contextra-sys` (Wrapper) | Fsync-Poisoning-Tests übernehmen, Crate entfällt | M4 |
| `contextra-crypto-ref` | `contextra-crypto` (Primitive) und `contextra-storage-adapters` (Binding) | XChaCha20-Poly1305, HKDF, HMAC in `contextra-crypto`; Binding in Adapter | M2 |
| `contextra-keys-file` | `contextra-storage-adapters` (Feature `keys-file`) | Argon2id-Parameter im Provider-Header | M2 |
| `contextra-keys-cohort` | `contextra-storage-adapters` (Feature `keys-cohort`, später) | erst bei Bedarf | M6 |
| `contextra-storage-bridge` | entfällt | Async-Bridge wird nicht gebaut. Migration erfolgt über ein Export/Import-Werkzeug in der Testcrate (E-15) | M2 |
| — | `contextra-storage-testkit` | neu: MemFs, FaultFs, Sim-Scheduler, Erasure-Oracle, KeyProvider-Konformanz, Golden-Vector-Runner, Canary-Scan | M2 |
| `contextra-sys` | `contextra-sys` | bleibt einzige FFI-Insel (O-2) | — |
| `contextra-vault` | entfällt | Session-Speicher wird `Volatile`-Space im Kern plus `MemPin`-Port (`contextra-engine::session`) | M4 |
| `contextra-erasure` (heute 32 361 LOC) | `contextra-erasure` (reduziert) | Modulverteilung nach Anhang A.2 | M3 |
| `contextra-checkpoint` | Kern-Checkpoint (Manifest) und `contextra-engine::checkpoint` (Registry) | Registry-Logik in Engine | M5c |
| `contextra-mvcc` | entfällt | Sichtbarkeit und Guards im Kern. Snapshot-Isolation mit SSI ist nicht Teil des Speichers (NZ-3, O-3) | M5c |
| `contextra-store` (LSM, 59 788 LOC) | entfällt | eingefroren; Löschung bei G-CUTOVER (M6) | M6 |
| `contextra-crypto` | `contextra-crypto` | einzige Primitivbibliothek; `aes-gcm-siv` nur für Altformat-Lesen, dann entfernt | M2, M6 |

**Bilanz:** Vorher 17 speicherbezogene Crates. Nachher sieben (`storage`, `base`, `storage-adapters`, `storage-testkit`, `erasure`, `crypto`, `sys`). Entfernt werden 13 (`storage-ports`, `storage-fs`, `storage-bridge`, `crypto-ref`, `keys-file`, `keys-cohort`, `env`, `ids`, `durable-fs`, `vault`, `mvcc`, `checkpoint`, `store`). Neu kommen drei hinzu (`base`, `storage-adapters`, `storage-testkit`). Die Speicherplattform selbst besteht aus fünf Crates.

### 4.2 Übrige Workspace-Mitglieder

Die Entscheidung für jedes der 36 Mitglieder steht in Anhang A.1. Sie betrifft Domain-Crates, die nicht verschmolzen werden. Diese Crates ändern nur Imports (Storage-Verbraucher) oder Ausrichtung (Owner).

---

## 5. Öffentliche Verträge

### 5.1 Vokabular (`contextra-base`, Ring 0)

```rust
// Speicher-Vokabular: nur Typen und Konstanten, keine Logik außer Konstruktion und Validierung
pub struct ClusterId(pub [u8; 16]);     // vom Kern zufällig erzeugt (SPEC R9.1 K-02)
pub struct SpaceId(pub u64);            // 0..=255 reserviert (R9.1 §5.9)
pub struct BranchId(pub u32);           // 0 = main
pub struct Stamp(u64);                  // < 2^56, Konstruktor prüft
pub struct InstanceId(pub [u8; 16]);
pub enum DataClass { NonPersonal, Personal }   // geschlossen, kein Default an API-Grenzen (R9.1 §4.7)
pub struct Key32(zeroize::Zeroizing<[u8; 32]>);
pub struct Redacted<T>(T);              // Debug/Display ohne Inhalt

// Umgebungs-Traits: einzige Definition im Ökosystem
pub trait Clock: Send + Sync {
    fn monotonic_nanos(&self) -> u64;
    fn utc_unix_ms(&self) -> Option<i64>;                    // None = nicht verlässlich
    fn skew_status(&self) -> SkewStatus { SkewStatus::Unknown }   // additiv, Default für alte Implementierungen
}
pub trait Entropy: Send + Sync { fn fill(&self, out: &mut [u8]) -> Result<(), EntropyError>; }
pub trait Metrics: Send + Sync { fn record(&self, event: MetricEvent); }   // geschlossener Katalog (R9.1 §16)
pub trait Observer: Send + Sync { fn observe(&self, event: ObserverEvent); }
```

`contextra-base` übernimmt außerdem den vollständigen Inhalt von `contextra-ids` (`TenantId`, `DocId`, `EntityId`, `TxId`, `Seq`, `Timestamp`, `DocRef`, `CollectionId`, `StoreId`, `FileId`, `TenantScoped`). Konflikte zwischen `Timestamp`/`Seq` und dem Speicher-`Stamp` werden in M1 aufgelöst. Domain-Begriffe bleiben unverändert, Speicherbegriffe gehören nur zum Kern (O-7).

### 5.2 Kern (`contextra-storage`, Ring 1)

```rust
pub mod ports {
    pub trait Fs: Send + Sync;             // SPEC R9.1 §3.2 ohne Checksum
    pub trait Anchor: Send + Sync;
    pub trait KeyProvider: Send + Sync;    // SPEC R9.1 §9.4
    pub trait Cipher: Send + Sync;         // AEAD-Port, Implementierung in contextra-crypto
    pub trait Mac: Send + Sync;
    pub trait Kdf: Send + Sync;
    pub trait MemoryBudget: Send + Sync;
    pub trait MemPin: Send + Sync;         // mlock/DONTDUMP, best effort, Adapter über contextra-sys
    pub trait Authority: Send + Sync;      // optional, FourEyes
    pub trait HostProbe: Send + Sync;      // optional, nur Diagnose
    pub trait ColdTier: Send + Sync;       // optional
    pub trait SegmentOwner: Send + Sync;   // SPEC R9.1 §12.1
    pub struct Ports { /* Fs, Clock, Entropy, Anchor, KeyProvider, Cipher, Mac, Kdf,
                          MemoryBudget, MemPin?, Metrics, Observer, Authority?, HostProbe?, ColdTier? */ }
}

pub struct Storage { /* ... */ }

impl Storage {
    pub fn open(cfg: OpenConfig, ports: ports::Ports) -> Result<(Storage, OpenReport), StorageError>;
    pub fn create_space(&self, opts: SpaceOptions) -> Result<SpaceId, StorageError>;
    pub fn register_cluster(&self, spec: ClusterSpec) -> Result<ClusterId, StorageError>;
    pub fn apply(&self, change: Change) -> Result<Receipt, StorageError>;
    pub fn view(&self) -> View;
    pub fn view_durable(&self) -> View;
    pub fn view_at(&self, stamp: Stamp) -> Result<View, StorageError>;
    pub fn fork(&self, c: ClusterId, parent: BranchId, at: ForkPoint, o: BranchOptions)
        -> Result<BranchId, StorageError>;
    pub fn feed(&self, from: FeedPosition) -> Result<Feed, StorageError>;   // Change | Redacted | Erased | Restricted
    pub fn register_owner(&self, owner: Arc<dyn SegmentOwner>) -> Result<OwnerHandle, StorageError>;
    pub fn checkpoint(&self, opts: CheckpointOptions) -> Result<CheckpointManifest, StorageError>;
    pub fn eraser(&self) -> Eraser<'_>;
}

pub struct Eraser<'s> { /* ... */ }
impl Eraser<'_> {
    pub fn cluster(&self, scope: &[ClusterId], req: EraseRequest) -> Result<Ticket, StorageError>;
    pub fn range(&self, c: ClusterId, r: KeyRange, req: EraseRequest) -> Result<Ticket, StorageError>;
    pub fn branch(&self, c: ClusterId, b: BranchId, req: EraseRequest) -> Result<Ticket, StorageError>;
    pub fn verify(&self, ticket: &Ticket) -> Result<VerifyReport, StorageError>;   // strukturell, ohne Schlüssel
}

// Ein Fehlertyp für den gesamten Kern. Jeder Fehler hat stabilen Typ und hint() (R9.1 Z-18).
pub enum StorageError { /* R9.1 §3.5 F-1…F-9 */ }
```

**Verbindliche Regel:** Der Kern exportiert keine Typen aus `contextra-storage-adapters`, `contextra-crypto` oder `contextra-erasure`. Die Schlüsseltypen für Fs, Krypto und Anchor bleiben Trait-Objekte. Die Domain sieht nur `Storage`, `View`, `Change`, `Receipt`, `Feed`, `SegmentOwner` und `StorageError`.

### 5.3 Composition Root (`contextra-storage-adapters`, Ring 2)

```rust
// Features: fs-posix (Standard), fs-windows, crypto (Standard), keys-file (Standard), keys-cohort, anchor-file, mem-pin
pub fn open_default(dir: &Path, opts: OpenOptions) -> Result<(Storage, OpenReport), StorageError>;
pub fn open_test(dir: &Path, opts: OpenOptions) -> Result<(Storage, OpenReport), StorageError>;  // nur mit Feature `testing`
```

`open_default` setzt `OpenProfile::Production` (R9.1 §16) und verweigert schwache Konfigurationen (`FilePassphrase` mit personenbezogenen Spaces, gemeinsames Ledger-Gerät). `open_test` ist nur über das Testkit-Feature verfügbar und darf nie in Produktionsbinaries auftauchen (`deny.toml`).

### 5.4 Erasure-Dienst (`contextra-erasure`, Ring 3)

```rust
pub struct ErasureService<'s> { storage: &'s Storage, signer: Arc<dyn Signer> }

impl ErasureService<'_> {
    /// Startet einen Löschauftrag auf einer bereits aufgelösten Clustermenge.
    /// Die Auflösung Person → Cluster liegt in contextra-engine (Identity-Owner), nicht hier.
    pub fn request(&self, clusters: &[ClusterId], level: EraseLevel, req: EraseRequestId,
                   deadline: PurgeDeadline) -> Result<Ticket, ErasureError>;

    /// Liefert den Löschbeweis erst nach Status Complete oder CompleteWithResidual.
    pub fn deletion_proof(&self, ticket: &Ticket) -> Result<DeletionProofV15, ErasureError>;
}

pub trait Signer: Send + Sync {
    fn key_id(&self) -> KeyId;
    fn sign(&self, canonical: &[u8]) -> Signature;     // ed25519, verify_strict
}
```

`verify_receipt` (offline, kanonische Bytes) bleibt im Kern (SPEC R9.1 §9.7). `contextra-erasure` ruft sie nur auf und bildet daraus `DeletionProofV15`. DeletionProof v1, v2, v3 und v5 entfallen (E-16).

### 5.5 Domain-Owner-Vertrag

Jeder Index oder Persistenzbesitzer implementiert `SegmentOwner` aus dem Kern (R9.1 §12.1). Die Domain stellt keine eigene Persistenz für personenbezogene Daten bereit. Owner-Lücken verhindern systemweites `Complete` (R9.1 §12.8, Punkt 9).

---

## 6. Löschkette über die Crates

```
contextra-engine            1. Identity-Owner: Subjekt → ClusterId-Menge        (Mechanismus: Kern-Lookup, §12.7 R9.1)
   │
contextra-erasure           2. ErasureService::request(clusters, level, req, deadline)
   │                           Policy (Retention, Legal Hold) liegt oberhalb, nicht im Kern
contextra-storage (Kern)    3. Eraser::cluster → ER0…ER7 (Gate, KeyDestroy, Purge)   ◄── einziger Orchestrator
   │                           Owner-Hooks: on_gate an alle SegmentOwner, verify_absent je Owner
contextra-erasure           4. Attestsammlung, Vollständigkeitsprüfung, Statuscaps
   │                        5. DeletionProofV15 (canonical bytes, Signer)
contextra-audit (Engine)    6. Verarbeitungsverzeichnis referenziert Digest, keine Freitexte
```

Verbindliche Regeln:

- **L-1** Nur `Eraser` führt ER0–ER7 aus. Kein anderes Crate schreibt Gate-, KeyDestroy- oder Purge-Schritte. `contextra-engine` ruft nur den Dienst auf (R10 §4.2, übernommen).
- **L-2** `contextra-erasure` erzeugt Beweise nur aus Kernbelegen und Owner-Attesten. Frei eingegebene Strings sind als Beleg unzulässig (EK-6, R10 §5).
- **L-3** Die strukturelle Prüfung (`Eraser::verify`) und die Inhaltsprüfung (Erasure-Oracle) sind getrennt. Der Oracle liegt in `contextra-storage-testkit`, weil er Klartext-Canaries braucht.
- **L-4** `MeasureKind` (R9.1 §13.2.1) ersetzt die Metrik `UNMEASURED_ERASURE_LAYERS`.

---

## 7. Krypto, Schlüssel, Identität

| Thema | Festlegung |
|---|---|
| **AEAD** | XChaCha20-Poly1305 mit 24-Byte-Zufallsnonce für alle neuen Daten (R9.1 §5.2). Implementierung in `contextra-crypto`. AES-GCM-SIV bleibt nur für das Lesen des Altformats, bis M6 (E-10, O-4). |
| **HKDF, HMAC** | eine Implementierung in `contextra-crypto`. Kein eigenes Duplikat in Adapter oder Erasure. |
| **BLAKE3** | nicht im Speicherpfad (R9.1 SD-03). Im Domain-Pfad nur dort, wo `contextra-crypto` es heute benötigt; Entfernung ist kein Blocker für diese Spec. |
| **Prüfsumme** | CRC32C als deterministische Kernfunktion (`codec`), kein Port (E-11). `crc32fast` (IEEE) ist für Formate verboten. |
| **Schlüsselanbieter** | `KeyProvider` im Kern (R9.1 §9.4). Implementierungen `keys-file` (Standard) und `keys-cohort` im Adapter. Ein KMS-Adapter ist ein eigenes Crate, sobald der Anbieter feststeht (O-6). |
| **Signaturschlüssel** | ed25519 aus 32-Byte-Seed (`SigningKey::from_bytes`, kein `rand`). Verwahrung über `Signer`-Port; Produktion mit Hardware-Assurance (O-5). |
| **Personenbezug** | `DataClass` lebt in `contextra-base`. `Personal` erzwingt geclustert und versiegelt (R9.1 §4.7). Die Zuordnung Person → Cluster liegt im Identity-Owner von `contextra-engine` nach R9.1 §12.7. |
| **Schlüsselmaterial in Logs** | keines. Alle Typen, die Schlüssel tragen, sind `Redacted` oder `Key32`. |

---

## 8. Domänenanbindung

| Crate | Rolle | Speichervertrag | Änderung |
|---|---|---|---|
| `contextra-engine` | Domäne, Identity-Owner, Session, Checkpoint-Registry, Erasure-Aufrufer | Kern-Fassade, `SegmentOwner` (Identity) | übernimmt Vault- und Checkpoint-Funktion, ersetzt Löschorchestrierung (R10 §4.2) |
| `contextra-vector` | Owner | `Rebuildable` (global) oder `ClusterSections` (lokal) | eigene Dateien entfallen, Watermark über `(instance_id, applied_stamp)` |
| `contextra-text` | Owner | `ClusterSections` (Postings als Kern-Records) | Löschpfad über Portschlüssel bleibt Referenz (R10 §2.3, positiv) |
| `contextra-graph` | Owner | `Rebuildable` mit Lineage, Kanten als Kern-Records | Dev-Abhängigkeit auf Store wird entfernt (Layering-Allowlist) |
| `contextra-kvcache` | Owner (Cache) | `Rebuildable`, clusterlokal, kein Spill | `delete` im Port ergänzen (R10 §4.2) |
| `contextra-rank` | keine Persistenz | — | bleibt speicherfrei |
| `contextra-ingest` | Erzeuger von `Change`-Beschreibungen | keiner direkt | bleibt speicherfrei, Persistenz über Engine |
| `contextra-cognition`, `contextra-agent` | Verbraucher | Feed, Fassade | Imports umstellen |
| `contextra-privacy` | Entscheidung (DLP) | keiner | Verbindung zur Löschung über Engine, nicht direkt |
| `contextra-db`, `contextra-mcp`, `contextra-py`, `contextra` | Oberfläche | über Umbrella | nur `contextra` bindet Adapter (E-13) |

**Konzeptabbildung** (übernommen aus R10 §3.2 mit Präzisierung):

- **Mandant:** Instanzpartition oder `tenant_tag` in der Packgruppe. Kein Löschbereich (R10 E-01).
- **Person:** Wurzelcluster, `Sealed`, `DataClass::Personal`.
- **Fall/Akte:** Fallcluster, Verknüpfung zu Personen per `link` nur bei gewollter Kaskade (R9.1 K-25).
- **Session:** Schlüsselpräfix im Personencluster oder `Volatile`-Space. Ein Cluster je Session ist nur bei eigener Löschdomäne zulässig (R9.1 K-35).
- **Chunk-ID:** `HMAC(cluster_secret, …)`, clusterlokal (R10 E-03, R9.1 K-32).
- **Audit-Eintrag:** Record im Cluster der betroffenen Person, Hashverkettung über Kern-Stamps.

---

## 9. Migration

Die Phasen sind sequenziell. Jede Phase endet mit einem Gate. **Kein Dual-Write.** Alt-Crate-Namen dürfen während M1 bis M5 nur als Re-Export-Crates ohne Logik bestehen. Diese Ausnahme endet mit der Umstellung des letzten Verbrauchers.

| Phase | Inhalt | Gate | Abhängig von Kernfortschritt? |
|---|---|---|---|
| **M0 Workspace-Reparatur** | Kern aus `crates/contextra-storage/crates/*` nach `crates/` heben, in `members` eintragen, `thiserror` auf 2, Platzhalter bleiben bis M2 | `cargo metadata` zeigt alle Speicher-Crates als Mitglieder. Build und Tests auf Pin 1.89.0 laufen (F-12). | nein |
| **M1 Vokabular** | `contextra-base` anlegen (IDs, env, Speicher-Vokabular, Umgebungs-Traits). `Clock` erweitern (additive Methoden mit Default). 37 env- und 102 ids-Imports umstellen. Storage-Dubletten entfernen. | `duplicate_definitions` (§10) = 0 für Clock, Entropy, Metrics, DataClass, Key32, Stamp, ClusterId. `cargo tree -p contextra-base` ohne interne Abhängigkeiten. | nein |
| **M2 Kern und Adapter** | `storage-ports` ins Kernmodul `ports`. `storage-fs` nach `storage-adapters` (`rustix`). `crypto-ref` in `crypto` und Adapter. `keys-file` als Feature. `storage-bridge` entfernen. `storage-testkit` aus Testcode bauen. Platzhalter löschen. | Layering-Test mit neuen Ringen grün. `cargo tree -p contextra-storage` nur `base` und externe Crates. Keine Platzhalter mehr. | nein |
| **M3 Löschstapel** | `contextra-erasure` nach Anhang A.2 reduzieren. Kernaufrufe ersetzen Gate, Journal, Ledger, Purge, Restore, Saga. R10-Pflichttests P0-1…P0-5 als Kern- oder Erasure-Tests ausführen. | `grep -r 'aes_gcm\|bincode\|blake3' crates/contextra-erasure/src` = 0. Pflichttests grün oder mit Befund dokumentiert. | teilweise (Eraser-API muss existieren) |
| **M4 Vault und Dateiprimitive** | `contextra-vault` nach `contextra-engine::session` (Volatile-Space). `contextra-durable-fs` nach Adapter. `contextra-sys`-Wrapper für Plattformoperationen. | Verbraucher der beiden Crates (3 bzw. 15 Dateien) umgestellt. Crate-Verzeichnisse entfernt. | ja, für Volatile-Space |
| **M5 Verbraucher** | Reihenfolge: (a) `contextra-engine`-Persistenz auf Kern, (b) Owner `text`, `graph`, `kvcache`, `vector`, (c) `cognition`, `agent`, `checkpoint`, `db`, `mvcc`-Nutzer, (d) Oberfläche `contextra`, `py`, `mcp`, `router`, `bench`. | Pro Schritt: Funktionstests grün und Differenztest Altstand gegen Kern (RAG-Ergebnisse, Recall@Budget). | **ja**, G-F bis G-R |
| **M6 Abbau** | `contextra-store`, `contextra-mvcc`, `contextra-checkpoint`, Legacy-Traits in `contextra-ports`, Übergangsausnahmen `deny.toml` (F-11), Dokumentation nach `docs/archive/` (E-15). | Crate-Budget (§10) eingehalten. Kein Import entfernter Crates. | **ja**, G-CUTOVER |

**Verhältnis zu SPEC R9.1 §14:** M0–M4 sind Strukturarbeit ohne Datenwirkung und können vor G-0 beginnen, sofern G-0 die Strukturentscheidung bestätigt. M5 und M6 folgen den Kern-Gates und dem Cutover-Gate G-CUTOVER aus R9.1 §14.

**Datenmigration aus dem LSM** (Altformate, `contextra-store-storage-migration.md`) ist Teil von M5 und nicht durch Crate-Konsolidierung lösbar. Sie erfolgt über Export und Import mit Re-Verschlüsselung (O-4).

---

## 10. Architekturgates

| Gate | Prüfung | Werkzeug |
|---|---|---|
| **AG-1 Schichten** | Abhängigkeitsregeln §3.2 gelten für alle Crates (inkl. `[dev-dependencies]` für produktive Pfade) | `tests/layering.rs` mit neuen Ringen, `cargo metadata` |
| **AG-2 Dubletten** | Jede Definition aus der Liste (`Clock`, `Entropy`, `Metrics`, `Observer`, `DataClass`, `Key32`, `ClusterId`, `SpaceId`, `BranchId`, `Stamp`, `KeyProvider`, `Cipher`, `StorageEngine`) existiert genau einmal im Workspace | Skript `duplicate_definitions` (R10 EK-7) |
| **AG-3 Crate-Budget** | Speicherplattform = höchstens 5 Crates, Speicher-nahe Menge (inkl. `crypto`, `sys`) höchstens 7. Ab M6 Fehler, vorher Warnung mit Liste | Test über `cargo metadata` |
| **AG-4 Unsafe-Inseln** | `unsafe` nur in `contextra-sys` und `contextra-wire`. Adapter und Kern: `#![forbid(unsafe_code)]` | `tests/unsafe_islands.rs` (vorhanden, zu aktualisieren) |
| **AG-5 Positivlisten** | `deny.toml`: Kern, Adapter, Erasure nur mit den Abhängigkeiten aus §3.2 | `cargo deny check` |
| **AG-6 Adapter-Bindung** | `contextra-storage-adapters` ist nur in `contextra` und `benchmarks/*` als normale Abhängigkeit zulässig | `deny.toml`-Wrapper |
| **AG-7 Testkit-Bindung** | `contextra-storage-testkit` nur als `dev-dependency` | `deny.toml`-Wrapper |
| **AG-8 Dokumentation** | genau eine normative Quelle je Ebene (E-15). Verweise zwischen Dokumenten werden geprüft | Skript (Link-Check) |
| **AG-9 Kernfunktion** | alle Kern-Gates aus R9.1 §13 und §14 | unverändert |

Die Gates AG-1 bis AG-8 gelten ab M2 (AG-3 ab M6). AG-9 ist Teil der Kern-Abnahme.

---

## 11. Entscheidungsregister

| ID | Entscheidung | Alternativen (verworfen) | Begründung |
|---|---|---|---|
| E-01 | Kern ist eine Crate mit Modul `ports`. `contextra-storage-ports` entfällt. | Ports getrennt lassen (R9.1 §3.1) | Ports und Kern teilen die Formatversion (G-F). Eine eigene Crate hat nur einen Versionszähler mehr. Die offene Grenze bleibt als Modul erhalten (L20). |
| E-02 | Vokabular in `contextra-base`, Verhalten im Kern. | Vokabular im Kern | Domain-Typen (`types`) würden sonst den Kern importieren. Die Ringregel verbietet das. |
| E-03 | `contextra-base` ersetzt `contextra-ids` und `contextra-env`. | beide getrennt lassen | Beide sind Null-Abhängigkeits-Mikrokerne. Ihre Zusammenlegung verliert nichts. Die Dublette `Clock` verschwindet. |
| E-04 | Adapter als ein Crate mit Features. Eigene Crates nur bei schweren Abhängigkeiten (KMS-SDKs). | ein Crate je Adapter (heute 4 Platzhalter) | Platzhalter kosten Crates ohne Nutzen. Features bilden denselben Umfang ab. |
| E-05 | Testkit ist ein eigenes dev-only Crate. | in `contextra-testkit` (Domain) zusammenführen (O-1) | Fault-Injection und Sim dürfen nicht in Domain-Testabhängigkeiten sickern. |
| E-06 | Ein Löschorchestrator, der Kern-`Eraser`. `contextra-erasure` ist Compliance-Schicht. | Erasure als Orchestrator behalten | Zwei Orchestratoren (heute) erzeugen zwei Wahrheiten. Ein Kern mit Gate-Protokoll ist die belastbare Stelle. |
| E-07 | `contextra-vault` entfällt. Session-Speicher ist `Volatile`-Space plus `MemPin`. | Vault als eigenes Crate | Doppelt vorhanden (F-06). Der Kern kennt flüchtige Spaces bereits (R9.1 §4.5). |
| E-08 | `contextra-mvcc` entfällt. Sichtbarkeit und Guards liegen im Kern. SSI nicht im Speicher. | MVCC-Crate behalten | Zwei Versionsmodelle. SSI ist nicht Teil des Speichers (NZ-3). Wird eine Fachanforderung SSI, entsteht ein Modul in `engine` auf Kern-Guards (O-3). |
| E-09 | `contextra-checkpoint` teilt sich in Kern-Manifest und `engine::checkpoint`-Registry. | Checkpoint-Crate behalten | Der Kern muss Manifeste ohnehin erzeugen (R9.1 §10.3). Die Registry ist Domänenlogik. |
| E-10 | Ein AEAD für neue Daten: XChaCha20-Poly1305. `contextra-crypto` ist die einzige Primitivbibliothek. `crypto-ref` entfällt. | AES-GCM-SIV beibehalten; zwei Stacks | R9.1 normiert XChaCha20 (SD-14). Zwei Stacks sind eine Fehlerquelle (F-07). AES bleibt nur für Altformat-Lesen. |
| E-11 | CRC32C ist eine Kernfunktion, kein Port. `Checksum` entfällt. | Port (R9.1 §3.2) | Die Funktion ist deterministisch und braucht keine Injektion. Fehlerinjektion geschieht auf der Fs-Ebene. Abweichung von R9.1 (Errata B). |
| E-12 | Ring-Modell R0–R5 ersetzt die Ring-Matrix Ring0–4 im Layering-Test. | Bestehende Matrix beibehalten | Die Matrix kennt Speicher-Crates nicht. Das Ring-Modell macht die Schichten direkt lesbar. |
| E-13 | Nur `contextra` (Umbrella) und `benchmarks/*` binden `contextra-storage-adapters`. | Adapter in jedem Konsumenten | Dependency-Inversion. Konsumenten bekommen den Kern über Injektion. |
| E-14 | `contextra-types` und `contextra-ports` bleiben als Domain-Hubs. Speicher-Traits werden aus `contextra-ports` entfernt. | Hubs auflösen | 729 bzw. 509 Dateien hängen daran. Ihre Zerlegung wäre ein eigener Großumbau ohne Speicherbezug. |
| E-15 | Dokumentation: Kern normativ in `crates/contextra-storage/SPEC.md`, Ökosystem in `docs/architecture/` (dieses Dokument). Alle übrigen Spec-Generationen nach `docs/archive/`. | Zentrale Sammlung in `docs/` | F-10: drei parallele Spec-Generationen erzeugen Widersprüche. Eine normative Quelle je Ebene. |
| E-16 | DeletionProof nur in Version 15, basierend auf `ErasureReceipt`. | v1, v2, v3, v5 weiterführen | Mehrere Beweisversionen sind nicht prüfbar gegen einen Vertrag (R10 P1-22). |

---

## 12. Offene Entscheidungen

Die folgenden Punkte müssen vor E-0 entschieden werden. Die Empfehlungen sind begründet, die Entscheidung liegt bei den Verantwortlichen.

| ID | Frage | Empfehlung | Begründung |
|---|---|---|---|
| O-1 | `contextra-storage-testkit` und `contextra-testkit` zusammenführen? | **Nein.** | Das Speicher-Testkit enthält Fault-Injection und Sim. Diese Werkzeuge dürfen nicht über Domain-Tests in Produktionspfade sickern (A-5). |
| O-2 | `contextra-sys` als einzige FFI-Insel behalten? | **Ja.** Klärung, was `contextra-vector` davon nutzt. | Der Storage-Pfad braucht nur Wrapper. Die Verbrauchsanalyse (16 Dateien) entscheidet über Inhalte, nicht über die Existenz. |
| O-3 | Braucht eine Fachanwendung Serialisierbarkeit (SSI)? | **Nein** für v1. Bei Bedarf Modul in `engine` auf Kern-Guards. | NZ-3 des Kerns. SSI im Speicher würde Commit-Pfad und Guards verteuern. |
| O-4 | Wie werden Altdaten mit AES-GCM-SIV migriert? | **Re-Verschlüsselung beim Import** über Export und Import mit neuem Schlüssel. Kein In-Place-Umschreiben. | Die Altformate sind nicht durch den Kern prüfbar. Import mit Verifikation ist die einzige belastbare Route. |
| O-5 | Wer hält die Signaturschlüssel für DeletionProof? | **Signer-Port mit Hardware-Assurance** (HSM/TPM oder KMS) in Produktion. Datei-Seed nur für Entwicklung. | Ein Beweis, den dieselbe Instanz signiert, deren Daten gelöscht werden, ist schwach. |
| O-6 | Wann entsteht `contextra-keys-kms`? | **Erst, wenn der Anbieter feststeht.** Bis dahin kein Platzhalter. | Ohne Anbieter ist jede Schnittstelle Spekulation. Ein Platzhalter-Crate erzeugt Erwartungen. |
| O-7 | Konflikt `Timestamp`/`Seq` (Domain) und `Stamp` (Kern). | **Domain-Begriffe unverändert.** Speicherbegriff nur im Kern. Mapping in der Engine. | Vermeidet Umbenennung in 729 Dateien. |

---

## 13. Risiken und Grenzen

| ID | Risiko | Mitigation |
|---|---|---|
| R-1 | Die Kernfunktion ist unvollständig (F-09). M5 und M6 sind blockiert, bis G-F bis G-CUTOVER bestanden sind. | M0–M4 sind unabhängig. Der Zeitplan für M5 folgt dem Kernfortschritt, nicht diesem Dokument. |
| R-2 | Die Erweiterung von `Clock` bricht Implementierungen (37 env-Verbraucher, Testdoubles). | Additive Methoden mit Default. Kompatibilitätstest mit `ManualClock` und `SystemClock`. |
| R-3 | Entfernen von `contextra-mvcc` könnte Funktion verlieren, die Vector oder DB nutzen. Die Anforderungen sind nicht vollständig erfasst. | Vor M5c: Funktionsinventar der 52 Verbraucher. Fehlende Funktion als Engine-Modul (O-3). |
| R-4 | Die Konsumentenmigration berührt 214 Dateien des LSM plus die Owner. | Reihenfolge nach §9 (M5a bis M5d). Jeder Schritt mit Differenztest. |
| R-5 | AES-GCM-SIV-Altdaten sind bis M6 lesbar, aber nicht neu schreibbar. | O-4. Alt- und Neuformat sind getrennte Spaces. |
| R-6 | Diese Spec ist nicht gebaut oder getestet. Der Toolchain-Pin 1.89.0 ist unverifiziert (F-12). | M0 belegt Build und Tests auf dem Pin. Ergebnisse gehen in `PHASE-STATUS.md`. |
| R-7 | `contextra-types` (729) und `contextra-ports` (509) sind riesige Hubs. Refactoring dort wäre teuer. | Bewusst stabil gehalten (E-14). |
| R-8 | Die Zusammenlegung von `contextra-ids` und `contextra-env` kann Namenskollisionen erzeugen. | M1 prüft Kollisionen mit dem Speicher-Vokabular (O-7). |
| R-9 | Die Verteilung von `unsafe` ist durch `sys` und `wire` zweigeteilt. Ein Fehler dort wirkt auf den Speicher. | AG-4. Jede FFI-Funktion mit Sicherheitsvertrag. |
| R-10 | `CLAUDE.md` ist leer. Projektweite Vorgaben für Agenten fehlen. | Vor M0 mit Verweis auf Ringe, Gates und diese Spec füllen. |

**Grenzen dieser Analyse.** Nicht gelesen wurden Inhalte von `contextra-engine`, `contextra-db`, `contextra-simd`, `contextra-wire` und `contextra-sandbox` im Detail. Die Zuordnung dieser Crates folgt aus Abhängigkeiten, Kopplungszählung und Kopfkommentaren. Ein Modul-Audit steht aus. Die Löschungsbehauptungen in R10 §4.2 sind nicht erneut am Code geprüft, außer der Lageangabe für `kv_shredding` (F-13).

---

## 14. Nächste Schritte

### 14.1 Sofortmaßnahmen (vor M0)

1. Entscheidungen O-1 bis O-7 treffen oder bestätigen.
2. `CLAUDE.md` mit Verweis auf Ringe (§3.1), Gates (§10) und diese Spec befüllen (R-10).
3. Toolchain `1.89.0` installieren und den Kern dort bauen. Ergebnis in `PHASE-STATUS.md` festhalten (F-12).
4. `docs/archive/` anlegen und die Spec-Generationen R7, R8 und Konzeptdateien dorthin verschieben (E-15). Dies ist die einzige Dokumentbewegung vor M0.

### 14.2 M0 (konkrete Arbeitsliste)

- `crates/contextra-storage/crates/contextra-storage` nach `crates/contextra-storage` verschieben, ebenso `contextra-storage-ports`, `-fs`, `-bridge`, `crypto-ref`, `keys-file`, `keys-cohort`. Pfade in allen `Cargo.toml` anpassen.
- In `Cargo.toml` (Root) `members` ergänzen.
- `thiserror = "1"` auf `"2"` heben; Anpassungen des Fehlertyps prüfen.
- `cargo metadata --no-deps` und `cargo check --workspace` auf Pin `1.89.0`.
- Ergebnis in `PHASE-STATUS.md` dokumentieren. Befunde, die nicht reproduzierbar sind, offen lassen.

### 14.3 Definition of Done dieser Spec

Die Spec gilt als abgenommen, wenn:

- alle Entscheidungen O-1 bis O-7 vorliegen,
- AG-1 bis AG-3 als Tests im Workspace vorhanden sind (rot ist zulässig, solange die Umsetzung aussteht),
- zwei Reviewer die Anhänge A und B bestätigt haben.

---

## Anhang A — Dispositionstabellen

### A.1 Alle 36 Workspace-Mitglieder

LOC gemäß Zählung am Klon, nur `*.rs`. Ring gemäß §3.1.

| Crate | LOC | Ring | Entscheidung | Begründung |
|---|---:|---:|---|---|
| `contextra-ids` | 1 020 | 0 | in `contextra-base` aufgehen (M1) | Mikrokern, keine Abhängigkeiten |
| `contextra-env` | 1 341 | 0 | in `contextra-base` aufgehen (M1) | Dublette `Clock` (F-03) |
| `contextra-durable-fs` | 3 491 | 0 | entfällt, nach Adapter (M4) | Fsync-Logik gehört in den Adapter (F-04) |
| `contextra-wire` | 6 211 | 0 | bleibt, außerhalb Speicher | zweite Unsafe-Insel (A-6) |
| `contextra-sys` | 2 988 | 0 | bleibt, einzige FFI-Insel | O-2 |
| `contextra-simd` | 6 461 | 0 | bleibt, außerhalb Speicher | — |
| `contextra-store` | 59 788 | — | entfällt bei G-CUTOVER (M6) | Legacy-LSM, 214 Konsumenten (F-08) |
| `contextra-vector` | 34 192 | 3 | bleibt, Owner | Index-Owner (§8) |
| `contextra-db` | 35 023 | 5 | bleibt, Oberfläche | Verbraucher über Umbrella |
| `contextra-text` | 18 176 | 3 | bleibt, Owner | Löschpfad als Referenz (R10 §2.3) |
| `contextra-checkpoint` | 11 186 | — | entfällt, Teil in Engine (M5c) | E-09 |
| `contextra-crypto` | 11 462 | 2 | bleibt, einzige Primitive | E-10 |
| `contextra-privacy` | 9 334 | 2 | bleibt, Entscheidungslogik | — |
| `contextra-graph` | 34 597 | 3 | bleibt, Owner | Dev-Abhängigkeit auf Store entfällt |
| `contextra-infer-onnx` | 1 176 | 4 | bleibt | — |
| `contextra-mcp` | 8 050 | 5 | bleibt, Oberfläche | — |
| `contextra-agent` | 11 217 | 4 | bleibt | — |
| `contextra-infer-ollama` | 7 370 | 4 | bleibt | — |
| `contextra-router` | 11 342 | 5 | bleibt, Oberfläche | — |
| `benchmarks/contextra-bench` | 5 344 | 5 | bleibt, Composition Root | darf Adapter binden (E-13) |
| `contextra-infer-candle` | 8 910 | 4 | bleibt | — |
| `contextra-sandbox` | 4 457 | — | bleibt, außerhalb Speicher | Isolation für untrusted Parser |
| `contextra-py` | 3 555 | 5 | bleibt, Oberfläche | — |
| `contextra-testkit` | 7 389 | Test | bleibt, Domain-Testkit | O-1 |
| `contextra` | 3 809 | 5 | bleibt, Umbrella und Composition Root | E-13 |
| `contextra-types` | 9 574 | 1 | bleibt, Domain-Hub | E-14 |
| `contextra-ports` | 7 208 | 2 | bleibt, Domain-Hub, Speicher-Traits raus | E-14 |
| `contextra-mvcc` | 9 017 | — | entfällt (M5c) | E-08, O-3 |
| `contextra-adapt` | 8 272 | 3 | bleibt | speicherfrei |
| `contextra-kvcache` | 11 614 | 3 | bleibt, Owner | `delete` im Port |
| `contextra-engine` | 9 335 | 4 | bleibt, absorbiert Vault, Checkpoint-Registry, Identity-Owner | E-06, E-07, E-09 |
| `contextra-cognition` | 11 979 | 4 | bleibt | — |
| `contextra-rank` | 10 810 | 3 | bleibt | speicherfrei |
| `contextra-erasure` | 32 361 | 3 | bleibt, reduziert auf Compliance (A.2) | E-06 |
| `contextra-ingest` | 30 737 | 3 | bleibt | speicherfrei |
| `contextra-prompt-guard` | 1 821 | 3 | bleibt | — |

Nicht Mitglied, aber Abhängigkeit: `contextra-vault` (816 LOC) entfällt (M4). Die Speicher-Crates `contextra-storage`, `-ports`, `-fs`, `-bridge`, `crypto-ref`, `keys-file`, `keys-cohort` werden in M0 bis M2 umgestellt.

### A.2 `contextra-erasure`: Modulverteilung

| Modul | Entscheidung | Ziel |
|---|---|---|
| `deletion_proof`, `ed25519_proof` | behalten (nur v15) | DeletionProofV15, Signer |
| `measure`, `report` | behalten | `MeasureKind`, Bericht |
| `orchestrator`, `erasure_domain`, `types_local` | ersetzen | Kern-Eraser, Kerntypen |
| `saga`, `gate`, `journal`, `ledger`, `purge`, `restore`, `restricted`, `supersede`, `replace`, `revocation_log` | ersetzen | Kern-ER0–ER7, Kern-Ledger, Kern-Restore |
| `scope`, `io` | ersetzen bzw. verschieben | Selektoren über Kern; `io` in Adapter |
| `vault` | entfernen | `engine::session` (E-07) |
| `crypto_local`, `kv_shredding` | entfernen | `contextra-crypto` bzw. Kern-KeyProvider |
| `backend`, `test_mocks` | verschieben | `contextra-storage-testkit` |

---

## Anhang B — Errata

### B.1 SPEC R9.1 (Kern)

| Stelle | Änderung |
|---|---|
| §3.1 Crate-Schnitt | ersetzt durch §3 dieser Spec |
| §3.2 Ports | `Checksum` entfällt (E-11). `Clock`, `Entropy`, `Metrics`, `Observer` liegen in `contextra-base` (E-02). |
| §3.7.2 | `contextra-crypto-ref` entfällt. Primitive in `contextra-crypto`, Binding in `contextra-storage-adapters` (E-10). |
| §3.7.3 | `RealFs` liegt in `contextra-storage-adapters`, nicht in `contextra-storage-fs` (E-04). |
| §3.7.5 | `contextra-erase-verify` entfällt. Oracle in `contextra-storage-testkit`. |
| §3.7.6 | `contextra-storage-api` entfällt. Die Kernfassade ist direkt `Storage`. |
| SD-03 | `ed25519-dalek` und `rand` gelten nur in den dort genannten Crates. Die Liste der Kern-Abhängigkeiten bleibt unverändert. |
| SD-33 | Namensraum: `contextra-storage` (Modul `ports`), `contextra-base`, `contextra-storage-adapters`, `contextra-storage-testkit`. |

### B.2 SPEC-ECOSYSTEM-R10

| Stelle | Änderung |
|---|---|
| §3.1 Schichten | Ring-Modell aus §3.1 dieser Spec ersetzt die Schichtdarstellung. |
| §4.1 | `contextra-storage-bridge` entfällt. Migration über Export/Import (E-15). |
| §4.2 | `kv_shredding` liegt im Erasure-Crate, nicht in `contextra-crypto` (F-13). Der Ort der Entfernung ist `contextra-erasure` (A.2). |
| §5.2 | Fassade `contextra-storage-api` entfällt (B.1, §3.7.6). |
| §6 Migrationsplan | ersetzt durch §9 dieser Spec. |
| §9 Verifikation | Pflichttests bleiben, Zuordnung zu M3 (§9). |
