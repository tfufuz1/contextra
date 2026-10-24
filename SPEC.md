# CONTEXTRA — Gesamtspezifikation (eigenständig, normativ)

Version: 1.0 · Sprache: Deutsch (Bezeichner, Code, Fehlercodes: Englisch) · Zielleser: Mensch **und** LLM-Implementierer (Google Jules, Claude Code o. Ä.)

---

## 0. Lesehilfe und Verbindlichkeit

0.1 Dieses Dokument ist die **einzige** Wahrheit über das Produkt. Es verweist auf kein anderes Dokument. Was hier nicht steht, ist nicht beauftragt.

0.2 Verbindlichkeitswörter (immer GROSS geschrieben, immer wörtlich zu nehmen):

| Wort | Bedeutung |
|---|---|
| **MUSS / MÜSSEN** | Pflicht. Verstoß = Aufgabe nicht erfüllt. |
| **DARF NICHT** | Verbot. Verstoß = Aufgabe nicht erfüllt, PR wird abgelehnt. |
| **SOLL** | Standard. Abweichung nur mit schriftlicher Begründung im PR. |
| **KANN** | Optional. |

0.3 **Bei Unklarheit anhalten.** Ist eine Anforderung mehrdeutig oder widersprüchlich, rät der Implementierer nicht. Er implementiert den unstrittigen Teil, legt im PR-Text einen Abschnitt `## FRAGE` mit der konkreten Frage und seinem Default-Vorschlag an und beendet die Aufgabe. Vorschläge für Lücken stehen in Abschnitt 17.

0.3a Dieses Dokument DARF NICHT durch den Implementierer geändert werden. Änderungen macht nur der Maintainer.

0.4 Alle Zahlenwerte (Größen, Schwellen, Parameter) sind **verbindliche Startwerte**, sofern nicht „Messziel“ dabeisteht. Messziele werden gemessen und dokumentiert; ihre Verfehlung löst eine Entscheidung aus (siehe Abschnitt 11), blockiert aber kein Release.

---

## 1. Produkt

### 1.1 Einzeiler

**Contextra ist eine lokale, verschlüsselte Wissensbasis für RAG-Anwendungen, in der „Vergessen“ technisch wirkt: Eine Person oder ein Objekt wird durch Vernichtung ihres Schlüssels unlesbar, und ein automatischer Test beweist das auf Dateiebene.**

### 1.2 Problem

Teams, die RAG-Systeme für Kunden im deutschsprachigen Raum bauen (Agenturen, Systemhäuser, Inhouse-IT), müssen auf Anfrage nachvollziehbar erklären, wie personenbezogene Daten aus Dokumenten, Chunks und Embeddings entfernt werden. Gängige Bausteine (Vektordatenbank plus Volltextindex) speichern Klartext und Embeddings an mehreren Stellen; „Löschen“ hinterlässt Kopien (WAL, Segmente, Snapshots, Backups). Contextra löst das durch **Verschlüsselung pro Subjekt** und **Schlüsselvernichtung**.

### 1.3 Zielkunde (Hypothese, wird validiert, siehe Abschnitt 15)

Primär: Entwickler und Dienstleister, die lokale oder On-Premise-RAG-Systeme (Air-Gap-fähig, ohne Cloud) in DACH ausliefern. Sekundär: Einzelentwickler, die einen lokalen, löschbaren Gedächtnisspeicher für KI-Agenten wollen.

### 1.4 Positionierung

- Contextra ist **kein** generischer Agenten-Speicher im Wettbewerb mit kostenlosen Open-Source-Lösungen. Der Unterschied ist **Löschbarkeit mit Prüfbarkeit** und **vollständiger Betrieb ohne Netzwerk** (außer zum lokalen Embedding-Dienst).
- Zugangswege: HTTP-API, MCP (stdio), Python-Client, CLI, Docker-Image. MCP ist **ein** Zugang von mehreren, nicht das Produkt.

### 1.5 Was Contextra NICHT verspricht (verbindlich auch für Marketing- und Doku-Texte)

Die Begriffe „rechtsgültig“, „DSGVO-konform“, „beweist“, „garantiert“ und „zertifiziert“ DÜRFEN NICHT in README, Doku, Fehlermeldungen oder Produkttexten stehen. Erlaubte Formulierung: *„Vergessen per Schlüsselvernichtung. Contextra liefert eine dokumentierte, testbare Löschmethode, die Sie in Ihr Löschkonzept aufnehmen können. Restrisiken: siehe RESIDUAL-RISKS.md.“*

---

## 2. Ziele, Nicht-Ziele, Parkplatz

### 2.1 Ziele nach Release

| Release | Inhalt | Ergebnis |
|---|---|---|
| **v0.1** | Verschlüsselt speichern, Hybridsuche, `forget`, HTTP + MCP + CLI, Canary-Test, grüne CI | Lauffähiges, ehrliches Minimalprodukt |
| **v0.2** | Python-Client, Docker-Image, Qualitätsmessung Deutsch, HNSW (nur falls Messziel verfehlt), Härtung | Auslieferbar an Pilotkunden |
| **v0.3** | Dokument-Versionen und Zeitreise (`as_of`) | Unterscheidungsmerkmal |
| **v0.4** | Minimale Web-Oberfläche, Audit-Report-Export | Verkaufbare Pakete |
| **danach** | nur nach Parkplatz-Regel (2.3) | |

### 2.2 Nicht-Ziele (DÜRFEN NICHT gebaut werden, auch nicht „vorbereitend“)

1. Löschkette, Purge-Pipeline, Fence, Witness, Anti-Resurrection-Ledger.
2. Signierte Löschnachweise, Zeitstempelanker (TSA), Transparenzlog, eIDAS-Funktionen.
3. Eigene Storage-Engine (LSM, WAL, Kompaktion). Persistenz erfolgt ausschließlich über `redb`.
4. Wissensgraph, Graph-RAG, Pfad-Diffusion.
5. Bandit-Routing, adaptive Parameter, Drift-Erkennung, „Kognition“, Gedächtnistypen, Decay.
6. KV-Cache, eigene Modell-Inferenz (Candle, ONNX), Cross-Encoder-Reranking.
7. WASM-Sandbox, Plugin-System, Agent-Workflows, Lizenz-Ring, AVV-Generator, Privacy-Gateway.
8. Mehrknoten-Betrieb, Replikation, Cloud-Anbindung, Telemetrie jeder Art.
9. Python-Bindings per FFI (PyO3). Der Python-Client spricht HTTP.
10. `unsafe`-Code und eigene SIMD-Kernel.

### 2.3 Parkplatz-Regel

Ein geparktes Thema darf erst begonnen werden, wenn **beide** Bedingungen gelten: (a) mindestens 3 voneinander unabhängige Pilotkunden oder Interviewpartner haben es schriftlich als Kaufgrund genannt, und (b) der Maintainer hat es in Abschnitt 2.1 eingetragen.

---

## 3. Begriffe

| Begriff | Definition |
|---|---|
| **Collection** | Isolationseinheit. Hat genau ein Embedding-Modell. Suchen laufen nie über Collections hinweg. |
| **Dokument** | Eingefügter Text mit Metadaten, gehört zu genau einer Collection und genau einem Subjekt. |
| **Chunk** | Teilstück eines Dokuments mit Embedding-Vektor. |
| **Subjekt** | Opaker String (max. 128 Bytes UTF-8), den der Aufrufer wählt, z. B. Kunden-ID. Löschung erfolgt pro Subjekt. Contextra empfiehlt, keine personenbezogenen Klartext-IDs zu verwenden. |
| **DEK** | Data Encryption Key: zufälliger 256-Bit-Schlüssel pro Subjekt. |
| **KEK** | Key Encryption Key: aus Passphrase oder Schlüsseldatei abgeleitet, verschlüsselt die DEKs im Keyring. |
| **Keyring** | Datei `keyring.json` mit den verschlüsselten DEKs. |
| **Forget** | Vernichtung des DEK eines Subjekts plus Entfernen aller seiner Daten aus Indizes und Speicher. |
| **Canary** | Zufälliger, eindeutiger Testtext, dessen Auftreten in Dateien geprüft wird. |
| **Walking Skeleton** | Dünnster durchgehender Ende-zu-Ende-Pfad (Einfügen → Suchen → Vergessen → Dateiprüfung) mit echten Komponenten. |

---

## 4. Harte Regeln (gelten für jede Aufgabe, jeden PR)

| ID | Regel |
|---|---|
| **R-01** | **Kein Platzhalter im Produktionscode.** Verboten: `todo!`, `unimplemented!`, Funktionen, die nur eine Konstante zurückgeben, leere Hooks, `AcceptAll*`/`Null*`/`Mock*`/`Fake*`-Typen außerhalb von Testcode. |
| **R-02** | **Keine Mocks im Produktionspfad.** Test-Doubles liegen ausschließlich in `contextra-testkit` oder unter `#[cfg(test)]`. Keine Produktions-Crate hat `contextra-testkit` als normale Abhängigkeit (nur `[dev-dependencies]`). CI prüft das mit `cargo tree`. |
| **R-03** | **Kein `unwrap`, `expect`, `panic!`, `unreachable!` im Produktionscode.** Workspace-Lints: `clippy::unwrap_used`, `clippy::expect_used`, `clippy::panic`, `clippy::indexing_slicing` = `deny`. In Tests erlaubt. |
| **R-04** | **Kein `unsafe`.** Jede Crate hat `#![forbid(unsafe_code)]`. |
| **R-05** | **Null Warnungen.** `RUSTFLAGS="-D warnings"`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo fmt --check`. |
| **R-06** | **Fail closed.** Fehlt der Schlüssel, ist der Keyring beschädigt oder stimmt eine Prüfsumme nicht, startet der Server nicht bzw. lehnt die Operation ab. Niemals Klartext-Fallback. |
| **R-07** | **Kein Klartext auf Platte.** Dokumenttext, Metadaten, Chunk-Text und Embedding-Vektoren werden vor jedem Schreiben verschlüsselt (Abschnitt 7). Das gilt auch für temporäre Dateien, Logs und Fehlermeldungen. |
| **R-08** | **Kein Inhalt in Logs.** Server-Logs DÜRFEN NICHT Dokumenttext, Suchanfragen, Metadatenwerte, Subjekt-IDs im Klartext oder Vektoren enthalten, auf keiner Log-Stufe. Erlaubt: Operationstyp, Collection-Name, Anzahl, Dauer, `subject_ref` (gekürzter HMAC, 8 Hex-Zeichen). |
| **R-09** | **Kein Netzwerk außer Embedding-Endpunkt.** Ausgehende Verbindungen gehen ausschließlich an die konfigurierte `embed_url`. Keine Telemetrie, keine Update-Checks, keine DNS-Auflösung anderer Hosts. |
| **R-10** | **Jede neue Abhängigkeit** steht vorab in `DEPENDENCIES.md` (Name, Version, Zweck, Lizenz) und besteht `cargo deny check`. Erlaubt sind nur MIT, Apache-2.0, BSD, ISC, Zlib, Unicode-DFS. |
| **R-11** | **Fertig heißt: End-to-End-Test grün** (Abschnitt 10.2), nicht „Code geschrieben“. Tests dürfen nicht gelöscht, übersprungen (`#[ignore]`) oder abgeschwächt werden, um grün zu werden. |
| **R-12** | **Determinismus.** Zeit und Zufall werden nur über die Traits `Clock` und `RandomSource` bezogen (Produktionsimplementierung: Systemuhr, OS-CSPRNG). Ausnahme: Schlüsselgenerierung nutzt direkt den OS-CSPRNG. |
| **R-13** | **Geheimnisse** (KEK, DEK, Passphrase) liegen in Typen mit `Zeroize + ZeroizeOnDrop`, haben kein `Debug`/`Display`/`Serialize`, und werden nie in Fehlertexte aufgenommen. |
| **R-14** | **Scope-Disziplin.** Eine Aufgabe ändert nur die in ihr genannten Dateien/Crates (Abschnitt 14). Keine „Verbesserungen“ nebenbei, keine Umbenennungen, keine neuen Crates. |

