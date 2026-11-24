//! Torn-tail-Ergebniszustände (INV-LOG-8).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TailState { Clean, Torn, CorruptMidLog }

/// Ein ungültiger Record ist Torn-Tail, sofern dahinter keine syntaktisch
/// vollständige Gruppe mehr gefunden wird.
pub fn tail_state(has_later_group: bool) -> TailState {
    if has_later_group { TailState::CorruptMidLog } else { TailState::Torn }
}
