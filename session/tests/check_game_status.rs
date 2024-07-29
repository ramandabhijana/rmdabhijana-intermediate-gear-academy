mod utils;

use gstd::MessageId;
use gtest::{Log, Program};
use session::consts::{
    err_msgs::PROGRAM_ONLY,
    game_rules::{DELAY_CHECK_STATUS_DURATION, MAX_ATTEMPTS},
};
use session_io::{Action, Event, GameOverStatus, GameStatus, State};
use utils::*;

#[test]
fn check_game_status_should_fail_when_called_by_other_actor() {
    let system = init_system();
    let proxy_program = init_programs(&system).proxy_program;

    // When: A user send action to check game status
    let result = proxy_program.send(
        USER,
        Action::CheckGameStatus {
            user: USER.into(),
            init_id: MessageId::zero(),
        },
    );

    // Then: Program reverts with appropriate error message
    let log = Log::builder()
        .source(PROXY_PROGRAM)
        .dest(USER)
        .payload_bytes(&final_panic_message(PROGRAM_ONLY));
    assert!(result.main_failed() && result.contains(&log));
}

#[test]
fn check_game_status_ignore_when_completed() {
    let system = init_system();
    let proxy_program = init_programs(&system).proxy_program;

    // Given: maximum number of attempts is reached
    proxy_program.send(USER, Action::StartGame);
    consume_all_attempts_with_wrong_answers(&proxy_program);
    let State { players, .. } = proxy_program.read_state(0).unwrap();
    let info = players.get(&USER.into()).unwrap();
    assert_eq!(
        info.game_status,
        GameStatus::Completed(GameOverStatus::Lose)
    );
    assert_eq!(info.attempts_count, MAX_ATTEMPTS);

    // When: check status period has come
    let result = system.spend_blocks(DELAY_CHECK_STATUS_DURATION);

    // Then: Result logs are empty
    assert!(result.first().unwrap().log().is_empty());
}

#[test]
fn check_game_status_ignore_when_init_id_changed() {
    let system = init_system();
    let proxy_program = init_programs(&system).proxy_program;

    // Given:
    // - Maximum number of attempts is reached
    // - User restarts the game at block `DELAY_CHECK_STATUS_DURATION` - 1
    proxy_program.send(USER, Action::StartGame);
    consume_all_attempts_with_wrong_answers(&proxy_program);
    let State { players, .. } = proxy_program.read_state(0).unwrap();
    let info = players.get(&USER.into()).unwrap();
    assert_eq!(info.attempts_count, MAX_ATTEMPTS);

    let prev_init_id = info.init_msg_id;

    system.spend_blocks(DELAY_CHECK_STATUS_DURATION - 1);
    proxy_program.send(USER, Action::StartGame);
    let State { players, .. } = proxy_program.read_state(0).unwrap();
    let info = players.get(&USER.into()).unwrap();
    assert_eq!(info.game_status, GameStatus::InProgress);

    // When: check status period from previous session has come
    let result = system.spend_blocks(1);

    // Then: Result logs are empty and init ID's mismatch
    assert!(result.first().unwrap().log().is_empty());
    assert_ne!(prev_init_id, info.init_msg_id);
}

#[test]
fn check_game_status_should_declare_game_over_when_time_is_up() {
    let system = init_system();
    let proxy_program = init_programs(&system).proxy_program;

    // Given: A game is in progress
    proxy_program.send(USER, Action::StartGame);

    // When: Time is up
    let result = system.spend_blocks(DELAY_CHECK_STATUS_DURATION);

    // Then:
    // - GameOver event is emitted
    // - The game status is set accordingly
    let log = Log::builder()
        .source(PROXY_PROGRAM)
        .dest(USER)
        .payload(Event::GameOver(GameOverStatus::Lose));
    assert!(result.first().unwrap().contains(&log));

    let State { players, .. } = proxy_program.read_state(0).unwrap();
    let info = players.get(&USER.into()).unwrap();
    assert_eq!(
        info.game_status,
        GameStatus::Completed(GameOverStatus::Lose)
    );
}

fn consume_all_attempts_with_wrong_answers(program: &Program) {
    for _ in 0..MAX_ATTEMPTS {
        program.send(
            USER,
            Action::CheckWord {
                word: "human".into(),
            },
        );
    }
}
