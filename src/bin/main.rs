//! The main binary of blog_os

#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_harness::test_runner)]
#![reexport_test_harness_main = "test_main"]

use blog_os::{print, println};
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

    // #[allow(unconditional_recursion)]
    // fn stack_overflow() {
    //   stack_overflow(); // for each recursion, the return address is pushed
    // }
    // stack_overflow();

    // provoke breakpoint
    // breakpoint();

    // invalid opcode exception
    // invalid_opcode();

    // provoke a page fault
    // page_fault();

    // provoke a deadlock
    loop {
      for _ in 0..10000 {

      }
      print!("-");
    }

    println!("It did not crash!");

    blog_os::hlt_loop();
  }
}

#[allow(dead_code)]
fn invalid_opcode() {
  unsafe {
    core::arch::asm!("ud2");
  }
}

#[allow(dead_code)]
fn breakpoint() {
  x86_64::instructions::interrupts::int3();
}

#[allow(dead_code)]
fn page_fault() {
  unsafe { *(0xdeadbeaf as *mut u64) = 42 };
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
  println!("{}", info);
  blog_os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
  blog_os::test_harness::test_panic_handler(info)
}
