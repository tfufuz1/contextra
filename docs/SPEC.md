# Contextra — Gesamtspezifikation (normativ)

**Version:** 2.1
**Datum:** 19. November 2026
**Status:** FINAL — Einzige Quelle der Wahrheit
**Geltungsbereich:** Gesamter Workspace `contextra`

---

## 0 Geltung und Rangfolge

Dieses Dokument und die zugehörigen Crate-Spezifikationen in `docs/crates/` sind die **einzige normative Referenz** für das Projekt. Frühere Spezifikationen, Coupling-Reports, Audit-Berichte, Verifikationsberichte, `reports/`-Verzeichnisse, `docs/diagnostics/`-Inhalte und `crates/*/docs/decoupling/`-Verzeichnisse sind ersetzt und zu löschen. Code und Spezifikation beschreiben denselben Soll-Zustand.

> **Zu löschende Altlastdokumente (Schritt 0 der Bereinigung):**
> `reports/`, `ALTLASTEN-AUDIT.md`, `LEGACY_CLEANUP_SPEC.md`,
> `ANALYSIS-E0-CONTRADICTIONS.md`, `docs/diagnostics/`,
> `crates/*/docs/decoupling/`, jegliche `capabilities.toml`,
> `tools/gate_layering.py`, `tests/layering.rs`

**Herkunft der Löschkette.** Die Löschkette ist in `docs/Finale Löscharchitektur.md` final konsolidiert. Sie übernimmt die Fast-Path-Saga aus der Spezifikation v3.1, das zweitaktige Ausführungsmodell der Neukonzeption, die Speicher- und Verifikationsprimitive des Erasure Core sowie die Härtungs- und Governance-Regeln. Das Verzeichnis `docs/delete/` ist Quellarchiv. Es ist nach der Extraktion nicht mehr normativ und wird archiviert, nicht gelöscht. Abweichungen zwischen Quellen sind in §17 aufgeführt.

**Rangfolge:**
1. Diese Gesamtspezifikation (`docs/SPEC.md`), einschließlich der Löschkette gemäß `docs/Finale Löscharchitektur.md`
2. Crate-Spezifikationen (`docs/crates/*.md`), normativ pro Crate
3. Crate-lokale `README.md`-Dateien, informativ

MUSS, DARF NICHT und SOLL gelten nach RFC 2119.

### 0.1 Änderungen gegenüber Version 2.0

| Bereich | Version 2.0 | Version 2.1 |
|---|---|---|
| Löschsaga | 6 Schritte, Schlüssel vor Purge | 9 Schritte plus Hygiene, Purge vor Schlüsselvernichtung (INV-DEL-ORDER-1) |
| Erfüllungsklassen | L, K, I, M | L, P, C\*, C1–C3, M mit Siegeln 1–4 |
| Schlüsselmodell | Abgeleitet per HKDF aus KEK | Zufällig und gewrappt (DocDek, ArtKey, KEK je Epoche), HKDF nur für strukturelle Schlüssel |
| Argon2id (K1) | m = 256 MB, t = 4 | m ≥ 64 MiB, t ≥ 3 |
| Signatur der Berichte | Ed25519, Feature `signing` | Ed25519 Pflicht, kein Feature (R3) |
| Schichten | Layer-Liste | Schichten S1–S8 mit Oberflächen-Inventar-Gate |
| Lesepfad | Prüfpunkte §6.2 | Zentrales `ReadGate` in `TenantScopedStorage`, `LiveMask` als Lesewahrheit |

---

## 1 Produktdefinition

### 1.1 Was Contextra ist

Contextra ist eine **souveräne, offline betreibbare Such- und Wissens-Engine** für Unternehmen und Behörden. Sie bietet:

1. **Hybrid-Suche** (Vektor + Volltext + Graph) über lokale Datenbestände
2. **Versionierung und Zeitreise** über Daten und Regeln
3. **DSGVO-konforme Löschung** mit gemessener Löschwirkung und belegbarem Löschnachweis
4. **Souveränität**: keine Telemetrie, kein Fernzugriff, lokale Schlüssel

Contextra ist eine **eingebettete Rust-Bibliothek** (kein eigenständiger Server). Die primäre Schnittstelle ist die Rust-API, sekundär MCP (Model Context Protocol) über stdio.

### 1.2 Nicht-Ziele (dauerhaft)

- Keine automatisierte Entscheidung über natürliche Personen (Art. 22 DSGVO)
- Kein Training / Fine-Tuning mit personenbezogenen Daten
- Kein Dokumentenmanagementsystem
- Kein Compliance-Versprechen ohne gemessene Stufe. „DSGVO-konform" ist eine Bewertung des Verantwortlichen, keine Eigenschaft der Software (INV-7, INV-CLAIMS-1)
- Keine Skalierungsbehauptung ohne belegten, erfolgreichen Durchlauf im Bench-Harness
- Keine Beschreibung entfernter Funktionen in `README.md` oder Dokumentation
- Keine Aussage „100 % gelöscht", „unwiederbringlich" oder „restlos entfernt" in Berichten oder Außentexten

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

**Excluded:** `vault` (dormant; darf nicht reaktiviert werden ohne G-REACTIVATE). Seine frühere Schlüsselverwahrung liegt bei `KeyAnchor` und `contextra-crypto`.

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
| **R1** | Kanten zeigen **ausschließlich abwärts** (E3→E2→E1→E0). Keine Kante von E0/E1 nach E2/E3. Keine optionalen Abhängigkeiten, die eine Aufwärtskante erzeugen, auch nicht über Feature-Unifikation. `feature_closure_allowed` existiert nicht. |
| **R2** | Dev-Abhängigkeiten auf **höhere Ebenen** sind verboten. `contextra-testkit` (E3) darf nie in `[dev-dependencies]` einer E1- oder E2-Crate erscheinen. |
| **R3** | `default`-Features MÜSSEN in allen Crates leer sein: `default = []`. Löschpfade sind nie feature-gesteuert. |
| **R4** | Ein Konzept hat **genau einen Besitzer**. Keine Typ-Dubletten. Die kanonischen Besitzer sind in §2.5 aufgelistet. |
| **R5** | `#![forbid(unsafe_code)]` in allen Crates außer `sys` und `simd`. |
| **R6** | Kein `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!` in Nicht-Test-Code. Die Workspace-Lints (§2.6) erzwingen dies. |

### 2.4 Architekturdiagramm

```
┌──────────────────────── E3: Endpunkte ──────────────────────┐
│  mcp          testkit (Erasure Lab)       bench             │
└──────────────────────────┬──────────────────────────────────┘
                           │ darf E0, E1, E2 nutzen
┌──────────────────────── E2: Integration ────────────────────┐
│  contextra ──► db ──► engine (deprecated = pub use db::*)   │
│   db: Verdrahtung der Löschsaga, Schichtregistrierung,      │
│       Lineage. Koordinator-Logik liegt in erasure (E0)      │
└──────────────────────────┬──────────────────────────────────┘
                           │ darf E0, E1 nutzen
┌──────────────────────── E1: Domäne ─────────────────────────┐
│  store    vector    text    graph    rank                    │
│  mvcc     crypto    checkpoint                              │
│  (je Schicht: Ebene-A-Purger + Ebene-B-Scanner)             │
└──────────────────────────┬──────────────────────────────────┘
                           │ darf nur E0 nutzen
┌──────────────────────── E0: Fundament & Ports ──────────────┐
│  ids   env   sys   durable-fs   simd   types   ports        │
│  privacy   erasure (Saga-Typen, ReadGate, ErasureLayer)     │
└─────────────────────────────────────────────────────────────┘
```

**Klarstellung zur Schichtentrennung:**

- `contextra-types` und `contextra-ports` liegen normativ auf **Ebene E0 (Fundament & Ports)**.
- `types` enthält kanonische Typen, IDs und Datenstrukturen (`DocId`, `ScoredDocument`, `ContextraError`) ohne interne I/O- oder Laufzeitabhängigkeiten.
- `ports` enthält die standardisierten Trait-Verträge (`StorageEngine`, `VectorIndex`, `TextIndex`, `GraphIndex`, `CommitBarrier`, `PurgeBarrier`, `KeyAnchor`, `Purgeable`) und hängt ausschließlich von `env` und `ids` ab.
- `ports` hängt nicht von `types` ab. `types` erreicht `privacy` (`DataClass`, `Validated<T>`). `privacy` erreicht `ports` (`Purgeable` für den Surrogat-Vault). Eine Kante `ports → types` würde den Zyklus `types → privacy → ports → types` schließen. Port-lokale Typen (`ScoredId`, `PortError`, `KeyError`) sind keine zweiten Definitionen von `SearchHit` oder `ContextraError` (R4). `contextra-db` mappt sie.
- E1-Crates implementieren die Traits aus `ports` und nutzen Typen aus `types`. Alle Kanten zeigen strikt abwärts (R1).

### 2.5 Kanonische Typbesitzer (R4)

| Typ | Kanonischer Besitzer |
|-----|---------------------|
| `DocId`, `TenantId`, `CollectionId`, `SubjectRef`, `PersonalRef` | `contextra-ids` |
| `ErasureId`, `GateId`, `OperatorRef`, `Seq`, `Timestamp`, `ScopeId` (128 Bit) | `contextra-ids` |
| `TenantScoped<T>`, `TenantScopeViolation`, `IdError` | `contextra-ids` |
| `Clock`, `SystemClock`, `SeededRng`, `MetricsSink` | `contextra-env` |
| `DataClass`, `Validated<T>`, `EntityRecognizer` | `contextra-privacy` |
| `ErasureScope`, `EraseTarget`, `SagaState`, `RetentionAttestation`, `ErasureTicket` | `contextra-erasure` |
| `ReadGate`, `GateDecision`, `RangeGate`, `ErasedSet`, `LiveMask`, `Slot` | `contextra-erasure` |
| `Scanner<S>`, `Armed`, `Unarmed`, `ControlEvidence`, `LayerWitness`, `ResidualReport` | `contextra-erasure` |
| `ErasureLayer`, `LayerId`, `LayerScope`, `PurgeCx`, `PurgeOutcome`, `KeyCoverage` | `contextra-erasure` |
| `DeletionProof` (V5), `Siegel`, `MeasureClass` (`L`, `P`, `C*`, `C1`–`C3`, `M`) | `contextra-erasure` |
| `EffaceableKeySlot`, `KeyAnchor`, `Custody` (K0–K3), `WrappedBlob`, `DestroyAttestation` | `contextra-ports` |
| `CommitBarrier`, `PurgeBarrier` | `contextra-ports` |
| `ContextraError`, `DocumentRecord`, `SearchHit` | `contextra-types` |
| `SecretKey`, `MasterKey`, `Kek` (je Collection und Epoche), `DocDek`, `ArtKey`, `KeyTable`, `ErasureKey` | `contextra-crypto` |
| `LineageRef`, `DerivedRef`, `LineageIndex` | `contextra-db` |

**Absolutregel:** Jeder Typ aus dieser Tabelle darf im gesamten Workspace nur einmal definiert sein, beim kanonischen Besitzer. Re-Exporte sind erlaubt, parallele Definitionen sind verboten. Der Typ `SubjectKey` wird durch `DocDek` ersetzt (siehe §3.2).

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

