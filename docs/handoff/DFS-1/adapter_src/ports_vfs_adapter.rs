// Preserved verbatim legacy adapter bridging contextra_ports::Vfs to contextra_durable_fs
// for adoption by class C adapters (ENG-2).

use std::path::Path;

pub struct PortsVfsAdapter<V> {
    inner: V,
}

impl<V> PortsVfsAdapter<V> {
    pub fn new(inner: V) -> Self {
        Self { inner }
    }

    pub fn into_inner(self) -> V {
        self.inner
    }
}
