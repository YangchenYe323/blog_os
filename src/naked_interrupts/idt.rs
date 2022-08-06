//! This modules defines IDT(Interrupt descripter tables)
//! type that works with the x86_86 architecture.

use x86_64::instructions::segmentation;
use x86_64::registers::segmentation::Segment;
use x86_64::structures::gdt::SegmentSelector;
use x86_64::PrivilegeLevel;

/// IDT is just an array of IDE entries. Technically
/// it can contain up to 256 entries, but we're only using
/// the first 16, which are:
/// 0 -> Divide by zero
/// 1 -> Debug
/// 2 -> Non-maskable interrupt
/// 3 -> Breakpoint
/// 4 -> Overflow
/// 5 -> Bound range exceeded
/// 6 -> Invalid Opcode
/// 7 -> Device not available
/// 8 -> Double fault
/// 9 -> Coprocessor segment overrun
/// 10 -> Invalid TSS
/// 11 -> Segment not present
/// 12 -> Stack-segment fault
/// 13 -> General protection fault
/// 14 -> Page fault
/// 15 -> Reserved
/// See https://wiki.osdev.org/Exceptions for detailed reference
pub struct Idt([Entry; 16]);

impl Idt {
  /// Create a default IDT with all missing entries
  pub fn new() -> Self {
    Self([Entry::missing(); 16])
  }

  /// Set handler function to the nth entry,
  /// returning mutable reference to options for customization
  pub fn set_handler(
    &mut self,
    entry: u8,
    handler: HandlerFunc,
  ) -> *mut EntryOptions {
    self.0[entry as usize] = Entry::new(segmentation::CS::get_reg(), handler);

    // since Entry is unaligned, we return a raw pointer here
    let ptr: *mut EntryOptions =
      core::ptr::addr_of_mut!(self.0[entry as usize].options);
    ptr
  }

  /// Load the current IDT for cpu to use
  /// * `&'static self` we need self to live for the whole lifetime of the program.
  /// Otherwise, cpu might read freed memory where it thinks the IDT resides.
  pub fn load(&'static self) {
    use core::mem::size_of;
    use x86_64::instructions::tables::{lidt, DescriptorTablePointer};

    let ptr = DescriptorTablePointer {
      // address of self
      base: x86_64::VirtAddr::new(self as *const _ as u64),
      // limit is the maximum-addressible byte, which is size - 1.
      limit: (size_of::<Self>() - 1) as u16,
    };

    unsafe { lidt(&ptr) };
  }
}

/// An IDT entry of the following format:
/// Type		Name			Description
/// u16			pointer_low		lower bits of the address of handler function
/// u16			gdt selector	selector of a code segment in the GDT.
/// u16			Options			See [EntryOptions]
/// u16			pointer_mid		middle bits of the address of handler function
///
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct Entry {
  pointer_low: u16,
  gdt_selector: SegmentSelector,
  options: EntryOptions,
  pointer_middle: u16,
  pointer_high: u32,
  reserved: u32,
}

impl Entry {
  /// Create a new IDT entry
  pub fn new(gdt_selector: SegmentSelector, handler: HandlerFunc) -> Self {
    let pointer = handler as u64;
    Entry {
      gdt_selector: gdt_selector,
      pointer_low: pointer as u16,
      pointer_middle: (pointer >> 16) as u16,
      pointer_high: (pointer >> 32) as u32,
      options: EntryOptions::new(),
      reserved: 0,
    }
  }

  /// Create a missing IDT entry
  fn missing() -> Self {
    Entry {
      gdt_selector: SegmentSelector::new(0, PrivilegeLevel::Ring0),
      pointer_low: 0,
      pointer_middle: 0,
      pointer_high: 0,
      options: EntryOptions::minimal(),
      reserved: 0,
    }
  }
}

/// EntryOptions wraps a 16-bit integer with the following structure:
/// Bits		Name							Description
/// 0-2			Interrupt Stack Table Index		0: don't switch stack, 1-7: switch to the nth stack in the table
/// 3-7			Reserved
/// 8			0: Interrupt Gate, 1: Trap Gate	If 0, disable hardware interrupts
/// 9-11		Must be 1
/// 12			Must be 0
/// 13-14		Descriptor Privilage Level		The minimal privilege level required for calling this handler.
/// 15			Present
#[derive(Debug, Clone, Copy)]
pub struct EntryOptions(u16);

const MINIMAL_VALID_OPTION: u16 = 0b0000111000000000;

impl EntryOptions {
  /// Create a new option with all 0s except must-1 bits
  pub fn minimal() -> Self {
    EntryOptions(MINIMAL_VALID_OPTION)
  }

  /// Create a new option with reasonable default.
  /// Present -> True
  /// Disable Interrupts -> True
  pub fn new() -> Self {
    let mut opt = Self::minimal();
    opt.set_present(true).disable_interrupts(true);
    opt
  }

  /// Set the present bit of the option.
  pub fn set_present(&mut self, present: bool) -> &mut Self {
    if present {
      self.0 |= 1 << 15;
    } else {
      self.0 &= !(1 << 15);
    }

    self
  }

  /// Set the interrupt gate bit.
  pub fn disable_interrupts(&mut self, disable: bool) -> &mut Self {
    if disable {
      self.0 &= !(1 << 8);
    } else {
      self.0 |= 1 << 8;
    }

    self
  }

  /// Set privilege level.
  pub fn set_privilege_level(&mut self, dpl: u16) -> &mut Self {
    if dpl >= 4 {
      panic!("Invalid privilege: {}", dpl);
    }
    self.0 = (self.0 & 0x9fff) | (dpl << 13);

    self
  }

  /// Set stack index.
  pub fn set_stack_index(&mut self, index: u16) -> &mut Self {
    if index >= 8 {
      panic!("Invalid stack index {}", index);
    }
    self.0 = (self.0 & 0xfff8) | index;

    self
  }
}

/// Type for interrupt handler functions
/// It doesn't have argument in its signature because the caller
/// doesn't supply argument in normal C calling conventions, so we have
/// to grab the arguments ourselves.
/// Also, the function must never return because the cpu doesn't use normal
/// `call` and `ret` instructions with the handler functions. Rather, they just
/// jump to us and we have to handle return ourselves.
pub type HandlerFunc = extern "C" fn() -> !;

#[cfg(test)]
mod tests {
  use super::*;

  #[test_case]
  fn test_default() {
    let mut opt = EntryOptions::minimal();
    assert_eq!(MINIMAL_VALID_OPTION, opt.0);
  }

  #[test_case]
  fn test_stack_index() {
    let mut opt = EntryOptions::minimal();
    opt.set_stack_index(7);
    assert_eq!(0b0000111000000111, opt.0);
  }

  #[test_case]
  fn test_set_present() {
    let mut opt = EntryOptions::minimal();
    opt.set_stack_index(4).set_present(true);
    assert_eq!(0b1000111000000100, opt.0);
  }

  #[test_case]
  fn test_disable_interrupts() {
    let mut opt = EntryOptions::minimal();
    opt
      .disable_interrupts(false)
      .set_stack_index(3)
      .set_present(true);
    assert_eq!(0b1000111100000011, opt.0);
  }
}