| Klasse | Name | Verschlüsselung | Löschbar pro Einheit |
|--------|------|-----------------|----------------------|
| **K** | Unternehmenswissen | `DocDek` je Betroffenen-Einheit bzw. `ArtKey` je Artefakt, gewrappt unter `KEK[coll, e]` | Ja, über Einheit oder Collection-Drop |
| **P** | Personenbezogene Daten | `DocDek` je Betroffenen-Einheit | Ja, atomar, unabhängig von der Datenmenge |
| **D** | Abgeleitete Daten | `ArtKey` des Index-Artefakts, mit obligatorischer `LineageRef` | Ja, kaskadiert über `LineageRef` (INV-DERIVED-1) |
| **E** | Löschmaterial | `ErasureKey` und Journal-Schlüssel `K_saga` | Nach Abschluss der Saga bzw. Aufbewahrungsfrist |

### 3.2 Betroffenen-Einheiten (P-Klasse)

**Kernentwurfsentscheidung:** Die atomare Löschbarkeit ergibt sich aus der Schlüsselhierarchie (§4), **nicht** aus physischem Löschen einzelner Bytes.

**Regeln:**

1. **Einheit.** Eine Betroffenen-Einheit ist die kleinste Menge von Records, die ohne Kollateralverlust für einen Betroffenen gelöscht werden kann. Standard: ein Dokument mit genau einem Betroffenen. Jede Einheit erhält genau einen `DocDek`, der bei `remember` erzeugt wird.
2. **Referenzen.** P-Daten werden über `SubjectRef` adressiert. `SubjectRef` ist eine 128-Bit-Zufalls-ID (`[u8; 16]`, `ZeroizeOnDrop`, kein `Clone`), erzeugt über `OsRng::try_fill_bytes`. Die Zuordnung `SubjectRef ↔ natürliche Person` liegt außerhalb des Systems.
3. **Mehrfachbezug.** Hat ein Dokument mehrere Betroffene, wird es in Segmente zerlegt. Segmente mit gleicher Betroffenen-Menge teilen einen `DocDek`. Die Behandlung bei Löschung ist in §5.6 geregelt.
4. **K-Daten dürfen nicht `SubjectRef` enthalten.** Personenbezug in K-Dokumenten wird ausschließlich über `PersonalRef`-Verweise realisiert.
5. **D-Daten tragen `LineageRef`.** Jeder Schreibvorgang in D-Klasse MUSS eine `LineageRef` angeben. Ohne Lineage ist Klasse I nicht erreichbar (INV-LINEAGE).

### 3.3 K-Schreibpfad-Schutz

K-Schreibpfade akzeptieren ausschließlich `Validated<T>` aus `contextra-privacy`. Diese Typengarantie verhindert zur Compile-Zeit, dass unerkannte PII in Klasse-K-Bestände gelangt.

---

## 4 Schlüsselarchitektur

### 4.1 Hierarchie

```
Wurzel (Custody-Slot K1 | K2 | K3)
 └─ KEK[coll, e]             je Collection und Epoche, zufällig, gewrappt unter der Wurzel
     ├─ DocDek[einheit]      je Betroffenen-Einheit, 32 Byte zufällig, gewrappt unter KEK
     └─ ArtKey[artefakt]     je Index-Artefakt (Text-Segment, Graph-Snapshot), zufällig, gewrappt unter KEK
K_eng                        Struktur und Integrität (keine Nutzdaten), HKDF aus Wurzel
K_ref[tenant, e]             Referenzschlüssel für Referenz-Hashes (§12), zufällig, gewrappt unter KEK
K_sc/*                       Zweckschlüssel der Sidecar-Ketten, HKDF aus K_eng
K_saga[erasure_id]           ephemer, verschlüsselt den ScopeDescriptor im Journal (§5.3 Schritt 1)
```

**Regel:** Zwischenschlüssel (`KEK`, `DocDek`, `ArtKey`, `K_ref`) werden **niemals** per HKDF aus übergeordneten Schlüsseln abgeleitet. Nur so können sie durch Vernichtung unwiederbringlich entfernt werden. HKDF ist ausschließlich für strukturelle Schlüssel (`K_eng`, `K_sc/*`) zulässig.

Jeder Schlüssel hat genau einen Zweck und ein Label im Zweckregister (INV-KEYPURPOSE-1). Kein Schlüssel hat mehr als einen Zweck.

### 4.2 Custody-Stufen

| Stufe | Wurzel | Erreichbare Klasse |
|---|---|---|
| **K0** | keine Verschlüsselung (nur Entwicklung, `doctor` rot) | nur L, P |
| **K1** | Argon2id aus Passphrase (m ≥ 64 MiB, t ≥ 3, ≥ 250 ms), Software-Keystore | C1 mit verpflichtender Annahme `A-FLASH` / `KeyStoreRemanence` |
| **K2** | Siegelschlüssel in manipulationsresistentem, nicht exportierbarem Modul (TPM 2.0, HSM, OS-Keyring). Eignung je Plattform ist Spike S-K2 | C2 nur nach bestandener Konformitätssuite |
| **K3** | externer KMS/HSM mit Zeitfenster-Löschung und Vernichtungsattest | C3 mit Anbieter-Vernichtungsattest |

K2 und K3 sind als Port-Implementierungen **vorgesehen**, aber nicht ausgeliefert, solange ihre Konformitätssuite nicht besteht. Der Port liefert bis dahin `CapabilityUnsupported`, nie einen Stub, der Erfolg meldet (REQ-DEL-N14).

**Hinweis zur Eignung:** Hardware-Effaceable-Konzepte dienen dem Schnell-Löschen eines Volumenschlüssels. Sie sind kein Mechanismus für beliebige Einzelschlüssel von Anwendungen. Eine Plattform wird nur als K2 anerkannt, wenn die Konformitätssuite nachweist, dass Schlüsselmaterial nach Vernichtung nicht aus Resten rekonstruierbar ist.

### 4.3 Port `EffaceableKeySlot` / `KeyAnchor` (in `contextra-ports`)

```rust
pub trait EffaceableKeySlot: Send + Sync {
    fn custody(&self) -> Custody;                              // K1 | K2 | K3
    fn wrap(&self, material: &Zeroizing<[u8; 32]>, ad: &[u8])
        -> Result<WrappedBlob, KeyError>;
    fn unwrap(&self, blob: &WrappedBlob, ad: &[u8])
        -> Result<Zeroizing<[u8; 32]>, KeyError>;
    fn rotate(&self) -> Result<SlotGeneration, KeyError>;      // neue Wurzel-Generation, alte bleibt bis destroy
    fn destroy(&self, generation: SlotGeneration)
        -> Result<DestroyAttestation, KeyError>;
}
```

`DestroyAttestation` enthält Custody, Verfahren (Zeroization), Zeitpunkt (aus `Clock`) und einen Beleg der Implementierung. Nur K1 liefert einen lokalen Beleg ohne Dritt-Attest.

### 4.4 KEK-Epochen und Epoch Seal (behebt F-01)

**Problem.** Wird nur der `DocDek` vernichtet, liegt sein gewrappter Blob als Rest auf dem Flash. Wer KEK und Passphrase besitzt, kann ihn entschlüsseln.

**Lösung.** Die KEK wechselt epochenweise. Ein **Epoch Seal** führt folgende Schritte durch:

1. Erzeugen von `KEK[coll, e+1]`, gewrappt unter der Wurzel.
2. Neu-Wrap aller lebenden `DocDek`s und `ArtKey`s unter `KEK[e+1]` (kein Datenrewrite; Aufwand O(Anzahl Schlüssel)).
3. Atomares Neuschreiben der `KeyTable`.
4. Vernichten von `KEK[e]` und seiner Wrap-Kopie.
5. Bei K1 mit Option `rotate_passphrase_on_seal`: Passphrasenwechsel, sodass die alte Wurzel nicht mehr ableitbar ist.
6. Bei K2/K3: `rotate()` und `destroy()` am Slot.

**Auslöser.** Zeitgesteuert (`kek_epoch_len`, Startwert 30 Tage), auf Anforderung (`seal-now`) und automatisch, wenn seit dem letzten Seal eine Löschung stattfand und `seal_after_deletion_max` (Startwert 7 Tage) erreicht ist. Löschungen innerhalb derselben Collection und Epoche teilen sich einen Seal.

**Klassenwirkung.** C1 bis C3 gelten erst nach dem Seal, der die Löschung umfasst. Davor weist der Bericht `C*` (vorläufig) mit `R-KEKEPOCH until` aus.

### 4.5 DocDek-Regeln

1. Jede Betroffenen-Einheit erhält genau einen `DocDek`, erzeugt bei `remember`.
2. Jeder Record in WAL, SSTable und Blob-Segment wird mit dem `DocDek` seiner Einheit verschlüsselt. Ein Block mit Records mehrerer Einheiten enthält Records unter verschiedenen Schlüsseln. Die Blockverschlüsselung unter `K_eng` bleibt äußere Schicht. Die Vernichtung eines `DocDek` macht deshalb **keine anderen Records unlesbar**.
3. Vektor-Payload (fp32 und SQ8) im persistenten Snapshot liegt je Slot unter dem `DocDek` (Siegel im Snapshot).
4. Der gewrappte `DocDek` liegt in der **KeyTable** (Ersetzen-Semantik), nicht in einem Append-Log.
5. Vernichtung: Eintrag mit Nullen überschreiben, `fsync`, Eintrag entfernen, `KeyTable` atomar ersetzen. Arbeitsspeicher wird zeroized (REQ-KEY-008).
6. Der Key-Lock der Einheit wird über die gesamte Saga bis einschließlich Schritt 5 gehalten. Ein konkurrierendes `remember` oder Update auf derselben Einheit wird abgewiesen.

### 4.6 Record-Format und Restmetadaten (behebt F-07)

- **AEAD:** `AES-256-GCM-SIV`, Nonce 96 Bit zufällig, Tag 128 Bit. AD = `scope_id ‖ doc_key_id ‖ record_version`.
- **Padding:** Klartextlänge wird nach dem **Padmé**-Schema aufgefüllt (Overhead höchstens 12 %). Mindestlänge je Record `min_record_bytes` (Startwert 256).
- **Header:** `doc_key_id` und Sequenzbezug stehen **innerhalb** des unter `K_eng` verschlüsselten Blockbereichs, nicht im Klartext des Records (REQ-DEL-N15).
- **Wrap der Schlüssel:** AEAD-Wrap mit `AES-256-GCM-SIV`, zufällige 96-Bit-Nonce, AD = `key_id ‖ kek_epoch`. Die Nonce dient als Suchmuster im Nadelscan (§5.8).
- **Krypto-Agilität:** Jedes persistierte kryptografische Objekt trägt Algorithmus- und Formatkennung. Ein Wechsel ist per Re-Wrap oder Kompaktion ohne Datenverlust möglich. Das Notfallverfahren wird halbjährlich per `doctor crypto-drill` geübt.

### 4.7 Index-Artefakte (behebt F-03)

Postings und Graph enthalten viele Dokumente und können nicht unter einem `DocDek` liegen.

