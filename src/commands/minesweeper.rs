use std::time::Instant;
use std::collections::HashMap;

use rand::seq::SliceRandom;

use serenity::builder::{CreateInteractionResponseData, CreateComponents};
use serenity::model::channel::ReactionType;
use serenity::prelude::*;
use serenity::model::interactions::{
    InteractionResponseType,
    application_command::ApplicationCommandInteraction,
    message_component::MessageComponentInteraction,
    message_component::ButtonStyle,
    application_command::ApplicationCommandInteractionDataOptionValue,
};
use serenity::model::id::UserId;

use tracing::{info, warn};

use crate::Error;

#[derive(Debug, Clone)]
pub enum MinesweeperCell {
    Safe,
    Checked,
    Bomb,
}

#[derive(Debug)]
pub struct MinesweeperGame {
    player: UserId,
    start_time: Instant,
    board: Vec<MinesweeperCell>,
}

#[derive(Debug)]
pub struct MinesweeperGames;

impl TypeMapKey for MinesweeperGames {
    type Value = HashMap<String, MinesweeperGame>;
}

fn create_game(mines: usize, player: UserId) -> MinesweeperGame {
    let mut rng = &mut rand::thread_rng();
    let mut board = vec![MinesweeperCell::Safe; 25];
    let bombs = 0..25_usize;
    let bombs = bombs.collect::<Vec<_>>();
    let bombs = bombs.choose_multiple(rng, mines).cloned();
    for b in bombs {
        board[b] = MinesweeperCell::Bomb;
    }

    MinesweeperGame {
    player,
    start_time: Instant::now(),
    board,
    }
}

fn render_board<'a, 'b>(game: &MinesweeperGame, response_data: &'b mut CreateInteractionResponseData<'a>, id: String, selected_cell_index: Option<usize>, game_over: bool) -> &'b mut CreateInteractionResponseData<'a> {
    let board = &game.board;
    let size = 5;
    response_data.components(|components| {
        for y in 0..size {
            components.create_action_row(|row| {
                for x in 0..size {
                    row.create_button(|button| {
                        let index = y*size+x;
                        let cell = &board[index];
                        let (emoji, cell_style, disabled) = match cell {
                            MinesweeperCell::Bomb => (if game_over { String::from("\u{1F4A3}") } else { String::from("\u{1F7E6}") }, ButtonStyle::Danger, game_over),
                            MinesweeperCell::Safe => (if game_over { number_to_emoji(count_adjacent_bombs(&board, index)) } else { String::from("\u{1F7E6}") }, ButtonStyle::Secondary, game_over),
                            MinesweeperCell::Checked => (number_to_emoji(count_adjacent_bombs(&board, index)), ButtonStyle::Success, true),
                        };

                        let mut style = ButtonStyle::Secondary;

                        if game_over && cell_style == ButtonStyle::Success {
                            style = cell_style;
                        }

                        if let Some(cell) = selected_cell_index {
                            if index == cell {
                                style = cell_style;
                            }
                        };

                        button
                            .custom_id(format!("minesweeper-{}-{}", id, index))
                            .style(style)
                            .emoji(ReactionType::Unicode(emoji))
                            .disabled(disabled)
                    });
                }
                row

            });
        }
        
        components
    });

    if game_over {
        let safes = board.iter().filter(|&c| if let MinesweeperCell::Safe = c { true } else { false } ).count();
        let bombs = board.iter().filter(|&c| if let MinesweeperCell::Bomb = c { true } else { false } ).count();

        if safes == 0 {
            response_data.content(format!("**You win!**\nBombs: {}\nTime: {}s", bombs, game.start_time.elapsed().as_secs()));
        }
        else {
            let checked = board.iter().filter(|&c| if let MinesweeperCell::Checked = c { true } else { false } ).count();
            response_data.content(format!("**Game over.**\nBombs: {}\nCells cleared: {}\nTime: {}s", bombs, checked, game.start_time.elapsed().as_secs()));
        }

        
    }
    response_data
}

