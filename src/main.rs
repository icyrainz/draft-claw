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

#[cfg(feature = "capture")]
#[tokio::main]
async fn main() {
    init();
    let context = app_context::create_context();

    app::main(&context).await;
}

#[cfg(feature = "bot")]
#[tokio::main]
async fn main() {
    init();
    let context = app_context::create_context();

    discord_bot::main(&context).await;
}
