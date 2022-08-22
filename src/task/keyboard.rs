//! This modules contains the kernel's keyboard task:
//! that task scans the global scan-code queue and process the
//! scan-code. The interrupt handler for keyboard interrupt pushes
//! to the queue.

use crate::print;
use crate::println;
use conquer_once::spin::OnceCell;
use core::{
  pin::Pin,
  task::{Context, Poll},
};
use crossbeam_queue::ArrayQueue;
use futures_util::task::AtomicWaker;
use futures_util::{stream::Stream, StreamExt};
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};

static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();

/// The waker used to wake up the [ScancodeStream] task.
static WAKER: AtomicWaker = AtomicWaker::new();

/// Called by the keyboard interrupt handler
///
/// Must not block or allocate.
pub(crate) fn add_scancode(scancode: u8) {
  if let Ok(queue) = SCANCODE_QUEUE.try_get() {
    if queue.push(scancode).is_err() {
      println!("WARNING: scancode queue full; dropping keyboard input");
    } else {
      // wake up whatever task that's waiting on us
      WAKER.wake();
    }
  } else {
    println!("WARNING: scancode queue uninitialized");
  }
}

/// The [ScancodeStream] type by which we interact with the scancode
/// asynchronously. This struct should only be initialized once for the whole
/// lifetime of the kernel.
pub struct ScancodeStream {
  // prevent construction of struct literal outside of the module
  _private: (),
}

impl ScancodeStream {
  /// Create a new instance.
  ///
  /// This function should be alled only once and will panic if called multiple times.
  #[allow(clippy::new_without_default)]
  pub fn new() -> Self {
    SCANCODE_QUEUE
      .try_init_once(|| ArrayQueue::new(100))
      .expect("ScancodeStream::new should only be called once");
    ScancodeStream { _private: () }
  }
}

impl Stream for ScancodeStream {
  type Item = u8;

  fn poll_next(
    self: Pin<&mut Self>,
    cx: &mut Context<'_>,
  ) -> Poll<Option<Self::Item>> {
    let queue = SCANCODE_QUEUE.try_get().unwrap();

    // fast path
    if let Ok(code) = queue.pop() {
      return Poll::Ready(Some(code));
    }

    WAKER.register(cx.waker());

    match queue.pop() {
      Ok(code) => {
        WAKER.take();
        Poll::Ready(Some(code))
      }
      Err(_) => Poll::Pending,
    }
  }
}

/// This creates a long-running task that monitors the [SCANCODE_QUEUE]
/// and prints to the screen if a valid character is available.
///
/// This function should be called only once as it initializes the global stream
/// inside.
pub async fn print_keypress() {
  let mut scancode_stream = ScancodeStream::new();
  // keyboard state machine to handle keyboard events
  let mut keyboard =
    Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore);

  while let Some(code) = scancode_stream.next().await {
    if let Ok(Some(event)) = keyboard.add_byte(code) {
      if let Some(key) = keyboard.process_keyevent(event) {
        match key {
          DecodedKey::Unicode(character) => print!("{}", character),
          DecodedKey::RawKey(key) => print!("{:?}", key),
        }
      }
    }
  }
}
