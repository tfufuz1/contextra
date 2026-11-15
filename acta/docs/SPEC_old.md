# Acta – Normative Gesamtspezifikation (Fassung 3.0 Final)

*Einheitliche und finale Spezifikation: Architektur, Datenmodelle, Kryptografie, Löschkette, Zeitreise und rechtliche Synthese*  
*Stand: 2026-11-15 · Bezug: Repository `tfufuz1/acta` (HEAD `bbfad85`) · Autorisierte Quelle: Ausschließlich `docs/` (`Acta_Architekturanalyse_Loeschung.md`, DSGVO-Rechts- und Architektursynthesen, SPEC 2.0-E / 3.0-Entwurf)*  
*Geltung: Ersetzt `docs/SPEC.md` (Fassung 2.0-E) und `docs/SPEC(13).md`. Alle Crate-spezifischen `SPEC.md` sowie `AGENTS.md` sind ungültig und ignoriert.*

Normative Schlüsselwörter nach RFC 2119: **MUSS**, **DARF NICHT**, **SOLL**, **KANN**.

---

## 0. Status, Methode, Beweisstand und Geltung

### 0.1 Beweismarken

| Marke | Bedeutung |
|---|---|
| **\[V\]** | Am Quelltext des Repositorys verifiziert (HEAD `bbfad85`, Dateipfad und Zeilen genannt). |
| **\[M\]** | Durch Messung/Experiment in der Toolchain (Rust 1.89.0, Linux x86_64) verifiziert. |
| **\[D\]** | Aus den autorisierten Analysen in `docs/` übernommen. |
| **\[E\]** | Normative Festlegung / Architekturentscheidung dieser Spezifikation. |
| **\[R\]** | Datenschutzrechtliche Anforderung gemäß EU-DSGVO / Rechtsprechung / Aufsichtsbehörden. |

### 0.2 Befundregister des Ist-Zustands (HEAD `bbfad85`)

Die Spezifikation behebt sämtliche verifizierten Mängel des bestehenden Codes und schließt alle Lücken vorheriger Entwürfe:

| ID | Schwere | Befund | Quelltext-Beleg \[V\] |
|---|---|---|---|
| **V-01** | Kritisch | **Crypto-Shredding im Sealer unwirksam und fehlermaskiert.** `resolve_dek` setzt Hierarchietags fehlerhaft (`tenant = coll = doc`, `subject = None`). Bei Nichterreichbarkeit eines Slots (insb. gelöschter Slot) liefert `else` `blake3::keyed_hash(&self.k_unit, key)`. `VaultSealer::new()` initialisiert `k_unit = [0x55; 32]`. Jeder DEK ist ohne Tresor aus einer Konstante rekonstruierbar. Nach Slot-Löschung liefert `open` Datenmüll statt `Err(Erased)`. | `acta-erasure/src/vault/sealer.rs:98-105, 249-285` |
| **V-02** | Kritisch | **Unsichere Stromchiffre statt AEAD.** Siegelung erfolgt per XOR mit 32-Byte-Strom aus `blake3::keyed_hash(dek, …)`. Keine Nonce, kein MAC-Tag, keine AAD; `SealCtx` ignoriert. Gleicher DEK leckt das XOR zweier Klartexte (`C1 ⊕ C2 = P1 ⊕ P2`). | `acta-erasure/src/vault/sealer.rs:299-345` |
| **V-03** | Kritisch | **Löschkern isoliert.** `acta-erasure` fehlt in `[workspace] members` und `default-members`. Einziger `ErasureLayer`-Implementierer ist ein Test-Mock. `async` (tokio) im Default-Feature-Set. | Root `Cargo.toml`, `acta-erasure/Cargo.toml` |
| **V-04** | Kritisch | **Tresor-Salts im Klartext, Epochenmaskierung tot.** `derive_dek_from_vault` liest Payload der Slots T/C/S/D unmaskiert. `mask_epoch_key` ist toter Code ohne Aufrufer. Eine alte Tresor-Sicherung plus langlebiger `K_unit` leitet vernichtete DEKs erneut ab (Backup-Wiederbelebung). | `acta-erasure/src/vault/sealer.rs:204-233`, `keyring.rs:33-55` |
| **V-05** | Hoch | **Fehlerbehandlung im Kryptopfad schluckt Fehler (S7 verletzt).** `seal_record` endet auf `unwrap_or_default()`; `derive_dek` verwirft HKDF-Fehler mit `let _ =` und substituiert fehlenden Subject-Salt mit Nullen `[0u8; 32]`; `ErasureConfig::default().k_idx` nutzt Nullschlüssel; `VaultKeyring::new_in_memory` nutzt `[0x42; 32]`. | `acta-crypto/src/aead.rs:24-71`, `dek.rs:68-89`, `saga/core.rs:72` |
| **V-06** | Hoch | **Store verschlüsselt keine Nutzdaten.** `RecordSealer`/`RecordAuthenticator` sind Dummy-Traits nur in Tests. Schlüssel sind Klartext-ASCII (`t:{tenant}:{collection}:chunk:{doc}`). | `acta-store/src/util.rs:47-135`, `tenant_codec.rs:37-100` |
| **V-07** | Mittel | **Klassentrennung im Store fehlt.** `PartitionId = u32` mit `DEFAULT_PARTITION` („solange Store nur eine physische Partition kennt“). Keine Trennung N/P/D im Speicherpfad; kein `ReadGate` in `may_drop_version`. | `acta-store/src/history.rs`, `lsm/ops/compaction.rs:11,154` |
| **V-08** | Hoch | **Lösch-Barriere unverdrahtet.** `FlushPurgeFence::enter_purge` hat im gesamten Workspace keinen einzigen Aufrufer. | `acta-store/src/lsm/fence.rs:59` |
| **V-09** | Hoch | **Fehlende Lineage-Konzepte.** Keine Implementierung von Herkunftsketten, Ordinalen oder Segment-Zuordnungen. Zusagen S10 und S9 sind im Datenmodell nicht verankert. | Grep `crates/` (0 Treffer) |
| **V-10** | Hoch | **Unkontrollierte SST-Hardlinks.** `CheckpointHardlinkCloner` verlinkt physische Dateien ohne Registrierung in einer `CopyRegistry`. | `acta-checkpoint/src/hardlink_cloner.rs:194-250` |
| **V-11** | Info | **Tresor-Grabstein nullt Tag (Code übertrifft alte SPEC).** Der Code nullt `tag` und `payload` bei Grabsteinen (nur `status = 0x02` und MAC bleiben). Dies verhindert, dass das Tag als permanentes Pseudonym überdauert. | `acta-erasure/src/vault/slot.rs:294-306` |
| **V-12** | Mittel | **Fehlende Batch-Vernichtung im Tresor.** `FileVault::destroy` führt Meta-Schreibvorgang und `sync_data` je Einzelslot aus. Unzureichend für Sweeper-Durchsatz (`destroy_many` erforderlich). | `acta-erasure/src/vault/durable.rs:480-530` |
| **V-13** | Mittel | **Golden-Vektor-Drift.** Checksummen-Drift bei `canonical_report_v1.golden` und `sample_report_v1.*`. | `tests/golden_erasure_test.rs` |
| **V-14** | Hoch | **Abnahme-Tests nicht kompilierbar.** `proof_crash_resume_matrix` (67 Fehler) und `proof_fail_closed_and_consistency` (13 Fehler) referenzieren nicht-existierende oder private Symbole. | Integrationstests `acta-erasure` |
| **V-15** | Hoch | **Rohkennungen im Widerrufs-Log.** `revocation_log` speichert Kennungen append-only; verletzt Linkability-Schutz. Erfordert keyed Commitments. | `acta-erasure/src/revocation_log.rs` |
| **V-16** | Mittel | **Feature-Default-Zirkelschluss.** Workspace enthielt `default = [workspace]`. Features müssen rein additiv sein. | Workspace-Konfiguration |
| **V-17** | Mittel | **Store Compaction Deadlock/Hänger.** `test_compaction_stress_and_gc` verharrt im Unendlichen auf Single-Core-Umgebungen. | `acta-store/src/lsm/ops/compaction.rs` |

### 0.3 Geltungsbereich und Hierarchie

Diese Spezifikation ist das alleinige, verbindliche Referenzdokument für die gesamte Acta-Architektur. Sie ersetzt alle vorherigen Fassungen. Widersprüche innerhalb des Repositories werden zugunsten dieser Spezifikation aufgelöst.

---

## 1. Zweck, Mission, Zusagen und Nicht-Zusagen

### 1.1 Mission \[E, R\]

Personenbezogene Daten MÜSSEN von ihrer Erfassung an konstruktiv so modelliert werden, dass sie jederzeit **atomar, physisch und kryptografisch rückstandsfrei** aus dem gesamten RAG-System entfernt werden können (Primärspeicher, Indizes, Caches, Ableitungen, Acta-eigene Kopien und Sicherungen).

Gleichzeitig MUSS das System eine **bitemporale Zeitreise (MVCC)** über nicht-personenbezogene Sachverhalte bereitstellen. Unternehmen und Regulatoren müssen lückenlos nachvollziehen können, auf Basis welcher Normen, Gesetze, Richtlinien, Prompt-Versionen, Modell-Stände und Grenzwerte ein LLM eine Entscheidung getroffen hat.

**Kernparadoxon und Lösung:**  
Eine klassische Zeitreise erfordert unveränderliche Historien (Append-Only / WORM). Ein datenschutzkonformes Löschen erfordert die absolute Vernichtung von Informationen. Beide Anforderungen sind **nur dadurch vereinbar, dass personenbezogene Daten und Zeitreise-Metadaten strikt entkoppelt werden**:
1. Personenbezogene Nutzdaten liegen im kryptografischen Tresor (Ebene P) und werden über flüchtige Schlüssel vernichtet.
2. Sach- und Entscheidungszusammenhänge liegen in der Chronik (Ebene N), sind streng anonymisiert und bitemporal abfragbar.
3. Der Löscherfolg triumphiert absolut über historische Zeitreiseabfragen: Ein `as_of(t)` in die Vergangenheit liefert für gelöschte Personendaten niemals Klartext, sondern belegt den Löschzustand (`Erased`).

