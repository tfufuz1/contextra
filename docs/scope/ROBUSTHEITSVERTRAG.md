# Contextra — Robustheitsvertrag & Evidenz-Register (ROBUSTHEITSVERTRAG.md)

**Stand:** 2026-10-12
**Autor:** Principal Senior Rust-Systemarchitekt / Workspace-Governance-Architekt
**Status:** Normativ

---

## 1. Fehlermodell

Contextra speichert Daten lokal für KI-Agenten und garantiert nachweisbares Vergessen. Das System ist gegen folgende reale Fehlerzustände gehärtet:

1. **Prozess-Kill (`SIGKILL` / Ungraceful Shutdown):** Plötzlicher Abbruch des Systemprozesses zu jedem beliebigen Zeitpunkt.
2. **Stromausfall (Power Outage):** Verpuffen flüchtiger Betriebssystem-Puffer mit Verlust nicht gefsyncter Daten.
3. **Teilschreibungen (Torn Writes / Partial Block Writes):** Abgebrochene Schreibvorgänge auf Sektor- oder Block-Ebene.
4. **`fsync`-Fehler:** I/O-Fehler (`EIO`, `ENOSPC`, Unreachable Storage) beim synchronen Durchschreiben auf den physikalischen Datenträger.
5. **Volle Platte (`ENOSPC`):** Schreibversuche bei vollständig belegtem Speicherplatz.
6. **Bit-Rot (Stille Datenkorruption):** Erkannte Bit-Fehler auf Dateiebene (via Blake3/CRC32 Checksummen erkannt; Erkennung garantiert, automatische Korrektur ohne Parität ist Nicht-Ziel).
7. **Beliebige Nutzereingaben:** Byteweise valide oder maliziöse Payloads, übergroße Felder, binäre Daten und ungültiges UTF-8.

---

## 2. Garantien

### G1: Crash-/WAL-Atomarität & Dauerhaftigkeit
Schreibtransaktionen werden vor Acknowledgment atomar in das Write-Ahead-Log (WAL v3 mit HMAC-Kette) geschrieben und synchron gefsynct. Beim Replay nach Crash/SIGKILL werden unvollständige oder beschädigte Transaktionen verworfen; bestätigte Transaktionen sind dauerhaft.

### G2: Kryptographische Löschung & Crypto-Shredding
Dokumente und Vektoren werden durch Schlüsselwiderruf (`RevocationLog`) unumkehrbar unlesbar gemacht. Das Löschen wird über kryptographische `DeletionProof`-Zertifikate (Ed25519 v3) nachweisbar belegt.

### G3: Nachweisbarkeit & Audit
Sicherheitsrelevante Operationen (Löschungen, Key-Revocations, Schema-Migationen) erzeugen unveränderliche Audit-Einträge. Exporte für DSGVO Art. 30 Verarbeitungsverzeichnisse sind deterministisch generierbar.

### G4: Mandantenisolation (Tenant Isolation)
Strikte logische und physikalische Isolation aller Daten nach `TenantId` (VETO-F10). Kein mandantenübergreifender Datenfluss oder KV-Cache-Leakage.

---

## 3. Nicht-Ziele (Out of Scope)

Folgende Bedrohungen und Restspeicher sind ausdrücklich **kein** Bestandteil der Systemgarantien von Contextra:

1. **SSD-Wear-Leveling-Reste:** Physikalische Speicherzellenreste auf Flash-Medien unterhalb der Betriebssystem-Dateisystem-Ebene (vor Krypto-Shredding).
2. **Backups außerhalb von Contextra:** Dateisystem-Snapshots, VM-Backups oder Cloud-Storage-Sicherungen, die vom Betreiber vor dem Löschbefehl angefertigt wurden.
3. **Bereits an Cloud-LLMs gesendete Daten:** Prompt-Payloads oder Kontext-Texte, die vor dem Löschaufruf an externe Drittanbieter-APIs übertragen wurden.
4. **Speicherabbilder (RAM Core Dumps):** Flüchtiger Prozessspeicher, Core Dumps oder OS-Hibernation-Dateien fremder Prozesse.
5. **Fremde Prozesse mit Dateizugriff:** Bösartige Root-Prozesse oder parallele Anwendungen, die direkte Schreibrechte auf die Contextra-Datendateien besitzen.
6. **Lügende Hardware:** Speichermedien, die `fsync`-Erfolge quittieren, ohne Daten flüchtig durchgeschrieben zu haben, oder I/O-Fehler stumm verschlucken.

---

