# Contextra — Gesamtspezifikation (normativ)

**Version:** 3.0
**Datum:** 19. November 2026
**Status:** FINAL (Soll-Zustand). Einzige Quelle der Wahrheit für Architektur, Löschkette, Schlüsselverwaltung und Nachweis.
**Ersetzt:** SPEC v2.0 (18.11.2026) in den Abschnitten §3.2, §4, §5, §6, §7, §9, §12, §13.8, §14 (Löschanteile), §16 und §20.1 (siehe §23).
**Geltungsbereich:** Gesamter Workspace `contextra`.

> **Rechtshinweis.** Diese Spezifikation ist eine technische Festlegung. Sie ist keine Rechtsberatung und keine Garantie einer Rechtswirkung. Software kann den technischen Zustand herstellen, den die vertretenen Rechtsauffassungen verlangen, und diesen Zustand belegen. Die rechtliche Bewertung trägt der Verantwortliche. Offene rechtliche Punkte stehen in §23 (P-xx) und müssen fachanwaltlich geklärt sein, bevor eine Außenaussage erfolgt.

---

## 0 Geltung und Rangfolge

**Rangfolge:**
1. Diese Gesamtspezifikation (`docs/SPEC.md`).
2. Crate-Spezifikationen (`docs/crates/*.md`), normativ pro Crate. Widersprechen sie dieser SPEC, gilt die SPEC; der Widerspruch ist als Befund zu melden.
3. Crate-lokale `README.md` und `AGENTS.md`: informativ bzw. Entwicklerrichtlinie.

Die früheren Löschkonzepte (Neukonzeption „Erasure by Design“, Implementierungsspezifikation „Erasure Core auf eigenem LSM-Tree“, Löschkonzept v3.1, Konsolidierungsfassung) sind mit dieser Fassung **vollständig ersetzt**. Ihre Entscheidungen sind, soweit übernommen, in §22 mit Herkunft ausgewiesen; abgelehnte Varianten stehen dort mit Begründung.

**Schlüsselwörter** MUSS, DARF NICHT, SOLL, DARF nach RFC 2119.

**Lesemarken in den Crate-Specs:** **[V]** am Quelltext verifiziert (Stand `deeddfd`, 19.11.2026), **[E]** Entwurfsentscheidung, **[S]** Schätzung/Zielwert ohne Messung, **[P]** offener Rechtspunkt.

---

## 1 Produktdefinition

### 1.1 Was Contextra ist

Contextra ist eine **souveräne, offline betreibbare Such- und Wissens-Engine** für Unternehmen und Behörden:

1. **Hybrid-Suche** (Vektor + Volltext + Graph) über lokale Bestände.
2. **Versionierung und Zeitreise** über Daten und Regeln.
3. **Löschung mit gestuftem, gemessenem Nachweis** (§5, §12, §13).
4. **Souveränität**: keine Telemetrie, kein Fernzugriff, lokale Schlüssel.

Contextra ist eine **eingebettete Rust-Bibliothek**. Primäre Schnittstelle: Rust-API. Sekundär: MCP über stdio.

### 1.2 Nicht-Ziele (dauerhaft)

- Keine automatisierte Entscheidung über natürliche Personen (Art. 22 DSGVO).
- Kein Training oder Fine-Tuning mit personenbezogenen Daten.
- Kein Dokumentenmanagementsystem.
- Keine Compliance-Aussage ohne gemessene Stufe (INV-7). „DSGVO-konform“ ist eine Bewertung des Verantwortlichen, keine Eigenschaft der Software.
- Keine Löschung in externen Backups, Kopien außerhalb von Contextra, Auftragsverarbeitern oder Modellanbietern. Diese werden deklariert (§17.3), nicht bereinigt.
- Keine Skalierungsbehauptung ohne belegten Bench-Lauf (INV-7).

---

## 2 Architektur

### 2.1 Workspace

Der Workspace umfasst **23 Workspace-Mitglieder** (`Cargo.toml` `members`) in vier Ebenen. Die Anzahl ist Teil des Gates §21 (`layering`).

| Ebene | Crates |
|---|---|
| **E0 — Fundament & Ports** | `ids`, `env`, `sys`, `durable-fs`, `simd`, `types`, `ports`, `privacy`, `erasure` |
| **E1 — Domänenbausteine** | `store`, `vector`, `text`, `graph`, `rank`, `mvcc`, `crypto`, `checkpoint` |
| **E2 — Integration** | `db`, `engine` (deprecated Re-Export), `contextra` |
| **E3 — Endpunkte** | `mcp`, `testkit`, `bench` |

**Entfernt, nicht reaktivieren ohne Gate G-REACTIVATE:** `contextra-cognition`, `contextra-agent`, `contextra-adapt`, `contextra-wire`, `contextra-py`, `contextra-prompt-guard`, `contextra-ingest`, `contextra-infer-onnx`, `contextra-infer-ollama`, `contextra-infer-candle`, `contextra-sandbox`, `contextra-router`, `contextra-kv-cache`. `contextra-vault` bleibt dormant in `exclude`; seine Löschfunktion geht in `erasure` und `crypto` auf (§2.5).

### 2.2 Abhängigkeitsregeln (R1–R6)

| Regel | Inhalt |
|---|---|
| **R1** | Kanten zeigen ausschließlich abwärts (E3→E2→E1→E0). Keine optionalen Abhängigkeiten, die eine Aufwärtskante erzeugen, auch nicht über Feature-Unifikation. |
| **R2** | Dev-Abhängigkeiten auf höhere Ebenen sind verboten. `contextra-testkit` (E3) darf nie in `[dev-dependencies]` einer E0- oder E1-Crate stehen. |
| **R3** | `default = []` in allen Crates. Löschpfade haben keine Feature-Schalter. |
| **R4** | Ein Konzept hat genau einen Besitzer (§2.5). |
| **R5** | `#![forbid(unsafe_code)]` außer `sys` und `simd`. |
| **R6** | Kein `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!` in Nicht-Test-Code. |

**Löschrelevante Folgerung aus R1:** Die Koordination der Löschkette (`erasure`) liegt auf E0. Sie kennt Schichten nur über die Trait-Objekte `ErasureLayer`, `KeyVault`, `ErasureJournal` und `ReadGate`, die sie selbst definiert. Die Implementierungen liegen in E1 (`store`, `crypto`, `vector`, `text`, `graph`, `checkpoint`, `mvcc`).

### 2.3 Architekturdiagramm

```
┌──────────────────────── E3: Endpunkte ──────────────────────┐
│  mcp            testkit (Erasure Lab)        bench           │
└──────────────────────────┬──────────────────────────────────┘
┌──────────────────────── E2: Integration ────────────────────┐
│  contextra ──► db (Orchestrierung, Lineage, SubjectIndex)   │
└──────────────────────────┬──────────────────────────────────┘
┌──────────────────────── E1: Domäne ─────────────────────────┐
│  store (LSM, Purge-Compaction)  vector  text  graph  rank   │
│  mvcc   crypto (Schlüssel, KeyTable, KeyAnchor)  checkpoint │
└──────────────────────────┬──────────────────────────────────┘
┌──────────────────────── E0: Fundament & Ports ──────────────┐
│  ids  env  sys  durable-fs (Vfs)  simd  types  ports        │
│  privacy  erasure (Saga, ReadGate, Ports, Messung, Bericht) │
└─────────────────────────────────────────────────────────────┘
```

### 2.4 Typbesitzer (R4, kanonisch)

| Typ / Port | Besitzer | Anmerkung |
|---|---|---|
| `DocId`, `SubjectRef`, `PersonalRef`, `TenantId`, `CollectionId`, `ErasureId`, `GateId`, `Seq`, `Timestamp` | `contextra-ids` | `DocId` ist keyed (§4.6) |
| `TenantScoped<T>`, `TenantScopeViolation`, `IdError` | `contextra-ids` | |
| `Clock`, `SystemClock`, `SeededRng`, `MetricsSink` | `contextra-env` | |
| **`Vfs`, `VfsFile`, `write_atomic`, Handle-Poisoning, `scrub_if_safe`** | **`contextra-durable-fs`** | **Einziger Besitzer** (env-Spec D-04). Heute doppelt in `ports`, `erasure::types_local`, `store::io_seam` [V]; Auflösung in WP L6 |
| `DataClass`, `Validated<T>`, `EntityRecognizer` | `contextra-privacy` | |
| `ContextraError`, `EraseTarget`, `ErasureScope`, `ClassSet`, `ResidualRisk`, `AssumptionCode` | `contextra-types` | Reine Datentypen, ohne I/O |
| `SagaState`, `RetentionAttestation`, `ReadGate`, `GateDecision`, `RangeGate`, `ErasureLayer`, `LayerId`, `LayerScope`, `PurgeCx`, `PurgeOutcome`, `Scanner<S>`, `LayerWitness`, `ErasureReport`, `ErasureJournal`, `KeyVault`, `KeyAnchor`, `ReportSink`, `ErasedSetMirror`, `PositiveControl` | `contextra-erasure` | Ports und Koordinator (E0) |
| `CommitBarrier`, `PurgeBarrier` | `contextra-ports` | |
| `DocDek`, `SubjectKey`, `CollectionKek`, `TenantKek`, `MasterKey`, `SecretKey`, `KeyTable`, `KeyCommitKey` (`K_ref`) | `contextra-crypto` | `KeyVault` und `KeyAnchor` sind Ports in `erasure`, Implementierung in `crypto` |
| `LineageRef`, `DerivedRef`, `LineageIndex`, `SubjectIndex` | `contextra-db` | |
| `ErasureLayer`-Implementierungen | `store`, `text`, `vector`, `graph`, `checkpoint`, `mvcc`, `crypto` | je Layer genau eine |

**Absolutregel:** Jeder dieser Typen ist im Workspace nur einmal definiert. Re-Exporte sind erlaubt. Der Gate `ownership` (§21) prüft das per `cargo metadata` und Quelltextsuche.

### 2.5 Workspace-Lints

```toml
[workspace.lints.rust]
unsafe_code = "forbid"

[workspace.lints.clippy]
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
todo = "deny"
unimplemented = "deny"
await_holding_lock = "deny"
indexing_slicing = "deny"
```

Kein Crate darf diese Lints lokal per `allow` überschreiben. Ausnahmen bei `unsafe` in `sys`/`simd` stehen an den Blöcken als `#[allow]` mit `// SAFETY:`-Kommentar.

---

## 3 Datenklassifikation und Löscheinheiten

### 3.1 Datenklassen

| Klasse | Name | Verschlüsselung | Löschung |
|---|---|---|---|
| **K** | Unternehmenswissen | `DocDek`, gewrappt unter `K_coll[e]` | Nur Collection-Löschung (oder Dokument-Löschung) |
| **P** | Personenbezogene Daten | `DocDek`, gewrappt unter `K_subj[coll, subject]` | Betroffenen-Löschung O(1) in der Kryptografie, Dokument-Löschung einzeln |
| **D** | Abgeleitete Daten | `ArtKey` bzw. `DocDek` der Ableitung, mit `LineageRef` | Kaskadiert über `LineageIndex` (INV-DERIVED-1) |
| **E** | Löschmaterial (Journal, Ledger, Berichte, `ErasedSet`) | `K_saga`, `K_eng`, `K_commit` | Nach Ablauf der jeweiligen Frist (§14, §16) |

### 3.2 Löscheinheit

Eine **Löscheinheit** ist die kleinste Menge von Records, die ohne Kollateralverlust gelöscht werden kann.

- **Standard:** Ein Dokument mit genau einem Betroffenen oder ohne Personenbezug ist eine Löscheinheit mit eigenem `DocDek`.
- **Mehrfachbezug:** Ein Dokument mit mehreren Betroffenen wird in Segmente zerlegt. Jedes Segment trägt die Menge seiner Betroffenen. Segmente mit identischer Menge teilen **keinen** Schlüssel; jedes Segment hat einen eigenen `DocDek` (§3.4).

### 3.3 Betroffenen-Referenzen

- `SubjectRef` ist eine 128-Bit-Zufalls-ID (`[u8; 16]`, `ZeroizeOnDrop`, kein `Clone`, `OsRng::try_fill_bytes`). Die Zuordnung zu einer natürlichen Person liegt außerhalb des Systems.
- Im Speicher und in allen Persistenzformaten erscheint `SubjectRef` **nie im Klartext**. Verwendet wird ausschließlich `subject_key_id = BLAKE3-keyed(K_ref[tenant, epoche], SubjectRef)` (§16).
- K-Dokumente tragen keine `SubjectRef` (INV-K-NOPERS). Personenbezug in K-Dokumenten läuft über `PersonalRef`.
- D-Daten tragen verpflichtend `LineageRef` (INV-LINEAGE).

### 3.4 Mixed-Policy (Mehrfachbezug)

Je Collection konfigurierbar, Standard `BlockAll`:

| Policy | Verhalten | Verantwortung |
|---|---|---|
| `BlockAll` (Standard, fail-closed) | Segment mit Betroffenem A und anderen Betroffenen wird gesperrt (Klasse L), Zustand `Partial{ReviewRequired}`, Frist 24 h Alarm, 14 Tage Eskalation (§15) | Betreiber entscheidet |
| `RedactAndRekey` | Betreiber liefert redigierten Text. Neuer `DocDek`, Neuindizierung, alter `DocDek` wird vernichtet | Betreiber |
| `DeleteAll` | Gesamtes Segment wird gelöscht; Kollateral beim Mitbetroffenen liegt beim Betreiber | Betreiber |

Contextra entscheidet nie selbst, ob ein Mixed-Segment nach Entfernung von A noch personenbezogen ist. Das ist eine Rechtsfrage nach Art. 17 Abs. 3 (P-02).

---

## 4 Schlüsselhierarchie, Custody und Verschlüsselung

### 4.1 Hierarchie

```
Wurzel (Custody-Slot, §4.3)
  │  MasterKey = Argon2id(Passphrase, Salt; m=256 MiB, t=4, p=1)   [K1]
  │  bzw. Wurzel im KeyAnchor                                     [K2/K3]
  │
  ├─ HKDF-SHA-256 ─► K_eng        Struktur- und Blockintegrität (keine Nutzdaten)
  ├─ HKDF-SHA-256 ─► K_idx        Blind-Index-Schlüssel (Terme, §8.2)
  ├─ HKDF-SHA-256 ─► K_scope      BLAKE3-Schlüssel für DocIds und scope_id (§4.6)
  ├─ HKDF-SHA-256 ─► K_sc/*       Zweckschlüssel der Sidecar-Ketten
  │
  ├─ Zufall, gewrappt ─► K_tenant[t]                         (je Mandant)
  │     └─ Zufall, gewrappt ─► K_coll[t, c, e]               (je Collection und Epoche e)
  │           ├─ Zufall, gewrappt ─► DocDek[d]               (K-Dokumente, D-Ableitungen)
  │           └─ Zufall, gewrappt ─► K_subj[t, c, subj]      (je Betroffenem je Collection)
  │                  └─ Zufall, gewrappt ─► DocDek[d]        (P-Dokumente)
  │
  ├─ Zufall, gewrappt ─► K_ref[t, e_ref]                     (Referenz-Hashes, §16)
  ├─ Zufall, gewrappt ─► ArtKey[a]                           (Index-Artefakte, §8)
  └─ Zufall, gewrappt ─► K_saga[erasure_id]                  (Journal-Scope, §10)
```

