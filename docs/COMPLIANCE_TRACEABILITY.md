# Compliance- und Traceability-Dokumentation: EU-Produkthaftung, Cyber Resilience Act, DSGVO-Rechenschaft

**Dokument-Status:** Technischer Compliance-Nachweis (keine Rechtsberatung).
**Normative Grundlage:** `docs/SPEC.md` (insbesondere Abschnitte 1.5, 1.6, 5, 6, 11, 12, 15, 16 sowie Anhänge B, C, D, F).

---

## 1. Einzige zulässige Außenaussage (SPEC §6.4)

Für alle Dokumente, Berichte, Metadaten und Außenaussagen gilt ausnahmslos der Wortlaut aus SPEC §6.4:

> **DE:** Contextra entfernt die Daten eines Löschziels aus dem Datenverzeichnis, aus den Indexen und aus den gespeicherten Zeitreise-Versionen und weist den Zustand je Schicht messtechnisch nach. Für jedes Ziel wird zusätzlich der zugehörige Schlüssel aus dem Schlüsseltresor entfernt; ob und wie das rechtlich zu werten ist, entscheidet der Betreiber. Nicht erfasst sind externe Sicherungen, Flash-Remanenz, Dateisystem-Journale, Prozessspeicher und externe Verarbeiter. Der Bericht listet diese Restrisiken.
>
> **EN:** Contextra removes the data of an erasure target from the data directory, from its indexes and from stored time-travel versions, and provides per-layer, measured evidence of the resulting state. For each target the associated key is additionally removed from the key vault; whether and how this is to be assessed legally is for the operator to decide. Not covered are external backups, flash remanence, file-system journals, process memory and external processors. The report lists these residual risks.

Unzulässige Außenaussagen und nicht belegbare Werbeaussagen sind untersagt und werden per automatisiertem Wording-Lint blockiert (SPEC §1.6, §6.4).

---

## 2. Rechtlicher Rahmen & Technische Haftungsumsetzung

