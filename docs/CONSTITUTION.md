# Contextra — Project Constitution (Verfassung)

<!-- doc-stamp: 2026-11-01 | Spec-Fassung: 2.0.0 (SOLL-Zustand Release 1.0.0) | Normativ -->

> **On-Demand Governance**
> Dieses Dokument definiert die unverrückbaren, dauerhaften Architektur-, Sicherheits- und Governance-Prinzipien von Contextra. Operative Anweisungen stehen in [`docs/SPEC.md`](SPEC.md), [`PLAN.md`](../PLAN.md) und [`AGENTS.md`](../AGENTS.md).

---

## 🏛️ 1. Grundlegende Architektur-Prinzipien

### 1.1 Safety First & Zero Panics (P7, SPEC §11.1)
- **Safe Rust:** Standardmäßig ist `unsafe` im gesamten Workspace verboten (`#![forbid(unsafe_code)]`).
- **Unsafe-Inseln (D-29):** `unsafe` ist ausschließlich in zwei deklarierten Inseln erlaubt: `contextra-sys` und `contextra-simd`. Jeder einzelne Block MUSS durch einen expliziten `// SAFETY:`-Beweiskommentar (Bounds, Alignment, Gültigkeit) begründet sein. Alle anderen 12 Produktivcrates erzwingen strikt `#![forbid(unsafe_code)]`.
- **Zero Panics:** Weder Bibliotheken noch öffentliche APIs dürfen paniken. `unwrap()`, `expect()`, `panic!`, `todo!`, `unimplemented!`, `unreachable!`, `assert!` und ungeprüftes Slicing (`slice[i]`) sind im Produktionscode strikt verboten (erzwungen über Deny-Lints). Fehler werden ausnahmslos typisiert über `ContextraError` oder domänenspezifische Fehlerbäume propagiert.

### 1.2 Haltbarkeit, Persistenz & Determinismus (P24, P28)
- **WAL First (P24, I-1):** Schreibreihenfolge ist ausnahmslos: Frame(s) → Gruppen-Commit → `sync_data` WAL → Memtable → Sichtbarkeit.
- **I/O ausschließlich über `Vfs` (D-04):** Kein direkter Aufruf von `std::fs` oder `tokio::fs` in Produktivcrates. Alle Operationen laufen über den `Vfs`-Port. Sämtliche `fsync`- und `sync_*`-Fehler werden ausnahmslos propagiert.
- **Cancel-Safety (I-14, D-C01):** Mutierende Commits laufen in entkoppelten Tasks; Abbruch des Aufrufer-Futures ändert den Ausgang nicht.
- **P28 Determinismus:** Zeit, Zufall und Bezeichner werden in der Kernlogik ausschließlich über die Ports `Clock`, `Rng` und `IdGen` bezogen. Direkte Aufrufe von `Instant::now()`, `SystemTime::now()`, `thread_rng()` oder `std::env::var()` sowie `HashMap`/`HashSet` mit Standard-RandomState sind verboten.

### 1.3 Strikte Schichtung & Ringmodell (SPEC §5.2)
- **Strikter DAG:** Die Modul- und Crate-Struktur folgt den Ringen 0 bis 4. Abhängigkeiten fließen ausnahmslos abwärts (Ring 4 → 3 → 2 → 1 → 0) und innerhalb Ring 2 gemäß Matrix SPEC §5.3.
- **Kern-Isolation:** Der Kern darf niemals von Rand-, Experimentier- oder entfernten Crates (wie `graph`, `ingest`, `privacy`, `checkpoint`, `license`, `wire`) abhängen.
- **Single-Node-Doktrin:** Contextra ist ein lokaler, eingebetteter Single-Node-Speicher. Ein Primärprozess sperrt das Datenverzeichnis exklusiv. Kein Server-Socket, kein Sharding, kein P2P, keine Netzwerk-Crates im Produktionsbaum, keine Telemetrie nach außen.

---

## 🔒 2. Datenschutz, Mandantentrennung & Löschung

### 2.1 Absolute Mandantentrennung (VETO-F10, D-07)
- **Mandant aus dem Kontext:** Die `TenantId` stammt zwingend aus dem Kontext und ist obligatorisch (`TenantPolicy::Required` ohne Opt-Out).
- **Keine Datenlecks:** Der Zugriff auf den Store erfolgt ausschließlich über `TenantScopedStorage` (T-ARCH-2). Caches, HNSW-Graphkanten, BM25-Statistiken und SSTables dürfen Mandantengrenzen niemals überschreiten.

### 2.2 Gemessenes Löschen & ADR-001 (SPEC §11, §12)
- **Keine Ebene B in v1 (D-01):** Gemäß ADR-001 entfällt Ebene B (kryptografische DocDek-Schlüsselvernichtung) in v1.
- **Physische Bereinigung (Ebene A):** Physisches Purging in Memtable, SSTable-Rewrite/Swap via Manifest, HNSW-Slot-Zeroing mit Batch-Rebuild, BM25-Segment-Rewrite.
- **Fail-Closed ReadGate:** Unmittelbar nach dem Status `Gated` blockiert das Gate (`ErasedSet`) alle Lese- und Suchpfade auf Zielschlüssel fail-closed. Aktive Snapshots werden widerrufen.
- **Messen vor Behaupten (INV-CLAIMS-1, D-35):** Restbestände stammen aus echten Messungen (`scan_residue` über `ErasureLayer`, E1 Rohscan, E2 Decoder-Scan, E3 Schlüsselabwesenheit) mit obligatorischer Positivkontrolle (Canary).
- **Fälschungssichere Nachweise (D-13):** Löschberichte (`ctx-erasure-report/1`) werden maschinenlesbar ausgestellt und ausschließlich per Ed25519-Signatur beglaubigt.

### 2.3 Sicherheit per Default (D-05, D-06)
- **Verschlüsselung at rest:** Pflicht in jedem Build ohne Opt-Out (`EncryptionMode::Required`, AES-256-GCM-SIV).
- **KDF:** Argon2id mit $m \ge 64\,\text{MiB}, t \ge 3, p \ge 1$ (`derive_master`). HKDF für Passphrasen ist verboten.
- **Geheimnisverwaltung:** Alle Schlüssel und sensiblen Puffer in `Zeroizing`, kein `Clone`, kein `Debug`-Inhalt, kein `Serialize`.

---

## 🚦 3. Qualitätssicherung & Evidenz (INV-EVIDENCE-1, PLAN §1)

- **Testgetriebene Entwicklung (TDD):** Vor der Implementierung muss ein fehlschlagender Regressionstest (Red Phase) vorliegen. Ohne roten Test kein Fix.
- **Ein PR = ein Arbeitspaket (WP) = max. ein Crate-Schnitt (REQ-CHG-2).**
- **Evidenzstufen:** Dokumentationsaussagen gelten als unbewiesene Behauptung (E0), bis sie durch Quelltextbelege (E1), Gate-Läufe (E2), reproduzierbare CI-Tests (E3) oder formale Verifikation (E4) nachgewiesen sind. Evidenz wird unter `evidence/<WP-ID>/` abgelegt.
- **Keine Ausnahmen ohne ADR:** Jede Abweichung von Architekturregeln erfordert ein Architecture Decision Record (ADR).
