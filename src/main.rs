//! The main binary of blog_os

#![no_std]
#![no_main]
#![warn(missing_docs)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

#[macro_use]
extern crate lazy_static;

mod serial;
mod vga_buffer;

use core::panic::PanicInfo;

/// Since we're not using std, we cannot use the panic
/// handler defined in std.
/// This handler will not be called anyway since we abort on panic
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
  println!("{}", info);
  loop {}
}

/// Test-only panic handler that prints to serial port
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
  serial_println!("[failed]\n");
  serial_println!("Error: {}\n", info);
  exit_qemu(QemuExitCode::Failed);
}

/// Qemu Exit Code
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
  /// Succes code: 00010000 -> 16
  Success = 0x10,
  /// Failure code: 00010001 -> 17
  Failed = 0x11,
}

/// Exit qemu by writing to the debug exit port
/// Node: the port should correspond to configuration in Cargo.toml
pub fn exit_qemu(exit_code: QemuExitCode) -> ! {
  use x86_64::instructions::port::Port;
  let mut port = Port::new(0xf4);
  loop {
    unsafe {
      port.write(exit_code as u32);
    }
  }
}

/// The entry point of our kernel
#[no_mangle]
pub extern "C" fn _start() -> ! {
  println!("Hello World!");

  // run test harness
  #[cfg(test)]
  test_main();

  loop {}
}

#[cfg(test)]
fn test_runner(tests: &[&dyn Testable]) {
  serial_println!("Running {} tests", tests.len());
  for test in tests {
    // just run each test function
    test.run();
  }
  // exit qemu when test harness is done
  exit_qemu(QemuExitCode::Success);
}

/// Testable Trait used for printing test information
/// in a unified way
pub trait Testable {
  /// Run the test function surrounded by informative logs
  fn run(&self) -> ();
}

impl<T> Testable for T
where
  T: Fn(),
{
  fn run(&self) {
    serial_print!("{}...\t", core::any::type_name::<T>());
    self();
    serial_println!("[ok]");
  }
}

#[cfg(test)]
mod tests {

  #[test_case]
  fn trivial_test() {
    assert_eq!(1, 1);
  }
}