---

## 5. Architektur

### 5.1 Repository-Struktur (neues Repository `contextra`)

```
contextra/
├─ Cargo.toml                 # Workspace, Lints (R-03..R-05), resolver = "2"
├─ rust-toolchain.toml        # channel = "1.89.0", components = rustfmt, clippy
├─ deny.toml
├─ DEPENDENCIES.md
├─ SPEC.md                    # diese Datei (unveränderlich für Implementierer)
├─ RESIDUAL-RISKS.md          # Abschnitt 12, wird gepflegt
├─ README.md
├─ crates/
│  ├─ contextra-types/        # IDs, Fehler, Config-Typen, Traits Clock/RandomSource
│  ├─ contextra-crypto/       # KEK/DEK, AEAD, Keyring, subject_ref
│  ├─ contextra-store/        # redb-Persistenz mit verschlüsselten Records
│  ├─ contextra-text/         # Tokenizer, BM25 (im Speicher)
│  ├─ contextra-vector/       # Exakte Vektorsuche (v0.1), HNSW (v0.2, bedingt)
│  ├─ contextra-embed/        # Trait EmbeddingProvider + OllamaProvider
│  ├─ contextra-engine/       # Chunking, Ingest, Hybridsuche, Forget, Startup
│  ├─ contextra-server/       # Binary `contextra`: HTTP, MCP stdio, CLI
│  └─ contextra-testkit/      # nur dev: Canary-Scanner, Fake-Ollama, Crash-Harness
├─ clients/python/            # reiner HTTP-Client (v0.2)
├─ eval/                      # Qualitätsmessung (Abschnitt 10.5)
├─ docker/
└─ .github/workflows/ci.yml
```

### 5.2 Abhängigkeitsrichtung (strikt, keine Zyklen, keine Kante nach oben)

```
types ← crypto ← store
types ← text
types ← vector
types ← embed
(types, crypto, store, text, vector, embed) ← engine ← server
testkit ← (nur dev-dependencies von engine und server)
```

`contextra-engine` DARF NICHT von `contextra-server` abhängen. `contextra-store` DARF NICHT von `text`, `vector`, `embed` abhängen.

### 5.3 Datenverzeichnis

```
<data_dir>/
├─ contextra.redb      # alle Daten (verschlüsselt, außer Metadaten laut 6.2)
├─ keyring.json        # verschlüsselte DEKs + KDF-Parameter
├─ contextra.lock      # exklusive Dateisperre, verhindert zweite Instanz
└─ tmp/                # leer im Normalbetrieb; nie Klartext
```

### 5.4 Konfiguration

Konfigurationsquellen in dieser Rangfolge: CLI-Flag > Umgebungsvariable > `config.toml` im Datenverzeichnis > Default.

| Schlüssel | Env | Default |
|---|---|---|
| `data_dir` | `CONTEXTRA_DATA_DIR` | `./contextra-data` |
| `bind` | `CONTEXTRA_BIND` | `127.0.0.1:7878` |
| `token_file` | `CONTEXTRA_TOKEN_FILE` | nicht gesetzt |
| `key_file` | `CONTEXTRA_KEY_FILE` | nicht gesetzt |
| `passphrase_file` | `CONTEXTRA_PASSPHRASE_FILE` | nicht gesetzt |
| `embed_url` | `CONTEXTRA_EMBED_URL` | `http://127.0.0.1:11434` |
| `embed_model` | `CONTEXTRA_EMBED_MODEL` | `bge-m3` |
| `top_k_default` | – | `10` |

Regeln: Ein Schlüssel kommt aus genau einer der Quellen `key_file` (32 rohe Zufallsbytes), `passphrase_file` (Textdatei) oder interaktiver Abfrage im CLI. Ist keine verfügbar oder sind mehrere gesetzt, bricht der Start mit `E_KEY_SOURCE` ab. Passphrase **direkt** in einer Umgebungsvariable ist verboten. Bindet `bind` nicht an Loopback (`127.0.0.1`/`::1`), ist `token_file` Pflicht, sonst Abbruch mit `E_CONFIG`.

---

## 6. Datenmodell und Formate

### 6.1 Typen

- `CollectionName`: `[a-z0-9_-]{1,64}`.
- `DocId`, `ChunkId`: UUID v4 als String (kleingeschrieben, mit Bindestrichen).
- `SubjectId`: String, 1–128 Bytes UTF-8, keine Steuerzeichen.
- `subject_ref`: `HMAC-SHA256(hmac_key, subject_id_bytes)`, 32 Bytes; in Logs nur die ersten 8 Hex-Zeichen.
- Zeitstempel: Unix-Millisekunden `u64` (aus `Clock`).

### 6.2 redb-Tabellen

| Tabelle | Schlüssel | Wert | Verschlüsselt |
|---|---|---|---|
| `collections` | `name` | JSON `{embed_model, dim, created_at}` | nein (enthält keine Personendaten) |
| `docs` | `doc_id` | `{collection, subject_ref, created_at, ct}` | `ct` = verschlüsselter Dokumentkörper (Text + Metadaten-JSON) |
| `chunks` | `chunk_id` | `{doc_id, ordinal, ct}` | `ct` = verschlüsselter Chunk (6.3) |
| `subject_docs` | `subject_ref ‖ doc_id` | leer | nein (Schlüssel besteht nur aus HMAC und Zufalls-UUID) |
| `forget_intent` | `subject_ref` | `{started_at, collection}` | nein |
| `forget_log` | laufende Nummer `u64` | JSON `{subject_ref_short, collection, at, docs, chunks, key_destroyed}` | nein |
| `meta` | Text | Schemaversion, `verifier` | nein |

Unverschlüsselt bleiben damit bewusst: Collection-Namen, `doc_id`/`chunk_id`, `subject_ref` (HMAC), Zeitstempel, Anzahl und Größe der Records. Das steht in RESIDUAL-RISKS.md.

### 6.3 Plaintext-Formate (vor der Verschlüsselung)

**Dokumentkörper:** `u32 LE len_text ‖ text (UTF-8) ‖ u32 LE len_meta ‖ meta (UTF-8 JSON-Objekt, max. 16 KiB)`.

**Chunk:** `u32 LE len_text ‖ text (UTF-8) ‖ u32 LE dim ‖ dim × f32 LE`.

### 6.4 Größenlimits

Dokument max. 5 MiB Text; Chunk-Länge siehe 8.1; max. 16 KiB Metadaten; `top_k` max. 100; Anfragetext max. 8 KiB. Überschreitung ergibt `E_LIMIT`.

---

## 7. Kryptografie und Schlüssel

7.1 **Bausteine** (ausschließlich diese Crates, keine Eigenbauten): `aes-gcm-siv` (AEAD), `argon2` (Passphrase-KDF), `hkdf` und `sha2` (Ableitungen), `hmac`, `zeroize`, `getrandom` (OS-CSPRNG), `subtle` (Konstantzeit-Vergleich).

7.2 **KEK:**
- Aus Passphrase: `Argon2id`, `m = 65536 KiB`, `t = 3`, `p = 1`, Salt 16 Byte zufällig, Ausgabe 32 Byte.
- Aus Schlüsseldatei: `HKDF-SHA256(ikm = Dateiinhalt (exakt 32 Byte), salt = leer, info = "contextra-kek-v1")`, Ausgabe 32 Byte.

7.3 **Abgeleitete Schlüssel:** `hmac_key = HKDF(KEK, info = "contextra-subject-ref-v1")`.

7.4 **DEK:** 32 Byte aus OS-CSPRNG, ein DEK pro Subjekt **und Collection**. Wird beim ersten Dokument des Subjekts angelegt und im Keyring abgelegt: `AEAD(KEK, nonce, DEK, aad = "keyring" ‖ key_id)`.

7.5 **Record-Verschlüsselung:** `AES-256-GCM-SIV`, Nonce 12 Byte zufällig pro Record, gespeichert als Präfix (`nonce ‖ ciphertext ‖ tag`). **AAD** = `tabellenname ‖ 0x00 ‖ record_key`. Dadurch ist ein Ciphertext an seinen Ort gebunden.

7.6 **Keyring-Format** (`keyring.json`, UTF-8 JSON):

```json
{
  "version": 1,
  "kdf": {"alg": "argon2id", "m_kib": 65536, "t": 3, "p": 1, "salt_b64": "..."},
  "verifier_b64": "<AEAD(KEK, \"contextra-verifier\")>",
  "keys": [
    {"key_id": "<subject_ref_hex>:<collection>", "wrapped_b64": "...", "created_at": 0}
  ]
}
```

Bei Schlüsseldatei-Betrieb steht `"kdf": {"alg": "keyfile"}`. Beim Start wird der `verifier` entschlüsselt; Fehlschlag ergibt `E_KEY_WRONG` und Abbruch.

