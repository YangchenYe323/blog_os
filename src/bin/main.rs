//! The main binary of blog_os

#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_harness::test_runner)]
#![reexport_test_harness_main = "test_main"]

use blog_os::println;
use core::panic::PanicInfo;

#[cfg(test)]
use blog_os::test_harness::{exit_qemu, QemuExitCode};

#[no_mangle]
pub extern "C" fn _start() -> ! {
  println!("Hello World!");
  blog_os::init();

  #[cfg(test)]
  {
    test_main();
    exit_qemu(QemuExitCode::Success)
  }

  #[cfg(not(test))]
  {
    // invoke breakpoint exception
    x86_64::instructions::interrupts::int3();
    println!("It did not crash!");
    loop {}
  }
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
  println!("{}", info);
  loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
  blog_os::test_harness::test_panic_handler(info)
}
