//! This crate provides library code for blog_os kernel

#![no_std]
#![feature(abi_x86_interrupt)] // enable the unstable "x86-interrupt" calling convention
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_harness::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![warn(missing_docs)]

#[macro_use]
extern crate lazy_static;

pub mod interrupts;
pub mod naked_interrupts;
pub mod serial;
pub mod test_harness;
pub mod vga_buffer;

#[cfg(feature = "naked")]
use naked_interrupts::init_idt;
#[cfg(not(feature = "naked"))]
use interrupts::init_idt;

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

/// The entry point of our kernel library,
/// only needed when running tests
#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
  init();
  test_main();
  exit_qemu(QemuExitCode::Success)
}

/// Init procedure for the kernel
pub fn init() {
  init_idt();
}

#[cfg(test)]
mod tests {
  #[test_case]
  fn trivial_test() {
    assert_eq!(1, 1);
  }
}