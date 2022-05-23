use std::collections::{HashMap, HashSet};
use std::iter::repeat_with;
use std::time::Instant;

use rand::seq::SliceRandom;

use serenity::builder::CreateComponents;
use serenity::model::channel::ReactionType;
use serenity::model::id::UserId;
use serenity::model::interactions::{
    application_command::ApplicationCommandInteraction,
    application_command::ApplicationCommandInteractionDataOptionValue,
    message_component::ButtonStyle, message_component::MessageComponentInteraction,
    InteractionResponseType,
};
use serenity::prelude::*;

use tracing::warn;

use crate::Error;

pub enum MinesweeperCell {
    Safe,
    Checked,
    Bomb,
}

pub struct MinesweeperGame {
    player: UserId,
    mines: usize,
    start_time: Option<Instant>,
    board: Option<Vec<MinesweeperCell>>,
}

impl MinesweeperGame {
    fn new(player: UserId, mines: usize) -> MinesweeperGame {
        MinesweeperGame {
            player,
            mines,
            start_time: None,
            board: None,
        }
    }

    fn start_game(&mut self, safe_cell_index: usize) {
        let rng = &mut rand::thread_rng();
        let mut board = repeat_with(|| MinesweeperCell::Safe)
            .take(25)
            .collect::<Vec<_>>();
        let bombs = (0..safe_cell_index)
            .chain((safe_cell_index + 1)..25)
            .collect::<Vec<_>>();
        let bombs = bombs.choose_multiple(rng, self.mines).cloned();
        for b in bombs {
            board[b] = MinesweeperCell::Bomb;
        }
        self.start_time = Some(Instant::now());
        self.board = Some(board);
    }
}

pub struct MinesweeperGames;

impl TypeMapKey for MinesweeperGames {
    type Value = HashMap<String, MinesweeperGame>;
}

