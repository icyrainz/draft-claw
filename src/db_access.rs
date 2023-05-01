use std::env;

use surrealdb::Surreal;
// use surrealdb::engine::remote::ws::{Wss, Client};
use surrealdb::engine::remote::http::{Https, Client};
use surrealdb::opt::auth::Root;

use crate::models::draft_data::*;
use crate::models::card::*;
use crate::models::card_rating::*;

async fn get_db() -> Result<Surreal<Client>, surrealdb::Error> {
    let host = env::var("SURREAL_DB_HOST").expect("SURREAL_DB_HOST not set");
    let user = env::var("SURREAL_DB_USER").expect("SURREAL_DB_USER not set");
    let pass = env::var("SURREAL_DB_PASS").expect("SURREAL_DB_PASS not set");

    println!("Connecting to {}", host);
    let db = Surreal::new::<Https>(host).await?;
    db.use_ns("dc").use_db("dc").await?;

    println!("Signing in as {}", user);
    db.signin(Root {
        username: &user,
        password: &pass,
    })
    .await?;

    println!("After signin");

    Ok(db)
}

pub async fn insert_draft_record(draft_records: &Vec<DraftRecord>) -> surrealdb::Result<()> {
    let db = get_db().await?;

    for record in draft_records {
        let record_id = format!("{}-{}", record.game_id, record.pick.pick_str);
        let db_record: DraftRecord = 
            db.create(("draft_record", record_id))
            .content(record)
            .await?;
        }

    Ok(())
}

pub async fn insert_card_rating(card_ratings: &Vec<CardRating>) -> surrealdb::Result<()> {
    let db = get_db().await?;

    for rating in card_ratings {
        let db_rating: CardRating = 
            db.create(("card_rating", rating.name.to_string()))
            .content(rating)
            .await?;
    }

    Ok(())
}