| Rechtsrahmen | Vorschrift / Norm | Risiko / Pflicht | Technische Umsetzung in Contextra | Nachweis / Test / Evidenz |
|---|---|---|---|---|
| **EU-Produkthaftung (ProdHaftG)** | RL (EU) 2024/2853 / ProdHaftG (ab 09.12.2026) | Fehlerhaftes Produkt, Datenverlust, unbemerkte Korruption, Verletzung von Nachbardaten | Ziel-Isolation: Bitidentische Erhaltung von Nicht-Zieldaten (P4, T-ISO-1); WAL-Integrität und HMAC-Kette (T-WAL-1); Crash- und Recovery-Konsistenz (T-CRASH-1..5); fsync-Vergiftung statt blinder Wiederholung (D-55, T-FSYNC-1). | `T-ISO-1`, `T-CRASH-1..5`, `T-WAL-1`, `T-FSYNC-1` |
| **Cyber Resilience Act (CRA)** | VO (EU) 2024/2847 Art. 14 (seit 11.09.2026) | Meldepflicht für aktiv ausgenutzte Schwachstellen und schwere Vorfälle | Dokumentierter Meldeweg an ENISA Single Reporting Platform (D-67); Sicherheitsrichtlinie in `docs/SECURITY.md`; Fristen: kritisch 7 Tage, hoch 30 Tage (HR-8). | `docs/SECURITY.md`, `T-SUPPLY-1` |
| **Cyber Resilience Act (CRA)** | VO (EU) 2024/2847 Art. 11 / Anhang I | Security by Design, Schwachstellenmanagement, SBOM | Vollständiges Verbot von Netzwerk-, Telemetrie- und Escrow-Komponenten im Produktionscode (HR-1..HR-3); strenge Dependency-Bans in `deny.toml` (D-54); SBOM-Erstellung (`cargo cyclonedx`); Toolchain-Pinning (D-82). | `T-ARCH-4`, `deny.toml`, `Cargo.lock` |
| **DSGVO Rechenschaftspflicht** | Art. 5 Abs. 2, Art. 24 DSGVO | Nachweisbarkeit und Überprüfbarkeit der Löschung | Ed25519-signierter Bericht im kanonischen JSON-Format (D-13, F.10); Messung aller Schichten (MK1, MK2, MK3) mit Positivkontrolle (D-35, D-72); unabhängige Verifikation via `ctx-verify scan`. | `T-PROOF-1`, `T-REPORT-1`, `T-RAW-2` |
| **DSGVO Recht auf Löschung** | Art. 17 DSGVO / SDM-Baustein 60 | Unkenntlichmachen, O(1)-Sperre, physischer Purge | Sofortige Sperre aller Lesepfade inkl. Zeitreisen (D-12, D-30, T-VER-1); O(1) Schlüsselvernichtung im Tresor (D-33, T-VAULT-1..4); physischer Purge in LSM, Text und Vektor (11.2); Dateischlüsselvernichtung (D-85, T-FK-1..3). | `T-GATE-1`, `T-VAULT-1..4`, `T-FK-1..3`, `BM-ERASE-O1` |
| **DSGVO Recht auf Einschränkung** | Art. 18 DSGVO / Art. 18 Abs. 2 | Sperre bei Erhalt der Daten, Zweckbindung für Rechtsansprüche | Gate-Variante `Restricted` (D-15, D-124); auditierter Zugriffspfad `get_restricted`/`scan_restricted` nur mit `RestrictedPurpose` und Journal-Eintrag vor Klartext-Rückgabe (D-116). | `T-RESTR-1..4` |
| **DSGVO Datenschutz durch Technik** | Art. 25 DSGVO | Privacy by Design und Privacy by Default | Nicht abschaltbare Block- und Record-Verschlüsselung (P5, D-32); Mandantentrennung im Typensystem erzwungen (`TenantContext`, D-07, T-API-1); Pseudonyme keyed BLAKE3-Tags statt Klartextkennungen (D-21, D-75). | `T-DEF-1`, `T-API-1`, `T-RAW-4`, `T-RAW-6` |
| **DSGVO Sicherheit der Verarbeitung** | Art. 32 DSGVO | Stand der Technik bei Kryptografie und Härtung | Argon2id mit strengen Parametern (D-06); Keyslots mit vollständiger AAD-Bindung (D-52); Speichersperre (`mlock`) und Coredump-Abschaltung (D-63). | `T-KDF-1/2`, `T-KEYSLOT-1..4`, `contextra-sys` |

---

## 3. Restrisiko-Register (SPEC Anhang D)

Jede Löschung erzeugt einen signierten Bericht, der technische Einschränkungen und Umgebungsrisiken explizit ausweist. Folgende Restrisiken sind normativ definiert:

