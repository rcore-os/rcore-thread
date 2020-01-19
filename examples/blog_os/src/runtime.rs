use blog_os::{exit_qemu, gdt, interrupts::init_idt, serial_println};
use core::alloc::Layout;
use core::panic::PanicInfo;
use linked_list_allocator::LockedHeap;

pub fn init() {
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

const HEAP_SIZE: usize = 0x100000;

static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

#[lang = "oom"]
fn oom(_: Layout) -> ! {
    panic!("out of memory");
}
