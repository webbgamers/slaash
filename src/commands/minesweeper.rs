use serenity::prelude::*;
use serenity::model::interactions::{
    InteractionResponseType,
    application_command::ApplicationCommandInteraction
};

use crate::Error;

pub async fn minesweeper(ctx: Context, command: ApplicationCommandInteraction) -> Result<(), Error> {
    command.create_interaction_response(&ctx.http, |response| {
        response
            .kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|message| message.content("Sweep mines."))
    })
    .await?;
    Ok(())
}