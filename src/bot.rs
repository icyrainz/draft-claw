mod bot_reply;

use crate::opt::*;

use std::{collections::HashMap, env, sync::Arc};

use indicium::simple::SearchIndex;

use serenity::{
    async_trait,
    model::prelude::*,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};

use crate::models::{card::*, draft_data::DraftPick};
use crate::{app_context::AppContext, models::draft_game::DraftGame};
use crate::{
    db_access::{self, get_last_draft_record},
    models::draft_game::DraftVote,
};

use self::bot_reply::BotReply;

const DRAFT_CMD: &str = if cfg!(debug_assertions) {
    "!dd"
} else {
    "!draft"
};
const DRAFT_CMD_HELP: &str = r#"
!draft - Get the current draft selection
!draft reg <game_id> - Register an existing draft
!draft own <game_id> - Register and own a game
!draft deck - Get the current deck
!draft vote <card_id|card_name> - Vote for a card
!draft commit - Commit the highest voted card. Only the owner can perform this.
"#;

const DRAFT_HELP_CMD: &str = "help";
const DRAFT_REG_CMD: &str = "reg";
const DRAFT_OWN_CMD: &str = "own";
const DRAFT_DECK_CMD: &str = "deck";
const DRAFT_VOTE_CMD: &str = "vote";
const DRAFT_COMMIT_CMD: &str = "commit";

const CARD_COMMAND: &str = "!card";

const DRAFT_THREAD_PREFIX: &str = "draft";

const CHANNEL_LIST_FILE: &str = "./resource/discord_channels.txt";

const CHANNEL_LIST_KEY: &str = "_channel_list";

async fn create_bot() {
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(BotHandler)
        .await
        .expect("Err creating client");

    tokio::spawn(async move {
        if let Err(why) = client.start().await {
            println!("Client error: {:?}", why);
        }
    });
    println!("Bot started");
}

struct BotCardData;
impl TypeMapKey for BotCardData {
    type Value = Arc<HashMap<String, Card>>;
}

struct BotCardIndex;
impl TypeMapKey for BotCardIndex {
    type Value = Arc<SearchIndex<String>>;
}

struct BotCache;
impl TypeMapKey for BotCache {
    type Value = Arc<RwLock<HashMap<String, String>>>;
}

struct BotAppContext;
impl TypeMapKey for BotAppContext {
    type Value = Arc<AppContext>;
}

fn log(s: String) {
    log_if(s.as_str(), DbgFlg::Bot);
}

fn load_channel_list() -> Vec<ChannelId> {
    let file = std::fs::File::open(CHANNEL_LIST_FILE).expect("Failed to open channel list file");
    let reader = std::io::BufReader::new(file);
    let channel_list: Vec<ChannelId> =
        serde_json::from_reader(reader).expect("Failed to parse channel list file");
    channel_list
}

async fn get_draft_data(game_id: &str) -> Option<(DraftPick, String)> {
    match db_access::get_last_draft_record(game_id).await {
        Ok(Some(record)) => Some((record.pick, record.selection_text)),
        _ => None,
    }
}

async fn find_card_in_list(list: &[String], input_str: &str) -> Option<u8> {
    let card_index = list.iter().fold(SearchIndex::default(), |mut acc, item| {
        acc.insert(item, item);
        acc
    });
    let search_result = card_index.search(input_str);

    let search_in_list_result = list
        .iter()
        .enumerate()
        .filter_map(|(i, s)| {
            if search_result.contains(&s) {
                Some(i as u8)
            } else {
                None
            }
        })
        .collect::<Vec<u8>>();

    if search_in_list_result.len() == 1 {
        Some(search_in_list_result[0])
    } else {
        None
    }
}

