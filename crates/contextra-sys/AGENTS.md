# AGENTS.md — `contextra-sys`

## 1. Rolle & Zweck (Baukasten-Prinzip)
`contextra-sys` kapselt alle hardware- und betriebssystemnahen Primitiven (POSIX Memory-Mapping via `mmap`, Prozess-Härtung, Memory-Locking via `mlock`) in einer isolierten, sicheren Fassade (Ring 0).
Als eine von genau drei genehmigten Unsafe-Inseln im Workspace isoliert es unsichere Systemaufrufe vollständig vom Rest des Systems und exportiert ausschließlich typsichere, panic-freie Rust-Abstraktionen.

## 2. Ring, Klasse, Reifegrad & `may_use`
- **Ring:** Ring 0 (Systemnahe Unsafe-Insel)
- **Klasse:** Kern `[K]`
- **Reifegrad:** `stable`
- **`may_use` (Obergrenze):** `contextra-types` (Spec §6.9, `capabilities.toml`).

## 3. Garantien & Architektur-Invarianten
- **REQ-SYS-001:** Sichere Wrapper um OS-Level Mmap und Prozess-Härtung (`harden_process`).
- **Unsafe-Disziplin (Contract V-SYS):** Rohe Deskriptor- und Memory-Mapping-Funktionen (`mmap_readonly`, `reopen_and_dup2`, `mem_lock`) sind strikt als `pub unsafe fn` deklariert und tragen obligatorische `# Safety`-Abschnitte.
- **Jeder `unsafe`-Block** im Crate besitzt einen expliziten, beweisbaren `// SAFETY:` Kommentar.
- **Fail-Closed Fallback:** Sauberes Fehler-Mapping auf Nicht-Linux-Plattformen (`io::ErrorKind::Unsupported`) ohne Panics.
- **Durability-Propagation:** Durability- und Sync-Fehler (`rename_durable`) werden strikt mit `?` an den Aufrufer propagiert.

## 4. Verbotene Muster
- Exponieren ungeschützter roher Pointer oder Dateideskriptoren an höhere Schichten.
- `unsafe`-Blöcke ohne `// SAFETY:` Begründung.
- Unkontrolliertes `unwrap()` oder `panic!` bei OS-Fehlern.
- Vermischung von OS-Primitiven mit höherer Domänen- oder Speicherlogik.

## 5. Schnittstellen & Datenfluss (Ports & Abstraktionen)
- Stellt speichergepufferte und versiegelte Mmap-Primitiven (`SealedMap`, `mmap_readonly`) für `contextra-store`, `contextra-durable-fs` und `contextra-vector` bereit.

## 6. Tests, Sabotage & Qualitätssicherung
- **Miri-Gate:** Miri-Verifikation für Memory-Mapping-Pointer, Aligment und Pointer-Aliasing.
- **Cross-Platform Tests:** Validierung des Fail-Closed Verhaltens in emulierten Nicht-Linux Umgebungen.
- **Sabotage-Testing:** Gezielte Fehlerinjektion bei Systemaufrufen zur Prüfung der Fehlerbehandlung.

## 7. Offene Punkte & Technische Schulden
- Keine.
