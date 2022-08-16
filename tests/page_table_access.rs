//! Test that the kernel starts and can access the page table
//! through its [memory] module.

#![no_std]
#![no_main]

use blog_os::{
  memory::active_level4_page_table,
  println, serial_print, serial_println,
  test_harness::{exit_qemu, QemuExitCode},
};
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use x86_64::VirtAddr;

entry_point!(test_kernel_entry);

fn test_kernel_entry(boot_info: &'static BootInfo) -> ! {
  serial_print!("page_table_access::page_table_access...\t");
  let table = unsafe {
    active_level4_page_table(VirtAddr::new(boot_info.physical_memory_offset))
  };
  // we don't want to see the table, just make sure it doesn't crash
  println!("{:?}", table);
  serial_println!("[ok]");
  exit_qemu(QemuExitCode::Success);
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
  blog_os::test_harness::test_panic_handler(info)
}
