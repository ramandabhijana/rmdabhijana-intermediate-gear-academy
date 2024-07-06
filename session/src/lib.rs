#![no_std]
use gstd::{msg, prelude::*, ActorId};
use session_io::*;

static mut SESSION: Option<Session> = None;

struct Session {
    target_program_id: ActorId,
}

impl Session {
    pub fn new(target_program_id: ActorId) -> Self {
        Self { target_program_id }
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
    unsafe { SESSION = Some(Session { target_program_id }) }
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
