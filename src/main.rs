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

async fn mode(context: &AppContext) {
    let matches = command!()
        .arg(
            arg!(<MODE>)
                .help("Choose mode")
                .value_parser([CAPTURE_MODE, BOT_MODE]),
        )
        .get_matches();

    let mut mode = String::from(DEFAULT_MODE);
    match matches.get_one::<String>("MODE") {
        None => {
            println!("'MODE' is not selected. Defaulting to {}", DEFAULT_MODE);
        }
        Some(m) => match m.as_str() {
            CAPTURE_MODE => mode = String::from(CAPTURE_MODE),
            BOT_MODE => mode = String::from(BOT_MODE),
            _ => {
                panic!("Unknown mode");
            }
        },
    }

    match mode.as_str() {
        CAPTURE_MODE => app::main(&context).await,
        BOT_MODE => discord_bot::main(&context).await,
        _ => {
            panic!("Unknown mode");
        }
    }
}
