//! This module contains the global descriptor table of the kernel
//! and some other structures.

use x86_64::VirtAddr;
use x86_64::structures::tss::TaskStateSegment;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};

/// The stack table index for the stack used for double fault
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
  static ref GDT: (GlobalDescriptorTable, Selectors) = {
    let mut gdt = GlobalDescriptorTable::new();
    let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
    let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
    (gdt, Selectors {code_selector, tss_selector})
  };

  static ref TSS: TaskStateSegment = {
    let mut tss = TaskStateSegment::new();
    tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
      const STACK_SIZE: usize = 4096 * 5;
      // we don't have memory allocator yet, so this is statically allocated
      static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
      let stack_start = VirtAddr::from_ptr(unsafe {&STACK});
      let stack_end = stack_start + STACK_SIZE;
      stack_end
    };
    tss
  };
}

struct Selectors {
  code_selector: SegmentSelector,
  tss_selector: SegmentSelector,
}

/// Initialize global descriptor table
pub fn init_gdt() {
  use x86_64::instructions::tables::load_tss;
  use x86_64::instructions::segmentation::{CS, Segment};

  GDT.0.load();

  unsafe {
    CS::set_reg(GDT.1.code_selector);
    load_tss(GDT.1.tss_selector);
  }
}