#![cfg_attr(not(test), no_std)]
#![feature(linkage)]
#![feature(asm)]
#![feature(naked_functions)]
#![feature(global_asm)]
#![deny(warnings)]

extern crate alloc;

mod context;
mod interrupt;
mod processor;
pub mod scheduler;
pub mod std_thread;
mod thread_pool;
mod timer;

pub use self::processor::Processor;
pub use self::thread_pool::*;
