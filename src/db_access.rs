use std::env;

use serde_json::json;
use surrealdb::Surreal;
// use surrealdb::engine::remote::ws::{Wss, Client};
use surrealdb::engine::remote::http::{Client, Https};
use surrealdb::opt::auth::Root;

use crate::models::card_rating::*;
use crate::models::draft_data::*;
use crate::models::draft_game::*;

const DRAFT_RECORD_TABLE: &str = "draft_record";
const CARD_RATING_TABLE: &str = "card_rating";
const DRAFT_GAME_TABLE: &str = "draft_game";
const DRAFT_VOTE_TABLE: &str = "draft_vote";

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

pub async fn upsert_draft_record(draft_record: &DraftRecord) -> surrealdb::Result<()> {
    let db = get_db().await?;

    let record_id = format!("{}-{}", draft_record.game_id, draft_record.pick.pick_str);
    let db_record: DraftRecord = db
        .update((DRAFT_RECORD_TABLE, record_id))
        .content(draft_record)
        .await?;

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
        let db_rating: CardRating = db
            .create((CARD_RATING_TABLE, rating.name.to_string()))
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
        DRAFT_GAME_TABLE, user_id
    );

    let mut result = db.query(query).bind(("table", DRAFT_GAME_TABLE)).await?;

    result.take(0)
}

pub async fn upsert_draft_game(draft_game: &DraftGame) -> surrealdb::Result<()> {
    let db = get_db().await?;

    let db_record: DraftGame = db
        .update((DRAFT_GAME_TABLE, draft_game.game_id.to_string()))
        .content(draft_game)
        .await?;

    Ok(())
}

pub async fn insert_draft_game(game_id: &str) -> surrealdb::Result<DraftGame> {
    let db = get_db().await?;

    let draft_game: Option<DraftGame> = db.select((DRAFT_GAME_TABLE, game_id)).await?;
    match draft_game {
        Some(game) => {
            return Ok(game);
        }
        None => {
            let create_game_query = format!(
                r#"
                CREATE {}:{} CONTENT {{
                    'game_id': '{}',
                    'time': time::now(),
                }}"#,
                DRAFT_GAME_TABLE, game_id, game_id
            );
            db.query(create_game_query).await?;

            return Ok(db.select((DRAFT_GAME_TABLE, game_id)).await?);
        }
    }
}

pub async fn upsert_draft_vote(draft_vote: &DraftVote) -> surrealdb::Result<()> {
    let db = get_db().await?;

    let db_record: DraftVote = db
        .update((DRAFT_VOTE_TABLE, draft_vote.get_record_key()))
        .content(draft_vote)
        .await?;

    Ok(())
}

pub async fn get_highest_voted_pick(
    game_id: &str,
    draft_pick: &DraftPick,
) -> Result<Option<u8>, String> {
    let db = get_db().await.map_err(|err| err.to_string())?;

    let query = format!(
        "SELECT vote_index, count() FROM {} WHERE game_id = '{}' AND draft_pick = {} GROUP BY vote_index ORDER BY count DESC LIMIT 1",
        DRAFT_VOTE_TABLE, game_id, json!(draft_pick).to_string()
    );

    let result: Option<(u32, u8)> = db
        .query(query)
        .bind(("table", DRAFT_VOTE_TABLE))
        .await.map_err(|err| err.to_string())?
        .take(0).map_err(|err| err.to_string())?;

    match result {
        Some((vote_index, _)) => Ok(Some(vote_index as u8)),
        None => Err("No votes found".to_string())
    }
}