7.7 **Atomares Schreiben des Keyrings:** in `tmp/keyring.json.new` schreiben, `fsync` der Datei, `rename` über `keyring.json`, `fsync` des Verzeichnisses. Datei-Rechte 0600.

7.8 **Schlüsselvernichtung:** DEK im Speicher `zeroize`n, Eintrag aus `keys` entfernen, Keyring atomar neu schreiben (7.7). Danach existiert kein Weg zurück zum Klartext des Subjekts, außer über Kopien des **alten** Keyrings (siehe 12).

7.9 **Schlüsselrotation des KEK** (`contextra rekey`): v0.2. Bis dahin nicht vorhanden.

---

## 8. Abläufe

### 8.1 Chunking (deterministisch)

Eingabe: Text. Zielgröße **1600 Zeichen** (Unicode-Skalarwerte), Überlappung **200 Zeichen**, Mindestgröße letzter Chunk 200 Zeichen (sonst an den vorherigen anhängen).

Algorithmus: (1) Text mit `\r\n` → `\n` normalisieren. (2) In Absätze an `\n\n` teilen. (3) Absätze zu Chunks bis zur Zielgröße zusammenfügen; ein Absatz, der allein die Zielgröße überschreitet, wird an Satzgrenzen (`. `, `! `, `? `, `\n`) geteilt, hilfsweise hart bei Zielgröße. (4) Überlappung: jeder Chunk (außer dem ersten) beginnt mit den letzten 200 Zeichen des Vorgängers, ausgerichtet auf die nächste Wortgrenze. Nie in der Mitte eines Unicode-Graphem-Clusters schneiden (`unicode-segmentation`). Gleiche Eingabe ergibt immer gleiche Chunks. Leerer Text ergibt `E_INVALID`.

### 8.2 Ingest

1. Eingabe validieren (Limits 6.4, Collection existiert).
2. Subjekt-DEK holen oder anlegen (7.4); Keyring-Änderung atomar schreiben **vor** dem Datenschreiben.
3. Chunking (8.1).
4. Embeddings über `EmbeddingProvider` holen (außerhalb jeder Sperre, Batches à 16). Schlägt der Provider fehl, bricht der Ingest mit `E_EMBED` ab und schreibt **nichts**.
5. Eine einzige redb-Schreibtransaktion: `docs`, `chunks`, `subject_docs` schreiben (alle Werte verschlüsselt laut 6.3/7.5).
6. Nach erfolgreichem Commit: In-Memory-Indizes (BM25, Vektor) aktualisieren.
7. Antwort `{doc_id, chunks}`.

Absturz zwischen 5 und 6 ist unkritisch: Beim Start werden die Indizes aus der Datenbank neu aufgebaut (8.5).

### 8.3 Suche (Hybrid)

1. Anfragetext einbetten (`EmbeddingProvider`).
2. BM25-Top-50 und Vektor-Top-50 (Kosinus-Ähnlichkeit) holen, beschränkt auf Collection (und optional `subject_id`).
3. **RRF-Fusion:** `score(chunk) = Σ über beide Listen 1 / (60 + rang)` (Rang ab 1). Chunk, der nur in einer Liste steht, bekommt nur diesen Summanden.
4. Absteigend nach `score` sortieren; Gleichstand: kleinere `chunk_id` zuerst (deterministisch). Top-`top_k` zurückgeben.
5. Chunk-Text aus dem Speicher-Index (Klartext nur im RAM). Ist der Schlüssel des Subjekts nicht mehr vorhanden, wird der Chunk nie geliefert.

**BM25:** `k1 = 1.2`, `b = 0.75`. **Tokenisierung:** Unicode NFKC, kleinschreiben, Trennung an Unicode-Wortgrenzen (`unicode-segmentation`), Tokens nur aus Buchstaben/Ziffern, **Snowball-Stemming** (`rust-stemmers`) für Deutsch und Englisch: Sprache je Collection konfigurierbar (`de` Default, `en`); Umlaute und ß werden vom Stemmer behandelt, zusätzlich wird `ß → ss` vorab gefaltet. Keine Stopwortliste in v0.1.

**Vektorsuche v0.1:** exakte Suche (Brute Force) über alle Chunk-Vektoren der Collection, Kosinus über vorab normalisierte `f32`-Vektoren (Skalarprodukt). HNSW erst v0.2 und nur gemäß Abschnitt 11.

### 8.4 Forget (idempotent, absturzsicher)

Eingabe: `collection` (optional: alle Collections) und `subject_id`.

1. `subject_ref` berechnen. redb-Transaktion: `forget_intent` schreiben (commit).
2. Alle Chunks/Dokumente des Subjekts aus den In-Memory-Indizes entfernen; Klartext-Puffer `zeroize`n.
3. **Schlüsselvernichtung** (7.8) für alle DEKs des Subjekts (in der betroffenen Collection bzw. allen).
4. redb-Transaktion: Records in `docs`, `chunks`, `subject_docs` des Subjekts löschen, `forget_intent` löschen, Eintrag in `forget_log` anhängen (commit).
5. Antwort `ForgetReceipt` (kein kryptografischer Beweis, nur Protokoll):

```json
{"subject_ref_short":"a1b2c3d4","collection":"x","at":1730000000000,
 "documents":3,"chunks":41,"key_destroyed":true,
 "residual_risks":["backups","swap","ssd_remanence","metadata","keyring_copies"]}
```

**Fortsetzung beim Start:** Existiert `forget_intent`, führt der Server Schritte 2–4 idempotent aus, bevor er Anfragen annimmt. Records, deren DEK fehlt, werden beim Start entfernt und nie ausgeliefert.

**Wiederverwendung:** Nach `forget` darf dieselbe `subject_id` neu verwendet werden; sie erhält einen **neuen** DEK.

### 8.5 Startup

1. Schlossdatei sperren (`E_LOCKED` falls belegt).
2. Schlüsselquelle laden (5.4), KEK ableiten, Keyring öffnen, `verifier` prüfen (`E_KEY_WRONG`).
3. redb öffnen, Schemaversion prüfen (`E_SCHEMA`).
4. Offene `forget_intent` abarbeiten (8.4).
5. Alle `chunks` lesen, entschlüsseln, Indizes im Speicher aufbauen; Records ohne DEK überspringen und löschen.
6. Erst danach den HTTP-Port öffnen / MCP-stdio bedienen. `/v1/health` meldet `ready: false` bis dahin.

**Persistente Index-Snapshots sind in v0.x verboten.** Ein Snapshot würde Klartextableitungen (Terme, Vektoren) außerhalb des Subjekt-Schlüssels ablegen. Eine spätere Optimierung braucht einen schriftlichen Entwurf des Maintainers (Snapshots pro Subjekt-Schlüssel).

### 8.6 Nebenläufigkeit

Ein Prozess pro Datenverzeichnis (Dateisperre). Schreibzugriffe (Ingest-Commit, Forget) laufen seriell unter einem `Mutex`. Suchen laufen parallel unter einem `RwLock` über den Indizes. Netzwerk-Anfragen an den Embedder laufen **ohne** gehaltene Sperre.

---

## 9. Schnittstellen

### 9.1 HTTP-API (JSON, UTF-8, Basis `/v1`)

Auth: Ist `token_file` gesetzt, ist `Authorization: Bearer <token>` Pflicht (Konstantzeit-Vergleich), sonst `401 E_AUTH`. Max. Request-Größe 6 MiB. Fehlerformat: `{"error":{"code":"E_…","message":"…"}}` (message ohne Inhalte, R-08).

| Methode, Pfad | Body | Antwort |
|---|---|---|
| `GET /v1/health` | – | `{"ready":true,"version":"0.1.0"}` |
| `POST /v1/collections` | `{"name","language":"de"\|"en"}` | `201 {"name","embed_model","dim","language"}` (`dim` wird durch Probe-Embedding ermittelt) |
| `GET /v1/collections` | – | `[{"name","documents","chunks"}]` |
| `POST /v1/documents` | `{"collection","subject_id","text","metadata":{}}` | `201 {"doc_id","chunks"}` |
| `GET /v1/documents/{doc_id}` | – | `{"doc_id","collection","subject_id","text","metadata","created_at"}` |
| `POST /v1/search` | `{"collection","query","top_k":10,"subject_id":null}` | `{"results":[{"doc_id","chunk_id","score","text","metadata","subject_id"}]}` |
| `POST /v1/forget` | `{"subject_id","collection":null}` | `200 ForgetReceipt` |
| `DELETE /v1/collections/{name}` | – | `204` (vernichtet alle DEKs der Collection, ablaufgleich zu 8.4) |

HTTP-Statuszuordnung: `E_INVALID`/`E_LIMIT` → 400/413, `E_AUTH` → 401, `E_NOT_FOUND` → 404, `E_EMBED` → 502, `E_LOCKED`/`E_KEY_*` nur beim Start, `E_INTERNAL` → 500.

### 9.2 MCP (stdio, JSON-RPC 2.0)

Implementierung mit dem offiziellen Rust-SDK `rmcp`. Ist die Nutzung technisch unmöglich, siehe 0.3. Der Prozess startet mit `contextra mcp` und bedient genau diese Tools (Namen verbindlich):

| Tool | Parameter | Wirkung |
|---|---|---|
| `memory_store` | `collection`, `subject_id`, `text`, `metadata?` | wie `POST /documents` |
| `memory_search` | `collection`, `query`, `top_k?`, `subject_id?` | wie `POST /search` |
| `memory_forget` | `subject_id`, `collection?` | wie `POST /forget` |
| `memory_collections` | – | wie `GET /collections` |

Alle Tools nutzen dieselbe `Engine`-Funktion wie HTTP; es gibt keine zweite Implementierung. Auf stdout steht ausschließlich Protokollverkehr; Logs gehen nach stderr.

### 9.3 CLI (Binary `contextra`)

`contextra serve` (HTTP), `contextra mcp` (stdio), `contextra init` (Datenverzeichnis, Schlüssel erzeugen), `contextra forget --subject <id> [--collection <c>]`, `contextra verify` (führt den Canary-Selbsttest aus 10.2 gegen ein temporäres Verzeichnis aus), `contextra version`. Exitcodes: 0 ok, 1 Fehler, 2 falsche Aufrufparameter.

### 9.4 Python-Client (v0.2)

Paket `contextra-client`, reine `httpx`-Abhängigkeit, Klassen `Client`, Methoden `create_collection`, `store`, `search`, `forget`. Typisiert (PEP 484), getestet gegen den echten Server.

