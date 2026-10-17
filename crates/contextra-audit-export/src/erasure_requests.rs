//! GDPR Art. 12 & 17 Erasure Requests register and deadline compliance tracking.
//!
//! # Compliance & Legal Notice
//! EDPB Coordinated Enforcement Action (10.02.2026) compliance:
//! Automated erasure tracking, strict 1-month base deadline calculation with
//! documented max 2-month extension (+2 months, max 3 months total from receipt),
//! mandatory justification and Art. 17(3) exception categories for refusals,
//! and deadline compliance KPI calculation.
//!
//! **LEGAL DISCLAIMER:**
//! Vorlage, keine Rechtsberatung.

use crate::error::AuditExportError;
use contextra_ports::Clock;
use serde::{Deserialize, Serialize};

/// Returns true if `year` is a leap year in the Gregorian calendar.
pub fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Returns the number of days in the given `month` (1..=12) for `year`.
pub fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

/// A pure calendar date (Year-Month-Day) for GDPR deadline calculations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CivilDate {
    /// Gregorian calendar year (e.g. 2026).
    pub year: i32,
    /// Month of year (1..=12).
    pub month: u8,
    /// Day of month (1..=31).
    pub day: u8,
}

impl CivilDate {
    /// Creates a new [`CivilDate`], validating year, month, and day.
    pub fn new(year: i32, month: u8, day: u8) -> Result<Self, AuditExportError> {
        if !(1..=12).contains(&month) {
            return Err(AuditExportError::InvalidDate(format!(
                "Invalid month: {month}"
            )));
        }
        let max_days = days_in_month(year, month);
        if day < 1 || day > max_days {
            return Err(AuditExportError::InvalidDate(format!(
                "Invalid day {day} for year {year}, month {month} (max {max_days})"
            )));
        }
        Ok(Self { year, month, day })
    }

    /// Adds `months` calendar months to this date according to GDPR Art. 12(3) rules.
    ///
    /// If the target day exceeds the maximum days in the target month (e.g. Jan 31 + 1 month),
    /// the date is clamped to the last day of the target month (e.g. Feb 28 or Feb 29).
    pub fn add_months(&self, months: u8) -> Self {
        let total_months = (self.month as u32) - 1 + (months as u32);
        let new_year = self.year + (total_months / 12) as i32;
        let new_month = ((total_months % 12) + 1) as u8;
        let max_days = days_in_month(new_year, new_month);
        let new_day = self.day.min(max_days);
        Self {
            year: new_year,
            month: new_month,
            day: new_day,
        }
    }

    /// Converts this civil date to Unix timestamp seconds at 23:59:59 UTC (end of day).
    pub fn to_unix_seconds_end_of_day(&self) -> u64 {
        let days = civil_to_days(self.year, self.month, self.day);
        (days as u64) * 86_400 + 86_399
    }

    /// Converts Unix nanoseconds to a UTC [`CivilDate`].
    pub fn from_unix_nanos(nanos: u64) -> Self {
        let secs = nanos / 1_000_000_000;
        let days = (secs / 86_400) as i64;
        let (y, m, d) = days_to_civil(days);
        Self {
            year: y,
            month: m,
            day: d,
        }
    }
}

/// Converts civil date (year, month 1..12, day 1..31) to days since Unix epoch (1970-01-01).
fn civil_to_days(year: i32, month: u8, day: u8) -> i64 {
    let y = if month <= 2 { year - 1 } else { year } as i64;
    let m = if month <= 2 {
        (month as i64) + 12
    } else {
        month as i64
    };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as u64; // [0, 399]
    let doy = (153 * (m - 3) + 2) / 5 + (day as i64) - 1; // [0, 365]
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + (doy as u64); // [0, 146096]
    era * 146_097 + (doe as i64) - 719_468
}

/// Converts days since Unix epoch (1970-01-01) to civil date (year, month 1..12, day 1..31).
fn days_to_civil(days: i64) -> (i32, u8, u8) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // [0, 399]
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u8; // [1, 31]
    let m = (if mp < 10 { mp + 3 } else { mp - 9 }) as u8; // [1, 12]
    let y = if m <= 2 { y + 1 } else { y };
    (y as i32, m, d)
}

/// Statutory exception grounds under GDPR Art. 17(3) justifying refusal of an erasure request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Art17_3Ground {
    /// Art. 17(3)(a) - Freedom of expression and information.
    FreedomOfExpression,
    /// Art. 17(3)(b) - Compliance with a legal obligation under Union or Member State law.
    LegalObligation,
    /// Art. 17(3)(c) - Public interest in the area of public health.
    PublicInterestHealth,
    /// Art. 17(3)(d) - Archiving purposes in the public interest, scientific/historical research, statistics.
    ArchivingPublicInterest,
    /// Art. 17(3)(e) - Establishment, exercise, or defense of legal claims.
    LegalClaims,
}

