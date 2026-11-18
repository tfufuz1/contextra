# contextra-erasure — Crate-Spezifikation (DSGVO-Löschkern & Saga-Orchestrator)

**Ebene:** E0 (Autarkes Fundament — Kern-Closure leer)  
**Status:** Normativ  
**Version:** 1.0  
**Geltungsbereich:** `crates/contextra-erasure`  
**Referenz:** `docs/SPEC.md` §4, §5, §6, §7, §10, §11, §12

---

## 1. Zweck & Systemrolle

`contextra-erasure` ist der zentrale Löschkern von Contextra. Es implementiert die rechtskonforme, nachweisbare und unumkehrbare Löschung personenbezogener Daten (Art. 17 DSGVO) sowie Einschränkungen der Verarbeitung (Art. 18 DSGVO):
- **Crypto-Shredding & Vault:** Verwaltung der Schlüsselhierarchie (MasterKey, TenantKEK, CollectionKEK, DocumentDEK, SubjectKey). Durch Vernichtung des Schlüssels werden Chiffretexte in $O(1)$ unlesbar.
- **6-Schritte-Löschsaga:** Transaktionale, idempotente und crash-sichere Orchestrierung der Löschphasen mit genau 4 synchronen Fsync-Ankerpunkten bis zur Unlesbarkeit.
- **ReadGate-Verwaltung:** Bereitstellung des synchronen $O(1)$ In-Memory `ReadGate` für alle Lesepfade (Fail-Closed).
- **Physische Bereinigung:** Steuerung des asynchronen Purge-Fensters über alle Speicherschichten (`ErasureLayer`).
- **Beweisbare Messung & Bericht:** Positivkontrolle (Canary-Scan), Ed25519-signierter Löschbericht und kryptographisch verkettetes Ledger außerhalb des Datenverzeichnisses.

---

## 2. Architektur & Seams (Nähte)

`contextra-erasure` hängt von keinen konkreten Speicher- oder Index-Crates ab (Ring-Isolation E0). Es steuert alle Subsysteme über asynchrone Port-Traits (Seams):

```
┌─────────────────────────────────────────────────────────────┐
│                      contextra-erasure                      │
│                                                             │
│   ┌─────────────────────────────────────────────────────┐   │
│   │                     ErasureCore                     │   │
│   │                                                     │   │
│   │   ┌────────────────────┐    ┌───────────────────┐   │   │
│   │   │ SagaJournal (WAL)  │    │ SagaGate          │   │   │
│   │   │ (fsync #1, #2, #4) │    │ (O(1) ReadGate)   │   │   │
│   │   └────────────────────┘    └───────────────────┘   │   │
│   │                                                     │   │
│   │   ┌────────────────────┐    ┌───────────────────┐   │   │
│   │   │ SagaVault          │    │ SagaMeasure       │   │   │
│   │   │ (fsync #3 Key Zero)│    │ (Scanner / Armed) │   │   │
│   │   └────────────────────┘    └───────────────────┘   │   │
│   │                                                     │   │
│   │   ┌────────────────────┐    ┌───────────────────┐   │   │
│   │   │ SagaReport         │    │ SagaLedger        │   │   │
│   │   │ (Ed25519 Signer)   │    │ (Chained Log)     │   │   │
│   │   └────────────────────┘    └───────────────────┘   │   │
│   │                                                     │   │
│   │   ┌─────────────────────────────────────────────┐   │   │
│   │   │ ErasureLayer-Registry (Store, Vec, Txt, ...)│   │   │
│   │   └─────────────────────────────────────────────┘   │   │
│   └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

---

## 3. Zustandsautomat der Löschsaga

```
Requested ──► Gated ──► KeyRevoked ──► Purging ──► Verifying ──► Completed
    │            │           │             │            │
    │            └───────────┴─────────────┴────────────┴──► Pending(reason) [wiederholbar]
    ├──► Restricted ──► Lifted  (Art. 18 DSGVO)
    │        └──► Gated        (Einschränkung → Löschung)
    └──► Refused               (terminal, Retention-Pflicht aktiv)
