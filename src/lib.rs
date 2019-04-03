#![cfg_attr(not(test), no_std)]
#![feature(alloc)]
#![feature(linkage)]
#![feature(asm)]
#![feature(naked_functions)]

extern crate alloc;

mod interrupt;
mod processor;
pub mod scheduler;
pub mod std_thread;
mod thread_pool;
mod timer;

#[cfg(target_arch = "x86_64")]
#[path = "./context/x86_64.rs"]
pub mod context;

#[cfg(target_arch = "aarch64")]
#[path = "./context/aarch64.rs"]
pub mod context;

#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
#[path = "./context/riscv.rs"]
pub mod context;

pub use crate::processor::Processor;
pub use crate::thread_pool::*;
