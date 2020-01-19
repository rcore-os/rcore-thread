#![no_std]
#![no_main]
#![feature(abi_efiapi)]
#![deny(warnings)]

extern crate alloc;

use alloc::sync::Arc;
use log::*;
use rcore_thread::{std_thread as thread, *};
use uefi::prelude::*;

const MAX_CPU_NUM: usize = 1;
const MAX_THREAD_NUM: usize = 32;

#[entry]
fn efi_main(_image: Handle, st: SystemTable<Boot>) -> uefi::Status {
    uefi_services::init(&st).expect_success("Failed to initialize utilities");

    // init processor
    let scheduler = scheduler::RRScheduler::new(5);
    let thread_pool = Arc::new(ThreadPool::new(scheduler, MAX_THREAD_NUM));
    unsafe {
        processor().init(0, thread_pool);
    }
    // init threads
    thread::spawn(|| {
        let tid = processor().tid();
        info!("[{}] yield", tid);
        thread::yield_now();
        info!("[{}] spawn", tid);
        let t2 = thread::spawn(|| {
            let tid = processor().tid();
            info!("[{}] yield", tid);
            thread::yield_now();
            info!("[{}] return 8", tid);
            8
        });
        info!("[{}] join", tid);
        let ret = t2.join();
        info!("[{}] get {:?}", tid, ret);
        info!("[{}] exit", tid);
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
