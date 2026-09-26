# Contextra-Jules VM Environment Specification (v1.0)

**Stand:** 2026-03-06
**Zielgruppe:** Google-Jules Autonome Agenten & Contextra Core Maintainer
**Geltungsbereich:** VM-Infrastruktur, Tooling, Laufzeitumgebung & Session-Persistenz für hochfrequente Agenten-Sitzungen (bis zu 165 Sitzungen pro Tag).

---

## 1. System- & Hardware-Topologie

| Parameter | Spezifikation | Bemerkung |
| :--- | :--- | :--- |
| **Betriebssystem** | Ubuntu 24.04.4 LTS (Noble Numbat) | Linux Kernel 6.8.0 x86_64 PREEMPT_DYNAMIC |
| **Prozessor (CPU)** | Intel(R) Xeon(R) Processor @ 2.30GHz (4 vCPUs) | NUMA Node 0 (CPUs 0-3) |
| **Arbeitsspeicher (RAM)** | 7.8 GiB total (~7.4 GiB verfügbar) | Swappiness: 0 (kein Swap zugewiesen) |
| **Speicherplatz (Disk)** | 98 GiB OverlayFS (`/rom/overlay` auf `/`) | ~93 GiB verfügbar für Build-Artefakte (`/target`) |
| **Arbeitsverzeichnis** | `/app` | Symlink / Root-Arbeitsverzeichnis für Contextra Workspace |

---

## 2. Verzeichnis- & Daten-Layout der VM

Die VM von Google-Jules ist strukturiert, um isolierte Ausführungen und schnelles Caching zu ermöglichen:

```
/
├── app/                          <-- Contextra Workspace Root (Git Checkout)
│   ├── .cargo/                   <-- Lokale Cargo-Konfiguration (Job-Limits, Target-Dir)
│   ├── .githooks/                <-- Git Pre-Commit & Commit-Msg Hooks
│   ├── .github/                  <-- Workflows & Copilot/Jules Directives
│   ├── .jules/                   <-- Jules-spezifische Konfigurationen & Setup-Skripte
│   │   ├── setup/
│   │   │   └── environment_script.sh <-- Offizielles VM-Initialisierungsskript
│   │   ├── JULES_CONTEXT.md      <-- Dynamischer Kontext-State
│   │   ├── JULES_LOG.md          <-- Audit- und Sitzungs-Logbuch
│   │   └── SESSION_BOOTSTRAP.md  <-- Protokoll für Sitzungs-Handshakes
│   ├── crates/                   <-- 31 Rust Workspace Crates (Ring 0 bis Ring 3)
│   ├── xtask/                    <-- Gate-System & Quality Enforcement Tooling
│   ├── target/                   <-- In-Memory / Ephemeres Build-Verzeichnis
│   └── verify_workspace.sh       <-- Master-Diagnose- & Verifikationsskript
├── home/jules/
│   ├── .cargo/bin/               <-- Rust-Binaries (rustc, cargo, rustup, just, nextest)
│   ├── .nvm/versions/node/       <-- Node.js v22.22.1 LTS & global packages (npm, pnpm, yarn)
│   ├── .pyenv/versions/          <-- Python 3.12.13 Standalone Environment
│   └── self_created_tools/       <-- Selbst erstellte Python-Werkzeuge für Jules
└── usr/local/bun/bin/bun         <-- Bun v1.2.14 Runtime
```

---

## 3. Technische Abhängigkeiten & Tooling-Katalog

### 3.1 System-Bibliotheken & Compiler (Apt / System)
* **GCC / G++:** v13.3.0 (`/usr/bin/gcc`, `/usr/bin/g++`)
* **Clang:** v18.1.3 (`/usr/bin/clang`)
* **Build Systems:** CMake 3.28.3, GNU Make 4.3, Ninja 1.11.1
* **FlatBuffers Compiler (`flatc`):** v24.3+ (Notwendig für FlatBuffers IPC Schema Compilierung in `contextra-wire`)
* **Tauri GUI System-Libs (Apt):**
  * `libwebkit2gtk-4.1-dev`
  * `libgtk-3-dev`
  * `libayatana-appindicator3-dev`
  * `librsvg2-dev`
  * `libssl-dev`
  * `pkg-config`

### 3.2 Rust Toolchain & Cargo Utilities
* **Rust Toolchain:** Rust 1.89.0 (`1.89.0-x86_64-unknown-linux-gnu`)
* **Profile & Components:** `minimal` Profile + `clippy` + `rustfmt` + `miri`
* **Cargo Subcommands (In `/home/jules/.cargo/bin/` zu installieren):**
  * `just` (Task Runner)
  * `cargo-nextest` (Hochparallele Test-Ausführung)
  * `cargo-llvm-cov` / `cargo-tarpaulin` (Testabdeckungsmessung)
  * `cargo-audit` / `cargo-deny` (Sicherheits- & Lizenzprüfung)
  * `ast-grep` (`sg`) (AST-basierte statische Code-Analyse)

### 3.3 Multi-Language Runtimes
* **Node.js Ecosystem:** Node v22.22.1, npm v11.11.0, pnpm v10.30.3, yarn v1.22.22
* **Bun:** v1.2.14
* **Python:** v3.12.13 (Pyenv managed), pip v26.0.1, NumPy, PyO3 Bindings Support

---

## 4. Multi-Session Strategie & High-Frequency Persistence (165 Sitzungen / Tag)

Um bis zu **165 Sitzungen pro Tag** effizient abzuwickeln, gelten folgende Richtlinien für die VM-Umgebung:

1. **State Isolation & Handshake:**
   * Jede Sitzung startet in einer frischen ephemeren VM-Instanz oder einem zurückgesetzten OverlayFS.
   * Das Ausführen von `.jules/setup/environment_script.sh` zu Sitzungsbeginn garantiert, dass innerhalb von < 10 Sekunden alle System-Bibliotheken und Binaries einsatzbereit sind.
2. **Incremental Target Warmup:**
   * `/app/target` speichert kompilierte Abhängigkeiten vor. Durch `--exclude contextra-tauri` bei Routine-Checks werden teure C++ GTK/WebKit Builds vermieden. <!-- crate-ref-ignore -->
3. **Deterministic xtask Gate Enforcement:**
   * Das xtask-System (`cargo run --manifest-path xtask/Cargo.toml -- <cmd>`) dient als alleinige Wahrheit bezüglich Quality Gates.
   * Keine Annahmen aus früheren Sitzungen übernehmen; stets den aktuellen Stand über `jules-preflight` prüfen.
