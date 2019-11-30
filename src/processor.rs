use crate::interrupt;
use crate::thread_pool::*;
use alloc::boxed::Box;
use alloc::sync::Arc;
use core::cell::UnsafeCell;
use log::*;

/// Thread executor
///
/// It's designed to be a per-CPU structure defined at global.
/// You should call `init` first, then call `run` to execute threads infinitely.
///
/// ## WARNING
/// All methods must be called with interrupt disabled,
/// or you must ensure the caller thread will not be switched to other CPUs.
#[derive(Default)]
pub struct Processor {
    inner: UnsafeCell<Option<ProcessorInner>>,
}

unsafe impl Sync for Processor {}

struct ProcessorInner {
    /// Processor ID
    id: usize,
    /// Current running thread
    thread: Option<(Tid, Box<dyn Context>)>,
    /// The context of
    loop_context: Box<dyn Context>,
    /// Reference to `ThreadPool`
    manager: Arc<ThreadPool>,
}

impl Processor {
    pub const fn new() -> Self {
        Processor {
            inner: UnsafeCell::new(None),
        }
    }

    /// Initialize the `Processor`
    pub unsafe fn init(&self, id: usize, context: Box<dyn Context>, manager: Arc<ThreadPool>) {
        *self.inner.get() = Some(ProcessorInner {
            id,
            thread: None,
            loop_context: context,
            manager,
        });
    }

    /// Get the inner data.
    /// This will panic if it has not been initialized.
    fn inner(&self) -> &mut ProcessorInner {
        unsafe { &mut *self.inner.get() }
            .as_mut()
            .expect("Processor is not initialized")
    }

    /// Begin running processes after CPU setup.
    ///
    /// This function never returns. It loops, doing:
    /// - choose a process to run
    /// - switch to start running that process
    /// - eventually that process transfers control
    ///   via switch back to the scheduler.
    pub fn run(&self) -> ! {
        let inner = self.inner();
        loop {
            if let Some(thread) = inner.manager.run(inner.id) {
                trace!("CPU{} begin running thread {}", inner.id, thread.0);
                inner.thread = Some(thread);
                unsafe {
                    inner
                        .loop_context
                        .switch_to(&mut *inner.thread.as_mut().unwrap().1);
                }
                let (tid, context) = inner.thread.take().unwrap();
                trace!("CPU{} stop running thread {}", inner.id, tid);
                inner.manager.stop(tid, context);
            } else {
                trace!("CPU{} idle", inner.id);
                unsafe {
                    interrupt::enable_and_wfi();
                    // wait for a timer interrupt
                    interrupt::disable_and_store();
                }
            }
        }
    }

    /// Called by process running on this Processor.
    /// Yield and reschedule.
    pub(crate) fn yield_now(&self) {
        let inner = self.inner();
        unsafe {
            inner
                .thread
                .as_mut()
                .unwrap()
                .1
                .switch_to(&mut *inner.loop_context);
        }
    }

    /// Get tid of current running thread.
    /// This will panic if this CPU is idle.
    pub fn tid(&self) -> Tid {
        self.inner().thread.as_ref().unwrap().0
    }

    /// Get tid of current running thread if it has.
    pub fn tid_option(&self) -> Option<Tid> {
        unsafe { &*self.inner.get() }
            .as_ref()
            .and_then(|inner| inner.thread.as_ref())
            .map(|t| t.0)
    }

    /// Get a reference to the Context of current running thread.
    pub fn context(&self) -> &dyn Context {
        &*self.inner().thread.as_ref().unwrap().1
    }

    /// Get the `ThreadPool`.
    pub fn manager(&self) -> &ThreadPool {
        &*self.inner().manager
    }

    /// Called by timer interrupt handler.
    ///
    /// The interrupt should be disabled in the handler.
    pub fn tick(&self) {
        // If I'm idle, tid == None, need_reschedule == false.
        // Will go back to `run()` after interrupt return.
        let tid = self.inner().thread.as_ref().map(|p| p.0);
        let need_reschedule = self.manager().tick(self.inner().id, tid);
        if need_reschedule {
            self.yield_now();
        }
    }
}
