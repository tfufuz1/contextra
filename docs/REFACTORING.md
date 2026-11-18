# Contextra — Normativer Refactoring-Plan

**Version:** 1.0  
**Datum:** 18. November 2026  
**Status:** Normativ — zu befolgen in der angegebenen Reihenfolge  
**Referenz:** `docs/SPEC.md` v2.0

---

## Überblick der kritischen Befunde und ihre Auflösung

Dieser Plan beschreibt die Reihenfolge und den Inhalt der Bereinigungsarbeiten. Die Reihenfolge ist normativ: Spätere Phasen setzen die Ergebnisse früherer voraus.

---

## Phase 0 — Altlast-Dokumente löschen

Folgende Dateien und Verzeichnisse enthalten widersprüchliche, veraltete oder falsche Informationen und müssen vor Beginn der Code-Arbeit gelöscht werden:

```bash
rm -rf reports/
rm -f ALTLASTEN-AUDIT.md
rm -f LEGACY_CLEANUP_SPEC.md
rm -f ANALYSIS-E0-CONTRADICTIONS.md
rm -rf docs/diagnostics/
rm -f tools/gate_layering.py
rm -f tests/layering.rs
find crates/ -type d -name decoupling -exec rm -rf {} +
find crates/ -name capabilities.toml -exec rm -f {} +
```

Nach dieser Phase: Einzige normative Referenz ist `docs/SPEC.md` v2.0 und `docs/crates/*.md`.

---

## Phase 1 — Workspace-Lints aktivieren (Blocking Gate)

**Grund:** Ohne aktive Workspace-Lints können R6-Verstöße nicht systematisch gefunden werden. Die Lints müssen zuerst aktiv sein.

**Aufgaben:**

1. Root `Cargo.toml` ergänzen:

```toml
[workspace.lints.rust]
unsafe_code = "forbid"

[workspace.lints.clippy]
unwrap_used     = "deny"
expect_used     = "deny"
panic           = "deny"
todo            = "deny"
unimplemented   = "deny"
await_holding_lock   = "deny"
indexing_slicing     = "deny"
```

2. Alle Crate-`Cargo.toml` ergänzen: `[lints] workspace = true`.
3. `cargo clippy --workspace -- -D warnings` ausführen, Ergebnis als Baseline dokumentieren.

**Fertig wenn:** `cargo clippy --workspace -- -D warnings` kompiliert (auch wenn viele Fehler kommen — die werden in Phase 3 behoben).

---

## Phase 2 — Kritische Typ-Korrekturen (Blocking)

Diese Fixes sind Voraussetzung für alles andere.

### 2.1 `SubjectRef` und `PersonalRef` in `contextra-ids` (HÖCHSTE PRIORITÄT)

**Befund:** `SubjectRef` ist `Vec<u8>` mit `Clone`, ohne `ZeroizeOnDrop`. Das untergräbt die P-Klassen-Garantie.

**Fix:**

```rust
// crates/contextra-ids/src/subject_ref.rs

use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Zeroize, ZeroizeOnDrop)]
pub struct SubjectRef([u8; 16]);

impl SubjectRef {
    pub fn generate() -> Result<Self, IdError> {
        use rand::RngCore;
        let mut bytes = [0u8; 16];
        rand::rngs::OsRng.try_fill_bytes(&mut bytes)
            .map_err(|e| IdError::EntropyExhaustion(e.to_string()))?;
        Ok(Self(bytes))
    }

    pub const fn from_bytes(bytes: [u8; 16]) -> Self { Self(bytes) }
    pub fn as_bytes(&self) -> &[u8; 16] { &self.0 }
}

// Kein Clone, kein Debug mit Inhalt, kein Serialize
impl core::fmt::Debug for SubjectRef {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "SubjectRef([REDACTED])")
    }
}
impl core::fmt::Display for SubjectRef {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "SubjectRef([REDACTED])")
    }
}
```

Gleiche Behandlung für `PersonalRef` (ohne `ZeroizeOnDrop`, aber ohne `Clone`).

### 2.2 `EncryptionConfig` und `SecretString` in `contextra-types`

**Befund:** `encryption_passphrase: Option<String>` in `contextra-db` ist nicht zeroisiert.

**Fix:**

