#!/usr/bin/env bash
# =============================================================================
# contextra_rueckbau.sh
#
# Entfernt ausschließlich Dateien und Verzeichnisse, die laut SPEC 5.1,
# SPEC Anhang A, PLAN WP-1.2/1.3/1.5 und Rückbau-Dokument VOLLSTÄNDIG
# gelöscht werden dürfen. Dateien, die nur teilweise betroffen sind
# (z. B. lib.rs, audit_chain.rs, collections.rs), werden NICHT angefasst.
#
# Standard ist ein Dry-Run. Gelöscht wird erst mit --apply.
#
# Aufruf (im Repo-Wurzelverzeichnis):
#   ./contextra_rueckbau.sh                 # Dry-Run, Gruppen 1-5
#   ./contextra_rueckbau.sh --apply         # löschen (git rm), Gruppen 1-5
#   ./contextra_rueckbau.sh --apply --with-legacy --with-extras
#
# Optionen:
#   --apply            tatsächlich löschen
#   --with-legacy      Gruppe 6 (Legacy-WAL-Tests, PLAN WP-2.3) einbeziehen
#   --with-extras      Gruppe 7 (abgeleitete Waisen, nicht in den Dokumenten)
#   --only g1,g2       nur diese Gruppen (siehe unten)
#   --allow-dirty      auch bei unsauberem Arbeitsbaum löschen
#   --no-archive       Branch archive/v0-full nicht anlegen
#   -h, --help         Hilfe
#
# Gruppen:
#   crates    1  ganze Crates (SPEC 5.1)
#   delpath   2  alter Löschpfad im Kern (SPEC Anhang A, Rückbau 2.1)
#   deltests  3  Tests/Benches/Fuzz/Beispiele, die nur den Löschpfad prüfen
#   deadtests 4  Tests, die nur gegen entfernte Crates kompilieren
#   rank      5  Kalibrierung in contextra-rank (D-26) und Relations-API (D-17)
#   legacy    6  Legacy-WAL-Migrationstests (opt-in, PLAN WP-2.3, D-05)
#   extras    7  abgeleitete Waisen (opt-in, eigene Ableitung)
#
# WICHTIG: Das Skript löscht nur. Es ändert keine Cargo.toml und keine
# lib.rs. Danach bricht der Build, bis die Verwender entkoppelt sind
# (siehe Begleitbericht, Abschnitt "Pflicht-Edits").
# =============================================================================
set -euo pipefail

APPLY=0; WITH_LEGACY=0; WITH_EXTRAS=0; ALLOW_DIRTY=0; NO_ARCHIVE=0; ONLY=""

usage() { sed -n '2,38p' "$0" | sed 's/^# \{0,1\}//'; }

while [[ $# -gt 0 ]]; do
  case "$1" in
    --apply) APPLY=1 ;;
    --with-legacy) WITH_LEGACY=1 ;;
    --with-extras) WITH_EXTRAS=1 ;;
    --allow-dirty) ALLOW_DIRTY=1 ;;
    --no-archive) NO_ARCHIVE=1 ;;
    --only) shift; ONLY="${1:-}" ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unbekannte Option: $1" >&2; usage; exit 2 ;;
  esac
  shift
done

# --- Vorprüfungen ------------------------------------------------------------
ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"
if [[ -z "$ROOT" || "$ROOT" != "$(pwd -P)" ]]; then
  echo "FEHLER: Bitte im Wurzelverzeichnis des Git-Repos ausführen." >&2; exit 1
fi
if [[ ! -f Cargo.toml || ! -d crates/contextra-store || ! -d crates/contextra-crypto ]]; then
  echo "FEHLER: Das ist nicht das Contextra-Repository." >&2; exit 1
fi