**Regel H1 (kein HKDF für Löschschlüssel):** Alle Schlüssel, die vernichtet werden können müssen (`K_tenant`, `K_coll`, `K_subj`, `DocDek`, `ArtKey`, `K_ref`, `K_saga`, `KEK_table`), werden **zufällig erzeugt und gewrappt** gespeichert. Abgeleitete Schlüssel sind jederzeit neu berechenbar und lassen sich deshalb nicht vernichten. HKDF ist nur für Schlüssel zulässig, die nie gelöscht, sondern rotiert werden (`K_eng`, `K_idx`, `K_scope`, `K_sc/*`). Diese Festlegung ersetzt die HKDF-Ableitung von `CollectionKEK` und `SubjectKey` aus SPEC v2.0 (§4.1, ER-E01).

**Regel H2 (Wrap-Kontext):** Jeder Wrap ist AEAD mit AD = `tenant ‖ collection ‖ KeyKind ‖ key_id ‖ kek_epoch ‖ fmt_ver`. Vertauschen von Wraps zwischen Schlüsseln oder Mandanten scheitert.

### 4.2 Algorithmen

| Funktion | Algorithmus | Crate |
|---|---|---|
| AEAD (Records, Wraps, Blöcke) | AES-256-GCM-SIV (RFC 8452), Nonce 96 Bit zufällig, Tag 128 Bit | `crypto` |
| Schlüsselableitung (nur rotierbare Schlüssel) | HKDF-SHA-256 (RFC 5869) | `crypto` |
| Passphrase | Argon2id, m = 256 MiB, t = 4, p = 1; Parameter im versionierten KDF-Header | `crypto` |
| Referenz-Hash, Scope-Commitment, Duplikaterkennung | BLAKE3-keyed (einheitlich, §16) | `crypto` |
| DocId, scope_id | BLAKE3-keyed, 128 Bit (§4.6) | `ids` / `crypto` |
| Signatur (Berichte, Ledger) | Ed25519 (`ProofSigner`-Port) | `erasure` (Port), `crypto` (Implementierung) |
| Zufall | `OsRng::try_fill_bytes`; Fehler ergibt `Err`, nie einen Chiffretext | alle |

**Hinweis zu Hashes:** SPEC v2.0 und die Konsolidierungsfassung nannten für Referenzschlüssel HMAC-SHA-256, die Implementierungsspezifikation BLAKE3-keyed. Festgelegt ist BLAKE3-keyed (ER-E08), weil BLAKE3 bereits Abhängigkeit ist und die Latenz im Lesepfad geringer ist. Beide sind PRFs. Der Wechsel ist im Bericht als `CommitmentAlgo` markiert, damit er ohne Formatbruch möglich bleibt (INV-1).

### 4.3 Custody-Stufen (`KeyAnchor`)

Die Wurzel liegt in einem `KeyAnchor`-Port (Definition in `contextra-erasure`, §9.4). Die Stufen:

| Stufe | Wurzelverwahrung | Maximale Klasse | Restannahme |
|---|---|---|---|
| **K0** | unverschlüsselt, nur Entwicklung | L, P | — (`doctor` rot) |
| **K1** | Argon2id aus Passphrase, Software-Keystore (`FileOnly`) | C1 | **A-FLASH**, **KeyStoreRemanence** |
| **K2** | Siegelschlüssel in manipulationsresistentem, nicht exportierbarem Modul (TPM 2.0, Secure Element, HSM) | C2 nach bestandener Konformitätssuite | Eignung je Plattform: Spike S-K2 |
| **K3** | externer KMS/HSM mit Vernichtungsattest und Wartefenster | C3 mit Anbieterattest (P-18) | Anbieterattest |

- **K2 und K3** sind als Port vorgesehen, aber **nicht ausgeliefert**, solange die Konformitätssuite nicht besteht. Bis dahin liefert der Port `CapabilityUnsupported` und nie einen Stub, der Erfolg meldet (REQ-DEL-N14, INV-WIRING-2). Das ändert die Entscheidung E27 aus v2.0 teilweise (ER-E09).
- **Hinweis:** Hardware-Effaceable-Konzepte (z. B. Apple Effaceable Storage) löschen einen Volumenschlüssel schnell. Sie sind kein Mechanismus für beliebige Einzelschlüssel von Anwendungen. Eine Plattform gilt nur dann als K2, wenn die Konformitätssuite nachweist, dass Schlüsselmaterial nach Vernichtung nicht aus Resten rekonstruierbar ist.

### 4.4 KEK-Epochen und Epoch Seal

**Problem.** Wird nur der `DocDek` vernichtet, bleibt sein gewrappter Blob als Rest auf dem Flash. Wer `K_coll` und die Wurzel besitzt, entwrappt ihn. Der Wrap-Schlüssel selbst ist ein Löschziel (NIST SP 800-88 r2 nennt Wrapping-Schlüssel ausdrücklich).

**Lösung: Epoch Seal.** Ein Epoch Seal für eine Collection:

1. erzeugt `K_coll[t, c, e+1]` (zufällig), gewrappt unter `K_tenant`;
2. wrappt alle lebenden `DocDek`s, `K_subj`s und `ArtKey`s der Collection unter `K_coll[e+1]` neu (Re-Wrap, **kein** Datenrewrite, Kosten O(Anzahl Schlüssel));
3. schreibt die `KeyTable` atomar neu (§4.5);
4. vernichtet `K_coll[e]` und dessen Wrap-Kopie im `KeyAnchor`;
5. Custody K1 mit Option `rotate_passphrase_on_seal = true`: Passphrasen-Rotation, sodass die alte Wurzel nicht mehr aus der alten Passphrase ableitbar ist (verkürzt A-FLASH zeitlich);
6. Custody K2/K3: `rotate()` und `destroy()` am Slot bzw. KMS.

**Auslöser:** zeitgesteuert (`kek_epoch_len`, Startwert 30 Tage), auf Anforderung (`seal-now`), und automatisch, wenn seit der letzten Vernichtung eine Löschung stattfand und `seal_after_deletion_max` (Startwert 7 Tage) erreicht ist. Löschungen derselben Collection und Epoche teilen sich einen Seal.

**Klassenwirkung:** C1–C3 gilt erst nach einem Epoch Seal, der die Löschung umfasst (INV-EPOCH-SEAL-1). Davor weist der Bericht `C*` (vorläufig) mit `residuals: [R-KEKEPOCH until]` aus.

**Crash-Verhalten:** Ein Absturz mitten im Seal lässt `K_coll[e]` und `K_coll[e+1]` koexistieren. Der Seal wird beim Start aus dem Journal fortgesetzt (§6.6).

### 4.5 KeyTable

- Die gewrappten Schlüssel (`DocDek`, `K_subj`, `ArtKey`, `K_coll`-Epochen) liegen in der `KeyTable` mit **Ersetzen-Semantik**: Keine Append-Logs für Schlüssel.
- Mutation: Neuschreiben der Segmentdatei als `tmp` → `fsync` → `rename` → `fsync(Verzeichnis)` über den Vfs-Port (`durable-fs`).
- Die Segmentdatei ist mit `KEK_table[g]` verschlüsselt. `KEK_table[g]` liegt **im `KeyAnchor`**, nicht auf dem Dateisystem. Nach jedem Replace wird die Vorgängergeneration im Anker vernichtet.
- Eintrag vernichten: Eintrag mit Nullen überschreiben, `fsync`, Eintrag entfernen, Segment atomar ersetzen. Speicherkopien werden `zeroize`d (INV-11).
- Das `KeyRegistry`-Sidecar führt nur Ereignisse (`created`, `destroyed`, `sealed`) mit Referenz-Hash, **ohne** Schlüsselmaterial und ohne Wrap-Blob (REQ-DEL-N5).
- Nach Vernichtung enthält die KeyTable keinen Eintrag mehr (REQ-DEL-N4).

### 4.6 DocId, scope_id und Kollisionsschutz

- **`DocId` = 128 Bit**, `BLAKE3-keyed(K_scope[t, c], user_key)`, abgeschnitten auf 128 Bit. Das Feature `docid-128` wird **Standard** (Abweichung von der heutigen 64-Bit-Unkeyed-Variante [V] `contextra-ids/src/lib.rs:216–227`, `DocId = BLAKE3(key)[..8]`).
- Begründung: Ungekeyte 64-Bit-IDs sind bei 10⁹ Dokumenten mit etwa 2,7 % Kollisionswahrscheinlichkeit behaftet (n²/2⁶⁵, Rechnung, keine Messung) und sind ohne Geheimnis aus der Nutzer-ID berechenbar (Pseudonym, nicht anonym).
- **Kollisionsschutz (Fail-closed):** Beim Insert wird ein vorhandenes Mapping `DocId → user_key` gegen den neuen `user_key` geprüft. Abweichung ⇒ `Err(DocIdCollision)`; kein Überschreiben, kein stilles Teilen.
- **Schlüssel im Store enthalten ausschließlich DocIds, nie Nutzer-IDs** (INV-KEY-OPAQUE). Damit entfällt Klartext-Metadatenleck über Schlüssel, soweit der Mandant nicht Teil des Schlüssels ist. Der Mandant ist kein personenbezogenes Datum (Annahme, P-07).
- Die Zuordnung `scope_id` für Löschziele ist `BLAKE3-keyed(K_scope, tenant ‖ collection ‖ user_key)` mit 128 Bit.

### 4.7 Record-Format

- **Padding:** Klartext wird nach dem Padmé-Schema aufgefüllt (Länge leakt höchstens O(log log M), Overhead ≤ 12 %). Mindestlänge `min_record_bytes` (Startwert 256).
- **Header:** `doc_key_id` und Sequenzbezug stehen **innerhalb** des unter `K_eng` verschlüsselten Blockbereichs, nicht im Klartext-Record.
- **Format:** `magic ‖ fmt_ver ‖ suite_id ‖ key_gen ‖ nonce(12) ‖ ct ‖ tag`. Jeder Record trägt Algorithmus- und Formatkennung (Krypto-Agilität, §17.6).

### 4.8 Krypto-Implementierungsregel

Kryptographische Primitive (AES-256-GCM-SIV, HKDF, Argon2id, Ed25519, BLAKE3-keyed als Commitment) sind **ausschließlich** in `contextra-crypto` implementiert (R4). Andere Crates listen `aes-gcm-siv`, `sha2`, `hmac`, `hkdf`, `argon2` nicht als direkte Abhängigkeit. Bestehende Duplikate in `contextra-erasure` (`crypto_local.rs`) und `contextra-store` werden in WP L6 entfernt.

---

## 5 Speicherlayout und Schichten

### 5.1 Zwei Ebenen, drei Spuren

- **Ebene A, Klartext-Strukturen.** Memtable, Index-Arena (fp32 und SQ8), Postings im Speicher, Caches (Block, Embedding, RRF, Antwort, Abfrage, Kontext), Surrogat-Vault. Diese Strukturen brauchen Klartext für Suche und werden physisch bereinigt (Nullen, `zeroize`, Neuschreiben). Gemessen wird per Scanner (§12.2).
- **Ebene B, persistente Reste.** SSTables, WAL, Blob-Segmente, Index-Artefakte, Snapshots, Checkpoints, Flash-Reste, Daten-Backups. Jeder persistierte Nutzwert liegt als AEAD-Chiffrat unter einem Schlüssel, der zu genau einer Löscheinheit gehört. Die Vernichtung dieses Schlüssels macht alle Kopien gleichzeitig unlesbar.
- **Spur Backup.** Daten-Backups enthalten keine Schlüssel. Schlüssel-Backups sind befristet (§14).

### 5.2 Schichtenkatalog (normativ)

Jede persistente Struktur ist genau einer Zeile zugeordnet (Gate `surface-coverage`, INV-SURFACE-1).

| Nr. | Ort | Inhalt mit Personenbezug | Mechanismus | Messung |
|---|---|---|---|---|
| S1 | Memtable (alle MVCC-Versionen) | Schlüssel (DocIds), Werte (Chiffrat) | Entzug der Einträge der Löscheinheit, Flush | Strukturscan + Canary/Selbsttest |
| S2 | WAL aktiv und versiegelt | Batch-Chiffrate | Rotation, Flush, Entfernen, Prüfung „Datei fehlt“ | Dateiliste + Rohscan |
| S3 | SSTable-Datenblöcke | Wert-Chiffrat unter `DocDek` | Schlüsselvernichtung + Purge-Compaction | Rohscan, Entschlüsselungsprobe |
| S4 | SSTable-Index, Bloom-Filter, Footer | DocIds im Klartext (keyed) | **Nur** Purge-Compaction | Rohscan nach DocId-Nadel |
| S5 | Manifest, Dateinamen | evtl. Bereichsgrenzen | Manifest-Austausch | Rohscan |
| S6 | MVCC-Versionen, SSI-Lesemengen | Schlüssel in S1/S3, RAM | Purge-Compaction über **alle** Versionen | Versionsscan je Sequenznummer |
| S7 | Blockcache | entschlüsselte Blöcke im RAM | Eviction und `zeroize` je ersetzter Datei | Heap-Scan |
| S8 | Textindex (Segmente) | blinde Terme, Slots in Postings | Segment-Neuschreiben mit neuem `ArtKey` (§8.2) | Segment-Scan |
| S9 | HNSW (fp32, SQ8, Arena, Snapshotdateien) | Vektor | Slot-Zeroing, Shadow-Rebuild (§8.1) | Slot-Scan, Heap-Scan |
| S10 | Graph, `__rel:` | Kanten, Entitäts-IDs | Eingangs- und Ausgangskanten entfernen; Neuaufbau | Kantenscan |
| S11 | Caches | Klartext | Register `Purgeable` (§8.3) | Registerprüfung |
| S12 | Checkpoints, Forks (Hardlink-Klone) | Hardlinks auf S3 | Policy `Rewrite` je Klon | Rohscan je Klon |
| S13 | Logs, Audit, Metriken, Traces | IDs in Feldern | Typ ohne Freitext, Lint, INV-12 | Log-Scan |
| S14 | KeyTable, KeyRegistry, Journal | gewrappte Schlüssel, Scope-Chiffrat | `KeyAnchor`-Replace, `K_saga`-Vernichtung bei `Completed` | Abwesenheitsprüfung |
| S15 | Exporte (`export`) | Klartext | Pflicht-Register (`RecipientLedger`, §15.2) | Dateiscan |
| S16 | Originaldateien (falls vorhanden) | Klartext | wie S3, falls Contextra Originale selbst hält | Rohscan |
| S17 | Surrogat-Vault | Ersatzwerte | wie S4 | Scan |
| S18 | Externe Sicherungen, Volume-Snapshots, Swap, Dumps | außerhalb | nicht adressierbar, deklariert (§17) | — |

