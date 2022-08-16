//! The main binary of blog_os

#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_harness::test_runner)]
#![reexport_test_harness_main = "test_main"]

use blog_os::println;
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;

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
    use x86_64::structures::paging::Translate;
    use x86_64::VirtAddr;

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);

    let addresses = [
      // the identity-mapped vga buffer page
      0xb8000,
      // some code page
      0x201008,
      // some stack page
      0x0100_0020_1a10,
      // virtual address mapped to physical address 0
      boot_info.physical_memory_offset,
    ];

    // use OffsetPageTable
    let mut mapper =
      unsafe { blog_os::memory::init_offset_page_table(phys_mem_offset) };
    for &address in &addresses {
      let virt = VirtAddr::new(address);
      // new: use the `mapper.translate_addr` method
      let phys = mapper.translate_addr(virt);
      println!("{:?} -> {:?}", virt, phys);
    }

    // map an unused page
    use x86_64::structures::paging::Page;
    let mut frame_allocator = unsafe {
      blog_os::memory::BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    let page = Page::containing_address(VirtAddr::new(0xdeadbeef));
    // map this page to 0x8000
    blog_os::memory::create_example_mapping(
      page,
      &mut mapper,
      &mut frame_allocator,
    );

    // write the string `New!` to the screen through the new mapping
    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e) };

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
    // loop {
    //   for _ in 0..10000 {

    //   }
    //   print!("-");
    // }

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
  // this is a read-only code page
  let ptr = 0x205495 as *mut i32;
  unsafe {
    // read should succeed and read in some garbage
    println!("{}", *ptr);
  }
  // this causes page fault
  unsafe { *(0x205495 as *mut u64) = 42 };
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
