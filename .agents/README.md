# Contextra – Agent & Antigravity Setup

Dieses Verzeichnis (`.agents/`) und die Root-Dateien (`AGENTS.md`, `GEMINI.md`) konfigurieren die KI-Entwicklungsumgebung für **Google Antigravity (CLI `agy`, IDE und Desktop App)** sowie verbundene Agenten (Claude, Jules).

---

## 1. Übersicht der Struktur

```
.
├── AGENTS.md                          # Normative Projektrichtlinien (für alle Agenten)
├── GEMINI.md                          # Antigravity/Gemini spezifische Einstiegsdatei
├── .antigravity/
│   └── settings.json                  # Workspace-spezifische Antigravity Settings
├── .gemini/
│   └── settings.json                  # Workspace-spezifische Gemini/Antigravity Settings
└── .agents/
    ├── README.md                      # Diese Dokumentation
    ├── settings.json                  # Workspace-Settings für Antigravity CLI
    ├── skills.json                    # Skills-Manifest für Antigravity
    ├── mcp_config.json                # Model Context Protocol (Contextra MCP Server)
    ├── hooks.json                     # Lifecycle Hooks (SessionStart etc.)
    ├── hooks/
    │   └── session_start.sh           # Hook-Skript zur Initialisierung
    ├── skills/                        # Spezialisierte Agenten-Skills
    │   ├── contextra-doctor/          # Diagnose & Reparatur (Cargo, B-01, xtask)
    │   ├── contextra-deletion-verifier/# Löschkette, Saga, Canary, Ed25519 Proofs
    │   ├── contextra-security-audit/  # KDF (Argon2id), Zeroize, Tenant-Isolation
    │   ├── contextra-invariants-gate/ # Ring-DAG, Zero-Panic, Unsafe-Inseln
    │   └── contextra-product-suite/   # EP1 Memory, EP6 Guard, EP2 Proof, EP7 Ingest
    ├── rules/                         # Modulare Governance-Regeln
    │   ├── deletion_pipeline.md       # Löschinvariante & Zweistufigkeit
    │   ├── zero_panic.md              # Zero-Panic & Error Handling
    │   ├── async-io.md                # Async-Reinheit in Ring 0
    │   ├── dependencies.md            # Crate- & Ring-Hierarchie
    │   ├── simd_safety.md             # Unsafe-Inseln & 64B-Alignment
    │   ├── testing.md                 # Deterministische Tests
    │   ├── test_quality.md            # Red-to-Green & Testqualität
    │   └── wal_crypto.md              # WAL-First & Krypto-Integrität
    └── governance/                    # Verfassung & Gate-Definitionen
```

---

## 2. Nutzung mit Antigravity CLI (`agy`)

1. **CLI starten**:
   Starte die CLI direkt im Repository-Wurzelverzeichnis:
   ```bash
   agy
   ```

2. **Skills nutzen**:
   Die Skills werden dank *Progressive Disclosure* automatisch erkannt. Sie können im Chat direkt angesprochen werden:
   - `/contextra-doctor`: Überprüft Workspace-Integrität, Manifeste und bekannte Compiler-Fehler.
   - `/contextra-deletion-verifier`: Führt durch die Verifikation der 6-Schichten-Löschpipeline und Canary-Tests.
   - `/contextra-security-audit`: Überprüft Sicherheits-Defaults (Argon2id, Verschlüsselung, TenantPolicy).
   - `/contextra-invariants-gate`: Prüft Ring-Hierarchie, Zero-Panic und Unsafe-Inseln.
   - `/contextra-product-suite`: Hilft beim Bauen und Testen der Sub-Produkte (Guard, Memory, Proof, Ingest).

3. **Status und Skills neu laden**:
   ```text
   /skills
   /skills reload
   ```

4. **MCP-Server testen**:
   Der projektinterne Contextra-MCP-Server ist in `.agents/mcp_config.json` hinterlegt und kann mit `agy` direkt aufgerufen werden.

---

## 3. Team-Zusammenarbeit via Git

Alle Konfigurationen, Skills, Hooks und Regeln in `.agents/` sind im Git versioniert. Jeder Entwickler, der das Repository klont, erhält sofort dieselben optimierten Richtlinien, Skills und Automatisierungen.
