//! This module contains CPU's interrupt handling functionalities.
//! It defines handlers for different kinds of interrupts and manipulates
//! the global interrupt descripter table
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use crate::println;

lazy_static! {
  static ref IDT: InterruptDescriptorTable = {
    let mut idt = InterruptDescriptorTable::new();
    idt.breakpoint.set_handler_fn(breakpoint_handler);
    idt
  };
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test_case]
  fn test_breakpoint_exception() {
    // invoke a breakpoint exception
    // should not crash
    x86_64::instructions::interrupts::int3();
  }
}
