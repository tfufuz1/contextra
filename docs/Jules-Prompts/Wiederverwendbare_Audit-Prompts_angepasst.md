> **Anpassungsvermerk (Stand 2026-11-02, HEAD `c41afc15`):** Dieses Dokument wurde gegen den Repo-Stand geprüft und automatisch angepasst. ⚠ markiert Crate-Namen, die im Repo nicht existieren. `⚠NICHT-IM-REPO(...)` markiert Befehle, deren `cargo xtask`-Target fehlt. Vorgenommene Änderungen:
> - 56 `cargo xtask`-Aufrufe markiert (xtask existiert nicht, kein passendes just-Target)
> - 20 Crate-Abschnitte entfernt (Crate existiert am HEAD nicht): II.2 contextra-core, II.5 contextra-adapt, II.6 contextra-graph, II.10 contextra-wire, II.15 contextra-checkpoint, II.16 contextra-kvcache, II.17 contextra-infer-candle, II.18 contextra-infer-ollama, II.19 contextra-infer-onnx, II.20 contextra-sandbox, II.21 contextra-db, II.23 contextra-router, II.24 contextra-cognition, II.25 contextra-agent, II.26 contextra-privacy, II.27 contextra-py, II.28 contextra-mcp, II.29 contextra-license, II.31 contextra-audit-export, II.32 contextra-avv-generator
> - Verbleibende Nennungen nicht existierender Crates in Teil I/IV oder Restprosa (manuell prüfen): contextra-adapt, contextra-agent, contextra-audit-export, contextra-avv-generator, contextra-checkpoint, contextra-cognition, contextra-core, contextra-db, contextra-graph, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-kvcache, contextra-license, contextra-mcp, contextra-privacy, contextra-py, contextra-router, contextra-sandbox, contextra-wire
> - II.7: Paketname contextra-privacy → contextra-crypto korrigiert (Prompt-Fehler im Original)
> - IV.1: Ring-Liste und Unsafe-Insel-Liste auf Ist-Stand (capabilities.toml) gesetzt
> - III.7 (CSR-Graph), III.9 (Adaptive Controller), III.10 (KV-Cache) entfernt: Zielcrates am HEAD nicht vorhanden

# Contextra — Elite Audit-Prompt-Bibliothek v5

> **Basis:** Repository `https://github.com/tfufuz1/contextra`, HEAD `c41afc15` (angepasst am 2026-11-02)
> **Normativität:** Code + grüne Gates > AGENTS.md > Spec > alles andere (AGENTS.md §1)
> **Verwendung:** Jeden Prompt einzeln in Jules (Coding-Agent) einfügen. Jeder Prompt ist vollständig selbstenthaltend — keine externe Kontext-Vorbedingung nötig. Wiederverwendbar nach jedem Merge, Sprint oder Release.

---

## Konventionen

- **Session-Claiming:** Jeder Prompt beginnt mit `claim_precheck.sh`. Ohne grünes Claim kein Code-Write.
- **Audit-Output:** Immer nach `docs/audits/<CRATE>_AUDIT_<YYYY-MM-DD>.md`. Logs zusätzlich nach `/tmp/audit-<CRATE>.log`.
- **VERDICT-Format:** Jede VERDICT-Zeile folgt das Zwei-Session-Protokoll aus `AUDIT_INTAKE_PROTOCOL.md`.
- **Kein Code-Write:** Alle Prompts unten sind primär auditierend. Befunde werden als `[BLOCKER]`/`[MAJOR]`/`[MINOR]` klassifiziert, nicht sofort behoben.
- **xtask-Pfad:** `cargo run --manifest-path xtask/Cargo.toml -- <CMD>` oder Kurzform `cargo xtask <CMD>`.

---

# TEIL I — MASTER-BOOTSTRAP-PROMPT

```
Du bist Principal Senior Rust Architect für Contextra (https://github.com/tfufuz1/contextra). Diese Sitzung dient dem Session-Bootstrap und der IST-Zustandserfassung. Keine Code-Änderungen.

SEQUENZ — exakt in dieser Reihenfolge:

1. SESSION-IDENTITÄT: SESSION_HASH=$(date -u +%Y%m%d%H%M%S | sha256sum | head -c 8); TS=$(date -u +%Y-%m-%dT%H:%M:%SZ); echo "SESSION: $SESSION_HASH | TS: $TS"

2. KONTEXT LADEN: bash .jules/context/gen_context_pack.sh 2>&1 | tail -5 — dann .jules/context/CONTEXT_PACK.md lesen (1 Read-Call, kein Explorieren).

3. BLOCKER-SCAN: grep -rn "AI-TAG\[.*\]\[BLOCKER\]\|AI-TAG\[.*\]\[CRITICAL\]" crates/ --include="*.rs" | grep -v RESOLVED — bei Fund: STOP, keine weitere Arbeit, Fund melden.

4. IN-PROGRESS ANCHORS: grep -rn "ANCHOR\[.*\] STATUS:IN-PROGRESS" crates/ --include="*.rs" | head -20

5. WORKING STATE: head -80 WORKING_STATE.md

6. VETO-CHECK: ⚠NICHT-IM-REPO(cargo xtask check-vetoes) 2>&1 | tail -20 — liste alle aktiven VETOs mit conditional_review_due-Datum und vergleiche gegen das heutige Datum; überfällige Reviews sind Governance-Befunde.

7. BUILD-GRUNDLAGE: cargo check --workspace 2>&1 | tail -10

8. GIT-KONTEXT: git log -n 10 --oneline; git log -n 3 --stat

9. NORMATIVES REGELWERK: AGENTS.md (Root), docs/CONSTITUTION.md, docs/VETOES.md vollständig lesen — insbesondere §6 Invarianten und §7 Nicht-tun.

10. PREFLIGHT: ⚠NICHT-IM-REPO(⚠NICHT-IM-REPO(cargo xtask jules-preflight)) 2>&1 | tail -30

11. RING-LAYERING: ⚠NICHT-IM-REPO(cargo xtask check-ring-layering-full) 2>&1 | tail -20

12. UNSAFE-INSELN: ⚠NICHT-IM-REPO(cargo xtask check-unsafe-islands) 2>&1 | tail -20 — erwartet: ONLY contextra-simd, contextra-sys, contextra-wire, contextra-crypto als unsafe_island=true.

13. ADR-DEADLINES: ⚠NICHT-IM-REPO(cargo xtask check-adr-deadlines) 2>&1 | tail -20

LIEFERGEGENSTAND (Markdown-Zusammenfassung):
- (a) SESSION_HASH + TS
- (b) BLOCKER/CRITICAL-Scan: Anzahl Funde + Fundstellen
- (c) Aktive VETOs (VETO-ID | due-Datum | überfällig?)
- (d) Build-Status: OK / Fehler-Count
- (e) Letzte 3 geänderte Module (aus git log --stat)
- (f) Ring-Layering: OK / Verletzungen
- (g) Unsafe-Inseln: konform / Abweichungen
- (h) Explizite GO / STOP-Empfehlung für produktive Arbeit
```

---

# TEIL II — CRATE-AUDIT-PROMPTS

## RING 0 — Fachkern & Unsafe-Inseln

---

### II.1 `contextra-types` — Ring 0, Domain-Identifikatoren(~7304 LOC, gemessen)

```
Du bist Principal Senior Rust Architect für Contextra. Auftrag: vollständiger Audit-Zyklus für crates/contextra-types/src/.

VORBEREITUNG:
bash .jules/verify/claim_precheck.sh --prune-expired --crate contextra-types --task "full-audit" 2>&1
cat crates/contextra-types/AGENTS.md
head -40 WORKING_STATE.md | grep contextra-types

KONTEXT: contextra-types ist Ring 0, #![forbid(unsafe_code)], definiert DocId, TxId, TenantId, EntityId, Embedding, DistanceMetric, Filter-AST, Budget-Typen. Keine Fachcrate-Abhängigkeiten erlaubt (may_depend_on = []). Alle anderen Crates importieren von hier — Typ-Fehler hier kaskadieren workspace-weit.

PRÜFPUNKTE (alle mit Datei:Zeile-Referenz):

P1 NEWTYPE-DISZIPLIN: grep -n "repr(transparent)" crates/contextra-types/src/ -r — sind DocId, TxId, TenantId, EntityId alle #[repr(transparent)] über u64? Feature docid-128 (ADR-082) muss u128 ergeben. Gibt es From<u64>-Impls die die Newtype-Grenze ohne Validierung durchbrechen?

P2 INV-TENANT-1 (TenantId(0) = SYSTEM-reserviert): Suche den try_new-Konstruktor. Verifiziere: try_new(0) -> Err, try_new(1) -> Ok. Existiert ein expliziter Unit-Test für diese Grenzfälle inkl. Serialisierungs-Rundreise (TenantId -> bytes -> TenantId)?

P3 DISTANZMETRIK: grep -n "DistanceMetric\|non_exhaustive" crates/contextra-types/src/ -r — ist #[non_exhaustive] gesetzt? Sind Cosine/Euclidean/DotProduct vollständig mit Wertebereich und Symmetrieeigenschaft dokumentiert?

P4 FILTER-AST TIEFENBEGRENZUNG: Finde den Ausdruck-Baum-Typ. Existiert eine maximale Verschachtelungstiefe-Prüfung (Schutz gegen Stack-Overflow bei tief verschachteltem, extern konstruiertem Filter)? Konstruiere testweise eine 1000-Ebenen-tiefe Filter-Struktur und prüfe ob Auswertung divergiert.

P5 BUDGET-ARITHMETIK: grep -n "saturating_sub\|checked_sub\|wrapping" crates/contextra-types/src/ -r — sind TokenBudget/ResourceBudget-Subtraktionen gegen Unterlauf abgesichert (kein raw - auf u64)?

P6 ZERO-UNSAFE: grep -rn "unsafe" crates/contextra-types/src/ — muss leer sein außer der forbid-Deklaration.

P7 DAG-REINHEIT: ⚠NICHT-IM-REPO(cargo xtask check-ring-layering-full) 2>&1 | grep contextra-types — kein Import von Ring-1+-Crates erlaubt.

P8 DUPLIKAT-PRIMITIVEN: ⚠NICHT-IM-REPO(cargo xtask check-duplicate-core-primitives) 2>&1 — Prüft ob andere Crates äquivalente Typen neu definieren statt von hier zu importieren.

TESTS & LINTS:
cargo test -p contextra-types --locked -- --nocapture 2>&1 | tee /tmp/audit-types-test.log
cargo clippy -p contextra-types --all-targets --all-features -- -D warnings 2>&1 | tee /tmp/audit-types-clippy.log
cargo doc -p contextra-types --no-deps 2>&1 | grep "warning: missing" | wc -l

LIEFERGEGENSTAND: docs/audits/contextra-types_AUDIT_<YYYY-MM-DD>.md
Pflichtabschnitte: (1) Prüftabelle P1-P8 (Status ✅/❌/⚠️ | Fundstelle | Befund), (2) Kollisionsmatrix (Typ | Duplikat in Crate | Schwere), (3) Test-Ergebnis + Coverage-Zahl, (4) Clippy-Ergebnis, (5) VERDICT: GO/CONDITIONAL/BLOCKED + VERIFIED-BY-SESSION: PENDING (TS: <ISO>).
Keine Code-Änderung ohne Folgeauftrag.
```

---

### II.3 `contextra-ports` — Ring 0, Port/Trait-System(~2668 LOC, gemessen)

```
Du bist Principal Senior Rust Architect für Contextra. Auftrag: vollständiger Audit-Zyklus für crates/contextra-ports/src/ — die zentrale Entkopplungsschicht (Ports-and-Adapters).

VORBEREITUNG:
bash .jules/verify/claim_precheck.sh --prune-expired --crate contextra-ports --task "full-audit" 2>&1
cat crates/contextra-ports/AGENTS.md

KONTEXT: Ring 0, #![forbid(unsafe_code)]. Definiert dyn-kompatible Traits für Storage, Vektorindex, Textindex, Graphindex, Embedder, Clock und Lizenz-Gate. AGENTS.md §6 (Root): "Kein SystemTime::now() und kein thread_rng() direkt in Produktionscode — ausschließlich über injizierte Clock- und Rng-Ports". ADR-098-Ausnahme: Kryptografisches Material MUSS OsRng nutzen — NUR in contextra-crypto.

PRÜFPUNKTE:

P1 DYN-KOMPATIBILITÄT: Für jeden Trait in diesem Crate: enthält er generische Parameter oder impl-Trait-Rückgabetypen die dyn-Kompatibilität verletzen? Test: kompiliert Box<dyn StorageReadPort>/Box<dyn VectorIndexPort>/Box<dyn TextIndexPort>/Box<dyn GraphIndexPort> ohne Fehler?

P2 CLOCK-PORT-VOLLSTÄNDIGKEIT: grep -rn "impl.*Clock for" crates/ --include="*.rs" — liste alle Implementoren. Mindestens erwartet: SystemClock (Produktion) und ManualClock (aus contextra-testkit). Ist SystemClock::now_unix_nanos() monoton?

P3 RNG-PORT-COMPLIANCE (ADR-098): grep -rn "thread_rng()\|OsRng" crates/ --include="*.rs" | grep -v "contextra-crypto" | grep -v "/tests/" | grep -v "_test.rs" — diese Liste muss leer oder mit expliziter Begründung sein. Jeder Treffer außerhalb von contextra-crypto ist P28-Verletzung.

P4 TRAIT-VERTRAGS-DOKUMENTATION: Für StorageRead, VectorIndex, TextIndex, GraphIndex: Ist für jede Methode dokumentiert (a) welche ContextraError-Varianten möglich sind, (b) Verhalten bei leeren Ergebnismengen, (c) Verhalten bei k==0 bei k-NN-Suche?

P5 LIZENZ-GATE FAIL-CLOSED: Suche den Lizenz-Gate-Trait. Gibt jede denkbare Fehlerbedingung (Netzwerkfehler, korrupte Signatur, abgelaufener Key) ein restriktives statt permissives Ergebnis?

P6 RING-REINHEIT: ⚠NICHT-IM-REPO(cargo xtask check-ring-layering-full) 2>&1 | grep contextra-ports — darf keine Ring-1+-Crates importieren.

TESTS & LINTS:
cargo test -p contextra-ports --locked -- --nocapture 2>&1 | tee /tmp/audit-ports-test.log
cargo clippy -p contextra-ports --all-targets -- -D warnings 2>&1 | tee /tmp/audit-ports-clippy.log

LIEFERGEGENSTAND: docs/audits/contextra-ports_AUDIT_<YYYY-MM-DD>.md
Pflichtabschnitte: (1) Trait-Inventar-Tabelle (Trait | dyn-kompatibel? | Implementoren-Count | Fehlerverhalten dokumentiert?), (2) Clock/Rng-Compliance-Sektion mit vollständiger grep-Ausgabe aus P3, (3) Lizenz-Gate-Fail-Closed-Nachweis, (4) VERDICT + VERIFIED-BY-SESSION: PENDING (TS: <ISO>).
```

