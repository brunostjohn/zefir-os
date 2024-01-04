#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)]

use bootloader_api::BootInfo;

#[path = "./main.rs"]
mod entry;
mod framebuffer;
pub mod interrupts;
mod panic;
mod test_runner;

bootloader_api::entry_point!(kernel_main);
pub fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    framebuffer::init_print(boot_info);
    interrupts::init_idt();

    #[cfg(test)]
    test_main();

    #[cfg(not(test))]
    entry::main();

    println!("Exiting successfully!");

    #[allow(clippy::empty_loop)]
    loop {}
}
