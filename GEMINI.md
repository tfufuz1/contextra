# Contextra – Gemini & Antigravity Instruktionen (GEMINI.md)

> Diese Datei steuert das Verhalten von Google Antigravity (CLI, IDE und Desktop App) im Workspace `/home/freddy/Projekte/contextra`.

## 1. Primäre Richtlinie
Folge jederzeit den in [AGENTS.md](file:///home/freddy/Projekte/contextra/AGENTS.md) normierten Vorgaben, insbesondere:
1. **Ring-Hierarchie einhalten**: Ring 0 bis Ring 4. Keine zirkulären oder aufsteigenden Kanten.
2. **Zero-Panic einhalten**: Absolutes Verbot von `unwrap()`, `expect()`, `panic!()` und ungeprüftem Indexing in Produktionscode.
3. **Determinismus**: Zeit (`Clock`), Zufall (`Rng`), IDs (`IdGen`) strikt über Ports.
4. **Mandantensicherheit (VETO-F10)**: Keine mandantenübergreifenden Leaks.
5. **I/O nur über Vfs**: Keine direkten Aufrufe von `std::fs` oder `tokio::fs` im Kern.

## 2. Progressive Disclosure & Skills
Nutze die bereitgestellten Skills unter `.agents/skills/`:
- **`contextra-doctor`**: Führe dies aus bei Workspace-Inkonsistenzen, Cargo-Metadatenfehlern oder Compiler-Blockern.
- **`contextra-deletion-verifier`**: Führe dies aus bei Arbeiten am Löschpfad (Ebene A/B, ErasureSaga, Ed25519-Beweise).
- **`contextra-security-audit`**: Führe dies aus zur Auditierung von Verschlüsselung, KDF (Argon2id) und Mandanten-Isolation.
- **`contextra-invariants-gate`**: Führe dies aus zur Validierung der Architekturregeln und Deny-Lints.
- **`contextra-product-suite`**: Führe dies aus bei Release-Zuschnitt (Memory, Guard, Proof, Ingest).

## 3. Kommunikation & Verhalten
- Halte Antworten präzise, faktenbasiert und belegt nach dem Evidenz-Prinzip (INV-EVIDENCE-1).
- Setze vor Codeänderungen stets einen fehlschlagenden Test auf (Red-to-Green Workflow).
- Verlinke relevante Symbole und Dateien stets mit Markdown-Links (`[file](file:///...)`).
