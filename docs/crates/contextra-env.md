# contextra-env — Crate-Spezifikation (Umgebungs- & Systemabstraktionen)

**Ebene:** E0 (Autarkes Fundament — Kern-Closure leer)  
**Status:** Normativ  
**Version:** 1.0  
**Geltungsbereich:** `crates/contextra-env`  
**Referenz:** `docs/SPEC.md` §2, §5

---

## 1. Zweck & Systemrolle

`contextra-env` abstrahiert alle äußeren Umgebungsfaktoren des Host-Systems, um vollständigen Determinismus, Testbarkeit und Unabhängigkeit von physischer Hardware zu gewährleisten:
- **Zeitquellen-Abstraktion (`Clock`):** Entkopplung von unkontrollierbarer Wall-Clock-Zeit (`std::time::Instant`, `SystemTime`) zur Ermöglichung deterministischer Zeitreise- und Ablauf-Tests.
- **Virtuelles Dateisystem (`Vfs`):** Abstraktion von Dateisystem-Operationen (In-Memory VFS für isolierte Unit-Tests, Disk VFS für Produktion).
- **Metriken- & Telemetriesenke (`MetricsSink`):** Abstraktion zur Erfassung von Latenzen, I/O-Zählern und Speicherauslastung ohne externe Netzwerk-Abhängigkeiten.

---

## 2. Kern-Abstraktionen

### 2.1 Zeitabstraktion (`Clock`)
SPEC §12 und Z-T1 fordern vollständigen Determinismus. Deshalb darf in Nicht-Test-Code außerhalb von `contextra-env` kein direkter Aufruf von `Instant::now()` oder `SystemTime::now()` erfolgen.

```rust
pub trait Clock: Send + Sync {
    /// Liefert die aktuelle Zeit in Nanosekunden seit Unix-Epoche.
    fn now_nanos(&self) -> u64;

    /// Liefert die aktuelle Zeit in Millisekunden seit Unix-Epoche.
    fn now_millis(&self) -> u64 {
        self.now_nanos() / 1_000_000
    }
}

/// Produktions-Implementierung: Greift auf OS-Systemzeit zu.
pub struct SystemClock;
impl Clock for SystemClock {
    fn now_nanos(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
    }
}

/// Test-Implementierung: Deterministisch manuell fortschreitende Uhr.
pub struct ManualClock {
    current_nanos: std::sync::atomic::AtomicU64,
}
impl ManualClock {
    pub fn new(initial_nanos: u64) -> Self;
    pub fn advance_millis(&self, ms: u64);
    pub fn set_nanos(&self, nanos: u64);
}
```

### 2.2 Virtuelles Dateisystem (`Vfs`)
Ermöglicht das Ausführen komplexer Crash-, Flush- und Recovery-Tests vollständig im RAM:
```rust
pub trait Vfs: Send + Sync {
    fn open_read(&self, path: &Path) -> Result<Box<dyn ReadSeek>, std::io::Error>;
    fn open_write(&self, path: &Path) -> Result<Box<dyn WriteSync>, std::io::Error>;
    fn create_dir_all(&self, path: &Path) -> Result<(), std::io::Error>;
    fn remove_file(&self, path: &Path) -> Result<(), std::io::Error>;
    fn rename(&self, from: &Path, to: &Path) -> Result<(), std::io::Error>;
    fn exists(&self, path: &Path) -> bool;
}
```

---

## 3. Invarianten

1. **`#![forbid(unsafe_code)]`:** Vollständig sicher.
2. **Kern-Closure leer:** Keine internen Abhängigkeiten im Workspace.
3. **Keine globalen statischen Zustände:** Uhren und VFS-Instanzen werden stets als injizierbare `Arc<dyn Clock>` bzw. `Arc<dyn Vfs>` übergeben.
