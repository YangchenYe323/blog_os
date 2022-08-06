use super::ExceptionStackFrame;
use crate::println;
use core::arch::asm;

/// Devide By Zero handler
pub extern "C" fn divide_by_zero_handler(frame: &ExceptionStackFrame) -> ! {
  println!("EXCEPTION: DIVIDE BY ZERO");
  println!("{:#?}", frame);
  loop {}
}

/// Break point handler
pub extern "C" fn breakpoint_handler(frame: &ExceptionStackFrame) -> ! {
  println!("EXCEPTION: BREAKPOINT");
  println!("{:#?}", frame);
  loop {}
}

/// Invalid OpCode handler
pub extern "C" fn invalid_opcode_handler(frame: &ExceptionStackFrame) -> ! {
  println!("EXCEPTION: INVALID OPCODE");
  println!("{:#x}\n{:#?}", frame.instruction_pointer, frame);
  loop {}
}

/// Page fault handler
pub extern "C" fn page_fault_handler(
  frame: &ExceptionStackFrame,
  err_code: u64,
) -> ! {
  println!(
    "EXCEPTION: PAGE FAULT with error code {:?}\n{:#?}",
    err_code, frame
  );
  loop {}
}

#[naked]
#[allow(dead_code)]
pub extern "C" fn breakpoint_handler_wrapper() -> ! {
  unsafe {
    // rdi is the place for the first argument in C calling convention,
    // and its value should be the address of the ExceptionStackFrame, which is in rsp.
    // We subract rsp by 8 to recover the 16-byte alignment of the rsp and rbp (The
    // ExceptionStackFrame contains 5 u64, which makes for a total 40 bytes, which breaks the
    // 16-byte alignment requirement)
    asm!("mov rdi, rsp;
              sub rsp, 8;
              call {}", 
              sym breakpoint_handler, 
              options(noreturn));
  }
}

/// This macro wraps a fn(&ExceptionFrame) -> ! in the naked function that
/// handles argument passing and raw stack manipulations, producing an fn() -> !
/// to use in the Interrupt Descripter Table [super::idt::Idt]
#[macro_export]
macro_rules! handler {
  ($name: ident) => {{
      #[naked]
      extern "C" fn wrapper() -> ! {
          unsafe {
              core::arch::asm!("mov rdi, rsp
                    sub rsp, 8 // align the stack pointer
                    call {}", sym $name, options(noreturn));
          }
      }
      wrapper
  }}
}

/// This macro wraps a fn(&ExceptionFrame, u64) -> ! in the naked function, which
/// handles CPU exceptions with an error code.
#[macro_export]
macro_rules! handler_with_err_code {
    ($name: ident) => {{
      #[naked]
      extern "C" fn wrapper() -> ! {
        unsafe {
          // pop error code into rsi, as error code is the lowest item
          // on the stack
          core::arch::asm!(
            "pop rsi;
             mov rdi, rsp;
             sub rsp, 8;
             call {}",
            sym $name,
            options(noreturn));
        }
      }
      wrapper
    }};
}
