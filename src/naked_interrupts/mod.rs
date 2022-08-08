//! This modules provides alternative implementation of the
//! interrupt handling functionalities without relying on the
//! [x86_84] crate. That is, it builds its own IDT type and handles
//! the calling conventions manually.

mod frame;
mod handlers;
mod idt;

pub(crate) use frame::ExceptionStackFrame;

use crate::{handler, handler_with_err_code};
use handlers::{
  breakpoint_handler, divide_by_zero_handler, invalid_opcode_handler,
  page_fault_handler, double_fault_handler
};

lazy_static! {
  /// Global IDT
  pub static ref IDT: idt::Idt = {
    let mut idt = idt::Idt::new();
    idt.set_handler(0, handler!(divide_by_zero_handler));
    idt.set_handler(3, handler!(breakpoint_handler));
    idt.set_handler(6, handler!(invalid_opcode_handler));
    idt.set_handler(14, handler_with_err_code!(page_fault_handler));
    unsafe {
      let opt = idt.set_handler(8, handler_with_err_code!(double_fault_handler));
      let mut new_opt = opt.read_unaligned();
      // the index is the array index, and should be added 1 for the option
      new_opt.set_stack_index(crate::gdt::DOUBLE_FAULT_IST_INDEX + 1);
      opt.write_unaligned(new_opt);
    }
    idt
  };
}

/// Initialize IDT
pub fn init_idt() {
  IDT.load();
}
