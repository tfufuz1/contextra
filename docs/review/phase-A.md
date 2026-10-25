# Phase A: Spezifikationsanalyse (CONTEXTRA SPEZIFIKATION 1.3)

## 1. Übersicht und Invarianten-Inventar (68 INV-Invariants)
Die Spezifikation v1.3 fordert 68 strikte Invarianten (INV-*), verteilt über Kerneigenschaften wie Ring-Isolation, Determinismus, Multi-Tenancy, Zero-Panic, SSI-Transaktionen, Löschbeweise und Formatversionierung.

### Kern-Invarianten im Detail (§27.6 Enforcement Matrix)
1. **INV-CORE-ISOLATION-1**: Ring 0/1 Crates dürfen keine optionalen Rand- oder Experiment-Crates einbinden.
2. **INV-NONDET-1**: Nichtdeterministische Quellen (LLM, Embedder, Zufall, Systemuhr) werden als Werte im WAL aufgezeichnet; Replay/Recovery ruft diese Quellen nie erneut auf.
3. **INV-WASM-DET-1**: WASM-Module liefern auf x86_64 und aarch64 bit-identische Ergebnisse.
4. **INV-TENANT-1**: Absolute Mandantentrennung auf Speicherebene, Indizes, Caches, Metriken und Auditing.
5. **INV-TENANT-IDX-1**: Vektor- und Textindizes isolieren Mandanten strikt (keine Datenlecks bei gemeinsamen Aggregaten/IDFs).
6. **INV-KEYSPACE-1**: Schlüsselräume sind strikt je Mandant getrennt.
7. **INV-ATOMIC-IDX-1**: Null-Divergenz zwischen Primärdaten und Indizes nach Crash-Recovery; WAL-Rebuild ist atomar.
8. **INV-WAL-SSOT-1**: Das WAL ist die Single Source of Truth für Zustandsübergänge.
9. **INV-INDEX-APPLY-1**: Index-Anwendung aus dem WAL ist deterministisch.
10. **INV-PROOF-COVERAGE-1**: Alle Löschschichten müssen im DeletionProof als belegt oder explizit unvollständig klassifiziert sein.
11. **INV-WITNESS-MINT-1**: `LayerWitness` darf nur aus den befugten Speicherschichten (`store`/`crypto`) ausgestellt werden.
12. **INV-PROOF-KEY-1**: Der Signierschlüssel des Beweises entstammt exklusiv dem `KeyManager` via domänengetrennter Ableitung.
13. **INV-RESTORE-HORIZON-1**: Restore verweigert fail-closed das Einspielen von Checkpoints/Backups, die älter als der Löschhorizont erfasster Keys/Tombstones sind.
14. **INV-CLAIMS-1**: Keine öffentlich beworbenen Sicherheitsclaims ohne wirksames automatisches Gate im CI-Pfad.
15. **INV-EVIDENCE-1**: Audit-Logs und Beweisketten sind kryptografisch verkettet und fälschungssicher.

---

## 2. Anforderungsinventar (167 REQ-Einträge)
- **REQ-DUR-001 bis REQ-DUR-025**: Durabilität, WAL-Protokollierung, Atomizität, fsync-Regeln, Crash-Consistency, Recovery B0–B9.
- **REQ-SEC-001 bis REQ-SEC-030**: Mandantentrennung, Key-Management, AES-GCM-SIV, Zeroize, Constant-Time Comparisons, Egress Guarding, Prompt-Injection-Pipeline.
- **REQ-DEL-001 bis REQ-DEL-035**: Löschpfade, Purge-Filter, WAL-Scrub, Anti-Resurrection-Ledger, LayerWitness, DeletionProof Typestate, Proof Policy.
- **REQ-SBX-001 bis REQ-SBX-015**: WASI Preview 1 Sandbox, Fuel Metering, Memory Capping, Deterministische PRNGs, Zero-Clock.
- **REQ-TX-001 bis REQ-TX-020**: SSI (Serializable Snapshot Isolation), ReadSet/WriteSet-Validierung, Lock-Free / Ordered Locking (`INV-LOCK-ORDER-1`).
- **REQ-PERF-001 bis REQ-PERF-020**: Latenzbudgets, Memory-Claiming, SQ8 Quantisierung, SIMD-Kernel Dispatch.
- **REQ-GOV-001 bis REQ-GOV-022**: Gates, Ratchet-Schwellenwerte, Proof-Claims-Validation, Anti-Collusion-Orakel.

---

## 3. Architektur- & Abhängigkeitsregeln
- **Ring-Modell (§6)**: Ring 0 (Fundament: `types`, `ports`, `simd`, `sys`, `wire`, `vector`, `text`), Ring 1 (`store`, `crypto`, `checkpoint`), Ring 2 (`engine`, `db`, `graph`, `cognition`), Ring 3 (`agent`, `mcp`, `audit-export`), Ring 4 (`contextra`).
- **Kanten- & Isolation-Regeln**: Strenge DAG-Struktur (Einrichtung nur von höherem Ring auf niederen Ring). Keine Zyklen. Strikte Kern-Geschlossenheit (`INV-CORE-ISOLATION-1`): Ring 0/1 dürfen keine optionalen `[R]` (Rand) oder `[E]` (Experimentell) Module importieren.
- **Unsafe-Inseln**: Punktgenau drei erlaubte Unsafe-Inseln im Workspace (`simd`, `sys`, `wire`). Außerhalb dieser drei Crates ist `unsafe` im gesamten Workspace durch Compiler-Flags / Lints verboten.

