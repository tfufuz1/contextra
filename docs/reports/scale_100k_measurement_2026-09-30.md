Misst ausschließlich Speicher- und Indexlatenz. Embedding-Inferenz addiert je nach Modell und Hardware 10-500 ms pro Anfrage.

# Contextra — Gemessener 100k-Lauf und Recovery-Nachmessung

**Datum:** 2026-09-30
**Status:** Abbruch (Memory budget exceeded)
**Rohdatei-Verweis:** `benchmarks/results/scale_100k_2026-09-30.json`

---

## 1. Systemumgebung

- **Datum des Laufs:** 2026-09-30
- **Git Commit (HEAD):** `30c3e80889d42c6305bc70900a8614f7c0edf0f5`
- **Compiler:** `rustc 1.89.0 (29483883e 2025-08-04)`
- **CPU Cores (`nproc`):** 4 Cores
- **Arbeitsspeicher (`free -m`):** 7959 MiB Gesamt / 7469 MiB Verfügbar
- **Betriebssystem:** Linux x86_64 (Jules Sandbox VM)

---

## 2. Ausgeführter Benchmark-Befehl

Gemäß `benchmarks/README.md` im Release-Profil ausgeführt:

```bash
cargo run -p contextra-bench --release --bin scale-100k -- --docs 100000 --step 10000 --out benchmarks/results/scale_100k_2026-09-30.json
```

**Zeitlimit:** Fixed Limit 60 Minuten.

---

## 3. Messergebnisse & Abbruchdokumentation

Der Skalierungslauf wurde gestartet, brach jedoch vor Erreichen des ersten Intervalls (10.000 Dokumente) bei 0 erfolgreich abgeschlossenen Intervallen ab.

### Zwischenwerte je 10.000 Dokumente

| Dokumente | Einfügerate (docs/s) | Suchlatenz p50 (ns) | Suchlatenz p95 (ns) | Suchlatenz p99 (ns) | Speicher-Peak (RSS MB) | Status |
|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| **10.000** | — | — | — | — | — | **N/A (Abbruch vor Intervall 1)** |

*Hinweis:* Nichts wurde als Messwert extrapoliert. Nur real gemessene Werte werden ausgewiesen.

### Fehler- & Abbruchbeschreibung

- **Abbruchstelle:** `aborted_at_docs: 0` (während der Einfügung der ersten Charge)
- **Fehlermeldung:**
  ```text
  Error during insertion: Transaction error: Storage error: Memory budget exceeded (95%)
  ```
- **Ursachenanalyse:** Das Binary `scale-100k` nutzt standardmäßig `ContextraConfig::default()`, bei dem ein RAM-Limit von `max_ram_mb: 2048` (2048 MB) konfiguriert ist. Während des Einfügens der Vektoren und Metadaten stieg der RSS-Speicher der LSM-Storage-Engine auf ca. 2.05 GB an, womit die 95%-Kapazitätsschwelle des internen `ResourceTracker` überschritten wurde. Folgende Einfügeoperationen wurden daraufhin von der Storage-Engine abgelehnt.

---

## 4. Prüfbefehl für KV-Recovery bei 1 Mio. Einträgen

In `benchmarks/README.md` existiert kein dokumentierter Befehl für das Ausführen eines KV-Recovery-Benchmarks bei 1 Mio. Einträgen. Gemäß den Vorgaben wird daher kein Befehl ausgeführt und keine neue Konfiguration angelegt.

---

## 5. Referenz auf Rohdaten

Die maschinenlesbare Rohausgabe des Laufs befindet sich unter:
`benchmarks/results/scale_100k_2026-09-30.json`
