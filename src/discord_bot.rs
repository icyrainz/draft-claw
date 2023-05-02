use std::future::Future;
use std::pin::Pin;
use std::{collections::HashMap, env, sync::Arc, thread};

use tokio::runtime::Handle;
use tokio::{sync::Mutex, task::JoinHandle};

use serenity::{
    async_trait,
    model::prelude::*,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};

use crate::{app, db_access};
use crate::app_context::AppContext;
use crate::models::draft_data::*;
use crate::models::card::*;

const DRAFT_COMMAND: &str = "!draft";
const CARD_COMMAND: &str = "!card";

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

struct BotCache;
impl TypeMapKey for BotCache {
    type Value = Arc<RwLock<HashMap<String, String>>>;
}

struct BotAppContext;
impl TypeMapKey for BotAppContext {
    type Value = Arc<AppContext>;
}

async fn get_draft_data(ctx: &Context, game_id: &str) -> String {
//     let draft_data = "[   Rare   ] Evening Hare                  : S-
// [ Uncommon ] First Watch                   : F
// [ Uncommon ] Colony Steward                : C-
// [ Uncommon ] Elite Myrmidon                : A-
// [  Common  ] Cobalt Acolyte                : C
// [  Common  ] Midnight Hunter               : C+
// [  Common  ] Highpeak Rider                : D
// [  Common  ] Apprentice Ranger             : C+
// [  Common  ] Daring Leap                   : D-
// [  Common  ] Twilight Lady                 : C+
// [  Common  ] Refuse Roller                 : D
// [  Common  ] Maggot Swarm                  : F+"
//         .to_string();

    let draft_data = 
        match db_access::get_last_draft_record(game_id).await {
            Ok(Some(record)) => {
                record.selection_text
            }
            _ => "No data".to_string(),
        };

    draft_data
}

async fn get_card(ctx: &Context, card_name: &str) -> Option<Card> {
    let card_data = {
        let data = ctx.data.read().await;
        data.get::<BotCardData>()
            .expect("Expected CardData in TypeMap.")
            .clone()
    };

    println!("Getting card {} from data {}", card_name, card_data.len());
    card_data.get(card_name).cloned()
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

async fn get_user_game(ctx: &Context, user: &str) -> Option<String> {
    let cache_lock = {
        let data = ctx.data.read().await;
        data.get::<BotCache>()
            .expect("Expected BotCache in TypeMap.")
            .clone()
    };

    let cache = cache_lock.read().await;
    cache.get(user).cloned()
}

async fn process_draft_command(ctx: &Context, channel_id: ChannelId, user: User, args: &str) {
    let mut cmd_parts = args.splitn(2, char::is_whitespace);
    let cmd = cmd_parts.next();

    match cmd {
        Some(sub_cmd) => {
            let args = cmd_parts.next().unwrap_or("");

            match sub_cmd {
                "reg" => {
                    let game_id = args;

                    register_user(&ctx, &user.name, game_id).await;

                    let reply = format!("Game [{}] is registered to {}", game_id, &user.name);
                    send_message(&ctx, channel_id, &reply).await;
                }
                _ => {
                    let game_id = get_user_game(&ctx, &user.name).await;

                    let game_id = match game_id {
                        Some(game_id) => {
                            let reply = format!("Game [{}]", game_id);
                            send_message(&ctx, channel_id, &reply).await;

                            game_id
                        }
                        None => {
                            let reply = format!("No game registered to {}", &user.name);
                            send_message(&ctx, channel_id, &reply).await;

                            "nDyQIWGt".to_string()
                        }
                    };

                    let draft_data = get_draft_data(&ctx, &game_id).await;
                    send_message(&ctx, channel_id, &draft_data).await;
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
    let found_card = get_card(&ctx, &card_name).await;
    match found_card {
        Some(card) => {
            let reply = format!("{}", card.image_url.to_string());
            send_message(&ctx, channel_id, &reply).await;
        }
        None => {
            let reply = format!("Card {} not found", card_name);
            send_message(&ctx, channel_id, &reply).await;
        }
    }
}

struct BotHandler;

#[async_trait]
impl EventHandler for BotHandler {
    async fn message(&self, ctx: Context, msg: Message) {
        let mut cmd_parts = msg.content.splitn(2, char::is_whitespace);
        let cmd = cmd_parts.next().ok_or(()).unwrap();
        let args = cmd_parts.next().unwrap_or("");

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

pub async fn main(context: &AppContext) {
    let token = env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN in the environment");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(BotHandler)
        .await
        .expect("Err creating client");
    {
        let mut data = client.data.write().await;
        let card_data = Arc::new(app::load_card_hashmap_by_name());
        data.insert::<BotCardData>(card_data);

        data.insert::<BotCache>(Arc::new(RwLock::new(HashMap::new())));
    }

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
