use serenity::prelude::*;
use serenity::model::interactions::application_command::ApplicationCommandInteraction;

use crate::Error;

pub async fn fail(_ctx: Context, command: ApplicationCommandInteraction) -> Result<(), Error> {
    Err(format!("Test error from {}", command.member.unwrap().display_name()).into())
}