# Acta – Finaler Implementierungsplan (entschieden)

*Principal-Rust-Engineering · Entscheidungsstand 2026-11-15 · Normativ: `acta/docs/SPEC.md` Fassung 3.0 · Quelle: `tfufuz1/contextra`, HEAD `50cdc3d` (gepinnt) · Vorgänger: `Implementierungsplan_Contextra___Acta.md` (Entwurf mit offenen ADR A1–A7)*

**Beweismarken:** **[V]** am Klon verifiziert · **[S]** aus der SPEC · **[D]** Entscheidung dieses Plans · **[A]** Annahme, bestätigungspflichtig

**Grenzen der Analyse (ehrlich vorweg):** Statische Analyse (Manifeste, Quelltext, Grep). In der Analyseumgebung ist keine Rust-Toolchain vorhanden; **nichts wurde kompiliert oder getestet**. Alle Code-Skizzen sind Entwurf und noch nicht vom Compiler geprüft. LOC-Zahlen sind Zeilen in `src/**/*.rs`. Aufwandsklassen (S/M/L/XL) sind relativ, keine Personenzeit.

---

## 0. Was dieser Plan gegenüber dem Entwurf leistet

Der Entwurf ließ sieben Entscheidungen (A1–A7) offen. Dieser Plan **entscheidet alle**, ergänzt **fünf weitere**, die sich bei der Nachprüfung ergeben haben (D-01, D-11, D-12, D-13, D-14), und korrigiert **sechs Aussagen** des Entwurfs bzw. der SPEC, die am Klon nicht tragen (Abschnitt 2). Jede Entscheidung nennt Alternativen, Begründung, den Preis (Kompromiss) und den Test, der sie absichert.

Die Mission (souveränes Unternehmenswissen, Transparenz, Präzision/Kontrolle, Effizienz, Schutz personenbezogener Daten, Datenverwaltung, Entscheidungshilfe, europäischer Datenschutz) ist in Abschnitt 9 auf Komponenten und messbare Abnahmen abgebildet.

---

## 1. Entscheidungsübersicht

| ID | Entscheidung | Kernbegründung | Preis |
| --- | --- | --- | --- |
| D-01 | **Level-Modell:** Ringe bleiben Dokumentationsgruppen, Gate G1 erzwingt numerische Levels (Kante nur zu strikt niedrigerem Level) | SPEC-Ringregel verbietet jede Querkante, widerspricht aber der eigenen Port-Tabelle und der Realität (`crypto→ids/env`, `store→mvcc`) | SPEC-Patch nötig; Gate etwas komplexer |
| D-02 | **Hubs:** nur `acta-types` und `acta-ports`, hart budgetiert; zusätzlich kleine **Vertrags-Leaf-Crates** (`acta-infer-api`); `acta-engine` ist Verbraucher, kein Hub (Fan-in-Limit) | Contextra-Hubs sind an Größe und Fremdwissen gescheitert | Mehr Crates, mehr `Cargo.toml` |
| D-03 | **Trait-Besitz nach Richtung** der Abhängigkeit, nicht pauschal „Verbraucher besitzt“ | Pauschale Verbraucher-Regel erzeugt Aufwärtskanten (Adapter → Engine) | Regel muss gelernt werden |
| D-04 | **Fehler je Crate**, gemeinsame Abstraktion nur `ErrorClass` + `ClassifiedError`; Aggregation erst in Level ≥ 6 | Contextra: 1 177-Zeilen-Monolith, jede `From`-Kante macht `types` abhängig | `From`-Boilerplate in Aggregatoren |
| D-05 | **AEAD: XChaCha20-Poly1305**, Umschlag v3, Nonce nur intern aus fallibler `CryptoRng`; Algorithmus-Byte für Agilität | SPEC §4.1/§4.4; I-15; 192-Bit-Zufallsnonce erlaubt sorgloses Zufallsziehen | Kein FIPS-Algorithmus; keine Nonce-Misuse-Resistenz |
| D-06 | **Laufzeit:** Kern (Level 0–6) synchron mit Threads, kein `tokio`; async nur an den Rändern | SPEC §10.1 (kein `.await` unter Locks), Loom- und Crash-Testbarkeit | Brücken am Rand; Contextra-Store ist async-first, daher Neubau |
| D-07 | **Commitments: HMAC-SHA256**; Verifier-sichtbare Hashes SHA-256; BLAKE3 nur intern | SPEC §4.1 und §6.6 widersprechen sich; ein Primitivsatz hält `acta-verify` ≤ 800 LOC | Etwas langsamer als BLAKE3 (irrelevant bei Ledger-Raten) |
| D-08 | **Ein Namensraum `acta/<zweck>/v<N>`**, Magic `ACTAFIL\0`, Report-Präfix `acta/report/v1` | Greenfield, keine Altformate; SPEC mischt `acta/…` und `ctx/…` | Contextra-Golden-Dateien byteweise nicht wiederverwendbar |
| D-09 | **Tags:** Tenant/Collection/Subject **deterministisch keyed** (HMAC unter `k_idx`-Teilschlüssel), `doc_tag` **zufällig** | Subject-Auflösung ohne Mapping-Tabelle; Mapping-Tabelle wäre selbst Personendatum | `k_idx`-Kompromittierung erlaubt Wörterbuchangriffe auf externe IDs |
| D-10 | **Mission-Module:** `acta-pii`, `acta-prompt-guard`, `acta-ingest`, `acta-router` als Leaf-Crates; Inferenz-Clients als Randcrates; Egress-Guard als Dekorator | Ringregel + Fail-Closed-Egress; Untrusted-Parser isolieren | Zusätzliche Verdrahtung in `acta-node` |
| D-11 | **Abgeleitete Segmente sind at rest versiegelt** (Segment-Schlüssel, Slot E), auch im Modus `Rebuild` | S2 verlangt AEAD für *jeden* persistierten Klartext eines Betroffenen; SPEC §6.5 verbietet unverschlüsselte Embeddings; §4.6 und §2.1 widersprechen dem | Segmente müssen in RAM entschlüsselt werden (kein `mmap` im Klartext) |
| D-12 | **Persistenzformate handgeschrieben und sicher** (`from_be_bytes`, geprüfte Slices), kein `bincode`/`serde` auf Platte | Format muss spezifiziert und goldengetestet sein, nicht aus Rust-Typen folgen | Mehr Code; dafür stabile, auditierbare Formate |
| D-13 | **Keine Code-Abhängigkeit zu Contextra.** Differentialtests laufen gegen **eingefrorene Fixtures** | Bewegliches Ziel (Contextra wird gerade entkoppelt); Hubs würden einsickern | Fixtures können unvollständig sein; Nachziehen ist Handarbeit |
| D-14 | **Bestehende `acta/`-Crates sind Entwurf.** `acta-crypto` wird zerlegt (`kv_segment`, `compat/` raus) | Verifiziert: 4 648 LOC, davon 1 642 `kv_segment`, plus `compat/`, `wal_crypto` (1 163), `crypto.rs` (1 041) | Verlust der „fertigen“ 30 k LOC |
| D-15 | **Store:** Neubau in drei Stufen (Skeleton → Datenmodell → Fähigkeiten); Contextra-Tests als triagiertes Orakel | 32 015 LOC async-first, Klartext-Schlüssel, unverdrahtete Fence | Größtes Einzelrisiko |
| D-16 | **Walking-Skeleton-Index: `acta-text`** | Kein `unsafe`, einfachste Lineage, BM25/WAND/Morphologie lassen sich portieren | Vektor-Löschung wird erst in R5 bewiesen |
| D-17 | **`contextra-db` wird nach Anwendungsfall zerlegt**, nicht verschoben; `acta-engine` ≤ 12 k LOC, Fan-in ≤ 2 | Contextra verschiebt `engine` in `db` (23 313 LOC) und baut den Hub neu | Strengeres Budget; Splitting-Trigger nötig |
| D-18 | **Gates als Rust-`xtask`**; Python-Werkzeuge aus Contextra sind Prototypen/Orakel | SPEC §8.5; hermetische CI | Einmalaufwand |
| D-19 | **Scope-Schnitt:** DEFER `acta-wire`, `acta-kvcache`, `acta-sandbox`, `acta-agent`, `acta-cognition`, `acta-mcp`, DiskANN, `acta-py`; DROP `acta-license`, `types_local`, `compat/`, `engine_local` | Fokus auf SPEC-Kern und Mission-Mindestumfang | Weniger Feature-Breite zum Start |
| D-20 | **Toolchain 1.89.0 gepinnt, Edition 2024, `unsafe` nur in `sys`/`simd`/`wire`** | SPEC §3.2; Greenfield erlaubt aktuelle Edition | Lint-Anpassungen |

---

## 2. Korrekturen am Entwurf und an der SPEC (am Klon verifiziert)

