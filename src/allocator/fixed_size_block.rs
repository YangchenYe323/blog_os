//! This module contains an implementation of
//! fixed-size allocator.

use core::{alloc::GlobalAlloc, ptr::NonNull};

use super::Locked;

/// The [ListNode] type describing a free
/// memory area. No size field is needed as the memory
/// blocks are fixed and known at all times
struct ListNode {
  // points to the next node
  next: Option<&'static mut ListNode>,
}

/// The block sizes to use.
///
/// The sizes must each be power of 2 because they are also used as
/// the block alignment (alignments must be always powers of 2).
const BLOCK_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048];

/// Choose an appropriate block size for the given layout.
///
/// Returns an index into the `BLOCK_SIZES` array.
fn list_index(layout: &core::alloc::Layout) -> Option<usize> {
  // Since our BLOCK_SIZE is also BLOCK_ALIGNMENT, the requested block's size
  // must not be smaller than its alignment to prevent situations like:
  // Layout {size: 4, alignment: 32}, which might ge allocated to the first slot.
  let required_block_size = layout.size().max(layout.align());
  BLOCK_SIZES.iter().position(|&s| s >= required_block_size)
}

/// The Fixed Size Allocator type. It maintains an array of linked-lists,
/// each pointing to free memory regions of a specific size defined by
/// [BLOCK_SIZES].
///
/// For larger allocations, it uses a fall-back allocator.
pub struct FixedSizeBlockAllocator {
  /// Array of linked list heads
  list_heads: [Option<&'static mut ListNode>; BLOCK_SIZES.len()],
  /// Fall back allocator
  fallback_allocator: linked_list_allocator::Heap,
}

impl FixedSizeBlockAllocator {
  /// Create a new instance with empty lists.
  pub const fn new() -> Self {
    const EMPTY: Option<&'static mut ListNode> = None;
    Self {
      list_heads: [EMPTY; BLOCK_SIZES.len()],
      fallback_allocator: linked_list_allocator::Heap::empty(),
    }
  }

  /// Initialize allocator with heap memory region.
  ///
  /// # Safety
  /// The caller must guarantee that heap_start + heap_size describes a valid
  /// heap memory area, and are unused. This function should be called only once.
  pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
    unsafe {
      self.fallback_allocator.init(heap_start, heap_size);
    }
  }

  /// Allocate a memory region of given layout using the fall-back
  /// allocator.
  fn fallback_alloc(&mut self, layout: core::alloc::Layout) -> *mut u8 {
    match self.fallback_allocator.allocate_first_fit(layout) {
      Ok(region) => region.as_ptr(),
      Err(_) => core::ptr::null_mut(),
    }
  }
}

unsafe impl GlobalAlloc for Locked<FixedSizeBlockAllocator> {
  unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
    let mut allocator = self.lock();

    match list_index(&layout) {
      Some(idx) => {
        match allocator.list_heads[idx].take() {
          Some(head) => {
            // Allocate from fixed-size list
            allocator.list_heads[idx] = head.next.take();
            head as *mut ListNode as *mut u8
          }

          None => {
            // Allocate from fall-back allocator
            let block_size = BLOCK_SIZES[idx];
            let block_align = block_size;
            let layout =
              core::alloc::Layout::from_size_align(block_size, block_align)
                .unwrap();
            allocator.fallback_alloc(layout)
          }
        }
      }

      // Block is too big, use fall back allocation
      None => allocator.fallback_alloc(layout),
    }
  }

  unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
    let mut allocator = self.lock();

    match list_index(&layout) {
      Some(idx) => {
        let old_head = allocator.list_heads[idx].take();
        let new_head: ListNode = ListNode { next: old_head };
        let addr = ptr as *mut ListNode;

        // verify that block has size and alignment required for storing node
        assert!(core::mem::size_of::<ListNode>() <= BLOCK_SIZES[idx]);
        assert!(core::mem::align_of::<ListNode>() <= BLOCK_SIZES[idx]);

        unsafe { addr.write(new_head) }
        allocator.list_heads[idx] = Some(unsafe { &mut *addr })
      }

      // This memory is allocated directly from fall-back allocator,
      // so just return back
      None => {
        let ptr: NonNull<u8> = NonNull::new(ptr).unwrap();
        unsafe {
          allocator.fallback_allocator.deallocate(ptr, layout);
        }
      }
    }
  }
}