### 1.2 Normative Zusagen (S1 bis S12)

Jede Zusage MUSS durch automatisierte Tests nachgewiesen werden. Eine Zusage ohne Test gilt als nicht erfüllt.

| ID | Normative Zusage | Verifikationstest |
|---|---|---|
| **S1** | **Sofortige Lesesperre.** Nach Rückkehr von `erase()` mit Status `Gated` liefert kein Lesepfad des Gesamtsystems Klartext des Ziels, auch nicht über Zeitreiseabfragen (`as_of`). Schreibvorgänge auf das Ziel werden sofort verweigert. | `T-DEL-1`, `T-CONC-1`, `T-TT-1..8` |
| **S2** | **Kryptografische Isolation.** Jeder persistierte Klartext eines Betroffenen liegt vor dem Schreiben in den WAL in einem AEAD-Umschlag unter Schlüsseln, die nach der Vernichtung physisch nicht mehr existieren. | `T-CRY-1`, `T-SEAL-1..4`, `T-WAL-ENC` |
| **S3** | **Mehrbetroffenen-Konsistenz.** Teilt ein Dokument mehrere Betroffene und ist nicht verlustfrei trennbar, wird es unlesbar, sobald ein Betroffener gelöscht wird (Modus `DeleteAll`; Mitbetroffene werden im Audit-Bericht deklariert). | `T-MULTI-1` |
| **S4** | **Crash-Invarianz.** Ein Systemabsturz an jedem beliebigen Durabilitätspunkt führt nach Wiederanlauf entweder zur Vollendung der Löschung oder zu einem Zustand `Pending` mit dauerhaft aktivem Gate. Ein einmal gesetztes Gate geht niemals verloren. | `T-CRASH-C1..C12` |
| **S5** | **Freiheit der Chronik von Personenbezug.** Die bitemporale Chronik (Ebene N) enthält weder direkt noch indirekt identifizierende personenbezogene Daten. Jeder Schreibversuch mit Personenbezug scheitert hart (Fail-Closed). | `T-CHR-1`, `T-ANON-1..3` |
| **S6** | **Zeitreise-Determinismus.** Historische Abfragen über Ebene N liefern bei identischen Parametern byte-identische Ergebnisse. Eine Löschung in Ebene P verändert die Hash-Kette von Ebene N nicht. | `T-CHR-2`, `T-RANK-1` |
| **S7** | **Fail-Closed Krypto-Semantik.** Kein Fehler im Schlüssel-, Zufalls-, KDF- oder Siegelpfad darf jemals zu einem Ersatzwert, Fallback-Schlüssel oder Klartext führen. Es gibt keine Default-Schlüssel. | `T-NOFALLBACK-*`, `T-RNG-1`, Gate `G-CRY-1` |
| **S8** | **Typestate-Beweisführung.** Ein Löschnachweis kann ausschließlich durch eine Messung mit bestandener Positivkontrolle erzeugt werden. Nicht implementierte Prüfungen liefern `NotImplemented`, niemals Erfolg. | `T-EVID-1`, `T-STUB-1` |
| **S9** | **Anti-Wiederauferstehung (Restore-Schutz).** Das Einspielen einer alten Datensicherung kann gelöschte Daten nicht wiederbeleben. Tresor-Sicherungen älter als `T_ROT_BACKUP` (72 h) werden hart abgewiesen. | `T-REST-1..3`, `RestoreGuard` |
| **S10** | **Herkunfts-Typzwang (Lineage).** Jede abgeleitete Struktur (Vektor-Segment, Posting, Graph-Kante) trägt zwingend eine Herkunftsinformation. Löschung der Quelle tilgt die Ableitung. Ableitungen ohne Herkunft können typbedingt nicht konstruiert werden. | `T-DERIV-1`, Compile-Fail-Tests |
| **S11** | **Container-Schlüsselisolation (Invariante I-17).** Zwei Datensätze verschiedener Verschlüsselungseinheiten teilen niemals denselben Chiffrierschlüssel innerhalb eines persistierten Containers (SST, WAL). Es gibt keine Block-Verschlüsselung über Dokumentgrenzen. | Property-Tests, `Sealed<P>` |
| **S12** | **Pseudonymitätsfreie Audit-Protokolle.** Weder Schlüsselmaterial noch rohe Kennungen dürfen im Klartext in Append-Only-Protokollen (Ledger, Widerrufs-Log) erscheinen. Protokolle verwenden rotierte, keyed Commitments. | `T-LOG-1` |

### 1.3 Verbotene Aussagen und Grenzen (G-WORD) \[E, R\]

Es ist ausnahmslos **verboten**, in Berichten, Dokumentationen, Schnittstellen oder Marketingunterlagen folgende Formulierungen zu verwenden:
- „DSGVO-konform“ oder „DSGVO-zertifiziert“ (datenschutzrechtlich unzulässig; Konformität ist ein Gesamtbetriebszustand, keine Software-Eigenschaft).
- „100 % gelöscht“, „garantiert vernichtet“, „unwiederbringlich“, „revisionssicher“, „fehlerfrei“.
- „Löscht aus allen Backups“ (technisch unzutreffend bei Offline- oder Bandmedien).

**Verbindliche Berichterstattung:**  
Acta berichtet ausschließlich objektiv messbare technische Erfüllungsklassen (**G, K, P, D, C**) mit dem Status `reached` oder `not_reached` unter Ausweisung expliziter Grenzannahmen (**A-FLASH, A-RAM, A-ANCHOR, A-CRYPTO, A-EXT**).

### 1.4 Anonymitätskriterien für Ebene N \[R\]

Gemäß Gutachten der Art.-29-Datenschutzgruppe (WP 216) und EDPB-Leitlinien müssen anonyme Daten drei Risiken ausschließen:
1. **Aussonderung (Singling Out):** Es darf unmöglich sein, einen Datensatz einer Einzelperson zuzuordnen.
2. **Verknüpfbarkeit (Linkability):** Es darf unmöglich sein, zwei Datensätze derselben betroffenen Person zuzuordnen.
3. **Ableitbarkeit (Inference):** Es darf unmöglich sein, aus dem Gesamtzustand Eigenschaften einer Person mit hoher Wahrscheinlichkeit zu errechnen.

Ebene N erzwingt diese Kriterien technisch über den **N-Validator** (Whitelist, Entropie-Check, k-Anonymitäts-Schwelle $K_{\text{MIN}} \ge 5$ für Aggregationen, kryptografisch zufällige Entscheidungs-Handles).

---

## 2. Begriffe, Ebenen und Datenklassen

### 2.1 Die drei Systemebenen

```
┌────────────────────────────────────────────────────────────────────────┐
│                        Ebene N: Chronik (WORM)                         │
│  - Gesetze, Richtlinien, Grenzwerte, Prompt-Versionen, Modellstände    │
│  - Audit-Quittungen (keyed Commitments), Hash-Kette                    │
│  - Streng frei von personenbezogenen Daten (PII)                      │
│  - Bitemporale Zeitreise: unbegrenzt, deterministisch                  │
│  - Löschung: Niemals im Normalbetrieb; Ausnahme nur juristischer       │
│    Takedown per Vier-Augen-Prinzip (NormKey-Vernichtung)               │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │ Referenz nur über zufällige Handles
┌───────────────────────────────────▼────────────────────────────────────┐
│                   Ebene P: Tresor (Personal Vault)                     │
│  - Personenbezogene Nutzdaten (Texte, Metadaten, Vektor-Embeddings)    │
│  - Jeder Datensatz verschlüsselt unter Dokument-DEK (Umschlag v3)      │
│  - Schlüsselhierarchie: Anker -> Epoche -> Mandant -> Coll -> Subj ->  │
│    Dokument -> DEK                                                     │
│  - Zeitreise: Streng begrenzt auf Horizont (max. 30 Tage)              │
│  - Löschung: Crypto-Shredding (O(1)) -> Physischer Purge (Hygiene)     │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │ Lineage: monotone doc_ord Bitmaps
┌───────────────────────────────────▼────────────────────────────────────┐
│                    Ebene D: Ableitungen (Indizes)                      │
│  - HNSW-Vektorgraph, BM25-Postings, Wissensgraph, Caches               │
│  - Tragen zwingend Herkunft (`Lineage { source: RoaringBitmap }`)      │
│  - Zwei Betriebsmodi:                                                  │
│      * Rebuild (Standard): Maskierung im Gate, Segment-Neubau          │
│      * Sealed (Spezialfall): Verschlüsselt unter ArtKey (≤ 50k Vekt.) │
│  - Löschung: Sofortige Maskierung im Gate -> Bereinigung/Rebuild       │
└────────────────────────────────────────────────────────────────────────┘
```

### 2.2 Datenklassen (Regel R-CLS)

Jeder Schreibvorgang MUSS explizit eine Datenklasse deklarieren. Fehlt die Angabe, bricht die Operation mit `ClassViolation` ab.

| Datenklasse | Ebene | Standard-Aufbewahrung | Primärer Löschmechanismus | Retention-Policy |
|---|---|---|---|---|
| `Norm` | N | Unbegrenzt | Juristischer Takedown | `KeepAll` |
| `Evidence` | N | Unbegrenzt | Unlöschbar (garantiert PII-frei) | `KeepAll` |
| `Personal` | P | Max. 30 Tage oder `retain_until` | Crypto-Shredding (Slot-Vernichtung) + Purge | `LiveWhileKeyAlive` |
| `Special` (Art. 9/10 DSGVO) | P | 0 Tage (keine Karenz) | Sofortiges Crypto-Shredding + Prioritärer Purge | `StrictZero` |
| `Derived` | D | Keine eigenständige Frist | Herkunftsbasierter Purge / Segment-Rebuild | `Windowed` |
| `Unclassified` | P | Quarantäne (max. 24 h) | Wie Personal; automatische Sperre bei Fristablauf | `Quarantine` |

---

## 3. Systemarchitektur, Ringmodell und Ports

### 3.1 Ringarchitektur (Abhängigkeitsinvariante)

Abhängigkeiten zeigen streng hierarchisch von höheren Ringen zu niedrigeren Ringen. Querverbindungen innerhalb eines Ringes sind verboten.

