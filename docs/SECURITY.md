# Contextra – Sicherheitsmodell (Baukasten-System)

<!-- doc-stamp: 2026-11-08 | Normativ: docs/SPEC.md Fassung 4.1 §3, §6, §10, §12 -->

Dieses Dokument beschreibt das Bedrohungs- und Sicherheitsmodell des modularen **Contextra Baukastensystems**.
Als Baukastensystem aus autarken Rust-Bausteinen (Klasse A) und dem persistenten Storage-Bundle (Klasse B) richtet sich das Sicherheitsmodell an Entwickler und Architekten, die Komponenten in eigene Zielanwendungen einbetten.

Dieses Dokument ist eine technische Beschreibung. Es stellt keine Rechtsberatung und keine formale Zertifizierung dar.

---

## 1. Schutzgüter

| Schutzgut | Ort | Schutzmechanismus |
|---|---|---|
| Dokumente und Nutzdaten | SSTables, MemTable, WAL (`contextra-store`) | Authenticated Encryption at Rest via AES-256-GCM-SIV (`contextra-crypto`), Integritätsketten (HMAC-SHA256, BLAKE3), CRC32-Prüfsummen in Blöcken |
| Schlüsselmaterial | Arbeitsspeicher des Prozesses | Verwaltung via typisierte Container (`dek.rs`), automatische Nullung (`ZeroizeOnDrop`), Speichersperren gegen Paging/Swap via `contextra-sys::mlock` (`INV-CRYPTO-ZEROIZE-1`, `INV-SYS-MEMLOCK-1`) |
| Mandanten- und Bereichsisolation | Indizes und Speicherschichten | Strikt deterministische Trennung nach `TenantId` und Scopes; keine Vermischung über Anfragen hinweg (`VETO-F10`) |
| Revisionssichere Löschung | Löschkern (`contextra-erasure`) | Kaskadierte Löschsagas, kryptografisches Crypto-Shredding, Ed25519-signierte Löschnachweise, Restore-Guards gegen Wiederherstellung aus Alt-Backups (`docs/SPEC.md` §6) |
| Datenabflussschutz (Egress) | Anwendungsränder (`contextra-privacy`) | Automatische Erkennung und Maskierung personenbezogener Daten (PII) vor Weitergabe an externe APIs (`INV-PRIVACY-FAILCLOSED-1`) |
| Ausführungssicherheit | Code-Ausführung (`contextra-sandbox`) | WASM-Isolation (Wasmtime) mit Fuel- und Speichergrenzen (`INV-SANDBOX-FUEL-1`) |
| Build- und Lieferkettenintegrität | Workspace & Crates | `deny.toml`, Lizenzprüfung (`cargo-deny`), strikte MSRV 1.89 |

---

## 2. Vertrauensgrenzen

- **Prozessgrenze:** Contextra-Bausteine laufen als Library im Prozess der Host-Anwendung. Die Host-Anwendung gilt als vertrauenswürdig bezüglich der Speicherallokation. Direkte Angriffe aus demselben Prozesskontext (z. B. willkürliches Memory-Patching) liegen außerhalb des Bibliotheksmodells.
- **Dateisystemgrenze:** Persistierte Dateien im Datenverzeichnis werden beim Einlesen stets als potenziell korrupt oder manipuliert betrachtet. Manipulierte Blöcke, WAL-Einträge oder Indizes führen zum sofortigen, deterministischen Abbruch (Fail-Closed, `INV-FSYNC-1`).
- **Netzwerkgrenze:** Bausteine des Fundaments und der Speicherkerne öffnen keine unautorisierten Netzwerkverbindungen (Single-Node-Doktrin für Persistenzkerne). Netzwerkzugriffe sind explizit auf Inferenz-Clients (`contextra-infer-ollama`) und Gateway-Protokolle (`contextra-mcp`) beschränkt.
- **Mandantengrenze:** Anfragen verschiedener Mandanten dürfen sich unter keinen Umständen gegenseitig beeinflussen (`VETO-F10`). Die `TenantId` wird aus dem autorisierten Kontext bezogen.

---

## 3. Bedrohungen und Gegenmaßnahmen

| Bedrohung | Gegenmaßnahme | Normative Referenz |
|---|---|---|
| Datenoffenlegung bei Verlust des Datenträgers | AES-256-GCM-SIV Verschlüsselung at Rest in `contextra-crypto` / `contextra-store` | `docs/SPEC.md` §12.3, §14.1 |
| Unbemerkte Daten- oder WAL-Manipulation | BLAKE3- und HMAC-basierte Integritätsketten, Block-Prüfsummen, Fail-Closed beim Recovery | `docs/SPEC.md` §10.1, §12.3, §14.1 |
| Mandantenübergreifende Datenabflüsse | Isolationsfilter vor Indexabfragen, konsequente Trennung in Scopes | `docs/SPEC.md` §3, `VETO-F10` |
| Datenverlust oder Inkonsistenz nach Absturz | Append-Only WAL, atomares Rename (`write_atomic`), Fsync-Poisoning bei E/A-Fehlern | `docs/SPEC.md` §3.5, §12.1 (`INV-FSYNC-1`) |
| Unbeabsichtigtes Wiederaufleben gelöschter Daten | Restore-Guard in `contextra-erasure`, der gelöschte Datenstände in Alt-Snapshots blockiert | `docs/SPEC.md` §6.2, §14.1 |
| Speichersicherheits-Schwachstellen | `#![forbid(unsafe_code)]` im gesamten Workspace außer den 3 isolierten Unsafe-Inseln (`simd`, `sys`, `wire`) | `docs/SPEC.md` §2 B10, §8 G10 |
| Ungewollter PII-Abfluss in externe Cloud-Modelle | Egress-Filterung und Fail-Closed-Maskierung in `contextra-privacy` | `docs/SPEC.md` §12.4 |
| Unberechenbare Schleifen oder Denial-of-Service bei Codeausführung | Strikte Fuel- und Timeout-Budgets in `contextra-sandbox` | `docs/SPEC.md` §13.2 |
| Schlüsselreste im Arbeitsspeicher | Implementierung von `ZeroizeOnDrop` und Memory-Locking (`mlock`) | `docs/SPEC.md` §10.1, §12.3 |

---

## 4. Nicht im Modell (Ausschlüsse)

- Angriffe auf die physische Hardware, CPU-Seitenkanäle auf Hardware-Ebene oder Kernel-Kompromittierungen des Host-Betriebssystems.
- Auslagerungsspeicher (Swap) und Memory-Dumps, sofern das Betriebssystem `mlock` oder `MADV_DONTDUMP` ignoriert oder einschränkt.
- Flash-Wear-Leveling und interne Überprovisionierung physischer SSD-Controller bei direktem Speicherüberschreiben.
- Nicht autorisierte Modifikationen der Anwendungs-Binaries nach dem Kompilieren.

---

## 5. Meldung von Sicherheitsbefunden

Sicherheitsbefunde bitten wir vertraulich direkt an die Maintainer des Repositories zu übermitteln und nicht über öffentliche GitHub-Issues zu teilen. Befunde werden priorisiert analysiert und gemäß den Baukasten-Qualitätsstandards behoben.

---

## 6. Verweise

- `docs/SPEC.md` (Fassung 4.1): §2 (B1–B12), §3 (Architekturprinzipien), §6 (Löschkette & Erasure), §10 (Low-Level), §12 (Krypto & Persistenz)
- `docs/VETOES.md` (VETO-F10, VETO-F02, VETO-ADR)
- `docs/CONSTITUTION.md` (Architektur- und Sicherheitsverfassung)
