//! This crate provides library code for blog_os kernel

#![no_std]
#![feature(naked_functions)] // enable naked functions for interrupt handling
#![feature(asm_sym)] // enable sym keyword in rust asm
#![feature(abi_x86_interrupt)] // enable the unstable "x86-interrupt" calling convention
#![feature(custom_test_frameworks)] // use custom test harness
#![feature(alloc_error_handler)] // specify a handler when allocation error occurs
#![test_runner(crate::test_harness::test_runner)] // specify test_runner
#![reexport_test_harness_main = "test_main"]
#![cfg_attr(test, no_main)]
#![warn(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]

#[macro_use]
extern crate lazy_static;
extern crate alloc;

pub mod allocator;
pub mod gdt;
pub mod interrupts;
pub mod memory;
pub mod naked_interrupts;
pub mod serial;
pub mod test_harness;
pub mod vga_buffer;

#[cfg(test)]
use bootloader::{entry_point, BootInfo};
#[cfg(not(feature = "naked"))]
use interrupts::init_idt;

#[cfg(feature = "naked")]
use naked_interrupts::init_idt;

#[cfg(test)]
use crate::test_harness::{exit_qemu, test_panic_handler, QemuExitCode};
#[cfg(test)]
use core::panic::PanicInfo;

/// Test-only panic handler that prints to serial port
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
  test_panic_handler(info)
}

#[cfg(test)]
entry_point!(test_kernel_main);

/// The entry point of our kernel library,
/// only needed when running tests
#[cfg(test)]
#[no_mangle]
pub fn test_kernel_main(boot_info: &'static BootInfo) -> ! {
  init();
  test_main();
  exit_qemu(QemuExitCode::Success)
}

/// Init procedure for the kernel
pub fn init() {
  gdt::init_gdt();
  init_idt();
  // initialize interrupt controller
  unsafe {
    interrupts::PICS.lock().initialize();
  }
  // enable hardware interrupts
  x86_64::instructions::interrupts::enable();
}

/// Halt the cpu until the next interrupt occurs using
/// a much cpu-cheap mechanism of the hlt instruction.
pub fn hlt_loop() -> ! {
  loop {
    x86_64::instructions::hlt();
  }
}

#[cfg(test)]
mod tests {
  #[test_case]
  fn trivial_test() {
    assert_eq!(1, 1);
  }
}