fn render_board(
    game: &MinesweeperGame,
    id: String,
    selected_cells: &Option<Vec<usize>>,
    game_over: bool,
) -> CreateComponents {
    let mut components = CreateComponents::default();

    if let Some(board) = &game.board {
        for y in 0..5 {
            components.create_action_row(|row| {
                for x in 0..5 {
                    row.create_button(|button| {
                        let index = y * 5 + x;
                        let cell = &board[index];
                        let (emoji, cell_style, disabled) = match cell {
                            MinesweeperCell::Bomb => (
                                if game_over {
                                    String::from("\u{1F4A3}")
                                } else {
                                    String::from("\u{1F7E6}")
                                },
                                ButtonStyle::Danger,
                                game_over,
                            ),
                            MinesweeperCell::Safe => (
                                if game_over {
                                    number_to_emoji(count_adjacent_bombs(board, index))
                                } else {
                                    String::from("\u{1F7E6}")
                                },
                                ButtonStyle::Secondary,
                                game_over,
                            ),
                            MinesweeperCell::Checked => (
                                number_to_emoji(count_adjacent_bombs(board, index)),
                                ButtonStyle::Success,
                                true,
                            ),
                        };

                        let mut style = ButtonStyle::Secondary;

                        if game_over && cell_style == ButtonStyle::Success {
                            style = cell_style;
                        }

                        if let Some(cells) = selected_cells {
                            if cells.iter().any(|&c| c == index) {
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
    } else {
        for y in 0..5 {
            components.create_action_row(|row| {
                for x in 0..5 {
                    row.create_button(|button| {
                        let index = y * 5 + x;
                        button
                            .custom_id(format!("minesweeper-{}-{}", id, index))
                            .style(ButtonStyle::Secondary)
                            .emoji(ReactionType::Unicode(String::from("\u{1F7E6}")))
                    });
                }
                row
            });
        }
    }

    components
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

fn get_adjacent_indexes(index: usize) -> Vec<usize> {
    let mut cells = Vec::new();

    let iy = index / 5;
    let ix = index % 5;

    for y in iy as isize - 1..iy as isize + 2 {
        if !(0..=5_isize - 1).contains(&y) {
            continue;
        }
        for x in ix as isize - 1..ix as isize + 2 {
            if !(0..=5_isize - 1).contains(&x) {
                continue;
            }
            if x == ix as isize && y == iy as isize {
                continue;
            }

            cells.push(y as usize * 5 + x as usize);
        }
    }

    cells
}

fn count_adjacent_bombs(board: &[MinesweeperCell], index: usize) -> usize {
    get_adjacent_indexes(index)
        .iter()
        .filter(|&&c| matches!(board[c], MinesweeperCell::Bomb))
        .count()
}

fn zero_fill(board: &Vec<MinesweeperCell>, mut set: HashSet<usize>) -> HashSet<usize> {
    let prev = set.clone();
    for s in prev.iter() {
        if count_adjacent_bombs(board, *s) == 0 {
            set.extend(get_adjacent_indexes(*s).iter());
            set = set
                .into_iter()
                .filter(|&c| !matches!(board[c], MinesweeperCell::Checked))
                .collect();
        }
    }
    if prev == set {
        return set;
    }

    zero_fill(board, set)
}

pub async fn minesweeper(
    ctx: Context,
    command: ApplicationCommandInteraction,
) -> Result<(), Error> {
    let mut bombs = 3;
    if let Some(option) = command.data.options.get(0) {
        if let Some(ApplicationCommandInteractionDataOptionValue::Integer(count)) = &option.resolved
        {
            bombs = *count as usize;
        }
    }
    let mut game_data = ctx.data.write().await;
    let game_list = game_data.get_mut::<MinesweeperGames>().unwrap();
    game_list.insert(
        command.id.to_string(),
        MinesweeperGame::new(command.user.id, bombs),
    );

    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|response_data| {
                    response_data.set_components(render_board(
                        game_list.get(&command.id.to_string()).unwrap(),
                        command.id.to_string(),
                        &None,
                        false,
                    ))
                })
        })
        .await?;
    Ok(())
}

pub async fn minesweeper_button(
    ctx: Context,
    component: MessageComponentInteraction,
) -> Result<(), Error> {
    let component_id = component.data.custom_id.clone();
    let mut split = component_id.split('-');
    split.next();
    let game_id = split
        .next()
        .ok_or("Missing game id in component custom id")?;
    let index = split
        .next()
        .ok_or("Missing cell index in component custom id")?
        .parse::<usize>()?;

    let mut game_data = ctx.data.write().await;
    let game_list = game_data.get_mut::<MinesweeperGames>().unwrap();
    match game_list.get_mut(game_id) {
        Some(game) => {
            if component.user.id == game.player {
                if game.board.is_none() {
                    game.start_game(index);
                }

                let mut game_over = false;
                let mut selected_cells = vec![index];

                {
                    let board = &mut game.board.as_mut().unwrap();

                    let selected_cell = &board[index];

                    match selected_cell {
                        MinesweeperCell::Bomb => game_over = true,
                        MinesweeperCell::Safe => {
                            let mut set = HashSet::new();
                            set.insert(index);
                            let fill = zero_fill(board, set);
                            for f in &fill {
                                board[*f] = MinesweeperCell::Checked;
                            }
                            selected_cells = fill.into_iter().collect::<Vec<_>>();

                            if !board.iter().any(|c| matches!(c, MinesweeperCell::Safe)) {
                                game_over = true;
                            }
                        }
                        MinesweeperCell::Checked => {
                            warn!("Checked cell selected which should be disabled.");
                        }
                    }
                }

                component
                    .create_interaction_response(&ctx.http, |response| {
                        response
                            .kind(InteractionResponseType::UpdateMessage)
                            .interaction_response_data(|data| {
                                if game_over {
                                    let safes = game
                                        .board
                                        .as_ref()
                                        .unwrap()
                                        .iter()
                                        .filter(|&c| matches!(c, MinesweeperCell::Safe))
                                        .count();
                                    let bombs = game.mines;

                                    if safes == 0 {
                                        data.content(format!(
                                            "**You win!**\nBombs: {}\nTime: {}s",
                                            bombs,
                                            game.start_time.unwrap().elapsed().as_secs()
                                        ));
                                    } else {
                                        data.content(format!(
                                            "**Game over.**\nBombs: {}\nCleared: {}/{}\nTime: {}s",
                                            bombs,
                                            25 - safes - bombs,
                                            25 - bombs,
                                            game.start_time.unwrap().elapsed().as_secs()
                                        ));
                                    }
                                }

                                data.set_components(render_board(
                                    game,
                                    String::from(game_id),
                                    &Some(selected_cells),
                                    game_over,
                                ))
                            })
                    })
                    .await?;
                if game_over {
                    game_list.remove(game_id);
                }
            } else {
                component
                    .create_interaction_response(&ctx.http, |response| {
                        response
                            .kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|data| {
                                data.ephemeral(true).content(
                                    "Thats not your game! Create your own with `/minesweeper`.",
                                )
                            })
                    })
                    .await?;
            }
        }
        None => {
            component
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::UpdateMessage)
                        .interaction_response_data(|data| {
                            data.content(
                                "This game has expired, start a new one with `/minesweeper`.",
                            )
                            .set_components(CreateComponents::default())
                        })
                })
                .await?;
        }
    };

    Ok(())
}