| Restrisiko-Code | Beschreibung | Technische Ursache & Systemgrenze | Betreiberpflicht / Kompensationsmaßnahme |
|---|---|---|---|
| `FlashRemanence` | Physikalische Restdaten in Flash-Speicherzellen | Wear-Leveling und interne Speichercontroller von SSDs/NVMe entziehen physische Blöcke dem direkten Überschreiben. | Einsatz von Full-Disk-Encryption (FDE) auf Betriebssystemebene mit Schlüsselvernichtung. |
| `FsJournalOrCow` | Metadaten und Datenfragmente im Dateisystem-Journal oder Copy-on-Write-Kopien | Dateisysteme (ext4-Journal, XFS-Log, ZFS/Btrfs CoW) puffern Blöcke außerhalb der Bibliothekskontrolle. | Dateisystem-Wahl gemäß Plattform-Vorgaben (SPEC D-56, D-114); Verzicht auf unkontrollierte Dateisystem-Snapshots. |
| `WrappedKeyRemanence` | Gewickelte Schlüsselreste nach Rehash oder Keyslot-Änderung | Prozessabsturz zwischen atomarem Rename und vollständigem Null-Überschreiben der Shard-Altdatei (D-111). | Wiederholung des Shard-Scrubbing (`vault_scrub`); physische Speicherlöschung bei Außerbetriebnahme. |
| `ExternalCopies` | Kopien außerhalb des `data_dir` (Backups, Replikate, Logs) | Manuelle oder automatisierte Kopien des Betreibers liegen außerhalb der Löschgrenze von Contextra. | Bereinigung externer Sicherungen; Einsatz des Restore-Guards (D-37, D-126) gegen Replay gelöschter Daten. |
| `ProcessMemory` | Residuen im RAM flüchtiger Prozesse | Daten im Arbeitsspeicher vor `mlock` oder temporäre Pufferstrukturen vor Garbage Collection / Zeroization. | Prozesshärtung aktivieren (`process_hardening = Required`), Swap verschlüsseln oder deaktivieren (D-63). |
| `BackupKeyMaterial` | Gemeinsame Sicherung von Schlüsseltresor und Datenverzeichnis | Liegt `vault_dir` im Datenverzeichnis oder wird gemeinsam gesichert, hebeln Backups die Schlüsselvernichtung aus (D-112). | `vault_dir` außerhalb von `data_dir` betreiben; Attestierung `SeparateShortRetention` mit kurzer Halbwertszeit. |
| `VerifiedViaCache` | Verifikation erfolgte über `posix_fadvise` oder Page-Cache statt Direct-I/O | Laufwerk oder Betriebssystem unterstützt kein `O_DIRECT` oder die Startprobe meldet `direct_io = false` (D-113). | Nutzung von zertifizierten Blockgeräten und Dateisystemen mit Direct-I/O-Unterstützung. |
| `IndexTopologyInference` | Rückschlussmöglichkeiten aus HNSW-Graphtopologie vor Rebuild | Surrogat ersetzt Vektordaten, Kanten bleiben bis zum Batch-Rebuild bestehen (D-77, D-122). | Aktivierung von `vector_strict_rebuild` bzw. Durchführung des periodischen Rebuilds. |
| `TextIndexUntilPurge` | Entkoppelte Postings (`unlinked`) bis zum Abschluss des Purge-Fensters | Unveränderliche Segmente halten Chiffrierfragmente bis zum Segment-Rewrite (D-49, D-121). | Purge-Fenster nicht unnötig verzögern (`batch_window <= erase_sla / 2`). |
| `SupersededWithSameKey` | Berichtigung mit Purge alter Versionen (`Supersede::Purge`) | Der DEK bleibt unverändert, da die Dokumentidentität fortbesteht (Klasse K `not_reached`, D-117). | Falls vollständige Schlüsselvernichtung gefordert ist: `replace_erasing` mit neuer Dokumentidentität nutzen (D-115). |

---

## 4. Unsafe-Register & Speichersicherheits-Audit

Gemäß SPEC P3, D-29 und AR-8 ist `unsafe` im gesamten Workspace **ausschließlich** in `contextra-sys` und `contextra-simd` gestattet. Alle anderen 12 Produktions-Crates setzen strikt `#![forbid(unsafe_code)]`.

### 4.1 `contextra-sys` (OS-Syscall-Insel)
- **Gesamtanzahl Blöcke:** 19 Blöcke (unterhalb des Basislimits von 57).
- **Prüfung:** Automatisierte Verifikation über `crates/contextra-sys/tests/sy_audit_unsafe.rs`.
- **Zweck:** Ausschließliche Kapselung minimaler POSIX/OS-Syscalls (`fsync`, `fdatasync`, `fcntl(F_FULLFSYNC)`, `statfs`, `mlock`, `munlock`, `madvise(MADV_DONTDUMP)`, `setrlimit`, `geteuid`).
- **Sicherheitsnachweis:** Jeder Block verfügt über einen verifizierten `// SAFETY:`-Kommentar mit Vorbedingung und Gültigkeitsnachweis (dokumentiert in `docs/unsafe-register.md`).

