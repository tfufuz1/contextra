# memfuse-calibration

`memfuse-calibration` ist ein Legacy-Crate für Score-Kalibrierung und Drift-Erkennung (Ring 0 / Legacy).

## Zweck

Historischer Crate für Isotonic/Platt-Kalibrierung und PID-Regelung. Befindet sich im Strangler-Prozess: Funktionalitäten wandern kontinuierlich nach `memfuse-rank` und `memfuse-adapt`.

## Ring-Zugehörigkeit & Status

- **Ring:** Ring 0 (Legacy Facade)
- **Status:** 🟡 In Migration / Strangler (Legacy-Crate)
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`

## Öffentliche API-Übersicht

- **Platt Scaling:** `PlattCalibrator`
- **Isotonic Regression:** `IsotonicCalibrator`
- **PID Control:** `PidController` (Re-Export)

## Architektur & Verweise

Für Details zum Strangler-Muster und der Umverteilung auf `memfuse-rank` und `memfuse-adapt` siehe [`ARCHITECTURE.md`](../../ARCHITECTURE.md) (folgt in Kürze) sowie `README.md` §20 im Repository-Root.
