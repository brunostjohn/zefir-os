use crate::println;
use core::panic::PanicInfo;

#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);

    loop {}
}
