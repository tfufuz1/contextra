#!/usr/bin/env bash
cat << 'EOF'
{
  "injectSteps": [
    {
      "systemMessage": {
        "systemMessage": "Contextra Workspace aktiv. Richtlinien aus AGENTS.md und docs/SPEC.md sind normativ bindend (Zero-Panic, Ring-Hierarchie, Tenant-Isolation VETO-F10, Löschkern nach SPEC 11, Version 4.3.0). Nutze Skills: /contextra-doctor, /contextra-deletion-verifier, /contextra-security-audit, /contextra-invariants-gate, /contextra-product-suite."
      }
    }
  ]
}
EOF
