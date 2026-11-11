//! # Loom-Test: `kvcache_loom_eviction_worker_shutdown_race.rs`
// Herkunfts-Commit: ca103a3c743df21a70c3b0b0e2b3972a693a5ca4
//!
//! Validiert die Thread-Sicherheit beim Shutdown des EvictionWorkers in `contextra-kvcache`.
//! Ausführung: RUSTFLAGS="--cfg loom" cargo test --release -p contextra-kvcache --test kvcache_loom_eviction_worker_shutdown_race

#![allow(unexpected_cfgs)]

#[cfg(loom)]
mod loom_tests {
    use loom::sync::Arc;
    use loom::sync::Mutex;
    use loom::thread;
    use std::collections::VecDeque;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum MockCommand {
        EvictLru { target_free_bytes: usize },
        Shutdown,
    }

    struct MockChannel {
        queue: Mutex<VecDeque<MockCommand>>,
    }

    impl MockChannel {
        fn new() -> Self {
            Self {
                queue: Mutex::new(VecDeque::new()),
            }
        }

        fn send(&self, cmd: MockCommand) {
            let mut guard = self.queue.lock().unwrap();
            guard.push_back(cmd);
        }

        fn recv(&self) -> MockCommand {
            loop {
                {
                    let mut guard = self.queue.lock().unwrap();
                    if let Some(cmd) = guard.pop_front() {
                        return cmd;
                    }
                }
                thread::yield_now();
            }
        }
    }

    pub struct MockEvictionWorker {
        sender: Mutex<Arc<MockChannel>>,
        handle: Mutex<Option<thread::JoinHandle<()>>>,
    }

    impl MockEvictionWorker {
        pub fn spawn() -> Self {
            let channel = Arc::new(MockChannel::new());
            let ch_recv = Arc::clone(&channel);

            let handle = thread::spawn(move || loop {
                let cmd = ch_recv.recv();
                match cmd {
                    MockCommand::EvictLru { .. } => {}
                    MockCommand::Shutdown => break,
                }
            });

            Self {
                sender: Mutex::new(channel),
                handle: Mutex::new(Some(handle)),
            }
        }

        pub fn trigger_eviction(&self, target_free_bytes: usize) {
            let channel = Arc::clone(&*self.sender.lock().unwrap());
            channel.send(MockCommand::EvictLru { target_free_bytes });
        }

        pub fn shutdown(&self) {
            {
                let channel = Arc::clone(&*self.sender.lock().unwrap());
                channel.send(MockCommand::Shutdown);
            }
            if let Some(handle) = self.handle.lock().unwrap().take() {
                let _ = handle.join();
            }
        }
    }

    impl Drop for MockEvictionWorker {
        fn drop(&mut self) {
            self.shutdown();
        }
    }

    #[test]
    fn test_shutdown_race_with_concurrent_trigger() {
        loom::model(|| {
            let worker = Arc::new(MockEvictionWorker::spawn());

            let w1 = Arc::clone(&worker);
            let w2 = Arc::clone(&worker);

            let t_shutdown = thread::spawn(move || {
                w1.shutdown();
            });

            let t_trigger = thread::spawn(move || {
                w2.trigger_eviction(1024);
            });

            t_shutdown.join().expect("shutdown thread finished cleanly");
            t_trigger.join().expect("trigger thread finished cleanly");
        });
    }

    #[test]
    fn test_double_shutdown_idempotent() {
        loom::model(|| {
            let worker = Arc::new(MockEvictionWorker::spawn());

            let w1 = Arc::clone(&worker);
            let w2 = Arc::clone(&worker);

            let t1 = thread::spawn(move || {
                w1.shutdown();
            });

            let t2 = thread::spawn(move || {
                w2.shutdown();
            });

            t1.join().expect("first shutdown finished cleanly");
            t2.join().expect("second shutdown finished cleanly");
        });
    }
}

#[cfg(not(loom))]
#[cfg(test)]
mod normal_tests {
    use std::collections::VecDeque;
    use std::sync::Arc;
    use std::sync::Condvar;
    use std::sync::Mutex;
    use std::thread;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum MockCommand {
        EvictLru { target_free_bytes: usize },
        Shutdown,
    }

    struct MockChannel {
        queue: Mutex<VecDeque<MockCommand>>,
        condvar: Condvar,
    }

    impl MockChannel {
        fn new() -> Self {
            Self {
                queue: Mutex::new(VecDeque::new()),
                condvar: Condvar::new(),
            }
        }

        fn send(&self, cmd: MockCommand) {
            let mut guard = self.queue.lock().unwrap();
            guard.push_back(cmd);
            self.condvar.notify_one();
        }

        fn recv(&self) -> MockCommand {
            let mut guard = self.queue.lock().unwrap();
            while guard.is_empty() {
                guard = self.condvar.wait(guard).unwrap();
            }
            guard.pop_front().unwrap()
        }
    }

    pub struct MockEvictionWorker {
        sender: Mutex<Arc<MockChannel>>,
        handle: Mutex<Option<thread::JoinHandle<()>>>,
    }

    impl MockEvictionWorker {
        pub fn spawn() -> Self {
            let channel = Arc::new(MockChannel::new());
            let ch_recv = Arc::clone(&channel);

            let handle = thread::spawn(move || loop {
                let cmd = ch_recv.recv();
                match cmd {
                    MockCommand::EvictLru { .. } => {}
                    MockCommand::Shutdown => break,
                }
            });

            Self {
                sender: Mutex::new(channel),
                handle: Mutex::new(Some(handle)),
            }
        }

        pub fn trigger_eviction(&self, target_free_bytes: usize) {
            let channel = Arc::clone(&*self.sender.lock().unwrap());
            channel.send(MockCommand::EvictLru { target_free_bytes });
        }

        pub fn shutdown(&self) {
            {
                let channel = Arc::clone(&*self.sender.lock().unwrap());
                channel.send(MockCommand::Shutdown);
            }
            if let Some(handle) = self.handle.lock().unwrap().take() {
                let _ = handle.join();
            }
        }
    }

    impl Drop for MockEvictionWorker {
        fn drop(&mut self) {
            self.shutdown();
        }
    }

    #[test]
    fn test_shutdown_race_with_concurrent_trigger_non_loom() {
        let worker = Arc::new(MockEvictionWorker::spawn());
        let w1 = Arc::clone(&worker);
        let w2 = Arc::clone(&worker);

        let t1 = thread::spawn(move || w1.shutdown());
        let t2 = thread::spawn(move || w2.trigger_eviction(512));

        t1.join().unwrap();
        t2.join().unwrap();
    }

    #[test]
    fn test_double_shutdown_idempotent_non_loom() {
        let worker = Arc::new(MockEvictionWorker::spawn());
        let w1 = Arc::clone(&worker);
        let w2 = Arc::clone(&worker);

        let t1 = thread::spawn(move || w1.shutdown());
        let t2 = thread::spawn(move || w2.shutdown());

        t1.join().unwrap();
        t2.join().unwrap();
    }
}