```
       R7: API & Schnittstellen (acta, acta-cli, acta-mcp)
                                │
       R6: Composition Root (acta-node)
                                │
       R5: Orchestrierung (acta-chronicle, acta-engine)
                                │
       R4: Adapter-Brücke (acta-erasure-adapters)
               /                                \
              ▼                                  ▼
   R3: Löschkern (acta-erasure)        R1: Persistenz & Indizes
                                       (acta-store, acta-mvcc,
                                        acta-vector, acta-text,
                                        acta-graph, acta-rank,
                                        acta-checkpoint)
              \                                  /
               ▼                                ▼
       R2: Systemverträge (acta-types, acta-ports)
                                │
       R0: Fundament (acta-ids, acta-crypto, acta-env, acta-sys,
                      acta-durable-fs, acta-simd, acta-wire)
```

**Regel R-DEP:**  
Gate-Lint `G1` prüft in der CI automatisiert via `cargo metadata`, dass keine unzulässigen Abhängigkeitskanten existieren.

### 3.2 Workspace- und Crate-Hygiene

1. `acta-erasure` MUSS zwingend in `[workspace] members` und `default-members` geführt werden (Behebung V-03).
2. Features sind rein additiv. Die Verwendung von `default = [workspace]` ist untersagt.
3. Im gesamten Produktionscode sind `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing, clippy::let_underscore_must_use)]` verbindlich aktiviert.
4. `unsafe`-Blöcke sind ausschließlich in `acta-sys`, `acta-simd` und generiertem `acta-wire`-Code gestattet. Jeder `unsafe`-Block erfordert einen formalen `// SAFETY:`-Kommentar.
5. Verbotene Standardfunktionen: Direkte Aufrufe von `std::fs::remove_file`, `std::fs::hard_link`, `std::fs::copy`, `std::time::SystemTime::now` und `rand::thread_rng` sind im Kernpfad per `clippy.toml` verboten (Prinzip P6). Alle Zugriffe müssen über die R0-Ports erfolgen.

### 3.3 Port-Spezifikation (Regel R-PORT)

Jeder logische Port besitzt im Produktionsbetrieb genau eine Implementierung:

| Port-Trait | Konsumierendes Crate | Produktionsimplementierung | Bereitstellendes Crate |
|---|---|---|---|
| `Clock`, `Rng`, `IdGen` | Alle | `OsClock`, `CryptoRng`, `MonotonicIdGen` | `acta-env` |
| `Vfs` | Persistenz, Tresor | `DurableVfs` (O_DIRECT, sync) | `acta-durable-fs` |
| `CryptoPrimitives` | Löschkern, Store | `ChaChaCrypto`, `Blake3Hasher` | `acta-crypto` |
| `KeyAnchor` | Löschkern (Keyring) | `HsmAnchor` / `KmsAnchor` (Prod), `FileAnchor` (Dev) | `acta-crypto` |
| `ErasureLayer` | Löschkern | `StoreLayer`, `VectorLayer`, `TextLayer`, `GraphLayer`, `CacheLayer`, `CheckpointLayer` | `acta-erasure-adapters` |
| `SnapshotRevoker` | Löschkern (Gate) | `MvccRevoker` | `acta-mvcc` |
| `ResidueScanner` | Löschkern (Messung) | `ForensicScanner` | `acta-erasure` |
| `ReportSigner` | Löschkern (Audit) | `Ed25519HsmSigner` | `acta-crypto` |

Die historische Duplizierung zwischen `acta-erasure::types_local` und `acta-ports` ist aufgehoben; alle Löschtypen sind kanonisch im Löschkern verankert.

---

## 4. Kryptografie, Schlüsselmodell und Tresor (normativ)

### 4.1 Kryptografische Primitive

| Zweck | Algorithmus / Standard | Parameter / Spezifikation |
|---|---|---|
| Datensatz-Verschlüsselung | XChaCha20-Poly1305 | 256-Bit Schlüssel, 192-Bit Zufallsnonce aus `Rng` (kein API-Parameter, I-15) |
| Schlüsselableitung (KDF) | HKDF-SHA256 | RFC 5869; Fehler zwingend propagiert, strikt ohne Fallback |
| Passwortbasierte Ableitung | Argon2id | RFC 9106; $m \ge 19\,456\text{ KiB}$, $t \ge 2$, $p \ge 1$ |
| Integritäts-MACs (WAL, Slots)| HMAC-SHA256 | FIPS 198-1; 256-Bit Schlüssel, gekürzt auf 64 Bit im Slot |
| Identifikatoren & Commitments| BLAKE3 Keyed | 256-Bit Domänentrennung |
| Berichtssignatur | Ed25519 | PureEd25519 (RFC 8032); Domänen-Präfix `ctx/report/v1` |

### 4.2 Die einheitliche Schlüsselhierarchie (Entscheidung D-05)

Die Ableitung von Nutzdaten-Schlüsseln erfolgt strikt deterministisch über eine hierarchische Kette von Zufalls-Salts im Tresor:

```
                           [ Hardware-Sicherheitsmodul / KeyAnchor ]
                                              │
                      ┌───────────────────────▼────────────────────────┐
                      │    Epochenschlüssel E_e (K_enc, K_vm, K_commit) │  (Rotation alle 24h)
                      └───────────────────────┬────────────────────────┘
                                              │ Maskierung (HKDF-Strom) & MAC
          ┌───────────────────────────────────┼──────────────────────────────────┐
          ▼                                   ▼                                  ▼
     [ Slot U ]                          [ Slot T ]                         [ Slot C ]
  Mandanten-Schlüssel                   Mandanten-Salt                   Collection-Salt
     (K_unit, 256b)                       (Salt_T, 256b)                     (Salt_C, 256b)
          │                                   │                                  │
          └─────────────────┬─────────────────┴──────────────────────────────────┘
                            │
              ┌─────────────▼─────────────┐
              │         [ Slot S ]        │  Betroffenen-Salt (Subject)
              │       (Salt_S, 256b)      │  (Wird bei Art.-17-Löschung vernichtet!)
              └─────────────┬─────────────┘
                            │
              ┌─────────────▼─────────────┐
              │         [ Slot D ]        │  Dokument-Salt
              │       (Salt_D, 256b)      │  (Wird bei Dokument-Löschung vernichtet!)
              └─────────────┬─────────────┘
                            │
                            ▼
          Dokument-DEK (Data Encryption Key, 256 Bit)
```

**Normative Ableitungsvorschrift:**
$$\text{Salt}_{\text{comb}} = \text{SHA-256}\Big(\text{"acta/kdf/v3"} \parallel \text{Salt}_T \parallel \text{Salt}_C \parallel n_{\text{u16}} \parallel \text{Salt}_{S,1} \dots \text{Salt}_{S,n} \parallel \text{Salt}_D\Big)$$
$$\text{DEK}(\text{doc}) = \text{HKDF-SHA256}\Big(\text{IKM} = K_{\text{unit}},\, \text{Salt} = \text{Salt}_{\text{comb}},\, \text{Info} = \text{"acta/dek/v3"} \parallel \text{doc\_tag}\Big)$$

**Hierarchie-Regeln (H1 bis H5):**
- **H1:** $T$, $C$, $S$, $D$ sind physisch getrennte Slots. `resolve_dek` MUSS `tenant_tag`, `collection_tag`, `subject_tag` und `doc_tag` getrennt auflösen (Behebung V-01).
- **H2:** Fehlt ein Slot in der Kette (Status Grabstein oder leer), MUSS das Ergebnis `Err(ErasureError::Erased(SlotType))` lauten. Ein Fallback ist absolut verboten (Fail-Closed).
- **H3:** Fehlt der Betroffenen-Salt $S$ bei einem Dokument mit Personenbezug, liegt ein schwerer Integritätsfehler vor.
- **H4:** Die Vernichtung wirkt kaskadierend nach oben: Vernichtung von $S$ macht augenblicklich alle DEKs sämtlicher Dokumente der betroffenen Person unberechenbar. Vernichtung von $D$ tilgt ein Einzeldokument. Vernichtung von $C$ tilgt eine Collection. Vernichtung von $U$ oder $T$ tilgt den gesamten Mandanten.
- **H5:** $K_{\text{unit}}$ liegt im Tresor als eigener Slot-Typ **U** (Tag = `tenant_id`).

### 4.3 Epochenrotation (Backup-Härtung)

Um alte Tresor-Sicherungen gegen nachträgliche Entschlüsselung abzusichern, erzwingt Acta eine rollierende Epochenrotation:
1. Alle $T_{\text{ROT}} = 24\text{ h}$ oder nach jeweils $N_{\text{SHRED}} = 10\,000$ Slot-Vernichtungen erzeugt das System einen neuen Epochenschlüssel $E_{e+1} = (K_{\text{enc}}, K_{\text{vm}}, K_{\text{commit}})_{e+1}$ im Anker.
2. Alle lebenden Slots werden unter $E_{e+1}$ neu maskiert und atomar in eine neue Tresordatei überführt (`fsync`, `rename`, `fsync(dir)`). Die DEKs der Nutzdaten verändern sich dabei **nicht**.
3. Der alte Epochenschlüssel $E_e$ wird im `KeyAnchor` unwiederbringlich physisch zerstört.
4. Ein Journal-Eintrag `EpochSealed { epoch: e }` wird festgeschrieben.

**Garantie:** Eine physische Kopie des Tresors aus Epoche $\le e$ ist nach Vernichtung von $E_e$ mathematisch unlesbar. Backup-Wiederbelebungen sind damit ausgeschlossen (Behebung V-04).

### 4.4 Datensatz-Umschlag v3 (AEAD Envelope)

Jeder Datensatz in Ebene P wird vor der Übergabe an den Store versiegelt:

```
┌──────────┬──────────┬─────────────┬────────────┬────────────────────────┬──────────┐
│ ver (u8) │ alg (u8) │ doc_tag(16) │ nonce (24) │   ciphertext (varlen)  │ tag (16) │
│  0x03    │  0x01    │             │            │                        │ Poly1305 │
└──────────┴──────────┴─────────────┴────────────┴────────────────────────┴──────────┘
```
- **Zusätzliche authentifizierte Daten (AAD):**  
  $\text{AAD} = \text{"acta/rec/v3"} \parallel \text{tenant\_tag} \parallel \text{class\_u8} \parallel \text{keyspace\_u16} \parallel \text{user\_key} \parallel \text{seq\_u64be}$