---

### II.4 `contextra-mvcc` — Ring 2, MVCC-Isolationsschicht(~4195 LOC, gemessen)

```
Du bist Principal Senior Rust Architect für Contextra. Auftrag: algorithmischer Audit- und Property-Test-Zyklus für crates/contextra-mvcc/src/ (seq_log.rs, snapshot.rs, tx_buffer.rs). Dies ist ein Algorithmus-Audit, kein Code-Stil-Audit.

VORBEREITUNG:
bash .jules/verify/claim_precheck.sh --prune-expired --crate contextra-mvcc --task "algorithm-audit" 2>&1
cat crates/contextra-mvcc/AGENTS.md

KONTEXT: Ring 0, strikt synchron (P26), #![forbid(unsafe_code)]. Implementiert MVCC-Snapshot-Isolation für contextra-store, contextra-vector. Kritische Formel: sichtbar(e, as_of) gdw. e.insert_seq <= as_of AND (e.delete_seq == None OR e.delete_seq > as_of). Kein Wanduhr-Bezug erlaubt (P28).

ALGORITHMUS-PRÜFPUNKTE (mit Code-Zeilen-Beleg):

P1 SICHTBARKEITSFORMEL-EXAKTHEIT: Finde die konkrete Implementierung in seq_log.rs. Verifiziere exakt den Grenzfall delete_seq == Some(as_of): gemäß Formel muss dieser Eintrag NICHT sichtbar sein (delete_seq > as_of ist streng, nicht >=). Belege mit Codezeile.

P2 O(1)-KOMPLEXITÄT DER SICHTBARKEITSPRÜFUNG: Ist der heißeste Prüfpfad (einzelner SeqLogEntry für ein as_of) allokationsfrei und schleifenfrei (maximal zwei Integer-Vergleiche)? Gib die genaue Zeile an.

P3 SNAPSHOT-REGISTRY-DATENSTRUKTUR: Welche konkrete Datenstruktur liegt unter SnapshotRegistry (BTreeMap, Vec, Heap)? Ist min_active_seqno() O(1) oder O(log n)? Stimmt die tatsächliche Komplexität mit der Dokumentation überein?

P4 KEIN-LOST-UPDATE: Suche jede Stelle die ein SeqLogEntry mutiert. Wird insert_seq nach Konstruktion je neu geschrieben? (Darf nicht — nur additive delete_seq-Setzung erlaubt.)

P5 P28-DETERMINISMUS: grep -rn "SystemTime::now()\|Instant::now()\|thread_rng()" crates/contextra-mvcc/src/ | grep -v "#\[cfg(test)\]" — muss leer sein.

P6 NEBENLÄUFIGKEITS-TEST: Existiert ein Loom-Test oder Stress-Test für gleichzeitige register/deregister-Aufrufe plus anschließende min_active_seqno()-Konsistenz? Wenn nicht: dokumentiere als MAJOR-Befund.

PROPERTY-TESTS (implementieren wenn nicht vorhanden, als Befund melden wenn fehlend):
cargo test -p contextra-mvcc --locked -- --nocapture 2>&1 | tee /tmp/audit-mvcc-test.log
Prüfe ob crates/contextra-mvcc/tests/ einen Property-Test enthält der: zufällige Tripel (insert_seq, delete_seq, as_of) gegen eine triviale Referenzimplementierung abgleicht (mind. 10.000 Fälle, explizite Grenzfälle as_of==insert_seq, as_of==delete_seq, delete_seq==None).

BENCHMARKS: Wenn Criterion-Benchmarks für register/unregister/min_active_seqno bei N={10, 1000, 100000} aktiven Snapshots vorhanden: Ausgabe lesen und Skalierungscharakter bewerten. Falls fehlend: als MAJOR-Befund melden.
cargo bench -p contextra-mvcc 2>&1 | tail -30

LINTS:
cargo clippy -p contextra-mvcc --all-targets -- -D warnings 2>&1 | tee /tmp/audit-mvcc-clippy.log

LIEFERGEGENSTAND: docs/audits/contextra-mvcc_AUDIT_<YYYY-MM-DD>.md
Pflichtabschnitte: (1) Formale Invarianten-Beweistabelle (P1-P6: Aussage | verifiziert | Beleg Datei:Zeile | Gegenbeispiel falls nein), (2) Datenstruktur-Analyse mit Komplexitätsaussagen, (3) Property-Test-Ergebnis (Anzahl Fälle / Fehlschläge), (4) Benchmark-Zahlen, (5) VERDICT + VERIFIED-BY-SESSION: PENDING (TS: <ISO>).
```

---

### II.7 `contextra-crypto` — Ring 1, Unsafe-Insel, Krypto-Kern(~3260 LOC, gemessen)

```
Du bist Principal Senior Rust Architect für Contextra. Auftrag: vollständiger Sicherheitsaudit für crates/contextra-crypto/src/ (Cargo-Package-Name: contextra-crypto). Dies ist eine Unsafe-Insel (unsafe_island = true) — jedes unsafe-Vorkommen muss mit SAFETY-Kommentar gerechtfertigt sein.

VORBEREITUNG:
bash .jules/verify/claim_precheck.sh --prune-expired --crate contextra-crypto --task "security-audit" 2>&1
cat crates/contextra-crypto/AGENTS.md
git log --oneline -5 -- crates/contextra-crypto/ — letzten Crypto-Commit prüfen

KONTEXT: Ring 0, unsafe_island=true. Verantwortlich für AES-256-GCM-SIV Encryption-at-Rest, HMAC-Chaining im WAL, HKDF Key-Derivation, Zeroize-Speicherhygiene, Ed25519 Löschnachweise, EgressVault DLP. KRITISCH: INV-CRYPTO-DUPLICATE-1 (hash_deleted_keys_length_prefixed ist kanonisch in deletion_proof.rs, ed25519_proof.rs re-exportiert statt dupliziert — Commit e4632e58 behauptet dies behoben).

PRÜFPUNKTE:

P1 INV-CRYPTO-DUPLICATE-1 VERIFIZIERUNG: grep -rn "fn hash_deleted_keys_length_prefixed" crates/contextra-crypto/src/ — darf NUR EINMAL in deletion_proof.rs vorkommen. In ed25519_proof.rs muss "pub use crate::deletion_proof::hash_deleted_keys_length_prefixed" stehen. Prüfe ob Commit e4632e58 wirklich die Duplikation eliminiert hat.

P2 UNSAFE-AUDIT (vollständig): grep -rn "unsafe" crates/contextra-crypto/src/ | grep -v "#\[cfg(test)\]" | grep -v "forbid\|allow\|deny" — liste JEDEN unsafe-Block. Für jeden: (a) existiert SAFETY-Kommentar? (b) ist die Safety-Begründung korrekt? (c) könnte ein safe-Äquivalent verwendet werden?

P3 HARDCODED-KEY-SCAN (BLOCKER wenn vorhanden): grep -rn "b\".*key\|b\".*Key\|hex::decode\|0x00,.*0x00" crates/contextra-crypto/src/ | grep -v test — kein Literal-Schlüssel im Produktionscode. Alle Keys MÜSSEN via HKDF von Passphrase+Salt abgeleitet werden.

P4 ZEROIZE-COMPLIANCE: grep -rn "VolatileEncryptionKey\|ZeroizeOnDrop\|Zeroizing" crates/contextra-crypto/src/ -r — Schlüsselmaterial darf nie als Vec<u8>/String außerhalb von Zeroize-Wrapper existieren. Suche Stellen wo key-Material als plain-bytes kopiert wird: grep -rn "\.clone()\|to_vec()\|to_owned()" crates/contextra-crypto/src/ | grep -i "key\|secret\|master" | grep -v test

P5 HMAC-CHAIN-INTEGRITÄT: In wal_crypto.rs: Prüfe verify_and_update_v3 — wird bei einem ungültigen HMAC (Manipulation oder Korruption) hart mit ContextraError::WalCorruption abgebrochen? let _ = verifier.verify_and_update(...) ist VERBOTEN (AGENTS.md §5 Anti-Pattern).

P6 NONCE-UNIQUENESS: Für encrypt_auto_nonce — wird jede Verschlüsselungsoperation mit einer neuen, kryptografisch sicheren Zufalls-Nonce (OsRng) versehen? Gibt es einen Pfad wo eine Nonce wiederverwendet werden könnte?

P7 ADR-098-KONFORMITÄT: OsRng/thread_rng-Nutzung ist AUSSCHLIESSLICH hier erlaubt. Prüfe: ⚠NICHT-IM-REPO(cargo xtask check-unsafe-islands) 2>&1 | grep crypto — Bestätigung dass crypto als legale unsafe-Insel registriert ist.

P8 EGRESS-VAULT FAIL-CLOSED: Bei Fehlern (Timeout, Index-Fehler, Regex-Fehler) im EgressClassifier: Wird Block zurückgegeben oder Allow? Muss Block sein (Fail-Closed-Invariante aus contextra-crypto AGENTS.md).

TESTS & LINTS:
cargo test -p contextra-crypto --locked -- --nocapture 2>&1 | tee /tmp/audit-crypto-test.log
cargo mutants -p contextra-crypto -- --locked 2>&1 | tail -30 | tee /tmp/audit-crypto-mutants.log
cargo clippy -p contextra-crypto --all-targets -- -D warnings 2>&1 | tee /tmp/audit-crypto-clippy.log

LIEFERGEGENSTAND: docs/audits/contextra-crypto_AUDIT_<YYYY-MM-DD>.md
Pflichtabschnitte: (1) Vollständige unsafe-Inventar-Tabelle (Block | SAFETY-Kommentar? | Korrekt? | Alternatif safe?), (2) Hardcoded-Key-Scan-Ergebnis (BLOCKER wenn Treffer), (3) Zeroize-Compliance-Nachweis, (4) INV-CRYPTO-DUPLICATE-1 Verifikation, (5) Mutationstest-Score, (6) VERDICT + VERIFIED-BY-SESSION: PENDING (TS: <ISO>).
```

---

### II.8 `contextra-simd` — Ring 1, Unsafe-Insel, SIMD-Distanz-Kernel(~2797 LOC, gemessen)

```
Du bist Principal Senior Rust Architect für Contextra. Auftrag: Sicherheits- und Korrektheitsprüfung für crates/contextra-simd/src/ (dispatch.rs, kernels/). Unsafe-Insel: unsafe_island=true, #![deny(unsafe_op_in_unsafe_fn)].

VORBEREITUNG:
bash .jules/verify/claim_precheck.sh --prune-expired --crate contextra-simd --task "unsafe-audit" 2>&1
cat crates/contextra-simd/AGENTS.md

KONTEXT: Ring 0, einzige Verantwortung: SIMD-beschleunigte Distanzberechnungen (Cosine, L2, DotProduct) mit Hardware-Dispatch (AVX-512 > AVX2 > NEON > Scalar). AGENTS.md §3: "Öffentliche Schnittstelle ist zu 100% safe." Alle unsafe-Blöcke müssen #![deny(unsafe_op_in_unsafe_fn)] erfüllen. Keine anderen Contextra-Fachcrates als Abhängigkeit.

PRÜFPUNKTE:

P1 SAFE-BOUNDARY VOLLSTÄNDIG: grep -rn "pub fn\|pub unsafe fn" crates/contextra-simd/src/lib.rs — alle öffentlich exportierten Funktionen müssen safe fn sein (kein pub unsafe fn in der öffentlichen API).

P2 UNSAFE_OP_IN_UNSAFE_FN COMPLIANCE: cargo clippy -p contextra-simd --all-targets -- -D unsafe_op_in_unsafe_fn 2>&1 | grep -v "^$" | tee /tmp/audit-simd-unsafe-lint.log — keine Warnungen erlaubt.

P3 SAFETY-KOMMENTAR VOLLSTÄNDIGKEIT: grep -n "unsafe {" crates/contextra-simd/src/ -r | wc -l als Gesamtzahl, dann grep -B1 "unsafe {" crates/contextra-simd/src/ -r | grep "SAFETY:" | wc -l — beide Zahlen müssen identisch sein. Jedes unsafe-Block ohne SAFETY-Kommentar ist Befund.

P4 DISPATCH-KORREKTHEIT: In dispatch.rs: Wird der Hardware-Feature-Check zur Laufzeit korrekt durchgeführt (is_x86_feature_detected! oder Äquivalent)? Kann ein falscher Feature-Branch ausgewählt werden (z.B. AVX-512 auf CPU ohne AVX-512)?

P5 SKALAR-FALLBACK MATHEMATISCHE IDENTITÄT: Für jede SIMD-Distanzfunktion: Liefert der Scalar-Fallback exakt dieselbe mathematische Operation wie die SIMD-Variante (nicht nur "ungefähr")? Ein Rundungsunterschied ist tolerabel, ein Algorithmus-Unterschied nicht. Prüfe: Haben Cosine-SIMD und Cosine-Scalar dieselbe Normierungsformel?

P6 ZERO-PANIC BEI LEERVEKTOR: Was passiert bei dimension=0 oder leeren Slices? Gibt es Panik oder definiertes Ergebnis (z.B. 0.0 oder Err)?

P7 ALIGNMENT-ANFORDERUNGEN: Haben SIMD-Kernel Alignment-Anforderungen an Eingabe-Pointer? Wenn ja: Ist das in SAFETY-Kommentaren dokumentiert und in lib.rs als Precondition für Caller?

TESTS:
cargo test -p contextra-simd --locked -- --nocapture 2>&1 | tee /tmp/audit-simd-test.log
cargo test -p contextra-simd --all-features --locked -- --nocapture 2>&1 | tee /tmp/audit-simd-test-all.log
cargo clippy -p contextra-simd --all-targets -- -D warnings 2>&1 | tee /tmp/audit-simd-clippy.log

LIEFERGEGENSTAND: docs/audits/contextra-simd_AUDIT_<YYYY-MM-DD>.md
Pflichtabschnitte: (1) unsafe-Inventar (Count vs. SAFETY-Kommentar Count), (2) Dispatch-Korrektheit-Analyse, (3) Scalar-Fallback-Äquivalenz-Nachweis, (4) Alignment-Anforderungen-Dokumentation, (5) VERDICT + VERIFIED-BY-SESSION: PENDING (TS: <ISO>).
```

