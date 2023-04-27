#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

mod platform;

#[macro_use]
extern crate objc;

fn main() {
    println!("OS type: {}", std::env::consts::OS);
    println!("OS version: {}", std::env::consts::ARCH);

    platform::run();
}