- Das Öffnen (`open`) liefert `Zeroizing<Vec<u8>>`. Jeder Fehler (Länge, MAC-Bruch, fehlender Slot) führt ohne Orakel zu `Err(CryptoError::DecryptionFailed)`. Ist der Slot nachweislich vernichtet, liefert der Pfad `Err(CryptoError::Erased)`.
- **Invariante I-17:** Jeder Datensatz besitzt seinen eigenen Umschlag. Persistierte Blöcke (SST-Blöcke, WAL-Frames) teilen niemals Schlüssel über verschiedene Dokumente hinweg.

### 4.5 Tresor-Layout (Vault Format)

Der Tresor besteht aus 16 Shards (`shard-00` bis `shard-15`):
- **Dateikopf (64 Byte):** Magic `CTXFILE\0`, `format_version = 3`, `kind = 10`, `alg_ids`, `file_id` (16B Zufall), `created_seq`, `epoch` (u32), `header_tag` (16B Poly1305).
- **Metablock (64 Byte):** `vault_id`, `shard_no`, `capacity_log2`, `live_count`, `tombstone_count`, `generation`, `scrub_cursor`, `meta_mac`.
- **Slots (jeweils 64 Byte ab Offset 128):**
  ```
  [slot_type u8][status u8][gen u48][tag 16][payload 32][mac 8]
  ```
  * `slot_type`: 0x01 U, 0x02 T, 0x03 C, 0x04 S, 0x05 D, 0x06 F, 0x07 E.
  * `status`: 0x00 Leer, 0x01 Aktiv, 0x02 Grabstein (Tombstone).
  * `payload`: Zwingend maskiert mit $\text{Salt} \oplus \text{HKDF}(K_{\text{enc}}, \text{"ctx/slotmask/v1"} \parallel \text{epoch} \parallel \text{tag})$.
  * `mac`: HMAC-SHA256 unter $K_{\text{vm}}$ gekürzt auf 8 Byte.
- **Grabstein-Invariante (Bestätigung V-11):**  
  Bei der Vernichtung werden `tag` und `payload` vollständig mit Nullen überschrieben (`0x00`). Nur `status = 0x02`, die erhöhte `gen` und der gültige `mac` bleiben erhalten. Dadurch wird verhindert, dass das Tag als permanentes Personen-Pseudonym überdauert.
- **Batch-Löschung:** Zur Entlastung des I/O bei Sweeper-Läufen stellt der Tresor `destroy_many(&[SlotRef])` bereit, welches Metadaten und `fsync` bündelt (Behebung V-12).

### 4.6 Ableitungssegmente (Modi Rebuild vs. Sealed)

1. **Modus `Rebuild` (Standard für Vektoren & Text):**  
   Segmente besitzen keinen Einzelschlüssel. Sie sind rein abgeleitete Daten. Nach dem Setzen des Gates (E2) maskiert der Index Anfragen über die Roaring-Bitmap `ErasedOrds`. Das betroffene Segment wird asynchron ohne die gelöschten Einheiten neu synthetisiert. Es entsteht kein Verfügbarkeitsverlust für fremde Betroffene (Behebung F-5).
2. **Modus `Sealed` (Spezialmodus für statische Archive):**  
   Segmente werden unter einem eigenen `ArtKey` (Slot-Typ E) versiegelt. Vernichtung des `ArtKey` macht das Segment sofort unlesbar. Erlaubt nur für Segmente mit maximal $50\,000$ Vektoren.

---

## 5. Das normative Löschprotokoll (Saga)

### 5.1 Löschziele und Geltung

| Ziel (`KeyScope`) | Auslöser / Rechtsgrundlage | Vernichteter Slot | Physische Auswirkung |
|---|---|---|---|
| `Subject` | DSGVO Art. 17 (Widerruf, Zweckfortfall) | **S** (Subject) | Sämtliche Dokumente und Ableitungen der Person |
| `Document` | Gezielte Einzellöschung | **D** (Document) | Einzelnes Dokument in allen Versionen |
| `Collection` | Fachliche Bereinigung | **C** (Collection) | Alle Dokumente der Collection |
| `Tenant` | Mandantenkündigung (Off-Boarding) | **T** und **U** | Vollständige Tilgung des Mandanten |
| `Expired` | Fristablauf (DSGVO Art. 5 Abs. 1 lit. e) | Entsprechender Slot | Durch Sweeper automatisiert getilgt |
| `Restrict` | DSGVO Art. 18 (Einschränkung) | *Keiner* | Sperre im Gate (`Restricted`), Daten bleiben erhalten |
| `Supersede` | DSGVO Art. 16 (Berichtigung) | *Keiner* | Alte Version physisch getilgt, DEK bleibt identisch |
| `Replace` | Vollständiger Dokumentaustausch | **D** (alt) | Neuer Datensatz vorab dauerhaft gespeichert |

### 5.2 Zustandsautomat (Saga Monotonie)

Der externe Löschzustand schreitet streng monoton voran. Es gibt **keinen Zustand `Failed`**.

```
  [Requested] ──────► [Gated] ──────► [KeyDestroyed] ──────► [Purging] ──────► [Verifying] ──────► [Completed]
       │
       ├──────────────► [Refused] (Terminal vor E2: Aufbewahrungspflicht greift)
       ├──────────────► [Restricted] (DSGVO Art. 18)
       └──────────────► [Pending(Grund)] / [Escalated(Grund)] (Automatische Wiederaufnahme)
```

Ein einmal gesetztes Gate (`Gated`) wird unter keinen Umständen wieder aufgehoben.

### 5.3 Die normativen Einzelschritte (E0 bis E9)

Zur Einhaltung der $O(1)$-Latenzzusage bis zum Rücksprung und zur Vermeidung von Klartextfenstern im RAM ist die Reihenfolge der Schritte exakt normiert (Behebung F-1 und F-2):

```
┌────────────────────────────────────────────────────────────────────────┐
│ E0: Validierung & Autorisierung                                       │
│     Prüfung Aufbewahrungspflichten; Auflösung Subject -> Dokumente     │
├────────────────────────────────────────────────────────────────────────┤
│ E1: Absichtsprotokollierung (fsync #1)                                │
│     Verschlüsselte Scope-Datei + Journal-Eintrag `Requested+Intent`    │
├────────────────────────────────────────────────────────────────────────┤
│ E2: Sofortige Torsperre: ReadGate setzen (fsync #2, Gruppen-Commit)   │
│     gate_epoch += 1; GateLog-Append; ErasedOrds aktualisieren;         │
│     Snapshot-Revocation.                                               │
│     ==> AB HIER: Kein Lesepfad liefert mehr Daten! O(1) garantiert!   │
├────────────────────────────────────────────────────────────────────────┤
│ E2a: Nadelstichprobe entnehmen (im Gate)                               │
│     Entnahme stratifizierter Klartextfenster (max. NEEDLE_MAX = 4096)  │
│     ausschließlich für spätere Forensik-Messung. Speicher im mlock.    │
├────────────────────────────────────────────────────────────────────────┤
│ E2b: Schicht-Quittung (on_gate Ack Pflicht!)                          │
│     Alle Schichten nullen flüchtige Caches & RAM-Vektoren. Fehlt Ack   │
│     einer Schicht, verharrt Saga in Pending(LayerNotAcked).           │
├────────────────────────────────────────────────────────────────────────┤
│ E3: Kryptografischer Commit (fsync #3 & #4)                           │
│     Slot-Grabstein im Tresor schreiben; Direct-Read-Verifikation;      │
│     Journal-Eintrag `KeyDestroyed`. Schlüssel ist physisch weg!        │
│     ==> Rückkehr an den Aufrufer mit Status Gated / KeyDestroyed       │
├────────────────────────────────────────────────────────────────────────┤
│ E4: Audit-Quittung & Chronik-Receipt                                  │
│     Ledger-Hash-Kette fortführen; Quittung in Ebene N eintragen        │
│     (ausschließlich keyed Commitments, strikt keine Roh-IDs!).         │
├────────────────────────────────────────────────────────────────────────┤
│ E5: Physisches Purge-Fenster (Reihenfolge a bis h, asynchron)         │
│     Compaction-Barriere; gefilterte Rewrites; Drop ganzer SSTs.        │
├────────────────────────────────────────────────────────────────────────┤
│ E6: Ableitungs-Synthese                                                │
│     Index-Segmente im Rebuild-Modus neu erzeugen; ArtKeys vernichten. │
├────────────────────────────────────────────────────────────────────────┤
│ E7: Forensische Typestate-Messung                                      │
│     Messklassen MK1, MK2, MK3 mit Positivkontrolle (LayerWitness).     │
├────────────────────────────────────────────────────────────────────────┤
│ E8: Audit-Bericht & Abschluss                                         │
│     ErasureReport signieren; Journal `Completed`; Scope-Datei scrubben.│
├────────────────────────────────────────────────────────────────────────┤
│ E9: Benachrichtigung & Bereinigung                                    │
│     Art.-19-Signalisierung an Empfänger; Gate-Kompaktion nach 90 Tagen.│
└────────────────────────────────────────────────────────────────────────┘
```

**Normative Regeln:**
- **R-E2-E3:** Schritt E3 DARF ERST DANN ausgeführt werden, wenn E2 durabel persistiert ist und E2b von allen Schichten bestätigt wurde.
- **R-O1:** Die Dauer bis zum Abschluss von E2 ist strikt unabhängig von der Datenmenge. Im synchronen Blockierpfad liegen ausschließlich E2 und E3.
- **R-RET:** `erase()` kehrt nach maximal `erase_return_timeout` ($500\text{ ms}$) mit dem Zustand `Gated` (oder `Pending(Timeout)` bei I/O-Stau) zurück.

### 5.4 Durabilitätspunkte und Crash-Recovery-Matrix

