# Contextra — Gesamtspezifikation (normativ)

**Version:** 2.0  
**Datum:** 18. November 2026  
**Status:** FINAL — Einzige Quelle der Wahrheit  
**Geltungsbereich:** Gesamter Workspace `contextra`

---

## 0 Geltung und Rangfolge

Dieses Dokument und die zugehörigen Crate-Spezifikationen in `docs/crates/` sind die **einzige normative Referenz** für das Projekt. Alle vorherigen Spezifikationen, Coupling-Reports, Audit-Berichte, Verifikationsberichte, `reports/`-Verzeichnisse, `docs/diagnostics/`-Inhalte und `crates/*/docs/decoupling/`-Verzeichnisse sind hiermit **ersetzt und zu löschen**. Code und Spezifikation beschreiben denselben Soll-Zustand.

> **Zu löschende Altlastdokumente (Schritt 0 der Bereinigung):**  
> `reports/`, `ALTLASTEN-AUDIT.md`, `LEGACY_CLEANUP_SPEC.md`,  
> `ANALYSIS-E0-CONTRADICTIONS.md`, `docs/diagnostics/`,  
> `crates/*/docs/decoupling/`, jegliche `capabilities.toml`,  
> `tools/gate_layering.py`, `tests/layering.rs`

**Rangfolge:**
1. Diese Gesamtspezifikation (`docs/SPEC.md`) — vorliegendes Dokument
2. Crate-Spezifikationen (`docs/crates/*.md`) — normativ pro Crate
3. Crate-lokale `README.md`-Dateien — informativ

MUSS, DARF NICHT und SOLL gelten nach RFC 2119.

---

## 1 Produktdefinition

### 1.1 Was Contextra ist

Contextra ist eine **souveräne, offline betreibbare Such- und Wissens-Engine** für Unternehmen und Behörden. Sie bietet:

1. **Hybrid-Suche** (Vektor + Volltext + Graph) über lokale Datenbestände
2. **Versionierung und Zeitreise** über Daten und Regeln
3. **DSGVO-konforme Löschung** mit belegbarem Crypto-Shredding
4. **Souveränität**: keine Telemetrie, kein Fernzugriff, lokale Schlüssel

Contextra ist eine **eingebettete Rust-Bibliothek** (kein eigenständiger Server). Die primäre Schnittstelle ist die Rust-API, sekundär MCP (Model Context Protocol) über stdio.

### 1.2 Nicht-Ziele (dauerhaft)

- Keine automatisierte Entscheidung über natürliche Personen (Art. 22 DSGVO)
- Kein Training / Fine-Tuning mit personenbezogenen Daten
- Kein Dokumentenmanagementsystem
- Kein Compliance-Versprechen ohne gemessene Stufe — „DSGVO-konform" ist eine Bewertung des Verantwortlichen, keine Eigenschaft der Software (INV-7)
- Keine Skalierungsbehauptung ohne belegten, erfolgreichen Durchlauf im Bench-Harness
- Keine Beschreibung entfernter Funktionen in `README.md` oder Dokumentation

---

## 2 Architektur

### 2.1 Verbleibende Crates

Der aktive Workspace umfasst **23 Crates** in 4 Ebenen:

| Ebene | Crates |
|-------|--------|
| **E0 — Fundament & Ports** | `ids`, `env`, `sys`, `durable-fs`, `simd`, `types`, `ports`, `privacy`, `erasure` |
| **E1 — Domänenbausteine** | `store`, `vector`, `text`, `graph`, `rank`, `mvcc`, `crypto`, `checkpoint` |
| **E2 — Integration** | `db`, `engine` (deprecated Re-Export), `contextra` |
| **E3 — Endpunkte** | `mcp`, `testkit`, `bench` |

**Excluded:** `vault` (dormant, Funktionalität in `erasure` aufgegangen — darf nicht reaktiviert werden ohne G-REACTIVATE)

### 2.2 Entfernte Crates (nicht reaktivieren ohne Gate G-REACTIVATE)

```
contextra-cognition    contextra-agent        contextra-adapt
contextra-wire         contextra-py           contextra-prompt-guard
contextra-ingest       contextra-infer-onnx   contextra-infer-ollama
contextra-infer-candle contextra-sandbox      contextra-router
contextra-kv-cache
```

### 2.3 Abhängigkeitsregeln (R1–R6) — normativ und ohne Ausnahme

| Regel | Inhalt |
|-------|--------|
| **R1** | Kanten zeigen **ausschließlich abwärts** (E3→E2→E1→E0). Keine Kante von E0/E1 nach E2/E3. Keine optionalen Abhängigkeiten, die eine Aufwärtskante erzeugen — auch nicht über Feature-Unifikation. `feature_closure_allowed` existiert nicht. |
| **R2** | Dev-Abhängigkeiten auf **höhere Ebenen** sind verboten. `contextra-testkit` (E3) darf nie in `[dev-dependencies]` einer E1- oder E2-Crate erscheinen. |
| **R3** | `default`-Features MÜSSEN in allen Crates leer sein: `default = []`. |
| **R4** | Ein Konzept hat **genau einen Besitzer**. Keine Typ-Dubletten. Die kanonischen Besitzer sind in §2.5 aufgelistet. |
| **R5** | `#![forbid(unsafe_code)]` in allen Crates außer `sys` und `simd`. |
| **R6** | Kein `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!` in Nicht-Test-Code. Die Workspace-Lints (§2.6) erzwingen dies. |

### 2.4 Architekturdiagramm

