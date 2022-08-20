//! This module implements the free-list allocator.

use core::alloc::GlobalAlloc;

use super::{align_up, Locked};

/// A node describes a free memory region for allocation.
/// It is stored at the head of that memory region itself, and points
/// to the next free region on the heap.
#[derive(Debug)]
struct ListNode {
  size: usize,
  // Pointer to the next free list node
  // static here since the memory will never be invalidated.
  next: Option<&'static mut ListNode>,
}

impl ListNode {
  /// Create a new node
  const fn new(size: usize) -> Self {
    ListNode { size, next: None }
  }

  /// Start address of the node, also
  /// the start address of the free memory region.
  fn start_addr(&self) -> usize {
    self as *const Self as usize
  }

  /// End address of the node, also
  /// the end address of the free memory region.
  fn end_addr(&self) -> usize {
    self.start_addr() + self.size
  }
}

/// A [LinkedListAllocator] backed by a free-list construct.
pub struct LinkedListAllocator {
  // This is always a sentinel node that the allocator owns in its
  // static area. The real nodes describing heap memory are stored
  // at the same heap memory regions.`
  head: ListNode,
}

impl LinkedListAllocator {
  /// Creates an empty LinkedListAllocator.
  pub const fn new() -> Self {
    Self {
      head: ListNode::new(0),
    }
  }

  /// Initialize the allocator with the given heap bounds.
  ///
  /// # Safety
  /// The caller must guarantee that the heap bounds are valid and that
  /// the heap is unused. This method must be called only once.
  pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
    unsafe {
      self.add_free_region(heap_start, heap_size);
    }
  }

  /// Adds the given memory region to the front of the list.
  unsafe fn add_free_region(&mut self, addr: usize, size: usize) {
    // addr should be aligned by the size of ListNode
    assert_eq!(addr, align_up(addr, core::mem::align_of::<ListNode>()));
    // size should be enough
    assert!(size >= core::mem::size_of::<ListNode>());

    // The value of the new list node, stored on kernel stack
    let mut node = ListNode::new(size);
    node.next = self.head.next.take();

    // this is the start address of the freed memory region.
    let node_ptr = addr as *mut ListNode;
    unsafe {
      // write node to that region
      node_ptr.write(node);
      self.head.next = Some(&mut *node_ptr);
    }
  }

  /// Finds an unused region along the free list that is able to hold
  /// the given size and alignment of allocation.
  fn find_region(
    &mut self,
    size: usize,
    align: usize,
  ) -> Option<(&'static mut ListNode, usize)> {
    let mut current = &mut self.head;

    while let Some(ref mut next_region) = current.next {
      if let Ok(alloc_start) = Self::alloc_from_region(next_region, size, align)
      {
        // Remove next_region from list and return
        let next_after = next_region.next.take();
        let ret = Some((current.next.take().unwrap(), alloc_start));
        current.next = next_after;
        return ret;
      }
      current = current.next.as_mut().unwrap();
    }

    // no allocation found
    None
  }

  /// Try to use the given region for an allocation with given size and
  /// alignment.
  ///
  /// Returns the allocation start address on success.
  fn alloc_from_region(
    region: &ListNode,
    size: usize,
    align: usize,
  ) -> Result<usize, ()> {
    // start address, if any, in this region
    let alloc_start = align_up(region.start_addr(), align);
    let alloc_end = alloc_start.checked_add(size).ok_or(())?;

    if alloc_end > region.end_addr() {
      // region too small
      return Err(());
    }

    let excess_size = region.end_addr() - alloc_end;
    if excess_size > 0 && excess_size < core::mem::size_of::<ListNode>() {
      // rest of region too small to hold a ListNode (required because the
      // allocation splits the region in a used and a free part)
      return Err(());
    }

    // region suitable for allocation
    Ok(alloc_start)
  }

  /// Adjust the given layout so that the resulting allocated memory
  /// region is also capable of storing a `ListNode`.
  ///
  /// This is necessary because an arbitrary allocation might:
  /// 1. have too small size
  /// 2. have inconsistent alignment
  ///
  /// Returns the adjusted size and alignment as a (size, align) tuple.
  fn size_align(layout: core::alloc::Layout) -> (usize, usize) {
    // make sure the layout aligns to a ListNode,
    // it might increase the alignment, which is fine because
    // bigger alignments are always compatible with lower ones
    let layout = layout
      .align_to(core::mem::align_of::<ListNode>())
      .expect("adjusting alignment failed")
      .pad_to_align(); // this ensures that the size will always be a multiple of size(LinkedListNode)
    let size = layout.size().max(core::mem::size_of::<ListNode>());
    (size, layout.align())
  }
}

unsafe impl GlobalAlloc for Locked<LinkedListAllocator> {
  unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
    // perform layout adjustments
    let (size, align) = LinkedListAllocator::size_align(layout);
    let mut list = self.lock();

    if let Some((region, alloc_start)) = list.find_region(size, align) {
      let alloc_end = alloc_start.checked_add(size).expect("overflow");
      let excess_size = region.end_addr() - alloc_end;
      if excess_size > 0 {
        unsafe {
          // add the rest of memory in the region back as free region
          list.add_free_region(alloc_end, excess_size);
        }
      }
      alloc_start as *mut u8
    } else {
      core::ptr::null_mut()
    }
  }

  unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
    let (size, _) = LinkedListAllocator::size_align(layout);
    let addr = ptr as usize;
    let mut list = self.lock();

    unsafe {
      list.add_free_region(addr, size);
    }
  }
}
