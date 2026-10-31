# Contextra – Sicherheitsmodell & Bedrohungsanalyse (Security Model)

<!-- doc-stamp: 2026-10-31 | Normativ: docs/SPEC.md §3, §8, §9, §12 | PLAN.md §6 -->

Dieses Dokument beschreibt das Bedrohungsmodell, die Sicherheitsarchitektur und den Härtungsfahrplan für Contextra.

---

## 1. Schutzgüter & Vertrauensgrenzen

| Schutzgut | Speicherort | Primärer Schutzmechanismus |
|---|---|---|
| **Dokumente & Metadaten** | SSTables, MemTable, WAL | AEAD-Verschlüsselung (AES-GCM-SIV / ChaCha20-Poly1305), HMAC-Verkettung im WAL, CRC32C in Datenblöcken. |
| **Schlüsselmaterial** | RAM des Prozesses | KDF via Argon2id (PLAN S-01), Zeroization beim Drop (`zeroize`), Paging-Schutz (`mlock` via `contextra-sys`). |
| **Mandantentrennung** | Alle Schichten | `TenantId` aus dem `RequestContext`, Präfix-Isolation, Partitionsfilter vor Distanzberechnungen (VETO-F10). |
| **Suchindizes** | HNSW (Vektor), BM25 (Text) | Physischer Purge bei Löschung, Rebuild nach VETO-F02, Isolation flüchtiger Segmente. |
| **Löschbeweise** | Export / MCP | Manipulationssichere Signaturkette (Ed25519 + HMAC), Prüfung auf Mindestentropie (>= 32 Bytes). |

### Vertrauensgrenzen
- **Prozessgrenze:** Contextra läuft eingebettet im Prozess der Trägeranwendung. Die Trägeranwendung wird als kooperativ vorausgesetzt; memory-dump-Schutz erfolgt über `mlock` und `Zeroize`.
- **Dateisystemgrenze:** Das Datenverzeichnis wird als feindlich bzw. unzuverlässig betrachtet. Unvollständige Schreibvorgänge, Crash-Reste oder unautorisierte Änderungen führen zu sofortigem Abbruch (INV-FAILCLOSED-1).
- **Mandantengrenze:** Anfragen verschiedener Mandanten sind informationstheoretisch und physisch strikt isoliert. Kein Mandant kann Indizes, Caches oder Daten eines anderen Mandanten einsehen (VETO-F10).
- **Netzgrenze (Single-Node):** Der Kern öffnet keinerlei Netzwerk-Sockets. Externe Kommunikation ist ausschließlich optional über das Adapter-Crate `contextra-infer-ollama` an eine lokale REST-Schnittstelle möglich.

---

## 2. Sicherheitsbefunde & Härtungsplan (PLAN §6)

Folgende kritische Sicherheitsbefunde sind im Rahmen von Stufe 1 und Stufe 2 zu härten:

| ID | Schwere | Befund & Fundstelle | Verbindliche Gegenmaßnahme |
|---|---|---|---|
| **S-01** | Kritisch | Passphrase-Ableitung via HKDF-SHA256 (`crypto.rs:88`) | Umstellung des Standard-KDF auf **Argon2id** (`try_new_with_kdf`). HKDF nur noch für Subkey-Ableitung aus sicherem Master-Material. |
| **S-02** | Kritisch | Verschlüsselung standardmäßig inaktiv (`config.rs`) | Invertierung des Defaults: **Verschlüsselung ist standardmäßig aktiv**. Unverschlüsselter Betrieb erfordert expliziten Opt-Out (`InsecurePlaintextStorage`). |
| **S-03** | Kritisch | Mandantenmodus optional (`TenantMode::Optional`) | Umstellung auf **`TenantMode::Required` als Standard**. Anfragen ohne `RequestContext` werden abgelehnt. |
| **S-04** | Kritisch | Automatische LLM-Entitätsextraktion aktiv | Standardmäßig auf **`Disabled`** setzen, um unbemerkten PII-Abfluss zu verhindern. |
| **S-05** | Hoch | Passphrase als Klartext-String im Builder (`builder.rs:355`) | Kapselung in `SecretString` mit redigiertem `Debug`-Format (`[REDACTED]`) und automatischer Speicher-Nullung. |
| **S-06** | Hoch | ODF-Dekompressionsbudget umgehbar (`odf/mod.rs:124`) | Hartes Limit auf tatsächlich entpackte Bytes bei allen Archiv-Formaten (Schutz vor Decompression Bombs). |
| **S-07** | Hoch | EPUB Fail-Open bei Entschlüsselungsfehlern (`epub.rs:122`) | Konsequentes Fail-Closed: Beschädigte oder verschlüsselte DRM-Dateien führen zum Abbruch. |
| **S-09** | Hoch | Fehlende Entropieprüfung für Proof-Keys (`mcp/proof_key.rs`) | Erzwingen von mindestens 256 Bit (32 Bytes) Entropie für Signaturschlüssel; Konsolidierung redundanter Funktionen. |
| **S-10** | Mittel | Egress-Audit standardmäßig No-Op | Scharfschalten lokaler Egress-Audits in `contextra-infer-ollama`. |
| **S-16** | Mittel | WAL-Integritätsschlüssel im Klartext (`wal/hmac.rs:221`) | Schlüssel wird über den `KeyManager` geschützt statt als ungeschützte Datei abgelegt. |

---

## 3. Zweistufiges Löschmodell (SPEC §12)

Zur Einhaltung von Art. 17 DSGVO ohne Leistungsverlust kombiniert Contextra zwei Sicherheitsstufen:

1. **Ebene A (Arbeitsspeicher & aktive Indizes):** Physischer Purge. Löschung entfernt Vektoren aus dem HNSW-Index (Tombstone + Rebuild nach VETO-F02), bereinigt BM25-Posting-Listen und invalidiert MemTables.
2. **Ebene B (Persistenter LSM-Speicher):** Krypto-Löschung (*Crypto-Shredding*). Der dokumentspezifische Schlüssel (`DocDek`) in der `KeyRegistry` wird unwiederbringlich genullt (`zeroize`). Persistierte SSTables bleiben physisch unverändert, sind jedoch informationstheoretisch unentschlüsselbar.

---

## 4. Meldung von Sicherheitslücken (Vulnerability Disclosure)

Sicherheitsrelevante Schwachstellen dürfen nicht über öffentliche GitHub-Issues gemeldet werden. Bitte senden Sie vertrauliche Berichte mit PoC an: `security@contextra.local` (oder PGP-verschlüsselt an die Maintainer).