# --- Tabelle: GRUPPE <TAB> PFAD <TAB> BEGRÜNDUNG -----------------------------
TABLE=$(cat <<'EOF'
crates	crates/contextra-graph	SPEC 5.1, D-17
crates	crates/contextra-checkpoint	SPEC 5.1, D-16
crates	crates/contextra-privacy	SPEC 5.1, D-24
crates	crates/contextra-ingest	SPEC 5.1, D-24
crates	crates/contextra-cognition	SPEC 5.1, D-24
crates	crates/contextra-adapt	SPEC 5.1, D-24
crates	crates/contextra-agent	SPEC 5.1, D-24
crates	crates/contextra-router	SPEC 5.1, D-24
crates	crates/contextra-kvcache	SPEC 5.1, D-13/D-24 (Cache-Verbot)
crates	crates/contextra-infer-candle	SPEC 5.1, D-24
crates	crates/contextra-infer-ollama	SPEC 5.1, D-24
crates	crates/contextra-infer-onnx	SPEC 5.1, D-24
crates	crates/contextra-sandbox	SPEC 5.1, D-24
crates	crates/contextra-audit-export	SPEC 5.1, D-24
crates	crates/contextra-avv-generator	SPEC 5.1, D-24
crates	crates/contextra-license	SPEC 5.1, D-23
crates	crates/contextra-mcp	SPEC 5.1, D-25 (v1.1-Adapter; enthält proof_key*.rs)
crates	crates/contextra-py	SPEC 5.1, D-25 (v1.1-Adapter)
crates	crates/contextra-wire	SPEC 5.1, D-29
crates	crates/contextra-core	SPEC 5.1, Konsumentenanalyse 4.4
crates	benchmarks/contextra-bench	SPEC 5.1 (nicht kompilierbar, A-B06)
delpath	crates/contextra-crypto/src/deletion_proof	Anhang A, D-13
delpath	crates/contextra-crypto/src/ed25519_proof.rs	Anhang A (Ersatz: proof_v1.rs)
delpath	crates/contextra-crypto/src/kv_shredding.rs	Anhang A, C-L12/C-L13
delpath	crates/contextra-crypto/src/kv_cipher.rs	Anhang A
delpath	crates/contextra-crypto/src/revocation_log.rs	Anhang A
delpath	crates/contextra-crypto/src/kv_segment	Anhang A
delpath	crates/contextra-store/src/scanner	Anhang A, C-L09/C-L10/C-L15/C-L20
delpath	crates/contextra-store/src/kv	Anhang A
delpath	crates/contextra-engine/src/contextra_impl/erasure.rs	Anhang A, C-L11/C-L25
delpath	crates/contextra-vector/src/hnsw/purge.rs	Anhang A, C-L23
delpath	crates/contextra-vector/src/hnsw/scanner.rs	Anhang A (alter Scanner)
delpath	crates/contextra-vector/src/hnsw/deletion.rs	Anhang A, D-18, VETO-F02
delpath	crates/contextra-text/src/inverted/purge.rs	Anhang A, D-20
delpath	crates/contextra-text/src/inverted/scanner.rs	Anhang A, C-L19
delpath	crates/contextra-ports/src/scanner	Anhang A (Scanner-Port)
delpath	crates/contextra-types/src/witness.rs	Anhang A (Zeugentypen)
deltests	crates/contextra-crypto/tests/campaign_cov_deletion_proof.rs	nur DeletionProof
deltests	crates/contextra-crypto/tests/campaign_cov_revocation_log.rs	nur RevocationLog
deltests	crates/contextra-crypto/tests/deletion_proof_external_verify_rejects_tamper.rs	nur DeletionProof
deltests	crates/contextra-crypto/tests/deletion_proof_hash_collision.rs	nur DeletionProof
deltests	crates/contextra-crypto/tests/deletion_proof_negative_and_golden.rs	nur DeletionProof
deltests	crates/contextra-crypto/tests/deletion_proof_prefix_collision_proptest.rs	nur hash_deleted_keys_length_prefixed
deltests	crates/contextra-crypto/tests/deletion_proof_signature_version_serde.rs	nur DeletionProof
deltests	crates/contextra-crypto/tests/deletion_proof_v3_validation.rs	nur DeletionProof
deltests	crates/contextra-crypto/tests/ed25519_proof_external_verify.rs	nur ed25519_proof
deltests	crates/contextra-crypto/tests/eviction_worker_tokio_runtime_nonblocking_regression.rs	nur kv_segment
deltests	crates/contextra-crypto/tests/guarantee_deletion_irrecoverable.rs	nur Löschbeweis/Shredding (F-T10)
deltests	crates/contextra-crypto/tests/kv_segment_concurrency.rs	nur kv_segment
deltests	crates/contextra-crypto/tests/kv_segment_integration.rs	nur kv_segment
deltests	crates/contextra-crypto/tests/kv_segment_proptests.rs	nur kv_segment
deltests	crates/contextra-crypto/tests/legacy_hmac_collision_property.rs	nur deletion_proof-Hashing
deltests	crates/contextra-crypto/tests/loom_eviction_worker_shutdown_race.rs	nur kv_segment
deltests	crates/contextra-crypto/tests/loom_kv_deferred_zeroize.rs	nur kv_segment
deltests	crates/contextra-crypto/tests/revocation_log_vfs.rs	nur RevocationLog
deltests	crates/contextra-crypto/benches/deletion_proof_latency_bench.rs	nur DeletionProof
deltests	crates/contextra-crypto/fuzz/fuzz_targets/deletion_proof_tamper.rs	nur DeletionProof
deltests	crates/contextra-crypto/fuzz/fuzz_targets/deletion_proof_verify_external_fuzz.rs	nur DeletionProof
deltests	crates/contextra-engine/tests/erasure_orchestrator_tests.rs	nur erasure.rs
deltests	crates/contextra-engine/tests/erasure_report_honesty.rs	nur ErasureReport/Witness
deltests	crates/contextra-engine/tests/proof_verify_not_constant.rs	nur DeletionProof
deltests	crates/contextra-store/tests/kv_delete_mode_wiring_test.rs	nur store::kv
deltests	crates/contextra-text/tests/text_scanner_purger_tests.rs	nur TextScanner/TextPurger
deltests	crates/contextra-vector/tests/hnsw_scanner_purge_test.rs	nur HNSW-Scanner/Purger
deltests	crates/contextra/tests/review_stub_proof.rs	nur Stub-Löschbeweise
deltests	crates/contextra/tests/deletion_chain_e2e.rs	nur alte Löschkette (C-L01)
deltests	crates/contextra-db/tests/deletion_proof_integration.rs	nur DeletionProof
deltests	crates/contextra-db/examples/verify_deletion_proof_external.rs	nur DeletionProof
deadtests	crates/contextra/tests/license_gate_enforcement.rs	nur contextra-license (D-23)
deadtests	crates/contextra/tests/license_gate_always_enforced.rs	nur contextra-license (D-23)
deadtests	crates/contextra-db/tests/checkpoint_layer_bounds.rs	nur contextra-checkpoint
deadtests	crates/contextra-db/tests/checkpoint_stress.rs	nur contextra-checkpoint
deadtests	crates/contextra-db/tests/instance_orphan_registry_test.rs	nur contextra-checkpoint
deadtests	crates/contextra-db/tests/cascade_invalidation_integration.rs	nur contextra-graph (PathRAG)
deadtests	crates/contextra-db/tests/cascade_invalidation_test.rs	nur contextra-graph (PathRAG)
deadtests	crates/contextra-db/tests/graph_entity_delete_test.rs	nur contextra-graph (CsrGraph)
deadtests	crates/contextra-db/tests/supersedes_cascading_tombstone_test.rs	nur contextra-graph (PathRAG)
rank	crates/contextra-rank/src/calibration	D-26 (Kalibrierung entfällt)
rank	crates/contextra-rank/tests/calibration_deep_tests.rs	nur Isotonic/Platt
rank	crates/contextra-rank/tests/calibration_stress_and_fault_tests.rs	nur Isotonic/Platt
rank	crates/contextra-rank/tests/calibration_migration_snapshot_test.rs	nur Isotonic/Platt
rank	crates/contextra-rank/tests/conformal_tests.rs	nur Conformal
rank	crates/contextra-rank/tests/isotonic_mutation_hardening_test.rs	nur Isotonic
rank	crates/contextra-engine/src/collection/relate.rs	D-17, PLAN WP-1.1 (Relations-API)
legacy	crates/contextra-store/tests/hmac_legacy_key_migration.rs	D-05, WP-2.3
legacy	crates/contextra-store/tests/wal_legacy_key_forced_migration.rs	D-05, WP-2.3
legacy	crates/contextra-store/tests/wal_legacy_key_migration_store.rs	D-05, WP-2.3
legacy	crates/contextra-store/tests/wal_legacy_key_retirement_marker.rs	D-05, WP-2.3
legacy	crates/contextra-store/tests/wal_legacy_key_tests.rs	D-05, WP-2.3
legacy	crates/contextra-store/tests/wal_legacy_migration_crash.rs	D-05, WP-2.3
extras	crates/contextra-ports/src/kv.rs	KV-Cache-Port, nach D-24 ohne Verwender
extras	crates/contextra-ports/src/kv_bridge_port.rs	KV-Bridge-Port, nach D-24 ohne Verwender
extras	benchmarks/relate_bench.rs	Relations-API (D-17), von keiner Cargo.toml gebaut
extras	mcp.json	startet contextra-mcp-server (D-25)
extras	contextra_data	eingecheckte Laufzeitdaten inkl. .wal_integrity_key
EOF
)

