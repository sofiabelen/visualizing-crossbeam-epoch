use crossbeam_epoch::{Atomic, Collector, Shared};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

struct DropTracker {
    flag: Arc<AtomicBool>
}

impl Drop for DropTracker {
    fn drop(&mut self) {
        self.flag.store(true, Ordering::SeqCst);
    }
}

fn run(t1_holds_a_guard: bool) {
    let collector = Collector::new();
    let (t1, t2) = (collector.register(), collector.register());

    let flag = Arc::new(AtomicBool::new(false));
    let tracker = Atomic::new(DropTracker {
        flag: flag.clone()
    });

    let g1 = t1_holds_a_guard.then(|| t1.pin());

    {
        let g2 = t2.pin();
        let old = tracker.swap(Shared::null(), Ordering::AcqRel, &g2);
        unsafe { g2.defer_destroy(old) };
    } // g2 unpinned here

    let mut flushes = 0;
    while !flag.load(Ordering::SeqCst) && flushes < 5 {
        t2.pin().flush(); // fresh pin every time (more on this below)
        flushes += 1;
        println!("flush #{flushes}: flag = {}", flag.load(Ordering::SeqCst));
    }

    if g1.is_some() && !flag.load(Ordering::SeqCst) {
        drop(g1);
        println!("-- T1 unpins --");
        while !flag.load(Ordering::SeqCst) && flushes < 10 {
            t2.pin().flush();
            flushes += 1;
            println!("flush #{}: flag = {}", flushes, flag.load(Ordering::SeqCst));
        }
    }
}

fn main() {
    run(false);
}