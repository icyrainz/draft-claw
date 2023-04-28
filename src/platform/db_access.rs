use dotenv::dotenv;
use std::env;

use surrealdb::Surreal;
use surrealdb::engine::remote::ws::Ws;
use surrealdb::opt::auth::Root;

async fn insert_card_rating() -> surrealdb::Result<()> {
    let host = env::var("SURREAL_DB_HOST").expect("SURREAL_DB_HOST not set");
    let user = env::var("SURREAL_DB_USER").expect("SURREAL_DB_USER not set");
    let pass = env::var("SURREAL_DB_PASS").expect("SURREAL_DB_PASS not set");

    let db = Surreal::new::<Ws>(host).await?;


    db.signin(Root {
        username: &user,
        password: &pass,
    })
    .await?;

    db.use_ns("dc").use_db("dc").await?;

    let sql = "CREATE card_rating SET name = $name, rating = $rating";

    Ok(())
}