# --- Gruppenauswahl ----------------------------------------------------------
declare -A ACTIVE=([crates]=1 [delpath]=1 [deltests]=1 [deadtests]=1 [rank]=1 [legacy]=0 [extras]=0)
[[ $WITH_LEGACY -eq 1 ]] && ACTIVE[legacy]=1
[[ $WITH_EXTRAS -eq 1 ]] && ACTIVE[extras]=1
if [[ -n "$ONLY" ]]; then
  for g in "${!ACTIVE[@]}"; do ACTIVE[$g]=0; done
  IFS=',' read -ra SEL <<< "$ONLY"
  for g in "${SEL[@]}"; do
    [[ -v ACTIVE[$g] ]] || { echo "FEHLER: unbekannte Gruppe '$g'" >&2; exit 2; }
    ACTIVE[$g]=1
  done
fi

# --- Sicherheitsprüfungen für --apply ---------------------------------------
if [[ $APPLY -eq 1 ]]; then
  if [[ $ALLOW_DIRTY -eq 0 && -n "$(git status --porcelain)" ]]; then
    echo "FEHLER: Arbeitsbaum ist nicht sauber. Erst committen/stashen oder --allow-dirty setzen." >&2
    exit 1
  fi
  if [[ $NO_ARCHIVE -eq 0 ]]; then
    if git show-ref --verify --quiet refs/heads/archive/v0-full; then
      echo "Archiv-Branch archive/v0-full existiert bereits."
    else
      git branch archive/v0-full
      echo "Archiv-Branch archive/v0-full auf $(git rev-parse --short HEAD) angelegt (D-22)."
    fi
  fi
