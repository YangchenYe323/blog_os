//! This module contains the kernel's heap memory allocators.

pub mod bump;
pub mod fixed_size_block;
pub mod linked_list;

use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use x86_64::{
  structures::paging::{
    mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
  },
  VirtAddr,
};

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

#[allow(dead_code)]
static DUMMY_ALLOCATOR: Dummy = Dummy;

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
  panic!("allocation error: {:?}", layout)
}

/// Start address of heap virtual memory
pub const HEAP_START: usize = 0x_4444_4444_0000;
/// Heap size
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

/// Initialize kernel's heap memory area by mapping all pages
/// in kernel's [HEAP_START, HEAP_START + HEAP_SIZE] range to
/// physical frames.
pub fn init_heap(
  mapper: &mut impl Mapper<Size4KiB>,
  frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
  let page_range = {
    let heap_start = VirtAddr::new(HEAP_START as u64);
    let heap_end = heap_start + HEAP_SIZE - 1u64;
    // Page::containing_address will do the 4KiB alignment for us
    let heap_start_page = Page::containing_address(heap_start);
    let heap_end_page = Page::containing_address(heap_end);
    Page::range_inclusive(heap_start_page, heap_end_page)
  };

  for page in page_range {
    let frame = frame_allocator
      .allocate_frame()
      .ok_or(MapToError::FrameAllocationFailed)?;
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
    unsafe { mapper.map_to(page, frame, flags, frame_allocator)?.flush() };
  }

  // give the initialized memory to allocator
  unsafe {
    ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
  }

  Ok(())
}

/// ALERT: don't use allocation inside an interrupt handler, as that might
/// cause deadlock for concurrent access to ALLOCATOR
#[cfg(not(any(feature = "bump", feature = "fixed")))]
#[global_allocator]
static ALLOCATOR: Locked<linked_list::LinkedListAllocator> =
  Locked::new(linked_list::LinkedListAllocator::new());

#[cfg(feature = "bump")]
#[global_allocator]
static ALLOCATOR: Locked<bump::BumpAllocator> =
  Locked::new(bump::BumpAllocator::new());

#[cfg(feature = "fixed")]
#[global_allocator]
static ALLOCATOR: Locked<fixed_size_block::FixedSizeBlockAllocator> =
  Locked::new(fixed_size_block::FixedSizeBlockAllocator::new());

/// A wrapper around [spin::Mutex] to permit trait implementations.
pub struct Locked<A> {
  inner: spin::Mutex<A>,
}

impl<A> Locked<A> {
  /// Create a new instance
  pub const fn new(inner: A) -> Self {
    Locked {
      inner: spin::Mutex::new(inner),
    }
  }

  /// Lock to get mutable reference of the inner
  pub fn lock(&self) -> spin::MutexGuard<A> {
    self.inner.lock()
  }
}

/// Align the given address `addr` upwards to alignment `align`.
///
/// Requires that `align` is a power of two.
pub(crate) fn align_up(addr: usize, align: usize) -> usize {
  (addr + align - 1) & !(align - 1)
}
