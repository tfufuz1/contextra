// TODO(acta-altlast): Bruecken-Modul der Altquelle (HR-6 verbietet Bruecken). Referenz fuer die Trait-Implementierungen (GraphIndex, Purger); beim Umbau in direkte Implementierungen aufloesen.
//! Extension trait for  providing entity removal functionality.
use crate::types::EntityId;

pub trait GraphIndexExt: acta_base::GraphIndex {
    fn remove_entity<'a>(
        &'a self,
        tx: acta_base::TxId,
        entity: EntityId,
    ) -> acta_base::BoxFuture<'a, acta_base::Result<()>>;
}
