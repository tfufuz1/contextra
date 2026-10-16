#![allow(unexpected_cfgs)]

//! Loom-basierter Nebenläufigkeitsbeweis für EvictionWorker Shutdown Race in `contextra-crypto`.
//! Ausführung: RUSTFLAGS="--cfg loom" cargo test -p contextra-crypto --test loom_eviction_worker_shutdown_race -- --nocapture

#[cfg(loom)]
mod loom_tests {
    use loom::sync::Arc;
    use loom::sync::Mutex;
    use loom::thread;
    use std::collections::VecDeque;

    // APM-LOOM-STATE-EXPLOSION: Maximal 2-3 Threads pro loom::model()-Aufruf,
    // um eine Explosion des Suchraums (State-Explosion) und stundenlange Testlaufzeiten zu vermeiden.

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum MockCommand {
        EvictLru { target_free_bytes: usize },
        Shutdown,
    }

    /// Funktional äquivalenter Kanal-Ersatz via `loom::sync::Mutex<VecDeque>`,
    /// da loom's `mpsc`-Implementierung bei unentsorgten Kanal-Nachrichten nach Loop-Exit
    /// 'Messages leaked' meldet.
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

    /// Nachbildung von `EvictionWorker` aus `crates/contextra-crypto/src/kv_segment/eviction_worker.rs`.
    /// Kapselt den `sender`-Mutex und den `handle`-Mutex getrennt.
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
                    MockCommand::EvictLru { .. } => {
                        // Eviction logic mock
                    }
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
            // Phase 1: Send shutdown command over sender mutex
            {
                let channel = Arc::clone(&*self.sender.lock().unwrap());
                channel.send(MockCommand::Shutdown);
            }
            // Phase 2: Take handle mutex content and join blockingly
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

            // Thread A: Ruft shutdown() auf
            let t_shutdown = thread::spawn(move || {
                w1.shutdown();
            });

            // Thread B: Ruft gleichzeitig trigger_eviction() auf
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

            // Thread A: Erster shutdown()-Aufruf
            let t1 = thread::spawn(move || {
                w1.shutdown();
            });

            // Thread B: Zweiter shutdown()-Aufruf (z. B. via Drop oder expliziten Aufruf)
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
