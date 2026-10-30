# Contextra – Sicherheitsmodell (Kern v1)

<!-- doc-stamp: 2026-10-30 | Ersetzt: Sicherheitsmodell Fassung 1.2 (Desktop-App, MCP, Ollama). Normativ: docs/SPEC.md §3, §8, §9 -->

Dieses Dokument beschreibt das Bedrohungsmodell des **reduzierten Kerns v1**. Das frühere Modell mit Desktop-App, MCP-Server, Ollama-Anbindung und Privacy-Gateway gilt nicht mehr. Es ist entfallen, weil die zugehörigen Crates entfernt wurden (SPEC §5.2).

Dieses Dokument ist eine technische Beschreibung. Es ist keine Zertifizierung und keine Rechtsauskunft.

---

## 1. Schutzgüter

| Schutzgut | Ort | Schutzmechanismus |
|---|---|---|
| Dokumente und Metadaten | SSTables, MemTable, WAL | AEAD-Verschlüsselung (`contextra-crypto`), Integritätsprüfung (HMAC-Kette im WAL, CRC32 in Blöcken) |
| Schlüsselmaterial | Speicher des Prozesses | Ableitung per HKDF, Löschung beim Verwerfen (zeroized) |
| Mandantentrennung | alle Schichten | Schlüssel-Präfix, Filter vor Indexabfrage, Mandant aus `RequestContext` (VETO-F10) |
| Suchindex | HNSW, BM25 | Löschung nach VETO-F02 |
| Build-Integrität | Lieferkette | `deny.toml`, `supply-chain/` (cargo-vet), `Cargo.lock` |

---

## 2. Vertrauensgrenzen

- **Prozessgrenze:** Contextra läuft als Bibliothek im Prozess der Anwendung. Die Anwendung ist vertrauenswürdig. Angriffe aus dem selben Prozess sind nicht Gegenstand des Modells.
- **Dateisystemgrenze:** Das Datenverzeichnis wird als nicht vertrauenswürdig behandelt, sobald es außerhalb des Prozesses verändert werden kann. Beschädigte oder manipulierte Dateien müssen zu einem benannten Fehler führen (Regel 10, Fail-Closed).
- **Mandantengrenze:** Anfragen verschiedener Mandanten dürfen sich nicht beeinflussen oder Daten offenlegen (VETO-F10).
- **Netzgrenze:** Contextra öffnet keine Netzwerkverbindung. Eine Netzwerkfunktion ist im Kern nicht vorgesehen (Single-Node-Doktrin).

---

## 3. Bedrohungen und Gegenmaßnahmen

| Bedrohung | Gegenmaßnahme | Status |
|---|---|---|
| Offenlegung der Daten bei Diebstahl des Datenträgers | Verschlüsselung at rest, in jedem Build aktiv (Regel 8) | Umgesetzt im Code. Feature-Konflikt `encryption-at-rest` offen (SPEC §16, B4) |
| Manipulation von WAL oder SSTables | HMAC-Verkettung, CRC32, Fail-Closed beim Öffnen | Umgesetzt im Code, Tests in `contextra-store` |
| Mandantenübergreifende Abfrage | Mandant aus Anfragekontext, Filter vor Indexzugriff | Tests vorhanden, Vollständigkeit wird in PLAN R1 geprüft |
| Verlust von Daten durch Absturz | WAL-Replay, `repair_on_open`, Fehler bei `fsync` werden propagiert | Umgesetzt, Testabdeckung in `contextra-store` unterschiedlich |
| Speicherfehler in unsicherem Code | Unsafe nur in `contextra-sys` und `contextra-simd`, jeder Block mit `// SAFETY:` | Prüfung über `tests/unsafe_islands.rs`; SAFETY-Kommentare werden in PLAN R1.5 vervollständigt |
| Ressourcenerschöpfung durch große Eingaben | Budgets in `contextra-types`, Obergrenzen bei Ergebnissen | Teilweise, Obergrenzen je Pfad in R1 zu prüfen |
| Schadhafte Abhängigkeit | `cargo deny`, `cargo vet`, `Cargo.lock` | Konfiguration vorhanden, Lauf in R4 |
| Nicht bereinigte Schlüssel im Speicher | Zeroize beim Verwerfen | Umgesetzt in `contextra-crypto` |

---

## 4. Nicht im Modell

- Angriffe auf das Betriebssystem, den Host oder die Hardware.
- Auslagerungsspeicher (Swap), Core-Dumps und Speicherabbilder außerhalb des gehärteten Prozesses.
- Datenträger-Firmware, Flash-Wear-Leveling und Überprovisionierung.
- Externe Sicherungen des Betreibers.
- Kopien und Übermittlung von Daten durch die Anwendung.
- Modellparameter und Prompt-Verläufe der Anwendung.

Diese Punkte entsprechen den Ausschlüssen in SPEC §9.3 und §14.

---

## 5. Meldung von Schwachstellen

Sicherheitsbefunde bitte nicht öffentlich als Issue melden. Die Meldung geht an den Maintainer des Repositorys. Eine Antwort wird angestrebt, aber nicht zugesichert. Eine Behebung folgt einem Plan nach `docs/PLAN.md`.

---

## 6. Verweise

- `docs/SPEC.md` §3 (Regeln), §8 (Persistenz), §9 (Löschung), §14 (Datenlebenszyklus)
- `docs/VETOES.md` (VETO-F10, VETO-F02)
- `deny.toml`, `supply-chain/`
