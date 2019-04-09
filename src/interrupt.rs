//! Enable and disable interrupt for each architecture.

#[cfg(all(not(feature = "userland"), target_arch = "x86_64"))]
pub use self::x86_64::*;

#[cfg(all(not(feature = "userland"), any(target_arch = "riscv32", target_arch = "riscv64")))]
pub use self::riscv::*;

#[cfg(all(not(feature = "userland"), target_arch = "aarch64"))]
pub use self::aarch64::*;

#[cfg(all(not(feature = "userland"), target_arch = "mips"))]
pub use self::mipsel::*;

#[cfg(feature = "userland")]
pub use self::dummy::*;

#[cfg(target_arch = "x86_64")]
mod x86_64 {
    #[inline]
    pub unsafe fn disable_and_store() -> usize {
        let rflags: usize;
        asm!("pushfq; popq $0; cli" : "=r"(rflags) ::: "volatile");
        rflags & (1 << 9)
    }

    #[inline]
    pub unsafe fn restore(flags: usize) {
        if flags != 0 {
            asm!("sti" :::: "volatile");
        }
    }

    #[inline]
    pub unsafe fn enable_and_wfi() {
        asm!("sti; hlt" :::: "volatile");
    }
}

#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
mod riscv {
    #[inline]
    pub unsafe fn disable_and_store() -> usize {
        let sstatus: usize;
        asm!("csrci sstatus, 1 << 1" : "=r"(sstatus) ::: "volatile");
        sstatus & (1 << 1)
    }

    #[inline]
    pub unsafe fn restore(flags: usize) {
        asm!("csrs sstatus, $0" :: "r"(flags) :: "volatile");
    }

    #[inline]
    pub unsafe fn enable_and_wfi() {
        asm!("csrsi sstatus, 1 << 1; wfi" :::: "volatile");
    }

}

#[cfg(target_arch = "aarch64")]
mod aarch64 {
    #[inline]
    pub unsafe fn disable_and_store() -> usize {
        let daif: u32;
        asm!("mrs $0, DAIF; msr daifset, #2": "=r"(daif) ::: "volatile");
        daif as usize
    }

    #[inline]
    pub unsafe fn restore(flags: usize) {
        asm!("msr DAIF, $0" :: "r"(flags as u32) :: "volatile");
    }

    #[inline]
    pub unsafe fn enable_and_wfi() {
        asm!("msr daifclr, #2; wfi" :::: "volatile");
    }
}

#[cfg(target_arch = "mips")]
mod mipsel {
    #[inline(always)]
    pub unsafe fn disable_and_store() -> usize {
        let cp0_status: usize;
        asm!("mfc0 $0, $$12;" : "=r"(cp0_status) ::: "volatile");
        let cp0_status_new = cp0_status & !1;
        asm!("mtc0 $0, $$12;" : : "r"(cp0_status_new) :: "volatile");
        cp0_status & 1
    }

    #[inline(always)]
    pub unsafe fn restore(flags: usize) {
        let cp0_status: usize;
        asm!("mfc0 $0, $$12;" : "=r"(cp0_status) ::: "volatile");
        let cp0_status_new = cp0_status | flags;
        asm!("mtc0 $0, $$12;" : : "r"(cp0_status_new) :: "volatile");
    }

    #[inline(always)]
    pub unsafe fn enable_and_wfi() {
        let cp0_status: usize;
        asm!("mfc0 $0, $$12;" : "=r"(cp0_status) ::: "volatile");
        let cp0_status_new = cp0_status | 1;
        asm!("mtc0 $0, $$12; wait;" : : "r"(cp0_status_new) :: "volatile");
    }
}


#[cfg(feature = "userland")]
mod dummy {
    #[inline]
    pub unsafe fn disable_and_store() -> usize {
        0
    }

    #[inline]
    pub unsafe fn restore(_flags: usize) {}

    #[inline]
    pub unsafe fn enable_and_wfi() {}
}
