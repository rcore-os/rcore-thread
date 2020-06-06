#![no_std]
#![no_main]
#![feature(llvm_asm, global_asm)]
#![feature(alloc_error_handler)]
#![deny(warnings)]

extern crate alloc;

use alloc::{boxed::Box, sync::Arc};
use rcore_thread::{context::Registers, std_thread as thread, *};

#[macro_use]
mod io;
mod runtime;
mod sbi;

#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    crate::runtime::init();

    // init processor
    let scheduler = scheduler::RRScheduler::new(5);
    let thread_pool = Arc::new(ThreadPool::new(scheduler, MAX_PROC_NUM));
    unsafe {
        processor().init(0, Thread::init(), thread_pool);
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

const STACK_SIZE: usize = 0x2000;
const MAX_CPU_NUM: usize = 1;
const MAX_PROC_NUM: usize = 32;

#[repr(C)]
struct Thread {
    rsp: *mut Registers,
    stack: [u8; STACK_SIZE],
}

impl Thread {
    #[allow(deprecated)]
    unsafe fn init() -> Box<Self> {
        Box::new(core::mem::uninitialized())
    }
    fn new(entry: extern "C" fn(usize) -> !, arg0: usize) -> Box<Self> {
        let mut thread = unsafe { Thread::init() };
        let stack_top = thread.stack.as_ptr() as usize + STACK_SIZE;
        thread.rsp = unsafe { Registers::new(entry, arg0, stack_top) };
        thread
    }
}

/// Implement `switch_to` for a thread
impl Context for Thread {
    /// Switch to another thread.
    unsafe fn switch_to(&mut self, target: &mut dyn Context) {
        let (to, _): (&mut Thread, usize) = core::mem::transmute(target);
        Registers::switch(&mut self.rsp, &mut to.rsp);
    }
}

/// Define global `Processor` for each core.
static PROCESSORS: [Processor; MAX_CPU_NUM] = [Processor::new()];

/// Now we only have one core.
fn cpu_id() -> usize {
    0
}

/// Implement dependency for `rcore_thread::std_thread`
#[no_mangle]
pub fn processor() -> &'static Processor {
    &PROCESSORS[cpu_id()]
}

/// Implement dependency for `rcore_thread::std_thread`
#[no_mangle]
pub fn new_kernel_context(entry: extern "C" fn(usize) -> !, arg0: usize) -> Box<dyn Context> {
    Thread::new(entry, arg0)
}
