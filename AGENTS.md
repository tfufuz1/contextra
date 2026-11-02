# Contextra – Agenten-Betriebsanleitung (AGENTS.md)

<!-- doc-stamp: 2026-11-02 | Normativ: docs/SPEC.md v4.0.0 -->

> **Normative Richtlinie für Antigravity CLI, IDE, Desktop App, Claude & Jules.**
> Diese Datei ist die zentrale Quelle der Arbeitsregeln im Workspace.
> Sie bindet alle Agenten und Entwickler ausnahmslos an die Vorgaben aus [docs/SPEC.md](docs/SPEC.md) (Fassung 4.0.0 für Release 1.0.0).
> **Ausschließlich `docs/SPEC.md` ist die Quelle der Wahrheit.** Jede Abweichung des Quellcodes oder anderer Dokumente von `docs/SPEC.md` ist ein Defekt.

---

## 1. Produkt & Mission (SPEC §1)

Contextra ist eine **lokale, eingebettete Rust-Bibliothek** (Single-Node, kein Server) für lokale Speicherung von Wissen und Gedächtnis von Agenten- und RAG-Systemen:
- **Speichern**: Transaktionaler LSM-Tree-Speicher mit Write-Ahead-Log (WAL) als einziger Quelle der Wahrheit (I-1: WAL → Memtable → Sichtbarkeit; WAL wird nie umgeschrieben, D-73).
- **Finden**: Hybrides Retrieval (HNSW-Vektorsuche + segmentierte BM25-Volltextsuche + RRF-Fusion), strikt mandantengrenzenbeachtend.
- **Löschen (Art. 17/18 DSGVO)**: Entkoppelter, gemessener Löschprozess auf **drei Säulen** (D-01, ADR-002, VETO-D01b):
  1. *Gate und physischer Purge:* Sofortige Lesesperre (`ReadGate`) und physisches Umschreiben betroffener Dateien (LSM-SSTable-Rewrite, HNSW-Batch-Kantenreparatur mit Navigationssurrogaten und deterministischem Rebuild D-77, Textsegment-Rewrite D-71).
  2. *Kryptografische Schlüsselvernichtung:* Hierarchischer Tresor (`FileVault`, D-79) vernichtet Mandanten- (`T`), Collection- (`C`), Betroffenen- (`S`) oder Dokument-Slots (`D`), wodurch untergeordnete DEKs sofort unableitbar werden; Dateischlüssel-Vernichtung (`F`-Slots, D-85) entwertet freigegebene Blöcke.
  3. *Unabhängige Messung & Nachweis:* Typisierter Zeuge `LayerWitness` (Messklassen MK1, MK2, MK3-Feld), Ed25519-signierter Bericht und unabhängige Feldprüfung via `ctx-verify scan/report` und Erasure Lab.
- **Single-Node-Doktrin (VETO-SN01, AR-7)**: Kein Server-Socket, kein Sharding, kein P2P, keine Netzwerk-Crates im Produktionsbaum. Öffentlicher Zugang ausschließlich über die Fassaden-Crate `contextra`.

---

## 2. Die 12 unverhandelbaren Kernregeln (SPEC §0.2, §2)

1. **Ring- & Schichten-Disziplin (P1, P30, SPEC §5.2)**:
   - Jede Abhängigkeit zeigt ausschließlich auf den gleichen oder einen niedrigeren Ring (Ring 4 → Ring 3 → Ring 2 → Ring 1 → Ring 0).
   - Innerhalb Ring 2 gilt strikt die Abhängigkeitsmatrix aus SPEC §5.3.
   - Der Kern darf niemals von Rand-, Experimentell- oder entfernten Crates abhängen.
2. **Zero-Panic in Produktion (P7, SPEC §11.1)**:
   - Strengstens verboten in Produktivpfaden: `unwrap()`, `expect()`, `panic!()`, `todo!()`, `unimplemented!()`, `unreachable!()`, `assert!()`, `std::process::exit` und ungeprüftes Index-Slicing (`slice[i]`).
   - Fehler werden ausnahmslos typisiert über `ContextraError` oder domänenspezifische Fehlerbäume propagiert.
   - Erzwungene Deny-Lints: `clippy::unwrap_used`, `clippy::expect_used`, `clippy::panic`, `clippy::indexing_slicing`.