---

### II.9 `contextra-sys` — Ring 1, Unsafe-Insel, OS-Bindings(~2035 LOC, gemessen)

```
Du bist Principal Senior Rust Architect für Contextra. Auftrag: Sicherheitsaudit für crates/contextra-sys/src/ (mmap.rs, mlock.rs, acl_win32.rs, posix.rs). Unsafe-Insel mit OS-Grenzkontakt.

VORBEREITUNG:
bash .jules/verify/claim_precheck.sh --prune-expired --crate contextra-sys --task "unsafe-audit" 2>&1
cat crates/contextra-sys/AGENTS.md

KONTEXT: Ring 0, unsafe_island=true, #![allow(unsafe_code)] mit #![deny(unsafe_op_in_unsafe_fn)]. Kapselt mmap (zero-copy readonly), mlock (physische Speichersperre gegen Swap), Win32-ACL, POSIX-Abstraktion. Alle öffentlichen Funktionen bieten safe Rust-Fassade.

PRÜFPUNKTE:

P1 SAFE-FASSADE: grep -rn "^pub fn\|^pub unsafe fn" crates/contextra-sys/src/ — kein pub unsafe fn in der öffentlichen API (lib.rs Re-Exports).

P2 MMAP-SICHERHEIT: In mmap.rs: Wird die mmap-Lifetime korrekt an die zugrundeliegende Datei gebunden? Kann der Datei-Deskriptor geschlossen werden während die mmap noch aktiv ist (use-after-close)? Gibt es eine RAII-Struktur die beides koppelt?

P3 MLOCK-ERROR-PROPAGATION: In mlock.rs: Werden Fehler aus mlock(2)/munlock(2) als Result zurückgegeben oder still verschluckt? AGENTS.md §3: "Fehler werden sauber als std::io::Result zurückgegeben."

P4 POSIX-FD-SICHERHEIT: In posix.rs: Werden File-Deskriptoren korrekt validiert vor Verwendung in dup2 o.ä.? Gibt es Pfade wo ein ungültiger fd zu undefined behavior führen kann?

P5 WINDOWS-ACL KORREKTHEIT: In acl_win32.rs: Funktioniert verify_file_acl_owner_only korrekt auf nicht-Windows (Compile-Guard #[cfg(windows)])?

P6 KEINE FACHCRATE-IMPORTS: cargo metadata --manifest-path crates/contextra-sys/Cargo.toml --no-deps --format-version 1 | python3 -c "import json,sys; d=json.load(sys.stdin); [print(p['name']) for p in d['packages'][0]['dependencies']]" — kein contextra-* Crate erlaubt außer vielleicht contextra-types.

TESTS:
cargo test -p contextra-sys --locked -- --nocapture 2>&1 | tee /tmp/audit-sys-test.log
cargo clippy -p contextra-sys --all-targets -- -D warnings 2>&1 | tee /tmp/audit-sys-clippy.log

LIEFERGEGENSTAND: docs/audits/contextra-sys_AUDIT_<YYYY-MM-DD>.md
Pflichtabschnitte: (1) unsafe-Block-Inventar mit SAFETY-Kommentar-Status, (2) mmap-Lifetime-Analyse, (3) Error-Propagation-Nachweis, (4) VERDICT + VERIFIED-BY-SESSION: PENDING (TS: <ISO>).
```

---

### II.11 `contextra-rank` — Ring 2, 4-Signal-Fusion(~2370 LOC, gemessen)

```
Du bist Principal Senior Rust Architect für Contextra. Auftrag: Algorithmus- und Korrektheitsaudit für crates/contextra-rank/src/ (fusion/, calibration/, drift.rs, explain.rs).

VORBEREITUNG:
bash .jules/verify/claim_precheck.sh --prune-expired --crate contextra-rank --task "algorithm-audit" 2>&1
cat crates/contextra-rank/AGENTS.md

KONTEXT: Ring 0, #![forbid(unsafe_code)]. Signal-Aggregation aus Vektor (HNSW), Text (BM25), Graph (PPR), Storage (LSM) via Reciprocal Rank Fusion. Enthält Isotonic/Platt-Kalibrierung, Drift-Detektion, Erklärbarkeits-API (explain.rs).

PRÜFPUNKTE:

P1 RRF-FORMEL-KORREKTHEIT: score(d) = Σ_i 1/(k + rank_i(d)). Finde die Implementierung in fusion/. Welcher k-Wert wird verwendet (typisch k=60)? Ist er konfigurierbar? Prüfe: Ein Dokument das in KEINEM Signal vorkommt erhält KEINEN Score (kein künstliches rank=∞ mit Ergebnis 0.0 das das Ranking verzerrt).

P2 ISOTONIC-KALIBRIERUNG MONOTONIE: Die Kalibrierungsfunktion MUSS monoton sein — ein höherer Rohscore darf niemals zu kleinerem kalibrierten Score führen. Test: Sortiere aufsteigend [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9] als Eingabe — ist Ausgabe ebenfalls nicht-fallend?

P3 DRIFT-DETEKTION THREAD-SAFETY: Wird der Kalibrierungsparameter-Austausch während einer laufenden Anfrage inkonsistente Zwischenzustände erzeugen können? Wie ist der Übergang von alter zu neuer Kalibrierung synchronisiert?

P4 ERKLÄRBARKEIT-KONSISTENZ (explain.rs): Liefert explain() für ein Suchergebnis eine Zerlegung bei der Σ(Signalbeiträge) den finalen Score reproduziert? Testfall: manuell einen Scores mit bekannten RRF-Beiträgen konstruieren und explain()-Ausgabe gegen manuelle Berechnung prüfen.

P5 DIBUD-FEATURE-FLAG: grep -A5 "feature.*dibud" crates/contextra-rank/Cargo.toml — ist dibud nicht Teil von default? (experimentelle Feature-Gate-Disziplin)

P6 SYNC-REINHEIT: grep -rn "tokio::\|async fn" crates/contextra-rank/src/ | grep -v test — Ring 0 muss sync bleiben.

BENCHMARKS:
cargo bench -p contextra-rank 2>&1 | tee /tmp/audit-rank-bench.log — RRF-Fusion bei 10/100/1000 Kandidaten (liegt im heißen Pfad jeder Suchanfrage).

TESTS:
cargo test -p contextra-rank --locked -- --nocapture 2>&1 | tee /tmp/audit-rank-test.log
cargo clippy -p contextra-rank --all-targets -- -D warnings 2>&1 | tee /tmp/audit-rank-clippy.log

LIEFERGEGENSTAND: docs/audits/contextra-rank_AUDIT_<YYYY-MM-DD>.md
Pflichtabschnitte: (1) RRF-Formel-Verifikation mit k-Wert und Null-Score-Nachweis, (2) Monotonie-Testtabelle, (3) Erklärbarkeits-Konsistenz-Nachweis, (4) Benchmark-Zahlen (Latenz bei 10/100/1000 Kandidaten), (5) VERDICT + VERIFIED-BY-SESSION: PENDING (TS: <ISO>).
```

---

### II.12 `contextra-text` — Ring 2, BM25 Inverted Index(~7151 LOC, gemessen)

```
Du bist Principal Senior Rust Architect für Contextra. Auftrag: Determinismus- und Snapshot-Isolation-Audit für crates/contextra-text/src/ (bm25.rs, inverted/, tokenizer.rs, morphology.rs).

VORBEREITUNG:
bash .jules/verify/claim_precheck.sh --prune-expired --crate contextra-text --task "determinism-audit" 2>&1
cat crates/contextra-text/AGENTS.md

KONTEXT: Ring 0, #![forbid(unsafe_code)]. Signal 2 der 4-Signal-Fusion. Kritischste Invariante: Determinismus der Tokenisierung — Query-Pfad MUSS exakt dieselbe Pipeline wie Index-Pfad verwenden. Silent-Recall-Drop ist der gefährlichste Fehlertyp (kein Crash, nur Qualitätsverlust).

PRÜFPUNKTE:

P1 PIPELINE-IDENTITÄT (kritisch): Identifiziere den Tokenizer-Aufruf im upsert_document-Pfad und im Such-Pfad. Sind es exakt dieselbe Funktion/Typ/Konfiguration? Selbst minimale Unterschiede (z.B. Lowercase in einem Pfad aber nicht im anderen) führen zu Silent-Recall-Drops. Erstelle expliziten Regressionstest: Dokument mit deutschen Umlauten, Komposita, Sonderzeichen indexieren, dann für jedes Token Punktsuche — alle müssen das Dokument finden.

P2 GERMAN-COMPOUND-SPLITTER KORREKTHEIT: Teste mindestens 20 deutsche Komposita (z.B. Datenschutzgrundverordnung, Bundesdatenschutzgesetz, Softwarearchitektur, Computerprogramm, Kraftfahrzeugsteuer, Bundesverfassungsgericht, Informationstechnologie, Datenbankmanagement, Künstliche Intelligenz, Maschinelles Lernen) — dokumentiere erwartete vs. tatsächliche Zerlegung. Falsch-positive Zerlegungen (sinnlose Teilwörter) und falsch-negative Nicht-Zerlegungen sind je ein Minor-Befund.

P3 UMLAUT-NORMALISIERUNG BI-DIREKTIONAL: ä/ae, ö/oe, ü/ue, ß/ss — Suche nach "Baer" muss indexiertes "Bär" finden UND Suche nach "Bär" muss "Baer" finden. Teste alle 4 Paare in beide Richtungen.

P4 SNAPSHOT-ISOLATION (search_at): Verifiziere dass BM25-Suche bei historischer Seq-No tatsächlich get_at_seq()/scan_prefix_at() nutzt. Testfall: Dok A bei seq=1, Dok A' (gleiche ID, anderer Text) bei seq=2 — search_at(query_A, seq=1) muss A finden, nicht A'.

P5 TRANSACTION-AWARENESS: Werden upsert_document/delete_document TxIds korrekt an StorageEngine weitergereicht? Test: Mutation unter TxId T, Rollback von T — ist der Text-Index-Eintrag danach weg?

P6 WAND-ALGORITHMUS (wand.rs): Falls vorhanden — ist der WAND-Pruning-Algorithmus korrekt implementiert (liefert er exakt dieselben Top-k-Ergebnisse wie brute-force BM25 über alle Dokumente, nur schneller)?

TESTS:
cargo test -p contextra-text --locked -- --nocapture 2>&1 | tee /tmp/audit-text-test.log
cargo test -p contextra-text --test tombstone_read_your_writes -- --nocapture 2>&1 | tee /tmp/audit-text-bug5.log
cargo test -p contextra-text --test proptest_bm25_invariants -- --nocapture 2>&1 | tee /tmp/audit-text-prop.log
cargo clippy -p contextra-text --all-targets -- -D warnings 2>&1 | tee /tmp/audit-text-clippy.log

LIEFERGEGENSTAND: docs/audits/contextra-text_AUDIT_<YYYY-MM-DD>.md
Pflichtabschnitte: (1) Pipeline-Identitäts-Nachweis oder konkrete Diskrepanz, (2) Komposita-Tabelle (20 Fälle: Wort | Erwartet | Tatsächlich | OK/Befund), (3) Umlaut-Bimap (8 Richtungen | OK/Befund), (4) Snapshot-Isolation-Nachweis, (5) VERDICT + VERIFIED-BY-SESSION: PENDING (TS: <ISO>).
```

---

### II.13 `contextra-vector` — Ring 2, HNSW Vektorindex(~13624 LOC, gemessen)

```
Du bist Principal Senior Rust Architect für Contextra. Auftrag: Struktur-, Konfigurations- und Invarianten-Audit für crates/contextra-vector/src/ (hnsw/, diskann/, distance.rs, quantize/, acorn/, candidate_stream.rs, compute_pool.rs, persistence/, partial_rebuild.rs). Tiefenprüfung des HNSW-Algorithmus siehe Teil III.

VORBEREITUNG:
bash .jules/verify/claim_precheck.sh --prune-expired --crate contextra-vector --task "structure-audit" 2>&1
cat crates/contextra-vector/AGENTS.md
⚠NICHT-IM-REPO(cargo xtask check-vetoes) 2>&1 | grep -A5 "VETO-F02" — VETO-F02 conditional_review_due: 2026-10-07 prüfen

KONTEXT: Ring 0, #![forbid(unsafe_code)] am Crate-Root. Implementiert HNSW (Hierarchical Navigable Small World), SQ8-Quantisierung, Hardware-SIMD-Dispatch (delegiert an contextra-simd), ACORN-Filterung. VETO-F02: kein partielles HNSW-Rewiring — ausschließlich Tombstone-Pruning in partial_rebuild.rs.

PRÜFPUNKTE:

P1 ZERO-UNSAFE: grep -rn "unsafe" crates/contextra-vector/src/ — jeder Treffer außerhalb von distance.rs ist Architekturverstoß. In distance.rs: Ist es ein dünner Wrapper um contextra-simd oder direkte SIMD-Intrinsics (letzteres = offener Migrationspunkt)?

P2 FEATURE-FLAG-HYGIENE: grep -A5 "^default" crates/contextra-vector/Cargo.toml — experimental-diskann, experimental-rabitq, experimental-predicate-augmented-search, docid-128 dürfen NICHT in default sein.

P3 VETO-F02-COMPLIANCE: Finde partial_rebuild.rs::rebuild_region() (oder Äquivalent). Enthält es ausschließlich Tombstone-Pruning (Entfernen toter Knoten) ohne Re-Wiring (kein Hinzufügen/Ändern von Kanten zwischen lebenden Knoten)? Jede Kanten-Mutation zwischen lebenden Knoten = Veto-Verletzung. Datum-Check: 2026-10-07 — Review überfällig?

P4 BOUNDS-CHECKED NEIGHBOR LOAD: Finde load_node() in persistence/. Wird neighbor_count gegen max_degree (M/M_max0) geprüft? Konstruiere synthetisch korrumpierte Node-Daten mit neighbor_count > max_degree — Erwartung: Err zurück, kein Panic, kein Out-of-Bounds.

P5 QUANTISIERUNGS-DRIFT: Verifiziere in quantize/: (a) ScalarQuantizer klemmt Werte außerhalb [min, max] statt wrap/truncate. (b) quantizer_drift_threshold (Standard 10%) löst asynchronen Rebuild aus wenn überschritten. Teste mit synthetischem Datensatz der schrittweise über die trainierte Min/Max-Grenze driftet.

P6 NaN-VALIDIERUNG: ⚠NICHT-IM-REPO(cargo xtask check-nan-validation-in-hot-loop) 2>&1 | grep contextra-vector — NaN in Query-Komponenten muss validiert werden bevor HNSW-Suche startet.

P7 MAX-RESULTS GEBUNDEN: ⚠NICHT-IM-REPO(cargo xtask check-max-results-unbound) 2>&1 | grep contextra-vector — keine Suchfunktion darf unbegrenztes Vec ohne k-Limit zurückgeben.

P8 PARKING_LOT vs TOKIO LOCKS: grep -rn "tokio::sync::RwLock\|tokio::sync::Mutex" crates/contextra-vector/src/ | grep -v test — im HNSW-Hotpath VERBOTEN (AGENTS.md §6: nur parking_lot).

TESTS:
cargo test -p contextra-vector --locked -- --nocapture 2>&1 | tee /tmp/audit-vector-test.log
cargo test -p contextra-vector --test nan_validation_policy -- --nocapture 2>&1 | tee /tmp/audit-vector-bug2.log
cargo test -p contextra-vector --test deleted_nodes_lock_contention -- --nocapture 2>&1 | tee /tmp/audit-vector-bug8.log
cargo clippy -p contextra-vector --all-targets -- -D warnings 2>&1 | tee /tmp/audit-vector-clippy.log

LIEFERGEGENSTAND: docs/audits/contextra-vector_AUDIT_<YYYY-MM-DD>.md
Pflichtabschnitte: (1) unsafe-Inventar in distance.rs (Wrapper vs. direkte Intrinsics), (2) VETO-F02-Status (Tombstone-only oder Veto-Verletzung + Fälligkeitsdatum), (3) Bounds-Checked-Load-Test-Ergebnis, (4) Drift-Threshold-Test-Ergebnis, (5) VERDICT + VERIFIED-BY-SESSION: PENDING (TS: <ISO>).
```


