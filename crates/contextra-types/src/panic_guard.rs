//! Panic protection and handle health state management.

use crate::error::{ContextraError, Result};
use std::any::Any;
use std::future::Future;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::task::{Context, Poll};

/// Total count of caught panics across the system.
pub static CONTEXTRA_PANICS_TOTAL: AtomicU64 = AtomicU64::new(0);

/// Health status classification of a facade or handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum HealthState {
    /// Normal operation.
    #[default]
    Healthy,
    /// Unwind caught; handle is poisoned.
    Poisoned,
}

impl HealthState {
    /// Returns `true` if the state is `Healthy`.
    pub fn is_healthy(self) -> bool {
        matches!(self, Self::Healthy)
    }

    /// Returns `true` if the state is `Poisoned`.
    pub fn is_poisoned(self) -> bool {
        matches!(self, Self::Poisoned)
    }
}

/// Poison hook callback signature called when a panic is caught.
pub type PoisonHook = Box<dyn Fn() + Send + Sync + 'static>;

struct CatchUnwind<F> {
    inner: Pin<Box<F>>,
}

impl<F: Future> Future for CatchUnwind<F> {
    type Output = std::result::Result<F::Output, Box<dyn Any + Send>>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let inner = self.inner.as_mut();
        match catch_unwind(AssertUnwindSafe(|| inner.poll(cx))) {
            Ok(Poll::Ready(val)) => Poll::Ready(Ok(val)),
            Ok(Poll::Pending) => Poll::Pending,
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}

fn catch_unwind_safe<F: Future>(f: F) -> CatchUnwind<F> {
    CatchUnwind {
        inner: Box::pin(f),
    }
}

fn extract_panic_message(payload: &Box<dyn Any + Send>) -> String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "unknown panic payload".to_string()
    }
}

/// Truncates a panic message to at most 100 characters and strips any potential user data.
fn sanitize_panic_message(msg: &str) -> String {
    let truncated = if msg.len() > 100 {
        &msg[..100]
    } else {
        msg
    };
    truncated.replace(['\r', '\n'], " ")
}

/// Wraps an async operation with panic catching, panic metric increment, and poison hook execution.
pub async fn guard_panic_with_hook<F, T>(op_name: &str, hook: Option<&PoisonHook>, f: F) -> Result<T>
where
    F: Future<Output = Result<T>> + Send,
{
    match catch_unwind_safe(f).await {
        Ok(res) => res,
        Err(payload) => {
            CONTEXTRA_PANICS_TOTAL.fetch_add(1, Ordering::Relaxed);
            if let Some(h) = hook {
                h();
            }
            let raw_msg = extract_panic_message(&payload);
            let clean_msg = sanitize_panic_message(&raw_msg);
            Err(ContextraError::Internal(format!(
                "panic in {op_name}: {clean_msg}"
            )))
        }
    }
}

/// Wraps an async operation with panic catching and panic metric increment.
pub async fn guard_panic<F, T>(op_name: &str, f: F) -> Result<T>
where
    F: Future<Output = Result<T>> + Send,
{
    guard_panic_with_hook(op_name, None, f).await
}

/// Declarative macro wrapping public async API methods with [`guard_panic`].
#[macro_export]
macro_rules! public_api {
    ($op_name:expr, $body:expr) => {
        $crate::guard_panic($op_name, async move { $body }).await
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicBool;
    use std::sync::Arc;

    fn block_on<F: Future>(f: F) -> F::Output {
        use std::sync::Arc;
        use std::task::{Context, Poll, Wake, Waker};

        struct DummyWaker;
        impl Wake for DummyWaker {
            fn wake(self: Arc<Self>) {}
        }

        let waker = Waker::from(Arc::new(DummyWaker));
        let mut cx = Context::from_waker(&waker);
        let mut pinned = Box::pin(f);
        loop {
            if let Poll::Ready(val) = pinned.as_mut().poll(&mut cx) {
                return val;
            }
        }
    }

    #[test]
    fn test_guard_panic_ok() {
        block_on(async {
            let res = guard_panic("test_op", async { Ok(42) }).await;
            assert_eq!(res.unwrap(), 42);
        });
    }

    #[test]
    fn test_guard_panic_catches_unwind_and_increments_metric() {
        block_on(async {
            let initial_panics = CONTEXTRA_PANICS_TOTAL.load(Ordering::Relaxed);
            let hook_called = Arc::new(AtomicBool::new(false));
            let hook_clone = hook_called.clone();
            let hook: PoisonHook = Box::new(move || {
                hook_clone.store(true, Ordering::Relaxed);
            });

            let res: Result<()> = guard_panic_with_hook("panic_op", Some(&hook), async {
                panic!("something went terribly wrong!");
            })
            .await;

            assert!(res.is_err());
            let err_str = res.unwrap_err().to_string();
            assert!(err_str.contains("panic in panic_op: something went terribly wrong!"));
            assert!(hook_called.load(Ordering::Relaxed));
            assert_eq!(
                CONTEXTRA_PANICS_TOTAL.load(Ordering::Relaxed),
                initial_panics + 1
            );
        });
    }
}
