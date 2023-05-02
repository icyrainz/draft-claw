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

#[tokio::main]
async fn main() {
    println!("OS type: {}", std::env::consts::OS);
    println!("OS version: {}", std::env::consts::ARCH);

    dotenv().ok();

    let context = app_context::create_context();

    app::main(&context).await
}

async fn mode(context: &AppContext) {
    let matches = command!()
        .arg(
            arg!(<MODE>)
            .help("Choose mode")
            .value_parser([CAPTURE_MODE, BOT_MODE])
        )
        .get_matches();

    match matches
        .get_one::<String>("MODE")
        .expect("'MODE' is required")
        .as_str()
        {
            CAPTURE_MODE => {
                app::main(&context).await
            }
            BOT_MODE => {
                discord_bot::main(&context).await
            }
            _ => {
                println!("Unknown mode");
            }
        }
}
