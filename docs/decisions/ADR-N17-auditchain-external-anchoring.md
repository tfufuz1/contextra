# ADR-N17: External-Anchor-Pattern zur Absicherung der AuditChain gegen Backup-Rollback- und Replay-Angriffe

* **Status:** Proposed
* **Datum:** 2026-10-04
* **Kontext / Auslöser:**
  ADR-N12 (`docs/decisions/ADR-N12-wal-high-water-mark-verankerung.md`) adressiert den Schutz gegen Truncation- und Rollback-Angriffe ausschließlich für das Write-Ahead-Log (WAL) des LSM-Stores. In den Audit- und Compliance-Subsystemen wurden jedoch drei unbeschränkte, voneinander unabhängige Audit-Ketten identifiziert (Befund-Kampagne J-26, Äquivalent FIND J-26-F02).
  Ohne eine externe, monoton steigende Verankerung (External Anchor) ist ein Backup-Restore, das den On-Disk-Zustand auf einen früheren, intern mathematisch konsistenten Kettenkopf zurücksetzt, von dem echten aktuellen Zustand mathematisch ununterscheidbar. Keines der drei Audit-Systeme kann ein solches Rollback aus eigener Kraft isoliert erkennen.

---

## 1. Belegter Ist-Zustand

Im Codebase existieren drei kryptografische bzw. strukturierte Audit-Log-Mechanismen in separaten Rings/Crates:

1. **`AuditChain` (`crates/contextra-crypto/src/audit_chain.rs`):**
   Eine Blake3-Hash-Kette (`entries: Vec<AuditChainEntry>`) mit `head_hash` und optionalen Ed25519-Signaturen. Jeder Eintrag verkettet `prev_hash`.
2. **`ContextEditAuditRecord` (`crates/contextra-privacy/src/context_edit_audit.rs`):**
   Eine Blake3-Hash-Kette für Kontext-Änderungen am Agenten-Scratchpad (`previous_hash`, `record_hash`). Die Verifikation (`verify_audit_chain`) prüft ausschließlich die interne Konsistenz der Auffolge-Hashes.
3. **`AuditLog` / `AuditEntry` (`crates/contextra-agent/src/audit.rs`):**
   Unveränderliches Append-Only Step-Protokoll für Agenten-Workflow-Schritte, gespeichert unter Schlüsseln der Form `audit:{task_id}:step:{n}`.

### Schwachstelle
Alle drei Audit-Systeme arbeiten rein lokal und speichern ihren Zustand auf demselben physischen oder logischen Datenträger wie die Datenbank. Wenn ein Angreifer oder Administrator ein Dateisystem-Backup einspielt:
- Werden alle drei Ketten auf einen früheren Stand $t_0 < t_{\text{aktuell}}$ zurückgesetzt.
- Die internen Hash-Verkettungen ($\text{Hash}_n = \text{Blake3}(\text{Hash}_{n-1} \parallel \dots)$) bleiben innerhalb des eingespielten Backups vollständig gültig und mathematisch valide.
- Nachfolgende Audit-Einträge werden nahtlos an den alten Kettenkopf angefügt, wodurch eine valide Alternativhistorie entsteht.

Da kein externes, nicht-manipulierbares Monotonie-Signal existiert, ist dieser Replay-/Rollback-Angriff für Prüfer und automatische Validatoren unsichtbar.

---

## 2. Optionen

### Option A: Abstraktion über ein `ExternalAnchor`-Trait

Einführung eines systemweiten Ports/Traits zur externen Verankerung von Audit-Kettenköpfen:

```rust
pub trait ExternalAnchor: Send + Sync {
    /// Verankert den aktuellen Kettenkopf (head_hash) und Index extern.
    fn anchor(&self, head_hash: &[u8; 32], index: u64) -> Result<AnchorReceipt, CryptoError>;

    /// Verifiziert ein Anker-Rezept gegen das externe Medium.
    fn verify_anchor(
        &self,
        receipt: &AnchorReceipt,
        head_hash: &[u8; 32],
        index: u64,
    ) -> Result<bool, CryptoError>;
}
```