| Absturzpunkt | Zustand beim Crash | Wiederanlaufverhalten (Recovery) |
|---|---|---|
| **C1** | Scope-Datei geschrieben, kein Intent im Journal | Waise; `scope_purge` entfernt Datei. Vorgang gilt als nie beantragt. |
| **C2** | Intent im Journal, Gate noch nicht aktiv | Recovery setzt bei Schritt E2 fort. |
| **C3** | Gate im RAM/Puffer, vor dem fsync | Gate ging verloren; Recovery liest Intent und führt E2 erneut aus. |
| **C4** | Gate gefsynct, Schlüssel noch lebendig | Gate ist aktiv; Recovery erzwingt E2b-Ack und führt E3 aus. |
| **C5** | Grabstein geschrieben, fsync nicht bestätigt | Recovery liest Slot physisch vom Medium; fehlt Grabstein, Wiederholung E3. |
| **C6** | Slot vernichtet, Journal `KeyDestroyed` fehlt | Recovery erkennt Grabstein am Medium und ergänzt `KeyDestroyed`. |
| **C7** | Während E5: Temporäre SST-Datei existiert | Temporäre Datei wird verworfen; Purge-Phase wird neu aufgesetzt. |
| **C8** | Manifest-Batch geschrieben, Altdatei noch da | Recovery gleicht obsolete Dateien mit Manifest ab; löscht Altdatei. |
| **C9** | Messung E7 schlägt fehl (`matches > 0`) | Status verbleibt in `Pending(WitnessMissing)`; E5 wird wiederholt. |
| **C10**| Vor Fertigstellung von Bericht oder Ledger | Zeugen liegen im Journal; Bericht wird determiniert nacherzeugt. |
| **C11**| Während Epochenrotation: Zwei Epochen vorhanden | Rotation wird wiederaufgenommen; alter Epochenschlüssel vernichtet. |
| **C12**| Nach Neustart fehlt Schicht-Quittung (E2b) | Status `Pending(LayerNotAcked)`; `on_gate` wird erneut getriggert. |

### 5.5 Physisches Purge-Fenster (Schritt E5, Reihenfolge a bis h)

Das Bereinigen persistierter Chiffrate erfolgt in festgelegter Priorität (`PurgePriority`):
1. **Phase a (Snapshots):** `revoke_covering` sperrt aktive MVCC-Leser; Checkpoint-Pins lösen.
2. **Phase b (Text-Index):** Term-Postings ohne Ziel-Ordinale neu schreiben.
3. **Phase c (Memtable & WAL):** Aktive Memtable drainen; WAL bis `gated_seq` recyceln (Scrub & Unlink).
4. **Phase d (SSTables):** Gefilterter Rewrite betroffener SSTs. Bei Betroffenen-Isolation (`cluster = subject_tag`) erfolgt direkter **Datei-Drop** ohne Rewrite (Behebung F-4).
5. **Phase e (Vektoren):** HNSW-Vektorslots nullen; Segmente im Rebuild-Modus neu kompilieren.
6. **Phase f (Tresor):** D-Slots abräumen; Rehash bei Shard-Lastfaktor $> 0{,}60$.
7. **Phase g (Caches):** Block-, Prompt-, KV- und Embedding-Caches vollständig invalidieren und nullen.
8. **Phase h (Kopien):** `CopyRegistry` aktualisieren; Chiffrat-Kopien bereinigen.

---

## 6. Datenmodell, Lineage und Indizes (normativ)

### 6.1 Schlüsselraum KeyV2 (Behebung V-06 und F-4)

Jeder Primärschlüssel im Store besitzt ein fixes, binäres Layout von 78 Byte (kein ASCII!):

```
┌──────────┬─────────────────┬─────────────────────┬─────────────┬────────────┬───────────┬──────────┬──────────┐
│ kind(u8) │ tenant_tag(16B) │ collection_tag(16B) │ cluster(16B)│ doc_tag(16)│ !seq(u64) │ flags(u8)│ rsv(4B)  │
└──────────┴─────────────────┴─────────────────────┴─────────────┴────────────┴───────────┴──────────┴──────────┘
◄─────────────────────────── Präfix: 65 Byte ────────────────────────────────►◄────── Suffix: 13 Byte ────────►
```
- `kind`: Exhaustives Enum (`0x01 DATA`, `0x02 DOCMAP`, `0x03 REL`, `0x04 INTENT`, `0x05 TEXT`, `0x06 GRAPH`, `0x07 VECTOR`, `0x08 CHRONICLE`, `0x09 JOURNAL`).
- **Verbindliche Cluster-Belegung (F-4):**  
  `cluster = subject_tag` des Hauptbetroffenen (Fallback bei betroffenenlosen Dokumenten: `doc_tag`).  
  *Wirkung:* Sämtliche Datensätze einer Person liegen im LSM-Baum strikt lokal beieinander. Bei einer Löschung können ganze SSTables per atomarem Manifest-Drop entfernt werden, statt teure I/O-Rewrites zu erzwingen.
- `!seq`: Bitweise invertierte Sequenznummer (neueste Versionen sortieren zuerst).
- `flags`: Bits 0–1 kodieren die `horizon_class`.

### 6.2 Lineage über monotone Ordinale und Roaring-Bitmaps (F-3)

Zur lückenlosen Verfolgung von Ableitungen gilt ein striktes, mathematisches Mengenmodell:
1. Jedes Dokument erhält beim Ingest ein dichtes, monotones **`doc_ord: u32`** je Mandant. Ordinale werden niemals wiederverwendet.
2. Jede abgeleitete Struktur (SST-Footer, HNSW-Knoten, Posting-Liste, Graph-Kante) führt ein `source_set: RoaringBitmap` aller enthaltenen `doc_ord`.
3. Das globale Gate führt **`ErasedOrds: RoaringBitmap`**.
4. **Gate-Prüfung:** `ReadGate::contains(doc_ord)` erfolgt in $O(1)$ ohne I/O.
5. **Purge-Entscheidung:** Eine Datei oder ein Segment ist genau dann betroffen, wenn:
   $$\text{source\_set} \cap \text{ErasedOrds} \neq \emptyset$$
6. **Typzwang (S10):** Der Typ `Derived<T>` kapselt Nutzdaten untrennbar mit seiner `Lineage`. Das Ablegen im Index verlangt zwingend `put_derived(Derived<T>)`. Ableitungen ohne Herkunft sind unkonstruierbar.
7. `doc_ord` ist rein intern und verschwindet mit dem D-Slot. Es erscheint niemals in Berichten oder Ebene N.

### 6.3 Store-Partitionen und Manifest v3

- Der Store erzwingt getrennte Partitionen **N, P, D** je Mandant (Behebung V-07). Compaction mischt niemals Daten verschiedener Partitionen (Invariante I-7).
- **Manifest v3 Einträge:** `AddSst { path, max_tx, partition, tag_min, tag_max }`, `RemoveSst { path }`, `SeqHwm { seq, tx }`, `HistoryFloor { partition, seq }`, `HorizonUntil { doc_tag, date }`.
- Rewrites erfolgen über atomare Batches `AddSst + RemoveSst`. Ein Schreibfehler im Manifest vergiftet den Store sofort (Fail-Closed).

### 6.4 SSTable (Format 5) und WAL-Frame v2

- **SSTable Format 5:** Footer enthält zwingend das `source_set` als Roaring-Bitmap sowie sortierte `doc_tag`-Grenzen (`tag_min`, `tag_max`).
- **WAL-Frame v2:**
  ```
  [magic u8][flags u8][len u32][hdr_crc32c u32][payload len][chain_tag 32]
  ```
  * `chain_tag` bildet eine HMAC-SHA256-Kette über alle bisherigen Frames.
  * Nutzlasten der Klasse P im WAL liegen ausnahmslos als Umschlag v3 vor.

### 6.5 Indizes (Vektor HNSW, Text, Graph)

- **HNSW Ghost Vectors:** Reines Soft-Delete via Bitmaske genügt den Anforderungen an eine Löschung nicht (EDPB/DSK-Verstoß). Acta kombiniert:
  1. Sofortiges logisches Ausblenden via `ErasedOrds` im Suchpfad.
  2. Physisches Nullen der Vektordaten im Segment.
  3. Neuaufbau der Graph-Topologie im Hintergrund-Rebuild ohne die gelöschten Knoten.
- **Vec2Text Inversionsschutz:** Vektor-Embeddings sind rechtlich als personenbezogene Daten (Ebene P) eingestuft. Rohe Embeddings dürfen niemals unverschlüsselt auf Speichermedien verbleiben.

### 6.6 Pseudonymitätsfreie Protokolle und Ledger (Behebung V-15)

- Protokolleinträge im Widerrufs-Log (`RevocationLog`) und Ledger dürfen keine Klartext-IDs enthalten.
- **Commitment-Vorschrift:**  
  $$\text{Commitment} = \text{HMAC-SHA256}\big(K_{\text{commit}}[e],\, \text{canonical}(\text{id})\big)$$
- $K_{\text{commit}}[e]$ wird mit jeder Epoche rotiert und nach Ablauf von `gate_compaction_delay` (90 Tage) physisch vernichtet. Danach sind historische Protokolleinträge selbst für den Betreiber mathematisch irreversibel anonymisiert.

---

## 7. Zeitreise (MVCC) und Entkopplung (normativ)

### 7.1 Drei orthogonale Zeitachsen

Acta verwaltet Versionen über drei unabhängige Zeitachsen:
1. **Gültigkeitszeit ($T_{\text{valid}}$):** Fachlicher Zeitraum, in dem ein Sachverhalt in der realen Welt Gültigkeit besitzt (`[valid_from, valid_to)`).
2. **Transaktionszeit ($T_{\text{tx}}$ / Sequenznummer):** Systemzeitpunkt, an dem die Information im Store erfasst wurde. Bildet die Basis für bitemporale Zeitreisen.
3. **Löschzeitpunkt ($T_{\text{erase}}$):** Physikalischer Zeitpunkt, an dem das Gate aktiviert und der Schlüssel zerstört wurde.

### 7.2 MVCC-Sichtbarkeit vs. Lösch-Gate

