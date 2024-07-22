use gstd::ActorId;
use gtest::{Log, Program, ProgramBuilder, System};
use session_io::{Action, Event, GameStatus, State};

const PROXY_PROGRAM: u64 = 1;
const TARGET_PROGRAM: u64 = 2;

const USER: u64 = 3;

#[test]
fn start_game_should_success() {
    let system = System::new();
    system.init_logger();

    let proxy_program = Program::current(&system);
    let target_program =
        ProgramBuilder::from_file("../target/wasm32-unknown-unknown/debug/wordle.opt.wasm")
            .with_id(TARGET_PROGRAM)
            .build(&system);

    let result = target_program.send_bytes(USER, []);
    assert!(!result.main_failed());

    let result = proxy_program.send::<_, ActorId>(USER, TARGET_PROGRAM.into());
    assert!(!result.main_failed());

    // TODO: refactor init stuffs above

    let result = proxy_program.send(USER, Action::StartGame);

    assert!(!result.main_failed());

    let log = Log::builder()
        .source(PROXY_PROGRAM)
        .dest(USER)
        // .payload(Event::GameStarted { user: USER.into() });
        .payload(Event::MessageSent);
    assert!(result.contains(&log));

    let State { players, .. } = proxy_program.read_state(0).unwrap();
    assert!(players.get(&USER.into()).is_some());

    let info = players.get(&USER.into()).unwrap();
    assert_eq!(info.game_status, GameStatus::InProgress);
    assert_eq!(info.attempts_count, 0);
}
