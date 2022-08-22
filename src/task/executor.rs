//! Implements a better executor type that utilizes the waker
//! support to manage asynchronous tasks more efficiently.

use super::{Task, TaskId};
use alloc::{collections::BTreeMap, sync::Arc, task::Wake};
use core::task::{Context, Waker};
use crossbeam_queue::ArrayQueue;

/// The [Executor] type.
pub struct Executor {
  tasks: BTreeMap<TaskId, Task>,
  task_queue: Arc<ArrayQueue<TaskId>>,
  waker_cache: BTreeMap<TaskId, Waker>,
}

impl Executor {
  /// Create a new instance.
  pub fn new() -> Self {
    Executor {
      tasks: BTreeMap::new(),
      task_queue: Arc::new(ArrayQueue::new(100)),
      waker_cache: BTreeMap::new(),
    }
  }

  /// Spawn a new task.
  pub fn spawn(&mut self, task: Task) {
    let task_id = task.task_id;
    if self.tasks.insert(task_id, task).is_some() {
      panic!("task with same ID already in tasks");
    }
    self.task_queue.push(task_id).expect("queue full");
  }

  /// Run the executor to completion.
  pub fn run(&mut self) -> ! {
    loop {
      self.run_ready_tasks();
      self.sleep_if_idle();
    }
  }

  /// This function scans the [task_queue] once and runs all the possibly ready tasks.
  fn run_ready_tasks(&mut self) {
    let Self {
      tasks,
      task_queue,
      waker_cache,
    } = self;

    while let Ok(task_id) = task_queue.pop() {
      let task = match tasks.get_mut(&task_id) {
        Some(task) => task,
        // This happens if a wake-up happens before a task completes, so that
        // when we execute this line because of the wake-up, the task is already gone.
        // E.g., the [ScancodeStream]
        None => continue,
      };

      let waker = waker_cache
        .entry(task_id)
        .or_insert_with(|| TaskWaker::new(task_id, Arc::clone(task_queue)));
      let mut ctx = Context::from_waker(waker);

      match task.poll(&mut ctx) {
        core::task::Poll::Ready(()) => {
          tasks.remove(&task_id);
          waker_cache.remove(&task_id);
        }
        core::task::Poll::Pending => {}
      }
    }
  }

  /// If the ready queue is empty, we can halt until next
  /// interrupt arrives.
  fn sleep_if_idle(&self) {
    use x86_64::instructions::interrupts::{self, enable_and_hlt};

    interrupts::disable();
    if self.task_queue.is_empty() {
      enable_and_hlt();
    } else {
      interrupts::enable();
    }
  }
}

impl Default for Executor {
  fn default() -> Self {
    Self::new()
  }
}

/// The Waker type used by the executor.
/// It wakes up by pushing the task_id to the task_queue so that
/// it will be ready for polling in the next run. The task_queue
/// is shared with the executor.
struct TaskWaker {
  task_id: TaskId,
  task_queue: Arc<ArrayQueue<TaskId>>,
}

impl TaskWaker {
  #[allow(clippy::new_ret_no_self)]
  fn new(task_id: TaskId, task_queue: Arc<ArrayQueue<TaskId>>) -> Waker {
    Waker::from(Arc::new(TaskWaker {
      task_id,
      task_queue,
    }))
  }

  fn wake_task(&self) {
    self
      .task_queue
      .push(self.task_id)
      .expect("Task Queue is full");
  }
}

impl Wake for TaskWaker {
  fn wake(self: Arc<Self>) {
    self.wake_task();
  }

  fn wake_by_ref(self: &Arc<Self>) {
    self.wake_task();
  }
}
