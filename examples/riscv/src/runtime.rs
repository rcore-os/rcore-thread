use core::alloc::Layout;
use core::panic::PanicInfo;
use linked_list_allocator::LockedHeap;

pub fn init() {
    unsafe {
        HEAP_ALLOCATOR
            .lock()
            .init(HEAP.as_ptr() as usize, HEAP_SIZE);
    }
}

global_asm!(
    r#"
    .section .text.entry
    .globl _start
_start:
    la sp, bootstacktop
    call rust_main

    .section .bss.stack
    .align 12
    .global bootstack
bootstack:
    .space 4096 * 4
    .global bootstacktop
bootstacktop:
"#
);

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[no_mangle]
extern "C" fn abort() -> ! {
    panic!("abort!");
}

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

const HEAP_SIZE: usize = 0x100000;

static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

#[alloc_error_handler]
fn oom(_: Layout) -> ! {
    panic!("out of memory");
}