/// Statutory restriction grounds under GDPR Art. 18(1) restricting processing instead of immediate deletion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Art18Ground {
    /// Art. 18(1)(a) - Accuracy of personal data is contested by data subject.
    AccuracyContested,
    /// Art. 18(1)(b) - Processing is unlawful and data subject opposes erasure.
    UnlawfulProcessingDataRetained,
    /// Art. 18(1)(c) - Controller no longer needs data, but required by data subject for legal claims.
    LegalClaims,
    /// Art. 18(1)(d) - Verification of overriding legitimate grounds pending.
    VerificationOfOverridingGrounds,
}

/// A non-empty, trimmed justification string enforcing documented decisions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NonEmptyJustification(String);

impl NonEmptyJustification {
    /// Creates a new [`NonEmptyJustification`], returning error if the input trimmed is empty.
    pub fn try_new(s: impl Into<String>) -> Result<Self, AuditExportError> {
        let val = s.into();
        if val.trim().is_empty() {
            return Err(AuditExportError::EmptyJustification);
        }
        Ok(Self(val))
    }

    /// Returns the inner justification string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Final decision or status for an erasure request under GDPR Art. 12, 17, and 18.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErasureDecision {
    /// Request fully granted and executed.
    Granted,
    /// Request restricted under Art. 18 DSGVO instead of deleted.
    Restricted(Art18Ground),
    /// Request refused under Art. 17(3) DSGVO. Requires non-empty justification and statutory ground.
    Refused {
        /// Statutory exception category under Art. 17(3) DSGVO.
        exception: Art17_3Ground,
        /// Mandatory non-empty justification explaining the refusal decision.
        justification: NonEmptyJustification,
    },
}

/// An erasure request entry in the GDPR Art. 12 / 17 audit register.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErasureRequest {
    /// Unique identifier for the request.
    pub id: String,
    /// Receipt date of the request.
    pub received_at: CivilDate,
    /// Keyed-hash reference of the data subject or resource scope (privacy-preserving).
    pub scope_keyed_hash: String,
    /// Whether formal notice of extension was communicated to data subject within 1 month (Art. 12 Abs. 3).
    pub extension_notice_sent: bool,
    /// Mandatory justification if an extension notice was sent.
    pub extension_justification: Option<NonEmptyJustification>,
    /// Number of extension months granted (0, 1, or 2). Max total deadline is 1 + extended_months (max 3 months).
    pub extended_months: u8,
    /// Final decision made on the request, if decided.
    pub decision: Option<ErasureDecision>,
    /// Completion date when the request was fulfilled or formally refused.
    pub completed_at: Option<CivilDate>,
}

impl ErasureRequest {
    /// Creates a new [`ErasureRequest`] with input validation.
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        id: impl Into<String>,
        received_at: CivilDate,
        scope_keyed_hash: impl Into<String>,
        extension_notice_sent: bool,
        extension_justification: Option<NonEmptyJustification>,
        extended_months: u8,
        decision: Option<ErasureDecision>,
        completed_at: Option<CivilDate>,
    ) -> Result<Self, AuditExportError> {
        if extended_months > 2 {
            return Err(AuditExportError::InvalidExtension(format!(
                "Extension months {extended_months} exceeds maximum 2 additional months"
            )));
        }
        if extended_months > 0 && !extension_notice_sent {
            return Err(AuditExportError::InvalidExtension(
                "Cannot claim extended deadline without sending extension notice within 1 month".to_string(),
            ));
        }
        if extended_months > 0 && extension_justification.is_none() {
            return Err(AuditExportError::EmptyJustification);
        }

        Ok(Self {
            id: id.into(),
            received_at,
            scope_keyed_hash: scope_keyed_hash.into(),
            extension_notice_sent,
            extension_justification,
            extended_months,
            decision,
            completed_at,
        })
    }

    /// Base deadline: 1 calendar month from `received_at` (Art. 12 Abs. 3).
    pub fn base_deadline(&self) -> CivilDate {
        self.received_at.add_months(1)
    }

    /// Effective statutory deadline taking documented extension into account.
    ///
    /// If notice was sent, deadline is extended by up to 2 months (max 3 months total from receipt).
    /// If notice was not sent, effective deadline remains `base_deadline()` regardless of `extended_months`.
    pub fn effective_deadline(&self) -> CivilDate {
        if self.extension_notice_sent {
            self.received_at.add_months(1 + self.extended_months)
        } else {
            self.base_deadline()
        }
    }

    /// Returns `Some(true)` if request was completed on or before `effective_deadline()`,
    /// `Some(false)` if completed after, or `None` if not yet completed.
    pub fn is_completed_within_deadline(&self) -> Option<bool> {
        self.completed_at
            .map(|completed| completed <= self.effective_deadline())
    }

    /// Returns whether the request is overdue given `current_date`.
    pub fn is_overdue(&self, current_date: CivilDate) -> bool {
        let deadline = self.effective_deadline();
        match self.completed_at {
            Some(completed) => completed > deadline,
            None => current_date > deadline,
        }
    }
}

