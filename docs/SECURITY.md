# Contextra – Sicherheitsmodell & Bedrohungsanalyse (Security Model)

<!-- doc-stamp: 2026-11-02 | Normativ: docs/SPEC.md §6, §10, §11, §17 -->

Dieses Dokument beschreibt das Bedrohungsmodell, die Sicherheitsarchitektur, kryptografische Standards und die CRA-Meldeprozesse für Contextra gemäß [docs/SPEC.md](SPEC.md).

---

## 1. Schutzgüter & Vertrauensgrenzen

| Schutzgut | Speicherort | Primärer Schutzmechanismus |
|---|---|---|
| **Dokumente & Datensätze** | SSTables, MemTable, WAL | Pflichtverschlüsselung mit AES-256-GCM-SIV (SPEC §9.4, §10), HMAC-SHA256-Kette im WAL (D-73), CRC32 für Torn-Write-Erkennung. |
| **Schlüsselmaterial** | RAM & Tresor | KDF via Argon2id ($m \in [64\,\text{MiB}, 4\,\text{GiB}], t \in [3, 16], p \in [1, 16]$; D-06), Keyslots (D-52), `ZeroizeOnDrop`, Paging-Schutz (`mlock` via `contextra-sys`). |
| **Schlüsselhierarchie (Tresor)** | `vault/vault.dat` | Hierarchischer Tresor mit Open-Addressing (D-79): Slots `T` (Mandant), `C` (Collection), `S` (Betroffener), `D` (Dokument). O(1)-Vernichtung übergeordneter Slots macht alle darunterliegenden DEKs unableitbar. |
| **Dateischlüssel** | `vault/vault.dat` | Dateischlüssel-Vernichtung (D-85): Datendateien besitzen eigene Zufallsschlüssel als `F`-Slots; Freigabe nullt den `F`-Slot vor dem `unlink`. |
| **Mandantentrennung** | Alle Schichten | Verpflichtender `TenantContext`, physische und logische Isolation über `TenantScopedStorage` (AR-6, VETO-F10). |
| **Suchindizes** | HNSW (Vektor), BM25 (Text) | AEAD-verschlüsselte Chunks und Segmente auf Platte; Vektor-Surrogat und Kantenreparatur (D-77); Textpostings unter DEK gebunden (D-69). |
| **Löschberichte** | Dateisystem / Export | Asymmetrisch Ed25519-signierter Bericht `ctx-erasure-report/1` (D-13) mit MK1/MK2/MK3 Messung. Keine Klartext-IDs. |

### Vertrauensgrenzen
- **Prozessgrenze:** Contextra läuft eingebettet im Prozess der Trägeranwendung (Single-Node-Doktrin, VETO-SN01). Memory-Dump-Schutz erfolgt über `mlock`, `RLIMIT_CORE` und `PR_SET_DUMPABLE` via `contextra-sys`.
- **Dateisystemgrenze:** Das Datenverzeichnis wird als potentiell manipulierbar oder fehleranfällig betrachtet. Manipulationen, unvollständige Schreibvorgänge oder CRC-Mismatches führen zum sofortigen kontrollierten Abbruch (P2 Fail-Closed, VETO-FAILCLOSED).
- **Mandantengrenze:** Anfragen verschiedener Mandanten sind informationstheoretisch und physisch strikt isoliert (VETO-F10). Kein geteilter Cache, keine gemischten SSTables, keine mandantenübergreifenden Kanten.
- **Netzgrenze:** Der Kern öffnet keinerlei Netzwerk-Sockets und enthält keine Netzwerk-Crates im Produktionsbaum (AR-7, D-54).

---

## 2. Kryptografische Richtlinien & Schlüsselhierarchie

- **Master Key ($K_\text{master}$):** Zufälliger 256-Bit-Schlüssel, gespeichert in Keyslots (SPEC §10.2, D-52). Keyslot entschlüsselt mit $K_\text{slot} = \text{Argon2id}(\text{Passphrase}, \text{Salt}_\text{slot})$.
- **Argon2id-Grenzen (D-06):** Parameter werden vor Ausführung strikt validiert:
  - Speicher $m$: $64\,\text{MiB} \le m \le 4\,\text{GiB}$ (Standard $128\,\text{MiB}$)
  - Iterationen $t$: $3 \le t \le 16$ (Standard $3$)
  - Parallelität $p$: $1 \le p \le 16$ (Standard $4$)
  - Bei Grenzüberschreitung: Abbruch mit `ContextraError::Config`.
- **DEK-Ableitung (D-79):**
  $$\text{DEK} = \text{HKDF-SHA256}(\text{ikm}=K_\text{unit}, \text{salt}=\text{SHA-256}(\text{"ctx/dek-salt/v2"} \mathbin{\Vert} \text{salt}_T \mathbin{\Vert} \text{salt}_C \mathbin{\Vert} \text{salt}_S \mathbin{\Vert} \text{salt}_D), \text{info}=\text{"ctx/dek/v2"} \mathbin{\Vert} \text{doc\_tag})$$
- **Nonces (D-60):** 4-Byte Zufallspräfix je `KeyManager` + 8-Byte monoton steigender Zähler.

---

## 3. Cyber Resilience Act (CRA) & Schwachstellenmanagement (D-67, SPEC §17.3)

Contextra behandelt sich vorsorglich als den Vorgaben des Cyber Resilience Act (VO (EU) 2024/2847, Art. 14) und ProdHaftG unterliegend:
- **Meldewege:** Sicherheitslücken können verschlüsselt an das PSIRT gemeldet werden (z. B. via `security@contextra.dev` oder Sicherheitskontakt im Repo).
- **CRA Art. 14 Fristen für aktiv ausgenutzte Schwachstellen:**
  - *Frühwarnung:* Meldung an CSIRTs / ENISA innerhalb von **24 Stunden** nach Bekanntwerden einer aktiv ausgenutzten Schwachstelle.
  - *Vollständige Meldung:* Detaillierte Meldung innerhalb von **72 Stunden**.
- **Interne Behebungsfristen (HR-8):**
  - Kritische Schwachstellen: Bereitstellung eines Sicherheitsupdates innerhalb von **7 Tagen**.
  - Hohe Schwachstellen: Bereitstellung eines Sicherheitsupdates innerhalb von **30 Tagen**.
- **Support-Zeitraum (D-67, SPEC §17.2):** Für jede Hauptversion werden mindestens **5 Jahre** Sicherheitsupdates bereitgestellt.
- **Transparenz:** Jedes Release liefert eine maschinenlesbare Software Bill of Materials (SBOM) im CycloneDX-Format (`cargo cyclonedx`) und signierte Artefakte.