---

## RING 1 — Storage & Persistence Layer

---

### II.14 `contextra-store` — Ring 2, LSM-Tree Storage Engine(~27025 LOC, gemessen)

```
Du bist Principal Senior Rust Architect für Contextra. Auftrag: vollständiger Invarianten-, Sicherheits- und Korrektheitsprüfung für crates/contextra-store/src/ (lsm/, wal/, sstable/, compaction/, memtable.rs, manifest/, tenant_codec.rs, system_pressure.rs). Größtes einzelnes Crate im Workspace — besondere Gründlichkeit.

VORBEREITUNG:
bash .jules/verify/claim_precheck.sh --prune-expired --crate contextra-store --task "lsm-wal-audit" 2>&1
cat crates/contextra-store/AGENTS.md
git log --oneline -10 -- crates/contextra-store/ — insbesondere "`c41afc15` fix(store): unify LsmState lock in commit paths to read lock" prüfen

KONTEXT: Ring 1, #![deny(unsafe_code)]. Implementiert vollständigen LSM-Tree: WAL (Append-Only, HMAC-Chaining, CRC32) -> MemTable (Skip-List) -> SSTable (Block-basiert, Bloom-Filter, LRU/SIEVE Block-Cache) -> Compaction. MVCC via Snapshot-Isolation. Fünf absolut-kritische Invarianten aus AGENTS.md.

PRÜFPUNKTE:

P1 FSYNC-ERROR-PROPAGATION (ABSOLUT KRITISCH): ⚠NICHT-IM-REPO(cargo xtask check-result-dropped-on-io) 2>&1 | tee /tmp/audit-store-io-check.log — dann manuell: grep -rn "let _ = .*sync_all\|let _ = .*sync_data\|let _ = .*flush" crates/contextra-store/src/ | grep -v test — JEDER Treffer ist BLOCKER. WAL-Durability-Garantie bricht sonst stillschweigend.

P2 LAST_COMMITTED_TX — SINGLE LOAD RULE: Finde get_at_seq() und scan_prefix_at(). Wird last_committed_tx exakt EINMAL am Methoden-Start in lokale Variable geladen und danach NICHT nochmals aus dem atomaren Zähler gelesen? (Concurrent Write zwischen zwei Reads = Snapshot-Isolation-Bruch).

P3 TOMBSTONE_BIT-MASKIERUNG (ADR-041): grep -rn "TOMBSTONE_BIT" crates/contextra-store/src/ — prüfe JEDEN Vergleich (entry.seq > max_seq, entry.seq <= as_of etc.): Ist immer (entry.seq & !TOMBSTONE_BIT) statt raw entry.seq verwendet? Ein einziger unmaskierter Vergleich = Phantom-Sichtbarkeit gelöschter Einträge.

P4 FLUSH-BEFORE-VISIBLE (ADR-043): In LsmStorage::flush: Wird last_committed_tx BEFORE sstables.push() aktualisiert? (Umgekehrte Reihenfolge = SSTable sichtbar bevor Tx committed — Race-Window).

P5 LOCK-HIERARCHIE-EINHALTUNG: AGENTS.md §6 definiert: write_lock (1) > memtable-RwLock (2) > sstables-RwLock (3) > snapshot_registry-Mutex (4). Prüfe ob Commit `c41afc15` ("unify LsmState lock in commit paths to read lock") diese Hierarchie korrekt umsetzt — gibt es Stellen wo (3) vor (2) akquiriert wird?

P6 WAL-HMAC-KEY-SOURCING: grep -rn "load_or_create_integrity_key\|derive_file_key" crates/contextra-store/src/ — wird ausschließlich diese Funktion für WAL-Keys genutzt (kein Literal/Hardcoded)?

P7 ATOMIC-RENAME-PATTERN: grep -rn "write_to_file\|atomic_rename\|\.rename\|rename(" crates/contextra-store/src/ | head -30 — alle Datei-Writes müssen tmp -> fsync -> rename -> dir-fsync folgen. Jeder direkter File-Write (ohne atomares Rename) = Korruptions-Risiko bei Crash.

P8 I/O-PATTERN (ADR-012): grep -rn "std::fs::File::open\|std::fs::File::create" crates/contextra-store/src/ | grep -v test | grep -v "spawn_blocking" — std::fs direkt im async-Kontext ist VERBOTEN. Nur tokio::fs für Lifecycle, std::fs NUR inside spawn_blocking.

P9 COMPACTION-TENANTGRENZE: Verliert Compaction Tenant-Isolation? Beim Merge mehrerer SSTables verschiedener Tenants: Werden Schlüssel korrekt via tenant_codec.rs getrennt behalten?

P10 LOOM-TEST ODER STRESS-TEST: Existiert ein Nebenläufigkeits-Test für gleichzeitige Reads und Writes + Compaction? cargo test -p contextra-store --test '*loom*' --locked 2>&1 | tee /tmp/audit-store-loom.log

TESTS:
cargo test -p contextra-store --locked -- --nocapture 2>&1 | tee /tmp/audit-store-test.log
cargo test -p contextra-store --test toctou_put_if_absent -- --nocapture 2>&1 | tee /tmp/audit-store-bug1.log
cargo test -p contextra-store --test manifest_corruption -- --nocapture 2>&1 | tee /tmp/audit-store-bug4.log
cargo test -p contextra-store --test wal_truncate_ordering -- --nocapture 2>&1 | tee /tmp/audit-store-bug7.log
just loom-store 2>&1 | tee /tmp/audit-store-loom2.log
cargo clippy -p contextra-store --all-targets -- -D warnings 2>&1 | tee /tmp/audit-store-clippy.log

LIEFERGEGENSTAND: docs/audits/contextra-store_AUDIT_<YYYY-MM-DD>.md
Pflichtabschnitte: (1) P1-P10 Tabelle (besonders P1 BLOCKER-Status und P2 Single-Load-Nachweis), (2) Lock-Hierarchie-Analyse mit Commit-`c41afc15`-Bezug, (3) Atomic-Rename-Pattern-Compliance-Liste, (4) Alle Bug-Proof-Test-Ergebnisse, (5) VERDICT + VERIFIED-BY-SESSION: PENDING (TS: <ISO>).
```

---

## RING 2 — Inference Backends

---

## RING 3 — Application Layer

---

### II.22 `contextra-engine` — Ring 3, Compute Pool(~19235 LOC, gemessen)

```
Du bist Principal Senior Rust Architect für Contextra. Auftrag: Audit für crates/contextra-engine/src/ (collection/, transaction/, background_workers/, chunker.rs, fusion.rs, filter.rs). Ring-0-Async-Wrapper (ADR-N02).

VORBEREITUNG:
bash .jules/verify/claim_precheck.sh --prune-expired --crate contextra-engine --task "engine-audit" 2>&1
cat crates/contextra-engine/AGENTS.md
⚠NICHT-IM-REPO(cargo xtask check-ring0-async-purity) 2>&1 | tee /tmp/audit-engine-ring0-async.log

KONTEXT: Ring 3, #![forbid(unsafe_code)]. Wraps synchrone Ring-0-Algorithmen in async via spawn_blocking. Monotone TxId-Allokation. Bounded Compute Pool.

PRÜFPUNKTE:

P1 RING-0-ASYNC-WRAPPER KORREKT: ⚠NICHT-IM-REPO(cargo xtask check-ring0-async-purity) 2>&1 — Prüft ob Ring-0-Aufrufe korrekt in spawn_blocking eingebettet sind und nicht direkt in async-Kontext blockieren.

P2 TXID-MONOTONITÄT: Ist der TxId-Zähler als AtomicU64 implementiert mit fetch_add(1, Ordering::Relaxed oder Ordering::SeqCst)? Welche Memory-Ordering-Semantik? Ist Monotonität über Threads hinweg garantiert?

P3 BACKGROUND-WORKERS SHUTDOWN: background_workers/ — Werden Background-Tasks (Expiry Cleanup, Orphan Cleanup) sauber beendet wenn die Engine gedroppt wird? Gibt es einen Graceful-Shutdown-Mechanismus?

P4 LOCK-OVER-AWAIT: grep -rn "\.lock()\|\.read()\|\.write()" crates/contextra-engine/src/ | grep -v "parking_lot\|tokio" — parking_lot-Guards DÜRFEN NICHT über .await gehalten werden (AGENTS.md §6 Invariante).

P5 COMPUTE-POOL-BOUNDEDNESS: Gibt es eine Obergrenze für gleichzeitige Aufgaben im Compute Pool? Was passiert bei Overflow — Backpressure oder Panic?

TESTS:
cargo test -p contextra-engine --locked -- --nocapture 2>&1 | tee /tmp/audit-engine-test.log
cargo clippy -p contextra-engine --all-targets -- -D warnings 2>&1 | tee /tmp/audit-engine-clippy.log

LIEFERGEGENSTAND: docs/audits/contextra-engine_AUDIT_<YYYY-MM-DD>.md
Pflichtabschnitte: (1) Ring-0-Async-Purity-Ergebnis, (2) TxId-Monotonität-Nachweis, (3) Background-Worker-Shutdown-Analyse, (4) VERDICT + VERIFIED-BY-SESSION: PENDING (TS: <ISO>).
```

---

## RING 4 — Facade & Governance

---

### II.30 `contextra` — Ring 4, Public API Facade(~1558 LOC, gemessen)

```
Du bist Principal Senior Rust Architect für Contextra. Auftrag: API-Konsistenz- und Dokumentationsaudit für crates/contextra/src/ — die public-facing Endanwender-API.

VORBEREITUNG:
bash .jules/verify/claim_precheck.sh --prune-expired --crate contextra --task "api-audit" 2>&1
⚠NICHT-IM-REPO(cargo xtask check-type-registry) 2>&1 | tee /tmp/audit-facade-type-registry.log

PRÜFPUNKTE:

P1 TYPE-REGISTRY-KONFORMITÄT: ⚠NICHT-IM-REPO(cargo xtask check-type-registry) — Alle öffentlichen Typen in docs/TYPE_REGISTRY.md registriert?

P2 DOC-COVERAGE: cargo doc -p contextra --no-deps 2>&1 | grep "warning: missing" | wc -l — öffentliche Facade muss vollständig dokumentiert sein.

P3 API-STABILITÄT: Sind semver-brechende Änderungen (Typ-Entfernung, Signatur-Änderung) durch docs/CONSTITUTION.md-Governance-Prozess gegangen?

P4 BREAKING-CHANGES-SCAN: cargo semver-checks check-release --package contextra 2>&1 | tail -20 (falls tool verfügbar).

P5 BEISPIEL-KOMPILIERBARKEIT: cargo test --doc -p contextra 2>&1 | tee /tmp/audit-facade-doctest.log — alle Dokumentations-Beispiele müssen kompilieren.

TESTS:
cargo test -p contextra --locked -- --nocapture 2>&1 | tee /tmp/audit-facade-test.log
cargo clippy -p contextra --all-targets -- -D warnings 2>&1 | tee /tmp/audit-facade-clippy.log

LIEFERGEGENSTAND: docs/audits/contextra-facade_AUDIT_<YYYY-MM-DD>.md
```

---

## TOOLING-CRATES

---

### II.33 `contextra-testkit` — Tooling, Test-Infrastruktur(~3347 LOC, gemessen)

