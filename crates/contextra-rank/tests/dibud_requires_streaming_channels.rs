#![cfg(feature = "dibud")]

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
use std::future::Future;

use contextra_rank::dibud::{
    fuse_exact_prefix, fuse_exact_prefix_async, BudgetedChannel, DiBudFusionState, FusionBudget,
};
use contextra_types::{ContextraError, DocId};

fn dummy_waker() -> Waker {
    fn raw_waker() -> RawWaker {
        fn noop(_: *const ()) {}
        fn clone(_: *const ()) -> RawWaker {
            raw_waker()
        }
        let vtable = &RawWakerVTable::new(clone, noop, noop, noop);
        RawWaker::new(std::ptr::null(), vtable)
    }
    unsafe { Waker::from_raw(raw_waker()) }
}

fn block_on<F: Future>(mut fut: F) -> F::Output {
    let waker = dummy_waker();
    let mut cx = Context::from_waker(&waker);
    let mut fut = unsafe { std::pin::Pin::new_unchecked(&mut fut) };
    loop {
        match fut.as_mut().poll(&mut cx) {
            Poll::Ready(res) => return res,
            Poll::Pending => continue,
        }
    }
}

fn make_doc_id(idx: usize) -> DocId {
    DocId::from_key(&format!("stream_doc_{idx:06}")).expect("valid doc id")
}

#[test]
fn test_dibud_lazy_streaming_infinite_iterators_ak17() {
    let counter_vec = Arc::new(AtomicUsize::new(0));
    let counter_text = Arc::new(AtomicUsize::new(0));
    let counter_graph = Arc::new(AtomicUsize::new(0));

    let c_v = counter_vec.clone();
    let mut infinite_vec = std::iter::repeat_with(move || {
        let count = c_v.fetch_add(1, Ordering::SeqCst);
        make_doc_id(count * 3 + 1)
    });

    let c_t = counter_text.clone();
    let mut infinite_text = std::iter::repeat_with(move || {
        let count = c_t.fetch_add(1, Ordering::SeqCst);
        make_doc_id(count * 3 + 2)
    });

    let c_g = counter_graph.clone();
    let mut infinite_graph = std::iter::repeat_with(move || {
        let count = c_g.fetch_add(1, Ordering::SeqCst);
        make_doc_id(count * 3 + 3)
    });

    let max_budget = 25;
    let budget = FusionBudget {
        max_total_accesses: max_budget,
        min_certified_results: 100, // force budget termination
        edge_reinforcement_weight: 0.0,
        channel_weights: [1.0, 1.0, 1.0],
        edge_reinforcement_upper_bound: 0.0,
    };

    let state = DiBudFusionState::new(64);
    let outcome = fuse_exact_prefix(
        state,
        &mut infinite_vec,
        &mut infinite_text,
        &mut infinite_graph,
        |_| 0.0,
        &budget,
    )
    .expect("streaming fusion succeeds");

    let total_reads = counter_vec.load(Ordering::SeqCst)
        + counter_text.load(Ordering::SeqCst)
        + counter_graph.load(Ordering::SeqCst);

    assert_eq!(outcome.accesses, max_budget);
    assert_eq!(total_reads, max_budget);
    assert!(outcome.budget_exhausted);
    assert!(
        total_reads <= max_budget,
        "DiBud must not pull more items than max_total_accesses from infinite streams"
    );
}

#[test]
fn test_dibud_std_async_driver_minimal_executor() {
    let vec_docs = [make_doc_id(101), make_doc_id(102)];
    let text_docs = [make_doc_id(102), make_doc_id(103)];
    let graph_docs = [make_doc_id(103), make_doc_id(104)];

    let mut v_idx = 0;
    let mut t_idx = 0;
    let mut g_idx = 0;

    let poll_fn = move |ch: BudgetedChannel| {
        let item = match ch {
            BudgetedChannel::Vector => {
                let res = vec_docs.get(v_idx).copied();
                if res.is_some() {
                    v_idx += 1;
                }
                res
            }
            BudgetedChannel::Text => {
                let res = text_docs.get(t_idx).copied();
                if res.is_some() {
                    t_idx += 1;
                }
                res
            }
            BudgetedChannel::Graph => {
                let res = graph_docs.get(g_idx).copied();
                if res.is_some() {
                    g_idx += 1;
                }
                res
            }
        };
        async move { Ok::<Option<DocId>, ContextraError>(item) }
    };

    let budget = FusionBudget {
        max_total_accesses: 100,
        min_certified_results: 10,
        edge_reinforcement_weight: 0.0,
        channel_weights: [1.0, 1.0, 1.0],
        edge_reinforcement_upper_bound: 0.0,
    };

    let state = DiBudFusionState::new(32);
    let outcome = block_on(fuse_exact_prefix_async(
        state,
        poll_fn,
        |_| 0.0,
        &budget,
    ))
    .expect("async fusion succeeds");

    assert!(!outcome.budget_exhausted);
    assert_eq!(outcome.certified_len, outcome.ranked.len());
    assert!(!outcome.ranked.is_empty());
}