```

### 3.1 Zustandsinvarianten
- `Completed`, `Refused` und `Lifted` sind **terminale Zustände**. Kein weiterer Übergang ist erlaubt.
- `Pending` ist nicht-terminal; Recovery erfolgt automatisch durch den Hintergrundläufer `run_purge_window`.
- Identischer Übergang ($A \to A$) ist für jeden Zustand erlaubt (vollständige Idempotenz, ER-18).
- Sobald `Gated` erreicht ist, liefert das `ReadGate` für betroffene Schlüssel zwingend `GateDecision::Erased`.

---

## 4. Die 6 Phasen im Detail (Fsync-Disziplin)

### Phase 1: Validierung, Resolve & Intent-Journal (fsync #1)
1. **Aufbewahrungsprüfung:** `RetentionAttestation` prüfen. Liegt eine aktive gesetzliche Pflicht vor (`ObligationActive`), schlägt die Löschung mit `SagaState::Refused` fehl.
2. **Duplikaterkennung:** HMAC über Target-Bytes. Falls dieselbe Löschung bereits aktiv ist, wird das bestehende Receipt zurückgegeben.
3. **Resolve:** Für jeden registrierten `ErasureLayer` wird `layer.resolve(&target)` aufgerufen $\to$ `LayerScope`.
4. **Journal-Intent:** Durabler Append-Eintrag im Journal via `contextra-durable-fs` (**fsync #1**).

### Phase 2: Lesesperre setzen (fsync #2)
1. Exklusives Halten der `CommitBarrier::exclusive()` $\to$ blockiert alle nebenläufigen Schreiber.
2. `SagaGate::add_gate(erasure_id, target)` eintrag $\to$ liefert `gated_seq`.
3. In-Memory `ReadGate` schaltet auf `Erased`.
4. Zustandsübergang zu `Gated` im Journal persistieren (**fsync #2**).
5. `CommitBarrier` freigeben. Ab diesem Zeitpunkt kann kein Schreiber mehr Daten für das Ziel committen.

### Phase 3: Layer on_gate & Schlüsselvernichtung (fsync #3)
1. Für alle Layer: `layer.on_gate(&scope)` ausführen (z.B. Vektor-Ausschlussmaske in-memory setzen).
2. **Schlüsselvernichtung im Tresor (`SagaVault`):**
   - Höchsten Schlüsselslot ermitteln (Tenant, Collection, Subject oder Document).
   - Slot mit kryptographischen Zufallsbytes aus dem OS CSPRNG überschreiben.
   - Slot mit `0x00` nullen (`zeroize`).
   - `fsync` auf die Keyring-Datei (**fsync #3**).
3. `verify_key_absent(&target)`: Lesende Gegenprüfung via Constant-Time-Vergleich. Ergibt der Slot nicht Null $\to$ Abbruch mit Fehler.

### Phase 4: Journal KeyRevoked & Receipt-Rückgabe (fsync #4)
1. Zustandsübergang zu `KeyRevoked` im Journal persistieren (**fsync #4**).
2. Observer-Event `ObservedPhase::KeyRevoked` feuern.
3. **Return `EraseReceipt { status: KeyRevoked, ... }` an den Aufrufer.**
   - Der Aufrufer ist aus dem synchronen Pfad entlassen.
   - Die Daten sind ab hier unwiderruflich und kryptographisch unlesbar.
   - Cancel-Safety ist gewährleistet (Droppen des Aufrufer-Futures ändert nichts).

### Phase 5: Asynchrones Purge-Fenster
Wird vom Hintergrundläufer `run_purge_window` unter exklusivem Halt der `PurgeBarrier` ausgeführt:
1. `check_disk_reserve`: Sicherstellen, dass min. 1GB Plattenreserve frei ist.
2. Status im Journal auf `Purging` setzen.
3. Schichtenweise Bereinigung in normativer Reihenfolge:
   - a) Snapshot-Widerruf (`contextra-mvcc`)
   - b) Text-Segment-Rewrite (`contextra-text`)
   - c) WAL-Drain & Memtable-Flush (`contextra-store`)
   - d) SSTable-Rewrite / Kompaktion (`contextra-store`)
   - e) HNSW-Graph-Reparatur (`contextra-vector`)
   - f) Graph-Bereinigung (`contextra-graph`)
   - g) Slot-Bereinigung (`contextra-crypto`)
   - h) In-Memory Cache-Invalidierung (alle Schichten)

### Phase 6: Messung, Bericht & Ledger
1. Status im Journal auf `Verifying` setzen.
2. **Positivkontrolle & Residuen-Scan:** Für jede Schicht `layer.witness(&scope, &scanner)` aufrufen.
   - Scanner MUSS Positivkontrolle bestanden haben (`Scanner<Armed>`).
   - Jeder Layer MUSS `residue_matches == 0` melden.
3. **Bericht signieren:** Ed25519-Signatur über den kanonischen JSON-Löschbericht generieren (keine PII-Klartexte im Bericht!).
4. **Ledger schreiben:** Hash-verketteter Eintrag im Ledger außerhalb des Datenverzeichnisses persistieren (`prev_hash || entry`).
5. Status im Journal auf `Completed` setzen (terminal).
6. Gate-Eintrag kompaktieren. Observer-Event `Completed` auslösen.

---

## 5. Behebung aller historischen Befunde (Normativ)

| Befund-ID | Historischer Mangel | Normative Festlegung in contextra-erasure |
|---|---|---|
| **E-F01** | Fallback-Schlüssel nach Löschung (`resolve_dek`) | **Verboten.** Fehlt ein Slot, MUSS `Err(ErasureError::SlotMissing)` geworfen werden. Keine BLAKE3-Ableitung. |
| **E-F02** | `seal` nutzte XOR-Maske statt echter AEAD | **Behoben.** Strikte Verwendung von AES-256-GCM-SIV aus `contextra-crypto`. |
| **E-F03** | Deterministische Nonces aus Schlüsseln | **Behoben.** 96-Bit Nonce zwingend zufällig aus fehlbarem OS CSPRNG. |
| **E-F04** | Stille Fallbacks bei HKDF-Fehlern | **Beseitigt.** Jeder KDF-Fehler führt zum sofortigen Abbruch (`Err`). |
| **E-F05** | Zufallsgenerator panikte bei Entropiemangel | **Behoben.** Nutzung von `rand::rngs::OsRng::try_fill_bytes` mit Fehlerbehandlung. |
| **E-F06** | Null-Schlüssel `[0; 32]` als Test-Standard | **Verboten.** Test-Vaults nutzen zufällige Einweg-Schlüssel. |
| **E-F07** | Schlüssel implementierten `Clone` | **Entfernt.** Schlüssel besitzen weder `Clone` noch `Debug`; `ZeroizeOnDrop` aktiv. |
| **E-F08** | Master-KEK nur flüchtig im RAM | **Behoben.** Master-Schlüssel wird aus Passphrase + Salt via Argon2id persistierbar abgeleitet. |
| **E-F11** | Typ-Dubletten zwischen ports und erasure | **Bereinigt.** Kanonische Typen stammen ausschließlich aus `contextra-ports`. |
| **E-F15** | Gate nutzte linearen/log-Scan | **Behoben.** $O(1)$ In-Memory Lookup via atomarer Hash-Map / BitSet. |
| **E-F16** | Positivkontrolle fälschbar | **Gehärtet.** Typestate `Scanner<Armed>` besitzt privaten Konstruktor, nur nach echtem Canary-Fund erzeugbar. |

---

## 6. Invarianten

1. **`#![forbid(unsafe_code)]`:** Vollständig sicher.
2. **Ring-Isolation:** Keine Importe von konkreten Speicher-Crates (`store`, `vector`, `text`, `graph`, `db`).
3. **Fail-Closed:** Bei jedem unklaren Zustand schlägt der Lesezugriff fehl (`Erased`).
4. **Idempotenz:** Jeder Schritt der Saga kann beliebig oft wiederholt werden, ohne Zustand zu beschädigen.
