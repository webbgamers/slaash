use serenity::model::interactions::{
    application_command::ApplicationCommandInteraction,
};
use serenity::prelude::*;

use tracing::info;

use crate::Error;



pub async fn blackjack(_ctx: Context, _command: ApplicationCommandInteraction) -> Result<(), Error> {
    info!("Blackjack!");
    Ok(())
}
