use super::ExceptionStackFrame;
use crate::{hlt_loop, println};

/// Devide By Zero handler
pub extern "C" fn divide_by_zero_handler(frame: &ExceptionStackFrame) -> ! {
  println!("EXCEPTION: DIVIDE BY ZERO");
  println!("{:#?}", frame);

  hlt_loop();
}

/// Break point handler
pub extern "C" fn breakpoint_handler(frame: &ExceptionStackFrame) {
  println!("EXCEPTION: BREAKPOINT");
  println!("{:#?}", frame);
}

/// Invalid OpCode handler
pub extern "C" fn invalid_opcode_handler(frame: &ExceptionStackFrame) -> ! {
  println!("EXCEPTION: INVALID OPCODE");
  println!("{:#x}\n{:#?}", frame.instruction_pointer, frame);

  hlt_loop();
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

  hlt_loop();
}

pub extern "C" fn double_fault_handler(
  frame: &ExceptionStackFrame,
  _err_code: u64,
) -> ! {
  panic!("EXCEPTION: DOUBLE_FAULT\n{:#?}", frame);
}

/// This macro wraps a fn(&ExceptionFrame) in the naked function that
/// handles argument passing and raw stack manipulations, producing an fn() -> !
/// to use in the Interrupt Descripter Table [super::idt::Idt]
#[macro_export]
macro_rules! handler {
  ($name: ident) => {{
    #[naked]
    extern "C" fn wrapper() -> ! {
      unsafe {
        core::arch::asm!("
          // safe all registers
          push rax;
          push rcx;
          push rdx;
          push rsi;
          push rdi;
          push r8;
          push r9;
          push r10;
          push r11;

          // calculate the address of the stack frame
          mov rdi, rsp;
          add rdi, 9*8;

          // call handler functions
          call {};

          // restore all registers
          pop r11;
          pop r10;
          pop r9;
          pop r8;
          pop rdi;
          pop rsi;
          pop rdx;
          pop rcx;
          pop rax;

          // return from exception handler
          iretq", sym $name, options(noreturn));
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
          core::arch::asm!("
            // save all registers
            push rax;
            push rcx;
            push rdx;
            push rsi;
            push rdi;
            push r8;
            push r9;
            push r10;
            push r11;

            // rsi should store the error code,
            // which locates at memory address (rsp + 9 * 8)
            mov rsi, rsp;
            add rsi, 9 * 8;
            mov rsi, [rsi];

            // rdi stores the address of the stack frame,
            // which is rsp + 10 * 8
            mov rdi, rsp;
            add rdi, 10 * 8;

            // align stack pointer
            sub rsp, 8;

            call {};

            // undo align
            add rsp, 8;

            // restore all registers
            pop r11;
            pop r10;
            pop r9;
            pop r8;
            pop rdi;
            pop rsi;
            pop rdx;
            pop rcx;
            pop rax;

            // pop error code
            add rsp, 8;

            iretq",
            sym $name,
            options(noreturn));
        }
      }
      wrapper
    }};
}