async fn get_card(ctx: &Context, input_str: &str) -> Result<Card, String> {
    log(format!("Searching for card with string: {}", input_str));
    let card_index = {
        let data = ctx.data.read().await;
        data.get::<BotCardIndex>()
            .expect("Expected CardIndex in TypeMap.")
            .clone()
    };
    let search_result = card_index.search(input_str);
    let found_card_name: &str;
    match search_result.len() {
        1 => {
            found_card_name = search_result[0];
        }
        0 => {
            return Err("No card found".to_string());
        }
        _ => {
            let found_cards = search_result
                .iter()
                .take(3)
                .map(|s| format!("[{}]", s))
                .collect::<Vec<String>>()
                .join(", ");
            return Err(format!("Multiple cards found: {}", &found_cards));
        }
    }

    let card_data = {
        let data = ctx.data.read().await;
        data.get::<BotCardData>()
            .expect("Expected CardData in TypeMap.")
            .clone()
    };

    println!(
        "Getting card {} from data {}",
        found_card_name,
        card_data.len()
    );
    Ok(card_data.get(found_card_name).cloned().unwrap())
}

async fn send_message(ctx: &Context, channel_id: ChannelId, msg: &str) {
    if let Err(why) = channel_id.say(&ctx.http, msg).await {
        println!("Error sending message: {:?}", why);
    }

    log(format!("Send to channel {}: {}", channel_id, msg));
}

async fn register_game_in_cache(ctx: &Context, user: &str, game_id: &str) {
    let cache_lock = {
        let data = ctx.data.read().await;
        data.get::<BotCache>()
            .expect("Expected BotCache in TypeMap.")
            .clone()
    };

    {
        let mut cache = cache_lock.write().await;
        cache
            .entry(String::from(user))
            .and_modify(|e| *e = game_id.to_string())
            .or_insert(game_id.to_string());
    }
}

async fn own_game(ctx: &Context, user: &str, game_id: &str) -> Result<(), String> {
    register_game_in_cache(ctx, user, game_id).await;

    let mut draft_game: DraftGame;
    match db_access::get_draft_game(game_id)
        .await
        .map_err(|err| err.to_string())?
    {
        Some(game) => {
            draft_game = game;
        }
        None => {
            draft_game = db_access::insert_draft_game(game_id)
                .await
                .map_err(|err| err.to_string())?;
        }
    }
    draft_game.user_id = Some(user.to_string());
    db_access::upsert_draft_game(&draft_game)
        .await
        .map_err(|err| err.to_string())?;

    Ok(())
}

async fn get_decklist(game_id: &str) -> String {
    let deck_list = match db_access::get_last_draft_record(game_id).await {
        Ok(Some(record)) => {
            format!("{}", record.decklist_text.join("\n"))
        }
        _ => "No data".to_string(),
    };

    deck_list
}

async fn get_chosen_pick(game_id: &str) -> Result<u8, String> {
    let draft_record = db_access::get_last_draft_record(game_id)
        .await
        .map_err(|err| err.to_string())?
        .ok_or(format!(
            "Unable to get last draft record for game {}",
            game_id
        ))?;

    let highest_pick = db_access::get_highest_voted_pick(&game_id, &draft_record.pick).await?;

    highest_pick.ok_or(format!(
        "Unable to get highest voted pick for pick {} in game [{}]",
        draft_record.pick.to_string(),
        game_id.to_string(),
    ))
}

async fn pick_card(game_id: &str, pick_idx: u8) -> Result<(DraftPick, String), String> {
    let draft_record = db_access::get_last_draft_record(game_id)
        .await
        .map_err(|err| err.to_string())?;
    match draft_record {
        Some(draft_record) => {
            if pick_idx as usize >= draft_record.selection_vec.len() {
                return Err(format!(
                    "Pick index {} is out of bounds for game {}",
                    pick_idx, game_id
                ));
            }

            let mut draft_record_with_pick = draft_record.clone();
            draft_record_with_pick.pick_card(pick_idx);

            log(format!("picking card at index {}", pick_idx));

            db_access::upsert_draft_record(&draft_record_with_pick)
                .await
                .map_err(|err| err.to_string())?;

            return Ok((
                draft_record_with_pick.pick,
                draft_record_with_pick.selection_vec[pick_idx as usize].to_string(),
            ));
        }
        None => {}
    }

    Err(format!("Unable to find draft record for game {}", game_id))
}

