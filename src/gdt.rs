//! This module contains the global descriptor table of the kernel
//! and some other structures.

use x86_64::structures::gdt::{
  Descriptor, GlobalDescriptorTable, SegmentSelector,
};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

/// The stack table index for the stack used for double fault
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
  // Global descriptor table that contains information needed for kernel and CPU
  static ref GDT: (GlobalDescriptorTable, Selectors) = {
    let mut gdt = GlobalDescriptorTable::new();
    let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
    let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
    (gdt, Selectors {code_selector, tss_selector})
  };

  // Task State Segment Descriptor that contains the interrupt stack table
  static ref TSS: TaskStateSegment = {
    let mut tss = TaskStateSegment::new();
    tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
      const STACK_SIZE: usize = 4096 * 5;
      // we don't have memory allocator yet, so this is statically allocated
      static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
      let stack_start = VirtAddr::from_ptr(unsafe {&STACK});

      stack_start + STACK_SIZE
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
  use x86_64::instructions::segmentation::{Segment, CS};
  use x86_64::instructions::tables::load_tss;

  GDT.0.load();

  unsafe {
    // update code segment register, which points to an entry in GDT in protected mode
    CS::set_reg(GDT.1.code_selector);
    // update tss segment register, which points to an entry in GDT in protected mod
    load_tss(GDT.1.tss_selector);
  }
}
