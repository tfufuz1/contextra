---
source: CONTEXTRA_SPEC_v4_MASTER.md
chapter: "91"
---
# Anhang C — Änderungsprotokoll Fassung 2.1 und Prüfnachweise

## C.1 Geändert oder ergänzt

| Bereich | Änderung | Grund |
|---|---|---|
| Kopf, ToC | Teil A ergänzt (Rekonstruktion, §A.1–§A.5); Verweis auf die nicht vorhandene Tabelle A2.9 → `§A2.1`; Verweis `§4.5` → `§4.2`; Leitentscheidung (4) | Teil A fehlte, A2.9 und §4.5 existierten nicht |
| §4.i | Systeminvarianten §4(1)–§4(7) definiert, mit Lokalität P24 als (6) | „§4(n)"/„Invariante n" waren nirgends definiert |
| §5.2a | `key_hash` in `KvKeyLocks` (feste Seeds), `acquire` mit `Result` | `RandomState::new()` pro Aufruf ⇒ instabile Shards |
| §5.3 | SPSC-`MaybeUninit`-Ring → begrenzter MPSC-Group-Commit-Actor | mehrere Produzenten, `unsafe` im Safe-Crate, `hmac_prev` vom Produzenten |
| §5.4 | `crossbeam-epoch`-SIEVE → sicherer Entwurf; LRU poison-tolerant | `unsafe`, `!Send`/`!Sync`, `.unwrap()` |
| §6.3, §6.4 | `Arc`-Payload-Sharing, capacity-basierte Schätzung, Spitzenschätzung, Regel S1 | tiefer Klon je Schreibzugriff, fehlender Rehash-Peak |
| §6.6 H2 | Hash über `kv_locks.key_hash`; `Arc::from(participants)` | s. o. |
| §6.6 H5, §6.7 | persistente idempotente Cascade-Queue, `Ok(CascadeReport)`, `PartialCascadeQueued` entfällt | In-Memory-Queue geht bei Absturz verloren |
| §6.6 H6, §7.5 | Konvention K, korrekter Beweis, Iterator über Snapshot | Beweis falsch, Iterator klont alle Hyperkanten |
| §7.4 | SQ8-Bias-Kalibrierung (gemessen) | absolute Schwellen verschieben sich |
| §8.1, §8.2, §8.5 | `Result`-Trait, Laufzeit-Dimension, Discount `γ⁻¹`, IPS-Randomisierung | Compile-Fehler, Richtungsfehler, Propensity 1 |
| §9.3, §A2.4 | Nonce-Zähler als offene Entscheidung | Zähler beginnt nach Neustart bei 0 |
| §15, §16.2 | AK-9 bis AK-15, AK-2 präzisiert, `check-unwrap-baseline` → `clippy-panic-lints` | Ratchet entfällt laut §0.4 |
| Anhang B | Block ans Ende verschoben, `B.`-Nummerierung, Skizzen korrigiert | Nummernkollision, widersprüchliche normative Skizzen |

## C.2 Bewusst nicht aus dem neueren Dokument übernommen

SIEVE-Skizze (`unsafe`, `!Send`), Bandit-Code (`[f32; D*D]`, `unsafe` im Router), WAL als SPSC-Ring, AVX-512-SQ8-Kern
(Repo hat `dot_product_u8_avx512vnni`), `scc::HashMap` im Snapshot, deterministischer Nonce-Zähler, Zeroize-on-Panic
als Panic-Schutz im `release-abort`-Profil, Ring-Puffer für die WAL-Queue.

## C.3 Prüfnachweise (Sandbox, rustc 1.75, Repo-Stand `729b19b1`)

- `ahash::RandomState::new().hash_one(42)` liefert je Aufruf und je Prozesslauf verschiedene Werte; `with_seeds` ist stabil.
- `AlignedVector<{ D * D }>` scheitert mit „generic parameters may not be used in const operations"; `Shared<'static, T>` ist weder `Send` noch `Sync`.
- Kompiliert und ausgeführt: `KvKeyLocks`, `ShermanMorrisonBandit` (inkl. `discount_once`, Dimensionsfehler), `SieveCacheBackend` (`Send + Sync`), `WalHandle` (Backpressure, geschlossener Flusher), `GraphInner`-Schätzung (`size_of::<RoleBinding>() = 16`), `star_weight`.
- Star-Expansion: Schur-Komplement gleich Clique-Laplace-Matrix für N ∈ {2,3,5,8,20} (Abweichung ≈ 1e-16); alte Formel bei N=3: Gradsumme 1,5w gegen 6w.
- SQ8 (synthetisch, D=384): Rundungsenergie 4,88e-5 gegen Σ Δ²/12 = 4,89e-5; Netto-Offset bei Gauß-Daten mit 99,9-%-Clipping ≈ −75·c_q, bei Gleichverteilung ≈ 2·c_q — daher gemessener Bias statt analytischer Konstante.
- Repo-Befunde: tiefer Klon in `InnerWriteGuard::drop`; `len()·size_of`-Schätzung; `StarExpansionIterator::new` mit `.cloned().collect()` und Gewicht `w`; Cascade-Queue nur als In-Memory-`VecDeque`, `enqueue` nur in Tests; WAL-Flusher mit `unbounded_channel` und `oneshot`-Ack nach `sync_all`; Bandit mit `γ⁻¹` und Laufzeit-Dimension; keine Propensity-Protokollierung; `OsRng`-Nonces.

## C.4 Offen und nicht geprüft

- Teil A ist eine Rekonstruktion (Kennzeichnung **[F2.1]**); die ursprüngliche Fassung ersetzt sie.
- Die Konvention K (Faktor 2, Gesamtmasse w) ist ein Arbeitsstand (§A2.4 Nr. 6).
- SQ8-Zahlen beruhen auf synthetischen Daten; die Nachmessung auf echten Embeddings steht aus.
- Der O(H)-Klon je Veröffentlichung bleibt (§6.3, Restkosten).
- Nicht geändert, aber aufgefallen: `quick_cache = "0.5"` in §0.2 gegen `0.6` im Repo; `unreachable!()` in `quantize.rs` des Repos widerspricht §4(1).

---

<a id="anhang-d"></a>
