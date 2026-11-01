# Contextra – Sicherheitsmodell & Bedrohungsanalyse (Security Model)

<!-- doc-stamp: 2026-11-01 | Normativ: docs/SPEC.md §10, §11, §12 | PLAN.md Phase 2 -->

Dieses Dokument beschreibt das Bedrohungsmodell, die Sicherheitsarchitektur und die Sicherheits-Defaults für Contextra gemäß [docs/SPEC.md](SPEC.md) und [PLAN.md](../PLAN.md).

---

## 1. Schutzgüter & Vertrauensgrenzen

| Schutzgut | Speicherort | Primärer Schutzmechanismus |
|---|---|---|
| **Dokumente & Datensätze** | SSTables, MemTable, WAL | Pflichtverschlüsselung mit AES-256-GCM-SIV, Envelope-Format (SPEC §9.3), HMAC-SHA256-Kette im WAL, CRC32 für Torn-Write-Erkennung. |
| **Schlüsselmaterial** | RAM des Prozesses | KDF via Argon2id ($m \ge 64\,\text{MiB}, t \ge 3, p \ge 1$), HKDF-Ableitung in `KeySet` (SPEC §13.1), `Zeroizing<T>`, Paging-Schutz (`mlock` via `contextra-sys`). |
| **Mandantentrennung** | Alle Schichten | Verpflichtende `TenantId`, physische und logische Isolation über `TenantScopedStorage` (T-ARCH-2, VETO-F10). |
| **Suchindizes** | HNSW (Vektor), BM25 (Text) | AEAD-verschlüsselte Chunks und Segmente auf Platte; Slot-Zeroing im RAM; physischer Purge-Rewrite bei Löschung. |
| **Löschberichte** | Dateisystem / Export | Asymmetrisch Ed25519-signierter Bericht `ctx-erasure-report/1` mit E1/E2/E3-Residuenmessung. Keine Klartext-IDs. |

### Vertrauensgrenzen
- **Prozessgrenze:** Contextra läuft eingebettet im Prozess der Trägeranwendung. Memory-Dump-Schutz erfolgt über `mlock_best_effort` und `Zeroize`.
- **Dateisystemgrenze:** Das Datenverzeichnis wird als potentiell feindlich oder fehleranfällig betrachtet. Manipulationen, unvollständige Schreibvorgänge oder CRC-Mismatches führen zum sofortigen kontrollierten Abbruch (INV-FAILCLOSED-1).
- **Mandantengrenze:** Anfragen verschiedener Mandanten sind informationstheoretisch und physisch strikt isoliert (VETO-F10). Kein geteilter Cache, keine gemischten SSTables, keine mandantenübergreifenden Kanten.
- **Netzgrenze (Single-Node-Doktrin):** Der Kern öffnet keinerlei Netzwerk-Sockets und enthält keine Netzwerk-Crates im Produktionsbaum.

---

## 2. Verbindliche Sicherheits-Defaults (SPEC §10)

| ID | Gegenstand | Sicherheits-Vorgabe (Normativ) | Begründung |
|---|---|---|---|
| **D-05** | Verschlüsselung | **Pflicht ohne Opt-Out** (`EncryptionMode::Required`). Kein unverschlüsselter Block- oder SSTable-Pfad im Produktionscode. | Schutz vor Datenremanenz auf persistenten Medien. |
| **D-06** | Passphrase-KDF | **Argon2id mit $m \ge 64\,\text{MiB}, t \ge 3, p \ge 1$** (`derive_master`). HKDF für Passphrasen ist strikt verboten (B-15). | Brute-Force-Resistenz gegen GPU/ASIC-Angriffe. |
| **D-07** | Mandantenisolation | **Verpflichtend ohne Opt-Out** (`TenantPolicy::Required`). `TenantId` stammt aus dem Kontext. | Ausschluss versehentlicher mandantenloser Speicherung. |
| **D-27** | Entitätsextraktion | **Vollständig entfallen in v1**. Kein unkontrollierter PII-Abfluss an externe LLM-Dienste. | Minimierung der Angriffsfläche und Datenschutzrisiken. |
| **D-29** | Unsafe-Code | **Strikt auf 2 Inseln beschränkt** (`contextra-sys`, `contextra-simd`). Alle anderen Crates erzwingen `#![forbid(unsafe_code)]`. | Speichersicherheit des Gesamtsystems. |
| **D-31** | Schlüsselraum | **Blinde Tags:** `doc_tag`, `coll_tag`, `tenant_tag`, `term_tag` via keyed BLAKE3 (`blind_tag`). Kein Klartext in Schlüsseln. | Schutz vor Metadaten-Leaks in Indizes und Bloom-Filtern. |

---

## 3. Lösch- und Nachweismodell (SPEC §11, §12, ADR-001)

Gemäß ADR-001 (SPEC §3.3, D-01) implementiert Contextra in v1 ein belegbares Löschmodell basierend auf physischer Bereinigung und Verifikation:
1. **Sofortige Lesesperre (`ReadGate`):** Sobald ein Löschauftrag `Gated` ist, blockiert `ErasedSet` fail-closed alle Lese- und Suchpfade auf Zielschlüssel. Aktive Snapshots, die Zielversionen sehen könnten, werden widerrufen (`SnapshotRevoked`).
2. **Physisches Purging (Ebene A):**
   - Store: `drain` versiegelt WAL; `rewrite_file` streamt neue SSTables ohne Zieldaten (Originalsequenzen bleiben erhalten, I-2); atomarer `swap` via Manifest; Block-Cache Eviction mit Zeroize.
   - Vector: `zero_slot` überschreibt RAM-Bytes mit Nullen, Tombstone-Flag, Rewrite betroffener Snapshot-Chunks, Rebuild ab Schwelle.
   - Text: Puffer leeren, betroffene Segmente ohne Zieldaten neu schreiben, Alt-Segmente löschen, Term-Statistik neu aggregieren.
3. **Residuenmessung & Zeugen:**
   - **E1**: Byte-genauer Rohscan des Datenverzeichnisses nach Needles.
   - **E2**: Strukturierter Scan nach Dekodierung aller Container.
   - **E3**: Nachweis der Schlüssel- und Slot-Abwesenheit.
   - Positivkontrolle (Canary) vor jeder Messung beweist Detektionsfähigkeit.
4. **Bericht `ctx-erasure-report/1`:**
   Ed25519-signiert, maschinenlesbar, enthält keine Klartext-IDs oder Personendaten. Verifizierbar durch die unabhängige Binärdatei `ctx-verify`.

---

## 4. Meldung von Sicherheitslücken (Vulnerability Disclosure)

Sicherheitsrelevante Schwachstellen dürfen nicht über öffentliche Issue-Tracker gemeldet werden. Vertrauliche Berichte mit Proof-of-Concept richten Sie bitte direkt an die Core-Maintainer.
