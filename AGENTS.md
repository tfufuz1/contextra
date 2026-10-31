# Contextra – Agenten-Betriebsanleitung (AGENTS.md)

> **Normative Richtlinie für Antigravity CLI, IDE, Desktop App, Claude & Jules.**
> Diese Datei ist die zentrale Quelle der Arbeitsregeln im Workspace `/home/freddy/Projekte/contextra`.
> Sie bindet alle Agenten und Entwickler an die Vorgaben aus [docs/SPEC.md](file:///home/freddy/Projekte/contextra/docs/SPEC.md) und [PLAN.md](file:///home/freddy/Projekte/contextra/PLAN.md).

---

## 1. Produkt & Mission

**Contextra ist ein lokaler, eingebetteter Wissens- und Gedächtnisspeicher in Rust (Single-Node, Library-First).**
- **Speichern**: Transaktionaler LSM- / B-Tree-Speicher mit Write-Ahead-Log (WAL) als einziger Quelle der Wahrheit.
- **Finden**: Hybrides Retrieval (HNSW-Vektorsuche + BM25-Volltextsuche + Reciprocal Rank Fusion / RRF), strikt mandantengrenzenbeachtend.
- **Löschen (DSGVO Art. 17)**: Physische und kryptografische Bereinigung über sechs Schichten (Ebene A: In-Memory- und Index-Purge, Ebene B: kryptografische DocDek-Schlüsselvernichtung) mit signiertem Ed25519-Löschbericht (Proof v4/v5) und Canary-Positivkontrolle.
- **Single-Node-Doktrin**: Kein Server-Socket, kein Sharding, kein P2P, keine Telemetrie nach außen. Zugänge über Rust-API (`contextra`, `contextra-db`) und MCP-Server (`contextra-mcp` via stdio).

---

## 2. Die 12 unverhandelbaren Kernregeln (SPEC §0.2)

1. **Ring- & Schichten-Disziplin (P1, P30)**:
   - Jede Abhängigkeit zeigt auf den gleichen oder einen niedrigeren Ring (Ring 4 → Ring 3 → Ring 2 → Ring 1 → Ring 0).
   - Der Kern `[K]` darf niemals von Rand- `[R]`, Experimentell- `[E]` oder Archiv-Crates `[A]` abhängen.
2. **Zero-Panic in Produktion (P7)**:
   - Strengstens verboten in Produktivpfaden: `unwrap()`, `expect()`, `panic!()`, `todo!()`, `unimplemented!()`, `unreachable!()`, `assert!()`, `std::process::exit` und ungeprüftes Index-Slicing (`slice[i]`).
   - Fehler werden ausnahmslos typisiert über `ContextraError` oder domänenspezifische Fehlerbäume propagiert.
   - Deny-Lints: `clippy::unwrap_used`, `clippy::expect_used`, `clippy::panic`, `clippy::indexing_slicing`.
3. **Determinismus & Port-Purity (P28)**:
   - Zeit, Zufall und IDs dürfen ausschließlich über die Ports `Clock`, `Rng` und `IdGen` aus `contextra-ports` bezogen werden.
   - Verboten in Kernlogik: `Instant::now()`, `SystemTime::now()`, `rand::thread_rng()`, `std::env::var()` sowie `HashMap`/`HashSet` mit Standard-RandomState.
   - Einzige Ausnahme: Kryptografisches Schlüssel- und Salt-Material aus dem OS-CSPRNG.
4. **WAL-First Consistency (P24)**:
   - Jeder persistente Zustand (Dokumente, Chunks, Indizes, Lineage) muss vor der In-Memory-Verarbeitung dauerhaft im WAL protokolliert sein.
   - Sicherheitszustände liegen ausschließlich in den Sidecar-Logs oder im KeyStore (`MFKS`).
5. **I/O ausschließlich über `Vfs` (P24)**:
   - Kein direkter Aufruf von `std::fs` oder `tokio::fs` in Produktivcrates.
   - `fsync`/`fdatasync`-Fehler müssen immer propagiert werden; niemals mit `let _ = ...` verwerfen!
6. **Unsafe nur in drei Inseln (P2)**:
   - `unsafe` Rust-Code ist ausschließlich in `contextra-simd`, `contextra-sys` und `contextra-wire` erlaubt.
   - Jeder einzelne `unsafe`-Block erfordert zwingend eine präzise `// SAFETY:`-Dokumentation, die Alignment, Bounds und Gültigkeit belegt.
   - Alle anderen Crates erzwingen `#![forbid(unsafe_code)]`.
7. **Mandantentrennung aus `RequestContext` (VETO-F10)**:
   - Mandanten-IDs stammen ausschließlich aus dem verifizierten `RequestContext`, niemals aus Nutzereingaben, Literalen oder Default-Werten.
   - Caches, HNSW-Graphkanten, BM25-Statistiken (z. B. IDF) und SSTables dürfen Mandantengrenzen niemals überschreiten.
8. **Sicherheit ist kein Feature-Schalter (A1)**:
   - Verschlüsselung, Löschmessung und Löschbericht sind in jedem Produktiv-Build obligatorisch.
   - Kein `cfg(not(feature))` darf Sicherheits- oder Krypto-Typen durch Stubs/No-Ops ersetzen.
9. **Messen vor Behaupten (INV-CLAIMS-1)**:
   - Ein Restbestand- oder Lösch-Zähler darf nur aus einem echten `LayerScanner` der jeweiligen Schicht stammen.
   - Keine Behauptung ohne maschinellen Zeugen (`LayerWitness`).
10. **Fail-Closed (INV-FAILCLOSED-1)**:
    - Unbekannter Zustand, korrupte Daten, unerwartete Eingaben oder ablaufende Fristen führen sofort zur geordneten Verweigerung, niemals zu stillschweigendem Weiterlaufen.
11. **Eine Typfamilie je Konzept (A2)**:
    - Krypto- und Löschtypen existieren ausschließlich in `contextra-crypto` bzw. `contextra-ports`.
12. **Ein PR = ein Arbeitspaket (WP) = max. ein Crate-Schnitt (REQ-CHG-2)**:
    - Vor der Implementierung muss ein fehlschlagender Regressionstest (Red Phase) vorliegen.
    - Nach jeder Änderung: Build, Testlauf, Lints und Gate-Prüfung.

---

## 3. Architektur-Ringe

| Ring | Rolle | Crates |
|---|---|---|
| **Ring 0** | Fundament & Primitive | `contextra-types`, `contextra-ports`, `contextra-mvcc`, `contextra-crypto`, `contextra-vector`, `contextra-text`, `contextra-rank`, `contextra-simd` [unsafe], `contextra-sys` [unsafe], `contextra-wire` [unsafe], `contextra-durable-fs` |
| **Ring 1** | Speicherschicht & Durability | `contextra-store`, `contextra-checkpoint` |
| **Ring 2** | Kern-Verarbeitung & Ingest | `contextra-ingest`, `contextra-privacy` |
| **Ring 3** | Orchestrierung & Embedded-API | `contextra-engine`, `contextra-db` |
| **Ring 4** | Fassade & Gateways | `contextra` (Composition Root), `contextra-mcp` (Stdio-Server) |

---

## 4. Löschpipeline & DSGVO-Vorgaben (SPEC §12, PLAN §10)

Contextra unterscheidet zwei komplementäre Ebenen, die **nur gemeinsam** wirken:
- **Ebene A (Klartext-Bereinigung)**:
  - Zeroizing der HNSW-Slots (`zeroize_and_free_node`), Rebuild bei Überschreitung der Tombstone-Schwelle (kein Nachbarschafts-Rewiring!).
  - Segment-Rewrite der BM25-Postinglisten.
  - Memtable-Purge über exakte Schlüssel (kein unvollständiges Prefix-Scanning!).
  - Cache-Invalidierung und Pufferbereinigung.
- **Ebene B (Kryptografische Vernichtung)**:
  - Sofortige Vernichtung des dokumentspezifischen Schlüssels (`DocDek`).
  - KeyTable-Update über atomaren Temp-Replace mit rotierendem Tabellenschlüssel.
  - Schutz persistenter Segmente (SSTables, WAL, Snapshots auf Flash/CoW-Medien).
- **Lösch-Saga Zustandsmaschine**:
  `Requested → Frozen → LogicallyDeleted → PlaintextPurged → PlaintextMeasured → KeyDestroyed → Measured → Completed`.
  - Ab Zustand `Frozen` blockiert der `ErasedSet` fail-closed alle Lese- und Suchpfade.
  - Nach Abstürzen setzt `ErasureSaga::recover()` vor der Freigabe des Lesepfads auf.
- **Löschbericht (Proof v4/v5)**:
  - Externe Prüfung ausschließlich per Ed25519-Signatur.
  - Keine HMAC-Abhängigkeit bei externen Prüfern, keine Klartext-IDs oder Personendaten im Report.
  - Ehrliche Angabe von Schranken (`UNMEASURED_ERASURE_LAYERS`).

---

## 5. Sicherheits-Defaults (S-01 bis S-19)

- **KDF**: Argon2id mit $m \ge 64\,\text{MiB}, t \ge 3, p \ge 1$ (`try_new_with_kdf`). Kein HKDF-SHA256 für Root-Passphrasen!
- **Verschlüsselung**: Standardmäßig aktiv (`EncryptionMode::Required`).
- **Mandantentrennung**: Standardmäßig verpflichtend (`TenantPolicy::Required`).
- **Entitätsextraktion**: Standardmäßig deaktiviert (`AutoExtractionMode::Disabled`).
- **Geheimnisverwaltung**: `ZeroizeOnDrop`, keine Klartext-Kopien in `String` oder ungelöschte Puffer.

---

## 6. Antigravity Skills für dieses Projekt

Verwende bei spezifischen Arbeitsaufträgen die dafür eingerichteten Antigravity-Skills:
- `/contextra-doctor`: Workspace-Integrität, Compiler-Fehler (B-01, B-03), Manifest-Reparaturen.
- `/contextra-deletion-verifier`: 6-Stufen-Löschkette, Saga, Canary-Validierung, Ed25519 Proofs, Mutations-Tests.
- `/contextra-security-audit`: Überprüfung der Sicherheits-Defaults (S-01 bis S-19), KDF, Zeroize, Tenant-Isolation.
- `/contextra-invariants-gate`: Ring-DAG, Zero-Panic-Audit, Unsafe-Inseln-Überprüfung, Vfs-Compliance.
- `/contextra-product-suite`: Produktprofile (EP1 Memory, EP6 Guard, EP2 Proof, EP7 Ingest) und Benchmarks.
