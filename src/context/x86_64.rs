#[derive(Debug, Default)]
#[repr(C)]
pub struct Registers {
    r15: usize,
    r14: usize,
    r13: usize,
    r12: usize,
    rbp: usize,
    rbx: usize,
    rip: usize,
}

impl Registers {
    #[naked]
    #[inline(never)]
    pub unsafe extern "sysv64" fn switch(_from: &mut *mut Self, _to: &mut *mut Self) {
        llvm_asm!(
        "
        // push rip (by caller)

        // Save self callee-save registers
        push rbx
        push rbp
        push r12
        push r13
        push r14
        push r15

        // Switch stacks
        mov [rdi], rsp      // *rdi == from_rsp
        mov rsp, [rsi]      // *rsi == to_rsp

        // Load target callee-save registers
        pop r15
        pop r14
        pop r13
        pop r12
        pop rbp
        pop rbx"
        :::: "intel" "volatile" "alignstack");

        // Load arg0 for entry
        #[cfg(target_os = "uefi")]
        llvm_asm!("mov rdi, rcx" :::: "intel" "volatile");
        #[cfg(not(target_os = "uefi"))]
        llvm_asm!("mov rdi, rbx" :::: "intel" "volatile");

        // ret (pop rip)
    }

    pub unsafe fn new(
        entry: extern "C" fn(usize) -> !,
        arg0: usize,
        stack_top: usize,
    ) -> *mut Self {
        let context = Self {
            rip: entry as usize,
            rbx: arg0,
            ..Self::default()
        };
        // push a Context at stack top
        let rsp = (stack_top as *mut Self).sub(1);
        rsp.write(context);
        rsp
    }
}

#[derive(Debug, Default)]
#[repr(C)]
pub struct RegistersCR3 {
    cr3: usize,
    r15: usize,
    r14: usize,
    r13: usize,
    r12: usize,
    rbp: usize,
    rbx: usize,
    rip: usize,
}

impl RegistersCR3 {
    #[naked]
    #[inline(never)]
    pub unsafe extern "sysv64" fn switch(_from: &mut *mut Self, _to: &mut *mut Self) {
        llvm_asm!(
        "
        // push rip (by caller)

        // Save self callee-save registers
        push rbx
        push rbp
        push r12
        push r13
        push r14
        push r15

        // Save self CR3
        mov r15, cr3
        push r15

        // Switch stacks
        mov [rdi], rsp      // *rdi == from_rsp
        mov rsp, [rsi]      // *rsi == to_rsp

        // Load target CR3
        pop r15
        mov cr3, r15

        // Load target callee-save registers
        pop r15
        pop r14
        pop r13
        pop r12
        pop rbp
        pop rbx"
        :::: "intel" "volatile" "alignstack");

        // Load arg0 for entry
        #[cfg(target_os = "uefi")]
        llvm_asm!("mov rdi, rcx" :::: "intel" "volatile");
        #[cfg(not(target_os = "uefi"))]
        llvm_asm!("mov rdi, rbx" :::: "intel" "volatile");

        // ret (pop rip)
    }

    pub unsafe fn new(
        entry: extern "C" fn(usize) -> !,
        arg0: usize,
        stack_top: usize,
        cr3: usize,
    ) -> *mut Self {
        let context = Self {
            rip: entry as usize,
            rbx: arg0,
            cr3,
            ..Self::default()
        };
        // push a Context at stack top
        let rsp = (stack_top as *mut Self).sub(1);
        rsp.write(context);
        rsp
    }
}