### 9.5 Embedding-Provider

Trait `EmbeddingProvider { fn model(&self)->&str; async fn embed(&self, texts:&[String]) -> Result<Vec<Vec<f32>>, EmbedError>; }`. Einzige Produktionsimplementierung in v0.x: `OllamaProvider` (HTTP `POST {embed_url}/api/embed`, Body `{"model":…,"input":[…]}`, Antwort `{"embeddings":[[…]]}`; die genaue Schnittstelle ist gegen die Ollama-Dokumentation zu prüfen und durch einen Integrationstest abzusichern). Timeouts: Connect 3 s, Gesamt 60 s. Ein Wiederholungsversuch bei Verbindungsfehler. Ändert sich Modell oder Dimension gegenüber der Collection, antwortet der Server mit `E_EMBED_MISMATCH`.

---

## 10. Qualitätssicherung

### 10.1 Testarten

Unit-Tests je Crate; Property-Tests (`proptest`) für Chunker (Abdeckung des gesamten Textes, keine Panics bei beliebigem Unicode, Determinismus) und Krypto-Roundtrips; Integrationstests mit `redb` auf temporären Verzeichnissen; End-to-End-Tests gegen das **echte Release-Binary** (10.2); Absturztests (10.3).

### 10.2 End-to-End-Abnahmetest (verbindliche Definition von „fertig“)

Testprogramm in `contextra-testkit`, startet das **Release-Binary als Kindprozess** und spricht HTTP (und separat MCP über stdio). Ablauf:

1. Temporäres Datenverzeichnis, Schlüsseldatei erzeugen, Embedding-Endpunkt bereitstellen (10.4).
2. Collection anlegen. Zwei Subjekte A und B. Für A ein Dokument mit Canary `CANARY-<32 zufällige Hex-Zeichen>` im Text und im Metadatenwert; für B ein Dokument mit anderem Canary.
3. **Vorher-Prüfung (R-07):** Der Canary von A und B darf in **keiner** Datei des Datenverzeichnisses vorkommen (Scanner 10.2a).
4. Suche nach dem Canary-Text liefert Dokument A auf Rang 1; Suche mit Filter `subject_id=A` liefert A, mit Filter B liefert A nicht.
5. `forget(A)`.
6. Suche liefert nichts mehr von A; `GET /documents/{A}` ergibt `404`. B ist unverändert auffindbar.
7. **Nachher-Prüfung:** Scanner über alle Dateien im Datenverzeichnis, über Prozess-stdout und -stderr (alles, was der Server geschrieben hat), und über `/tmp`-Dateien dieses Tests.
8. Neustart des Servers; Schritte 6 und 7 wiederholen.

**10.2a Scanner:** Sucht bytegenau (Dateien als Binärdaten) nach: (a) Canary als UTF-8, (b) Canary als UTF-16LE, (c) Canary als Base64 in drei Offsets, (d) den ersten 16 `f32`-Werten des Embeddings als Little-Endian-Bytefolge, (e) dem Metadaten-Canary. Jeder Treffer = Test rot, mit Dateiname und Offset in der Ausgabe.

### 10.3 Absturztests

Harness startet den Server und beendet ihn mit `SIGKILL` an definierten Punkten: (i) während des Ingest (5 zufällige Zeitpunkte), (ii) zwischen Schritt 1 und 3 des Forget, (iii) zwischen Schritt 3 und 4 des Forget (per Test-Feature `crash-points`, das **nur** in `contextra-testkit`-Builds aktiv ist und im Release-Binary fehlt). Nach Neustart gelten: kein Canary in Dateien, `forget_intent` abgearbeitet, Index konsistent, B unverändert. Je Szenario 20 Wiederholungen.

### 10.4 Embedding-Endpunkt in Tests

- **CI-Standard (`e2e-fast`):** ein kleiner, deterministischer HTTP-Dienst `fake-ollama` in `contextra-testkit` (implementiert dieselbe `/api/embed`-Schnittstelle; Vektor = gehashte Token-Statistik, normalisiert, Dimension 64). Er ist ausschließlich Testwerkzeug (R-02 bleibt gewahrt, da das Produktionsbinary ihn nicht enthält).
- **Nightly (`e2e-real`):** gegen einen echten Ollama-Prozess mit einem kleinen Embedding-Modell. Dieser Job MUSS vor jedem Release grün sein.

### 10.5 Qualitätsmessung Retrieval (v0.2)

`eval/de_qa_v1.jsonl`: mindestens 50 von Menschen geprüfte deutsche Frage-Dokument-Paare (Maintainer gibt frei). `contextra-testkit` misst Recall@5 und MRR für (a) nur BM25, (b) nur Vektor, (c) Hybrid. Ergebnis wird in `eval/RESULTS.md` mit Commit-Hash protokolliert. **Messziel:** Hybrid Recall@5 ≥ 0,80. Wird es verfehlt, entsteht eine Analyse, kein Blocker.

### 10.6 CI (`.github/workflows/ci.yml`)

Jobs: `fmt`, `clippy` (R-05), `test`, `deny` (R-10), `build-release`, `forbid-testkit` (R-02), `no-placeholders` (greift auf `todo!`, `unimplemented!`, `AcceptAll`, `Mock`, `Fake` im Nicht-Testcode zu, R-01), `e2e-fast`, `crash`. Branch-Schutz: Merge nur bei grünen Jobs, keine Force-Pushes auf `main`. Zeitlich getrennt: `e2e-real` (nightly).

---

## 11. Leistungs-Messziele (messen, dokumentieren, dann entscheiden)

Referenzlast: 100.000 Chunks, 1024 Dimensionen, 8 Kerne, 16 GiB RAM, SSD.

| Messgröße | Ziel |
|---|---|
| Suche (ohne Embedding), p95 | ≤ 150 ms |
| Startzeit (Indexaufbau) | ≤ 30 s |
| Speicherbedarf Prozess | ≤ 2 GiB |
| Ingest-Durchsatz | begrenzt durch Embedder; Server-Overhead ≤ 20 % der Zeit |

**Entscheidungsregel HNSW:** Verfehlt die exakte Suche p95 ≤ 150 ms bei 100.000 Chunks, wird in v0.2 HNSW eingeführt (Crate-Wahl im PR begründet, `DEPENDENCIES.md`), mit Recall@10 ≥ 0,95 gegenüber der exakten Suche als Abnahmekriterium. Andernfalls bleibt die exakte Suche. Benchmarks liegen unter `crates/contextra-vector/benches` und laufen nur manuell.

---

## 12. Rechtliche Ehrlichkeit und Restrisiken

12.1 **Einordnung (keine Rechtsberatung):** Die DSGVO schreibt keine Löschmethode vor; maßgeblich ist, dass Informationen nicht mehr wiederherstellbar sind, und der Verantwortliche muss sein Vorgehen nachweisen können (Rechenschaftspflicht). Kryptografisches Löschen (Schlüsselvernichtung) gilt als anerkanntes Verfahren, wenn es im Löschkonzept des Verantwortlichen bewusst als Methode festgelegt und dokumentiert ist. Ein selbst erzeugtes `ForgetReceipt` ist ein Protokolleintrag, **kein** gesetzlich definierter Nachweis. Contextra liefert eine Methode und einen Test, keine Rechtsgarantie.

12.2 **`RESIDUAL-RISKS.md`** (Pflichtdatei, wird mit dem Produkt ausgeliefert) enthält mindestens:

1. **Backups:** Kopien von `keyring.json` aus der Zeit vor `forget` machen Daten zusammen mit alten `contextra.redb`-Kopien wieder lesbar. Empfehlung: Keyring nicht im selben Backup wie die Datenbank speichern.
2. **Arbeitsspeicher, Swap, Core-Dumps, Ruhezustand:** Klartext existiert zur Laufzeit im RAM. Empfehlung: verschlüsselter Swap, Core-Dumps deaktivieren.
3. **SSD-Remanenz:** Überschriebene Keyring-Blöcke können physisch noch vorhanden sein (Wear-Leveling). Empfehlung: Festplattenverschlüsselung.
4. **Metadaten:** Collection-Namen, IDs, Zeitstempel, Anzahl und Größe der Records bleiben sichtbar.
5. **Kopien außerhalb von Contextra:** Was die Anwendung zuvor an Dritte, Logs oder LLMs weitergegeben hat, löscht Contextra nicht.
6. **Gruppierung:** Die Löschung greift nur so fein wie die gewählte `subject_id`.
7. **Embeddings:** Werden mit dem Subjektschlüssel verschlüsselt und mit vernichtet; Embeddings, die die Anwendung selbst außerhalb speichert, nicht.

12.3 Der Maintainer lässt Abschnitt 12 und die Produkttexte vor einer kommerziellen Aussage zur Löschwirkung von einer datenschutzrechtlich qualifizierten Person prüfen. Bis dahin gilt 1.5.

---

## 13. Umgang mit dem bisherigen Code (Legacy)

13.1 Der bisherige Code (~108.000 Zeilen Produktionscode, ~179.000 Zeilen Test in vielen Crates) wird als Tag `archive/legacy-v1` eingefroren. Er **kompiliert nicht** (Store) und enthält Platzhalter. Er gilt als **nicht vorhanden**, bis einzelne Dateien die Tore aus Abschnitt 10 bestehen.

13.2 Übernahme nur **Datei für Datei** per eigenem PR: Quelle benennen, an die Regeln R-01..R-14 anpassen (insb. `unwrap`/`unsafe` entfernen), Tests mitbringen, in eine der Crates aus 5.1 einsortieren. Kandidaten (Eignung ungeprüft, Prüfung im Rahmen von WP-02..WP-06):

| Neue Crate | Mögliche Quelle im Legacy | Voraussetzung |
|---|---|---|
| `contextra-crypto` | Krypto-Crate (AEAD, Keyring, Argon2, HKDF) | bestehende Bausteine nur übernehmen, wenn sie 7.x exakt erfüllen; sonst neu |
| `contextra-text` | Text-Crate (BM25, Tokenizer) | muss ohne Async und ohne Store-Abhängigkeit laufen |
| `contextra-vector` | Vektor-Crate | nur Distanzfunktionen/Normalisierung; kein `unsafe`/SIMD |
| `contextra-store` | `redb`-Adapter | muss verschlüsselte Records gemäß 6.2 schreiben |
| Hybrid/RRF | Engine-Crate | nur die Fusionslogik, nicht die Engine |

