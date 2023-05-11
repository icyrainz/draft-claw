#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use dotenv::dotenv;

#[cfg(feature = "capture")]
mod app;

#[cfg(feature = "bot")]
mod discord_bot;

mod app_context;
mod db_access;
mod models;
mod card_loader;

fn init() {
    println!("OS type: {}", std::env::consts::OS);
    println!("OS version: {}", std::env::consts::ARCH);

    dotenv().ok();
}

#[cfg(all(feature = "capture", not(feature = "bot")))]
#[tokio::main]
async fn main() {
    init();
    let context = app_context::create_context();

    app::main(&context).await;
}

#[cfg(all(feature = "bot", not(feature = "capture")))]
#[tokio::main]
async fn main() {
    init();
    let context = app_context::create_context();

    discord_bot::main(&context).await;
}

#[cfg(all(feature = "bot", feature = "capture"))]
#[tokio::main]
async fn main() {
    init();
    let context = app_context::create_context();

    if cfg!(feature = "bot") {
        discord_bot::main(&context).await;
    } else if cfg!(feature = "capture") {
        app::main(&context).await;
    } else {
        panic!("No feature specified");
    }
}
