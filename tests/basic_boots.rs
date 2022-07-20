#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_harness::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

use blog_os::println;
use blog_os::test_harness::{exit_qemu, test_panic_handler, QemuExitCode};

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn _start() -> ! {
  test_main();
  exit_qemu(QemuExitCode::Success);
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
  test_panic_handler(info)
}

#[test_case]
fn test_println() {
  println!("test_println output");
}
