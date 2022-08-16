//! Tess the kernel handles stack overflow exception correctly
//! by switching to a new kernel stack for double fault.

#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use blog_os::{
  serial_print, serial_println, test_harness::exit_qemu,
  test_harness::QemuExitCode,
};
use core::panic::PanicInfo;
use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

lazy_static! {
  static ref TEST_IDT: InterruptDescriptorTable = {
    let mut idt = InterruptDescriptorTable::new();
    unsafe {
      idt
        .double_fault
        .set_handler_fn(test_double_fault_handler)
        .set_stack_index(blog_os::gdt::DOUBLE_FAULT_IST_INDEX);
    }
    idt
  };
}

extern "x86-interrupt" fn test_double_fault_handler(
  _frame: InterruptStackFrame,
  _err_code: u64,
) -> ! {
  serial_println!("[ok]");
  exit_qemu(QemuExitCode::Success);
}

fn init_test_idt() {
  TEST_IDT.load();
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
  serial_print!("stack_overflow::stack_overflow...\t");

  blog_os::gdt::init_gdt();
  init_test_idt();

  stack_overflow();
  panic!("Execution continued after stack overflow");
}

#[allow(unconditional_recursion)]
fn stack_overflow() {
  stack_overflow();
  volatile::Volatile::new(0).read(); // prevent tail recursion optimizations
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
  blog_os::test_harness::test_panic_handler(info)
}