### 4.2 `contextra-simd` (Vektor-Distanz-Insel)
- **Gesamtanzahl Blöcke:** 202 Blöcke (Ratchet-Limit 285).
- **Zweck:** AVX2-, AVX-512- und NEON-f32-Distanzberechnungen (Dot Product, L2, Cosine).
- **Sicherheitsnachweis:** Keine `unsafe fn` in der öffentlichen Schnittstelle; alle Kernel haben sichere Entry-Points mit Vorabvalidierung (`validate_vector`); 100 % der Blöcke tragen `// SAFETY:`-Invarianten; vollständige Äquivalenzprüfung gegen Skalar-Referenz (T-SIMD-1, T-SIMD-2).

---

## 5. Software Bill of Materials (SBOM) & Lieferketten-Compliance

- **Lieferkettenintegrität (CRA Art. 11 / Anhang I):**
  - Dependency-Whitelist gemäß `governance/deps.toml` und Richtlinie D-105.
  - Ban aller Netzwerk-Bibliotheken (`mio`, `socket2`, `reqwest`, `hyper`, `tokio/net` etc.) via `deny.toml` (HR-1).
  - Verbot ungetesteter Cargo-Features und Schalter für Sicherheitsmechanismen (P5, D-84).
- **Compiler & Toolchain:**
  - Gepinnte Build-Toolchain: Rust 1.94.0 (SPEC D-82).
  - Minimal Supported Rust Version (MSRV): Rust 1.89.0.
- **Meldewege & Vulnerability Management (CRA Art. 14):**
  - Sicherheitsmeldung via `security@contextra.local` / PGP-Schlüssel (SPEC 17.5).
  - Reaktionsfristen: Kritisch (CVSS >= 9.0): Patch & Advisory innerhalb von 7 Tagen; Hoch (CVSS 7.0–8.9): 30 Tage.
  - Gesetzliche Meldung aktiv ausgenutzter Schwachstellen an die ENISA Single Reporting Platform innerhalb von 24 Stunden.

---

## 6. Traceability-Matrix: SPEC-Anforderung → Test-ID → Nachweis