```
Abfrage: as_of(t_past)
   │
   ├─── 1. Prüfung ReadGate: Ist das Dokument in `ErasedOrds` enthalten?
   │         │
   │         ├── JA  ──► Sofortiger Abbruch: Err(RecordErased) !
   │         │           (Erase triumphiert ausnahmslos über historische Zeitreise!)
   │         │
   │         └── NEIN ─► 2. Standard MVCC-Sichtbarkeit:
   │                        Suche Version mit max(seq) <= t_past.
   │                        Prüfung MVCC-Tombstone.
```

Ein MVCC-Tombstone markiert lediglich das fachliche Überschreiben oder Entfernen einer Version. Er ist **keine Löschung im Sinne des Datenschutzes**. Eine echte Löschung sperrt das Gate für alle Zeitpunkte der Historie.

### 7.3 Store-Retention-Schnittstelle (F-8)

Die Bereinigung historischer Versionen im Store erfolgt entkoppelt von Fachlogik:
```rust
pub trait ErasedView: Send + Sync {
    fn is_erased(&self, ord: u32) -> bool;
}

impl RetentionPolicy {
    pub fn may_drop_version(&self, v: VersionRef<'_>, gate: &dyn ErasedView) -> bool {
        if gate.is_erased(v.doc_ord()) {
            return true; // Chiffrat gelöschter Betroffener wird sofort verworfen!
        }
        self.check_retention_horizon(v)
    }
}
```

### 7.4 Entscheidungsnachvollziehbarkeit ohne Personendaten

Ebene N speichert strukturierte Entscheidungs-Datensätze:
```rust
pub struct ChronicleEntry {
    pub id: u128,
    pub kind: ChronicleKind, // Norm | Policy | Config | Model | Rule | Receipt
    pub matter: String,
    pub valid_range: Range<Timestamp>,
    pub recorded_seq: u64,
    pub content_hash: [u8; 32],
    pub payload_ref: BlobRef, // Verschlüsselt unter NormKey
    pub supersedes: Option<u128>,
    pub prev_hash: [u8; 32],
}
```
Wurde eine LLM-Entscheidung unter Zuhilfenahme von Personendaten getroffen, referenziert der Chronicle-Eintrag lediglich das flüchtige Handle $H_d$. Nach der Löschung der Person bleibt die Begründungskette (Gesetzesstand, Leitlinien, Prompt, Modell) zu 100 % nachvollziehbar; die Eingabe-Referenz liefert `Erased`.

### 7.5 Takedown in Ebene N (Vier-Augen-Prinzip)

Muss ein Eintrag in Ebene N aufgrund eines rechtskräftigen Gerichtsbeschlusses getilgt werden:
1. Vernichtung des spezifischen `NormKey` im Anker.
2. Zwei autorisierte Administratoren müssen die Aktion kryptografisch signieren.
3. Der Payload-Blob wird unlesbar; die kryptografische Hash-Kette des Chronik-Ledgers bleibt vollständig intakt.

---

## 8. Messung, Nachweis und Verifikation (normativ)

### 8.1 Messklassen

| Klasse | Prüfbereich | Methode |
|---|---|---|
| **MK1** | Logische Strukturen | Verifikation von Indizes, Manifesten, Katalogen und Bitmaps auf Abwesenheit von Ziel-IDs. |
| **MK2** | Forensischer Rohbyte-Scan | Mustererkennung ohne Format-Parser über alle Datendateien, WAL-Segmente und den Heap. |
| **MK3** | Physische Schlüsselprüfung | Byteweise Prüfung des Tresor-Slots am Datenträger (`VerifiedVia::Direct`). |

### 8.2 Nachweisklassen und Beziehung zur DSGVO-Synthese

| Nachweisklasse | Technische Aussage | Äquivalente Klasse der DSGVO-Synthese |
|---|---|---|
| **G** | Sämtliche Lesepfade und Schnittstellen sind gesperrt. | **L** (Logisch gesperrt) |
| **K** | Schlüsselmaterial im Tresor physisch vernichtet (MK3). | **C** (Kryptografisch geschreddert, primär) |
| **P** | Chiffrate und Klartexte physisch aus WAL/SST überschrieben/entfernt (MK2).| **P** (Physisch bereinigt) |
| **D** | Ableitungen in Indizes (HNSW, Text, Graph) bereinigt (`LayerWitness`). | **P** (Teilbereich Indizes) |
| **C** | Registrierte Kopien und Checkpoints bereinigt (`CopyRegistry`). | **C** (Kopienbereich) |

Klasse **E** („Vollständig gelöscht“) gilt genau dann als erreicht, wenn alle Klassen **G, K, P, D und C** den Status `reached` aufweisen.

### 8.3 Typestate-Messung mit Positivkontrolle (S8)

Ein `LayerWitness` kann ausschließlich über einen typgesicherten Ablauf erzeugt werden:

```rust
// 1. Scanner instanziieren (Unbewaffnet)
let scanner: Scanner<Unarmed> = Scanner::new(target_needle);

// 2. Positivkontrolle erzwingen: Scanner MUSS Testmuster im Puffer finden
let armed_scanner: Scanner<Armed> = scanner.arm_with_synthetic_positive_control()?;

// 3. Echte Durchsuchung der Schicht
let scan_result = armed_scanner.scan_layer(&layer)?;

// 4. Zeuge kann NUR bei exakt 0 Treffern instanziiert werden!
let witness: LayerWitness = scan_result.into_witness()?;
```

### 8.4 Unabhängiges Verifikationstool `acta-verify`

Zur Prüfung von Löschnachweisen dient das eigenständige Tool `acta-verify` ($\le 800$ Zeilen Rust, ohne Abhängigkeit zu Acta-Interna). Es prüft:
1. Gültigkeit der Ed25519-Signatur des Berichts gegen den hinterlegten Notar-Schlüssel.
2. Integrität der Ledger-Hash-Kette.
3. Direktes Auslesen des Tresors zur Verifikation des Grabsteins (`0x00`-Payloads).

### 8.5 Automatisierte CI-Architektur-Gates

Alle Gates laufen in der CI und lokal über `cargo xtask gates`:
- **G1:** Abhängigkeitsprüfung Ringmodell (`cargo metadata`).
- **G2:** Dead-Port-Check: Jeder Port besitzt genau eine aktive Verdrahtung.
- **G3:** Determinismus: Verbot nicht-injizierter Zeit- und Zufallsquellen.
- **G-CRY-1:** Lint-Verbot aller Krypto-Fallbacks (`clippy::let_underscore_must_use`, Verbot von `Default` für Schlüssel).
- **G-FS-1:** Verbot von `std::fs::remove_file` ohne vorherigen sicheren Scrub.
- **G-ERA-1:** Löschungen ausschließlich über `ErasureCore::execute`.
- **G-READ-1:** Öffentliche Lesepfade zwingend über gegateten Reader.
- **G-DEF-1:** Alle Workspace-Crates in `default-members`.
- **G-WORD:** Wortfilter gegen verbotene Marketing-Aussagen (Abschnitt 1.3).
- **G-SUPPLY:** Lizenz- und MSRV-Prüfung (`cargo deny`).

---

## 9. Rechtliche Synthese und Betreibergrenzen

### 9.1 Europäischer Rechtsrahmen (DSGVO)

- **Art. 17 DSGVO (Recht auf Löschung):** Die Löschung ist eine Form der Verarbeitung (Art. 4 Nr. 2 DSGVO). Das Gesetz schreibt kein spezifisches technisches Verfahren vor (Technologieneutralität). Ziel ist das irreversible Aufheben der Personenbeziehbarkeit.
- **Art. 5 Abs. 1 lit. e (Speicherbegrenzung):** Daten dürfen nur so lange in identifizierbarer Form existieren, wie es der Zweck erfordert.
- **Art. 5 Abs. 1 lit. c (Datenminimierung) & Art. 25 (Privacy by Design):** RAG-Systeme müssen Chunks und Kontextfenster von vornherein so strukturieren, dass personenbezogene Anteile isolierbar bleiben.

### 9.2 Behördenpraxis und Rechtsprechung zu Crypto-Shredding

1. **EDPB Guidelines 02/2025 v2 (07.07.2026):**  
   Verschlüsselte Daten bleiben datenschutzrechtlich personenbezogen, solange ein Schlüssel existiert. Die Vernichtung des Schlüssels hebt den Personenbezug auf, birgt jedoch das Restrisiko künftiger Algorithmenbrüche. Der EDPB verlangt daher Krypto-Agilität, dokumentierte Risikobewertungen und flankierende physische Löschkonzepte.
2. **EuGH Rechtsprechung (Rs. C-582/14 *Breyer*, Rs. C-413/23 P *EDPS/SRB*):**  
   Der Personenbezug ist relativ. Wenn der Verantwortliche über keinerlei Mittel mehr verfügt, die Daten mit verhältnismäßigem Aufwand zu re-identifizieren, entfällt der Personenbezug für diese Stelle.
3. **DSK Orientierungshilfe RAG (10/2025):**  
   Für Vektordatenbanken fordern deutsche Aufsichtsbehörden die tatsächliche Unzugänglichkeit von Einträgen. Reines logisches Ausblenden (Soft-Delete) ohne physische Bereinigung wird ausdrücklich abgelehnt.
4. **Unanwendbarkeit von § 35 BDSG:**  
   Die Ausnahmeregelung des § 35 BDSG (Löschung wegen unverhältnismäßigem Aufwand durch Sperrung ersetzbar) gilt nach herrschender Auslegung **nicht** für moderne, automatisierte IT- und Vektordatenbanken. Systeme dürfen nicht auf § 35 BDSG aufbauen.

### 9.3 Standard-Datenschutzmodell (SDM Baustein 60)

Acta implementiert die Kontrollanforderungen des SDM-Bausteins 60:
- **Unkenntlichmachung:** Vollständiger Verlust des semantischen Informationsgehalts durch Schlüsselvernichtung.
- **Vollständigkeit:** Einbeziehung sämtlicher Nebenstrukturen (Indizes, Caches, WAL).
- **Protokollierung:** Lösch-Paradoxon-konformer Nachweis über keyed Commitments.
- **Wiederanlaufschutz:** `RestoreGuard` verhindert das Wiedereinspielen gelöschter Stände.

### 9.4 Deklarierte Annahmen und Systemgrenzen (Custody Boundary)

