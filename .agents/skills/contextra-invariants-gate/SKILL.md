---
name: contextra-invariants-gate
description: >-
  Use this skill to enforce Contextra architectural invariants, ring hierarchy (0-4),
  Zero-Panic deny-lints (unwrap, expect, panic, slice indexing), Unsafe island bounds, and deterministic port usage.
---

# Contextra Invariants Gate – Architektur- & Qualitätsprüfung

Dieser Skill führt die formalen Gates und Architekturprüfungen nach [docs/SPEC.md §3](file:///home/freddy/Projekte/contextra/docs/SPEC.md#L186) und [AGENTS.md §2](file:///home/freddy/Projekte/contextra/AGENTS.md) durch.

## Schnell-Ausführung

```bash
bash .agents/skills/contextra-invariants-gate/scripts/check_invariants.sh
```

---

## 1. Die Invarianten im Detail

### Invariante 1: Ring-Hierarchie (P1, P5, P30)
- Abhängigkeiten dürfen nur seitwärts oder nach unten verlaufen.
- Ring 0 darf weder `tokio` noch Async-Runtimes in Produktions-Dependencies führen (P26).
- Ring 0 darf niemals von Ring 1, 2, 3 oder 4 abhängen.

### Invariante 2: Zero-Panic Richtlinie (P7)
In Produktionscode (`src/` außerhalb von `#[cfg(test)]`):
- Kein `.unwrap()`
- Kein `.expect()`
- Kein `panic!()`
- Kein `todo!()` oder `unimplemented!()`
- Kein ungeprüftes Indexing `arr[i]` (stattdessen `.get(i).ok_or(...)`)

Verstöße werden über den Clippy-Lintsatz auf `deny`-Ebene blockiert:
```bash
cargo clippy --workspace --all-targets -- \
  -D clippy::unwrap_used \
  -D clippy::expect_used \
  -D clippy::panic \
  -D clippy::indexing_slicing
```

### Invariante 3: Unsafe-Inseln (P2)
`unsafe` ist ausschließlich in diesen drei Crates gestattet:
1. `crates/contextra-simd` (AVX2 / AVX-512 Vektorbeschleunigung)
2. `crates/contextra-sys` (OS-Primitive, mlock, sync)
3. `crates/contextra-wire` (FlatBuffers Layouts)

Jeder Block **muss** einen vorangestellten Kommentar `// SAFETY:` besitzen, der die Garantien belegt.

### Invariante 4: Determinismus & Port-Purity (P28)
- Verboten in Kern-Logik:
  - `Instant::now()`
  - `SystemTime::now()`
  - `rand::thread_rng()`
  - `std::env::var()`
- Zulässig: Ports `Clock`, `Rng`, `IdGen` aus `contextra-ports`.

### Invariante 5: Vfs-Disziplin (P24)
- Direkte Zugriffe via `std::fs` oder `tokio::fs` sind im Kern verboten. Alle Dateioperationen laufen über den `Vfs`-Port.

---

## 2. Abhilfemaßnahmen bei Gate-Fehlschlägen

1. **Clippy meldet `unwrap_used`**:
   - Ersetze `.unwrap()` durch Fehlerweiterleitung mit `?` oder `.ok_or_else(|| ContextraError::...)`.
2. **Ring-Verletzung**:
   - Trenne Typdefinitionen in `contextra-types` oder Ports in `contextra-ports` aus, anstatt zyklisch zu importieren.
3. **Fehlendes `// SAFETY:`**:
   - Ergänze eine fundierte Begründung, warum Pointer gültig und Speicherbereiche aligned sind.