| Artefakt | Schlüssel | Bereinigung |
|---|---|---|
| Vektor-Payload im Snapshot | `DocDek` | Vernichtung des `DocDek` |
| Graph-Snapshot (Topologie, Slot-Nummern) | `ArtKey` der Snapshot-Generation | Neuaufbau, danach Vernichtung des `ArtKey` (§8) |
| Text-Segment (Postings, Term-Wörterbuch, Längen) | `ArtKey` je Segment | Neuschreiben mit neuem `ArtKey`, alter `ArtKey` sofort vernichtet (INV-ARTKEY-1) |

### 4.8 KeyTable und KeyRegistry

- `KeyTable` ist ein eigener Speicherbereich mit Ersetzen-Semantik. Nach jeder Vernichtung und jedem Seal wird die Segmentdatei ohne vernichtete Einträge neu geschrieben (REQ-DEL-N4).
- `KeyRegistry` (`MFKR`) führt Ereignisse (`created`, `destroyed`, `sealed`) mit Referenz-Hash. Es enthält kein Schlüsselmaterial und keinen Wrap-Blob (REQ-DEL-N5).

### 4.9 MasterKey-Persistenz und Konfiguration

Der Salt liegt in `<data_dir>/master.salt`. Die Wurzel wird je Custody-Stufe aus der Passphrase (K1, Argon2id) bzw. dem Slot (K2, K3) bezogen.

**`ContextraConfig::encryption`** hat exakt zwei Zustände:
- `Required { passphrase: SecretString }`: Verschlüsselung obligatorisch
- `None`: kein Crypto-Shredding möglich. Löschung erreicht maximal Klasse L

`encryption_passphrase: Option<String>` ist ein Verstoß gegen diese Spezifikation.

### 4.10 Krypto-Implementierungsregel (R4)

Kryptographische Primitive (`AES-256-GCM-SIV`, HKDF, Argon2id, Ed25519) sind **ausschließlich** in `contextra-crypto` implementiert. Ausnahme ist die Signatur der Berichte, die gemäß §5.9 `contextra-erasure` über den Port aus `contextra-crypto` nutzt. Jede andere Crate, die `aes-gcm-siv`, `sha2` oder `hmac` als direkte Dependency listet, verstößt gegen R4. Die Duplikate in `contextra-erasure` sind zu entfernen (Befund C-10, C-13).

---

## 5 Löschkette

### 5.1 Löschziel-Typen

| Ziel | Vernichteter Schlüssel | Auswirkung |
|------|------------------------|------------|
| **Einzeldokument / Betroffenen-Einheit** | `DocDek` | Diese Einheit und ihre D-Ableitungen |
| **Betroffener (P)** | `DocDek` aller Einheiten des `SubjectRef` | Alle P-Dokumente dieses Betroffenen über den Lineage-Index |
| **Collection** | `KEK[coll, *]` plus alle `DocDek`s und `ArtKey`s der Collection | Alle Dokumente der Collection, Index-Dateien entfallen |
| **Mandant** | alle `KEK`s des Mandanten | Alle Collections des Mandanten |

### 5.2 Zustände einer Löschung

Die Zustandsnamen der Finalen Löscharchitektur sind kanonisch. Die Namen aus Version 2.0 werden wie folgt abgebildet:

| Kanonisch (2.1) | Name in 2.0 | Klasse | Lesen |
|---|---|---|---|
| `Requested` | — | — | Nein (noch nicht autorisiert) |
| `Authorized` | — | — | Nein |
| `Frozen` | `Gated` | L | Nein |
| `LogicallyDeleted` | — | L | Nein |
| `PlaintextPurged` | `Purging` | P (Vorbereitung) | Nein |
| `PlaintextMeasured` | `Verifying` (Teil) | P | Nein |
| `KeyDestroyed` | `KeyRevoked` | P / C* | Nein |
| `CryptoMeasured` | — | C\* (vorläufig) | Nein |
| `EpochSealed` | — | C1 / C2 / C3 | Nein |
| `Hygiene` | — | — | Nein |
| `HygieneMeasured` | — | M | Nein |
| `Reported` | — | — | Nein |
| `Completed` | `Completed` | Abschluss (terminal) | Nein |
| `Restricted{basis, until}` | `Restricted` | — | Nur mit Audit-Eintrag |
| `Partial{outstanding}` | — | — | Je nach Mixed-Policy |
| `Pending(reason)` | `Pending` | unverändert | Abhängig vom Resume-Zustand |
| `Refused` | `Refused` | — | Ja (terminal) |
| `Lifted` | `Lifted` | — | Ja (terminal) |

**Terminale Zustände:** `Completed`, `Refused`, `Lifted` (INV-9).

**Übergänge:**

```
Requested → Authorized → Frozen → LogicallyDeleted → PlaintextPurged → PlaintextMeasured
          → KeyDestroyed → CryptoMeasured → EpochSealed → Hygiene → HygieneMeasured
          → Reported → Completed

Requested → Refused                       (Aufbewahrungspflicht, terminal)
Authorized → Restricted{basis, until}     (Art. 18, reversibel)
Restricted → Lifted                       (terminal)
Restricted → Frozen                       (Eskalation zur Löschung)
Jeder nicht-terminale Zustand → Pending(reason) → Resume-Zustand
```

**Invarianten:**
- Ab `Frozen` ist das Datum für alle Lesepfade unsichtbar (INV-10).
- Ab `KeyDestroyed` ist es kryptographisch unlesbar.
- Beide Wirkungen sind irreversibel.

### 5.3 Die Löschsaga (Schritte 1–9)

Jeder Schritt ist idempotent und wiederholbar. Die Reihenfolge ist normativ (INV-DEL-ORDER-1) und darf nicht vertauscht werden. Insbesondere gilt **Purge vor Schlüsselvernichtung**. Die Vorgängerversion (Schlüssel zuerst) ist verworfen.

Die Saga wird in zwei Takten ausgeführt:

- **Takt 1 (synchron, kurz, durabel):** Schritt 1 und das Setzen der Lesesperre. Ziel p99 ≤ 5 ms.
- **Takt 2 (gebündelt, asynchron):** Schritte 2 bis 9. Ein Worker sammelt Tickets bis zum Zeitfenster (Standard 5 s, Obergrenze `epoch_max` 60 s) oder zur Mengenschwelle (Standard 1.024 Löschungen).

#### Schritt 1 — Validierung, Intent und Lesesperre (Takt 1, Klasse L, Siegel 1)

1. **Eingang und Validierung.** Der zentrale Koordinator nimmt einen typisierten `ErasureScope` (`TenantId`, `CollectionId`, `ErasureTarget`) entgegen, prüft Mandantenbindung und Berechtigung (`Authorized`) und löst die 128-Bit-`scope_id` im Katalog auf den dichten `u32`-`Slot` auf. Existiert kein gültiger Scope, wird mit einem typisierten Fehler abgebrochen.
2. **Modus-Weiche.** Liegt eine gesetzliche Aufbewahrungspflicht (Art. 17 Abs. 3 DSGVO) oder ein Legal Hold vor, geht die Saga in `Restricted{basis, until}`: Lesesperre aktiv, keinerlei Daten- oder Schlüsselvernichtung. Für reguläre Löschaufträge (Art. 17 DSGVO) wird der irreversible Pfad `Frozen` eingeleitet.
3. **Durabilität und Speicherbarriere.**
   1. Der `ScopeDescriptor` wird unter einem ephemeren `K_saga[erasure_id]` verschlüsselt. Zusammen mit dem Referenz-Hash `BLAKE3-keyed(K_ref, scope)` wird er als `Frozen`-Datensatz in das `ErasureJournal` geschrieben. Damit enthält das Journal keinen Klartext-Personenbezug (Lösung des Lösch-Paradoxons).
   2. Parallele Löschanfragen teilen sich im Group-Commit (Wartefenster 1–2 ms) einen `fsync`/`fdatasync`. Schlägt der Flush fehl, liefert Takt 1 sofort `Err` zurück, ohne die Lesesperre zu aktivieren.
   3. Nach erfolgreichem `fsync` wird der Eintrag in das `ErasedSet` aufgenommen und das Bit in der `LiveMask` gesetzt. Ein `SeqCst`-Speicher-Fence (Dekker-Protokoll, §10.3) stellt sicher, dass kein Leser eine gesetzte `LiveMask` ohne durablen Journal-Zustand sieht und umgekehrt keinen veralteten Maskenzustand nach Rückgabe von Takt 1.
4. **Wirkung und Siegel 1.** Ab diesem Zeitpunkt verweigert das `ReadGate` jeden Lesezugriff auf den Scope, einschließlich laufender Snapshots, `as_of`-Zeitreisen, Checkpoints und Exporte. Der Koordinator stellt **Siegel 1** aus (Klasse L, `legal_state = Eingeschränkt`), trägt es in die `AuditChain` ein und gibt das `ErasureTicket` synchron zurück.

#### Schritt 2 — Logische Löschung und Lineage-Kaskade (Takt 2)

1. Der Koordinator erwirbt den Key-Lock der Einheit (`kv_locks.lock_for(id)`) und hält ihn ununterbrochen bis einschließlich Schritt 5.
2. Vor jeder destruktiven Änderung liest der Koordinator das unter dem `DocDek` verschlüsselte `ErasureManifest` (§5.4), um alle betroffenen Hilfsschlüssel zu erhalten.
3. Im WAL-Batch wird der logische Tombstone gesetzt (`LogicallyDeleted`). Über `INV-DERIVED-1` wird die Lineage-Kaskade für abgeleitete Artefakte angestoßen (Zusammenfassungen, Chunk-Präfixe, Abfrage- und Embedding-Kontexte). Ableitungen mit weiteren, nicht gelöschten Quellen werden als `stale` markiert oder neu berechnet.
4. Bei Mehrpersonen-Dokumenten greift die konfigurierte Mixed-Policy (§5.6).

#### Schritt 3 — Ebene A: Bereinigung und Text-Segment-Rotation (Takt 2)

Vor jeder Bereinigung wird der Übergang nach `PlaintextPurged` im Journal protokolliert. Alle Ebene-A-Purger arbeiten asynchron ohne `block_on`.

- **HNSW-Arena (S2):** Die Vektor-Slots der Einheit werden in `fp32` und `SQ8` deterministisch mit Nullen überschrieben (`zeroize`). Eine Kanten-Umverdrahtung findet **nicht** statt (VETO-F02). Die Slot-Nummer bleibt bis zum Hintergrund-Neuaufbau gesperrt (INV-HNSW-SLOT-1).
- **Caches und MemTable (S4, S1):** Alle im `Purgeable`-Register eingetragenen Caches (Block-, Embedding-, RRF-, Antwort-Cache, Surrogat-Vault) werden für den Scope invalidiert und zeroized. In der aktiven MemTable werden Klartext-Einträge des Scopes entzogen bzw. geflusht.
- **Text-Segment-Neuschreibung (S3):** Das betroffene Text-Segment wird unter Ausschluss des Scopes neu geschrieben. Terme, die nur im gelöschten Scope vorkamen, werden aus dem Wörterbuch entfernt, BM25-Statistiken (IDF, Dokumentlängen, Zähler) neu berechnet. Das neue Segment wird unter einem frisch erzeugten `ArtKey` verschlüsselt, der alte `ArtKey` wird sofort vernichtet (INV-ARTKEY-1).