### 5.3 Datenfluss-Inventar für Ebene B

S3 und S4 sind getrennt zu behandeln: **Chiffrat-Blöcke** (S3) werden durch Schlüsselvernichtung unlesbar. **Schlüssel im Klartext** (S4, S1, S6) werden nur durch **Purge-Compaction** entfernt, weil sie nicht unter einem `DocDek` stehen. Die Schlüssel tragen aber nur keyed DocIds (§4.6), nicht Nutzer-IDs.

---

## 6 Löschpfad

### 6.1 Zwei Takte

Die Löschung läuft in zwei Takten. Takt 1 macht die Löschung **sofort wirksam** (Klasse L). Takt 2 erreicht die Klassen P, C und M gebündelt im Hintergrund.

| Takt | Ausführung | Zielwert [S] | Ergebnis |
|---|---|---|---|
| **1** | synchron im Aufrufer (asynchrone API, Group-Commit-Journal) | p99 ≤ 5 ms, unabhängig von der Datenmenge | Klasse **L**, Ticket |
| **2** | Hintergrund-Worker, Epochen aus Zeitfenster oder Mengenschwelle | Klasse C ≤ 60 s nach Takt 1 (Zeitfenster-Standard 5 s) | Klassen P, C1–C3, Siegel 2–3 |
| **Hygiene** | terminiert, Hintergrund | ≤ `purge_rebuild_max_days` (14 Tage) | Klasse M, Siegel 4 |

### 6.2 Zustände

```
Requested ─► Frozen (L) ─► LogicallyDeleted ─► PlaintextPurged ─► PlaintextMeasured (P)
                                    │                                     │
                                    │                                     ▼
                                    │                             KeyDestroyed ─► CryptoMeasured (C*)
                                    │                                                     │
                                    │                                                     ▼
                                    │                                              EpochSealed (C1–C3)
                                    │                                                     │
                                    │                                                     ▼
                                    │                                   Hygiene ─► HygieneMeasured (M) ─► Reported ─► Completed
                                    │
Frozen ─► Restricted{basis, until} ─► (Release) ─► Requested        (Art. 18 / Art. 17 Abs. 3)
Jeder nicht-terminale Zustand ─► Pending{reason, attempt, next_retry_at}
Terminal: Completed, Refused, Lifted (INV-9)
```

Übergänge sind in `erasure::state::step` total kodiert. Ungültige Kombinationen liefern `TransitionError`. Jeder Übergang erscheint **nach** einem durablen Journal-Eintrag (fsync). Ohne Journal-Eintrag kein Zustandswechsel.

### 6.3 Takt 1 — Einstieg (Klasse L)

Eingang: `Eraser::erase(scope, on_retention)`.

1. **Validierung.** Mandantenbindung, Berechtigung, `ErasureScope`-Typ. Scope-Auflösung: `Document` → `DocId` → dichter `Slot`. `Subject` → `subject_key_id` → `SubjectSlot` (u32). `Collection` und `Tenant` → Epochenmarke.
2. **Aufbewahrungsprüfung.** `RetentionAttestation`: aktive Pflicht ⇒ bei `Reject` `Refused` (terminal); bei `Restrict` ⇒ `Restricted` (§6.7).
3. **Duplikat.** Keyed-BLAKE3-Duplikaterkennung über das Scope-Commitment. Laufende Saga ⇒ bestehendes Ticket zurück, kein zweiter Lauf.
4. **Scope-Chiffrat.** `ScopeDescriptor` wird unter `K_saga[erasure_id]` verschlüsselt. Der Journal-Eintrag enthält nur dieses Chiffrat und den Referenz-Hash.
5. **Group-Commit.** Gleichzeitig eintreffende Löschungen teilen sich einen `fdatasync` (Wartefenster 1–2 ms). Schlägt der Flush fehl, liefert Takt 1 `Err` zurück und setzt **keine** Sperre.
6. **Sperre publizieren.** Nach erfolgreichem `fsync`: `ErasedSet` aktualisieren, Bit in `LiveMask` setzen bzw. Subject-Bit bzw. Epochenmarke. Publikation mit `SeqCst`-Speicherordnung (Dekker-Muster). Ein Leser sieht eine gesetzte Sperre nur mit durablem Journal-Zustand.
7. **Ticket.** Rückgabe mit `ErasureTicket { id, state: Frozen }`. Siegel 1 wird in die `AuditChain` eingetragen.

Ab Schritt 6 verweigert das `ReadGate` jeden Lesezugriff auf den Scope (§7).

### 6.4 Takt 2 — Epochen (Klassen P, C)

Ein Worker sammelt Tickets. Eine Epoche schließt, wenn das Zeitfenster (`epoch_window`, Standard 5 s, Obergrenze `epoch_max` 60 s) oder die Mengenschwelle (Standard 1.024 Löschungen) erreicht ist. Reihenfolge innerhalb der Epoche:

1. **Vorbereitung.** Scope-Chiffrate aus dem Journal entschlüsseln (`K_saga` ist noch vorhanden). Für `Subject`: `SubjectIndex` lesen, Slot-Liste erzeugen.
2. **Ebene A (Klartext, Schichten S1, S6–S11).** Slots nullen (fp32 und SQ8), Postings-Iteratoren markieren, Caches invalidieren, Memtable-Einträge entziehen. Journal-Eintrag `PlaintextPurged` vor jedem Teilschritt.
3. **Messung A.** Pro Löschung O(1)-Prüfungen (§12.3). Ergebnis als `LayerWitness` mit Positivkontrolle.
4. **Pre-Destruction-Inventar.** Für jede betroffene `key_id`: Digest und Zähler aller noch vorhandenen Chiffratblöcke (Datei, Offset, Länge werden nur als Digest festgehalten, nie im Klartext). Dient in Schritt 6 dem Nachweis, dass jeder Block einem vernichteten Schlüssel zugeordnet ist.
5. **Journal-Intent (Regel J1).** `KeyDestroyed`-Intent mit der 96-Bit-Wrap-Nonce jedes zu vernichtenden Schlüssels wird **vor** der Vernichtung durabel geschrieben.
6. **Vernichtung.** `KeyVault::destroy_batch`: Einträge aus der `KeyTable` entfernen, Segmente atomar ersetzen, `KeyRegistry` protokolliert `destroyed`. Speicherkopien `zeroize`d. Idempotent.
7. **Messung C.** Nadelscan (§12.3), Entschlüsselungsprobe (muss `KeyAbsent` liefern), Lesepfad-Probe (muss `Erased` liefern). Ergebnis `C*`.
8. **Epoch Seal** (§4.4), sobald fällig oder per `seal-now`. Danach Klasse C1, C2 oder C3 je nach Custody (§12.1), Siegel 3.

**Idempotenz:** Ist ein Schlüssel nach protokolliertem Intent in der KeyTable bereits nicht mehr auffindbar, gilt die Vernichtung als vollzogen und Schritt 6 wird übersprungen.

**`DestroyKey` nach Messung A** ist Pflicht: Ebene-A-Arbeit kann noch Entschlüsselung brauchen (Rebuild-Hilfsdaten, Snapshot der Surrogate).

### 6.5 Hygiene (Klasse M)

Terminiert durch `purge_rebuild_max_days` (Startwert 14 Tage). Läuft im Hintergrund, ist aber nicht optional.

1. **Purge-Compaction** (§10.4) für alle SSTables und Blob-Segmente, die Schlüssel der Löscheinheit enthalten (S3, S4, S6). Atomarer Austausch im Manifest, PendingDelete-Journal für alte Dateien.
2. **WAL-Entfernung mit Pflichtprüfung.** Aktive WAL rotieren, Memtables flushen, versiegelte WALs entfernen, per Vfs prüfen, dass sie nicht mehr existieren. Bleibt eine Datei, Zustand `Pending` mit Retry (N9).
3. **Index-Neuaufbau** (§8.1, §8.2): HNSW-Generation `g+1` aus lebenden Slots, atomarer Umschalt-Commit, danach `ArtKey` der Generation `g` vernichten. Text-Segmente wie in §8.2.
4. **Checkpoint-Sweep** (S12). Policy `Rewrite` ist Standard. Klone werden einzeln gescannt, weil Hardlinks durch das Neuschreiben der Quelle nicht mitbereinigt werden.
5. **Lineage-Kaskade** abgeschlossen (INV-DERIVED-1).
6. **Messung M.** Alle Schichten erneut mit Positivkontrolle. Siegel 4.

### 6.6 Crash-Verhalten und Recovery

`recover()` läuft **vor** Freigabe des Lesepfads (INV-RECOVER-1):

| Letzter Journal-Zustand | Fortsetzung |
|---|---|
| `Requested` | Takt 1 ab Schritt 4 neu (idempotent) |
| `Frozen` | Sperre aus Journal laden; Takt-2-Einreihung |
| `PlaintextPurged` | Ebene A fortsetzen; Scope-Chiffrat ist noch vorhanden |
| `PlaintextMeasured` | Vernichtung (Schritt 5) |
| `KeyDestroyed` mit Intent ohne Ausführungsnachweis | Vernichtung wiederholen; Idempotenz prüft Abwesenheit |
| `CryptoMeasured` | Epoch Seal fortsetzen |
| Mitten im Epoch Seal | `K_coll[e]` und `K_coll[e+1]` koexistieren; Re-Wrap fortsetzen |
| `Hygiene` | Hygiene fortsetzen, mit `Pending`-Retry |
| `Restricted` | Sperre laden, keine Vernichtung |
| `Completed`, `Refused`, `Lifted` | nur Gate laden |

Ein Absturz nach Vernichtung-Intent ist **nie** ein Anlass, Lesezugriff zu erlauben. Unklarer Zustand heißt Lesen verweigern (fail-closed).

### 6.7 Restricted (Art. 18) und Aufbewahrung

- **Restrict** setzt Klasse L, Zustand `Restricted{basis, until}`. Kein Purge, keine Schlüsselvernichtung, Lesezugriff nur mit Audit-Eintrag (T-DEL-13).
- Übergänge: `Restricted → Requested` (Freigabe für Löschung, dann Takt 1 erneut), `Restricted → Lifted` (Aufhebung, terminal).
- `Refused` (Aufbewahrungspflicht aktiv, `Reject`) ist terminal und ändert den Datenbestand nicht.
- Eine Ausweisung als „gelöscht“ ist für `Restricted` und `Refused` ausgeschlossen (Display-Typ erzwingt „eingeschränkt“ bzw. „nicht gelöscht“).

### 6.8 Karenz und Rücknahme

- `erasure_grace` (Startwert 0 h, Obergrenze 72 h, im Regulated-Tier Pflichtwert ohne Default, REQ-DEL-N26) legt die Wartezeit zwischen Schritt 4 (Messung A) und Schritt 5 (Vernichtung) fest.
- Während der Karenz gilt `legal_state = Eingeschränkt`. `Unfreeze` mit auditierter Begründung ist bis Schritt 5 möglich.
- Die Karenz ist kein Ersatz für die Pflicht zur unverzüglichen Löschung. Sie ist ein Irrtumsschutz.

---

## 7 ReadGate und Zeitreisen

### 7.1 Anforderungen

- **O(1), ohne I/O, ohne Blockierung.** Prüfung über `LiveMask` (Slot-Bit), `SubjectMask` (Betroffenen-Bit) und Epochenmarken.
- **Fail-closed.** Unklarer Zustand ⇒ `Erased`.
- **Vor Entschlüsselung.** Die Prüfung steht vor jedem Zugriff auf Chiffrat.
- **Gleiches Fehlerbild.** Gelöscht ist nicht von „nie vorhanden“ unterscheidbar.

### 7.2 Trait

```rust
pub trait ReadGate: Send + Sync {
    /// Synchron, O(1). Muss vor jeder Entschlüsselung aufgerufen werden.
    fn check(&self, slot: Slot, subject: SubjectSlot, coll: CollectionEpoch) -> GateDecision;
    /// Bereichsprüfung für Kompaktionen.
    fn check_range(&self, start: &[u8], end: &[u8]) -> RangeGate;
}
pub enum GateDecision { Open, Erased, Restricted }
pub enum RangeGate { AllOpen, ContainsErased, RestrictedNeverDiscard }
```

`check_range` nimmt nur Schlüssel entgegen, die bereits DocIds sind (§4.6). Nutzer-IDs liegen nie in Schlüsseln.

### 7.3 Zeitreisen: „Erase gewinnt“

- Das ReadGate wird **immer mit dem aktuellen Stand** geprüft, unabhängig von `as_of` (SPEC v2.0 §6.2, §9.2 Regel 3). Ein Zeitreise-Lesevorgang auf ein heute gelöschtes Ziel liefert `Erased`.
- Offene Snapshots **blockieren das Löschen nicht mehr** (SPEC v2.0 Schritt 4a „Snapshot-Widerruf“ entfällt, ER-E12). Lesende Snapshots erhalten `Erased` für gelöschte Löscheinheiten. Alte Versionen bleiben bis zur Purge-Compaction als Chiffrat liegen und sind ohne `DocDek` nicht lesbar.
- Der Preflight „kein Snapshot offen“ entfällt ersatzlos.

### 7.4 Vollständigkeit der Prüfpunkte (INV-GATE-COMPLETENESS)

| Lesepfad | Prüfstelle | Verhalten bei `Erased` |
|---|---|---|
| `get`, `get_at_seq`, `get_at_snapshot` | in `TenantScopedStorage`, vor Entschlüsselung | `Err(ContextraError::Erased)` |
| `scan`, `scan_prefix`, `scan_prefix_at` | je Eintrag, vor Ausgabe | still auslassen |
| Vektor-/Textsuche | Kandidatenebene vor Fusion | verwerfen, auf `k` auffüllen |
| Hybridsuche | vor Fusion und Ranking | ignorieren |
| Graph-Traversal | je Kante und je Zielknoten | Pfad abbrechen |
| Export | je Eintrag | auslassen |
| MCP-Tools | über darunterliegende Pfade | wie jeweiliger Pfad |
| Kompaktionen | `check_range` | `ContainsErased` ⇒ Bereich nicht ins Ergebnis |
| Checkpoints, Forks | über den Lesepfad des Klons | wie jeweiliger Pfad |

Die Prüfung in `TenantScopedStorage` ist die **einzige** Engstelle: `LsmStorage` ist nach außen `pub(crate)`. Ein Architekturtest verbietet andere Zugänge (T-DEL-11).

