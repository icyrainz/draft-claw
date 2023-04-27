#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

mod platform;

fn main() {
    println!("OS type: {}", std::env::consts::OS);
    println!("OS version: {}", std::env::consts::ARCH);

    platform::run();
}
