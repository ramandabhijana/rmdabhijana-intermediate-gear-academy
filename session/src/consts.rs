pub mod err_msgs {
    pub const GAME_IS_PLAYING: &str = "A game is in progress for this user";
    pub const SEND_FAILED: &str = "Error in sending message";
    pub const SEND_DELAYED_FAILED: &str = "Error in sending delayed message";
    pub const GAME_NOT_FOUND: &str = "Game does not exist for the user";
    pub const GAME_NOT_PLAYABLE: &str = "Game is not available to play";
    pub const INVALID_WORD_LEN: &str = "Word must be 5 character long";
    pub const INVALID_WORD_CASE: &str = "Word must be lowercased";
    pub const LOAD_FAILED: &str = "Unable to message's payload";
    pub const PLAYER_INFO_NOT_FOUND: &str = "Player info does not exist";
    pub const PROGRAM_ONLY: &str = "Callable by current program only";
    pub const RESUME_FAILED: &str = "Error in resuming paused message";
    pub const READ_REPLY_FAILED: &str = "Error in reading replied Message ID";
}

pub mod game_rules {
    pub const DELAY_CHECK_STATUS_DURATION: u32 = 200;
    pub const MAX_ATTEMPTS: u32 = 5;
}