```
Du bist Principal Senior Rust Architect für Contextra. Auftrag: Qualitätsaudit für crates/contextra-testkit/src/ (in_memory_store.rs, manual_clock.rs, fault_vfs.rs) — Testinfrastruktur die alle anderen Tests verwenden.

VORBEREITUNG:
bash .jules/verify/claim_precheck.sh --prune-expired --crate contextra-testkit --task "testkit-audit" 2>&1
cat crates/contextra-testkit/src/lib.rs

PRÜFPUNKTE:

P1 MANUAL-CLOCK MONOTONITÄT: ManualClock — Kann die Uhr rückwärts gesetzt werden (für Tests legitim, aber muss dokumentiert sein)? Unterscheidet sich die API klar von SystemClock?

P2 IN-MEMORY-STORE VOLLSTÄNDIGKEIT: Implementiert InMemoryStorageEngine ALLE Methoden von StorageEngine (kein unimplemented!() das Tests gibt die Tests-passing aber Impl missing)?

P3 FAULT-VFS VOLLSTÄNDIGKEIT: FaultVfs — Welche Fault-Typen werden simuliert (IO-Error, Partial-Write, Fsync-Failure)? Fehlen kritische Faults die Store-Tests benötigen (z.B. WAL-Korruption nach Partial-Write)?

P4 ZERO-UNSAFE: grep -rn "unsafe" crates/contextra-testkit/src/ — muss leer sein (#![forbid(unsafe_code)]).

P5 KEINE PRODUKTIONS-DEPENDENCY: Testkit darf nur Tooling-Ring-Crates und contextra-types/contextra-ports importieren — kein contextra-store o.ä. (sonst zirkuläre Abhängigkeit).

TESTS:
cargo test -p contextra-testkit --locked -- --nocapture 2>&1 | tee /tmp/audit-testkit-test.log
cargo clippy -p contextra-testkit --all-targets -- -D warnings 2>&1 | tee /tmp/audit-testkit-clippy.log

LIEFERGEGENSTAND: docs/audits/contextra-testkit_AUDIT_<YYYY-MM-DD>.md
Pflichtabschnitte: (1) InMemoryStore-Vollständigkeit (vollständige Methoden-Tabelle vs. StorageEngine-Trait), (2) FaultVfs-Fault-Typen, (3) VERDICT + VERIFIED-BY-SESSION: PENDING (TS: <ISO>).
```

---

# TEIL III — ELITE TIEFENPRÜFUNGS-PROMPTS (Algorithmen & Subsysteme)

---

### III.1 LSM-Tree-Kern — Tiefenprüfung (`lsm/engine.rs`, `commit.rs`, `recovery.rs`, `group_commit.rs`)

```
Du bist Principal Senior Rust Architect für Contextra. Auftrag: algorithmischer Tiefenaudit des LSM-Tree-Kerns in crates/contextra-store/src/lsm/ (engine.rs, commit.rs, group_commit.rs, recovery.rs, flush.rs, guard.rs, validate.rs, ops/).

VORBEREITUNG:
bash .jules/verify/claim_precheck.sh --prune-expired --crate contextra-store --task "lsm-core-deep-audit" 2>&1
cat crates/contextra-store/AGENTS.md
git log --oneline -5 -- crates/contextra-store/src/lsm/

KONTEXT: LSM-Tree ist das Storage-Herz des gesamten Systems. Daten-Verlust oder Korruption hier kaskadiert in alle Subsysteme. Commit `c41afc15` hat Lock-Strategie in Commit-Pfaden verändert — muss gründlich analysiert werden.

ALGORITHMUS-PRÜFPUNKTE:

A1 COMMIT-PFAD ATOMIZITÄT: In commit.rs — wird write_lock korrekt gehalten während WAL-Write AND MemTable-Insert (atomic als Einheit)? Wenn WAL schreibt aber MemTable-Insert fehlschlägt: wird WAL-Eintrag als orphaned markiert?

A2 GROUP-COMMIT-KORREKTHEIT (group_commit.rs): Werden gebatchte Writes in korrekter Reihenfolge committed (keine Umordnung die MVCC-Sichtbarkeit verletzt)? Was passiert wenn einer von N gruppierten Commits fehlschlägt — werden alle zurückgerollt oder nur der fehlerhafte?

A3 RECOVERY-VOLLSTÄNDIGKEIT (recovery.rs): Durchläuft der WAL-Replay beim Öffnen ALLE WAL-Segments (nicht nur das letzte)? Werden Partial-Writes am WAL-Ende (durch Crash vor fsync) korrekt als verworfen behandelt?

A4 FLUSH-ORDERING INVARIANTE (ADR-043): Finde den exakten Flush-Pfad in flush.rs. Extrahiere die Code-Zeilen für (1) MemTable-Freeze, (2) SSTable-Write+Fsync, (3) last_committed_tx-Update, (4) sstables.push(). Die Reihenfolge MUSS: 1 -> 2 -> 3 -> 4 sein. Jede Abweichung ist Race-Condition.

A5 GUARD-SEMANTIK (guard.rs): Wie implementiert der LSM-Guard den Lease? Gibt es einen Fall wo ein Guard expired ist aber noch referenziert wird?

A6 VALIDATE-CHECKS (validate.rs): Welche Invarianten prüft validate.rs vor dem Commit? Erkennt es: (a) Duplikat-Sequenznummern, (b) Tombstone ohne vorherigen Insert (logisch erlaubt aber dokumentiert?), (c) TenantId-Wechsel innerhalb einer Tx?

A7 OPS-PFADE (ops/): read.rs, write.rs, compaction.rs, maintenance.rs — Werden Reads und Writes über den sstables-RwLock korrekt koordiniert? Ist ein Write-Starvation-Szenario möglich (unendlich viele Reads blockieren Write)?

A8 LsmState LOCK-UNIFICATION (`c41afc15`): Was genau hat Commit `c41afc15` geändert ("unify LsmState lock in commit paths to read lock")? Lies den Diff via git show `c41afc15`c -- crates/contextra-store/src/ | head -100. Ist das Ersetzen durch Read-Lock korrekt — gibt es einen Pfad wo ein Commit-Pfad tatsächlich Write-Zugriff benötigt aber jetzt nur Read-Lock hält?

STRESS-TEST:
cargo test -p contextra-store --locked -- lsm 2>&1 | tee /tmp/audit-lsm-core-test.log
just loom-store 2>&1 | tee /tmp/audit-lsm-loom.log — Loom-Test für Nebenläufigkeits-Coverage

BENCHMARKS:
cargo bench -p contextra-store --bench '*' 2>&1 | grep -A5 "commit\|flush\|recovery" | tee /tmp/audit-lsm-bench.log

LIEFERGEGENSTAND: docs/audits/lsm-core_DEEP_AUDIT_<YYYY-MM-DD>.md
Pflichtabschnitte: (1) Commit-Pfad-Atomizität-Analyse mit Codezeilen-Belegen, (2) Flush-Ordering-Reihenfolge-Nachweis (exakte Zeilennummern), (3) Recovery-Vollständigkeit-Nachweis, (4) `c41afc15`-Lock-Änderung-Bewertung, (5) Group-Commit-Rollback-Semantik, (6) VERDICT + VERIFIED-BY-SESSION: PENDING (TS: <ISO>).
```

---

### III.2 WAL-Subsystem — Tiefenprüfung (`wal/encode.rs`, `io.rs`, `hmac.rs`, `replay.rs`, `flusher.rs`)

```
Du bist Principal Senior Rust Architect für Contextra. Auftrag: kryptografischer und Durability-Audit des WAL-Subsystems in crates/contextra-store/src/wal/.

VORBEREITUNG:
bash .jules/verify/claim_precheck.sh --prune-expired --crate contextra-store --task "wal-deep-audit" 2>&1
cat crates/contextra-store/AGENTS.md
cat rules/wal_crypto.md 2>/dev/null || find . -name "wal_crypto.md" | head -3

KONTEXT: WAL ist das Durability-Fundament. HMAC-Chaining (WalHmac) sichert Integrität. Flusher-Actor koordiniert async WAL-Appends. WAL-Replay bei Recovery muss vollständig sein. Bounds-Prüfung auf WAL-Einträge schützt gegen Korruption.

PRÜFPUNKTE:

W1 HMAC-CHAIN-UNUNTERBRECHBARKEIT: In hmac.rs — Verkettet jeder WAL-Eintrag seinen HMAC mit dem vorherigen Eintrag? Ist der Chain-Seed beim ersten Eintrag nach WAL-Rotation korrekt (kein Reset auf Null-Key)?

W2 BOUNDS-PRÜFUNG (io.rs): Werden eingehende WAL-Eintragslängen gegen ein Maximum geprüft bevor allokiert wird? Ein korrumpierter Längen-Header könnte sonst zu OOM-Allocation führen.

W3 CRC32-VALIDIERUNG: Wird für jeden gelesenen WAL-Block CRC32 verifiziert bevor der Inhalt verarbeitet wird? Bei Mismatch: Err statt undefined behavior?

W4 FLUSHER-ACTOR-KORREKTHEIT (flusher.rs): Wird fsync() durch den Flusher-Actor aufgerufen bevor die Erfolgs-Bestätigung an den Caller zurückgegeben wird? Ein Flusher der nur buffert aber kein fsync garantiert = keine WAL-Durabilität.

W5 PARTIAL-WRITE-RECOVERY (replay.rs): Was passiert bei einem WAL mit unvollständigem letzten Eintrag (Crash nach Schreib-Anfang vor fsync)? Erkennt replay.rs dies und truncated den WAL auf den letzten vollständigen Eintrag?

W6 WAL-ROTATION-ATOMIZITÄT: rotate_and_seal() — Wird nach Rotation garantiert, dass der neue WAL vollständig geöffnet ist bevor der alte als sealed markiert wird? Race zwischen rotate und Recovery?

W7 MULTIPLE-WAL-SEGMENTS BEI RECOVERY: Wenn mehrere WAL-Segment-Files existieren (nach mehreren Rotationen ohne Compaction): Werden ALLE beim Öffnen replayt (nicht nur das neueste)?

ENCODE/DECODE-SYMMETRIE-TEST:
cargo test -p contextra-store --locked -- wal 2>&1 | tee /tmp/audit-wal-test.log
cargo test -p contextra-store --test wal_truncate_ordering -- --nocapture 2>&1 | tee /tmp/audit-wal-bug7.log

FUZZ (wenn cargo-fuzz verfügbar):
echo "WAL-Fuzz-Targets (manuell prüfen): Corrupted WAL-Bytes, Übergroße Längenfelder, Abgeschnittene HMAC-Ketten"
just fuzz-minimize contextra-store wal_replay 2>&1 | head -30 | tee /tmp/audit-wal-fuzz.log

LIEFERGEGENSTAND: docs/audits/wal-subsystem_DEEP_AUDIT_<YYYY-MM-DD>.md
Pflichtabschnitte: (1) HMAC-Chain-Verifikation mit Codezeilen-Belegen, (2) Partial-Write-Recovery-Nachweis, (3) WAL-Segment-Recovery-Vollständigkeit, (4) Flusher-Fsync-Garantie-Nachweis, (5) VERDICT + VERIFIED-BY-SESSION: PENDING (TS: <ISO>).
```

---

### III.3 SSTable & Block Cache — Tiefenprüfung (`sstable/builder.rs`, `reader.rs`, `bloom.rs`, `block_cache.rs`)

```
Du bist Principal Senior Rust Architect für Contextra. Auftrag: Korrektheit- und Performance-Audit des SSTable-Subsystems in crates/contextra-store/src/sstable/.

VORBEREITUNG:
bash .jules/verify/claim_precheck.sh --prune-expired --crate contextra-store --task "sstable-deep-audit" 2>&1

PRÜFPUNKTE:

S1 BUILDER-SORTIERGARANTIE (builder.rs): Werden Keys innerhalb einer SSTable garantiert in lexikografischer Reihenfolge geschrieben? Was passiert wenn ein Caller Keys unsortiert eingibt (Panic, Err, oder stilles Chaos)?

S2 BLOOM-FILTER-FEHLERRATE (bloom.rs): Ist der konfigurierte False-Positive-Rate (Standard: 1%) tatsächlich erreichbar bei der implementierten Hasher-Anzahl und Filtergröße? Teste: N_keys=10000 einfügen, dann N_nonkeys=100000 prüfen — liegt FP-Rate unter 1%?

S3 BLOCK-CACHE LRU/SIEVE-KORREKTHEIT (block_cache.rs): (a) LRU: Ist die LRU-Eviction O(1)? (b) SIEVE (opt-in): Ist SIEVE als Non-Default Feature hinter Feature-Gate? Produzieren beide Implementierungen exakt dieselben Get/Put-Ergebnisse (Korrektheit-Äquivalenz)?

S4 CRC32-VERIFIKATION BEIM LESEN (reader.rs): Wird für jeden gelesenen Block CRC32 verifiziert? Bei Mismatch: Err(WalCorruption) oder Err(DataCorruption)? Wird dieser Fehler als nicht-retryable behandelt (Daten sind beschädigt, Retry ändert nichts)?

S5 SPAWN-BLOCKING-COMPLIANCE: grep -n "std::fs::File" crates/contextra-store/src/sstable/ -r — alle std::fs::File-Operationen müssen in spawn_blocking eingebettet sein (ADR-012).

S6 BLOCK-SEARCH BINARY-SEARCH-KORREKTHEIT (block_search.rs): Ist der Binary-Search auf dem SSTable-Index korrekt bei Duplicate-Keys (was nicht vorkommen sollte, aber defensiv)?

S7 READER-EXT STREAM-KORREKTHEIT (reader_ext.rs, stream.rs): Liefert ein Prefix-Scan exakt alle Keys mit dem gegebenen Präfix — kein Off-by-One am Ende des Präfixes, kein Überschreiten in den nächsten Präfix?

BENCHMARKS:
cargo bench -p contextra-store --bench '*' 2>&1 | grep -A5 "sstable\|bloom\|cache" | tee /tmp/audit-sstable-bench.log

TESTS:
cargo test -p contextra-store --locked -- sstable 2>&1 | tee /tmp/audit-sstable-test.log

LIEFERGEGENSTAND: docs/audits/sstable_DEEP_AUDIT_<YYYY-MM-DD>.md
Pflichtabschnitte: (1) Builder-Sortier-Invariante, (2) Bloom-FP-Rate-Messung, (3) Block-Cache-Korrektheit-Nachweis, (4) CRC32-Error-Handling-Analyse, (5) VERDICT + VERIFIED-BY-SESSION: PENDING (TS: <ISO>).
```

---

### III.4 Compaction Engine — Tiefenprüfung (`compaction/engine.rs`, `config.rs`)