```rust
// crates/contextra-types/src/config.rs

use zeroize::Zeroize;

pub struct SecretString(String);

impl SecretString {
    pub fn new(s: String) -> Self { Self(s) }
    pub fn as_str(&self) -> &str { &self.0 }
}

impl Drop for SecretString {
    fn drop(&mut self) { self.0.zeroize(); }
}
// Kein Debug, kein Clone, kein Serialize

pub enum EncryptionConfig {
    None,
    Required { passphrase: SecretString },
}
```

`contextra-db` und `contextra` müssen `encryption_passphrase: Option<String>` durch `EncryptionConfig` ersetzen.

### 2.3 Kanonisierung `SagaState::KeyRevoked`

Globale Suche-und-Ersetze: `KeyDestroyed` → `KeyRevoked` in allen Quelldateien.

---

## Phase 3 — Ebenenbereinigung (Abhängigkeiten, R1–R3)

### 3.1 E0-Crates: Keine E1/E2-Abhängigkeiten

Betroffen: `erasure`, `privacy`, `simd`

**Für `contextra-erasure`:**
- `aes-gcm-siv`, `sha2`, `hmac` aus `[dependencies]` entfernen
- Alle Krypto-Operationen aus `erasure` nach `contextra-crypto` verschieben
- `erasure` ruft `crypto` nicht direkt auf — Vault-Slot-Management nutzt nur `zeroize`

**Für alle E1-Crates (`store`, `vector`, `text`, `graph`, `crypto`, `mvcc`, `rank`, `checkpoint`):**
- `contextra-types` aus `[dependencies]` entfernen (E1 → E2 verboten)
- `contextra-ports` aus `[dependencies]` entfernen (E1 → E2 verboten)
- Features `workspace`, `types-compat`, `ports-compat` entfernen
- `contextra-testkit` aus `[dev-dependencies]` entfernen (R2: dev-dep auf höhere Ebene verboten)

### 3.2 `default = []` für alle Crates (R3)

Folgende bekannte Verstöße beheben:
- `contextra-crypto`: `default = ["kv-encryption"]` → `default = []`
- `contextra-erasure`: `default = ["signing", "vault", "async"]` → `default = []`
- `contextra-graph`: alle Experimentalfeatures aus `default` entfernen

### 3.3 `contextra-ports` bereinigen

- `src/erasure.rs` löschen: `ErasureLayer` ist nicht in `ports` definiert
- `src/embedding.rs` bereinigen: Referenzen auf `infer-candle` und `infer-ollama` entfernen

### 3.4 Typ-Dubletten eliminieren (R4)

| Duplikat | In Crate | Auflösung |
|----------|----------|-----------|
| `ErasureId`, `GateId` | `contextra-types`, `contextra-erasure` | Kanonisch in `contextra-ids`; alle anderen löschen |
| `EraseTarget`, `SagaState` | `contextra-types` (`src/erasure.rs`) | Datei löschen, re-export aus `contextra-erasure` |
| `LayerWitness`, `Scanner` | `contextra-ports` | Löschen, re-export aus `contextra-erasure` |
| `TenantId`, `Clock` | `contextra-privacy` (`src/types.rs`) | Lokale Kopien löschen, aus `contextra-ids`/`contextra-env` importieren |
| Krypto-Primitives | `contextra-erasure` | Entfernen, `contextra-crypto` nutzen |

---

## Phase 4 — Zero-Panic-Policy (R6)

Nach Phase 1 (Lints aktiv) und Phase 3 (Dependencies bereinigt):

1. `cargo clippy --workspace -- -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic` ausführen
2. Alle Fundstellen in Nicht-Test-Code systematisch beheben:
   - `unwrap()` → `?` oder `map_err(...)?`
   - `expect("msg")` → `ok_or(Error::...)? `
   - `panic!("msg")` → `return Err(Error::...)`
3. Testcode (`#[cfg(test)]`) ist ausgenommen

**Schätzung:** >1.500 Vorkommen, davon ein Teil in Testcode. Systematischer Ansatz:

```bash
# Fundstellen in Nicht-Test-Code finden
grep -rn "\.unwrap()\|\.expect(\|panic!(\|todo!(\|unimplemented!(" \
  crates/*/src/ --include="*.rs" | grep -v "#\[cfg(test)\]" | grep -v "mod tests"
```

---

## Phase 5 — Löschketten-Härtung

### 5.1 Purge-Pfad öffentlich machen (Befund 1.3)

**Befund:** Über die öffentliche Fassade gibt es keinen Compact- oder Purge-Pfad. T-DEL-1 schlägt fehl, weil SST-Dateien nicht bereinigt werden.