```
┌──────────────────────── E3: Endpunkte ──────────────────────┐
│  mcp          testkit          bench                        │
└──────────────────────────┬──────────────────────────────────┘
                           │ darf E0, E1, E2 nutzen
┌──────────────────────── E2: Integration ────────────────────┐
│  contextra ──► db ──► engine (deprecated = pub use db::*)   │
└──────────────────────────┬──────────────────────────────────┘
                           │ darf E0, E1 nutzen
┌──────────────────────── E1: Domäne ─────────────────────────┐
│  store    vector    text    graph    rank                    │
│  mvcc     crypto    checkpoint                              │
└──────────────────────────┬──────────────────────────────────┘
                           │ darf nur E0 nutzen
┌──────────────────────── E0: Fundament & Ports ──────────────┐
│  ids   env   sys   durable-fs   simd   types   ports        │
│  privacy   erasure                                          │
└─────────────────────────────────────────────────────────────┘
```

**Klarstellung zur Schichtentrennung (Auflösung von Befund 1.1):**

- `contextra-types` und `contextra-ports` liegen normativ auf **Ebene E0 (Fundament & Ports)**.
- `types` enthält kanonische Typen, IDs und Datenstrukturen (`DocId`, `ScoredDocument`, `ContextraError`) ohne interne I/O- oder Laufzeitabhängigkeiten.
- `ports` enthält die standardisierten Trait-Verträge (`StorageEngine`, `VectorIndex`, `TextIndex`, `GraphIndex`, `CommitBarrier`, `PurgeBarrier`) und hängt ausschließlich von `env` und `types` ab (E0 → E0 ✓).
- E1-Crates (`store`, `vector`, etc.) implementieren die Traits aus `ports` und nutzen Typen aus `types`. Da `ports` und `types` auf E0 liegen, zeigt die Abhängigkeitskante **strikt abwärts** (E1 → E0 ✓, R1 erfüllt).
- Es bedarf **keiner Ausnahmeregelung** (`feature_closure_allowed`) und **keiner Versteckspiele** hinter `features = ["workspace"]`. Alle E1→E0 Kanten sind explizit, sauber und architektonisch normativ.

### 2.5 Kanonische Typbesitzer (R4)

| Typ | Kanonischer Besitzer |
|-----|---------------------|
| `DocId`, `TenantId`, `CollectionId`, `SubjectRef`, `PersonalRef` | `contextra-ids` |
| `ErasureId`, `GateId`, `OperatorRef`, `Seq`, `Timestamp` | `contextra-ids` |
| `TenantScoped<T>`, `TenantScopeViolation`, `IdError` | `contextra-ids` |
| `Clock`, `SystemClock`, `SeededRng`, `MetricsSink` | `contextra-env` |
| `DataClass`, `Validated<T>`, `EntityRecognizer` | `contextra-privacy` |
| `SagaState`, `EraseTarget`, `RetentionAttestation`, `ReadGate` | `contextra-erasure` |
| `GateDecision`, `RangeGate`, `Scanner<S>`, `LayerWitness` | `contextra-erasure` |
| `ErasureLayer`, `LayerId`, `LayerScope`, `PurgeCx`, `PurgeOutcome` | `contextra-erasure` |
| `CommitBarrier`, `PurgeBarrier` | `contextra-ports` |
| `ContextraError`, `DocumentRecord`, `SearchHit` | `contextra-types` |
| `SecretKey`, `MasterKey`, `TenantKek`, `CollectionKek` | `contextra-crypto` |
| `DocumentDek`, `SubjectKey`, `SubjectDocDek`, `ErasureKey` | `contextra-crypto` |
| `LineageRef`, `DerivedRef`, `LineageIndex` | `contextra-db` |

**Absolutregel:** Jeder Typ aus dieser Tabelle darf im gesamten Workspace nur einmal definiert sein — beim kanonischen Besitzer. Re-Exporte sind erlaubt, parallele Definitionen sind verboten.

### 2.6 Workspace-Lints (Cargo.toml root)

```toml
[workspace.lints.rust]
unsafe_code = "forbid"

[workspace.lints.clippy]
unwrap_used     = "deny"
expect_used     = "deny"
panic           = "deny"
todo            = "deny"
unimplemented   = "deny"
await_holding_lock   = "deny"
indexing_slicing     = "deny"
```

Diese Lints gelten für alle Crates. Kein Crate darf sie lokal mit `allow` überschreiben. Ausnahmen für `sys` und `simd` bei `unsafe_code` werden über `#[allow]`-Attribute auf den unsafe-Blöcken geregelt, nicht auf Crate-Ebene.

---

## 3 Datenklassifikation

### 3.1 Vier Datenklassen

Jedes Datum im System gehört zu genau einer Klasse:

| Klasse | Name | Verschlüsselung | Löschbar pro Betroffenen |
|--------|------|-----------------|-------------------------|
| **K** | Unternehmenswissen | Collection-KEK | Nein (nur Collection-Drop) |
| **P** | Personenbezogene Daten | SubjectDocDEK (Subject-Key-Ableitung) | Ja, atomar O(1) via Crypto-Shredding |
| **D** | Abgeleitete Daten | Mit obligatorischer LineageRef | Ja, kaskadiert über LineageRef |
| **E** | Löschmaterial | ErasureKey | Nach Aufbewahrungsfrist |

### 3.2 Kapselung personenbezogener Daten (P-Klasse)

**Kernentwurfsentscheidung:** Die atomare Löschbarkeit ergibt sich aus der Schlüsselhierarchie (§4), **nicht** aus physischem Löschen einzelner Bytes.

**Regeln:**

1. **P-Daten werden über `SubjectRef` adressiert.** `SubjectRef` ist eine 128-Bit-Zufalls-ID (`[u8; 16]`, `ZeroizeOnDrop`, kein `Clone`), erzeugt über `OsRng::try_fill_bytes`. Die Zuordnung `SubjectRef ↔ natürliche Person` liegt außerhalb des Systems.