#### Schritt 4 — Messung Ebene A, Positivkontrolle und Pre-Destruction-Inventar (Klasse P, Siegel 2)

- **Unabhängige Schicht-Scanner.** Die besitzenden Schicht-Scanner (`LsmScanner`, `VectorScanner`, `TextScanner`, `CacheScanner`, `LineageScanner`) prüfen Ebene-A-Strukturen direkt auf Speicherebene, ohne Code der Purger zu teilen (INV-WITNESS-MINT-1). Geprüft wird per Rohbyte-Scan (kanonische Terme, Roh-IDs, Embedding-Bytes, Chunk-Prüfsummen) und über die dekodierte Sicht.
- **Positivkontrolle `AC-1`.** Jeder Scan-Lauf schleust ein definiertes Kontrolldatum (Canary) ein und verifiziert, dass der Scanner es findet. Schlägt die Positivkontrolle fehl, ist der gesamte Messlauf ungültig und führt zu einem typisierten Fehler mit Wiederholung. Der Scanner-Lauf fließt in die Messung ein (Befund C-06 wird damit behoben).
- **Pre-Destruction-Inventar (nur Metadaten).** Unmittelbar vor der Schlüsselvernichtung erfasst der Scanner für die betroffene `key_id` alle noch vorhandenen Chiffratblöcke auf Platte (Datei, Offset, Länge) ausschließlich als Digest und Blockzähler, ohne Nutzdaten. Nach Schritt 5 dient das Inventar dem Nachweis, dass jeder verbliebene Block einem vernichteten Schlüssel zugeordnet ist.
- **Siegel 2** wird ausgestellt, wenn alle Ebene-A-Schichten ohne Restbefund gemessen sind und die Positivkontrolle bestanden ist.

#### Schritt 5 — Idempotente Schlüsselvernichtung (Takt 2)

1. **Keine Karenz nach `Frozen`.** Nach dem erfolgreichen Journal-Commit in Schritt 1 wird der Löschpfad ohne konfigurierbare Wartezeit fortgesetzt. `Restricted` ist ausschließlich der Pfad für einen Hold oder eine Aufbewahrungspflicht vor `Frozen`; er ist keine Rücknahme eines autorisierten Löschauftrags.
2. **Journal-Intent (Regel J1).** Der unwiderrufliche Vernichtungs-Intent wird durabel im Journal festgehalten (`fsync`), einschließlich der 96-Bit-Wrap-Nonce des zu vernichtenden `DocDek` für den Nadelscan.
3. **Atomares Replace.** Der gewrappte `DocDek` wird im Speicher (`Zeroizing`) genullt, in der `KeyTable` mit Nullen überschrieben (`fsync`), und die `KeyTable`-Segmentdatei wird atomar ohne den vernichteten Eintrag neu geschrieben (`tmp` → `fsync` → `rename` → Verzeichnis-`fsync`).
4. **Registrierung.** Die `KeyRegistry` protokolliert `destroyed` mit Referenz-Hash, ohne Schlüsselmaterial.
5. **Idempotenz.** Ist der `DocDek` nach protokolliertem Intent nicht mehr in der `KeyTable` auffindbar, gilt die Vernichtung als vollzogen. Erst nach Abschluss von Schritt 5 wird der Key-Lock freigegeben.

#### Schritt 6 — Messung Ebene B: Nadelscan und Entschlüsselungsprobe (Klasse C\*)

- **Nadelscan (Schlüsselabwesenheit).** Der Ebene-B-Scanner durchsucht alle persistenten Dateien (`KeyTable`, Keystore, Sidecars, Checkpoints, temporäre Dateien, `.bak`) im Rohbyte-Modus nach der 96-Bit-Wrap-Nonce des vernichteten `DocDek`. Erwartetes Ergebnis: **0 Treffer**.
- **Entschlüsselungsprobe.** Auf Basis des Pre-Destruction-Inventars wird eine echte `unwrap`/`open`-Probe auf mindestens einen Chiffratblock der gelöschten Einheit ausgeführt. Sie MUSS mit `KeyAbsent` fehlschlagen.
- **Kanarien-Probe.** Ein eigener Kanarien-Datensatz mit eigenem `DocDek` wird über exakt denselben Vernichtungspfad entfernt. Die Probe belegt, dass Vernichtungsmechanismus und Scanner end-to-end funktionieren. Sie belegt **nicht** die Vernichtung des konkreten `DocDek`; dafür stehen Nadelscan und Inventar.
- **Zustand `CryptoMeasured`.** Ist Ebene B sauber gemessen, trägt der Status die Klasse **`C*` (vorläufig)** mit dem offenen Residual `R-KEKEPOCH until`.

#### Schritt 7 — Gebündelter Epoch Seal und Aufwertung auf C1–C3 (Siegel 3)

Der Epoch Seal folgt §4.4. Löschungen innerhalb derselben Collection und Epoche teilen sich einen Seal (spätestens nach `seal_after_deletion_max` 7 Tagen oder sofort über `seal-now`). Erst nach abgeschlossenem Seal (INV-EPOCH-SEAL-1) entfällt `R-KEKEPOCH`.

| Custody-Profil | Erreichte Klasse | Bedingung |
|---|---|---|
| K1 / `FileOnly` | **C1** | mit Annahme `A-FLASH` / `KeyStoreRemanence` |
| K2 / `Hardware` oder `OsKeyring` | **C2** | nach bestandener Konformitätssuite |
| K3 / externer KMS | **C3** | mit Anbieter-Vernichtungsattest |

Der Koordinator signiert und verkettet **Siegel 3**.

#### Schritt 8 — Terminierte Hintergrund-Hygiene (Klasse M, Siegel 4)

Die Hygiene läuft asynchron, ist aber durch `purge_rebuild_max_days` (Startwert 14 Tage) hart terminiert. Sie erfüllt die strengste Rechtsauffassung (physische Entfernung aller erreichbaren Kopien).

1. **Purge-Compaction (SSTables und Blobs).** `LsmStorage::erase_keys` schreibt betroffene SSTables und Blob-Segmente streamend unter `create_like` neu (gleicher `KeyManager`, Formatversion, originale `seq` und `tx_id`). Dabei werden alle MVCC-Versionen, Tombstones und Klartext-Indexschlüssel der Zielschlüssel verworfen. Der Austausch erfolgt atomar über `tmp` → `fsync` → Manifest-Commit → `fsync` → Verzeichnis-`fsync` → Reader-Swap. Alte Dateien werden über das `PendingDelete`-Journal erst nach Ablauf offener Zeitreisen entfernt. Der Blockcache ersetzter Dateien wird zeroized.
2. **WAL-Entfernung mit Pflichtprüfung.** Aktive WALs werden rotiert, Memtables geflusht, versiegelte WAL-Dateien gelöscht. Per `Vfs` wird geprüft, dass die Dateien nicht mehr existieren. Verbleibt eine Datei, bleibt der Zustand `Pending` mit Retry.
3. **Graph-Neuaufbau und Generationswechsel (S2).** Bei Tombstone-Anteil ≥ 0,2 oder Fristablauf baut ein Hintergrund-Worker eine neue HNSW-Generation `g+1` ausschließlich aus lebenden Slots auf (deterministisch, INV-INDEX-APPLY-1). Nach dem atomaren Umschalt-Commit auf `g+1` wird der `ArtKey` der Generation `g` vernichtet. Damit schließt sich `R-GRAPH`. Die alten Slots werden wiederverwendbar (§5.10).
4. **Checkpoint-Sweep (S6).** Lokale Checkpoints und Hardlink-Klone werden gemäß Policy bereinigt (Standard `Rewrite`), einzeln gescannt. Hardlinks werden durch das Neuschreiben der Quelldatei nicht mitbereinigt.
5. **Abschlussmessung (`HygieneMeasured`).** Nach erneuter Schichtmessung und Positivkontrolle erreicht der Vorgang Klasse **M** (messvollständig). **Siegel 4** wird ausgestellt.

#### Schritt 9 — Kanonischer Bericht, Bescheinigung und Außentext

1. **Kanonischer `DeletionProof` V5.** Aus den `LayerWitness`-Datensätzen wird deterministisch in kanonischer Kodierung der Gesamtbericht erstellt. Pflichtfelder: erreichte Klasse (`L`, `P`, `C*`, `C1`–`C3`, `M`), verkettete Siegel 1–4, `legal_state`, `backup_window`, `assumptions[]` (`A-FLASH`, `A-ALGO`, `A-KEYLEAK`, `A-RAM`, `A-EXTBACKUP`, `A-HOSTHIST`), `residuals[]`, Schicht-Digests inklusive Positivkontrollen, `Custody`, `DestroyAttestation`, Epoch-Seal-Referenz, `subject_ref_hash` (ohne Klartext-IDs), Pflicht-Ausschlüsse sowie `Verification`- und `Validation`-Status.
2. **Signatur und Zeitanker.** Alle Felder werden als kanonisches Byte-Array mit **Ed25519** signiert (INV-PROOF-KEY-1). Kanonische Kodierung ist Pflicht. Mehrdeutige Formate wie `bincode` sind über alle signierten Felder untersagt. Optional wird der Digest über einen qualifizierten Zeitstempel (RFC 3161, eIDAS Art. 41, 42) verankert. Bei `Completed` wird `K_saga` vernichtet, sodass das Journal keinen Personenbezug mehr enthält.
3. **Datensparsame Löschbescheinigung.** Aus dem Bericht wird eine einseitige Bescheinigung abgeleitet, die nur Datum, Scope-Art, erreichte Klassen, Annahmen, Ausschlüsse, Digest, Signatur und Zeitanker enthält, **niemals** IDs, Dateinamen oder Inhalte (REQ-DEL-N21).
4. **Faktengesteuerter Außentext (INV-CLAIMS-1).** Außentexte werden ausschließlich über eine geschlossene Satzbibliothek erzeugt. Jeder Satz ist an einen gemessenen Wert des Berichts gebunden. Absolutbegriffe wie „100 % gelöscht", „unwiederbringlich" oder „DSGVO-konform" sind durch das Build-Gate `deletion-honesty` gesperrt.

### 5.4 Erfüllungsklassen

| Klasse | Name | Erreicht, wenn | Siegel |
|--------|------|----------------|--------|
| **L** | Logisch entfernt | Lesesperre aktiv (`Frozen`), Lesepfad liefert `Erased` | 1 |
| **P** | Ebene A gemessen | Alle Ebene-A-Schichten ohne Restbefund, Positivkontrolle bestanden | 2 |
| **C\*** | Kryptografisch vorläufig | Schlüssel vernichtet, Nadelscan 0 Treffer, Kanarien-Probe bestanden; Seal ausstehend | — |
| **C1 / C2 / C3** | Kryptografisch bestätigt | Epoch Seal abgeschlossen, Custody-Stufe ausgewiesen | 3 |
| **M** | Messvollständig | Hygiene abgeschlossen, Graph-Generation gewechselt, alle Schichten mit Positivkontrolle gemessen | 4 |

**Wortregel:** „gelöscht" steht nur mit Klasse C1 oder höher bzw. M. Vor dem Seal gilt `C*` (vorläufig). Ohne `EncryptionConfig::Required` ist Klasse C nicht erreichbar.