| SPEC-ID | Themenbereich / Anforderung | Normative Vorgabe | Verknüpfte Test-ID(s) | Implementierter Nachweispfad / Testdatei | Status / Evidenz |
|---|---|---|---|---|---|
| **D-01** | Drei-Säulen-Modell | Gate & Purge, Schlüsselvernichtung, Messung | `T-GATE-1`, `T-VAULT-1`, `T-RAW-1` | `crates/contextra-erasure/tests/` | Verifiziert |
| **D-03** | Asynchrone Schnittstelle | Vollständig asynchron, statischer Dispatch | `T-API-1` | `crates/contextra/tests/` | Verifiziert |
| **D-05** | Pflichtverschlüsselung | Nicht abschaltbare Verschlüsselung, kein Klartext | `T-DEF-1`, `T-CFG-1` | `crates/contextra-crypto/tests/` | Verifiziert |
| **D-06** | KDF-Grenzen | Argon2id m >= 64 MiB, t >= 3, p >= 1 | `T-KDF-1`, `T-KDF-2` | `crates/contextra-crypto/tests/` | Verifiziert |
| **D-07** | Mandanten-Bindung | `TenantContext` bei jedem Aufruf zwingend | `T-API-1` | `crates/contextra/tests/` | Verifiziert |
| **D-08** | Keyed Tags | Pseudonyme keyed BLAKE3-Tags | `T-RAW-4` | `crates/contextra-crypto/tests/` | Verifiziert |
| **D-09** | Schlüsselsortierung | Lexikografische Sortierung `DATA ‖ ...` | `T-KEY-1` | `crates/contextra-types/tests/` | Verifiziert |
| **D-10** | Matcher-Länge | Feste Tag-Länge, keine variablen Präfixe | `T-ISO-2`, `T-KEY-1` | `crates/contextra-types/tests/` | Verifiziert |
| **D-11** | Snapshot-Widerruf | Widerruf offener Mandanten-Snapshots `seq <= gated_seq` | `T-VER-1`, `T-SNAP-1` | `crates/contextra-mvcc/tests/` | Verifiziert |
| **D-12** | ReadGate | Sofortige Sperre vor Purge | `T-VER-1`, `T-GATE-1` | `crates/contextra-erasure/tests/` | Verifiziert |
| **D-13** | Signierter Bericht | Ed25519-Signatur über kanonisches JSON | `T-PROOF-1`, `T-REPORT-1` | `crates/contextra-erasure/tests/` | Verifiziert |
| **D-14** | Betroffenen-Löschung | Löschung über `SubjectRef` / `cluster_tag` | `T-SUBJ-1` | `crates/contextra-engine/tests/` | Verifiziert |
| **D-15** | Restriktion (Art. 18) | `RetentionAttestation`, Gate `Restricted` | `T-API-2`, `T-RESTR-1` | `crates/contextra-engine/tests/` | Verifiziert |
| **D-16** | Keine Klone | Keine Checkpoints, Forks oder Hardlinks | `T-ARCH-3` | `tests/arch/tests/arch.rs` | Verifiziert |
| **D-18** | Vektor-Löschung | Surrogat-Ersetzung, Kantenreparatur, Rebuild | `T-VEC-DEL-1..3` | `crates/contextra-vector/tests/` | Verifiziert |
| **D-19** | Vektor-Snapshots | Envelope unter DEK, Bindung im Chiffrat | `T-RAW-3`, `T-KEYCOV-1` | `crates/contextra-vector/tests/` | Verifiziert |
| **D-20** | Textindex-Segmente | Verschlüsselte autonome Textsegmente | `T-TEXT-1..9` | `crates/contextra-text/tests/` | Verifiziert |
| **D-21** | BLAKE3-Labels | Konstante Kontext-Labels für alle Tags | `T-RAW-4` | `crates/contextra-crypto/tests/` | Verifiziert |
| **D-26** | Rank-Umfang | Reine RRF-Fusion (`k = 60`) | `T-RANK-1` | `crates/contextra-rank/tests/` | Verifiziert |
| **D-28** | SIMD-Sicherheit | f32-Kernel mit Skalar-Cross-Check, kein VNNI | `T-SIMD-1`, `T-SIMD-2` | `crates/contextra-simd/tests/` | Verifiziert |
| **D-29** | Unsafe-Inseln | `unsafe` nur in `sys` und `simd` | `T-UNSAFE-1` | `tests/arch/tests/arch.rs` | Verifiziert |
| **D-30** | O(1)-Gate-Latenz | Latenz unabhängig von Zieldokumentanzahl | `T-SLA-1`, `BM-ERASE-O1` | `crates/contextra-erasure/tests/` | Verifiziert |
| **D-31** | Löschcluster-Raum | Zusammenhängende Schlüsselbereiche je Ziel | `T-KEY-1`, `BM-ERASE-WA` | `crates/contextra-store/tests/` | Verifiziert |
| **D-32** | Record-Envelope | Zweischichtige AEAD-Verschlüsselung (DEK + Dateischlüssel) | `T-ENC-1..4`, `T-VAULT-1` | `crates/contextra-store/tests/` | Verifiziert |
| **D-33** | Tresor-Hierarchie | Slots T, C, S, D, F, E; Nullung und Prüfung | `T-VAULT-1..4` | `crates/contextra-erasure/tests/` | Verifiziert |
| **D-35** | Messklassen MK1-3 | Messung mit Positivkontrolle (Struktur, Needle, Key) | `T-RAW-2` | `crates/contextra-erasure/tests/` | Verifiziert |
| **D-36** | Klassenverband G,K,P | Ausweisung der Schutzklassen im Bericht | `T-CLASS-1`, `T-KEYCOV-1` | `crates/contextra-erasure/tests/` | Verifiziert |
| **D-37** | Restore-Guard | Replay-Schutz gelöschter Daten über Ledger | `T-RESTORE-1..3` | `crates/contextra-erasure/tests/` | Verifiziert |
| **D-38** | Mandanten-Erase | O(1)-Offboarding über T-Slot-Vernichtung | `T-TENANT-1` | `crates/contextra-erasure/tests/` | Verifiziert |
| **D-39** | Purge-Fenster | Bündelung bis `batch_window <= erase_sla / 2` | `BM-ERASE-BATCH` | `crates/contextra-erasure/tests/` | Verifiziert |
| **D-40** | Betroffenen-Identität | Betroffener unveränderlicher Teil der Dokument-ID | `T-SUBJ-2` | `crates/contextra-engine/tests/` | Verifiziert |
| **D-41** | Scan-Ordnung | Aufsteigend nach `(cluster_tag, doc_tag)` | `T-SCAN-1`, `T-SCAN-2` | `crates/contextra-store/tests/` | Verifiziert |
| **D-42** | Header-Versionierung | Format- und Algorithmus-IDs in jedem Header | `T-FMT-1` | `crates/contextra-store/tests/` | Verifiziert |
| **D-43** | Zero-Network | Kein Netzwerk, keine Telemetrie, kein Escrow | `T-ARCH-4` | `tests/arch/tests/arch.rs` | Verifiziert |
| **D-48** | Commitment-Epoche | Rotation nach `commit_epoch`, Unverknüpfbarkeit alter Berichte | `T-JRN-3`, `T-EPOCH-1` | `crates/contextra-erasure/tests/` | Verifiziert |
| **D-50** | Exklusiver Lock | Exklusiver Prozess-Lock über `Vfs` | `T-LOCK-1` | `crates/contextra-durable-fs/tests/` | Verifiziert |
| **D-51** | Soft-Delete | `soft_delete` ist Tombstone, keine Löschaussage | `T-API-3`, `T-LINT-1` | `crates/contextra/tests/` | Verifiziert |
| **D-52** | Keyslots | AAD-Bindung aller KDF- und Slot-Parameter | `T-KEYSLOT-1..4`, `T-HDR-1` | `crates/contextra-crypto/tests/` | Verifiziert |
| **D-53** | Schlüsselanker-Port | Austauschbarer Port `RecordSealer`/`FileKeyring` | `T-ANCHOR-1` | `crates/contextra-ports/tests/` | Verifiziert |
| **D-54** | Tokio-Minimalismus | Keine Netzwerk-/FS-Features von Tokio im Kern | `T-ARCH-4`, `deny.toml` | `tests/arch/tests/arch.rs` | Verifiziert |
| **D-55** | Fsync-Poisoning | Handle bei fsync-Fehler vergiften, keine Wiederholung | `T-FSYNC-1` | `crates/contextra-durable-fs/tests/` | Verifiziert |
| **D-70** | Adressierung | Direkte Adressierung ohne Locator-Region | `T-DOCREF-1` | `crates/contextra-types/tests/` | Verifiziert |
| **D-72** | Positivkontrolle | Zeuge `LayerWitness` nur mit Positivkontrolle | `T-RAW-2` | `crates/contextra-erasure/tests/` | Verifiziert |
| **D-74** | Ledger-Lebensdauer | Ledger-Aufbewahrung = `backup_retention` | `T-RESTORE-3` | `crates/contextra-erasure/tests/` | Verifiziert |
| **D-75** | Pseudonyme Pfade | Nur Tags in Datei- und Verzeichnisnamen | `T-RAW-6` | `crates/contextra-engine/tests/` | Verifiziert |
| **D-77** | Surrogat-Ersetzung | Navigation über Surrogat vor Schlüsselvernichtung | `T-VEC-DEL-1..2` | `crates/contextra-vector/tests/` | Verifiziert |
| **D-80** | Algorithmen-Satz | AES-256-GCM-SIV, HKDF, HMAC, BLAKE3, Ed25519 | `T-KDF-2`, `T-ENC-1..4` | `crates/contextra-crypto/tests/` | Verifiziert |
| **D-81** | Textanalyse | UAX-#29, Snowball-German, `term_tag` keyed | `T-TEXT-ANALYZER-1` | `crates/contextra-text/tests/` | Verifiziert |
| **D-82** | Toolchain-Pinning | Aktuelle Stable, MSRV 1.89 | `T-SUPPLY-2` | `rust-toolchain.toml` | Verifiziert |
| **D-85** | Dateischlüssel-Löschung | Slot `F` vor Datei-Unlink genullt | `T-FK-1..3`, `T-CRASH-6` | `crates/contextra-store/tests/` | Verifiziert |
| **D-86** | Vektor-Arena | Zeroisierbarer Block für alle Vektor-Darstellungen | `T-VEC-ARENA-1` | `crates/contextra-vector/tests/` | Verifiziert |
| **D-87** | Feste Limits | Grenzwerte für Dokumentgröße, Dimension, Terme | `T-LIMIT-1` | `crates/contextra-types/tests/` | Verifiziert |
| **D-88** | Ring-Architektur | Strikte Einhaltung der Ringmatrix SPEC §5.3 | `T-ARCH-3`, `T-ARCH-6` | `tests/arch/tests/arch.rs` | Verifiziert |
| **D-89** | Fassaden-Schnitt | Fassade `contextra` ohne Geschäftslogik | `T-API-1`, `T-ARCH-7` | `tests/arch/tests/arch.rs` | Verifiziert |
| **D-90** | Namens-Regeln | Keine `util`/`misc`/`mod.rs`-Dateien | `T-ARCH-7` | `tests/arch/tests/arch.rs` | Verifiziert |
| **D-91** | Crate-Köpfe | Ringangabe und SPEC-Verweis im Header | `T-ARCH-5`, `T-LINT-4` | `tests/arch/tests/arch.rs` | Verifiziert |
| **D-107** | Zählwerkzeug | Syn-basierte Zählung von unwrap/expect/panic | `T-LINT-3`, `T-ARCH-8` | `tests/arch/tests/arch.rs` | Verifiziert |
| **D-111** | 16-Shard-Tresor | 64-Byte-Slots, atomare Nullung mit Tombstone-MAC | `T-VAULT-6..8` | `crates/contextra-erasure/tests/` | Verifiziert |
| **D-112** | Tresor-Platzierung | `vault_dir` standardmäßig außerhalb `data_dir` | `T-VAULT-9`, `T-REPORT-2` | `crates/contextra-erasure/tests/` | Verifiziert |
| **D-113** | Medium-Verifikation | `read_verified` mit Direct-I/O oder Cache-Drop | `T-VERIFY-1..2` | `crates/contextra-durable-fs/tests/` | Verifiziert |
| **D-114** | Startprobe | Prüfung Dateisystem, Atomarität und fsync-Plausibilität | `T-PLAT-2`, `T-OPS-3` | `crates/contextra-durable-fs/tests/` | Verifiziert |
| **D-115** | `replace_erasing` | Protokoll R1-R4 für Mehr-Personen-Dokumente | `T-REPL-1..3` | `crates/contextra-engine/tests/` | Verifiziert |
| **D-116** | Zugriff Art. 18(2) | `RestrictedPurpose` und Audit vor Klartext-Rückgabe | `T-RESTR-2..4` | `crates/contextra-engine/tests/` | Verifiziert |
| **D-117** | `Supersede::Purge` | Matcher `PrefixSeq`, Klasse K ehrlich `not_reached` | `T-SUP-1..3`, `T-REPORT-3` | `crates/contextra-engine/tests/` | Verifiziert |
| **D-118** | Erase-Observer | At-least-once Zustandsbenachrichtigung ohne PII | `T-OBS-1..3` | `crates/contextra-erasure/tests/` | Verifiziert |
| **D-119** | Idempotenz | Crash-Wiederaufnahme bestehender `ErasureId` | `T-ERID-1..3` | `crates/contextra-erasure/tests/` | Verifiziert |
| **D-121** | Text-Chunk-Bindung | Chunks mit `version_seq` und AEAD-Envelope | `T-TEXT-5..9` | `crates/contextra-text/tests/` | Verifiziert |
| **D-122** | HNSW-Löschung | Deterministischer Rebuild, Slot-Bindung | `T-VEC-5..8`, `T-SIMD-2` | `crates/contextra-vector/tests/` | Verifiziert |
| **D-123** | Fester Schlüsselraum | 78-Byte-Schlüssel mit Tags und komplementierter Seq | `T-KEY-1`, `T-EXISTS-1` | `crates/contextra-types/tests/` | Verifiziert |
| **D-124** | Status-Matrix | Matrix Aufbewahrungscode x Erase-Modus | `T-STATUS-1`, `T-TXN-1` | `crates/contextra-types/tests/` | Verifiziert |
| **D-125** | Epochen-Slot E | Zufälliger Epochenschlüssel im Slot E | `T-EPOCH-1` | `crates/contextra-erasure/tests/` | Verifiziert |
| **D-126** | Restore-Guard Alg. | Filterung gegen Ledger beim Öffnen | `T-RESTORE-1..3` | `crates/contextra-erasure/tests/` | Verifiziert |
| **D-127** | WAL-Purge-Ordnung | Versiegeln, Filtern, Flushen, Manifest, Segment-Drop | `T-WAL-3` | `crates/contextra-store/tests/` | Verifiziert |
| **D-128** | I/O-Spuren | Trennung Critical vs Normal auf separaten OS-Threads | `T-IO-1`, `T-LOCK-2` | `crates/contextra-store/tests/` | Verifiziert |
| **D-130** | ReportSigner-Port | Abstraktion der Signaturerzeugung für HSM-Support | `T-SIG-1` | `crates/contextra-ports/tests/` | Verifiziert |