2. **P-Daten bekommen einen eigenen DEK pro (Subject, Document, Version).** Vernichtung des `SubjectKey` macht alle Daten dieses Betroffenen unlesbar — in O(1), unabhängig von der Datenmenge.

3. **K-Daten DÜRFEN NICHT `SubjectRef` enthalten.** Personenbezug in K-Dokumenten wird ausschließlich über `PersonalRef`-Verweise (`[u8; 16]`, kein `Clone`) realisiert.

4. **D-Daten tragen `LineageRef`.** Jeder Schreibvorgang in D-Klasse MUSS eine `LineageRef` angeben. Ohne Lineage ist Klasse I (§5.4) nicht erreichbar.

### 3.3 K-Schreibpfad-Schutz

K-Schreibpfade akzeptieren ausschließlich `Validated<T>` aus `contextra-privacy`. Diese Typengarantie verhindert zur Compile-Zeit, dass unerkannte PII in Klasse-K-Bestände gelangt.

---

## 4 Schlüsselhierarchie

### 4.1 Hierarchie (4 Stufen)

```
MasterKey (Passphrase → Argon2id → 256 Bit)
    │
    ├── HKDF-SHA-256 ──► TenantKEK(tenant_id)
    │                        │
    │                        ├── HKDF-SHA-256 ──► CollectionKEK(collection_id, epoch)
    │                        │                        │
    │                        │                        ├── HKDF ──► DocumentDEK(doc_id, version)
    │                        │                        │                └── AEAD ──► K-Payload
    │                        │                        │
    │                        │                        └── HKDF ──► SubjectKey(subject_ref)
    │                        │                                         └── HKDF ──► SubjectDocDEK(doc_id, version)
    │                        │                                                          └── AEAD ──► P-Payload
    │                        │
    │                        └── (Epochenrotation bei Collection-Neuanlage nach Drop)
    │
    └── HKDF-SHA-256 ──► ErasureKey (Journal, Ledger, Berichte)
```

### 4.2 Algorithmen

| Funktion | Algorithmus | Crate |
|----------|------------|-------|
| AEAD | AES-256-GCM-SIV (RFC 8452) | `contextra-crypto` |
| Schlüsselableitung | HKDF-SHA-256 (RFC 5869) | `contextra-crypto` |
| Master-Key-Ableitung | Argon2id (m=256 MB, t=4, p=1) | `contextra-crypto` |
| Nonces | 96 Bit, zufällig via `OsRng::try_fill_bytes` | `contextra-crypto` |
| Signatur (Berichte) | Ed25519 | `contextra-erasure` (feature `signing`) |
| Hashing (IDs) | BLAKE3 | `contextra-ids` |

### 4.3 Bindungsregeln

1. **Eindeutigkeit:** Zwei Versionen eines Dokuments DÜRFEN NICHT denselben DEK teilen.
2. **AAD:** Jede AEAD-Operation bindet `(tenant_id, collection_id, doc_id, version)` als Associated Data. Chiffretext ist nicht in einen anderen Kontext verschiebbar.
3. **Fail-Closed:** Fehlt ein Schlüssel → Operation bricht ab. KEIN Fallback auf Klartext, KEIN Null-Schlüssel, KEIN deterministischer Notfallschlüssel.
4. **Fehlbare Zufallszahlen:** `OsRng::try_fill_bytes` — Fehler DARF NICHT zu einem Chiffretext führen.
5. **Zeroize:** Schlüsselmaterial implementiert `ZeroizeOnDrop`. Kein `Clone`, kein `Debug`, kein `Serialize` für Schlüsseltypen.

### 4.4 MasterKey-Persistenz

```
Salt: 32 Byte Zufall, gespeichert in <data_dir>/master.salt
MasterKey = Argon2id(passphrase, salt, m=256 MB, t=4, p=1)
```

**`ContextraConfig::encryption`** hat exakt zwei Zustände:
- `Required { passphrase: SecretString }` — Verschlüsselung obligatorisch
- `None` — Kein Crypto-Shredding möglich; Löschung erreicht maximal Klasse L

Es gibt keinen Modus, der Verschlüsselung vortäuscht oder implizit deaktiviert. `encryption_passphrase: Option<String>` im Code ist ein Verstoß gegen diese Spezifikation.

### 4.5 Krypto-Implementierungsregel (Befund 2.1)

Kryptographische Primitives (AES-256-GCM-SIV, HKDF, Argon2id) sind **ausschließlich** in `contextra-crypto` implementiert. Jede andere Crate, die `aes-gcm-siv`, `sha2`, `hmac` als direkte Dependencies listet, verstößt gegen R4. Die Duplikate in `contextra-erasure` sind zu entfernen.

---

## 5 Löschkette

### 5.1 Löschziel-Typen

| Ziel | Schlüssel vernichtet | Auswirkung |
|------|---------------------|------------|
| **Einzeldokument (K)** | `DocumentDEK` | Dieses Dokument + alle D-Ableitungen |
| **Betroffener (P)** | `SubjectKey` | Alle P-Dokumente dieses Betroffenen, O(1) |
| **Collection** | `CollectionKEK` | Alle Dokumente der Collection, O(1) |
| **Mandant** | `TenantKEK` | Alle Collections des Mandanten, O(1) |

### 5.2 Zustände eines Datums

