//! This module contains an implementation of Bump Allocator.

use core::alloc::{GlobalAlloc, Layout};

use super::Locked;

/// The [BumpAllocator] type.
/// It maintains a `next` field that points to the start of unused
/// heap memories. On allocation, it increases the `next` field. Memory
/// is allocated linearly by BumpAllocator, and only reclaimed all at once
/// when `allocations` reaches zero.
#[derive(Debug)]
pub struct BumpAllocator {
  heap_start: usize,
  heap_end: usize,
  next: usize,
  allocations: usize,
}

impl BumpAllocator {
  /// Create a new instance.
  pub const fn new() -> Self {
    BumpAllocator {
      heap_start: 0,
      heap_end: 0,
      next: 0,
      allocations: 0,
    }
  }

  /// Initializes the allocator with heap memory ranges.
  ///
  /// # Safety
  /// The caller must guarantee that `heap_start`..`heap_end` contains
  /// valid virtual addresses.
  pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
    self.heap_start = heap_start;
    self.heap_end = heap_start + heap_size - 1;
    self.next = heap_start;
  }
}

unsafe impl GlobalAlloc for Locked<BumpAllocator> {
  unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
    let mut s = self.lock();

    let addr_start = align_up(s.next, layout.align());
    let addr_end = match addr_start.checked_add(layout.size()) {
      Some(end) => end,
      None => return core::ptr::null_mut(),
    };

    if addr_end > s.heap_end {
      return core::ptr::null_mut();
    }

    s.next = addr_end;
    s.allocations += 1;

    addr_start as *mut u8
  }

  unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
    let mut s = self.lock();

    s.allocations -= 1;
    if s.allocations == 0 {
      s.next = s.heap_start;
    }
  }
}

/// Align the given address `addr` upwards to alignment `align`.
///
/// Requires that `align` is a power of two.
fn align_up(addr: usize, align: usize) -> usize {
  (addr + align - 1) & !(align - 1)
}