| # | Befund | Beleg [V] | Folge |
| --- | --- | --- | --- |
| K-1 | **SPEC-Ringregel ist nicht erfüllbar.** Sie verbietet Querkanten im Ring, aber `store→mvcc` und `vector→mvcc` (beide R1) sind in Contextra Manifestkanten; `crypto→ids/env` (R0) ebenso | `contextra-store/Cargo.toml`, `contextra-vector/Cargo.toml`; `acta-crypto` nutzt `acta-ids`/`acta-env` | D-01 |
| K-2 | **Pauschales „Verbraucher besitzt den Trait“ erzeugt Aufwärtskanten:** Inferenz-Adapter (unterhalb) müssten `acta-engine` (oberhalb) importieren | Entwurf 6.3 HR-4 + 7.6 | D-03, D-10 |
| K-3 | **S2 ist mit Klartext-Postings und -Vektoren auf Platte nicht erfüllbar.** SPEC §2.1/§4.6 lässt `Rebuild`-Segmente schlüssellos, §6.5 verbietet unverschlüsselte Embeddings, S2 verlangt AEAD für jeden persistierten Klartext | SPEC §1.2 S2, §2.1, §4.6, §6.5 | D-11 |
| K-4 | **`acta-rank` ist nicht „autark“:** Manifest nennt `ports` und `types` (ggf. optional) | `contextra-rank/Cargo.toml` | Kanten beim Port entfernen |
| K-5 | **`acta-crypto` enthält mehr als Krypto:** `kv_segment/` (1 642 LOC, Store-Logik), `compat/`, `wal_crypto.rs` (1 163), `anti_tamper.rs`, `contract.rs` | `acta/crates/acta-crypto/src` | D-14 |
| K-6 | **SPEC-Widersprüche in Primitiven:** Commitment BLAKE3 (§4.1) gegen HMAC-SHA256 (§6.6); Anhang A.1 zeigt `blake3:` für `scope_commitment`; Domänenpräfixe `acta/…` gegen `ctx/…` | SPEC §4.1, §6.6, §4.5, Anh. A.1 | D-07, D-08 |

Bestätigt [V]: A-01 (`aead.rs` nutzt AES-256-GCM-SIV, 12-Byte-Nonce, Präfix `ctx/rec/v2`), A-02 (`aead.rs:71` `unwrap_or_default()`), A-03 (`dek.rs:87` `let _ = hkdf_expand(…)`), A-05 (`Rng::fill_bytes` unfehlbar), A-10 (`SYSTEM = DEFAULT = INVALID = 0`, öffentliches Tuple-Feld), V-01 (`sealer.rs:101` `[0x55u8; 32]`), V-08 (`enter_purge` in `lsm/fence.rs:59` ohne Aufrufer). Die async-Zahl des Entwurfs (710) und meine Grep-Zählung (ca. 3 092 Zeilen mit `async fn`, inkl. Tests) weichen ab; sie bestätigt in jedem Fall, dass der Contextra-Kern async-first ist.

---

## 3. Zielarchitektur

### 3.1 Level-Modell (D-01)

**Regel:** Eine Abhängigkeit `A → B` ist erlaubt genau dann, wenn `level(A) > level(B)`. Die Level stehen maschinenlesbar in `acta/rings.toml`; `xtask gates` (G1) liest sie und `cargo metadata`. Die SPEC-Ringe R0–R7 bleiben als Dokumentationsgruppen erhalten; jede nach SPEC legale Kante bleibt legal.

| Level | Ring | Crates | Zulässige Abhängigkeiten (Beispiele) |
| --- | --- | --- | --- |
| L0 | R0a | `acta-ids`, `acta-env` | keine internen |
| L1 | R0b | `acta-crypto`, `acta-sys`, `acta-durable-fs`, `acta-simd`\*, `acta-infer-api` | ids, env |
| L2 | R2a | `acta-types` | ids |
| L3 | R2b | `acta-ports` | types, ids, env |
| L4 | R1a/R3 | `acta-mvcc`, `acta-rank`, `acta-erasure`, `acta-text`, `acta-vector`, `acta-graph`, `acta-checkpoint`, `acta-pii`, `acta-prompt-guard`, `acta-ingest`, `acta-router` | ports, types, crypto, durable-fs, sys, simd, ids, env |
| L5 | R1b | `acta-store` | zusätzlich `acta-mvcc` |
| L6 | R4 | `acta-erasure-adapters` | erasure, store, text, vector, graph, checkpoint, mvcc |
| L7 | R5 | `acta-chronicle`, `acta-engine`, `acta-privacy` (Egress-Guard), `acta-infer-*` (Randcrates), `acta-audit-export`, `acta-avv-generator` | alles darunter, aber nicht untereinander |
| L8 | R6 | `acta-node` | alle (Composition Root) |
| L9 | R7 | `acta`, `acta-cli` | node |
| – | R7 | `acta-verify` | **keine** Acta-Abhängigkeit (SPEC §8.4) |

\* `acta-wire` (unsafe-Insel) wird erst mit CLI/IPC gebraucht (DEFER, D-19).

**Warum nicht die SPEC wörtlich?** Wörtlich umgesetzt müsste man `mvcc` in den Store einbauen oder Kopien halten (neue Duplikate, Gate G-DUP schlägt an). Levels lösen das ohne Zyklus und bleiben automatisch prüfbar.
**Alternative verworfen:** Querkanten per Ausnahmeliste erlauben. Das ist der Weg, auf dem Contextra seine 61 Kantenverstöße (Baseline 2026-11-08) angesammelt hat.
**Preis:** SPEC-Patch (§3.1, §8.5 G1), eine zusätzliche Konfigurationsdatei.
**Test:** `xtask gates g1` mit negativen Fixtures (absichtlich verbotene Kante muss rot werden).

### 3.2 Hub-Regeln (D-02, D-03)

**Hubs sind genau zwei:** `acta-types` (L2) und `acta-ports` (L3). Es gibt weder `engine`- noch `db`-Hub.

| Regel | Inhalt | Gate |
| --- | --- | --- |
| HR-1 | Nur `types` und `ports` sind Hubs; Vertrags-Leafs (`acta-infer-api`, ≤ 500 LOC, ein Zweck) sind erlaubt, wachsen nicht | G-HUB-1 |
| HR-2 | `acta-types`: nur Werte, Referenzen, Taxonomien (`ErrorClass`), `Lineage`/`Derived<T>`, Filter-AST. Keine Algorithmus-Konfiguration, keine Fremdfehler | G-HUB-3 |
| HR-3 | `acta-ports`: nur Traits, bei denen Erbringer und Nutzer **mindestens drei Level auseinanderliegen** und ≥ 2 Nutzer **und** ≥ 2 Erbringer oder ein gesetzlich/normativ benannter Port (SPEC §3.3) existieren | G-HUB-2 |
| HR-4 | **Trait-Besitz nach Richtung:** (a) Erbringer liegt **über** dem Nutzer → Nutzer besitzt den Trait (Dependency Inversion), z. B. `ErasureLayer` im Löschkern, implementiert in `acta-erasure-adapters`. (b) Erbringer liegt **unter** dem Nutzer → Erbringer besitzt ihn, z. B. `Vfs` in `acta-durable-fs`. (c) Weder noch (Nutzer und Erbringer auf Geschwisterlevels oder beide Seiten brauchen ihn) → kleinstes mögliches Vertrags-Leaf auf dem niedrigsten tragfähigen Level | G-PORT-OWNER |
| HR-5 | Rule of Two: Element erst im Hub, wenn ≥ 2 Crates es nachweislich nutzen | G-HUB-2 |
| HR-6 | Ein Fehlertyp je Crate (`thiserror`, `#[non_exhaustive]`, kein `Other(String)`) | G-ERR |
| HR-7 | `default = []`, Features rein additiv, keine Rückkante nach oben, keine `compat/`-Brücken | G-FEAT |
| HR-8 | Keine Alias-Re-Exports (`pub use x as y`), kein `include!`, kein `#[deprecated]`, keine Shims | G-SHIM |
| HR-9 | Die Fassade `acta` enthält keine Logik; `acta-node` ist die einzige Stelle, die konkrete Crates verdrahtet (G2: jeder Port genau einmal verdrahtet) | G1, G2 |
| HR-10 | Typinvarianten wohnen im besitzenden Crate (`Sealed<P>` in `acta-crypto`, nicht im Hub) | Compile-Fail-Tests |
| HR-11 | **Fan-in-Limit:** `acta-engine` darf von höchstens 2 Crates abhängen lassen (`node`, `acta`); kein Crate ab Level 7 darf ein anderes Level-7-Crate sehen | G-HUB-4 |

**Hub-Budget (hart, Änderung nur per ADR):** `acta-types` ≤ 3 000 LOC; `acta-ports` ≤ 1 500 LOC und ≤ 12 öffentliche Traits. Zum Vergleich Ist-Stand: types 7 510 LOC, ports 6 755 LOC mit 43 Traits.

**Warum feste statt „zu kalibrierende“ Budgets?** Ein Budget, das nach der ersten Messung verschoben werden darf, ist kein Gate. Die Startwerte sind bewusst knapp; wer sie reißen will, muss eine ADR schreiben und damit die Frage „gehört das wirklich in einen Hub?“ beantworten.

### 3.3 Verteilung der heutigen Hub-Inhalte (final)