```
Live ──► Gated ──► KeyRevoked ──► Purging ──► Verifying ──► Completed
  │        │           │             │             │
  │        └───────────┴─────────────┴─────────────┴──► Pending(reason)
  ├──► Restricted (Art. 18 DSGVO)
  │        ├──► Lifted  (terminal)
  │        └──► Gated   (→ Löschung)
  └──► Refused (Aufbewahrungspflicht aktiv, terminal)
```

| Zustand | Lesen | Schlüssel vorhanden | Reversibel |
|---------|-------|--------------------|-----------| 
| `Live` | Ja | Ja | — |
| `Gated` | Nein | Ja | **Nein** (INV-10) |
| `KeyRevoked` | Nein | Nein | Nein |
| `Purging` | Nein | Nein | Nein |
| `Verifying` | Nein | Nein | Nein |
| `Completed` | Nein | Nein | **Nein** (terminal, INV-9) |
| `Pending` | Nein | Abhängig von Phase | Recovery durch Hintergrundläufer |
| `Restricted` | Nur mit Audit-Eintrag | Ja | Ja (→ `Lifted` oder → `Gated`) |
| `Refused` | Ja | Ja | **Nein** (terminal, INV-9) |
| `Lifted` | Ja | Ja | **Nein** (terminal, INV-9) |

**Invariante:** Ab `Gated` ist das Datum für alle Lesepfade unsichtbar (INV-10). Ab `KeyRevoked` ist es kryptographisch unlesbar. Beides ist irreversibel.

**Zustands-Übergänge:**

| Von | Nach | Trigger |
|-----|------|---------|
| `Live` | `Gated` | Schritt 2: Gate gesetzt |
| `Live` | `Restricted` | Art. 18-Antrag |
| `Live` | `Refused` | Aufbewahrungspflicht aktiv |
| `Restricted` | `Gated` | Einschränkung → Löschung |
| `Restricted` | `Lifted` | Einschränkung aufgehoben |
| `Gated` | `KeyRevoked` | Schritt 3: Schlüsselvernichtung |
| `KeyRevoked` | `Purging` | Schritt 4: Hintergrundläufer |
| `Purging` | `Verifying` | Schritt 5: Scan startet |
| `Verifying` | `Completed` | Alle `residue_matches == 0` |
| Jeder nicht-terminale | `Pending` | Transiente Blockierung |
| `Pending` | Vorheriger Zustand | Blockierung aufgelöst |

### 5.3 Die 6-Schritte-Lösch-Saga

Jeder Schritt ist idempotent und wiederholbar. Die Reihenfolge ist normativ und darf nicht vertauscht werden.

#### Schritt 1 — Validierung und Intent-Journal (fsync #1)