**Klassenbedingungen (REQ-DEL-N3):** Klasse C darf nur ausgewiesen werden, wenn Nadelscan, Inventar, Kanarien-Probe und Epoch Seal bestanden sind. Vor dem Seal ist nur `C*` zulässig.

**Zielwerte der Fristen (intern, keine Außenzusage):** Klasse L sofort, Klasse P ≤ 1 Stunde, `C*` unmittelbar nach P, Epoch Seal ≤ 7 Tage, Klasse M ≤ `purge_rebuild_max_days` (14 Tage).

### 5.5 Recovery nach Crash

```
Neustart
  ├─► recover() läuft VOR Freigabe des Lesepfads
  ├─► Journal wiedergeben; Frozen-, Restricted- und alle offenen Scopes sofort
  │       in ErasedSet und LiveMask laden
  ├─► Für jede offene Saga: letzten Journal-Zustand lesen
  │       Requested / Authorized   → Takt 1 neu bewerten
  │       Frozen                   → ab Schritt 3 (Ebene A) fortsetzen
  │       PlaintextPurged          → Schritt 3 fortsetzen
  │       PlaintextMeasured        → Schritt 5 (Intent-Prüfung, Idempotenz)
  │       KeyDestroyed             → Schritt 6 (Messung fortsetzen)
  │       EpochSealed-Re-Wrap      → Re-Wrap fortsetzen
  │       Hygiene                  → Schritt 8 neu starten, Idempotenz
  │       Pending(reason)          → beim nächsten Takt 2 aufgreifen
  └─► Lesepfad erst nach vollständigem recover() öffnen
```

**Regeln:**
- Ein Absturz vor Schritt 5 setzt bei `PlaintextPurged` fort. Der Scope bleibt gesperrt.
- Ein Absturz nach protokolliertem Vernichtungs-Intent prüft die Idempotenz der Schlüsselvernichtung und setzt die Messung fort. Ein Intent ohne Ausführungsnachweis wird beim Start wiederholt.
- Ein Absturz mitten im Epoch Seal lässt `KEK[e]` und `KEK[e+1]` koexistieren. Der Re-Wrap wird fortgesetzt.
- Ein Bericht trägt nie Klasse C vor abgeschlossener Messung.
- Ein Neustart DARF NICHT dazu führen, dass gelöschte Daten temporär lesbar werden (T-DEL-4, T-DEL-12).

Alle Schritte sind idempotent. Der Journal-Zustand ist der Wahrheitsspeicher.

### 5.6 Mehrfachbezug und Mixed-Segmente

| Segmentart bei Löschung für Betroffenen A | Behandlung |
|---|---|
| Menge = {A} | `DocDek` vernichten |
| Menge enthält A und andere (Mixed) | `BlockAll`: Segment bleibt gesperrt, `Partial{ReviewRequired}` mit Frist. |
| Menge enthält A nicht | unberührt |

**Mixed-Policy:** Version 2.1 unterstützt ausschließlich `BlockAll`. Ein Mixed-Segment bleibt gesperrt und der Vorgang `Partial{ReviewRequired}`, bis eine neue, separat spezifizierte Betreiberaktion einen Ersatzdatensatz als neuen regulären Schreibvorgang anlegt. Diese Aktion ist kein Teil einer Löschsaga und darf den ursprünglichen `DocDek` nicht wiederverwenden. `RedactAndRekey` und `DeleteAll` sind keine konfigurierbaren Modi dieser Version.

Mixed-Segmente dürfen nicht automatisch freigegeben oder gelöscht werden (REQ-DEL-N17). Contextra entscheidet nicht, ob ein Mixed-Segment nach Entfernung von A noch personenbezogen ist.

**Ableitungen** kaskadieren über die Lineage (INV-DERIVED-1). **Collection-Löschung (Akte):** Alle `DocDek`s, `ArtKey`s und `KEK[coll, *]` werden in einer Saga vernichtet. Die Index-Dateien der Collection entfallen.

### 5.7 Restricted und Legal Hold (Art. 18 DSGVO)

Bei Aufbewahrungspflicht (Art. 17 Abs. 3) oder Legal Hold geht die Saga in `Restricted{basis, until}`. Es erfolgt keine Löschoperation und keine Schlüsselvernichtung. Die Lesesperre bleibt. Lesezugriff nur mit Audit-Eintrag (T-DEL-13). Übergänge: `Restricted → Frozen` (Eskalation zur Löschung) oder `Restricted → Lifted` (Aufhebung, terminal).

Ein Hold betrifft die Saga, nicht die Lesesperre. Er wird im `RevocationLog` und in der Audit geführt. Checkpoint-Policy `ExpireAfterHold` gilt nur für Checkpoints.

**Trennung ist normativ:** Die reversible Einschränkung (Art. 18) und die irreversible Löschung (Art. 17) bleiben strikt getrennt. `Restricted` wird vor `Frozen` entschieden und ist keine Rücknahme einer Löschung.

### 5.8 Messung und Nachweis

#### 5.8.1 Grundsätze

1. **Messen vor Behaupten.** Ein Beleg entsteht nur aus einem Scanner der besitzenden Schicht (INV-WITNESS-MINT-1).
2. **Positivkontrolle.** Jeder Scan-Lauf hat eine bestandene Positivkontrolle. Ohne sie ist der Lauf ungültig.
3. **Messung am Speicher, nicht am Lesepfad.** Der Lesepfad-Test ist eine zusätzliche Prüfung, kein Ersatz.
4. **Zwei Quellen je Pflichtschicht.** Jede Pflichtschicht wird durch ihren Scanner und durch eine unabhängige Black-Box-Probe des Orchestrators gemessen (D-13). Eine reine Selbstauskunft des Layers über `witness()` ist unzulässig.
5. **Verification und Validation sind getrennt.** Verification ist der technische Lauf. Validation ist die Abnahme durch den Betreiber. Im Regulated-Tier ist die Concurrence-Signatur Pflicht (REQ-DEL-N20).

#### 5.8.2 Ebene A: Klartext-Scanner

1. Rohbyte-Scan aller Dateien des Datenverzeichnisses nach kanonischen Termen, Roh-IDs, Embedding-Bytes und Chunk-Prüfsummen der Einheit.
2. Dekodierte Sicht über die normalen Lesepfade mit allen aktiven Schlüsseln.
3. Positivkontrolle `AC-1`: Ein eingeschleuster Kontrolldatensatz wird in beiden Sichten gefunden.

#### 5.8.3 Ebene B: drei Nachweise

1. **Pre-Destruction-Inventar.** Vor der Vernichtung wird je `key_id` die Liste aller Chiffratblöcke (Datei, Offset, Länge) als Digest und Zähler festgehalten. Danach muss jeder Block einem vernichteten oder lebenden Schlüssel zugeordnet sein. Blöcke ohne Zuordnung sind ein Fehler (fail-closed).
2. **Nadelscan (Schlüsselabwesenheit).** Gesucht wird nach der Wrap-Nonce des vernichteten Schlüssels in allen Dateien. Ergebnis: null Treffer. Eine Hash-Suche über Blob-Offsets ist nicht zulässig.
3. **Kanarien-Probe.** Siehe §5.3 Schritt 6. Sie belegt Scanner und Vernichtungsweg, nicht die Vernichtung des konkreten `DocDek`.

#### 5.8.4 Grenzen (immer im Bericht)

Der Scanner sieht keine Flash-Remanenz unterhalb des Dateisystems, keinen Schlüssel in RAM, Swap oder Dumps (`A-RAM`), keine externen Backups (`A-EXTBACKUP`) und keine Copy-on-Write-Kopien unterhalb der Dateisystemebene. Der Bericht weist diese Grenzen als Annahmen aus (REQ-DEL-N8). Der Canary-Scan beweist die Korrektheit der Bereinigung, nicht die physische Zellfreiheit. Die kryptografische Garantie liegt in der Schlüsselvernichtung (Schritt 5).

#### 5.8.5 Restmetadaten-Selbsttest

Ein Selbsttest prüft, dass im Klartext keine Nutzerschlüssel, Dokument-IDs oder Schlüssel-Hilfsdaten vorkommen (F-07). Er läuft im Gate `surface-coverage`.

### 5.9 Schichten

| Schicht | Crate | Ebene | Maßnahme | Messung |
|---|---|---|---|---|
| **S1** LSM und WAL | `store` | A (MemTable), B | Records unter `DocDek`. MemTable zeroizen. Hygiene: `compact_range(scope)`, WAL-Scrub | `LsmScanner` (Klartext, Chiffrat, Nadel) |
| **S1b** Blob-Segmente | `store` | B | Records unter `DocDek`. Hygiene: Segment-Rewrite | `LsmScanner` |
| **S2** Vektorindex | `vector` | A, B | Arena-Slots (fp32, SQ8) nullen. Snapshot-Payload unter `DocDek`. Graph unter `ArtKey`, Neuaufbau nach Tombstone-Anteil ≥ 0,2 oder `purge_rebuild_max_days`. Keine Slot-Wiederverwendung vor Neuaufbau (INV-HNSW-SLOT-1). Kein Rewiring (VETO-F02) | `VectorScanner` |
| **S3** Textindex | `text` | A, B | Segmente neu schreiben, neuer `ArtKey`, alter sofort vernichtet. Terme des Scopes aus dem Wörterbuch entfernen. Statistik neu berechnen | `TextScanner` |
| **S4** Caches | `store`, `rank`, `privacy` | A | Block-Cache (`store`), dokumentbezogener RRF-Cache `er_cache` (`rank`), Surrogat-Vault (`privacy`). Embedding- und Antwort-Cache registriert `db` als `Purgeable`, sobald sie existieren. Invalidieren und zeroizen (REQ-DEL-N7). `engine` hält keinen Cache | `CacheScanner` |
| **S5** Lineage und Ableitungen | `db` | A | `LineageIndex` und Kaskade (INV-DERIVED-1). `engine` re-exportiert `db` und implementiert keine Schicht | `LineageScanner` |
| **S6** Checkpoints | `checkpoint` | B | Policy `Rewrite` (Standard), `ExpireNow`, `ExpireAfterHold` nur unter Hold. Sauber erst nach Rewrite | `CheckpointScanner` |
| **S7** Sidecars | `crypto` | – | Nur Referenz-Hashes, keine Nutzinhalte, keine Schlüssel | `SidecarScanner` |
| **S8** Externe Sicherungen | – | – | Ausweisen als `ExternalBackups`. Daten-Backups aus Contextra fallen unter §5.11 | keine Messung |

**Oberflächen-Inventar (Gate `surface-coverage`, REQ-DEL-N24):** Jede persistente Datei- und Strukturklasse des Format-Registers steht in genau einer Zeile dieser Tabelle oder in einem Ausschluss. Ein neues Format ohne Schichtzuordnung bricht den Build.

### 5.10 Index-Generationen und Restinformation

**Residual `R-GRAPH`.** Im HNSW-Graph bleiben nach der Löschung Routing-Knoten ohne Nutzdaten. Die Kanten enthalten nur interne Slot-Nummern. Die Topologie kann Nachbarschaftsbeziehungen zu gelöschten Vektoren offenlegen. Das bleibt als `R-GRAPH` mit Frist im Bericht, bis der Neuaufbau erfolgt ist (REQ-DEL-N25).