13.3 Nicht übernommen werden: Store-Engine, Purge/Fence/Ledger, Graph, Adapt, Router, KV-Cache, Candle, Privacy, Sandbox, Lizenz, AVV, Audit-Export, Checkpoint, `xtask`-Gates.

13.4 Ist eine Übernahme aufwendiger als die Neuimplementierung nach dieser Spezifikation, ist neu zu schreiben (Entscheidung im PR begründen).

---

## 14. Arbeitspakete (Reihenfolge und Parallelität)

Jedes Arbeitspaket (WP) ist **ein PR**. Jeder PR enthält Code, Tests und aktualisierte `DEPENDENCIES.md`. DoD: Abschnitt 16.2. „Dateien“ = einzig erlaubter Änderungsumfang (R-14).

### Welle 0 (sequenziell, zuerst)

**WP-00 Repository-Gerüst.** Dateien: Wurzel (`Cargo.toml`, `rust-toolchain.toml`, `deny.toml`, `DEPENDENCIES.md`, `RESIDUAL-RISKS.md`, `README.md`, `SPEC.md`), `.github/workflows/ci.yml`, leere Crates laut 5.1 mit `#![forbid(unsafe_code)]` und Workspace-Lints. Akzeptanz: `cargo build/clippy/fmt/test --workspace` grün; alle CI-Jobs aus 10.6 laufen (auch die Prüfjobs R-01/R-02 greifen und sind beweisbar rot bei absichtlichem Verstoß).

**WP-01 Typen.** Crate `contextra-types`: Newtypes aus 6.1, Fehlerenum mit den Codes aus Anhang A, `Clock`, `RandomSource`, Config-Strukturen (5.4) samt Laden/Validieren. Akzeptanz: Property-Tests für Validierung; alle Konfigurationsregeln aus 5.4 testbar.

### Welle 1 (parallel, disjunkte Dateien)

**WP-02 Krypto.** Crate `contextra-crypto`: KEK-Ableitung (7.2), `subject_ref`, AEAD-Record-Verschlüsselung mit AAD (7.5), Keyring lesen/schreiben/atomar (7.6, 7.7), DEK-Vernichtung (7.8). Akzeptanz: Roundtrip- und Manipulationstests (veränderter Ciphertext, falsche AAD, falscher Schlüssel → Fehler, nie Panic); Keyring-Absturztest (Prozess-Kill während Schreiben → alte oder neue Datei intakt, nie halbe); Geheimnis-Typen ohne `Debug` (compile-fail-Test).

**WP-03 Text.** Crate `contextra-text`: Tokenizer, Stemming, BM25-Index im Speicher mit `add`, `remove(chunk_id)`, `search(query, k, filter)`. Akzeptanz: deterministische Ergebnisse, deutsche Beispiele (Umlaute, ß, Komposita-Stemming), Property-Test: nach `remove` taucht der Chunk nie mehr auf.

**WP-04 Vektor.** Crate `contextra-vector`: Normalisierung, exakte Kosinus-Suche mit `add`, `remove`, `search(query, k, filter)`. Akzeptanz: Vergleich gegen naive Referenz auf Zufallsdaten; Benchmark-Skelett (manuell).

**WP-05 Embed.** Crate `contextra-embed`: Trait und `OllamaProvider` (9.5). Akzeptanz: Integrationstest gegen `fake-ollama` aus WP-08; Fehlerfälle (Timeout, Status ≠ 200, falsche Dimension) ergeben definierte Fehler.

**WP-08 Testkit-Grundlagen.** Crate `contextra-testkit`: Scanner (10.2a), `fake-ollama`, Prozess-Harness (Binary starten, warten bis `ready`, beenden). Akzeptanz: Scanner-Selbsttests (findet absichtlich eingeschmuggelte Canaries in allen fünf Kodierungen; findet nichts in sauberen Dateien).

### Welle 2

**WP-06 Store.** Crate `contextra-store`: redb-Tabellen (6.2), Transaktionsfunktionen für Ingest-Commit, Forget-Schritte 1 und 4, Scan aller Chunks beim Start, `forget_log`. Verschlüsselung über `contextra-crypto`. Akzeptanz: Datei-Scan nach Schreiben zeigt keinen Klartext (Scanner aus WP-08); Transaktions-Atomarität getestet.

### Welle 3

**WP-07 Engine.** Crate `contextra-engine`: Chunker (8.1), Ingest (8.2), Hybridsuche (8.3), Forget (8.4), Startup (8.5), Nebenläufigkeit (8.6). Akzeptanz: Integrationstests mit echtem Store, echter Krypto, `fake-ollama`; Chunker-Property-Tests; Forget-Wiederaufnahme nach simuliertem Abbruch.

### Welle 4

**WP-09 Server.** Crate `contextra-server`: HTTP (9.1) mit `axum`, MCP (9.2), CLI (9.3), Logging nach R-08. Akzeptanz: alle Endpunkte und Fehlerfälle mit Statuscodes getestet; Log-Inhaltstest (Canary darf in stderr nie auftauchen).

### Welle 5 (Release v0.1)

**WP-10 End-to-End und Absturztests.** Abschnitt 10.2 und 10.3 vollständig automatisiert in CI (`e2e-fast`, `crash`). **v0.1 ist erreicht, wenn WP-00..WP-10 gemergt sind, alle CI-Jobs grün sind und `e2e-real` einmal grün lief.** Danach: Tag `v0.1.0`.

### Welle 6 (v0.2)

WP-11 Python-Client (9.4) · WP-12 Dockerfile (Multi-Stage, nicht-root, schreibgeschütztes Root-FS möglich, Healthcheck) · WP-13 Qualitätsmessung (10.5) und Benchmarks (11) · WP-14 HNSW (bedingt, 11) · WP-15 `contextra rekey` (7.9) · WP-16 Härtung: `setrlimit(RLIMIT_CORE, 0)` über eine sichere Crate-API oder dokumentierte Container-Option (ohne `unsafe`, sonst Doku-Hinweis statt Code).

### Welle 7 (v0.3, Zeitreise)

WP-17 **Versionen:** Ein erneutes `POST /documents` mit gleicher `doc_id` erzeugt eine neue Version (Version-Zähler, alle Versionen unter demselben Subjekt-DEK verschlüsselt, Forget vernichtet alle). `POST /search` erhält `as_of` (Unix-ms); es wird die zum Zeitpunkt gültige Version gefunden. WP-18 `GET /documents/{id}/versions` und Diff. Akzeptanz: Canary-Test deckt alle Versionen ab; `as_of` vor dem Löschen liefert nach `forget` nichts.

### Welle 8 (v0.4)

WP-19 Minimale Web-Oberfläche (statische Seiten, vom Server ausgeliefert, kein externes CDN, kein Build-Zwang außer optionalem lokalem Bundling) mit Collections, Suche, Forget (mit Bestätigung), Forget-Log. WP-20 Audit-Report-Export: `GET /v1/forget-log` als JSON/CSV plus Textbaustein „Löschmethode“ für das Löschkonzept des Kunden (ohne Rechtsbehauptung, 1.5).

### Parallelisierungsvorgabe

In Welle 1 SOLLEN höchstens 5 Aufgaben gleichzeitig laufen, je eine pro WP. Vor jedem Start einer Welle prüft der Maintainer mit einem Konflikt-Skript, dass die Dateimengen der gleichzeitigen WPs disjunkt sind. Es werden nie zwei WPs parallel gestartet, die dieselbe Datei berühren.

---

## 15. Markt, Validierung, Geschäftsmodell

15.1 **Ausgangslage:** Der Markt für generischen KI-Agenten-Speicher ist von kapitalstarken, größtenteils kostenlosen Open-Source-Angeboten besetzt. Ein Einzelentwickler kann dort nicht über „Memory per MCP“ konkurrieren. Contextra konkurriert über **Löschbarkeit, Prüfbarkeit, Lokalität**. Ob dafür Zahlungsbereitschaft besteht, ist **unbelegt** und wird validiert.

15.2 **Validierungsplan (parallel zu Welle 0–2, vor jedem Feature jenseits v0.1):**

- 15 Gespräche mit Agenturen, Systemhäusern, Inhouse-KI-Teams, Datenschutzbeauftragten in DACH.
- Pflichtfragen: (1) Wie löschen Sie heute Personendaten aus Ihrem RAG (Chunks, Embeddings, Backups)? (2) Wer fragt danach, wie oft? (3) Was kostet Sie das heute (Zeit, Risiko)? (4) Würden Sie einen Pilot bezahlen? Wie viel?
- Ergebnisse werden in `docs/interviews.md` protokolliert (ohne personenbezogene Daten der Befragten).

15.3 **Kill- und Entscheidungsregeln:** Nennen weniger als 3 von 15 Befragten die nachweisbare Löschung als echten, wiederkehrenden Schmerz → Produkt-These wird verworfen oder auf ein vertikales Produkt (z. B. lokale „Aktenwissen“-Box für eine Berufsgruppe) gewendet; die Engine bleibt, der Zuschnitt ändert sich. Zahlt kein Pilotkunde bis Ende von v0.2, wird der Aufwand eingestellt oder radikal reduziert.

15.4 **Geschäftsmodell (Hypothese):** Open Core (Kern unter MIT/Apache-2.0). Bezahlt werden: Installationspaket und Support, Audit-Report-Paket (20), Schulung/Einrichtung. Ein Pilotkunde zahlt, **bevor** das nächste Feature gebaut wird.

---

## 16. Arbeitsregeln für LLM-Implementierer

### 16.1 Aufgabenformat

Jede Aufgabe nennt: WP-ID, erlaubte Dateien, Akzeptanzkriterien, zu schreibende Tests. Der Implementierer arbeitet nur daran.

### 16.2 Definition of Done (alle Punkte, sonst Aufgabe offen)

1. `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` grün (lokal reproduziert, Ausgabe im PR).
2. Alle Akzeptanzkriterien des WP erfüllt und durch Tests belegt.
3. Keine Regelverletzung R-01..R-14; keine neuen Abhängigkeiten ohne Eintrag in `DEPENDENCIES.md`.
4. Keine Änderung an `SPEC.md`, keine Änderung außerhalb der erlaubten Dateien.
5. Öffentliche Items haben Rustdoc. Fehlerfälle sind dokumentiert.
6. PR-Text enthält: Zusammenfassung, Liste der Tests, offene Fragen (`## FRAGE`), bekannte Grenzen.

### 16.3 Verbotenes Verhalten

