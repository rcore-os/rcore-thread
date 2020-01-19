#![no_std]
#![no_main]
#![feature(asm, global_asm)]
#![feature(alloc_error_handler)]
#![deny(warnings)]

extern crate alloc;

use alloc::sync::Arc;
use alloc::vec::Vec;
use rcore_thread::{std_thread as thread, *};

#[macro_use]
mod io;
mod runtime;
mod sbi;

#[no_mangle]
pub extern "C" fn rust_main(cpu_id: usize) -> ! {
    crate::runtime::init(cpu_id);
    println!("Hello! I'm CPU {}.", cpu_id);

    // init processor
    let scheduler = scheduler::RRScheduler::new(5);
    let thread_pool = Arc::new(ThreadPool::new(scheduler, MAX_THREAD_NUM));
    unsafe {
        processor().init(cpu_id, thread_pool);
    }
    if cpu_id != 0 {
        processor().run();
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

const MAX_CPU_NUM: usize = 4;
const MAX_THREAD_NUM: usize = 32;

/// Define global `Processor` for each core.
static PROCESSORS: [Processor; MAX_CPU_NUM] = [
    Processor::new(),
    Processor::new(),
    Processor::new(),
    Processor::new(),
];

fn cpu_id() -> usize {
    let id: usize;
    unsafe {
        asm!("" : "={x4}"(id) ::: "volatile");
    }
    id
}

/// Implement dependency for `rcore_thread::std_thread`
#[export_name = "hal_thread_processor"]
pub extern "C" fn processor() -> &'static Processor {
    &PROCESSORS[cpu_id()]
}
