#![no_std]
use gstd::{msg, prelude::*, ActorId};

static mut SESSION: Option<Session> = None;

struct Session {
    target_program_id: ActorId,
}

#[no_mangle]
extern "C" fn init() {
    let target_program_id = msg::load().expect("Could not decode target program ID");
    unsafe { SESSION = Some(Session { target_program_id }) }
}