```
Du bist Principal Senior Rust Architect für Contextra. Auftrag: Korrektheit- und MVCC-Kompatiblität-Audit der Compaction-Engine in crates/contextra-store/src/compaction/.

VORBEREITUNG:
bash .jules/verify/claim_precheck.sh --prune-expired --crate contextra-store --task "compaction-deep-audit" 2>&1

PRÜFPUNKTE:

C1 MVCC-SNAPSHOT-RESPEKT: Darf die Compaction-Engine eine Version löschen die noch von einem aktiven Snapshot (gepinnt via SnapshotRegistry) referenziert wird? Wie interagiert CompactionEngine mit min_active_seqno() aus dem MVCC-Layer?

C2 TOMBSTONE-RETENTION: Tombstones dürfen NUR eliminiert werden wenn keine Snapshot-Versionen auf die gelöschte Version zeigen können. Was ist die konkrete Bedingung im Code?

C3 TIERED vs. LEVELED: Welche Strategie ist implementiert? Tiered: Einträge mit ähnlicher Größe werden zusammengeführt. Leveled: Einträge werden in Levels organisiert mit Größengarantien. Ist die Strategie korrekt implementiert gegen ihre eigene Komplexitätstheorie?

C4 CONCURRENT-COMPACTION-SAFETY: Kann Compaction während eines aktiven Reads die SSTable-Liste inkonsistent machen? Wie wird sstables-RwLock während Compaction (lang-laufender Hintergrundoperation) gehalten?

C5 TENANT-KEY-BEIBEHALTUNG: Erhält die Compaction Tenant-Prefix in gemergten SSTables? Werden Keys verschiedener Tenants niemals vermischt?

C6 MANIFEST-ATOMIZITÄT: Wird nach dem Merge neuer SSTables das Manifest atomar aktualisiert (alter Stand bleibt gültig bis neuer vollständig geschrieben)?

TESTS:
cargo test -p contextra-store --locked -- compaction 2>&1 | tee /tmp/audit-compaction-test.log

LIEFERGEGENSTAND: docs/audits/compaction_DEEP_AUDIT_<YYYY-MM-DD>.md
Pflichtabschnitte: (1) MVCC-Snapshot-Respekt-Nachweis mit Codezeilen, (2) Tombstone-Retention-Bedingung, (3) Concurrent-Safety-Analyse, (4) VERDICT + VERIFIED-BY-SESSION: PENDING (TS: <ISO>).
```

---

### III.5 HNSW Vektorindex — Tiefenprüfung (`hnsw/core_insert.rs`, `core_search.rs`, `deletion.rs`, `arena.rs`, `types.rs`)

```
Du bist Principal Senior Rust Architect für Contextra. Auftrag: algorithmischer Tiefenaudit des HNSW-Graphalgorithmus in crates/contextra-vector/src/hnsw/.

VORBEREITUNG:
bash .jules/verify/claim_precheck.sh --prune-expired --crate contextra-vector --task "hnsw-deep-audit" 2>&1
cat crates/contextra-vector/AGENTS.md
⚠NICHT-IM-REPO(cargo xtask check-recall-stability) 2>&1 | tee /tmp/audit-hnsw-recall.log

KONTEXT: HNSW ist der Performance-kritischste Algorithmus im System. Fehler im HNSW-Graph (disconnected nodes, falsche Candidate-Selektion, Race-Conditions unter concurrent Insert+Delete) führen zu Recall-Drop ohne Fehlerindikation.

ALGORITHMUS-PRÜFPUNKTE:

H1 INSERT-KORREKTHEIT (core_insert.rs): Implementiert der Insert-Algorithmus exakt den Malkov-et-al.-2018-Algorithmus? Prüfe: (a) Layer-Selection via -ln(rand) / ln(M), (b) ef_construction-limitierter Candidate-Heap, (c) Heuristic-Neighbor-Selection bevorzugt diversere Nachbarn über nähere. Ist Heuristic-vs-Simple-Selektion korrekt implementiert (M vs. M_max0 auf Layer 0)?

H2 SEARCH-KORREKTHEIT (core_search.rs): Implementiert greedy_search korrekt: Entry-Point auf höchstem Layer, dann Layer-für-Layer absteigen mit ef-Candidates-Heap? Ist der Heap korrekt als Max-Heap für Candidates und Min-Heap für Results implementiert?

H3 DELETION-TOMBSTONE-PATTERN (deletion.rs): Wie werden gelöschte Knoten behandelt? Echtes Löschen (Re-Wiring aller Nachbarn) oder Tombstone-Only (VETO-F02: nur Tombstone erlaubt für normale Deletes)? Bei Tombstone: Werden tote Knoten bei der Suche übersprungen?

H4 ARENA-SPEICHERVERWALTUNG (arena.rs): Wie wird der Arena-Allocator für HNSW-Knoten und -Kanten verwaltet? Gibt es eine maximale Arena-Größe? Was passiert bei Overflow (OOM oder Panic)?

H5 CONCURRENT-INSERT-SAFETY: Können zwei gleichzeitige Inserts einen inkonsistenten Graphzustand erzeugen (ein Knoten hat einen Nachbarn der den Rücklink nicht kennt)? Welche Lock-Strategie schützt den Graphen während Insert?

H6 ACORN-FILTERUNG (acorn/): Die ACORN-Erweiterung erlaubt Predicate-Augmented-Search. Wird beim ersten Schritt garantiert, dass der Query-Vektor finite (non-NaN) Komponenten hat?

H7 RECALL-STABILITY: ⚠NICHT-IM-REPO(cargo xtask check-recall-stability) — Ergebnis lesen. Wie wird Recall-Stabilität gemessen (welches Testdataset, welche k-Werte)?

H8 SQ8-ASYMMETRISCHE-DISTANZ (sq8_bias.rs): Verifiziere die Bias-Korrektur für asymmetrische SQ8-Distanzberechnung (Query in Float, Index in UInt8). Ist der Bias mathematisch korrekt für das konfigurierte Distanzmaß?

RECALL-BENCHMARK:
cargo bench -p contextra-vector 2>&1 | grep -A5 "recall\|hnsw\|search" | tee /tmp/audit-hnsw-bench.log

TESTS:
cargo test -p contextra-vector --locked -- hnsw 2>&1 | tee /tmp/audit-hnsw-test.log
cargo test -p contextra-vector --test deleted_nodes_lock_contention -- --nocapture 2>&1 | tee /tmp/audit-hnsw-bug8.log
cargo test -p contextra-vector --test nan_validation_policy -- --nocapture 2>&1 | tee /tmp/audit-hnsw-bug2.log

LIEFERGEGENSTAND: docs/audits/hnsw_DEEP_AUDIT_<YYYY-MM-DD>.md
Pflichtabschnitte: (1) Malkov-2018-Algorithmus-Konformität-Analyse (H1+H2), (2) Deletion-Strategie-Bewertung vs. VETO-F02, (3) Concurrent-Safety-Nachweis, (4) Recall-Stabilität-Messung, (5) SQ8-Bias-Korrektheit, (6) VERDICT + VERIFIED-BY-SESSION: PENDING (TS: <ISO>).
```

---

### III.6 BM25 Inverted Index — Tiefenprüfung (`bm25.rs`, `inverted/`, `tokenizer.rs`, `morphology.rs`, `wand.rs`, `posting_list.rs`)

```
Du bist Principal Senior Rust Architect für Contextra. Auftrag: algorithmischer und Determinismus-Tiefenaudit des BM25-Systems in crates/contextra-text/src/.

VORBEREITUNG:
bash .jules/verify/claim_precheck.sh --prune-expired --crate contextra-text --task "bm25-deep-audit" 2>&1

PRÜFPUNKTE:

B1 BM25-FORMEL-KORREKTHEIT (bm25.rs): BM25-Score = IDF(t) * (TF(t,d) * (k1+1)) / (TF(t,d) + k1 * (1 - b + b * |d|/avgdl)). Welche k1- und b-Werte werden verwendet (Standard: k1=1.5, b=0.75)? Sind sie konfigurierbar? Verifiziere die Formel im Code gegen die Robertson-et-al.-Definition.

B2 IDF-BERECHNUNG: IDF = ln((N - df(t) + 0.5) / (df(t) + 0.5) + 1) (Robertson-IDF) oder andere Variante? Ist df(t) (Document-Frequency) korrekt als die Anzahl Dokumente die Term t enthalten (nicht Term-Frequency)?

B3 AVGDL-KONSISTENZ: Wird avgdl (Average Document Length) korrekt inkrementell bei Insert und Löschen aktualisiert? Test: 100 Dokumente einfügen (alle 10 Tokens), 50 löschen — ist avgdl noch 10?

B4 POSTING-LIST SORTIERGARANTIE (posting_list.rs): Sind Posting-Lists nach DocId sortiert (für WAND-Algorithmus notwendig)? Wird diese Sortierung bei transaktionalem Insert und Delete beibehalten?

B5 WAND-KORREKTHEIT (wand.rs): WAND (Weak AND) Pruning-Korrektheit: Liefert WAND bei k Ergebnissen exakt dieselben Top-k-Dokumente wie ein vollständiger BM25-Scan über alle Dokumente? Erstelle Korrektheit-Test: 1000 Dokumente, Query mit 3 Terms, vergleiche WAND-Top-10 gegen brute-force-Top-10.

B6 MORPHOLOGIE-KORREKTHEIT (morphology.rs): GermanCompoundSplitter — Wird ein bereits aufgeteiltes Kompositum bei erneutem Aufruf mit dem aufgeteilten Wort korrekt behandelt (Idempotenz)? Wird ein Eigenname (Muster erkennbar) vor falscher Zerlegung geschützt?

B7 STOPWORD-FILTERUNG: Sind Stopwörter sowohl im Index-Pfad als auch im Query-Pfad gefiltert? Eine Stopwort-Suche sollte 0 Ergebnisse oder alle Dokumente zurückgeben (je nach Impl) — nicht zu falschen Matches führen.

B8 INVERTED-INDEX-MVCC (inverted/): Nutzt InvertedIndex die StorageEngine-MVCC korrekt? Können parallele Transaktionen (Insert T1, Search T2 mit as_of < T1) korrekt isoliert werden?

TESTS:
cargo test -p contextra-text --locked -- --nocapture 2>&1 | tee /tmp/audit-bm25-test.log
cargo test -p contextra-text --test tombstone_read_your_writes -- --nocapture 2>&1 | tee /tmp/audit-bm25-bug5.log
cargo test -p contextra-text --test proptest_bm25_invariants -- --nocapture 2>&1 | tee /tmp/audit-bm25-prop.log

BENCHMARKS:
cargo bench -p contextra-text 2>&1 | tee /tmp/audit-bm25-bench.log

LIEFERGEGENSTAND: docs/audits/bm25_DEEP_AUDIT_<YYYY-MM-DD>.md
Pflichtabschnitte: (1) BM25-Formel mit k1/b-Werten und Codezeilen-Nachweis, (2) WAND-Korrektheit-Testnachweis (WAND vs. Brute-Force Übereinstimmung), (3) avgdl-Konsistenz-Test, (4) Morphologie-Korrektheit (Komposita-Tabelle), (5) VERDICT + VERIFIED-BY-SESSION: PENDING (TS: <ISO>).
```

---

### III.8 Krypto-Kernel — Tiefenprüfung (`wal_crypto.rs`, `deletion_proof.rs`, `ed25519_proof.rs`, `anti_tamper.rs`, `kdf.rs`)

```
Du bist Principal Senior Rust Architect für Contextra. Auftrag: kryptografischer Tiefenaudit des Crypto-Kernels in crates/contextra-crypto/src/.

VORBEREITUNG:
bash .jules/verify/claim_precheck.sh --prune-expired --crate contextra-crypto --task "crypto-deep-audit" 2>&1
cat crates/contextra-crypto/AGENTS.md
git log --oneline -5 -- crates/contextra-crypto/ — insbesondere e4632e58 (refactor duplicate) und f8c01434 (ADR-098-Dokumentation)

PRÜFPUNKTE:

K1 HKDF-ABLEITUNG KORREKTHEIT (kdf.rs): Wird HKDF (RFC 5869) korrekt implementiert? Prüfe: extract() mit korrektem Salt + IKM -> PRK, dann expand() mit PRK + Info -> OKM. Ist die Ableitung deterministisch (kein Zufall nach dem initialen Salt)?

K2 AES-256-GCM-SIV NONCE-MANAGEMENT: Wird für encrypt_auto_nonce JEDE Verschlüsselung mit einer frischen Nonce (96-bit, OsRng) versehen? Gibt es einen Pfad wo Nonce aus Kontext (z.B. Dateinamen-Hash) abgeleitet wird statt zufällig (unsicher bei Nonce-Reuse-Szenarien)?

K3 HMAC-V3-PROTOKOLL (wal_crypto.rs): verify_and_update_v3 — Was unterscheidet v3 von früheren Versionen? Ist v3 rückwärtskompatibel mit v2-WAL-Files oder gibt es einen Migration-Pfad?

K4 DELETION-PROOF-INTEGRITÄT (deletion_proof.rs): DeletionProof — Ist der Beweis kryptografisch an die spezifischen Dokument-IDs gebunden (kein Proof-Transfer: Beweis für Dok A kann nicht als Beweis für Dok B ausgegeben werden)?

K5 ED25519-VERIFIKATION (ed25519_proof.rs): Wird der Public Key gegen eine Trust-Anchor-List validiert oder kann beliebiges Key-Material für Proofs verwendet werden?

K6 ANTI-TAMPER EMERGENCY-WIPE (anti_tamper.rs): emergency_wipe() — Wird das gesamte Schlüsselmaterial sicher überschrieben (nicht nur deallokiert)? Ist Compiler-Optimierung-Unterdrückung implementiert (volatile_write oder compiler_fence)?

K7 ZEROIZE AUDIT: grep -rn "drop\|mem::forget\|ManuallyDrop" crates/contextra-crypto/src/ | grep -v test — Jedes ManuallyDrop auf Schlüsselmaterial ist Sicherheitsproblem (verhindert ZeroizeOnDrop).

K8 KDF-SALT-UNIQUENESS: Wird für jede Schlüsselableitung ein einzigartiger Salt verwendet? Wird Salt persistiert (für Reproduzierbarkeit bei Neustart)?

MUTATION-TESTING:
cargo mutants -p contextra-crypto --locked 2>&1 | tail -50 | tee /tmp/audit-crypto-deep-mutants.log — Crypto-Code sollte hohe Mutanten-Kill-Rate haben (>90%). Überlebende Mutanten = potenzielle Tests-die-nicht-testen.

TESTS:
cargo test -p contextra-crypto --locked -- --nocapture 2>&1 | tee /tmp/audit-crypto-deep-test.log

LIEFERGEGENSTAND: docs/audits/crypto-kernel_DEEP_AUDIT_<YYYY-MM-DD>.md
Pflichtabschnitte: (1) HKDF-Korrektheit-Nachweis, (2) Nonce-Management-Analyse, (3) HMAC-v3-Protokoll-Beschreibung, (4) Deletion-Proof-Binding-Nachweis, (5) Anti-Tamper-Wipe-Analyse, (6) Mutationstest-Score, (7) VERDICT + VERIFIED-BY-SESSION: PENDING (TS: <ISO>).
```

