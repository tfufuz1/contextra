//! Auswertung und Erfassung von Restrisiken gemäß Katalog (SPEC §11.7, Anhang D, ER-28).
//!
//! Die vollständige Liste der Restrisiken wird immer bewertet.
//! Alle 15+ Bedingungen aus Anhang D sind maschinenprüfbar abgebildet.

use serde::{Deserialize, Serialize};

/// Identifikatoren für Restrisiken nach SPEC Anhang D.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ResidualRiskId {
    /// Flash-Remanenz auf FTL-Ebene.
    FlashRemanence,
    /// CoW-Dateisysteme oder Dateisystemjournale.
    FsJournalOrCow,
    /// Reste verpackter Schlüsselkopien.
    WrappedKeyRemanence,
    /// Reste in freigegebenen Speicherblöcken vor Wiederbelegung.
    FreedBlockRemanence,
    /// Reste überschriebener Keyslots.
    OldKeyslotRemanence,
    /// Flüchtigen Prozessspeicher.
    ProcessMemory,
    /// Rückschlüsse aus verbleibender Indextopologie.
    IndexTopologyInference,
    /// Textindex bis zum nächsten Segment-Rewrite.
    TextIndexUntilPurge,
    /// Replika-Synchronisationsfenster.
    ReplicaWindow,
    /// Externe Datensicherungen / Replikate.
    ExternalCopies,
    /// Externe Auftragsverarbeiter.
    ExternalProcessors,
    /// Kryptografisches Pseudonym-Commitment.
    PseudonymCommitment,
    /// Purge-Dauer skaliert mit Datenbankgröße.
    PurgeScalesWithDb,
    /// Verzögerter Purge-Lauf.
    PurgeDeferred,
    /// Schlüsselkompromittierung vor Löschung.
    KeyCompromise,
    /// Verifikation stützt sich auf Cache statt Medium (D-113).
    VerifiedViaCache,
    /// Backup enthält Schlüsselmaterial (D-112).
    BackupKeyMaterial,
    /// R2/R3-Überlappung bei Dokument-Ersetzung (D-115).
    ReplacementOverlap,
    /// Berichtigung mit Purge belässt Schlüssel (D-117).
    SupersededWithSameKey,
}