**Neuaufbau:**
1. Hintergrund-Neuaufbau erzeugt Generation `g+1` ohne gelöschte Dokumente.
2. Atomarer Umschalt-Commit auf `g+1`.
3. Vernichtung des `ArtKey` der Generation `g`.

Mehrere Löschungen teilen sich einen Neuaufbau (Batch). Auslöser: Tombstone-Anteil ≥ 0,2, Fristablauf, `seal-now`.

**Restkanäle.** Treffermengen und Timing-Unterschiede durch Ghost-Knoten sind nur klassenweise zugesichert. Der Neuaufbau beseitigt sie. Die Statistik (IDF, Dokumentlängen, Zähler) liegt im Text-Segment und wird mit der Segment-Neuschreibung neu berechnet.

### 5.11 Backup, Restore und Restore-Guard

- **Zwei-Spuren-Backup (Spur D und Spur K).** Daten-Backups (Spur D) enthalten ausschließlich Chiffrate ohne `KeyTable` und ohne Keystore (REQ-DEL-N18). Schlüssel-Backups (Spur K) tragen ein signiertes Ablaufdatum `valid_until` (`key_backup_max_age`, Startwert 30 Tage) und werden nach Ablauf verweigert (`RestoreBeyondHorizon`).
- **Restore-Guard.** Bei jedem Restore gleicht der Guard vor dem ersten Lesezugriff das lokale `ErasedSet` sowie den externen `ErasedSetMirror` ab, vernichtet wiederhergestellte `DocDek`s gelöschter Einheiten sofort erneut und protokolliert `restore_reapplied` im Audit (REQ-DEL-N11).
- **`ErasedSetMirror`.** Im Regulated-Tier Pflicht. Ein fehlender oder veralteter Mirror verweigert Restore. Im Standard-Tier ist ein unguarded Restore nur mit der expliziten lokalen Option `--accept-unguarded` zulässig und wird im Bericht ausgewiesen (REQ-DEL-N19).
- **Aufbewahrung des `ErasedSet`.** Der `ErasedSet`-Eintrag bleibt für die Dauer der Aufbewahrung des Berichts erhalten. Er enthält nur Referenz-Hashes.
- **Backup-Test.** Backup vor Löschung, Löschung, Restore mit altem und mit frischem Schlüssel-Backup. Der Scope bleibt gesperrt, keine lesbaren Reste (T-DEL-18).

### 5.12 Fristen, Empfänger und Legal Hold

| Frist | Quelle | Wert |
|---|---|---|
| Beantwortung Betroffenenanfrage | Art. 12 Abs. 3 DSGVO | ein Monat, Verlängerung um zwei Monate mit Mitteilung binnen eines Monats |
| `Frozen` (Klasse L) | intern | sofort nach Autorisierung |
| Klasse P | intern, Zielgröße | ≤ 1 Stunde |
| Klasse C\* | intern | unmittelbar nach P |
| Epoch Seal | intern | ≤ 7 Tage nach Vernichtung |
| Hygiene (Klasse M) | intern | ≤ `purge_rebuild_max_days` (14 Tage) |
| Alarm bei `Partial` | intern | 24 h |
| Eskalation bei `Partial` | intern | 14 Tage |
| Meldung Fristgefährdung | intern | 25 Tage |

Die Werte sind Zielgrößen. Contextra sendet nichts nach außen. Alarme erscheinen in `doctor`, Metrik, CLI, MCP-Antwort und `AuditChain`.

**`RecipientLedger` (Art. 17 Abs. 2, Art. 19 DSGVO, REQ-DEL-N22).** Je Dokument führt Contextra ein Register der Ausleitungen (Export, Egress-Kanal, Checkpoint-Export) ohne Inhalte: Empfängerkennung, Zeitpunkt, Datenklasse. Bei Löschung liefert `doctor` die Empfängerliste. Die Mitteilung an Empfänger erfolgt durch den Verantwortlichen. Ausleitungen außerhalb von Contextra sind ausgeschlossen.

### 5.13 Härtung

`doctor` MUSS Härtungslücken ausweisen (REQ-DEL-N23): Core-Dumps, Swap, fehlendes `mlock`, Hibernation. Diese Befunde gehen als Annahmen in den Bericht ein.

---

## 6 Lesesperre (ReadGate)

### 6.1 Anforderungen

