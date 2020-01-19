#[cfg(target_arch = "riscv32")]
global_asm!(
    r"
.equ XLENB, 4
.macro LOAD reg, mem
    lw \reg, \mem
.endm
.macro STORE reg, mem
    sw \reg, \mem
.endm"
);
#[cfg(target_arch = "riscv64")]
global_asm!(
    r"
.equ XLENB, 8
.macro LOAD reg, mem
    ld \reg, \mem
.endm
.macro STORE reg, mem
    sd \reg, \mem
.endm"
);

#[derive(Debug, Default)]
#[repr(C)]
struct Registers {
    /// Callee-saved registers
    s: [usize; 12],
    /// Return address
    ra: usize,
}

#[repr(C)]
pub struct Context(*mut Registers);

impl Context {
    #[naked]
    #[inline(never)]
    pub unsafe extern "C" fn switch_to(&mut self, _target: &mut Self) {
        asm!("
        // save from's registers
        addi  sp, sp, (-XLENB*13)
        STORE sp, 0(a0)
        STORE s0, 0*XLENB(sp)
        STORE s1, 1*XLENB(sp)
        STORE s2, 2*XLENB(sp)
        STORE s3, 3*XLENB(sp)
        STORE s4, 4*XLENB(sp)
        STORE s5, 5*XLENB(sp)
        STORE s6, 6*XLENB(sp)
        STORE s7, 7*XLENB(sp)
        STORE s8, 8*XLENB(sp)
        STORE s9, 9*XLENB(sp)
        STORE s10, 10*XLENB(sp)
        STORE s11, 11*XLENB(sp)
        STORE ra, 12*XLENB(sp)

        // restore to's registers
        LOAD sp, 0(a1)
        LOAD s0, 0*XLENB(sp)
        LOAD s1, 1*XLENB(sp)
        LOAD s2, 2*XLENB(sp)
        LOAD s3, 3*XLENB(sp)
        LOAD s4, 4*XLENB(sp)
        LOAD s5, 5*XLENB(sp)
        LOAD s6, 6*XLENB(sp)
        LOAD s7, 7*XLENB(sp)
        LOAD s8, 8*XLENB(sp)
        LOAD s9, 9*XLENB(sp)
        LOAD s10, 10*XLENB(sp)
        LOAD s11, 11*XLENB(sp)
        LOAD ra, 12*XLENB(sp)
        addi sp, sp, (XLENB*13)

        // load arg0 for entry
        mv a0, s0

        STORE zero, 0(a1)
        ret"
        : : : : "volatile" )
    }

    pub unsafe fn new(
        entry: extern "C" fn(usize) -> !,
        arg0: usize,
        stack_top: usize,
    ) -> Self {
        let mut context = Registers::default();
        context.ra = entry as usize;
        context.s[0] = arg0;

        // push a Context at stack top
        let sp = (stack_top as *mut Registers).sub(1);
        sp.write(context);
        Context(sp)
    }

    pub unsafe fn uninit() -> Self {
        Context(core::ptr::null_mut())
    }
}

#[derive(Debug, Default)]
#[repr(C)]
pub struct RegistersSatp {
    /// Callee-saved registers
    s: [usize; 12],
    /// Return address
    ra: usize,
    /// Page table token
    satp: usize,
}

impl RegistersSatp {
    #[naked]
    #[inline(never)]
    pub unsafe extern "C" fn switch(_from: &mut *mut Self, _to: &mut *mut Self) {
        asm!("
        // save from's registers
        addi  sp, sp, (-XLENB*14)
        STORE sp, 0(a0)
        STORE s0, 0*XLENB(sp)
        STORE s1, 1*XLENB(sp)
        STORE s2, 2*XLENB(sp)
        STORE s3, 3*XLENB(sp)
        STORE s4, 4*XLENB(sp)
        STORE s5, 5*XLENB(sp)
        STORE s6, 6*XLENB(sp)
        STORE s7, 7*XLENB(sp)
        STORE s8, 8*XLENB(sp)
        STORE s9, 9*XLENB(sp)
        STORE s10, 10*XLENB(sp)
        STORE s11, 11*XLENB(sp)
        STORE ra, 12*XLENB(sp)
        csrr  s11, satp
        STORE s11, 13*XLENB(sp)

        // restore to's registers
        LOAD sp, 0(a1)
        LOAD s11, 13*XLENB(sp)
        csrw satp, s11
        LOAD s0, 0*XLENB(sp)
        LOAD s1, 1*XLENB(sp)
        LOAD s2, 2*XLENB(sp)
        LOAD s3, 3*XLENB(sp)
        LOAD s4, 4*XLENB(sp)
        LOAD s5, 5*XLENB(sp)
        LOAD s6, 6*XLENB(sp)
        LOAD s7, 7*XLENB(sp)
        LOAD s8, 8*XLENB(sp)
        LOAD s9, 9*XLENB(sp)
        LOAD s10, 10*XLENB(sp)
        LOAD s11, 11*XLENB(sp)
        LOAD ra, 12*XLENB(sp)
        addi sp, sp, (XLENB*14)

        // load arg0 for entry
        mv a0, s0

        STORE zero, 0(a1)
        ret"
        : : : : "volatile" )
    }

    pub unsafe fn new(
        entry: extern "C" fn(usize) -> !,
        arg0: usize,
        stack_top: usize,
        satp: usize,
    ) -> *mut Self {
        let mut context = Self::default();
        context.ra = entry as usize;
        context.s[0] = arg0;
        context.satp = satp;

        // push a Context at stack top
        let rsp = (stack_top as *mut Self).sub(1);
        rsp.write(context);
        rsp
    }
}