impl ResidualRiskId {
    /// Gibt den kanonischen Bezeichner nach Anhang D und `ctx-verify` zurück.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::FlashRemanence => "FlashRemanence",
            Self::FsJournalOrCow => "FsJournalOrCow",
            Self::WrappedKeyRemanence => "WrappedKeyRemanence",
            Self::FreedBlockRemanence => "FreedBlockRemanence",
            Self::OldKeyslotRemanence => "OldKeyslotRemanence",
            Self::ProcessMemory => "ProcessMemory",
            Self::IndexTopologyInference => "IndexTopologyInference",
            Self::TextIndexUntilPurge => "TextIndexUntilPurge",
            Self::ReplicaWindow => "ReplicaWindow",
            Self::ExternalCopies => "ExternalCopies",
            Self::ExternalProcessors => "ExternalProcessors",
            Self::PseudonymCommitment => "PseudonymCommitment",
            Self::PurgeScalesWithDb => "PurgeScalesWithDb",
            Self::PurgeDeferred => "PurgeDeferred",
            Self::KeyCompromise => "KeyCompromise",
            Self::VerifiedViaCache => "VerifiedViaCache",
            Self::BackupKeyMaterial => "BackupKeyMaterial",
            Self::ReplacementOverlap => "ReplacementOverlap",
            Self::SupersededWithSameKey => "SupersededWithSameKey",
        }
    }

    /// Gibt den Token nach SPEC Anhang D zurück.
    pub const fn id_token(&self) -> &'static str {
        match self {
            Self::FlashRemanence => "RISK-FLASH-REMANENCE",
            Self::FsJournalOrCow => "RISK-FS-JOURNAL-OR-COW",
            Self::WrappedKeyRemanence => "RISK-WRAPPED-KEY-REMANENCE",
            Self::FreedBlockRemanence => "RISK-FREED-BLOCK-REMANENCE",
            Self::OldKeyslotRemanence => "RISK-OLD-KEYSLOT-REMANENCE",
            Self::ProcessMemory => "RISK-PROCESS-MEMORY",
            Self::IndexTopologyInference => "RISK-INDEX-TOPOLOGY-INFERENCE",
            Self::TextIndexUntilPurge => "RISK-TEXT-INDEX-UNTIL-PURGE",
            Self::ReplicaWindow => "RISK-REPLICA-WINDOW",
            Self::ExternalCopies => "RISK-EXTERNAL-COPIES",
            Self::ExternalProcessors => "RISK-EXTERNAL-PROCESSORS",
            Self::PseudonymCommitment => "RISK-PSEUDONYM-COMMITMENT",
            Self::PurgeScalesWithDb => "RISK-PURGE-SCALES-WITH-DB",
            Self::PurgeDeferred => "RISK-PURGE-DEFERRED",
            Self::KeyCompromise => "RISK-KEY-COMPROMISE",
            Self::VerifiedViaCache => "RISK-VERIFIED-VIA-CACHE",
            Self::BackupKeyMaterial => "RISK-BACKUP-KEY-MATERIAL",
            Self::ReplacementOverlap => "RISK-REPLACEMENT-OVERLAP",
            Self::SupersededWithSameKey => "RISK-SUPERSEDED-WITH-SAME-KEY",
        }
    }

    /// Gibt die Kurzbezeichnung / Text nach SPEC Anhang D zurück.
    pub const fn description(&self) -> &'static str {
        match self {
            Self::FlashRemanence => "Flash-Medien koennen Altdaten in nicht adressierbaren Zellen halten",
            Self::FsJournalOrCow => "Dateisystem-Journal oder Copy-on-Write kann Altbloecke halten",
            Self::WrappedKeyRemanence => "Genullter Tresor-Slot oder Altblock kann auf dem Medium verbleiben",
            Self::FreedBlockRemanence => "Freigegebene Dateibloecke bleiben bis zum Ueberschreiben auf dem Medium",
            Self::OldKeyslotRemanence => "Ein alter Keyslot kann nach Passphrasenwechsel auf dem Medium verbleiben",
            Self::ProcessMemory => "Klartext kann im Prozessspeicher, Swap oder Coredump liegen",
            Self::IndexTopologyInference => "Reparierte HNSW-Kanten sind eine Funktion der Zielnachbarschaft",
            Self::TextIndexUntilPurge => "Entkoppelte Postings enthalten den Wortschatz des Ziels ohne Zuordnung",
            Self::ReplicaWindow => "Zeit zwischen Anfrage und Abschluss; Kopien in diesem Fenster",
            Self::ExternalCopies => "Kopien ausserhalb von Contextra (Backups, Exporte, Logs)",
            Self::ExternalProcessors => "Empfaenger und Auftragsverarbeiter ausserhalb von Contextra",
            Self::PseudonymCommitment => "Commitments, Bereichs-Tags im Bericht, Ledger sind pseudonym",
            Self::PurgeScalesWithDb => "Physischer Aufwand waechst mit der Datenbank",
            Self::PurgeDeferred => "Physischer Purge erfolgt verzoegert bei Kompaktion",
            Self::KeyCompromise => "Wer Passphrase oder K_master vor der Loeschung besass, konnte Daten entschluesseln und kopieren",
            Self::VerifiedViaCache => "Die Messung stuetzt sich auf Page-Cache statt Medium",
            Self::BackupKeyMaterial => "Ein Backup des Datenverzeichnisses enthaelt Schluesselmaterial",
            Self::ReplacementOverlap => "Zwischen Ersetzung R2 und R3 sind alter und neuer Inhalt lesbar",
            Self::SupersededWithSameKey => "Berichtigung mit Purge entfernt Daten, der Schluessel bleibt; Klasse K nicht erreicht",
        }
    }

    /// Gibt den Betreiberhinweis nach SPEC Anhang D zurück.
    pub const fn operator_guidance(&self) -> &'static str {
        match self {
            Self::FlashRemanence => "Volume-Verschluesselung, Medienaussonderung",
            Self::FsJournalOrCow => {
                "Dateischluessel-Vernichtung mindert (D-85); Volume-Verschluesselung"
            }
            Self::WrappedKeyRemanence => "Hardware-Anker (D-53) oder Volume-Verschluesselung",
            Self::FreedBlockRemanence => {
                "keine weitere Massnahme erforderlich, ausser hohem Schutzbedarf"
            }
            Self::OldKeyslotRemanence => {
                "Alte Passphrase gilt als kompromittiert; Master-Rotation v1.1"
            }
            Self::ProcessMemory => "Swap verschluesseln, Coredumps abschalten",
            Self::IndexTopologyInference => "vector_strict_rebuild aktivieren",
            Self::TextIndexUntilPurge => "Abschluss abwarten",
            Self::ReplicaWindow => "erase_sla klein halten",
            Self::ExternalCopies => "Checkliste 16",
            Self::ExternalProcessors => "Information nach Art. 17 Abs. 2, 19",
            Self::PseudonymCommitment => "Aufbewahrungsdauer begruenden (DSFA)",
            Self::PurgeScalesWithDb => "-",
            Self::PurgeDeferred => "Abschluss abwarten",
            Self::KeyCompromise => "Schluesselverwaltung des Betreibers",
            Self::VerifiedViaCache => "Betrieb mit O_DIRECT-faehigem Dateisystem (D-113)",
            Self::BackupKeyMaterial => "Tresor getrennt sichern, RB-9 (D-112)",
            Self::ReplacementOverlap => "Abschluss abwarten (D-115)",
            Self::SupersededWithSameKey => {
                "Mehr-Personen-Dokument: Entscheidungsregel D-115 (D-117)"
            }
        }
    }
}

