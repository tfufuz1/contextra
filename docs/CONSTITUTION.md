# Contextra — Project Constitution (Verfassung)

<!-- doc-stamp: 2026-11-02 | Spec-Fassung: 4.0.0 (SOLL-Zustand Release 1.0.0) | Normativ -->

> **On-Demand Governance**
> Dieses Dokument definiert die unverrückbaren, dauerhaften Architektur-, Sicherheits- und Governance-Prinzipien von Contextra.
> Alleinige Quelle der Wahrheit ist [`docs/SPEC.md`](SPEC.md). Jede Abweichung ist ein Defekt.

---

## 1. Grundlegende Architektur-Prinzipien

### 1.1 Safety First & Zero Panics (P7, SPEC §11.1)
- **Safe Rust:** Standardmäßig ist `unsafe` im gesamten Workspace verboten (`#![forbid(unsafe_code)]`).
- **Unsafe-Inseln (D-29, SPEC §5.2):** `unsafe` ist ausschließlich in zwei deklarierten Inseln erlaubt: `contextra-sys` (POSIX-Plattformprimitive) und `contextra-simd` (Distanzkernel). Jeder einzelne Block MUSS durch einen expliziten `// SAFETY:`-Beweiskommentar (Bounds, Alignment, Gültigkeit) begründet sein. Alle anderen 12 Produktivcrates erzwingen strikt `#![forbid(unsafe_code)]`.
- **Zero Panics:** Weder Bibliotheken noch öffentliche APIs dürfen paniken. `unwrap()`, `expect()`, `panic!`, `todo!`, `unimplemented!`, `unreachable!`, `assert!` und ungeprüftes Slicing (`slice[i]`) sind im Produktionscode strikt verboten (erzwungen über Deny-Lints). Fehler werden ausnahmslos typisiert über `ContextraError` oder domänenspezifische Fehlerbäume propagiert.

### 1.2 Haltbarkeit, Persistenz & Determinismus (P24, P28)
- **WAL First (P24, I-1):** Schreibreihenfolge ist ausnahmslos: Frame(s) → Gruppen-Commit → `sync_data` WAL → Memtable → Sichtbarkeit. Der WAL wird nie umgeschrieben (D-73).
- **I/O ausschließlich über `Vfs` (D-04):** Kein direkter Aufruf von `std::fs` oder `tokio::fs` in Produktivcrates. Alle Operationen laufen über den `Vfs`-Port (`RealVfs` in `contextra-durable-fs`, `FaultVfs` in Tests). Sämtliche `fsync`- und `sync_*`-Fehler werden ausnahmslos propagiert (D-55).
- **Cancel-Safety (I-14, D-C01):** Mutierende Commits laufen in entkoppelten Tasks; Abbruch des Aufrufer-Futures ändert den Ausgang nicht.
- **P28 Determinismus:** Zeit, Zufall und Bezeichner werden in der Kernlogik ausschließlich über die Ports `Clock`, `Rng` und `IdGen` bezogen. Direkte Aufrufe von `Instant::now()`, `SystemTime::now()`, `thread_rng()` oder `std::env::var()` sowie `HashMap`/`HashSet` mit Standard-RandomState sind verboten.

### 1.3 Strikte Schichtung & Ringmodell (SPEC §5.1–§5.3)
- **Strikter DAG:** Die Workspace-Struktur folgt 5 Ringen:
  - Ring 4: Fassade `contextra`
  - Ring 3: Orchestrierung `contextra-engine`, Löschkern `contextra-erasure`
  - Ring 2: Module `contextra-mvcc`, `contextra-store`, `contextra-text`, `contextra-vector`, `contextra-rank`
  - Ring 1: Fundamente `contextra-sys`, `contextra-simd`, `contextra-durable-fs`, `contextra-crypto`
  - Ring 0: Reine Typen und Schnittstellen `contextra-types`, `contextra-ports`
  - Dev/Tools: `contextra-testkit`, `contextra-lab`, `ctx-verify`, `tests/arch`
- **Abhängigkeitsregeln:** Abhängigkeiten fließen ausnahmslos abwärts (Ring 4 → 3 → 2 → 1 → 0) und innerhalb Ring 2 strikt nach Matrix SPEC §5.3.
- **Kern-Isolation:** Der Kern darf niemals von Rand-, Experimentier- oder entfernten Crates (wie `graph`, `ingest`, `privacy`, `checkpoint`, `license`, `wire`) abhängen.

### 1.4 Löschen in drei Säulen (D-01, ADR-002, SPEC §1.1, §4.1)
Contextra realisiert Löschen über drei zusammenwirkende Säulen:
1. **Gate und physischer Purge:** Sofortige Lesesperre (`ReadGate`) und physisches Umschreiben betroffener Dateien (LSM-SSTable-Rewrite, Batch-Kantenreparatur und deterministischer HNSW-Rebuild, Textsegment-Rewrite, WAL-Truncate nach Flush).
2. **Schlüsselvernichtung (Tresor):** Vernichtung des Schlüssels der höchsten Ebene des Löschziels (Mandant `T`, Collection `C`, Betroffener `S`, Dokument `D`) im dateibasierten Hash-Tresor (`FileVault`, D-79) sowie Dateischlüssel-Vernichtung (`F`-Slots, D-85). Vernichtung eines übergeordneten Schlüssels macht alle darunterliegenden DEKs sofort in O(1) unlesbar.
3. **Unabhängige Messung & signierter Bericht:** Typisierte Zeugenerhebung (`LayerWitness`) mit Positivkontrolle (Messklassen MK1, MK2, MK3-Feld), Erstellung eines Ed25519-signierten Prüfberichts (`ctx-erasure-report/1`) und unabhängige Prüfbarkeit mit `ctx-verify scan/report` und im Erasure Lab.

### 1.5 Fail-Closed & Mandantenisolation (P2, D-07, VETO-F10)
- **P2 Fail-Closed:** Unbekannte Zustände, fehlerhafte Checksummen, Krypto- oder I/O-Fehler führen zwingend zur Verweigerung oder zum Zustand `Pending`, niemals zum stillen Übergehen.
- **Absolute Mandantentrennung (D-07, VETO-F10):** Jede Operation erfordert einen gültigen `TenantContext`. Speicherzugriff erfolgt ausschließlich über `TenantScopedStorage` (AR-6). Mandantenübergreifender Daten- oder Cache-Fluss ist ausgeschlossen.
