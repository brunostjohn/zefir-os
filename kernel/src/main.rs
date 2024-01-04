use alloc::vec;

use crate::println;

pub async fn main() {
    println!("Hi there!");

    let mut vector = vec![1, 2, 2];
    vector.push(4);
    println!("Vector: {:?}", vector);

    let number = async_number().await;
    println!("Async number: {}", number);

    // x86_64::instructions::interrupts::int3();

    // unsafe {
    //     *(0xdeadbeef as *mut u8) = 42;
    // };
}

async fn async_number() -> u32 {
    42
}