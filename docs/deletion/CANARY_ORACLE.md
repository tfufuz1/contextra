# Contextra Canary Oracle & Deletion Verification Architecture

## 1. Übersicht & Zielsetzung

Das Canary-Orakel ist ein von der Produktionslogik vollständig unabhängiges Verifikationssystem für das Contextra-Alleinstellungsmerkmal **"nachweisbar vergessen"**.

Es beantwortet die zentrale Prüffrage:
> *Wenn eine Person aus der Datenbank gelöscht wird, existieren danach noch irgendwelche Fragmente oder Kodierungen ihrer Daten im gesamten Datenverzeichnis auf der Festplatte?*

Der Verifikations-Scanner teilt **keinen Code** mit der zu prüfenden Produktionslogik, verwendet keine Contextra-Dekodierer und stützt sich nicht auf produktionsseitige Zeugen (`witness_scan`). Er arbeitet direkt auf den rohen Dateibytes des Dateisystems.

---

## 2. Orakel-Architektur & Funktionsweise

### 2.1 Unabhängigkeit
Der Scanner nutzt ausschließlich die Standardbibliothek (`std::fs`, `std::io`) zur rekursiven Verzeichnisanalyse:
- Es werden **alle Dateien** analysiert (inklusive versteckter Segmente, temporärer Dateien, WALs, SSTables, Indizes und Manifesten).
- Pufferüberlappung (`64 KiB` Chunks mit Überlappung) stellt sicher, dass Suchmuster über Chunk-Grenzen hinweg lückenlos erkannt werden.

### 2.2 Canary-Generierung
Ein Canary ist ein eindeutiger, zufälliger Wert pro Person und Datenart mit mindestens **24 Byte Entropie** aus einem festen Seed.

Für jeden Canary werden automatisch alle relevanten Kodierungsvarianten konstruiert:
1. **Raw UTF-8**: Reine Bytefolge der Canary-Zeichenkette.
2. **JSON-escaped**: Maskierte Zeichenfolgen in JSON-Payloads (`\"`, `\n`, etc.).
3. **Hex (lowercase & uppercase)**: Hexadezimale Repräsentationen.
4. **Base64 (Standard)**: Base64-kodierte Zeichenfolgen.
5. **UTF-16LE & UTF-16BE**: In-Memory- und Index-Kodierungen.
6. **Lowercase String**: Kleingeschriebene Suchmustersynthese (für tokenisierte Textindizes).
7. **Fragmente**: Teilfolgen ab 12 Byte Länge.
8. **Vektor-f32 (Little-Endian)**: Die rohe Little-Endian `f32`-Bytefolge der Vektorkomponenten.

---

## 3. Test-Matrizen

### 3.1 Injektions-Matrix (`p08_canary_oracle.rs`)
Zur Verifikation der Orakel-Gültigkeit wird ein Positivtest durchgeführt: Ein Canary wird absichtlich in eine Kopie jeder Dateifamilie aus `docs/deletion/KEYFAMILIES.toml` injiziert. Der Scanner **muss** jeden injizierten Canary finden:
- WAL Log-Segmente (`.wal`)
- SSTables (`.sst`)
- Manifest-Dateien (`MANIFEST-*`)
- BM25 Textindizes (`.idx`)
- HNSW Vektorindizes (`.bin`)
- CSR Graph-Dateien (`.adj`)
- Temporäre Scratch-Dateien (`.tmp`)
- Checkpoints (`.meta`)
- Audit-Protokolle (`.log`)
- Unbekannte Zusatzdateien (`.dat`)

### 3.2 Kodierungs-Matrix (`p08_canary_oracle.rs`)
Jede der 9 Kodierungsvarianten wird separat injiziert und muss vom Scanner mit Korrektheit von Pfad, Offset und Kodierungstyp gemeldet werden.

Gegenprobe: Ein sauberes Verzeichnis ohne Canaries liefert exakt **0 Treffer**.

