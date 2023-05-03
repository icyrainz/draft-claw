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

#[tokio::main]
async fn main() {
    println!("OS type: {}", std::env::consts::OS);
    println!("OS version: {}", std::env::consts::ARCH);

    dotenv().ok();

    let context = app_context::create_context();

    mode(&context).await
}

fn get_mode(s: &str) -> Result<String, String> {
    match s {
        "" | CAPTURE_MODE => Ok(String::from(CAPTURE_MODE)),
        BOT_MODE => Ok(String::from(BOT_MODE)),
        _ => Err("Unknown mode".to_string()),
    }
}

async fn mode(context: &AppContext) {
    let matches = command!()
        .arg(arg!(<MODE>).help("Choose mode").value_parser(get_mode))
        .get_matches();

    if let Some(m) = matches.get_one::<String>("MODE") {
        match m.as_str() {
            CAPTURE_MODE => app::main(&context).await,
            BOT_MODE => discord_bot::main(&context).await,
            _ => {
                panic!("Unknown mode");
            }
        }
    }
}