---

## 8 Index-Artefakte, Generationen und Restinformation

### 8.1 HNSW

- **Slots.** Jeder Vektor hat einen dichten `u32`-Slot pro Collection. Der Graph referenziert nur Slots.
- **Keine Umverdrahtung** (VETO-F02): Löschen entfernt keine Kanten im laufenden Graphen. Der tote Knoten bleibt als Routing-Knoten bis zum Neuaufbau.
- **Modi:**

| Modus | Verhalten | Standard |
|---|---|---|
| `Strict` | Toter Knoten wird in der Navigation übersprungen (Distanz ∞). Rebuild bei Tombstone-Anteil ≥ 5 % | Standard für sensible Collections |
| `Waypoint` | Durch den Mittelwert lebender Nachbarn ersetzt | **nicht freigegeben**; erst nach Recall-Messung und Rechtsbewertung (O-03, P-09) |

- **Shadow-Rebuild.** Hintergrund-Thread baut aus lebenden Slots einen neuen Index. Schreibvorgänge landen im Delta-Log. Umschalten per `Arc`-Swap. Kein langer Schreib-Mutex. Der Rebuild entfernt die Kanten zu toten Slots und gibt die Slots frei.
- **Slot-Wiederverwendung** erst nach Rebuild (INV-HNSW-SLOT-1).
- **Rebuild-Auslöser:** Tombstone-Anteil ≥ `tombstone_rebuild_ratio` (Standard 0,2), spätestens nach `purge_rebuild_max_days`, `seal-now`.
- **Persistenz.** Vektor-Payload im Snapshot liegt je Slot unter dem `DocDek` (Siegel im Snapshot). Der Graph-Snapshot liegt unter dem `ArtKey` seiner Generation. Die Persistenzformate sind bis Gate G1 (§21) als **ungeprüft** zu behandeln (R-HNSW-PERSIST).
- **Amortisierung** [S]: Bei Rebuild-Kosten `C(N)` und Schwelle ρ kostet eine Löschung rechnerisch `C(N)/(ρ·N)`, im Hintergrund. Der Beitrag steht im Bericht.

### 8.2 Volltext

- **Segmentiert und unveränderlich.** Neue Dokumente gehen in ein In-Memory-Segment, das periodisch als Segment geschrieben wird. Segmente werden gemergt.
- **Blinde Terme.** Postings und Wörterbuch verwenden `blind(term) = HMAC-SHA-256(K_idx, term)` bzw. BLAKE3-keyed, 16 Byte. Ein Segment enthält keine lesbaren Wörter. Präfix- und Wildcard-Suche sind im Erstprodukt **nicht** möglich (ER-E13, P-10).
- **Löschen = Maske.** Posting-Zeilen bleiben bis zum Merge unverändert, der Iterator überspringt tote Slots. Kein Forward-Index-Rewrite pro Dokument.
- **Merge.** Entfernt tote Einträge physisch und Terme mit `df == 0`. Läuft nach Tombstone-Schwelle oder Zeitplan. Ein Segment, das Schlüssel einer gelöschten Löscheinheit enthält, wird mit **neuem `ArtKey`** neu geschrieben; der alte `ArtKey` wird sofort vernichtet (INV-ARTKEY-1).
- **Statistik.** IDF, Dokumentlängen und Zähler liegen im Segment und werden mit dem Merge neu berechnet. Bis zum Merge zählen tote Dokumente mit. Der Fehler ist begrenzt und getestet (T-IDX-2).

### 8.3 Caches und Hilfsstrukturen

- Jede Struktur mit Slot-adressierten Kopien registriert sich im `Purgeable`-Register (Pflicht). Ein Test gleicht das Register gegen den Schichtenkatalog (§5.2) ab (`surface-coverage`).
- Caches verwenden als Schlüssel nur Slots oder DocIds, nie Klartext.
- Eingefrorene Schichten (Graph-Persistenz, Checkpoints) stehen als `OutOfScope` mit Begründung im Bericht, sofern sie nicht Teil des Löschpfads sind.

### 8.4 Graph

- Eingangs- und Ausgangskanten einer Entität werden bei Löschung des Ziels entfernt (S10). Entitäten, die nur von gelöschten Quellen stammen, werden entfernt.
- Der Graph-Snapshot wird in der Hygiene neu erzeugt (Generation `g+1`), danach `ArtKey(g)` vernichtet.

### 8.5 Restinformation

- **`R-GRAPH`** (Topologie bis Neuaufbau): Die Kanten enthalten nur Slot-Nummern. Die Topologie kann Nachbarschaften verraten. Der Rest ist im Bericht bis zum Neuaufbau ausgewiesen.
- **`R-IDF`**: Bis zum Merge zählen tote Dokumente mit.
- Beide sind zeitlich begrenzte Residuen mit Frist (§13.2).

### 8.6 Vergleich mit Marktsystemen

Qdrant, Milvus, Weaviate und Chroma behandeln Löschung als Wartung (Tombstones, Vacuum, Garbage Collection), ohne Frist, ohne Messung und ohne Schlüsselbezug. Contextra setzt die Frist durch und misst (Quelle: Sekundärquellen, nicht geprüft; siehe Crate-Spec `contextra-erasure` §11).

---

## 9 Schnittstellen der Löschkette

### 9.1 `ErasureLayer` (kanonisch in `contextra-erasure`)

```rust
pub trait ErasureLayer: Send + Sync {
    fn id(&self) -> LayerId;
    fn key_coverage(&self) -> KeyCoverage;      // Full | None | Partial(&'static str)
    fn resolve(&self, target: &ErasureScope) -> BoxFuture<'_, Result<LayerScope, ErasureError>>;
    fn on_gate(&self, scope: &LayerScope)    -> BoxFuture<'_, Result<(), ErasureError>>;
    fn purge(&self, scope: &LayerScope, cx: &PurgeCx) -> BoxFuture<'_, Result<PurgeOutcome, ErasureError>>;
    fn witness(&self, scope: &LayerScope, scanner: &Scanner<Armed>) -> BoxFuture<'_, Result<LayerWitness, ErasureError>>;
}
```

**Invarianten für Implementierer:**
1. `purge` ist idempotent. Zweiter Aufruf ist No-op (T-DEL-14).
2. `witness` ist rein lesend und wird vom Purger nicht gekannt (Trennung der Verantwortung).
3. Logs enthalten nie Klartext des Ziels (INV-12).
4. E1-Crates importieren `ErasureLayer` aus `contextra-erasure`.

**Schichtzuordnung (aktualisiert):**

| Layer | Crate | Takt / Phase |
|---|---|---|
| `StoreLayer` | `store` | Takt 2 Ebene A (Memtable), Hygiene (Purge-Compaction, WAL) |
| `TextLayer` | `text` | Takt 2 (Maske), Hygiene (Segment-Neuschreiben) |
| `VectorLayer` | `vector` | Takt 2 (Slots), Hygiene (Rebuild) |
| `GraphLayer` | `graph` | Takt 2 (Kanten), Hygiene (Neuaufbau) |
| `CryptoLayer` | `crypto` | Schlüsselvernichtung, Epoch Seal, KeyTable |
| `CheckpointLayer` | `checkpoint` | Hygiene (Sweep) |
| `CacheLayer` | `db`, `store`, `vector`, `text` | Takt 2 (Invalidierung über `Purgeable`) |

`SnapshotRevoker` (SPEC v2.0) entfällt (§7.3). Die Layer-Liste ist Teil des Gates `surface-coverage`.

### 9.2 `ErasureJournal`

```rust
pub trait ErasureJournal: Send + Sync {
    /// Ok erst nach durablem Flush (Group-Commit intern).
    fn append(&self, rec: JournalRecord) -> BoxFuture<'_, Result<JournalSeq, JournalError>>;
    fn replay(&self, from: JournalSeq) -> BoxFuture<'_, Result<Vec<JournalRecord>, JournalError>>;
}
```

### 9.3 `KeyVault` (Port, Implementierung in `crypto`)

```rust
pub trait KeyVault: Send + Sync {
    fn custody(&self) -> Custody;               // K0 | K1 | K2 | K3
    fn present(&self, key: &KeyRef) -> BoxFuture<'_, Result<bool, VaultError>>;
    fn destroy_batch(&self, keys: &[KeyRef]) -> BoxFuture<'_, Result<DestroyStats, VaultError>>; // idempotent
    fn rotate_epoch(&self, coll: CollectionId, next: EpochNo) -> BoxFuture<'_, Result<(), VaultError>>;
    fn verify_absent(&self, key: &KeyRef) -> BoxFuture<'_, Result<AbsenceEvidence, VaultError>>;
}
```

`verify_absent` prüft Abwesenheit im Schlüsselspeicher. Ein Erfolg ist **keine** Aussage über Flash-Reste (das ist Ebene B, §12.3).

### 9.4 `KeyAnchor` (Port, Implementierung in `crypto`)

```rust
pub trait KeyAnchor: Send + Sync {
    fn profile(&self) -> AnchorProfile;         // Hardware | OsKeyring | FileOnly | External
    fn put(&self, id: AnchorId, secret: &Zeroizing<[u8; 32]>) -> BoxFuture<'_, Result<(), AnchorError>>;
    fn get(&self, id: AnchorId) -> BoxFuture<'_, Result<Option<Zeroizing<[u8; 32]>>, AnchorError>>;
    fn destroy(&self, id: AnchorId) -> BoxFuture<'_, Result<DestroyEvidence, AnchorError>>; // idempotent
}
```

`DestroyEvidence` enthält Verfahren, Zeitpunkt (über `Clock`) und einen Beleg. Nur K1 liefert einen lokalen Beleg ohne Dritt-Attest; K3 liefert ein Anbieterattest.

### 9.5 `ErasedSetMirror`

```rust
pub trait ErasedSetMirror: Send + Sync {
    fn append(&self, entry: ErasedEntry) -> BoxFuture<'_, Result<(), MirrorError>>; // fsync am Ziel
    fn snapshot(&self) -> BoxFuture<'_, Result<Vec<ErasedEntry>, MirrorError>>;
}
```

Ziel außerhalb des Datenverzeichnisses (zweiter Datenträger, WORM, Fremdsystem). Im Regulated-Tier Pflicht (REQ-DEL-N19).

### 9.6 `PositiveControl` und `ProofSigner`

```rust
pub trait PositiveControl: Send + Sync {
    /// Liefert einen Kontrolldatensatz, den der Scanner finden muss.
    fn plant(&self, layer: LayerId) -> BoxFuture<'_, Result<ControlSample, ControlError>>;
    fn retract(&self, sample: ControlSample) -> BoxFuture<'_, Result<(), ControlError>>;
}

pub trait ProofSigner: Send + Sync {
    fn sign(&self, canonical_bytes: &[u8]) -> BoxFuture<'_, Result<Ed25519Signature, SignError>>;
    fn public_key(&self) -> Ed25519PublicKey;
}
```

Die Positivkontrolle ist in §12.2 festgelegt. `ProofSigner` ist im Gegensatz zu `KmsProvider` aus v2.0 ein Port ohne Löschbezug.

### 9.7 `ReportSink`

```rust
pub trait ReportSink: Send + Sync {
    /// Ok erst nach durablem Schreiben in das Ledger außerhalb des Datenverzeichnisses.
    fn publish(&self, report: &SignedErasureReport) -> BoxFuture<'_, Result<LedgerIndex, SinkError>>;
}
```

---

## 10 Journal, Barrieren und Nebenläufigkeit

### 10.1 Journal

- **Eigene Datei** über den Vfs-Port, nicht im LSM-WAL (keine Rekursion, eigener Lebenszyklus, ER-E15).
- **Eintrag:** `erasure_id (128 Bit) ‖ state ‖ attempt ‖ Zeit (Clock) ‖ payload ‖ CRC/Hash-Kette`.
- **Scope** als `AEAD(K_saga[erasure_id], ScopeDescriptor)`. `K_saga` liegt im `KeyAnchor` und wird bei `Completed` vernichtet. Danach enthält das Journal **keinen** Personenbezug mehr (Lösch-Paradoxon).
- **Torn-Tail:** Ein abgerissener letzter Eintrag wird beim Start verworfen. Vollständige Einträge werden nicht verworfen (T-JRN-1).
- **Idempotenz:** Jeder Übergang ist wiederholbar.

### 10.2 Group-Commit

- Wartefenster 1–2 ms. Ein `fdatasync` deckt alle Einträge im Fenster ab.
- Flush-Fehler ⇒ alle wartenden Aufrufer erhalten `Err`. Keine Sperre wird gesetzt.
- Der Group-Commit läuft auf einem dedizierten Thread. Die Kette wartet async über ein Oneshot-Signal. `block_on` ist in den Pfaden verboten (INV-ASYNC-1).

### 10.3 Barrieren

| Barriere | Zweck | Umsetzung |
|---|---|---|
| `CommitBarrier` | Trennt Schreibpfade vom Sperr-Commit | `shared()` für Writer, `exclusive()` nur für die Publikation der Sperre in Takt 1; Haltezeit ≤ 1 ms |
| Key-Lock je Löscheinheit | serialisiert `remember`/Update gegen Löschung desselben Ziels | Lock wird **über Takt 2 bis einschließlich Schritt 5** gehalten (SPEC v2.0 Schritt 2 bestätigt). Wartende Writer erhalten `Pending` |
| `PurgeBarrier` | Exklusivität der Purge-Compaction gegen reguläre Compaction | `compaction()` für reguläre Kompaktion, `purge()` für Purge-Compaction; wartende Kompaktionen werden über `purge_waiting()` gewarnt |

### 10.4 Purge-Compaction

Neues LSM-Primitiv `LsmStorage::erase_keys(matcher, ctx)`. Es ist der einzige Weg, Schlüssel aus Ebene B zu entfernen, die nicht unter einem `DocDek` stehen.

1. **Barriere** (`PurgeBarrier::purge`). Fehlende Barriere ⇒ Fehler, kein unbegrenztes Warten.
2. **WAL-Drain.** Rotation, Flush, versiegelte WALs entfernen, Prüfung (S2).
3. **Kandidaten.** SSTables, deren Schlüsselbereich den Matcher schneidet. Bloom-Filter nur als Vorfilter, exakte Bestätigung per Stream.
4. **Streamender Rewrite** über `create_like`: gleicher `KeyManager`, gleiche Formatversion, **Originalwerte** für `seq`, `tx_id`, Tombstone-Flag. Verworfen werden **alle** MVCC-Versionen und Tombstones der Zielschlüssel. Kein `Vec` über die ganze Datei. Schreibfehler bricht ab, der alte Reader bleibt aktiv (behebt N1: heute `create(path)` mit `key_manager = None`, `tx_id = 1` [V]).
5. **Atomarer Austausch.** `tmp` → `fsync` → Manifest-Eintrag „ersetze {alt} durch {neu}“ → `fsync(Manifest)` → `fsync(Verzeichnis)` → Reader-Swap. Alte Datei ins PendingDelete-Journal; Unlink erst, wenn keine Zeitreise mehr sie hält.
6. **Matcher.** `Exact(bytes)` für vollständige Schlüssel. `Prefix{prefix, fixed_suffix_len}` nur für Familien mit fester Länge (`__docid:` + 16 Byte). Freie `starts_with`-Abfragen auf Strings sind verboten (behebt F01 [V]: `purge_prefix(b"tenant-a/")` trifft den ganzen Mandanten).
7. **Blockcache:** Einträge ersetzter Dateien entfernen, `zeroize`.
8. **Messung** übernimmt der unabhängige Rohscanner (§12.3), nicht der Rewriter.

