# ADR-V1-0002: Modell-Wahl Qwen2.5-Coder für Contextra Local Coding Context Server

- **Status**: Akzeptiert
- **Datum**: 2026-10-21
- **Kontext**: Contextra v1.0 fokussiert auf einen lokalen Coding-Kontext-Server (Modell Qwen2.5-Coder, Kontext aus Repository-Index, exakter KV-Präfix-Cache).

## Kontext und Problemstellung

Für die Inferenz im lokalen Entwicklerkontext wird ein hocheffizientes, performantes und qualitativ hochwertiges Open-Source-Code-Modell benötigt, das direkt auf Standard-Entwickler-Hardware (CPU, Metal, CUDA) ohne externe Cloud-Abhängigkeiten läuft.

## Entscheidungsgründe und Empfohlene Varianten

Wir wählen **Qwen2.5-Coder** in folgenden zwei Ausprägungen für GGUF-native Inferenz über `contextra-infer-candle`:

1. **Qwen2.5-Coder-1.5B-Instruct Q8_0**:
   - **Einsatzbereich**: CI-Workflows, schnelle Hilfsaufgaben, ressourcenbeschränkte Umgebungen.
   - **Vorteile**: Minimale RAM- und CPU-Latenz bei hoher Präzision im 8-Bit Quantisierungsgrad.

2. **Qwen2.5-Coder-7B-Instruct Q4_K_M**:
   - **Einsatzbereich**: Standard-Entwickler-Laufzeit (Lokaler Coding-Kontext-Server).
   - **Vorteile**: Exzellente Code-Synthese und Kontextverständnis bei moderatem Arbeitsspeicherbedarf.

## KV-Cache Speicherschätzung je Token

Die nachfolgenden Werte stellen ungemessene, theoretische Schätzungen für das KV-Cache-Footprint dar:

- **7B Modell (Qwen2.5-Coder-7B)**:
  - Schichten ($L$): 28
  - KV-Köpfe ($H_{kv}$): 4
  - Kopf-Dimension ($D_{head}$): 128
  - Datentyp: f16 (2 Bytes je Element)
  - **Formel**: $2 \times L \times H_{kv} \times D_{head} \times 2 \text{ Bytes} = 2 \times 28 \times 4 \times 128 \times 2 = 57.344 \text{ Bytes}$ (~56 KiB je Token).

- **1.5B Modell (Qwen2.5-Coder-1.5B)**:
  - Schichten ($L$): 28
  - KV-Köpfe ($H_{kv}$): 2
  - Kopf-Dimension ($D_{head}$): 128
  - Datentyp: f16 (2 Bytes je Element)
  - **Formel**: $2 \times L \times H_{kv} \times D_{head} \times 2 \text{ Bytes} = 2 \times 28 \times 2 \times 128 \times 2 = 28.672 \text{ Bytes}$ (~28 KiB je Token).

## Offene Benchmark-Felder (In-Situ Messungen)

| Modell Variant | Prefill (Tokens/s) | Decode (Tokens/s) | Hardware Target |
| :--- | :--- | :--- | :--- |
| Qwen2.5-Coder-1.5B-Instruct Q8_0 | *offen* | *offen* | Apple M-Series / x86 CPU |
| Qwen2.5-Coder-7B-Instruct Q4_K_M | *offen* | *offen* | Apple M-Series / x86 CPU |

## Nicht im Scope für v1.0

- **Qwen3-Coder (MoE)**: Mixture-of-Experts Modelle sind nicht Teil der v1.0 Architektur und werden erst in späteren Meilensteinen evaluiert.