---

## 7. BLOCKER-Meldung (Teil B)

- **Betroffenes Crate:** `crates/contextra-store`
- **Datei:** `crates/contextra-store/Cargo.toml` (Zeilen 38–51)
- **Fehlerbeschreibung:** Doppelte Deklaration von Abhängigkeitsschlüsseln (`aes-gcm-siv`, `hmac`, `sha2`, `hkdf`, `subtle`). Cargo bricht das Laden des Manifests mit Status 101 (`error: duplicate key`) ab, was alle workspace-weiten Cargo-Aufrufe blockiert.
- **Kompilierblocker im Crate:** In `crates/contextra-store/src/lsm/recovery.rs` fehlt der Re-Export/Import von `KeyManager` (`use crate::wal::KeyManager;`), und in `crates/contextra-store/src/lsm/scan.rs` wird `MAX_SCAN_MERGE_ACCUMULATOR` direkt aus `contextra_ports` referenziert, liegt jedoch in `contextra_ports::storage::MAX_SCAN_MERGE_ACCUMULATOR`.
- **Minimaler Lösungsvorschlag für Teil B:**
  1. In `crates/contextra-store/Cargo.toml` die doppelten Einträge in den Zeilen 38–42 entfernen.
  2. In `crates/contextra-store/src/lsm/recovery.rs` den Import `use crate::wal::KeyManager;` ergänzen.
  3. In `crates/contextra-store/src/lsm/scan.rs` den Import `use contextra_ports::storage::MAX_SCAN_MERGE_ACCUMULATOR;` ergänzen und die Zugriffe angleichen.
- **Entscheidung gem. Handlungsanweisung:** Da `contextra-store` Gegenstand aktiver Arbeiten in Teil B ist, nimmt der Compliance-Autor an dieser Stelle keine eigenmächtige Codeänderung vor, sondern meldet diesen Blocker ordnungsgemäß.
