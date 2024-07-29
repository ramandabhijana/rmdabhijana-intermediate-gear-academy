mod utils;

use gstd::MessageId;
use gtest::Log;
use session::consts::{err_msgs::PROGRAM_ONLY, game_rules::DELAY_CHECK_STATUS_DURATION};
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
fn check_game_status_ignore_when_completed() {}

#[test]
fn check_game_status_ignore_when_init_id_changed() {}

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