async fn vote_card(
    ctx: &Context,
    game_id: &str,
    user: &str,
    vote_text: &str,
) -> Res<(DraftPick, String)> {
    let draft_record = get_last_draft_record(game_id)
        .await
        .map_err(|err| err.to_string())?
        .ok_or("Could not find draft record".to_string())?;

    let vote_idx: u8;
    if let Ok(pick_num) = vote_text.parse::<u8>() {
        let pick_num = pick_num - 1;
        match draft_record.selection_vec.iter().nth((pick_num) as usize) {
            Some(card_name) => {
                vote_idx = pick_num;
            }
            _ => {
                return Err(format!("Unable to find card at index {}", pick_num));
            }
        }
    } else {
        vote_idx = find_card_in_list(&draft_record.selection_vec, vote_text)
            .await
            .ok_or("Unable to find card in list with vote text".to_string())?;
    }

    let draft_vote = DraftVote::new(game_id, user, &draft_record.pick, vote_idx);
    db_access::upsert_draft_vote(&draft_vote)
        .await
        .map_err(|err| err.to_string())?;

    Ok((
        draft_record.pick.clone(),
        draft_record.selection_vec[vote_idx as usize].to_string(),
    ))
}

async fn get_cached_data(ctx: &Context, cache_key: &str) -> Option<String> {
    let cache_lock = {
        let data = ctx.data.read().await;
        data.get::<BotCache>()
            .expect("Expected BotCache in TypeMap.")
            .clone()
    };

    let cache = cache_lock.read().await;
    cache.get(cache_key).cloned()
}

async fn process_draft_command(ctx: &Context, channel_id: ChannelId, user: User, args: &str) {
    let mut reply = BotReply::new();
    let mut cmd_parts = args.splitn(2, char::is_whitespace);
    let cmd = cmd_parts.next();

    match cmd {
        Some(sub_cmd) => {
            let args = cmd_parts.next().unwrap_or("");

            match sub_cmd {
                DRAFT_HELP_CMD => {
                    reply.add(get_help_text());
                }
                DRAFT_REG_CMD => {
                    let game_id = args;

                    register_game_in_cache(&ctx, &user.name, game_id).await;

                    reply.add(format!(
                        "Game [{}] is now registered to {}",
                        game_id, &user.name
                    ));
                }
                DRAFT_OWN_CMD => {
                    let game_id = args;

                    match own_game(ctx, &user.name, game_id).await {
                        Ok(_) => {
                            reply.add(format!(
                                "Game [{}] is now owned by [{}]",
                                game_id, &user.name
                            ));
                        }
                        Err(err) => {
                            reply.add(format!("Unable to own game: [{}]", err));
                        }
                    }
                }
                other => {
                    let game_id = get_cached_data(&ctx, &user.name).await;
                    let game_id = match game_id {
                        Some(game_id) => {
                            reply.add(format!("Game [{}]", game_id));

                            game_id
                        }
                        None => {
                            reply.add(format!("No game is registered to [{}]", &user.name));

                            send_message(&ctx, channel_id, &reply.to_string()).await;
                            return;
                        }
                    };

                    match other {
                        DRAFT_VOTE_CMD => {
                            let vote = args;

                            match vote_card(&ctx, &game_id, &user.name, vote).await {
                                Ok((draft_pick, picked_card_str)) => {
                                    reply.add(format!(
                                        "[{}] voted card for pick [{}]:\n{}",
                                        user.name.to_string(),
                                        draft_pick.to_string(),
                                        picked_card_str,
                                    ));
                                }
                                Err(err) => {
                                    reply.add(format!("Err: {}", err));

                                    send_message(&ctx, channel_id, &reply.to_string()).await;
                                    return;
                                }
                            }
                        }
                        DRAFT_COMMIT_CMD => match get_chosen_pick(&game_id).await {
                            Ok(chosen_pick) => match pick_card(&game_id, chosen_pick).await {
                                Ok((draft_pick, chosen_pick_str)) => {
                                    reply.add(format!(
                                        "[{}] committed card for pick [{}]:\n{}",
                                        user.name.to_string(),
                                        draft_pick.to_string(),
                                        chosen_pick_str
                                    ));
                                }
                                Err(err) => {
                                    reply.add(format!("Unable to pick: {}", err));

                                    send_message(&ctx, channel_id, &reply.to_string()).await;
                                    return;
                                }
                            },
                            Err(err) => {
                                reply.add(format!("Unable to get pick: {}", err));

                                send_message(&ctx, channel_id, &reply.to_string()).await;
                                return;
                            }
                        },
                        DRAFT_DECK_CMD => {
                            reply.add_boxed(get_decklist(&game_id).await);
                        }
                        _ => {
                            let draft_data = get_draft_data(&game_id).await;
                            match draft_data {
                                Some(draft_data_unwrap) => {
                                    reply
                                        .add(format!("Card {} of 48", draft_data_unwrap.0.pick_id));
                                    reply.add_boxed(draft_data_unwrap.1);
                                }
                                None => {
                                    reply.add(format!("No draft data available"));
                                }
                            }
                        }
                    }
                }
            }
        }
        None => {
            reply.add("Unknown command".to_string());
        }
    }

    send_message(&ctx, channel_id, &reply.to_string()).await;
}