Tests löschen, deaktivieren oder abschwächen; Regeln mit `#[allow(...)]` umgehen (jede `allow` braucht eine Zeile Begründung im PR und Freigabe des Maintainers); Code kopieren, der Tore nicht besteht; große Refactorings; Dateien anderer WPs „reparieren“; Ergebnisse erfinden (nie behaupten, ein Test sei gelaufen, wenn er nicht lief).

### 16.4 Ehrlichkeit

Ist ein Teil nicht fertig, schreibt der PR das ausdrücklich. Ein unvollständiger, ehrlicher PR ist erwünscht; ein scheinbar vollständiger PR mit versteckten Lücken wird abgelehnt.

---

## 17. Offene Entscheidungen mit Default

| Nr. | Frage | Default bis zur Entscheidung |
|---|---|---|
| E-1 | Standard-Embedding-Modell | `bge-m3` (mehrsprachig); Qualität in WP-13 messen |
| E-2 | HNSW-Crate | erst nach Messung (11) |
| E-3 | Web-Framework für HTTP | `axum` |
| E-4 | Serialisierung intern | wie in 6.3 beschrieben (handgeschriebenes Binärformat), JSON nur für Metadaten und Keyring |
| E-5 | Lizenz | MIT OR Apache-2.0 |
| E-6 | Mehrere Subjekte pro Dokument | nicht erlaubt: genau ein Subjekt pro Dokument |
| E-7 | Sprache Collection-übergreifend | je Collection genau eine Sprache (`de` Default) |
| E-8 | Windows-Unterstützung | v0.1 nur Linux und macOS getestet; Windows nicht zugesichert |

---

## Anhang A: Fehlercodes

| Code | Bedeutung |
|---|---|
| `E_INVALID` | Eingabe ungültig (Format, leerer Text, ungültiger Name) |
| `E_LIMIT` | Größenlimit überschritten |
| `E_AUTH` | Token fehlt oder falsch |
| `E_NOT_FOUND` | Collection/Dokument unbekannt oder Subjekt vergessen |
| `E_CONFIG` | Konfiguration unzulässig (z. B. Nicht-Loopback ohne Token) |
| `E_KEY_SOURCE` | Keine oder mehrere Schlüsselquellen gesetzt |
| `E_KEY_WRONG` | Passphrase/Schlüsseldatei passt nicht zum Keyring |
| `E_KEYRING` | Keyring beschädigt oder nicht lesbar |
| `E_SCHEMA` | Schemaversion der Datenbank unbekannt |
| `E_LOCKED` | Datenverzeichnis wird von anderer Instanz benutzt |
| `E_EMBED` | Embedding-Dienst nicht erreichbar oder fehlerhaft |
| `E_EMBED_MISMATCH` | Modell oder Dimension weicht von der Collection ab |
| `E_CRYPTO` | Entschlüsselung/Authentifizierung fehlgeschlagen |
| `E_INTERNAL` | Unerwarteter interner Fehler (ohne Inhalte in der Meldung) |

## Anhang B: Kurzcheckliste für Reviewer

1. Läuft der End-to-End-Test gegen das echte Release-Binary und ist er grün?
2. Findet der Scanner den Canary **vor** `forget` nirgends und **nach** `forget` nirgends?
3. Stehen in Produktionscode keine Mocks, keine Platzhalter, kein `unwrap`, kein `unsafe`?
4. Enthält irgendein Log, eine Fehlermeldung oder eine temporäre Datei Inhalte?
5. Gibt es neue Abhängigkeiten oder Dateien außerhalb des WP-Umfangs?
6. Verwendet ein Text eines der verbotenen Wörter aus 1.5?

---

# TEIL II — Repository-Befund und Erweiterung (Version 1.1)

**Stand:** 23.10.2026 · **Quelle:** `https://github.com/tfufuz1/contextra`, Branch `main`, HEAD `076a9882e` · **Status:** normativ als Ergänzung zu Teil I.

Teil I (Abschnitte 0–17, Anhänge A–B, bis einschließlich Zeile 613) bleibt unverändert und ist die Produktspezifikation. Teil II liefert den verifizierten Ist-Zustand des Repositories, die Gap-Analyse gegen Teil I, das Übernahme-Mapping und die offenen Fragen. Bei Widerspruch zwischen Teil I und Teil II gilt Teil I; der Widerspruch ist als `## FRAGE` zu melden (0.3).

## D. Methode und Grenzen

D.1 Die Analyse ist **statisch**. Es wurde nichts kompiliert und kein Test ausgeführt. Die Rust-Toolchain war im Analyse-Container nicht installiert, `static.rust-lang.org` nicht erreichbar. Alle Zahlen stammen aus `git`, `grep`, `awk`, `wc`.

D.2 Zahlen sind Momentaufnahmen. Die Reproduktion steht in Anhang J.

D.3 Aussagen aus Projektdokumenten (`WORKING_STATE.md`, `capabilities.toml`, `AGENTS.md`) werden nur übernommen, wenn sie im Code nachweisbar sind. Abweichungen sind in E.3 und Anhang G benannt.

## E. Ist-Zustand des Repositories

### E.1 Kennzahlen (gemessen)

| Messgröße | Wert | Anmerkung |
|---|---|---|
| Commits auf `main` | 5.432 | |
| Remote-Branches | 415 | überwiegend `jules-*`, `audit-*`, `fix/*`, `feat/*` |
| Tags | `park/EP1/v1` … `park/EP6/v1`, `restore-base-pre-R13`, `snapshot/pre-cut` | **`archive/legacy-v1` fehlt** (Teil I, 13.1) |
| Rust-Dateien | 1.939 | |
| Rust-Zeilen gesamt | 426.548 | inkl. Tests, Benches, `xtask` |
| Workspace-Mitglieder | 32 | `default-members`: 17 |
| `#[test]` / `#[tokio::test]` | 3.099 / 2.244 | |
| `#[ignore]` | 36 | R-11 verbietet das |
| `#[allow(` | 787 | jede braucht Begründung (AGENTS.md §6, Teil I 16.3) |
| `.unwrap()` / `.expect(` / `panic!(` | 8.472 / 6.355 / 411 | alle Dateien inkl. Tests |
| `todo!` / `unimplemented!` | 6 | R-01 |
| `unsafe`-Treffer | 336 in 45 Dateien | R-04 |
| `*.rs.parked` | 6 | Geparkter Code (AGENTS.md §8) |
| Zeilen in `.github/unwrap_baseline.txt` | 775 | Ratchet-Basis |
| Mock-/Fake-/Null-Typdeklarationen | mindestens 75 (Top-15-Liste, nicht vollständig) | z. B. `MockEmbedder` 22×, `MockStorage` 13×, `FakeEmbedder` 7× |
| Toolchain | `1.89.0`, Komponenten `rustfmt`, `clippy` | identisch zu Teil I, 5.1 |

### E.2 Crate-Inventar

Zeilen = alle `*.rs`-Zeilen der Crate (Produktion, Tests, Benches). Ring und Status laut `capabilities.toml`/`WORKING_STATE.md`. Die dortigen LOC-Angaben sind nicht identisch mit der Messung und wurden nicht übernommen.

| Crate (Pfad) | Zeilen | Ring | Status (laut Repo) | Entscheidung für v1 (siehe G) |
|---|---:|---|---|---|
| `contextra-store` | 83.505 | 4 | stable | **neu** (Teil I 6.2 + 2.2.3) |
| `xtask` | 52.709 | – | Governance-Tooling | **entfällt** (13.3) |
| `contextra-engine` | 39.510 | 6 | stable | **neu** (Teil I 8.x) |
| `contextra-crypto` | 33.303 | 2 | stable | **neu**; Bausteine nur nach Prüfung |
| `contextra-graph` | 30.024 | 3 | stable | **entfällt** (2.2.4) |
| `contextra-vector` | 29.013 | 5 | stable | **neu**, nur exakte Suche (8.3) |
| `contextra-db` | 15.349 | 8 | stable | **entfällt** |
| `contextra-text` | 14.637 | 2 | stable | **Übernahme prüfen** (BM25/Tokenizer) |
| `contextra-mcp` | 13.715 | – | stable | **neu** (9.2, andere Tool-Namen) |
| `contextra-checkpoint` | 9.829 | 4 | stable | **entfällt** |
| `contextra-infer-candle` | 9.099 | 3 | stable | **entfällt** (2.2.6) |
| `contextra-router` | 8.755 | 3 | stable | **entfällt** (2.2.5) |
| `contextra-types` | 8.656 | 0 | stable | **Übernahme prüfen** (Newtypes, Fehler) |
| `contextra-testkit` | 8.258 | 2 | stable | **Basis** für Scanner, Harness (10.2a, WP-08) |
| `contextra-kvcache` | 7.575 | 3 | stable | **entfällt** (2.2.6) |
| `contextra-adapt` | 7.535 | 2 | experimental | **entfällt** (PID-Controller) |
| `contextra-rank` | 6.887 | 2 | stable | **nur RRF-Logik prüfen** (8.3) |
| `contextra-mvcc` | 6.190 | 2 | stable | **entfällt** |
| `contextra-ports` | 5.807 | 1 | stable | **entfällt** (Traits neu in types) |
| `contextra` (Facade) | 5.561 | 9 | stable | **entfällt**; API-Namen nicht übernehmen |
| `contextra-infer-ollama` | 5.324 | 3 | stable | **Übernahme prüfen**, nur `/api/embed` (9.5) |
| `contextra-privacy` | 4.787 | 2 | stable | **entfällt** (2.2.7) |
| `contextra-sys` | 3.537 | 0 | stable, unsafe-Insel | **entfällt** (R-04) |
| `contextra-simd` | 3.091 | 4 | stable, unsafe-Insel | **entfällt** (2.2.10) |
| `contextra-durable-fs` | 2.634 | – | – | **entfällt** (WAL-nah) |
| `contextra-wire` | 2.104 | 0 | stable, unsafe-Insel (FlatBuffers) | **entfällt** (R-04) |
| `contextra-license` | 2.003 | 2 | experimental | **entfällt** (2.2.7) |
| `contextra-audit-export` | 1.879 | 2 | experimental | **entfällt** (2.2.2, 13.3) |
| `benchmarks/store_vs_redb` | 1.344 | – | – | **entfällt** |
| `contextra-store-redb` | 1.188 | – | – | **Übernahme prüfen** (redb-Adapter, 13.2) |
| `contextra-avv-generator` | 725 | 1 | experimental | **entfällt** (2.2.7) |
| `contextra-core` | 517 | 3 | stable | **entfällt** |
| `benchmarks/contextra-bench` | 0 `.rs` | 9 | „unklassifiziert“ | Verzeichnis existiert, leer |

