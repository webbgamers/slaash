use std::collections::HashMap;
use std::iter::repeat_with;
use std::time::Instant;

use serenity::model::interactions::application_command::ApplicationCommandInteractionDataOptionValue;
use serenity::prelude::*;
use serenity::{
    builder::CreateComponents,
    model::{
        channel::ReactionType,
        id::{InteractionId, UserId},
        interactions::{
            application_command::ApplicationCommandInteraction,
            message_component::{ButtonStyle, MessageComponentInteraction},
            InteractionResponseType,
        },
    },
};

use crate::Error;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TictactoeCell {
    X,
    O,
}

impl TictactoeCell {
    fn render(&self) -> String {
        String::from(match &self {
            TictactoeCell::X => "\u{274C}",
            TictactoeCell::O => "\u{2B55}",
        })
    }
}

#[derive(Clone)]
pub struct TictactoeGame {
    player1: UserId,
    player2: Option<UserId>,
    size: usize,
    board: Vec<Option<TictactoeCell>>,
    turn: TictactoeCell,
    start_time: Option<Instant>,
}

impl TictactoeGame {
    fn new(player: UserId, size: usize) -> TictactoeGame {
        TictactoeGame {
            player1: player,
            player2: None,
            size,
            board: repeat_with(|| None).take(size * size).collect::<Vec<_>>(),
            turn: TictactoeCell::X,
            start_time: None,
        }
    }

    fn start(&mut self, player: UserId) {
        self.player2 = Some(player);
        self.start_time = Some(Instant::now())
    }

    fn check_win(&self, cell_index: usize) -> Option<Vec<usize>> {
        fn reverse(num: &usize, range: usize) -> usize {
            (-(*num as isize) + range as isize - 1) as usize
        }

        let cell_x = cell_index % self.size;
        let cell_y = cell_index / self.size;

        // TODO: reduce code duplication

        // Rows
        {
            let mut indexes = (0..self.size).map(|x| cell_y * self.size + x);
            let first = indexes.next().unwrap();
            if indexes.clone().all(|i| self.board[i] == self.board[first]) {
                let mut index_list = vec![first];
                index_list.extend(indexes);
                return Some(index_list);
            }
        }
        // Columns
        {
            let mut indexes = (0..self.size).map(|y| y * self.size + cell_x);
            let first = indexes.next().unwrap();
            if indexes.clone().all(|i| self.board[i] == self.board[first]) {
                let mut index_list = vec![first];
                index_list.extend(indexes);
                return Some(index_list);
            }
        }
        // Diagonal 1
        {
            if cell_x == cell_y {
                let mut indexes = (0..self.size).map(|i| i * self.size + i);
                let first = indexes.next().unwrap();
                if indexes.clone().all(|i| self.board[i] == self.board[first]) {
                    let mut index_list = vec![first];
                    index_list.extend(indexes);
                    return Some(index_list);
                }
            }
        }
        // Diagonal 2
        {
            let rev_x = reverse(&cell_x, self.size);
            if rev_x == cell_y {
                let mut indexes = (0..self.size).map(|i| i * self.size + reverse(&i, self.size));
                let first = indexes.next().unwrap();
                if indexes.clone().all(|i| self.board[i] == self.board[first]) {
                    let mut index_list = vec![first];
                    index_list.extend(indexes);
                    return Some(index_list);
                }
            }
        }
        None
    }

    fn current_player(&self) -> Option<UserId> {
        match self.turn {
            TictactoeCell::X => Some(self.player1),
            TictactoeCell::O => self.player2,
        }
    }

    fn render_board(&self, highlight_cells: Vec<usize>) -> CreateComponents {
        let mut components = CreateComponents::default();

        let (highlight_style, game_over) = match highlight_cells.len() {
            0..=1 => (ButtonStyle::Primary, false),
            _ => (ButtonStyle::Success, true),
        };

        for y in 0..self.size {
            components.create_action_row(|row| {
                for x in 0..self.size {
                    row.create_button(|button| {
                        let index = y * self.size + x;
                        if self.board[index].is_some() {
                            button
                                .emoji(ReactionType::Unicode(
                                    self.board[index].as_ref().unwrap().render(),
                                ))
                                .disabled(true);
                        } else {
                            button.label(" ").disabled(game_over);
                        }

                        let style = if highlight_cells.iter().any(|&c| c == index) {
                            highlight_style
                        } else {
                            ButtonStyle::Secondary
                        };

                        button
                            .custom_id(format!("tictactoe-game-{}", index))
                            .style(style)
                    });
                }
                row
            });
        }

        components
    }
}

pub struct TictactoeGames;

impl TypeMapKey for TictactoeGames {
    type Value = HashMap<InteractionId, TictactoeGame>;
}

