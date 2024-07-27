#![no_std]
use gstd::{collections::BTreeMap, exec, msg, prelude::*, ActorId};
use ops::Not;
use session_io::*;
use wordle_io::{Action as WordleAction, Event as WordleEvent};

#[macro_use]
mod macros;

const DELAY_CHECK_STATUS_DURATION: u32 = 200;
const MAX_ATTEMPTS: u32 = 5;

create_inner_state!(SESSION, Session);

struct Session {
    pub target_program_id: ActorId,
    pub players: BTreeMap<ActorId, PlayerInfo>,
}

impl Session {
    pub fn new(target_program_id: ActorId) -> Self {
        Self {
            target_program_id,
            players: BTreeMap::new(),
        }
    }

    pub fn start_game(&mut self, user: ActorId) {
        if let Some(player) = self.players.get_mut(&user) {
            // ensure the game is not progressing
            assert!(
                matches!(
                    player.game_status,
                    GameStatus::Started | GameStatus::Completed(..)
                ),
                "A game is in progress for this user"
            );

            if player.game_status == GameStatus::Started {
                return Self::set_status_and_reply(
                    player,
                    GameStatus::InProgress,
                    Event::GameStarted,
                );
            }
        }

        // Send `StartGame` message to Wordle program
        let sent_msg_id = msg::send(self.target_program_id, WordleAction::StartGame { user }, 0)
            .expect("Error in sending message");
        let original_msg_id = msg::id();

        self.players
            .insert(user, PlayerInfo::new(sent_msg_id, original_msg_id));

        // Send a delayed message with `CheckGameStatus` action to monitor game's progress
        msg::send_delayed(
            exec::program_id(),
            Action::CheckGameStatus { user },
            0,
            DELAY_CHECK_STATUS_DURATION,
        )
        .expect("Error in sending delayed message");

        // Wait for the response
        exec::wait();
    }

    pub fn check_word(&mut self, user: ActorId, word: String) {
        let player = self
            .players
            .get_mut(&user)
            .expect("Game does not exist for the user");

        // Ensure the game exists and is in correct status
        assert!(
            matches!(
                player.game_status,
                GameStatus::Starting | GameStatus::Started | GameStatus::Completed(..)
            )
            .not(),
            "Game is not available to play"
        );

        if let GameStatus::WordChecked {
            correct_positions,
            contained_in_word,
            is_guessed,
        } = player.game_status.clone()
        {
            return Self::handle_word_checked(
                player,
                correct_positions,
                contained_in_word,
                is_guessed,
            );
        }

        // Validate the submitted word is in lowercase and is 5 character long
        assert!(
            word.len() == wordle_io::WORD_LENGTH,
            "Word must be 5 character long"
        );
        assert!(
            word.chars().all(|c| c.is_lowercase()),
            "Word must be lowercased"
        );

        // Send `CheckWord` message to wordle program
        let sent_msg_id = msg::send(
            self.target_program_id,
            WordleAction::CheckWord { user, word },
            0,
        )
        .expect("Error in sending message");

        player.set_msg_ids(sent_msg_id, msg::id());
        player.game_status = GameStatus::CheckingWord;

        exec::wait();
    }

    fn handle_word_checked(
        player_info: &mut PlayerInfo,
        correct_positions: Vec<u8>,
        contained_in_word: Vec<u8>,
        is_guessed: bool,
    ) {
        player_info.increment_attempt();

        if is_guessed {
            return Session::complete_game(player_info, GameOverStatus::Win);
        }

        if player_info.attempts_count == MAX_ATTEMPTS {
            return Session::complete_game(player_info, GameOverStatus::Lose);
        }

        Self::set_status_and_reply(
            player_info,
            GameStatus::InProgress,
            Event::WordChecked {
                correct_positions,
                contained_in_word,
            },
        )
    }

    fn complete_game(info: &mut PlayerInfo, status: GameOverStatus) {
        Self::set_status_and_reply(
            info,
            GameStatus::Completed(status.clone()),
            Event::GameOver(status),
        )
    }

    fn set_status_and_reply(info: &mut PlayerInfo, status: GameStatus, event: Event) {
        info.game_status = status;
        reply!(event)
    }
}

#[no_mangle]
extern "C" fn init() {
    let target_program_id = msg::load().expect("Could not decode target program ID");
    unsafe { init_inner_state(Session::new(target_program_id)) }
}

#[no_mangle]
extern "C" fn handle() {
    let action = msg::load::<Action>().expect("Invalid action payload");
    let session = get_inner_state_mut();

    match action {
        Action::StartGame => session.start_game(msg::source()),
        Action::CheckWord { word } => session.check_word(msg::source(), word),
        Action::CheckGameStatus { user } => todo!(),
    }
}

#[no_mangle]
extern "C" fn handle_reply() {
    let reply_message_id = msg::reply_to().expect("Failed to query reply_to data");

    let session = get_inner_state_mut();

    let reply_message = msg::load::<WordleEvent>().expect("Unable to decode WordleEvent");

    let user: ActorId = reply_message.clone().into();

    let player_info = session
        .players
        .get(&user)
        .expect("Game does not exist for the player");

    let sent_message_id = player_info.sent_msg_id();
    let original_message_id = player_info.original_msg_id();

    if reply_message_id == sent_message_id {
        let game_status: GameStatus = reply_message.into();
        session.players.entry(user).and_modify(|info| {
            info.game_status = game_status;
        });

        exec::wake(original_message_id).expect("Failed to wake message");
    }
}

#[no_mangle]
extern "C" fn state() {
    let state: State = get_inner_state().into();
    reply!(state)
}

impl From<Session> for State {
    fn from(value: Session) -> Self {
        Self {
            target_program_id: value.target_program_id,
            players: value.players.clone(),
        }
    }
}
