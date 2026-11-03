//! Liste bekannter Abweichungen von den Architektur- und Lint-Regeln.
//!
//! Schema je Eintrag: (Crate, Regel, Arbeitspaket, Grund)
//!
//! Jede Sitzung entfernt ihre Einträge in ihrem jeweiligen PR.
//! Ein Eintrag ohne gültiges Arbeitspaket (`WP-...`) ist ein Testfehler.
//! Am Ende aller Arbeitspakete ist diese Liste vollständig leer.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Deviation {
    pub crate_name: &'static str,
    pub rule: &'static str,
    pub work_package: &'static str,
    pub reason: &'static str,
}

pub const KNOWN_DEVIATIONS: &[Deviation] = &[
    // Kantenabweichungen zu SPEC §5.3 (AR-1, D-88, D-89):
    Deviation {
        crate_name: "contextra-vector",
        rule: "T-ARCH-3:contextra-mvcc",
        work_package: "WP-8.1",
        reason: "vector hängt noch an mvcc; Entkopplung nach D-88 (2) erfolgt in WP-8.1",
    },
    Deviation {
        crate_name: "contextra-vector",
        rule: "T-ARCH-3:contextra-sys",
        work_package: "WP-8.1",
        reason: "vector hängt noch an sys; mlock/madvise über Port in engine erfolgt in WP-8.1",
    },
    Deviation {
        crate_name: "contextra",
        rule: "T-ARCH-3:contextra-crypto",
        work_package: "WP-1.5",
        reason: "Fassade contextra re-exportiert noch Krypto-Typen; Bereinigung nach D-89 erfolgt in WP-1.5",
    },
    Deviation {
        crate_name: "contextra",
        rule: "T-ARCH-3:contextra-ports",
        work_package: "WP-1.5",
        reason: "Fassade contextra hängt noch an ports; Bereinigung nach D-89 erfolgt in WP-1.5",
    },
    Deviation {
        crate_name: "contextra",
        rule: "T-ARCH-3:contextra-rank",
        work_package: "WP-1.5",
        reason: "Fassade contextra re-exportiert noch rank; Bereinigung nach D-89 erfolgt in WP-1.5",
    },
    Deviation {
        crate_name: "contextra",
        rule: "T-ARCH-3:contextra-store",
        work_package: "WP-1.5",
        reason: "Fassade contextra hängt noch direkt an store; Schnitt nach D-89 erfolgt in WP-1.5",
    },

    // T-ARCH-5: Ring im Manifest [package.metadata.contextra] ring vs SPEC §5.2 Matrix
    Deviation {
        crate_name: "contextra-simd",
        rule: "T-ARCH-5:manifest-ring",
        work_package: "WP-1.5",
        reason: "Manifest hat ring = 0, laut SPEC §5.2 Ring 1; Aktualisierung in WP-1.5",
    },
    Deviation {
        crate_name: "contextra-crypto",
        rule: "T-ARCH-5:manifest-ring",
        work_package: "WP-1.5",
        reason: "Manifest hat ring = 0, laut SPEC §5.2 Ring 1; Aktualisierung in WP-1.5",
    },
    Deviation {
        crate_name: "contextra-mvcc",
        rule: "T-ARCH-5:manifest-ring",
        work_package: "WP-1.5",
        reason: "Manifest hat ring = 0, laut SPEC §5.2 Ring 2; Aktualisierung in WP-1.5",
    },
    Deviation {
        crate_name: "contextra-store",
        rule: "T-ARCH-5:manifest-ring",
        work_package: "WP-1.5",
        reason: "Manifest hat ring = 1, laut SPEC §5.2 Ring 2; Aktualisierung in WP-1.5",
    },
    Deviation {
        crate_name: "contextra-vector",
        rule: "T-ARCH-5:manifest-ring",
        work_package: "WP-1.5",
        reason: "Manifest hat ring = 0, laut SPEC §5.2 Ring 2; Aktualisierung in WP-1.5",
    },
    Deviation {
        crate_name: "contextra-text",
        rule: "T-ARCH-5:manifest-ring",
        work_package: "WP-1.5",
        reason: "Manifest hat ring = 0, laut SPEC §5.2 Ring 2; Aktualisierung in WP-1.5",
    },
    Deviation {
        crate_name: "contextra-rank",
        rule: "T-ARCH-5:manifest-ring",
        work_package: "WP-1.5",
        reason: "Manifest hat ring = 0, laut SPEC §5.2 Ring 2; Aktualisierung in WP-1.5",
    },

    // T-ARCH-5: Ring im //! Kopf von src/lib.rs

    Deviation {
        crate_name: "contextra-crypto",
        rule: "T-ARCH-5:header-ring",
        work_package: "WP-1.5",
        reason: "Kopf in lib.rs nennt noch Ring 0 statt Ring 1; Aktualisierung in WP-1.5",
    },
    Deviation {
        crate_name: "contextra-mvcc",
        rule: "T-ARCH-5:header-ring",
        work_package: "WP-1.5",
        reason: "Kopf in lib.rs nennt Ring 2 noch nicht normativ; Aktualisierung in WP-1.5",
    },
    Deviation {
        crate_name: "contextra-store",
        rule: "T-ARCH-5:header-ring",
        work_package: "WP-1.5",
        reason: "Kopf in lib.rs nennt noch Ring 1 statt Ring 2; Aktualisierung in WP-1.5",
    },
    Deviation {
        crate_name: "contextra-vector",
        rule: "T-ARCH-5:header-ring",
        work_package: "WP-1.5",
        reason: "Kopf in lib.rs nennt Ring 2 noch nicht normativ; Aktualisierung in WP-1.5",
    },
    Deviation {
        crate_name: "contextra-text",
        rule: "T-ARCH-5:header-ring",
        work_package: "WP-1.5",
        reason: "Kopf in lib.rs nennt Ring 2 noch nicht normativ; Aktualisierung in WP-1.5",
    },
    Deviation {
        crate_name: "contextra-engine",
        rule: "T-ARCH-5:header-ring",
        work_package: "WP-1.5",
        reason: "Kopf in lib.rs nennt Ring 3 noch nicht normativ; Aktualisierung in WP-1.5",
    },

    // T-UNSAFE-1: Unsafe in Nicht-Inseln
    Deviation {
        crate_name: "contextra-store",
        rule: "T-UNSAFE-1:tests/chaos_dropped_write.rs",
        work_package: "WP-3.3",
        reason: "Chaos-Test nutzt raw libc::dup2 / /proc/self/fd zur FS-Fehlersimulation; Umstellung auf FaultVfs in WP-3.3",
    },
    Deviation {
        crate_name: "contextra-text",
        rule: "T-UNSAFE-1:tests/alloc_profiler.rs",
        work_package: "WP-7.1",
        reason: "Allocation Profiling Test Harness implementiert GlobalAlloc mit unsafe; Bereinigung/Auslagerung in WP-7.1",
    },

    // T-ARCH-6: dev-Abhängigkeiten (AR-14, D-97)
    Deviation {
        crate_name: "contextra-crypto",
        rule: "T-ARCH-6:dev-self",
        work_package: "WP-3.2",
        reason: "Selbst-Dev-Abhängigkeit zum Feature-Testen; Bereinigung in WP-3.2",
    },
    Deviation {
        crate_name: "contextra-store",
        rule: "T-ARCH-6:dev-self",
        work_package: "WP-3.3",
        reason: "Selbst-Dev-Abhängigkeit zum Feature-Testen; Bereinigung in WP-3.3",
    },
    Deviation {
        crate_name: "contextra-store",
        rule: "T-ARCH-6:dev-dep:contextra-vector",
        work_package: "WP-3.3",
        reason: "store dev-depends auf vector; Entkopplung in WP-3.3",
    },
    Deviation {
        crate_name: "contextra",
        rule: "T-ARCH-6:dev-dep:contextra-crypto",
        work_package: "WP-1.5",
        reason: "Fassade dev-depends auf crypto; Bereinigung in WP-1.5",
    },

    // T-ARCH-6: Glob-Re-Exporte pub use ...::*
    Deviation {
        crate_name: "contextra-crypto",
        rule: "T-ARCH-6:glob-export",
        work_package: "WP-3.2",
        reason: "Glob Re-Exporte in crypto lib.rs; Bereinigung in WP-3.2",
    },
    Deviation {
        crate_name: "contextra-types",
        rule: "T-ARCH-6:glob-export",
        work_package: "WP-3.1",
        reason: "Glob Re-Exporte in types.rs und lib.rs; Bereinigung in WP-3.1",
    },

    Deviation {
        crate_name: "contextra-simd",
        rule: "T-ARCH-6:glob-export",
        work_package: "WP-8.1",
        reason: "Glob Re-Exporte in simd lib.rs / scalar; Bereinigung in WP-8.1",
    },
    Deviation {
        crate_name: "contextra-testkit",
        rule: "T-ARCH-6:glob-export",
        work_package: "WP-0.1",
        reason: "Glob Re-Export in testkit lib.rs; Bereinigung in WP-0.1",
    },
    Deviation {
        crate_name: "contextra-mvcc",
        rule: "T-ARCH-6:glob-export",
        work_package: "WP-3.3",
        reason: "Glob Re-Export in mvcc lib.rs; Bereinigung in WP-3.3",
    },

    // T-ARCH-7: Namensregeln D-90 und Fassaden-Schnitt D-89
    Deviation {
        crate_name: "contextra-erasure",
        rule: "T-ARCH-7:mod_rs",
        work_package: "WP-4.1",
        reason: "mod.rs in Untermodulen; Bereinigung nach D-90 in WP-4.1",
    },
    Deviation {
        crate_name: "contextra-types",
        rule: "T-ARCH-7:naming",
        work_package: "WP-3.1",
        reason: "Modul types und misc in contextra-types; Bereinigung nach D-90 in WP-3.1",
    },
    Deviation {
        crate_name: "contextra-simd",
        rule: "T-ARCH-7:mod_rs",
        work_package: "WP-8.1",
        reason: "mod.rs in kernels; Bereinigung nach D-90 in WP-8.1",
    },
    Deviation {
        crate_name: "contextra-testkit",
        rule: "T-ARCH-7:mod_rs",
        work_package: "WP-0.1",
        reason: "mod.rs in bench/conformance; Bereinigung nach D-90 in WP-0.1",
    },
    Deviation {
        crate_name: "contextra-crypto",
        rule: "T-ARCH-7:naming",
        work_package: "WP-3.2",
        reason: "Modul crypto.rs trägt Crate-Namen; Bereinigung nach D-90 in WP-3.2",
    },
    Deviation {
        crate_name: "contextra-store",
        rule: "T-ARCH-7:naming",
        work_package: "WP-3.3",
        reason: "Modul util.rs und mod.rs in lsm/wal/manifest/engine; Bereinigung nach D-90 in WP-3.3",
    },
    Deviation {
        crate_name: "contextra-vector",
        rule: "T-ARCH-7:naming",
        work_package: "WP-8.1",
        reason: "vector_index_impl.rs (_impl) und mod.rs in diskann/hnsw; Bereinigung nach D-90 in WP-8.1",
    },
    Deviation {
        crate_name: "contextra-text",
        rule: "T-ARCH-7:mod_rs",
        work_package: "WP-7.1",
        reason: "mod.rs in domain; Bereinigung nach D-90 in WP-7.1",
    },
    Deviation {
        crate_name: "contextra-rank",
        rule: "T-ARCH-7:mod_rs",
        work_package: "WP-8.2",
        reason: "mod.rs in fusion; Bereinigung nach D-90 in WP-8.2",
    },
    Deviation {
        crate_name: "contextra-engine",
        rule: "T-ARCH-7:naming",
        work_package: "WP-1.5",
        reason: "contextra_impl (_impl) und mod.rs in Untermodulen; Bereinigung nach D-90 in WP-1.5",
    },
    Deviation {
        crate_name: "contextra",
        rule: "T-ARCH-7:facade-structure",
        work_package: "WP-1.5",
        reason: "Fassade enthält query_rewriter, collection_profile, etc.; Bereinigung nach D-89 in WP-1.5",
    },

    // T-DOC-1: Sitzungsmarken (ANCHOR[, FILE-CONTEXT, STAND:, SESSION:, /home/) im Altbestand
    Deviation { crate_name: "contextra-types", rule: "T-DOC-1:session-marks", work_package: "WP-0.1", reason: "Altbestand an Sitzungsmarken wird in WP-0.1 bereinigt" },
    Deviation { crate_name: "contextra-ports", rule: "T-DOC-1:session-marks", work_package: "WP-0.1", reason: "Altbestand an Sitzungsmarken wird in WP-0.1 bereinigt" },
    Deviation { crate_name: "contextra-sys", rule: "T-DOC-1:session-marks", work_package: "WP-1.5", reason: "Altbestand an Sitzungsmarken wird in WP-1.5 bereinigt" },
    Deviation { crate_name: "contextra-simd", rule: "T-DOC-1:session-marks", work_package: "WP-8.1", reason: "Altbestand an Sitzungsmarken wird in WP-8.1 bereinigt" },
    Deviation { crate_name: "contextra-durable-fs", rule: "T-DOC-1:session-marks", work_package: "WP-1.5", reason: "Altbestand an Sitzungsmarken wird in WP-1.5 bereinigt" },
    Deviation { crate_name: "contextra-crypto", rule: "T-DOC-1:session-marks", work_package: "WP-3.2", reason: "Altbestand an Sitzungsmarken wird in WP-3.2 bereinigt" },
    Deviation { crate_name: "contextra-mvcc", rule: "T-DOC-1:session-marks", work_package: "WP-3.3", reason: "Altbestand an Sitzungsmarken wird in WP-3.3 bereinigt" },
    Deviation { crate_name: "contextra-store", rule: "T-DOC-1:session-marks", work_package: "WP-3.3", reason: "Altbestand an Sitzungsmarken wird in WP-3.3 bereinigt" },
    Deviation { crate_name: "contextra-vector", rule: "T-DOC-1:session-marks", work_package: "WP-8.1", reason: "Altbestand an Sitzungsmarken wird in WP-8.1 bereinigt" },
    Deviation { crate_name: "contextra-text", rule: "T-DOC-1:session-marks", work_package: "WP-7.1", reason: "Altbestand an Sitzungsmarken wird in WP-7.1 bereinigt" },
    Deviation { crate_name: "contextra-rank", rule: "T-DOC-1:session-marks", work_package: "WP-8.2", reason: "Altbestand an Sitzungsmarken wird in WP-8.2 bereinigt" },
    Deviation { crate_name: "contextra-engine", rule: "T-DOC-1:session-marks", work_package: "WP-1.5", reason: "Altbestand an Sitzungsmarken wird in WP-1.5 bereinigt" },
    Deviation { crate_name: "contextra", rule: "T-DOC-1:session-marks", work_package: "WP-1.5", reason: "Altbestand an Sitzungsmarken wird in WP-1.5 bereinigt" },
    Deviation { crate_name: "contextra-testkit", rule: "T-DOC-1:session-marks", work_package: "WP-0.1", reason: "Altbestand an Sitzungsmarken wird in WP-0.1 bereinigt" },

    // T-LINT-4: Q-09 Marken
    Deviation { crate_name: "contextra-types", rule: "T-LINT-4:trademark-marks", work_package: "WP-0.1", reason: "Altbestand an Q-09 Marken wird in WP-0.1 bereinigt" },
    Deviation { crate_name: "contextra-ports", rule: "T-LINT-4:trademark-marks", work_package: "WP-0.1", reason: "Altbestand an Q-09 Marken wird in WP-0.1 bereinigt" },
    Deviation { crate_name: "contextra-sys", rule: "T-LINT-4:trademark-marks", work_package: "WP-1.5", reason: "Altbestand an Q-09 Marken wird in WP-1.5 bereinigt" },
    Deviation { crate_name: "contextra-simd", rule: "T-LINT-4:trademark-marks", work_package: "WP-8.1", reason: "Altbestand an Q-09 Marken wird in WP-8.1 bereinigt" },
    Deviation { crate_name: "contextra-durable-fs", rule: "T-LINT-4:trademark-marks", work_package: "WP-1.5", reason: "Altbestand an Q-09 Marken wird in WP-1.5 bereinigt" },
    Deviation { crate_name: "contextra-crypto", rule: "T-LINT-4:trademark-marks", work_package: "WP-3.2", reason: "Altbestand an Q-09 Marken wird in WP-3.2 bereinigt" },
    Deviation { crate_name: "contextra-mvcc", rule: "T-LINT-4:trademark-marks", work_package: "WP-3.3", reason: "Altbestand an Q-09 Marken wird in WP-3.3 bereinigt" },
    Deviation { crate_name: "contextra-store", rule: "T-LINT-4:trademark-marks", work_package: "WP-3.3", reason: "Altbestand an Q-09 Marken wird in WP-3.3 bereinigt" },
    Deviation { crate_name: "contextra-vector", rule: "T-LINT-4:trademark-marks", work_package: "WP-8.1", reason: "Altbestand an Q-09 Marken wird in WP-8.1 bereinigt" },
    Deviation { crate_name: "contextra-text", rule: "T-LINT-4:trademark-marks", work_package: "WP-7.1", reason: "Altbestand an Q-09 Marken wird in WP-7.1 bereinigt" },
    Deviation { crate_name: "contextra-rank", rule: "T-LINT-4:trademark-marks", work_package: "WP-8.2", reason: "Altbestand an Q-09 Marken wird in WP-8.2 bereinigt" },
    Deviation { crate_name: "contextra-engine", rule: "T-LINT-4:trademark-marks", work_package: "WP-1.5", reason: "Altbestand an Q-09 Marken wird in WP-1.5 bereinigt" },
    Deviation { crate_name: "contextra", rule: "T-LINT-4:trademark-marks", work_package: "WP-1.5", reason: "Altbestand an Q-09 Marken wird in WP-1.5 bereinigt" },
    Deviation { crate_name: "contextra-testkit", rule: "T-LINT-4:trademark-marks", work_package: "WP-0.1", reason: "Altbestand an Q-09 Marken wird in WP-0.1 bereinigt" },

    // T-TRACE-1: Test-Traceability Altbestand
    Deviation { crate_name: "contextra-types", rule: "T-TRACE-1:test-traceability", work_package: "WP-0.1", reason: "Bestehende Tests werden in WP-0.1 mit SPEC-IDs annotiert" },
    Deviation { crate_name: "contextra-ports", rule: "T-TRACE-1:test-traceability", work_package: "WP-0.1", reason: "Bestehende Tests werden in WP-0.1 mit SPEC-IDs annotiert" },
    Deviation { crate_name: "contextra-sys", rule: "T-TRACE-1:test-traceability", work_package: "WP-1.5", reason: "Bestehende Tests werden in WP-1.5 mit SPEC-IDs annotiert" },
    Deviation { crate_name: "contextra-simd", rule: "T-TRACE-1:test-traceability", work_package: "WP-8.1", reason: "Bestehende Tests werden in WP-8.1 mit SPEC-IDs annotiert" },
    Deviation { crate_name: "contextra-durable-fs", rule: "T-TRACE-1:test-traceability", work_package: "WP-1.5", reason: "Bestehende Tests werden in WP-1.5 mit SPEC-IDs annotiert" },
    Deviation { crate_name: "contextra-crypto", rule: "T-TRACE-1:test-traceability", work_package: "WP-3.2", reason: "Bestehende Tests werden in WP-3.2 mit SPEC-IDs annotiert" },
    Deviation { crate_name: "contextra-mvcc", rule: "T-TRACE-1:test-traceability", work_package: "WP-3.3", reason: "Bestehende Tests werden in WP-3.3 mit SPEC-IDs annotiert" },
    Deviation { crate_name: "contextra-store", rule: "T-TRACE-1:test-traceability", work_package: "WP-3.3", reason: "Bestehende Tests werden in WP-3.3 mit SPEC-IDs annotiert" },
    Deviation { crate_name: "contextra-vector", rule: "T-TRACE-1:test-traceability", work_package: "WP-8.1", reason: "Bestehende Tests werden in WP-8.1 mit SPEC-IDs annotiert" },
    Deviation { crate_name: "contextra-text", rule: "T-TRACE-1:test-traceability", work_package: "WP-7.1", reason: "Bestehende Tests werden in WP-7.1 mit SPEC-IDs annotiert" },
    Deviation { crate_name: "contextra-rank", rule: "T-TRACE-1:test-traceability", work_package: "WP-8.2", reason: "Bestehende Tests werden in WP-8.2 mit SPEC-IDs annotiert" },
    Deviation { crate_name: "contextra-engine", rule: "T-TRACE-1:test-traceability", work_package: "WP-1.5", reason: "Bestehende Tests werden in WP-1.5 mit SPEC-IDs annotiert" },
    Deviation { crate_name: "contextra", rule: "T-TRACE-1:test-traceability", work_package: "WP-1.5", reason: "Bestehende Tests werden in WP-1.5 mit SPEC-IDs annotiert" },
    Deviation { crate_name: "contextra-testkit", rule: "T-TRACE-1:test-traceability", work_package: "WP-0.1", reason: "Bestehende Tests werden in WP-0.1 mit SPEC-IDs annotiert" },

    // T-LINT-3: Wording / Evidenzbegriffe Altbestand
    Deviation { crate_name: "contextra-simd", rule: "T-LINT-3:wording", work_package: "WP-8.1", reason: "Wortlaut im Altbestand wird in WP-8.1 bereinigt" },
    Deviation { crate_name: "contextra-store", rule: "T-LINT-3:wording", work_package: "WP-3.3", reason: "Wortlaut im Altbestand wird in WP-3.3 bereinigt" },
    Deviation { crate_name: "contextra-mvcc", rule: "T-LINT-3:wording", work_package: "WP-3.3", reason: "Wortlaut im Altbestand wird in WP-3.3 bereinigt" },
    Deviation { crate_name: "contextra-vector", rule: "T-LINT-3:wording", work_package: "WP-8.1", reason: "Wortlaut im Altbestand wird in WP-8.1 bereinigt" },
    Deviation { crate_name: "contextra-types", rule: "T-LINT-3:wording", work_package: "WP-3.1", reason: "Wortlaut im Altbestand wird in WP-3.1 bereinigt" },
];

/// Prüft, ob für ein bestimmtes Crate und eine Regel eine dokumentierte Ausnahme vorliegt.
pub fn is_deviation_allowed(crate_name: &str, rule: &str) -> bool {
    KNOWN_DEVIATIONS
        .iter()
        .any(|d| d.crate_name == crate_name && d.rule == rule)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_deviations_have_valid_work_package() {
        for dev in KNOWN_DEVIATIONS {
            assert!(
                dev.work_package.starts_with("WP-"),
                "Abweichung für '{}' (Regel '{}') muss einem Arbeitspaket 'WP-...' zugeordnet sein, gefunden: '{}'",
                dev.crate_name,
                dev.rule,
                dev.work_package
            );
            assert!(
                !dev.reason.trim().is_empty(),
                "Abweichung für '{}' (Regel '{}') erfordert eine Begründung",
                dev.crate_name,
                dev.rule
            );
        }
    }
}