### 10.5 Nebenläufigkeit

- Erase und Put desselben Ziels sind gegeneinander serialisiert (Key-Lock, §10.3).
- Erase mit offenem Snapshot: gelingt; der Snapshot erhält `Erased` (§7.3).
- Erase parallel zu Kompaktion: Purge-Compaction wartet auf die reguläre Kompaktion über `PurgeBarrier`, nie umgekehrt.
- `loom`-Tests decken `LiveMask`-Publikation, ReadGate und Barriere ab (T-CONC-1).

---

## 11 Fehlerverhalten

### 11.1 Kategorien

| Kategorie | Beispiele | Verhalten |
|---|---|---|
| **Transient** | I/O, `ENOSPC` vor Journal-Flush | `Pending{IoBackoff}`, Retry mit Backoff, Zustand bleibt |
| **Journal** | Flush schlägt fehl | Takt 1 liefert `Err`, **keine** Sperre |
| **Nach Gate** | Fehler in Takt 2 vor Vernichtung | Zustand bleibt `Frozen`/`PlaintextPurged`, Sperre bleibt |
| **Vernichtung** | `destroy` schlägt fehl | Zustand bleibt `PlaintextMeasured`, `Err` an Worker |
| **Messung** | Positivkontrolle fehlgeschlagen | Messung ungültig, Lauf wiederholen, **kein** Klasse-Eintrag |
| **Lesepfad** | fehlender Schlüssel | `Erased`, nicht unterscheidbar von „nie vorhanden“ |
| **Krypto** | AEAD-Fehler | einheitlicher Fehler, nicht weiter unterscheidbar |
| **Konfiguration** | `erasure_grace > 72 h`, leere Passphrase | Start verweigert |

### 11.2 Fail-closed

- Fehlender Schlüssel ⇒ Fehler, nie Klartext-Fallback.
- Fehlendes ReadGate bei `GuardPolicy::Required` ⇒ Öffnungsfehler.
- Unbekannter Gate-Zustand ⇒ ablehnen.
- Jeder Fehler im Löschpfad erscheint im Journal.

### 11.3 Keine stillen Erfolge

`Ok(verified: false)` ist verboten (heutiger Code `erasure.rs`, Befund E8 [V]). Ein nicht verifizierter Lauf ist ein Zustand (`Pending` oder `Frozen`) oder ein Fehler, nie ein erfolgreicher Rückgabewert.

---

## 12 Klassen, Annahmen und zulässige Aussagen

### 12.1 Klassen

Die Klassen bauen aufeinander auf. Ein Bericht weist nur die höchste Klasse aus, deren Voraussetzungen gemessen sind.

| Klasse | Name | Aussage | Voraussetzungen |
|---|---|---|---|
| **L** | Gesperrt | Lesepfade liefern `Erased`; rechtlich einer Einschränkung nach Art. 18 nahe | Journal-Eintrag durabel, Lesepfad-Probe |
| **P** | Klartext bereinigt | In keiner Klartextstruktur (S1, S6–S11) ist ein Inhalt des Ziels auffindbar | Ebene A gemessen, Positivkontrolle bestanden |
| **C\*** | Schlüssel vernichtet, vorläufig | DocDek/K_subj vernichtet, Epoch Seal steht aus | Messung C bestanden; Residual `R-KEKEPOCH` |
| **C1** | Schlüssel vernichtet, Software-Wurzel | Persistente Kopien nur als Chiffrat; Schlüssel vernichtet. Annahme A-FLASH | C*, Epoch Seal, Custody K1 |
| **C2** | wie C1, Wurzel im manipulationsresistenten Modul | A-FLASH entfällt für die Wurzel | C*, Epoch Seal, Custody K2 mit Konformitätssuite |
| **C3** | wie C1, externer KMS mit Vernichtungsattest | A-FLASH durch Anbieterattest ersetzt (Prüfpflicht P-18) | C*, Epoch Seal, Custody K3 |
| **M** | Messvollständig | P und C erreicht, Hygiene vollzogen, alle Schichten gemessen oder ausgeschlossen | P, C, Hygiene, Siegel 4 |

**Zuordnung zur alten Klassenlogik (v2.0 §5.4):** Klasse K (Schlüssel vernichtet) ⇒ C1–C3. Klasse I (Indizes bereinigt) ⇒ Teil von P und M. Klasse M (vollständig gemessen) ⇒ M.

### 12.2 Orthogonale Angaben (immer im Bericht)

- `legal_state ∈ {Eingeschränkt, TechnischGelöscht, BeschränktAufbewahrt, NichtGelöscht}`
- `backup_window ∈ {Closed, OpenUntil(date), ExternalUnmanaged}` (§14)
- `custody ∈ {K0, K1, K2, K3}` und `commitment_algo`
- `assumptions[]` (§12.3)
- `residuals[]` mit Frist (z. B. `R-GRAPH until`, `R-KEKEPOCH until`)
- `subject_ref_hash` (keyed, §16), keine Klartext-IDs

### 12.3 Annahmecodes

| Code | Annahme |
|---|---|
| **A-FLASH** | Flash-Reste gewrappter Schlüssel und die Passphrase kommen nicht zusammen in die Hand derselben Stelle. Nur bei K1 |
| **A-ALGO** | AES-256-GCM-SIV, Argon2id, HKDF und BLAKE3 sind nicht gebrochen (EDPB 02/2025 Rn. 51) |
| **A-KEYLEAK** | Der Schlüssel war vor Vernichtung nicht kompromittiert (EDPB Rn. 51) |
| **A-RAM** | Kein Klartext und kein Schlüssel in Swap, Dumps oder Hibernation (P-03) |
| **A-EXTBACKUP** | Backups außerhalb von Contextra sind nicht erfasst |
| **A-HOSTHIST** | Verläufe des Host-Systems und des Modellanbieters sind nicht erfasst |

**Textregel:** Jeder Außentext nennt die Annahmen seiner Klasse. Eine Klasse ohne Annahmenliste ist ein Build-Fehler (Gate `claim-generator`).

### 12.4 Messung je Schicht (Klasse-Beleg)

| Klasse | Beleg | Kosten je Löschung |
|---|---|---|
| L | Journal-Eintrag durabel; Slot-Bit gesetzt; Lesepfad-Probe liefert `Erased` | O(1) |
| P | Slot nullbytig (fp32 und SQ8); Text-Maske gesetzt; Cache-Register leer; Scanner-Messung je Schicht | O(d) |
| C\* | `verify_absent == Absent`; Entschlüsselungsprobe liefert `KeyAbsent`; Nadelscan 0 Treffer (Wrap-Nonce) | O(1) je Schlüssel + Scan der Schlüsseldateien |
| C1–C3 | zusätzlich Epoch Seal bestätigt, Re-Wrap vollständig, Custody-Nachweis | O(Schlüssel der Collection) |
| M | Rohscan aller Dateien des Datenverzeichnisses nach Nadeln; Versionsscan; Checkpoint-Sweep | Hintergrund |

**Nadelscan:** Gesucht wird nach der **96-Bit-Wrap-Nonce** des vernichteten Schlüssels. Sie steht im Intent-Journal (§6.4 Schritt 5). Null Treffer ist Erwartung. Ein Hash des gewrappten Blobs ist kein Suchmuster, weil Rohbyte-Suche nach einem Hash alle Offsets hashen müsste (F04 [V]).

**Positivkontrolle (Pflicht je Scan-Lauf, INV-WITNESS-MINT-1):**

- **Scanner-Selbsttest (Standard in Produktion):** Der Scanner findet ein synthetisches Muster in einem Puffer, den er selbst in ein Scratch-Verzeichnis im Datenvolumen schreibt und danach löscht. Der Scratch-Pfad liegt außerhalb des Stores, nicht im WAL.
- **Live-Canary (Pflicht in Lab und CI, optional im Regulated-Tier verschärfbar):** Ein synthetischer Datensatz mit eigenem `DocDek` wird über den regulären Schreibpfad eingefügt, mit demselben Vernichtungspfad entfernt, und die Entschlüsselung muss danach scheitern. Der Canary darf **nie** in dauerhafte Nutzdaten gelangen und wird im selben Lauf gelöscht (behebt F16 [V]: Canary im produktiven WAL).

Die Positivkontrolle beweist, dass der **Scanner und der Vernichtungspfad** arbeiten. Sie beweist **nicht** die Vernichtung des konkreten `DocDek` (F05). Dafür stehen Nadelscan und Inventar.

### 12.5 Grenzen (immer im Bericht)

Der Scanner sieht nicht: Flash-Remanenz unterhalb des Dateisystems, Schlüssel in RAM, Swap, Dumps (A-RAM), externe Backups (A-EXTBACKUP), Modellanbieter-Logs (A-HOSTHIST), Controller-Caches. Diese Grenzen stehen als `ResidualRisk` im Bericht.

### 12.6 Zulässige und verbotene Aussagen

**Verboten** (Gate `deletion-honesty`): „100 % gelöscht“, „unwiederbringlich“, „DSGVO-konform“, „Art.-17-konform“, „rechtskonform gelöscht“, „garantiert gelöscht“, „kryptografisch gelöscht“ ohne Custody und Annahmen, „Löschzertifikat“ ohne Scope und Klasse, „Beweis“ (als Bezeichnung des Berichts), „informationstheoretisch sicher“.

**Erzeugt aus dem Bericht** (INV-CLAIM-BIND-1): Jeder Satz eines Außentextes stammt aus einer geschlossenen Satzbibliothek und ist an einen gemessenen Wert gebunden. Beispiel Klasse C1:

> „Der Scope ist gesperrt. Klartextstrukturen sind bereinigt. Persistente Kopien liegen nur als Chiffrat vor. Die zugehörigen Schlüssel sind vernichtet. Dies gilt unter den Annahmen A-FLASH, A-ALGO, A-KEYLEAK, A-RAM. Ausgeschlossen sind: externe Backups, Kopien außerhalb der Anwendung, Speicherabbilder.“

**Klasse L** darf nur „logisch gesperrt“ oder „eingeschränkt“ heißen. Das Wort „gelöscht“ erfordert mindestens Klasse C1 (INV-7).

---

## 13 Nachweis: Bericht, Bescheinigung, Zeitanker

### 13.1 Bericht `ErasureReport` V5

- **Ein** Format, Ed25519-signiert, **kanonische Kodierung** (festgelegtes Feldreihenfolge-Schema, deterministische Längenpräfixe). `bincode` und andere mehrdeutige Formate sind für signierte Felder verboten (ER-E16).
- **Pflichtfelder:** `report_id`, `class`, Siegel 1–4 als verkettete Hashes, `legal_state`, `backup_window`, `custody`, `commitment_algo`, `assumptions[]`, `residuals[]`, Layer-Witnesses (Ebene, Methode, Zahlen, Digest, Positivkontrolle, Ergebnis), `KeyDestroyEvidence`, `EpochSealRef`, `subject_ref_hash`, `exclusions[]`, `verification` und `validation` (§13.3), `timestamp` (über `Clock`), Prüfsumme über alle Felder.
- **Verboten im Bericht:** Klartext-IDs, Dateinamen, Inhalte, Chunk-Hashes, DocIds (INV-12, N4).
- **Unbekannte Felder** lehnt der Verifier ab. Jedes exportierte Feld ist signiert. Merkle-Pfade binden Einzelwitnesses an die Wurzel (behebt R-13 aus der Analyse).
- **Feld-Zuordnung zu NIST SP 800-88 r2 (CE-Zertifikat):** Produkt und Version, Medium (Dateisystemtyp), Methode (Purge), Technik (Cryptographic Erase, Blockrewrite), Validierungsentscheidung, Concurrence, CE-Traceability (Algorithmen, Schlüsselstärken, Schlüsseltypen, Escrow- und Backup-Historie). Eine vollständige CE-Aussage erfordert zusätzlich die Bedingungen der Norm (§23, P-19).

### 13.2 Siegel

| Siegel | Zeitpunkt | Inhalt |
|---|---|---|
| 1 | nach Takt 1 | Klasse L, `legal_state` |
| 2 | nach Messung A | Klasse P |
| 3 | nach Epoch Seal | Klasse C1–C3 mit Annahmen |
| 4 | nach Hygiene | Klasse M |

Jedes Siegel ist ein signierter Teilbericht, in die `AuditChain` eingetragen und mit dem vorherigen verkettet (REQ-DEL-N27). Der Betreiber kann damit belegen, wann welcher Zustand erreicht war.

### 13.3 Verification und Validation (NIST r2)

| Zustand | Bedeutung |
|---|---|
| `Verified` | Alle Schichten gemessen, alle Positivkontrollen bestanden |
| `Validated` | Eine zuständige Person nimmt das Ergebnis ab. Im Regulated-Tier mit Concurrence-Signatur (Vier-Augen) |

Die Klasse gilt für beide Zustände. `Validated` ist ein eigenes Feld.

### 13.4 Löschbescheinigung

Aus dem Bericht wird eine einseitige, datensparsame Bescheinigung erzeugt (OLG Dresden 15.10.2024, 4 U 940/24; AG Lörrach 03.03.2025, 3 C 1099/24):

- **enthält:** Datum, Art des Scopes (z. B. „Akte“), Klasse je Schichtgruppe, Annahmen, zusammengefasste Ausschlüsse, Digest des Berichts, Signatur, Zeitanker-Status.
- **enthält nicht:** IDs, Dateinamen, Inhalte, Embedding-Daten, DocIds.

Ob und an wen sie ausgeliefert wird, entscheidet der Verantwortliche. Ein Anspruch der betroffenen Person auf diesen Nachweis ist nicht gesichert (P-14).

### 13.5 Zeitanker

- Der Digest des Berichts wird optional über einen qualifizierten Zeitstempel (RFC 3161, eIDAS Art. 41, 42) verankert. Der Zeitstempel belegt Existenz und Datenintegrität zum Zeitpunkt, **nicht** die Löschung.
- Offline-Betrieb: `anchor: pending`, Nachholen bei Verbindung. Verifikation offline mit Zertifikatskette und Vertrauensliste als Schnappschuss im Bericht (Langzeitvalidierungsdaten).
- Ein ausgestelltes Token ist nicht widerrufbar. Die Anbieterwahl ist offen (O-34).

