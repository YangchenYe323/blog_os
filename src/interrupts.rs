//! This module contains CPU's interrupt handling functionalities.
//! It defines handlers for different kinds of interrupts and manipulates
//! the global interrupt descripter table
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use crate::println;

lazy_static! {
  static ref IDT: InterruptDescriptorTable = {
    let mut idt = InterruptDescriptorTable::new();
    idt.breakpoint.set_handler_fn(breakpoint_handler);
    unsafe {
      idt.double_fault.set_handler_fn(double_fault_handler).set_stack_index(crate::gdt::DOUBLE_FAULT_IST_INDEX);
    }
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

/// Double fault is triggered when a CPU exception occurs but the cpu failed to invoke
/// the corresponding handler.
/// We catch double fault to avoid the fatal triple fault which causes the system to reset.
extern "x86-interrupt" fn double_fault_handler(frame: InterruptStackFrame, _err_code: u64) -> ! {
  panic!("EXCEPTION: DOUBLE_FAULT\n{:#?}", frame);
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
