//! This module provides test harness to use in the context
//! of a kernel

use crate::{serial_print, serial_println};
use core::panic::PanicInfo;

/// Test panic handler that prints
/// information to the serial port
pub fn test_panic_handler(info: &PanicInfo) -> ! {
  serial_println!("[failed]\n");
  serial_println!("Error: {}\n", info);
  exit_qemu(QemuExitCode::Failed);
}

/// Testable Trait used for printing test information
/// in a unified way
pub trait Testable {
  /// Run the test function surrounded by informative logs
  fn run(&self);
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

/// Test runner
pub fn test_runner(tests: &[&dyn Testable]) {
  serial_println!("Running {} tests", tests.len());
  for test in tests {
    // just run each test function
    test.run();
  }
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