| Heute (types / ports) | Ziel | Grund |
| --- | --- | --- |
| `error.rs` (46 Varianten, 1 177 Zeilen) | Fehler je Crate; `acta-types` behält `ErrorClass`, `ClassifiedError` | D-04 |
| `PprConfig`, `TlHfdParams` | `acta-graph` | Algorithmus-Konfiguration gehört dem Algorithmus |
| `DistanceMetric` | `acta-simd` | Eigentümer der Kernel |
| `TOMBSTONE_BIT`, `MAX_REF_LEN`, Tags, `DocOrd`, `DataClass` | `acta-ids` | Formatkonstanten; `acta-crypto` braucht sie für AAD (L1 darf nicht nach L2 sehen) |
| `MAX_SEARCH_K`, `saos.rs`, `RetrievalStrategy` | `acta-engine` | Retrieval-Orchestrierung |
| `SagaRecord`, `ErasureId`, `GateId`, `WorkflowState`, `witness.rs`, `erasure.rs`, `ScanMethod` | `acta-erasure` | SPEC §3.3: Löschtypen kanonisch im Löschkern |
| `tombstone.rs` | entfällt | Bitformat in `acta-ids` |
| ports: `clock`, `rng`, `id_gen`, `metrics` | `acta-env` | SPEC §3.3 |
| ports: `vfs`, `fs` | `acta-durable-fs` | SPEC §3.3 |
| ports: `keys`, `key_material`, `report_signer` | `acta-crypto` | SPEC §3.3 |
| ports: `erasure`, `erasure_legacy`, `scanner`, `ledger`, `conformance` | `acta-erasure` | `acta-erasure-spi` entfällt |
| ports: `barrier`, Lesesicht von `gate`, `ErasedView`, `SnapshotRevoker` | bleiben in `acta-ports` | Erbringer L4, Nutzer L4/L5 und L4 (Löschkern): Taille zwischen Persistenz und Löschkern |
| ports: `embedding`, `inference`, `reranker`, `query_rewriter`, `attention` | `acta-infer-api` (Vertrags-Leaf, L1) bzw. `acta-engine` | D-10 |
| ports: `graph*`, `vector_index`, `text_index`, `kv*`, `storage`, `lifecycle`, `plugin` | Besitz des jeweiligen Nutzers (HR-4) bzw. entfallen (`plugin`, `license`) | Contextra-Seams ohne zweiten Erbringer |
| `ModelFingerprint`, `schema.rs` | `acta-types`, falls ≥ 2 Nutzer (Store-Manifest, Inferenz), sonst Nutzer | HR-5 |

### 3.4 Fehlerarchitektur (D-04)

```rust
// acta-types (L2): einzige gemeinsame Fehlerabstraktion
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorClass { Transient, Integrity, Crypto, Erased, Policy, Invalid }

pub trait ClassifiedError: std::error::Error {
    fn class(&self) -> ErrorClass;
    fn code(&self) -> &'static str;   // stabil, ohne Nutzdaten und ohne IDs
}

// acta-store (L5): geschlossener Fehler; Integrity/Crypto lösen Poisoning aus (SPEC §10.2)
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum StoreError { /* … */ }

// acta-node (L8): Aggregation nur hier
#[derive(Debug, thiserror::Error)]
pub enum NodeError { #[error(transparent)] Store(#[from] StoreError), /* … */ }
```

**Begründung:** `ErrorClass` bindet die SPEC-Taxonomie (§10.2) an einen Typ, ohne dass der Hub Fremdvarianten kennt. **Kompromiss:** Aggregatoren schreiben `From`-Impls selbst. **Test:** Property-Test „jede öffentliche Fehlervariante liefert `class()` und ein nicht leeres `code()`“; Test „`Display` enthält keine Roh-IDs“ (S12).

---

## 4. Entscheidungen im Detail

### D-05 · AEAD und Nonce (ersetzt offene ADR A2)

**Entscheidung:** XChaCha20-Poly1305 (RustCrypto `chacha20poly1305`), Umschlag v3 nach SPEC §4.4, 192-Bit-Nonce **ausschließlich intern** aus einer fehlbaren `CryptoRng` gezogen. `alg = 0x01` ist XChaCha20-Poly1305; `0x02` ist für einen späteren zweiten Algorithmus **reserviert, aber nicht implementiert**.
**Warum:** (1) Die SPEC schreibt es vor und der aktuelle Code weicht ab (A-01). (2) Bei 192 Bit ist Zufalls-Nonce-Kollision kein praktisches Risiko, das ersetzt die Nonce-Misuse-Resistenz von GCM-SIV. (3) Ein Algorithmus-Byte erfüllt die EDPB-Forderung nach Krypto-Agilität (§9.2), ohne heute zwei Algorithmen auditieren zu müssen.
**Alternativen:** AES-256-GCM-SIV (Ist-Stand Acta): nonce-misuse-resistent, aber 12-Byte-Nonce, SPEC-Abweichung, Nonce heute als öffentlicher Parameter (A-04). AES-256-GCM: FIPS-tauglich, aber Nonce-Misuse fatal.
**Preis:** XChaCha20-Poly1305 ist **kein FIPS-140-Algorithmus**. Kunden mit FIPS-Pflicht (typisch Behörden, teilweise Finanzsektor) brauchen dann `alg = 0x02` = AES-256-GCM mit streng gezählter Nonce. **Revisit-Trigger:** erster Kunde mit schriftlicher FIPS-Anforderung. HSM-Key-Wrap erzwingt **nicht** den Record-AEAD, weil der `KeyAnchor` nur Epochenschlüssel schützt.
**Test:** Testvektoren (`draft-irtf-cfrg-xchacha`), Gegenimplementierung im Test, `envelope_is_aead` (jedes manipulierte Byte → Fehler), `T-RNG-1` (Entropiefehler propagiert, kein Umschlag entsteht).

```rust
// acta-env (L0)
pub trait CryptoRng: Send + Sync {
    fn try_fill(&self, dst: &mut [u8]) -> Result<(), EntropyError>;
}

// acta-crypto (L1): Konstruktion nur über seal(); kein Default, kein Clone für Schlüssel
pub struct Dek(Zeroizing<[u8; 32]>);            // kein Clone/Default/Debug

pub struct Sealed<C: PersonalClass> { bytes: Vec<u8>, _c: PhantomData<C> } // Felder privat

pub fn seal<C: PersonalClass>(
    rng: &dyn CryptoRng, dek: &Dek, ctx: &SealCtx<'_>, plain: &[u8],
) -> Result<Sealed<C>, CryptoError>;

pub fn open(dek: &Dek, ctx: &SealCtx<'_>, sealed: &[u8])
    -> Result<Zeroizing<Vec<u8>>, CryptoError>; // Fehler ohne Orakel; Erased separat
```

### D-06 · Laufzeitmodell (ersetzt ADR A3)

**Entscheidung:** Alle Crates bis einschließlich Level 6 sind synchron und nutzen Threads; `tokio` ist dort weder Abhängigkeit noch Dev-Abhängigkeit (Gate G-NOTOKIO). Async existiert nur in Randcrates (`acta-infer-ollama`, `acta-node` für Netzwerkrand, später `acta-mcp`) und ruft den Kern über `spawn_blocking`/begrenzten Thread-Pool.
**Warum:** SPEC §10.1 verbietet, `FileVaultShardLock` über `.await` zu halten; `CommitBarrier` darf nie über I/O gehalten werden. In synchronem Code ist das per Typen-/Lock-Rang prüfbar, Loom-Modelle (T-CONC-1, Flush gegen Purge) sind direkt anwendbar, und die Crash-Matrix mit `FaultVfs` ist deterministisch. Async im Kern bräuchte zusätzlich Cancel-Safety-Nachweise an jedem Durabilitätspunkt (C1–C12).
**Alternative:** async-Store behalten (Contextra-Stand). Spart nur Portierungsaufwand, den wir ohnehin nicht haben, weil der Store neu entsteht (D-15).
**Preis:** Brücken am Rand; hohe Parallelität bei vielen Verbindungen läuft über den Pool, nicht über Tasks. **Revisit-Trigger:** Messung zeigt, dass Pool-Latenz die `erase_return_timeout` (500 ms) gefährdet.
**Absicherung:** `OrderedMutex<const RANK: u8>` (Debug-Assertion + Loom) in `acta-sys` kodiert die Rangfolge aus SPEC §10.1.

### D-07 · Commitments und Hash-Familie (ersetzt ADR A4)

**Entscheidung:** (1) Alle Commitments für Ledger, Widerrufs-Log und `scope_commitment`: **HMAC-SHA256** unter `K_commit[e]` (SPEC §6.6). (2) Alle Hashes, die `acta-verify` nachrechnen muss (Ledger-/Chronik-Kette, Report-Signatur): **SHA-256/HMAC-SHA256/Ed25519**. (3) **BLAKE3** nur für interne, nicht verifier-sichtbare Zwecke (Dedup, Blockprüfsummen).
**Warum:** `acta-verify` ist auf ≤ 800 LOC und null Acta-Abhängigkeiten festgelegt. Ein Primitivsatz (SHA-2-Familie plus Ed25519) hält das Tool klein und auditierbar. Außerdem ist `K_commit` ohnehin Teil des HMAC-orientierten Epochen-Tripels (`K_enc`, `K_vm`, `K_commit`).
**Folge:** Anhang A.1 der SPEC zeigt künftig `hmac-sha256:` statt `blake3:`; Annahme A-CRYPTO listet BLAKE3 nur, soweit intern genutzt.
**Preis:** HMAC-SHA256 ist langsamer als BLAKE3; bei Ledger-Raten (Löschvorgänge, nicht Datensätze) belanglos.
**Test:** `T-LOG-1` (Bytes-Grep über Ledger und Log), Golden-Test Report, `acta-verify` baut ohne Acta-Crates.

### D-08 · Domänen-Präfixe und Magic (ersetzt ADR A5)

