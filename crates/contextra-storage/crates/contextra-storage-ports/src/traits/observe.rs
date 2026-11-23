//! Metriken und Beobachter (E-44). Keine Inhalte, nur Namen und Zähler.
pub trait Metrics: Send + Sync {
    fn incr(&self, name: &'static str, by: u64);
    fn gauge(&self, name: &'static str, value: u64);
}

pub trait Observer: Send + Sync {
    fn event(&self, name: &'static str, labels: &[(&'static str, &'static str)]);
}
