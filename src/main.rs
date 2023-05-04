#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use app_context::AppContext;
use dotenv::dotenv;

use clap::{arg, command};

mod app;
mod app_context;
mod db_access;
mod discord_bot;
mod models;

#[macro_use]
extern crate lazy_static;

const VERSION: &str = env!("CARGO_PKG_VERSION");

const CAPTURE_MODE: &str = "capture";
const BOT_MODE: &str = "bot";

const DEFAULT_MODE: &str = CAPTURE_MODE;

#[cfg(feature = "capture")]
#[tokio::main]
async fn main() {
    println!("OS type: {}", std::env::consts::OS);
    println!("OS version: {}", std::env::consts::ARCH);

    dotenv().ok();

    let context = app_context::create_context();

    println!("1: {}, 2: {}", CAPTURE_MODE, BOT_MODE);
    let mut buffer = String::new();
    std::io::stdin().read_line(&mut buffer).unwrap();
    match buffer.trim() {
        "" | "1" | CAPTURE_MODE => app::main(&context).await,
        "2" | BOT_MODE => discord_bot::main(&context).await,
        _ => panic!("Invalid mode"),
    }
}

#[cfg(feature = "bot")]
#[shuttle_runtime::main]
async fn serenity() -> shuttle_serenity::ShuttleSerenity {
    dotenv().ok();
    let context = app_context::create_context();

    Ok(discord_bot::init_client(&context).await.into())
}
