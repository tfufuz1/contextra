#[tracing::instrument(skip_all)]
fn a(key: &[u8]) {}

#[instrument(skip(self, key, doc_id), fields(n = 1))]
async fn b(&self, key: &[u8], doc_id: DocId, keys_len: usize) {}

#[instrument]
fn c(count: usize, monkey: Zoo<id::Kind>) {}

fn d(id: u64) {}
