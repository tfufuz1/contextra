# Harness State & Local Directory Architecture (`.jules/local/`)

## Architecture & Purpose

The `.jules/local/` directory serves as an isolated, untracked local state storage for session governance tools, iteration loop guards, environment attestation, and PR body generation.

```
.jules/local/
├── env-attest.json     # Output of cargo xtask env-attest
├── loop-guard.json     # Error hash counts and file edit counters
├── ESCALATE.md         # Auto-generated escalation report on loop stop
├── PR_BODY.md          # Generated pull request description body
└── sessions/           # Session logs and telemetry (session 7)
```

## State Lifecycle & Governance

1. **Transient Creation**: State files in `.jules/local/` are dynamically created during a local agent session or CI execution.
2. **Local Advisory Only**: Local state files are strictly **advisory** and are never treated as trust anchors ("Vertrauensanker"). Bindings and proofs must be validated independently in CI via reproducible tree hashes and official CI ledgers.
3. **Untracked Exclusion**: `.jules/local/` is ignored in `.gitignore` to prevent session leaks and dynamic commit churn across parallel sessions.
