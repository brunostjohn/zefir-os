use crate::println;

pub fn main() {
    println!("Hi there!");

    unsafe {
        *(0xdeadbeef as *mut u8) = 42;
    };
}
