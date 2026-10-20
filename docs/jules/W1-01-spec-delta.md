# Spec-Delta: SESSION W1-01 — Entropie-Fundament (contextra-ports / contextra-sys)

## 1. Übersicht & Ziel
Entropie-Fundament zur Absicherung kryptografischer Schlüssel-, Salz- und ID-Erzeugung gegen unzulässige oder geschwächte Zufallsquellen (Befunde F-01, F-23, X-02, B0-d, X-09; Invariante `INV-SEC-1`).

---

## 2. API- & Trait-Änderungen

### 2.1 `contextra-ports::rng`
- **`RngError`**: Neuer Fehler-Enum (`Unavailable(String)`, `SecretUnsupported`).
- **`EntropyRng`**: Erweitert um fehlbaren Vertrag `fn fill_secret(&self, dest: &mut [u8]) -> Result<(), RngError>` (Default: `Err(RngError::SecretUnsupported)`). Normativ dokumentiert: Nur echte OS/KMS-Entropiequellen überschreiben diese Methode.
- **`SystemRng`**:
  - `impl EntropyRng for SystemRng` überschreibt `fill_secret` **NICHT** (ruft Default und gibt `Err(SecretUnsupported)` zurück).
  - Mit Deprecation-Hinweis im Doc versehen (Entfall in W3-01 nach Migration aller Workspace-Aufrufer).

### 2.2 `contextra-ports::secret` (`SecretBytes<N>`)
- Neuer Typ `SecretBytes<N>(zeroize::Zeroizing<[u8; N]>)`.
- Bietet `generate(rng: &dyn EntropyRng) -> Result<Self, RngError>`, welches ausschließlich `rng.fill_secret` aufruft.
- Re-exportiert `expose(&self) -> &[u8; N]` und konstante Zeit-Gleichheit `ct_eq(&self, other: &Self) -> bool`.
- Redagierte `Debug`-Formatierung (`SecretBytes<16>((redacted))`), kein `Clone`, kein `Copy`, kein `Serialize`.

### 2.3 `contextra-ports::os_entropy` (`OsEntropyRng`)
- Echte OS-Entropiequelle basierend auf `getrandom`.
- Überschreibt `fill_secret` mit `getrandom::getrandom`.
- **Fail-Closed bei Quellenfehler**: Tritt bei Syscall-Fehler in permanenten Vergiftungszustand (`POISONED` Flag), protokolliert per `tracing::error!` und gibt `Err(RngError::Unavailable)` zurück.
- Legacy-Methoden `fill_bytes`/`next_u64` liefern bei Fehler Fallback-Entropie ohne Panics oder Nullbytes und sind ausdrücklich als nicht geheimnistauglich dokumentiert.

### 2.4 `contextra-sys::determinism`
- Struktur `SeededRng` in `SplitMix64` umbenannt.
- `pub type SeededRng = SplitMix64` als `#[deprecated]` Typalias erhalten.
- `SystemClock` und `SystemRng` in `contextra-sys` als `#[deprecated]` markiert.

---

## 3. Architektur-Entscheidung E-10
**E-10: Kanonische Port-Heimat und Konsolidierung von Ring 0 (`contextra-sys`) & Ring 1 (`contextra-ports`)**
- `contextra-ports` bleibt die kanonische Heimat der Port-Trait-Definitionen und standardmäßigen Port-Implementierungen (`SystemClock`, `SystemRng`, `OsEntropyRng`, `SecretBytes`).
- Ring 0 (`contextra-sys`) stellt untergeordnete, ungelinkte System-Primitive bereit (wie `SplitMix64`).
- Auf die Einführung einer zyklischen Abhängigkeit oder eines separaten Adapter-Crates zwischen Ring 0 und Ring 1 wird verzichtet, um die saubere Ring-DAG-Hierarchie beizubehalten.
- Die Pflege von `capabilities.toml` obliegt dem Maintainer.

---

## 4. Workspace-Migrationsliste (Aufrufer der veralteten `sys`/`ports` Typen)
Folgende Stellen nutzen veraltete Typen/Aliase und sind in Folgesessions (W2-02, W3-01) auf `contextra_ports` bzw. `SplitMix64` zu migrieren:
1. `crates/contextra-adapt/src/bandit.rs` (`use contextra_sys::SeededRng;`)
2. `crates/contextra-adapt/src/pid_latency_controller.rs` (`use contextra_ports::{Clock, SystemClock};`)
3. `crates/contextra-adapt/src/homeostat.rs` (`use contextra_ports::{Clock, SystemClock};`)
4. `crates/contextra-infer-ollama/src/client/core.rs` (`pub struct OllamaRng(pub contextra_sys::SeededRng);`)
5. `crates/contextra-crypto/src/wiring.rs` (`use contextra_ports::SystemClock;`)
6. `crates/contextra-testkit/src/dst/rng.rs` (`use contextra_ports::{EntropyRng as EntropyRngPort, SystemRng};`)
7. `crates/contextra-testkit/src/seeded_rng.rs` (`use contextra_ports::SeededRng as SeededRngPort;`)
8. `crates/contextra-sys/tests/system_hardening_aligned.rs` (`use contextra_sys::{SystemClock, SystemRng};`)
