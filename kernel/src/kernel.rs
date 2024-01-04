#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)]

extern crate alloc;

use bootloader_api::{config::Mapping, BootInfo, BootloaderConfig};
use x86_64::VirtAddr;

use crate::{memory::BootInfoFrameAllocator, task::{executor::Executor, Task}};

mod allocator;
#[path = "./main.rs"]
mod entry;
mod framebuffer;
pub mod gdt;
pub mod interrupts;
mod memory;
mod panic;
mod test_runner;
mod task;

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

bootloader_api::entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);
pub fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    let BootInfo {
        ref mut framebuffer,
        ref memory_regions,
        ref physical_memory_offset,
        ..
    } = boot_info;
    framebuffer::init_print(framebuffer);
    println_color!([0, 255, 0], "[OK] Initialised framebuffer.");
    gdt::init();
    println_color!([0, 255, 0], "[OK] Initialised GDT.");
    interrupts::init_idt();
    println_color!([0, 255, 0], "[OK] Initialised IDT.");
    let phys_mem_offset = VirtAddr::new(physical_memory_offset.into_option().unwrap());
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(memory_regions) };
    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("[ERR] Failed to initialise heap allocator!.");
    println_color!([0, 255, 0], "[OK] Initialised heap allocator.");
    let mut executor = Executor::new();
    println_color!([0, 255, 0], "[OK] Initialised executor.");

    #[cfg(test)]
    test_main();

    #[cfg(not(test))]
    println_color!([255, 255, 0], "[...] Adding kernel to task list.");
    #[cfg(not(test))]
    executor.spawn(Task::new(entry::main()));

    println_color!([0, 255, 0], "[...] Running tasks...\n");
    executor.run();
}
