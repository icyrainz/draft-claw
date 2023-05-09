use std::{collections::HashMap, env, sync::Arc};

use indicium::simple::SearchIndex;

use serenity::{
    async_trait,
    model::prelude::*,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};

use crate::app_context::AppContext;
use crate::db_access;
use crate::models::card::*;

const DRAFT_COMMAND: &str = if cfg!(debug_assertions) {
    "!dd"
} else {
    "!draft"
};
const DRAFT_COMMAND_HELP: &str = r#"
!draft reg <game_id> - Register a draft (use the id generated from the app)
!draft - Get the current draft selection
!draft deck - Get the current deck
"#;
const CARD_COMMAND: &str = "!card";

const DRAFT_THREAD_PREFIX: &str = "draft";

const CHANNEL_LIST_FILE: &str = "./resource/discord_channels.txt";

const CHANNEL_LIST_KEY: &str = "channel_list";

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

fn load_channel_list() -> Vec<ChannelId> {
    let file = std::fs::File::open(CHANNEL_LIST_FILE).expect("Failed to open channel list file");
    let reader = std::io::BufReader::new(file);
    let channel_list: Vec<ChannelId> =
        serde_json::from_reader(reader).expect("Failed to parse channel list file");
    channel_list
}

async fn get_draft_data(ctx: &Context, game_id: &str) -> String {
    let draft_data = match db_access::get_last_draft_record(game_id).await {
        Ok(Some(record)) => {
            format!(
                "Card {} of 48\n```{}```",
                record.pick.pick_id, record.selection_text
            )
        }
        _ => "No data".to_string(),
    };

    draft_data
}

async fn get_card(ctx: &Context, input_str: &str) -> Result<Card, String> {
    println!("Searching for card with string: {}", input_str);
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
}

async fn register_user(ctx: &Context, user: &str, game_id: &str) {
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

async fn get_decklist(ctx: &Context, game_id: &str) -> String {
    let deck_list = match db_access::get_last_draft_record(game_id).await {
        Ok(Some(record)) => {
            format!("```{}```", record.decklist_text.join("\n"))
        }
        _ => "No data".to_string(),
    };

    deck_list
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
    let mut cmd_parts = args.splitn(2, char::is_whitespace);
    let cmd = cmd_parts.next();

    match cmd {
        Some(sub_cmd) => {
            let args = cmd_parts.next().unwrap_or("");

            match sub_cmd {
                "help" => {
                    send_message(&ctx, channel_id, &get_help_text()).await;
                }
                "reg" => {
                    let game_id = args;

                    register_user(&ctx, &user.name, game_id).await;

                    let reply = format!("Game [{}] is registered to {}", game_id, &user.name);
                    send_message(&ctx, channel_id, &reply).await;
                }
                other => {
                    let game_id = get_cached_data(&ctx, &user.name).await;
                    let game_id = match game_id {
                        Some(game_id) => {
                            let reply = format!("Game [{}]", game_id);
                            send_message(&ctx, channel_id, &reply).await;

                            game_id
                        }
                        None => {
                            let reply = format!("No game is registered to {}", &user.name);
                            send_message(&ctx, channel_id, &reply).await;

                            return;
                        }
                    };

                    match other {
                        "deck" => {
                            let decklist = get_decklist(&ctx, &game_id).await;
                            send_message(&ctx, channel_id, &decklist).await;
                        }
                        _ => {
                            let draft_data = get_draft_data(&ctx, &game_id).await;
                            send_message(&ctx, channel_id, &draft_data).await;
                        }
                    }
                }
            }
        }
        None => {
            send_message(&ctx, channel_id, "Unknown command").await;
        }
    }
}

async fn process_card_command(ctx: &Context, channel_id: ChannelId, user: User, args: &str) {
    let card_name = args;
    let find_card_result = get_card(&ctx, &card_name).await;
    match find_card_result {
        Ok(found_card) => {
            let reply = format!("{}", found_card.image_url.to_string());
            send_message(&ctx, channel_id, &reply).await;
        }
        Err(e) => {
            send_message(&ctx, channel_id, &e).await;
        }
    }
}

fn get_help_text() -> String {
    format!("```{}```", DRAFT_COMMAND_HELP)
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
            DRAFT_COMMAND => {
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
