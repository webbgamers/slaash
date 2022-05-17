mod commands;

use std::sync::Arc;
use serenity::async_trait;
use serenity::client::bridge::gateway::ShardManager;
use serenity::model::gateway::Ready;
use serenity::model::interactions::{
    Interaction,
    application_command::ApplicationCommandInteraction,
    application_command::ApplicationCommandOptionType
};
use serenity::model::id::GuildId;
use serenity::Client;
use serenity::prelude::*;

use tracing::{error, info};

use crate::commands::ping::*;
use crate::commands::minesweeper::*;

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct Handler;

type Error = Box<dyn std::error::Error + Send + Sync>;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) -> () {
        if let Interaction::ApplicationCommand(command) = interaction {
            let command_name = command.data.name.clone();
            let command_name = command_name.as_str();
            let result: Result<(), Error> = match command_name {
                "ping" => ping(ctx, command).await,
                "minesweeper" => minesweeper(ctx, command).await,
                "bad" => bad(ctx, command).await,
                
                _ => Err("Not implemented.".into()),
            };

            if let Err(err) = result {
                error!("Command '{}' failed: {}", command_name, err)
            }
        };
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let guild_id = GuildId(567206658070020107);

        let new_commands = GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
            commands
                .create_application_command(|command| {
                    command.name("ping").description("Pong hopefully.")
                })
                .create_application_command(|command| {
                    command.name("minesweeper").description("Play a game of minesweeper!")
                    .create_option(|option| {
                        option
                            .name("mines").description("Number of mines.")
                            .kind(ApplicationCommandOptionType::Integer)
                            .min_int_value(1).max_int_value(23)
                    })
                })
                .create_application_command(|command| {
                    command.name("error").description("Test error.")
                })
            })
            .await
            .unwrap();

            info!("Registered commands: {:?}.", new_commands.into_iter().map(|c| c.name ).collect::<Vec<_>>());
        }
    }  


async fn bad(ctx: Context, command: ApplicationCommandInteraction) -> Result<(), Error> {
    command.member.unwrap().kick_with_reason(&ctx.http, "Test (hopefully doesnt ban)").await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    // Init logging
    tracing_subscriber::fmt::init();

    // Get token from .env
    let token = dotenvy::var("DISCORD_TOKEN").expect("Unable to find discord token.");

    // Setup intents
    let intents = GatewayIntents::empty();

    // Create client
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client.");

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
    }

    let shard_manager = client.shard_manager.clone();

    // Ctrl+C Handler
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.expect("Error registering Ctrl+C handler.");
        info!("Recieved Ctrl+C, shutting down.");
        shard_manager.lock().await.shutdown_all().await;
    });
        
    // Start client
    if let Err(err) = client.start().await {
        error!("Error starting client: {:?}.", err);
    };
}