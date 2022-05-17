use serenity::prelude::*;
use serenity::model::interactions::{
    InteractionResponseType,
    application_command::ApplicationCommandInteraction
};

use tracing::info;

use crate::Error;


pub async fn ping(ctx: Context, command: ApplicationCommandInteraction) -> Result<(), Error> {
    command.create_interaction_response(&ctx.http, |response| {
        response
            .kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|message| message.content("Pong!"))
    })
    .await?;
    
    info!("Ping from {}.", command.member.unwrap().display_name());
    Ok(())
}