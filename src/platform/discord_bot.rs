use std::{env, collections::HashMap};
use dotenv::dotenv;

use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};

const DRAFT_COMMAND: &str = "!draft";

struct BotData;

impl TypeMapKey for BotData {
    type Value = HashMap<String, String>;
}

async fn get_draft_data(ctx: &Context, game_id: &str) -> String {
    println!("Getting draft data");
    let draft_data = "[   Rare   ] Evening Hare                  : S-
[ Uncommon ] First Watch                   : F
[ Uncommon ] Colony Steward                : C-
[ Uncommon ] Elite Myrmidon                : A-
[  Common  ] Cobalt Acolyte                : C
[  Common  ] Midnight Hunter               : C+
[  Common  ] Highpeak Rider                : D
[  Common  ] Apprentice Ranger             : C+
[  Common  ] Daring Leap                   : D-
[  Common  ] Twilight Lady                 : C+
[  Common  ] Refuse Roller                 : D
[  Common  ] Maggot Swarm                  : F+".to_string();

    let mut data = ctx.data.write().await;
    let bot_data = data.get_mut::<BotData>().unwrap();
    bot_data.insert(game_id.to_string(), draft_data);

    drop(data);

    let data = ctx.data.read().await;
    let bot_data = data.get::<BotData>().unwrap();
    let bot_data = bot_data.get(game_id).unwrap().into();
    bot_data
}

struct BotHandler;

#[async_trait]
impl EventHandler for BotHandler {
    async fn message(&self, ctx: Context, msg: Message) {
        match msg.content.as_str() {
            DRAFT_COMMAND => {
                let game_id = "123";
                let draft_data = get_draft_data(&ctx, game_id).await;
                if let Err(why) = msg.channel_id.say(&ctx.http, draft_data).await {
                    println!("Error sending message: {:?}", why);
                }
            },
            "!ping" => {
                if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                    println!("Error sending message: {:?}", why);
                }
            },
            _ => {}
        };
    }
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
pub async fn run() {
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents = 
        GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(BotHandler)
        .await
        .expect("Err creating client");
    {
        let mut data = client.data.write().await;
        data.insert::<BotData>(HashMap::new());
    }

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
    println!("Bot started");
}