**Fix:**
- `contextra-erasure::SagaMachine::run_purge_window()` muss von `contextra-db` nach einem Gate-Setzen aufgerufen werden können
- `contextra-db` exponiert `trigger_purge()` intern, das `run_purge_window()` anstößt
- Alle Integrationstests für T-DEL-1 bis T-DEL-15 müssen `trigger_purge()` aufrufen

### 5.2 ReadGate-Persistenz beim Neustart

**Fix in `contextra-erasure::SagaMachine::open()`:**

```rust
pub async fn open(journal_path: &Path, gate: Arc<dyn ReadGate>) -> Result<Self, ErasureError> {
    let open_sagas = journal.recover_open_sagas().await?;
    for saga in &open_sagas {
        if saga.state.blocks_reads() {
            gate.activate(saga.target_key_range());
        }
    }
    // ...
}
```

### 5.3 Argon2-Parameter (Befund 2.3)

**Fix in `contextra-crypto::kdf::derive_master_key`:**

```rust
let params = Params::new(
    256 * 1024, // 256 MB — normativ gemäß SPEC §4.4
    4,          // t=4
    1,          // p=1
    Some(32),
)?;
```

Keine OWASP-Mindestparameter als Default erlaubt. Validierung DARF nur prüfen, ob die Konfiguration mindestens den Spec-Parametern entspricht.

### 5.4 SSTable-Scrubbing koppeln

**Fix in `contextra-store::compaction`:**

```rust
// Nach dem Schreiben neuer SSTables:
for old_sst in replaced_tables {
    contextra_durable_fs::scrub_file(&old_sst.path)?;  // Nullen
    std::fs::remove_file(&old_sst.path)?;               // Dann löschen
}
```

---

## Phase 6 — Layering-Gate (Ersatz für `gate_layering.py`)

Das defekte Python-Skript wird durch einen Rust-Integration-Test ersetzt.

Datei: `crates/contextra-testkit/tests/layering.rs`

```rust
#[test]
fn layering_gate_r1_r2_r3() {
    // cargo metadata auswerten
    // Für jedes Crate:
    //   - default = [] prüfen (R3)
    //   - Ebene aus package.metadata.contextra.ring lesen
    //   - Für E0/E1: keine Abhängigkeit auf E2/E3 (auch nicht über feature-Unifikation)
    //   - Für E1 dev-deps: keine E3-Crates
    //   - Kein Crate aus der Entfernt-Liste als workspace-Member aktiv
    //   - [lints] workspace = true gesetzt
}
```

Dieser Test läuft in CI als erster Schritt.

---

## Phase 7 — README und Dokumentation bereinigen

**Befund 2.4:** README beschreibt entfernte Features (Agent-Ausführung, PDF/Word-Ingest, ONNX, Ollama, Sandbox, Python-Paket).

**Befund 1.5:** README verspricht Art.-28-DSGVO-Generator — der nicht existiert.

**Befund 1.6:** README behauptet 100k-Skalierung — widerlegt durch Bench.

**Fix:**
- README vollständig neu schreiben: Nur beschreiben, was tatsächlich existiert und funktioniert
- Keine Compliance-Aussagen (INV-7)
- Keine Skalierungsbehauptungen ohne aktuellen, erfolgreichen Bench-Lauf
- Bench-Ergebnisse: belegbare Skalierung ist 2.000 Dokumente (Stand: letzter erfolgreicher Lauf)

---

## Abschlusskriterien (Definition of Done)

Das Refactoring ist abgeschlossen, wenn:

- [ ] `cargo clippy --workspace -- -D warnings` fehlerfrei
- [ ] `cargo test --workspace` fehlerfrei
- [ ] Layering-Gate-Test (`tests/layering.rs`) grün
- [ ] T-DEL-1 bis T-DEL-15 alle grün
- [ ] Kein Vorkommen von `KeyDestroyed` im Code
- [ ] Kein Vorkommen von `encryption_passphrase: Option<String>` im Code
- [ ] `SubjectRef` ist `[u8; 16]` mit `ZeroizeOnDrop`, ohne `Clone`
- [ ] `aes-gcm-siv` / `sha2` / `hmac` sind nicht in `contextra-erasure`'s `Cargo.toml`
- [ ] Keine E1-Crate hat `contextra-types` oder `contextra-ports` in `[dependencies]`
- [ ] Alle Crates haben `default = []`
- [ ] README beschreibt keine entfernten Features
- [ ] Kein Crate aus der Entfernt-Liste (SPEC §2.2) ist Workspace-Member