**Entscheidung:** Alle Domänen-Strings folgen `acta/<zweck>/v<N>`: `acta/kdf/v3`, `acta/dek/v3`, `acta/rec/v3`, `acta/slotmask/v1`, `acta/report/v1`, `acta/tag/v1`, `acta/commit/v1`. Magic des Dateikopfs: `ACTAFIL\0`. `ctx/…` und `CTXFILE` entfallen.
**Warum:** Es gibt keine Acta-Altdaten; ein Mischnamensraum bietet nur Verwechslungsrisiko. Domänentrennung ist ein Sicherheitsmerkmal, kein Zierrat.
**Preis:** Contextra-Golden-Dateien sind **byteweise** nicht wiederverwendbar (Logik-Orakel bleibt). **Gate:** `G-NOREN` lehnt `"ctx/` in Produktionscode ab.

### D-09 · Herkunft der Tags (ersetzt ADR A6; Annahme [A], mit SPEC-Autor zu bestätigen)

**Entscheidung:**

| Tag | Erzeugung | Grund |
| --- | --- | --- |
| `tenant_tag`, `collection_tag` | HMAC-SHA256(`k_idx`-Teilschlüssel, `acta/tag/v1` ‖ Art ‖ externe ID), auf 16 Byte gekürzt | stateless auflösbar, stabil über Neustarts |
| `subject_tag` | wie oben, Teilschlüssel je Mandant | KeyV2-Cluster und Subject-Auflösung brauchen eine Abfrage ohne Mapping-Tabelle |
| `doc_tag` | 16 Byte aus `CryptoRng`, gespeichert im `DOCMAP` (kind `0x02`) | Dokumente haben oft keine stabile externe ID; Zufall verhindert Verknüpfbarkeit |

`k_idx`: 256 Bit, **kein Default**, Systemstart ohne Konfiguration verweigert (SPEC §10.3), bezogen über `KeyAnchor`, nie als Klartext im Konfigurationsfile.
**Alternative:** Alle Tags zufällig, Mapping-Tabelle extern-ID → Tag. Das wäre unverknüpfbarer, aber die Tabelle ist selbst ein Personendatum mit eigenem Löschproblem (Art. 17 träfe die Tabelle) und ein weiterer Index auf P.
**Preis:** Wer `k_idx` besitzt, kann externe IDs testen (Wörterbuchangriff auf z. B. E-Mail-Adressen). Mitigation: `k_idx` im HSM/KMS, Teilschlüssel je Mandant, Tag niemals in Ebene N, Grabstein nullt den Tag (V-11).
**Test:** Compile-Fail (`TenantTag(…)` nicht konstruierbar außerhalb `acta-ids`/`acta-crypto`); `no_default_k_idx`; `tag_never_in_chronicle` (N-Validator).

### D-10 · Mission-Module, Inferenz, Egress (ersetzt ADR A7)

**Entscheidung:**
- **Vertrags-Leaf** `acta-infer-api` (L1, ≤ 500 LOC): `Embedder`, `Generator`, `ModelFingerprint`, Kostenbeschreibung. Beide Seiten hängen davon ab: der Router (L4) liest nur Beschreibungen; die Clients (`acta-infer-candle`/`-onnx`/`-ollama`, L7-Randcrates) implementieren.
- **`acta-router`** (L4): reine, deterministische Routing-Entscheidung (Kaskade klein → groß); **keine** Clients. Entscheidungen, die in die Chronik fließen, sind reproduzierbar (Seed/Regelversion im Eintrag).
- **`acta-pii`** (L4, Detektoren) wird vom **Engine-Pipeline-Schritt** aufgerufen, nicht vom Ingest-Crate (gleiches Level, Querkante verboten). `acta-chronicle` (L7) nutzt `acta-pii` für den N-Validator.
- **Egress-Guard** in `acta-privacy` (L7): implementiert `Generator` als Dekorator; `acta-node` verdrahtet `engine ← guard ← remote client`. Default: **kein Egress**; Remote nur per expliziter Konfiguration und im Bericht als Zone Z4 / A-EXT ausgewiesen.
- **`acta-ingest`** (L4) verarbeitet nicht vertrauenswürdige Dokumente: Ressourcenlimits, Fuzzing von Beginn an, Datenklasse (R-CLS) Pflicht am Ausgang. Prozessisolation (Sandbox) ist DEFER, aber das Crate-Interface (`parse(bytes, limits) -> Chunks`) ist so geschnitten, dass es später in einen Kindprozess wandern kann.
**Warum:** löst K-2 und hält die Ringregel ein, ohne Logik in `acta-ports` zu kippen.
**Preis:** Mehr Verdrahtung in `acta-node`; der Inferenz-Vertrag ist ein eigenes Mini-Crate.
**Test:** `no_egress_default`; `router_is_deterministic`; `ingest_resource_limits`; Fuzz-Targets für jeden Parser.

### D-11 · Abgeleitete Segmente sind at rest versiegelt (neu; **SPEC-Änderung nötig**)

**Befund (K-3):** SPEC §1.2 S2 verlangt, dass *jeder* persistierte Klartext eines Betroffenen in einem AEAD-Umschlag liegt. Text-Postings enthalten Terme personenbezogener Dokumente, HNSW-Segmente enthalten (quantisierte) Embeddings. §6.5 stuft Embeddings als Ebene P ein und verbietet sie unverschlüsselt auf Medien. Gleichzeitig beschreibt §4.6, `Rebuild`-Segmente hätten keinen Einzelschlüssel.
**Entscheidung:** Jedes persistierte Segment, das aus Ebene-P-Dokumenten abgeleitet ist (Text, Vektor, Graph), wird unter einem **Segmentschlüssel** (`ArtKey`, Slot-Typ E) versiegelt. `Rebuild` und `Sealed` sind **Löschstrategien**, keine Speichermodi:

| Strategie | Ablauf nach Gate | Schlüssel |
| --- | --- | --- |
| `Rebuild` (Standard) | Maske über `ErasedOrds` sofort; Ersatzsegment ohne gelöschte Ordinale unter **neuem** ArtKey schreiben; atomarer Austausch; **danach** alten ArtKey vernichten und Altdatei scrubben | Fremdbetroffene bleiben verfügbar (F-5) |
| `Sealed` (≤ `segment_max_vectors` = 50 000) | ArtKey sofort vernichten; Segment sofort unlesbar; Rest wird aus Ebene P neu aufgebaut | Verfügbarkeitsverlust akzeptiert (statische Archive) |

Segmente, die ausschließlich aus Klasse `Norm`/`Evidence` (Ebene N) stammen, bleiben unversiegelt.
**Warum:** (1) S2 wird wörtlich erfüllt. (2) Backups und Kopien von Indexdateien (Phase h, `CopyRegistry`) werden mit der Vernichtung des alten ArtKeys mit unlesbar, ohne jede Kopie zu finden. (3) Das Restfenster zwischen Gate und Austausch enthält nur Chiffrat statt Klartext; das MK2-Messergebnis (Rohbyte-Scan) wird stabiler.
**Alternative:** `Rebuild`-Segmente im Klartext (SPEC wörtlich). Einfacher und `mmap`-fähig, aber S2 verletzt, Vec2Text-Risiko offen, Backups enthalten Klartext-Indizes.
**Preis:** Segmente müssen zum Laden entschlüsselt und im Arbeitsspeicher gehalten werden (kein Klartext-`mmap`); Kaltstart und RAM-Bedarf steigen; Klartextpuffer liegen in `mlock`-Speicher (A-RAM). **Revisit-Trigger:** Benchmark zeigt, dass Ladezeit die Verfügbarkeitsziele reißt; dann segmentweises Lazy-Decrypt statt Vollladen.
**Test:** `hnsw_no_residue_after_erase`, `text_segment_bytes_have_no_terms` (Bytes-Grep mit Nadel), `rebuild_swap_then_old_key_destroyed`, Crash C7/C8 für den Austausch.

### D-12 · Persistenzformate (neu)

**Entscheidung:** KeyV2 (78 Byte), Tresor-Slot (64 Byte), WAL-Frame v2, SSTable-5-Footer, Manifest v3 werden **von Hand** kodiert und dekodiert (`u32::from_be_bytes`, `split_first_chunk`/`get(..)`, keine Indexierung, vereinbar mit `clippy::indexing_slicing`). `serde_json` ausschließlich für den Bericht (JCS, RFC 8785). `bincode` und `serde` sind für Plattenformate untersagt (Gate G-NOBINCODE).
**Warum:** Ein Plattenformat ist eine Schnittstelle über Jahre; sie darf nicht aus Struktur-Layouts oder Crate-Versionen folgen. Handkodierung ist außerdem frei von `unsafe` (D-20) und goldentestbar. Der Pflegestatus von `bincode` 1.x ist unabhängig davon prüfungsbedürftig (`cargo audit`).
**Preis:** Mehr Code; dafür stabile, audierbare Formate. **Test:** Golden-Tests je Format, Proptest-Roundtrip, Fuzz-Target je Parser (WAL, Manifest, SST-Footer, Tresor).

### D-13 · Verhältnis zu Contextra (neu)

**Entscheidung:** (1) Acta hat **keine** Pfad-/Git-Abhängigkeit zu Contextra. (2) Differential- und Golden-Tests laufen gegen **eingefrorene Fixtures**, einmalig aus `50cdc3d` per Generator-Skript außerhalb des Workspace erzeugt und eingecheckt (Muster existiert in `acta-ids/tests/differential_ids.rs`, `tests/legacy_reference/`). (3) Je Crate gibt es `UPSTREAM.md` (Quellpfade, Commit, Klasse, Begründung). (4) Pro Iteration ein `git diff 50cdc3d..HEAD` auf die im Ledger genannten Pfade; Sicherheitsfixes werden bewusst **nachgebaut**, nie gemergt.
**Warum:** Contextra entkoppelt sich noch (Baseline 2026-11-08: 61 Kantenverstöße, Hub-Abbau Schritte 1, 2, 3 und 5 offen). Eine Live-Abhängigkeit würde dessen Zwischenstände (Re-Exports, Brücken, Shims) in Acta einschleppen und das Ziel „keine Fehler übernehmen“ untergraben.
**Preis:** Fixtures können Verhalten übersehen; Nachziehen ist Handarbeit. **Mitigation:** Fixture-Katalog als Teil von `UPSTREAM.md`, Review bei jedem Diff-Lauf.

