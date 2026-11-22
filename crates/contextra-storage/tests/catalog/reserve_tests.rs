#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

use contextra_storage::catalog::edit::CatalogEdit;
use contextra_storage::catalog::reserve::{IdError, IdReserve, ID_RESERVE_BLOCK};
use contextra_storage::catalog::state::{apply_edit, CatalogState};

#[test]
fn test_id_reserve_blocks_and_exhaustion() {
    let mut state = CatalogState::default();
    let mut reserve = IdReserve::new(&state);

    // Vor der ersten Reservierung muss alloc NeedsReservation melden
    assert_eq!(reserve.alloc_run_id(), Err(IdError::NeedsReservation));

    // Hochwassermarken für Blöcke zu 256 planen
    let (r_hwm, b_hwm, s_hwm, t_hwm, k_gen) = reserve.plan_reservation().unwrap();
    assert_eq!(r_hwm, ID_RESERVE_BLOCK);
    assert_eq!(b_hwm, ID_RESERVE_BLOCK);

    // Edit anwenden
    let mut edit = CatalogEdit::default();
    edit.reserve = Some((r_hwm, b_hwm, s_hwm, t_hwm, k_gen));
    state = apply_edit(&state, &edit).unwrap();
    reserve.apply_reservation(&state);

    // Nun können IDs von 0 bis 255 vergeben werden
    for expected in 0..ID_RESERVE_BLOCK {
        let id = reserve.alloc_run_id().unwrap();
        assert_eq!(id, expected);
    }

    // Bei Erreichen von 256 muss wieder NeedsReservation gemeldet werden
    assert_eq!(reserve.alloc_run_id(), Err(IdError::NeedsReservation));
}

#[test]
fn test_id_reservation_survives_crash_and_no_reuse() {
    // INV-KEY-2: IDs werden erst nach dauerhaftem Hochwassermarken-Edit benutzt und nie wiederverwendet.
    let mut state = CatalogState::default();
    let mut reserve_before_crash = IdReserve::new(&state);

    // 1. Reservierungs-Edit commiten (HWM = 256)
    let (r_hwm, b_hwm, s_hwm, t_hwm, k_gen) = reserve_before_crash.plan_reservation().unwrap();
    let mut edit = CatalogEdit::default();
    edit.reserve = Some((r_hwm, b_hwm, s_hwm, t_hwm, k_gen));
    state = apply_edit(&state, &edit).unwrap();
    reserve_before_crash.apply_reservation(&state);

    // 2. Ein paar IDs vergeben
    let id0 = reserve_before_crash.alloc_run_id().unwrap();
    let id1 = reserve_before_crash.alloc_run_id().unwrap();
    let id2 = reserve_before_crash.alloc_run_id().unwrap();
    assert_eq!(id0, 0);
    assert_eq!(id1, 1);
    assert_eq!(id2, 2);

    // 3. Absturz simulieren!
    // Nach Recovery wird IdReserve neu aus dem persistenten CatalogState initialisiert.
    let mut reserve_after_crash = IdReserve::new(&state);

    // INV-KEY-2: Vergabe beginnt nach Recovery bei der Hochwassermarke (256)
    // Bevor neue IDs vergeben werden können, muss die nächste Hochwassermarke reserviert werden
    assert_eq!(reserve_after_crash.alloc_run_id(), Err(IdError::NeedsReservation));

    let (r_hwm, b_hwm, s_hwm, t_hwm, k_gen) = reserve_after_crash.plan_reservation().unwrap();
    assert_eq!(r_hwm, 512);

    let mut edit2 = CatalogEdit::default();
    edit2.reserve = Some((r_hwm, b_hwm, s_hwm, t_hwm, k_gen));
    state = apply_edit(&state, &edit2).unwrap();
    reserve_after_crash.apply_reservation(&state);

    // Erste vergebene ID nach dem Absturz
    let next_id = reserve_after_crash.alloc_run_id().unwrap();
    assert_eq!(next_id, 256);

    // Keine der zuvor ungenutzten IDs (3..255) oder genutzten IDs (0..2) wird je wiederverwendet!
    assert!(next_id >= 256);
}