- **O(1) ohne I/O, ohne Blockierung.** Synchroner In-Memory-Lookup gegen `LiveMask` und `ErasedSet`.
- **Fail-Closed.** Bei unklarem Zustand: ablehnen, nie durchlassen.
- **Gleiches Fehlerbild.** Ein gelöschtes Dokument ist nicht von einem nie existierenden unterscheidbar.
- **Vor Entschlüsselung.** Gate-Check vor jedem Zugriff auf Chiffretext.
- **Zentrale Prüfstelle.** `LsmStorage` ist nach außen `pub(crate)`. Jeder Lesezugriff der Engine läuft über `TenantScopedStorage`. Dort ist die einzige Prüfstelle (Befund: „Erase gewinnt gegen Zeitreisen").

### 6.2 Lesewahrheit: `Slot` und `LiveMask`

- Jede 128-Bit-`DocId` wird pro Collection auf einen dichten `u32`-`Slot` abgebildet. Die `DocId` ist `BLAKE3-keyed(K_scope, tenant ‖ collection ‖ user_key)` mit Kollisionsprüfung beim Insert.
- Eine lock-freie Bitmap `LiveMask` (atomare Wörter oder Copy-on-Write-`Arc`-Swap) ist die unmittelbare Lesewahrheit für HNSW-Kandidatenfilterung, BM25-Posting-Iteratoren und Chunk-Abrufe.
- Ein `Slot` wird erst nach vollständigem Index-Rebuild wiederverwendet (INV-HNSW-SLOT-1).
- Offene Snapshots blockieren den Löschbeginn **nicht** (Takt 1 unabhängig von der Snapshot-Zahl, siehe Snapshot-Freiheit).

### 6.3 Prüfpunkte (vollständig, INV-GATE-COMPLETENESS)

**Jeder einzelne Lesepfad MUSS vor der Rückgabe synchron O(1) gegen das ReadGate prüfen:**

| Lesepfad | Gate-Check-Stelle | Verhalten bei `Erased` |
|----------|-------------------|-----------------------|
| `get`, `get_at_seq` | Vor Entschlüsselung | `Err(ContextraError::Erased)` |
| `scan`, `scan_prefix`, `scan_prefix_at` | Je Eintrag vor Ausgabe | Eintrag still auslassen |
| Vektor-/Textsuche | Auf Kandidatenebene vor Fusion | Kandidat verwerfen, auf `k` auffüllen |
| Hybrid-Suche | Vor Fusion und Ranking | Kandidat ignorieren |
| Graph-Traversal | Je Kante und je Zielknoten | Pfad bricht ab |
| Export | Je Eintrag | Gesperrte auslassen |
| Checkpoints | Vor Wiederherstellung | Verweigern |
| MCP-Tools | Über darunterliegende Pfade | Wie jeweiliger Pfad |
| Kompaktionen | `check_range` → `ContainsErased` | Range nicht in Ergebnis aufnehmen |

**Zeitreisen:** Das ReadGate wird immer mit dem **aktuellen** Stand geprüft, nicht mit historischen Rechten. Ein Zeitreise-Query auf `as_of = T` liefert `Erased`, wenn das Ziel heute gelöscht ist. Das gilt auch für offene Snapshots.

### 6.4 Ausdrucksform (`ReadGate`-Trait, kanonisch in `contextra-erasure`)

```rust
pub trait ReadGate: Send + Sync {
    /// Synchroner O(1)-Check. Muss vor jeder Entschlüsselung aufgerufen werden.
    fn check(&self, key: &[u8]) -> GateDecision;

    /// Bereichsprüfung für LSM-Kompaktionen.
    fn check_range(&self, start: &[u8], end: &[u8]) -> RangeGate;
}

pub enum GateDecision { Open, Erased, Restricted }
pub enum RangeGate   { AllOpen, ContainsErased, RestrictedNeverDiscard }
```

**Restricted hat Vorrang vor Erased.** Ein Eintrag im Zustand `Restricted` liefert nur mit Audit-Eintrag (T-DEL-13).

---

## 7 ErasureLayer — Schnittstelle für Schichten

**Kanonisch in `contextra-erasure` (E0) definiert.** Implementiert von E1-Crates. Die Schnittstelle ist an die Schichten S1–S8 (§5.9) gebunden.

```rust
pub trait ErasureLayer: Send + Sync {
    fn id(&self) -> LayerId;
    fn key_coverage(&self) -> KeyCoverage;                     // Full | None

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
1. `purge` ist idempotent: der zweite Aufruf ist ein No-op (T-DEL-14).
2. `witness` ist rein lesend.
3. Logs enthalten nie Klartextinhalte des Ziels (INV-12).
4. E1-Crates importieren `ErasureLayer` aus `contextra-erasure`, **nicht** aus `contextra-ports`.
5. `Scanner<Armed>` kann nur nach bestandener Positivkontrolle entstehen (INV-WITNESS-MINT-1). Der Konstruktor ist in `contextra-erasure` `pub(crate)`, eine öffentliche Konstruktion über `contextra-types` ist untersagt (Befund C-06).

**Layer-Zuordnung:**

| Layer | Crate | Saga-Schritt |
|-------|-------|--------------|
| StoreLayer | `contextra-store` | Schritt 3 (MemTable, WAL), Schritt 8 (SSTable, WAL-Scrub) |
| TextLayer | `contextra-text` | Schritt 3 (Segment-Neuschreibung) |
| VectorLayer | `contextra-vector` | Schritt 3 (Arena), Schritt 8 (Graph-Neuaufbau) |
| CacheLayer | `contextra-store`, `contextra-rank`, `contextra-privacy`, registriert in `contextra-db` | Schritt 3 (S4) |
| LineageLayer | `contextra-db` | Schritt 2 (Kaskade) |
| CheckpointLayer | `contextra-checkpoint` | Schritt 8 (Sweep) |
| SnapshotRevoker | `contextra-mvcc` | Schritt 1 (Zeitreisen ausschließen) |
| CryptoLayer | `contextra-crypto` | Schritt 5 (`KeyTable`, Sidecars) |

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

1. Jeder Schreibvorgang in D-Klasse MUSS eine `LineageRef` angeben (INV-LINEAGE).
2. `LineageIndex::derived_of(source, version)` liefert alle `DerivedRef`s.
3. Schritt 2 der Saga nutzt den Lineage-Index für die Kaskade. Schritt 8 bereinigt die Ableitungen physisch.
4. Ableitungen mit weiteren, nicht gelöschten Quellen werden neu abgeleitet oder als `stale` markiert.
5. Ohne Lineage ist Klasse C1 für abgeleitete Daten nicht erreichbar.

---

## 9 Zeitreise

### 9.1 Zwei unabhängige Achsen

| Achse | Typ | Semantik |
|-------|-----|----------|
| **Transaktionszeit** (`as_of`) | `u64` (Sequenznummer) | „Was war zum Zeitpunkt X im System bekannt?" |
| **Gültigkeitszeit** (`valid_at`) | `i32` (Kalendertage seit 1970-01-01) | „Was galt zum fachlichen Zeitpunkt Y?" |

### 9.2 Regeln

1. `as_of` und `valid_at` sind unabhängig kombinierbar.
2. `None` bedeutet „offen" (kein Ende bzw. kein Anfang), nicht `0`.
3. Das ReadGate wird immer mit dem **aktuellen** Stand geprüft, unabhängig von `as_of`. „Erase gewinnt gegen Zeitreisen."
4. Spätere Schreibvorgänge ändern frühere `as_of`-Antworten nicht (Z-T2).
5. Alte Dateien von Compaction und Purge werden erst nach Ablauf offener Zeitreisen entfernt (`PendingDelete`-Journal, §5.3 Schritt 8).

---

## 10 Concurrency-Barrieren und Speicherordnung

### 10.1 CommitBarrier (kanonisch in `contextra-ports`)

```
Writer 1: shared()      → [schreibt]
Writer 2: shared()      → [schreibt]
Saga:     exclusive()   → [Lesesperre setzen, Journal-Commit] → drop permit
```

`shared()` erlaubt mehrere parallele Writer. `exclusive()` wartet auf alle `shared`-Permits und blockiert neue. Schützt den Gate-Commit.

### 10.2 PurgeBarrier (kanonisch in `contextra-ports`)

```
Kompaktions-Job: compaction()  → [kompaktiert]
Saga-Purge:      purge()       → [exklusive Purge-/Compaction-Barriere]
```

`purge_waiting()` signalisiert wartenden Kompaktionen, dass ein Purge ansteht. Kompaktionen sollen ihren Slot zügig freigeben. Der exklusive Zugriff gilt für die Purge-Compaction in Schritt 8.

### 10.3 Dekker-Fence zwischen Journal und `LiveMask` (normativ)

Zwischen Journal-Commit und Lesepfad gilt ein `SeqCst`-Protokoll:

1. Der Schreiber schreibt das Journal, `fdatasync` und setzt danach das Bit in `LiveMask` mit `SeqCst`.
2. Der Leser liest `LiveMask` mit `SeqCst`, bevor er Daten zurückgibt.
3. Ein `#[must_use]`-Ticket hält die Publikation bis nach WAL-Append und Memtable-Apply.
4. Kein Leser sieht eine gesetzte Maske ohne durablen Journal-Zustand. Kein Leser sieht nach Rückgabe von Takt 1 einen veralteten Maskenzustand.

### 10.4 Single-Flight und ein Koordinator

- Genau **ein** zentraler Saga-Koordinator führt die Saga aus (INV-ERA-1, WP-10 aus der Umsetzungsprüfung).
- Ein Scope ist höchstens einmal aktiv (Single-Flight). Duplikate liefern das bestehende Ticket.
- Der Koordinator nutzt typisierte Fehler und Retry mit Backoff.

---

## 11 Fehlerverhalten

### 11.1 Fehler-Hierarchie

| Kategorie | Fehler | Verhalten |
|-----------|--------|-----------|
| **Transient** | I/O-Fehler im Index oder Journal | Wiederholung mit Backoff, Zustand bleibt, `Pending(IoBackoff)` mit Resume-Ziel |
| **Logisch** | Rückstand im Scan, fehlende Positivkontrolle, fehlende Gate-Unterstützung | Zustand unverändert, Fehler wird zurückgegeben |
| **Nach Frozen** | Fehler nach Schritt 1 | Datum bleibt `Frozen`, Bereinigung wird nachgeholt |
| **Nach KeyDestroyed** | Fehler nach Schritt 5 | Datum bleibt `KeyDestroyed`, Messung wird nachgeholt |
| **Schlüsselvernichtung** | `ShredVerificationFailed` | Zustand bleibt `Frozen`, Aufrufer erhält `Err` |
| **Capability** | K2/K3 ohne Konformitätssuite | `CapabilityUnsupported`, nie Erfolg melden |
| **Im Lesepfad** | Fehlender Schlüssel | `Erased`, kein Unterschied zu „nie existiert" |
| **Crypto** | AEAD-Fehler | Einheitlicher Fehler, nicht weiter unterscheidbar |

Der Zustand wird erst nach erfolgreichem Journal-Append (`fsync`) geändert.

### 11.2 Fail-Closed-Prinzip

- Fehlender Schlüssel führt zu einem Fehler, nie zu einem Klartext-Fallback.
- Fehlender ReadGate führt zu einem Öffnungsfehler (bei `GuardPolicy::Required`).
- Unbekannter Gate-Zustand führt zur Ablehnung.
- Jeder Fehler im Löschpfad MUSS im Journal erscheinen.
- Ein Journal, das nicht durabel ist, führt dazu, dass Takt 1 `Err` liefert und die Lesesperre nicht gesetzt wird.

---

## 12 Testkatalog (Löschkette)

| Kennung | Prüft | Kategorie |
|---------|-------|-----------|
| **T-DEL-1** | Nach Insert und Flush sind alle persistierten Bytes verschlüsselt (kein Klartext-Canary in Dateien) | Integration |
| **T-DEL-2** | Nach Löschung, Purge und Compaction liefert der Lesepfad `Erased`; Canary-Scan `residue_matches == 0` | Integration |
| **T-DEL-3** | Löschnachweis-Verifikation schlägt mit falschem Schlüssel fehl | Unit |
| **T-DEL-4** | Nach Löschung und Neustart (ReadGate-Reload) bleibt das Datum `Erased` | Integration |
| **T-DEL-5** | Ohne `EncryptionConfig::Required` erreicht Löschung maximal Klasse L | Integration |
| **T-DEL-6** | Nach Löschung findet weder Volltext- noch Vektorsuche noch Graph-Traversal das Dokument | Integration |
| **T-DEL-7** | Positivkontrolle: `Scanner<Unarmed>` findet eingebettetes Canary-Muster und wird zu `Scanner<Armed>` | Unit |
| **T-DEL-8** | Collection-Drop widerruft `KEK[coll, *]`; alle Dokumente der Collection sind `Erased` | Integration |
| **T-DEL-9** | Subject-Löschung: alle `DocDek`s des `SubjectRef` vernichtet, alle P-Daten `Erased` | Integration |
| **T-DEL-10** | Lineage-Kaskade: Quelldatum gelöscht, alle D-Ableitungen bereinigt, `residue_matches == 0` | Integration |
| **T-DEL-11** | ReadGate blockiert alle Lesepfade aus §6.3 (vollständige Prüfliste) | Integration |
| **T-DEL-12** | Crash in Schritt 3 mit Neustart: Saga wird korrekt fortgesetzt, Gate ist sofort aktiv | Integration |
| **T-DEL-13** | Restrict-Modus: Lesezugriff nur mit Audit-Eintrag; ohne Eintrag `Err(RestrictedReadAuditRequired)` | Integration |
| **T-DEL-14** | `purge` ist idempotent: der zweite Aufruf ist ein No-op | Unit |
| **T-DEL-15** | `verify_key_absent` schlägt fehl, wenn der Slot nicht vollständig `0x00` ist | Unit |
| **T-DEL-16** | Crash-Matrix: Abbruch an jedem Zustandsübergang der Saga und des Epoch Seal mit Neustart. Geprüft: Lesesperre, Schlüsselstatus, Berichtszustand | Integration |
| **T-DEL-17** | Rohdatei-Forensik: Nach Löschung durchsuchen Rohsicht und entschlüsselte Sicht alle Dateien nach Kanarien-Klartext und Wrap-Nonce | Integration |
| **T-DEL-18** | Backup/Restore: Scope bleibt gesperrt, keine lesbaren Reste; abgelaufenes Schlüssel-Backup wird verweigert; Restore ohne Mirror wird ausgewiesen | Integration |
| **T-DEL-19** | Gleichzeitigkeit (loom): `remember` und `forget` auf derselben Einheit, Epoch Seal | Loom |
| **T-DEL-20** | Mixed-Segment bleibt `BlockAll`; eine automatische Freigabe oder Kollaterallöschung ist ausgeschlossen | Integration |
| **T-DEL-21** | Propertytests: Nach Seal und Hygiene enthält keine Datei die Nonce eines vernichteten Schlüssels; kein Schlüssel hat mehr als einen Zweck | Property |
| **T-DEL-22** | Berichts-Negativsuite: Bit-Flip je Feld, Feldvertauschung, Downgrade, falscher Schlüssel, fehlende Ausschlüsse, Klasse ohne Annahmen, gefälschtes Siegel | Unit |
| **T-DEL-23** | Mutation (AC-2): eine Konstante statt Messung lässt einen Test fehlschlagen | Mutation |
| **T-DEL-24** | Gate `claim-generator`: jeder Satz der Satzbibliothek ist an einen Messwert gebunden; ein überzogener Text lässt das Gate fehlschlagen | Gate |
| **T-DEL-25** | Orakel (INV-ORACLE-1): ein unabhängiger Referenz-Prüfer verifiziert Berichte | Integration |
| **T-DEL-26** | Last: Leseleistung mit `DocDek`-Lookup, Padding-Overhead ≤ 12 %, Re-Wrap-Dauer; Ziele in `benchmarks/budgets.toml` | Bench |

**Test-Implementierungsregel:** Jeder Test MUSS die vollständige Saga inklusive Purge durchlaufen. Ein Test, der nur das Gate setzt und keinen Purge aufruft, prüft maximal Klasse L.

**Verdrahtungs-Gate (INV-WIRING-2):** Die Löschkette MUSS produktiv verdrahtet sein. Ein Stub, der Erfolg meldet, ist ein Build-Fehler. Die Gates `surface-coverage`, `claim-generator` und `deletion-honesty` sind Pflicht im CI.

---

## 13 Invarianten

| ID | Invariante |
|----|------------|
| **INV-1** | Kein persistiertes Byte ändert sich ohne Formatvermerk. |
| **INV-2** | Jeder öffentliche Pfad bleibt auflösbar. |
| **INV-3** | `forbid(unsafe_code)` außerhalb der Inseln (`sys`, `simd`). |
| **INV-4** | Kein `unwrap`/`expect`/`panic!`/`todo!`/`unimplemented!` in Nicht-Test-Code. |
| **INV-5** | Gleiche Eingabe führt zur gleichen Rangfolge, Tie-Break ID aufsteigend. |
| **INV-6** | WAL, Manifest, Flush und Recovery-Verhalten bleiben beim Umbau unverändert. |
| **INV-7** | Keine Compliance-Aussagen. „Gelöscht" nur mit gemessener Stufe. Keine Skalierungsbehauptung ohne belegten Bench-Lauf. |
| **INV-8** | Abhängigkeitskanten nur abwärts (R1). |
| **INV-9** | `Completed`, `Refused` und `Lifted` sind terminale Saga-Zustände. |
| **INV-10** | Ab `Frozen` wird die Lesesperre nicht aufgehoben. Ein Löschauftrag kehrt nicht in `Restricted` oder `Lifted` zurück. |
| **INV-11** | Schlüsselbytes werden mit `zeroize` genullt vor Drop. Kein `Clone` für Schlüsseltypen. |
| **INV-12** | Logs und Traces enthalten nie Klartextinhalte des Löschziels oder PII. |
| **INV-DEL-ORDER-1** | Purge (Schritt 3 und 4) liegt vor der Schlüsselvernichtung (Schritt 5). Die Vorgängerreihenfolge „Schlüssel zuerst" ist verboten. |
| **INV-GATE-COMPLETENESS** | Jeder Lesepfad aus §6.3 prüft das ReadGate. Eine Ausnahme ist ein Implementierungsfehler. |
| **INV-WITNESS-MINT-1** | `Scanner<Armed>` kann nur nach bestandener Positivkontrolle instanziiert werden. |
| **INV-LINEAGE** | Jede D-Klasse-Schreiboperation ohne `LineageRef` ist ein Implementierungsfehler. |
| **INV-DERIVED-1** | Ableitungen kaskadieren über die Lineage und werden in Schritt 2 erfasst. |
| **INV-ARTKEY-1** | Beim Neuschreiben eines Index-Artefakts wird der alte `ArtKey` sofort vernichtet. |
| **INV-HNSW-SLOT-1** | Ein `Slot` wird vor Abschluss des Neuaufbaus nicht wiederverwendet. |
| **INV-EPOCH-SEAL-1** | `C1`–`C3` gelten erst nach abgeschlossenem Epoch Seal. Vorher nur `C*`. |
| **INV-KEYPURPOSE-1** | Jeder Schlüssel hat genau einen Zweck und ein Label im Zweckregister. |
| **INV-PROOF-KEY-1** | Berichte werden mit Ed25519 signiert. Der Signaturschlüssel ist kein Datenschlüssel. |
| **INV-CLAIMS-1** | Außentexte entstehen nur aus der geschlossenen Satzbibliothek mit Messbindung. |
| **INV-WIRING-2** | Löschkette ist produktiv verdrahtet, ohne Stubs, die Erfolg melden. |
| **INV-ORACLE-1** | Berichte werden von einem unabhängigen Referenz-Prüfer verifiziert. |
| **VETO-F02** | Kein Rewiring im HNSW-Graph beim Löschen. |
| **REQ-DEL-N1–N27** | Verbindliche Löschanforderungen gemäß §5 der Finalen Löscharchitektur. |

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
    Required,  // Ohne ReadGate kein Öffnen (regulierte Umgebungen)
}

