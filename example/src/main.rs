#![no_std]
#![no_main]
#![feature(asm)]
#![feature(alloc)]
#![feature(lang_items)]

extern crate alloc;

use alloc::{boxed::Box, sync::Arc};
use core::alloc::Layout;
use core::panic::PanicInfo;

use blog_os::{exit_qemu, gdt, interrupts::init_idt, serial_println};
use linked_list_allocator::LockedHeap;
use rcore_thread::{context::Registers, std_thread as thread, *};

const STACK_SIZE: usize = 0x2000;
const HEAP_SIZE: usize = 0x100000;
const MAX_CPU_NUM: usize = 1;
const MAX_PROC_NUM: usize = 32;

/// The entry of the kernel
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // init x86
    gdt::init();
    init_idt();
    // init log
    init_log();
    // init heap
    unsafe {
        HEAP_ALLOCATOR
            .lock()
            .init(HEAP.as_ptr() as usize, HEAP_SIZE);
    }
    // init processor
    let scheduler = scheduler::RRScheduler::new(5);
    let thread_pool = Arc::new(ThreadPool::new(scheduler, MAX_PROC_NUM));
    unsafe {
        processor().init(0, Thread::init(), thread_pool);
    }
    // init threads
    thread::spawn(|| {
        let tid = processor().tid();
        serial_println!("[{}] yield", tid);
        thread::yield_now();
        serial_println!("[{}] spawn", tid);
        let t2 = thread::spawn(|| {
            let tid = processor().tid();
            serial_println!("[{}] yield", tid);
            thread::yield_now();
            serial_println!("[{}] return 8", tid);
            8
        });
        serial_println!("[{}] join", tid);
        let ret = t2.join();
        serial_println!("[{}] get {:?}", tid, ret);
        serial_println!("[{}] exit", tid);
    });
    // run threads
    processor().run();
}

fn init_log() {
    use log::*;
    struct SimpleLogger;
    impl Log for SimpleLogger {
        fn enabled(&self, _metadata: &Metadata) -> bool {
            true
        }
        fn log(&self, record: &Record) {
            serial_println!("[{:>5}] {}", record.level(), record.args());
        }
        fn flush(&self) {}
    }
    static LOGGER: SimpleLogger = SimpleLogger;
    set_logger(&LOGGER).unwrap();
    set_max_level(LevelFilter::Trace);
}

#[repr(C)]
struct Thread {
    rsp: *mut Registers,
    stack: [u8; STACK_SIZE],
}

impl Thread {
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
    unsafe fn switch_to(&mut self, target: &mut Context) {
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
pub fn new_kernel_context(entry: extern "C" fn(usize) -> !, arg0: usize) -> Box<Context> {
    Thread::new(entry, arg0)
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("\n{}", info);

    unsafe {
        exit_qemu();
    }
    loop {}
}

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

#[lang = "oom"]
fn oom(_: Layout) -> ! {
    panic!("out of memory");
}