Folgende technische Randbedingungen werden im Audit-Bericht offengelegt:
- **A-FLASH:** Wear-Leveling auf SSDs und Dateisystem-Journale können physische Datenblöcke unkontrolliert vorhalten. Gegen diesen Restbestand wirkt der Entzug des Schlüssels (D-02, D-15).
- **A-RAM:** Flüchtige Reste im Arbeitsspeicher werden durch `mlock`, `Zeroizing`, `MADV_DONTDUMP` und `RLIMIT_CORE = 0` minimiert, sind jedoch nicht manipulationssicher gegen Kernel-Angriffe.
- **A-ANCHOR:** Das zugrundeliegende HSM / KMS erfüllt die zugesagte Vernichtungsgarantie für Schlüsselmaterial.
- **A-CRYPTO:** Die eingesetzten mathematischen Krypto-Primitive (XChaCha20, Poly1305, HKDF, BLAKE3) bleiben im geforderten Zeithorizont ungebrochen.
- **A-EXT:** Daten, die an externe Verarbeiter (Zone Z4, z. B. externe LLM-APIs) oder in Kundendokumenten (Zone Z5) liegen, befinden sich außerhalb der Einflusssphäre von Acta.

---

## 10. Nebenläufigkeit, Fehlertaxonomie und Konfiguration

### 10.1 Sperrenordnung (Deadlock-Freiheit)

Sperren dürfen ausschließlich in strikt aufsteigender Rangfolge akquiriert werden:

| Rang | Sperre | Geltungsbereich | Haltezeit |
|---|---|---|---|
| **1** | `CommitBarrier` | Committer (Shared), Gate-Setzer (Exclusive) | Extrem kurz: nur für `gate_epoch += 1` und Bitmap-Insert. Niemals über I/O oder fsync! |
| **2** | `PurgeBarrier` | Purge-Worker (Exclusive), Compaction (Shared) | Über das gesamte Purge-Fenster einer Phase. |
| **3** | `ErasureFence` | Schreibzulassung je Partition/Bereich | Ticket-basiert. |
| **4** | `TxBufferShards` | Sharded Transaktionspuffer | Streng aufsteigend nach Shard-Index. |
| **5** | `SnapshotRegistry` | Registrierung aktiver MVCC-Leser | Nur O(1) In-Memory Operationen. |
| **6** | `FileVaultShardLock` | Tresor-Shard-Zugriff | Niemals über `.await` halten! |
| **7** | `ReadGateLock` | Aktualisierung von `ErasedOrds` | RwLock; Leser ohne I/O. |

### 10.2 Fehlertaxonomie und Vergiftung (Poisoning)

- **Transiente Fehler:** I/O-Timeouts, temporärer Ressourcenmangel $\rightarrow$ `Pending(Grund)` mit exponentiellem Backoff ($1\text{ s}, 2\text{ s}, 4\text{ s} \dots \max 15\text{ min}$). Das gesetzte Gate bleibt aktiv.
- **Integritäts- & Krypto-Fehler:** MAC-Fehlschlag, ungültige Prüfsummen, Krypto-Ableitungsfehler $\rightarrow$ **Sofortige Vergiftung (Poisoning).** Der Store blockiert alle Schreibvorgänge; Lesezugriffe auf Ebene P werden gesperrt. Aufhebung nur durch Administrator-Eingriff und Recovery.
- **SLA-Überschreitung:** Dauer der Saga überschreitet 72 Stunden $\rightarrow$ Status `Escalated(SlaExceeded)` mit Alarmierung.

### 10.3 Normative Systemkonfiguration

| Parameter | Normativer Standardwert | Bedeutung / Zweck |
|---|---|---|
| `erase_return_timeout` | $500\text{ ms}$ | Maximale Blockierzeit im synchronen API-Aufruf bis zum Zustand `Gated`. |
| `erase_sla` | $24\text{ h}$ | Zielzeit bis zum Erreichen von Phase E8 (`Completed`). |
| `escalate_after` | $72\text{ h}$ | Automatische Alarmauslösung bei verzögerten Löschungen. |
| `legal_max` | $30\text{ Tage}$ | Gesetzliche Höchstfrist nach Art. 12 Abs. 3 DSGVO. |
| `batch_window` | $60\text{ s}$ | Bündelungsfenster für nachgelagerte Purge-Läufe. |
| `min_free_bytes` | $1\text{ GiB}$ | Notwendige Festplattenreserve vor Ausführung von Rewrites. |
| `T_ROT` | $24\text{ h}$ | Intervall der Epochenrotation im Tresor. |
| `N_SHRED` | $10\,000$ | Maximale Anzahl an Slot-Löschungen vor erzwungener Epochenrotation. |
| `T_ROT_BACKUP` | $72\text{ h}$ | Maximales Alter einer zulässigen Tresor-Sicherung. |
| `K_MIN` | $5$ | Minimale Gruppengröße (k-Anonymität) für Aggregationen in Ebene N. |
| `NEEDLE_MAX` | $4\,096$ | Obergrenze stratifizierter Klartextfenster für die Forensik-Stichprobe. |
| `history_default_days` | $30\text{ Tage}$ | Aufbewahrungshorizont für Datenklasse `Personal`. |
| `checkpoint_every` | $1\,024$ | Anzahl Chronik-Einträge zwischen zwei signierten Checkpoints. |
| `segment_max_vectors` | $50\,000$ | Maximale Vektoranzahl für Indizes im Modus `Sealed`. |
| `derived_mode` | `Rebuild` | Standardmodus für abgeleitete Vektor- und Textsegmente. |
| `vault_shards` | $16$ | Fixe Anzahl Shards im physischen Tresor. |
| `gate_compaction_delay`| $90\text{ Tage}$ | Frist bis zur endgültigen Tilgung rotierter Commitment-Schlüssel. |
| `boot_selftest` | `true` | Nicht abschaltbarer Selbsttest beim Systemstart. |
| `k_idx` | **Kein Default!** | Systemstart ohne explizite Konfiguration verweigert (Behebung V-05). |

---

## 11. Teststrategie und Pflichttests

### 11.1 Matrix: Zusage zu Abnahmetests

| Zusage | Primäre Abnahmetests |
|---|---|
| **S1** (Lesesperre) | `T-DEL-1` (Lesepfade nach Gate), `T-CONC-1` (Loom Gate-Linearisierung), `T-TT-1..8` (Zeitreise-Szenarien) |
| **S2** (Krypto-Isolation) | `T-CRY-1` (Bytes-Grep), `T-SEAL-1..4` (AEAD, AAD, Manipulation), `T-WAL-ENC` (WAL-Chiffrat) |
| **S3** (Mehrbetroffene) | `T-MULTI-1` (Unlesbarkeit bei Tilgung eines Subjekts) |
| **S4** (Crash-Invarianz) | `T-CRASH-C1..C12` (Vollständige Crash-Matrix mit `FaultVfs`) |
| **S5** (PII-Freiheit N) | `T-CHR-1` (N-Validator, Schema-Whitelist), `T-ANON-1..3` (Singling Out, Linkability, Inference) |
| **S6** (Determinismus N) | `T-CHR-2` (Identische Hashes bei Replay), `T-RANK-1` |
| **S7** (Fail-Closed) | `T-NOFALLBACK-*`, `T-RNG-1`, Gate `G-CRY-1` |
| **S8** (Typestate) | `T-EVID-1` (Positivkontrolle Pflicht), `T-STUB-1` (`NotImplemented` bei Lücken) |
| **S9** (Restore-Schutz) | `T-REST-1..3` (Abweisung alter Tresor-Backups, Erase-Replay) |
| **S10** (Lineage) | `T-DERIV-1`, Compile-Fail Test `derived_requires_lineage` |
| **S11** (I-17 Isolation) | Property-Test `no_shared_dek_in_container` |
| **S12** (Audit-Schutz) | `T-LOG-1` (Bytes-Grep über Ledger und Log mit bekannten DEKs und PII-IDs) |

### 11.2 Pflichttests für Ist-Defekte (vor Release zwingend grün)

| Defekt | Verbindlicher Testfall |
|---|---|
| **V-01** | `vault_missing_slot_is_erased`: Vernichteter Slot MUSS nach Neuerzeugung des Sealers `Err(Erased)` liefern, niemals Datenmüll oder Fallback. |
| **V-01** | `no_default_sealer_in_release`: `VaultSealer::new()` existiert ausschließlich unter `#[cfg(test)]`. |
| **V-02** | `envelope_is_aead`: Manipulation jedes beliebigen Bytes im Chiffretext führt zu Authentifizierungsfehler. Zwei Datensätze unter gleichem DEK teilen keinen XOR-Strom. |
| **V-04** | `slot_payload_masked_at_rest`: Binärer Bytes-Grep über die Tresordatei findet zu keinem Zeitpunkt unmaskierte Salts. |
| **V-13** | `golden_match_code`: Sämtliche Golden-Audit-Berichte stimmen exakt mit dem Serialisierungs-Code überein. |
| **V-14** | `proof_targets_compile_and_pass`: `proof_crash_resume_matrix` und `proof_fail_closed_and_consistency` kompilieren fehlerfrei und bestehen. |
| **V-15** | `revocation_log_has_no_raw_ids`: Binär-Scan des Widerrufs-Logs bestätigt die Abwesenheit von Klartext-Kennungen. |

---

## 12. Verbindlicher Umsetzungsplan (Walking Skeleton zuerst)

Der Umbau folgt dem Prinzip **„Dünner vertikaler Durchstich vor horizontaler Breite“**, um das Schnittstellenrisiko zwischen Löschkern und Speicher frühzeitig zu eliminieren.

```
       [ R0: Stop-the-Line Hygiene ]
                     │
       [ R1: Krypto-Kern & Tresor ]
                     │
       [ R2: Walking Skeleton Durchstich ]  <── Erstes Ende-zu-Ende Löschen!
                     │
       [ R3: Datenmodell & Lineage ]
                     │
        ┌────────────┴────────────┐
        ▼                         ▼
 [ R4: Store-Fähigkeiten ]  [ R5: Indizes & Schichten ]
        └────────────┬────────────┘
                     │
       [ R6: Chronik & Orchestrierung ]
                     │
       [ R7: Audit, HSM & Verifikation ]
```

