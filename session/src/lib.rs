#![no_std]
use gstd::{collections::BTreeMap, exec, msg, prelude::*, ActorId, MessageId};
use session_io::*;

static mut SESSION: Option<Session> = None;

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

        // Send `StartGame` message to Wordle program

        // Wait for the response

        // Send a delayed message with `CheckGameStatus` action to monitor game's progress

        // Notify that the game has been successfully started
    }

    pub fn check_word(&mut self, user: ActorId, word: String) {
        // Ensure the game exists and is in correct status

        // Validate the submitted word is in lowercase and is 5 character long

        // Send `CheckWord` message to wordle program

        // Wait for the response

        // Notify that the move was successful
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
        Action::StartGame { user } => session.start_game(user),
        Action::CheckWord { user, word } => session.check_word(user, word),
        Action::CheckGameStatus => todo!(),
    }
}

#[no_mangle]
extern "C" fn handle_reply() {
    // which message was replied to
    let reply_message_id = msg::reply_to().expect("Failed to query reply_to data");

    let session = unsafe { SESSION.as_mut().expect("The session is not initialized") };

    /*
    TODO: Process and store the result depending on the reply
        If a `GameStarted` response is received, it updates the game status to indicate that the game was successfully started.
        If a `WordChecked` response is received, it saves the response, increments the number of tries, and checks if the word was guessed.
        If the word has been guessed, it switches the game status to `GameOver(Win)`.
        If all attempts are used up and the word is not guessed, it switches the game status to `GameOver(Lose)`.

    TODO: Call `wake()` to acknowledge the response
    */
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
