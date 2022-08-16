//! This module contains the kernel's Virtual Memory Functionalities.

use x86_64::structures::paging::{
  FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PhysFrame, Size4KiB,
};
use x86_64::{PhysAddr, VirtAddr};
use bootloader::bootinfo::{MemoryMap, MemoryRegionType};

/// Initialize a new [OffsetPageTable].
/// It assumes that the entire physical memory is mapped to offset given by
/// `phycial_memory_offset`
pub unsafe fn init_offset_page_table(
  physical_memory_offset: VirtAddr,
) -> OffsetPageTable<'static> {
  unsafe {
    let level_4_table = active_level4_page_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
  }
}

/// Returns a handle to the Level-4 PageTable of the current process.
///
/// This function is unsafe because the caller has to guarantee that
/// the provided offset is valid, i.e., the complete physical memory is
/// mapped to virtual memory at the passed offset Also, this function must
/// be only called once to avoid aliasing `&mut` references
/// (which is undefined behavior).
///
/// * `physical_memory_offset` the offset by which physical memory is mapped
pub unsafe fn active_level4_page_table(
  physical_memory_offset: VirtAddr,
) -> &'static mut PageTable {
  use x86_64::registers::control::Cr3;

  // the physical frame of the level4 page table
  let (level_4_table_frame, _) = Cr3::read();

  // the physical address of the level4 page table
  let phys = level_4_table_frame.start_address();
  // the virtual address of the level4 page table
  let virt = physical_memory_offset + phys.as_u64();
  let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

  unsafe { &mut *page_table_ptr }
}

// x86_64 virtual address format:
// [63 - 48]        [47 - 39]    [38 - 30]    [29 - 21]    [20 - 12] .  [11 - 0]
// [sign extention][page4 index][page3 index][page2 index][page1 index][offset in page]
const IDX_MASK: u64 = 0b11111111;
const OFFSET_MASK: u64 = 0xfff;

fn level4_page_table_index(addr: VirtAddr) -> u64 {
  let addr = addr.as_u64();
  let index = (addr & IDX_MASK << 39) >> 39;
  index
}

fn level3_page_table_index(addr: VirtAddr) -> u64 {
  let addr = addr.as_u64();
  let index = (addr & IDX_MASK << 30) >> 30;
  index
}

fn level2_page_table_index(addr: VirtAddr) -> u64 {
  let addr = addr.as_u64();
  (addr & IDX_MASK << 21) >> 21
}

fn level1_page_table_index(addr: VirtAddr) -> u64 {
  let addr = addr.as_u64();
  (addr & IDX_MASK << 12) >> 12
}

fn offset_in_page(addr: VirtAddr) -> u64 {
  let addr = addr.as_u64();
  addr & OFFSET_MASK
}

/// Translate a given [VirtAddr] to the mapped [PhysAddr] by the process's page table.
///
/// * 'addr' Virtual Address to translate
/// * 'physical_memory_offset' Physical memory offset
pub unsafe fn translate_virt_address(
  addr: VirtAddr,
  physical_memory_offset: VirtAddr,
) -> Option<PhysAddr> {
  use x86_64::registers::control::Cr3;
  use x86_64::structures::paging::page_table::FrameError;

  let (level4_table_frame, _) = Cr3::read();
  let indexes = [
    level4_page_table_index(addr),
    level3_page_table_index(addr),
    level2_page_table_index(addr),
    level1_page_table_index(addr),
  ];

  let mut current_frame = level4_table_frame;

  // traverse the indexes
  for &index in &indexes {
    let virt = physical_memory_offset + current_frame.start_address().as_u64();
    let table_ptr: *const PageTable = virt.as_ptr();
    let table = unsafe { &*table_ptr };

    // read the index in the table
    let entry = &table[index as usize];
    current_frame = match entry.frame() {
      Ok(frame) => frame,
      Err(FrameError::FrameNotPresent) => return None,
      Err(FrameError::HugeFrame) => panic!("huge pages not supported"),
    }
  }

  // Calculate exact address with offset in page
  Some(current_frame.start_address() + offset_in_page(addr))
}

/// A FrameAllocator that returns usable frames from the bootloader's memory map.
pub struct BootInfoFrameAllocator {
  memory_map: &'static MemoryMap,
  next: usize,
}

impl BootInfoFrameAllocator {
  /// Create a FrameAllocator from the passed memory map.
  ///
  /// This function is unsafe because the caller must guarantee that the passed
  /// memory map is valid. The main requirement is that all frames that are marked
  /// as `USABLE` in it are really unused.
  pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
      BootInfoFrameAllocator {
          memory_map,
          next: 0,
      }
  }
}

impl BootInfoFrameAllocator {
  /// Returns an iterator over the usable frames specified in the memory map.
  fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
      // get usable regions from memory map
      let regions = self.memory_map.iter();
      let usable_regions = regions
          .filter(|r| r.region_type == MemoryRegionType::Usable);
      // map each region to its address range
      let addr_ranges = usable_regions
          .map(|r| r.range.start_addr()..r.range.end_addr());
      // transform to an iterator of frame start addresses
      // the ranges are already page-aligned, so we're guaranteed to have valid page-start addresses
      let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
      // create `PhysFrame` types from the start addresses
      frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
  }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
  fn allocate_frame(&mut self) -> Option<PhysFrame> {
      let frame = self.usable_frames().nth(self.next);
      self.next += 1;
      frame
  }
}

// Experimental Functions and Structures for concept demonstration

/// A FrameAllocator that always returns `None`.
pub struct EmptyFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
  fn allocate_frame(&mut self) -> Option<PhysFrame> {
    None
  }
}

/// Creates an example mapping for the given page to frame `0xb8000`.
pub fn create_example_mapping(
  page: Page,
  mapper: &mut OffsetPageTable,
  frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
  use x86_64::structures::paging::PageTableFlags as Flags;

  let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
  let flags = Flags::PRESENT | Flags::WRITABLE;

  let map_to_result = unsafe {
    // FIXME: this is not safe, we do it only for testing
    mapper.map_to(page, frame, flags, frame_allocator)
  };
  map_to_result.expect("map_to failed").flush();
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test_case]
  fn test_page_index() {
    let addr = VirtAddr::new(0xdeadbeaf);
    assert_eq!(u64::from(addr.p1_index()), level1_page_table_index(addr));
    assert_eq!(u64::from(addr.p2_index()), level2_page_table_index(addr));
    assert_eq!(u64::from(addr.p3_index()), level3_page_table_index(addr));
    assert_eq!(u64::from(addr.p4_index()), level4_page_table_index(addr));
    assert_eq!(u64::from(addr.page_offset()), offset_in_page(addr));
  }
}