fi

# --- Ausführung --------------------------------------------------------------
n_paths=0; n_files=0; n_missing=0
declare -A PER_GROUP_FILES=()
MISSING=()

while IFS=$'\t' read -r grp path why; do
  [[ -z "${grp:-}" ]] && continue
  [[ "${ACTIVE[$grp]:-0}" -eq 1 ]] || continue

  # Pfadsicherheit: relativ, kein "..", muss unter dem Repo-Root liegen
  if [[ "$path" = /* || "$path" == *..* ]]; then
    echo "ABBRUCH: unzulässiger Pfad '$path'" >&2; exit 1
  fi

  if [[ ! -e "$path" ]]; then
    MISSING+=("$path"); n_missing=$((n_missing+1)); continue
  fi

  cnt=$(find "$path" -type f | wc -l | tr -d ' ')
  n_paths=$((n_paths+1)); n_files=$((n_files+cnt))
  PER_GROUP_FILES[$grp]=$(( ${PER_GROUP_FILES[$grp]:-0} + cnt ))

  if [[ $APPLY -eq 1 ]]; then
    if git ls-files --error-unmatch -- "$path" >/dev/null 2>&1; then
      git rm -r -q -f -- "$path"
    fi
    [[ -e "$path" ]] && rm -rf -- "$path"
    printf 'GELÖSCHT  %-4s %s (%s Dateien)\n' "$grp" "$path" "$cnt"
  else
    printf 'DRY-RUN   %-9s %s (%s Dateien)  # %s\n' "$grp" "$path" "$cnt" "$why"
  fi
done <<< "$TABLE"

# --- Zusammenfassung ---------------------------------------------------------
echo
echo "================ Zusammenfassung ================"
for g in crates delpath deltests deadtests rank legacy extras; do
  [[ "${ACTIVE[$g]}" -eq 1 ]] && printf '  %-10s %6s Dateien\n' "$g" "${PER_GROUP_FILES[$g]:-0}"
done
printf '  Pfade: %s   Dateien gesamt: %s   bereits fehlend: %s\n' "$n_paths" "$n_files" "$n_missing"
if [[ ${#MISSING[@]} -gt 0 ]]; then
  echo "  Nicht vorhanden (übersprungen):"; printf '    - %s\n' "${MISSING[@]}"
fi
if [[ $APPLY -eq 0 ]]; then
  echo
  echo "Dry-Run: es wurde nichts gelöscht. Mit --apply ausführen."
else
  echo
  echo "Fertig. Die Änderungen sind gestaged (git rm), noch nicht committet."
  echo "Als Nächstes: Cargo.toml (members, default-members, workspace.dependencies,"
  echo "Features) und die lib.rs der behaltenen Crates entkoppeln (Pflicht-Edits)."
fi
