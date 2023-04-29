#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use crate::action::Action;

use dotenv::dotenv;

mod platform;
mod cli;
mod discord_bot;
mod action;

#[tokio::main]
async fn main() {
    println!("OS type: {}", std::env::consts::OS);
    println!("OS version: {}", std::env::consts::ARCH);

    dotenv().ok();

    let mut actions = vec![
        Action::new(
            "do",
            "Do something",
            || { Box::pin(async { println!("Doing something"); }) },
        ),
    ];

    let mut bot_actions = discord_bot::cli_actions();

    actions.append(&mut bot_actions);

    match cli::main(&actions).await {
        Ok(_) => println!("Done"),
        Err(e) => println!("Error: {}", e),
    }
}
