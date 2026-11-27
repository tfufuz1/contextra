#[tracing::instrument] //~
fn a(&self, key: &[u8]) {}

#[instrument(level = "debug", skip(self))] //~
async fn b<F: Fn(u8) -> u8>(&self, doc_id: DocId, f: F) {}

#[instrument(skip(key))] //~
#[must_use]
pub fn c(key: &[u8], mut target: Target) {}

#[cfg_attr(feature = "trace", tracing::instrument)] //~
fn d(_term: &str) {}