1. `retention.code.blocks_erasure()` prüfen. Bei aktiver Pflicht → `Refused` (terminal, kein weiterer Schritt).
2. HMAC-basierte Duplikaterkennung über Target. Bei laufender Saga → idempotentes Receipt zurückgeben, kein neuer Lauf.
3. Für jeden registrierten `ErasureLayer` → `resolve(target)` → `LayerScope` ermitteln.
4. Journal-Eintrag `Requested` schreiben (fsync #1).

#### Schritt 2 — Gate setzen (fsync #2)

1. `CommitBarrier::exclusive()` — exklusiv gegen alle parallelen Writer.
2. `ReadGate` im SagaGate aktivieren: alle Lesepfade für das Ziel → `GateDecision::Erased`.
3. Journal `Gated` schreiben (fsync #2).
4. Barrier freigeben.

**Kritische Reihenfolge:** Gate VOR Schlüsselvernichtung. Bei Absturz zwischen Schritt 2 und 3 ist das Datum über das Gate unlesbar — der Schlüssel existiert noch, aber kein Lesepfad kommt durch.

#### Schritt 3 — Schlüsselvernichtung (fsync #3)

1. `on_gate` an alle Layer senden — RAM-Zustand vorbereiten (Surrogate markieren, ExclusionFilter befüllen).
2. Schlüssel im Vault lokalisieren (DEK / SubjectKey / CollectionKEK / TenantKEK).
3. Slot mit kryptographischen Zufallsbytes überschreiben (`OsRng::try_fill_bytes`).
4. Slot mit `0x00` nullen (`zeroize`).
5. `fsync` auf die Keyring-Datei (fsync #3).
6. `verify_key_absent`: Constant-Time-Vergleich — Slot muss vollständig `0x00` sein. Bei Fehlschlag → `ErasureError::ShredVerificationFailed`, Zustand bleibt `Gated`.
7. Journal `KeyRevoked` schreiben.

**Ab hier ist das Datum kryptographisch unlesbar. Der Aufrufer erhält das `EraseReceipt { status: KeyRevoked }`.**

#### Schritt 4 — Physische Bereinigung (Purge, asynchron)

Läuft im Hintergrundläufer (`run_purge_window`) unter exklusivem `PurgeBarrier::purge()`.

Mindestens 1 GB Plattenreserve muss vorhanden sein (`check_disk_reserve`).

**Reihenfolge der Sub-Schritte (normativ):**

| Prio | Sub-Schritt | Layer-Crate | Aktion |
|------|-------------|-------------|--------|
| a | Snapshot-Widerruf | `contextra-mvcc` | Snapshots bis `gated_seq` invalidieren |
| b | Text-Rewrite | `contextra-text` | Inverted-Index-Segmente ohne gelöschte DocIds neuschreiben |
| c | WAL-Drain | `contextra-store` | WAL flushen, aktiven MemTable immutable machen |
| d | SSTable-Rewrite | `contextra-store` | Kompaktions-Rewrite ohne gelöschte Keys; alte Dateien via `scrub_if_safe` nullen |
| e | Vektor-Reparatur | `contextra-vector` | HNSW-Knoten entfernen, Nachbarknoten neu vernetzen, Slots nullen |
| f | Graph-Bereinigung | `contextra-graph` | Entitäten und Kanten des Ziels entfernen |
| g | Slot-Bereinigung | `contextra-crypto` | Residuale Datei-Schlüssel nullen |
| h | Cache-Invalidierung | Alle Layer | In-Memory-Caches leeren |

#### Schritt 5 — Messung mit Positivkontrolle (fsync #4)

1. **Canary einfügen:** Bekanntes Byte-Muster über den regulären Schreibpfad einfügen, flushen, kompaktieren.
2. **Positivkontrolle (obligatorisch):** `Scanner<Unarmed>` sucht Canary. Wird er gefunden → `ControlEvidence::observed()` → `Scanner<Armed>`. Wird er **nicht** gefunden → Messung ungültig, `Verifying` bleibt, Fehler im Journal.
3. **Canary löschen und messen:** Canary löschen, Schritt 4 (Purge) für Canary ausführen, dann `Scanner<Armed>` starten. `residue_matches` MUSS `0` sein für jeden Layer.
4. **Lesepfad-Test:** Eine Leseabfrage auf das gelöschte Ziel MUSS `Erased` liefern.
5. **Witness-Sammlung:** Alle Layer liefern `LayerWitness`. Jedes `residue_matches == 0`.

**Canary-Scan-Einschränkung (Befund 1.3):** Ein Canary-Scan über Dateien beweist bei SSDs und Copy-on-Write-Dateisystemen keine physische Zellfreiheit. Die kryptographische Garantie liegt in der Schlüsselvernichtung (Schritt 3). Der Scan beweist, dass die Bereinigungs-Implementierung korrekt arbeitet — nicht mehr. Klasse K (§5.4) basiert auf Schritt 3, nicht auf Schritt 5.

#### Schritt 6 — Bericht und Abschluss

1. Ed25519-signierter JSON-Bericht: `erasure_id`, `target_kind`, `witnesses`, `attestation.code`, `completed_at`. **Kein PII im Bericht** (INV-12).
2. Hash-verketteten Ledger-Eintrag **außerhalb des Datenverzeichnisses** schreiben (fsync #4).
3. Journal `Completed` schreiben (terminal).
4. Gate-Eintrag kompaktieren.
5. Observer-Event `Completed` auslösen.

### 5.4 Erfüllungsklassen

| Klasse | Name | Erreicht, wenn |
|--------|------|----------------|
| **L** | Logisch entfernt | Gate gesetzt (`Gated`), Lesepfad liefert `Erased` |
| **K** | Schlüssel vernichtet | DEK/KEK/SubjectKey widerrufen, durabel, `verify_key_absent` bestanden |
| **I** | Indizes bereinigt | Alle Layer-Witnesses mit `residue_matches == 0`; Lineage-Index bereinigt |
| **M** | Vollständig gemessen | K + I + alle Schichten Scanner<Armed> mit Positivkontrolle |

**Wortregel:** „gelöscht" steht nur mit Klasse K oder höher. Bei Klasse L heißt es „logisch entfernt". Klasse K erfordert Verschlüsselung — ohne `EncryptionConfig::Required` ist Klasse K nicht erreichbar.

### 5.5 Recovery nach Crash

```
Neustart
  ├─► journal.recover_open_sagas() → [alle nicht-terminalen Sagas]
  ├─► Für jede Saga: letzten Journal-Zustand lesen
  │       Requested  → Schritt 2 starten
  │       Gated      → Schritt 3 starten (Gate ist bereits aktiv im ReadGate-Reload)
  │       KeyRevoked → Schritt 4 starten (asynchron)
  │       Purging    → Schritt 4 fortsetzen
  │       Verifying  → Schritt 5 neu starten
  │       Pending    → Beim nächsten run_purge_window aufgreifen
  └─► ReadGate aus Journal laden (alle Gated-Einträge wieder aktivieren)
```

**ReadGate-Persistenz:** Beim Start MUSS das System alle Sagas im Zustand `Gated`, `KeyRevoked`, `Purging` oder `Verifying` aus dem Journal laden und das ReadGate für ihre Ziele sofort aktivieren. Ein Neustart DARF NICHT dazu führen, dass gelöschte Daten temporär lesbar werden.

Alle Schritte sind idempotent. Der Journal-Zustand ist der Wahrheitsspeicher.

### 5.6 P-Klasse: Atomare Löschung eines Betroffenen

Wenn ein Betroffener gelöscht wird:

1. `SubjectKey(subject_ref)` wird vernichtet → O(1)
2. Alle P-Dokumente dieses Subjects in allen Collections sofort kryptographisch unlesbar
3. `LineageIndex` liefert alle D-Ableitungen (Embeddings, Postings, Graph-Knoten)
4. Purge-Phase bereinigt alle D-Daten kaskadiert
5. `PersonalRef`-Verweise in K-Dokumenten werden zu toten Links (K-Daten werden nicht umgeschrieben)

### 5.7 Restrict-Modus (Art. 18 DSGVO)

Gate wird mit `GateDecision::Restricted` statt `Erased` gesetzt. Lesezugriff nur mit Audit-Eintrag. Kein physischer Purge. Übergänge: `Restricted → Gated` (Löschung) oder `Restricted → Lifted` (Aufhebung, terminal).

---

## 6 Lesesperre (ReadGate)

### 6.1 Anforderungen

- **O(1) ohne I/O, ohne Blockierung** — synchroner In-Memory-Lookup
- **Fail-Closed** — bei unklarem Zustand: ablehnen, nie durchlassen
- **Gleiches Fehlerbild** — gelöschtes Dokument ist nicht von nie existierendem unterscheidbar
- **Vor Entschlüsselung** — Gate-Check vor jedem Zugriff auf Chiffretext

### 6.2 Prüfpunkte (vollständig, INV-GATE-COMPLETENESS)

**Jeder einzelne Lesepfad MUSS vor der Rückgabe synchron O(1) gegen das ReadGate prüfen:**

| Lesepfad | Gate-Check-Stelle | Verhalten bei `Erased` |
|----------|-------------------|-----------------------|
| `get`, `get_at_snapshot` | Vor Entschlüsselung | `Err(ContextraError::Erased)` |
| `scan`, `scan_prefix` | Je Eintrag vor Ausgabe | Eintrag still auslassen |
| Vektor-/Textsuche | Auf Kandidatenebene vor Fusion | Kandidat verwerfen, auf `k` auffüllen |
| Hybrid-Suche | Vor Fusion und Ranking | Kandidat ignorieren |
| Graph-Traversal | Je Kante und je Zielknoten | Pfad bricht ab |
| Export | Je Eintrag | Gesperrte auslassen |
| MCP-Tools | Über darunterliegende Pfade | Wie jeweiliger Pfad |
| Kompaktionen | `check_range` → `ContainsErased` | Range nicht in Ergebnis aufnehmen |

**Zugriffs-Zeitrichtlinie:** Das ReadGate wird immer mit dem **aktuellen** Stand geprüft, nicht mit historischen Rechten. Ein Zeitreise-Query auf `as_of=T` liefert dennoch `Erased`, wenn das Ziel heute gelöscht ist.

### 6.3 ReadGate-Trait (kanonisch in `contextra-erasure`)

```rust
pub trait ReadGate: Send + Sync {
    /// Synchroner O(1) Check. Muss vor jeder Entschlüsselung aufgerufen werden.
    fn check(&self, key: &[u8]) -> GateDecision;

    /// Bereichsprüfung für LSM-Kompaktionen.
    fn check_range(&self, start: &[u8], end: &[u8]) -> RangeGate;
}

pub enum GateDecision { Open, Erased, Restricted }
pub enum RangeGate   { AllOpen, ContainsErased, RestrictedNeverDiscard }
```

---

## 7 ErasureLayer — Schnittstelle für Layer-Implementierer

**Kanonisch in `contextra-erasure` (E0) definiert.** Implementiert von E1-Crates.

```rust
pub trait ErasureLayer: Send + Sync {
    fn id(&self) -> LayerId;
    fn key_coverage(&self) -> KeyCoverage;  // Full | None

    fn resolve(&self, target: &EraseTarget)
        -> BoxFuture<'_, Result<LayerScope, ErasureError>>;

    fn on_gate(&self, scope: &LayerScope)
        -> BoxFuture<'_, Result<(), ErasureError>>;

    fn purge(&self, scope: &LayerScope, cx: &PurgeCx)
        -> BoxFuture<'_, Result<PurgeOutcome, ErasureError>>;

    fn witness(&self, scope: &LayerScope, scanner: &Scanner<Armed>)
        -> BoxFuture<'_, Result<LayerWitness, ErasureError>>;
}
```

**Invarianten für Implementierer:**
1. `purge` ist idempotent: zweiter Aufruf = no-op
2. `witness` ist rein lesend
3. Logs enthalten nie Klartextinhalte des Ziels (INV-12)
4. E1-Crates importieren `ErasureLayer` aus `contextra-erasure`, **nicht** aus `contextra-ports`

**Layer-Zuordnung:**

| Layer | Crate | Saga-Prio |
|-------|-------|-----------|
| StoreLayer | `contextra-store` | WAL-Drain (c), SSTable-Rewrite (d) |
| TextLayer | `contextra-text` | Segment-Rewrite (b) |
| VectorLayer | `contextra-vector` | HNSW-Reparatur (e) |
| GraphLayer | `contextra-graph` | Entitäten/Kanten (f) |
| CryptoLayer | `contextra-crypto` | Slot-Bereinigung (g) |
| SnapshotRevoker | `contextra-mvcc` | Snapshot-Widerruf (a) |

---

## 8 Lineage-System

### 8.1 Zweck

Jede Ableitung (D-Klasse) verweist auf ihr Quelldatum. Bei Löschung des Quelldatums liefert der Lineage-Index alle betroffenen Ableitungen.

### 8.2 Datenmodell (kanonisch in `contextra-db`)

```rust
pub struct LineageRef {
    pub source_class: DataClass,     // K oder P
    pub source_id: SourceId,         // DocId oder SubjectRef
    pub source_version: u64,
}

pub struct DerivedRef {
    pub layer: LayerId,
    pub segment_id: u64,
    pub offset: u64,
}
```

### 8.3 Regeln

1. Jeder Schreibvorgang in D-Klasse MUSS eine `LineageRef` angeben.
2. `LineageIndex::derived_of(source, version)` liefert alle `DerivedRef`s.
3. Die Purge-Phase (Schritt 4) nutzt den Lineage-Index, um alle Ableitungen zu finden.
4. Ohne Lineage ist Klasse I nicht erreichbar (§5.4).

---

## 9 Zeitreise

### 9.1 Zwei unabhängige Achsen

| Achse | Typ | Semantik |
|-------|-----|----------|
| **Transaktionszeit** (`as_of`) | `u64` (Sequenznummer) | „Was war zum Zeitpunkt X im System bekannt?" |
| **Gültigkeitszeit** (`valid_at`) | `i32` (Kalendertage seit 1970-01-01) | „Was galt zum fachlichen Zeitpunkt Y?" |

### 9.2 Regeln

1. `as_of` und `valid_at` sind unabhängig kombinierbar.
2. `None` bedeutet „offen" (kein Ende / kein Anfang), nicht `0`.
3. ReadGate wird immer mit dem **aktuellen** Stand geprüft, unabhängig von `as_of`.
4. Spätere Schreibvorgänge ändern frühere `as_of`-Antworten nicht (Z-T2).

---

## 10 Concurrency-Barrieren

### 10.1 CommitBarrier (kanonisch in `contextra-ports`)

```
Writer 1: shared()    → [schreibt]
Writer 2: shared()    → [schreibt]
Saga:     exclusive() → [setzt Gate] → drop permit
```

`shared()` erlaubt mehrere parallele Writer. `exclusive()` wartet auf alle shared-Permits und blockiert neue. Schützt den Gate-Gruppen-Commit.

### 10.2 PurgeBarrier (kanonisch in `contextra-ports`)

```
Kompaktions-Job: compaction() → [kompaktiert]
Saga-Purge:      purge()      → [exklusiver Purge]
```

`purge_waiting()` signalisiert wartenden Kompaktionen, dass ein Purge ansteht — Kompaktionen sollen ihren Slot zügig freigeben.

---

## 11 Fehlerverhalten

### 11.1 Fehler-Hierarchie

| Kategorie | Fehler | Verhalten |
|-----------|--------|-----------| 
| **Transient** | I/O-Fehler im Index | Wiederholung mit Backoff, Zustand bleibt |
| **Nach Gate** | Fehler nach Schritt 2 | Datum bleibt `Gated`, Bereinigung nachgeholt |
| **Nach KeyRevoked** | Fehler nach Schritt 3 | Datum bleibt `KeyRevoked`, Bereinigung nachgeholt |
| **Im Schritt 3** | Schredding-Fehler | Operation schlägt fehl, Datum bleibt `Gated`, Aufrufer erhält `Err` |
| **Im Lesepfad** | Fehlender Schlüssel | `Erased` — kein Unterschied zu „nie existiert" |
| **Crypto** | AEAD-Fehler | Einheitlicher Fehler, nicht weiter unterscheidbar |

### 11.2 Fail-Closed-Prinzip

- Fehlender Schlüssel → Fehler, nie Klartext-Fallback
- Fehlender ReadGate → Öffnungsfehler (bei `GuardPolicy::Required`)
- Unbekannter Gate-Zustand → Ablehnen
- Jeder Fehler im Löschpfad MUSS im Journal erscheinen

---

## 12 Testkatalog (Löschkette)

| Kennung | Prüft | Kategorie |
|---------|-------|-----------| 
| **T-DEL-1** | Nach Insert+Flush: alle persistierten Bytes sind verschlüsselt (kein Klartext-Canary in Dateien) | Integration |
| **T-DEL-2** | Nach Löschung+Purge+Compaction: Lesepfad liefert `Erased`; Canary-Scan `residue_matches == 0` | Integration |
| **T-DEL-3** | Löschnachweis-Verifikation schlägt mit falschem Schlüssel fehl | Unit |
| **T-DEL-4** | Nach Löschung und Neustart (ReadGate-Reload) bleibt Datum `Erased` | Integration |
| **T-DEL-5** | Ohne `EncryptionConfig::Required` erreicht Löschung maximal Klasse L, nicht Klasse K | Integration |
| **T-DEL-6** | Nach Löschung findet weder Volltext- noch Vektorsuche noch Graph-Traversal das Dokument | Integration |
| **T-DEL-7** | Positivkontrolle: `Scanner<Unarmed>` findet eingebettetes Canary-Muster → wird zu `Scanner<Armed>` | Unit |
| **T-DEL-8** | Collection-Drop widerruft CollectionKEK; alle Dokumente der Collection `Erased` | Integration |
| **T-DEL-9** | Subject-Löschung: SubjectKey vernichten → alle P-Daten des Subjects `Erased` (O(1)) | Integration |
| **T-DEL-10** | Lineage-Kaskade: Quelldatum gelöscht → alle D-Ableitungen bereinigt, `residue_matches == 0` | Integration |
| **T-DEL-11** | ReadGate blockiert alle Lesepfade aus §6.2 — vollständige Prüfliste | Integration |
| **T-DEL-12** | Crash in Phase 3 → Neustart → Saga wird korrekt fortgesetzt; Gate ist sofort aktiv | Integration |
| **T-DEL-13** | Restrict-Modus: Lesezugriff nur mit Audit-Eintrag; ohne Eintrag → `Err(RestrictedReadAuditRequired)` | Integration |
| **T-DEL-14** | `purge` ist idempotent: zweiter Aufruf auf derselben Saga-Phase = no-op, kein Fehler | Unit |
| **T-DEL-15** | `verify_key_absent` schlägt fehl, wenn Slot nicht vollständig `0x00` ist | Unit |

**Test-Implementierungsregel:** Jeder Test MUSS die vollständige Saga inkl. Purge durchlaufen. Ein Test, der nur Gate setzt aber keinen Purge aufruft, prüft maximal Klasse L.

---

## 13 Invarianten

| ID | Invariante |
|----|------------|
| **INV-1** | Kein persistiertes Byte ändert sich ohne Formatvermerk. |
| **INV-2** | Jeder öffentliche Pfad bleibt auflösbar. |
| **INV-3** | `forbid(unsafe_code)` außerhalb der Inseln (`sys`, `simd`). |
| **INV-4** | Kein `unwrap`/`expect`/`panic!`/`todo!`/`unimplemented!` in Nicht-Test-Code. |
| **INV-5** | Gleiche Eingabe → gleiche Rangfolge, Tie-Break ID aufsteigend. |
| **INV-6** | WAL, Manifest, Flush, Recovery-Verhalten unverändert bei Umbau. |
| **INV-7** | Keine Compliance-Aussagen. „Gelöscht" nur mit gemessener Stufe. Keine Skalierungsbehauptung ohne belegten Bench-Lauf. |
| **INV-8** | Abhängigkeitskanten nur abwärts (R1). |
| **INV-9** | `Completed`, `Refused` und `Lifted` sind terminale Saga-Zustände. |
| **INV-10** | Gate wird nie zurückgenommen. |
| **INV-11** | Schlüsselbytes werden mit `zeroize` genullt vor Drop. Kein `Clone` für Schlüsseltypen. |
| **INV-12** | Logs/Traces enthalten nie Klartextinhalte des Löschziels oder PII. |
| **INV-GATE-COMPLETENESS** | Jeder Lesepfad aus §6.2 prüft das ReadGate. Eine Ausnahme ist ein Implementierungsfehler. |
| **INV-WITNESS-MINT-1** | `Scanner<Armed>` kann nur nach nachgewiesener Positivkontrolle instanziiert werden. |
| **INV-LINEAGE** | Jede D-Klasse-Schreiboperation ohne `LineageRef` ist ein Implementierungsfehler. |

---

## 14 Konfiguration

```rust
pub struct ContextraConfig {
    pub data_dir: PathBuf,
    pub encryption: EncryptionConfig,
    pub tenant_policy: TenantPolicy,
    pub guard_policy: GuardPolicy,
    pub data_class_policy: DataClassPolicy,
    pub erasure: ErasureConfig,
}

pub enum EncryptionConfig {
    /// Kein Crypto-Shredding möglich. Löschung erreicht maximal Klasse L.
    None,
    /// Verschlüsselung obligatorisch. Passphrase wird als SecretString übergeben.
    Required { passphrase: SecretString },
}

/// SecretString: ZeroizeOnDrop, kein Debug, kein Clone, kein Serialize.
pub struct SecretString(String);

pub enum GuardPolicy {
    Optional,  // ReadGate kann, muss aber nicht gesetzt sein
    Required,  // Ohne ReadGate kein Öffnen — für regulierte Umgebungen
}

pub enum DataClassPolicy {
    Legacy,    // Schreibmethoden ohne Klasse erlaubt (Migrationspfad)
    Enforced,  // Jeder Schreibvorgang MUSS Klasse deklarieren
}

pub struct ErasureConfig {
    pub ledger_dir: PathBuf,        // MUSS außerhalb von data_dir liegen
    pub batch_window_secs: u64,     // Intervall für run_purge_window
    pub disk_reserve_bytes: u64,    // Mindest-Plattenreserve vor Purge (min. 1 GB)
}
```

**Konfigurationsvalidierung:** `Contextra::open` MUSS vor dem Öffnen prüfen:
- `data_dir` ist vorhanden und beschreibbar
- `erasure.ledger_dir` ist nicht ein Unterverzeichnis von `data_dir`
- `erasure.disk_reserve_bytes >= 1_073_741_824` (1 GB)
- Bei `EncryptionConfig::Required`: Passphrase ist nicht leer

---

## 15 Layering-Gate (Ersatz für das defekte `gate_layering.py`)

Das Layering-Gate ist kein externes Python-Skript, sondern ein Cargo-Integration-Test in `crates/contextra-testkit/tests/layering.rs`. Er nutzt `cargo metadata` und prüft:

1. Für jede Crate: `default = []` (R3)
2. Für jede E0/E1-Crate: Keine Abhängigkeit auf E2/E3-Crates (R1) — auch nicht über Feature-Unifikation
3. Für E1-Crates: Kern-Closure ⊆ E0, max. 5 Mitglieder
4. Für alle Crates: `[lints] workspace = true` ist gesetzt
5. Kein Crate aus der Entfernt-Liste (§2.2) ist als workspace-Member aktiv

Der Gate-Test läuft in CI als separater Schritt vor dem Build. Schlägt er fehl, bricht CI ab. Es gibt keine `feature_closure_allowed`-Ausnahme.

---

## 16 Glossar

| Begriff | Definition |
|---------|------------|
| **Canary** | Bekanntes Byte-Muster, das über den Schreibpfad eingefügt wird, um den Scanner zu kalibrieren |
| **Crypto-Shredding** | Vernichtung des Schlüssels macht Chiffretext unlesbar, ohne physisches Löschen |
| **DEK** | Data Encryption Key — verschlüsselt ein einzelnes Payload |
| **Gate** | Synchrone In-Memory-Lesesperre, die Zugriff auf gelöschte Daten verhindert |
| **KEK** | Key Encryption Key — umhüllt DEKs einer Gruppe |
| **Lineage** | Herkunftsverweis von abgeleiteten Daten (D-Klasse) auf ihr Quelldatum |
| **PersonalRef** | 128-Bit-Zufalls-ID (`[u8; 16]`, kein Clone) als Verweis auf P-Daten in K-Dokumenten |
| **Saga** | Die 6-Schritte-Löschsequenz mit Journal und Recovery |
| **SubjectRef** | 128-Bit-Zufalls-ID (`[u8; 16]`, ZeroizeOnDrop, kein Clone), die einen Betroffenen identifiziert |
| **SubjectKey** | Aus CollectionKEK abgeleiteter Schlüssel für einen Betroffenen |
| **SecretString** | ZeroizeOnDrop-Wrapper für Passphrasen — kein Debug, Clone, Serialize |
| **Vault** | Dateibasierter persistenter Schlüsselspeicher in `contextra-erasure` |