pub async fn tictactoe(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> Result<(), Error> {
    let mut game_data = ctx.data.write().await;
    let game_list = game_data.get_mut::<TictactoeGames>().unwrap();

    let mut size = 3;
    if let Some(option) = command.data.options.get(0) {
        if let Some(ApplicationCommandInteractionDataOptionValue::Integer(osize)) = &option.resolved
        {
            size = *osize as usize;
        }
    }

    game_list.insert(command.id, TictactoeGame::new(command.user.id, size));

    let name = command.user.mention();

    command
        .create_interaction_response(&ctx.http, |response| {
            response.interaction_response_data(|data| {
                data.content(format!(
                    "{} has started a game of tic-tac-toe! Who would like to play?",
                    name
                ))
                .allowed_mentions(|mentions| mentions.empty_users())
                .components(|components| {
                    components.create_action_row(|row| {
                        row.create_button(|button| {
                            button
                                .label("Join")
                                .custom_id("tictactoe-join")
                                .style(ButtonStyle::Success)
                        })
                    })
                })
            })
        })
        .await?;
    Ok(())
}

pub async fn tictactoe_button(
    ctx: &Context,
    component: &MessageComponentInteraction,
) -> Result<(), Error> {
    let component_id = component.data.custom_id.clone();
    let mut split = component_id.split('-');
    split.next();

    let mut game_data = ctx.data.write().await;
    let game_list = game_data.get_mut::<TictactoeGames>().unwrap();
    match game_list.get_mut(&component.message.interaction.as_ref().unwrap().id) {
        Some(game) => match split.next().ok_or("Missing button type")? {
            "join" => {
                if component.user.id
                    != component
                        .message
                        .interaction
                        .as_ref()
                        .ok_or("Couldnt find original interaction")?
                        .user
                        .id
                {
                    if game.player2.is_none() {
                        game.start(component.user.id);
                        component
                            .create_interaction_response(&ctx.http, |response| {
                                response
                                    .kind(InteractionResponseType::UpdateMessage)
                                    .interaction_response_data(|data| {
                                        data.content(format!("{}'s turn!", game.player1.mention()))
                                            .set_components(game.render_board(Vec::new()))
                                    })
                            })
                            .await?
                    } else {
                        component.create_interaction_response(&ctx.http, |response| {
                                response.interaction_response_data(|data| {
                                    data
                                        .content("Someone already joined this game! You can create your own with `/tictactoe`.")
                                        .ephemeral(true)
                                })
                            })
                            .await?
                    }
                } else {
                    component
                        .create_interaction_response(&ctx.http, |response| {
                            response.interaction_response_data(|data| {
                                data.content(
                                    "You can't join your own game! Find someone else to play with.",
                                )
                                .ephemeral(true)
                            })
                        })
                        .await?
                }
            }
            "game" => {
                let index = split
                    .next()
                    .ok_or("Missing cell index in component custom id")?
                    .parse::<usize>()?;

                if component.user.id == game.player1 || component.user.id == game.player2.unwrap() {
                    if component.user.id == game.current_player().unwrap() {
                        let mut temp_game = game.clone();

                        if temp_game.board[index].is_none() {
                            temp_game.board[index] = Some(temp_game.turn);
                        } else {
                            return Err(Error::from(
                                "Filled cell selected which should be disabled",
                            ));
                        }

                        let highlight_tiles =
                            temp_game.check_win(index).unwrap_or_else(|| vec![index]);

                        let won = highlight_tiles.len() > 1;
                        let tie = temp_game.board.iter().all(|&c| c.is_some());

                        temp_game.turn = match temp_game.turn {
                            TictactoeCell::X => TictactoeCell::O,
                            TictactoeCell::O => TictactoeCell::X,
                        };

                        if won {
                            component
                                .create_interaction_response(&ctx.http, |response| {
                                    response
                                        .kind(InteractionResponseType::UpdateMessage)
                                        .interaction_response_data(|data| {
                                            data.content(format!(
                                                "{} won!",
                                                game.current_player().unwrap().mention()
                                            ))
                                            .allowed_mentions(|mentions| mentions.empty_users())
                                            .set_components(temp_game.render_board(highlight_tiles))
                                        })
                                })
                                .await?;

                            game_list.remove(&component.message.interaction.as_ref().unwrap().id);
                        } else if tie {
                            component
                                .create_interaction_response(&ctx.http, |response| {
                                    response
                                        .kind(InteractionResponseType::UpdateMessage)
                                        .interaction_response_data(|data| {
                                            data.content("It's a tie!").set_components(
                                                temp_game.render_board(highlight_tiles),
                                            )
                                        })
                                })
                                .await?
                        } else {
                            component
                                .create_interaction_response(&ctx.http, |response| {
                                    response
                                        .kind(InteractionResponseType::UpdateMessage)
                                        .interaction_response_data(|data| {
                                            data.content(format!(
                                                "{}'s turn.",
                                                temp_game.current_player().unwrap().mention()
                                            ))
                                            .set_components(temp_game.render_board(highlight_tiles))
                                        })
                                })
                                .await?;

                            *game = temp_game;
                        }
                    } else {
                        component.create_interaction_response(&ctx.http, |response| {
                                response.interaction_response_data(|data| {
                                    data
                                        .content("It's not your turn! Wait for the other player to make a move.")
                                        .ephemeral(true)
                                })
                            })
                            .await?
                    }
                } else {
                    component
                        .create_interaction_response(&ctx.http, |response| {
                            response.interaction_response_data(|data| {
                                data.content(
                                    "That's not your game! Create your own with `/tictactoe`.",
                                )
                                .ephemeral(true)
                            })
                        })
                        .await?
                }
            }

            _ => return Err(Error::from("Unknown button type")),
        },
        None => {
            component
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::UpdateMessage)
                        .interaction_response_data(|data| {
                            data.content(
                                "This game has expired, start a new one with `/tictactoe`.",
                            )
                            .set_components(CreateComponents::default())
                        })
                })
                .await?;
        }
    };
    Ok(())
}
