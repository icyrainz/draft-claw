#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use dotenv::dotenv;

mod context;
mod db_access;
mod discord_bot;
mod models;
mod app;

#[tokio::main]
async fn main() {
    println!("OS type: {}", std::env::consts::OS);
    println!("OS version: {}", std::env::consts::ARCH);

    dotenv().ok();

    let context = context::create_context();

    // app::main(&context).await;
    discord_bot::main().await;
}