---

## 4. Spezifikationsinterne Widersprüche & Antinomien

Bei der tiefgehenden, vergleichenden Textanalyse der Spezifikation v1.3 wurden folgende **normative Widersprüche** identifiziert:

### Widerspruch 1: Bit-Identischer Rebuild vs. SIMD Float-Toleranzen (§26 vs. §9.6 / INV-NONDET-1)
- **Problem**: In §5 (Determinisimus) und INV-NONDET-1 wird gefordert: *"Gleicher Anfangszustand + gleiche WAL-Folge ⇒ bit-identischer Zustand. Index-Rebuild aus dem WAL ist bit-identisch."*
- **Gegen-Regel in §9.6**: Bei SIMD-Ausführung (AVX-512 vs AVX2 vs NEON vs Skalar) wird für Gleitkomma-Vektorrechnungen vereinbart: *"SIMD-Dispatch ... skalar ist Orakel (Bit-/ULP-Toleranz getestet)."*
- **Spezifikationskonflikt**: Gleitkomma-Summationsreihenfolgen bei Fused-Multiply-Add (FMA) oder unterschiedlichen SIMD-Vektorbreiten erzeugen systembedingt minimale ULP-Abweichungen in Cosine-/Dot-Product-Ergebnissen. Wenn Vektorindizes (z. B. HNSW-Graph-Nachbarschaftslisten oder BM25 Floating-Point Scores) auf Basis von Float-Vergleichen aufgebaut werden, führt eine ULP-Toleranz bei der Distanzberechnung auf unterschiedlichen CPU-Architekturen zu abweichenden HNSW-Kanten. Somit ist der Rebuild über unterschiedliche CPU-Generationen **nicht bit-identisch**, obwohl INV-NONDET-1 und §26 dies bedingungslos fordern.

### Widerspruch 2: Löschversprechen vs. Backup Key-Escrow / Key-Recovery (§20 vs. §24.4 / §20.15)
- **Problem**: §20 garantiert unwiderrufliches Löschen ("Forget/Shredding") durch kryptografische Schlüsselvernichtung (`KeyRevocation`) und physisches Purging.
- **Gegen-Regel in §24.4 / §20.15**: §24.4 erlaubt Backup/Checkpoint-Exporte und deklariert Key-Escrow bzw. HSM-Verwahrung (Verwahrungsstufen K0, K1, K2). In §20.15 ("Verwahrer-Realismus") räumt die Spezifikation selbst ein: *"Eine lokal gespeicherte oder im Backup befindliche EpochKek ist nicht sicher vernichtbar ... Backup-Kopien weiter möglich."*
- **Spezifikationskonflikt**: Wenn der Hauptschlüssel oder Epoch-Key in einem Backup / Escrow existiert, bricht das normative Versprechen von REQ-DEL-001 (unwiderrufliches Vergessen). Ein Restore eines alten Backups hebt die Löschung faktisch auf, sofern der Key-Escrow den Schlüssel wiederherstellt. Zwar versucht `INV-RESTORE-HORIZON-1` dies zu verhindern, jedoch widersprechen sich die absolute Aussage von §20 ("Unwiderruflich gelöscht") und die Zugeständnisse in §20.15/§24.4.

### Widerspruch 3: Cross-Crate Besitz der Beweisausstellung (INV-WITNESS-MINT-1 vs. §20.4a & §20.5 Text)
- **Problem**: INV-WITNESS-MINT-1 legt fest: *"LayerWitness nur aus store/vector ausstellbar; Bypass-Konstruktoren außerhalb von Tests verboten."*
- **Gegen-Regel in §20.5 Text & Matrix**: An anderer Stelle in §20.5 heißt eswörtlich: *"INV-WITNESS-MINT-1: LayerWitness lebt in crypto, hat private Felder ... Ausstellung nur über LayerWitness::mint."*
- **Spezifikationskonflikt**: Rust-Sicherheits- / Sichtbarkeitsregeln erlauben private Feldkonstruktoren nur innerhalb des Crates, in dem der Typ definiert ist. Wenn `LayerWitness` in `crypto` definiert ist, können `store` und `vector` ohne public/crate-internal Bypass-Funktionen keine `LayerWitness`-Instanzen erzeugen. Die Spezifikation verlangt zeitgleich, dass `crypto` keine Abhängigkeiten auf `store` oder `vector` hat (Ring 1 Kantenregel).

---

## 5. Komplexitätsbudget-Bewertung
Die Spezifikation v1.3 umfasst 34 Workspace-Crates, 68 Invarianten, 167 REQ-Anforderungen und ein 8-stufiges Löschbeweis-System.
- **Architektureller Überbau**: Die Durchsetzung von 68 Invarianten erfordert massive Laufzeit- und Typestate-Prüfungen. Insbesondere das 8-Schichten-Löschsystem (`DeletionProof`) mit synchronen Fences, Anti-Resurrection-Ledger und Sidecar-Ketten erzeugt erhebliche Komplexität auf dem Schreib- und Löschpfad.
- **Ergebnis Phase A**: Die Spezifikation ist präzise formuliert, leidet jedoch an 3 klaren normativen Antinomien (Bit-Identität vs. SIMD ULP, Absolute Löschung vs. Backup Escrow, LayerWitness Crate Ownership).
