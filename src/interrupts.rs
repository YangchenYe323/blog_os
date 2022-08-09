//! This module contains CPU's interrupt handling functionalities.
//! It defines handlers for different kinds of interrupts and manipulates
//! the global interrupt descripter table
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use crate::{println, print};
use pic8259::ChainedPics;
use spin;

/// the nth interrupt in primary pic is mapped to PIC_1_OFFSET + n
/// this effectively maps the 0 -> 15 of the interrupts to 32 -> 47,
/// which doesn't conflict with the original entries in IDT, which occupies
/// interrupt descriptor table's indext 0 -> 31
pub const PIC_1_OFFSET: u8 = 32;
/// Offset of the secondary pic slots
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

/// Represents the index in the IDT of all the hardware interrupts.
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
  /// Timer interrupt
  Timer = PIC_1_OFFSET,
}

impl InterruptIndex {
  /// Convert enum to u8 value
  fn as_u8(self) -> u8 {
    self as u8
  }

  /// Convert enum to usize value for array indexing
  fn as_usize(self) -> usize {
    usize::from(self.as_u8())
  }
}

lazy_static! {
  static ref IDT: InterruptDescriptorTable = {
    let mut idt = InterruptDescriptorTable::new();
    idt.breakpoint.set_handler_fn(breakpoint_handler);
    unsafe {
      idt.double_fault.set_handler_fn(double_fault_handler).set_stack_index(crate::gdt::DOUBLE_FAULT_IST_INDEX);
    }

    // set up timer interrupt handler
    idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);

    idt
  };

  /// Global programmable interrupt controller handle
  pub static ref PICS: spin::Mutex<ChainedPics> = spin::Mutex::new(unsafe {
    ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET)
  });
}

/// Initialize interupt descripter table
pub fn init_idt() {
  IDT.load();
}

/// BreakPoint exception is raised when CPU executes the `int3` instructions,
/// it is commonly used by debuggers for setting up break points in the program.
extern "x86-interrupt" fn breakpoint_handler(frame: InterruptStackFrame) {
  println!("EXCEPTION: BREAKPOINT\n{:#?}", frame);
}

/// Double fault is triggered when a CPU exception occurs but the cpu failed to invoke
/// the corresponding handler.
/// We catch double fault to avoid the fatal triple fault which causes the system to reset.
extern "x86-interrupt" fn double_fault_handler(frame: InterruptStackFrame, _err_code: u64) -> ! {
  panic!("EXCEPTION: DOUBLE_FAULT\n{:#?}", frame);
}

/// Handles timer interrupt.
extern "x86-interrupt" fn timer_interrupt_handler(_frame: InterruptStackFrame) {
  print!(".");
  
  // PIC expects to receive an "end-of-interrupt" signal so that it will send the next
  // interrupt. Sending this signal to notify PIC that we're done processing the current interrupt
  unsafe {
    PICS.lock()
      .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
  }
}

#[cfg(test)]
mod tests {
  #[test_case]
  fn test_breakpoint_exception() {
    // invoke a breakpoint exception
    // should not crash
    x86_64::instructions::interrupts::int3();
  }
}
