//! This module implements a simple executor to execute
//! asynchronous tasks.

use super::Task;
use alloc::collections::VecDeque;
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

/// The [SimpleExecutor] type. It manages all
/// pending tasks in a [VecDeque], and runs by continually
/// polling each task in the deque in a round-robin manner.
/// The simple executor does not use the [Waker][core::task::Waker]
/// facility of Rust and will always occupy full CPU to do the
/// polling.
pub struct SimpleExecutor {
  /// Queue of pending tasks
  task_queue: VecDeque<Task>,
}

impl SimpleExecutor {
  /// Create a new executor with no pending tasks.
  pub fn new() -> Self {
    Self {
      task_queue: VecDeque::new(),
    }
  }

  /// Spawn a new task, i.e., put it to the managed task space
  /// of the executor and let executor handle polling it.
  pub fn spawn(&mut self, task: Task) {
    self.task_queue.push_back(task)
  }

  /// Run the executor by continually polling all the tasks in the
  /// task_queue.
  pub fn run(&mut self) {
    while let Some(mut next_task) = self.task_queue.pop_front() {
      let waker = dummy_waker();
      let mut ctx = Context::from_waker(&waker);
      match next_task.poll(&mut ctx) {
        Poll::Ready(_) => {}
        Poll::Pending => {
          self.task_queue.push_back(next_task);
        }
      }
    }
  }
}

impl Default for SimpleExecutor {
  fn default() -> Self {
    Self::new()
  }
}

/// Create a dummy raw waker.
fn dummy_raw_waker() -> RawWaker {
  fn no_op(_: *const ()) {}
  fn clone(_: *const ()) -> RawWaker {
    dummy_raw_waker()
  }

  let vtable = &RawWakerVTable::new(clone, no_op, no_op, no_op);
  RawWaker::new(core::ptr::null(), vtable)
}

/// Create a dummy Waker by a raw waker.
fn dummy_waker() -> Waker {
  unsafe { Waker::from_raw(dummy_raw_waker()) }
}
