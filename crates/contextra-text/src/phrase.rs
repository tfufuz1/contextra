//! Phrasensuche über gespeicherte Positionen (SPEC D-81, T-TEXT-PHRASE-1).

pub fn match_phrase_positions(term_positions: &[&[u32]]) -> bool {
    if term_positions.is_empty() {
        return false;
    }
    if term_positions.len() == 1 {
        return term_positions.first().map(|p| !p.is_empty()).unwrap_or(false);
    }

    let first = match term_positions.first() {
        Some(f) => *f,
        None => return false,
    };
    for &start_pos in first {
        let mut matched = true;
        for (i, &next_list) in term_positions.iter().enumerate().skip(1) {
            let offset_i = u32::try_from(i).unwrap_or(u32::MAX);
            let expected_pos = start_pos.saturating_add(offset_i);
            if !next_list.contains(&expected_pos) {
                matched = false;
                break;
            }
        }
        if matched {
            return true;
        }
    }

    false
}