### D-14 · Disposition der vorhandenen `acta/`-Crates (neu)

| Crate | Entscheidung | Konkrete Maßnahme |
| --- | --- | --- |
| `acta-ids` | REWRITE (Teile) | 16-Byte-Tags, `DocOrd(u32)`, `DataClass`, `TOMBSTONE_BIT`; keine öffentlichen Felder; `SYSTEM`, `DEFAULT`, `INVALID` getrennt bzw. `DEFAULT` entfernt; Differentialtests behalten |
| `acta-env` | HARDEN | fallibler `CryptoRng`; Eigentümer von `Clock`, `Rng`, `IdGen`, Metrics (408 LOC auf Notwendigkeit prüfen) |
| `acta-crypto` | REWRITE (Kern) + Zerlegung | Siehe unten |
| `acta-types` | REWRITE | Verteilung 3.3, ≤ 3 000 LOC |
| `acta-ports` | REWRITE | ≤ 1 500 LOC, ≤ 12 Traits; `erasure_legacy.rs` entfällt |

`acta-crypto` im Detail [V]: `aead.rs`, `dek.rs`, `kdf.rs`, `secret.rs`, `error.rs` → neu aus SPEC (Clean-Room, siehe unten). `crypto.rs` (1 041 LOC), `contract.rs`, `anti_tamper.rs`, `wal_completeness.rs` → einzeln auf SPEC-Bezug prüfen, Übernahme nur mit Test. `wal_crypto.rs` (1 163) → Logik gehört in `acta-store` (WAL-Frame v2, HMAC-Kette), dort neu. `kv_segment/` (1 642) → DEFER nach `acta-kvcache` (Algorithmuswechsel nötig, kein R0-Thema). `compat/` → DROP. `audit_chain.rs` (374) → wird zur **einen** generischen `HashChain` (Ledger und Chronik), geprüft und getestet; die Duplikate in `agent` und `privacy` entfallen.
**Clean-Room-Regel für Krypto/Tresor/Umschlag/Schlüsselraum:** Autor schreibt aus SPEC-Abschnitt und Testvektoren; Review (vier Augen) prüft gegen die Defektliste, nicht gegen Altcode.
**Preis:** Die vorhandenen ca. 30 k LOC zählen nicht als Fortschritt; die Tests (insbesondere Differentialtests in `acta-ids`) bleiben Orakel.

### D-15 · Store-Neubau in drei Stufen

**Entscheidung:** `acta-store` wird nicht portiert. Stufe 1 (R2): Minimalpfad (WAL v2, Memtable, reduzierter KeyV2, Siegelung vor WAL). Stufe 2 (R3): vollständiger KeyV2 mit `cluster = subject_tag`, Partitionen N/P/D, Manifest v3, SSTable 5, `doc_ord`. Stufe 3 (R4): `FlushPurgeFence` verdrahtet, gefilterter Rewrite und Datei-Drop, V-17 behoben.
**Test-Triage:** Die 160 Contextra-Testdateien werden in drei Körbe sortiert: *übertragbar* (Verhalten laut SPEC, z. B. SSI-Konflikte, Iteratoren), *anzupassen* (Schlüsselformat, Async), *verwerfen* (schreiben Defekte fest, z. B. ASCII-Schlüssel, Dummy-Sealer). Nur Korb 1 und 2 werden Orakel.
**Warum:** V-06/V-07/V-08 sind im Strukturkern (Schlüsselformat, Partitionierung, Fence-Verdrahtung); ein Port würde jeden Test anfassen und gleichzeitig Defekte mitnehmen.
**Preis/Risiko:** XL, größtes Einzelrisiko. **Mitigation:** vertikaler Durchstich zuerst; Formate früh einfrieren (Golden); V-17 wird als Pflichtfall mit Ein-Kern-CI-Job abgesichert.

### D-16 · Index im Walking Skeleton: `acta-text`

**Entscheidung:** `acta-text` ist der erste abgeleitete Index. Algorithmen (BM25, WAND, deutsche Morphologie: 0 Contextra-Referenzen außerhalb des Crates) werden nach Audit übernommen; Segment-/Persistenzschicht, Lineage (`Derived<T>`, `source_set`) und Versiegelung (D-11) sind neu.
**Warum:** keine `unsafe`-Blöcke (Vektor: 10 laut grober Zählung), Posting-Lineage ist die kleinste vollständige Ableitung, und die deutsche Morphologie ist ein echtes Mission-Merkmal (Souveränität, Präzision).
**Alternative:** Vektor zuerst. Zeigt das härtere Löschproblem früher, bringt aber SIMD, `unsafe` und HNSW-Rebuild in den Durchstich, wo das Schnittstellenrisiko Löschkern ↔ Store bewiesen werden soll.
**Preis:** Die Vektor-Löschung bleibt bis R5 unbewiesen; das ist akzeptiert, weil das Muster (Lineage + Gate + Rebuild) dann bereits steht.

### D-17 · Zerlegung von `contextra-db`

| Contextra `db` | Acta-Ziel |
| --- | --- |
| `collection/` (12 550) | `acta-engine::{ingest, retrieval, crud}`; Datenklasse bei jedem Schreibvorgang, Gate vor jedem Lesepfad |
| `transaction/` (1 658) | `acta-engine::tx`; Intent-Journal mit Erasure-Journal abstimmen |
| `chunker/` (1 533), `extraction/` (642) | `acta-ingest` bzw. `acta-engine::chunk`; PII-Hinweise am Chunk |
| `background_workers/` (1 298), `decay_controller.rs` | `acta-engine::maintenance`; Decay nur auf Ebene D und N, **nie** als Löschersatz |
| `contextra_impl/` (1 452), `backends.rs`, `vault_*.inc` | `acta-node` (`.inc`-Includes entfallen, HR-8) |
| `export.rs`, `import.rs`, `filter.rs`, `temporal_filter.rs`, `fusion.rs` | `acta-engine`; Export nur über gegateten Reader |
| `multistep.rs`, `pid_latency_controller.rs`, `homeostat.rs`, `cognition/` | DEFER (`acta-adapt`, `acta-cognition`) |

**Budget:** `acta-engine` ≤ 12 000 LOC; Überschreitung löst einen **Split-ADR** aus (Kandidaten: `acta-retrieval`, `acta-maintenance`), niemals Verlagerung in einen Hub. **Warum:** Contextra hat `engine` aufgelöst, indem es dessen Module nach `db` verschob (15 LOC Fassade, `db` 23 313 LOC). Das Problem wanderte, es verschwand nicht.

### D-18 · Gates in Rust

`cargo xtask gates` ersetzt die Python-Prototypen (`gate_g1_layering.py`, `isolation_proof.py`, `count_bridge_refs.py`), die als Orakel für die Rust-Implementierung dienen. Vollständige Gate-Liste: Abschnitt 8. **Warum:** SPEC §8.5 verlangt die Rust-Variante; Python wäre eine zusätzliche Laufzeit in der CI. **Preis:** Einmalaufwand in R0.

### D-19 · Scope-Schnitt

| Status | Komponenten | Begründung |
| --- | --- | --- |
| **DEFER** | `acta-wire` (bis CLI/IPC), `acta-kvcache` (braucht D-11-konforme Siegelung), `acta-sandbox`, `acta-agent`, `acta-cognition`, `acta-mcp` (nur gegen Fassade, `forget` nur über `ErasureCore::execute`), DiskANN, `acta-py` | nicht Teil des SPEC-Kerns oder durch Kernabhängigkeit blockiert |
| **DROP** | `acta-license` (Contextra-Entscheidung D-08: eingefroren), `types_local` (2 439 LOC), `crypto_local`, `io.rs`, `test_mocks.rs`, `engine_local`-Shims, `compat/` | Duplikate bzw. Brücken |
| **NEW** | `acta-chronicle`, `acta-erasure-adapters`, `acta-node`, `acta`, `acta-cli`, `acta-verify`, `acta-infer-api`, `xtask` | kein Contextra-Pendant |

### D-20 · Toolchain und Lints

Rust 1.89.0 (SPEC, Pin in `rust-toolchain.toml`), **Edition 2024** (Greenfield; Edition ist pro Crate, keine Abhängigkeitswirkung), `rust-version = "1.89"`. `#![forbid(unsafe_code)]` in allen Crates außer `acta-sys`, `acta-simd`, `acta-wire`. SPEC-Lints: `unwrap_used`, `expect_used`, `panic`, `indexing_slicing`, `let_underscore_must_use`. `clippy.toml`: verbotene Aufrufe laut SPEC §3.2 Nr. 5 **ohne** die bisherige `thread_rng`-Ausnahme für Schlüssel, Salt und Nonce (A-12 widerspricht S7). **Preis:** Edition-2024-Lint-Anpassungen in den übernommenen Algorithmus-Crates.

---

## 5. Crate-Katalog (final)

Klassen: ADOPT · HARDEN · REWRITE · NEW · DEFER · DROP. Aufwand: S, M, L, XL (relativ). LOC = Contextra `src/` an `50cdc3d`.