/// KPI report summarizing deadline compliance for a set of erasure requests (EDPB Coordinated Action).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeadlineKpiReport {
    /// Total number of erasure requests evaluated.
    pub total_requests: usize,
    /// Number of requests completed.
    pub completed_count: usize,
    /// Number of requests completed within statutory deadline.
    pub completed_on_time_count: usize,
    /// Number of requests completed after statutory deadline expired.
    pub completed_overdue_count: usize,
    /// Number of pending (uncompleted) requests.
    pub pending_count: usize,
    /// Number of pending requests still within statutory deadline.
    pub pending_on_time_count: usize,
    /// Number of pending requests past statutory deadline.
    pub pending_overdue_count: usize,
    /// Compliance percentage of completed requests completed on time (0.0..=100.0).
    pub completed_compliance_rate_percent: f64,
    /// Compliance percentage of all requests currently on time (0.0..=100.0).
    pub overall_compliance_rate_percent: f64,
}

/// Calculates deadline compliance KPI metrics for a list of erasure requests using an injected `Clock`.
pub fn calculate_deadline_kpi(requests: &[ErasureRequest], clock: &dyn Clock) -> DeadlineKpiReport {
    let now_nanos = clock.now_unix_nanos();
    let current_date = CivilDate::from_unix_nanos(now_nanos);

    let total_requests = requests.len();
    let mut completed_count = 0;
    let mut completed_on_time_count = 0;
    let mut completed_overdue_count = 0;
    let mut pending_count = 0;
    let mut pending_on_time_count = 0;
    let mut pending_overdue_count = 0;

    for req in requests {
        let deadline = req.effective_deadline();
        match req.completed_at {
            Some(completed) => {
                completed_count += 1;
                if completed <= deadline {
                    completed_on_time_count += 1;
                } else {
                    completed_overdue_count += 1;
                }
            }
            None => {
                pending_count += 1;
                if current_date <= deadline {
                    pending_on_time_count += 1;
                } else {
                    pending_overdue_count += 1;
                }
            }
        }
    }

    let completed_compliance_rate_percent = if completed_count > 0 {
        (completed_on_time_count as f64 / completed_count as f64) * 100.0
    } else {
        100.0
    };

    let overall_on_time = completed_on_time_count + pending_on_time_count;
    let overall_compliance_rate_percent = if total_requests > 0 {
        (overall_on_time as f64 / total_requests as f64) * 100.0
    } else {
        100.0
    };

    DeadlineKpiReport {
        total_requests,
        completed_count,
        completed_on_time_count,
        completed_overdue_count,
        pending_count,
        pending_on_time_count,
        pending_overdue_count,
        completed_compliance_rate_percent,
        overall_compliance_rate_percent,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_civil_date_month_addition_and_clamping() {
        // Jan 31 + 1 month -> Feb 28 (non-leap year 2026)
        let jan31_2026 = CivilDate::new(2026, 1, 31).unwrap();
        let feb_deadline = jan31_2026.add_months(1);
        assert_eq!(feb_deadline, CivilDate::new(2026, 2, 28).unwrap());

        // Jan 31 + 1 month -> Feb 29 (leap year 2028)
        let jan31_2028 = CivilDate::new(2028, 1, 31).unwrap();
        let feb_leap_deadline = jan31_2028.add_months(1);
        assert_eq!(feb_leap_deadline, CivilDate::new(2028, 2, 29).unwrap());

        // Jan 31 + 3 months -> Apr 30
        let apr_deadline = jan31_2026.add_months(3);
        assert_eq!(apr_deadline, CivilDate::new(2026, 4, 30).unwrap());
    }

    #[test]
    fn test_civil_date_unix_conversion_roundtrip() {
        let date = CivilDate::new(2026, 2, 15).unwrap();
        let end_secs = date.to_unix_seconds_end_of_day();
        let nanos = end_secs * 1_000_000_000;
        let converted = CivilDate::from_unix_nanos(nanos);
        assert_eq!(converted, date);
    }
}
