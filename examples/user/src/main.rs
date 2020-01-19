use std::sync::Arc;

use rcore_thread::{std_thread as thread, *};

const MAX_THREAD_NUM: usize = 32;

fn main() {
    env_logger::init();

    // init processor
    let scheduler = scheduler::RRScheduler::new(5);
    let thread_pool = Arc::new(ThreadPool::new(scheduler, MAX_THREAD_NUM));

    // use system threads to emulate CPUs
    for cpu_id in 1..4 {
        let thread_pool = thread_pool.clone();
        std::thread::spawn(move || {
            unsafe {
                processor().init(cpu_id, thread_pool);
            }
            processor().run();
        });
    }

    unsafe {
        processor().init(0, thread_pool);
    }
    // init threads
    thread::spawn(|| {
        let tid = thread::current().id();
        println!("[{}] yield", tid);
        thread::yield_now();
        let handles: Vec<_> = (0..8)
            .map(|i| {
                println!("[{}] spawn {}", tid, i);
                thread::spawn(move || {
                    let tid = thread::current().id();
                    println!("[{}] yield", tid);
                    thread::yield_now();
                    println!("[{}] return {}", tid, i);
                    i
                })
            })
            .collect();
        for handle in handles {
            let ret = handle.join();
            println!("[{}] join => {:?}", tid, ret);
        }
        println!("[{}] exit", tid);
    });
    // run threads
    processor().run();
}

thread_local! {
    /// Define global `Processor` for each core.
    static PROCESSOR: Processor = Processor::new();
}

/// Implement dependency for `rcore_thread::std_thread`
#[export_name = "hal_thread_processor"]
pub extern "C" fn processor() -> &'static Processor {
    // UNSAFE: extend lifetime
    PROCESSOR.with(|p| unsafe { std::mem::transmute(p) })
}