3. **Determinismus & Port-Purity (P28, SPEC §8.2)**:
   - Zeit, Zufall und IDs dürfen ausschließlich über die Ports `Clock`, `Rng` und `IdGen` aus `contextra-ports` bezogen werden.
   - Verboten in Kernlogik: `Instant::now()`, `SystemTime::now()`, `rand::thread_rng()`, `std::env::var()` sowie `HashMap`/`HashSet` mit Standard-RandomState (stattdessen `BTreeMap`/`BTreeSet`).
   - Einzige Ausnahme: Kryptografisches Schlüssel- und Salt-Material aus dem OS-CSPRNG (`OsRng`).
4. **WAL-First Consistency (P24, I-1, SPEC §5.5)**:
   - Schreibreihenfolge ist ausnahmslos: Frame(s) → Gruppen-Commit → `sync_data` WAL → Memtable → Sichtbarkeit.
   - Der WAL wird nie umgeschrieben (D-73); Purge flusht Memtable, siegelt das WAL-Segment und löscht Segmente bis zum Flush-Punkt als Ganzes.
5. **I/O ausschließlich über `Vfs` (P24, D-04, SPEC §8.2, §8.5)**:
   - Kein direkter Aufruf von `std::fs` oder `tokio::fs` in Produktivcrates.
   - Alle Dateioperationen laufen über den `Vfs`-Port (`RealVfs` in `contextra-durable-fs`, `FaultVfs` in Tests).
   - `fsync`/`fdatasync`/`sync_data`/`sync_all`-Fehler müssen immer propagiert werden; niemals mit `let _ = ...` verwerfen (D-55)!
6. **Unsafe ausschließlich in zwei Inseln (P2, D-29, SPEC §5.2)**:
   - `unsafe` Rust-Code ist ausschließlich in `contextra-simd` (Distanzkernel) und `contextra-sys` (Plattformprimitive) erlaubt.
   - Jeder einzelne `unsafe`-Block erfordert zwingend eine präzise `// SAFETY:`-Dokumentation (Alignment, Bounds, Gültigkeit).
   - Alle anderen 12 Produktivcrates erzwingen strikt `#![forbid(unsafe_code)]`.
7. **Mandantentrennung verpflichtend (D-07, SPEC §7.2, §8.8)**:
   - Mandanten-IDs (`TenantId`) stammen aus dem Kontext und sind Pflicht.
   - Speicherzugriff erfolgt ausschließlich über `TenantScopedStorage` (AR-6, T-ARCH-2).
   - Caches, HNSW-Graphkanten, BM25-Statistiken und SSTables dürfen Mandantengrenzen niemals überschreiten.
8. **Sicherheit ist kein Feature-Schalter (D-05, SPEC §10)**:
   - Verschlüsselung at rest (AES-256-GCM-SIV) und WAL-HMAC-Integrität sind Pflicht ohne Opt-Out.
   - KDF: Argon2id mit strengen Grenzen $m \in [64\,\text{MiB}, 4\,\text{GiB}], t \in [3, 16], p \in [1, 16]$ (D-06). Kein HKDF für Passphrasen!
   - Kein Feature schwächt Krypto, Integrität oder Mandantentrennung (D-84, HR-13).
9. **Wirkung vor Behauptung & Messung (P1, D-35, D-72, SPEC §12)**:
   - Status und Zähler entstehen nur aus echten Messungen über `ErasureLayer` und `LayerWitness`.
   - Trennung von Evidenzstufen (**EV0–EV4**) und Messklassen (**MK1–MK3**).
   - Typisierte Positivkontrolle (`Scanner<Armed>`) vor jeder Zeugenausstellung verpflichtend.