pub enum DataClassPolicy {
    Legacy,    // Schreibmethoden ohne Klasse erlaubt (Migrationspfad)
    Enforced,  // Jeder Schreibvorgang MUSS Klasse deklarieren
}

pub enum TenantPolicy {
    Standard,
    Regulated,
}

pub enum CustodyProfile {
    K0,
    K1,
    K2,
    K3,
}

pub enum MixedPolicy {
    BlockAll,
}

pub struct ErasureConfig {
    pub ledger_dir: PathBuf,              // MUSS außerhalb von data_dir liegen
    pub disk_reserve_bytes: u64,          // Mindest-Plattenreserve vor Purge, min. 1 GB
    pub custody: CustodyProfile,          // K0 | K1; K2/K3 werden abgewiesen
    pub mixed_policy: MixedPolicy,        // ausschließlich BlockAll
    pub kek_epoch_len: Duration,          // Startwert 30 Tage
    pub seal_after_deletion_max: Duration,// Startwert 7 Tage
    pub purge_rebuild_max_days: u32,      // Startwert 14 Tage
    pub key_backup_max_age: Duration,     // Startwert 30 Tage
    pub group_commit_window_ms: u32,      // 1 bis 2 ms
    pub epoch_window: Duration,           // Startwert 5 s
    pub epoch_max: Duration,              // Obergrenze 60 s
    pub epoch_set_threshold: u32,         // Startwert 1.024
    pub tombstone_rebuild_ratio: f32,     // Startwert 0,2
    pub min_record_bytes: u32,            // Startwert 256
    pub erased_set_mirror: Option<MirrorConfig>, // Pflicht im Regulated-Tier
}
```

**Konfigurationsvalidierung:** `Contextra::open` MUSS vor dem Öffnen prüfen:
- `data_dir` ist vorhanden und beschreibbar.
- `erasure.ledger_dir` ist kein Unterverzeichnis von `data_dir`.
- `erasure.disk_reserve_bytes >= 1_073_741_824` (1 GB).
- Bei `EncryptionConfig::Required`: Passphrase ist nicht leer.
- `erasure.mixed_policy == MixedPolicy::BlockAll`.
- Im Regulated-Tier gilt `EncryptionConfig::Required`, `GuardPolicy::Required` und ein gesetzter `erased_set_mirror` (REQ-DEL-N19); die Concurrence-Signatur ist für jeden Bericht erforderlich.
- `custody == K0` oder `K1`. K2 und K3 führen immer zu `CapabilityUnsupported`.

Alle Zielwerte sind in `benchmarks/budgets.toml` zu hinterlegen (O-24).

---

## 15 Layering-Gate

Das Layering-Gate ist kein externes Python-Skript, sondern ein Cargo-Integration-Test in `crates/contextra-testkit/tests/layering.rs`. Er nutzt `cargo metadata` und prüft:

1. Für jede Crate: `default = []` (R3).
2. Für jede E0/E1-Crate: keine Abhängigkeit auf E2/E3-Crates (R1), auch nicht über Feature-Unifikation.
3. Für E1-Crates: domänenspezifische Kern-Closure ⊆ E0, höchstens 5 Mitglieder. `contextra-ports` und `contextra-erasure` zählen nicht in die fünf. Jede Schicht, die einen Port oder `ErasureLayer` implementiert, darf beide zusätzlich nutzen. Beispiel: `contextra-store` hat die Domänen-Closure `ids`, `env`, `sys`, `durable-fs`. `contextra-vector` hat `ids`, `env`, `simd`, `durable-fs`. `contextra-crypto` und `contextra-mvcc` sind keine zulässigen Abhängigkeiten einer anderen E1-Crate.
4. Für alle Crates: `[lints] workspace = true` ist gesetzt.
5. Kein Crate aus der Entfernt-Liste (§2.2) ist als Workspace-Member aktiv.
6. Die Gates `surface-coverage`, `claim-generator` und `deletion-honesty` sind im CI eingebunden (INV-WIRING-2).

Der Gate-Test läuft in CI als separater Schritt vor dem Build. Schlägt er fehl, bricht CI ab. Es gibt keine `feature_closure_allowed`-Ausnahme.

---

## 16 Glossar

| Begriff | Definition |
|---------|------------|
| **Canary** | Bekanntes Byte-Muster, das über den Schreibpfad eingefügt wird, um den Scanner zu kalibrieren |
| **Crypto-Shredding** | Vernichtung des Schlüssels macht Chiffretext unlesbar, ohne physisches Löschen |
| **C\* (vorläufig)** | Kryptografische Klasse vor abgeschlossenem Epoch Seal |
| **DocDek** | Zufälliger 32-Byte-Schlüssel je Betroffenen-Einheit, gewrappt unter der KEK |
| **ArtKey** | Zufälliger Schlüssel je Index-Artefakt (Text-Segment, Graph-Snapshot) |
| **Epoch Seal** | Rotation der KEK je Epoche mit Neu-Wrap aller lebenden Schlüssel und Vernichtung der alten KEK |
| **Frozen** | Erster durabler Lesesperr-Zustand (Klasse L). Kanonischer Name, Vorgänger „Gated" |
| **Gate** | Synchrone In-Memory-Lesesperre (`ReadGate`), die Zugriff auf gelöschte Daten verhindert |
| **KEK** | Key Encryption Key, umhüllt die `DocDek`s und `ArtKey`s einer Collection in einer Epoche |
| **KeyAnchor** | Port für die Wurzelverwahrung mit Custody-Stufen K0 bis K3 |
| **KeyTable** | Speicherbereich mit Ersetzen-Semantik für gewrappte Schlüssel |
| **LiveMask** | Lock-freie Bitmap über `Slot`s, unmittelbare Lesewahrheit |
| **Lineage** | Herkunftsverweis abgeleiteter Daten (D-Klasse) auf ihr Quelldatum |
| **Nadelscan** | Rohbyte-Suche nach der Wrap-Nonce eines vernichteten Schlüssels |
| **Pre-Destruction-Inventar** | Metadaten-Digest aller Chiffratblöcke je `key_id` vor der Vernichtung |
| **PersonalRef** | 128-Bit-Zufalls-ID als Verweis auf P-Daten in K-Dokumenten |
| **Positivkontrolle (AC-1)** | Pflichtprüfung, dass der Scanner einen eingeschleusten Kontrolldatensatz findet |
| **R-GRAPH / R-KEKEPOCH** | Restkanäle im Bericht: Graph-Topologie bzw. KEK-Epochen bis zum Seal |
| **Saga** | Die Löschsequenz aus den Schritten 1 bis 9 mit Journal und Recovery |
| **Siegel 1–4** | Verkettete, signierte Teilberichte nach den Schritten 1, 4, 7 und 8 |
| **Slot** | Dichte `u32`-Adresse einer Einheit innerhalb einer Collection |
| **SubjectRef** | 128-Bit-Zufalls-ID, die einen Betroffenen identifiziert |
| **Verification / Validation** | Technischer Lauf bzw. Abnahme durch den Betreiber, getrennt ausgewiesen |
| **Vault** | Historischer Name. Schlüsselverwahrung liegt bei `KeyAnchor` und `contextra-crypto`; `contextra-vault` bleibt ausgeschlossen. |

---

## 17 Finalisierte Architekturentscheidungen

| Thema | Entscheidung für Version 2.1 |
|---|---|
| Irreversibilität | Nach `Frozen` gibt es keine Karenz und keinen Rückweg. `Restricted` entsteht nur vor `Frozen` durch Hold oder Aufbewahrungspflicht. |
| Custody | Ausgeliefert sind K0 und K1. K2 und K3 werden abgewiesen, bis eine spätere Version Konformitätssuite und Implementierung gemeinsam liefert. |
| K1 | Argon2id verwendet mindestens 64 MiB, drei Iterationen und 250 ms; höhere Zielwerte werden nur über Benchmarks angehoben. |
| Schlüssel und Nachweis | Ed25519 ist Pflicht. Referenz-Hashes sind BLAKE3-keyed unter `K_ref`. `REQ-DEL-N1` bis `N27` bleiben die einzigen normativen Anforderungskennungen; `N-01` bis `N-29` werden nicht verwendet. |
| Lineage | Jede persistierte D-Ableitung, einschließlich Embeddings, Textpostings, Graph-Snapshots, Cache-Snapshots und Exporte, trägt mindestens eine vollständige Quellreferenz. Mehrere Quellen werden kanonisch sortiert und dedupliziert gespeichert. |
| Mixed-Segmente | Ausschließlich `BlockAll`; keine automatische Redaction oder Kollaterallöschung. |
| Vault | `contextra-vault` bleibt dauerhaft aus dem Workspace ausgeschlossen. Eine Reaktivierung erfordert G-REACTIVATE; seine frühere Funktion ist nicht Teil von `erasure`. |
| Quellarchiv | `docs/delete/` bleibt ein nichtnormatives Archiv. Es darf keine Links aus der Produktdokumentation und keine CI-Eingaben erhalten. |

Abweichende historische Texte sind nicht normativ. Eine Änderung dieser Entscheidungen erfordert eine neue SPEC-Version und eine Migration für jedes persistierte Format gemäß INV-1.