### 13.6 Verifikation

`doctor verify-report` prüft offline: Signatur, Prüfsumme, Klasse, Siegelkette, Zeitanker-Format, Pflichtausschlüsse (REQ-DEL-N9). Ein Bericht mit Klasse M, aber einer Schicht ohne Positivkontrolle, ist ungültig. Ein unabhängiger Referenz-Prüfer in anderer Sprache dient als Testorakel und gehört nicht zum Produkt (INV-ORACLE-1).

### 13.7 Aufbewahrung des Berichts

Der Bericht enthält nur keyed Referenz-Hashes und gilt selbst als personenbezogen (EDPB 02/2025 Rn. 52). Aufbewahrung nach `report_retention`; danach Ablauf mit dem Referenzschlüssel (§16). Die Dauer ist Rechtsfrage (P-07).

---

## 14 Backup und Restore

### 14.1 Zwei-Spuren-Backup

| Spur | Inhalt | Lebensdauer | Schutz |
|---|---|---|---|
| **D (Daten)** | SSTables, WAL, Blob-Segmente, Index-Artefakte (Chiffrat), ohne KeyTable und ohne Keystore | beliebig (Betreiber) | ohne Schlüssel nicht lesbar |
| **K (Schlüssel)** | KeyTable-Snapshot (gewrappte Schlüssel, Epochen) | **befristet**: `key_backup_max_age` (Startwert 30 Tage), signiertes `valid_until` | Verschlüsselung unter der Wurzel |

**Wirkung.** Nach Löschung und Ablauf aller Schlüssel-Backups, die den Schlüssel enthalten, existiert er nirgends mehr. Die Daten-Backups sind für diese Löscheinheit unlesbar. Die Löschung wirkt in Backups spätestens nach `key_backup_max_age`, ohne Eingriff in das Backup.

**Bericht.** `backup_window = OpenUntil(deletion_at + key_backup_max_age)` für von Contextra verwaltete Backups. `ExternalUnmanaged` für alles andere.

### 14.2 Restore-Guard

Vor dem ersten Lesezugriff nach einem Restore:

1. `ErasedSet` des Zielsystems und den `ErasedSetMirror` lesen. Nicht aus dem Backup.
2. Gegenseitig abgleichen. Für jeden Eintrag: Takt 2 ab Schritt 5 erneut anwenden, wiederhergestellte Schlüssel vernichten.
3. Messung wie in §12.4. Journal-Ereignis `restore_reapplied`.
4. Dokumentation (Grund, Zeitpunkt) im Audit (BayLfD 30. TB Nr. 12.5).

**Horizont.** Ein Schlüssel-Backup mit abgelaufenem `valid_until` oder unbestimmbarem Alter wird verweigert (`RestoreBeyondHorizon`). Daten-Backups unterliegen keinem Horizont, weil sie ohne Schlüssel inert sind.

### 14.3 ErasedSetMirror

Nach Totalverlust fehlt der `ErasedSet` des Zielsystems. Der Mirror (§9.5) hält jeden Eintrag außerhalb des Datenverzeichnisses.

- Regulated-Tier: Pflicht (REQ-DEL-N19).
- Ohne Mirror, oder mit einem Mirror, der älter ist als das Schlüssel-Backup: Restore nur mit `--accept-unguarded`. Der Bericht und `doctor` weisen `restore_unguarded` aus.
- Ein zentrales Löschregister außerhalb von Contextra (O-35) ist ein zulässiges Mirror-Ziel.

### 14.4 Aufbewahrung des ErasedSet

`erased_set_retention ≥ key_backup_max_age + Sicherheitszuschlag` (Startwert insgesamt 90 Tage). Die Konfiguration erzwingt `key_backup_max_age ≤ erased_set_retention`. Die 365 Tage aus v2.0 entfallen, weil nur Schlüssel-Backups gelöschte Inhalte zurückbringen können.

### 14.5 Aufsichtsauffassungen

CEF-Bericht 2025 (Issue 6) und BayLfD 12.5 verlangen Nachverfolgung und Nachlöschung beim Restore sowie Begründung, wenn Backups nicht sofort bereinigt werden. Der Zwei-Spuren-Ansatz erfüllt das mit einem festen, dokumentierten Fenster. Ob ein Chiffrat ohne Schlüssel für den Besitzer des Daten-Backups keine personenbezogene Information mehr ist, bleibt P-02.

---

## 15 Fristen, Empfänger, Aufbewahrung

### 15.1 Fristen

| Frist | Quelle | Wert |
|---|---|---|
| Beantwortung | Art. 12 Abs. 3 DSGVO | ein Monat; Verlängerung um zwei Monate mit Mitteilung binnen eines Monats |
| Klasse L | intern | sofort nach Takt 1 |
| Klasse P | intern, Zielgröße | ≤ 1 Stunde nach Takt 1 |
| Klasse C\* | intern | unmittelbar nach P, zuzüglich `erasure_grace` |
| Epoch Seal | intern | ≤ `seal_after_deletion_max` (7 Tage) nach Vernichtung |
| Hygiene (M) | intern | ≤ `purge_rebuild_max_days` (14 Tage) |
| Alarm bei `Partial` | intern | 24 h |
| Eskalation bei `Partial` | intern | 14 Tage |
| Warnung bei Fristgefährdung | intern | 25 Tage |

Die Werte sind Zielgrößen. Contextra sendet nichts nach außen. Alarme erscheinen in `doctor`, Metrik, CLI, MCP-Antwort und AuditChain.

**Offen (P-11):** Ob eine Verzögerung von Stunden bis Tagen in Takt 2 mit „unverzüglich“ vereinbar ist, wird durch Takt 1 entschärft: Ab Takt 1 ist das Datum gesperrt. Die Rechtsfrage bleibt.

### 15.2 RecipientLedger (Art. 17 Abs. 2, Art. 19)

Je Dokument führt Contextra ein Register der Ausleitungen (Export, Egress-Kanal, Checkpoint-Export) ohne Inhalte: Empfängerkennung, Zeitpunkt, Datenklasse. Bei Löschung liefert `doctor` die Empfängerliste. Die Mitteilung an Empfänger erfolgt durch den Verantwortlichen. Ausleitungen außerhalb von Contextra sind Ausschluss (§17.3).

### 15.3 Legal Hold

Ein Hold wird im `RevocationLog` und im Audit geführt. Er betrifft die Saga (Zustand `Restricted`), nicht die Lesesperre. Ein Hold darf nur mit dokumentierter Rechtsgrundlage gesetzt werden.

---

## 16 Referenzschlüssel und Commitments

**Zweck.** Berichte, `ErasedSet`, `KeyRegistry`, Journal und Duplikaterkennung müssen auf Betroffene verweisen, ohne sie zu identifizieren.

**Regeln.**

1. `subject_ref_hash = BLAKE3-keyed(K_ref[tenant, e_ref], canonical(SubjectRef))`. Dies ist ein **Referenz-Hash mit Schlüssel** und bleibt personenbezogen (EDPB 02/2025 Rn. 52). Die Bezeichnung „Commitment“ ist nur für das Scope-Commitment in der Duplikaterkennung zulässig (ER-E08).
2. `K_ref` ist zufällig, gewrappt unter `K_tenant`, je (Mandant, Referenzepoche). Nicht per HKDF ableitbar (H1).
3. Nach `ref_epoch_len` (Startwert 90 Tage) plus `erased_set_retention` wird `K_ref` vernichtet. Danach sind die Hashes dieser Epoche keiner Person mehr zuordenbar, soweit `K_ref` nicht in Backups liegt (Spur K ist befristet).
4. Sidecars enthalten keine Nutzinhalte und keine Schlüssel.
5. Die kanonische Betroffenen-Referenz hat Mindestentropie (128 Bit, zufällig, nicht Name oder E-Mail-Adresse). Sie steht nur im Dokumentdatensatz, nie in Sidecars.

---

## 17 Härtung, Ausschlüsse und Krypto-Agilität

### 17.1 Härtung des Prozesses (NIST: kein Klartext auf dem Medium vor Verschlüsselung)

1. **Core-Dumps aus.** `RLIMIT_CORE = 0`. Speicherbereiche für Schlüssel und Klartext-Arenen von Dumps ausgenommen (`MADV_DONTDUMP` bzw. plattformäquivalent).
2. **Schlüsselpuffer gesperrt.** `mlock` für Schlüsselpuffer. Schlägt `mlock` fehl, meldet `doctor` die Lücke.
3. **Swap.** `doctor` prüft, ob Swap aktiv und verschlüsselt ist. Große Arenen (Vektoren) lassen sich nicht sperren; bei unverschlüsseltem Swap erscheint eine Warnung, und A-RAM bleibt im Bericht.
4. **Temp-Dateien.** Keine Klartext-Zwischenspeicher auf Platte. Zwischenergebnisse liegen im Speicher oder verschlüsselt unter `ArtKey`/`K_eng`.
5. **Dateisystem.** Unterstützte lokale Typen (INV-PLATFORM-1). Dateisystem-Snapshots, Journale und Time Machine gelten als externe Kopien und stehen im Bericht.
6. **Logs.** Nur IDs (keyed) und Zähler (INV-12, Lint).
7. **Hibernation.** `doctor` weist auf aktive Hibernation hin (A-RAM).

Die plattformspezifischen Aufrufe sind in Spike S-HRD zu verifizieren. Fehlt ein Mittel, meldet `doctor` die Lücke und die Annahme bleibt im Bericht.

### 17.2 Speicherung und Trim

- Ledger- und KeyTable-Überschreiben sind nur auf Dateisystemen wirksam, deren Schreibpfad überschreibt. Auf CoW-, Netz-, FUSE- oder Cloud-Dateisystemen gilt `ResidualRisk::FlashRemanence` unabhängig von der Custody-Stufe, sofern nicht K2/K3 greifen.
- Discard-Hinweise (`FITRIM`) sind Optimierung, kein Nachweis.

### 17.3 Ausschlüsse (verpflichtend im Bericht)

- Externe Sicherungen außerhalb der Spur Backup (A-EXTBACKUP).
- Prozessspeicher, Swap, Dumps, Hibernation vor der Vernichtung (A-RAM).
- Hardwareinterne Reste (Wear-Leveling, Controller-Cache). Gegenmaßnahme: Verschlüsselung plus Epoch Seal; Restannahme A-FLASH bei K1.
- Trainings- und Fine-Tuning-Daten externer Modelle.
- Kopien beim Auftragsverarbeiter, Modellanbieter-Logs, Prompt-Caches, Host-Historien (A-HOSTHIST).
- Ableitungen außerhalb der Lineage (Exporte, die der Betreiber angelegt hat).
- Dateisystem-Snapshots, Journale des Betriebssystems.

Der Ausschlusstext darf nicht gekürzt werden, wenn der Bericht weitergegeben wird.

### 17.4 Restrisiko Schlüsselanker

Ohne `K2`/`K3` gilt: Der Betreiber, der die Passphrase kennt, kann alte gewrappte Schlüssel aus Flash-Resten nur entwrappen, wenn er zusätzlich Zugriff auf den Flash hat (Q5 in der Perspektivenmatrix, §17.6). Gegen Angreifer **ohne** Passphrase wirkt die Verschlüsselung unabhängig von der Löschung.

### 17.5 Perspektivenmatrix

| Perspektive | Zugang | nach L | nach P | nach C1 | Anmerkung |
|---|---|---|---|---|---|
| Q1 Leser (MCP, API, `as_of`, Export) | Lesepfade | gesperrt | gesperrt | gesperrt | Gate vor Rückgabe |
| Q2 Betreiber mit Prozess- und Dateizugriff und Passphrase | Engine, Dateien, Keystore | Klartext nur über gesperrte Pfade | keine Klartextstruktur des Scopes | Kopien nur Chiffrat unter vernichteten Schlüsseln | Betreiber ist Verantwortlicher |
| Q3 Dateiangreifer ohne Passphrase | Datenverzeichnis | verschlüsselt | verschlüsselt | verschlüsselt | unabhängig von Löschung |
| Q4 Medienangreifer mit Flash-Resten, ohne Passphrase | Chip-off | verschlüsselt | verschlüsselt | verschlüsselt | |
| Q5 Medienangreifer mit Flash-Resten **und** Passphrase | wie Q4 plus Schlüssel | lesbar | lesbar (Chiffrat) | nur K1: möglich (A-FLASH); K2/K3: nicht | Kern der Restannahme |
| Q6 Besitzer eines Daten-Backups ohne Schlüssel-Backup | Backup-Dateien | verschlüsselt | verschlüsselt | nicht lesbar | Zwei-Spuren-Backup |
| Q7 Besitzer eines Schlüssel-Backups älter als Löschung | Schlüssel-Backup | lesbar bis Fensterende | lesbar bis Fensterende | unlesbar nach `key_backup_max_age` | Restore-Guard greift |
| Q8 Empfänger und Dritte (Exporte, Modellanbieter, Auftragsverarbeiter) | außerhalb | nicht erfasst | nicht erfasst | nicht erfasst | Ausschluss §17.3 |

### 17.6 Krypto-Agilität und Notfallverfahren

- Jedes persistierte kryptografische Objekt trägt `suite_id` und `fmt_ver`.
- Wechsel von AEAD, KDF, Hash oder Signatur ist ohne Datenverlust möglich: Re-Wrap von Schlüsseln, Neuverschlüsselung von Records per Kompaktion.
- Das Notfallverfahren bei Algorithmenversagen (EDPB 02/2025 Rn. 85, 96, Empf. 13) ist dokumentiert und wird halbjährlich per Test geübt (`doctor crypto-drill`). Die Eignung der Primitive wird periodisch neu bewertet (P-13).

---

## 18 Konfiguration

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
    /// Kein Crypto-Shredding. Löschung erreicht maximal Klasse L.
    None,
    /// Verschlüsselung obligatorisch. Passphrase als SecretString.
    Required { passphrase: SecretString, custody: CustodyChoice },
}