### 5.1 Fundament und Verträge (L0–L3)

| Crate | Level | Quelle (LOC) | Klasse | Entscheidende Punkte | Aufwand |
| --- | --- | --- | --- | --- | --- |
| acta-ids | L0 | ids (720) | REWRITE (Teile) | D-09, D-14 | S |
| acta-env | L0 | env (1 207) | HARDEN | `CryptoRng` fallibel; einziger Eigentümer von Clock/Rng/IdGen | S |
| acta-crypto | L1 | crypto (6 290), `erasure/vault/sealer` (nur als Defektquelle) | REWRITE + Zerlegung | D-05, D-07, D-08, D-14; Hierarchie H1–H5; `KeyAnchor`, `ReportSigner`; Primitive nur RustCrypto | L |
| acta-sys | L1 | sys (2 402), vault-Puffer (529) | HARDEN | SAFETY-Audit; `mlock`, `MADV_DONTDUMP`, `RLIMIT_CORE=0`; `SecureBuf`; `OrderedMutex` | M |
| acta-durable-fs | L1 | durable-fs (1 910) | HARDEN | atomares Rename mit `fsync(dir)`, Scrub+Unlink, `FaultVfs`; **einziger** Dateizugriff im Workspace (G-FS-1) | M |
| acta-simd | L1 | simd (3 627; 287 `unsafe`) | ADOPT nach SAFETY-Audit (R5) | `DistanceMetric` hier | M |
| acta-infer-api | L1 | NEW (aus `ports`) | NEW | D-10; ≤ 500 LOC | S |
| acta-types | L2 | types (7 510) | REWRITE | ≤ 3 000 LOC; `Lineage`, `Derived<T>` | M |
| acta-ports | L3 | ports (6 755) | REWRITE | ≤ 1 500 LOC, ≤ 12 Traits | S–M |
| acta-testkit | dev | testkit (5 446) | HARDEN | `FaultVfs`, `ManualClock`, `SeededRng`, Konformitäts-Suites; früh, weil V-14 an fehlenden Symbolen scheiterte | M |
| acta-wire | L1 | wire (3 234; 45 `unsafe`) | DEFER | erst R7 | S |

### 5.2 Persistenz, Indizes, Löschkern (L4–L6)

| Crate | Level | Quelle (LOC) | Klasse | Entscheidende Punkte | Aufwand |
| --- | --- | --- | --- | --- | --- |
| acta-mvcc | L4 | mvcc (4 858) | HARDEN | `MvccRevoker`; `as_of` nur über `ErasedView`; drei Zeitachsen | M |
| acta-rank | L4 | rank (5 852) | ADOPT (Kanten zu ports/types entfernen) | Determinismus `T-RANK-1`; `exp-*` nicht im Default | S |
| acta-erasure | L4 | erasure (22 910) | REWRITE modulweise (5.3) | keine Abhängigkeit zu Store/Index | XL |
| acta-text | L4 | text (9 535) | HARDEN (Algorithmen), REWRITE (Segmente) | D-11, D-16 | M–L |
| acta-vector | L4 | vector (17 744; 10 `unsafe`) | HARDEN (R5) | Lineage, physisches Nullen, Rebuild, D-11; Embeddings = Ebene P | L–XL |
| acta-graph | L4 | graph (23 593) | HARDEN (R5) | 167 `types`-Verwendungen entfernen; Lineage auf Kanten; `experimental-graph` aus Default | L–XL |
| acta-checkpoint | L4 | checkpoint (5 593) | HARDEN | `CopyRegistry` (V-10); Hardlinks nur über Vfs; kein Wissen über Store/Index (Engine reicht Dateilisten) | M |
| acta-store | L5 | store (32 015; 160 Testdateien) | REWRITE (D-15) | KeyV2, Partitionen N/P/D, Manifest v3, SST 5, Siegelung vor WAL | XL |
| acta-erasure-adapters | L6 | NEW (Wissen aus `docs/erasure/loeschkette_zuordnung.csv`) | NEW | `StoreLayer`, `VectorLayer`, `TextLayer`, `GraphLayer`, `CacheLayer`, `CheckpointLayer`; keine Löschlogik duplizieren | L |

### 5.3 Modulplan `contextra-erasure` → `acta-erasure`

| Contextra-Modul (LOC) | Acta-Ziel | Klasse | Anmerkung |
| --- | --- | --- | --- |
| `vault/` + `vault.rs` (3 577) | `::vault` | REWRITE | Slots U, T, C, S, D, F, E; Epochenmaskierung/Rotation; `destroy_many`; Grabstein nullt `tag` und `payload` (V-11) |
| `gate/` (1 367) | `::gate` | REWRITE | `ErasedOrds` als Roaring-Bitmap, `gate_epoch`, GateLog |
| `journal/` (1 435) | `::journal` | HARDEN | Crash-Matrix C1–C12 |
| `saga/` (1 437), `orchestrator.rs` | `::saga` | REWRITE | E0–E9 nach SPEC §5.3; Pending-Gründe und Backoff; kein Zustand `Failed` |
| `measure/` (1 466) | `::measure` | REWRITE | `Scanner<Unarmed/Armed>`, MK1–MK3 |
| `purge/` (637) | `::purge` | REWRITE | Phasen a–h, `PurgePriority` |
| `report/` (1 555), `deletion_proof.rs` (2 743), `ed25519_proof.rs` | `::report` | REWRITE + Konsolidierung | JCS, Klassen G/K/P/D/C, Annahmen im Bericht |
| `ledger/` (1 538), `revocation_log.rs` (466) | `::ledger` | REWRITE | keyed Commitments (D-07), Epoche, Vernichtung nach 90 Tagen |
| `restore/` (144) | `::restore` | REWRITE | `RestoreGuard`, `T_ROT_BACKUP` = 72 h |
| `scope/` (905) | `::scope` | HARDEN | verschlüsselte Scope-Datei, Single-Flight |
| `supersede`, `replace`, `restricted` (≈ 520) | `::modes` | HARDEN | Art. 16, 18, Replace |
| `kv_shredding.rs` (982) | `acta-kvcache` | DEFER | AES-GCM-SIV → Wechsel nötig |
| `types_local.rs` (2 439), `crypto_local.rs` (274), `io.rs`, `test_mocks.rs` | – | DROP | Duplikate |

### 5.4 Anwendung, Integration, Mission (L4, L7–L9)

| Crate | Level | Quelle (LOC) | Klasse | Anmerkung | Aufwand |
| --- | --- | --- | --- | --- | --- |
| acta-chronicle | L7 | NEW (Wissen aus `crypto/audit_chain`, `agent/audit`, `privacy/context_edit_audit`) | NEW | N-Validator (Whitelist, Entropie, `K_MIN` ≥ 5), `ChronicleEntry`, Hash-Kette, Checkpoint alle 1 024 Einträge, Vier-Augen-Takedown, zufällige Handles | L |
| acta-engine | L7 | db (23 313) | REWRITE (D-17) | ≤ 12 k LOC, Fan-in ≤ 2 | XL |
| acta-node | L8 | NEW (aus `db/contextra_impl`, `backends.rs`) | NEW | Composition Root, Boot-Selbsttest (nicht abschaltbar), `k_idx` ohne Default | M |
| acta, acta-cli | L9 | contextra (2 042) | NEW (dünn) | kuratierte API, keine Logik | S |
| acta-verify | – | NEW | NEW | eigenständig, ≤ 800 LOC, keine Acta-Abhängigkeit | M |
| acta-pii | L4 | privacy (Detektoren) | HARDEN | Namenskollision „Vault“ (Surrogate-Vault gegen Speicherpuffer) auflösen | M |
| acta-privacy (Egress) | L7 | privacy (5 782, Rest) | HARDEN | `Generator`-Dekorator, Default-Deny | M |
| acta-prompt-guard | L4 | prompt-guard (1 389) | ADOPT | Signaturen pflegen; keine Aussage „schützt vollständig“ | S |
| acta-ingest | L4 | ingest (21 140; 2 `unsafe`) | HARDEN | Fuzzing, Dependency-Review (`zip`, `lopdf`, `calamine`), Ressourcenlimits | L |
| acta-router, acta-adapt | L4 | router (8 337), adapt (5 428) | HARDEN / ADOPT | D-10; Router ohne Clients | M |
| acta-infer-candle, -onnx, -ollama | L7 (Rand) | 6 808, 1 016, 6 782 | HARDEN | implementieren `acta-infer-api`; async nur hier erlaubt | M |
| acta-audit-export, acta-avv-generator | L7 | 992, 1 078 | HARDEN | Art.-30-Register und AVV-Vorlagen nur mit Bezug auf Messklassen und Annahmen (G-WORD) | S–M |
| DEFER laut D-19 | – | – | – | – | – |

---

## 6. Qualitätssicherung gegen „Fehler aus Contextra übernehmen“

### 6.1 Defect Firewall (Gate G-NOREN)

