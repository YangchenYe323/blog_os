//! This module contains the kernels multi-tasking functionalities,
//! which includes an executor to run asynchronous tasks in the kernel.

pub mod executor;
pub mod keyboard;
pub mod simple_executor;

use alloc::boxed::Box;
use core::{
  future::Future,
  pin::Pin,
  sync::atomic::{AtomicU64, Ordering},
  task::{Context, Poll},
};

/// Unique identifier of tasks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct TaskId(u64);

impl TaskId {
  /// Generate an auto-incrementing task id.
  fn new() -> Self {
    static NEXT_ID: AtomicU64 = AtomicU64::new(0);
    TaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
  }
}

/// The kernel's [Task] type, which wraps around
/// rust's native future type with unit outputs type.
pub struct Task {
  task_id: TaskId,
  inner_future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
  /// Create new instances from given future.
  pub fn new(future: impl Future<Output = ()> + 'static) -> Self {
    Self {
      task_id: TaskId::new(),
      inner_future: Box::pin(future),
    }
  }

  /// Wrapper poll function.
  fn poll(&mut self, ctx: &mut Context) -> Poll<()> {
    // Pin<Box<Future>> -> Pin<&mut Future> -> Poll<()>
    self.inner_future.as_mut().poll(ctx)
  }
}
