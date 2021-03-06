mod commands;

use std::collections::HashMap;
use std::sync::Arc;

use serenity::async_trait;
use serenity::client::bridge::gateway::ShardManager;
use serenity::model::gateway::Ready;
use serenity::model::id::GuildId;
use serenity::model::interactions::{
    application_command::ApplicationCommandOptionType, Interaction,
};
use serenity::prelude::*;
use serenity::Client;

use tracing::{error, info};

use crate::commands::blackjack::*;
use crate::commands::error::*;
use crate::commands::minesweeper::*;
use crate::commands::ping::*;
use crate::commands::tictactoe::*;

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct Handler;

type Error = Box<dyn std::error::Error + Send + Sync>;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::ApplicationCommand(command) => {
                let command_name = command.data.name.clone();
                let command_name = command_name.as_str();
                let result: Result<(), Error> = match command_name {
                    "ping" => ping(&ctx, &command).await,
                    "error" => fail(&ctx, &command).await,
                    "minesweeper" => minesweeper(&ctx, &command).await,
                    "blackjack" => blackjack(&ctx, &command).await,
                    "tictactoe" => tictactoe(&ctx, &command).await,

                    _ => Err("Command not implemented".into()),
                };

                if let Err(err) = result {
                    error!("Command '{}' failed: {}", command_name, err);
                    command.create_interaction_response(&ctx.http, |response| {
                        response.interaction_response_data(|data| {
                            data
                                .content(format!("There was an issue running your command.\n```{}```\nTry running it again or report the issue.", err))
                                .ephemeral(true)
                        })
                    })
                    .await
                    .unwrap_or_else(|err| error!("Failed to send error message: {}", err))
                }
            }
            Interaction::MessageComponent(component) => {
                let component_name = component.data.custom_id.split('-').next().unwrap();
                let result: Result<(), Error> = match component_name {
                    "minesweeper" => minesweeper_button(&ctx, &component).await,
                    "blackjack" => blackjack_button(&ctx, &component).await,
                    "tictactoe" => tictactoe_button(&ctx, &component).await,

                    // Ideas: connect 4 (or 3), liars dice, kakurasu
                    _ => Err("Unknown message component id".into()),
                };

                if let Err(err) = result {
                    error!("Component '{}' failed: {}", component.data.custom_id, err);
                    component.create_interaction_response(&ctx.http, |response| {
                        response.interaction_response_data(|data| {
                            data
                                .content(format!("There was an issue handling your interaction.\n```{}```\nTry again or again or report the issue.", err))
                                .ephemeral(true)
                        })
                    })
                    .await
                    .unwrap_or_else(|err| error!("Failed to send error message: {}", err))
                }
            }
            _ => error!("Unexpected interaction type"),
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);

        let guild_id = GuildId(567206658070020107);

        let new_commands = GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
            commands
                .create_application_command(|command| {
                    command.name("ping").description("Pong hopefully.")
                })
                .create_application_command(|command| {
                    command
                        .name("minesweeper")
                        .description("Clear tiles until you win, but dont hit a mine!")
                        .create_option(|option| {
                            option
                                .name("mines")
                                .description("Number of mines.")
                                .kind(ApplicationCommandOptionType::Integer)
                                .min_int_value(1)
                                .max_int_value(23)
                        })
                })
                .create_application_command(|command| {
                    command
                        .name("blackjack")
                        .description("Start a game of blackjack.")
                })
                .create_application_command(|command| {
                    command
                        .name("tictactoe")
                        .description("Play a classic game of tic-tac-toe.")
                        .create_option(|option| {
                            option
                                .name("size")
                                .description("Size of tic-tac-toe board.")
                                .kind(ApplicationCommandOptionType::Integer)
                                .min_int_value(2)
                                .max_int_value(5)
                        })
                })
                .create_application_command(|command| {
                    command.name("error").description("Test error.")
                })
        })
        .await
        .unwrap();

        info!(
            "Registered commands: {:?}",
            new_commands.into_iter().map(|c| c.name).collect::<Vec<_>>()
        );
    }
}

#[tokio::main]
async fn main() {
    // Init logging
    tracing_subscriber::fmt::init();

    // Get token from .env
    let token = dotenvy::var("DISCORD_TOKEN").expect("Unable to find discord token");

    // Setup intents
    let intents = GatewayIntents::empty();

    // Create client
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    // Setup persistent data stores
    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
        data.insert::<MinesweeperGames>(HashMap::default());
        data.insert::<TictactoeGames>(HashMap::default());
    }

    let shard_manager = client.shard_manager.clone();

    // Ctrl+C Handler
    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Error registering Ctrl+C handler");
        info!("Recieved Ctrl+C, shutting down");
        shard_manager.lock().await.shutdown_all().await;
    });

    // Start client
    if let Err(err) = client.start().await {
        error!("Client error: {:?}", err);
    };
}