Hartes Gate, das den Neueintritt bekannter Contextra-Muster in Produktionscode verhindert:
- `unwrap_or_default()` und `let _ =` auf `Result` in `acta-crypto`, `acta-erasure`, `acta-store`, `acta-checkpoint` (Contextra `src/`: 131 bzw. 648 Treffer ungefiltert, nicht jede Stelle ist ein Fehler, im Krypto-, Store- und Erasure-Pfad ist jede prüfpflichtig)
- Schlüssel-Literale (`[0x..u8; 32]`) außerhalb von `#[cfg(test)]`; `impl Default` für Schlüsselmaterial, `k_idx`, Identitätstypen
- direkte `std::fs::{remove_file, copy, hard_link, rename}` außerhalb von `acta-durable-fs` (Contextra: 57 Stellen, grob gefiltert)
- `pub use … as …`, `include!("*.inc")`, `#[deprecated]`, Verzeichnisse `compat/`
- `"ctx/` als Domänenpräfix; `bincode`; `tokio` in Kern-Crates
- G-WORD: verbotene Aussagen laut SPEC §1.3 (Treffer in Contextra: `erasure/src/journal/scope_purge.rs:25` „unwiederbringlich“, `types_local.rs:571` „Revisionssicherer“; **dieselbe Formulierung steht auch in SPEC §4.3: „unwiederbringlich“ → SPEC-Patch, Abschnitt 11**)

### 6.2 Pipeline je Crate

1. **Spec-Slice** (eine Seite, offene Fragen als ADR-Entwurf) → 2. **Befundkatalog** (V-/A-/K-Befunde + Mustersuche) → 3. **Charakterisierungstests am Orakel** (nur SPEC-konformes Verhalten) → 4. **Redesign** (Typen zuerst: Newtypes, Typestate; dann Algorithmen) → 5. **Differential-/Golden-/Property-/Loom-Tests** → 6. **Gates grün** → 7. **Review** (vier Augen bei Krypto, Erasure, Store-Format; sonst zwei) → 8. **`UPSTREAM.md`**.

### 6.3 Typen, die Fehlerklassen unmöglich machen

| Typ | Verhindert | Beleg-Test |
| --- | --- | --- |
| `Sealed<P>` mit privatem Konstruktor | Klartext-P im Store (V-06, S2) | Compile-Fail `store_rejects_plaintext_p` |
| `Derived<T>` mit nicht leerer `Lineage` | Ableitung ohne Herkunft (S10, V-09) | Compile-Fail `derived_requires_lineage` |
| `Dek`, `K_unit`, `k_idx`: kein `Default`/`Clone`/`Debug` | Null-/Konstantenschlüssel (V-01, V-05) | Compile-Fail `key_has_no_default`; Gate G-CRY-1 |
| `ErasedView` im `RetentionPolicy::may_drop_version` | Historie überlebt Löschung (SPEC §7.3) | `T-TT-1..8` |
| `Scanner<Unarmed>` → `Scanner<Armed>` → `LayerWitness` | Löschnachweis ohne Positivkontrolle (S8) | `T-EVID-1`, `T-STUB-1` |
| Tag-Newtypes mit privaten Feldern | `TenantId(0)`-Konstruktion (A-10) | Compile-Fail `tag_not_constructible` |

---

## 7. Roadmap und Abnahme

Prinzip der SPEC: dünner vertikaler Durchstich vor horizontaler Breite. Eine Stufe beginnt erst, wenn die Abnahme der vorherigen grün ist. Es gibt keine Kalenderschätzung, weil Teamgröße und Kapazität nicht vorliegen.

### R0 Stop-the-Line (V-01, V-03, V-05, V-13/14 Vorbereitung, V-16; A-02, A-03, A-05, A-06, A-11, A-12, K-5)

| WP | Inhalt |
| --- | --- |
| 0.1 | Toolchain 1.89.0 bereitstellen; Baseline `cargo check`/`cargo test` je Crate in `acta/` und Contextra-Klon; Messprotokoll (**nicht in dieser Analyse geschehen**) |
| 0.2 | `xtask gates`: G1 (Level-Modell, `rings.toml`), G-DEF-1, G-NOREN, G-FEAT, G-SHIM, G-NOTOKIO; negative Fixtures |
| 0.3 | `acta-ids` neu (Tags, `DocOrd`, `DataClass`, keine öffentlichen Felder) |
| 0.4 | `acta-env`: `CryptoRng` fallibel; Fault-Injection-Test `T-RNG-1` |
| 0.5 | `acta-crypto` fail-closed: `seal`/`derive_dek` liefern `Result`; Kante zu `types`/`ports` entfernt; `default = []`; `kv_segment/` und `compat/` ausgegliedert |
| 0.6 | `clippy.toml` bereinigen (Clock-Pfad nach `acta-env`, `thread_rng`-Ausnahme streichen); `acta-testkit` Grundgerüst |
| 0.7 | **SPEC 3.1** mit allen Änderungen aus Abschnitt 11 |
| 0.8 | Hub-Inventur-Werkzeug (LOC, Traits, Fan-in je Element) und Verteilungstabelle 3.3 als Datei im Repo |
| **Abnahme** | `cargo test --workspace` grün; `T-NOFALLBACK-*` grün; G1 (inkl. negativer Fixtures) und G-NOREN grün |

### R1 Krypto-Kern und Tresor (V-02, V-04, V-12, V-16; A-01, A-04)

Umschlag v3 (D-05), Hierarchie H1–H5, Tresor-Layout §4.5 mit Maskierung, Epochenrotation (Crash C11), `destroy_many`, Grabstein, `KeyAnchor` (`FileAnchor` nur Dev; KMS/HSM als Trait-Implementierungen), `ReportSigner`, generische `HashChain`.
**Abnahme:** Testvektoren; `envelope_is_aead`; `slot_payload_masked_at_rest`; `vault_missing_slot_is_erased`; `no_default_sealer_in_release`; Bytes-Grep über Tresordateien; Compile-Fail-Suite 6.3 für Schlüsseltypen.

### R2 Walking Skeleton (V-08, V-09 Teil, V-15)

Pfad: Ingest (`Sealed<P>`) → WAL → `acta-text` mit Lineage und Segmentsiegelung (D-11) → `ErasureCore` synchron (E0–E3) → MK2 → Bericht.
Pakete: Hub-Neuschnitt nur soweit nötig (WP-2.1); `acta-durable-fs` mit `FaultVfs`, Store-Minimalpfad (2.2); `acta-text` mit `Derived<T>` (2.3); `acta-erasure` Gate/Journal/Saga E0–E3/MK2/Bericht (2.4); Ende-zu-Ende-Kanarientest und Crash C1–C6 (2.5).
**Abnahme:** `T-DEL-1`; C1–C6; ein Bericht ist mit `acta-verify` unabhängig verifizierbar.

### R3 bis R7

| Stufe | Inhalt | Behobene Befunde | Abnahme |
| --- | --- | --- | --- |
| R3 Datenmodell | monotone `doc_ord`-Lineage, KeyV2 mit `cluster = subject_tag`, Partitionen N/P/D, Manifest v3, SST 5, Siegelung vor WAL | V-06, V-07 | I-5, I-6, I-7; `no_ascii_keys`, `no_v2_envelope_in_P` |
| R4 Store-Fähigkeiten | `FlushPurgeFence` verdrahtet, gefilterter Rewrite, Datei-Drop, WAL-Frame v2 mit `chain_tag` | V-08, V-17 | Loom Flush gegen Purge; C7, C8; Compaction-Stresstest auch auf einem Kern |
| R5 Indizes/Schichten | `acta-vector`, `acta-graph`, Cache, Checkpoint mit `CopyRegistry`, `acta-erasure-adapters` | V-09, V-10 | `T-VEC`, `T-TEXT`, `T-GRAPH`, `T-REST`; `hnsw_no_residue_after_erase` |
| R6 Chronik/Node | `acta-chronicle`, `acta-engine`, `acta-node`, Boot-Selbsttest | V-14 (Rest) | G1, G2, G-ERA-1, G-READ-1; `T-CHR-1..2` |
| R7 Audit/Härtung | `acta-verify`, Notary, HSM-Signatur, externe Krypto-Prüfung (G4), Rechtsfreigabe (G5), Fassade, CLI | D-12 | unabhängiges Audit ohne Acta-Code; Messwerte statt Zielwerte |

### Parallele Spuren (ab R2, ohne den Kernpfad zu blockieren)

- **M1 Datenschutz/Eingang:** `acta-pii`, `acta-prompt-guard`, `acta-ingest` (Fuzzing von Anfang an)
- **M2 Effizienz:** `acta-simd`, `acta-adapt`, `acta-router`
- **M3 Souveränität:** `acta-infer-api`, lokale Clients (`candle`, `onnx`), `ollama` nur lokal im Default
- **M4 Nachweise:** `acta-audit-export`, `acta-avv-generator` ab R6
- **M5 Entscheidungshilfe:** Agent/Cognition nach R6 (DEFER laut D-19)

---

## 8. Gates

| Gate | Prüfung | Herkunft |
| --- | --- | --- |
| G1 | Level-Modell aus `rings.toml` gegen `cargo metadata` | SPEC §8.5, D-01 |
| G2, G3 | jeder Port genau einmal verdrahtet; keine nicht injizierte Zeit-/Zufallsquelle | SPEC §8.5 |
| G-CRY-1, G-FS-1, G-ERA-1, G-READ-1, G-DEF-1, G-WORD, G-SUPPLY | Krypto-Fallbacks, Dateizugriff, Löschpfad, gegateter Lesepfad, `default-members`, Wortfilter, `cargo deny` | SPEC §8.5 |
| G-NOREN | Muster aus 6.1 | dieser Plan |
| G-HUB-1…4 | LOC-/Trait-Budget, Rule of Two, keine Algorithmus-Konfiguration/Fremdfehler im Hub, Fan-in-Limit | dieser Plan |
| G-PORT-OWNER | Trait-Besitz nach HR-4 | dieser Plan |
| G-ERR, G-FEAT, G-SHIM, G-DUP | Fehler je Crate; additive Features; keine Shims/Aliase; keine doppelten Seams | dieser Plan |
| G-NOTOKIO, G-NOBINCODE | kein `tokio` in Level ≤ 6; keine `bincode`/`serde`-Plattenformate | D-06, D-12 |
| G-UPSTREAM | `UPSTREAM.md` je portiertem Crate aktuell | D-13 |