| Stufe | Fokus | Behobene Befunde | Verbindliche Abnahmekriterien |
|---|---|---|---|
| **R0: Stop-the-Line** | Workspace-Integration, Beseitigung aller Krypto-Fallbacks und Dummy-Schlüssel, Reparatur der Integrations-Tests. | V-01, V-03, V-05, V-13, V-14 | `acta-erasure` in Workspace; Pflichttests V-01, V-05 grün; `cargo test` baut sauber. |
| **R1: Krypto-Kern** | Umschlag v3 (XChaCha20-Poly1305), Hierarchie H1–H5, Epochenmaskierung und Epochenrotation, `destroy_many`. | V-02, V-04, V-12, V-16 | Testvektoren Umschlag v3; Rotation mit Crash C11; Bytes-Grep auf Tresor-Dateien. |
| **R2: Walking Skeleton** | Minimaler Pfad: Ingest (`Sealed<P>`) $\rightarrow$ WAL $\rightarrow$ ein Index (`acta-text`) mit Lineage $\rightarrow$ `ErasureCore` synchron $\rightarrow$ E2, E2a, E2b, E3 $\rightarrow$ Messung MK2 $\rightarrow$ Bericht. | V-08, V-09 (Teil), V-15 | `T-DEL-1` grün; Kanarien-Test Ende-zu-Ende lauffähig; Crash-Tests C1 bis C6 bestanden. |
| **R3: Datenmodell** | Monotone `doc_ord`-Lineage; KeyV2 mit `cluster = subject_tag`; Partitionen N/P/D im Manifest; Wert-Verschlüsselung vor WAL. | V-06, V-07, B-04 | Invarianten I-5, I-6, I-7; Tests `no_ascii_keys`, `no_v2_envelope_in_P`. |
| **R4: Store-Fähigkeiten** | Asymmetrisches `FlushPurgeFence`, Verdrahtung von `enter_purge`; gefilterter Rewrite mit Datei-Drop; WAL-Frame v2. | V-08, B-09, B-19 | Loom-Modelle für Flush/Purge-Nebenläufigkeit; Crash-Tests C7 und C8; Behebung V-17. |
| **R5: Indizes & Schichten**| `VectorLayer` (HNSW Zeroing & Rebuild), `TextLayer`, `GraphLayer`, `CacheLayer`, `CheckpointLayer` mit `CopyRegistry`. | V-09, V-10 | Tests `T-VEC`, `T-TEXT`, `T-GRAPH`, `T-REST`; Bestätigung `hnsw_no_residue_after_erase`. |
| **R6: Chronik & Node** | `acta-chronicle`, N-Validator mit PII-Filter, `acta-engine`, `acta-node` Composition Root, Boot-Selbsttest. | V-14 (Rest) | Architektur-Gates G1, G2, G-ERA-1, G-READ-1; Tests `T-CHR-1..2`. |
| **R7: Audit & Härtung** | `acta-verify` Standalone-Tool, Notary, HSM-Signatur (A1), Replik-Commit, externe Krypto-Prüfung (G4) und Rechtsfreigabe (G5). | D-12 | Unabhängiges Audit ohne Acta-Code besteht; Messwerte statt Zielwerte belegt. |

---

## Anhang A – Testvektoren und Kanonische Formate

### A.1 Kanonisches JSON des Audit-Berichts (`ErasureReport` v1)

Der Bericht wird zur Signaturprüfung streng deterministisch nach RFC 8785 (JSON Canonicalization Scheme, JCS) in UTF-8 formatiert:

```json
{
  "assumptions": [
    "A-FLASH",
    "A-RAM",
    "A-ANCHOR",
    "A-CRYPTO",
    "A-EXT"
  ],
  "attestation_class": "A1",
  "classes_reached": {
    "C": true,
    "D": true,
    "G": true,
    "K": true,
    "P": true
  },
  "collateral": [],
  "erasure_id": "018f4a2b-8c3d-7e9a-b1c2-d3e4f5a6b7c8",
  "mode": "DeleteAll",
  "notary_key_id": "hsm-slot-4-key-1",
  "public_fingerprint": "ed25519:6fbc8a...",
  "residual_risks": [
    "Flash wear-leveling slack blocks retained without key (A-FLASH)"
  ],
  "scope_commitment": "blake3:9a8b7c...",
  "target_type": "Subject",
  "timestamps": {
    "completed_at": 1731632460,
    "gated_at": 1731632400,
    "key_destroyed_at": 1731632401,
    "requested_at": 1731632399
  },
  "witnesses": [
    {
      "layer": "Store",
      "matches": 0,
      "method": "ForensicRawScan",
      "scanned_bytes": 104857600
    },
    {
      "layer": "Vector",
      "matches": 0,
      "method": "TopologyAndNeedleScan",
      "scanned_nodes": 45000
    }
  ]
}
```

Die Signatur wird über `ctx/report/v1` $\parallel$ `kanonische_bytes_ohne_signature` gebildet und als `signature: "base64url:..."` angehängt.

---

## Anhang B – Matrix der Unterschiede zur Vorgängerfassung 2.0-E

| Architektur-Aspekt | Vorgängerfassung (2.0-E) | Fassung 3.0 Final (Normativ) | Begründung / Behobene Lücke |
|---|---|---|---|
| **Schrittfolge der Saga** | Nadelmenge-Extraktion (E1) vor dem Gate. | **Gate (E2) zuerst**, Nadelstichprobe (E2a) nachgelagert und gedeckelt. | Beseitigt $O(\text{Datenmenge})$-Latenz vor dem Gate; garantiert $O(1)$ bis `Gated` (F-1). |
| **Klartext im RAM vs. Key** | Schlüsselvernichtung (E3) ohne Schicht-Bestätigung. | **Pflicht-Ack von `on_gate` (E2b)** als zwingende Vorbedingung für E3. | Verhindert das Zeitfenster „Schlüssel vernichtet, aber Klartext-Vektoren im RAM lesbar“ (F-2). |
| **Container-Verschlüsselung** | Nicht normiert; Blockverschlüsselung möglich. | **Invariante I-17:** Strikt ein Umschlag je Record; niemals Multi-Record-Blöcke. | Ermöglicht gefilterte SST-Rewrites über Chiffretext nach Schlüsselvernichtung (F-2). |
| **Lineage-Modell** | Uneinheitliche Tag-Listen und XOR-Filter (`tag_set`). | **Monotone `doc_ord: u32` mit Roaring-Bitmaps** (`source_set`, `ErasedOrds`). | Eliminiert Falsch-Positive; $O(1)$ Mengenoperationen über alle Indizes (F-3). |
| **Schlüsselraum KeyV2** | Schlüsselfeld `cluster` undefiniert. | **`cluster = subject_tag`** verbindlich festgelegt. | Schafft physische Datenlokalität; erlaubt Datei-Drop ganzer SSTables (F-4). |
| **Ableitungs-Vektoren** | Segmente unter gemeinsamem `ArtKey` sofort vernichtet. | **Standardmodus `Rebuild`** mit maskiertem Abfragefenster; `Sealed` nur opt-in. | Verhindert massiven Verfügbarkeitsausfall für fremde Betroffene bei Vektorlöschung (F-5). |
| **Audit-Protokolle** | Widerrufs-Log speicherte Klartext-IDs. | **Keyed Commitments** unter epochenbasiertem $K_{\text{commit}}$ und rotiertem $K_{\text{scope}}$. | Behebt Linkability-Risiko und schließt das Lösch-Paradoxon (F-6, V-15). |
| **Tresor-Grabstein** | SPEC forderte „`tag` bleibt zur Sondierung“. | **Code bestätigt (V-11):** `tag` und `payload` werden vollständig genullt. | Verhindert, dass das Tag als permanentes Pseudonym überdauert. |
| **Tresor-Maskierung** | Payloads T/C/S/D im Klartext auf Platte. | **Payloads zwingend unter Epochenschlüssel maskiert; Epochenrotation.** | Verhindert Wiederbelebung gelöschter Daten aus alten Tresor-Sicherungen (V-04). |
| **Klassifizierung L/C/P/E** | Unklares Verhältnis zwischen L/C/P/E und G/K/P/D/C. | **Kanonisches Mapping (Abschnitt 8.2)**; Klasse E = G+K+P+D+C erreicht. | Schafft einheitliches rechtlich-technisches Berichtsvokabular. |
| **Rechtliche Einordnung** | Kapitel 13 und 14 fehlten vollständig. | **Vollständige Integration** von Rechtsprechung, EDPB/DSK-Praxis und Betreibergrenzen. | Erfüllt das Vollständigkeits-Gate G-DOC; klärt UWG-Risiko und § 35 BDSG. |

---

## Anhang C – Quellenverzeichnis (ausschließlich docs/)

Dieses Dokument wurde unter strenger Exklusion von `AGENTS.md` und Crate-spezifischen `SPEC.md` ausschließlich aus den folgenden verifizierten Dokumenten synthetisiert:
1. `docs/Acta_Architekturanalyse_Loeschung.md` (Principal-Review zu HEAD `bbfad85`, Befunde V-01..V-12, Architektur-Findings F-1..F-8, Konstruktionsprinzipien P1..P10, Roadmap R0..R7).
2. `docs/dsgvo/Crypto-Shredding_RAG_Rechts-und-Architektursynthese.md` (Umfassende Rechts- und Architektursynthese, SDM Baustein 60, EDPB/DSK-Analysen).
3. `docs/dsgvo/Analyse der Rechtslage zum Löschen personenbezogener Daten in RAG-Systemen.md` & `docs/dsgvo/Analyse der Rechtslage zum Löschen personenbezogener Daten.md`.
4. `docs/dsgvo/Systemische Rechts- und Architektur-Analyse zur Evaluation der Schlüsselvernichtung.md`.
5. `docs/dsgvo/Löschen personenbezogener Daten in RAG-Systemen durch Schlüsselvernichtung (Cryp.md`.
6. `docs/SPEC.md` (Fassung 2.0-E) und `docs/SPEC(13).md` (Fassung 3.0 Entwurf).