pub struct ErasureConfig {
    pub ledger_dir: PathBuf,                 // MUSS außerhalb von data_dir liegen
    pub epoch_window: Duration,              // Standard 5 s, max. epoch_max = 60 s
    pub epoch_max: Duration,                 // 60 s
    pub kek_epoch_len: Duration,             // 30 Tage
    pub seal_after_deletion_max: Duration,   // 7 Tage
    pub erasure_grace: Duration,             // 0 h, max. 72 h; Regulated: Pflichtwert
    pub purge_rebuild_max: Duration,         // 14 Tage
    pub tombstone_rebuild_ratio: f32,        // 0.2 (HNSW Strict: 0.05)
    pub key_backup_max_age: Duration,        // 30 Tage
    pub erased_set_retention: Duration,      // 90 Tage, >= key_backup_max_age
    pub ref_epoch_len: Duration,             // 90 Tage
    pub report_retention: Duration,          // Betreiberentscheid (P-07)
    pub disk_reserve_bytes: u64,             // min. 1 GiB
    pub mixed_policy: MixedPolicy,           // BlockAll (Standard)
    pub positive_control: PositiveControlMode, // ScannerSelftest | LiveCanary
    pub mirror: Option<MirrorTarget>,        // Pflicht im Regulated-Tier
}
```

**Konfigurationsvalidierung (`Contextra::open` vor dem Öffnen):**
- `data_dir` vorhanden und beschreibbar.
- `erasure.ledger_dir` liegt **nicht** unterhalb von `data_dir`.
- `disk_reserve_bytes ≥ 1 GiB`.
- Bei `Required`: Passphrase nicht leer.
- `erasure_grace ≤ 72 h`; im Regulated-Tier gesetzt.
- `key_backup_max_age ≤ erased_set_retention`.
- `tombstone_rebuild_ratio ∈ (0, 1)`.
- Im Regulated-Tier: `mirror` gesetzt, `positive_control = LiveCanary`.

---

## 19 Testkatalog und Erasure Lab

### 19.1 Kennungen aus v2.0 (bleiben gültig)

| Kennung | Prüft | Kategorie |
|---|---|---|
| T-DEL-1 | Nach Insert und Flush: alle persistierten Bytes verschlüsselt, kein Klartext-Canary | Integration |
| T-DEL-2 | Nach Löschung, Purge und Compaction: Lesepfad `Erased`, Rohscan 0 Treffer | Integration |
| T-DEL-3 | Löschnachweis-Verifikation schlägt mit falschem Schlüssel fehl | Unit |
| T-DEL-4 | Nach Löschung und Neustart bleibt das Datum `Erased` | Integration |
| T-DEL-5 | Ohne `EncryptionConfig::Required` erreicht Löschung maximal L | Integration |
| T-DEL-6 | Nach Löschung findet weder Volltext-, Vektor- noch Graphsuche das Dokument | Integration |
| T-DEL-7 | Positivkontrolle: Scanner findet Muster, wird zu `Scanner<Armed>` | Unit |
| T-DEL-8 | Collection-Löschung widerruft `K_coll`; alle Dokumente `Erased` | Integration |
| T-DEL-9 | Betroffenen-Löschung: `K_subj` vernichten, alle P-Daten `Erased`, O(1) in Kryptografie | Integration |
| T-DEL-10 | Lineage-Kaskade: Quelle gelöscht ⇒ alle Ableitungen bereinigt, Rohscan 0 | Integration |
| T-DEL-11 | ReadGate blockiert alle Lesepfade aus §7.4 | Integration |
| T-DEL-12 | Crash in Phase 3 ⇒ Neustart ⇒ Saga fortgesetzt, Gate sofort aktiv | Integration |
| T-DEL-13 | Restrict: Lesen nur mit Audit-Eintrag, sonst `RestrictedReadAuditRequired` | Integration |
| T-DEL-14 | `purge` idempotent | Unit |
| T-DEL-15 | `verify_absent` schlägt fehl, wenn der Slot nicht vollständig `0x00` ist | Unit |

**Test-Regel:** Jeder Löschtest durchläuft die vollständige Saga inklusive Purge. Ein Test, der nur das Gate setzt, prüft höchstens Klasse L.

### 19.2 Neue Kennungen (v3.0)

| Kennung | Prüft |
|---|---|
| T-DEL-16 | Dokument A und B in zwei Collections desselben Mandanten; `erase(A)`; B bleibt in allen Schichten lesbar (NeighborDigest bitgleich) |
| T-DEL-17 | Präfixe von IDs (`alice`/`alice2`, `1`/`10`) werden durch Löschung nicht verwechselt |
| T-DEL-18 | Erzwungene DocId-Kollision: Insert wird abgelehnt; Erase trifft nur den Scope |
| T-DEL-19 | Nach Erase liefert **jeder** Lesepfad `Erased`, für alle Sequenznummern, inklusive `scan_prefix_at`, Checkpoint, Fork |
| T-DEL-20 | Absturz nach **jeder** Vfs-Operation der Saga: nach `recover()` ist der Zustand terminal erreichbar, nie Lesezugriff auf einen gesperrten Scope |
| T-DEL-21 | Abgerissener Write an jedem Byte von Journal, KeyTable und Manifest: Recovery bleibt konsistent |
| T-DEL-22 | `ENOSPC` und `EIO` bei `fsync` in der Purge-Compaction: Zustand `Pending`, Nachbarn unverändert |
| T-DEL-23 | Nach Vernichtung scheitert die Entschlüsselung des Canary-Blocks; `destroy` ist idempotent |
| T-DEL-24 | `step` für alle Zustandskombinationen; `classify` monoton; E impliziert L, P, C |
| T-DEL-25 | Erase parallel zu Put desselben Ziels; Erase mit offenem Snapshot; Barriere gegen Kompaktion (loom und Stress) |
| T-DEL-26 | Zufallsfolgen (Insert, Update, Flush, Kompaktion, Neustart, Erase) gegen `ReferenceModel` (Differential) |
| T-DEL-27 | Jeder Schlüssel jeder öffentlichen Collection-Operation liegt im `LayerCatalog` und im `ErasureManifest` |
| T-DEL-28 | Änderung eines beliebigen Bytes des Berichts wird abgelehnt; Testvektoren für den externen Verifier |
| T-DEL-29 | Mixed-Segment: `BlockAll`, `RedactAndRekey`, `DeleteAll` mit erwartetem Zustand |
| T-DEL-30 | Restore: Daten-Backup vor Löschung, Löschung, Restore mit altem und frischem Schlüssel-Backup; abgelaufenes Schlüssel-Backup wird verweigert |
| T-DEL-31 | Epoch Seal: Crash-Matrix über Re-Wrap; nach Abschluss hält `K_coll[e]` nicht mehr in der KeyTable |
| T-DEL-32 | Tor-Probe: Chiffrat unter `K_subj(A)` ist mit `K_subj(B)` nicht öffnen (AAD) |
| T-DEL-33 | Scan-Lauf ohne bestandene Positivkontrolle erzeugt keine Klasse |
| T-DEL-34 | Schlüssel mit Nutzer-ID im Klartext: Der Store-Scan findet keine Nutzer-ID (INV-KEY-OPAQUE) |

### 19.3 Erasure Lab (`contextra-testkit`, E3)

| Komponente | Zweck |
|---|---|
| `FaultVfs` | Absturz nach Operation *n*, abgerissene Writes an Byte *b*, `ENOSPC`, `EIO` bei `fsync`, Verlust ungesyncter Daten, nicht atomares `rename` |
| `ForensicScanner` | Rohscan nach Nadeln: Schlüsselbytes, Wertbytes, DocId (LE), UTF-8 und UTF-16LE, Base64, Chiffretext-Präfixe; je Datei und Offset; berücksichtigt Hardlink-Klone. **Ohne** Store-Parser |
| `HeapScanner` (Linux) | liest `/proc/self/mem` nach Nadeln inklusive fp32-Vektormuster |
| `CanaryFactory` | eindeutige, hochentropische Nadeln je Dokument und Feld, reproduzierbar per Seed |
| `ReferenceModel` | In-Memory-Orakel (BTreeMap, ErasedSet) mit identischer Operationsfolge |
| `RecordingStorage` | Dekorator, protokolliert jeden geschriebenen Schlüssel je öffentlicher Operation (blinde Flecken) |
| `NeighborDigest` | Hash über `(key, seq, flags, value)` aller Nicht-Ziel-Paare |
| `LeakyPurger` | absichtlich fehlerhafter Purger, Selbsttest des Labs (T-DEL-35) |
| `cargo-mutants` | Gate ≥ 95 % für `step`, `classify`, Matcher, Barriere, Anker |
| `loom`, Fuzz | ErasedSet, ReadGate, Barriere; Journal-Replay, KeyTable-Parser, SSTable-Index, WAL-Frame |

**Lab-Selbsttest T-DEL-35:** Der `LeakyPurger` muss entdeckt werden, das saubere System muss bestehen. Ohne bestandenen Selbsttest ist jeder Lab-Lauf ungültig.

**Blinde Flecken ausschließen:** (a) `KeyKind` und `LayerCatalog` sind exhaustiv (Kompilierzeit); (b) `RecordingStorage` meldet Schlüssel ohne Katalogeintrag (Laufzeit, T-DEL-27); (c) der Scanner findet absichtlich gelassene Reste in Fixtures je Schicht; (d) T-DEL-19 iteriert alle Sequenznummern; (e) nicht Messbares steht als `ResidualRisk` im Bericht.

### 19.4 Benchmarks und Gates

| Messgröße | Gate |
|---|---|
| Takt 1 p50/p99 bei 10⁴, 10⁶ Dokumenten, Einzel- und 64-fach parallel | p99 ≤ 5 ms [S], unabhängig von N |
| Durchsatz Takt 1 (Group-Commit) | ≥ 5.000 Löschungen/s [S] |
| Takt 2 bis Klasse C | ≤ 60 s bei Standard-Zeitfenster [S] |
| Lese-Latenz während Massenlöschung | Regression ≤ 5 % |
| ReadGate-Overhead auf `get` | ≤ 3 % p50 |
| Recall@10 nach Löschung von 10 % | ≥ Ausgangswert − 1 Punkt, je Modus getrennt |
| Padding-Overhead | ≤ 12 % |
| Schreib-Overhead je Dokument durch DocDek | ≤ 5 % Ingest-Durchsatz [S] |
| Purge-Compaction Schreibverstärkung | Budget (Messung, O-24), Regression ≤ 5 % |
| RSS-Spitze beim Rewrite | ≤ Konstante × Blockgröße |
| `erase_collection` (Vordergrund) | O(1) |
| Regression Bestand (`lsm_concurrency_bench`, `wal_bench`, `sstable_bench`) | ≤ 5 % |

Zielwerte [S] sind Vorschläge für Gates. Erst die Messung entscheidet, welche Gates bindend werden. Die Basislinie wird im Bench-Harness gespeichert (INV-7).

---

## 20 Invarianten

| ID | Invariante |
|---|---|
| INV-1 | Kein persistiertes Byte ändert sich ohne Formatvermerk |
| INV-2 | Jeder öffentliche Pfad bleibt auflösbar |
| INV-3 | `forbid(unsafe_code)` außerhalb der Inseln (`sys`, `simd`) |
| INV-4 | Kein `unwrap`/`expect`/`panic!`/`todo!`/`unimplemented!` in Nicht-Test-Code |
| INV-5 | Gleiche Eingabe ⇒ gleiche Rangfolge, Tie-Break ID aufsteigend |
| INV-6 | WAL, Manifest, Flush und Recovery bleiben bei Umbau unverändert (außer durch die in §23 genannten Änderungen, die je einen ADR und einen Golden-File-Test brauchen) |
| INV-7 | Keine Compliance-Aussage. „Gelöscht“ nur mit gemessener Klasse ≥ C1. Keine Skalierungsbehauptung ohne Bench-Lauf |
| INV-8 | Abhängigkeitskanten nur abwärts (R1) |
| INV-9 | `Completed`, `Refused`, `Lifted` sind terminal |
| INV-10 | Gate wird nie zurückgenommen (außer durch `Lifted` nach `Restricted`) |
| INV-11 | Schlüsselbytes werden mit `zeroize` genullt vor Drop; kein `Clone`, `Debug`, `Serialize` für Schlüsseltypen |
| INV-12 | Logs, Traces, Berichte enthalten nie Klartext des Löschziels oder PII |
| INV-GATE-COMPLETENESS | Jeder Lesepfad aus §7.4 prüft das ReadGate |
| INV-WITNESS-MINT-1 | `Scanner<Armed>` nur nach bestandener Positivkontrolle |
| INV-LINEAGE | Jede D-Schreiboperation ohne `LineageRef` ist ein Fehler |
| INV-DERIVED-1 | Lineage-Kaskade bei Löschung der Quelle |
| INV-K-NOPERS | K-Dokumente enthalten keine `SubjectRef` |
| INV-KEY-OPAQUE | Schlüssel im Store enthalten nur keyed DocIds, nie Nutzer-IDs |
| INV-H1 | Löschschlüssel sind zufällig und gewrappt, nie abgeleitet |
| INV-EPOCH-SEAL-1 | C1–C3 erst nach Epoch Seal, der die Löschung umfasst |
| INV-BACKUP-SPLIT-1 | Kein Daten-Backup enthält Schlüsselmaterial; jedes Schlüssel-Backup trägt `valid_until` |
| INV-ARTKEY-1 | Jedes Index-Artefakt hat genau einen `ArtKey`; beim Neuschreiben wird der alte vernichtet |
| INV-CLAIM-BIND-1 | Jeder Satz eines Außentextes ist an einen gemessenen Wert gebunden |
| INV-SURFACE-1 | Jede persistente Struktur ist einer Schicht oder einem Ausschluss zugeordnet |
| INV-RECOVER-1 | `recover()` läuft vor Freigabe des Lesepfads |
| INV-ASYNC-1 | Keine `block_on`-Aufrufe in Löschpfaden |
| INV-JRN-1 | Jeder Zustandsübergang steht nach durablem Journal-Eintrag |
| INV-WIRING-2 | Keine Stubs im Produktionscode, die Erfolg melden |
| INV-PROOF-KEY-1 | Berichte nur mit Ed25519-Schlüssel aus `ProofSigner` |

---

## 21 Gates (Build und CI)

| Gate | Prüfung | Bei Fehler |
|---|---|---|
| `layering` (§15 v2.0) | R1–R6, Closure E1 ≤ 5, Entfernt-Liste | CI bricht ab |
| `ownership` | Typbesitzer (§2.4): keine Dubletten, `Vfs` nur in `durable-fs` | CI bricht ab |
| `surface-coverage` | Jede persistente Struktur in §5.2 oder §17.3; jedes `Purgeable`-Register-Element | CI bricht ab |
| `claim-generator` | jeder Satz der Satzbibliothek ist an einen Messwert gebunden; absichtlich überzogener Text schlägt fehl | CI bricht ab |
| `deletion-honesty` | verbotene Begriffe (§12.6) in Quelltext, Docs und Berichtstexten | CI bricht ab |
| `key-remnant` | nach Vernichtung enthält keine Datei die Wrap-Nonce des Schlüssels (Testdateien und Lab-Fixtures) | CI bricht ab |
| `loc-budget` | Produktionscode der Löschkette (`erasure`, `crypto::keytable`, `store::erase`, `engine::erasure`) ≤ 2.500 Zeilen | CI bricht ab; Erhöhung braucht ADR |
| `unwired` | Ports ohne Implementierung melden `CapabilityUnsupported`, nicht Erfolg (INV-WIRING-2) | CI bricht ab |

---

## 22 Entscheidungsregister

Herkunft: **[S]** = SPEC v2.0 bzw. Repo-Stand, **[N]** = neu in v3.0 nach Auswertung der Konzeptdokumente.

| ID | Festlegung | Herkunft | Status |
|---|---|---|---|
| **ER-E01** | Löschschlüssel zufällig und gewrappt (H1). Ersetzt HKDF für `CollectionKEK`, `SubjectKey`, `TenantKEK` | [N], ersetzt [S] §4.1 | verbindlich, ADR |
| **ER-E02** | Zwischenebene `K_subj` je (Collection, Betroffener) für P-Daten. Subjekt-Löschung O(1) in der Kryptografie | [N] (Konflikt zwischen Per-Dokument-Schlüsseln und T-DEL-9 aufgelöst) | verbindlich, ADR |
| **ER-E03** | Löscheinheit = Dokument oder Segment mit fester Betroffenenmenge; Mixed-Policy Standard `BlockAll` | [N] | verbindlich |
| **ER-E04** | Epoch Seal mit KEK-Epochen, Re-Wrap statt Datenrewrite | [N] | verbindlich, ADR |
| **ER-E05** | KeyTable mit Ersetzen-Semantik, KEK_table im `KeyAnchor` | [N] | verbindlich |
| **ER-E06** | Custody K0–K3; K2/K3 als Port, Auslieferung nach Konformitätssuite | [N], teilweise Rücknahme [S] E27 | ADR |
| **ER-E07** | Passphrase-Argon2id: m = 256 MiB, t = 4 (strenger als 64 MiB in den Konzepten) | [S] | verbindlich |
| **ER-E08** | Einheitlich BLAKE3-keyed für Referenz-Hashes und Scope-Commitments. HMAC-SHA-256 entfällt | [N], ersetzt [S] Hinweis | verbindlich |
| **ER-E09** | Gesamtbild K2/K3: Port vorgesehen, `CapabilityUnsupported` bis Konformitätssuite | [N] | ADR |
| **ER-E10** | Zwei Takte: Takt 1 synchron wirksam (L), Takt 2 gebündelt | [N] | verbindlich |
| **ER-E11** | Saga-Reihenfolge: Ebene A vor Vernichtung; Vernichtung vor Hygiene. Die Begründung „Vernichtung vor Bereinigung macht andere Dokumente unlesbar“ war falsch, weil Records einzeln verschlüsselt sind. Die Reihenfolge bleibt aus Messgründen | [N], ersetzt [S] Schritt 3 vs. 4 | verbindlich |
| **ER-E12** | Snapshot-Widerruf entfällt; Erase gewinnt über ReadGate | [N], ersetzt [S] Schritt 4a | verbindlich |
| **ER-E13** | Blinde Terme im Volltextindex; Präfix- und Wildcard-Suche entfallen im Erstprodukt | [N] | Produktentscheidung (P-10) |
| **ER-E14** | DocId 128 Bit keyed als Standard; Kollisionsschutz fail-closed | [N], ersetzt 64-Bit-Unkeyed [V] | verbindlich, ADR |
| **ER-E15** | Journal als eigene Datei, nicht im LSM-WAL | [N] | verbindlich |
| **ER-E16** | Bericht V5 mit kanonischer Kodierung, Ed25519; V4 bleibt eingefroren | [S] E26 ersetzt | ADR |
| **ER-E17** | Positivkontrolle: Scanner-Selbsttest Standard; Live-Canary Pflicht in Lab/CI, Regulated optional verschärft. Kein Canary im produktiven WAL | [N], löst Widerspruch zwischen Konzepten | verbindlich |
| **ER-E18** | Async-Ports an der Kante (BoxFuture), Group-Commit auf eigenem Thread, kein `block_on` | [N], ersetzt [S] synchrone Ports (Konzept) | verbindlich |
| **ER-E19** | `Vfs` exklusiv in `durable-fs` (env-Spec D-04) | [S] env, Befund R4 [V] | verbindlich |
| **ER-E20** | Store bleibt eigener LSM. Entscheidungsgate G0 (Forensik-Vergleich mit redb-Adapter) vor jeder Löschaussage | [N] | Gate G0 |
| **ER-E21** | `erase(scope, on_retention: Reject | Restrict)`; `Restricted` ohne Purge und ohne Vernichtung | [S] §5.7, [N] Präzisierung | verbindlich |
| **ER-E22** | Mandant (`TenantId`) ist kein personenbezogenes Datum (Annahme P-07) | [N] | Rechtsfrage P-07 |
| **ER-E23** | HNSW-Modi Strict (Standard) und Waypoint (nicht freigegeben) | [N] | Waypoint gesperrt bis Recall-Messung und Rechtsbewertung |
| **ER-E24** | Keine Stubs. `ErasureLayer`-Implementierungen fehlen → Lesepfad fail-closed, nie Erfolg | [S] INV-WIRING-2 | verbindlich |

**Abgelehnte Varianten (mit Begründung):**
- *Vernichtung unmittelbar nach `Frozen`* (Key-first): verhindert die Messung der entschlüsselten Sicht und jede Rücknahme. Ersetzt durch Karenz (§6.8).
- *Redb als Speicher*: Die Behauptung „redb kann kein Crypto-Shredding“ trägt nicht, weil Verschlüsselung pro Dokument auf jeder Engine funktioniert. Der Eigenbau bleibt wegen der Kontrolle über Versionen und Kompaktion, vorbehaltlich G0.
- *Waypoint-Modus als Standard*: hinterlässt Mittelwerte lebender Nachbarn, die dem gelöschten Vektor ähnlich sind. Gesperrt bis zur Rechtsbewertung.
- *Dedup über Dokumente hinweg*: teilt Schlüssel über Löscheinheiten; verletzt H1 und Löschbarkeit. Ausgeschlossen.
- *Umwandlung von Klasse K in „Schlüssel vernichtet“ ohne Epoch Seal*: unzulässig (INV-EPOCH-SEAL-1).

---

## 23 Offene Punkte und Übergangsregeln

### 23.1 Offene Rechtspunkte (P) — vor Außenkommunikation zu klären

| ID | Frage | Hinweis |
|---|---|---|
| P-01 | Gilt Schlüsselvernichtung als Löschung, Anonymisierung oder Einschränkung? | entscheidend für C. Anonymisierung als „Minus“ (OLG Dresden 4 U 292/25) nur Sekundärquelle |
| P-02 | Entfällt der Personenbezug von Chiffraten nach Schlüsselvernichtung für jede Stelle? | relative Betrachtung (EuGH C-413/23 P); EDPB-Entwurf 02/2026 |
| P-03 | Nachweis der Abwesenheit von Schlüsselmaterial in RAM, Swap, Dumps | nicht vollständig möglich, A-RAM |
| P-05 | Embeddings als personenbezogen | vorsorglich ja (Ghost Vectors, arXiv 2606.18497, Preprint) |
| P-06 | Frist für Schlüssel-Backups und Wiedereinspielsperre | bestimmt `key_backup_max_age` und `erased_set_retention` |
| P-07 | Personenbezug der Nachweis-Metadaten und des Mandanten | Referenz-Hash bleibt personenbezogen (EDPB Rn. 52); Mandant ER-E22 |
| P-08 | Rolle des Anbieters bei lokal betriebener Software | kein Datenzugriff, keine Telemetrie |
| P-09 | Aufbewahrungspflichten und Dauer von `Restricted` | O-12 |
| P-10 | Blinde Terme: Produktentscheidung zur Suchfunktion | ER-E13 |
| P-11 | „Unverzüglich“ und Fristen der Hygiene | §15.1 |
| P-12 | Empfängerinformation Art. 17 Abs. 2, Art. 19 | §15.2 |
| P-13 | Notfallplan bei Algorithmenversagen | §17.6 |
| P-14 | Volltextprüfung aller Quellen mit Status „Entwurf“ oder „Sekundärquelle“ (EDPB 02/2026, Pseudonymisierung 01/2025, AG Lörrach, OLG Dresden) | vor jeder Außenaussage |
| P-15 | Checkpoint-Policy als Schicht und Haltefrist | O-15 |
| P-16 | Restinformation im Graph bis zum Neuaufbau, Frist | §8.5 |
| P-17 | CRA, AI Act, NIS2-Lieferkette | offen |
| P-18 | AVV, Drittland, Beweiswert bei K3-Verwahrern | §12.1 |
| P-19 | CE-Bedingungen NIST SP 800-88 r2 (Norm, nicht nur FAQ) | Volltext ausstehend |
| P-20 | Weitere Betroffenenrechte (Auskunft, Berichtigung) | außerhalb dieser SPEC |

### 23.2 Offene technische Punkte (O)

| ID | Frage | Status |
|---|---|---|
| O-03 | Waypoint-Modus: Recall-Messung | gesperrt |
| O-12 | Dauer und Zulässigkeit von `Restricted` | Rechtsfrage P-09 |
| O-15 | Standard-`checkpoint_policy` | Standard `Rewrite`, Haltefrist offen |
| O-24 | Budgets (Takt 1, Re-Wrap, Padding, Purge-Schreibverstärkung) | Benchmarks |
| O-34 | Zeitstempel-Anbieter mit Offline-Verifikation | Auswahl |
| O-35 | Zentrales Löschregister als Mirror-Ziel | Betreiberentscheid |
| O-37 | Spike S-K2: Plattformen für Konformitätssuite K2 | Verifikation |
| O-38 | Spike S-HRD: plattformspezifische Härtung | Verifikation |
| O-39 | Standardwerte `kek_epoch_len`, `key_backup_max_age`, `purge_rebuild_max_days` | Rechtsprüfung P-06, P-11 |
| O-41 | Status DIN 66398 und Version SDM Baustein 60 | Prüfung |
| O-42 | Eignung von `R-HNSW-PERSIST`: Liegen Vektoren im Persistenzformat im Klartext? | **Vor WP L-6 prüfen** (Befund aus Analyse, nicht verifiziert) |
| O-43 | Ledger-Dateisystem-Verhalten (CoW, Netz, FUSE) und Erkennung | §17.2 |

### 23.3 Übergangsregeln (Ersetzungstabelle)

| Stelle in v2.0 | Wirkung in v3.0 |
|---|---|
| §3.2 Punkt 2 (P-Daten pro Subject+Document) | ersetzt durch §3.2, §4.1 (`K_subj`) |
| §4.1 Hierarchie | ersetzt durch §4.1 (zufällig, gewrappt) |
| §4.2 Algorithmen (HKDF für KEK) | ersetzt durch §4.2 |
| §4.3 Bindungsregeln | präzisiert durch H2 (§4.1) |
| §5.2 Zustände | ersetzt durch §6.2 |
| §5.3 Sechs Schritte | ersetzt durch §6.3–6.5; Schrittnummern entfallen |
| §5.4 Klassen K, I, M | ersetzt durch §12.1 |
| §5.5 Recovery | ersetzt durch §6.6 |
| §5.6 P-Klasse | ersetzt durch §4.1, T-DEL-9 |
| §5.7 Restrict-Modus | präzisiert durch §6.7 |
| §6 ReadGate | präzisiert durch §7 (Zeitreisen, Snapshots) |
| §7 ErasureLayer | ersetzt durch §9.1 (`SnapshotRevoker` entfällt) |
| §8 Lineage | unverändert; Kaskade in §6.5 |
| §9 Zeitreise | §9.2 Regel 3 bleibt; §7.3 ergänzt |
| §10.1 CommitBarrier | präzisiert durch §10.3 |
| §10.2 PurgeBarrier | präzisiert durch §10.3, §10.4 |
| §11.1 Fehler-Hierarchie | ersetzt durch §11.1 |
| §12 Testkatalog | ergänzt durch §19 (T-DEL-16 bis 35) |
| §13 Invarianten | ergänzt durch §20 |
| §14 Konfiguration | ersetzt durch §18 |
| §15 Layering-Gate | ersetzt durch §21 (`layering`, `ownership` …) |
| §16 Glossar | ergänzt durch §24 |

### 23.4 Umsetzungsreihenfolge

Die Crate-Specs legen Arbeitspakete fest. Die Reihenfolge ist normativ: Phase 0 Härtung, Phase 1 Erasure Lab, Phase 2 Löschkette. Ohne Erasure Lab (Phase 1) darf Phase 2 nicht als „fertig“ gelten (ER-E20 / Gate G0).

---

## 24 Glossar

| Begriff | Definition |
|---|---|
| **Canary** | synthetisches Muster zur Kalibrierung des Scanners; nie Nutzdaten (§12.4) |
| **Crypto-Shredding** | Vernichtung des Schlüssels macht Chiffrat unlesbar, ohne Überschreiben jeder Kopie |
| **DocDek** | Data Encryption Key je Löscheinheit |
| **DocId** | 128-Bit-BLAKE3-keyed-ID eines Dokuments; im Store der einzige Dokumentbezug |
| **Epoch Seal** | Re-Wrap aller Schlüssel einer Collection unter neuer Epoche und Vernichtung der alten |
| **ErasedSet** | durable Menge der gesperrten Löschziele (Referenz-Hashes) |
| **Gate / ReadGate** | synchrone O(1)-Lesesperre vor jedem Zugriff auf Chiffrat |
| **K_coll, K_subj, K_tenant** | zufällige, gewrappte Schlüssel für Collection, Betroffenen, Mandanten |
| **KeyAnchor** | Port für die Wurzelverwahrung (Custody K0–K3) |
| **KeyTable** | Speicher der gewrappten Schlüssel mit Ersetzen-Semantik |
| **Lineage** | Herkunftsverweis abgeleiteter Daten auf ihr Quelldatum |
| **Löscheinheit** | kleinste Menge von Records, die ohne Kollateralverlust gelöscht werden kann |
| **Ebene A / Ebene B** | Klartext-Strukturen / persistente Chiffrat-Reste (§5.1) |
| **Mixed-Segment** | Segment mit Betroffenen, die nicht alle zu löschen sind |
| **Positivkontrolle** | Nachweis, dass der Scanner ein bekanntes Muster findet, bevor er einen Negativbefund meldet |
| **PersonalRef** | 128-Bit-Verweis auf P-Daten in K-Dokumenten |
| **Saga** | die Löschsequenz mit Journal, Zuständen und Recovery (§6) |
| **SubjectRef** | 128-Bit-Zufalls-ID eines Betroffenen; nie im Klartext gespeichert |
| **Wrap-Nonce** | 96-Bit-Nonce eines gewrappten Schlüssels; dient als Suchmuster im Nadelscan |
| **Vault** | historischer Begriff für den Schlüsselspeicher; ersetzt durch `KeyTable` und `KeyAnchor` |