/// Kontext zur Auswertung der dynamischen Bedingungen für Restrisiken (SPEC Anhang D).
#[derive(Debug, Clone)]
pub struct ResidualRiskContext {
    /// Physischer Purge vollständig abgeschlossen.
    pub purge_complete: bool,
    /// Medium ist rotierend (z. B. HDD). Wenn false oder unbekannt: FlashRemanence.
    pub is_rotational: bool,
    /// Tier 2 CoW-Dateisystem (btrfs, zfs, apfs) oder ext4 mit data=journal.
    pub is_cow_or_journaling: bool,
    /// FileVault ist der Tresor-Anker (standardmäßig immer true).
    pub is_file_vault: bool,
    /// Passphrase wurde mindestens einmal gewechselt.
    pub passphrase_changed: bool,
    /// Prozesshärtung ist Off oder nur Teilerfolg.
    pub process_hardening_off_or_partial: bool,
    /// Vektorlöschung aktiv vor nächstem Rebuild.
    pub vector_topology_dirty: bool,
    /// Fallback-Stufe 4 aktiv (11.9).
    pub fallback_stage_4: bool,
    /// Fallback-Stufe 3 aktiv (11.9).
    pub fallback_stage_3: bool,
    /// Verifikation stützt sich auf Cache (`Cached` oder `CacheDropped`).
    pub verified_via_cache: bool,
    /// Backup-Scope des Tresors ist `SameAsData` oder `Unknown`.
    pub vault_backup_scope_same_or_unknown: bool,
    /// Operation ist `replace_erasing` vor dem Gate.
    pub is_replace_erasing_pre_gate: bool,
    /// Operation ist `update_with(..., Purge)`.
    pub is_superseded_with_same_key: bool,
}

impl Default for ResidualRiskContext {
    fn default() -> Self {
        Self {
            purge_complete: true,
            is_rotational: false,
            is_cow_or_journaling: false,
            is_file_vault: true,
            passphrase_changed: false,
            process_hardening_off_or_partial: false,
            vector_topology_dirty: false,
            fallback_stage_4: false,
            fallback_stage_3: false,
            verified_via_cache: false,
            vault_backup_scope_same_or_unknown: true,
            is_replace_erasing_pre_gate: false,
            is_superseded_with_same_key: false,
        }
    }
}