---

### III.11 SIMD-Distanz-Kernel — Tiefenprüfung (`kernels/`, `dispatch.rs`)

```
Du bist Principal Senior Rust Architect für Contextra. Auftrag: Korrektheit- und Performance-Tiefenaudit der SIMD-Kernel in crates/contextra-simd/src/.

VORBEREITUNG:
bash .jules/verify/claim_precheck.sh --prune-expired --crate contextra-simd --task "simd-deep-audit" 2>&1

PRÜFPUNKTE:

S1 KOSINUS-ÄHNLICHKEIT KORREKTHEIT (ALLE VARIANTEN): Kosinus-Ähnlichkeit = dot(a,b) / (||a|| * ||b||). Prüfe für Scalar, AVX2, AVX-512, NEON-Variante jeweils: (a) Ist die Division durch Null bei Null-Vektor abgesichert? (b) Sind ||a|| und ||b|| korrekt als L2-Norm berechnet (nicht L1)?

S2 L2-DISTANZ KORREKTHEIT: L2(a,b) = sqrt(sum((a_i - b_i)^2)). Berechnen alle Varianten korrekt das Quadrat VOR der Summe (kein sum(a_i - b_i)^2 Fehler)?

S3 DOT-PRODUCT OVERFLOW: Bei sehr großen Vektoren (dimension=4096, alle Komponenten nahe float-MAX): Kann die Dot-Product-Akkumulation zu f32-Overflow (Inf) führen? Gibt es Pairwise-Summation oder Kahan-Summation zur Numerikverbesserung?

S4 AVX-512 FALLBACK-EQUIVALENZ-TEST: Führe Kosinus-Berechnung für 1000 zufällige Vektoren-Paare mit (a) AVX-512-Kernel und (b) Scalar-Kernel durch. Maximale absolute Abweichung muss < 1e-5 sein. Gleiche Prüfung für AVX2 vs. Scalar.

S5 DISPATCH KEINE FALSCHE FEATURE-DETEKTION: Teste mit RUSTFLAGS="-C target-cpu=native" und mit einem explizit eingeschränkten CPU-Feature-Set — wählt Dispatch immer den für die vorliegende CPU optimalen Kernel?

BENCHMARKS:
cargo bench -p contextra-simd 2>&1 | tee /tmp/audit-simd-bench.log — Vergleich Scalar vs. AVX2 vs. AVX-512 bei dimension 128/512/1536/4096.

TESTS:
cargo test -p contextra-simd --locked -- --nocapture 2>&1 | tee /tmp/audit-simd-deep-test.log

LIEFERGEGENSTAND: docs/audits/simd-kernels_DEEP_AUDIT_<YYYY-MM-DD>.md
Pflichtabschnitte: (1) Kosinus-Korrektheit-Nachweis (alle 4 Varianten), (2) AVX-512-vs-Scalar-Abweichungsmessung, (3) Overflow-Analyse, (4) Benchmark-Tabelle (Variante × Dimension → ns/op), (5) VERDICT + VERIFIED-BY-SESSION: PENDING (TS: <ISO>).
```

---

# TEIL IV — CROSS-CUTTING META-PROMPTS

---

### IV.1 Ring-DAG-Integrität — Vollaudit (Feature-Flag-vollständig)

```
Du bist Principal Senior Rust Architect für Contextra. Auftrag: vollständige Ring-DAG-Integritätsprüfung des gesamten Workspace unter Berücksichtigung ALLER Feature-Flags (Default und non-Default). AUDIT_INTAKE_PROTOCOL.md §3 "Feature-Flag-vollständige Ring-Layering-Analyse" — Default-Feature-Analyse allein reicht nicht.

VORBEREITUNG:
bash .jules/verify/claim_precheck.sh --prune-expired --crate all-workspace --task "ring-dag-integrity" 2>&1

RING-MODELL (aus capabilities.toml — normative Quelle):
Ring 0: contextra-types, contextra-ports · Ring 1: contextra-sys, contextra-simd (unsafe), contextra-durable-fs, contextra-crypto · Ring 2: contextra-mvcc, contextra-store, contextra-text, contextra-vector, contextra-rank · Ring 3: contextra-engine (contextra-erasure ist Soll, leer) · Ring 4: contextra (Fassade) · dev: contextra-testkit, tests/arch

PRÜFPUNKTE:

D1 STANDARD-LAYERING: ⚠NICHT-IM-REPO(cargo xtask check-ring-layering-full) 2>&1 | tee /tmp/audit-dag-standard.log

D2 ALL-FEATURES-LAYERING: cargo metadata --all-features --format-version 1 | python3 -c "
import json, sys, collections
meta = json.load(sys.stdin)
pkg_rings = {}  # Würde aus capabilities.toml gelesen
# Ausgabe: alle Abhängigkeitskanten mit Ring-Verletzungsmarkierung
print('All-Feature dependency graph analyzed')
" 2>&1 | tee /tmp/audit-dag-allfeatures.log

D3 UNSAFE-INSELN COMPLIANCE: ⚠NICHT-IM-REPO(cargo xtask check-unsafe-islands) 2>&1 | tee /tmp/audit-dag-unsafe.log — Nur contextra-simd und contextra-sys dürfen unsafe_island=true (capabilities.toml) haben.

D4 PEER-IMPORT SCAN: Ring-0-Crates dürfen keine anderen Ring-0-Crates importieren außer contextra-types/contextra-ports. Prüfe: grep -rn "^use contextra_" crates/contextra-store/src/ — store ist Ring 1, darf contextra-core⚠ importieren aber nicht contextra-vector (Ring-0-Peer).

D5 CAPABILITIES.TOML vs. WORKING_STATE.MD DIVERGENZ: Dokumentiere explizit alle Crates wo Ring-Zuordnung in capabilities.toml von Layer-Zuordnung in WORKING_STATE.md abweicht. WORKING_STATE ist build-graph-abgeleitet (WARNUNG im Header), capabilities.toml ist normativ.

D6 ZIRKULÄRE ABHÄNGIGKEITEN: cargo build --workspace 2>&1 | grep "cyclic\|cycle" | tee /tmp/audit-dag-cycles.log — sollte leer sein.

D7 PHANTOM-IMPORTS: ⚠NICHT-IM-REPO(cargo xtask check-phantom-files) 2>&1 | tee /tmp/audit-dag-phantom.log
⚠NICHT-IM-REPO(cargo xtask check-orphan-modules) 2>&1 | tee /tmp/audit-dag-orphans.log

VOLLSTÄNDIGE MATRIX:
just dag-check 2>&1 | tee /tmp/audit-dag-just.log

LIEFERGEGENSTAND: docs/audits/ring-dag-integrity_AUDIT_<YYYY-MM-DD>.md
Pflichtabschnitte: (1) Ring-Layering-Ergebnis (Default-Features), (2) Ring-Layering-Ergebnis (All-Features), (3) Unsafe-Insel-Compliance-Tabelle, (4) Peer-Import-Verletzungen, (5) capabilities.toml vs. WORKING_STATE.md Divergenztabelle, (6) VERDICT + VERIFIED-BY-SESSION: PENDING (TS: <ISO>).
```

---

### IV.2 Zero-Panic / Unsafe — Workspace-Vollaudit

```
Du bist Principal Senior Rust Architect für Contextra. Auftrag: vollständiger Zero-Panic- und Unsafe-Audit des gesamten Workspace. AGENTS.md §6: "Kein unwrap(), expect() oder panic!() in Produktionspfaden."

VORBEREITUNG:
bash .jules/verify/claim_precheck.sh --prune-expired --crate all-workspace --task "zero-panic-unsafe-audit" 2>&1

PRÜFUNG:

Z1 UNWRAP-INVENTAR GESAMT:
just debt-audit 2>&1 | tee /tmp/audit-zeropanic-debtaudit.log

Z2 MANUELLER SCAN (präziser als debt-audit):
grep -rn "\.unwrap()\b" crates/ --include="*.rs" | grep -v "\.rs:[[:space:]]*\/\/" | grep -v "/tests/" | grep -v "_test.rs" | grep -v "benches/" | grep -v "#\[cfg(test)\]" | sort | tee /tmp/audit-zeropanic-unwrap.log | wc -l

grep -rn "\.expect(" crates/ --include="*.rs" | grep -v "/tests/" | grep -v "_test.rs" | grep -v "benches/" | grep -v "#\[cfg(test)\]" | sort | tee /tmp/audit-zeropanic-expect.log | wc -l

grep -rn "\bpanic!(" crates/ --include="*.rs" | grep -v "/tests/" | grep -v "_test.rs" | grep -v "benches/" | grep -v "#\[cfg(test)\]" | sort | tee /tmp/audit-zeropanic-panic.log | wc -l

Z3 UNSAFE VOLLSTÄNDIG-INVENTAR (außerhalb der drei erlaubten Inseln):
grep -rn "unsafe" crates/ --include="*.rs" | grep -v "contextra-crypto\|contextra-simd\|contextra-sys\|contextra-wire⚠" | grep -v "forbid\|allow\|deny\|//.*unsafe" | grep -v "/tests/" | grep -v "_test.rs" | tee /tmp/audit-zeropanic-unsafe.log | wc -l

⚠NICHT-IM-REPO(cargo xtask check-unsafe-islands) 2>&1 | tee /tmp/audit-zeropanic-islands.log

Z4 FFI-PANIC-BOUNDARY:
⚠NICHT-IM-REPO(cargo xtask check-ffi-panic-boundary) 2>&1 | tee /tmp/audit-zeropanic-ffi.log

Z5 SLICE-BOUNDS:
⚠NICHT-IM-REPO(cargo xtask lint-unsafe-slice-bounds) 2>&1 | tee /tmp/audit-zeropanic-slice.log

Z6 RESULT-DROPPED-ON-IO:
⚠NICHT-IM-REPO(cargo xtask check-result-dropped-on-io) 2>&1 | tee /tmp/audit-zeropanic-io.log

Z7 KLASSIFIZIERUNG aller Funde:
Erstelle für jeden Befund aus Z2: Datei:Zeile | Crate | Typ (unwrap/expect/panic) | Begründung (toleriert: warum? | Befund: BLOCKER/MAJOR/MINOR)

LIEFERGEGENSTAND: docs/audits/zero-panic-unsafe_WORKSPACE_AUDIT_<YYYY-MM-DD>.md
Pflichtabschnitte: (1) Zusammenfassung (Count: unwrap, expect, panic, unsafe je Crate), (2) Priorisierte Befundliste (alle BLOCKER zuerst), (3) Unsafe-Inseln-Compliance-Status, (4) Tolerierten-Liste mit Begründungen, (5) VERDICT + VERIFIED-BY-SESSION: PENDING (TS: <ISO>).
```

---

### IV.3 SOLL-vs-IST Gap-Analyse — Spezifikations-Konformität

```
Du bist Principal Senior Rust Architect für Contextra. Auftrag: systematische SOLL-vs-IST-Gap-Analyse zwischen der normativen Spezifikation (docs/SPEC.md  oder CONTEXTRA_SPEZIFIKATION.md) und dem tatsächlichen Quellcode.

VORBEREITUNG:
bash .jules/verify/claim_precheck.sh --prune-expired --crate all-workspace --task "soll-ist-gap-analysis" 2>&1
ls docs/SPEC.md  2>/dev/null || find docs -name "*.md" | head -10
cat docs/SPEC.md CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md | head -200 2>/dev/null || cat CONTEXTRA_SPEZIFIKATION*.md | head -200 2>/dev/null

METHODIK:
1. Für jeden normativen Abschnitt der Spezifikation (A-F, X, Y, INV-Tabellen): Kürze Spezifikation auf prüfbare Aussagen.
2. Für jede Aussage: Verifiziere am tatsächlichen Code-Stand (nicht am Commit-Titel).
3. Kategorisiere: IMPLEMENTIERT | TEILWEISE-IMPLEMENTIERT | OFFEN | ABWEICHEND.

SPEZIFISCHE GAP-CHECKS:

G1 INV-CALIBRATION-CONFORMAL-1 (Spec §Y.3.3): Conformal-Prediction-Kalibrierung mit Coverage-Garantie. Suche: grep -rn "conformal\|coverage_guarantee\|nonconformity" crates/ --include="*.rs" | grep -v test — implementiert oder offen?

G2 INV-PID-ANTIWINDUP-1 (Spec §Y.4.2): Bereits geprüft in III.9 — aktualisiere Status hier.

G3 MVCC-FORMEL IMPLEMENTIERT (Spec §B.2): Bereits geprüft in II.4 — Status hier eintragen.

G4 SPEC-SYNC GATE: ⚠NICHT-IM-REPO(cargo xtask check-doc-references) 2>&1 | tee /tmp/audit-gap-docref.log — jede API-Änderung muss docs/spec synchron halten.

G5 ADR-DEADLINES: ⚠NICHT-IM-REPO(cargo xtask check-adr-deadlines) 2>&1 | tee /tmp/audit-gap-adr.log — welche ADRs haben Umsetzungsfristen die überschritten sind?

G6 CAPABILITIES-TOML-SPEC-MAPPING: Für jeden Crate in capabilities.toml mit spec-Feldern: Sind die referenzierten Spec-Abschnitte tatsächlich vollständig implementiert?

G7 OFFENE AI-TAGS: grep -rn "AI-TAG\[TODO\]\|AI-TAG\[SMELL\]\|AI-TAG\[DEBT\]" crates/ --include="*.rs" | grep -v RESOLVED | tee /tmp/audit-gap-aitags.log — offene technische Schulden aus dem Code.

G8 VETO-REVIEW-ÜBERFÄLLIGKEIT: Vergleiche conditional_review_due-Daten aller VETOs gegen aktuelles Datum. VETO-F02: 2026-10-07, VETO-OP03: 2026-10-07. Beide sind nach Stand 2026-09-27 noch nicht überfällig — aber zeitnah zu eskalieren.

LIEFERGEGENSTAND: docs/audits/soll-ist-gap_AUDIT_<YYYY-MM-DD>.md
Pflichtabschnitte: (1) Gap-Matrix (Spec-Abschnitt | IST-Status | Befund | Priorität), (2) Offene INV-Tabellen-Einträge, (3) ADR-Deadline-Status, (4) VETO-Review-Kalender, (5) VERDICT + VERIFIED-BY-SESSION: PENDING (TS: <ISO>).
```