### E.3 In Projektdokumenten genannt, aber nicht auf Platte

Diese Crates stehen in `WORKING_STATE.md` oder `capabilities.toml`, fehlen aber im Klon: `contextra-agent`, `contextra-sandbox`, `contextra-py`, `contextra-infer-onnx`, `contextra-cognition`. `Cargo.toml` verweist unter `[workspace.dependencies]` auf Pfade dieser Crates. Ob `cargo` deswegen scheitert, ist ungeprüft.

Zusätzlich: `docs/spec/CONTEXTRA_V1_LOCAL_CODING_CONTEXT.md`, das AGENTS.md §8 als maßgeblich nennt, existiert nicht.

### E.4 Abhängigkeitsgraph (normale Dependencies, gemessen)

- **Kein Zyklus** in normalen Abhängigkeiten. `contextra-kvcache → contextra-crypto` ist normal; die Rückrichtung `contextra-crypto → contextra-kvcache` existiert nur als Dev-Abhängigkeit.
- **`contextra-testkit`** ist in 8 Crates Dev-Abhängigkeit und in keiner normalen. R-02 ist damit im Ist-Zustand erfüllt. Den CI-Job `forbid-testkit` gibt es in den Workflows nicht (grep negativ).
- **Schichtung weicht ab** von Teil I, 5.2. `contextra-crypto` hängt von `contextra-durable-fs` und `contextra-ports` ab, `contextra-mcp` von der Facade `contextra` und von `contextra-infer-candle`.
- **`contextra-mcp`** bündelt 15 Tools. Eines heißt `contextra_cloud_query`; das widerspricht R-09 und 2.2.8.

### E.5 Schnittstellen im Ist-Zustand

- **MCP (15 Tools):** `contextra_collections`, `contextra_create_collection`, `contextra_drop_collection`, `contextra_insert`, `contextra_upsert`, `contextra_get`, `contextra_delete`, `contextra_search`, `contextra_relate`, `contextra_relate_n_ary`, `contextra_explain`, `contextra_forget`, `contextra_cloud_query`, `contextra_plugin_status`, `contextra_mcp`. Teil I, 9.2 verlangt `memory_store`, `memory_search`, `memory_forget`, `memory_collections`.
- **Binaries:** `contextra-mcp-server` und `contextra-mcp-cli` (in `crates/contextra-mcp/src/bin/`). Ein Binary `contextra` mit `serve`, `mcp`, `init`, `forget`, `verify`, `version` (Teil I, 9.3) existiert nicht.
- **HTTP:** keine `axum`- oder `hyper`-Abhängigkeit, keine HTTP-API. Teil I, 9.1 ist vollständig offen.
- **Facade (`crates/contextra/src/lib.rs`):** exportiert `AgentMemory`, `Memory`, `MemoryId`, `ContextraBuilder`, `ContextraError`, `DocId`, `ScoredDocument`, `DistanceMetric`. `AgentMemory::forget(&self, id: &MemoryId)` steht in `agent_memory.rs:207`.
- **Ollama-Client:** Endpunkt `/api/embed` an 9 Stellen, `/api/embeddings` an 4 Stellen. Teil I, 9.5 kennt nur `/api/embed`.

### E.6 Kryptografie im Ist-Zustand

- **Crate-Abhängigkeiten `contextra-crypto`:** `ring`, `sha2`, `hmac`, `aes-gcm-siv`, `argon2`, `hkdf`, `subtle`, `zeroize` (mit `zeroize_derive`), `ed25519-dalek`. Teil I, 7.1 erlaubt exakt: `aes-gcm-siv`, `argon2`, `hkdf`, `sha2`, `hmac`, `zeroize`, `getrandom`, `subtle`. `ring` und `ed25519-dalek` sind nicht erlaubt; `ed25519-dalek` ist ausdrücklich verboten (Signaturen, Teil I 2.2.2).
- **Schlüsselmodell:** Es gibt kein `keyring.json` und kein DEK-pro-Subjekt-Modell. Vorhanden sind `keystore.rs`, `kms.rs`, `scope_keyring.rs`, `kdf.rs`, `kv_shredding.rs`, `wal_crypto.rs`, `revocation_log.rs`, `audit_chain.rs`, `wrapped_dek`/`wrapped_kek`-Felder und ein `ErasureLedger`.
- **Löschnachweis:** `crates/contextra-engine/src/contextra_impl/forget.rs` erzeugt `ForgetReceipt` mit `ProofV4` (Ed25519-signiert, mit Assurance-Stufen, Layer-Witnesses und Ledger). Das ist genau das, was Teil I 2.2.1/2.2.2 verbietet.

### E.7 Speicher im Ist-Zustand

- **redb:** `contextra-store/Cargo.toml` führt `redb = "2"`, zusätzlich gibt es `contextra-store-redb`.
- **Hauptpfad ist ein LSM-Baum:** Module `lsm/`, `wal/`, `sstable/`, `memtable.rs`, `compaction/`, `manifest/`, `erasure_ledger/`, `restore_guard.rs`. Teil I, 2.2.3 verbietet eine eigene Storage-Engine (LSM, WAL, Kompaktion); Persistenz ist ausschließlich `redb`.

### E.8 Forget, Canary, Testkit im Ist-Zustand

- **Vorhandene Bausteine, die für Teil I, 10.2a/WP-08 nutzbar sind:** `crates/contextra-testkit/src/erasure_harness/canary.rs`, `crates/contextra-testkit/src/deletion_oracle/rawsweep.rs`, `deletion_oracle/marker.rs`, `process_image_probe.rs`.
- **Nicht vorhanden:** `fake-ollama` (eigene Implementierung von `/api/embed`), Crash-Harness mit Feature `crash-points`, `e2e-fast`/`e2e-real`-Tests gegen das Release-Binary.

### E.9 Lints, Lizenzen, Dateien

- **Workspace-Lints** (`Cargo.toml`): `unsafe_code = deny` mit drei Ausnahmen; `unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`, `unreachable` = `deny` (erfüllt R-03). `indexing_slicing` = **`warn`** (Teil I verlangt `deny`), `arithmetic_side_effects` = `warn`, `unexpected_cfgs` = `warn` (kein `-D warnings`), `let_underscore_must_use` nur als TODO.
- **`deny.toml`:** Allowlist `MIT`, `Apache-2.0`, `Apache-2.0 WITH LLVM-exception`, `BSD-2-Clause`, `BSD-3-Clause`, `ISC`, `Unicode-DFS-2016`, `CC0-1.0`, `Zlib`. `CC0-1.0` steht **nicht** in R-10.
- **Fehlende Pflichtdateien:** `SPEC.md`, `DEPENDENCIES.md`, `RESIDUAL-RISKS.md`, `.github/workflows/ci.yml`, `eval/`, `clients/`, `docker/`.
- **Vorhanden, aber Konkurrenz zu Teil I:** `AGENTS.md`, `CONSTITUTION.md`, `VETOES.md`, `capabilities.toml`, `WORKING_STATE.md`, zwei Produktspezifikationen in `docs/spec/` (`CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md`, `CONTEXTRA_SPEZIFIKATION_v15.md`, dazu 30+ Kapitel), 136 ADRs in `docs/decisions/`, 15+ Workflows in `.github/workflows/` (u. a. `rust-ci.yml`, `pr-gate.yml`, `loom-gate.yml`, `fuzz-extended.yml`).

## F. Gap-Analyse: Teil I gegen Ist-Zustand

| Regel | Befund (gemessen) | Status |
|---|---|---|
| R-01 Kein Platzhalter | 6 × `todo!`/`unimplemented!`; ≥ 75 Mock/Fake-Typen (Test-Zugehörigkeit ungeprüft) | **verletzt** / prüfen |
| R-02 Keine Mocks im Prod-Pfad | keine normale Abhängigkeit auf `testkit`; CI-Job fehlt | **teilweise** |
| R-03 Kein unwrap/expect/panic | Lints `deny` gesetzt; 8.472 `.unwrap()` im Repo, Ratchet bei 775 | **verletzt** (Bestand) |
| R-04 Kein unsafe | `unsafe` in 45 Dateien; `simd`, `sys`, `wire` ohne `forbid` | **verletzt** |
| R-05 Null Warnungen | `unexpected_cfgs = warn`; `-D warnings` in CI ungeprüft | **teilweise** |
| R-06 Fail closed | kein `keyring.json`, kein Startabbruch nach Teil I-Muster | **nicht erfüllt** |
| R-07 Kein Klartext auf Platte | nicht geprüft; WAL/SSTable-Format unbekannt | **offen** |
| R-08 Kein Inhalt in Logs | nicht geprüft; `tracing` ist Workspace-Dependency | **offen** |
| R-09 Kein Netzwerk außer Embedder | Tool `contextra_cloud_query`; Telemetrie nicht geprüft | **verletzt** (Tool) |
| R-10 Abhängigkeiten | `deny.toml` vorhanden, `CC0-1.0` zusätzlich erlaubt; `DEPENDENCIES.md` fehlt | **teilweise** |
| R-11 E2E grün | keine E2E-Tests gegen Release-Binary; 36 `#[ignore]` | **nicht erfüllt** |
| R-12 Determinismus | Ports `Clock`/`Rng` laut AGENTS.md; `SystemTime::now` nicht gezählt | **teilweise** |
| R-13 Geheimnistypen | `zeroize` vorhanden; `ZeroizeOnDrop` und fehlendes `Debug` nicht geprüft | **offen** |
| R-14 Scope-Disziplin | Repo-weit Verstöße (große Crates, Querabhängigkeiten) | **verletzt** (Bestand) |

## G. Übernahme-Mapping (Teil I, 13.2 und 13.4)

**Grundsatz:** Der Bestand wird nicht gehärtet, sondern als Quelle für gezielte Übernahme genutzt. Übernahme erfolgt nur nach Prüfung, dass der Code 7.x, 8.x und R-01..R-14 erfüllt. Ist die Anpassung aufwendiger als Neuschreiben, wird neu geschrieben (13.4).

**G.1 Prüfen und ggf. übernehmen**

