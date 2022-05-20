use std::collections::HashMap;
use std::default;

use serenity::builder::{CreateInteractionResponseData, CreateComponents};
use serenity::model::channel::ReactionType;
use serenity::prelude::*;
use serenity::model::interactions::{
    InteractionResponseType,
    application_command::ApplicationCommandInteraction,
    message_component::MessageComponentInteraction,
    message_component::ButtonStyle,
};

use tracing::info;

use crate::Error;

pub enum MinesweeperCell {
    Safe,
    Checked,
    Bomb,
}

pub struct MinesweeperGames;

impl TypeMapKey for MinesweeperGames {
    type Value = HashMap<String, Vec<MinesweeperCell>>;
}

fn create_game(mines: u8) -> Vec<MinesweeperCell> {
    vec![
        MinesweeperCell::Safe, MinesweeperCell::Safe, MinesweeperCell::Safe, MinesweeperCell::Safe, MinesweeperCell::Safe,
        MinesweeperCell::Safe, MinesweeperCell::Safe, MinesweeperCell::Safe, MinesweeperCell::Safe, MinesweeperCell::Safe,
        MinesweeperCell::Safe, MinesweeperCell::Safe, MinesweeperCell::Bomb, MinesweeperCell::Safe, MinesweeperCell::Safe,
        MinesweeperCell::Safe, MinesweeperCell::Safe, MinesweeperCell::Safe, MinesweeperCell::Safe, MinesweeperCell::Safe,
        MinesweeperCell::Safe, MinesweeperCell::Safe, MinesweeperCell::Safe, MinesweeperCell::Safe, MinesweeperCell::Safe
        ]
}

fn create_board<'a, 'b>(board: &Vec<MinesweeperCell>, response_data: &'b mut CreateInteractionResponseData<'a>, id: String) -> &'b mut CreateInteractionResponseData<'a> {
    let size = 5;
    response_data.components(|components| {
        for y in 0..size {
            components.create_action_row(|row| {
                for x in 0..size {
                    row.create_button(|button| {
                        let index = y*size+x;
                        let cell = &board[index];
                        let emoji = match cell {
                            MinesweeperCell::Bomb => "💣",
                            MinesweeperCell::Safe => "🟦",
                            MinesweeperCell::Checked => "8️⃣",
                        };
                        button
                            .custom_id(format!("minesweeper-{}-{}", id, index))
                            .style(ButtonStyle::Secondary)
                            .emoji(ReactionType::Unicode(String::from(emoji)))
                    });
                }
                row
                
            });
        }
        components
    })
}

pub async fn minesweeper(ctx: Context, command: ApplicationCommandInteraction) -> Result<(), Error> {
    let mut game_data = ctx.data.write().await;
    let game_list = game_data.get_mut::<MinesweeperGames>().unwrap();
    game_list.insert(command.id.to_string(), create_game(3));

    command.create_interaction_response(&ctx.http, |response| {
        response
            .kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|response_data| {
                create_board(game_list.get(&command.id.to_string()).unwrap(), response_data, command.id.to_string())
            })
    })
    .await?;
    Ok(())
}

pub async fn minesweeper_button(ctx: Context, component: MessageComponentInteraction) -> Result<(), Error> {
    let component_id = component.data.custom_id.clone();
    let mut split = component_id.split('-');
    split.next();
    let game_id = split.next().ok_or::<Error>("Invalid component id format".into())?;
    let index = split.next().ok_or::<Error>("Invalid component id format".into())?.parse::<usize>()?;



    let mut game_data = ctx.data.write().await;
    let game_list = game_data.get_mut::<MinesweeperGames>().unwrap();
    match game_list.get_mut(game_id) {
        Some(board) => {
            board[index] = MinesweeperCell::Bomb;

            component.create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::UpdateMessage)
                    .interaction_response_data(|data| {
                        create_board(game_list.get(game_id).unwrap(), data, String::from(game_id))
                    })
            })
            .await?
        },
        None => {
            component.create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::UpdateMessage)
                    .interaction_response_data(|data| {
                        data
                            .content("This game has expired, create a new game with `/minesweeper`.")
                            .set_components(CreateComponents::default())
                    })
            })
            .await?;
        }
    };
    

    Ok(())
}