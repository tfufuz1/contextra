# Contextra — AI-Assistenten-Kontext (`contextra-adapt`)

## Verifizierter Codestand · Ring 0 (Fachkern)

> **Für AI-Assistenten:** Diese Datei beschreibt den Crate `contextra-adapt`.
> `contextra-adapt` stellt adaptive Algorithmen bereit (LinUCB-Bandit, Lyapunov-Drift-Wächter, PID-Regler, Off-Policy IPS).
> Er ist strikt synchron (P26), frei von `tokio` und erzwingt `#![forbid(unsafe_code)]`.

---

## Crate-Topologie

- **Ring 0 Fachkern**:
  - `bandit`: LinUCB Contextual Bandit für adaptives Routing.
  - `lyapunov`: Proaktiver Distributional-Drift-Wächter.
  - `pid`: PID-Regler zur dynamischen Steuerung der Reranking-Poolgröße.
  - `offpolicy`: Inverse Propensity Scoring (IPS) zur kontrafaktischen Evaluation.
