#![no_std]
#![no_main]
#![feature(asm)]
#![feature(lang_items)]
#![deny(warnings)]

extern crate alloc;

use alloc::sync::Arc;
use blog_os::serial_println as println;
use rcore_thread::{std_thread as thread, *};

mod runtime;

const MAX_CPU_NUM: usize = 1;
const MAX_PROC_NUM: usize = 32;

/// The entry of the kernel
#[no_mangle]
pub extern "C" fn _start() -> ! {
    self::runtime::init();

    // init processor
    let scheduler = scheduler::RRScheduler::new(5);
    let thread_pool = Arc::new(ThreadPool::new(scheduler, MAX_PROC_NUM));
    unsafe {
        processor().init(0, thread_pool);
    }
    // init threads
    thread::spawn(|| {
        let tid = processor().tid();
        println!("[{}] yield", tid);
        thread::yield_now();
        println!("[{}] spawn", tid);
        let t2 = thread::spawn(|| {
            let tid = processor().tid();
            println!("[{}] yield", tid);
            thread::yield_now();
            println!("[{}] return 8", tid);
            8
        });
        println!("[{}] join", tid);
        let ret = t2.join();
        println!("[{}] get {:?}", tid, ret);
        println!("[{}] exit", tid);
    });
    // run threads
    processor().run();
}

/// Define global `Processor` for each core.
static PROCESSORS: [Processor; MAX_CPU_NUM] = [Processor::new()];

/// Now we only have one core.
fn cpu_id() -> usize {
    0
}

/// Implement dependency for `rcore_thread::std_thread`
#[export_name = "hal_thread_processor"]
pub extern "C" fn processor() -> &'static Processor {
    &PROCESSORS[cpu_id()]
}