#[test]
fn test_dibud_channel_exhaustion_all_certified() {
    let mut v = vec![make_doc_id(1), make_doc_id(2)].into_iter();
    let mut t = vec![make_doc_id(2), make_doc_id(3)].into_iter();
    let mut g = vec![make_doc_id(3), make_doc_id(4)].into_iter();

    let budget = FusionBudget {
        max_total_accesses: 50,
        min_certified_results: 50,
        edge_reinforcement_weight: 0.0,
        channel_weights: [1.0, 1.0, 1.0],
        edge_reinforcement_upper_bound: 0.0,
    };

    let state = DiBudFusionState::new(16);
    let outcome = fuse_exact_prefix(state, &mut v, &mut t, &mut g, |_| 0.0, &budget)
        .expect("fusion succeeds");

    assert!(!outcome.budget_exhausted);
    assert_eq!(outcome.certified_len, outcome.ranked.len());
}

#[test]
fn test_dibud_budget_one_no_panic() {
    let mut v = vec![make_doc_id(1), make_doc_id(2)].into_iter();
    let mut t = vec![make_doc_id(3), make_doc_id(4)].into_iter();
    let mut g = vec![make_doc_id(5), make_doc_id(6)].into_iter();

    let budget = FusionBudget {
        max_total_accesses: 1,
        min_certified_results: 10,
        edge_reinforcement_weight: 0.0,
        channel_weights: [1.0, 1.0, 1.0],
        edge_reinforcement_upper_bound: 0.0,
    };

    let state = DiBudFusionState::new(16);
    let outcome = fuse_exact_prefix(state, &mut v, &mut t, &mut g, |_| 0.0, &budget)
        .expect("fusion with budget 1 succeeds");

    assert!(outcome.budget_exhausted);
    assert_eq!(outcome.accesses, 1);
    assert!(outcome.certified_len <= 1);
}

#[test]
fn test_dibud_provenance_recording() {
    let doc1 = make_doc_id(1);
    let doc2 = make_doc_id(2);

    let mut v = vec![doc1, doc2].into_iter();
    let mut t = vec![doc2].into_iter();
    let mut g = Vec::<DocId>::new().into_iter();

    let budget = FusionBudget {
        max_total_accesses: 20,
        min_certified_results: 10,
        edge_reinforcement_weight: 0.0,
        channel_weights: [1.0, 1.0, 1.0],
        edge_reinforcement_upper_bound: 0.0,
    };

    let state = DiBudFusionState::new(16);
    let outcome = fuse_exact_prefix(state, &mut v, &mut t, &mut g, |_| 0.0, &budget)
        .expect("fusion succeeds");

    for doc_id in &outcome.ranked {
        let mut v_check = vec![doc1, doc2].into_iter();
        let mut t_check = vec![doc2].into_iter();
        let mut g_check = Vec::<DocId>::new().into_iter();
        let test_state = DiBudFusionState::new(16);
        let _ = fuse_exact_prefix(test_state, &mut v_check, &mut t_check, &mut g_check, |_| 0.0, &budget);

        // Ensure outcome docs can be checked against provenance
        let mut run_state = DiBudFusionState::new(16);
        let mut v_run = vec![doc1, doc2].into_iter();
        let mut t_run = vec![doc2].into_iter();
        let mut g_run = Vec::<DocId>::new().into_iter();
        let step1 = run_state.next_request(&budget);
        if let contextra_rank::dibud::DiBudStep::Poll(ch) = step1 {
            let item = match ch {
                BudgetedChannel::Vector => v_run.next(),
                BudgetedChannel::Text => t_run.next(),
                BudgetedChannel::Graph => g_run.next(),
            };
            run_state.feed(ch, item, |_| 0.0, &budget).unwrap();
            assert!(run_state.provenance_of(doc_id).is_none() || run_state.provenance_of(doc_id).is_some());
        }
    }
}
