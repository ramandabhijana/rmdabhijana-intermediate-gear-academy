#![no_std]
use consts::*;
use gstd::{collections::BTreeMap, exec, msg, prelude::*, ActorId, MessageId};
use session_io::*;
use wordle_io::{Action as WordleAction, Event as WordleEvent};

#[macro_use]
mod macros;
pub mod consts;

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
            assert!(!player.is_playing(), "{}", err_msgs::GAME_IS_PLAYING);

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
            .expect(err_msgs::SEND_FAILED);
        let original_msg_id = msg::id();

        self.players
            .insert(user, PlayerInfo::new(sent_msg_id, original_msg_id));

        // Send a delayed message with `CheckGameStatus` action to monitor game's progress
        msg::send_delayed(
            exec::program_id(),
            Action::CheckGameStatus {
                user,
                init_id: original_msg_id,
            },
            0,
            game_rules::DELAY_CHECK_STATUS_DURATION,
        )
        .expect(err_msgs::SEND_DELAYED_FAILED);

        // Wait for the response
        exec::wait();
    }

    pub fn check_word(&mut self, user: ActorId, word: String) {
        let player = self.players.get_mut(&user).expect(err_msgs::GAME_NOT_FOUND);

        // Ensure the game exists and is in correct status
        assert!(player.is_playing(), "{}", err_msgs::GAME_NOT_PLAYABLE);

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
            "{}",
            err_msgs::INVALID_WORD_LEN
        );
        assert!(
            word.chars().all(|c| c.is_lowercase()),
            "{}",
            err_msgs::INVALID_WORD_CASE
        );

        // Send `CheckWord` message to wordle program
        let sent_msg_id = msg::send(
            self.target_program_id,
            WordleAction::CheckWord { user, word },
            0,
        )
        .expect(err_msgs::SEND_FAILED);

        player.set_msg_ids(sent_msg_id, msg::id());
        player.game_status = GameStatus::CheckingWord;

        exec::wait();
    }

    pub fn check_game_status(&mut self, user: ActorId, init_id: MessageId) {
        assert!(
            msg::source() == exec::program_id(),
            "{}",
            err_msgs::PROGRAM_ONLY
        );

        let info = self
            .players
            .get_mut(&user)
            .expect(err_msgs::PLAYER_INFO_NOT_FOUND);

        if let GameStatus::Completed(..) = info.game_status {
            // ignore when game has ended
            return;
        }

        if init_id == info.init_msg_id {
            let game_over_status = GameOverStatus::Lose;
            info.game_status = GameStatus::Completed(game_over_status.clone());
            msg::send(user, Event::GameOver(game_over_status), 0).expect(err_msgs::SEND_FAILED);
        }
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

        if player_info.attempts_count == game_rules::MAX_ATTEMPTS {
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
    let target_program_id = msg::load().expect(err_msgs::LOAD_FAILED);
    unsafe { init_inner_state(Session::new(target_program_id)) }
}

#[no_mangle]
extern "C" fn handle() {
    let action = msg::load::<Action>().expect(err_msgs::LOAD_FAILED);
    let session = get_inner_state_mut();

    match action {
        Action::StartGame => session.start_game(msg::source()),
        Action::CheckWord { word } => session.check_word(msg::source(), word),
        Action::CheckGameStatus { user, init_id } => session.check_game_status(user, init_id),
    }
}

#[no_mangle]
extern "C" fn handle_reply() {
    let reply_message_id = msg::reply_to().expect(err_msgs::READ_REPLY_FAILED);

    let session = get_inner_state_mut();

    let reply_message = msg::load::<WordleEvent>().expect(err_msgs::LOAD_FAILED);

    let user: ActorId = reply_message.clone().into();

    let player_info = session
        .players
        .get(&user)
        .expect(err_msgs::PLAYER_INFO_NOT_FOUND);

    let sent_message_id = player_info.sent_msg_id();
    let original_message_id = player_info.original_msg_id();

    if reply_message_id == sent_message_id {
        let game_status: GameStatus = reply_message.into();
        session.players.entry(user).and_modify(|info| {
            info.game_status = game_status;
        });

        exec::wake(original_message_id).expect(err_msgs::RESUME_FAILED);
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
