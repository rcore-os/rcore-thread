#![allow(dead_code)]

#[cfg(target_arch = "x86_64")]
include!("x86_64.rs");

#[cfg(target_arch = "aarch64")]
include!("aarch64.rs");

#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
include!("riscv.rs");

#[cfg(target_arch = "mips")]
include!("mips.rs");
