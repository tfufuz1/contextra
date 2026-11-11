# Audit- & Messreport: `contextra-ingest`

- **Crate-Pfad:** `crates/contextra-ingest`
- **Gesamtstatus:** **BESTANDEN (PASS)**
- **Test-Statistik:** 156 bestanden, 0 fehlgeschlagen, 1 ignoriert
- **Kompilierung (cargo check):** ✅ Erfolgreich (3.5185s)

---

## 1. Komponenten & Abhängigkeiten

- **Abhängigkeiten (18):** `contextra-types`, `serde`, `serde_json`, `thiserror`, `blake3`, `unicode-segmentation`, `zip`, `quick-xml`, `encoding_rs`, `html5ever`, `markup5ever_rcdom`, `calamine`, `csv`, `lopdf`, `cfb`, `mail-parser`, `yaml-rust2`, `toml`
- **Features (17):** `default`, `fmt-text`, `fmt-web`, `fmt-docx`, `fmt-pptx`, `fmt-odf`, `fmt-sheets`, `fmt-pdf`, `fmt-ole`, `fmt-email`, `fmt-structured`, `zip-container`, `xml`, `all-formats`, `fuzz`, `workspace`, `types-compat`

---

## 2. Test-Ergebnisse im Detail

### Unit-Tests (`--lib`)
- **Status:** ✅ Bestanden
- **Laufzeit:** 0.4173s
- **Tests bestanden:** 32
- **Tests fehlgeschlagen:** 0
- **Tests ignoriert:** 0

### Integrationstests (`--tests`)
| Test-Target | Status | Bestanden | Fehlgeschlagen | Laufzeit (s) |
|---|---|---|---|---|
| `__all__` | ✅ OK | 124 | 0 | 17.8361s |

### Dokumentationstests (`--doc`)
- **Status:** ✅ Bestanden
- **Laufzeit:** 0.6979s
- **Tests bestanden:** 0

---

## 3. Performance & Benchmark-Messungen

| Benchmark-Target | Status | Ausführungszeit (s) |
|---|---|---|
| `bm_ingest` | ⏱️ TIMEOUT | 20.0202s |

---

## 4. Aufgetretene Fehler & Compiler-Befunde

_Keine Compiler- oder Systemfehler registriert._
