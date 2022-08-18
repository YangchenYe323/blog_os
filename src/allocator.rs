//! This module contains the kernel's heap memory allocators.

use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;

/// A dummy allocator implementation that would return nullptr
/// for every call to alloc
pub struct Dummy;

unsafe impl GlobalAlloc for Dummy {
  unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
    // return nullptr to signal failure
    null_mut()
  }

  unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
    panic!("dealloc should be never called")
  }
}

#[global_allocator]
static DUMMY_ALLOCATOR: Dummy = Dummy;

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
  panic!("allocation error: {:?}", layout)
}