**Testarten:** Golden (JCS, Formate), Differential gegen eingefrorene Fixtures, Property (`proptest`), Loom (Gate, Fence, Barrier), Crash-Matrix mit `FaultVfs`, Mutationstests, Fuzzing (Ingest, WAL, Manifest, Tresor, Wire), Compile-Fail, `cargo deny`, `cargo audit`. Eine Zusage ohne automatisierten Test gilt laut SPEC als nicht erfüllt.

---

## 9. Mission → Komponenten → Abnahme

| Säule | Komponenten | SPEC-Anker | Messbare Abnahme |
| --- | --- | --- | --- |
| Souveränes Unternehmenswissen | Tresor/Store on-premise, `KeyAnchor` beim Kunden, lokale Inferenz, **Egress-Default-Deny** | §3.3, §9.4 | `no_egress_default`; Schlüssel nur im Kundenanker; Z4 im Bericht als A-EXT |
| Transparenz und Nachvollziehbarkeit | Chronik, Ledger-Hash-Kette, signierter Bericht, `acta-verify`, Art.-30-Export | §7.4, §8 | `T-CHR-1/2`; `as_of` byte-identisch (S6); `acta-verify` ohne Acta-Code |
| Präzision und Kontrolle | Rank (Kalibrierung, Drift), Gate vor jedem Lesepfad, Prompt-Guard, R-CLS | §2.2, §6 | `ClassViolation` ohne Klassendeklaration; `T-RANK-1`; Recall-/Precision-Messwerte |
| Effizienz und Ressourcenschonung | SIMD, Quantisierung, Kaskaden-Router, Rebuild statt Sealed, `destroy_many`, Gruppen-Commit | §4.6, §5.5, §10.3 | Gate-Latenz unabhängig von der Datenmenge (R-O1); Benchmarks gegen Contextra-Baseline (**D-11 und D-06 sind hier zu messende Kostenpunkte**) |
| Schutz personenbezogener Daten | Tresor, Umschlag v3, Crypto-Shredding, Lineage, Gate, PII-Erkennung, **versiegelte Segmente** | S1–S4, S10–S12 | `T-CRY-1`, `T-DEL-1`, `hnsw_no_residue_after_erase`, `text_segment_bytes_have_no_terms` |
| Verwaltung und Organisation | Mandanten/Collections, Partitionen N/P/D, Retention-Klassen, Sweeper, Checkpoints, Ingest | §2.2, §6.3 | Retention-Tests je Datenklasse; `T-REST` |
| Entscheidungs- und Verwaltungshilfe | Chronik als Entscheidungsnachweis, Zeitreise, Handle `H_d`; Agent später | §7.4 | Entscheidung nach Löschung der Person rekonstruierbar; Eingabe-Referenz liefert `Erased` |
| Europäischer Datenschutz | Art. 5, 12, 16–19, 25 (erase, restrict, supersede), SLA 24 h/72 h, Art.-19-Meldung, SDM-Baustein 60, G-WORD | §9 | Fristentests (`legal_max` 30 Tage), Wortfilter, Annahmen im Bericht; Rechtsfreigabe G5 vor Release |

*Dieser Plan ist keine Rechtsberatung. Rechtsaussagen stammen aus SPEC §9; die Rechtsfreigabe (G5) bleibt Teil von R7.*

---

## 10. Kompromiss-Register

| Entscheidung | Wer zahlt | Preis konkret | Revisit-Trigger |
| --- | --- | --- | --- |
| D-01 Levels | Architektur-Team | Zusatzdatei, SPEC-Patch | nie nötig, solange G1 negative Fixtures hat |
| D-02/HR-11 Budgets | Feature-Teams | Neue Features müssen Crate-Zuschnitt begründen | Budget-Reißen → ADR |
| D-05 XChaCha | FIPS-Kunden | kein zertifizierter Algorithmus | schriftliche FIPS-Anforderung |
| D-06 sync-Kern | Netzwerk-Rand | Thread-Pool-Brücke statt Tasks | `erase_return_timeout` gefährdet |
| D-07 HMAC statt BLAKE3 | – | geringe Performanceeinbuße im Ledger | keiner |
| D-09 deterministische Tags | Betroffene bei `k_idx`-Leck | Wörterbuchangriff auf externe IDs | Bedrohungsmodell fordert Unverknüpfbarkeit höher als Statelessness |
| D-11 versiegelte Segmente | Betrieb | RAM-/Kaltstartkosten, kein Klartext-`mmap` | Ladezeit reißt Verfügbarkeitsziele |
| D-12 Handformate | Entwickler | mehr Code | keiner |
| D-13 Fixtures statt Live-Orakel | Test-Team | unvollständige Abdeckung möglich | Fixture-Lücken im Diff-Lauf |
| D-14 Neubau Krypto | Planung | 30 k vorhandene LOC zählen nicht | – |
| D-15 Store-Neubau | Gesamtprojekt | größtes Zeit-/Regressionsrisiko | Durchstich R2 reißt Abnahme |
| D-19 Scope-Schnitt | Produkt | Agent/Cognition/MCP später | Kundenanforderung |

---

## 11. SPEC-Änderungsliste (Fassung 3.1)

1. **§3.1/§8.5 G1:** Level-Modell statt „keine Querkanten im Ring“; `rings.toml` als Quelle (D-01).
2. **§3.1 Ringtabelle:** `acta-infer-api` (R0b), `acta-pii`, `acta-prompt-guard`, `acta-ingest`, `acta-router` (Leaf-Ebene), `acta-privacy`, Inferenz-Randcrates (R5) ergänzen (D-10).
3. **§4.1:** Commitment-Zeile auf HMAC-SHA256 ändern; Anhang A.1 `scope_commitment` auf `hmac-sha256:` (D-07).
4. **§4.1, §4.2, §4.5, §4.4, Anh. A.1:** `ctx/…`, `CTXFILE` → `acta/…`, `ACTAFIL\0` (D-08).
5. **§2.1, §4.6:** Modi `Rebuild`/`Sealed` als **Löschstrategien** formulieren; alle Segmente aus Ebene-P-Quellen at rest versiegelt (D-11).
6. **§4.3:** „unwiederbringlich“ ersetzen (verletzt eigenes G-WORD, §1.3).
7. **§4.2/§6.1:** Herkunft der Tags (`tenant`, `collection`, `subject` deterministisch keyed; `doc_tag` zufällig) ergänzen (D-09).
8. **§10.1/§3.2:** Laufzeitmodell festschreiben: Kern synchron, async nur am Rand (D-06).
9. **§3.3:** Port-Tabelle um Besitzregel HR-4 ergänzen; `acta-erasure-spi` streichen.
10. **§4.5:** Slot-MAC auf 64 Bit gekürzt: im Krypto-Review ausdrücklich bestätigen (bei geheimem MAC-Schlüssel vertretbar, aber nicht stillschweigend).

---

## 12. Nächste Schritte (erste zwei Wochen)

1. Toolchain bereitstellen, Baseline messen (WP-0.1). Erst danach ist belegbar, ob der Ist-Stand baut.
2. A-02 und A-03 beheben (kleinste, sicherste Änderungen), jeweils mit Pflichttest.
3. Entscheidungstermin zu **D-09 (Tags)** und **D-11 (Segmentsiegelung)** mit dem SPEC-Autor: beide sind SPEC-Änderungen bzw. Annahmen.
4. `xtask gates` mit G1 (Levels), G-DEF-1, G-NOREN aufsetzen.
5. Hub-Inventur-Werkzeug bauen, Verteilung 3.3 ins Repo legen.
6. SPEC-Slice für R1 schreiben, Testvektoren (XChaCha20-Poly1305, HKDF, HMAC) beschaffen.

---

## Anhang: Methodik und Nachprüfbarkeit

- **Gelesen:** `acta/docs/SPEC.md` vollständig (880 Zeilen); Vorgängerplan vollständig; Manifeste von `store`, `vector`, `graph`, `text`, `checkpoint`, `mvcc`, `rank`, `ingest`, `router`, `privacy`, `erasure`; `acta/Cargo.toml`, `rust-toolchain.toml`; Quelltext an den genannten Stellen (`aead.rs`, `dek.rs`, `rng.rs`, `acta-ids/src/lib.rs`, `sealer.rs`, `fence.rs`).
- **Nicht gelesen:** Contextra-ADR-Dokumente (`HUB_REMOVAL.md`, ADR-0120/0121/0122) wurden in diesem Durchgang nicht erneut geöffnet; ihre Aussagen stammen aus dem Vorgängerplan und sind dort als [V] markiert. Prüfen Sie sie bei Bedarf gegen den Klon.
- **Nicht gebaut, nicht getestet:** V-13, V-14, V-17 sind nicht reproduziert; alle Rust-Skizzen sind ungeprüft.
- **Manifestkanten** (Abschnitt 2) enthalten ggf. Dev-Dependencies und optionale Abhängigkeiten; die Ringverstöße `store→mvcc` und `vector→mvcc` sind als harte `[dependencies]` zu bestätigen, bevor K-1 in der SPEC final gefasst wird.
