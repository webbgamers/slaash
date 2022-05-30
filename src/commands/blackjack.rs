use std::collections::HashMap;

use serenity::model::{
    id::UserId,
    interactions::{
        application_command::ApplicationCommandInteraction,
        message_component::{ButtonStyle, MessageComponentInteraction},
    },
};
use serenity::prelude::*;

use tracing::info;

use crate::Error;

pub struct BlackjackGame {
    players: Vec<UserId>,
}

pub struct BlackjackGames;

impl TypeMapKey for BlackjackGames {
    type Value = HashMap<String, BlackjackGame>;
}

pub async fn blackjack(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> Result<(), Error> {
    let name = command.user.mention();

    command
        .create_interaction_response(&ctx.http, |response| {
            response.interaction_response_data(|data| {
                data.content(format!(
                    "{} has started a game of blackjack! Who would like to play?",
                    name
                ))
                .allowed_mentions(|mentions| mentions.empty_users())
                .components(|components| {
                    components.create_action_row(|row| {
                        row.create_button(|button| {
                            button
                                .label("Join")
                                .custom_id("blackjack-join")
                                .style(ButtonStyle::Success)
                        })
                        .create_button(|button| {
                            button
                                .label("Start")
                                .custom_id("blackjack-start")
                                .style(ButtonStyle::Secondary)
                                .disabled(true)
                        })
                    })
                })
            })
        })
        .await?;
    Ok(())
}

pub async fn blackjack_button(
    _ctx: &Context,
    _component: &MessageComponentInteraction,
) -> Result<(), Error> {
    Err(Error::from("Blackjack button."))
}
