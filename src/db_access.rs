use crate::models::card_rating::*;
use crate::models::draft_data::*;
use crate::models::draft_game::*;
use crate::opt::*;

use std::env;

use surrealdb::Surreal;
// use surrealdb::engine::remote::ws::{Wss, Client};
use surrealdb::engine::remote::http::{Client, Https};
use surrealdb::opt::auth::Root;

const DRAFT_RECORD_TABLE: &str = "draft_record";
const CARD_RATING_TABLE: &str = "card_rating";
const DRAFT_GAME_TABLE: &str = "draft_game";
const DRAFT_VOTE_TABLE: &str = "draft_vote";

fn log(s: String) {
    log_if(s.as_str(), DbgFlg::Db);
}

async fn get_db() -> Res<Surreal<Client>> {
    let host = env::var("SURREAL_DB_HOST").expect("SURREAL_DB_HOST not set");
    let user = env::var("SURREAL_DB_USER").expect("SURREAL_DB_USER not set");
    let pass = env::var("SURREAL_DB_PASS").expect("SURREAL_DB_PASS not set");

    log(format!("Connecting to {}", host));
    let db = Surreal::new::<Https>(host).await.err_to_str()?;
    db.use_ns("dc").use_db("dc").await.err_to_str()?;

    db.signin(Root {
        username: &user,
        password: &pass,
    })
    .await
    .err_to_str()?;

    Ok(db)
}

pub async fn upsert_draft_record(draft_record: &DraftRecord) -> Res<()> {
    let db = get_db().await?;

    let db_record: DraftRecord = db
        .update((DRAFT_RECORD_TABLE, draft_record.get_id()))
        .content(draft_record)
        .await
        .err_to_str()?;

    log(format!("Upserted draft record: {:?}", db_record));
    Ok(())
}

pub async fn get_last_draft_record(game_id: &str) -> Res<Option<DraftRecord>> {
    let db = get_db().await?;

    let query = format!(
        "SELECT * FROM {} WHERE game_id = '{}' ORDER BY pick.pick_id DESC LIMIT 1",
        DRAFT_RECORD_TABLE, game_id
    );
    let mut result = db
        .query(query)
        .bind(("table", DRAFT_RECORD_TABLE))
        .await
        .err_to_str()?;
    let result_item: Option<DraftRecord> = result.take(0).err_to_str()?;

    log(format!("Got last draft record: {:?}", result_item));
    Ok(result_item)
}

pub async fn get_draft_record(game_id: &str, pick: &DraftPick) -> Res<Option<DraftRecord>> {
    let db = get_db().await?;

    let record: Option<DraftRecord> = db
        .select((DRAFT_RECORD_TABLE, DraftRecord::generate_id(game_id, pick)))
        .await.err_to_str()?;

    Ok(record)
}

pub async fn insert_card_rating(card_ratings: &Vec<CardRating>) -> Res<()> {
    let db = get_db().await?;

    for rating in card_ratings {
        let db_rating: CardRating = db
            .create((CARD_RATING_TABLE, rating.name.to_string()))
            .content(rating)
            .await
            .err_to_str()?;
    }

    Ok(())
}

pub async fn get_draft_game(game_id: &str) -> Res<Option<DraftGame>> {
    let db = get_db().await?;

    let draft_game = db.select((DRAFT_GAME_TABLE, game_id)).await.err_to_str()?;

    log(format!("Got draft game: {:?}", draft_game));
    Ok(draft_game)
}

pub async fn get_last_draft_game_by_user(user_id: &str) -> Res<Option<DraftGame>> {
    let db = get_db().await?;

    let query = format!(
        "SELECT * FROM {} WHERE user_id = '{}' ORDER BY time DESC LIMIT 1",
        DRAFT_GAME_TABLE, user_id
    );

    let mut result = db
        .query(query)
        .bind(("table", DRAFT_GAME_TABLE))
        .await
        .err_to_str()?;

    let result_out = result.take(0).err_to_str();

    log(format!("Got last draft game: {:?}", result_out));
    result_out
}

pub async fn upsert_draft_game(draft_game: &DraftGame) -> Res<()> {
    let db = get_db().await?;

    let db_record: DraftGame = db
        .update((DRAFT_GAME_TABLE, draft_game.game_id.to_string()))
        .content(draft_game)
        .await
        .err_to_str()?;

    log(format!("Upserted draft game: {:?}", db_record));
    Ok(())
}

pub async fn insert_draft_game(game_id: &str) -> Res<DraftGame> {
    let db = get_db().await?;

    let draft_game: Option<DraftGame> =
        db.select((DRAFT_GAME_TABLE, game_id)).await.err_to_str()?;
    match draft_game {
        Some(game) => {
            log(format!("Found existing game: {:?}", game));
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
            db.query(create_game_query).await.err_to_str()?;

            let new_game = db.select((DRAFT_GAME_TABLE, game_id)).await.err_to_str()?;
            return Ok(new_game);
        }
    }
}

pub async fn upsert_draft_vote(draft_vote: &DraftVote) -> Res<()> {
    let db = get_db().await?;

    let db_record: DraftVote = db
        .update((DRAFT_VOTE_TABLE, draft_vote.get_id()))
        .content(draft_vote)
        .await
        .err_to_str()?;

    log(format!("Upserted draft vote: {:?}", db_record));
    Ok(())
}

pub async fn get_highest_voted_pick(game_id: &str, draft_pick: &DraftPick) -> Res<Option<u8>> {
    let db = get_db().await.err_to_str()?;

    let query = format!(
        "SELECT vote_idx, count() FROM {} WHERE game_id = '{}' AND draft_pick.pick_id = {} GROUP BY vote_idx ORDER BY count DESC LIMIT 1",
        DRAFT_VOTE_TABLE, game_id, draft_pick.pick_id
    );

    log(format!("Query: {}", query));

    let result: Res<Option<u8>> = db
        .query(query)
        .bind(("table", DRAFT_VOTE_TABLE))
        .await
        .err_to_str()?
        .take("vote_idx")
        .err_to_str();

    log(format!("Got highest voted pick: {:?}", result));
    result
}