/// Bewertet alle Restrisiken mit vollständigem Kontext gemäß Anhang D.
pub fn evaluate_residual_risks_with_context(ctx: &ResidualRiskContext) -> Vec<ResidualRiskId> {
    let mut risks = Vec::new();

    // 1. FlashRemanence: immer (Medium nicht rotierend oder unbekannt)
    if !ctx.is_rotational {
        risks.push(ResidualRiskId::FlashRemanence);
    }

    // 2. FsJournalOrCow: Tier 2 (btrfs/ZFS/APFS) immer; ext4/XFS bei Journal-Modus data=journal
    if ctx.is_cow_or_journaling {
        risks.push(ResidualRiskId::FsJournalOrCow);
    }

    // 3. WrappedKeyRemanence: immer, solange der Anker FileVault ist
    if ctx.is_file_vault {
        risks.push(ResidualRiskId::WrappedKeyRemanence);
    }

    // 4. FreedBlockRemanence: immer
    risks.push(ResidualRiskId::FreedBlockRemanence);

    // 5. OldKeyslotRemanence: nach change_passphrase
    if ctx.passphrase_changed {
        risks.push(ResidualRiskId::OldKeyslotRemanence);
    }

    // 6. ProcessMemory: process_hardening in {Off} oder Teilerfolg; sonst als Hinweis
    if ctx.process_hardening_off_or_partial {
        risks.push(ResidualRiskId::ProcessMemory);
    }

    // 7. IndexTopologyInference: nach Vektorlöschung bis zum nächsten Rebuild
    if ctx.vector_topology_dirty {
        risks.push(ResidualRiskId::IndexTopologyInference);
    }

    // 8. TextIndexUntilPurge: zwischen KeyDestroyed und Segment-Rewrite
    if !ctx.purge_complete {
        risks.push(ResidualRiskId::TextIndexUntilPurge);
    }

    // 9. ReplicaWindow: immer
    risks.push(ResidualRiskId::ReplicaWindow);

    // 10. ExternalCopies: immer
    risks.push(ResidualRiskId::ExternalCopies);

    // 11. ExternalProcessors: immer
    risks.push(ResidualRiskId::ExternalProcessors);

    // 12. PseudonymCommitment: immer
    risks.push(ResidualRiskId::PseudonymCommitment);

    // 13. PurgeScalesWithDb: nur Fallbackstufe 4 (11.9)
    if ctx.fallback_stage_4 {
        risks.push(ResidualRiskId::PurgeScalesWithDb);
    }

    // 14. PurgeDeferred: nur Fallbackstufe 3 (11.9)
    if ctx.fallback_stage_3 {
        risks.push(ResidualRiskId::PurgeDeferred);
    }

    // 15. KeyCompromise: immer
    risks.push(ResidualRiskId::KeyCompromise);

    // 16. VerifiedViaCache: verified_via in {Cached, CacheDropped}
    if ctx.verified_via_cache {
        risks.push(ResidualRiskId::VerifiedViaCache);
    }

    // 17. BackupKeyMaterial: vault_backup_scope in {SameAsData, Unknown}
    if ctx.vault_backup_scope_same_or_unknown {
        risks.push(ResidualRiskId::BackupKeyMaterial);
    }

    // 18. ReplacementOverlap: bei replace_erasing, bis zum Gate
    if ctx.is_replace_erasing_pre_gate {
        risks.push(ResidualRiskId::ReplacementOverlap);
    }

    // 19. SupersededWithSameKey: bei update_with(..., Purge)
    if ctx.is_superseded_with_same_key {
        risks.push(ResidualRiskId::SupersededWithSameKey);
    }

    risks
}

/// Abwärtskompatible Auswertung aller Restrisiken (ER-28).
///
/// Beinhaltet standardmäßig alle bedingungslosen und die Purge-abhängigen Risiken.
pub fn evaluate_residual_risks(purge_complete: bool) -> Vec<ResidualRiskId> {
    let ctx = ResidualRiskContext {
        purge_complete,
        ..Default::default()
    };
    evaluate_residual_risks_with_context(&ctx)
}

/// Serialisiert ein Restrisiko als maschinenlesbaren Eintrag.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResidualRiskEntry {
    /// Token nach SPEC Anhang D (z. B. "RISK-FLASH-REMANENCE").
    pub id: String,
    /// Name / Bezeichner (z. B. "FlashRemanence").
    pub name: String,
    /// Kurztext / Beschreibung.
    pub description: String,
    /// Betreiberhinweis.
    pub operator_guidance: String,
}

impl ResidualRiskEntry {
    /// Erstellt einen Eintrag aus einer Risiko-ID.
    pub fn from_id(risk: ResidualRiskId) -> Self {
        Self {
            id: risk.id_token().to_string(),
            name: risk.as_str().to_string(),
            description: risk.description().to_string(),
            operator_guidance: risk.operator_guidance().to_string(),
        }
    }

    /// Abwärtskompatibler Alias.
    pub fn from_risk(risk: &ResidualRiskId) -> Self {
        Self::from_id(*risk)
    }
}
