//! Extension trait for  providing entity removal functionality.
use crate::types::EntityId;

pub trait GraphIndexExt: contextra_ports::GraphIndex {
    fn remove_entity<'a>(
        &'a self,
        tx: contextra_types::TxId,
        entity: EntityId,
    ) -> contextra_ports::BoxFuture<'a, contextra_types::Result<()>>;
}
