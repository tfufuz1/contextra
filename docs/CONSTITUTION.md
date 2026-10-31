# Contextra — Project Constitution (Verfassung)

<!-- doc-stamp: 2026-10-31 | Spec-Fassung: 2.4 | Normativ -->

> **On-Demand Governance**
> Dieses Dokument definiert die unverrückbaren, dauerhaften Architektur-, Sicherheits- und Governance-Prinzipien von Contextra. Operative Anweisungen stehen in [`docs/SPEC.md`](SPEC.md) und [`PLAN.md`](../PLAN.md).

---

## 🏛️ 1. Grundlegende Architektur-Prinzipien

### 1.1 Safety First & Zero Panics (P7)
- **Safe Rust:** Standardmäßig ist `unsafe` im gesamten Workspace verboten (`#![forbid(unsafe_code)]`).
- **Unsafe-Inseln:** `unsafe` ist ausschließlich in drei deklarierten Inseln erlaubt: `contextra-sys`, `contextra-simd` und `contextra-wire`. Jeder einzelne Block MUSS durch einen expliziten `// SAFETY:`-Beweiskommentar begründet sein.
- **Zero Panics:** Weder Bibliotheken noch öffentliche APIs dürfen paniken. `unwrap()`, `expect()`, `panic!`, `todo!`, `unimplemented!`, `unreachable!` und ungeprüftes Slicing (`[i]`) sind im Produktionscode strikt verboten (erzwungen über Clippy-Lints). Fehler werden ausnahmslos typisiert über `Result<T, ContextraError>` behandelt.

### 1.2 Haltbarkeit, Persistenz & Determinismus
- **WAL First (P24):** Vor jeder In-Memory-Zustandsänderung muss der Log-Eintrag dauerhaft im Write-Ahead-Log (WAL) per `fsync` synchronisiert sein.
- **Deterministische Wiederherstellung:** Der Zustand des Speichers muss nach einem Absturz vollständig aus den Protokolldateien rekonstruierbar sein.
- **Keine stillen Fehler (INV-FAILCLOSED-1):** Dateisystem- und I/O-Fehler dürfen niemals mit `let _ =` oder Standardwerten verschluckt werden. Jeder Fehler wird via `?` propagiert.
- **P28 Determinismus:** Zeit, Zufall und Bezeichner werden in der Geschäftslogik über die Ports `Clock`, `Rng` und `IdGen` injiziert. Direkte Aufrufe von `Instant::now()`, `SystemTime::now()` oder `thread_rng()` sind in Ring 0–3 verboten (Ausnahme: kryptografisches Zufallsmaterial für Keys/Salts aus dem System-RNG).

### 1.3 Strikte Schichtung & Ringmodell
- **Strikter DAG (Directed Acyclic Graph):** Die Modul- und Crate-Struktur folgt den Ringen 0 bis 4. Abhängigkeiten fließen ausnahmslos abwärts (Ring 4 → 3 → 2 → 1 → 0).
- **Kern-Isolation:** Der Kern (`[K]`) kennt keine Randprodukte (`[R]`), Experimente (`[E]`) oder Benutzeroberflächen.
- **Single-Node-Doktrin:** Contextra ist ein lokaler, eingebetteter Speicher. Ein Primärprozess hält das Datenverzeichnis. Kein Sharding, kein P2P, kein Netzwerk-Listener im Kern.

---

## 🔒 2. Datenschutz, Mandantentrennung & Löschung

### 2.1 Absolute Mandantentrennung (VETO-F10)
- **Mandant aus dem Kontext:** Die `TenantId` stammt zwingend und unveränderlich aus dem `RequestContext`, niemals aus Nutzereingaben, URL-Parametern oder Standardwerten.
- **Keine Datenlecks:** Mandantenübergreifende Abfragen, geteilte Caches oder Wissensaggregationen sind kategorisch verboten.

### 2.2 Gemessenes Löschen & Crypto-Shredding (SPEC §12)
- **Zweistufiges Löschen:** Löschung erfordert die Kombination aus physischem Purge flüchtiger Strukturen im Arbeitsspeicher (Ebene A) und kryptografischer Schlüsselvernichtung persistenter Reste im LSM/WAL (Ebene B).
- **Messen vor Behaupten:** Ein Löschbericht darf nur Restbestände ausweisen, die durch tatsächliche Prüf-Scanner gemessen wurden.
- **Fälschungssichere Nachweise:** Löschberichte werden über HMAC und Ed25519-Signaturen manipulationssicher signiert.

---

## 🚦 3. Qualitätssicherung & Evidenz (INV-EVIDENCE-1)

- **Testgetriebene Entwicklung (TDD):** Fehlerbehebungen und neue Features beginnen mit einem zuvor fehlschlagenden (roten) Test.
- **Evidenzstufen:** Dokumentationsaussagen gelten als unbewiesene Behauptung (E0), bis sie durch Quelltextbelege (E1), Gate-Läufe (E2), reproduzierbare CI-Tests (E3) oder formale Verifikation (E4) nachgewiesen sind.
- **Keine Ausnahmen ohne ADR:** Jede Abweichung von Architekturregeln erfordert ein Architecture Decision Record (ADR) und den Commit-Trailer `ADR: <Pfad>`.
