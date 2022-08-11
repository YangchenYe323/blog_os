//! This module contains CPU's interrupt handling functionalities.
//! It defines handlers for different kinds of interrupts and manipulates
//! the global interrupt descripter table
use x86_64::structures::idt::{
  InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode,
};

use crate::{hlt_loop, print, println};
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
  /// Keyboard interrupt
  Keyboard,
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
    idt.page_fault.set_handler_fn(page_fault_handler);

    // set up timer interrupt handler
    idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
    // set up keyboard interrupt handler
    idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);

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

/// Page fault handler
extern "x86-interrupt" fn page_fault_handler(
  frame: InterruptStackFrame,
  error_code: PageFaultErrorCode,
) {
  use x86_64::registers::control::Cr2;

  println!("EXCEPTION: PAGE FAULT");
  println!("Accessed Address: {:?}", Cr2::read());
  println!("Error Code: {:?}", error_code);
  println!("{:#?}", frame);
  hlt_loop();
}

/// Double fault is triggered when a CPU exception occurs but the cpu failed to invoke
/// the corresponding handler.
/// We catch double fault to avoid the fatal triple fault which causes the system to reset.
extern "x86-interrupt" fn double_fault_handler(
  frame: InterruptStackFrame,
  _err_code: u64,
) -> ! {
  panic!("EXCEPTION: DOUBLE_FAULT\n{:#?}", frame);
}

/// Handles timer interrupt.
extern "x86-interrupt" fn timer_interrupt_handler(_frame: InterruptStackFrame) {
  // nothing todo

  // PIC expects to receive an "end-of-interrupt" signal so that it will send the next
  // interrupt. Sending this signal to notify PIC that we're done processing the current interrupt
  unsafe {
    PICS
      .lock()
      .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
  }
}

/// Keyboard interrupt handler
extern "x86-interrupt" fn keyboard_interrupt_handler(
  _frame: InterruptStackFrame,
) {
  use pc_keyboard::{
    layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1,
  };
  use spin::Mutex;
  use x86_64::instructions::port::Port;

  // the global state machine for processing key events and map event to characters
  lazy_static! {
    static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
      Mutex::new(Keyboard::new(
        layouts::Us104Key,
        ScancodeSet1,
        HandleControl::Ignore
      ));
  }

  let mut keyboard = KEYBOARD.lock();
  // the data port of PS/2 controller, which is our I/O port
  let mut port = Port::new(0x60);
  let scancode: u8 = unsafe { port.read() };
  // record this key-press operation, see if an event is matched
  if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
    // if processing the events generates a character to display
    if let Some(key) = keyboard.process_keyevent(key_event) {
      match key {
        DecodedKey::Unicode(character) => print!("{}", character),
        DecodedKey::RawKey(key) => print!("{:?}", key),
      }
    }
  }

  unsafe {
    PICS
      .lock()
      .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
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