async fn process_card_command(ctx: &Context, channel_id: ChannelId, user: User, args: &str) {
    let mut reply = BotReply::new();

    let card_name = args;
    let find_card_result = get_card(&ctx, &card_name).await;
    match find_card_result {
        Ok(found_card) => {
            reply.add(format!("{}", found_card.image_url.to_string()));
        }
        Err(e) => {
            reply.add(format!("Unable to find card: {}", e));
        }
    }

    send_message(&ctx, channel_id, &reply.to_string()).await;
}

fn get_help_text() -> String {
    format!("```{}```", DRAFT_CMD_HELP)
}

struct BotHandler;

#[async_trait]
impl EventHandler for BotHandler {
    async fn message(&self, ctx: Context, msg: Message) {
        let mut cmd_parts = msg.content.splitn(2, char::is_whitespace);
        let cmd = cmd_parts.next().ok_or(()).unwrap();
        let args = cmd_parts.next().unwrap_or("");

        let channel_list_str: String = get_cached_data(&ctx, CHANNEL_LIST_KEY)
            .await
            .unwrap_or_default();
        if !channel_list_str
            .split(',')
            .any(|s| s.trim() == &msg.channel_id.to_string())
        {
            let channel_info = ctx.http.get_channel(msg.channel_id.0).await;
            match channel_info {
                Ok(channel) => match channel {
                    Channel::Guild(channel) if channel.name.starts_with(DRAFT_THREAD_PREFIX) => {}
                    _ => {
                        return;
                    }
                },
                Err(e) => {
                    println!("Error getting channel info: {:?}", e);
                    return;
                }
            }
        }

        match cmd {
            DRAFT_CMD => {
                process_draft_command(&ctx, msg.channel_id, msg.author, args).await;
            }
            CARD_COMMAND => {
                process_card_command(&ctx, msg.channel_id, msg.author, args).await;
            }
            "!ping" => {
                send_message(&ctx, msg.channel_id, "Pong!").await;
            }
            _ => {}
        };
    }
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

pub async fn init_client(context: &AppContext) -> Client {
    let token = env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN in the environment");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let client = Client::builder(&token, intents)
        .event_handler(BotHandler)
        .await
        .expect("Err creating client");
    {
        let mut data = client.data.write().await;
        let card_data = crate::card_loader::load_card_hashmap_by_name();
        let mut card_index = SearchIndex::default();

        for (key, value) in card_data.iter() {
            card_index.insert(key, value);
        }

        // Read list of channel from CHANNEL_LIST_FILE
        let mut file =
            std::fs::File::open(CHANNEL_LIST_FILE).expect("Unable to open channel list file");
        let mut contents = String::new();
        std::io::Read::read_to_string(&mut file, &mut contents)
            .expect("Unable to read channel list file");

        data.insert::<BotCardData>(Arc::new(card_data));
        data.insert::<BotCardIndex>(Arc::new(card_index));

        let mut initial_data: HashMap<String, String> = HashMap::new();
        initial_data.insert(CHANNEL_LIST_KEY.to_string(), contents);

        // Pre-loaded data for testing
        // TODO: Remove this
        initial_data.insert(format!("Akio"), format!("C4UU1MSy"));
        // end of pre-loaded data

        data.insert::<BotCache>(Arc::new(RwLock::new(initial_data)));
    }

    client
}

pub async fn main(context: &AppContext) {
    let mut client = init_client(context).await;

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