## 4. Fail-Closed-Zustandsautomat

Contextra steuert die Betriebssicherheit über einen strikten Vier-Stufen-Zustandsautomaten:

```
    [ Healthy ] ───(Transient I/O Warning)───► [ Degraded ]
         │                                          │
    (ENOSPC / Read-Only)                      (ENOSPC / Read-Only)
         │                                          │
         ▼                                          ▼
    [ ReadOnly ] ◄───────────────────────────[ ReadOnly ]
         │                                          │
   (fsync-Fehler / Checksum Corruption / WAL Replay Fail)
         │
         ▼
    [ Poisoned ] ◄─── (HARD FAIL-CLOSED: Kein Weiterschreiben!)
```

### Zustandserklärungen & Übergangsregeln:
* **Healthy:** Normalbetrieb. Lese- und Schreibtransaktionen aktiv.
* **Degraded:** Vorübergehender Warnzustand (z. B. temporäre Verzögerung in Compaction oder Cache-Pressure). Schreibbetrieb bleibt möglich.
* **ReadOnly:** Datenträger voll (`ENOSPC`) oder Read-Only-Mount. Schreibtransaktionen werden mit Fehler abgelehnt, Lesezugriffe bleiben erlaubt.
* **Poisoned:** Unwiederbringlicher Fehlerzustand (Tritt ein bei jedem `fsync`-Fehler, WAL-HMAC-Korruption oder Checksummen-Fehler im Log).
* **Harte Regel:** **Nach einem `fsync`-Fehler wird NIEMALS weitergeschrieben.** Der Motor schaltet augenblicklich in den Zustand `Poisoned`. Alle nachfolgenden Schreibtransaktionen schlagen fehlschlagend (`fail-closed`) fehl.

---

## 5. Evidenz & Verifikationsstand

### 5.1 Garantien vs. Ausführungsbefehle

| Garantie | Verifikationsbefehl | Ist-Status in dieser Sitzung |
|---|---|---|
| **G1 (Crash/WAL-Atomarität)** | `cargo xtask wal-replay-verify` | NICHT NACHGEWIESEN (In dieser Sitzung nicht ausgeführt) |
| **G2 (Löschung/Krypto-Shredding)** | `cargo test -p contextra-crypto --locked` | NICHT NACHGEWIESEN (In dieser Sitzung nicht ausgeführt) |
| **G3 (Audit / DSGVO Art. 30)** | `cargo test -p contextra-audit-export --locked` | NICHT NACHGEWIESEN (In dieser Sitzung nicht ausgeführt) |
| **G4 (Tenant Isolation)** | `cargo test -p contextra-kvcache --locked` | NICHT NACHGEWIESEN (In mehrerem Scope nicht ausgeführt) |
| **Workspace Compilation** | `cargo check --workspace --all-targets` | teilweise (Exit-Code 101 am 2026-10-12 wegen `contextra-mcp` VOR und NACH Änderung identisch) |
| **Capabilities Consistency** | `cargo xtask check-ring-capabilities-consistency` | nachgewiesen (Exit-Code 0 am 2026-10-12) |
| **Veto Deadline Gate** | `cargo xtask check-vetoes` | nachgewiesen (Exit-Code 0 am 2026-10-12) |
| **Ring Layering Full** | `cargo xtask check-ring-layering-full` | nachgewiesen (Warning-Mode Exit-Code 0, 6 unallowlistete Verstöße dokumentiert in RING_DEBT.md am 2026-10-12) |

### 5.2 Evidenzleiter (E0 bis E6)

| Stufe | Verifikationsstufe | Ist-Stand | Nachweis / Befehl |
|---|---|---|---|
| **E0** | Kompilierbarkeit | teilweise | `cargo check --workspace --all-targets` (Exit 101 wegen pre-existing `contextra-mcp` Issue) |
| **E1** | Alle Test-Targets bestanden | unbekannt | In dieser Sitzung nicht ausgeführt |
| **E2** | Modelltest / Property-based Testing | unbekannt | In dieser Sitzung nicht ausgeführt |
| **E3** | Fuzzing mit Korpus | unbekannt | In dieser Sitzung nicht ausgeführt |
| **E4** | Miri / Loom am echten Code | unbekannt | In mehrerem Scope nicht ausgeführt |
| **E5** | Deterministische Crash-Simulation + Mutation + INV_MATRIX | unbekannt | In dieser Sitzung nicht ausgeführt |
| **E6** | Externe Prüfung (Third-Party Security Audit) | unbekannt | Ausstehend |
