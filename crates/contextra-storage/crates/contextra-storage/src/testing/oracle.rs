//! Differential erasure and visibility oracle (SPEC §13.2, SP-VERIFY).

use super::model::ReferenceModel;
use contextra_storage_ports::{BranchId, ClusterId, Error, Stamp};

pub fn assert_erased_result<T: core::fmt::Debug>(res: Result<T, Error>) -> Result<(), Error> {
    match res {
        Err(Error::Erased { .. } | Error::ClusterRetired | Error::Shredded) => Ok(()),
        other => Err(Error::Corrupt {
            place: if other.is_ok() {
                "erasure oracle: erased cluster returned data"
            } else {
                "erasure oracle: unexpected error on erased cluster"
            },
        }),
    }
}

pub fn verify_point_equivalence(
    model: &ReferenceModel,
    cluster: ClusterId,
    branch: BranchId,
    key: &[u8],
    view_stamp: Stamp,
    actual: Result<Option<Vec<u8>>, Error>,
) -> Result<(), Error> {
    let expected = model.get(cluster, branch, key, view_stamp);
    match (expected, actual) {
        (Ok(exp), Ok(act)) if exp == act => Ok(()),
        (Err(Error::Erased { .. }), Err(Error::Erased { .. } | Error::ClusterRetired)) => Ok(()),
        (Err(Error::BranchClosed), Err(Error::BranchClosed)) => Ok(()),
        _ => Err(Error::Corrupt {
            place: "differential oracle mismatch",
        }),
    }
}
