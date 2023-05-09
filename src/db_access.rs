use std::env;

use surrealdb::Surreal;
// use surrealdb::engine::remote::ws::{Wss, Client};
use surrealdb::engine::remote::http::{Https, Client};
use surrealdb::opt::auth::Root;

use crate::models::draft_game::*;
use crate::models::draft_data::*;
use crate::models::card_rating::*;

const DRAFT_RECORD_TABLE: &str = "draft_record";
const CARD_RATING_TABLE: &str = "card_rating";
const DRAFT_GAME_TABLE: &str = "draft_game";

async fn get_db() -> surrealdb::Result<Surreal<Client>> {
    let host = env::var("SURREAL_DB_HOST").expect("SURREAL_DB_HOST not set");
    let user = env::var("SURREAL_DB_USER").expect("SURREAL_DB_USER not set");
    let pass = env::var("SURREAL_DB_PASS").expect("SURREAL_DB_PASS not set");

    println!("Connecting to {}", host);
    let db = Surreal::new::<Https>(host).await?;
    db.use_ns("dc").use_db("dc").await?;

    db.signin(Root {
        username: &user,
        password: &pass,
    })
    .await?;

    Ok(db)
}

pub async fn upsert_draft_record(draft_records: &Vec<DraftRecord>) -> surrealdb::Result<()> {
    let db = get_db().await?;

    for record in draft_records {
        let record_id = format!("{}-{}", record.game_id, record.pick.pick_str);
        let db_record: DraftRecord = 
            db.update((DRAFT_RECORD_TABLE, record_id))
            .content(record)
            .await?;
        }

    Ok(())
}

pub async fn get_last_draft_record(game_id: &str) -> surrealdb::Result<Option<DraftRecord>> {
    let db = get_db().await?;

    let query = format!(
        "SELECT * FROM {} WHERE game_id = '{}' ORDER BY pick.pick_id DESC LIMIT 1",
        DRAFT_RECORD_TABLE, game_id
    );
    let mut result = db.query(query).bind(("table", DRAFT_RECORD_TABLE)).await?;
    let result_item: Option<DraftRecord> = result.take(0)?;
    Ok(result_item)
}

pub async fn insert_card_rating(card_ratings: &Vec<CardRating>) -> surrealdb::Result<()> {
    let db = get_db().await?;

    for rating in card_ratings {
        let db_rating: CardRating = 
            db.create((CARD_RATING_TABLE, rating.name.to_string()))
            .content(rating)
            .await?;
    }

    Ok(())
}

pub async fn get_draft_game(game_id: &str) -> surrealdb::Result<Option<DraftGame>> {
    let db = get_db().await?;

    let draft_game = db.select((DRAFT_GAME_TABLE, game_id)).await?;
    Ok(draft_game)
} 

pub async fn get_last_draft_game_by_user(user_id: &str) -> surrealdb::Result<Option<DraftGame>> {
    let db = get_db().await?;

    let query = format!(
        "SELECT * FROM {} WHERE user_id = '{}' ORDER BY time DESC LIMIT 1",
        DRAFT_GAME_TABLE,
        user_id
    );

    let mut result = db.query(query).bind(("table", DRAFT_GAME_TABLE)).await?;

    result.take(0)
}

pub async fn insert_draft_game(game_id: &str) -> surrealdb::Result<()> {
    let db = get_db().await?;

    let draft_game = DraftGame::new(game_id);
    let query = format!(r#"
        CREATE {}:{} CONTENT {{
            'game_id': '{}',
            'time': time::now(),
        }}"#,
        DRAFT_GAME_TABLE,
        game_id,
        game_id);

    let result = db.query(query).await?;

    Ok(())
}
