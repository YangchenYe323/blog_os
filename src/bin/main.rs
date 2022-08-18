//! The main binary of blog_os

#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_harness::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::vec::Vec;
use blog_os::{memory::BootInfoFrameAllocator, println};
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use x86_64::VirtAddr;

#[cfg(test)]
use blog_os::test_harness::{exit_qemu, QemuExitCode};

// register [kernel_main] as the entry point called by bootloader.
entry_point!(kernel_main);

/// The [bootloader] we use passes in [BootInfo] to the start procedure.
/// It contains an overview of the memory layout of the system
/// and an offset from which physical addresses start. That is, we can
/// visit arbitrary physical address 'x' by visiting virtual address
/// 'x' + 'offset'.
#[no_mangle]
fn kernel_main(boot_info: &'static BootInfo) -> ! {
  println!("Hello World!");
  blog_os::init();

  #[cfg(test)]
  {
    test_main();
    exit_qemu(QemuExitCode::Success)
  }

  #[cfg(not(test))]
  {
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);

    // use OffsetPageTable
    let mut mapper =
      unsafe { blog_os::memory::init_offset_page_table(phys_mem_offset) };
    let mut frame_allocator =
      unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    blog_os::allocator::init_heap(&mut mapper, &mut frame_allocator).unwrap();

    // now we can use dynamic allocation
    let mut v = Vec::new();
    for i in 0..10 {
      v.push(i);
    }
    println!("{:?}", v);
    // this is the vector fat pointer on the stack
    println!("Vec ref at {:p}", &v);
    // this points to the underlying data on the heap
    println!("Vec at {:p}", &v[..]);
    // this points to the third slot in the slice
    println!("Vec[2] at {:p}", &v[2]);

    println!("It did not crash!");

    blog_os::hlt_loop();
  }
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
