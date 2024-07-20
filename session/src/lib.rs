#![no_std]
use gstd::{collections::BTreeMap, exec, msg, prelude::*, ActorId, MessageId};
use session_io::*;
use wordle_io::{Action as WordleAction, Event as WordleEvent};

static mut SESSION: Option<Session> = None;

const DELAY_CHECK_STATUS_DURATION: u32 = 200;

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
        // Check if a game already exists for the user
        if self.players.get(&user).is_some() {
            assert!(
                msg::source() == exec::program_id(),
                "Game already exists for this user"
            );

            msg::reply(Event::GameStarted { user }, 0).expect("Error in sending reply");
            return;
        }

        // Send `StartGame` message to Wordle program
        let sent_msg_id = msg::send(self.target_program_id, WordleAction::StartGame { user }, 0)
            .expect("Error in sending message");

        // Wait for the response
        exec::wait();

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

        // Notify that the game has been successfully started
        msg::reply(Event::MessageSent, 0).expect("Error in sending a reply");
    }

    pub fn check_word(&mut self, user: ActorId, word: String) {
        // Ensure the game exists and is in correct status

        // Validate the submitted word is in lowercase and is 5 character long

        // Send `CheckWord` message to wordle program

        // Wait for the response

        // Notify that the move was successful
    }

    pub fn set_game_status(&mut self, user: &ActorId, status: GameStatus) {
        self.players.entry(*user).and_modify(|info| {
            info.game_status = status;
        });
    }

    pub fn get_info(&self, user: &ActorId) -> PlayerInfo {
        let info = self
            .players
            .get(user)
            .expect("Game does not exist for the player");
        info.clone()
    }
}

#[no_mangle]
extern "C" fn init() {
    let target_program_id = msg::load().expect("Could not decode target program ID");
    unsafe { SESSION = Some(Session::new(target_program_id)) }
}

#[no_mangle]
extern "C" fn handle() {
    let action = msg::load::<Action>().expect("Invalid action payload");
    let session = unsafe { SESSION.as_mut().expect("Session is not initialized") };

    match action {
        Action::StartGame => session.start_game(msg::source()),
        Action::CheckWord { word } => session.check_word(msg::source(), word),
        Action::CheckGameStatus { user } => todo!(),
    }
}

#[no_mangle]
extern "C" fn handle_reply() {
    let reply_message_id = msg::reply_to().expect("Failed to query reply_to data");

    let session = unsafe { SESSION.as_mut().expect("The session is not initialized") };

    let reply_message = msg::load::<WordleEvent>().expect("Unable to decode WordleEvent");

    let user: ActorId = reply_message.clone().into();

    let player_info = session.get_info(&user);
    let sent_message_id = player_info.sent_msg_id();
    let original_message_id = player_info.original_msg_id();

    if reply_message_id == sent_message_id {
        let game_status: GameStatus = reply_message.into();

        session.set_game_status(&user, game_status);

        exec::wake(original_message_id).expect("Failed to wake message");
    }
}

#[no_mangle]
extern "C" fn state() {
    let session = unsafe { SESSION.take().expect("Unititialized Session state") };
    msg::reply::<State>(session.into(), 0).expect("Failed to share state");
}

impl From<Session> for State {
    fn from(value: Session) -> Self {
        Self {
            target_program_id: value.target_program_id,
            players: value.players.clone(),
        }
    }
}