10. **Fail-Closed (P2, INV-FAILCLOSED-1)**:
    - Unbekannter Zustand, korrupte Daten, unerwartete Eingaben oder ablaufende Fristen führen sofort zur geordneten Verweigerung, niemals zu stillschweigendem Weiterlaufen.
11. **Eine Typfamilie je Konzept (SPEC §0.2, §8.1)**:
    - Reine Domänentypen, IDs, Tags und Fehler existieren in `contextra-types`.
    - Kryptografische Operationen existieren in `contextra-crypto`.
12. **Ein PR = ein Arbeitspaket (WP) = max. ein Crate-Schnitt (REQ-CHG-2, SPEC §15.4)**:
    - Vor der Implementierung liegt ein fehlschlagender Regressionstest (Red Phase) vor.
    - Nach jeder Änderung: Build, Testlauf, Lints und Gate-Prüfung.
    - Strikte Wortlaut-Lint-Einhaltung (T-LINT-1) nach SPEC §6.4.

---

## 3. Architektur-Ringe & Crate-Topologie (SPEC §5.1, §5.2)

```
Ring 4  contextra                         (Fassade, Composition Root)
Ring 3  contextra-erasure   contextra-engine
Ring 2  contextra-mvcc  contextra-store  contextra-text  contextra-vector  contextra-rank
Ring 1  contextra-sys  contextra-simd  contextra-durable-fs  contextra-crypto
Ring 0  contextra-types  contextra-ports
dev     testkit  lab  ctx-verify  tests/arch
```

---

## 4. Löschpipeline & Drei Säulen (SPEC §11, §12, ADR-002)

Contextra implementiert das Löschen als **eigenen, entkoppelten, gemessenen Prozess**:
- **Zustandsmaschine (SPEC §8.2, §11.2)**:
  `Requested → Gated → Drained → Rewritten → Cleaned → Measured → Completed` (Fehlerfall: `Pending` mit Backoff oder `Failed`).
- **ReadGate (`ErasedSet`)**:
  - Unmittelbar nach Zustand `Gated` blockiert das Gate fail-closed jeden Lese- und Suchzugriff auf Zielschlüssel und Tags.
  - Snapshot-Widerruf: Aktive Snapshots, die Zielversionen sehen könnten, werden widerrufen (`SnapshotRevoked`).
- **Kryptografische Schlüsselvernichtung (Tresor, D-79, D-85)**:
  - Vernichtung des zugehörigen Slots (`T`, `C`, `S`, `D`) im `FileVault`. Dadurch sind alle betroffenen DEKs sofort unableitbar (O(1) vor `Gated`).
  - Dateischlüssel-Vernichtung (`F`-Slots) entwertet freigegebene Blöcke.
- **Physischer Purge**:
  - **Store**: `drain` versiegelt WAL, Memtable flush, Truncate alter WAL-Segmente; SSTables umschreiben und Manifest-Swap.
  - **Vector**: Vektorsurrogat vor `Gated`; Batch-Kantenreparatur im Purge-Fenster; deterministischer Rebuild bei Schwellenüberschreitung (D-77).
  - **Text**: Textpuffer bereinigen; betroffene Segmente ohne Zieldokumente neu schreiben; Alt-Segmente löschen; Term-Statistik neu berechnen (D-71).
- **Residuenmessung & Klassifikation (SPEC §12)**:
  - **MK1**: Strukturelle Prüfung auf Abwesenheit von Ziel-Records in LSM, Postings, Slots.
  - **MK2**: Byte-genauer Rohscan des Datenverzeichnisses nach Needles.
  - **MK3**: Prüfung der Schlüsselabwesenheit im Tresor.
  - Positivkontrolle vor jedem Scan obligatorisch (`Scanner<Armed>`).
- **Löschbericht (`ctx-erasure-report/1`, D-13, SPEC §11.7)**:
  - Ed25519-signiert, maschinenlesbar, enthält keine Klartext-IDs oder Personendaten.
  - Verifizierbar durch die unabhängige Binärdatei `ctx-verify`.