---

### IV.4 Fuzz-Kampagne — Systematische Corpus-basierte Fuzzing

```
Du bist Principal Senior Rust Architect für Contextra. Auftrag: systematische Fuzz-Kampagne für die sicherheitskritischsten Parsing- und Deserialisierungs-Pfade.

VORBEREITUNG:
bash .jules/verify/claim_precheck.sh --prune-expired --crate all-workspace --task "fuzz-campaign" 2>&1
which cargo-fuzz || cargo install cargo-fuzz
ls fuzz/fuzz_targets/ 2>/dev/null || echo "Fuzz targets inventory needed"

PRÜFUNG VORHANDENER FUZZ-TARGETS:
find . -name "fuzz_targets" -type d | xargs ls 2>/dev/null | tee /tmp/audit-fuzz-targets.log

FUZZ-PRIORIÄTSLISTE (Höchste Priorität zuerst):

F1 WAL-REPLAY (contextra-store): Malformierte WAL-Bytes — Ziel: kein Panic, nur ContextraError::WalCorruption.
just fuzz-all 300 2>&1 | grep -i "store\|wal" | tee /tmp/audit-fuzz-wal.log

F2 HNSW-PERSISTENCE (contextra-vector): Korrumpierte .hnsw-Datei beim Laden — Ziel: Err(DataCorruption) statt Panic/OOB.

F3 BM25-TOKENIZER (contextra-text): Zufällige Unicode-Strings (inkl. 0-bytes, surrogates, RLM/LRM-Zeichen) — Ziel: keine Panic, korrekte Tokenisierung oder leeres Token-Set.

F4 FLATBUFFERS-IPC (contextra-wire⚠): Korrumpierte FlatBuffer-Bytes — Ziel: Fehler statt OOB-Read.

F5 JSON-RPC-PARSING (contextra-mcp⚠): Malformierte JSON-RPC-Pakete, extrem große Payloads — Ziel: kein Panic, definierter McpError.

AUSFÜHRUNG:
just fuzz-all 300 2>&1 | tee /tmp/audit-fuzz-all.log
echo "Minimiere gefundene Crashes:"
just fuzz-minimize contextra-store wal_replay 2>&1 | head -20

COVERAGE DER FUZZ-CORPUS:
⚠NICHT-IM-REPO(cargo xtask check-coverage-gate) 2>&1 | tee /tmp/audit-fuzz-coverage.log

LIEFERGEGENSTAND: docs/audits/fuzz-campaign_AUDIT_<YYYY-MM-DD>.md
Pflichtabschnitte: (1) Fuzz-Targets-Inventar (vorhanden vs. fehlend), (2) Fuzz-Ergebnis je Ziel (Crashes: 0=✅ | >0=❌ + Reproducer), (3) Coverage-Bericht, (4) VERDICT + VERIFIED-BY-SESSION: PENDING (TS: <ISO>).
```

---

### IV.5 Mutation-Testing-Kampagne — Testgüte-Verifikation

```
Du bist Principal Senior Rust Architect für Contextra. Auftrag: Mutation-Testing-Kampagne um zu verifizieren dass Tests tatsächlich das richtige Verhalten prüfen (nicht nur Coverage haben).

VORBEREITUNG:
bash .jules/verify/claim_precheck.sh --prune-expired --crate all-workspace --task "mutation-testing" 2>&1
which cargo-mutants || cargo install cargo-mutants

PRIORITISIERTE KAMPAGNE (Crypto und Algorithmen zuerst):

M1 CRYPTO-MUTANTS (höchste Priorität):
just mutants-crypto 2>&1 | tee /tmp/audit-mutants-crypto.log
Erwartete Kill-Rate: >95%. Überlebende Mutanten in Crypto = Kritische Test-Lücken.

M2 STORE/WAL MUTANTS:
just mutants -- -p contextra-store 2>&1 | tee /tmp/audit-mutants-store.log

M3 MVCC MUTANTS:
cargo mutants -p contextra-mvcc --locked 2>&1 | tee /tmp/audit-mutants-mvcc.log

M4 RANK-FUSION MUTANTS:
cargo mutants -p contextra-rank --locked 2>&1 | tee /tmp/audit-mutants-rank.log

M5 MUTATION-SCORE GATE:
⚠NICHT-IM-REPO(cargo xtask check-mutation-score-gate) 2>&1 | tee /tmp/audit-mutants-gate.log
⚠NICHT-IM-REPO(cargo xtask record-mutation-score) 2>&1 | tee /tmp/audit-mutants-record.log

ANALYSE DER ÜBERLEBENDEN MUTANTEN:
Für jeden überlebenden Mutanten: (a) Crate, (b) Datei:Zeile, (c) Mutation-Art (z.B. `+` -> `-`, `>` -> `>=`), (d) Warum überlebt (fehlender Test oder akzeptabler Äquivalent-Mutant)?

LIEFERGEGENSTAND: docs/audits/mutation-testing_AUDIT_<YYYY-MM-DD>.md
Pflichtabschnitte: (1) Kill-Rate-Tabelle (Crate | Mutanten | Killed | Survived | Rate), (2) Analysierte überlebende Mutanten (prioritisiert nach Crate-Kritikalität), (3) Empfohlene neue Tests für kritische Überlebende, (4) VERDICT + VERIFIED-BY-SESSION: PENDING (TS: <ISO>).
```

---

### IV.6 Chaos Matrix — Fehler-Injektions-Kampagne

```
Du bist Principal Senior Rust Architect für Contextra. Auftrag: systematische Chaos-Testing-Kampagne mit FaultVfs und dem integrierten Chaos-Test-Harness.

VORBEREITUNG:
bash .jules/verify/claim_precheck.sh --prune-expired --crate all-workspace --task "chaos-matrix" 2>&1
just chaos-test 2>&1 | head -10 | tee /tmp/audit-chaos-start.log

FEHLER-SZENARIEN:

C1 IO-FAULT WÄHREND WAL-WRITE: FaultVfs injiziert IO-Fehler nach N bytes WAL-Write. Erwartung: sauberer Error-Pfad, kein Datenverlust der bereits committed Daten.

C2 FSYNC-FAULT: FaultVfs injiziert fsync-Fehler. Erwartung: ContextraError::FsyncFailed propagiert bis zum Aufrufer (nicht let _ = ignoriert).

C3 PARTIAL-WRITE: FaultVfs schreibt nur N von M bytes. Erwartung: Recovery beim nächsten Open erkennt Partial-Write und truncates.

C4 CONCURRENT-WRITE-DURING-COMPACTION: Viele parallele Writes während Compaction läuft. Erwartung: keine Deadlocks, kein Datenverlust, konsistente Sicht nach Compaction.

C5 HNSW-CONCURRENT-INSERT-DELETE: 8 Threads gleichzeitig Insert und Delete auf HnswIndex. Erwartung: kein Panic, Recall nach Abschluss > 0.9.

C6 GRAPH-CONCURRENT-ADD-EDGE: 16 Threads gleichzeitig relate() auf CsrGraph. Erwartung: alle Kanten persistent, CsrGraph intern konsistent.

C7 MEMORY-PRESSURE: System-Memory-Pressure-Simulation — wird system_pressure.rs korrekt Flush/Throttling auslösen?

AUSFÜHRUNG:
just chaos-test 2>&1 | tee /tmp/audit-chaos-full.log
cargo test --workspace --test '*chaos*' -- --nocapture 2>&1 | tee /tmp/audit-chaos-tests.log

THREAD-SANITIZER:
just tsan 2>&1 | tee /tmp/audit-chaos-tsan.log — ThreadSanitizer für Data-Race-Detection

LOOM:
just loom-store 2>&1 | tee /tmp/audit-chaos-loom.log

LIEFERGEGENSTAND: docs/audits/chaos-matrix_AUDIT_<YYYY-MM-DD>.md
Pflichtabschnitte: (1) Ergebnis je Chaos-Szenario (✅ sauber behandelt | ❌ Panic/Datenverlust/Deadlock), (2) TSan-Ausgabe (Data-Races), (3) Loom-Ergebnis, (4) VERDICT + VERIFIED-BY-SESSION: PENDING (TS: <ISO>).
```

---

### IV.7 Vollständiges Release-Gate — Pre-Release-Verifikation

```
Du bist Principal Senior Rust Architect für Contextra. Auftrag: vollständige Pre-Release-Verifikation. Dieser Prompt wird VOR jedem Release-Tag ausgeführt. Kein Release ohne ✅ auf alle Punkte.

VORBEREITUNG:
bash .jules/verify/claim_precheck.sh --prune-expired --crate all-workspace --task "release-gate" 2>&1
SESSION_HASH=$(date -u +%Y%m%d%H%M%S | sha256sum | head -c 8)
echo "RELEASE-GATE SESSION: $SESSION_HASH"

GATE 1 — BUILD-GRUNDLAGE:
cargo build --workspace --release --locked 2>&1 | tail -5 | tee /tmp/release-gate-1-build.log
echo "Gate 1: $(tail -1 /tmp/release-gate-1-build.log)"

GATE 2 — ALLE TESTS GRÜN:
just test 2>&1 | tee /tmp/release-gate-2-tests.log
echo "Gate 2 Exit: $?"

GATE 3 — BUG-PROOF-TESTS:
just prove-bugs 2>&1 | tee /tmp/release-gate-3-bugproof.log
echo "Gate 3 Exit: $?"

GATE 4 — PROPERTY-TESTS:
just prop-tests 2>&1 | tee /tmp/release-gate-4-prop.log
echo "Gate 4 Exit: $?"

GATE 5 — LINTS KOMPLETT:
just check 2>&1 | tee /tmp/release-gate-5-check.log
echo "Gate 5 Exit: $?"

GATE 6 — RING-LAYERING (ALL-FEATURES):
⚠NICHT-IM-REPO(cargo xtask check-ring-layering-full) 2>&1 | tee /tmp/release-gate-6-ring.log
echo "Gate 6 Exit: $?"

GATE 7 — ZERO-PANIC / UNSAFE:
just debt-audit 2>&1 | tee /tmp/release-gate-7-debt.log
echo "Gate 7 Exit: $?"

GATE 8 — UNSAFE-INSELN:
⚠NICHT-IM-REPO(cargo xtask check-unsafe-islands) 2>&1 | tee /tmp/release-gate-8-unsafe.log
echo "Gate 8 Exit: $?"

GATE 9 — SECURITY-AUDIT:
just security-audit 2>&1 | tee /tmp/release-gate-9-security.log
cargo deny check 2>&1 | tee /tmp/release-gate-9b-deny.log
echo "Gate 9 Exit: $?"

GATE 10 — VETO-COMPLIANCE:
⚠NICHT-IM-REPO(cargo xtask check-vetoes) 2>&1 | tee /tmp/release-gate-10-vetoes.log
echo "Gate 10 Exit: $?"

GATE 11 — ADR-DEADLINES:
⚠NICHT-IM-REPO(cargo xtask check-adr-deadlines) 2>&1 | tee /tmp/release-gate-11-adr.log
echo "Gate 11 Exit: $?"

GATE 12 — COVERAGE:
⚠NICHT-IM-REPO(cargo xtask check-coverage-gate) 2>&1 | tee /tmp/release-gate-12-coverage.log
echo "Gate 12 Exit: $?"

GATE 13 — MUTATION-SCORE:
⚠NICHT-IM-REPO(cargo xtask check-mutation-score-gate) 2>&1 | tee /tmp/release-gate-13-mutants.log
echo "Gate 13 Exit: $?"

GATE 14 — TYPE-REGISTRY:
⚠NICHT-IM-REPO(cargo xtask check-type-registry) 2>&1 | tee /tmp/release-gate-14-types.log
⚠NICHT-IM-REPO(cargo xtask check-duplicate-core-primitives) 2>&1 | tee /tmp/release-gate-14b-dupes.log
echo "Gate 14 Exit: $?"

GATE 15 — FLATBUFFERS-DRIFT:
⚠NICHT-IM-REPO(cargo xtask check-flatbuffers-drift) 2>&1 | tee /tmp/release-gate-15-flatbuf.log
echo "Gate 15 Exit: $?"

GATE 16 — SYNC-DOCS:
just sync-docs-check 2>&1 | tee /tmp/release-gate-16-docs.log
echo "Gate 16 Exit: $?"

GATE 17 — RECALL-STABILITY:
⚠NICHT-IM-REPO(cargo xtask check-recall-stability) 2>&1 | tee /tmp/release-gate-17-recall.log
echo "Gate 17 Exit: $?"

GATE 18 — AGENTS-INTEGRITY:
⚠NICHT-IM-REPO(cargo xtask check-agents-integrity) 2>&1 | tee /tmp/release-gate-18-agents.log
echo "Gate 18 Exit: $?"

GATE 19 — VOLLSTÄNDIGE QA:
just qa-full 2>&1 | tee /tmp/release-gate-19-qa.log
echo "Gate 19 Exit: $?"

GATE 20 — SUBMITTABLE:
⚠NICHT-IM-REPO(cargo xtask jules-submit-gate) --crate all 2>&1 | tee /tmp/release-gate-20-submit.log
echo "Gate 20 Exit: $?"

ZUSAMMENFASSUNG:
echo "=== RELEASE-GATE ZUSAMMENFASSUNG ===" | tee /tmp/release-gate-summary.log
for i in $(seq 1 20); do
  LOG="/tmp/release-gate-$i-*.log"
  STATUS=$(ls $LOG 2>/dev/null | xargs tail -1 2>/dev/null)
  echo "Gate $i: $STATUS"
done | tee -a /tmp/release-gate-summary.log

LIEFERGEGENSTAND: docs/audits/release-gate_<VERSION>_AUDIT_<YYYY-MM-DD>.md
Pflichtabschnitte: (1) Gate-Tabelle (Gate 1-20 | Status ✅/❌ | Exit-Code | kritische Ausgabe), (2) Offene Blocker (alle ❌ Gates), (3) Explizite GO / NO-GO Empfehlung, (4) VERDICT: GO/BLOCKED + VERIFIED-BY-SESSION: PENDING (TS: <ISO>).
ABSOLUTE REGEL: Kein Release-Tag bei einem einzigen ❌. Nicht mal ein "minor" Gate.
```