### 3.3 Echte Löschprüfung (`p08_forget_canary.rs`)
Mehrere Personen (A, B, C) werden mit verschiedenen Erinnerungsarten (Text, Vektor, Beziehungen) angelegt.
- **Vor dem Löschen**: Der Scanner weist mindestens 1 Treffer pro Person nach.
- **Nach dem Löschen von Person A**:
  - Reopen der Datenbank.
  - Scanner über das gesamte Datenverzeichnis.
  - Erwartung: 0 Treffer für Person A.
  - Personen B und C bleiben zu 100% lesbar und abfragbar.
  - Ein DeletionProof wird erstellt und mit dem öffentlichen Ed25519-Verifikationsschlüssel geprüft.

### 3.4 Crash-Matrix (`p08_forget_crash.rs`)
Re-Exec-Muster für Absturztests während des Löschvorgangs:
- Der Elternprozess startet die Kindprozess-Workload.
- Das Kind emittiert Fortschrittsmarkierungen auf `stdout`:
  1. `MARKER_1_BEFORE_DELETE`
  2. `MARKER_2_STAGING_DELETE_OPS`
  3. `MARKER_3_BEFORE_STORAGE_FLUSH`
  4. `MARKER_4_AFTER_STORAGE_FLUSH`
  5. `MARKER_5_AFTER_PROOF_GENERATION`
- Der Elternprozess beendet das Kind gezielt per `Child::kill()` an jeder Fortschrittsmarkierung.
- Nach dem Neustart gelten exakt **zwei erlaubte atomare Ergebnisse**:
  1. **Löschung abgeschlossen**: Scanner liefert 0 Treffer für Person A, Nachweis ist gültig (falls ausgestellt).
  2. **Löschung zurückgerollt**: Person A ist per API wieder vollständig lesbar, kein Nachweis ausgestellt.
- Personen B und C bleiben in jedem Fall unversehrt.

---

## 4. Ausführungsanleitung

```bash
# 1. Kompilierprüfung der Datenbank und Test-Targets
cargo check -p contextra-db --all-targets

# 2. Ausführung der Injektions-, Kodierungs- und Orakel-Tests
cargo test -p contextra-db --test p08_canary_oracle

# 3. Ausführung des echten Löschtests
cargo test -p contextra-db --test p08_forget_canary

# 4. Ausführung der Absturz-Matrix (Re-Exec Pattern)
cargo test -p contextra-db --test p08_forget_crash

# 5. Ausführung der interaktiven Demo
cargo run -p contextra-db --example p08_forget_crash_demo -- 100
```

---

## 5. Was dieser Test NICHT beweist (Nicht-Garantien)

Das Canary-Orakel prüft die Anwesenheit von Datenfragmenten im konfigurierten Datenverzeichnis des Dateisystems. Es deckt folgende Systemebenen explizit **nicht** ab:

1. **Hardware- & Controller-Ebene (SSD Wear-Leveling & TRIM)**
   - Flash-Memory-Controller auf SSDs können gelöschte Blöcke vor dem physischen Erhalten auf Halbleiterebene verschieben.
2. **Dateisystem-Journaling-Reste**
   - Reste in Dateisystem-Journals (z. B. ext4 journal, NTFS USN Journal, ZFS Intent Log) außerhalb des Anwendungs-Datenverzeichnisses.
3. **Betriebssystem-Speicherabbilder (RAM / Swap / Core Dumps)**
   - Reste im Arbeitsspeicher oder Swap-Speicher des Betriebssystems.
4. **Bereits exportierte oder replizierte Daten**
   - Datenkopien, die vor dem Löschaufruf an externe Systeme, Backups oder Egress-Caches übertragen wurden.
5. **Schlüsselverwahrungsstufe**
   - Die physische Sicherheit oder HSM-Schutzstufe von verwendeten Tenant-Crypto-Schlüsseln.