fn number_to_emoji(number: usize) -> String {
    let str = match number {
        0 => "\u{0030}\u{FE0F}\u{20E3}",
        1 => "\u{0031}\u{FE0F}\u{20E3}",
        2 => "\u{0032}\u{FE0F}\u{20E3}",
        3 => "\u{0033}\u{FE0F}\u{20E3}",
        4 => "\u{0034}\u{FE0F}\u{20E3}",
        5 => "\u{0035}\u{FE0F}\u{20E3}",
        6 => "\u{0036}\u{FE0F}\u{20E3}",
        7 => "\u{0037}\u{FE0F}\u{20E3}",
        8 => "\u{0038}\u{FE0F}\u{20E3}",
        _ => "\u{0023}\u{FE0F}\u{20E3}",
    };
    String::from(str)
}

fn count_adjacent_bombs(board: &Vec<MinesweeperCell>, index: usize) -> usize {
    let size = 5;
    let mut count = 0;
    let iy = index/size;
    let ix = index%size;

    for y in iy as isize-1..iy as isize+2 {
        if y < 0 || y > size as isize - 1 { continue; }
        for x in ix as isize-1..ix as isize+2 {
            if x < 0 || x > size as isize - 1 { continue; }
            if x == ix as isize && y == iy as isize { continue; }
            
            if let MinesweeperCell::Bomb = board[y as usize * size + x as usize] { count += 1; }
        }
    };




    count
}

pub async fn minesweeper(ctx: Context, command: ApplicationCommandInteraction) -> Result<(), Error> {
    let mut bombs = 3;
    if let Some(option) = command.data.options.get(0) {
        if let Some(value) = option.resolved.as_ref() {
            if let ApplicationCommandInteractionDataOptionValue::Integer(count) = value {
                bombs = *count as usize;
            }
        }
    }
    let mut game_data = ctx.data.write().await;
    let game_list = game_data.get_mut::<MinesweeperGames>().unwrap();
    game_list.insert(command.id.to_string(), create_game(bombs, command.user.id));

    command.create_interaction_response(&ctx.http, |response| {
        response
            .kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|response_data| {
                render_board(&game_list.get(&command.id.to_string()).unwrap(), response_data, command.id.to_string(), None, false)
            })
    })
    .await?;
    Ok(())
}

pub async fn minesweeper_button(ctx: Context, component: MessageComponentInteraction) -> Result<(), Error> {
    let component_id = component.data.custom_id.clone();
    let mut split = component_id.split('-');
    split.next();
    let game_id = split.next().ok_or::<Error>("Missing game id in component custom id".into())?;
    let index = split.next().ok_or::<Error>("Missing cell index in component custom id".into())?.parse::<usize>()?;



    let mut game_data = ctx.data.write().await;
    let game_list = game_data.get_mut::<MinesweeperGames>().unwrap();
    match game_list.get_mut(game_id) {
        Some(game) => {
            if component.user.id == game.player { 
                let board = &mut game.board;
                let selected_cell = &board[index];
                let mut game_over = false;

                match selected_cell {
                    MinesweeperCell::Bomb => game_over = true,
                    MinesweeperCell::Safe => {
                        board[index] = MinesweeperCell::Checked;
                        if let None = board.iter().find(|&c| if let MinesweeperCell::Safe = c { true } else { false }) {
                            game_over = true;
                        }
                    }
                    MinesweeperCell::Checked => { warn!("Checked cell selected which should be disabled."); }
                }

                component.create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::UpdateMessage)
                        .interaction_response_data(|data| {
                            render_board(&game_list.get(game_id).unwrap(), data, String::from(game_id), Some(index), game_over)
                        })
                })
                .await?;
                if game_over {
                    game_list.remove(game_id);
                }
            }
            else {
                component.create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|data| {
                            data
                                .ephemeral(true)
                                .content("Thats not your game! Create your own with `/minesweeper`.")
                        })
                })
                .await?;
            }
        },
        None => {
            component.create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::UpdateMessage)
                    .interaction_response_data(|data| {
                        data
                            .content("This game has expired, start a new one with `/minesweeper`.")
                            .set_components(CreateComponents::default())
                    })
            })
            .await?;
        }
    };
    

    Ok(())
}
