# ADR-N10: Kein globaler veränderlicher Zustand

* **Status:** Beschlossen / Final (Prinzip P29, Gesamtspezifikation §3, §20.3)
* **Datum:** 2026-09-17
* **Kontext / Auslöser:**
  Veränderlicher globaler Zustand (`static mut`, global mutable `OnceLock`/`RwLock`/`Mutex` Singletons) erzeugt in hochgradig nebenläufigen, asynchronen Rust-Systemen schwere Architekturmängel:
  1. **Race Conditions in Tests:** Parallele Testausführungen (`cargo test`) beeinflussen sich gegenseitig über globale Singletons.
  2. **Verletzung der Mandantentrennung & Kapselung:** Globale Registries erschweren die saubere Isolation von Instanzen und Multi-Tenant-Grenzen.
  3. **Undefiniertes Shutdown-Verhalten:** Keine deterministische Lebensdauer-Steuerung von Ressourcen.

  Zur nachhaltigen Behebung dieser Risiken formuliert **Prinzip P29** das strikte Verbot von globalem veränderlichem Zustand im gesamten Workspace.

---

## 1. Das Architektur-Prinzip P29

> **P29 — Kein globaler veränderlicher Zustand.**
> `static OnceLock`/`Lazy` sind ausschließlich für **unveränderliche Konstanten** zulässig (z.B. vorkompilierte Regex-Muster, statische Stopwort-Listen, Standard-Konfigurations-Defaults).
> **Veränderlicher Zustand gehört ausnahmslos einer konkreten Instanz** (z.B. eingebracht via Dependency Injection, Struct-Felder oder Context-Handles).

---

## 2. Fallbeispiele & Refactoring-Analyse

### Fallbeispiel 1: `ORPHAN_REGISTRY` in `memfuse-checkpoint`
* **Problem-Analyse:**
  In `crates/memfuse-checkpoint/src/orphan.rs` existierte ein globaler statischer Singleton `static ORPHAN_REGISTRY: OnceLock<OrphanRegistry>`.
  In parallelen Unit-Tests (`cargo test`) führte der simultane Zugriff auf dieses Singleton zu sporadischen Flaky Tests und Lock-Kontention (dokumentiert als Race-Condition in `AUDIT_memfuse-checkpoint.md`).
* **Soll-Zustand / Refactoring:**
  Entfernung des globalen `ORPHAN_REGISTRY` Singletons. Die Waisen-Registrierung (`OrphanRegistry`) wird direkt als Instanzfeld in den `PersistentCheckpointStore` bzw. die jeweilige `StorageEngine`-Instanz eingebettet. Lebensdauer und State-Tracking sind somit strikt an die jeweilige Store-Instanz gebunden.

### Fallbeispiel 2: `CIPHER_INSTANCE` & Nonce-Counter in `memfuse-crypto` / Security
* **Problem-Analyse:**
  Entwürfe mit globalen `static CIPHER_INSTANCE` oder globalen RAM-basierten `AtomicU64`-Nonce-Zählern verstoßen ebenfalls gegen P29. Ein RAM-basierter Nonce-Zähler beginnt nach einem Prozess-Neustart wieder bei 0, was bei Wiederverwendung desselben Schlüssels zum kryptographischen Kollaps führen würde.
* **Soll-Zustand / Refactoring:**
  Schlüssel- und Cipher-Manager werden per Dependency Injection über Instanzen (`KeyManager`) verwaltet. Cryptographic Nonces nutzen `OsRng` bzw. persistierte Hochwasserstände anstelle RAM-globaler Zähler.

---

## 3. Konsequenzen

* **Erzwingung bei Reviews:** Jedes PR-Code-Review prüft neue `static`-Variablen streng auf Unveränderlichkeit.
* **Test-Isolation:** Sämtliche Unit- und Integrationstests laufen vollständig isoliert und ohne gegenseitige Beeinflussung ab.
* **Dependency Injection:** Komponenten nehmen benötigte Registries oder Manager explizit über Konstruktoren (`new(registry: Arc<...>)`) entgegen.