| Quelle im Repo | Ziel-Crate | Was übernehmen | Was verwerfen |
|---|---|---|---|
| `contextra-text` (Tokenizer, BM25) | `contextra-text` | Tokenisierung, BM25-Kern, Property-Tests | Async, Store-Abhängigkeit, Ports-Kopplung |
| `contextra-rank` (RRF) | `contextra-engine` | Nur die RRF-Formel mit Tests | Kalibrierung, Bonus-Scorer, Isotonic/Platt |
| `contextra-vector` (Distanz, Normalisierung) | `contextra-vector` | Normalisierung, Skalarprodukt | HNSW, DiskANN, Quantisierung, `unsafe`-SIMD |
| `contextra-testkit/src/erasure_harness/canary.rs`, `deletion_oracle/rawsweep.rs` | `contextra-testkit` | Scanner-Logik als Basis für 10.2a | Bindung an Ledger/Proof |
| `contextra-infer-ollama` (HTTP-Client) | `contextra-embed` | `OllamaProvider`-Gerüst, Fehlertypen | `/api/embeddings`, Chat-/Generate-Pfade |
| `contextra-store-redb` | `contextra-store` | redb-Öffnen, Tabellenhilfen | nur wenn verschlüsselte Records gemäß 6.2 schreibbar sind |
| `contextra-types` | `contextra-types` | Newtype-Muster, Fehlerenum-Struktur | Tenant-Typen, Ring-spezifische Typen |

**G.2 Neu schreiben (nicht übernehmen)**

- `contextra-crypto`: Schlüsselmodell DEK pro Subjekt und Collection, Keyring-Format 7.6, atomares Schreiben 7.7, AAD 7.5. Der Bestand hat ein anderes Modell (Wrapping pro Scope, Ledger, Proofs).
- `contextra-store`: redb-Tabellen 6.2 mit verschlüsselten Records. Der Bestand ist ein LSM-Baum und verboten.
- `contextra-engine`: Chunking 8.1, Ingest 8.2, Hybridsuche 8.3, Forget 8.4, Startup 8.5, Nebenläufigkeit 8.6.
- `contextra-server` (neu, Binary `contextra`): HTTP mit `axum` (E-3), MCP mit `rmcp` und den vier `memory_*`-Tools, CLI (9.3).

**G.3 Entfernen (Nicht-Ziele, Teil I 2.2 und 13.3)**

Graph (`contextra-graph`), KV-Cache (`contextra-kvcache`), Candle (`contextra-infer-candle`), Router und Bandit (`contextra-router`), Adapt/PID (`contextra-adapt`), Checkpoint, MVCC, Privacy, AVV-Generator, Audit-Export, Lizenz, `contextra-db`, `contextra-durable-fs`, `contextra-sys`, `contextra-simd`, `contextra-wire`, `contextra-core`, `contextra-ports`, `xtask`, `benchmarks/`, `*.rs.parked`-Dateien.

Die Entfernung erfolgt **nicht** im Hauptzweig vor dem Tag `archive/legacy-v1` (siehe Anhang I, Schritt 1).

## H. Technische Lücken in Teil I (mit Vorschlag, 0.3)

Die folgenden Punkte sind in Teil I nicht eindeutig. Vorschläge gelten als Default bis zur Entscheidung.

- **H.1 `top_k` größer als Kandidatenpool.** 8.3 holt je Liste Top-50; 6.4 erlaubt `top_k` bis 100. Die RRF-Ausgabe kann dann nicht 100 Treffer liefern. *Vorschlag:* Kandidatenpool = `max(50, top_k)`, Konstante dokumentieren.
- **H.2 Nullvektor und Normalisierung.** 8.3 verlangt vorab normalisierte Vektoren. Ein Nullvektor ist nicht normalisierbar. *Vorschlag:* Nullvektor beim Ingest mit `E_EMBED` abweisen, Abbruch vor dem Schreiben.
- **H.3 „Wortgrenze“ bei Überlappung.** 8.1 richtet die 200-Zeichen-Überlappung auf die „nächste Wortgrenze“ aus, ohne Definition. *Vorschlag:* Wortgrenze = UAX #29 Word Boundary (`unicode-segmentation`), dann nächste Grenze nach dem 200-Zeichen-Schnitt.
- **H.4 Text kürzer als Zielgröße.** Die Mindestgröße letzter Chunk (200) greift nicht, wenn es nur einen Chunk gibt. *Vorschlag:* Ein Chunk, unabhängig von der Länge (solange nicht leer).
- **H.5 Forget unbekannter Subjekte.** 8.4 fordert Idempotenz; 9.1 mappt `E_NOT_FOUND` auf 404. Widerspruch für `POST /v1/forget`. *Vorschlag:* Unbekanntes Subjekt liefert `200` mit `documents: 0, chunks: 0, key_destroyed: false`; `E_NOT_FOUND` nur bei `GET /documents/{id}`.
- **H.6 `forget_intent` bei `collection: null`.** 8.4 schreibt `{started_at, collection}`; für „alle Collections“ fehlt ein Wert. *Vorschlag:* Feld `collection` mit `"*"` belegen.
- **H.7 Startup ohne Schlüsselsource bei vorhandenem Datenverzeichnis.** Nicht definiert. *Vorschlag:* `E_KEY_SOURCE` vor jedem Dateizugriff außer `contextra init`.
- **H.8 `contextra init` auf vorhandenem Datenverzeichnis.** Nicht definiert. *Vorschlag:* Abbruch mit `E_INVALID`, kein Überschreiben.
- **H.9 Klartext im BM25-Index.** Der Index hält Terme im Heap. Teil I 8.4 Schritt 2 fordert „Klartext-Puffer zeroizen“; das deckt HashMap-Schlüssel nicht ab. *Vorschlag:* Term-Schlüssel in `Zeroizing<String>` bzw. Index-Entfernung plus Eintrag in `RESIDUAL-RISKS.md` Nr. 2.
- **H.10 Embedding-Dimension bei Collection-Anlage.** 9.1 ermittelt `dim` per Probe-Embedding. Bei Modellwechsel des Embedders zwischen Anlage und Ingest greift `E_EMBED_MISMATCH` (9.5). *Vorschlag:* Probe-Ergebnis in `collections` persistieren, nicht neu ermitteln.

## I. Offene Entscheidungen des Maintainers (FRAGEN)

| Nr. | Frage | Default bis zur Entscheidung |
|---|---|---|
| F-01 | Neues Repository (Teil I, 5.1) oder Neubau in `tfufuz1/contextra`? | Neubau in diesem Repo auf Branch `v1`, `main` unangetastet |
| F-02 | Rangfolge: Teil I gegen `AGENTS.md`/`docs/spec/` | Teil I gilt für v1; `AGENTS.md` verweist auf Teil I |
| F-03 | MCP-Toolnamen: `memory_*` (Teil I) oder `contextra_*` (Repo) | `memory_*` (9.2 ist verbindlich) |
| F-04 | `/api/embeddings` zusätzlich zu `/api/embed` | Nur `/api/embed` |
| F-05 | WP-05 (Embed) braucht `fake-ollama` aus WP-08, beide stehen in Welle 1 parallel | WP-08 vor WP-05 einplanen (Teil I 14 anpassen durch Maintainer) |
| F-06 | Krypto-Bausteine: `ring`, `ed25519-dalek`, `blake3` im Repo | Nur Teil I 7.1; `getrandom` neu aufnehmen (Eintrag in `DEPENDENCIES.md`) |
| F-07 | `CC0-1.0` in `deny.toml` | Nicht aufnehmen (R-10) |
| F-08 | `contextra_cloud_query` im Repo | Im Neubau nicht vorhanden |
| F-09 | Teil I 13.1: „108.000 / 179.000 Zeilen“ | Nicht reproduzierbar; gemessen 426.548 Zeilen gesamt. Maintainer aktualisiert Teil I |
| F-10 | `Cargo.toml` Zeile „Spezifikation Section 0.5 / A.3“ | Kommentar veraltet, ignorieren |
| F-11 | Löschnachweise (`ProofV4`, Ledger) im Bestand | Nicht übernehmen (2.2.1, 2.2.2) |
| F-12 | Lizenzen `rust-stemmers`, `unicode-segmentation` | Vor WP-03 in `DEPENDENCIES.md` prüfen (R-10) |
| F-13 | CI-Jobs aus Teil I 10.6 (`forbid-testkit`, `no-placeholders`, `e2e-fast`, `crash`) fehlen | In WP-00 anlegen |
| F-14 | Verhalten Forget bei unbekanntem Subjekt (H.5) | Vorschlag H.5 |
| F-15 | Klartext im Index (H.9) | Vorschlag H.9 |

## J. Prüfkommandos (reproduzierbar, ohne `rg`)

```bash
# Klon und Stand festhalten
git clone https://github.com/tfufuz1/contextra && cd contextra && git rev-parse --short HEAD

# R-01 Platzhalter (ohne Testverzeichnisse)
grep -rnE 'todo!|unimplemented!' crates --include='*.rs' | grep -v '/tests/'

# R-03 Panik-Stellen in Produktionscode (grob; Test-Module manuell ausschließen)
grep -rnE '\.unwrap\(\)|\.expect\(|panic!\(|unreachable!\(' crates --include='*.rs' | grep -v '/tests/'

# R-04 unsafe
grep -rnE '\bunsafe\b' crates --include='*.rs'
grep -L 'forbid(unsafe_code)' crates/*/src/lib.rs

# R-02 keine normale Abhängigkeit auf testkit (nach Umbau: cargo tree)
cargo tree -e normal -i contextra-testkit   # erwartet: "nothing to print" bzw. Fehler

# R-10 Lizenzen und Advisories
cargo deny check

# Gesamtgates (Teil I 16.2)
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## K. Ablauf für den ersten Implementierer

1. **Maintainer:** F-01 bis F-03 entscheiden. Danach Tag setzen: `git tag archive/legacy-v1 076a9882e` (neuer Tag, kein Verschieben bestehender Tags, AGENTS.md §8).
2. **Maintainer:** Teil I 14 (Welle 1) um die Abhängigkeit WP-08 → WP-05 korrigieren (F-05).
3. **WP-00** auf Branch `v1` mit Dateiliste aus Teil I 14. Der Bestand bleibt bis dahin unberührt.
4. **Vor jedem WP:** Prüfkommandos aus Anhang J ausführen und Ergebnis in den PR-Text schreiben.
5. **Übernahme aus dem Bestand** nur über einen eigenen PR je Datei (13.2) mit Quellangabe (Commit-Hash und Pfad).

Teil II ist ein Arbeitsdokument. Änderungen macht nur der Maintainer (0.3a).
