use serenity::model::interactions::application_command::ApplicationCommandInteraction;
use serenity::prelude::*;

use crate::Error;

pub async fn fail(_ctx: &Context, command: &ApplicationCommandInteraction) -> Result<(), Error> {
    Err(Error::from(format!(
        "Test error from {}",
        command.member.as_ref().unwrap().display_name()
    )))
}
