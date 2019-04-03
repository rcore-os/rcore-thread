#[derive(Debug, Default)]
#[repr(C)]
pub struct Registers {
    x19to29: [usize; 11],
    lr: usize,
}

impl Registers {
    #[naked]
    #[inline(never)]
    pub unsafe extern "C" fn switch(_from: &mut *mut Self, _to: &mut *mut Self) {
        asm!(
        "
        // store self sp
        mov x10, #-(12 * 8)
        add x8, sp, x10
        str x8, [x0]

        // store callee-saved registers
        stp x19, x20, [x8], #16
        stp x21, x22, [x8], #16
        stp x23, x24, [x8], #16
        stp x25, x26, [x8], #16
        stp x27, x28, [x8], #16
        stp x29, lr, [x8], #16

        // load target sp
        ldr x8, [x1]
        str xzr, [x1]

        // load callee-saved registers
        ldp x19, x20, [x8], #16
        ldp x21, x22, [x8], #16
        ldp x23, x24, [x8], #16
        ldp x25, x26, [x8], #16
        ldp x27, x28, [x8], #16
        ldp x29, lr, [x8], #16
        mov sp, x8

        // load arg0 for entry
        mov x0, x19

        ret"
        : : : : "volatile" );
    }

    pub unsafe fn new(
        entry: extern "C" fn(usize) -> !,
        arg0: usize,
        stack_top: usize,
    ) -> *mut Self {
        let mut context = Self::default();
        context.lr = entry as usize;
        context.x19to29[0] = arg0;

        // push a Context at stack top
        let rsp = (stack_top as *mut Self).sub(1);
        rsp.write(context);
        rsp
    }
}