#### Hardware-Implementierung (Primär): TPM 2.0 Non-Volatile Index
- **Mechanismus:** Nutzung von `TPM2_NV_Increment` auf einem dedizierten TPM 2.0 NVRAM-Index.
- **Sicherheitsgarantie:** Der Zähler kann durch Hardware-Garantien niemals dekrementiert oder zurückgesetzt werden (monoton steigender Hardware-Zähler).
- **Verhalten bei Rollback:** Nach dem Einspielen eines Backups ist der im Backup gespeicherte Index $i_{\text{backup}}$ kleiner als der NV-Zählerstand im TPM $i_{\text{TPM}}$. Die Verifikation schlägt fehl.

#### Software-Fallback (Sekundär): Append-Only Datei auf separatem Speichermedium
- **Mechanismus (Linux):** Schreiben von Anker-Einträgen in eine gesonderte Protokolldatei auf einem entfernten oder getrennten Mount mit dem `chattr +a` (Append-Only) Attribut.
- **Plattform-Einschränkung (Wichtig):**
  Das `chattr +a` Flag ist **Linux/ext-Dateisystem-spezifisch** und steht unter Windows (NTFS / exFAT) oder MacOS nicht zur Verfügung.
- **Offene Frage für Windows/NTFS:**
  Für Windows-Umgebungen existiert im aktuellen Design kein direktes Äquivalent zu `chattr +a`. Mögliche Alternativen (wie NTFS-ACLs mit eingeschränkten `FILE_APPEND_DATA`-Rechten oder eine extern signierte Sequenzdatei) müssen separat untersucht und bewertet werden. Die konkrete Fallback-Mechanik für Windows bleibt eine offene Architekturfrage.

---

### Option B: Architektur der Anker-Zuordnung (Shared vs. Isolated Anchors)

Zur Koppelung der drei Audit-Ketten an das `ExternalAnchor`-System stehen zwei Ansätze zur Auswahl:

1. **Ein einziger gemeinsamer Anker (Single Shared External Anchor):**
   - Ein zentraler `ExternalAnchor` dient allen drei Audit-Ketten.
   - Jeder `anchor()` Aufruf übergibt eine eindeutige `anchor_id` (z.B. `"crypto_audit"`, `"privacy_context_edit"`, `"agent_workflow"`).
   - *Vorteil:* Minimaler Ressourcenbedarf im TPM (nur 1 NV-RAM Slot oder 1 Shared Lock) und einheitliche Konfiguration.
   - *Nachteil:* Mögliche Latch-Locking-Contention im TPM bei hochfrequenten parallelen Audit-Schreibvorgängen across Subsystemen.

2. **Drei unabhängig konfigurierbare Anker (Independent Anchors):**
   - Jedes der drei Subsysteme (`contextra-crypto`, `contextra-privacy`, `contextra-agent`) erhält seinen eigenen `ExternalAnchor`-Instanzkanal.
   - *Vorteil:* Vollständige Isolierung der Subsysteme; unterschiedliche Sicherheitsniveaus möglich (z.B. Hardware-TPM für `AuditChain`, Software-Fallback für Agent-Workflow).
   - *Nachteil:* Höhere TPM-NVRAM-Belegung (3 NV-Slots) und komplexere Initialisierung.

*Hinweis:* Die Wahl zwischen Option B.1 und B.2 wird in diesem ADR bewusst als offener Entscheidungsraum dokumentiert und nicht unilateral festgelegt.

---

## 3. Empfehlung

1. **Architektur-Muster:**
   Es wird empfohlen, das `ExternalAnchor`-Trait als zentralen Abstraktions-Port einzuführen, mit TPM 2.0 `TPM2_NV_Increment` als primärer High-Security-Implementierung und einer Append-Only-Datei als Sekundär-Fallback.

2. **Offene Punkte mit Sign-off-Pflicht durch den Chefarchitekten:**
   Vor einer Implementierung müssen die folgenden zwei Punkte explizit durch den menschlichen Architekten freigegeben werden:
   - **Plattform-Fallback für Windows:** Festlegung des konkreten Append-Only-Emulationsmechanismus unter Windows/NTFS (da `chattr +a` nicht anwendbar ist).
   - **Anker-Topologie:** Entscheidung über Single Shared Anchor (Option B.1) vs. Drei unabhängige Anker (Option B.2).

Aufgrund dieser offenen Architekturentscheidungen verbleibt dieses ADR im Status **Proposed**.
